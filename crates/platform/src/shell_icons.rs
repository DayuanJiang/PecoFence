//! Reading and toggling the "Show desktop icons" shell setting (plan §5.3).
//!
//! Reading uses `SHGetSetSettings(SSF_HIDEICONS)`. Toggling sends the same `WM_COMMAND 0x7402`
//! that Explorer's "View → Show desktop icons" menu item sends to `SHELLDLL_DefView`, so Explorer
//! persists the state itself (registry `HideIcons`) and the menu stays in sync.

use crate::bindings::*;

const HIDE_ICONS_BIT: u32 = 1 << 12; // SHELLSTATE::fHideIcons
const CMD_TOGGLE_DESKTOP_ICONS: usize = 0x7402;

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

/// Sets the hidden state, toggling only when it differs. Returns the state read back afterwards.
pub fn set_desktop_icons_hidden(hidden: bool, def_view: HWND, fallback: HWND) -> Option<bool> {
    if desktop_icons_hidden() != hidden && !toggle_desktop_icons(def_view, fallback) {
        return None;
    }
    Some(desktop_icons_hidden())
}
