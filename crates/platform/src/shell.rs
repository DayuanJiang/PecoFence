//! Shell services: known folders, desktop entries, icons/thumbnails, launching, shortcut targets.

use crate::bindings::*;
use crate::wide::to_wide;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use windows_core::{GUID, Interface, PCWSTR, PWSTR, Result};

// FOLDERID_Desktop / FOLDERID_PublicDesktop (KnownFolders.h).
const FOLDERID_DESKTOP: GUID = GUID::from_u128(0xB4BFCC3A_DB2C_424C_B029_7FE99A87C641);
const FOLDERID_PUBLIC_DESKTOP: GUID = GUID::from_u128(0xC4AA340D_F20F_4863_AFEF_F87EF2E6BA25);
// FOLDERID_Documents / FOLDERID_Downloads / FOLDERID_Pictures (KnownFolders.h).
const FOLDERID_DOCUMENTS: GUID = GUID::from_u128(0xFDD39AD0_238F_46AF_ADB4_6C85480369C7);
const FOLDERID_DOWNLOADS: GUID = GUID::from_u128(0x374DE290_123F_4565_9164_39C4925E467B);
const FOLDERID_PICTURES: GUID = GUID::from_u128(0x33E28130_4E1E_4676_835A_98395C3BC3BB);

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

/// The user's Documents folder (may be redirected, e.g. OneDrive).
pub fn user_documents() -> Option<PathBuf> {
    known_folder(&FOLDERID_DOCUMENTS)
}

/// The user's Downloads folder.
pub fn user_downloads() -> Option<PathBuf> {
    known_folder(&FOLDERID_DOWNLOADS)
}

/// The user's Pictures folder (may be redirected, e.g. OneDrive).
pub fn user_pictures() -> Option<PathBuf> {
    known_folder(&FOLDERID_PICTURES)
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
    enumerate_desktop_reporting().0
}

/// [`enumerate_desktop`], plus the desktop folders it could not read in full (access denied,
/// an error part way): some of what they hold is missing from the list.
pub fn enumerate_desktop_reporting() -> (Vec<DesktopEntry>, Vec<EntryOrigin>) {
    let mut out = Vec::new();
    let mut unread = Vec::new();
    for (dir, origin) in [
        (user_desktop(), EntryOrigin::UserDesktop),
        (public_desktop(), EntryOrigin::PublicDesktop),
    ] {
        let Some(dir) = dir else { continue };
        if !enumerate_dir_into(&dir, origin, &mut out) {
            unread.push(origin);
        }
    }
    (out, unread)
}

/// The shell namespace items Windows can draw on the desktop and whether Windows shows them
/// by default: `(CLSID, shown by default)`. Users toggle them in "Desktop icon settings".
const SPECIAL_DESKTOP_ITEMS: [(&str, bool); 6] = [
    (RECYCLE_BIN_CLSID, true),
    ("{20D04FE0-3AEA-1069-A2D8-08002B30309D}", false), // This PC
    ("{59031a47-3f72-44a7-89c5-5595fe6b30ee}", false), // User's Files
    ("{F02C1A0D-BE21-4350-88B0-7367FC96EF3C}", false), // Network
    ("{5399E694-6CE5-4D6C-8FCE-1D8870FDCBA0}", false), // Control Panel
    (SPOTLIGHT_CLSID, true),
];
pub const RECYCLE_BIN_CLSID: &str = "{645FF040-5081-101B-9F08-00AA002F954E}";
/// Windows Spotlight's "Learn about this picture": Windows adds it to the desktop namespace
/// only while Spotlight is the background.
const SPOTLIGHT_CLSID: &str = "{2CC5CA98-6485-489A-920E-B3E88A6CCCE3}";
const DESKTOP_NAMESPACE_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\Desktop\NameSpace";
const DESKTOP_ICON_SETTINGS_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\HideDesktopIcons\NewStartPanel";
/// Group Policy "Remove X icon from desktop": `{CLSID} = 1` here hides it regardless of the
/// user's own choice.
const NON_ENUM_POLICY_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Policies\NonEnum";

/// True for a shell parsing name of a namespace item (`::{CLSID}`) rather than a file path.
pub fn is_namespace_path(path: &Path) -> bool {
    path.to_string_lossy().starts_with("::{")
}

/// True for a path on a local fixed drive (`C:\...`): asking whether it exists is quick. A
/// share or a mapped network drive may keep the caller waiting until the network gives up.
pub fn is_on_fixed_drive(path: &Path) -> bool {
    let s = path.to_string_lossy();
    let mut chars = s.chars();
    let (Some(letter), Some(':')) = (chars.next(), chars.next()) else {
        return false;
    };
    let root = to_wide(&format!("{letter}:\\"));
    // SAFETY: the string is NUL-terminated and outlives the call.
    unsafe { GetDriveTypeW(PCWSTR(root.as_ptr())) == DRIVE_FIXED as u32 }
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

/// Whether `clsid` is registered in the desktop's namespace (HKCU or HKLM).
fn in_desktop_namespace(clsid: &str) -> bool {
    let path = to_wide(&format!(r"{DESKTOP_NAMESPACE_KEY}\{clsid}"));
    [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE]
        .into_iter()
        .any(|root| {
            let mut key = HKEY::default();
            // SAFETY: the string outlives the call; `key` is a local out-pointer, closed below.
            unsafe {
                let status = RegOpenKeyExW(
                    root,
                    PCWSTR(path.as_ptr()),
                    None,
                    ACCESS_MASK(KEY_QUERY_VALUE as u32),
                    &mut key,
                );
                if status.0 != 0 {
                    return false;
                }
                let _ = RegCloseKey(key);
            }
            true
        })
}

/// The Recycle Bin is asked again (see `recycle_bin_has_items`) only after this (a folder
/// change, the shell's notice, a refresh) or ten minutes after the last answer.
pub fn recycle_bin_changed() {
    RECYCLE_BIN_STALE.store(true, std::sync::atomic::Ordering::Relaxed);
}

static RECYCLE_BIN_STALE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Whether the Recycle Bin holds anything (decides between the full and empty icon). Each
/// fixed drive is asked on its own: the all-drives form fails as a whole when one volume
/// (card reader without media, offline mapped drive) cannot answer. When no drive answers,
/// the last known state is kept rather than flipping to "empty". Asking touches every fixed
/// drive (a sleeping one wakes, which can take seconds), so the answer is reused until
/// `recycle_bin_changed` or for ten minutes.
pub fn recycle_bin_has_items() -> bool {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};
    static LAST_KNOWN: AtomicBool = AtomicBool::new(false);
    static ASKED: Mutex<Option<Instant>> = Mutex::new(None);
    let mut asked = ASKED.lock().unwrap_or_else(|e| e.into_inner());
    let recent = asked.is_some_and(|t| t.elapsed() < Duration::from_secs(600));
    if recent && !RECYCLE_BIN_STALE.swap(false, Ordering::Relaxed) {
        return LAST_KNOWN.load(Ordering::Relaxed);
    }
    RECYCLE_BIN_STALE.store(false, Ordering::Relaxed);
    *asked = Some(Instant::now());
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
        if desktop_icon_hidden_in_windows(clsid, default_visible)
            || (clsid == SPOTLIGHT_CLSID && !in_desktop_namespace(clsid))
        {
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

/// False when `dir` could not be read in full.
fn enumerate_dir_into(dir: &Path, origin: EntryOrigin, out: &mut Vec<DesktopEntry>) -> bool {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return false;
    };
    let mut complete = true;
    for entry in rd {
        let Ok(entry) = entry else {
            return false;
        };
        let Ok(meta) = entry.metadata() else {
            complete = false;
            continue;
        };
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
    complete
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

/// [`display_name`] of every item the shell lists in `dir`, keyed by lowercase file name. One
/// bind of the folder instead of a parse of each full path (several ms per item on OneDrive).
pub fn display_names_in(dir: &Path) -> std::collections::HashMap<String, String> {
    let mut names = std::collections::HashMap::new();
    let Ok(folder) = shell_item(dir) else {
        return names;
    };
    // SAFETY: COM calls on live interfaces; every returned string is freed by `take_string`.
    unsafe {
        let items: Result<IEnumShellItems> = folder.BindToHandler(None, &BHID_EnumItems);
        let Ok(items) = items else {
            return names;
        };
        loop {
            let mut item: Option<IShellItem> = None;
            if items.Next(1, &mut item, None) != S_OK {
                break;
            }
            let Some(item) = item else { break };
            let file = item
                .GetDisplayName(SIGDN_PARENTRELATIVEPARSING)
                .map(take_string);
            let edit = item
                .GetDisplayName(SIGDN_PARENTRELATIVEEDITING)
                .map(take_string);
            if let (Ok(file), Ok(edit)) = (file, edit)
                && !file.is_empty()
                && !edit.is_empty()
            {
                names.insert(file.to_lowercase(), edit);
            }
        }
    }
    names
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
    let text = url_file_text(path)?;
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

/// The text of an Internet shortcut (`.url`).
fn url_file_text(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    // .url files are ANSI or UTF-8 in practice; a UTF-16 one starts with a BOM.
    Some(if bytes.starts_with(&[0xFF, 0xFE]) {
        let units: Vec<u16> = bytes[2..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .collect();
        String::from_utf16_lossy(&units)
    } else {
        match String::from_utf8(bytes) {
            Ok(text) => text,
            // Not UTF-8: the system ANSI code page (GBK on a Chinese Windows).
            Err(e) => ansi_to_string(e.as_bytes()),
        }
    })
}

/// Decodes `bytes` in the system ANSI code page.
fn ansi_to_string(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    // SAFETY: the first call only measures; the second writes at most `n` units into `w`.
    unsafe {
        let text = bytes.as_ptr().cast();
        let len = bytes.len() as i32;
        let n = MultiByteToWideChar(CP_ACP as u32, 0, text, len, None, 0);
        if n <= 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }
        let mut w = vec![0u16; n as usize];
        let n = MultiByteToWideChar(CP_ACP as u32, 0, text, len, Some(PWSTR(w.as_mut_ptr())), n);
        String::from_utf16_lossy(&w[..n.max(0) as usize])
    }
}

/// The icon of an Internet shortcut at `size_px`, extracted from its `IconFile`. None when the
/// .url names no icon, the icon file is gone or extraction fails.
fn url_icon(path: &Path, size_px: u32) -> Option<ShellImage> {
    let (file, index) = url_icon_source(path)?;
    icon_file_image(&file, index, size_px)
}

/// The Recycle Bin's `full` or empty icon at `size_px` (see `recycle_bin_icon_source`); None
/// when neither DefaultIcon key names one that can be extracted (`shell_image` then).
pub fn recycle_bin_image(full: bool, size_px: u32) -> Option<ShellImage> {
    let (file, index) = recycle_bin_icon_source(full)?;
    icon_file_image(&file, index, size_px)
}

/// The Recycle Bin's full or empty icon (`file`, `index`) from the `full` / `empty` entries of
/// its DefaultIcon key, the user's own (Desktop icon settings) before the Windows default. The
/// shell's extraction reads the key's default value instead, which Explorer switches only some
/// time after the bin filled or emptied: asked right after a file was recycled it still hands
/// back the empty bin, and that image would stay cached as the full one.
fn recycle_bin_icon_source(full: bool) -> Option<(PathBuf, i32)> {
    let name = if full { "full" } else { "empty" };
    [
        (
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\CLSID",
        ),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Classes\CLSID"),
    ]
    .into_iter()
    .find_map(|(root, clsids)| {
        let key = format!(r"{clsids}\{RECYCLE_BIN_CLSID}\DefaultIcon");
        let location = reg_string(root, &key, name)?;
        let (file, index) = location.rsplit_once(',').unwrap_or((&location, "0"));
        Some((
            PathBuf::from(file.trim().trim_matches('"')),
            index.trim().parse().ok()?,
        ))
    })
}

/// Reads `HKxx\key\name` as a string, environment variables expanded; `None` when absent or
/// empty.
fn reg_string(root: HKEY, key: &str, name: &str) -> Option<String> {
    const RRF_RT_REG_SZ: u32 = 0x02;
    let key = to_wide(key);
    let name = to_wide(name);
    let mut buf = [0u16; 1024];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: the strings outlive the call; `buf` is a live out buffer of `size` bytes.
    let status = unsafe {
        RegGetValueW(
            root,
            PCWSTR(key.as_ptr()),
            PCWSTR(name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    if status.0 != 0 {
        return None;
    }
    // `size` can be that of the unexpanded value, shorter than the expanded one when a
    // variable stands for a longer path: the string ends at the first NUL of the zeroed buffer.
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    let s = String::from_utf16_lossy(&buf[..len]);
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_string())
}

/// Icon `index` of `file` at `size_px`, extracted the way Explorer does (`SHDefExtractIcon`
/// picks the nearest image and scales it). None when the file is gone or extraction fails.
fn icon_file_image(file: &Path, index: i32, size_px: u32) -> Option<ShellImage> {
    if !file.exists() {
        return None;
    }
    let wide = path_wide(file);
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

/// `SFGAO_LINK`: the shell draws its shortcut arrow on the item.
const SFGAO_LINK: u32 = 0x0001_0000;
/// `SIID_LINK` / `SHGSI_ICONLOCATION` (shellapi.h).
const SIID_LINK: i32 = 29;
const SHGSI_ICONLOCATION: u32 = 0;

/// `SHSTOCKICONINFO` (shellapi.h).
#[repr(C)]
struct StockIconInfo {
    size: u32,
    icon: HICON,
    system_image_index: i32,
    icon_index: i32,
    path: [u16; 260],
}

windows_core::link!("shell32.dll" "system" fn SHGetStockIconInfo(siid: i32, flags: u32, info: *mut StockIconInfo) -> windows_core::HRESULT);

/// True when Explorer draws its shortcut arrow on the item: the shell reports it as a link
/// (`SFGAO_LINK`), which .lnk, .url, .appref-ms and every other file type marked `IsShortcut`
/// are.
pub fn is_link(path: &Path) -> bool {
    shell_item(path)
        // SAFETY: COM call on a live item.
        .and_then(|item| unsafe { item.GetAttributes(SFGAOF(SFGAO_LINK)) })
        .is_ok_and(|a| a.0 & SFGAO_LINK != 0)
}

/// Draws Windows' shortcut arrow over `image` at `size_px`, aligned to its bottom-left like
/// Explorer's overlay: the icon `SHGetStockIconInfo(SIID_LINK)` names (the user's own arrow, or
/// a blank one that removes it, when `Shell Icons` value 29 is set). The arrow is extracted once
/// per size on each worker thread.
pub fn add_link_overlay(image: &mut ShellImage, size_px: u32) {
    thread_local! {
        static OVERLAYS: std::cell::RefCell<std::collections::HashMap<u32, Option<ShellImage>>> =
            Default::default();
    }
    OVERLAYS.with_borrow_mut(|overlays| {
        let overlay = overlays.entry(size_px).or_insert_with(|| {
            let (file, index) = link_overlay_source()?;
            icon_file_image(&file, index, size_px)
        });
        if let Some(o) = overlay {
            draw_over_bottom_left(image, o);
        }
    });
}

/// The icon file and index of the shell's shortcut arrow.
fn link_overlay_source() -> Option<(PathBuf, i32)> {
    // SAFETY: plain-data struct; all-zero is a valid value.
    let mut info: StockIconInfo = unsafe { std::mem::zeroed() };
    info.size = size_of::<StockIconInfo>() as u32;
    // SAFETY: `info` is a live out-struct of the declared size.
    unsafe { SHGetStockIconInfo(SIID_LINK, SHGSI_ICONLOCATION, &mut info) }
        .ok()
        .ok()?;
    let len = info.path.iter().position(|&c| c == 0).unwrap_or(0);
    (len > 0).then(|| {
        (
            PathBuf::from(String::from_utf16_lossy(&info.path[..len])),
            info.icon_index,
        )
    })
}

/// Premultiplied source-over of `top` onto `image`, their bottom-left corners aligned, clipped
/// to `image`.
fn draw_over_bottom_left(image: &mut ShellImage, top: &ShellImage) {
    let dy = image.height as i32 - top.height as i32;
    let w = top.width.min(image.width) as usize;
    for ty in 0..top.height as i32 {
        let iy = dy + ty;
        if iy < 0 || iy >= image.height as i32 {
            continue;
        }
        for x in 0..w {
            let s = (ty as usize * top.width as usize + x) * 4;
            let d = (iy as usize * image.width as usize + x) * 4;
            let keep = 255 - top.bgra[s + 3] as u32;
            for c in 0..4 {
                let v = top.bgra[s + c] as u32 + (image.bgra[d + c] as u32 * keep + 127) / 255;
                image.bgra[d + c] = v.min(255) as u8;
            }
        }
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
        let mut image = wic_to_bgra(&factory, wic.cast()?, max_px)?;
        premultiply_if_straight(&mut image.bgra);
        Ok(image)
    }
}

/// `IShellItemImageFactory::GetImage` hands icons back with straight alpha, although the bitmap
/// is read as premultiplied (thumbnails can be either). Straight data shows itself by a colour
/// channel above its alpha, which premultiplied data cannot have; left as is, those anti-aliased
/// edge pixels draw too bright and the icon outline looks jagged.
fn premultiply_if_straight(bgra: &mut [u8]) {
    let straight = bgra
        .as_chunks::<4>()
        .0
        .iter()
        .any(|p| p[0].max(p[1]).max(p[2]) > p[3]);
    if !straight {
        return;
    }
    for p in bgra.as_chunks_mut::<4>().0 {
        let a = p[3] as u32;
        for c in &mut p[..3] {
            *c = ((*c as u32 * a + 127) / 255) as u8;
        }
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
        // Cancel in the shell's conflict or error dialog, or a declined administrator prompt,
        // fails the operation too: report that as a cancel, not an error.
        let performed = op.PerformOperations();
        if op.GetAnyOperationsAborted().is_ok_and(|b| b.as_bool())
            || performed == COPYENGINE_E_USER_CANCELLED
            || performed == E_CANCELLED
        {
            return Ok(false);
        }
        performed.ok()?;
        Ok(true)
    }
}

const COPYENGINE_E_USER_CANCELLED: windows_core::HRESULT =
    windows_core::HRESULT(0x8027_0000_u32 as i32);
/// HRESULT_FROM_WIN32(ERROR_CANCELLED).
const E_CANCELLED: windows_core::HRESULT = windows_core::HRESULT(0x8007_04C7_u32 as i32);

/// Renames `path` to `new_name` (the complete file name) with the shell's file operation, so
/// a folder the user may not write to (the Public Desktop) gets Explorer's own
/// administrator-permission prompt. Blocks like [`transfer_to_folder`]; returns false when
/// the user cancelled.
pub fn rename_with_prompt(path: &Path, new_name: &str, owner: Option<HWND>) -> Result<bool> {
    let name = to_wide(new_name);
    // SAFETY: COM calls on objects created here; the caller's thread has COM initialized and
    // `name` outlives the operation.
    unsafe {
        let op: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_ALL as u32)?;
        op.SetOperationFlags((FOF_ALLOWUNDO | FOFX_ADDUNDORECORD) as u32)
            .ok()?;
        if let Some(h) = owner {
            let _ = op.SetOwnerWindow(h);
        }
        op.RenameItem(&shell_item(path)?, PCWSTR(name.as_ptr()), None)
            .ok()?;
        // Declining the prompt fails the operation too: report that as a cancel, not an error.
        let performed = op.PerformOperations();
        if op.GetAnyOperationsAborted().is_ok_and(|b| b.as_bool()) {
            return Ok(false);
        }
        performed.ok()?;
        Ok(true)
    }
}

/// `path` for a file API: as is when short, beyond that in the `\\?\` form `std::fs` uses
/// (`\\?\C:\…`, `\\?\UNC\server\share\…`), without which the call fails on a long path. The
/// threshold is std's: 248 UTF-16 units including the NUL (the limit for a folder).
fn long_path_wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Component, Prefix};
    let plain = path_wide(path);
    if plain.len() < 248 {
        return plain;
    }
    // The prefix turns normalisation off, so the path must be absolute and clean first.
    let abs = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let abs_w: Vec<u16> = abs.as_os_str().encode_wide().collect();
    let mut w: Vec<u16> = r"\\?\".encode_utf16().collect();
    match abs.components().next() {
        Some(Component::Prefix(p)) => match p.kind() {
            Prefix::Disk(_) => w.extend(&abs_w),
            // `\\server\share\…` keeps one of its leading backslashes.
            Prefix::UNC(..) => {
                w.extend("UNC".encode_utf16());
                w.extend(&abs_w[1..]);
            }
            _ => return plain,
        },
        _ => return plain,
    }
    w.push(0);
    w
}

/// Renames `from` to `to`. `std::fs::rename` replaces a file already at `to`; this fails with
/// `ErrorKind::AlreadyExists` instead, so a rename never overwrites another file.
pub fn rename_no_replace(from: &Path, to: &Path) -> std::io::Result<()> {
    let (from, to) = (long_path_wide(from), long_path_wide(to));
    // SAFETY: plain FFI call; both strings are NUL-terminated and outlive it.
    let ok = unsafe { MoveFileExW(PCWSTR(from.as_ptr()), PCWSTR(to.as_ptr()), 0) };
    if ok.as_bool() {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
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
    shortcut_info(path).map(|(target, _)| target)
}

/// A `.lnk`'s target path and its command-line arguments (`None` when it has none).
pub fn shortcut_info(path: &Path) -> Option<(String, Option<String>)> {
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
        let target = crate::wide::from_wide(&buf);
        if target.is_empty() {
            return None;
        }
        // Longer arguments come back truncated, which is enough to tell shortcuts apart.
        let mut args = [0u16; 1024];
        let arguments = link
            .GetArguments(PWSTR(args.as_mut_ptr()), args.len() as i32)
            .ok()
            .ok()
            .map(|_| crate::wide::from_wide(&args).trim().to_string())
            .filter(|a| !a.is_empty());
        Some((target, arguments))
    }
}

/// Creates the `.lnk` Explorer writes for "在当前位置创建快捷方式": `<name> - 快捷方式.lnk` in
/// `dir` pointing at `target` (a file or a folder), with a ` (2)` … suffix on collision. Returns
/// the shortcut's path. A target that is no absolute file-system path (the Start menu's
/// `{known-folder GUID}\Steam\steam.exe`) is refused: `SetPath` would write a link to nowhere.
pub fn create_shortcut(target: &Path, dir: &Path) -> Result<PathBuf> {
    if !target.is_absolute() && !is_namespace_path(target) {
        return Err(windows_core::Error::from_hresult(E_INVALIDARG));
    }
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
    // Named in the Windows display language, as Explorer names the shortcuts it makes: the
    // file stays on disk for every other program, whatever language PecoFence shows.
    use pecofence_core::i18n;
    let windows_language = i18n::Language::from_locale(crate::locale::ui_language());
    let path = unique_path(
        dir,
        &i18n::interpolate(
            i18n::text_in(windows_language, "{0} - 快捷方式"),
            std::slice::from_ref(&base),
        ),
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
    let text = url_file_text(path)?;
    text.lines()
        .find_map(|l| l.strip_prefix("URL=").map(|u| u.trim().to_string()))
}

#[cfg(test)]
mod url_tests {
    use super::url_shortcut_target;

    /// PowerShell 5.1 writes UTF-16 with a BOM; an ANSI file may hold bytes that are not UTF-8.
    #[test]
    fn url_target_reads_utf16_and_ansi_files() {
        let dir = std::env::temp_dir().join(format!("pecofence-url-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let utf16 = dir.join("utf16.url");
        let mut bytes = vec![0xFF, 0xFE];
        for unit in "[InternetShortcut]\r\nURL=https://github.com/\r\n".encode_utf16() {
            bytes.extend(unit.to_le_bytes());
        }
        std::fs::write(&utf16, bytes).unwrap();
        // The URL ends in 桌面 in GBK: decoded in the system code page, whichever it is, not
        // as broken UTF-8.
        let ansi = dir.join("ansi.url");
        std::fs::write(
            &ansi,
            b"[InternetShortcut]\r\nURL=https://example.com/\xd7\xc0\xc3\xe6\r\nIconFile=C:\\a.ico\r\n",
        )
        .unwrap();
        assert_eq!(
            url_shortcut_target(&utf16).as_deref(),
            Some("https://github.com/")
        );
        let target = url_shortcut_target(&ansi).unwrap();
        let tail = target.strip_prefix("https://example.com/").unwrap();
        assert!(!tail.is_empty() && !tail.contains('\u{FFFD}'), "{target:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod path_tests {
    use super::long_path_wide;
    use std::path::Path;

    /// Only a long path (248 UTF-16 units with the NUL, as in std) gets the prefix; a network
    /// path keeps one backslash.
    #[test]
    fn long_paths_get_the_verbatim_prefix() {
        let wide = |s: &str| {
            s.encode_utf16()
                .chain(std::iter::once(0))
                .collect::<Vec<_>>()
        };
        let short = r"C:\Users\Me\Desktop\note.txt";
        assert_eq!(long_path_wide(Path::new(short)), wide(short));
        let deep = format!(r"C:\Users\Me\Desktop\{}\note.txt", "x".repeat(250));
        assert_eq!(
            long_path_wide(Path::new(&deep)),
            wide(&format!(r"\\?\{deep}"))
        );
        let unc = format!(r"\\server\share\{}\note.txt", "x".repeat(250));
        assert_eq!(
            long_path_wide(Path::new(&unc)),
            wide(&format!(r"\\?\UNC\{}", &unc[2..]))
        );
        let verbatim = format!(r"\\?\{deep}");
        assert_eq!(long_path_wide(Path::new(&verbatim)), wide(&verbatim));
    }
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

    /// Straight-alpha edge pixels get premultiplied; already premultiplied data is left alone.
    #[test]
    fn premultiply_if_straight_fixes_only_straight_data() {
        use super::premultiply_if_straight;
        let mut straight = vec![255, 128, 0, 128, 10, 20, 30, 255, 200, 200, 200, 0];
        premultiply_if_straight(&mut straight);
        assert_eq!(straight, vec![128, 64, 0, 128, 10, 20, 30, 255, 0, 0, 0, 0]);
        let mut premultiplied = vec![100, 64, 0, 128, 10, 20, 30, 255, 0, 0, 0, 0];
        let before = premultiplied.clone();
        premultiply_if_straight(&mut premultiplied);
        assert_eq!(premultiplied, before);
    }
}

#[cfg(test)]
mod link_overlay_tests {
    use super::*;

    #[test]
    fn overlays_are_drawn_at_the_bottom_left() {
        // A 4x4 opaque blue image under a 2x2 overlay: one opaque white pixel, one half-covered
        // white pixel and two transparent ones.
        let mut image = ShellImage {
            width: 4,
            height: 4,
            bgra: [255, 0, 0, 255].repeat(16),
        };
        let top = ShellImage {
            width: 2,
            height: 2,
            bgra: [[255, 255, 255, 255], [128, 128, 128, 128], [0; 4], [0; 4]].concat(),
        };
        draw_over_bottom_left(&mut image, &top);
        let px = |x: usize, y: usize| &image.bgra[(y * 4 + x) * 4..][..4];
        assert_eq!(px(0, 2), [255, 255, 255, 255]);
        assert_eq!(px(1, 2), [255, 128, 128, 255]);
        assert_eq!(px(0, 3), [255, 0, 0, 255]);
        assert_eq!(px(0, 0), [255, 0, 0, 255]);
        assert_eq!(px(3, 3), [255, 0, 0, 255]);
    }

    #[test]
    fn shortcuts_are_links_and_the_arrow_has_a_source() {
        let _com = crate::com::OleGuard::init().expect("COM");
        let dir = std::env::temp_dir().join(format!("pf-link-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.txt");
        std::fs::write(&file, "x").unwrap();
        let link = create_shortcut(&file, &dir).unwrap();
        assert!(is_link(&link));
        assert!(!is_link(&file));
        assert!(!is_link(&dir));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(link_overlay_source().is_some_and(|(file, _)| file.exists()));
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
    /// name), in the Windows display language, and never overwrites an existing shortcut.
    #[test]
    fn create_shortcut_writes_explorer_named_lnk() {
        use pecofence_core::i18n;
        let _com = crate::com::OleGuard::init().expect("COM");
        let language = i18n::Language::from_locale(crate::locale::ui_language());
        let named = |stem: &str| {
            i18n::interpolate(
                i18n::text_in(language, "{0} - 快捷方式"),
                &[stem.to_string()],
            )
        };
        let base = std::env::temp_dir().join(format!("pecofence-lnk-{}", std::process::id()));
        std::fs::create_dir_all(&base).unwrap();
        let file = base.join("probe.txt");
        std::fs::write(&file, b"x").unwrap();
        let lnk = create_shortcut(&file, &base).expect("shortcut");
        assert_eq!(lnk, base.join(format!("{}.lnk", named("probe"))));
        assert!(lnk.exists());
        // Canonicalize both sides: the temp directory may be an 8.3 short path (as on
        // CI runners) while the shell stores the long form in the link.
        let target = shortcut_target(&lnk).expect("target");
        assert_eq!(
            std::fs::canonicalize(&target).expect("canonical target"),
            std::fs::canonicalize(&file).expect("canonical file")
        );
        let second = create_shortcut(&file, &base).expect("shortcut 2");
        assert_eq!(second, base.join(format!("{} (2).lnk", named("probe"))));
        let folder = base.join("sub.dir");
        std::fs::create_dir_all(&folder).unwrap();
        let flnk = create_shortcut(&folder, &base).expect("folder shortcut");
        assert_eq!(flnk, base.join(format!("{}.lnk", named("sub.dir"))));
        // The Start menu's known-folder-relative app path is no file: no link to nowhere.
        let start_menu = Path::new(r"{7C5A40EF-A0FB-4BFC-874A-C0F2E0B9FA8E}\Steam\steam.exe");
        assert!(create_shortcut(start_menu, &base).is_err());
        assert!(!base.join(format!("{}.lnk", named("steam"))).exists());
        let _ = std::fs::remove_dir_all(&base);
    }

    /// The shell rename takes the complete file name: a shortcut's hidden `.lnk` is not added
    /// a second time.
    #[test]
    fn rename_with_prompt_takes_the_full_file_name() {
        let _com = crate::com::OleGuard::init().expect("COM");
        let base = std::env::temp_dir().join(format!("pecofence-ren-{}", std::process::id()));
        std::fs::create_dir_all(&base).unwrap();
        let file = base.join("probe.txt");
        std::fs::write(&file, b"x").unwrap();
        let lnk = create_shortcut(&file, &base).expect("shortcut");
        assert!(rename_with_prompt(&file, "renamed.txt", None).expect("rename file"));
        assert!(rename_with_prompt(&lnk, "renamed.lnk", None).expect("rename link"));
        let mut names: Vec<String> = std::fs::read_dir(&base)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, ["renamed.lnk", "renamed.txt"]);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn rename_no_replace_keeps_an_existing_target() {
        let base = std::env::temp_dir().join(format!("pecofence-mv-{}", std::process::id()));
        std::fs::create_dir_all(&base).unwrap();
        let (a, b) = (base.join("a.txt"), base.join("b.txt"));
        std::fs::write(&a, b"a").unwrap();
        std::fs::write(&b, b"b").unwrap();
        let err = rename_no_replace(&a, &b).expect_err("b.txt must not be replaced");
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read(&b).unwrap(), b"b");
        // A case-only rename is the same file and goes through.
        rename_no_replace(&a, &base.join("A.txt")).expect("case change");
        rename_no_replace(&base.join("A.txt"), &base.join("c.txt")).expect("plain rename");
        assert_eq!(std::fs::read(base.join("c.txt")).unwrap(), b"a");
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

    /// The bin's full and empty icons come from its own DefaultIcon entries, not from whatever
    /// state the shell last noticed.
    #[test]
    fn recycle_bin_has_a_full_and_an_empty_icon() {
        let _com = crate::com::OleGuard::init().expect("COM");
        let full = recycle_bin_icon_source(true).expect("full icon entry");
        let empty = recycle_bin_icon_source(false).expect("empty icon entry");
        assert_ne!(full, empty);
        for (file, index) in [full, empty] {
            let image = icon_file_image(&file, index, 48).expect("bin icon");
            assert_eq!((image.width, image.height), (48, 48), "{file:?},{index}");
        }
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
