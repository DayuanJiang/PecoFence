//! Shell services: known folders, desktop entries, icons/thumbnails, launching, shortcut targets.

use crate::bindings::*;
use crate::wide::to_wide;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use windows_core::{GUID, Interface, PCWSTR, PWSTR, Result};

// FOLDERID_Desktop / FOLDERID_PublicDesktop (KnownFolders.h).
const FOLDERID_DESKTOP: GUID = GUID::from_u128(0xB4BFCC3A_DB2C_424C_B029_7FE99A87C641);
const FOLDERID_PUBLIC_DESKTOP: GUID = GUID::from_u128(0xC4AA340D_F20F_4863_AFEF_F87EF2E6BA25);

fn take_string(pwstr: PWSTR) -> String {
    if pwstr.is_null() {
        return String::new();
    }
    // SAFETY: shell-allocated NUL-terminated string; freed exactly once.
    unsafe {
        let s = pwstr.to_string().unwrap_or_default();
        CoTaskMemFree(pwstr.0.cast());
        s
    }
}

fn known_folder(id: &GUID) -> Option<PathBuf> {
    // SAFETY: plain FFI call; result freed by take_string.
    let p = unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) }.ok()?;
    let s = take_string(p);
    (!s.is_empty()).then(|| PathBuf::from(s))
}

/// The user's Desktop folder (may be redirected, e.g. OneDrive).
pub fn user_desktop() -> Option<PathBuf> {
    known_folder(&FOLDERID_DESKTOP)
}

/// The shared Public Desktop folder.
pub fn public_desktop() -> Option<PathBuf> {
    known_folder(&FOLDERID_PUBLIC_DESKTOP)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryOrigin {
    UserDesktop,
    PublicDesktop,
    /// A shell namespace item such as the Recycle Bin: no file behind it, `path` holds the
    /// parsing name (`::{CLSID}`).
    Namespace,
}

/// A file or folder on one of the desktop folders.
#[derive(Clone, Debug)]
pub struct DesktopEntry {
    pub path: PathBuf,
    /// File name including extension.
    pub file_name: String,
    pub origin: EntryOrigin,
    pub is_folder: bool,
    pub attributes: u32,
    /// Last write time as Unix seconds.
    pub mtime: i64,
    pub size: u64,
    /// Creation time as Unix seconds (time-of-day / weekday rules).
    pub created: i64,
}

fn filetime_to_unix(ft: u64) -> i64 {
    // FILETIME: 100-ns intervals since 1601-01-01.
    (ft as i64 / 10_000_000) - 11_644_473_600
}

/// False when the user's Desktop folder cannot be read right now (removable drive unplugged,
/// network share offline). Syncing then must not orphan every item.
pub fn desktop_available() -> bool {
    user_desktop().is_some_and(|d| std::fs::read_dir(&d).is_ok())
}

/// Lists desktop entries, skipping hidden/system files and `desktop.ini`.
pub fn enumerate_desktop() -> Vec<DesktopEntry> {
    let mut out = Vec::new();
    for (dir, origin) in [
        (user_desktop(), EntryOrigin::UserDesktop),
        (public_desktop(), EntryOrigin::PublicDesktop),
    ] {
        let Some(dir) = dir else { continue };
        enumerate_dir_into(&dir, origin, &mut out);
    }
    out
}

/// The shell namespace items Windows can draw on the desktop and whether Windows shows them
/// by default: `(CLSID, shown by default)`. Users toggle them in "Desktop icon settings".
const SPECIAL_DESKTOP_ITEMS: [(&str, bool); 5] = [
    (RECYCLE_BIN_CLSID, true),
    ("{20D04FE0-3AEA-1069-A2D8-08002B30309D}", false), // This PC
    ("{59031a47-3f72-44a7-89c5-5595fe6b30ee}", false), // User's Files
    ("{F02C1A0D-BE21-4350-88B0-7367FC96EF3C}", false), // Network
    ("{5399E694-6CE5-4D6C-8FCE-1D8870FDCBA0}", false), // Control Panel
];
pub const RECYCLE_BIN_CLSID: &str = "{645FF040-5081-101B-9F08-00AA002F954E}";
const DESKTOP_ICON_SETTINGS_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\HideDesktopIcons\NewStartPanel";
/// Group Policy "Remove X icon from desktop": `{CLSID} = 1` here hides it regardless of the
/// user's own choice.
const NON_ENUM_POLICY_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Policies\NonEnum";

/// True for a shell parsing name of a namespace item (`::{CLSID}`) rather than a file path.
pub fn is_namespace_path(path: &Path) -> bool {
    path.to_string_lossy().starts_with("::{")
}

/// True for the Recycle Bin's parsing name in any letter case (item keys are lower-cased).
pub fn is_recycle_bin_path(path: &Path) -> bool {
    path.to_string_lossy()
        .strip_prefix("::")
        .is_some_and(|clsid| clsid.eq_ignore_ascii_case(RECYCLE_BIN_CLSID))
}

/// The paths of a selection as handed to the shell (context menu, verbs, drag data). The
/// shell serves one parent folder at a time, and namespace items live under the desktop root
/// while files live under their folder: a selection holding both keeps only the files.
pub fn paths_for_shell(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mixed =
        paths.iter().any(|p| is_namespace_path(p)) && paths.iter().any(|p| !is_namespace_path(p));
    if !mixed {
        return paths;
    }
    paths
        .into_iter()
        .filter(|p| !is_namespace_path(p))
        .collect()
}

/// Reads `HKxx\key\name` as a DWORD; `None` when absent or not a DWORD.
fn reg_dword(root: HKEY, key: &str, name: &str) -> Option<u32> {
    let key = to_wide(key);
    let name = to_wide(name);
    let mut value: u32 = 0;
    let mut size = size_of::<u32>() as u32;
    // SAFETY: the strings outlive the call; `value` is a live DWORD-sized out buffer.
    let status = unsafe {
        RegGetValueW(
            root,
            PCWSTR(key.as_ptr()),
            PCWSTR(name.as_ptr()),
            RRF_RT_REG_DWORD as u32,
            None,
            Some((&mut value as *mut u32).cast()),
            Some(&mut size),
        )
    };
    (status.0 == 0).then_some(value)
}

/// Whether Windows hides `clsid` from the desktop, the way Explorer decides it: a policy entry
/// wins, then the user's "Desktop icon settings" (HKCU, HKLM as fallback), then the Windows
/// default (only the Recycle Bin is shown out of the box).
fn desktop_icon_hidden_in_windows(clsid: &str, default_visible: bool) -> bool {
    for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        if reg_dword(root, NON_ENUM_POLICY_KEY, clsid).is_some_and(|v| v != 0) {
            return true;
        }
    }
    for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        if let Some(v) = reg_dword(root, DESKTOP_ICON_SETTINGS_KEY, clsid) {
            return v != 0;
        }
    }
    !default_visible
}

/// Whether the Recycle Bin holds anything (decides between the full and empty icon). Each
/// fixed drive is asked on its own: the all-drives form fails as a whole when one volume
/// (card reader without media, offline mapped drive) cannot answer. When no drive answers,
/// the last known state is kept rather than flipping to "empty".
pub fn recycle_bin_has_items() -> bool {
    use std::sync::atomic::{AtomicBool, Ordering};
    static LAST_KNOWN: AtomicBool = AtomicBool::new(false);
    // SAFETY: no arguments; returns a bitmask.
    let drives = unsafe { GetLogicalDrives() };
    let mut answered = false;
    let mut full = false;
    for i in 0..26u32 {
        if drives & (1 << i) == 0 {
            continue;
        }
        let root = to_wide(&format!("{}:\\", (b'A' + i as u8) as char));
        // SAFETY: the string is NUL-terminated and outlives the call.
        if unsafe { GetDriveTypeW(PCWSTR(root.as_ptr())) } != DRIVE_FIXED as u32 {
            continue;
        }
        let mut info = SHQUERYRBINFO {
            cbSize: size_of::<SHQUERYRBINFO>() as u32,
            ..Default::default()
        };
        // SAFETY: `info` is a live struct with `cbSize` set; `root` is a NUL-terminated path.
        if unsafe { SHQueryRecycleBinW(PCWSTR(root.as_ptr()), &mut info) }.is_ok() {
            answered = true;
            if info.i64NumItems > 0 {
                full = true;
                break;
            }
        }
    }
    if answered {
        LAST_KNOWN.store(full, Ordering::Relaxed);
        full
    } else {
        LAST_KNOWN.load(Ordering::Relaxed)
    }
}

/// The special desktop items the user has enabled in Windows (Recycle Bin, This PC, ...), as
/// entries whose `path` is the shell parsing name. Hiding the real desktop icons hides these
/// too, so the inbox fence shows them instead. The Recycle Bin's `mtime` encodes whether it
/// holds anything, so its icon is re-read when the state flips. Display names are fixed for
/// the session (they follow the Windows display language), so the shell is asked only once.
pub fn enumerate_special_desktop_items() -> Vec<DesktopEntry> {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static NAMES: OnceLock<Mutex<HashMap<&'static str, String>>> = OnceLock::new();
    let names = NAMES.get_or_init(Default::default);
    let cached_name = |clsid: &str| {
        names
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(clsid)
            .cloned()
    };
    let mut out = Vec::new();
    for (clsid, default_visible) in SPECIAL_DESKTOP_ITEMS {
        if desktop_icon_hidden_in_windows(clsid, default_visible) {
            continue;
        }
        let path = PathBuf::from(format!("::{clsid}"));
        let name = cached_name(clsid).or_else(|| {
            let name = display_name(&path)?;
            names
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(clsid, name.clone());
            Some(name)
        });
        // The shell cannot resolve the item (feature removed on this edition): skip it.
        let Some(file_name) = name else {
            continue;
        };
        let mtime = if clsid == RECYCLE_BIN_CLSID && recycle_bin_has_items() {
            1
        } else {
            0
        };
        out.push(DesktopEntry {
            path,
            file_name,
            origin: EntryOrigin::Namespace,
            is_folder: false,
            attributes: 0,
            mtime,
            size: 0,
            created: 0,
        });
    }
    out
}

/// Lists one folder the same way the desktop is listed (folder portals).
pub fn enumerate_folder(dir: &Path) -> Vec<DesktopEntry> {
    let mut out = Vec::new();
    enumerate_dir_into(dir, EntryOrigin::UserDesktop, &mut out);
    out
}

fn enumerate_dir_into(dir: &Path, origin: EntryOrigin, out: &mut Vec<DesktopEntry>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        let attrs = meta.file_attributes();
        if attrs & (FILE_ATTRIBUTE_HIDDEN as u32 | FILE_ATTRIBUTE_SYSTEM as u32) != 0 {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.eq_ignore_ascii_case("desktop.ini") {
            continue;
        }
        out.push(DesktopEntry {
            path: entry.path(),
            file_name,
            origin,
            is_folder: meta.is_dir(),
            attributes: attrs,
            mtime: filetime_to_unix(meta.last_write_time()),
            size: meta.file_size(),
            created: filetime_to_unix(meta.creation_time()),
        });
    }
}

/// Reads one entry (for watcher events). `None` if it vanished or is hidden/system.
pub fn stat_entry(path: &Path, origin: EntryOrigin) -> Option<DesktopEntry> {
    let meta = std::fs::metadata(path).ok()?;
    let attrs = meta.file_attributes();
    if attrs & (FILE_ATTRIBUTE_HIDDEN as u32 | FILE_ATTRIBUTE_SYSTEM as u32) != 0 {
        return None;
    }
    let file_name = path.file_name()?.to_string_lossy().to_string();
    if file_name.eq_ignore_ascii_case("desktop.ini") {
        return None;
    }
    Some(DesktopEntry {
        path: path.to_path_buf(),
        file_name,
        origin,
        is_folder: meta.is_dir(),
        attributes: attrs,
        mtime: filetime_to_unix(meta.last_write_time()),
        size: meta.file_size(),
        created: filetime_to_unix(meta.creation_time()),
    })
}

fn path_wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn shell_item(path: &Path) -> Result<IShellItem> {
    let w = path_wide(path);
    // SAFETY: string outlives the call.
    unsafe { SHCreateItemFromParsingName(PCWSTR(w.as_ptr()), None) }
}

/// Explorer-style editing name: no `.lnk`, respects the "hide extensions" setting.
pub fn display_name(path: &Path) -> Option<String> {
    let item = shell_item(path).ok()?;
    // SAFETY: COM call on a live item.
    let name = unsafe { item.GetDisplayName(SIGDN_PARENTRELATIVEEDITING) }.ok()?;
    let s = take_string(name);
    (!s.is_empty()).then_some(s)
}

/// Opaque BGRA (premultiplied) image returned by the icon/thumbnail extractor.
#[derive(Clone, Debug)]
pub struct ShellImage {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

/// Gets the item's icon (or thumbnail, for media files) at `size_px`, premultiplied BGRA.
///
/// Uses `IShellItemImageFactory::GetImage`, the same source Explorer uses, so custom icons,
/// shortcut targets and image thumbnails all behave like the real desktop.
pub fn shell_image(path: &Path, size_px: u32, icon_only: bool) -> Result<ShellImage> {
    // Internet shortcuts first: the shell's own handler hands back the generic globe whenever
    // the `IconFile` .ico has no image at (or above) the requested size — Steam's 32 px icons
    // at 64 / 96 px, typically — while Explorer shows the icon scaled up. Reading the .url and
    // extracting the icon ourselves matches Explorer; anything odd falls through to the shell.
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("url"))
        && let Some(img) = url_icon(path, size_px)
    {
        return Ok(img);
    }
    let item = shell_item(path)?;
    let factory: IShellItemImageFactory = item.cast()?;
    let mut flags = SIIGBF_BIGGERSIZEOK as u32;
    if icon_only {
        flags |= SIIGBF_ICONONLY as u32;
    }
    // SAFETY: COM/GDI/WIC calls; the HBITMAP is deleted after conversion.
    unsafe {
        let hbitmap = factory.GetImage(
            SIZE {
                cx: size_px as i32,
                cy: size_px as i32,
            },
            flags as i32,
        )?;
        let result = hbitmap_to_bgra(hbitmap, size_px);
        let _ = DeleteObject(HGDIOBJ(hbitmap.0));
        result
    }
}

/// Converts the shell bitmap to premultiplied BGRA, minifying it to fit `max_px` on its long
/// side with WIC's Fant filter when the shell handed back a bigger image (`SIIGBF_BIGGERSIZEOK`
/// returns the 256 px jumbo icon / thumbnail for 64 and 96 px requests). Explorer downsamples
/// the jumbo list the same way; drawing the 256 px bitmap into a 96 px box with a 2x2 linear
/// tap would alias and shimmer instead. Exact-size returns are left untouched.
/// `IconFile=` / `IconIndex=` of an Internet shortcut, when the file names an icon.
fn url_icon_source(path: &Path) -> Option<(PathBuf, i32)> {
    let bytes = std::fs::read(path).ok()?;
    // .url files are ANSI or UTF-8 in practice; a UTF-16 one starts with a BOM.
    let text = if bytes.starts_with(&[0xFF, 0xFE]) {
        let units: Vec<u16> = bytes[2..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .collect();
        String::from_utf16_lossy(&units)
    } else {
        String::from_utf8_lossy(&bytes).into_owned()
    };
    let mut file = None;
    let mut index = 0;
    for line in text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("IconFile=") {
            let v = v.trim();
            if !v.is_empty() {
                file = Some(PathBuf::from(v));
            }
        } else if let Some(v) = line.strip_prefix("IconIndex=") {
            index = v.trim().parse().unwrap_or(0);
        }
    }
    file.map(|f| (f, index))
}

/// The icon of an Internet shortcut at `size_px`, extracted from its `IconFile` the way
/// Explorer does (`SHDefExtractIcon` picks the nearest image and scales it). None when the
/// .url names no icon, the icon file is gone or extraction fails.
fn url_icon(path: &Path, size_px: u32) -> Option<ShellImage> {
    let (file, index) = url_icon_source(path)?;
    if !file.exists() {
        return None;
    }
    let wide = path_wide(&file);
    let mut hicon = HICON::default();
    // SAFETY: NUL-terminated path outlives the call; the icon is destroyed after conversion.
    unsafe {
        let hr = SHDefExtractIconW(
            PCWSTR(wide.as_ptr()),
            index,
            0,
            Some(&mut hicon),
            None,
            size_px.clamp(16, 256),
        );
        if hr.is_err() || hicon.0.is_null() {
            return None;
        }
        let image = (|| -> Result<ShellImage> {
            let factory: IWICImagingFactory =
                CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
            let wic = factory.CreateBitmapFromHICON(hicon)?;
            wic_to_bgra(&factory, wic.cast()?, size_px)
        })();
        let _ = DestroyIcon(hicon);
        image.ok()
    }
}

unsafe fn hbitmap_to_bgra(hbitmap: HBITMAP, max_px: u32) -> Result<ShellImage> {
    // SAFETY: caller owns the bitmap for the duration of the call.
    unsafe {
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
        let wic = factory.CreateBitmapFromHBITMAP(
            hbitmap,
            HPALETTE::default(),
            WICBitmapUsePremultipliedAlpha,
        )?;
        wic_to_bgra(&factory, wic.cast()?, max_px)
    }
}

/// Minifies `source` to fit `max_px` (Fant) and converts it to tightly packed premultiplied
/// BGRA.
unsafe fn wic_to_bgra(
    factory: &IWICImagingFactory,
    mut source: IWICBitmapSource,
    max_px: u32,
) -> Result<ShellImage> {
    // SAFETY: WIC calls on live objects; the pixel buffer is sized from the reported size.
    unsafe {
        let (mut w, mut h) = (0u32, 0u32);
        source.GetSize(&mut w, &mut h).ok()?;
        if let Some((dst_w, dst_h)) = fit_within(w, h, max_px) {
            let scaler = factory.CreateBitmapScaler()?;
            scaler
                .Initialize(&source, dst_w, dst_h, WICBitmapInterpolationModeFant)
                .ok()?;
            source = scaler.cast()?;
            (w, h) = (dst_w, dst_h);
        }
        // Convert to PBGRA in case the DIB was 24-bit or straight alpha.
        let converter = factory.CreateFormatConverter()?;
        converter
            .Initialize(
                &source,
                &GUID_WICPixelFormat32bppPBGRA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeMedianCut,
            )
            .ok()?;
        let converted: IWICBitmapSource = converter.cast()?;
        let stride = w * 4;
        let mut bgra = vec![0u8; (stride * h) as usize];
        (Interface::vtable(&converted).CopyPixels)(
            Interface::as_raw(&converted),
            core::ptr::null(),
            stride,
            bgra.len() as u32,
            bgra.as_mut_ptr(),
        )
        .ok()?;
        Ok(ShellImage {
            width: w,
            height: h,
            bgra,
        })
    }
}

/// Size an `w` x `h` image shrinks to so its long side is at most `max_px` (aspect kept,
/// rounded, at least 1 px); None when it already fits (never upscales).
fn fit_within(w: u32, h: u32, max_px: u32) -> Option<(u32, u32)> {
    if max_px == 0 || w == 0 || h == 0 || (w <= max_px && h <= max_px) {
        return None;
    }
    let k = (max_px as f32 / w as f32).min(max_px as f32 / h as f32);
    Some((
        ((w as f32 * k).round() as u32).max(1),
        ((h as f32 * k).round() as u32).max(1),
    ))
}

/// Moves files/folders into `dest` with the shell's own file operation: progress UI, collision
/// prompts and Ctrl+Z undo in Explorer (plan §5.9 / §5.12 `IFileOperation`).
/// Returns `Ok(false)` when the user cancelled.
pub fn move_to_folder(paths: &[PathBuf], dest: &Path, owner: Option<HWND>) -> Result<bool> {
    transfer_to_folder(paths, dest, owner, false, false)
}

/// Copies files/folders into `dest` (Ctrl held while dropping, like Explorer).
pub fn copy_to_folder(paths: &[PathBuf], dest: &Path, owner: Option<HWND>) -> Result<bool> {
    transfer_to_folder(paths, dest, owner, true, false)
}

/// Copies into `dest` renaming on collision (`FOF_RENAMEONCOLLISION`): pasting a file into its
/// own folder yields Explorer's "name - 副本" duplicate instead of a prompt.
pub fn duplicate_in_folder(paths: &[PathBuf], dest: &Path, owner: Option<HWND>) -> Result<bool> {
    transfer_to_folder(paths, dest, owner, true, true)
}

/// One shell file operation (`IFileOperation`, with undo and the shell's own conflict /
/// progress UI): moves or copies `paths` into `dest`. Blocks until it finishes — callers on
/// the UI thread run it on a worker thread with its own STA (the operation takes hundreds of
/// milliseconds on a OneDrive folder). Returns false when the user aborted.
pub fn transfer_to_folder(
    paths: &[PathBuf],
    dest: &Path,
    owner: Option<HWND>,
    copy: bool,
    rename_on_collision: bool,
) -> Result<bool> {
    if paths.is_empty() {
        return Ok(true);
    }
    // SAFETY: COM calls on objects created here; the caller's thread has COM initialized.
    unsafe {
        let op: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_ALL as u32)?;
        let mut flags = (FOF_ALLOWUNDO | FOFX_ADDUNDORECORD | FOF_NOCONFIRMMKDIR) as u32;
        if rename_on_collision {
            flags |= FOF_RENAMEONCOLLISION as u32;
        }
        op.SetOperationFlags(flags).ok()?;
        if let Some(h) = owner {
            let _ = op.SetOwnerWindow(h);
        }
        let dest_item = shell_item(dest)?;
        for p in paths {
            let item = shell_item(p)?;
            if copy {
                op.CopyItem(&item, &dest_item, PCWSTR::null(), None).ok()?;
            } else {
                op.MoveItem(&item, &dest_item, PCWSTR::null(), None).ok()?;
            }
        }
        op.PerformOperations().ok()?;
        let aborted = op
            .GetAnyOperationsAborted()
            .map(|b| b.as_bool())
            .unwrap_or(false);
        Ok(!aborted)
    }
}

/// Opens an item with the shell (default verb), or runs `verb` (`"runas"`, `"properties"`,
/// `"openas"`…).
pub fn shell_execute(path: &Path, verb: Option<&str>, owner: Option<HWND>) -> Result<()> {
    let file = path_wide(path);
    let verb_w = verb.map(to_wide);
    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: (SEE_MASK_INVOKEIDLIST | SEE_MASK_NOASYNC) as u32,
        hwnd: owner.unwrap_or_default(),
        lpVerb: verb_w
            .as_ref()
            .map(|v| PCWSTR(v.as_ptr()))
            .unwrap_or(PCWSTR::null()),
        lpFile: PCWSTR(file.as_ptr()),
        nShow: SW_SHOWNORMAL,
        ..Default::default()
    };
    // SAFETY: all referenced buffers outlive the call.
    unsafe { ShellExecuteExW(&mut info).ok() }
}

/// Resolves a `.lnk` file's target path (raw, unexpanded), or `None`.
pub fn shortcut_target(path: &Path) -> Option<String> {
    let w = path_wide(path);
    // SAFETY: standard IShellLink usage; buffers are sized and NUL-terminated.
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let persist: IPersistFile = link.cast().ok()?;
        persist
            .Load(PCWSTR(w.as_ptr()), STGM_READ as u32)
            .ok()
            .ok()?;
        let mut buf = [0u16; 1024];
        link.GetPath(
            PWSTR(buf.as_mut_ptr()),
            buf.len() as i32,
            core::ptr::null_mut(),
            SLGP_RAWPATH,
        )
        .ok()
        .ok()?;
        let s = crate::wide::from_wide(&buf);
        (!s.is_empty()).then_some(s)
    }
}

/// Creates the `.lnk` Explorer writes for "在当前位置创建快捷方式": `<name> - 快捷方式.lnk` in
/// `dir` pointing at `target` (a file or a folder), with a ` (2)` … suffix on collision. Returns
/// the shortcut's path.
pub fn create_shortcut(target: &Path, dir: &Path) -> Result<PathBuf> {
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let base = if target.is_dir() {
        name
    } else {
        target
            .file_stem()
            .map(|n| n.to_string_lossy().into_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or(name)
    };
    let path = unique_path(
        dir,
        &pecofence_core::i18n::format("{0} - 快捷方式", std::slice::from_ref(&base)),
        ".lnk",
    );
    let target_w = path_wide(target);
    let dir_w = target.parent().map(path_wide);
    let path_w = path_wide(&path);
    // SAFETY: standard IShellLink usage; all strings outlive the calls.
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        link.SetPath(PCWSTR(target_w.as_ptr())).ok()?;
        if let Some(d) = &dir_w {
            let _ = link.SetWorkingDirectory(PCWSTR(d.as_ptr()));
        }
        let persist: IPersistFile = link.cast()?;
        persist.Save(PCWSTR(path_w.as_ptr()), true).ok()?;
    }
    Ok(path)
}

/// `dir\<base><ext>`, or `dir\<base> (n)<ext>` for the first free n ≥ 2 (Explorer's collision
/// naming).
pub fn unique_path(dir: &Path, base: &str, ext: &str) -> PathBuf {
    let mut path = dir.join(format!("{base}{ext}"));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{base} ({n}){ext}"));
        n += 1;
    }
    path
}

/// Reads a `.url` internet shortcut's URL.
pub fn url_shortcut_target(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    text.lines()
        .find_map(|l| l.strip_prefix("URL=").map(|u| u.trim().to_string()))
}

#[cfg(test)]
mod scale_tests {
    use super::fit_within;

    /// Only oversized shell images are minified: exact / smaller returns stay untouched, the
    /// aspect is kept and the long side lands on the requested size.
    #[test]
    fn fit_within_only_shrinks_oversized_images() {
        assert_eq!(fit_within(256, 256, 96), Some((96, 96)));
        assert_eq!(fit_within(256, 256, 64), Some((64, 64)));
        assert_eq!(fit_within(256, 144, 96), Some((96, 54)));
        assert_eq!(fit_within(144, 256, 96), Some((54, 96)));
        assert_eq!(fit_within(96, 96, 96), None);
        assert_eq!(fit_within(48, 48, 96), None, "never upscales");
        assert_eq!(fit_within(0, 256, 96), None);
        assert_eq!(fit_within(256, 256, 0), None);
        assert_eq!(fit_within(1000, 1, 96), Some((96, 1)));
    }
}

#[cfg(test)]
mod move_tests {
    use super::*;

    /// `IFileOperation` moves a file into a folder (headless: no collision, no prompt).
    #[test]
    fn move_to_folder_moves_a_file() {
        let _com = crate::com::OleGuard::init().expect("COM");
        let base = std::env::temp_dir().join(format!("pecofence-move-{}", std::process::id()));
        let src_dir = base.join("src");
        let dst_dir = base.join("dst");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&dst_dir).unwrap();
        let file = src_dir.join("probe.txt");
        std::fs::write(&file, b"x").unwrap();
        let ok = move_to_folder(std::slice::from_ref(&file), &dst_dir, None).expect("move");
        assert!(ok);
        assert!(!file.exists());
        assert!(dst_dir.join("probe.txt").exists());
        let _ = std::fs::remove_dir_all(&base);
    }

    /// The Link drop effect writes Explorer's `<stem> - 快捷方式.lnk` (folders keep their full
    /// name) and never overwrites an existing shortcut.
    #[test]
    fn create_shortcut_writes_explorer_named_lnk() {
        let _com = crate::com::OleGuard::init().expect("COM");
        let base = std::env::temp_dir().join(format!("pecofence-lnk-{}", std::process::id()));
        std::fs::create_dir_all(&base).unwrap();
        let file = base.join("probe.txt");
        std::fs::write(&file, b"x").unwrap();
        let lnk = create_shortcut(&file, &base).expect("shortcut");
        assert_eq!(lnk, base.join("probe - 快捷方式.lnk"));
        assert!(lnk.exists());
        // Canonicalize both sides: the temp directory may be an 8.3 short path (as on
        // CI runners) while the shell stores the long form in the link.
        let target = shortcut_target(&lnk).expect("target");
        assert_eq!(
            std::fs::canonicalize(&target).expect("canonical target"),
            std::fs::canonicalize(&file).expect("canonical file")
        );
        let second = create_shortcut(&file, &base).expect("shortcut 2");
        assert_eq!(second, base.join("probe - 快捷方式 (2).lnk"));
        let folder = base.join("sub.dir");
        std::fs::create_dir_all(&folder).unwrap();
        let flnk = create_shortcut(&folder, &base).expect("folder shortcut");
        assert_eq!(flnk, base.join("sub.dir - 快捷方式.lnk"));
        let _ = std::fs::remove_dir_all(&base);
    }
}

#[cfg(test)]
mod namespace_tests {
    use super::*;

    #[test]
    fn parsing_names_are_recognized() {
        assert!(is_namespace_path(Path::new(
            "::{645FF040-5081-101B-9F08-00AA002F954E}"
        )));
        assert!(!is_namespace_path(Path::new(
            r"C:\Users\me\Desktop\notes.txt"
        )));
        assert!(!is_namespace_path(Path::new("")));
        assert!(is_recycle_bin_path(Path::new(
            "::{645ff040-5081-101b-9f08-00aa002f954e}"
        )));
        assert!(!is_recycle_bin_path(Path::new(
            "::{20D04FE0-3AEA-1069-A2D8-08002B30309D}"
        )));
    }

    #[test]
    fn mixed_selections_keep_only_files_for_the_shell() {
        let bin = PathBuf::from(format!("::{RECYCLE_BIN_CLSID}"));
        let file = PathBuf::from(r"C:\Users\me\Desktop\notes.txt");
        assert_eq!(paths_for_shell(vec![bin.clone()]), vec![bin.clone()]);
        assert_eq!(paths_for_shell(vec![file.clone()]), vec![file.clone()]);
        assert_eq!(paths_for_shell(vec![bin, file.clone()]), vec![file]);
    }

    /// The Recycle Bin resolves through the same parsing-name path as files: display name and
    /// icon come back, so it can be drawn and launched like any other item.
    #[test]
    fn recycle_bin_resolves_like_a_file() {
        let _com = crate::com::OleGuard::init().expect("COM");
        let bin = PathBuf::from(format!("::{RECYCLE_BIN_CLSID}"));
        let name = display_name(&bin).expect("recycle bin display name");
        assert!(!name.is_empty() && !name.starts_with("::"), "{name}");
        let image = shell_image(&bin, 32, true).expect("recycle bin icon");
        assert!(image.width > 0 && image.height > 0);
        // Querying the bin never fails on a desktop session; the answer itself may be either.
        let _ = recycle_bin_has_items();
        assert_eq!(
            crate::fileinfo::type_name(&bin, false)
                .as_deref()
                .map(str::is_empty),
            Some(false),
            "the shell names the bin's type"
        );
    }

    /// Every special item comes back as a namespace entry with a shell-provided name.
    #[test]
    fn special_items_are_namespace_entries() {
        let _com = crate::com::OleGuard::init().expect("COM");
        for entry in enumerate_special_desktop_items() {
            assert!(is_namespace_path(&entry.path), "{:?}", entry.path);
            assert_eq!(entry.origin, EntryOrigin::Namespace);
            assert!(!entry.is_folder);
            assert!(!entry.file_name.is_empty());
        }
    }
}
