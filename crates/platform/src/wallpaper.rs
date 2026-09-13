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

impl WallpaperSnapshot {
    /// Fingerprint the same snapshot used to build the backdrops, including monitor geometry.
    /// File metadata is re-read so callers can reject a file changed during decoding.
    pub fn signature(&self) -> String {
        let mut sig = format!("{:?}|{:?}", self.position, self.background);
        for m in &self.monitors {
            sig.push('|');
            sig.push_str(&m.monitor_id);
            sig.push_str(&format!(
                "@{},{},{},{}",
                m.rect.left, m.rect.top, m.rect.right, m.rect.bottom
            ));
            if let Some(p) = &m.path {
                sig.push('=');
                sig.push_str(&p.to_string_lossy());
                if let Ok(meta) = std::fs::metadata(p) {
                    use std::os::windows::fs::MetadataExt;
                    sig.push_str(&format!("#{}#{}", meta.file_size(), meta.last_write_time()));
                }
            }
        }
        sig
    }
}

/// Queries a fingerprint of the current wallpaper. Requires COM on this thread.
pub fn signature() -> Option<String> {
    Some(query().ok()?.signature())
}

/// A cheap hint for detecting missed desktop notifications. Never use it as the wallpaper
/// itself: Explorer may publish the new picture slightly after changing this value.
pub fn desktop_id() -> Option<[u8; 16]> {
    let mut id = [0u8; 16];
    let mut size = id.len() as u32;
    // RRF_RT_REG_BINARY constrains the type; RegGetValueW opens the current key on each read,
    // so this also survives Explorer replacing a key that a registry watcher had opened.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            windows_core::w!(r"Software\Microsoft\Windows\CurrentVersion\Explorer\VirtualDesktops"),
            windows_core::w!("CurrentVirtualDesktop"),
            0x08,
            None,
            Some(id.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    (status.0 == 0 && size == 16).then_some(id)
}

/// Watch both wallpaper settings and Explorer's virtual-desktop state. The latter is only
/// an opportunistic trigger (an Explorer implementation detail); callers must keep a poll
/// fallback and re-query IDesktopWallpaper before displaying anything.
pub fn watch_changes(on_change: impl Fn() + Send + 'static) -> Result<()> {
    // The two watcher threads serialize access to a callback that need only be Send.
    let on_change = std::sync::Arc::new(std::sync::Mutex::new(on_change));
    let mut result = Err(windows_core::Error::from_hresult(E_FAIL));
    for (path, subtree, name) in [
        (r"Control Panel\Desktop", false, "pecofence-wallpaper-watch"),
        (
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\VirtualDesktops",
            true,
            "pecofence-desktop-watch",
        ),
    ] {
        let callback = on_change.clone();
        match watch_registry_key(path, subtree, name, move || {
            if let Ok(callback) = callback.lock() {
                callback();
            }
        }) {
            Ok(()) => result = Ok(()),
            Err(error) => tracing::warn!(path, %error, "wallpaper change source unavailable"),
        }
    }
    result
}

fn watch_registry_key(
    path: &str,
    subtree: bool,
    name: &str,
    on_change: impl Fn() + Send + 'static,
) -> Result<()> {
    let key_path = to_wide(path);
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
    let thread = std::thread::Builder::new()
        .name(name.into())
        .stack_size(64 * 1024)
        .spawn(move || {
            let key = HKEY(raw as *mut core::ffi::c_void);
            tracing::debug!(?key, subtree, "wallpaper registry watcher started");
            loop {
                // SAFETY: synchronous wait on a key we opened; returns when a value changes.
                let st = unsafe {
                    RegNotifyChangeKeyValue(
                        key,
                        subtree,
                        REG_NOTIFY_CHANGE_LAST_SET as u32,
                        None,
                        false,
                    )
                };
                if st.0 != 0 {
                    tracing::warn!(status = st.0, ?key, "wallpaper registry watcher stopped");
                    break;
                }
                tracing::debug!(?key, "wallpaper registry change received");
                on_change();
            }
            // SAFETY: closing the key we opened.
            unsafe {
                let _ = RegCloseKey(key);
            }
        });
    if thread.is_err() {
        // No thread took ownership of the key when spawning failed.
        unsafe {
            let _ = RegCloseKey(key);
        }
    }
    thread
        .map(|_| ())
        .map_err(|_| windows_core::Error::from_hresult(E_FAIL))
}

pub fn query() -> Result<WallpaperSnapshot> {
    // SAFETY: standard COM activation.
    let dw: IDesktopWallpaper =
        unsafe { CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL as u32)? };
    // SAFETY: COM calls on a live interface.
    unsafe {
        let position = match dw.GetPosition()? {
            DWPOS_CENTER => Position::Center,
            DWPOS_TILE => Position::Tile,
            DWPOS_STRETCH => Position::Stretch,
            DWPOS_FIT => Position::Fit,
            DWPOS_SPAN => Position::Span,
            _ => Position::Fill,
        };
        let bg = dw.GetBackgroundColor()?.0;
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
            // Explorer retains disconnected outputs and returns E_FAIL for their rectangle.
            // They are not part of the visible wallpaper and must not fail the whole read.
            let rect = dw.GetMonitorRECT(PCWSTR(id_w.as_ptr())).unwrap_or_default();
            // A failed query on a live monitor is not a solid-colour wallpaper. Preserve
            // that distinction so a switch-in-progress cannot replace good pixels with black.
            let path = if rect.right > rect.left && rect.bottom > rect.top {
                let path = take_string(dw.GetWallpaper(PCWSTR(id_w.as_ptr()))?);
                (!path.is_empty()).then(|| PathBuf::from(path))
            } else {
                None
            };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_watcher_delivers_a_real_value_change() {
        use std::{sync::mpsc, time::Duration};
        use windows_sys::Win32::System::Registry as registry;
        let path = format!(
            r"Software\PecoFenceWallpaperWatchTest-{}",
            std::process::id()
        );
        let wide_path = to_wide(&path);
        let value = to_wide("change");
        let mut key = core::ptr::null_mut();
        unsafe {
            assert_eq!(
                registry::RegCreateKeyW(registry::HKEY_CURRENT_USER, wide_path.as_ptr(), &mut key,),
                0
            );
        }
        let (send, receive) = mpsc::channel();
        watch_registry_key(&path, false, "wallpaper-test", move || {
            let _ = send.send(());
        })
        .unwrap();
        let mut delivered = false;
        for version in 0..20u32 {
            unsafe {
                assert_eq!(
                    registry::RegSetValueExW(
                        key,
                        value.as_ptr(),
                        0,
                        registry::REG_DWORD,
                        version.to_le_bytes().as_ptr(),
                        4,
                    ),
                    0
                );
            }
            if receive.recv_timeout(Duration::from_millis(50)).is_ok() {
                delivered = true;
                break;
            }
        }
        unsafe {
            registry::RegCloseKey(key);
            registry::RegDeleteTreeW(registry::HKEY_CURRENT_USER, wide_path.as_ptr());
        }
        assert!(
            delivered,
            "real registry writes did not wake the wallpaper watcher"
        );
    }

    #[test]
    fn cached_wallpaper_tracks_monitor_placement_and_size() {
        let mut snapshot = WallpaperSnapshot {
            position: Position::Fill,
            background: [20, 30, 40],
            monitors: vec![MonitorWallpaper {
                monitor_id: "display-a".into(),
                rect: RECT {
                    left: 0,
                    top: 0,
                    right: 1920,
                    bottom: 1080,
                },
                path: None,
            }],
        };
        let initial = snapshot.signature();
        assert_eq!(initial, snapshot.clone().signature());
        snapshot.monitors[0].rect.left = -1920;
        snapshot.monitors[0].rect.right = 0;
        assert_ne!(initial, snapshot.signature());
        let moved = snapshot.signature();
        snapshot.monitors[0].rect.bottom = 1200;
        assert_ne!(moved, snapshot.signature());
    }

    #[test]
    fn fit_and_solid_colour_changes_invalidate_cached_pixels() {
        let mut snapshot = WallpaperSnapshot {
            position: Position::Fill,
            background: [0, 0, 0],
            monitors: Vec::new(),
        };
        let initial = snapshot.signature();
        snapshot.position = Position::Fit;
        assert_ne!(initial, snapshot.signature());
        let fit = snapshot.signature();
        snapshot.background = [255, 255, 255];
        assert_ne!(fit, snapshot.signature());
    }
}
