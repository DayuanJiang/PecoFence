//! A tracking tooltip (`tooltips_class32`, `TTF_TRACK`) for a never-activated window: shown at
//! an explicit screen point, hidden by the owner. Explorer uses the same control for its icon
//! infotips.

use crate::bindings::*;
use crate::wide::to_wide;
use windows_core::{Error, PCWSTR, PWSTR, Result};

const TOOL_ID: usize = 1;

pub struct Tooltip {
    hwnd: HWND,
    owner: HWND,
    /// Backing store for `lpszText`; the control keeps the pointer we hand it.
    text: Vec<u16>,
}

impl Tooltip {
    /// Creates a hidden tracking tooltip owned by `owner` (`dark` picks Explorer's dark tip).
    pub fn create(owner: HWND, dark: bool) -> Result<Self> {
        let init = INITCOMMONCONTROLSEX {
            dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_BAR_CLASSES as u32,
        };
        // SAFETY: plain FFI calls; the tool struct outlives each SendMessage.
        unsafe {
            let _ = InitCommonControlsEx(&init);
            let hwnd = CreateWindowExW(
                (WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE) as u32,
                TOOLTIPS_CLASSW,
                PCWSTR::null(),
                WS_POPUP | (TTS_NOPREFIX | TTS_ALWAYSTIP) as u32,
                0,
                0,
                0,
                0,
                Some(owner),
                None,
                None,
                None,
            );
            if hwnd.0.is_null() {
                return Err(Error::from_thread());
            }
            let mut text = to_wide("");
            let mut ti = tool_info(owner, &mut text);
            SendMessageW(
                hwnd,
                TTM_ADDTOOLW as u32,
                WPARAM(0),
                LPARAM(&mut ti as *mut TTTOOLINFOW as isize),
            );
            // A max width turns "\n" into line breaks (multi-line infotip).
            let dpi = GetDpiForWindow(owner).max(96);
            SendMessageW(
                hwnd,
                TTM_SETMAXTIPWIDTH as u32,
                WPARAM(0),
                LPARAM((360 * dpi / 96) as isize),
            );
            if dark {
                let theme = to_wide("DarkMode_Explorer");
                let _ = SetWindowTheme(hwnd, PCWSTR(theme.as_ptr()), PCWSTR::null());
            }
            Ok(Self { hwnd, owner, text })
        }
    }

    /// Shows `text` with its top-left below the screen point (the control keeps it on-screen).
    pub fn show(&mut self, text: &str, sx: i32, sy: i32) {
        self.text = to_wide(text);
        // SAFETY: the text buffer lives in `self` for as long as the tip is shown.
        unsafe {
            let mut ti = tool_info(self.owner, &mut self.text);
            SendMessageW(
                self.hwnd,
                TTM_UPDATETIPTEXTW as u32,
                WPARAM(0),
                LPARAM(&mut ti as *mut TTTOOLINFOW as isize),
            );
            let pos = ((sy as u16 as usize) << 16) | (sx as u16 as usize);
            SendMessageW(
                self.hwnd,
                TTM_TRACKPOSITION as u32,
                WPARAM(0),
                LPARAM(pos as isize),
            );
            SendMessageW(
                self.hwnd,
                TTM_TRACKACTIVATE as u32,
                WPARAM(1),
                LPARAM(&mut ti as *mut TTTOOLINFOW as isize),
            );
        }
    }

    pub fn hide(&mut self) {
        // SAFETY: plain FFI call.
        unsafe {
            let mut ti = tool_info(self.owner, &mut self.text);
            SendMessageW(
                self.hwnd,
                TTM_TRACKACTIVATE as u32,
                WPARAM(0),
                LPARAM(&mut ti as *mut TTTOOLINFOW as isize),
            );
        }
    }

    /// `SPI_GETMOUSEHOVERTIME`: how long Explorer waits before an infotip (default 400 ms).
    pub fn hover_delay_ms() -> u32 {
        crate::sysparams::mouse_hover_time_ms()
    }

    /// comctl32 `TTDT_RESHOW` default: `GetDoubleClickTime() / 5` (100 ms) — the delay before a
    /// tip re-shows when the pointer moves onto a neighbouring tool while one was just up.
    pub fn reshow_delay_ms() -> u32 {
        (crate::sysparams::double_click_time_ms() / 5).clamp(50, 500)
    }
}

fn tool_info(owner: HWND, text: &mut [u16]) -> TTTOOLINFOW {
    TTTOOLINFOW {
        cbSize: size_of::<TTTOOLINFOW>() as u32,
        uFlags: TTF_TRACK as u32,
        hwnd: owner,
        uId: TOOL_ID,
        rect: RECT::default(),
        hinst: HINSTANCE::default(),
        lpszText: PWSTR(text.as_mut_ptr()),
        lParam: LPARAM(0),
        lpReserved: core::ptr::null_mut(),
    }
}

impl Drop for Tooltip {
    fn drop(&mut self) {
        // SAFETY: we created the window.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}
