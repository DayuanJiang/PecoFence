//! Seer (1218.io), the Space-bar previewer for Explorer. Other programs ask it for a preview with
//! `WM_COPYDATA` messages to its `SeerWindowClass` window, as its developer guide tells them to
//! (1218.io/docs/seer/app-compatibility.html, "Custom App Integration"): `SEER_INVOKE_W32` with
//! the file's path shows it, `SEER_IS_VISIBLE` asks whether the preview is open, `SEER_HIDE`
//! closes it. Two previews must be asked for at least 200 ms apart, so the messages go out from
//! one background thread, which waits out the gap and keeps only the newest of a burst of arrow
//! keys; the UI thread never waits for Seer.

use crate::bindings::*;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const WM_COPYDATA: u32 = 0x004A;
const SEER_INVOKE_W32: usize = 5000;
const SEER_IS_VISIBLE: usize = 5004;
const SEER_HIDE: usize = 5005;
/// Seer's minimum interval between two `SEER_INVOKE_*` messages.
const INVOKE_GAP: Duration = Duration::from_millis(200);

/// `COPYDATASTRUCT` (winuser.h).
#[repr(C)]
struct CopyData {
    data: usize,
    len: u32,
    ptr: *const core::ffi::c_void,
}

enum Request {
    Toggle(PathBuf),
    Switch(PathBuf),
    Close,
}

/// Set once a preview was asked for: until then there is no preview of ours to switch or close.
static TOGGLED: AtomicBool = AtomicBool::new(false);

/// Seer's window, while Seer runs.
fn window() -> Option<HWND> {
    crate::desktop::windows_of_class("SeerWindowClass")
        .first()
        .copied()
}

/// Opens Seer's preview of `path`, or closes the open one. False when Seer is not running.
pub fn toggle(path: &Path) -> bool {
    if window().is_none() {
        tracing::debug!("seer not running");
        return false;
    }
    TOGGLED.store(true, Ordering::Relaxed);
    request(Request::Toggle(path.to_path_buf()));
    true
}

/// Shows `path` in the open preview instead (nothing when no preview is open).
pub fn switch(path: &Path) {
    if TOGGLED.load(Ordering::Relaxed) {
        request(Request::Switch(path.to_path_buf()));
    }
}

/// Closes the open preview, if any.
pub fn close() {
    if TOGGLED.load(Ordering::Relaxed) {
        request(Request::Close);
    }
}

fn request(req: Request) {
    static TX: OnceLock<Option<Mutex<Sender<Request>>>> = OnceLock::new();
    let tx = TX.get_or_init(|| {
        let (tx, rx) = channel();
        std::thread::Builder::new()
            .name("pecofence-seer".into())
            .spawn(move || worker(rx))
            .ok()
            .map(|_| Mutex::new(tx))
    });
    if let Some(tx) = tx {
        let _ = tx.lock().unwrap_or_else(|e| e.into_inner()).send(req);
    }
}

fn worker(rx: Receiver<Request>) {
    let mut last_invoke: Option<Instant> = None;
    let mut next: Option<Request> = None;
    loop {
        let req = match next.take() {
            Some(r) => r,
            None => match rx.recv() {
                Ok(r) => r,
                Err(_) => return,
            },
        };
        let Some(hwnd) = window() else {
            tracing::debug!("seer not running");
            continue;
        };
        let visible = send(hwnd, SEER_IS_VISIBLE, None) == Some(1);
        let path = match req {
            Request::Close | Request::Toggle(_) if visible => {
                let sent = send(hwnd, SEER_HIDE, None).is_some();
                tracing::info!(sent, "seer preview closed");
                continue;
            }
            Request::Close => continue,
            Request::Switch(_) if !visible => continue,
            Request::Toggle(p) | Request::Switch(p) => p,
        };
        // Wait out the gap since the last preview; a newer request meanwhile (the next arrow
        // key) replaces this one.
        let wait = last_invoke.map_or(Duration::ZERO, |t| INVOKE_GAP.saturating_sub(t.elapsed()));
        if !wait.is_zero() {
            match rx.recv_timeout(wait) {
                Ok(newer) => {
                    next = Some(newer);
                    continue;
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        let sent = send(hwnd, SEER_INVOKE_W32, Some(&path)).is_some();
        last_invoke = Some(Instant::now());
        tracing::info!(sent, switch = visible, "seer preview asked for");
    }
}

/// Sends `command` to Seer's window, with `path` as its NUL-terminated UTF-16 payload; Seer's
/// answer, or None when it did not answer within a second.
fn send(hwnd: HWND, command: usize, path: Option<&Path>) -> Option<usize> {
    let wide: Vec<u16> = path
        .map(|p| p.as_os_str().encode_wide().chain([0]).collect())
        .unwrap_or_default();
    let cds = CopyData {
        data: command,
        len: (wide.len() * 2) as u32,
        ptr: if wide.is_empty() {
            core::ptr::null()
        } else {
            wide.as_ptr().cast()
        },
    };
    let mut result: usize = 0;
    // SAFETY: `cds` and the string it points to outlive the call; the system copies them into
    // Seer's process for WM_COPYDATA.
    let ok = unsafe {
        SendMessageTimeoutW(
            hwnd,
            WM_COPYDATA,
            WPARAM(0),
            LPARAM(&cds as *const CopyData as isize),
            SMTO_ABORTIFHUNG as u32,
            1000,
            Some(&mut result),
        )
    };
    (ok.0 != 0).then_some(result)
}
