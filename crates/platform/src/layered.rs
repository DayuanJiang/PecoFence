//! Per-pixel-alpha layered windows (`UpdateLayeredWindow`), used for the fence shadow.

use crate::bindings::*;
use windows_core::{Error, Result};

/// A premultiplied BGRA image in a GDI DIB section (selected into its own memory DC), ready
/// to present to a `WS_EX_LAYERED` window. After presenting, the window keeps the content:
/// drop the image and use [`set_alpha`] for fades. The GDI objects are released on drop.
pub struct LayeredImage {
    mem: HDC,
    dib: HBITMAP,
    old: HGDIOBJ,
    /// The DIB section's pixels (top-down BGRA), alive as long as `dib`.
    bits: *mut u8,
    /// Size of the DIB section.
    width: i32,
    height: i32,
    /// The rows of it, from the top, that hold the image.
    rows: i32,
}

impl LayeredImage {
    /// Uploads a `w` x `h` premultiplied BGRA image (`bgra.len()` must be `w * h * 4`).
    pub fn new(w: i32, h: i32, bgra: &[u8]) -> Result<Self> {
        Self::with_room(w, h, h, bgra)
    }

    /// [`Self::new`] in a DIB section `room_h` rows tall (at least `h`): later images of the
    /// same width and up to that height go into the same section ([`Self::refill`]).
    pub fn with_room(w: i32, h: i32, room_h: i32, bgra: &[u8]) -> Result<Self> {
        if w <= 0 || h <= 0 || bgra.len() != (w * h * 4) as usize {
            return Err(Error::from_hresult(E_INVALIDARG));
        }
        let room_h = room_h.max(h);
        // SAFETY: the DIB section's bits are written before use and the buffer size matches
        // the header; the DC and bitmap created here are owned by the returned value and
        // released in `Drop` (or right here on failure).
        unsafe {
            let screen = GetDC(None);
            let mem = CreateCompatibleDC(Some(screen));
            let info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w,
                    biHeight: -room_h, // top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB as u32,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits: *mut core::ffi::c_void = core::ptr::null_mut();
            let dib = CreateDIBSection(
                Some(screen),
                &info,
                DIB_RGB_COLORS as u32,
                &mut bits,
                None,
                0,
            );
            let _ = ReleaseDC(None, screen);
            if dib.0.is_null() || bits.is_null() {
                let err = Error::from_thread();
                let _ = DeleteDC(mem);
                return Err(err);
            }
            core::ptr::copy_nonoverlapping(bgra.as_ptr(), bits.cast::<u8>(), bgra.len());
            let old = SelectObject(mem, HGDIOBJ(dib.0));
            Ok(Self {
                mem,
                dib,
                old,
                bits: bits.cast::<u8>(),
                width: w,
                height: room_h,
                rows: h,
            })
        }
    }

    /// Pixel size of the image.
    pub fn size(&self) -> (i32, i32) {
        (self.width, self.rows)
    }

    /// Puts another image into the DIB section in place of the one it holds: `w` must be its
    /// width and `h` at most its height. Returns false, and changes nothing, when the image
    /// does not fit. A shadow that follows a fence's height frame by frame made a new section
    /// of 1.5 to 2.4 MB for every frame otherwise.
    pub fn refill(&mut self, w: i32, h: i32, bgra: &[u8]) -> bool {
        if w != self.width || h <= 0 || h > self.height || bgra.len() != (w * h * 4) as usize {
            return false;
        }
        // SAFETY: the section holds `width` x `height` pixels and the image is no larger.
        unsafe { core::ptr::copy_nonoverlapping(bgra.as_ptr(), self.bits, bgra.len()) };
        self.rows = h;
        true
    }

    /// Every pixel of the section (its whole room, `width * 4` bytes a row, top row first),
    /// for a caller that rewrites some rows of the image in place; `set_rows` then says how
    /// many rows the image has.
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        // SAFETY: the section holds `width` x `height` pixels of 4 bytes for as long as
        // `self` lives, and nothing else reads them while this borrow is held.
        unsafe {
            core::slice::from_raw_parts_mut(self.bits, (self.width * self.height * 4) as usize)
        }
    }

    /// The image is the section's first `h` rows from now on. Returns false, and changes
    /// nothing, when the section has no room for that many.
    pub fn set_rows(&mut self, h: i32) -> bool {
        if h <= 0 || h > self.height {
            return false;
        }
        self.rows = h;
        true
    }

    /// Presents the image to `hwnd` at (`x`, `y`) (screen pixels), sizing the window to the
    /// image. `alpha` is the constant alpha multiplied into every pixel
    /// (`SourceConstantAlpha`): 255 shows the image as is, smaller values fade the whole window
    /// without re-rendering it.
    pub fn present(&self, hwnd: HWND, x: i32, y: i32, alpha: u8) -> Result<()> {
        // SAFETY: the memory DC still has the DIB selected (both live as long as `self`); the
        // screen DC is released before returning.
        unsafe {
            let screen = GetDC(None);
            let dst = POINT { x, y };
            let size = SIZE {
                cx: self.width,
                cy: self.rows,
            };
            let src = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: alpha,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let ok = UpdateLayeredWindow(
                hwnd,
                Some(screen),
                Some(&dst),
                Some(&size),
                Some(self.mem),
                Some(&src),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA as u32,
            );
            let _ = ReleaseDC(None, screen);
            ok.ok()
        }
    }
}

/// Changes only the constant alpha of a layered window whose content was presented before.
/// The system keeps that content, so the source bitmap may already be gone
/// (`UpdateLayeredWindow` with a NULL source DC: "the shape and visual context of the window
/// will not change").
pub fn set_alpha(hwnd: HWND, alpha: u8) -> Result<()> {
    let blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: alpha,
        AlphaFormat: AC_SRC_ALPHA as u8,
    };
    // SAFETY: plain call; every optional pointer is NULL except the blend on the stack.
    unsafe {
        UpdateLayeredWindow(
            hwnd,
            None,
            None,
            None,
            None,
            None,
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA as u32,
        )
        .ok()
    }
}

impl Drop for LayeredImage {
    fn drop(&mut self) {
        // SAFETY: the objects were created in `new` and are released exactly once here.
        unsafe {
            let _ = SelectObject(self.mem, self.old);
            let _ = DeleteObject(HGDIOBJ(self.dib.0));
            let _ = DeleteDC(self.mem);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_later_image_of_the_same_width_goes_into_the_same_section() {
        let first = vec![7u8; 8 * 4 * 4];
        let mut image = LayeredImage::with_room(8, 4, 6, &first).unwrap();
        assert_eq!(image.size(), (8, 4));
        // Taller, up to the room: it fits and is the image from now on.
        let taller = vec![9u8; 8 * 6 * 4];
        assert!(image.refill(8, 6, &taller));
        assert_eq!(image.size(), (8, 6));
        // SAFETY: the section is 8 x 6 pixels.
        let held = unsafe { std::slice::from_raw_parts(image.bits, 8 * 6 * 4) };
        assert_eq!(held, &taller[..]);
        // Shorter again.
        assert!(image.refill(8, 2, &[1u8; 8 * 2 * 4]));
        assert_eq!(image.size(), (8, 2));
        // Past the room, another width, or pixels that do not match the size: left as it is.
        assert!(!image.refill(8, 7, &[0u8; 8 * 7 * 4]));
        assert!(!image.refill(9, 2, &[0u8; 9 * 2 * 4]));
        assert!(!image.refill(8, 2, &[0u8; 8 * 2 * 4 - 1]));
        assert_eq!(image.size(), (8, 2));
        // `new` leaves no room.
        let mut exact = LayeredImage::new(8, 4, &first).unwrap();
        assert!(!exact.refill(8, 5, &[0u8; 8 * 5 * 4]));
        assert!(exact.refill(8, 4, &first));
    }
}
