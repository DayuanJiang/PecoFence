//! Reading and toggling the "Show desktop icons" shell setting (plan §5.3).
//!
//! Reading uses `SHGetSetSettings(SSF_HIDEICONS)`. Toggling sends the same `WM_COMMAND 0x7402`
//! that Explorer's "View → Show desktop icons" menu item sends to `SHELLDLL_DefView`, so Explorer
//! persists the state itself (registry `HideIcons`) and the menu stays in sync.

use crate::bindings::*;
use crate::wide::to_wide;
use windows_core::PCWSTR;

const HIDE_ICONS_BIT: u32 = 1 << 12; // SHELLSTATE::fHideIcons
const DOUBLE_CLICK_BIT: u32 = 1 << 5; // SHELLSTATE::fDoubleClickInWebView
const SSF_DOUBLECLICKINWEBVIEW: u32 = 0x80;
const SHOW_INFO_TIP_BIT: u32 = 1 << 11; // SHELLSTATE::fShowInfoTip
const SSF_SHOWINFOTIP: u32 = 0x2000;
const CMD_TOGGLE_DESKTOP_ICONS: usize = 0x7402;
const LVM_GETSELECTEDCOUNT: u32 = 0x1000 + 50; // LVM_FIRST + 50
const LVM_GETHOTITEM: u32 = 0x1000 + 61; // LVM_FIRST + 61
const LVM_GETITEMCOUNT: u32 = 0x1000 + 4; // LVM_FIRST + 4
const LVM_GETITEMSTATE: u32 = 0x1000 + 44; // LVM_FIRST + 44
const LVIS_DROPHILITED: isize = 0x0008;

/// The first bit field of the shell's `SHELLSTATE`, with the flags in `mask` filled in.
fn shell_flags(mask: u32) -> u32 {
    let mut state = SHELLSTATEW::default();
    // SAFETY: SHELLSTATEA and SHELLSTATEW share the same layout; `state` outlives the call.
    unsafe {
        SHGetSetSettings(Some((&mut state as *mut SHELLSTATEW).cast()), mask, false);
    }
    state._bitfield1.0 as u32
}

/// True when the shell currently hides desktop icons.
pub fn desktop_icons_hidden() -> bool {
    shell_flags(SSF_HIDEICONS as u32) & HIDE_ICONS_BIT != 0
}

/// True when Explorer opens items with a single click (Folder Options › "Single-click to open
/// an item (point to select)"), as the desktop then does.
pub fn single_click_open() -> bool {
    shell_flags(SSF_DOUBLECLICKINWEBVIEW) & DOUBLE_CLICK_BIT == 0
}

/// True when Explorer shows a tip for the item under a resting pointer (Folder Options › View ›
/// "Show pop-up description for folder and desktop items", on by default).
pub fn show_info_tips() -> bool {
    shell_flags(SSF_SHOWINFOTIP) & SHOW_INFO_TIP_BIT != 0
}

/// Sends the toggle command to the desktop view (falls back to `Progman` when `def_view` is
/// null). Returns false if Explorer did not answer within the timeout.
pub fn toggle_desktop_icons(def_view: HWND, fallback: HWND) -> bool {
    let target = if def_view.0.is_null() {
        fallback
    } else {
        def_view
    };
    if target.0.is_null() {
        return false;
    }
    let mut result: usize = 0;
    // SAFETY: plain FFI call with a timeout; Explorer owns the target window.
    let ok = unsafe {
        SendMessageTimeoutW(
            target,
            WM_COMMAND as u32,
            WPARAM(CMD_TOGGLE_DESKTOP_ICONS),
            LPARAM(0),
            SMTO_ABORTIFHUNG as u32,
            1000,
            Some(&mut result),
        )
    };
    ok.0 != 0
}

/// How many desktop icons are selected in Explorer's list view under `def_view`; None when
/// there is no view or Explorer did not answer.
pub fn desktop_selected_count(def_view: HWND) -> Option<usize> {
    if def_view.0.is_null() {
        return None;
    }
    let class = to_wide("SysListView32");
    // SAFETY: the string outlives the call.
    let list =
        unsafe { FindWindowExW(Some(def_view), None, PCWSTR(class.as_ptr()), PCWSTR::null()) };
    if list.0.is_null() {
        return None;
    }
    let mut count: usize = 0;
    // SAFETY: plain FFI call with a timeout; Explorer owns the list view.
    let ok = unsafe {
        SendMessageTimeoutW(
            list,
            LVM_GETSELECTEDCOUNT,
            WPARAM(0),
            LPARAM(0),
            SMTO_ABORTIFHUNG as u32,
            200,
            Some(&mut count),
        )
    };
    (ok.0 != 0).then_some(count)
}

/// True when the pointer is on one of Windows' own desktop icons: `window` (what
/// `WindowFromPoint` found under it) is Explorer's desktop list view, and the icon it tracks for
/// its hover highlight is set. False for its empty space, any other window, or no answer.
pub fn desktop_icon_hovered(window: HWND) -> bool {
    if crate::desktop::class_name(window) != "SysListView32" {
        return false;
    }
    let mut item = usize::MAX;
    // SAFETY: plain FFI call with a timeout; Explorer owns the list view.
    unsafe {
        SendMessageTimeoutW(
            window,
            LVM_GETHOTITEM,
            WPARAM(0),
            LPARAM(0),
            SMTO_ABORTIFHUNG as u32,
            200,
            Some(&mut item),
        );
    }
    (item as isize) >= 0
}

/// How long [`desktop_icon_drop_target`] may ask Explorer in all. 1500 icons take about
/// 30 ms when Explorer is idle.
const DROP_TARGET_BUDGET: std::time::Duration = std::time::Duration::from_millis(300);

/// True while a drag is over one of Windows' own desktop icons that takes the drop (a folder,
/// a program, the Recycle Bin): `window` is Explorer's desktop list view, which marks that icon
/// drop-highlighted. False for its empty space, any other window, or no answer. Each icon is
/// asked on its own: on build 26300 `LVM_GETNEXTITEM` finds no drop-highlighted icon while
/// `LVM_GETITEMSTATE` shows it. The asking ends at the first question Explorer does not
/// answer within 200 ms and after `DROP_TARGET_BUDGET` in all (a busy Explorer answered each
/// one only after up to 200 ms, a freeze of icons x 200 ms): the drop then goes back to the
/// fence, as on empty desktop space.
pub fn desktop_icon_drop_target(window: HWND) -> bool {
    if crate::desktop::class_name(window) != "SysListView32" {
        return false;
    }
    let started = std::time::Instant::now();
    let ask = |msg: u32, wparam: usize, lparam: isize| {
        let mut out = 0usize;
        // SAFETY: plain FFI call with a timeout; Explorer owns the list view.
        let answered = unsafe {
            SendMessageTimeoutW(
                window,
                msg,
                WPARAM(wparam),
                LPARAM(lparam),
                SMTO_ABORTIFHUNG as u32,
                200,
                Some(&mut out),
            )
        };
        (answered.0 != 0).then_some(out)
    };
    let Some(count) = ask(LVM_GETITEMCOUNT, 0, 0).map(|n| n.min(4096)) else {
        tracing::info!("desktop icon under a drop: Explorer did not answer; not a drop target");
        return false;
    };
    for i in 0..count {
        match ask(LVM_GETITEMSTATE, i, LVIS_DROPHILITED) {
            Some(state) if state & LVIS_DROPHILITED as usize != 0 => return true,
            Some(_) if started.elapsed() < DROP_TARGET_BUDGET => {}
            answer => {
                tracing::info!(
                    asked = i + 1,
                    count,
                    answered = answer.is_some(),
                    elapsed_ms = started.elapsed().as_millis(),
                    "desktop icon under a drop: Explorer too slow to ask every icon; not a drop target"
                );
                return false;
            }
        }
    }
    false
}

/// Sets the hidden state, toggling only when it differs. Returns the state read back afterwards.
pub fn set_desktop_icons_hidden(hidden: bool, def_view: HWND, fallback: HWND) -> Option<bool> {
    if desktop_icons_hidden() != hidden && !toggle_desktop_icons(def_view, fallback) {
        return None;
    }
    Some(desktop_icons_hidden())
}

/// What [`set_desktop_icons_hidden_within`] found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconToggle {
    /// The state read back (after a toggle, or at once when it already matched).
    Done(bool),
    /// Explorer has not answered within the wait: the toggle is still out, and no other is
    /// sent until it is back.
    Waiting,
    /// No answer from Explorer (it hangs), or no window to send to.
    Failed,
}

/// A toggle sent from [`set_desktop_icons_hidden_within`]'s worker that Explorer has not
/// answered yet: 0 none, `TOGGLE_HIDE` or `TOGGLE_SHOW`. A second one sent meanwhile would undo
/// the first once both are processed.
static TOGGLE_OUT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
const TOGGLE_HIDE: u8 = 1;
const TOGGLE_SHOW: u8 = 2;

/// How long the worker waits for Explorer to process a toggle. Explorer hides 1400 icons in
/// about 0.3 s when idle; at sign-in, or with a large copy into the desktop, it took 20 s.
const TOGGLE_ANSWER_MS: u32 = 60_000;

/// [`set_desktop_icons_hidden`] on a worker thread, waiting `wait` at most for it. The UI
/// thread waited on Explorer for 20.6 s here (the 1000 ms timeout of the call did not hold
/// there), with every fence frozen. A toggle the other way still out is waited for first
/// (within `wait`); one the same way is not sent again. A toggle still out when the process
/// ends is lost: exit waits longer.
pub fn set_desktop_icons_hidden_within(
    hidden: bool,
    def_view: HWND,
    fallback: HWND,
    wait: std::time::Duration,
) -> IconToggle {
    use std::sync::atomic::Ordering;
    let started = std::time::Instant::now();
    let want = if hidden { TOGGLE_HIDE } else { TOGGLE_SHOW };
    loop {
        match TOGGLE_OUT.load(Ordering::Acquire) {
            0 => break,
            out if out == want || started.elapsed() >= wait => return IconToggle::Waiting,
            _ => std::thread::sleep(std::time::Duration::from_millis(10)),
        }
    }
    let target = if def_view.0.is_null() {
        fallback
    } else {
        def_view
    };
    if target.0.is_null() {
        return IconToggle::Failed;
    }
    let (answer, answered) = std::sync::mpsc::channel();
    let target = target.0 as isize;
    TOGGLE_OUT.store(want, Ordering::Release);
    let spawned = std::thread::Builder::new()
        .name("pecofence-icons-toggle".into())
        .stack_size(64 * 1024)
        .spawn(move || {
            let started = std::time::Instant::now();
            let target = HWND(target as *mut core::ffi::c_void);
            // SAFETY: plain FFI call with a timeout; Explorer owns the target window. No
            // SMTO_ABORTIFHUNG: a busy Explorer that pumps no messages for 5 s still processes
            // the message later, and the next toggle must wait for it.
            let send = |msg: u32, wparam: usize| unsafe {
                let mut result: usize = 0;
                SendMessageTimeoutW(
                    target,
                    msg,
                    WPARAM(wparam),
                    LPARAM(0),
                    SMTO_NORMAL as u32,
                    TOGGLE_ANSWER_MS,
                    Some(&mut result),
                )
                .0 != 0
            };
            // Explorer answers sent messages in the order they came: once it has answered this
            // one, a toggle sent before (by an attempt given up on, or by a PecoFence that has
            // quit since: its message still arrives) is done, and the state read next is the
            // one to act on. Without it the watchdog toggled the icons back to hidden.
            let state = if !send(WM_NULL as u32, 0) {
                None
            } else if desktop_icons_hidden() == hidden {
                Some(hidden)
            } else {
                send(WM_COMMAND as u32, CMD_TOGGLE_DESKTOP_ICONS).then(desktop_icons_hidden)
            };
            TOGGLE_OUT.store(0, Ordering::Release);
            tracing::info!(
                hidden,
                ?state,
                elapsed_ms = started.elapsed().as_millis(),
                "desktop icons toggle answered"
            );
            let _ = answer.send(state);
        });
    if spawned.is_err() {
        TOGGLE_OUT.store(0, Ordering::Release);
        return set_desktop_icons_hidden(hidden, def_view, fallback)
            .map_or(IconToggle::Failed, IconToggle::Done);
    }
    let outcome = match answered.recv_timeout(wait.saturating_sub(started.elapsed())) {
        Ok(Some(state)) => IconToggle::Done(state),
        Ok(None) => IconToggle::Failed,
        Err(_) => IconToggle::Waiting,
    };
    if outcome != IconToggle::Done(hidden) {
        tracing::info!(
            hidden,
            ?outcome,
            waited_ms = started.elapsed().as_millis(),
            "desktop icons toggle not done within the wait"
        );
    }
    outcome
}
