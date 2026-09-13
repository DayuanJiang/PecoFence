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

pub fn set_immersive_dark_mode(hwnd: HWND, dark: bool) -> Result<()> {
    let value: i32 = dark as i32;
    set_attribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &value)
}

/// Disables DWM's minimize/restore transition animations for this window.
pub fn set_transitions_disabled(hwnd: HWND, disabled: bool) -> Result<()> {
    let value: i32 = disabled as i32;
    set_attribute(hwnd, DWMWA_TRANSITIONS_FORCEDISABLED, &value)
}

/// Excludes the window from Aero Peek / "Show desktop" preview.
pub fn set_excluded_from_peek(hwnd: HWND, excluded: bool) -> Result<()> {
    let value: i32 = excluded as i32;
    set_attribute(hwnd, DWMWA_EXCLUDED_FROM_PEEK, &value)
}

/// Extends the DWM frame into the whole client area (sheet-of-glass).
pub fn extend_frame_into_entire_client_area(hwnd: HWND) -> Result<()> {
    let margins = MARGINS {
        cxLeftWidth: -1,
        cxRightWidth: -1,
        cyTopHeight: -1,
        cyBottomHeight: -1,
    };
    // SAFETY: margins outlives the call.
    unsafe { DwmExtendFrameIntoClientArea(hwnd, &margins).ok() }
}

pub fn is_composition_enabled() -> bool {
    // SAFETY: plain FFI call.
    unsafe {
        DwmIsCompositionEnabled()
            .map(|b| b.as_bool())
            .unwrap_or(false)
    }
}
