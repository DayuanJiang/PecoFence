//! Desktop Window Manager attributes.

use crate::bindings::*;
use windows_core::Result;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CornerPreference {
    Default,
    DoNotRound,
    Round,
    RoundSmall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemBackdrop {
    Auto,
    None,
    MainWindow,
    TransientWindow,
    TabbedWindow,
}

windows_core::link!("kernel32.dll" "system" fn QueryPerformanceCounter(count: *mut i64) -> i32);
windows_core::link!("kernel32.dll" "system" fn QueryPerformanceFrequency(frequency: *mut i64) -> i32);

/// How long until Windows puts the windows on screen next, and how long one refresh takes
/// (ms), from the compositor's own clock. For log lines that say which refresh a draw makes:
/// what is drawn now is on screen at the next one. `None` when Windows reports no timing.
pub fn next_refresh_ms() -> Option<(f32, f32)> {
    let mut info = DWM_TIMING_INFO {
        cbSize: size_of::<DWM_TIMING_INFO>() as u32,
        ..Default::default()
    };
    let (mut now, mut freq) = (0i64, 0i64);
    // SAFETY: `info` is a live DWM_TIMING_INFO with its size set (a null window asks for the
    // desktop's timing); the two counters are written through live pointers.
    unsafe {
        if DwmGetCompositionTimingInfo(HWND::default(), &mut info).is_err() {
            return None;
        }
        QueryPerformanceCounter(&mut now);
        QueryPerformanceFrequency(&mut freq);
    }
    let period = info.qpcRefreshPeriod.0 as i64;
    if freq <= 0 || period <= 0 {
        return None;
    }
    let since = (now - info.qpcVBlank.0 as i64).rem_euclid(period);
    let ms = |ticks: i64| ticks as f32 * 1000.0 / freq as f32;
    Some((ms(period - since), ms(period)))
}

fn set_attribute<T>(hwnd: HWND, attribute: i32, value: &T) -> Result<()> {
    // SAFETY: `value` outlives the call and its size is passed explicitly.
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            attribute as u32,
            (value as *const T).cast(),
            size_of::<T>() as u32,
        )
        .ok()
    }
}

pub fn set_corner_preference(hwnd: HWND, preference: CornerPreference) -> Result<()> {
    let value: i32 = match preference {
        CornerPreference::Default => DWMWCP_DEFAULT,
        CornerPreference::DoNotRound => DWMWCP_DONOTROUND,
        CornerPreference::Round => DWMWCP_ROUND,
        CornerPreference::RoundSmall => DWMWCP_ROUNDSMALL,
    };
    set_attribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &value)
}

pub fn set_system_backdrop(hwnd: HWND, backdrop: SystemBackdrop) -> Result<()> {
    let value: i32 = match backdrop {
        SystemBackdrop::Auto => DWMSBT_AUTO,
        SystemBackdrop::None => DWMSBT_NONE,
        SystemBackdrop::MainWindow => DWMSBT_MAINWINDOW,
        SystemBackdrop::TransientWindow => DWMSBT_TRANSIENTWINDOW,
        SystemBackdrop::TabbedWindow => DWMSBT_TABBEDWINDOW,
    };
    set_attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &value)
}

/// Removes the 1px accent/gray border DWM draws around top-level windows.
pub fn set_border_color_none(hwnd: HWND) -> Result<()> {
    let value: u32 = DWMWA_COLOR_NONE;
    set_attribute(hwnd, DWMWA_BORDER_COLOR, &value)
}

/// Custom composition windows draw their own frame and shadow. Leaving the DWM frame
/// enabled exposes its smaller corner arcs outside a larger composition clip.
pub fn set_nonclient_rendering(hwnd: HWND, enabled: bool) -> Result<()> {
    let value = if enabled {
        DWMNCRP_ENABLED
    } else {
        DWMNCRP_DISABLED
    };
    set_attribute(hwnd, DWMWA_NCRENDERING_POLICY, &value)
}

/// Paints the caption in a solid colour (COLORREF, 0x00BBGGRR) so it matches an opaque page.
pub fn set_caption_color(hwnd: HWND, colorref: u32) -> Result<()> {
    set_attribute(hwnd, DWMWA_CAPTION_COLOR, &colorref)
}

/// The first build that knows the dark title bar attribute under its number of today (20), and
/// that repaints a title bar as soon as the attribute changes: Windows 10 2004's.
const DARK_TITLE_BAR_BUILD: u32 = 18985;

/// A dark title bar and frame. The builds before Windows 10 2004 (1809 to 1909) know the
/// attribute as 19, and the settings window kept a white title bar over its dark page there
/// (#219).
pub fn set_immersive_dark_mode(hwnd: HWND, dark: bool) -> Result<()> {
    const DWMWA_USE_IMMERSIVE_DARK_MODE_OLD: i32 = 19;
    let value: i32 = dark as i32;
    let attribute = if crate::desktop::os_build() < DARK_TITLE_BAR_BUILD {
        DWMWA_USE_IMMERSIVE_DARK_MODE_OLD
    } else {
        DWMWA_USE_IMMERSIVE_DARK_MODE
    };
    set_attribute(hwnd, attribute, &value)
}

/// For a window on screen whose mode just changed ([`set_immersive_dark_mode`]): the builds
/// before Windows 10 2004 paint the title bar in its new colour only when the window is
/// activated or deactivated next (measured on build 17763: a frame change or a frame redraw
/// does not do it). The frame is told both, ending with its real state.
pub fn repaint_title_bar(hwnd: HWND) {
    if crate::desktop::os_build() >= DARK_TITLE_BAR_BUILD {
        return;
    }
    let active = crate::desktop::foreground_window() == hwnd;
    crate::window::send_message(hwnd, WM_NCACTIVATE as u32, usize::from(!active), 0);
    crate::window::send_message(hwnd, WM_NCACTIVATE as u32, usize::from(active), 0);
}

/// Waits until Windows has put the windows on screen once more (`DwmFlush`: one frame at
/// most), so that a window shown and painted just now is there before what follows is drawn.
pub fn wait_for_frame() {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = DwmFlush();
    }
}

/// Disables DWM's minimize/restore transition animations for this window.
pub fn set_transitions_disabled(hwnd: HWND, disabled: bool) -> Result<()> {
    let value: i32 = disabled as i32;
    set_attribute(hwnd, DWMWA_TRANSITIONS_FORCEDISABLED, &value)
}
