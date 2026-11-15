//! The OLE clipboard as Explorer uses it: a `CF_HDROP` file list plus the shell's
//! `Preferred DropEffect` word that distinguishes 剪切 (move) from 复制 (copy).

use crate::bindings::*;
use crate::dragdrop;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use windows_core::{PCWSTR, Result};

static PREFERRED_EFFECT_FORMAT: OnceLock<u16> = OnceLock::new();
static PASTE_SUCCEEDED_FORMAT: OnceLock<u16> = OnceLock::new();
/// `CF_DIB`: a picture (a browser's 「复制图片」).
const CF_DIB: u16 = 8;

/// Files currently on the clipboard and whether they were cut (`true`) or copied (`false`).
/// A missing `Preferred DropEffect` counts as copy, as in Explorer.
pub fn file_list() -> Option<(Vec<PathBuf>, bool)> {
    // SAFETY: plain OLE call; the returned object is released when dropped.
    let data = unsafe { OleGetClipboard() }.ok()?;
    if !dragdrop::has_hdrop(&data) {
        return None;
    }
    let paths = dragdrop::hdrop_paths(&data);
    if paths.is_empty() {
        return None;
    }
    let cut = is_cut(&data);
    Some((paths, cut))
}

/// Did the source cut (`Preferred DropEffect` says move and not copy)?
fn is_cut(data: &IDataObject) -> bool {
    let fmt = dragdrop::hglobal_format(dragdrop::registered_format(
        "Preferred DropEffect",
        &PREFERRED_EFFECT_FORMAT,
    ));
    let effect = dragdrop::hglobal_bytes(data, &fmt)
        .filter(|b| b.len() >= 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .unwrap_or(DROPEFFECT_COPY as u32);
    effect & DROPEFFECT_MOVE as u32 != 0 && effect & DROPEFFECT_COPY as u32 == 0
}

/// The clipboard's data object when it holds items only the shell can write out and no file
/// list: files copied inside a .zip, an Outlook attachment, a phone's photos
/// ([`dragdrop::has_shell_items`]: their contents or shell items, no file paths). With whether
/// they were cut.
pub fn shell_items() -> Option<(IDataObject, bool)> {
    // SAFETY: plain OLE call; the returned object is released when dropped.
    let data = unsafe { OleGetClipboard() }.ok()?;
    if dragdrop::has_hdrop(&data) || !dragdrop::has_shell_items(&data) {
        return None;
    }
    let cut = is_cut(&data);
    Some((data, cut))
}

/// What the clipboard holds, for the log line of a paste that found nothing to paste: which of
/// a few standard kinds it offers (no contents).
pub fn describe() -> String {
    // SAFETY: plain OLE call.
    let Ok(data) = (unsafe { OleGetClipboard() }) else {
        return "unreadable".into();
    };
    let has = |cf: u16| {
        let fmt = dragdrop::hglobal_format(cf);
        // SAFETY: COM call with a fully initialized FORMATETC.
        unsafe { data.QueryGetData(&fmt).is_ok() }
    };
    let kinds = [
        ("files", dragdrop::has_hdrop(&data)),
        ("shell items", dragdrop::has_shell_items(&data)),
        ("text", has(CF_UNICODETEXT as u16)),
        ("image", has(CF_DIB)),
    ];
    let found: Vec<&str> = kinds.iter().filter(|k| k.1).map(|k| k.0).collect();
    if found.is_empty() {
        "nothing known".into()
    } else {
        found.join(", ")
    }
}

/// Puts the files `paths` on the clipboard as Explorer's Ctrl+C / Ctrl+X does: the shell's own
/// data object of the items (`CF_HDROP`, the shell's id list and its other formats) with
/// `Preferred DropEffect` saying copy or, for a cut, move. Every program pastes from it as from
/// a copy made in Explorer.
///
/// The items' `copy` and `cut` verbs end in the same call, but for a verb the shell first loads
/// the menu extension of every installed program, here on the UI thread at every key press
/// (see `shell::delete_items`).
pub fn set_files(paths: &[&Path], cut: bool) -> Result<()> {
    let data = dragdrop::data_object_for_paths(paths)?;
    let effect = if cut {
        DROPEFFECT_MOVE as u32
    } else {
        (DROPEFFECT_COPY | DROPEFFECT_LINK) as u32
    };
    set_dword(
        &data,
        "Preferred DropEffect",
        &PREFERRED_EFFECT_FORMAT,
        effect,
    )?;
    // SAFETY: plain OLE call.
    unsafe { OleSetClipboard(&data).ok() }
}

/// Writes one of the shell's DWORD formats (`Preferred DropEffect`, `Paste Succeeded`) onto
/// `data`.
fn set_dword(
    data: &IDataObject,
    name: &'static str,
    slot: &'static OnceLock<u16>,
    value: u32,
) -> Result<()> {
    let fmt = dragdrop::hglobal_format(dragdrop::registered_format(name, slot));
    // SAFETY: the global block is filled while locked; on success the data object owns it
    // (fRelease = TRUE), otherwise it is freed here.
    unsafe {
        let mem = GlobalAlloc(GMEM_MOVEABLE as u32, size_of::<u32>());
        let dst = GlobalLock(mem) as *mut u32;
        if dst.is_null() {
            let _ = GlobalFree(mem);
            return Err(E_FAIL.into());
        }
        dst.write_unaligned(value);
        let _ = GlobalUnlock(mem);
        let medium = STGMEDIUM {
            tymed: TYMED_HGLOBAL as u32,
            Anonymous: uSTGMEDIUM_0 { hGlobal: mem },
            pUnkForRelease: core::mem::ManuallyDrop::new(None),
        };
        let set = data.SetData(&fmt, &medium, true);
        if set.is_err() {
            let _ = GlobalFree(mem);
        }
        set.ok()
    }
}

/// Pastes the clipboard's shell items ([`shell_items`]) into `dest` through the folder's own
/// drop target, as Explorer's 粘贴 does: copied, or moved when they were cut. After a move the
/// source is told so (`Paste Succeeded`: a source that left the removal of the originals to the
/// paste does it then) and the clipboard is emptied, as Explorer does after pasting cut files,
/// unless something else was copied meanwhile. Returns the effect the shell performed.
///
/// **Runs the shell's modal UI** (progress, name conflicts): the caller must hold no
/// `RefCell` borrow.
pub fn paste_shell_items(dest: &Path) -> Result<dragdrop::DropEffect> {
    let (data, cut) = shell_items().ok_or(windows_core::Error::from(E_FAIL))?;
    let before = sequence();
    let allowed = if cut {
        DROPEFFECT_MOVE
    } else {
        DROPEFFECT_COPY
    } as u32;
    let effect = dragdrop::shell_folder_drop(
        dest,
        &data,
        0,
        false,
        dragdrop::DragPoint { x: 0, y: 0 },
        allowed,
    )?;
    if effect == dragdrop::DropEffect::Move {
        let unchanged = sequence() == before;
        if let Err(error) = set_dword(
            &data,
            "Paste Succeeded",
            &PASTE_SUCCEEDED_FORMAT,
            DROPEFFECT_MOVE as u32,
        ) {
            tracing::debug!(%error, "Paste Succeeded not taken by the source");
        }
        if unchanged {
            clear();
        }
        tracing::debug!(cleared = unchanged, "cut items pasted through the shell");
    }
    Ok(effect)
}

/// Cheap check for menu enabling: is there a file list on the clipboard, or items only the
/// shell writes out ([`shell_items`])?
pub fn has_file_list() -> bool {
    // SAFETY: plain OLE call.
    unsafe { OleGetClipboard() }
        .map(|d| dragdrop::has_hdrop(&d) || dragdrop::has_shell_items(&d))
        .unwrap_or(false)
}

/// Empties the clipboard (Explorer does this after pasting cut files).
pub fn clear() {
    // SAFETY: plain OLE call; a null object empties the clipboard.
    unsafe {
        let _ = OleSetClipboard(None::<&IDataObject>);
    }
}

/// `GetClipboardSequenceNumber`: changes whenever any program changes the clipboard.
pub fn sequence() -> u32 {
    // SAFETY: plain FFI call.
    unsafe { GetClipboardSequenceNumber() }
}

/// Puts `text` on the clipboard as Unicode text (the search's 复制路径).
pub fn set_text(owner: HWND, text: &str) -> bool {
    let wide = crate::wide::to_wide(text);
    // SAFETY: plain reinterpretation of the UTF-16 units as their bytes.
    let bytes = unsafe { std::slice::from_raw_parts(wide.as_ptr().cast::<u8>(), wide.len() * 2) };
    put(owner, CF_UNICODETEXT as u32, bytes)
}

/// Empties the clipboard and puts `bytes` on it as `format`.
fn put(owner: HWND, format: u32, bytes: &[u8]) -> bool {
    // SAFETY: the global block is filled while locked; on success the clipboard owns it,
    // otherwise it is freed here.
    unsafe {
        if !OpenClipboard(Some(owner)).as_bool() {
            return false;
        }
        let _ = EmptyClipboard();
        let mem = GlobalAlloc(GMEM_MOVEABLE as u32, bytes.len());
        let mut ok = false;
        if !mem.0.is_null() {
            let dst = GlobalLock(mem) as *mut u8;
            if !dst.is_null() {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
                let _ = GlobalUnlock(mem);
                ok = !SetClipboardData(format, Some(HANDLE(mem.0))).0.is_null();
            }
            if !ok {
                let _ = GlobalFree(mem);
            }
        }
        let _ = CloseClipboard();
        ok
    }
}

static EXCLUDE_FORMAT: OnceLock<u16> = OnceLock::new();
static HISTORY_FORMAT: OnceLock<u16> = OnceLock::new();
static VIEWER_IGNORE_FORMAT: OnceLock<u16> = OnceLock::new();

/// The clipboard's content is marked private (a password manager's copy): it says
/// `ExcludeClipboardContentFromMonitorProcessing`, `Clipboard Viewer Ignore`, or
/// `CanIncludeInClipboardHistory` = 0. Plugins never see such content. With the clipboard open.
fn marked_private() -> bool {
    let format = |name, slot| u32::from(dragdrop::registered_format(name, slot));
    // SAFETY: plain FFI calls with the clipboard open.
    let has = |f: u32| unsafe { IsClipboardFormatAvailable(f) }.as_bool();
    let exclude = format(
        "ExcludeClipboardContentFromMonitorProcessing",
        &EXCLUDE_FORMAT,
    );
    let ignore = format("Clipboard Viewer Ignore", &VIEWER_IGNORE_FORMAT);
    let history = format("CanIncludeInClipboardHistory", &HISTORY_FORMAT);
    has(exclude)
        || has(ignore)
        || has(history) && data(history).is_none_or(|b| b.get(..4) == Some(&[0; 4]))
}

/// The bytes of `format` on the open clipboard.
fn data(format: u32) -> Option<Vec<u8>> {
    with_data(format, <[u8]>::to_vec)
}

/// `f` of the bytes of `format` on the open clipboard, read in place (not copied).
fn with_data<T>(format: u32, f: impl FnOnce(&[u8]) -> T) -> Option<T> {
    // SAFETY: GetClipboardData's block belongs to the clipboard; it is only read while locked.
    unsafe {
        let mem: HGLOBAL = GetClipboardData(format);
        if mem.0.is_null() {
            return None;
        }
        let ptr = GlobalLock(mem) as *const u8;
        if ptr.is_null() {
            return None;
        }
        let out = f(std::slice::from_raw_parts(ptr, GlobalSize(mem)));
        let _ = GlobalUnlock(mem);
        Some(out)
    }
}

/// Runs `read` with the clipboard open (tried for 200 ms while another program holds it). None
/// when it could not be opened or its content is marked private ([`marked_private`]).
fn read<T>(read: impl FnOnce() -> Option<T>) -> Option<T> {
    for _ in 0..10 {
        // SAFETY: plain FFI calls; the clipboard is closed again on this thread.
        if unsafe { OpenClipboard(None) }.as_bool() {
            let private = marked_private();
            let out = if private { None } else { read() };
            // SAFETY: closes what this thread opened.
            unsafe {
                let _ = CloseClipboard();
            }
            if private {
                tracing::debug!("clipboard content marked private: not read");
            }
            return out;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    tracing::debug!("clipboard not read: another program holds it open");
    None
}

/// The clipboard's content is marked private (its change is not told to plugins).
pub fn is_private() -> bool {
    read(|| Some(())).is_none()
}

/// The clipboard's text, at most `max` bytes of UTF-8 (cut at a character), and whether it
/// was cut. Any thread.
pub fn read_text(max: usize) -> Option<(String, bool)> {
    let bytes = read(|| data(CF_UNICODETEXT as u32))?;
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect();
    let end = units.iter().position(|&u| u == 0).unwrap_or(units.len());
    let text = String::from_utf16_lossy(&units[..end]);
    if text.len() <= max {
        return Some((text, false));
    }
    let cut = (0..=max)
        .rev()
        .find(|&i| text.is_char_boundary(i))
        .unwrap_or(0);
    Some((text[..cut].to_string(), true))
}

/// The files on the clipboard (`CF_HDROP`), without OLE: any thread.
pub fn read_files() -> Option<Vec<PathBuf>> {
    read(|| {
        // SAFETY: the drop handle belongs to the open clipboard; each name fits its buffer.
        unsafe {
            let drop = HDROP(GetClipboardData(CF_HDROP as u32).0);
            if drop.0.is_null() {
                return None;
            }
            let count = DragQueryFileW(drop, u32::MAX, None, 0);
            let paths = (0..count).map(|i| {
                let mut buf = vec![0u16; DragQueryFileW(drop, i, None, 0) as usize + 1];
                let len = buf.len() as u32;
                let n = DragQueryFileW(drop, i, Some(windows_core::PWSTR(buf.as_mut_ptr())), len);
                PathBuf::from(String::from_utf16_lossy(&buf[..n as usize]))
            });
            Some(paths.collect())
        }
    })
}

/// A picture: width, height and top-down BGRA pixels; or the width and height of one larger
/// than asked for, read from its header alone.
pub type Picture = std::result::Result<(u32, u32, Vec<u8>), (u32, u32)>;

/// The picture on the clipboard (`CF_DIB`, 24 or 32 bits a pixel), at most `max_side` pixels
/// on a side: a larger one is told by its header before anything is copied. Any thread.
pub fn read_image(max_side: u32) -> Option<Picture> {
    read(|| with_data(CF_DIB as u32, |dib| dib_pixels(dib, max_side)).flatten())
}

/// The pixels of the `CF_DIB` block `dib`, when its header is of a kind read and the block
/// holds the pixels it says; every size is checked before it is used.
fn dib_pixels(dib: &[u8], max_side: u32) -> Option<Picture> {
    let field = |at: usize| Some(u32::from_le_bytes(dib.get(at..at + 4)?.try_into().ok()?));
    let (header, width, height) = (field(0)? as usize, field(4)? as i32, field(8)? as i32);
    let (bits, compression) = (*dib.get(14)? as usize, field(16)?);
    if header < 40 || width <= 0 || height == 0 || !(bits == 24 || bits == 32) || compression > 3 {
        tracing::debug!(
            header,
            bits,
            compression,
            "clipboard picture of a kind not read"
        );
        return None;
    }
    let (w, h) = (width as u32, height.unsigned_abs());
    if w > max_side || h > max_side {
        tracing::debug!(w, h, "clipboard picture larger than asked for: not read");
        return Some(Err((w, h)));
    }
    // BI_BITFIELDS after a plain header: three colour masks come before the pixels.
    let start = header.checked_add(if compression == 3 && header == 40 {
        12
    } else {
        0
    })?;
    let (w, h, bpp) = (w as usize, h as usize, bits / 8);
    let stride = w.checked_mul(bpp)?.div_ceil(4).checked_mul(4)?;
    let pixels = dib.get(start..start.checked_add(stride.checked_mul(h)?)?)?;
    let mut out = Vec::with_capacity(w.checked_mul(h)?.checked_mul(4)?);
    for y in 0..h {
        let row = if height < 0 { y } else { h - 1 - y };
        for px in pixels[row * stride..][..w * bpp].chunks_exact(bpp) {
            out.extend_from_slice(&[px[0], px[1], px[2], if bpp == 4 { px[3] } else { 255 }]);
        }
    }
    // Most programs leave the alpha of a 32-bit picture at 0: it is opaque.
    if out.as_chunks::<4>().0.iter().all(|p| p[3] == 0) {
        out.as_chunks_mut::<4>()
            .0
            .iter_mut()
            .for_each(|p| p[3] = 255);
    }
    Some(Ok((w as u32, h as u32, out)))
}

/// Puts a picture (top-down BGRA) on the clipboard as a 32-bit `CF_DIB`.
pub fn set_image(owner: HWND, width: u32, height: u32, bgra: &[u8]) -> bool {
    let mut dib = Vec::with_capacity(40 + bgra.len());
    for v in [
        40,
        width,
        height,
        1 | (32 << 16),
        0,
        bgra.len() as u32,
        0,
        0,
        0,
        0,
    ] {
        dib.extend_from_slice(&v.to_le_bytes());
    }
    for row in bgra.chunks_exact(width as usize * 4).rev() {
        dib.extend_from_slice(row);
    }
    put(owner, CF_DIB as u32, &dib)
}

/// Saves a picture (top-down BGRA) as a PNG file with WIC. On a thread with COM.
pub fn save_png(path: &Path, width: u32, height: u32, bgra: &[u8]) -> Result<()> {
    let wide = crate::wide::to_wide(&path.to_string_lossy());
    // SAFETY: standard WIC encoding; every interface lives until the end of the block.
    unsafe {
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
        let bitmap = factory.CreateBitmapFromMemory(
            width,
            height,
            &GUID_WICPixelFormat32bppBGRA,
            width * 4,
            bgra.len() as u32,
            bgra.as_ptr(),
        )?;
        let stream = factory.CreateStream()?;
        (stream.InitializeFromFilename(PCWSTR(wide.as_ptr()), GENERIC_WRITE as u32)).ok()?;
        let encoder = factory.CreateEncoder(&GUID_ContainerFormatPng, core::ptr::null())?;
        encoder.Initialize(&stream, WICBitmapEncoderNoCache).ok()?;
        let (mut frame, mut options) = (None, None);
        encoder.CreateNewFrame(&mut frame, &mut options).ok()?;
        let frame = frame.ok_or_else(windows_core::Error::empty)?;
        frame.Initialize(options.as_ref()).ok()?;
        frame.SetSize(width, height).ok()?;
        let mut format = GUID_WICPixelFormat32bppBGRA;
        frame.SetPixelFormat(&mut format).ok()?;
        frame.WriteSource(&bitmap, core::ptr::null()).ok()?;
        frame.Commit().ok()?;
        encoder.Commit().ok()
    }
}

/// `hwnd` gets `WM_CLIPBOARDUPDATE` whenever the clipboard changes, until it is destroyed.
pub fn listen(hwnd: HWND) -> bool {
    // SAFETY: plain FFI call; Windows removes the listener with the window.
    unsafe { AddClipboardFormatListener(hwnd) }.as_bool()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `CF_DIB` header (40 bytes) and `pixels`.
    fn dib(width: i32, height: i32, bits: u16, pixels: &[u8]) -> Vec<u8> {
        let mut d = Vec::new();
        d.extend_from_slice(&40u32.to_le_bytes());
        d.extend_from_slice(&width.to_le_bytes());
        d.extend_from_slice(&height.to_le_bytes());
        d.extend_from_slice(&1u16.to_le_bytes());
        d.extend_from_slice(&bits.to_le_bytes());
        d.extend_from_slice(&[0; 24]);
        d.extend_from_slice(pixels);
        d
    }

    #[test]
    fn a_picture_is_read_bottom_up_into_top_down_bgra() {
        // 2 x 2, 24 bits: rows of 6 bytes padded to 8, the bottom row first.
        let pixels = [1, 2, 3, 4, 5, 6, 0, 0, 7, 8, 9, 10, 11, 12, 0, 0];
        let (w, h, bgra) = dib_pixels(&dib(2, 2, 24, &pixels), 4096).unwrap().unwrap();
        assert_eq!((w, h), (2, 2));
        assert_eq!(
            bgra,
            [7, 8, 9, 255, 10, 11, 12, 255, 1, 2, 3, 255, 4, 5, 6, 255]
        );
    }

    #[test]
    fn a_huge_or_short_picture_is_refused_by_its_header() {
        // Larger than asked for: told without reading (or allocating for) its pixels.
        let huge = dib(100_000, -100_000, 32, &[]);
        assert_eq!(dib_pixels(&huge, 4096), Some(Err((100_000, 100_000))));
        assert_eq!(
            dib_pixels(&dib(1, i32::MIN, 32, &[]), 4096),
            Some(Err((1, 1 << 31)))
        );
        // Sizes that would overflow, or more pixels than the block holds: not read.
        assert_eq!(
            dib_pixels(&dib(i32::MAX, i32::MAX, 32, &[]), u32::MAX),
            None
        );
        assert_eq!(dib_pixels(&dib(4096, 4096, 32, &[0; 64]), 4096), None);
        assert_eq!(dib_pixels(&dib(2, 2, 8, &[0; 16]), 4096), None);
        assert_eq!(dib_pixels(&[40, 0, 0], 4096), None);
    }
}
