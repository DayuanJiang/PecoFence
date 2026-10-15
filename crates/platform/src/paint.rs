//! Small GDI painting for the native popups (the Peek search box): solid fills and one line of
//! text between `BeginPaint` / `EndPaint`.

use crate::bindings::*;
use crate::wide::to_wide;
use windows_core::PCWSTR;

/// A GDI font, deleted on drop.
pub struct Font(HFONT);

impl Font {
    /// `face` at an em height of `size_dip` DIPs for `dpi`.
    pub fn new(face: &str, size_dip: f32, dpi: u32) -> Self {
        let face = to_wide(face);
        // SAFETY: the face buffer outlives the call.
        let font = unsafe {
            CreateFontW(
                -(size_dip * dpi as f32 / 96.0).round() as i32,
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
                CLEARTYPE_QUALITY as u32,
                (DEFAULT_PITCH | FF_DONTCARE) as u32,
                PCWSTR(face.as_ptr()),
            )
        };
        Self(font)
    }
}

impl Drop for Font {
    fn drop(&mut self) {
        // SAFETY: we own the font; it is not selected into any DC once painting ended.
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.0.0));
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// One `WM_PAINT`: begun on creation, ended on drop.
pub struct Paint {
    hwnd: HWND,
    ps: PAINTSTRUCT,
    hdc: HDC,
}

impl Paint {
    pub fn begin(hwnd: HWND) -> Self {
        let mut ps = PAINTSTRUCT::default();
        // SAFETY: `ps` lives in the returned value until `EndPaint` in drop.
        let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
        Self { hwnd, ps, hdc }
    }

    /// Fills `r` with `color` (COLORREF 0x00BBGGRR).
    pub fn fill(&self, r: RECT, color: u32) {
        // SAFETY: plain GDI calls on the paint DC; the brush is deleted right after use.
        unsafe {
            let brush = CreateSolidBrush(COLORREF(color));
            FillRect(self.hdc, &r, brush);
            let _ = DeleteObject(HGDIOBJ(brush.0));
        }
    }

    /// One line of `text` in `r`, vertically centred, cut with an ellipsis when too long.
    pub fn text(&self, r: RECT, text: &str, font: &Font, color: u32, align: Align) {
        // DrawTextW reads the buffer even for a zero length: never hand it an empty one.
        if text.is_empty() {
            return;
        }
        let wide: Vec<u16> = text.encode_utf16().collect();
        let mut r = r;
        let align = match align {
            Align::Left => DT_LEFT,
            Align::Center => DT_CENTER,
            Align::Right => DT_RIGHT,
        };
        // SAFETY: plain GDI calls on the paint DC; the old font is selected back before
        // returning, and the text buffer outlives the call.
        unsafe {
            let old = SelectObject(self.hdc, HGDIOBJ(font.0.0));
            SetBkMode(self.hdc, TRANSPARENT);
            let _ = SetTextColor(self.hdc, COLORREF(color));
            DrawTextW(
                self.hdc,
                PCWSTR(wide.as_ptr()),
                wide.len() as i32,
                &mut r,
                (align | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS) as u32,
            );
            SelectObject(self.hdc, old);
        }
    }
}

impl Drop for Paint {
    fn drop(&mut self) {
        // SAFETY: ends the paint begun in `begin` with the same structure.
        unsafe {
            let _ = EndPaint(self.hwnd, &self.ps);
        }
    }
}

/// Asks for a repaint of the whole client area (background included).
pub fn invalidate(hwnd: HWND) {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, true);
    }
}
