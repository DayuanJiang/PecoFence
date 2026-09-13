//! Per-pixel-alpha layered windows (`UpdateLayeredWindow`), used for the fence shadow.

use crate::bindings::*;
use windows_core::{Error, Result};

/// A premultiplied BGRA image resident in a GDI DIB section (selected into its own memory
/// DC) so it can be presented to a `WS_EX_LAYERED` window any number of times — e.g. once per
/// frame of a fade with only the constant alpha changing — without allocating a bitmap or
/// copying the pixels again. The GDI objects are released on drop.
pub struct LayeredImage {
    mem: HDC,
    dib: HBITMAP,
    old: HGDIOBJ,
    width: i32,
    height: i32,
}

impl LayeredImage {
    /// Uploads a `w` x `h` premultiplied BGRA image (`bgra.len()` must be `w * h * 4`).
    pub fn new(w: i32, h: i32, bgra: &[u8]) -> Result<Self> {
        if w <= 0 || h <= 0 || bgra.len() != (w * h * 4) as usize {
            return Err(Error::from_hresult(E_INVALIDARG));
        }
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
                    biHeight: -h, // top-down
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
                width: w,
                height: h,
            })
        }
    }

    /// Pixel size of the image.
    pub fn size(&self) -> (i32, i32) {
        (self.width, self.height)
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
                cy: self.height,
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
