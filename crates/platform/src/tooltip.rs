//! A tracking tooltip (`tooltips_class32`, `TTF_TRACK`) for a never-activated window: shown at
//! an explicit screen point, hidden by the owner. Explorer uses the same control for its icon
//! infotips.

use crate::bindings::*;
use crate::wide::to_wide;
use windows_core::{Error, PCWSTR, PWSTR, Result};

const TOOL_ID: usize = 1;
// Not in the generated bindings.
/// The tip stands exactly where `TTM_TRACKPOSITION` says.
const TTF_ABSOLUTE: u32 = 0x0080;
/// Where a tip comes up to be measured: left of and above every monitor.
const OFF_SCREEN: i32 = -30000;

/// Top-left of a tip `size` px large asked for at `at`, inside the work area `work` (left, top,
/// right, bottom): moved left when it would cross the right edge, and with no room below its
/// bottom edge goes to `above`.
pub fn place_tip(
    at: (i32, i32),
    above: i32,
    size: (i32, i32),
    work: (i32, i32, i32, i32),
) -> (i32, i32) {
    let x = at.0.min(work.2 - size.0).max(work.0);
    let y = if at.1 + size.1 > work.3 {
        (above - size.1).max(work.1)
    } else {
        at.1
    };
    (x, y)
}

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

    /// Shows `text` with its top-left at the screen point (`sx`, `sy`), moved left when it would
    /// cross the right edge of the work area there. A tip with no room below goes up instead:
    /// its bottom edge to `above` (for a tip beside the pointer, just above the pointer). Left
    /// to itself the control puts such a tip above the window that owns it, a whole fence away
    /// from the icon it describes.
    pub fn show(&mut self, text: &str, sx: i32, sy: i32, above: i32) {
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
            // The control knows the size of a tip only once it is up: the tip comes up off
            // screen, is measured there, and then goes to its place.
            let hwnd = self.hwnd;
            let track = |x: i32, y: i32| {
                let pos = ((y as u16 as usize) << 16) | (x as u16 as usize);
                SendMessageW(
                    hwnd,
                    TTM_TRACKPOSITION as u32,
                    WPARAM(0),
                    LPARAM(pos as isize),
                );
            };
            track(OFF_SCREEN, OFF_SCREEN);
            SendMessageW(
                self.hwnd,
                TTM_TRACKACTIVATE as u32,
                WPARAM(1),
                LPARAM(&mut ti as *mut TTTOOLINFOW as isize),
            );
            let mut r = RECT::default();
            let _ = GetWindowRect(self.hwnd, &mut r);
            let size = (r.right - r.left, r.bottom - r.top);
            let (x, y) = match crate::monitors::query(crate::monitors::monitor_from_point(sx, sy)) {
                Some(m) => {
                    let w = m.work_area;
                    place_tip((sx, sy), above, size, (w.left, w.top, w.right, w.bottom))
                }
                None => (sx, sy),
            };
            if (x, y) != (sx, sy) {
                tracing::debug!(
                    sx,
                    sy,
                    x,
                    y,
                    w = size.0,
                    h = size.1,
                    "tip moved to stay on screen"
                );
            }
            track(x, y);
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

/// `TTTOOLINFOW_V2_SIZE`. The manifest asks for no Common Controls 6, so the app gets comctl32
/// 5, whose `TTM_ADDTOOLW` refuses the full struct (`lpReserved` came with version 6): the tool
/// was never added and no tip ever showed.
const TOOL_INFO_SIZE: u32 = std::mem::offset_of!(TTTOOLINFOW, lpReserved) as u32;

fn tool_info(owner: HWND, text: &mut [u16]) -> TTTOOLINFOW {
    TTTOOLINFOW {
        cbSize: TOOL_INFO_SIZE,
        uFlags: TTF_TRACK as u32 | TTF_ABSOLUTE,
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

#[cfg(test)]
mod tests {
    /// A tip with room stays where it was asked for; at the right edge it moves left; with no
    /// room below it goes just above the pointer, not above the window that owns it.
    #[test]
    fn a_tip_without_room_below_goes_just_above_the_pointer() {
        let work = (0, 0, 1600, 852);
        // Pointer at (700, 400): the tip 12 px right and 20 px below it.
        assert_eq!(
            super::place_tip((712, 420), 396, (240, 66), work),
            (712, 420)
        );
        assert_eq!(
            super::place_tip((1500, 420), 396, (240, 66), work),
            (1360, 420)
        );
        // Pointer at (700, 814), in the bottom row of a fence that ends above the taskbar.
        assert_eq!(
            super::place_tip((712, 834), 810, (240, 66), work),
            (712, 744)
        );
        // A tip taller than the room above the pointer starts at the top of the work area.
        assert_eq!(super::place_tip((712, 834), 40, (240, 66), work), (712, 0));
        // A second monitor left of the first.
        assert_eq!(
            super::place_tip((-100, 300), 276, (240, 66), (-1920, 0, 0, 1040)),
            (-240, 300)
        );
    }

    /// comctl32 5 takes TTTOOLINFOW_V2_SIZE: everything up to `lParam`, 64 bytes on x64.
    #[test]
    fn tool_info_has_the_comctl32_5_size() {
        assert_eq!(
            super::TOOL_INFO_SIZE as usize,
            size_of::<super::TTTOOLINFOW>() - size_of::<usize>()
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!(super::TOOL_INFO_SIZE, 64);
    }
}
