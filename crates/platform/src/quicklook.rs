//! QuickLook (github.com/QL-Win/QuickLook), the Space-bar previewer for Explorer. Other
//! programs ask it for a preview through its named pipe `\\.\pipe\QuickLook.App.Pipe.<user
//! SID>`, one `QuickLook.App.PipeMessages.<message>|<path>` line per connection, as its wiki
//! tells developers to (QL-Win/QuickLook#557). The pipe exists while QuickLook runs, whichever
//! edition (MSI, ZIP or Store) is installed.

use crate::wide::to_wide;
use std::io::Write;
use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use windows_core::PCWSTR;

windows_core::link!("kernel32.dll" "system" fn WaitNamedPipeW(lpnamedpipename: PCWSTR, ntimeout: u32) -> windows_core::BOOL);

const ERROR_PIPE_BUSY: i32 = 231;

/// Set once a preview was asked for: until then there is no preview of ours to switch or close.
static TOGGLED: AtomicBool = AtomicBool::new(false);

/// Opens QuickLook's preview of `path`, or closes it when it already shows `path`. False when
/// QuickLook is not running.
pub fn toggle(path: &Path) -> bool {
    let sent = send("Toggle", path);
    if sent {
        TOGGLED.store(true, Ordering::Relaxed);
    }
    sent
}

/// Shows `path` in the open preview instead (QuickLook ignores it when no preview is open).
pub fn switch(path: &Path) {
    if TOGGLED.load(Ordering::Relaxed) {
        send("Switch", path);
    }
}

/// Closes the open preview, if any.
pub fn close() {
    if TOGGLED.load(Ordering::Relaxed) {
        send("Close", Path::new(""));
    }
}

fn send(message: &str, path: &Path) -> bool {
    static PIPE: OnceLock<Option<String>> = OnceLock::new();
    let Some(pipe) = PIPE.get_or_init(|| {
        crate::pipe::current_user_sid()
            .ok()
            .map(|sid| format!(r"\\.\pipe\QuickLook.App.Pipe.{sid}"))
    }) else {
        return false;
    };
    let line = format!(
        "QuickLook.App.PipeMessages.{message}|{}\n",
        path.to_string_lossy()
    );
    // Its single pipe instance is busy only while it reads a line.
    for _ in 0..3 {
        match std::fs::OpenOptions::new().write(true).open(pipe) {
            Ok(mut file) => {
                let sent = file.write_all(line.as_bytes()).is_ok();
                tracing::info!(message, sent, "quicklook message");
                return sent;
            }
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                let wide = to_wide(pipe);
                // SAFETY: the string outlives the call.
                unsafe {
                    let _ = WaitNamedPipeW(PCWSTR(wide.as_ptr()), 100);
                }
            }
            Err(e) => {
                tracing::debug!(message, error = %e, "quicklook not running");
                return false;
            }
        }
    }
    false
}
