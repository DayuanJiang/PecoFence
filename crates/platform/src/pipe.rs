//! Named-pipe server side of the CLI IPC (`\\.\pipe\PecoFence[.<instance>]`).
//!
//! One pipe instance serves one connection. [`PipeListener`] keeps exactly one instance waiting
//! in `ConnectNamedPipe` at all times: the next instance is created *before* a connected one is
//! handed out, so a client never sees `ERROR_FILE_NOT_FOUND` between two requests and mistakes
//! a busy server for a stopped one.
//!
//! Every instance carries a DACL that admits only SYSTEM, the Administrators group and the user
//! this process runs as; other local accounts cannot open the pipe at all.

use crate::bindings::*;
use crate::wide::{from_wide, to_wide};
use std::fs::File;
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use windows_core::{Error, PCWSTR, PWSTR, Result};

/// In/out buffer size of every instance.
const BUFFER_BYTES: u32 = 64 * 1024;

struct SendHandle(HANDLE);
// SAFETY: HANDLE is a kernel object reference; usable from any thread.
unsafe impl Send for SendHandle {}

/// A self-relative security descriptor allocated by `ConvertStringSecurityDescriptorToSecurityDescriptorW`.
struct SecurityDescriptor(PSECURITY_DESCRIPTOR);
// SAFETY: an opaque LocalAlloc block that is only read after construction.
unsafe impl Send for SecurityDescriptor {}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        // SAFETY: the block came from ConvertStringSecurityDescriptorToSecurityDescriptorW,
        // which documents LocalFree as its deallocator.
        unsafe {
            let _ = LocalFree(HANDLE(self.0.0));
        }
    }
}

/// Owns the instance currently waiting for a client. Dropping it closes that instance; a
/// blocked [`PipeListener::accept`] on another thread is released with [`connect_self`] or by
/// cancelling the thread's I/O ([`ThreadHandle::cancel_io`]).
pub struct PipeListener {
    name: Vec<u16>,
    waiting: Option<SendHandle>,
    security: Option<SecurityDescriptor>,
}

impl PipeListener {
    /// Creates the first instance with `FILE_FLAG_FIRST_PIPE_INSTANCE`: fails when another
    /// process already owns the name (the caller should then run without IPC).
    pub fn bind(name: &str) -> Result<Self> {
        let wide = to_wide(name);
        let security = match owner_only_security_descriptor() {
            Ok(sd) => Some(sd),
            Err(error) => {
                tracing::warn!(%error, "ipc: could not build the pipe DACL; using the default one");
                None
            }
        };
        let first = create_instance(&wide, true, security.as_ref())?;
        Ok(Self {
            name: wide,
            waiting: Some(first),
            security,
        })
    }

    /// Whether an instance is currently listening (creation of one failed otherwise; the
    /// caller should back off briefly before the next [`accept`]).
    pub fn is_listening(&self) -> bool {
        self.waiting.is_some()
    }

    fn listen(&mut self) {
        match create_instance(&self.name, false, self.security.as_ref()) {
            Ok(next) => self.waiting = Some(next),
            Err(error) => {
                tracing::warn!(%error, "ipc: could not create the next pipe instance");
                self.waiting = None;
            }
        }
    }

    /// Blocks until a client connects and returns the connected instance as a `File` (read the
    /// request, write the reply, then [`finish`] it). A fresh listening instance is created
    /// before this returns, on success and on failure alike, so a later [`connect_self`] always
    /// finds one; only when creation itself failed is the listener left without one.
    pub fn accept(&mut self) -> Result<File> {
        let waiting = match self.waiting.take() {
            Some(h) => h,
            // The previous accept could not create a successor: try again now.
            None => create_instance(&self.name, false, self.security.as_ref())?,
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
                self.listen();
                return Err(error);
            }
        }
        // Keep the name listening before anyone else sees this connection.
        self.listen();
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

fn create_instance(
    name: &[u16],
    first: bool,
    security: Option<&SecurityDescriptor>,
) -> Result<SendHandle> {
    let mut open_mode = PIPE_ACCESS_DUPLEX as u32;
    if first {
        open_mode |= FILE_FLAG_FIRST_PIPE_INSTANCE as u32;
    }
    let pipe_mode =
        (PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS) as u32;
    let attributes = security.map(|sd| SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: sd.0.0,
        bInheritHandle: false.into(),
    });
    // SAFETY: plain FFI call with a NUL-terminated name; the attributes (if any) outlive it.
    let h = unsafe {
        CreateNamedPipeW(
            PCWSTR(name.as_ptr()),
            open_mode,
            pipe_mode,
            PIPE_UNLIMITED_INSTANCES as u32,
            BUFFER_BYTES,
            BUFFER_BYTES,
            0,
            attributes.as_ref().map(|a| a as *const SECURITY_ATTRIBUTES),
        )
    };
    if h == INVALID_HANDLE_VALUE || h.0.is_null() {
        return Err(Error::from_thread());
    }
    Ok(SendHandle(h))
}

// ---- security ------------------------------------------------------------------------------

/// SDDL granting full access to SYSTEM, the Administrators group and `user_sid` only, with a
/// protected DACL (no inherited entries) and no other ACEs.
pub fn owner_only_sddl(user_sid: &str) -> String {
    format!("D:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;{user_sid})")
}

/// String SID (`S-1-5-21-…`) of the account this process runs as.
fn current_user_sid() -> Result<String> {
    struct Token(HANDLE);
    impl Drop for Token {
        fn drop(&mut self) {
            // SAFETY: we own the token handle.
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
    let mut raw = HANDLE::default();
    // SAFETY: GetCurrentProcess is a pseudo handle; `raw` receives the token.
    if !unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY as u32, &mut raw) }.as_bool() {
        return Err(Error::from_thread());
    }
    let token = Token(raw);
    let mut needed = 0u32;
    // SAFETY: a size query; failing with ERROR_INSUFFICIENT_BUFFER is the expected outcome.
    unsafe {
        let _ = GetTokenInformation(token.0, TokenUser, None, 0, &mut needed);
    }
    if needed == 0 {
        return Err(Error::from_thread());
    }
    let mut buffer = vec![0u8; needed as usize];
    // SAFETY: the buffer is `needed` bytes, as the previous call requested.
    if !unsafe {
        GetTokenInformation(
            token.0,
            TokenUser,
            Some(buffer.as_mut_ptr().cast()),
            needed,
            &mut needed,
        )
    }
    .as_bool()
    {
        return Err(Error::from_thread());
    }
    // SAFETY: the call above filled the buffer with a TOKEN_USER whose Sid points inside it.
    let sid = unsafe { (*buffer.as_ptr().cast::<TOKEN_USER>()).User.Sid };
    let mut text = PWSTR::null();
    // SAFETY: `sid` points into `buffer`, which is alive; `text` receives a LocalAlloc string.
    if !unsafe { ConvertSidToStringSidW(sid, &mut text) }.as_bool() {
        return Err(Error::from_thread());
    }
    // SAFETY: `text` is a NUL-terminated string we must LocalFree.
    let result = unsafe {
        let s = from_wide(text.as_wide());
        let _ = LocalFree(HANDLE(text.0.cast()));
        s
    };
    Ok(result)
}

fn owner_only_security_descriptor() -> Result<SecurityDescriptor> {
    let sddl = to_wide(&owner_only_sddl(&current_user_sid()?));
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    // SAFETY: plain FFI call; `descriptor` receives a LocalAlloc block owned by the returned
    // wrapper.
    if !unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(sddl.as_ptr()),
            SDDL_REVISION_1 as u32,
            &mut descriptor,
            None,
        )
    }
    .as_bool()
    {
        return Err(Error::from_thread());
    }
    if descriptor.0.is_null() {
        return Err(Error::from_thread());
    }
    Ok(SecurityDescriptor(descriptor))
}

// ---- I/O completion and cancellation ---------------------------------------------------------

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

/// Win32 error code a blocked read/write/connect reports after [`ThreadHandle::cancel_io`]
/// (`ERROR_OPERATION_ABORTED`).
pub fn is_cancelled(error: &std::io::Error) -> bool {
    error.raw_os_error() == Some(ERROR_OPERATION_ABORTED)
}

/// A real handle to a thread, good for [`cancel_io`](Self::cancel_io). Closed on drop.
pub struct ThreadHandle(HANDLE);
// SAFETY: a kernel handle; the watchdog thread uses it while the owning thread may be blocked.
unsafe impl Send for ThreadHandle {}
// SAFETY: CancelSynchronousIo is safe to call concurrently on the same handle.
unsafe impl Sync for ThreadHandle {}

impl ThreadHandle {
    /// Handle to the calling thread (`OpenThread(THREAD_TERMINATE)`, the right the cancel
    /// needs). `None` when the handle could not be opened.
    pub fn current() -> Option<Self> {
        // SAFETY: plain FFI calls; the id is our own.
        let h = unsafe { OpenThread(THREAD_TERMINATE as u32, false, GetCurrentThreadId()) };
        (!h.0.is_null()).then_some(Self(h))
    }

    /// Wraps a handle owned by someone else (e.g. `JoinHandle::as_raw_handle`); the wrapper
    /// does not close it.
    pub fn cancel_io_on_raw(raw: std::os::windows::io::RawHandle) {
        // SAFETY: cancelling I/O on a foreign thread handle only affects that thread's pending
        // synchronous operations.
        unsafe {
            let _ = CancelSynchronousIo(HANDLE(raw as _));
        }
    }

    /// Aborts the synchronous I/O the thread is blocked in right now (it fails with
    /// `ERROR_OPERATION_ABORTED`); a no-op when none is pending.
    pub fn cancel_io(&self) {
        // SAFETY: the handle is valid until drop.
        unsafe {
            let _ = CancelSynchronousIo(self.0);
        }
    }
}

impl Drop for ThreadHandle {
    fn drop(&mut self) {
        // SAFETY: we own the handle.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sddl_grants_system_admins_and_the_user_only() {
        let sddl = owner_only_sddl("S-1-5-21-1-2-3-1001");
        assert_eq!(
            sddl,
            "D:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;S-1-5-21-1-2-3-1001)"
        );
        // Protected DACL, exactly three allow ACEs, no Everyone / Authenticated Users entries.
        assert!(sddl.starts_with("D:P("));
        assert_eq!(sddl.matches("(A;;").count(), 3);
        assert!(!sddl.contains(";;;WD)") && !sddl.contains(";;;AU)"));
    }

    #[test]
    fn current_user_sid_is_well_formed_and_the_descriptor_builds() {
        let sid = current_user_sid().expect("token query");
        assert!(sid.starts_with("S-1-"), "{sid}");
        assert!(
            sid.split('-').skip(1).all(|p| p.parse::<u64>().is_ok()),
            "{sid}"
        );
        let sd = owner_only_security_descriptor().expect("descriptor");
        assert!(!sd.0.0.is_null());
    }

    #[test]
    fn accept_blocked_with_no_client_is_released_by_cancelling_the_threads_io() {
        let name = format!(
            r"\\.\pipe\PecoFence.test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let mut listener = PipeListener::bind(&name).expect("bind");
        let (tx, rx) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let result = listener.accept();
            let cancelled = result
                .as_ref()
                .err()
                .is_some_and(|e| e.code().0 as u16 == ERROR_OPERATION_ABORTED as u16);
            // The listener recreated an instance right away (the error was not creation).
            let _ = tx.send((cancelled, listener.is_listening()));
        });
        // Let the thread reach ConnectNamedPipe, then abort it the way IpcServer::drop does.
        std::thread::sleep(std::time::Duration::from_millis(200));
        ThreadHandle::cancel_io_on_raw(thread.as_raw_handle());
        let (cancelled, listening) = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("accept must return after CancelSynchronousIo");
        assert!(cancelled, "accept should fail with ERROR_OPERATION_ABORTED");
        assert!(listening);
        thread.join().unwrap();
    }

    #[test]
    fn cancelled_and_peer_gone_codes_are_recognised() {
        let aborted = std::io::Error::from_raw_os_error(ERROR_OPERATION_ABORTED);
        assert!(is_cancelled(&aborted));
        assert!(!is_peer_gone(&aborted));
        let broken = std::io::Error::from_raw_os_error(ERROR_BROKEN_PIPE);
        assert!(is_peer_gone(&broken));
        assert!(!is_cancelled(&broken));
        assert!(ThreadHandle::current().is_some());
    }
}
