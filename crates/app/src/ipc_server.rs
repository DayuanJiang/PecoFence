//! Named-pipe server for `pecofence-cli`: a listener thread accepts connections, a small pool
//! of connection threads reads one request each, hands it to the UI thread through
//! [`PendingQueue`] + `WM_APP_COMMAND` (the same worker→UI pattern as `fileops.rs`) and writes
//! the reply back. Nothing here touches app state; see `app/ipc.rs` for the UI-thread half.
//!
//! The release profile aborts on panic, so no I/O result is ever unwrapped on these threads.

use crate::commands::WM_APP_COMMAND;
use pecofence_ipc::{ErrorCode, IpcError, MAX_REQUEST_BYTES, PROTOCOL_VERSION, Request, Response};
use pecofence_platform::HWND;
use pecofence_platform::pipe::{self, PipeListener};
use pecofence_platform::window;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Connections served at the same time; anything beyond is told `busy` at once.
const MAX_CONNECTIONS: usize = 8;
/// Grace on top of the client's own timeout before a connection thread gives up waiting for
/// the UI thread (the client has left by then).
const REPLY_GRACE: Duration = Duration::from_secs(5);

/// A parsed request waiting for the UI thread.
pub struct Pending {
    pub request: Request,
    pub reply: mpsc::Sender<Response>,
    pub arrived: Instant,
}

pub type PendingQueue = Arc<Mutex<VecDeque<Pending>>>;

pub struct IpcServer {
    name: String,
    stop: Arc<AtomicBool>,
    listener: Option<JoinHandle<()>>,
}

impl IpcServer {
    /// Binds the pipe and starts the listener thread. `None` when the name is taken by another
    /// process (logged; the app runs without IPC) or the thread could not be started.
    pub fn start(pipe_name: &str, control: HWND, pending: PendingQueue) -> Option<Self> {
        let listener = match PipeListener::bind(pipe_name) {
            Ok(l) => l,
            Err(error) => {
                tracing::warn!(target: "pecofence::ipc", pipe = pipe_name, %error, "pipe name unavailable; CLI disabled");
                return None;
            }
        };
        let stop = Arc::new(AtomicBool::new(false));
        let control = control.0 as isize;
        let thread = {
            let stop = stop.clone();
            std::thread::Builder::new()
                .name("pecofence-ipc".into())
                .spawn(move || listen_loop(listener, stop, control, pending))
        };
        match thread {
            Ok(handle) => {
                tracing::info!(target: "pecofence::ipc", pipe = pipe_name, "listening");
                Some(Self {
                    name: pipe_name.to_string(),
                    stop,
                    listener: Some(handle),
                })
            }
            Err(error) => {
                tracing::warn!(target: "pecofence::ipc", %error, "listener thread failed to start");
                None
            }
        }
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        pipe::connect_self(&self.name);
        if let Some(t) = self.listener.take() {
            let _ = t.join();
        }
    }
}

fn listen_loop(
    mut listener: PipeListener,
    stop: Arc<AtomicBool>,
    control: isize,
    pending: PendingQueue,
) {
    let active = Arc::new(AtomicUsize::new(0));
    while !stop.load(Ordering::SeqCst) {
        let connection = match listener.accept() {
            Ok(c) => c,
            Err(error) => {
                if stop.load(Ordering::SeqCst) {
                    break;
                }
                tracing::warn!(target: "pecofence::ipc", %error, "accept failed");
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
        };
        if stop.load(Ordering::SeqCst) {
            break;
        }
        if active.load(Ordering::SeqCst) >= MAX_CONNECTIONS {
            let spawned = std::thread::Builder::new()
                .name("pecofence-ipc-busy".into())
                .spawn(move || {
                    let error = IpcError::new(
                        ErrorCode::Busy,
                        format!("{MAX_CONNECTIONS} CLI connections are already being served"),
                    )
                    .hint("Retry in a moment");
                    write_reply(&connection, &Response::err(error));
                });
            if let Err(error) = spawned {
                tracing::warn!(target: "pecofence::ipc", %error, "busy-reply thread failed to start");
            }
            continue;
        }
        active.fetch_add(1, Ordering::SeqCst);
        let counter = active.clone();
        let pending = pending.clone();
        let spawned = std::thread::Builder::new()
            .name("pecofence-ipc-conn".into())
            .spawn(move || {
                serve(connection, control, &pending);
                counter.fetch_sub(1, Ordering::SeqCst);
            });
        if let Err(error) = spawned {
            active.fetch_sub(1, Ordering::SeqCst);
            tracing::warn!(target: "pecofence::ipc", %error, "connection thread failed to start");
        }
    }
    tracing::debug!(target: "pecofence::ipc", "listener stopped");
}

/// One connection: read a line, validate, wait for the UI thread, reply.
fn serve(connection: File, control: isize, pending: &PendingQueue) {
    let started = Instant::now();
    let (method, response) = match read_request(&connection) {
        Err(error) => (String::from("-"), Response::err(error)),
        Ok(request) => {
            let method = request.method.name();
            let timeout = Duration::from_millis(u64::from(request.timeout_ms)) + REPLY_GRACE;
            let (tx, rx) = mpsc::channel();
            if let Ok(mut q) = pending.lock() {
                q.push_back(Pending {
                    request,
                    reply: tx,
                    arrived: Instant::now(),
                });
            }
            window::post_message(
                HWND(control as *mut core::ffi::c_void),
                WM_APP_COMMAND,
                0,
                0,
            );
            let response = match rx.recv_timeout(timeout) {
                Ok(r) => r,
                Err(_) => Response::err(
                    IpcError::new(ErrorCode::Timeout, "PecoFence did not answer in time")
                        .hint("Close any open PecoFence menu or dialog and retry"),
                ),
            };
            (method, response)
        }
    };
    write_reply(&connection, &response);
    tracing::info!(
        target: "pecofence::ipc",
        method,
        ok = response.ok,
        ms = started.elapsed().as_secs_f32() * 1000.0,
        "request"
    );
}

fn read_request(connection: &File) -> Result<Request, IpcError> {
    let mut line = String::new();
    let mut reader = BufReader::new(connection.take(MAX_REQUEST_BYTES));
    match reader.read_line(&mut line) {
        Ok(0) => return Err(usage("empty request")),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
            return Err(usage("request is not valid UTF-8"));
        }
        Err(error) => return Err(usage(format!("could not read the request: {error}"))),
    }
    if !line.ends_with('\n') && line.len() as u64 >= MAX_REQUEST_BYTES {
        return Err(usage(format!("request exceeds {MAX_REQUEST_BYTES} bytes")));
    }
    let text = line.trim();
    if text.is_empty() {
        return Err(usage("empty request"));
    }
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| usage(format!("request is not valid JSON: {e}")))?;
    if !value.is_object() {
        return Err(usage("request must be a JSON object"));
    }
    match value.get("protocol").and_then(serde_json::Value::as_u64) {
        Some(v) if v == u64::from(PROTOCOL_VERSION) => {}
        Some(v) => {
            return Err(IpcError::new(
                ErrorCode::VersionMismatch,
                format!(
                    "the CLI speaks protocol {v}, this PecoFence {} speaks protocol {PROTOCOL_VERSION}",
                    env!("CARGO_PKG_VERSION")
                ),
            )
            .hint(format!(
                "Update pecofence-cli to match PecoFence {}, or restart PecoFence",
                env!("CARGO_PKG_VERSION")
            )));
        }
        None => return Err(usage("missing \"protocol\" field")),
    }
    // Struct-variant methods whose parameters are all optional (`items.list`, `settings.get`)
    // still need a `params` object for serde; accept its absence as `{}`.
    let retry = match value.get("params") {
        None => {
            let mut with_params = value.clone();
            if let Some(map) = with_params.as_object_mut() {
                map.insert(
                    "params".into(),
                    serde_json::Value::Object(Default::default()),
                );
            }
            Some(with_params)
        }
        Some(_) => None,
    };
    match serde_json::from_value::<Request>(value) {
        Ok(request) => Ok(request),
        Err(first) => match retry.map(serde_json::from_value::<Request>) {
            Some(Ok(request)) => Ok(request),
            _ => Err(usage(format!("invalid request: {first}"))),
        },
    }
}

fn usage(message: impl Into<String>) -> IpcError {
    IpcError::usage(message).hint(
        "Send one JSON object per line, e.g. {\"protocol\":1,\"method\":\"status.get\"}; run `pecofence-cli describe` for the method list",
    )
}

/// Writes the reply line and lets the client read it before the instance is disconnected.
/// Write errors are ignored (`ERROR_NO_DATA`: the client already left).
fn write_reply(connection: &File, response: &Response) {
    let mut bytes = match serde_json::to_vec(response) {
        Ok(b) => b,
        Err(error) => {
            tracing::warn!(target: "pecofence::ipc", %error, "reply could not be serialized");
            return;
        }
    };
    bytes.push(b'\n');
    let mut writer = connection;
    if let Err(error) = writer.write_all(&bytes)
        && !pipe::is_peer_gone(&error)
    {
        tracing::debug!(target: "pecofence::ipc", %error, "reply write failed");
    }
    pipe::finish(connection);
}
