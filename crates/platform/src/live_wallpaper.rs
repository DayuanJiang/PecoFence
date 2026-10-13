//! Live wallpapers (Wallpaper Engine, Lively, ...): another program's player window inside the
//! desktop's wallpaper layer, which hides the Windows wallpaper, and that window's pixels.

use crate::bindings::*;
use crate::desktop::{class_name, is_cloaked, is_visible, shell_window, window_pid};
use crate::wallpaper::DecodedImage;
use crate::window::window_rect;
use windows_core::PCWSTR;

/// The child of `parent` after `after` (the first one for `None`), of `class` if given; a null
/// handle when there is none. A `None` parent walks the top-level windows.
fn next_child(parent: Option<HWND>, after: Option<HWND>, class: Option<&str>) -> HWND {
    let wide = class.map(crate::wide::to_wide);
    let class = wide.as_ref().map_or(PCWSTR::null(), |w| PCWSTR(w.as_ptr()));
    // SAFETY: the class string outlives the call.
    unsafe { FindWindowExW(parent, after, class, PCWSTR::null()) }
}

/// Visible windows of programs other than Explorer that are direct children of Progman, of a
/// WorkerW under Progman, or of one of Explorer's top-level WorkerW windows, with their screen
/// rectangles. Windows 10 puts Lively's player into the top-level WorkerW behind the icons;
/// Windows 11 24H2+ into Progman, between the icons and Explorer's own wallpaper WorkerW.
/// About 20 to 40 us.
pub fn windows() -> Vec<(HWND, RECT)> {
    let progman = shell_window();
    if progman.0.is_null() || class_name(progman) != "Progman" {
        return Vec::new();
    }
    let explorer = window_pid(progman);
    let mut parents = vec![progman];
    for parent in [Some(progman), None] {
        let mut worker = next_child(parent, None, Some("WorkerW"));
        while !worker.0.is_null() {
            if window_pid(worker) == explorer {
                parents.push(worker);
            }
            worker = next_child(parent, Some(worker), Some("WorkerW"));
        }
    }
    let mut out = Vec::new();
    for parent in parents {
        let mut child = next_child(Some(parent), None, None);
        while !child.0.is_null() {
            if window_pid(child) != explorer && is_visible(child) && !is_cloaked(child) {
                out.push((child, window_rect(child)));
            }
            child = next_child(Some(parent), Some(child), None);
        }
    }
    out
}

/// The window of `live` covering at least 90 % of `monitor`.
pub fn covering(live: &[(HWND, RECT)], monitor: RECT) -> Option<(HWND, RECT)> {
    let area = (monitor.right - monitor.left) as i64 * (monitor.bottom - monitor.top) as i64;
    live.iter().copied().find(|(_, r)| {
        let w = (r.right.min(monitor.right) - r.left.max(monitor.left)).max(0) as i64;
        let h = (r.bottom.min(monitor.bottom) - r.top.max(monitor.top)).max(0) as i64;
        area > 0 && w * h * 10 >= area * 9
    })
}

/// The window's rectangle and its current pixels as opaque BGRA. PrintWindow with
/// PW_RENDERFULLCONTENT is the one way that gets video and web players on Windows 10 and 11
/// without side effects; it blocks while DWM draws the window (50 to 130 ms for a 4K window),
/// so call it off the UI thread. All black when the window drew nothing.
pub fn capture(hwnd: HWND) -> Option<(RECT, DecodedImage)> {
    windows_core::link!("user32.dll" "system" fn PrintWindow(hwnd: HWND, hdc: HDC, flags: u32) -> windows_core::BOOL);
    const PW_RENDERFULLCONTENT: u32 = 2;
    let rect = window_rect(hwnd);
    let (w, h) = (rect.right - rect.left, rect.bottom - rect.top);
    if w <= 0 || h <= 0 {
        return None;
    }
    // SAFETY: GDI objects created and released here; the DIB's bits are read while selected.
    unsafe {
        let dc = CreateCompatibleDC(None);
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB as u32,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut core::ffi::c_void = core::ptr::null_mut();
        let dib = CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS as u32, &mut bits, None, 0);
        if dib.0.is_null() || bits.is_null() {
            let _ = DeleteDC(dc);
            return None;
        }
        let old = SelectObject(dc, HGDIOBJ(dib.0));
        let drawn = PrintWindow(hwnd, dc, PW_RENDERFULLCONTENT).as_bool();
        let bgra = drawn.then(|| {
            let mut bgra =
                std::slice::from_raw_parts(bits.cast::<u8>(), (w * h * 4) as usize).to_vec();
            for px in bgra.as_chunks_mut::<4>().0 {
                px[3] = 255;
            }
            bgra
        });
        SelectObject(dc, old);
        let _ = DeleteObject(HGDIOBJ(dib.0));
        let _ = DeleteDC(dc);
        let image = DecodedImage {
            width: w as u32,
            height: h as u32,
            bgra: bgra?,
        };
        Some((rect, image))
    }
}
