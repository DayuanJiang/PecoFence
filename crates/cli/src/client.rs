//! One request over the app's named pipe: connect, write a line, read a line.

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use pecofence_ipc::{ErrorCode, IpcError, Method, Request, Response, ipc_pipe_name};

/// `SECURITY_IDENTIFICATION`: the server may learn who we are but cannot impersonate us. std's
/// `security_qos_flags` ORs `SECURITY_SQOS_PRESENT` in by itself (so that the zero-valued
/// `SECURITY_ANONYMOUS` is representable), which is why only the level is passed here.
const SECURITY_IDENTIFICATION: u32 = 0x0001_0000;

const ERROR_FILE_NOT_FOUND: i32 = 2;
const ERROR_BROKEN_PIPE: i32 = 109;
const ERROR_PIPE_BUSY: i32 = 231;
const ERROR_NO_DATA: i32 = 232;

/// The server keeps at least one listening pipe instance at all times, but a slow machine can
/// still show a gap between two `CreateNamedPipe` calls; a few short retries hide it.
const NOT_FOUND_RETRIES: u32 = 5;
const NOT_FOUND_DELAY: Duration = Duration::from_millis(50);
const BUSY_RETRIES: u32 = 4;
const BUSY_WAIT_MS: u32 = 2000;
/// Extra wait on top of the request's own timeout. The server's own `timeout` reply is not
/// what this covers: it comes only after `timeout_ms` plus a 5 s grace, i.e. after we have
/// long given up. The slack exists for the boundary case where the UI thread picks the request
/// up just before its expiry check (`age > timeout_ms`) would have dropped it: the command then
/// runs, and the reply arrives shortly after `timeout_ms`; without the slack we would report a
/// timeout for a command that did execute.
const TIMEOUT_SLACK: Duration = Duration::from_millis(1500);
/// Shortest wait the CLI accepts; the server clamps `timeout_ms` the same way, so an agent
/// passing `--timeout 0` cannot make every request expire before it runs.
pub const MIN_TIMEOUT_MS: u32 = 100;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn WaitNamedPipeW(name: *const u16, timeout: u32) -> i32;
}

pub const NOT_RUNNING_HINT: &str =
    "Start PecoFence, or pass --instance / set PECOFENCE_INSTANCE to match the running instance";
pub const TIMEOUT_HINT: &str =
    "Close any open PecoFence menu or dialog and retry, or raise --timeout";

/// Sends one request and waits for the reply. `Ok` carries the server's envelope, which may
/// itself be `ok: false`; transport failures are mapped to `not_running` / `timeout` /
/// `internal`.
pub fn send(instance: Option<&str>, method: Method, timeout_ms: u32) -> Result<Response, IpcError> {
    let timeout_ms = timeout_ms.max(MIN_TIMEOUT_MS);
    let name = ipc_pipe_name(instance);
    let request = Request::new(method).with_timeout(timeout_ms);
    let mut line = serde_json::to_string(&request)
        .map_err(|e| IpcError::internal(format!("cannot encode request: {e}")))?;
    line.push('\n');

    // Blocking Win32 I/O has no portable cancellation; the exchange runs on a helper thread and
    // the main thread simply stops waiting. The process exits right after, so the thread is
    // never joined.
    let (tx, rx) = mpsc::channel();
    thread::Builder::new()
        .name("pipe".into())
        .spawn(move || {
            let _ = tx.send(exchange(&name, &line));
        })
        .map_err(|e| IpcError::internal(format!("cannot start I/O thread: {e}")))?;
    match rx.recv_timeout(Duration::from_millis(u64::from(timeout_ms)) + TIMEOUT_SLACK) {
        Ok(result) => result,
        Err(_) => Err(IpcError::new(
            ErrorCode::Timeout,
            format!("no reply from PecoFence within {timeout_ms} ms"),
        )
        .hint(TIMEOUT_HINT)),
    }
}

fn exchange(name: &str, line: &str) -> Result<Response, IpcError> {
    let mut pipe = open(name)?;
    pipe.write_all(line.as_bytes())
        .and_then(|_| pipe.flush())
        .map_err(|e| match e.raw_os_error() {
            Some(ERROR_BROKEN_PIPE) | Some(ERROR_NO_DATA) => closed(),
            _ => IpcError::internal(format!("cannot write to {name}: {e}")),
        })?;
    let mut reader = BufReader::new(pipe);
    let mut reply = String::new();
    match reader.read_line(&mut reply) {
        Ok(0) => return Err(closed()),
        Ok(_) => {}
        Err(e) if e.raw_os_error() == Some(ERROR_BROKEN_PIPE) => return Err(closed()),
        Err(e) => return Err(IpcError::internal(format!("cannot read from {name}: {e}"))),
    }
    serde_json::from_str::<Response>(reply.trim_end()).map_err(|e| {
        IpcError::internal(format!("malformed reply from PecoFence: {e}"))
            .hint("The app and the CLI may be different versions; reinstall or update both")
    })
}

fn closed() -> IpcError {
    IpcError::internal("connection closed by PecoFence")
        .hint("Check the app's log; if it crashed, start it again")
}

fn open(name: &str) -> Result<std::fs::File, IpcError> {
    let mut not_found = 0;
    let mut busy = 0;
    loop {
        let attempt = OpenOptions::new()
            .read(true)
            .write(true)
            .security_qos_flags(SECURITY_IDENTIFICATION)
            .open(name);
        let err = match attempt {
            Ok(file) => return Ok(file),
            Err(e) => e,
        };
        match err.raw_os_error() {
            Some(ERROR_FILE_NOT_FOUND) if not_found < NOT_FOUND_RETRIES => {
                not_found += 1;
                thread::sleep(NOT_FOUND_DELAY);
            }
            Some(ERROR_FILE_NOT_FOUND) => {
                return Err(IpcError::new(
                    ErrorCode::NotRunning,
                    format!("PecoFence is not running (no pipe {name})"),
                )
                .hint(NOT_RUNNING_HINT));
            }
            Some(ERROR_PIPE_BUSY) if busy < BUSY_RETRIES => {
                busy += 1;
                let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
                // Returns 0 on timeout; we retry the open either way and let it fail properly.
                let _ = unsafe { WaitNamedPipeW(wide.as_ptr(), BUSY_WAIT_MS) };
            }
            Some(ERROR_PIPE_BUSY) => {
                return Err(IpcError::new(
                    ErrorCode::Busy,
                    "every PecoFence pipe instance is busy",
                )
                .hint("Retry in a moment; run fewer CLI calls in parallel"));
            }
            _ => {
                return Err(IpcError::internal(format!("cannot open {name}: {err}")));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_instance_is_not_running() {
        let err = send(
            Some("pecofence-cli-unit-test-nosuch"),
            Method::StatusGet,
            500,
        )
        .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotRunning);
        assert_eq!(err.code.exit_code(), 3);
        assert_eq!(err.hint.as_deref(), Some(NOT_RUNNING_HINT));
    }

    #[test]
    fn zero_timeout_is_clamped_not_instant() {
        // With a real 0 ms wait the not-found retries alone would outlast the deadline and the
        // error would be `timeout`; the clamp keeps it a clear `not_running`.
        let err = send(Some("pecofence-cli-unit-test-nosuch"), Method::StatusGet, 0).unwrap_err();
        assert_eq!(err.code, ErrorCode::NotRunning);
    }
}
