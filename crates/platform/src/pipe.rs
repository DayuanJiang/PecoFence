//! Named-pipe server side of the CLI IPC (`\\.\pipe\PecoFence[.<instance>]`).
//!
//! One pipe instance serves one connection. [`PipeListener`] keeps exactly one instance waiting
//! in `ConnectNamedPipe` at all times: the next instance is created *before* a connected one is
//! handed out, so a client never sees `ERROR_FILE_NOT_FOUND` between two requests and mistakes
//! a busy server for a stopped one.

use crate::bindings::*;
use crate::wide::to_wide;
use std::fs::File;
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use windows_core::{Error, PCWSTR, Result};

/// In/out buffer size of every instance.
const BUFFER_BYTES: u32 = 64 * 1024;

struct SendHandle(HANDLE);
// SAFETY: HANDLE is a kernel object reference; usable from any thread.
unsafe impl Send for SendHandle {}

/// Owns the instance currently waiting for a client. Dropping it closes that instance; a
/// blocked [`PipeListener::accept`] on another thread is released with [`connect_self`].
pub struct PipeListener {
    name: Vec<u16>,
    waiting: Option<SendHandle>,
}

impl PipeListener {
    /// Creates the first instance with `FILE_FLAG_FIRST_PIPE_INSTANCE`: fails when another
    /// process already owns the name (the caller should then run without IPC).
    pub fn bind(name: &str) -> Result<Self> {
        let wide = to_wide(name);
        let first = create_instance(&wide, true)?;
        Ok(Self {
            name: wide,
            waiting: Some(first),
        })
    }

    /// Blocks until a client connects and returns the connected instance as a `File` (read the
    /// request, write the reply, then [`finish`] it). A fresh listening instance is created
    /// before this returns.
    pub fn accept(&mut self) -> Result<File> {
        let waiting = match self.waiting.take() {
            Some(h) => h,
            // The previous accept could not create a successor: try again now.
            None => create_instance(&self.name, false)?,
        };
        // SAFETY: `waiting` is an unconnected server instance we own.
        let connected = unsafe { ConnectNamedPipe(waiting.0, None) }.as_bool();
        if !connected {
            // SAFETY: plain FFI call.
            let code = unsafe { GetLastError() };
            if code != ERROR_PIPE_CONNECTED as u32 {
                let error = Error::from_thread();
                // SAFETY: we own the handle; it is not handed out.
                unsafe {
                    let _ = CloseHandle(waiting.0);
                }
                return Err(error);
            }
        }
        // Keep the name listening before anyone else sees this connection.
        match create_instance(&self.name, false) {
            Ok(next) => self.waiting = Some(next),
            Err(error) => {
                tracing::warn!(%error, "ipc: could not create the next pipe instance");
                self.waiting = None;
            }
        }
        // SAFETY: the handle is a valid, connected pipe instance whose ownership moves into the
        // File (closed on drop).
        Ok(unsafe { File::from_raw_handle(waiting.0.0 as _) })
    }
}

impl Drop for PipeListener {
    fn drop(&mut self) {
        if let Some(h) = self.waiting.take() {
            // SAFETY: we own the handle.
            unsafe {
                let _ = CloseHandle(h.0);
            }
        }
    }
}

fn create_instance(name: &[u16], first: bool) -> Result<SendHandle> {
    let mut open_mode = PIPE_ACCESS_DUPLEX as u32;
    if first {
        open_mode |= FILE_FLAG_FIRST_PIPE_INSTANCE as u32;
    }
    let pipe_mode =
        (PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS) as u32;
    // SAFETY: plain FFI call with a NUL-terminated name.
    let h = unsafe {
        CreateNamedPipeW(
            PCWSTR(name.as_ptr()),
            open_mode,
            pipe_mode,
            PIPE_UNLIMITED_INSTANCES as u32,
            BUFFER_BYTES,
            BUFFER_BYTES,
            0,
            None,
        )
    };
    if h == INVALID_HANDLE_VALUE || h.0.is_null() {
        return Err(Error::from_thread());
    }
    Ok(SendHandle(h))
}

/// Waits until the client has read everything written so far, then disconnects the instance.
/// (Plain `CloseHandle` may discard unread reply bytes; `DisconnectNamedPipe` alone certainly
/// does.) Errors are ignored: a client that already left cannot be helped.
pub fn finish(connection: &File) {
    let h = HANDLE(connection.as_raw_handle() as _);
    // SAFETY: the File owns a connected pipe instance; both calls only act on that handle.
    unsafe {
        let _ = FlushFileBuffers(h);
        let _ = DisconnectNamedPipe(h);
    }
}

/// Connects to our own pipe once so a listener thread blocked in [`PipeListener::accept`] wakes
/// up and can notice its stop flag. Failures are ignored (no instance was waiting).
pub fn connect_self(name: &str) {
    let _ = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(name);
}

/// Win32 error code a client sees when the server closed the pipe (`ERROR_BROKEN_PIPE`) or
/// when it wrote to a pipe the peer has already closed (`ERROR_NO_DATA`).
pub fn is_peer_gone(error: &std::io::Error) -> bool {
    matches!(
        error.raw_os_error(),
        Some(code) if code == ERROR_BROKEN_PIPE || code == ERROR_NO_DATA
    )
}
