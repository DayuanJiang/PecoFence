//! Shortcut targets for 类型：程序 (a shortcut is a program when it points to one), read off the
//! UI thread: IShellLink reads the .lnk file, slow on a cold or sleeping drive. One thread per
//! drive (its own STA), one cache by path and modified time shared by them. The UI asks with
//! [`known`], which never reads; [`ask`] queues an unknown one, and the control window gets
//! [`WM_APP_LNK_TARGETS`] once reads are done: the Peek list and the fence filters then show
//! the rows they held back.

use pecofence_platform::{HWND, shell, window};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

/// Reads finished (worker thread → the control window).
pub(super) const WM_APP_LNK_TARGETS: u32 = pecofence_platform::msg::WM_APP + 35;
/// `WM_APP_LNK_TARGETS` handled; retried while the App is borrowed.
pub(super) const TIMER_LNK_TARGETS: usize = 96;
/// The cache starts over here (a folder of shortcuts is a few hundred).
const CACHE_MAX: usize = 4096;

/// Lower-case path → the modified time it was read at (unix seconds) and its target.
static CACHE: LazyLock<Mutex<HashMap<String, (i64, Option<String>)>>> =
    LazyLock::new(Mutex::default);
/// A read finished since the UI last took them: one post for a batch.
static POSTED: AtomicBool = AtomicBool::new(false);

thread_local! {
    static THREADS: RefCell<HashMap<String, Sender<(String, Option<i64>)>>> =
        RefCell::new(HashMap::new());
    /// Asked and not taken yet (asked again only after the next batch is taken).
    static ASKED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    static NOTIFY: Cell<isize> = const { Cell::new(0) };
}

/// The window the workers tell (the control window).
pub(super) fn init(control: HWND) {
    NOTIFY.with(|n| n.set(control.0 as isize));
}

/// The target of the shortcut at `path` when it was read (at `mtime`; None: at any time); the
/// outer None when it is not known yet. Never reads the file.
pub(super) fn known(path: &str, mtime: Option<i64>) -> Option<Option<String>> {
    let c = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let (m, t) = c.get(&path.to_lowercase())?;
    mtime.is_none_or(|x| x == *m).then(|| t.clone())
}

/// Queues a read of `path`'s target on its drive's thread.
pub(super) fn ask(path: &str, mtime: Option<i64>) {
    if !ASKED.with(|a| a.borrow_mut().insert(path.to_lowercase())) {
        return;
    }
    let drive = crate::peek_search::drive_of(Path::new(path));
    THREADS.with(|t| {
        let mut t = t.borrow_mut();
        if !t.contains_key(&drive) {
            let Some(tx) = spawn(&drive) else { return };
            t.insert(drive.clone(), tx);
        }
        if let Some(tx) = t.get(&drive) {
            let _ = tx.send((path.to_string(), mtime));
        }
    });
}

/// The UI took the reads: later asks queue again (a read still on its way is skipped then).
pub(super) fn taken() {
    POSTED.store(false, Ordering::Release);
    ASKED.with(|a| a.borrow_mut().clear());
}

fn spawn(drive: &str) -> Option<Sender<(String, Option<i64>)>> {
    let (tx, rx) = channel::<(String, Option<i64>)>();
    let notify = NOTIFY.with(|n| n.get());
    let spawned = std::thread::Builder::new()
        .name(format!("pecofence-lnk-{drive}"))
        .spawn(move || {
            let _sta = pecofence_platform::com::OleGuard::init();
            for (path, mtime) in rx {
                let mtime = mtime.unwrap_or_else(|| file_mtime(&path));
                let key = path.to_lowercase();
                let fresh = CACHE
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .get(&key)
                    .is_some_and(|(m, _)| *m == mtime);
                if fresh {
                    continue;
                }
                let started = Instant::now();
                crate::peek_search::test_slow_read();
                let target = shell::shortcut_target(Path::new(&path));
                let took = started.elapsed();
                let limit = crate::peek_search::slow_limit();
                if took.as_millis() > limit || limit == 0 {
                    tracing::info!(stage = "lnk_target", ms = took.as_millis() as u64, us = took.as_micros() as u64, drive = %crate::peek_search::drive_of(Path::new(&path)), "search: a stage was slow");
                }
                {
                    let mut c = CACHE.lock().unwrap_or_else(|e| e.into_inner());
                    if c.len() >= CACHE_MAX {
                        c.clear();
                    }
                    tracing::debug!(
                        cached = c.len() + 1,
                        found = target.is_some(),
                        "type filter: shortcut target read"
                    );
                    c.insert(key, (mtime, target));
                }
                if !POSTED.swap(true, Ordering::AcqRel) {
                    window::post_message(
                        HWND(notify as *mut core::ffi::c_void),
                        WM_APP_LNK_TARGETS,
                        0,
                        0,
                    );
                }
            }
        });
    match spawned {
        Ok(_) => Some(tx),
        Err(e) => {
            tracing::warn!(error = %e, "type filter: no thread for shortcut targets");
            None
        }
    }
}

/// `path`'s modified time (unix seconds), 0 when it cannot be read.
fn file_mtime(path: &str) -> i64 {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs() as i64)
}
