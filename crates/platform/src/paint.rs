//! Small GDI painting for the native popups (the Peek search box, the filter box): solid and
//! rounded fills and one line of text between `BeginPaint` / `EndPaint`.

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
    /// `begin_buffered`: the window's DC, the memory bitmap `hdc` draws to and its size.
    buffer: Option<(HDC, HGDIOBJ, i32, i32)>,
}

windows_core::link!("gdi32.dll" "system" fn CreateCompatibleBitmap(hdc: HDC, w: i32, h: i32) -> *mut core::ffi::c_void);
windows_core::link!("gdi32.dll" "system" fn BitBlt(dst: HDC, x: i32, y: i32, w: i32, h: i32, src: HDC, sx: i32, sy: i32, rop: u32) -> windows_core::BOOL);
const SRCCOPY: u32 = 0x00CC_0020;

impl Paint {
    pub fn begin(hwnd: HWND) -> Self {
        let mut ps = PAINTSTRUCT::default();
        // SAFETY: `ps` lives in the returned value until `EndPaint` in drop.
        let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
        Self {
            hwnd,
            ps,
            hdc,
            buffer: None,
        }
    }

    /// Like [`Self::begin`], but everything is drawn to a memory bitmap of the client area that
    /// reaches the window in one step on drop: no half-drawn frame is ever on screen (a list
    /// whose rows take long to draw, a fill that changes every frame). Without the memory for
    /// the bitmap it draws straight to the window.
    pub fn begin_buffered(hwnd: HWND) -> Self {
        let mut p = Self::begin(hwnd);
        let mut rc = RECT::default();
        // SAFETY: plain GDI calls; the bitmap is selected out and deleted in drop, the memory
        // DC is deleted there too.
        unsafe {
            let _ = GetClientRect(hwnd, &mut rc);
            let mem = CreateCompatibleDC(Some(p.hdc));
            let bitmap = HGDIOBJ(CreateCompatibleBitmap(
                p.hdc,
                rc.right.max(1),
                rc.bottom.max(1),
            ));
            if mem.0.is_null() || bitmap.0.is_null() {
                let _ = DeleteObject(bitmap);
                let _ = DeleteDC(mem);
                return p;
            }
            SelectObject(mem, bitmap);
            p.buffer = Some((p.hdc, bitmap, rc.right, rc.bottom));
            p.hdc = mem;
        }
        p
    }

    /// A buffered paint that drew nothing (its owner was busy): the window keeps what it shows.
    pub fn keep_window(&mut self) {
        if let Some((window, bitmap, _, _)) = self.buffer.take() {
            // SAFETY: the memory DC and its bitmap are ours; deleting the DC frees the bitmap.
            unsafe {
                let _ = DeleteDC(self.hdc);
                let _ = DeleteObject(bitmap);
            }
            self.hdc = window;
        }
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

impl Paint {
    /// A rounded rectangle `r` (corner radius `radius` px) filled with `fill` and outlined 1 px
    /// in `border` (COLORREF 0x00BBGGRR).
    pub fn round_rect(&self, r: RECT, radius: i32, fill: u32, border: u32) {
        windows_core::link!("gdi32.dll" "system" fn CreatePen(style: i32, width: i32, color: u32) -> isize);
        windows_core::link!("gdi32.dll" "system" fn RoundRect(hdc: HDC, l: i32, t: i32, r: i32, b: i32, w: i32, h: i32) -> windows_core::BOOL);
        // SAFETY: plain GDI calls on the paint DC; the pen and brush are selected out and
        // deleted before returning.
        unsafe {
            let pen = HGDIOBJ(CreatePen(0, 1, border) as *mut core::ffi::c_void);
            let brush = CreateSolidBrush(COLORREF(fill));
            let old_pen = SelectObject(self.hdc, pen);
            let old_brush = SelectObject(self.hdc, HGDIOBJ(brush.0));
            let _ = RoundRect(
                self.hdc,
                r.left,
                r.top,
                r.right,
                r.bottom,
                radius * 2,
                radius * 2,
            );
            SelectObject(self.hdc, old_pen);
            SelectObject(self.hdc, old_brush);
            let _ = DeleteObject(pen);
            let _ = DeleteObject(HGDIOBJ(brush.0));
        }
    }

    /// Width in pixels of `text` in `font` (one line, no clipping).
    pub fn text_width(&self, text: &str, font: &Font) -> i32 {
        if text.is_empty() {
            return 0;
        }
        let wide: Vec<u16> = text.encode_utf16().collect();
        let mut size = SIZE::default();
        // SAFETY: plain GDI calls on the paint DC; the old font is selected back.
        unsafe {
            let old = SelectObject(self.hdc, HGDIOBJ(font.0.0));
            let _ = GetTextExtentPoint32W(self.hdc, &wide, &mut size);
            SelectObject(self.hdc, old);
        }
        size.cx
    }

    /// Draws `icon` with its top-left corner at (`x`, `y`), `size` pixels square.
    pub fn icon(&self, x: i32, y: i32, size: i32, icon: &crate::fileinfo::FileIcon) {
        // SAFETY: the icon handle is owned by `icon` and alive for the call.
        unsafe {
            let _ = DrawIconEx(
                self.hdc,
                x,
                y,
                icon.handle(),
                size,
                size,
                0,
                None,
                DI_NORMAL as u32,
            );
        }
    }
}

impl Drop for Paint {
    fn drop(&mut self) {
        // SAFETY: ends the paint begun in `begin` with the same structure; a buffered paint
        // copies its bitmap to the window first (deleting the memory DC frees the bitmap for
        // deletion).
        unsafe {
            if let Some((window, bitmap, w, h)) = self.buffer.take() {
                let _ = BitBlt(window, 0, 0, w, h, self.hdc, 0, 0, SRCCOPY);
                let _ = DeleteDC(self.hdc);
                let _ = DeleteObject(bitmap);
            }
            let _ = EndPaint(self.hwnd, &self.ps);
        }
    }
}

/// Asks for a repaint of `rect` (client pixels, background included).
pub fn invalidate_rect(hwnd: HWND, rect: RECT) {
    // SAFETY: plain FFI call; the rect outlives it.
    unsafe {
        let _ = InvalidateRect(Some(hwnd), Some(&rect), true);
    }
}

/// Asks for a repaint of the whole client area (background included).
pub fn invalidate(hwnd: HWND) {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, true);
    }
}
