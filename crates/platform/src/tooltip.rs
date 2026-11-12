//! A tracking tooltip (`tooltips_class32`, `TTF_TRACK`) for a never-activated window: shown at
//! an explicit screen point, hidden by the owner. Explorer uses the same control for its icon
//! infotips.

use crate::bindings::*;
use crate::wide::to_wide;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use windows_core::{Error, PCWSTR, PWSTR, Result};

const TOOL_ID: usize = 1;
/// How long a tip takes to appear and to leave, linearly: the durations of the app's motion
/// (`pecofence_render::motion::FAST` and `NORMAL`) nearest to Windows' own tips. The tip of a
/// desktop icon in Explorer, recorded at 30 frames a second on Windows Server 2025
/// (2026-11-12), comes over about 205 ms and leaves over about 275 ms. comctl32 fades in only
/// the tips it shows by itself: a tracking tip, which the owner shows, was there at full
/// strength in one frame (it left with comctl32's own fade, also with Windows' animations off).
const FADE_IN: Duration = Duration::from_millis(167);
const FADE_OUT: Duration = Duration::from_millis(250);
/// The fade's timer: a step about every frame.
const FADE_TICK_MS: u32 = 15;
// Not in the generated bindings: the control does no fade or slide of its own.
const TTS_NOANIMATE: u32 = 0x10;
const TTS_NOFADE: u32 = 0x20;

/// A fade of one tip window in flight, by its timer's id.
struct Fade {
    tip: HWND,
    owner: HWND,
    start: Instant,
    from: u8,
    to: u8,
}

impl Fade {
    fn alpha(&self, now: Instant) -> u8 {
        fade_alpha(
            self.from,
            self.to,
            now.saturating_duration_since(self.start),
        )
    }
}

/// How long the fade from `from` to `to` takes: the part of the whole fade (in or out) that
/// is left to go, so a tip that comes back half-way takes half the time.
fn fade_duration(from: u8, to: u8) -> Duration {
    let whole = if to > from { FADE_IN } else { FADE_OUT };
    whole.mul_f32(f32::from(from.abs_diff(to)) / 255.0)
}

thread_local! {
    static FADES: RefCell<HashMap<usize, Fade>> = RefCell::new(HashMap::new());
}

/// The alpha of a fade from `from` to `to` after `elapsed` (linear over `fade_duration`).
fn fade_alpha(from: u8, to: u8, elapsed: Duration) -> u8 {
    let dur = fade_duration(from, to).as_secs_f32();
    let p = if dur > 0.0 {
        (elapsed.as_secs_f32() / dur).min(1.0)
    } else {
        1.0
    };
    (f32::from(from) + (f32::from(to) - f32::from(from)) * p).round() as u8
}

/// One step of a tip's fade. At the end of a fade-out the tip is taken down and gets its full
/// strength back for the next show.
unsafe extern "system" fn fade_tick(_hwnd: HWND, _msg: u32, id: usize, _time: u32) {
    let step = FADES.with(|f| {
        let mut fades = f.borrow_mut();
        let fade = fades.get(&id)?;
        let alpha = fade.alpha(Instant::now());
        let step = (fade.tip, fade.owner, alpha, alpha == fade.to, fade.to == 0);
        if step.3 {
            fades.remove(&id);
        }
        Some(step)
    });
    let Some((tip, owner, alpha, done, hides)) = step else {
        // SAFETY: plain FFI call; a timer nobody knows is stopped.
        unsafe {
            let _ = KillTimer(None, id);
        }
        return;
    };
    // SAFETY: plain FFI calls on the tip window; the tool struct outlives the SendMessage.
    unsafe {
        let _ = SetLayeredWindowAttributes(tip, COLORREF(0), alpha, LWA_ALPHA as u32);
        if done {
            let _ = KillTimer(None, id);
            if hides {
                track_activate(tip, owner, false);
                let _ = SetLayeredWindowAttributes(tip, COLORREF(0), 255, LWA_ALPHA as u32);
            }
        }
    }
}

/// `TTM_TRACKACTIVATE`: puts the tip up or takes it down.
unsafe fn track_activate(tip: HWND, owner: HWND, on: bool) {
    let mut none = [0u16];
    let mut ti = tool_info(owner, &mut none);
    // SAFETY: the tool struct outlives the SendMessage; the control reads only the tool's
    // window and id for this message.
    unsafe {
        SendMessageW(
            tip,
            TTM_TRACKACTIVATE as u32,
            WPARAM(on as usize),
            LPARAM(&mut ti as *mut TTTOOLINFOW as isize),
        );
    }
}
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
    /// The timer of a fade in flight (`FADES`), 0 without one.
    fade_timer: usize,
    /// The tip is up (also while it fades out).
    up: bool,
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
                // Layered: the tip fades in and out by its alpha (`fade_to`).
                (WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED) as u32,
                TOOLTIPS_CLASSW,
                PCWSTR::null(),
                WS_POPUP | (TTS_NOPREFIX | TTS_ALWAYSTIP) as u32 | TTS_NOANIMATE | TTS_NOFADE,
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
            let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA as u32);
            Ok(Self {
                hwnd,
                owner,
                text,
                fade_timer: 0,
                up: false,
            })
        }
    }

    /// Stops a fade in flight and returns the alpha it had reached (None without one).
    fn stop_fade(&mut self) -> Option<u8> {
        let id = std::mem::take(&mut self.fade_timer);
        // `try_with`: a tip dropped while the thread ends finds the list gone already.
        let fade = FADES
            .try_with(|f| f.borrow_mut().remove(&id))
            .ok()
            .flatten()?;
        // SAFETY: plain FFI call.
        unsafe {
            let _ = KillTimer(None, id);
        }
        Some(fade.alpha(Instant::now()))
    }

    /// Fades the tip from `from` to full strength (`to` 255) or away (`to` 0, taken down at
    /// the end). Without Windows' animations, or when no timer can be had, it is there or gone
    /// at once.
    fn fade_to(&mut self, from: u8, to: u8) {
        // SAFETY: plain FFI calls on our own window.
        unsafe {
            let id = if crate::sysparams::client_area_animation() && from != to {
                SetTimer(None, 0, FADE_TICK_MS, Some(fade_tick))
            } else {
                0
            };
            if id == 0 {
                if to == 0 {
                    track_activate(self.hwnd, self.owner, false);
                }
                let _ = SetLayeredWindowAttributes(self.hwnd, COLORREF(0), 255, LWA_ALPHA as u32);
                return;
            }
            let _ = SetLayeredWindowAttributes(self.hwnd, COLORREF(0), from, LWA_ALPHA as u32);
            let fade = Fade {
                tip: self.hwnd,
                owner: self.owner,
                start: Instant::now(),
                from,
                to,
            };
            FADES.with(|f| f.borrow_mut().insert(id, fade));
            self.fade_timer = id;
        }
    }

    /// Shows `text` with its top-left at the screen point (`sx`, `sy`), moved left when it would
    /// cross the right edge of the work area there. A tip with no room below goes up instead:
    /// its bottom edge to `above` (for a tip beside the pointer, just above the pointer). Left
    /// to itself the control puts such a tip above the window that owns it, a whole fence away
    /// from the icon it describes.
    pub fn show(&mut self, text: &str, sx: i32, sy: i32, above: i32) {
        // It fades in over 167 ms (at once without Windows' animations): from nothing, or
        // from where a fade in flight had it (a tip that was leaving, or still coming).
        let from = match self.stop_fade() {
            Some(alpha) => alpha,
            None if self.up => 255,
            None => 0,
        };
        self.text = to_wide(text);
        // SAFETY: the text buffer lives in `self` for as long as the tip is shown.
        unsafe {
            if from < 255 && crate::sysparams::client_area_animation() {
                // Before the tip comes up to be measured: not one frame at full strength.
                let _ = SetLayeredWindowAttributes(self.hwnd, COLORREF(0), from, LWA_ALPHA as u32);
            }
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
        self.up = true;
        self.fade_to(from, 255);
    }

    /// Takes the tip down: it fades out over 250 ms (at once without Windows' animations).
    pub fn hide(&mut self) {
        // Not up, or already on its way out (that fade goes on).
        if !std::mem::take(&mut self.up) {
            return;
        }
        let from = self.stop_fade().unwrap_or(255);
        self.fade_to(from, 0);
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
        self.stop_fade();
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

    /// A tip comes over 167 ms and leaves over 250 ms, linearly, and from wherever it was.
    #[test]
    fn a_tip_fades_in_over_167_ms_and_out_over_250_ms() {
        use std::time::Duration;
        let ms = Duration::from_millis;
        assert_eq!(super::fade_alpha(0, 255, ms(0)), 0);
        assert_eq!(super::fade_alpha(0, 255, ms(167)), 255);
        assert_eq!(super::fade_alpha(0, 255, ms(500)), 255);
        let half = super::fade_alpha(0, 255, ms(83));
        assert!((120..=135).contains(&half), "half way after 83 ms: {half}");
        assert_eq!(super::fade_alpha(255, 0, ms(250)), 0);
        let half = super::fade_alpha(255, 0, ms(125));
        assert!(
            (120..=135).contains(&half),
            "half gone after 125 ms: {half}"
        );
        // A tip that was leaving comes back from where it was, in the part of the time left.
        assert_eq!(super::fade_alpha(100, 255, ms(0)), 100);
        assert_eq!(super::fade_alpha(128, 255, ms(84)), 255);
        assert_eq!(super::fade_alpha(100, 100, ms(10)), 100);
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
