//! Font availability. Windows 11 ships Segoe UI Variable and Segoe Fluent Icons; Windows 10
//! has Segoe UI and Segoe MDL2 Assets instead. Each face is resolved once per process.

use std::sync::OnceLock;

use crate::bindings::*;
use crate::wide::{from_wide, to_wide};
use windows_core::{PCWSTR, PWSTR};

/// True when GDI resolves `face` to a font of that exact family name (the font mapper
/// silently substitutes another family when the requested one is not installed).
pub fn font_installed(face: &str) -> bool {
    resolved_face(face).is_some_and(|actual| actual.eq_ignore_ascii_case(face))
}

/// Whether a font family is installed under the name `face`, in any language: GDI reports
/// Microsoft YaHei as 微软雅黑 on a Chinese system. A name no font has gets the same
/// substitute as a made-up one.
pub fn font_family_exists(face: &str) -> bool {
    match resolved_face(face) {
        Some(actual) => {
            actual.eq_ignore_ascii_case(face)
                || resolved_face("PecoFence Missing Font").is_some_and(|other| other != actual)
        }
        None => false,
    }
}

/// The family GDI picks for a font named `face`.
fn resolved_face(face: &str) -> Option<String> {
    with_font(face, |dc| {
        let mut buf = [0u16; 64];
        // SAFETY: `buf` holds the 64 characters its length says.
        let len = unsafe { GetTextFaceW(dc, buf.len() as i32, Some(PWSTR(buf.as_mut_ptr()))) };
        (len > 0).then(|| {
            let actual = from_wide(&buf[..(len as usize).min(buf.len())]);
            actual.trim_end_matches('\0').to_string()
        })
    })
}

/// Runs `read` with a memory DC that has the font GDI picks for `face` selected.
fn with_font<R>(face: &str, read: impl FnOnce(HDC) -> Option<R>) -> Option<R> {
    let wide = to_wide(face);
    // SAFETY: every GDI object created here is released before returning.
    unsafe {
        let dc = CreateCompatibleDC(None);
        if dc.0.is_null() {
            return None;
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
        let mut result = None;
        if !font.0.is_null() {
            let old = SelectObject(dc, HGDIOBJ(font.0));
            result = read(dc);
            SelectObject(dc, old);
            let _ = DeleteObject(HGDIOBJ(font.0));
        }
        let _ = DeleteDC(dc);
        result
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

/// Han text drawn with GDI (the Peek search list) as the fence titles draw it with DirectWrite:
/// the face `pecofence_core::i18n::han_for` gives the UI language (GDI's own fallback on an
/// English Windows drew 下载 in Japanese or Traditional forms). Segoe UI when it is missing.
pub fn han_text_face() -> &'static str {
    static FACE: OnceLock<&'static str> = OnceLock::new();
    let (_, preferred) = pecofence_core::i18n::han_for(crate::locale::ui_language());
    pick(&FACE, preferred, text_face())
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

/// Whether the glyph font has `glyph`. Segoe MDL2 Assets grew with the Windows 10 releases:
/// the one of version 1809 (font version 1.68) lacks code points the later ones have, and a
/// glyph a font lacks is drawn as an empty box. True when GDI cannot tell.
pub fn icon_font_has(glyph: char) -> bool {
    windows_core::link!("gdi32.dll" "system" fn GetGlyphIndicesW(hdc: HDC, text: PCWSTR, count: i32, indices: *mut u16, flags: u32) -> u32);
    /// The index of a character the font lacks is 0xFFFF.
    const GGI_MARK_NONEXISTING_GLYPHS: u32 = 1;
    let mut buf = [0u16; 2];
    let text = glyph.encode_utf16(&mut buf);
    if text.len() != 1 {
        return true;
    }
    with_font(icon_face(), |dc| {
        let mut index = 0u16;
        // SAFETY: one character in, room for one glyph index out.
        let found = unsafe {
            GetGlyphIndicesW(
                dc,
                PCWSTR(text.as_ptr()),
                1,
                &mut index,
                GGI_MARK_NONEXISTING_GLYPHS,
            )
        };
        (found == 1).then_some(index != 0xFFFF)
    })
    .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn families_are_found_by_any_name_and_missing_ones_are_not() {
        assert!(font_family_exists("Arial"));
        // Ships with every Windows 10 and 11; GDI calls it 微软雅黑 on a Chinese system.
        assert!(font_family_exists("Microsoft YaHei"));
        assert!(!font_family_exists("PecoFence Font That Does Not Exist"));
        assert!(!font_installed("PecoFence Font That Does Not Exist"));
    }

    #[test]
    fn the_icon_font_knows_which_glyphs_it_has() {
        // Setting (the gear) is in every version of both icon fonts; U+F8FF, the last
        // private-use code point, is in none.
        assert!(icon_font_has('\u{E713}'));
        assert!(!icon_font_has('\u{F8FF}'));
    }
}
