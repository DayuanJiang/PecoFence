//! Directory change watcher: one blocking `ReadDirectoryChangesW` thread per folder.

use crate::bindings::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use windows_core::{Error, PCWSTR, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FsEvent {
    Added(PathBuf),
    Removed(PathBuf),
    Modified(PathBuf),
    RenamedOld(PathBuf),
    RenamedNew(PathBuf),
    /// The buffer overflowed; callers must re-enumerate.
    Overflow,
}

struct SendHandle(HANDLE);
// SAFETY: HANDLE is a kernel object reference; usable from any thread.
unsafe impl Send for SendHandle {}
unsafe impl Sync for SendHandle {}

/// Watches a single directory (non-recursive). Dropping stops the thread.
pub struct DirWatcher {
    handle: Arc<SendHandle>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl DirWatcher {
    /// Starts watching `dir`; `on_events` runs on the watcher thread for every batch.
    pub fn start(dir: &Path, on_events: impl Fn(Vec<FsEvent>) + Send + 'static) -> Result<Self> {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = dir
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        // SAFETY: opening a directory handle for change notification.
        let handle = unsafe {
            CreateFileW(
                PCWSTR(wide.as_ptr()),
                FILE_LIST_DIRECTORY as u32,
                (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as u32,
                None,
                OPEN_EXISTING as u32,
                FILE_FLAG_BACKUP_SEMANTICS as u32,
                None,
            )
        };
        if handle == INVALID_HANDLE_VALUE || handle.0.is_null() {
            return Err(Error::from_thread());
        }
        let handle = Arc::new(SendHandle(handle));
        let stop = Arc::new(AtomicBool::new(false));
        let dir = dir.to_path_buf();
        let thread = {
            let handle = handle.clone();
            let stop = stop.clone();
            std::thread::Builder::new()
                .name(format!("pecofence-watch-{}", dir.display()))
                .stack_size(256 * 1024)
                .spawn(move || watch_loop(handle, dir, stop, on_events))
                .map_err(|e| Error::new(windows_core::HRESULT(E_FAIL.0), e.to_string()))?
        };
        Ok(Self {
            handle,
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for DirWatcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // SAFETY: cancels the blocking ReadDirectoryChangesW on the watcher thread, then closes.
        unsafe {
            let _ = CancelIoEx(self.handle.0, None);
        }
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
        // SAFETY: handle is no longer used by any thread.
        unsafe {
            let _ = CloseHandle(self.handle.0);
        }
    }
}

const FILTER: u32 = (FILE_NOTIFY_CHANGE_FILE_NAME
    | FILE_NOTIFY_CHANGE_DIR_NAME
    | FILE_NOTIFY_CHANGE_ATTRIBUTES
    | FILE_NOTIFY_CHANGE_LAST_WRITE) as u32;

fn watch_loop(
    handle: Arc<SendHandle>,
    dir: PathBuf,
    stop: Arc<AtomicBool>,
    on_events: impl Fn(Vec<FsEvent>),
) {
    let mut buf = vec![0u8; 64 * 1024];
    while !stop.load(Ordering::SeqCst) {
        let mut returned = 0u32;
        // SAFETY: buffer is DWORD-aligned (Vec<u8> from the allocator is 8/16-aligned) and sized.
        let ok = unsafe {
            ReadDirectoryChangesW(
                handle.0,
                buf.as_mut_ptr().cast(),
                buf.len() as u32,
                false,
                FILTER,
                Some(&mut returned),
                None,
                None,
            )
        };
        if !ok.as_bool() {
            if stop.load(Ordering::SeqCst) {
                break;
            }
            // SAFETY: plain FFI call.
            let err = unsafe { GetLastError() };
            if err == ERROR_NOTIFY_ENUM_DIR as u32 {
                on_events(vec![FsEvent::Overflow]);
                continue;
            }
            tracing::warn!(error = err, dir = %dir.display(), "ReadDirectoryChangesW failed; watcher exiting");
            // The directory is gone (deleted / renamed / unmounted): nudge the app to re-enumerate
            // so it can fall back and re-point its watcher.
            on_events(vec![FsEvent::Overflow]);
            break;
        }
        if returned == 0 {
            on_events(vec![FsEvent::Overflow]);
            continue;
        }
        let events = parse(&buf[..returned as usize], &dir);
        if !events.is_empty() {
            on_events(events);
        }
    }
}

fn parse(buf: &[u8], dir: &Path) -> Vec<FsEvent> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    loop {
        if offset + 12 > buf.len() {
            break;
        }
        // SAFETY: bounds checked above; FILE_NOTIFY_INFORMATION header is three u32s followed by
        // a UTF-16 name of FileNameLength bytes.
        let next = u32::from_ne_bytes(buf[offset..offset + 4].try_into().unwrap()) as usize;
        let action = u32::from_ne_bytes(buf[offset + 4..offset + 8].try_into().unwrap());
        let name_len =
            u32::from_ne_bytes(buf[offset + 8..offset + 12].try_into().unwrap()) as usize;
        let name_start = offset + 12;
        let name_end = (name_start + name_len).min(buf.len());
        let name_u16: Vec<u16> = buf[name_start..name_end]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_ne_bytes([c[0], c[1]]))
            .collect();
        let name = String::from_utf16_lossy(&name_u16);
        let path = dir.join(name);
        let ev = match action {
            a if a == FILE_ACTION_ADDED as u32 => Some(FsEvent::Added(path)),
            a if a == FILE_ACTION_REMOVED as u32 => Some(FsEvent::Removed(path)),
            a if a == FILE_ACTION_MODIFIED as u32 => Some(FsEvent::Modified(path)),
            a if a == FILE_ACTION_RENAMED_OLD_NAME as u32 => Some(FsEvent::RenamedOld(path)),
            a if a == FILE_ACTION_RENAMED_NEW_NAME as u32 => Some(FsEvent::RenamedNew(path)),
            _ => None,
        };
        if let Some(ev) = ev {
            out.push(ev);
        }
        if next == 0 {
            break;
        }
        offset += next;
    }
    out
}
