//! System theme (light/dark) detection from the Personalize registry key, and the user's
//! accent palette (Settings › Personalization › Colours) from Explorer's Accent key.

use crate::bindings::*;
use crate::wide::to_wide;
use windows_core::PCWSTR;

const PERSONALIZE: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const ACCENT: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Accent";
/// `RRF_RT_REG_BINARY` (not in the generated bindings).
const RRF_RT_REG_BINARY: u32 = 0x08;

/// The eight-shade accent palette Windows derives from the chosen accent colour (the same
/// values `UISettings.GetColorValue` returns as `AccentLight3 .. AccentDark3`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccentPalette {
    pub light3: [u8; 3],
    pub light2: [u8; 3],
    pub light1: [u8; 3],
    pub accent: [u8; 3],
    pub dark1: [u8; 3],
    pub dark2: [u8; 3],
    pub dark3: [u8; 3],
}

impl AccentPalette {
    /// Parses the `AccentPalette` REG_BINARY value: 8 × `[R, G, B, 0x00]` in the order Light3,
    /// Light2, Light1, Accent, Dark1, Dark2, Dark3, (spare).
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 28 {
            return None;
        }
        let rgb = |i: usize| [bytes[i * 4], bytes[i * 4 + 1], bytes[i * 4 + 2]];
        Some(Self {
            light3: rgb(0),
            light2: rgb(1),
            light1: rgb(2),
            accent: rgb(3),
            dark1: rgb(4),
            dark2: rgb(5),
            dark3: rgb(6),
        })
    }
}

/// The user's current accent palette, or `None` when the key is unreadable (the caller keeps
/// its built-in default look). Re-read on `WM_SETTINGCHANGE` / `WM_DWMCOLORIZATIONCOLORCHANGED`.
pub fn accent_palette() -> Option<AccentPalette> {
    let key = to_wide(ACCENT);
    let value = to_wide("AccentPalette");
    let mut data = [0u8; 64];
    let mut size: u32 = data.len() as u32;
    // SAFETY: out-pointers reference locals; `size` bounds the buffer and RRF_RT_REG_BINARY
    // constrains the type.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(key.as_ptr()),
            PCWSTR(value.as_ptr()),
            RRF_RT_REG_BINARY,
            None,
            Some(data.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    if status.0 != 0 {
        return None;
    }
    AccentPalette::from_bytes(&data[..(size as usize).min(data.len())])
}

fn read_dword(value: &str) -> Option<u32> {
    let key = to_wide(PERSONALIZE);
    let value = to_wide(value);
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
    (status.0 == 0).then_some(data)
}

/// `SystemUsesLightTheme` — what the taskbar / Start use. Defaults to dark when unreadable.
pub fn system_uses_light_theme() -> bool {
    read_dword("SystemUsesLightTheme").unwrap_or(0) != 0
}

/// `AppsUseLightTheme` — what applications use. Defaults to light when unreadable.
pub fn apps_use_light_theme() -> bool {
    read_dword("AppsUseLightTheme").unwrap_or(1) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accent_palette_layout() {
        // Windows 11 default palette as stored in the registry.
        let hex = "99EBFF004CC2FF000091F8000078D4000067C000003E9200001A6800F7630C00";
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap())
            .collect();
        let p = AccentPalette::from_bytes(&bytes).unwrap();
        assert_eq!(p.light2, [0x4C, 0xC2, 0xFF]);
        assert_eq!(p.accent, [0x00, 0x78, 0xD4]);
        assert_eq!(p.dark1, [0x00, 0x67, 0xC0]);
        assert_eq!(p.dark3, [0x00, 0x1A, 0x68]);
        assert!(AccentPalette::from_bytes(&bytes[..20]).is_none());
    }
}
