//! Reading and toggling the "Show desktop icons" shell setting (plan §5.3).
//!
//! Reading uses `SHGetSetSettings(SSF_HIDEICONS)`. Toggling sends the same `WM_COMMAND 0x7402`
//! that Explorer's "View → Show desktop icons" menu item sends to `SHELLDLL_DefView`, so Explorer
//! persists the state itself (registry `HideIcons`) and the menu stays in sync.

use crate::bindings::*;
use crate::wide::to_wide;
use windows_core::PCWSTR;

const HIDE_ICONS_BIT: u32 = 1 << 12; // SHELLSTATE::fHideIcons
const CMD_TOGGLE_DESKTOP_ICONS: usize = 0x7402;
const LVM_GETSELECTEDCOUNT: u32 = 0x1000 + 50; // LVM_FIRST + 50
const LVM_GETHOTITEM: u32 = 0x1000 + 61; // LVM_FIRST + 61
const LVM_GETITEMCOUNT: u32 = 0x1000 + 4; // LVM_FIRST + 4
const LVM_GETITEMSTATE: u32 = 0x1000 + 44; // LVM_FIRST + 44
const LVIS_DROPHILITED: isize = 0x0008;

/// True when the shell currently hides desktop icons.
pub fn desktop_icons_hidden() -> bool {
    let mut state = SHELLSTATEW::default();
    // SAFETY: SHELLSTATEA and SHELLSTATEW share the same layout; `state` outlives the call.
    unsafe {
        SHGetSetSettings(
            Some((&mut state as *mut SHELLSTATEW).cast()),
            SSF_HIDEICONS as u32,
            false,
        );
    }
    (state._bitfield1.0 as u32) & HIDE_ICONS_BIT != 0
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

/// True while a drag is over one of Windows' own desktop icons that takes the drop (a folder,
/// a program, the Recycle Bin): `window` is Explorer's desktop list view, which marks that icon
/// drop-highlighted. False for its empty space, any other window, or no answer. Each icon is
/// asked on its own: on build 26300 `LVM_GETNEXTITEM` finds no drop-highlighted icon while
/// `LVM_GETITEMSTATE` shows it.
pub fn desktop_icon_drop_target(window: HWND) -> bool {
    if crate::desktop::class_name(window) != "SysListView32" {
        return false;
    }
    let ask = |msg: u32, wparam: usize, lparam: isize| {
        let mut out = 0usize;
        // SAFETY: plain FFI call with a timeout; Explorer owns the list view.
        unsafe {
            SendMessageTimeoutW(
                window,
                msg,
                WPARAM(wparam),
                LPARAM(lparam),
                SMTO_ABORTIFHUNG as u32,
                200,
                Some(&mut out),
            );
        }
        out
    };
    let count = ask(LVM_GETITEMCOUNT, 0, 0).min(4096);
    (0..count).any(|i| ask(LVM_GETITEMSTATE, i, LVIS_DROPHILITED) & LVIS_DROPHILITED as usize != 0)
}

/// Sets the hidden state, toggling only when it differs. Returns the state read back afterwards.
pub fn set_desktop_icons_hidden(hidden: bool, def_view: HWND, fallback: HWND) -> Option<bool> {
    if desktop_icons_hidden() != hidden && !toggle_desktop_icons(def_view, fallback) {
        return None;
    }
    Some(desktop_icons_hidden())
}
