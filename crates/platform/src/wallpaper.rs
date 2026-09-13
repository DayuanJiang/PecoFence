//! Per-monitor wallpaper discovery (`IDesktopWallpaper`) and WIC decoding to a small BGRA buffer.

use crate::bindings::*;
use crate::wide::to_wide;
use std::path::{Path, PathBuf};
use windows_core::{Interface, PCWSTR, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    Center,
    Tile,
    Stretch,
    Fit,
    Fill,
    Span,
}

#[derive(Clone, Debug)]
pub struct MonitorWallpaper {
    /// Device path id used by `IDesktopWallpaper` (`\\?\DISPLAY#...`).
    pub monitor_id: String,
    /// Monitor rectangle in virtual-screen coordinates.
    pub rect: RECT,
    /// Image path, if a picture wallpaper is set for this monitor.
    pub path: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct WallpaperSnapshot {
    pub position: Position,
    /// Solid background colour (RGB) shown where the picture does not cover.
    pub background: [u8; 3],
    pub monitors: Vec<MonitorWallpaper>,
}

fn take_string(pwstr: windows_core::PWSTR) -> String {
    if pwstr.is_null() {
        return String::new();
    }
    // SAFETY: the shell allocated this NUL-terminated string with CoTaskMemAlloc.
    unsafe {
        let s = pwstr.to_string().unwrap_or_default();
        CoTaskMemFree(pwstr.0.cast());
        s
    }
}

/// Queries the current wallpaper configuration. Requires COM initialized on this thread.
/// A compact fingerprint of the current wallpaper state (position, colour, per-monitor image
/// path + file size/mtime). Two equal signatures mean the blurred backdrops are still valid.
pub fn signature() -> Option<String> {
    let snap = query().ok()?;
    let mut sig = format!("{:?}|{:?}", snap.position, snap.background);
    for m in &snap.monitors {
        sig.push('|');
        sig.push_str(&m.monitor_id);
        if let Some(p) = &m.path {
            sig.push('=');
            sig.push_str(&p.to_string_lossy());
            if let Ok(meta) = std::fs::metadata(p) {
                use std::os::windows::fs::MetadataExt;
                sig.push_str(&format!("#{}#{}", meta.file_size(), meta.last_write_time()));
            }
        }
    }
    Some(sig)
}

/// Watches `HKCU\Control Panel\Desktop` (Wallpaper / TranscodedImageCache change whenever
/// the picture, slideshow slide or fit changes) and calls `on_change` from a worker thread.
/// Settings and slideshows do not always broadcast `WM_SETTINGCHANGE(SPI_SETDESKWALLPAPER)`,
/// so this is the reliable trigger; the caller should still compare `signature()`.
pub fn watch_changes(on_change: impl Fn() + Send + 'static) -> Result<()> {
    let key_path = to_wide(r"Control Panel\Desktop");
    let mut key = HKEY::default();
    // SAFETY: out-pointer references a local; the string outlives the call.
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_path.as_ptr()),
            None,
            ACCESS_MASK(KEY_NOTIFY as u32),
            &mut key,
        )
    };
    if status.0 != 0 {
        return Err(windows_core::Error::from_hresult(E_FAIL));
    }
    let raw = key.0 as isize;
    std::thread::Builder::new()
        .name("pecofence-wallpaper-watch".into())
        .stack_size(64 * 1024)
        .spawn(move || {
            let key = HKEY(raw as *mut core::ffi::c_void);
            loop {
                // SAFETY: synchronous wait on a key we opened; returns when a value changes.
                let st = unsafe {
                    RegNotifyChangeKeyValue(
                        key,
                        false,
                        REG_NOTIFY_CHANGE_LAST_SET as u32,
                        None,
                        false,
                    )
                };
                if st.0 != 0 {
                    break;
                }
                on_change();
            }
            // SAFETY: closing the key we opened.
            unsafe {
                let _ = RegCloseKey(key);
            }
        })
        .map(|_| ())
        .map_err(|_| windows_core::Error::from_hresult(E_FAIL))
}

pub fn query() -> Result<WallpaperSnapshot> {
    // SAFETY: standard COM activation.
    let dw: IDesktopWallpaper =
        unsafe { CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL as u32)? };
    // SAFETY: COM calls on a live interface.
    unsafe {
        let position = match dw.GetPosition().unwrap_or(DWPOS_FILL) {
            DWPOS_CENTER => Position::Center,
            DWPOS_TILE => Position::Tile,
            DWPOS_STRETCH => Position::Stretch,
            DWPOS_FIT => Position::Fit,
            DWPOS_SPAN => Position::Span,
            _ => Position::Fill,
        };
        let bg = dw.GetBackgroundColor().map(|c| c.0).unwrap_or(0);
        let background = [
            (bg & 0xff) as u8,
            ((bg >> 8) & 0xff) as u8,
            ((bg >> 16) & 0xff) as u8,
        ];

        let count = dw.GetMonitorDevicePathCount()?;
        let mut monitors = Vec::with_capacity(count as usize);
        for i in 0..count {
            let id = take_string(dw.GetMonitorDevicePathAt(i)?);
            let id_w = to_wide(&id);
            let rect = dw.GetMonitorRECT(PCWSTR(id_w.as_ptr())).unwrap_or_default();
            let path = dw
                .GetWallpaper(PCWSTR(id_w.as_ptr()))
                .map(take_string)
                .ok()
                .filter(|p| !p.is_empty())
                .map(PathBuf::from);
            monitors.push(MonitorWallpaper {
                monitor_id: id,
                rect,
                path,
            });
        }
        Ok(WallpaperSnapshot {
            position,
            background,
            monitors,
        })
    }
}

/// Decoded opaque BGRA pixels.
#[derive(Clone, Debug)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

/// Decodes `path` and scales it (preserving aspect ratio) so it fits within
/// `max_width` x `max_height`. The scaler is attached directly to the frame so the full-size
/// bitmap is never materialized.
pub fn decode_scaled(path: &Path, max_width: u32, max_height: u32) -> Result<DecodedImage> {
    let wide: Vec<u16> = path.as_os_str().encode_wide_nul();
    // SAFETY: standard WIC pipeline; every interface is owned for the duration of the call.
    unsafe {
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
        let decoder = factory.CreateDecoderFromFilename(
            PCWSTR(wide.as_ptr()),
            core::ptr::null(),
            GENERIC_READ,
            WICDecodeMetadataCacheOnDemand,
        )?;
        let frame = decoder.GetFrame(0)?;
        let frame_src: IWICBitmapSource = frame.cast()?;
        let (mut w, mut h) = (0u32, 0u32);
        frame_src.GetSize(&mut w, &mut h).ok()?;
        if w == 0 || h == 0 {
            return Err(windows_core::Error::from_hresult(E_FAIL));
        }
        let scale = (max_width as f32 / w as f32)
            .min(max_height as f32 / h as f32)
            .min(1.0);
        let dst_w = ((w as f32 * scale).round() as u32).max(1);
        let dst_h = ((h as f32 * scale).round() as u32).max(1);

        let scaler = factory.CreateBitmapScaler()?;
        scaler
            .Initialize(&frame_src, dst_w, dst_h, WICBitmapInterpolationModeFant)
            .ok()?;
        let converter = factory.CreateFormatConverter()?;
        converter
            .Initialize(
                &scaler,
                &GUID_WICPixelFormat32bppPBGRA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeMedianCut,
            )
            .ok()?;
        let source: IWICBitmapSource = converter.cast()?;
        let stride = dst_w * 4;
        let mut bgra = vec![0u8; (stride * dst_h) as usize];
        // The generated wrapper mis-projects CopyPixels' buffer as a return value; call the
        // vtable slot directly.
        (Interface::vtable(&source).CopyPixels)(
            Interface::as_raw(&source),
            core::ptr::null(),
            stride,
            bgra.len() as u32,
            bgra.as_mut_ptr(),
        )
        .ok()?;
        Ok(DecodedImage {
            width: dst_w,
            height: dst_h,
            bgra,
        })
    }
}

trait EncodeWideNul {
    fn encode_wide_nul(&self) -> Vec<u16>;
}

impl EncodeWideNul for std::ffi::OsStr {
    fn encode_wide_nul(&self) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        self.encode_wide().chain(std::iter::once(0)).collect()
    }
}
