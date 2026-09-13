//! User-tunable system parameters (`SystemParametersInfoW`) that govern timing and motion.
//!
//! Explorer reads the same values, so honouring them is what makes the fences feel native:
//! a user who turned animations off in Settings › Accessibility gets snaps instead of
//! transitions, and one who set the wheel to 5 lines gets 5 lines here too.

use crate::bindings::*;
use crate::wide::to_wide;
use windows_core::{BOOL, PCWSTR};

fn read_u32(spi: i32, default: u32) -> u32 {
    let mut value: u32 = default;
    // SAFETY: out-pointer to a local of the size the SPI expects (a UINT for every SPI used
    // in this module).
    let ok = unsafe {
        SystemParametersInfoW(
            spi as u32,
            0,
            &mut value as *mut u32 as *mut core::ffi::c_void,
            0,
        )
    };
    if ok.as_bool() { value } else { default }
}

fn read_bool(spi: i32, default: bool) -> bool {
    let mut value: BOOL = BOOL(default as i32);
    // SAFETY: out-pointer to a BOOL-sized local, as the SPI expects.
    let ok = unsafe {
        SystemParametersInfoW(
            spi as u32,
            0,
            &mut value as *mut BOOL as *mut core::ffi::c_void,
            0,
        )
    };
    if ok.as_bool() {
        value.as_bool()
    } else {
        default
    }
}

/// `SPI_GETCLIENTAREAANIMATION`: false when the user disabled animations
/// (Settings › Accessibility › Visual effects › Animation effects). Re-read on
/// `WM_SETTINGCHANGE`.
pub fn client_area_animation() -> bool {
    read_bool(SPI_GETCLIENTAREAANIMATION, true)
}

/// `SPI_GETWHEELSCROLLLINES`: rows per wheel notch (default 3; `WHEEL_PAGESCROLL` = a page).
pub fn wheel_scroll_lines() -> u32 {
    read_u32(SPI_GETWHEELSCROLLLINES, 3)
}

/// `SPI_GETMOUSEHOVERTIME`: hover delay before an infotip, in ms (default 400).
pub fn mouse_hover_time_ms() -> u32 {
    read_u32(SPI_GETMOUSEHOVERTIME, 400).clamp(100, 2000)
}

/// `SPI_GETMESSAGEDURATION`: how long a tooltip / notification stays, in seconds (default 5).
pub fn message_duration_secs() -> u32 {
    read_u32(SPI_GETMESSAGEDURATION, 5).clamp(1, 60)
}

/// `SPI_GETMOUSEHOVERWIDTH` / `SPI_GETMOUSEHOVERHEIGHT`: the rectangle (physical px, centred
/// on the pointer) the pointer must stay inside for `SPI_GETMOUSEHOVERTIME` before
/// `TrackMouseEvent(TME_HOVER)` fires — the infotip rest box (default 4 × 4).
pub fn mouse_hover_rect() -> (i32, i32) {
    (
        read_u32(SPI_GETMOUSEHOVERWIDTH, 4).clamp(4, 256) as i32,
        read_u32(SPI_GETMOUSEHOVERHEIGHT, 4).clamp(4, 256) as i32,
    )
}

/// Settings › Accessibility › Text size, in percent (100–225; 100 when unset). The slider
/// stores `HKCU\Software\Microsoft\Accessibility\TextScaleFactor` (WinRT
/// `UISettings.TextScaleFactor`); it does not rewrite the GDI icon-title `LOGFONT`.
pub fn text_scale_percent() -> u32 {
    let key = to_wide(r"Software\Microsoft\Accessibility");
    let value = to_wide("TextScaleFactor");
    let mut data: u32 = 0;
    let mut size: u32 = size_of::<u32>() as u32;
    // SAFETY: out-pointers reference locals; RRF_RT_REG_DWORD constrains the type.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(key.as_ptr()),
            PCWSTR(value.as_ptr()),
            RRF_RT_REG_DWORD as u32,
            None,
            Some((&mut data as *mut u32).cast()),
            Some(&mut size),
        )
    };
    if status.0 == 0 && data != 0 {
        data.clamp(100, 225)
    } else {
        100
    }
}

/// The font the desktop draws icon labels with: `SPI_GETICONTITLELOGFONT` read at 96 DPI so
/// `size_dip` is in DIPs, multiplied by the Accessibility text-size factor
/// (`text_scale_percent`, which the LOGFONT itself does not carry).
#[derive(Clone, Debug, PartialEq)]
pub struct IconTitleFont {
    pub family: String,
    /// Em size in DIPs (default Segoe UI 9 pt = 12).
    pub size_dip: f32,
    /// `LOGFONT` weight (400 regular, 700 bold).
    pub weight: i32,
}

/// `SPI_GETICONTITLELOGFONT` via `SystemParametersInfoForDpi(96)`, scaled by the text-size
/// factor; None when the call fails (callers keep their built-in 12 px default).
pub fn icon_title_font() -> Option<IconTitleFont> {
    let mut lf = LOGFONTW::default();
    // SAFETY: out-pointer to a LOGFONTW-sized local, which is what this SPI fills in.
    let ok = unsafe {
        SystemParametersInfoForDpi(
            SPI_GETICONTITLELOGFONT as u32,
            size_of::<LOGFONTW>() as u32,
            &mut lf as *mut LOGFONTW as *mut core::ffi::c_void,
            0,
            96,
        )
    };
    if !ok.as_bool() {
        return None;
    }
    let len = lf.lfFaceName.iter().position(|c| *c == 0).unwrap_or(32);
    let family = String::from_utf16_lossy(&lf.lfFaceName[..len]);
    if family.is_empty() {
        return None;
    }
    Some(IconTitleFont {
        family,
        size_dip: icon_title_size_dip(lf.lfHeight, text_scale_percent()),
        weight: if lf.lfWeight <= 0 { 400 } else { lf.lfWeight },
    })
}

/// `lfHeight` is negative for em sizes (the usual case), positive for cell heights, 0 for the
/// default; all become a DIP em size, scaled by the text-size percentage and clamped to
/// something drawable.
fn icon_title_size_dip(lf_height: i32, text_scale_percent: u32) -> f32 {
    let size = match lf_height {
        0 => 12.0,
        h => h.unsigned_abs() as f32,
    };
    (size * text_scale_percent as f32 / 100.0).clamp(8.0, 40.0)
}

/// `GetDoubleClickTime` in ms.
pub fn double_click_time_ms() -> u32 {
    // SAFETY: no arguments, no failure mode.
    unsafe { GetDoubleClickTime() }
}

#[cfg(test)]
mod tests {
    use super::icon_title_size_dip;

    #[test]
    fn icon_title_height_maps_to_dip_em_size() {
        assert_eq!(icon_title_size_dip(-12, 100), 12.0);
        assert_eq!(icon_title_size_dip(-15, 100), 15.0);
        assert_eq!(icon_title_size_dip(16, 100), 16.0);
        assert_eq!(icon_title_size_dip(0, 100), 12.0);
        assert_eq!(icon_title_size_dip(-2, 100), 8.0);
        assert_eq!(icon_title_size_dip(-99, 100), 40.0);
    }

    #[test]
    fn text_scale_percent_scales_the_em_size() {
        assert_eq!(icon_title_size_dip(-12, 125), 15.0);
        assert_eq!(icon_title_size_dip(-12, 150), 18.0);
        assert_eq!(icon_title_size_dip(-12, 225), 27.0);
        assert_eq!(icon_title_size_dip(-20, 225), 40.0);
    }
}
