//! Font availability. Windows 11 ships Segoe UI Variable and Segoe Fluent Icons; Windows 10
//! has Segoe UI and Segoe MDL2 Assets instead. Each face is resolved once per process.

use std::sync::OnceLock;

use crate::bindings::*;
use crate::wide::{from_wide, to_wide};
use windows_core::{PCWSTR, PWSTR};

/// True when GDI resolves `face` to a font of that exact family name (the font mapper
/// silently substitutes another family when the requested one is not installed).
pub fn font_installed(face: &str) -> bool {
    let wide = to_wide(face);
    // SAFETY: every GDI object created here is released before returning.
    unsafe {
        let dc = CreateCompatibleDC(None);
        if dc.0.is_null() {
            return false;
        }
        let font = CreateFontW(
            -12,
            0,
            0,
            0,
            FW_NORMAL,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            ANTIALIASED_QUALITY as u32,
            (DEFAULT_PITCH | FF_DONTCARE) as u32,
            PCWSTR(wide.as_ptr()),
        );
        let mut installed = false;
        if !font.0.is_null() {
            let old = SelectObject(dc, HGDIOBJ(font.0));
            let mut buf = [0u16; 64];
            let len = GetTextFaceW(dc, buf.len() as i32, Some(PWSTR(buf.as_mut_ptr())));
            if len > 0 {
                let actual = from_wide(&buf[..(len as usize).min(buf.len())]);
                installed = actual.trim_end_matches('\0').eq_ignore_ascii_case(face);
            }
            SelectObject(dc, old);
            let _ = DeleteObject(HGDIOBJ(font.0));
        }
        let _ = DeleteDC(dc);
        installed
    }
}

fn pick(
    cell: &'static OnceLock<&'static str>,
    preferred: &'static str,
    fallback: &'static str,
) -> &'static str {
    cell.get_or_init(|| {
        if font_installed(preferred) {
            preferred
        } else {
            tracing::info!(preferred, fallback, "font not installed; using fallback");
            fallback
        }
    })
}

/// Body text: Segoe UI Variable Text, or Segoe UI on Windows 10.
pub fn text_face() -> &'static str {
    static FACE: OnceLock<&'static str> = OnceLock::new();
    pick(&FACE, "Segoe UI Variable Text", "Segoe UI")
}

/// Captions and labels: Segoe UI Variable Small, or Segoe UI on Windows 10.
pub fn small_face() -> &'static str {
    static FACE: OnceLock<&'static str> = OnceLock::new();
    pick(&FACE, "Segoe UI Variable Small", "Segoe UI")
}

/// Headings: Segoe UI Variable Display, or Segoe UI on Windows 10.
pub fn display_face() -> &'static str {
    static FACE: OnceLock<&'static str> = OnceLock::new();
    pick(&FACE, "Segoe UI Variable Display", "Segoe UI")
}

/// Glyph font: Segoe Fluent Icons, or Segoe MDL2 Assets on Windows 10. The two fonts share
/// their code points for everything this app draws except `Hide` (see `fluent_icons`).
pub fn icon_face() -> &'static str {
    static FACE: OnceLock<&'static str> = OnceLock::new();
    pick(&FACE, "Segoe Fluent Icons", "Segoe MDL2 Assets")
}

/// True when the glyph font is Segoe Fluent Icons rather than the MDL2 fallback.
pub fn fluent_icons() -> bool {
    icon_face() == "Segoe Fluent Icons"
}
