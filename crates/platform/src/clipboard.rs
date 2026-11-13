//! The OLE clipboard as Explorer uses it: a `CF_HDROP` file list plus the shell's
//! `Preferred DropEffect` word that distinguishes 剪切 (move) from 复制 (copy).

use crate::bindings::*;
use crate::dragdrop;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use windows_core::Result;

static PREFERRED_EFFECT_FORMAT: OnceLock<u16> = OnceLock::new();

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
    let fmt = dragdrop::hglobal_format(dragdrop::registered_format(
        "Preferred DropEffect",
        &PREFERRED_EFFECT_FORMAT,
    ));
    let effect = dragdrop::hglobal_bytes(&data, &fmt)
        .filter(|b| b.len() >= 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .unwrap_or(DROPEFFECT_COPY as u32);
    let cut = effect & DROPEFFECT_MOVE as u32 != 0 && effect & DROPEFFECT_COPY as u32 == 0;
    Some((paths, cut))
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
    let fmt = dragdrop::hglobal_format(dragdrop::registered_format(
        "Preferred DropEffect",
        &PREFERRED_EFFECT_FORMAT,
    ));
    // SAFETY: the global block is filled while locked; on success the data object owns it
    // (fRelease = TRUE), otherwise it is freed here.
    unsafe {
        let mem = GlobalAlloc(GMEM_MOVEABLE as u32, size_of::<u32>());
        let dst = GlobalLock(mem) as *mut u32;
        if dst.is_null() {
            let _ = GlobalFree(mem);
            return Err(E_FAIL.into());
        }
        dst.write_unaligned(effect);
        let _ = GlobalUnlock(mem);
        let medium = STGMEDIUM {
            tymed: TYMED_HGLOBAL as u32,
            Anonymous: uSTGMEDIUM_0 { hGlobal: mem },
            pUnkForRelease: core::mem::ManuallyDrop::new(None),
        };
        let set = data.SetData(&fmt, &medium, true);
        if set.is_err() {
            let _ = GlobalFree(mem);
            return set.ok();
        }
        OleSetClipboard(&data).ok()
    }
}

/// Cheap check for menu enabling: is there a file list on the clipboard?
pub fn has_file_list() -> bool {
    // SAFETY: plain OLE call.
    unsafe { OleGetClipboard() }
        .map(|d| dragdrop::has_hdrop(&d))
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
    let bytes = wide.len() * 2;
    // SAFETY: the global block is filled while locked; on success the clipboard owns it,
    // otherwise it is freed here.
    unsafe {
        if !OpenClipboard(Some(owner)).as_bool() {
            return false;
        }
        let _ = EmptyClipboard();
        let mem = GlobalAlloc(GMEM_MOVEABLE as u32, bytes);
        let mut ok = false;
        if !mem.0.is_null() {
            let dst = GlobalLock(mem) as *mut u16;
            if !dst.is_null() {
                std::ptr::copy_nonoverlapping(wide.as_ptr(), dst, wide.len());
                let _ = GlobalUnlock(mem);
                ok = !SetClipboardData(CF_UNICODETEXT as u32, Some(HANDLE(mem.0)))
                    .0
                    .is_null();
            }
            if !ok {
                let _ = GlobalFree(mem);
            }
        }
        let _ = CloseClipboard();
        ok
    }
}
