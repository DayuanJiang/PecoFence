//! Shortcut targets for 类型：程序 (a shortcut is a program when it points to one), read off the
//! UI thread: IShellLink reads the .lnk file, slow on a cold or sleeping drive. One thread per
//! drive (its own STA), one cache by path and modified time shared by them. The UI asks with
//! [`known`], which never reads; [`ask`] queues an unknown one (or, without a time, a check of a
//! known one against its file), and the control window gets [`WM_APP_LNK_TARGETS`] once a batch
//! is done: the Peek list and the fence filters then show the rows they held back.

use pecofence_platform::com::{StaGuard, StaSender, sta_channel};
use pecofence_platform::{HWND, shell, window};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
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
/// A request ended since the UI last took them: one post for a batch.
static POSTED: AtomicBool = AtomicBool::new(false);
/// A target was read (the cache changed) since the UI last took them.
static CHANGED: AtomicBool = AtomicBool::new(false);
/// The requests that ended (lower-case paths): only then do they leave `ASKED`.
static DONE: LazyLock<Mutex<Vec<String>>> = LazyLock::new(Mutex::default);

thread_local! {
    static THREADS: RefCell<HashMap<String, StaSender<(String, Option<i64>)>>> =
        RefCell::new(HashMap::new());
    /// Asked and not ended yet: not asked again meanwhile.
    static ASKED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    static NOTIFY: Cell<isize> = const { Cell::new(0) };
}

/// The window the workers tell (the control window).
pub(super) fn init(control: HWND) {
    NOTIFY.with(|n| n.set(control.0 as isize));
}

/// The target of the shortcut at `path` when it was read (at `mtime`; None, for Everything's
/// hits which come without one: the last one read, checked against the file by an [`ask`] on
/// each answer); the outer None when it is not known yet. Never reads the file.
pub(super) fn known(path: &str, mtime: Option<i64>) -> Option<Option<String>> {
    let c = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let (m, t) = c.get(&path.to_lowercase())?;
    mtime.is_none_or(|x| x == *m).then(|| t.clone())
}

/// Queues a read of `path`'s target on its drive's thread: at `mtime`, or without one the file's
/// own time, read there (a target already known for that time is not read again).
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
            tx.send((path.to_string(), mtime));
        }
    });
}

/// The UI takes the batch: the ended requests may be asked again; true when a target was read.
pub(super) fn taken() -> bool {
    POSTED.store(false, Ordering::Release);
    let done = std::mem::take(&mut *DONE.lock().unwrap_or_else(|e| e.into_inner()));
    ASKED.with(|a| {
        let mut a = a.borrow_mut();
        for k in done {
            a.remove(&k);
        }
    });
    CHANGED.swap(false, Ordering::AcqRel)
}

fn spawn(drive: &str) -> Option<StaSender<(String, Option<i64>)>> {
    let (tx, rx) = sta_channel::<(String, Option<i64>)>();
    let notify = NOTIFY.with(|n| n.get());
    let spawned = std::thread::Builder::new()
        .name(format!("pecofence-lnk-{drive}"))
        .spawn(move || {
            let _sta = StaGuard::init();
            loop {
                let (path, mtime) = rx.recv();
                let key = path.to_lowercase();
                // The cache first; the file's time only when the asker had none.
                let cached = CACHE
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .get(&key)
                    .map(|(m, _)| *m);
                let mtime = match (mtime, cached) {
                    (Some(m), Some(c)) if m == c => None,
                    (Some(m), _) => Some(m),
                    (None, c) => Some(file_mtime(&path)).filter(|m| Some(*m) != c),
                };
                let Some(mtime) = mtime else {
                    ended(key, notify);
                    continue;
                };
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
                    c.insert(key.clone(), (mtime, target));
                }
                CHANGED.store(true, Ordering::Release);
                ended(key, notify);
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

/// A request ended: its path may be asked again once the UI takes the batch (one post for it).
fn ended(key: String, notify: isize) {
    DONE.lock().unwrap_or_else(|e| e.into_inner()).push(key);
    if !POSTED.swap(true, Ordering::AcqRel) {
        window::post_message(
            HWND(notify as *mut core::ffi::c_void),
            WM_APP_LNK_TARGETS,
            0,
            0,
        );
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
