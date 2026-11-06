//! 小工具「对话框跳转」: an Open / Save dialog of any program jumps to the folder the user was
//! just looking at, the way Listary's Quick Switch behaves:
//!
//! - A dialog keeps its own folder when it opens.
//! - Coming back to it from an Explorer window makes it jump, once, to that window's folder.
//! - Pressing a folder fence (a folder portal, or a fence showing an opened folder) counts like
//!   an Explorer window; with the dialog in front it jumps at once.
//! - Ctrl+G in the dialog jumps there at any time. The shortcut is registered only while a
//!   dialog is in front, so Ctrl+G stays free in every other window.
//! - While Listary runs, coming back from Explorer leaves the jump to Listary's own Quick
//!   Switch (see `listary_running`); Ctrl+G and folder fences still jump.
//!
//! While the tool is off and has never been offered, a file dialog that comes to the front
//! while an Explorer window shows another folder is offered a jump there (`set_offering`); the
//! app asks once in the corner and switches the tool on when the offer is taken.
//!
//! The foreground is followed with an out-of-context WinEvent hook (callbacks on this thread);
//! the jump itself runs on a short-lived worker thread, as it waits for the other program.

use pecofence_platform::dialogs::{self, JumpOutcome};
use pecofence_platform::winevent::{self, WinEventHook};
use pecofence_platform::{HWND, desktop, hotkey};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

/// `WM_HOTKEY` id of Ctrl+G on the control window.
pub const HOTKEY_ID: i32 = 4;
/// Control-window timer: a `#32770` window came to the front before its controls existed
/// (a file dialog builds its address bar and file list after it is shown); look again.
pub const RECHECK_TIMER: usize = 70;
const RECHECK_MS: u32 = 250;
const RECHECKS: u8 = 6;
/// Control-window timer while a file dialog is open: the window in front is read again, as
/// Explorer's own foreground event can be missing (seen when its tab row was clicked).
pub const POLL_TIMER: usize = 72;
const POLL_MS: u32 = 400;
const VK_G: u32 = 0x47;
/// Dialogs remembered as already seen (so their later activations may jump).
const SEEN_MAX: usize = 16;
/// The offer looks at a dialog this long after it came to the front: it builds its address
/// bar after it is shown.
const OFFER_SETTLE_MS: u64 = 400;

/// Where a jump goes.
#[derive(Clone, Debug)]
enum Source {
    /// An Explorer window: its folder is read when the jump runs.
    Explorer(isize),
    /// A folder fence's folder.
    Folder(PathBuf),
}

struct Jumper {
    _hook: WinEventHook,
    control: HWND,
    /// The last Explorer window or folder fence the user was in.
    source: Option<Source>,
    /// The source was used since a dialog was last in front: that dialog's next activation
    /// jumps (once).
    pending: bool,
    /// Dialogs that have been in front (newest last); the first activation of a dialog is its
    /// opening, which keeps the folder the program chose.
    seen: Vec<isize>,
    /// The dialog Ctrl+G is registered for.
    hotkey_for: Option<isize>,
    /// A `#32770` window that came to the front before it was recognizable as a file dialog,
    /// and how many looks are left for it.
    recheck_for: Option<isize>,
    rechecks: u8,
    /// The last window handled as the one in front (events and polls).
    last_front: Option<isize>,
}

thread_local! {
    static JUMPER: RefCell<Option<Jumper>> = const { RefCell::new(None) };
}

fn h(raw: isize) -> HWND {
    HWND(raw as *mut core::ffi::c_void)
}

/// Turns the tool on or off (settings `tools.dialogJump`). `control` receives Ctrl+G.
pub fn set_enabled(on: bool, control: HWND) {
    let running = JUMPER.with(|j| j.borrow().is_some());
    if on == running {
        return;
    }
    if !on {
        if let Some(j) = JUMPER.with(|j| j.borrow_mut().take()) {
            if j.hotkey_for.is_some() {
                hotkey::unregister(j.control, HOTKEY_ID);
            }
            pecofence_platform::window::kill_timer(j.control, POLL_TIMER);
            pecofence_platform::window::kill_timer(j.control, RECHECK_TIMER);
        }
        tracing::info!("dialog jump off");
        return;
    }
    let hook = WinEventHook::install(
        winevent::SYSTEM_FOREGROUND,
        winevent::SYSTEM_FOREGROUND,
        Box::new(|_event, hwnd| on_foreground(hwnd)),
    );
    match hook {
        Ok(hook) => {
            JUMPER.with(|j| {
                *j.borrow_mut() = Some(Jumper {
                    _hook: hook,
                    control,
                    source: None,
                    pending: false,
                    seen: Vec::new(),
                    hotkey_for: None,
                    recheck_for: None,
                    rechecks: 0,
                    last_front: None,
                })
            });
            tracing::info!("dialog jump on");
            // A dialog already in front: it counts as seen and gets Ctrl+G.
            on_foreground(desktop::foreground_window());
        }
        Err(e) => tracing::error!(error = %e, "dialog jump: SetWinEventHook failed"),
    }
}

fn on_foreground(hwnd: HWND) {
    JUMPER.with(|cell| {
        let mut guard = cell.borrow_mut();
        let Some(j) = guard.as_mut() else {
            return;
        };
        let raw = hwnd.0 as isize;
        j.last_front = Some(raw);
        let is_dialog = dialogs::is_file_dialog(hwnd);
        tracing::debug!(
            window = raw,
            window_class = %desktop::class_name(hwnd),
            is_dialog,
            "dialog jump: foreground"
        );
        if !is_dialog && desktop::class_name(hwnd) == "#32770" {
            // Possibly a file dialog still building its controls: look again shortly.
            if j.recheck_for != Some(raw) {
                j.recheck_for = Some(raw);
                j.rechecks = RECHECKS;
            }
            pecofence_platform::window::set_timer(j.control, RECHECK_TIMER, RECHECK_MS);
        } else {
            j.recheck_for = None;
        }
        if is_dialog {
            if adopt(j, hwnd) {
                // Its first time in front: it just opened and keeps the folder its program chose.
                j.pending = false;
                return;
            }
            if j.pending
                && let Some(source) = j.source.clone()
            {
                j.pending = false;
                if listary_running() {
                    tracing::info!(
                        dialog = raw,
                        "dialog jump: back in the dialog, left to Listary (it runs)"
                    );
                } else {
                    start_jump(hwnd, source, "back in the dialog");
                }
            } else {
                tracing::debug!(
                    dialog = raw,
                    pending = j.pending,
                    "dialog jump: dialog in front, no jump"
                );
            }
            return;
        }
        if let Some(d) = j.hotkey_for.take() {
            hotkey::unregister(j.control, HOTKEY_ID);
            tracing::debug!(dialog = d, "dialog jump: Ctrl+G released");
        }
        if dialogs::is_explorer(hwnd) {
            j.source = Some(Source::Explorer(raw));
            j.pending = true;
            tracing::debug!(
                explorer = raw,
                "dialog jump: Explorer in front, noted as the source"
            );
        }
    });
}

/// Ctrl+G for `dlg` and, the first time it is seen, notes it as seen. True when it was new.
fn adopt(j: &mut Jumper, dlg: HWND) -> bool {
    let raw = dlg.0 as isize;
    if j.hotkey_for != Some(raw) {
        if j.hotkey_for.is_some() {
            hotkey::unregister(j.control, HOTKEY_ID);
        }
        let ok = hotkey::register(
            j.control,
            HOTKEY_ID,
            hotkey::mods::CONTROL | hotkey::mods::NOREPEAT,
            VK_G,
        );
        j.hotkey_for = ok.then_some(raw);
        if !ok {
            tracing::warn!("dialog jump: Ctrl+G is taken by another program");
        }
    }
    if j.seen.contains(&raw) {
        return false;
    }
    j.seen.retain(|&d| desktop::is_window(h(d)));
    if j.seen.len() >= SEEN_MAX {
        j.seen.remove(0);
    }
    j.seen.push(raw);
    pecofence_platform::window::set_timer(j.control, POLL_TIMER, POLL_MS);
    tracing::info!(
        dialog = raw,
        kind = ?dialogs::dialog_kind(dlg),
        address = ?dialogs::dialog_address(dlg),
        "dialog jump: a dialog opened; it keeps its own folder"
    );
    true
}

/// `POLL_TIMER`: while a dialog seen here is open, a change of the window in front that no
/// event reported is handled as if it had been. Stops once every such dialog is closed.
pub fn poll(control: HWND) {
    let changed = JUMPER.with(|cell| {
        let mut guard = cell.borrow_mut();
        let j = guard.as_mut()?;
        j.seen.retain(|&d| desktop::is_window(h(d)));
        if j.seen.is_empty() {
            pecofence_platform::window::kill_timer(control, POLL_TIMER);
            return None;
        }
        let front = desktop::foreground_window();
        let raw = front.0 as isize;
        // PecoFence's own windows (the Peek, menus, settings) give no events on purpose.
        if front.0.is_null()
            || j.last_front == Some(raw)
            || desktop::window_pid(front) == pecofence_platform::process::current_pid()
        {
            return None;
        }
        Some(front)
    });
    if let Some(front) = changed {
        tracing::debug!(
            window = front.0 as isize,
            "dialog jump: front window changed without an event"
        );
        on_foreground(front);
    }
}

/// `RECHECK_TIMER`: the `#32770` window that came to the front may have become a file dialog.
pub fn recheck(control: HWND) {
    pecofence_platform::window::kill_timer(control, RECHECK_TIMER);
    let target = JUMPER.with(|cell| {
        let mut guard = cell.borrow_mut();
        let j = guard.as_mut()?;
        j.rechecks = j.rechecks.saturating_sub(1);
        Some((j.recheck_for?, j.rechecks))
    });
    let Some((raw, left)) = target else {
        return;
    };
    let dlg = h(raw);
    if dialogs::is_file_dialog(dlg) {
        JUMPER.with(|cell| {
            if let Some(j) = cell.borrow_mut().as_mut() {
                j.recheck_for = None;
            }
        });
        if desktop::foreground_window() == dlg {
            on_foreground(dlg);
        }
    } else if left > 0 && desktop::is_window(dlg) {
        pecofence_platform::window::set_timer(control, RECHECK_TIMER, RECHECK_MS);
    } else {
        tracing::info!(
            window = raw,
            "dialog jump: a #32770 window in front is no file dialog"
        );
    }
}

/// A folder fence was pressed: its folder is the source now; a dialog in front jumps at once
/// (the fence takes no activation, so the dialog may never leave the front).
pub fn note_folder(dir: &Path) {
    JUMPER.with(|cell| {
        let mut guard = cell.borrow_mut();
        let Some(j) = guard.as_mut() else {
            return;
        };
        j.source = Some(Source::Folder(dir.to_path_buf()));
        let front = desktop::foreground_window();
        if dialogs::is_file_dialog(front) {
            // Pressing a folder fence over a dialog means "go there", seen before or not.
            adopt(j, front);
            j.pending = false;
            start_jump(
                front,
                Source::Folder(dir.to_path_buf()),
                "folder fence pressed",
            );
        } else {
            j.pending = true;
            tracing::debug!(dir = %dir.display(), "dialog jump: folder fence noted as the source");
        }
    });
}

/// Ctrl+G (`WM_HOTKEY` with [`HOTKEY_ID`]): the dialog in front jumps to the last source, or
/// to the topmost Explorer window when there is none yet.
pub fn on_hotkey() {
    let front = desktop::foreground_window();
    if !dialogs::is_file_dialog(front) {
        tracing::debug!("dialog jump: Ctrl+G with no dialog in front; passed on");
        JUMPER.with(|cell| {
            if let Some(j) = cell.borrow_mut().as_mut()
                && j.hotkey_for.take().is_some()
            {
                hotkey::unregister(j.control, HOTKEY_ID);
            }
        });
        pecofence_platform::window::send_key(VK_G);
        return;
    }
    let source = JUMPER.with(|cell| {
        let mut guard = cell.borrow_mut();
        let j = guard.as_mut()?;
        j.pending = false;
        j.source.clone()
    });
    let source =
        source.or_else(|| dialogs::topmost_explorer().map(|e| Source::Explorer(e.0 as isize)));
    match source {
        Some(s) => start_jump(front, s, "Ctrl+G"),
        None => tracing::info!("dialog jump: Ctrl+G, but no Explorer window or folder fence yet"),
    }
}

/// The search box chose `path` while dialog `dlg` waited behind the Peek: the dialog opens
/// that folder, or opens / takes that file (`dialogs::choose`). Works with the tool off too;
/// a jump the tool had pending is dropped so it cannot follow.
pub fn send_to_dialog(dlg: HWND, path: PathBuf, is_folder: bool) {
    JUMPER.with(|cell| {
        if let Some(j) = cell.borrow_mut().as_mut() {
            j.pending = false;
        }
    });
    let dlg_raw = dlg.0 as isize;
    let spawned = std::thread::Builder::new()
        .name("dialog-choose".into())
        .spawn(move || {
            let started = std::time::Instant::now();
            let kind = dialogs::dialog_kind(h(dlg_raw));
            let outcome = dialogs::choose(h(dlg_raw), &path, is_folder);
            tracing::info!(
                path = %path.display(),
                is_folder,
                kind = ?kind,
                outcome = ?outcome,
                ms = started.elapsed().as_millis() as u64,
                "search result sent to the dialog"
            );
        });
    if let Err(e) = spawned {
        tracing::error!(error = %e, "dialog choose: worker thread failed to start");
    }
}

/// Listary (Listary.exe) has a window. Its Quick Switch moves the dialog from inside the
/// dialog's program when the user comes back from Explorer, at the same moment as this jump:
/// measured with both on, 4 of 20 returns ended in Windows' "... is currently unavailable" box
/// and 4 more jumped twice. Ctrl+G (PecoFence's hotkey takes it) and folder fences were clean.
fn listary_running() -> bool {
    let mut seen = std::collections::HashSet::new();
    desktop::top_level_windows().into_iter().any(|w| {
        let pid = desktop::window_pid(w);
        seen.insert(pid)
            && pecofence_platform::process::image_path(pid)
                .and_then(|p| p.file_name().map(|n| n.eq_ignore_ascii_case("Listary.exe")))
                .unwrap_or(false)
    })
}

/// The offer's foreground hook while it watches, and the dialogs it has looked at.
struct Watch {
    _hook: WinEventHook,
    control: HWND,
    seen: Vec<isize>,
}

thread_local! {
    static WATCH: RefCell<Option<Watch>> = const { RefCell::new(None) };
}

/// What the offer worker found: the dialog and the folder of the Explorer window in front of
/// the others.
static FOUND: std::sync::Mutex<Option<(isize, PathBuf)>> = std::sync::Mutex::new(None);

/// Watches for a dialog to offer the tool to (see the module text). `control` receives
/// `WM_APP_DIALOG_OFFER`.
pub fn set_offering(on: bool, control: HWND) {
    let running = WATCH.with(|w| w.borrow().is_some());
    if on == running {
        return;
    }
    if !on {
        WATCH.with(|w| w.borrow_mut().take());
        tracing::info!("dialog jump offer: not watching");
        return;
    }
    let hook = WinEventHook::install(
        winevent::SYSTEM_FOREGROUND,
        winevent::SYSTEM_FOREGROUND,
        Box::new(|_event, hwnd| on_offer_foreground(hwnd)),
    );
    match hook {
        Ok(hook) => {
            WATCH.with(|w| {
                *w.borrow_mut() = Some(Watch {
                    _hook: hook,
                    control,
                    seen: Vec::new(),
                })
            });
            tracing::info!("dialog jump offer: watching for a file dialog");
        }
        Err(e) => tracing::warn!(error = %e, "dialog jump offer: SetWinEventHook failed"),
    }
}

/// A `#32770` window came to the front: the first time for this window, a worker looks
/// whether it is a file dialog that does not show the folder of the Explorer window on top.
fn on_offer_foreground(hwnd: HWND) {
    if desktop::class_name(hwnd) != "#32770" {
        return;
    }
    let raw = hwnd.0 as isize;
    let control = WATCH.with(|cell| {
        let mut guard = cell.borrow_mut();
        let w = guard.as_mut()?;
        if w.seen.contains(&raw) {
            return None;
        }
        w.seen.retain(|&d| desktop::is_window(h(d)));
        if w.seen.len() >= SEEN_MAX {
            w.seen.remove(0);
        }
        w.seen.push(raw);
        Some(w.control.0 as isize)
    });
    let Some(control) = control else {
        return;
    };
    let spawned = std::thread::Builder::new()
        .name("dialog-offer".into())
        .spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(OFFER_SETTLE_MS));
            let dlg = h(raw);
            if !dialogs::is_file_dialog(dlg) || desktop::foreground_window() != dlg {
                tracing::debug!(
                    window = raw,
                    "dialog jump offer: no file dialog in front (any more)"
                );
                return;
            }
            if listary_running() {
                tracing::info!(dialog = raw, "dialog jump offer: Listary runs; no offer");
                return;
            }
            // Explorer's COM objects need an apartment on this thread.
            let _com = pecofence_platform::com::OleGuard::init();
            let Some(dir) = dialogs::topmost_explorer()
                .and_then(dialogs::explorer_folder)
                .filter(|d| d.is_dir())
            else {
                tracing::debug!(dialog = raw, "dialog jump offer: no Explorer folder");
                return;
            };
            if dialogs::dialog_shows(dlg, &dir) {
                tracing::debug!(
                    dialog = raw,
                    "dialog jump offer: the dialog shows it already"
                );
                return;
            }
            tracing::info!(dialog = raw, dir = %dir.display(), "dialog jump offer: found one");
            if let Ok(mut found) = FOUND.lock() {
                *found = Some((raw, dir));
            }
            pecofence_platform::window::post_message(
                h(control),
                crate::commands::WM_APP_DIALOG_OFFER,
                0,
                0,
            );
        });
    if let Err(e) = spawned {
        tracing::error!(error = %e, "dialog jump offer: worker thread failed to start");
    }
}

/// The dialog and folder the offer worker found, once.
pub fn take_offer() -> Option<(HWND, PathBuf)> {
    FOUND.lock().ok()?.take().map(|(d, dir)| (h(d), dir))
}

/// The offer was taken: `dlg` goes to `dir` now.
pub fn jump_to(dlg: HWND, dir: PathBuf) {
    start_jump(dlg, Source::Folder(dir), "offer taken");
}

fn start_jump(dlg: HWND, source: Source, reason: &'static str) {
    let dlg_raw = dlg.0 as isize;
    let spawned = std::thread::Builder::new()
        .name("dialog-jump".into())
        .spawn(move || {
            let started = std::time::Instant::now();
            let dir = match &source {
                Source::Folder(p) => Some(p.clone()),
                Source::Explorer(e) => {
                    // Explorer's COM objects need an apartment on this thread.
                    let _com = pecofence_platform::com::OleGuard::init();
                    // The window noted is closed: another one still open will do.
                    let window = if desktop::is_window(h(*e)) {
                        Some(h(*e))
                    } else {
                        dialogs::topmost_explorer()
                    };
                    window.and_then(dialogs::explorer_folder)
                }
            };
            let Some(dir) = dir.filter(|d| d.is_dir()) else {
                let kind = match &source {
                    Source::Explorer(_) => "Explorer",
                    Source::Folder(_) => "folder fence",
                };
                tracing::info!(
                    reason,
                    kind,
                    "dialog jump: the source has no folder (closed, or a place like This PC)"
                );
                return;
            };
            let outcome = dialogs::jump(h(dlg_raw), &dir);
            tracing::info!(
                reason,
                dir = %dir.display(),
                outcome = ?outcome,
                ms = started.elapsed().as_millis() as u64,
                address = ?dialogs::dialog_address(h(dlg_raw)),
                "dialog jump"
            );
            if !matches!(
                outcome,
                JumpOutcome::Done | JumpOutcome::Unconfirmed | JumpOutcome::AlreadyThere
            ) {
                tracing::warn!(outcome = ?outcome, "dialog jump failed");
            }
        });
    if let Err(e) = spawned {
        tracing::error!(error = %e, "dialog jump: worker thread failed to start");
    }
}
