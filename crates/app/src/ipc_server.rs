//! Named-pipe server for `pecofence-cli`: a listener thread accepts connections, a small pool
//! of connection threads reads one request each, hands it to the UI thread through
//! [`PendingQueue`] + `WM_APP_COMMAND` (the same worker→UI pattern as `fileops.rs`) and writes
//! the reply back. Nothing here touches app state; see `app/ipc.rs` for the UI-thread half.
//!
//! Every blocking pipe operation runs under an [`IoDeadline`]: a client that connects and
//! never sends, or never reads its reply, is cut off after a few seconds instead of holding a
//! thread (and one of the [`MAX_CONNECTIONS`] slots) for ever.
//!
//! The release profile aborts on panic, so no I/O result is ever unwrapped on these threads.

use crate::commands::WM_APP_COMMAND;
use pecofence_ipc::{ErrorCode, IpcError, MAX_REQUEST_BYTES, PROTOCOL_VERSION, Request, Response};
use pecofence_platform::HWND;
use pecofence_platform::pipe::{self, PipeListener, ThreadHandle};
use pecofence_platform::window;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::windows::io::AsRawHandle;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Connections served at the same time; anything beyond is told `busy` at once.
const MAX_CONNECTIONS: usize = 8;
/// `busy` replies in flight at the same time; further connections are dropped unanswered so a
/// flood cannot grow the thread count without bound.
const MAX_BUSY_REPLIES: usize = 8;
/// Grace on top of the client's own timeout before a connection thread gives up waiting for
/// the UI thread (the client has left by then).
const REPLY_GRACE: Duration = Duration::from_secs(5);
/// A client must deliver its request line within this long after connecting.
const READ_DEADLINE: Duration = Duration::from_secs(10);
/// A client must have read its reply within this long after it was written.
const WRITE_DEADLINE: Duration = Duration::from_secs(5);
/// Pause after a failed pipe-instance creation before the listener tries again.
const CREATE_BACKOFF: Duration = Duration::from_millis(100);

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
    /// Stops the listener and waits for it. The thread may be blocked in `ConnectNamedPipe`:
    /// a self-connection releases it when an instance is waiting, cancelling its I/O releases
    /// it when none is (creation failed). Connection threads are detached and bounded by their
    /// deadlines; they are not waited for.
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.listener.take() {
            pipe::connect_self(&self.name);
            ThreadHandle::cancel_io_on_raw(t.as_raw_handle());
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
    let busy = Arc::new(AtomicUsize::new(0));
    while !stop.load(Ordering::SeqCst) {
        let connection = match listener.accept() {
            Ok(c) => c,
            Err(error) => {
                if stop.load(Ordering::SeqCst) {
                    break;
                }
                tracing::warn!(target: "pecofence::ipc", %error, "accept failed");
                if !listener.is_listening() {
                    // No instance could be created: wait before trying again, but stay
                    // responsive to the stop flag.
                    std::thread::sleep(CREATE_BACKOFF);
                }
                continue;
            }
        };
        if stop.load(Ordering::SeqCst) {
            break;
        }
        if active.load(Ordering::SeqCst) >= MAX_CONNECTIONS {
            if busy.load(Ordering::SeqCst) >= MAX_BUSY_REPLIES {
                tracing::warn!(target: "pecofence::ipc", "connection flood; dropped unanswered");
                continue;
            }
            busy.fetch_add(1, Ordering::SeqCst);
            let counter = busy.clone();
            let spawned = std::thread::Builder::new()
                .name("pecofence-ipc-busy".into())
                .spawn(move || {
                    serve_busy(connection);
                    counter.fetch_sub(1, Ordering::SeqCst);
                });
            if let Err(error) = spawned {
                busy.fetch_sub(1, Ordering::SeqCst);
                tracing::warn!(target: "pecofence::ipc", %error, "busy-reply thread failed to start");
            }
            continue;
        }
        active.fetch_add(1, Ordering::SeqCst);
        let counter = active.clone();
        let pending = pending.clone();
        let spawned = std::thread::Builder::new()
            .name("pecofence-ipc-conn".into())
            .spawn(move || serve(connection, control, &pending, &counter));
        if let Err(error) = spawned {
            active.fetch_sub(1, Ordering::SeqCst);
            tracing::warn!(target: "pecofence::ipc", %error, "connection thread failed to start");
        }
    }
    tracing::debug!(target: "pecofence::ipc", "listener stopped");
}

/// Cancels the calling thread's blocking pipe I/O when it is still running after `after`.
/// Disarmed (and the pending cancel suppressed) on drop.
struct IoDeadline {
    armed: Arc<Mutex<bool>>,
}

impl IoDeadline {
    fn arm(thread: &Option<Arc<ThreadHandle>>, after: Duration) -> Self {
        let armed = Arc::new(Mutex::new(true));
        if let Some(thread) = thread {
            let flag = armed.clone();
            let thread = thread.clone();
            let spawned = std::thread::Builder::new()
                .name("pecofence-ipc-deadline".into())
                .spawn(move || {
                    std::thread::sleep(after);
                    // Holding the lock across the cancel means a disarm cannot slip in between
                    // the check and the call, so the cancel never hits a later operation.
                    if let Ok(still_armed) = flag.lock()
                        && *still_armed
                    {
                        thread.cancel_io();
                        tracing::debug!(target: "pecofence::ipc", after_ms = after.as_millis() as u64, "connection I/O deadline hit");
                    }
                });
            if let Err(error) = spawned {
                tracing::warn!(target: "pecofence::ipc", %error, "deadline thread failed to start; connection runs unbounded");
            }
        }
        Self { armed }
    }
}

impl Drop for IoDeadline {
    fn drop(&mut self) {
        if let Ok(mut armed) = self.armed.lock() {
            *armed = false;
        }
    }
}

/// One connection: read a line, validate, wait for the UI thread, reply. A stream method
/// (`events.subscribe`) then keeps the connection: every further `Response` the UI thread
/// sends down the same channel is written as one line until the client closes the pipe (the
/// write fails) or the UI thread drops its sender. `active` (the connection-slot counter) is
/// released as soon as the request is answered, so an open stream does not occupy one of the
/// [`MAX_CONNECTIONS`] slots (the UI thread caps streams separately).
fn serve(connection: File, control: isize, pending: &PendingQueue, active: &AtomicUsize) {
    let started = Instant::now();
    let thread = ThreadHandle::current().map(Arc::new);
    if thread.is_none() {
        tracing::warn!(target: "pecofence::ipc", "no thread handle; connection deadlines disabled");
    }
    let read = {
        let _deadline = IoDeadline::arm(&thread, READ_DEADLINE);
        read_request(&connection)
    };
    let (method, response, flush, stream) = match read {
        Err(ReadError::PeerGone) => {
            tracing::debug!(target: "pecofence::ipc", ms = started.elapsed().as_secs_f32() * 1000.0, "client left before sending a request");
            active.fetch_sub(1, Ordering::SeqCst);
            return;
        }
        Err(ReadError::TimedOut) => {
            tracing::info!(target: "pecofence::ipc", "no request within the deadline; connection dropped");
            active.fetch_sub(1, Ordering::SeqCst);
            return;
        }
        // A malformed request came from a live client: answer, but do not wait for it to read.
        Err(ReadError::Rejected(error)) => (String::from("-"), Response::err(error), false, None),
        Ok(request) => {
            let method = request.method.name();
            let is_stream = request.method.is_stream();
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
            let stream = (is_stream && response.ok).then_some(rx);
            (method, response, stream.is_none(), stream)
        }
    };
    let written = {
        let _deadline = IoDeadline::arm(&thread, WRITE_DEADLINE);
        write_reply(&connection, &response, flush)
    };
    tracing::info!(
        target: "pecofence::ipc",
        method,
        ok = response.ok,
        ms = started.elapsed().as_secs_f32() * 1000.0,
        "request"
    );
    active.fetch_sub(1, Ordering::SeqCst);
    let Some(rx) = stream else {
        return;
    };
    if !written {
        return;
    }
    let mut lines = 0u64;
    while let Ok(event) = rx.recv() {
        let _deadline = IoDeadline::arm(&thread, WRITE_DEADLINE);
        if !write_reply(&connection, &event, false) {
            break;
        }
        lines += 1;
    }
    // Either the client left (write failed; the UI thread notices at its next send) or the app
    // is shutting down (sender dropped). Flush what the client may still be reading.
    pipe::finish(&connection);
    tracing::info!(
        target: "pecofence::ipc",
        lines,
        secs = started.elapsed().as_secs_f32(),
        "event stream ended"
    );
}

/// Over the connection limit: take the request line (so the client's own write has completed
/// and it is reading), answer `busy`, close. Both steps are bounded; nothing is flushed.
fn serve_busy(connection: File) {
    let thread = ThreadHandle::current().map(Arc::new);
    {
        let _deadline = IoDeadline::arm(&thread, READ_DEADLINE);
        let mut sink = Vec::new();
        let mut reader = BufReader::new((&connection).take(MAX_REQUEST_BYTES));
        if reader.read_until(b'\n', &mut sink).is_err() {
            return;
        }
    }
    let error = IpcError::new(
        ErrorCode::Busy,
        format!("{MAX_CONNECTIONS} CLI connections are already being served"),
    )
    .hint("Retry in a moment");
    let _deadline = IoDeadline::arm(&thread, WRITE_DEADLINE);
    write_reply(&connection, &Response::err(error), false);
}

/// Why no request could be read from a connection.
#[derive(Debug)]
enum ReadError {
    /// The client closed (or never wrote and vanished): nothing to answer.
    PeerGone,
    /// The read deadline cancelled the wait.
    TimedOut,
    /// Bytes arrived but were not a valid request: this is the reply.
    Rejected(IpcError),
}

impl From<IpcError> for ReadError {
    fn from(e: IpcError) -> Self {
        ReadError::Rejected(e)
    }
}

/// Reads and validates one request line from `connection` (at most [`MAX_REQUEST_BYTES`]).
fn read_request(connection: impl Read) -> Result<Request, ReadError> {
    let mut line = String::new();
    let mut reader = BufReader::new(connection.take(MAX_REQUEST_BYTES));
    match reader.read_line(&mut line) {
        Ok(0) => return Err(ReadError::PeerGone),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
            return Err(usage("request is not valid UTF-8").into());
        }
        Err(error) if pipe::is_cancelled(&error) => return Err(ReadError::TimedOut),
        Err(error) if pipe::is_peer_gone(&error) => return Err(ReadError::PeerGone),
        Err(error) => return Err(usage(format!("could not read the request: {error}")).into()),
    }
    if !line.ends_with('\n') && line.len() as u64 >= MAX_REQUEST_BYTES {
        return Err(usage(format!("request exceeds {MAX_REQUEST_BYTES} bytes")).into());
    }
    let text = line.trim();
    if text.is_empty() {
        return Err(usage("empty request").into());
    }
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| usage(format!("request is not valid JSON: {e}")))?;
    if !value.is_object() {
        return Err(usage("request must be a JSON object").into());
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
            ))
            .into());
        }
        None => return Err(usage("missing \"protocol\" field").into()),
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
            _ => Err(usage(format!("invalid request: {first}")).into()),
        },
    }
}

fn usage(message: impl Into<String>) -> IpcError {
    IpcError::usage(message).hint(
        "Send one JSON object per line, e.g. {\"protocol\":1,\"method\":\"status.get\"}; run `pecofence-cli describe` for the method list",
    )
}

/// Writes the reply line; `true` when the bytes went out. With `flush`, waits until the client
/// has read it before the instance is disconnected (bounded by the caller's [`IoDeadline`]);
/// without, the bytes stay readable until the client closes or the connection is dropped. Write
/// errors are not reported to anyone (`ERROR_NO_DATA`: the client already left;
/// `ERROR_OPERATION_ABORTED`: the deadline).
fn write_reply(connection: &File, response: &Response, flush: bool) -> bool {
    let mut bytes = match serde_json::to_vec(response) {
        Ok(b) => b,
        Err(error) => {
            tracing::warn!(target: "pecofence::ipc", %error, "reply could not be serialized");
            return false;
        }
    };
    bytes.push(b'\n');
    let mut writer = connection;
    if let Err(error) = writer.write_all(&bytes) {
        if !pipe::is_peer_gone(&error) && !pipe::is_cancelled(&error) {
            tracing::debug!(target: "pecofence::ipc", %error, "reply write failed");
        }
        return false;
    }
    if flush {
        pipe::finish(connection);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use pecofence_ipc::Method;

    fn read(text: &str) -> Result<Request, ReadError> {
        read_request(text.as_bytes())
    }

    fn rejected(text: &str) -> IpcError {
        match read(text) {
            Err(ReadError::Rejected(e)) => e,
            other => panic!("expected a rejection for {text:?}, got {other:?}"),
        }
    }

    #[test]
    fn valid_requests_parse() {
        let req = read("{\"protocol\":1,\"method\":\"status.get\"}\n").unwrap();
        assert_eq!(req.method, Method::StatusGet);
        assert_eq!(req.timeout_ms, pecofence_ipc::DEFAULT_TIMEOUT_MS);
        // A missing trailing newline is fine when the client closes the pipe.
        let req = read("{\"protocol\":1,\"timeoutMs\":7,\"method\":\"peek.end\"}").unwrap();
        assert_eq!(req.method, Method::PeekEnd);
        assert_eq!(req.timeout_ms, 7);
    }

    #[test]
    fn missing_params_is_retried_as_an_empty_object() {
        let req = read("{\"protocol\":1,\"method\":\"items.list\"}\n").unwrap();
        assert_eq!(req.method, Method::ItemsList { fence: None });
        let req = read("{\"protocol\":1,\"method\":\"settings.get\"}\n").unwrap();
        assert_eq!(req.method, Method::SettingsGet { path: None });
        // A method that does need a parameter is still refused.
        let err = rejected("{\"protocol\":1,\"method\":\"fences.get\"}\n");
        assert_eq!(err.code, ErrorCode::Usage);
        assert!(
            err.message.starts_with("invalid request"),
            "{}",
            err.message
        );
    }

    #[test]
    fn protocol_mismatch_is_reported_as_such() {
        let err = rejected("{\"protocol\":2,\"method\":\"status.get\"}\n");
        assert_eq!(err.code, ErrorCode::VersionMismatch);
        assert!(err.message.contains("protocol 2"), "{}", err.message);
        assert!(err.hint.is_some());
        let err = rejected("{\"method\":\"status.get\"}\n");
        assert_eq!(err.code, ErrorCode::Usage);
        assert!(err.message.contains("protocol"), "{}", err.message);
    }

    #[test]
    fn garbage_is_usage() {
        for text in ["nonsense\n", "[1,2]\n", "   \n", "{\"protocol\":1}\n"] {
            let err = rejected(text);
            assert_eq!(err.code, ErrorCode::Usage, "{text:?}");
            assert!(err.hint.as_deref().unwrap_or("").contains("status.get"));
        }
        assert!(matches!(read(""), Err(ReadError::PeerGone)));
    }

    #[test]
    fn non_utf8_is_usage() {
        let bytes: &[u8] = b"{\"protocol\":1,\"method\":\"\xff\xfe\"}\n";
        match read_request(bytes) {
            Err(ReadError::Rejected(e)) => {
                assert_eq!(e.code, ErrorCode::Usage);
                assert!(e.message.contains("UTF-8"), "{}", e.message);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn oversized_line_is_usage() {
        let mut text = String::from("{\"protocol\":1,\"method\":\"status.get\",\"pad\":\"");
        text.push_str(&"x".repeat(MAX_REQUEST_BYTES as usize + 16));
        text.push_str("\"}\n");
        let err = rejected(&text);
        assert_eq!(err.code, ErrorCode::Usage);
        assert!(err.message.contains("exceeds"), "{}", err.message);
    }

    #[test]
    fn cancelled_and_broken_reads_are_not_answered() {
        struct Failing(i32);
        impl Read for Failing {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::from_raw_os_error(self.0))
            }
        }
        assert!(matches!(
            read_request(Failing(995)),
            Err(ReadError::TimedOut)
        ));
        assert!(matches!(
            read_request(Failing(109)),
            Err(ReadError::PeerGone)
        ));
    }

    #[test]
    fn deadline_disarms_on_drop() {
        let thread = ThreadHandle::current().map(Arc::new);
        let deadline = IoDeadline::arm(&thread, Duration::from_millis(20));
        let flag = deadline.armed.clone();
        drop(deadline);
        assert!(!*flag.lock().unwrap());
        std::thread::sleep(Duration::from_millis(60));
        // Blocking on a pipe read now must not be interrupted by the expired watchdog.
        assert!(!*flag.lock().unwrap());
    }
}
