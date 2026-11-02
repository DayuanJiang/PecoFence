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
    /// Image path, if a picture wallpaper is set for this monitor: Windows' own copy of the
    /// picture when the file it was set from is gone.
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

const DESKTOPS_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\VirtualDesktops";

/// Reads a registry value of `kind` (RRF_RT_*) into a byte buffer.
fn registry_value(key: &str, value: &str, kind: u32) -> Option<Vec<u8>> {
    let key = to_wide(key);
    let value = to_wide(value);
    let mut size = 0u32;
    // SAFETY: size query, then a read into a buffer of that size; strings outlive the calls.
    unsafe {
        let status = RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(key.as_ptr()),
            PCWSTR(value.as_ptr()),
            kind,
            None,
            None,
            Some(&mut size),
        );
        if status.0 != 0 || size == 0 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        let status = RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(key.as_ptr()),
            PCWSTR(value.as_ptr()),
            kind,
            None,
            Some(data.as_mut_ptr().cast()),
            Some(&mut size),
        );
        (status.0 == 0).then(|| {
            data.truncate(size as usize);
            data
        })
    }
}

/// `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` for a GUID stored in its binary layout.
fn guid_string(id: &[u8; 16]) -> String {
    let d1 = u32::from_le_bytes([id[0], id[1], id[2], id[3]]);
    let d2 = u16::from_le_bytes([id[4], id[5]]);
    let d3 = u16::from_le_bytes([id[6], id[7]]);
    let tail: String = id[8..].iter().map(|b| format!("{b:02X}")).collect();
    format!(
        "{{{d1:08X}-{d2:04X}-{d3:04X}-{}-{}}}",
        &tail[..4],
        &tail[4..]
    )
}

/// Every virtual desktop in task-view order with the picture Explorer keeps for it (`None`
/// when the desktop has no picture of its own). Like [`desktop_id`], an Explorer detail: a
/// hint for preparing backdrops, never what is displayed.
pub fn desktop_wallpapers() -> Vec<([u8; 16], Option<PathBuf>)> {
    const RRF_RT_REG_SZ: u32 = 0x02;
    const RRF_RT_REG_BINARY: u32 = 0x08;
    let Some(ids) = registry_value(DESKTOPS_KEY, "VirtualDesktopIDs", RRF_RT_REG_BINARY) else {
        return Vec::new();
    };
    ids.as_chunks::<16>()
        .0
        .iter()
        .map(|chunk| {
            let id = *chunk;
            let key = format!(r"{DESKTOPS_KEY}\Desktops\{}", guid_string(&id));
            let path =
                registry_value(&key, "Wallpaper", RRF_RT_REG_SZ).and_then(|data| utf16_path(&data));
            (id, path)
        })
        .collect()
}

/// The NUL-terminated UTF-16LE path at the start of `data`, `None` when it is empty.
fn utf16_path(data: &[u8]) -> Option<PathBuf> {
    let wide: Vec<u16> = data
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|&c| c != 0)
        .collect();
    (!wide.is_empty()).then(|| PathBuf::from(String::from_utf16_lossy(&wide)))
}

/// The copy Windows made of the picture `path` when it was set as the wallpaper, for a
/// picture deleted or moved since (a wallpaper app that set it from a temporary file): the
/// desktop keeps showing that copy. `Themes\TranscodedWallpaper` is the picture of all
/// monitors, `Themes\Transcoded_000`, `_001` … one per monitor when they show different
/// pictures; `Control Panel\Desktop\TranscodedImageCache` (`_000` …) records each copy's
/// source path, in UTF-16 from byte 24.
fn windows_copy(path: &Path) -> Option<PathBuf> {
    const RRF_RT_REG_BINARY: u32 = 0x08;
    let themes = PathBuf::from(std::env::var_os("APPDATA")?).join(r"Microsoft\Windows\Themes");
    let wanted = path.to_string_lossy();
    std::fs::read_dir(themes).ok()?.flatten().find_map(|entry| {
        let name = entry.file_name().into_string().ok()?;
        let value = match name.strip_prefix("Transcoded_") {
            Some(index) => format!("TranscodedImageCache_{index}"),
            None if name == "TranscodedWallpaper" => "TranscodedImageCache".into(),
            None => return None,
        };
        let data = registry_value(r"Control Panel\Desktop", &value, RRF_RT_REG_BINARY)?;
        let source = utf16_path(data.get(24..)?)?;
        source
            .to_string_lossy()
            .eq_ignore_ascii_case(&wanted)
            .then(|| entry.path())
    })
}

/// The virtual desktop `hwnd` is on, `None` when Explorer does not say (the desktop itself,
/// windows shown on every desktop, a failed call). Requires COM on this thread.
pub fn window_desktop(hwnd: HWND) -> Option<u128> {
    use std::mem::ManuallyDrop;
    // Never released at exit: thread-local destructors run inside ExitProcess, where releasing
    // Explorer's object waits for an RPC reply that never comes. The process then hangs for
    // good, holding the single-instance mutex, and PecoFence cannot start again.
    thread_local! {
        static MANAGER: std::cell::RefCell<Option<ManuallyDrop<IVirtualDesktopManager>>> =
            const { std::cell::RefCell::new(None) };
    }
    MANAGER.with_borrow_mut(|manager| {
        if manager.is_none() {
            // SAFETY: standard COM activation.
            *manager = unsafe { CoCreateInstance(&VirtualDesktopManager, None, CLSCTX_ALL as u32) }
                .ok()
                .map(ManuallyDrop::new);
        }
        // SAFETY: COM call on a live interface.
        match unsafe { manager.as_ref()?.GetWindowDesktopId(hwnd) } {
            Ok(id) => Some(id.to_u128()).filter(|&id| id != 0),
            Err(_) => {
                // Explorer restarted, perhaps: ask a new object next time.
                drop(manager.take().map(ManuallyDrop::into_inner));
                None
            }
        }
    })
}

/// Whether a virtual-desktop switch can change the picture: more than one desktop, and they
/// do not all keep the same one (one without a picture of its own differs from one with;
/// Windows 10 keeps none per desktop). A hint like [`desktop_wallpapers`].
pub fn desktops_differ() -> bool {
    let desktops = desktop_wallpapers();
    desktops.len() > 1 && desktops.iter().any(|(_, path)| *path != desktops[0].1)
}

/// Watch the wallpaper settings, Explorer's virtual-desktop state and the folder of Explorer's
/// copies of the picture. The last two are only opportunistic triggers (Explorer
/// implementation details); callers must keep a poll fallback and re-query IDesktopWallpaper
/// before displaying anything.
pub fn watch_changes(on_change: impl Fn() + Send + 'static) -> Result<()> {
    // The watcher threads serialize access to a callback that need only be Send.
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
    // The Store edition gets no registry notification: MSIX merges HKCU with the package's own
    // hive, and RegNotifyChangeKeyValue there reports only the package's writes. Explorer
    // writes its copy of every new picture (a slideshow step too) to this folder, and its
    // change notifications reach a packaged process as well.
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let themes = PathBuf::from(appdata).join(r"Microsoft\Windows\Themes");
        let callback = on_change.clone();
        match crate::watcher::DirWatcher::start(&themes, move |_| {
            note_signal(Signal::ThemesFolder);
            if let Ok(callback) = callback.lock() {
                callback();
            }
        }) {
            Ok(watcher) => {
                // Watches for the life of the process, like the registry watchers.
                std::mem::forget(watcher);
                result = Ok(());
            }
            Err(error) => tracing::warn!(%error, "wallpaper change source unavailable"),
        }
    }
    result
}

/// A source of wallpaper change signals, for the log: whether a signal arrived before a refresh
/// (a packaged process may not get them all).
#[derive(Clone, Copy)]
pub enum Signal {
    /// WM_SETTINGCHANGE with SPI_SETDESKWALLPAPER.
    SettingChange,
    /// The watch on `Control Panel\Desktop`.
    DesktopKey,
    /// The watch on Explorer's `VirtualDesktops` key.
    DesktopsKey,
    /// The watch on the folder of Explorer's copies of the picture (`Themes`).
    ThemesFolder,
}

/// Per [`Signal`]: when it last came, in ms after [`SIGNAL_EPOCH`] plus one (0: not yet).
static LAST_SIGNAL: [std::sync::atomic::AtomicU64; 4] =
    [const { std::sync::atomic::AtomicU64::new(0) }; 4];
static SIGNAL_EPOCH: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

fn signal_clock() -> u64 {
    SIGNAL_EPOCH
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_millis() as u64
        + 1
}

/// Notes that `signal` arrived now.
pub fn note_signal(signal: Signal) {
    LAST_SIGNAL[signal as usize].store(signal_clock(), std::sync::atomic::Ordering::Relaxed);
}

/// Per [`Signal`] (in its order): how many ms ago it last came, `None` when it has not this run.
pub fn signal_ages_ms() -> [Option<u64>; 4] {
    let now = signal_clock();
    LAST_SIGNAL.each_ref().map(|at| {
        let at = at.load(std::sync::atomic::Ordering::Relaxed);
        (at != 0).then(|| now.saturating_sub(at))
    })
}

/// What `RegNotifyChangeKeyValue` returns once the watched key was deleted.
const ERROR_KEY_DELETED: i32 = 1018;

fn open_notify_key(path: &[u16]) -> Option<HKEY> {
    let mut key = HKEY::default();
    // SAFETY: out-pointer references a local; the string outlives the call.
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(path.as_ptr()),
            None,
            ACCESS_MASK(KEY_NOTIFY as u32),
            &mut key,
        )
    };
    (status.0 == 0).then_some(key)
}

fn watch_registry_key(
    path: &str,
    subtree: bool,
    name: &str,
    on_change: impl Fn() + Send + 'static,
) -> Result<()> {
    let key_path = to_wide(path);
    let Some(key) = open_notify_key(&key_path) else {
        return Err(windows_core::Error::from_hresult(E_FAIL));
    };
    let raw = key.0 as isize;
    let thread = std::thread::Builder::new()
        .name(name.into())
        .stack_size(64 * 1024)
        .spawn(move || {
            let mut key = HKEY(raw as *mut core::ffi::c_void);
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
                if st.0 == 0 {
                    tracing::debug!(?key, "wallpaper registry change received");
                    note_signal(if subtree {
                        Signal::DesktopsKey
                    } else {
                        Signal::DesktopKey
                    });
                    on_change();
                    continue;
                }
                // SAFETY: closing the key we opened.
                unsafe {
                    let _ = RegCloseKey(key);
                }
                if st.0 != ERROR_KEY_DELETED {
                    tracing::warn!(status = st.0, ?key, "wallpaper registry watcher stopped");
                    break;
                }
                // Explorer replaced the key: watch the one that is there now, and report the
                // changes missed meanwhile.
                key = loop {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    if let Some(key) = open_notify_key(&key_path) {
                        break key;
                    }
                };
                tracing::info!(?key, "wallpaper registry watcher reopened its key");
                on_change();
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
                (!path.is_empty()).then(|| {
                    let path = PathBuf::from(path);
                    if path.is_file() {
                        path
                    } else {
                        windows_copy(&path).unwrap_or(path)
                    }
                })
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
    decode(path, |w, h| {
        let scale = (max_width as f32 / w as f32)
            .min(max_height as f32 / h as f32)
            .min(1.0);
        let dst_w = ((w as f32 * scale).round() as u32).max(1);
        let dst_h = ((h as f32 * scale).round() as u32).max(1);
        ((dst_w, dst_h), [0, 0, dst_w, dst_h])
    })
}

/// Decodes `path` at its own size and keeps at most `max_width` x `max_height` pixels of it:
/// the middle when `centered`, else the top-left corner. That is all a monitor of that size
/// shows of a centred or a tiled picture.
pub fn decode_native(
    path: &Path,
    max_width: u32,
    max_height: u32,
    centered: bool,
) -> Result<DecodedImage> {
    decode(path, |w, h| {
        let (cw, ch) = (w.min(max_width.max(1)), h.min(max_height.max(1)));
        let (x, y) = if centered {
            ((w - cw) / 2, (h - ch) / 2)
        } else {
            (0, 0)
        };
        ((w, h), [x, y, cw, ch])
    })
}

/// The WIC pipeline of both: `size` maps the picture's size to the size it is scaled to and
/// the part of the scaled picture (x, y, width, height) that is copied out.
fn decode(
    path: &Path,
    size: impl FnOnce(u32, u32) -> ((u32, u32), [u32; 4]),
) -> Result<DecodedImage> {
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
        let ((dst_w, dst_h), [x, y, out_w, out_h]) = size(w, h);

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
        let stride = out_w * 4;
        let mut bgra = vec![0u8; (stride * out_h) as usize];
        let rect = WICRect {
            X: x as i32,
            Y: y as i32,
            Width: out_w as i32,
            Height: out_h as i32,
        };
        // The generated wrapper mis-projects CopyPixels' buffer as a return value; call the
        // vtable slot directly.
        (Interface::vtable(&source).CopyPixels)(
            Interface::as_raw(&source),
            &rect,
            stride,
            bgra.len() as u32,
            bgra.as_mut_ptr(),
        )
        .ok()?;
        Ok(DecodedImage {
            width: out_w,
            height: out_h,
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
    fn a_picture_copy_rewritten_in_place_is_reported() {
        use std::{sync::mpsc, time::Duration};
        // Explorer rewrites Themes\TranscodedWallpaper for every new picture: the folder watch
        // `watch_changes` relies on must report a rewrite of a file that already exists.
        let dir = std::env::temp_dir().join(format!(
            "pecofence-wallpaper-folder-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let copy = dir.join("TranscodedWallpaper");
        std::fs::write(&copy, b"old picture").unwrap();
        let (send, receive) = mpsc::channel();
        let watcher = crate::watcher::DirWatcher::start(&dir, move |_| {
            let _ = send.send(());
        })
        .unwrap();
        // The watch starts on its own thread: rewrite until it reports.
        let mut delivered = false;
        for version in 0..20 {
            std::fs::write(&copy, format!("new picture {version}")).unwrap();
            if receive.recv_timeout(Duration::from_millis(250)).is_ok() {
                delivered = true;
                break;
            }
        }
        drop(watcher);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            delivered,
            "a rewritten picture copy did not wake the folder watch"
        );
    }

    #[test]
    fn desktop_ids_format_as_their_registry_key_names() {
        let id = [
            0x39, 0xF8, 0x5B, 0xB3, 0x0F, 0xB5, 0xC7, 0x40, 0xAB, 0x31, 0xB2, 0xF7, 0xDB, 0xD5,
            0x5B, 0x32,
        ];
        assert_eq!(guid_string(&id), "{B35BF839-B50F-40C7-AB31-B2F7DBD55B32}");
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
