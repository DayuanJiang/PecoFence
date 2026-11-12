//! OLE drop target registration with a hand-written COM object.
//!
//! `windows-bindgen` 0.100 does not emit `_Impl` traits for Win32 COM interfaces, so the
//! `IDropTarget` vtable is laid out by hand. The object is single-threaded (STA) and is kept
//! alive by COM reference counting; `DropTargetRegistration` holds one reference.

use crate::bindings::*;
use crate::wide::to_wide;
use std::cell::RefCell;
use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};
use windows_core::{GUID, HRESULT, Interface, PCWSTR, Result};

pub use crate::bindings::{IDataObject, POINTL};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DropEffect {
    #[default]
    None,
    Copy,
    Move,
    Link,
}

impl DropEffect {
    pub fn to_raw(self) -> u32 {
        match self {
            DropEffect::None => DROPEFFECT_NONE as u32,
            DropEffect::Copy => DROPEFFECT_COPY as u32,
            DropEffect::Move => DROPEFFECT_MOVE as u32,
            DropEffect::Link => DROPEFFECT_LINK as u32,
        }
    }

    /// The single effect a drop target reported (Move wins over Copy over Link when several
    /// bits are set, matching how Explorer resolves its own feedback).
    pub fn from_raw(raw: u32) -> Self {
        if raw & DROPEFFECT_MOVE as u32 != 0 {
            DropEffect::Move
        } else if raw & DROPEFFECT_COPY as u32 != 0 {
            DropEffect::Copy
        } else if raw & DROPEFFECT_LINK as u32 != 0 {
            DropEffect::Link
        } else {
            DropEffect::None
        }
    }
}

/// `DROPEFFECT_MOVE | DROPEFFECT_COPY | DROPEFFECT_LINK`: everything a real desktop icon allows.
pub const ALL_EFFECTS: u32 =
    DROPEFFECT_MOVE as u32 | DROPEFFECT_COPY as u32 | DROPEFFECT_LINK as u32;

/// `grfKeyState` bits of OLE drag/drop (same values as the `MK_*` mouse-message flags).
pub const MK_LBUTTON: u32 = 0x0001;
pub const MK_RBUTTON: u32 = 0x0002;
pub const MK_SHIFT: u32 = 0x0004;
pub const MK_CONTROL: u32 = 0x0008;
pub const MK_ALT: u32 = 0x0020;

/// Explorer's modifier table for a file drag: Ctrl = Copy, Alt or Ctrl+Shift = Link (create
/// shortcut), Shift alone forces Move. No modifier = Move within one volume, Copy from another
/// drive or a network share (`same_volume`, see [`same_volume`]).
pub fn modifier_effect(key_state: u32, same_volume: bool) -> DropEffect {
    let ctrl = key_state & MK_CONTROL != 0;
    let shift = key_state & MK_SHIFT != 0;
    let alt = key_state & MK_ALT != 0;
    if (ctrl && shift) || alt {
        DropEffect::Link
    } else if ctrl {
        DropEffect::Copy
    } else if shift || same_volume {
        DropEffect::Move
    } else {
        DropEffect::Copy
    }
}

/// The effect of a file drag where Windows' `DefaultDropEffect` may be set (`user_default`, see
/// [`default_drop_effect`]): without Ctrl, Shift or Alt the drag has that effect on one volume
/// and across volumes alike, as in File Explorer; a held key decides as in [`modifier_effect`].
pub fn file_drag_effect(
    key_state: u32,
    same_volume: bool,
    user_default: Option<DropEffect>,
) -> DropEffect {
    match user_default {
        Some(effect) if key_state & (MK_CONTROL | MK_SHIFT | MK_ALT) == 0 => effect,
        _ => modifier_effect(key_state, same_volume),
    }
}

/// Windows' `DefaultDropEffect` for file-system items, when the user set one: the value under
/// `HKCR\*` or `HKCR\AllFilesystemObjects` (1 copy, 2 move, 4 create shortcut), which File
/// Explorer gives a drag without a modifier key instead of "move within a volume, copy across"
/// (feedback #153). Read from the user's classes, then the machine's, as `HKCR` merges them.
pub fn default_drop_effect() -> Option<DropEffect> {
    for class in ["*", "AllFilesystemObjects"] {
        let key = format!(r"Software\Classes\{class}");
        for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
            match crate::shell::reg_dword(root, &key, "DefaultDropEffect") {
                Some(1) => return Some(DropEffect::Copy),
                Some(2) => return Some(DropEffect::Move),
                Some(4) => return Some(DropEffect::Link),
                _ => {}
            }
        }
    }
    None
}

/// The volume part of `path` for Explorer's same-volume rule: the drive letter, or the
/// `\\server\share` of a UNC path, lower-cased. None for a path without either.
pub fn volume_of(path: &std::path::Path) -> Option<String> {
    use std::path::{Component, Prefix};
    let Some(Component::Prefix(prefix)) = path.components().next() else {
        return None;
    };
    match prefix.kind() {
        Prefix::Disk(d) | Prefix::VerbatimDisk(d) => {
            Some(format!("{}:", (d as char).to_ascii_lowercase()))
        }
        Prefix::UNC(server, share) | Prefix::VerbatimUNC(server, share) => Some(format!(
            r"\\{}\{}",
            server.to_string_lossy().to_lowercase(),
            share.to_string_lossy().to_lowercase()
        )),
        _ => None,
    }
}

/// Whether a drop from `source` (a [`volume_of`] result) into `dest` stays on one volume.
/// Unknown on either side counts as the same volume, which keeps the plain-drag Move.
pub fn same_volume(source: Option<&str>, dest: Option<&std::path::Path>) -> bool {
    match (source, dest.and_then(volume_of)) {
        (Some(s), Some(d)) => s == d,
        _ => true,
    }
}

/// The effect a target may report given the source's `allowed` mask (the incoming
/// `*pdwEffect`): `desired` when the source offers it, otherwise the best the source does offer
/// in Explorer's order Move > Copy > Link (a zip folder or a browser download shelf offers
/// COPY|LINK only, so a plain drag onto us copies). `None` stays `None` (not allowed).
pub fn resolve(desired: DropEffect, allowed: u32) -> DropEffect {
    if desired == DropEffect::None {
        return DropEffect::None;
    }
    if desired.to_raw() & allowed != 0 {
        desired
    } else {
        DropEffect::from_raw(allowed)
    }
}

/// Screen-space drag position in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DragPoint {
    pub x: i32,
    pub y: i32,
}

/// Callbacks for a window's drop target. All calls arrive on the UI thread. `allowed` is the
/// source's effect mask (the incoming `*pdwEffect`); the returned effect must be one of its
/// bits (see [`resolve`]) or `None`.
pub trait DropHandler {
    fn drag_enter(
        &mut self,
        data: &IDataObject,
        key_state: u32,
        pt: DragPoint,
        allowed: u32,
    ) -> DropEffect;
    fn drag_over(&mut self, key_state: u32, pt: DragPoint, allowed: u32) -> DropEffect;
    fn drag_leave(&mut self);
    fn drop(
        &mut self,
        data: &IDataObject,
        key_state: u32,
        pt: DragPoint,
        allowed: u32,
    ) -> DropEffect;
}

#[repr(C)]
struct DropTargetObject {
    vtbl: *const IDropTarget_Vtbl,
    refs: AtomicU32,
    handler: RefCell<Box<dyn DropHandler>>,
}

static VTBL: IDropTarget_Vtbl = IDropTarget_Vtbl {
    base__: windows_core::IUnknown_Vtbl {
        QueryInterface: query_interface,
        AddRef: add_ref,
        Release: release,
    },
    DragEnter: drag_enter,
    DragOver: drag_over,
    DragLeave: drag_leave,
    Drop: drop_,
};

const E_NOINTERFACE: HRESULT = HRESULT(0x8000_4002_u32 as i32);
const E_POINTER: HRESULT = HRESULT(0x8000_4003_u32 as i32);
const S_OK: HRESULT = HRESULT(0);

unsafe extern "system" fn query_interface(
    this: *mut core::ffi::c_void,
    iid: *const GUID,
    out: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // SAFETY: COM contract — `this` is a live DropTargetObject, `iid`/`out` are valid.
    unsafe {
        if out.is_null() {
            return E_POINTER;
        }
        let iid = &*iid;
        if *iid == IDropTarget::IID || *iid == windows_core::IUnknown::IID {
            add_ref(this);
            *out = this;
            S_OK
        } else {
            *out = core::ptr::null_mut();
            E_NOINTERFACE
        }
    }
}

unsafe extern "system" fn add_ref(this: *mut core::ffi::c_void) -> u32 {
    // SAFETY: COM contract.
    unsafe {
        (*(this as *mut DropTargetObject))
            .refs
            .fetch_add(1, Ordering::AcqRel)
            + 1
    }
}

unsafe extern "system" fn release(this: *mut core::ffi::c_void) -> u32 {
    // SAFETY: COM contract; the box is freed exactly when the count reaches zero.
    unsafe {
        let obj = this as *mut DropTargetObject;
        let remaining = (*obj).refs.fetch_sub(1, Ordering::AcqRel) - 1;
        if remaining == 0 {
            drop(Box::from_raw(obj));
        }
        remaining
    }
}

unsafe fn with_handler<R>(
    this: *mut core::ffi::c_void,
    f: impl FnOnce(&mut dyn DropHandler) -> R,
) -> Option<R> {
    // SAFETY: `this` is a live DropTargetObject (COM contract).
    let obj = unsafe { &*(this as *const DropTargetObject) };
    let mut guard = obj.handler.try_borrow_mut().ok()?;
    Some(f(guard.as_mut()))
}

/// The source's permitted effects, as OLE passes them in `*pdwEffect` on entry (everything
/// when the pointer is missing, which no real source does).
unsafe fn allowed_mask(effect: *mut u32) -> u32 {
    if effect.is_null() {
        ALL_EFFECTS
    } else {
        // SAFETY: COM contract — a non-null `pdwEffect` points at the caller's DWORD.
        unsafe { *effect }
    }
}

/// Views the caller's `IDataObject*` for the duration of the callback. `IDataObject` is a
/// transparent wrapper around the pointer, so the returned reference borrows the *parameter
/// slot* itself — it must be taken in the thunk's own frame (a helper returning it would hand
/// back a dangling reference into its dead frame, which is exactly what crashed drag-out: the
/// slot got reused and `Interface::vtable` read a garbage pointer).
/// Expands inside the thunk's `unsafe` block. SAFETY there: `$ptr` is the callback's own
/// parameter, alive for the whole call, and OLE keeps the object referenced meanwhile.
macro_rules! borrow_data {
    ($ptr:ident) => {
        IDataObject::from_raw_borrowed(&$ptr)
    };
}

unsafe extern "system" fn drag_enter(
    this: *mut core::ffi::c_void,
    data: *mut core::ffi::c_void,
    key_state: u32,
    pt: POINTL,
    effect: *mut u32,
) -> HRESULT {
    // SAFETY: COM contract.
    unsafe {
        let Some(data) = borrow_data!(data) else {
            return E_POINTER;
        };
        let allowed = allowed_mask(effect);
        let result = with_handler(this, |h| {
            h.drag_enter(data, key_state, DragPoint { x: pt.x, y: pt.y }, allowed)
        })
        .unwrap_or_default();
        if !effect.is_null() {
            *effect = result.to_raw();
        }
        S_OK
    }
}

unsafe extern "system" fn drag_over(
    this: *mut core::ffi::c_void,
    key_state: u32,
    pt: POINTL,
    effect: *mut u32,
) -> HRESULT {
    // SAFETY: COM contract.
    unsafe {
        let allowed = allowed_mask(effect);
        let result = with_handler(this, |h| {
            h.drag_over(key_state, DragPoint { x: pt.x, y: pt.y }, allowed)
        })
        .unwrap_or_default();
        if !effect.is_null() {
            *effect = result.to_raw();
        }
        S_OK
    }
}

unsafe extern "system" fn drag_leave(this: *mut core::ffi::c_void) -> HRESULT {
    // SAFETY: COM contract.
    unsafe {
        let _ = with_handler(this, |h| h.drag_leave());
    }
    S_OK
}

unsafe extern "system" fn drop_(
    this: *mut core::ffi::c_void,
    data: *mut core::ffi::c_void,
    key_state: u32,
    pt: POINTL,
    effect: *mut u32,
) -> HRESULT {
    // SAFETY: COM contract.
    unsafe {
        let Some(data) = borrow_data!(data) else {
            return E_POINTER;
        };
        let allowed = allowed_mask(effect);
        let result = with_handler(this, |h| {
            h.drop(data, key_state, DragPoint { x: pt.x, y: pt.y }, allowed)
        })
        .unwrap_or_default();
        if !effect.is_null() {
            *effect = result.to_raw();
        }
        S_OK
    }
}

/// Keeps a window registered as an OLE drop target. Revokes on drop.
pub struct DropTargetRegistration {
    hwnd: HWND,
    _target: IDropTarget,
}

impl DropTargetRegistration {
    /// Registers `handler` as the drop target of `hwnd`. `OleInitialize` must have run on
    /// this thread.
    pub fn register(hwnd: HWND, handler: Box<dyn DropHandler>) -> Result<Self> {
        let object = Box::new(DropTargetObject {
            vtbl: &VTBL,
            refs: AtomicU32::new(1),
            handler: RefCell::new(handler),
        });
        // SAFETY: the raw pointer is a valid COM object with one owned reference, which is
        // transferred to the `IDropTarget` wrapper.
        let target: IDropTarget = unsafe { IDropTarget::from_raw(Box::into_raw(object).cast()) };
        // SAFETY: valid HWND and interface pointer.
        unsafe { RegisterDragDrop(hwnd, &target).ok()? };
        Ok(Self {
            hwnd,
            _target: target,
        })
    }
}

impl Drop for DropTargetRegistration {
    fn drop(&mut self) {
        // SAFETY: balances RegisterDragDrop; failure (window gone) is ignored.
        unsafe {
            let _ = RevokeDragDrop(self.hwnd);
        }
    }
}

const CF_HDROP_FORMAT: u16 = CF_HDROP as u16;
const DVASPECT_CONTENT_VALUE: u32 = 1;

pub(crate) fn hglobal_format(cf: u16) -> FORMATETC {
    FORMATETC {
        cfFormat: CLIPFORMAT(cf),
        ptd: core::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT_VALUE,
        lindex: -1,
        tymed: TYMED_HGLOBAL as u32,
    }
}

pub(crate) fn hdrop_format() -> FORMATETC {
    hglobal_format(CF_HDROP_FORMAT)
}

/// Registers (once) a named clipboard format and returns its id.
pub(crate) fn registered_format(name: &'static str, slot: &'static OnceLock<u16>) -> u16 {
    *slot.get_or_init(|| {
        let w = to_wide(name);
        // SAFETY: string outlives the call.
        unsafe { RegisterClipboardFormatW(PCWSTR(w.as_ptr())) as u16 }
    })
}

static INET_URL_FORMAT: OnceLock<u16> = OnceLock::new();
static FILE_GROUP_FORMAT: OnceLock<u16> = OnceLock::new();
static FILE_GROUP_FORMAT_A: OnceLock<u16> = OnceLock::new();
static SHELL_IDLIST_FORMAT: OnceLock<u16> = OnceLock::new();

/// `CFSTR_INETURLW`: the link a browser is dragging.
fn inet_url_format() -> FORMATETC {
    hglobal_format(registered_format(
        "UniformResourceLocatorW",
        &INET_URL_FORMAT,
    ))
}

/// Reads a `TYMED_HGLOBAL` medium of `fmt` as raw bytes (copied out before release).
pub(crate) fn hglobal_bytes(data: &IDataObject, fmt: &FORMATETC) -> Option<Vec<u8>> {
    // SAFETY: standard GetData / GlobalLock protocol; the medium is released afterwards.
    unsafe {
        let medium = data.GetData(fmt).ok()?;
        let hglobal = medium.Anonymous.hGlobal;
        let mut out = None;
        if !hglobal.0.is_null() {
            let ptr = GlobalLock(hglobal) as *const u8;
            if !ptr.is_null() {
                let len = GlobalSize(hglobal);
                out = Some(std::slice::from_raw_parts(ptr, len).to_vec());
                let _ = GlobalUnlock(hglobal);
            }
        }
        ReleaseStgMedium(&medium);
        out
    }
}

/// Does the data object carry a browser link (`UniformResourceLocatorW`)? Read, not just
/// queried: a zip folder's items answer `QueryGetData` for it and then fail `GetData`.
pub fn has_inet_url(data: &IDataObject) -> bool {
    inet_url(data).is_some()
}

/// The dragged link, if it is an http(s) / ftp / file URL.
pub fn inet_url(data: &IDataObject) -> Option<String> {
    let bytes = hglobal_bytes(data, &inet_url_format())?;
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .take_while(|&u| u != 0)
        .collect();
    let s = String::from_utf16_lossy(&units).trim().to_string();
    let lower = s.to_lowercase();
    ["http://", "https://", "ftp://", "file://"]
        .iter()
        .any(|p| lower.starts_with(p))
        .then_some(s)
}

fn file_group_format() -> FORMATETC {
    hglobal_format(registered_format(
        "FileGroupDescriptorW",
        &FILE_GROUP_FORMAT,
    ))
}

/// The file name a browser suggests for a dragged link (`FileGroupDescriptorW`, first entry;
/// usually `<page title>.url`).
pub fn file_group_descriptor_name(data: &IDataObject) -> Option<String> {
    file_group_descriptor_names(data).into_iter().next()
}

/// The names of the virtual files a source offers (`FileGroupDescriptorW`: an Outlook mail's
/// `<subject>.msg`, an attachment, a browser link's `.url`). Entries may be relative paths
/// (`folder\file.txt`) when a whole tree is dragged.
pub fn file_group_descriptor_names(data: &IDataObject) -> Vec<String> {
    let Some(bytes) = hglobal_bytes(data, &file_group_format()) else {
        return Vec::new();
    };
    let header = std::mem::offset_of!(FILEGROUPDESCRIPTORW, fgd);
    let size = std::mem::size_of::<FILEDESCRIPTORW>();
    let Some(count) = bytes
        .get(..header)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)
    else {
        return Vec::new();
    };
    (0..count)
        .map_while(|i| {
            let start = header + i * size;
            let entry = bytes.get(start..start + size)?;
            // SAFETY: `entry` is exactly one FILEDESCRIPTORW long; the struct is plain packed
            // data, so an unaligned read of a copy is fine.
            let desc: FILEDESCRIPTORW =
                unsafe { std::ptr::read_unaligned(entry.as_ptr() as *const FILEDESCRIPTORW) };
            let name: [u16; 260] = desc.cFileName;
            let n = name.iter().position(|&c| c == 0).unwrap_or(name.len());
            Some(String::from_utf16_lossy(&name[..n]))
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Does the data object carry items only the shell's own folder drop target takes apart:
/// virtual files (`FileGroupDescriptorW` / `FileGroupDescriptor` + `FileContents`, as Outlook
/// drags mail and attachments) or a shell id list (`Shell IDList Array`: Start-menu apps,
/// Control Panel items, a phone's files)?
pub fn has_shell_items(data: &IDataObject) -> bool {
    [
        file_group_format(),
        hglobal_format(registered_format(
            "FileGroupDescriptor",
            &FILE_GROUP_FORMAT_A,
        )),
        hglobal_format(registered_format(
            "Shell IDList Array",
            &SHELL_IDLIST_FORMAT,
        )),
    ]
    .iter()
    // SAFETY: COM call with a fully initialized FORMATETC.
    .any(|fmt| unsafe { data.QueryGetData(fmt).is_ok() })
}

/// Does a drag off the desktop hold one of the desktop's own namespace items (This PC, the
/// Recycle Bin, OneDrive, the user's folder)? The icons of OneDrive and of the user's folder
/// also name that folder in `CF_HDROP`, but the folder is not theirs to move (moving OneDrive's
/// fails with "0x80004001 Not implemented").
pub fn has_desktop_namespace_item(data: &IDataObject) -> bool {
    let fmt = hglobal_format(registered_format(
        "Shell IDList Array",
        &SHELL_IDLIST_FORMAT,
    ));
    hglobal_bytes(data, &fmt).is_some_and(|b| ida_has_root_item(&b))
}

/// Reads a `CIDA` (item count, then the offsets of the parent's ID list and of each item's):
/// the parent is the desktop (an empty list) and an item is a root namespace item (its first
/// ID has type 0x1F).
fn ida_has_root_item(b: &[u8]) -> bool {
    let u32_at = |i: usize| {
        b.get(i..i + 4)
            .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]) as usize)
    };
    let Some(count) = u32_at(0) else {
        return false;
    };
    let on_desktop = u32_at(4).and_then(|o| b.get(o..o + 2)) == Some(&[0, 0][..]);
    on_desktop
        && (1..=count.min(1 << 16))
            .any(|k| u32_at(4 + 4 * k).and_then(|o| b.get(o + 2)) == Some(&0x1F))
}

/// Hands a drop to the shell's own drop target for `folder` — what dropping onto that folder's
/// icon in Explorer runs — for data we do not take apart ourselves (see [`has_shell_items`]):
/// the shell writes the mail as `.msg`, creates the Start-menu app's shortcut, asks about name
/// collisions and, after a right-button drag (`right_button`), shows its own Move / Copy /
/// Shortcut menu. `key_state` holds the modifiers at the drop, `allowed` the source's mask.
/// Returns the effect the shell performed (what the source must be told).
///
/// **Runs the shell's modal UI** (progress, conflict dialog, drop menu): the caller must hold
/// no `RefCell` borrow.
pub fn shell_folder_drop(
    folder: &Path,
    data: &IDataObject,
    key_state: u32,
    right_button: bool,
    pt: DragPoint,
    allowed: u32,
) -> Result<DropEffect> {
    let w = to_wide(&folder.to_string_lossy());
    let pt = POINTL { x: pt.x, y: pt.y };
    let modifiers = key_state & !(MK_LBUTTON | MK_RBUTTON);
    // The button that carried the drag: the target remembers it from DragEnter / DragOver and
    // decides on its menu in Drop, when no button is down any more.
    let held = modifiers | if right_button { MK_RBUTTON } else { MK_LBUTTON };
    // SAFETY: standard IDropTarget protocol on the UI (STA) thread; every DragEnter is
    // balanced by a Drop or a DragLeave; the strings outlive the calls.
    unsafe {
        let item: IShellItem = SHCreateItemFromParsingName(PCWSTR(w.as_ptr()), None)?;
        let target: IDropTarget = item.BindToHandler(None, &BHID_SFUIObject)?;
        let mut effect = allowed;
        target.DragEnter(data, held, pt, &mut effect).ok()?;
        effect = allowed;
        let over = target.DragOver(held, pt, &mut effect);
        if over.is_err() || effect == 0 {
            let _ = target.DragLeave();
            over.ok()?;
            return Ok(DropEffect::None);
        }
        effect = allowed;
        target.Drop(data, modifiers, pt, &mut effect).ok()?;
        Ok(DropEffect::from_raw(effect))
    }
}

/// A program file the Shell opens dropped folders with (`.exe`, `.com`, `.bat`, `.cmd`), by
/// its extension.
pub fn is_program_path(path: &Path) -> bool {
    path.extension().is_some_and(|ext| {
        ["exe", "com", "bat", "cmd"]
            .iter()
            .any(|suffix| ext.eq_ignore_ascii_case(suffix))
    })
}

/// A drag delegated to a program file or an application's `.lnk` (preserving its arguments and
/// working directory). Keep it for one hover, and call without holding a view's RefCell borrow:
/// Shell extensions may pump messages. Dropping opens the dragged files and folders, never
/// transfers them.
pub struct ShortcutDropTarget {
    target: IDropTarget,
    entered: bool,
    button: u32,
}

impl ShortcutDropTarget {
    /// The Shell's drop target of a program file or of a shortcut to one. Anything else is no
    /// target, and the error says why (a fixed text for the caller's log).
    pub fn bind(shortcut: &Path) -> std::result::Result<Self, &'static str> {
        let link = shortcut
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"));
        let resolved = if link {
            crate::shell::shortcut_target_expanded(shortcut)
        } else {
            Some(shortcut.to_path_buf())
        };
        let refused = match &resolved {
            // A Store app's link, or a shortcut that cannot be read.
            None => Some("shortcut without a file target"),
            // A document or a folder.
            Some(p) if !is_program_path(p) => Some("target is no program"),
            // A broken shortcut, or a program file that is gone.
            Some(p) if !p.is_file() => Some("program is missing"),
            Some(_) => None,
        };
        tracing::debug!(
            shortcut = %shortcut.display(),
            target = ?resolved,
            refused,
            "program or shortcut under a drag of files or folders"
        );
        if let Some(reason) = refused {
            return Err(reason);
        }
        // Resolve only to check the target type. Bind the link (or the program) itself, so
        // the Shell keeps its saved arguments, working directory and launch behaviour.
        let path = to_wide(&shortcut.to_string_lossy());
        // SAFETY: COM is initialized on the UI STA and the string outlives the call.
        let target: Result<IDropTarget> = unsafe {
            match SHCreateItemFromParsingName::<_, _, IShellItem>(PCWSTR(path.as_ptr()), None) {
                Ok(item) => item.BindToHandler(None, &BHID_SFUIObject),
                Err(e) => Err(e),
            }
        };
        let target = target.map_err(|_| "the Shell gave no drop target")?;
        Ok(Self {
            target,
            entered: false,
            button: MK_LBUTTON,
        })
    }

    pub fn drag_over(
        &mut self,
        data: &IDataObject,
        keys: u32,
        pt: DragPoint,
        allowed: u32,
    ) -> DropEffect {
        // An application consumes paths; MOVE would let a source remove its originals.
        let mut effect = allowed & (DROPEFFECT_COPY | DROPEFFECT_LINK) as u32;
        let point = POINTL { x: pt.x, y: pt.y };
        if keys & (MK_LBUTTON | MK_RBUTTON) != 0 {
            self.button = keys & (MK_LBUTTON | MK_RBUTTON);
        }
        // The final hover check happens after button-up. The Shell remembers which
        // button carried the drag from DragEnter/DragOver, not from Drop.
        let keys = keys | self.button;
        // SAFETY: one DragEnter per hover, balanced by Drop or DragLeave on this STA.
        let result = unsafe {
            if self.entered {
                self.target.DragOver(keys, point, &mut effect)
            } else {
                let result = self.target.DragEnter(data, keys, point, &mut effect);
                self.entered = result.is_ok();
                result
            }
        };
        if result.is_err() {
            return DropEffect::None;
        }
        DropEffect::from_raw(effect & allowed & (DROPEFFECT_COPY | DROPEFFECT_LINK) as u32)
    }

    pub fn drop_items(
        mut self,
        data: &IDataObject,
        keys: u32,
        pt: DragPoint,
        allowed: u32,
    ) -> Result<DropEffect> {
        if self.drag_over(data, keys, pt, allowed) == DropEffect::None {
            return Ok(DropEffect::None);
        }
        let mut effect = allowed & (DROPEFFECT_COPY | DROPEFFECT_LINK) as u32;
        self.entered = false;
        // SAFETY: a successful DragEnter preceded this Drop; data lives through the call.
        unsafe {
            self.target
                .Drop(
                    data,
                    keys & !(MK_LBUTTON | MK_RBUTTON),
                    POINTL { x: pt.x, y: pt.y },
                    &mut effect,
                )
                .ok()?;
        }
        Ok(DropEffect::from_raw(
            effect & allowed & (DROPEFFECT_COPY | DROPEFFECT_LINK) as u32,
        ))
    }
}

impl Drop for ShortcutDropTarget {
    fn drop(&mut self) {
        if self.entered {
            // SAFETY: balance an entered hover when it leaves, is rejected or is cancelled.
            unsafe {
                let _ = self.target.DragLeave();
            }
        }
    }
}

#[cfg(test)]
mod shortcut_tests {
    use super::*;
    use std::path::PathBuf;

    fn shortcut(path: &Path, target: &Path, args: &str, cwd: &Path) {
        let path = to_wide(&path.to_string_lossy());
        let target = to_wide(&target.to_string_lossy());
        let args = to_wide(args);
        let cwd = to_wide(&cwd.to_string_lossy());
        // SAFETY: COM is initialized by the test and every string outlives the call.
        unsafe {
            let link: IShellLinkW =
                CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).unwrap();
            link.SetPath(PCWSTR(target.as_ptr())).ok().unwrap();
            link.SetArguments(PCWSTR(args.as_ptr())).ok().unwrap();
            link.SetWorkingDirectory(PCWSTR(cwd.as_ptr())).ok().unwrap();
            link.SetShowCmd(0).ok().unwrap();
            let persist: IPersistFile = link.cast().unwrap();
            persist.Save(PCWSTR(path.as_ptr()), true).ok().unwrap();
        }
    }

    /// Real Shell drop: the shortcut keeps its arguments/cwd, folder and file paths containing
    /// spaces and Chinese arrive intact, and neither the source folders, their files nor the
    /// dropped file move (#198: files open with a program as folders do).
    #[test]
    fn shortcut_drop_opens_items_without_moving_them() {
        let _ole = crate::com::OleGuard::init().unwrap();
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!(
            "pecofence-shortcut-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&base).unwrap();
        let folders = [base.join("项目 one"), base.join("project two")];
        for folder in &folders {
            std::fs::create_dir(folder).unwrap();
            std::fs::write(folder.join("keep.txt"), "unchanged").unwrap();
        }
        let script = base.join("capture.ps1");
        std::fs::write(&script, "[IO.File]::WriteAllLines((Join-Path $PSScriptRoot 'received.txt'), (@((Get-Location).Path) + @($args)), [Text.UTF8Encoding]::new($false))").unwrap();
        let executable = PathBuf::from(std::env::var_os("SystemRoot").unwrap())
            .join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
        let link = base.join("test application.lnk");
        shortcut(
            &link,
            &executable,
            &format!(
                "-NoProfile -NonInteractive -ExecutionPolicy Bypass -File \"{}\" \"saved argument\"",
                script.display()
            ),
            &base,
        );
        let file = base.join("说明 note.txt");
        std::fs::write(&file, "unchanged").unwrap();
        let refs: Vec<&Path> = folders
            .iter()
            .map(|p| p.as_path())
            .chain([file.as_path()])
            .collect();
        let data = data_object_for_paths(&refs).unwrap();
        let pt = DragPoint { x: 0, y: 0 };
        let received = base.join("received.txt");
        {
            let mut cancelled = ShortcutDropTarget::bind(&link).unwrap();
            assert_eq!(
                cancelled.drag_over(&data, MK_LBUTTON, pt, DropEffect::Move.to_raw()),
                DropEffect::None
            );
        }
        {
            let mut cancelled = ShortcutDropTarget::bind(&link).unwrap();
            assert_ne!(
                cancelled.drag_over(&data, MK_LBUTTON, pt, ALL_EFFECTS),
                DropEffect::None
            );
        }
        assert!(
            !received.exists(),
            "hovering/cancelling must not launch the application"
        );
        let mut target = ShortcutDropTarget::bind(&link).unwrap();
        let entered = target.drag_over(&data, MK_LBUTTON, pt, ALL_EFFECTS);
        assert!(
            matches!(entered, DropEffect::Copy | DropEffect::Link),
            "{entered:?}"
        );
        let dropped = target.drop_items(&data, 0, pt, ALL_EFFECTS).unwrap();
        assert!(
            matches!(dropped, DropEffect::Copy | DropEffect::Link),
            "{dropped:?}"
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        let lines = loop {
            let text = std::fs::read_to_string(&received).unwrap_or_default();
            let lines: Vec<String> = text.lines().map(str::to_string).collect();
            if lines.len() == 5 {
                break lines;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "application did not receive the items: {text:?}"
            );
            std::thread::sleep(std::time::Duration::from_millis(25));
        };
        // Compare canonical paths: %TEMP% can be an 8.3 short path (C:\Users\RUNNER~1 on CI)
        // while the application reports the long form.
        let same = |a: &Path, b: &Path| {
            assert_eq!(
                std::fs::canonicalize(a).unwrap(),
                std::fs::canonicalize(b).unwrap()
            )
        };
        same(Path::new(&lines[0]), &base);
        assert_eq!(lines[1], "saved argument");
        for (folder, argument) in folders.iter().zip(&lines[2..4]) {
            same(Path::new(argument), folder);
            assert_eq!(
                std::fs::read_to_string(folder.join("keep.txt")).unwrap(),
                "unchanged"
            );
        }
        same(Path::new(&lines[4]), &file);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "unchanged");
        // Neither a folder shortcut nor a document shortcut is an application drop target.
        for (name, destination) in [
            ("folder", folders[0].as_path()),
            ("document", script.as_path()),
        ] {
            let path = base.join(format!("{name}.lnk"));
            shortcut(&path, destination, "", &base);
            assert_eq!(
                ShortcutDropTarget::bind(&path).err(),
                Some("target is no program")
            );
        }
        let missing = base.join("missing.lnk");
        shortcut(&missing, &base.join("missing.exe"), "", &base);
        assert_eq!(
            ShortcutDropTarget::bind(&missing).err(),
            Some("program is missing")
        );
        // A program file itself is a target too; a document or a missing program is not.
        let bat = base.join("run.bat");
        std::fs::write(&bat, "@echo off").unwrap();
        assert!(ShortcutDropTarget::bind(&executable).is_ok());
        assert!(ShortcutDropTarget::bind(&bat).is_ok());
        assert!(ShortcutDropTarget::bind(&script).is_err());
        assert!(ShortcutDropTarget::bind(&base.join("missing.exe")).is_err());
        // The launched PowerShell can keep the folder as its working directory for a moment
        // after writing its file (os error 32 on CI), so cleanup retries and never fails the test.
        for _ in 0..50 {
            if std::fs::remove_dir_all(&base).is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

/// Test hook: a data object another process marshaled into `bytes` (`CoMarshalInterface`,
/// table-strong) — a cross-process source like Outlook's drag, without moving the mouse.
#[cfg(debug_assertions)]
pub fn unmarshal_data_object(bytes: &[u8]) -> Result<IDataObject> {
    windows_core::link!("shlwapi.dll" "system" fn SHCreateMemStream(pinit: *const u8, cbinit: u32) -> *mut core::ffi::c_void);
    windows_core::link!("ole32.dll" "system" fn CoUnmarshalInterface(pstm: *mut core::ffi::c_void, riid: *const GUID, ppv: *mut *mut core::ffi::c_void) -> HRESULT);
    // SAFETY: the memory stream copies `bytes` and is owned by `stream`; the unmarshaled
    // pointer carries one reference, handed to the wrapper.
    unsafe {
        let raw = SHCreateMemStream(bytes.as_ptr(), bytes.len() as u32);
        if raw.is_null() {
            return Err(windows_core::Error::from_hresult(E_FAIL));
        }
        let stream = IStream::from_raw(raw);
        let mut out = core::ptr::null_mut();
        CoUnmarshalInterface(stream.as_raw(), &IDataObject::IID, &mut out).ok()?;
        Ok(IDataObject::from_raw(out))
    }
}

/// The shell's own data object for a set of file-system paths (`CF_HDROP`, shell id lists,
/// `IShellItemArray` ...) - the same object Explorer hands out when its icons are dragged, so
/// every drop target (Explorer, Recycle Bin, browsers, mail clients) treats the items as real
/// files. Unknown paths are skipped.
pub fn data_object_for_paths(paths: &[&Path]) -> Result<IDataObject> {
    if paths.is_empty() {
        return Err(windows_core::Error::from_hresult(E_INVALIDARG));
    }
    let mut pidls: Vec<LPCITEMIDLIST> = Vec::with_capacity(paths.len());
    for p in paths {
        let w = to_wide(&p.to_string_lossy());
        let mut pidl: LPITEMIDLIST = std::ptr::null_mut();
        // SAFETY: string outlives the call; pidl is freed below.
        let hr =
            unsafe { SHParseDisplayName(PCWSTR(w.as_ptr()), None, &mut pidl, SFGAOF(0), None) };
        if hr.is_ok() && !pidl.is_null() {
            pidls.push(pidl as LPCITEMIDLIST);
        }
    }
    let bound = (|| -> Result<IDataObject> {
        if pidls.is_empty() {
            return Err(windows_core::Error::from_hresult(E_FAIL));
        }
        // SAFETY: the array copies the id lists; BHID_DataObject yields the folder's data object
        // for exactly these items.
        unsafe {
            let array: IShellItemArray = SHCreateShellItemArrayFromIDLists(&pidls)?;
            array.BindToHandler(None, &BHID_DataObject)
        }
    })();
    for p in pidls {
        // SAFETY: each pidl came from SHParseDisplayName.
        unsafe { ILFree(Some(p)) };
    }
    bound
}

/// Starts an OLE drag of `data` from `hwnd` with the shell's default drop source or ours (Esc /
/// right button cancel, standard Move / Copy / Link cursors) and the shell drag image.
///
/// How an outgoing drag ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DragOutcome {
    /// The user released over a target (`DRAGDROP_S_DROP`); false = Esc / right button /
    /// failure (`DRAGDROP_S_CANCEL`), in which case nothing must change.
    pub dropped: bool,
    /// Effect the target performed; `None` also when nothing accepted the drop.
    pub effect: DropEffect,
}

const DRAGDROP_S_DROP: i32 = 0x0004_0100;

/// Asked when the left button comes up, with the key state at that moment (`MK_*`): true ends
/// the drag there without a drop, so the target under the pointer gets `DragLeave` and writes
/// nothing.
pub type SkipDrop = Box<dyn Fn(u32) -> bool>;

/// **Runs a nested message loop** until the button is released: the caller must hold no
/// `RefCell` borrow and must have released mouse capture. Distinguishes a cancelled drag from
/// a drop nobody accepted (the generated binding folds both into `Ok(0)`). With `skip_drop`
/// our own drop source replaces the shell's default one, and a skipped drop is reported as a
/// drop nobody accepted.
pub fn do_drag_drop(
    hwnd: HWND,
    data: &IDataObject,
    allowed: u32,
    skip_drop: Option<SkipDrop>,
) -> DragOutcome {
    windows_core::link!("shell32.dll" "system" fn SHDoDragDrop(hwnd: HWND, pdata: *mut core::ffi::c_void, pdsrc: *mut core::ffi::c_void, dweffect: u32, pdweffect: *mut u32) -> windows_core::HRESULT);
    allow_drop_description_text(data);
    let source = skip_drop.map(|skip| new_drop_source(data, skip));
    let mut effect: u32 = 0;
    // SAFETY: valid HWND and interfaces; a null IDropSource asks the shell for its default one.
    let hr = unsafe {
        SHDoDragDrop(
            hwnd,
            data.as_raw(),
            source
                .as_ref()
                .map_or(core::ptr::null_mut(), |s| s.as_raw()),
            allowed,
            &mut effect,
        )
    };
    if hr.is_err() {
        tracing::warn!(error = %windows_core::Error::from_hresult(hr), "SHDoDragDrop failed");
    }
    // SAFETY: `source` still holds its reference, so the object is alive.
    let skipped = source
        .as_ref()
        .is_some_and(|s| unsafe { (*(s.as_raw() as *const DropSourceObject)).skipped.get() });
    DragOutcome {
        dropped: hr.0 == DRAGDROP_S_DROP || skipped,
        effect: DropEffect::from_raw(effect),
    }
}

static DRAG_SOURCE_HELPER_FLAGS_FORMAT: OnceLock<u16> = OnceLock::new();

/// Lets the drag image carry the target's text beside its badge ("移动到 <栅栏>"), as a drag out
/// of an Explorer window does: the `DragSourceHelperFlags` format with
/// `DSH_ALLOWDROPDESCRIPTIONTEXT`, which is what `IDragSourceHelper2::SetFlags` leaves on the
/// data object. Without it the image shows the badge alone, whatever description a target sets.
/// A data object that refuses `SetData` just keeps the badge.
fn allow_drop_description_text(data: &IDataObject) {
    windows_core::link!("kernel32.dll" "system" fn GlobalAlloc(uflags: u32, dwbytes: usize) -> HGLOBAL);
    windows_core::link!("kernel32.dll" "system" fn GlobalFree(hmem: HGLOBAL) -> HGLOBAL);
    const GMEM_MOVEABLE_ZEROINIT: u32 = 0x0042;
    const DSH_ALLOWDROPDESCRIPTIONTEXT: u32 = 1;
    let fmt = hglobal_format(registered_format(
        "DragSourceHelperFlags",
        &DRAG_SOURCE_HELPER_FLAGS_FORMAT,
    ));
    // SAFETY: standard GlobalAlloc / GlobalLock / SetData protocol; on success the data object
    // owns the memory (fRelease = TRUE), on failure it is freed here.
    let set = unsafe {
        let hmem = GlobalAlloc(GMEM_MOVEABLE_ZEROINIT, size_of::<u32>());
        if hmem.0.is_null() {
            return;
        }
        let ptr = GlobalLock(hmem) as *mut u32;
        if ptr.is_null() {
            let _ = GlobalFree(hmem);
            return;
        }
        *ptr = DSH_ALLOWDROPDESCRIPTIONTEXT;
        let _ = GlobalUnlock(hmem);
        let medium = STGMEDIUM {
            tymed: TYMED_HGLOBAL as u32,
            Anonymous: uSTGMEDIUM_0 { hGlobal: hmem },
            pUnkForRelease: core::mem::ManuallyDrop::new(None),
        };
        let set = data.SetData(&fmt, &medium, true);
        if set.is_err() {
            let _ = GlobalFree(hmem);
        }
        set
    };
    tracing::debug!(
        allowed = set.is_ok(),
        hresult = set.0,
        "drag image text beside the badge (DragSourceHelperFlags on the data object)"
    );
}

// ---- Drop source (the shell's default one plus `SkipDrop`) -----------------------------------

#[repr(C)]
struct DropSourceVtbl {
    base: windows_core::IUnknown_Vtbl,
    query_continue_drag:
        unsafe extern "system" fn(*mut core::ffi::c_void, windows_core::BOOL, u32) -> HRESULT,
    give_feedback: unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> HRESULT,
}

#[repr(C)]
struct DropSourceObject {
    vtbl: *const DropSourceVtbl,
    refs: AtomicU32,
    data: IDataObject,
    skip_drop: SkipDrop,
    skipped: std::cell::Cell<bool>,
    /// The button that carries the drag (`MK_LBUTTON` or `MK_RBUTTON`), known from the first
    /// `QueryContinueDrag`; 0 before it.
    button: std::cell::Cell<u32>,
}

static SOURCE_VTBL: DropSourceVtbl = DropSourceVtbl {
    base: windows_core::IUnknown_Vtbl {
        QueryInterface: source_query_interface,
        AddRef: source_add_ref,
        Release: source_release,
    },
    query_continue_drag: source_query_continue_drag,
    give_feedback: source_give_feedback,
};

fn new_drop_source(data: &IDataObject, skip_drop: SkipDrop) -> IDropSource {
    let object = Box::new(DropSourceObject {
        vtbl: &SOURCE_VTBL,
        refs: AtomicU32::new(1),
        data: data.clone(),
        skip_drop,
        skipped: std::cell::Cell::new(false),
        button: std::cell::Cell::new(0),
    });
    // SAFETY: a valid COM object with one owned reference, transferred to the wrapper.
    unsafe { IDropSource::from_raw(Box::into_raw(object).cast()) }
}

unsafe extern "system" fn source_query_interface(
    this: *mut core::ffi::c_void,
    iid: *const GUID,
    out: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // SAFETY: COM contract — `this` is a live DropSourceObject, `iid`/`out` are valid.
    unsafe {
        if out.is_null() {
            return E_POINTER;
        }
        let iid = &*iid;
        if *iid == IDropSource::IID || *iid == windows_core::IUnknown::IID {
            source_add_ref(this);
            *out = this;
            S_OK
        } else {
            *out = core::ptr::null_mut();
            E_NOINTERFACE
        }
    }
}

unsafe extern "system" fn source_add_ref(this: *mut core::ffi::c_void) -> u32 {
    // SAFETY: COM contract.
    unsafe {
        (*(this as *mut DropSourceObject))
            .refs
            .fetch_add(1, Ordering::AcqRel)
            + 1
    }
}

unsafe extern "system" fn source_release(this: *mut core::ffi::c_void) -> u32 {
    // SAFETY: COM contract; the box is freed exactly when the count reaches zero.
    unsafe {
        let obj = this as *mut DropSourceObject;
        let remaining = (*obj).refs.fetch_sub(1, Ordering::AcqRel) - 1;
        if remaining == 0 {
            drop(Box::from_raw(obj));
        }
        remaining
    }
}

/// The button that carries a drag, from the key state of the first `QueryContinueDrag`: the
/// right one when it alone is held, else the left one (also when it is already up, which
/// drops at once).
fn carrying_button(key_state: u32) -> u32 {
    if key_state & (MK_LBUTTON | MK_RBUTTON) == MK_RBUTTON {
        MK_RBUTTON
    } else {
        MK_LBUTTON
    }
}

/// How a drag carried by `button` goes on: `Some(true)` it continues, `Some(false)` it is
/// cancelled (Esc or the other button), `None` the button came up, which is the drop.
fn drag_goes_on(button: u32, escape: bool, key_state: u32) -> Option<bool> {
    let buttons = key_state & (MK_LBUTTON | MK_RBUTTON);
    if escape || buttons & !button != 0 {
        Some(false)
    } else if buttons & button != 0 {
        Some(true)
    } else {
        None
    }
}

/// The shell's rules for a drag with the left or the right button (Esc or the other button
/// cancels, releasing the one that carries the drag drops), with `skip_drop` asked at the
/// release.
unsafe extern "system" fn source_query_continue_drag(
    this: *mut core::ffi::c_void,
    escape: windows_core::BOOL,
    key_state: u32,
) -> HRESULT {
    // SAFETY: COM contract — `this` is a live DropSourceObject.
    let obj = unsafe { &*(this as *const DropSourceObject) };
    if obj.button.get() == 0 {
        let button = carrying_button(key_state);
        tracing::debug!(
            right = button == MK_RBUTTON,
            "drag out: which button carries it"
        );
        obj.button.set(button);
    }
    match drag_goes_on(obj.button.get(), escape.as_bool(), key_state) {
        Some(true) => S_OK,
        Some(false) => DRAGDROP_S_CANCEL,
        None if (obj.skip_drop)(key_state) => {
            obj.skipped.set(true);
            DRAGDROP_S_CANCEL
        }
        None => HRESULT(DRAGDROP_S_DROP),
    }
}

static IS_SHOWING_LAYERED_FORMAT: OnceLock<u16> = OnceLock::new();
static DRAG_WINDOW_FORMAT: OnceLock<u16> = OnceLock::new();

/// A DWORD the shell's drag image keeps in the data object under the format `name`.
fn data_dword(data: &IDataObject, name: &'static str, slot: &'static OnceLock<u16>) -> Option<u32> {
    let bytes = hglobal_bytes(data, &hglobal_format(registered_format(name, slot)))?;
    Some(u32::from_le_bytes(bytes.get(..4)?.try_into().ok()?))
}

/// What the shell's default drop source does: while the drag image is a layered window the
/// pointer stays an arrow and the image carries the effect's badge (`DDWM_SETCURSOR`,
/// WM_USER + 2: 1 none, 2 move, 3 copy, 4 link), otherwise OLE's drag cursors.
unsafe extern "system" fn source_give_feedback(
    this: *mut core::ffi::c_void,
    effect: u32,
) -> HRESULT {
    const DRAGDROP_S_USEDEFAULTCURSORS: HRESULT = HRESULT(0x0004_0102);
    // SAFETY: COM contract — `this` is a live DropSourceObject.
    let obj = unsafe { &*(this as *const DropSourceObject) };
    if data_dword(&obj.data, "IsShowingLayered", &IS_SHOWING_LAYERED_FORMAT).unwrap_or(0) == 0 {
        return DRAGDROP_S_USEDEFAULTCURSORS;
    }
    if let Some(window) = data_dword(&obj.data, "DragWindow", &DRAG_WINDOW_FORMAT) {
        let badge = match DropEffect::from_raw(effect) {
            DropEffect::None => 1,
            DropEffect::Move => 2,
            DropEffect::Copy => 3,
            DropEffect::Link => 4,
        };
        // SAFETY: posting to a window handle; a stale one just fails.
        unsafe {
            let _ = PostMessageW(
                Some(HWND(window as usize as *mut core::ffi::c_void)),
                WM_USER as u32 + 2,
                WPARAM(badge),
                LPARAM(0),
            );
        }
    }
    crate::window::set_standard_cursor(crate::window::StandardCursor::Arrow);
    S_OK
}

/// Does the data object carry a file list (`CF_HDROP`)?
pub fn has_hdrop(data: &IDataObject) -> bool {
    let fmt = hdrop_format();
    // SAFETY: COM call with a fully initialized FORMATETC.
    unsafe { data.QueryGetData(&fmt).is_ok() }
}

/// Extracts the dropped file paths from a `CF_HDROP` data object.
pub fn hdrop_paths(data: &IDataObject) -> Vec<std::path::PathBuf> {
    let fmt = hdrop_format();
    let mut out = Vec::new();
    // SAFETY: standard HDROP extraction; the medium is released afterwards.
    unsafe {
        let Ok(medium) = data.GetData(&fmt) else {
            return out;
        };
        let hglobal = medium.Anonymous.hGlobal;
        if !hglobal.0.is_null() {
            let hdrop = HDROP(hglobal.0);
            let count = DragQueryFileW(hdrop, u32::MAX, None, 0);
            for i in 0..count {
                let len = DragQueryFileW(hdrop, i, None, 0);
                let mut buf = vec![0u16; len as usize + 1];
                let n = DragQueryFileW(
                    hdrop,
                    i,
                    Some(windows_core::PWSTR(buf.as_mut_ptr())),
                    buf.len() as u32,
                );
                if n > 0 {
                    out.push(std::path::PathBuf::from(crate::wide::from_wide(
                        &buf[..n as usize],
                    )));
                }
            }
        }
        ReleaseStgMedium(&medium);
    }
    out
}

// ---- Shell drag image (DI_GETDRAGIMAGE) -------------------------------------------------

/// `SHDRAGIMAGE` (shobjidl_core.h): what a drag source answers to `DI_GETDRAGIMAGE` with. The
/// shell's drag-source helper takes ownership of the bitmap.
#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
struct SHDRAGIMAGE {
    size_drag_image: SIZE,
    pt_offset: POINT,
    hbmp_drag_image: HBITMAP,
    cr_color_key: COLORREF,
}

/// `CLR_NONE`: the bitmap's own alpha channel is used, no colour key.
const CLR_NONE: u32 = 0xFFFF_FFFF;

static DI_GETDRAGIMAGE: OnceLock<u32> = OnceLock::new();

/// The registered `DI_GETDRAGIMAGE` window message (`"ShellGetDragImage"`): `SHDoDragDrop`
/// sends it to the source window before the drag starts; answering it with a bitmap replaces
/// the shell's generic drag image. 0 if registration failed (never matches a real message).
pub fn di_getdragimage_msg() -> u32 {
    *DI_GETDRAGIMAGE.get_or_init(|| {
        let name = to_wide("ShellGetDragImage");
        // SAFETY: NUL-terminated string outlives the call.
        unsafe { RegisterWindowMessageW(PCWSTR(name.as_ptr())) }
    })
}

/// A rendered drag image: tightly packed, top-down, premultiplied BGRA of `width` x `height`
/// device pixels, plus the point inside it that sits under the cursor.
pub struct DragImage {
    pub width: u32,
    pub height: u32,
    pub offset: (i32, i32),
    pub bgra: Vec<u8>,
}

/// Answers `DI_GETDRAGIMAGE`: uploads `image` into a 32-bit DIB and fills the `SHDRAGIMAGE`
/// the message points at. Returns false (nothing written) when the DIB could not be created
/// or the buffer size does not match, so the caller returns 0 and the shell falls back to its
/// generic image.
///
/// # Safety
/// `lparam` must be the `SHDRAGIMAGE*` of a `DI_GETDRAGIMAGE` message being processed.
pub unsafe fn answer_drag_image(lparam: isize, image: &DragImage) -> bool {
    let (w, h) = (image.width, image.height);
    if lparam == 0
        || w == 0
        || h == 0
        || w > 8192
        || h > 8192
        || image.bgra.len() != (w as usize) * (h as usize) * 4
    {
        return false;
    }
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w as i32,
            biHeight: -(h as i32), // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB as u32,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits: *mut core::ffi::c_void = core::ptr::null_mut();
    // SAFETY: the header describes a w*h*4-byte top-down DIB, the caller's buffer has exactly
    // that many bytes, and the shell owns the section once the message returns TRUE.
    unsafe {
        let dib = CreateDIBSection(None, &info, DIB_RGB_COLORS as u32, &mut bits, None, 0);
        if dib.0.is_null() || bits.is_null() {
            return false;
        }
        core::ptr::copy_nonoverlapping(image.bgra.as_ptr(), bits.cast::<u8>(), image.bgra.len());
        let out = lparam as *mut SHDRAGIMAGE;
        (*out).size_drag_image = SIZE {
            cx: w as i32,
            cy: h as i32,
        };
        (*out).pt_offset = POINT {
            x: image.offset.0.clamp(0, w as i32 - 1),
            y: image.offset.1.clamp(0, h as i32 - 1),
        };
        (*out).hbmp_drag_image = dib;
        (*out).cr_color_key = COLORREF(CLR_NONE);
    }
    true
}

// ---- Drop description + IDropTargetHelper (the modern drag-image badge) ----------------------

/// `DROPIMAGETYPE`: which badge the shell draws on the drag image next to the description.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropImage {
    /// No description set: the shell derives the badge from the reported effect.
    Invalid,
    /// The red "not allowed" badge.
    None,
    Copy,
    Move,
    Link,
    Label,
    Warning,
    NoImage,
}

impl DropImage {
    fn to_raw(self) -> i32 {
        match self {
            DropImage::Invalid => -1,
            DropImage::None => 0,
            DropImage::Copy => DROPEFFECT_COPY,
            DropImage::Move => DROPEFFECT_MOVE,
            DropImage::Link => DROPEFFECT_LINK,
            DropImage::Label => 6,
            DropImage::Warning => 7,
            DropImage::NoImage => 8,
        }
    }

    /// The badge matching an effect a target reports.
    pub fn for_effect(effect: DropEffect) -> Self {
        match effect {
            DropEffect::None => DropImage::None,
            DropEffect::Copy => DropImage::Copy,
            DropEffect::Move => DropImage::Move,
            DropEffect::Link => DropImage::Link,
        }
    }
}

/// `DROPDESCRIPTION` (shobjidl_core.h): the text the drag image carries while over a target.
/// `%1` in `sz_message` is replaced by `sz_insert`, which the shell paints in the accent colour.
#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
struct DROPDESCRIPTION {
    r#type: i32,
    sz_message: [u16; 260],
    sz_insert: [u16; 260],
}

static DROP_DESCRIPTION_FORMAT: OnceLock<u16> = OnceLock::new();

fn drop_description_format() -> FORMATETC {
    hglobal_format(registered_format(
        "DropDescription",
        &DROP_DESCRIPTION_FORMAT,
    ))
}

fn copy_wide_fixed<const N: usize>(dst: &mut [u16; N], s: &str) {
    let w = to_wide(s);
    let n = w.len().min(N - 1);
    dst[..n].copy_from_slice(&w[..n]);
    dst[n] = 0;
}

/// Sets (or, with `DropImage::Invalid`, clears) the drop description on the data object being
/// dragged over us, as Explorer does for its own folders: the shell's drag image then reads
/// e.g. "移动到 <insert>" with the matching badge instead of the legacy cursor glyph. Sources
/// that refuse `SetData` are ignored (their image just keeps the default look).
pub fn set_drop_description(data: &IDataObject, image: DropImage, message: &str, insert: &str) {
    windows_core::link!("kernel32.dll" "system" fn GlobalAlloc(uflags: u32, dwbytes: usize) -> HGLOBAL);
    windows_core::link!("kernel32.dll" "system" fn GlobalFree(hmem: HGLOBAL) -> HGLOBAL);
    const GMEM_MOVEABLE_ZEROINIT: u32 = 0x0042;
    let fmt = drop_description_format();
    // SAFETY: standard GlobalAlloc / GlobalLock / SetData protocol; on success the data object
    // owns the memory (fRelease = TRUE), on failure it is freed here.
    unsafe {
        let hmem = GlobalAlloc(GMEM_MOVEABLE_ZEROINIT, size_of::<DROPDESCRIPTION>());
        if hmem.0.is_null() {
            return;
        }
        let ptr = GlobalLock(hmem) as *mut DROPDESCRIPTION;
        if ptr.is_null() {
            let _ = GlobalFree(hmem);
            return;
        }
        (*ptr).r#type = image.to_raw();
        copy_wide_fixed(&mut (*ptr).sz_message, message);
        copy_wide_fixed(&mut (*ptr).sz_insert, insert);
        let _ = GlobalUnlock(hmem);
        let medium = STGMEDIUM {
            tymed: TYMED_HGLOBAL as u32,
            Anonymous: uSTGMEDIUM_0 { hGlobal: hmem },
            pUnkForRelease: core::mem::ManuallyDrop::new(None),
        };
        if data.SetData(&fmt, &medium, true).is_err() {
            let _ = GlobalFree(hmem);
        }
    }
}

/// `CLSID_DragDropHelper`.
const CLSID_DRAG_DROP_HELPER: GUID = GUID::from_u128(0x4657278A_411B_11D2_839A_00C04FD918D0);

/// The shell's `IDropTargetHelper`: told about every DragEnter / DragOver / DragLeave / Drop it
/// keeps the source's drag image (and its description badge) painted and positioned over our
/// window, exactly as Explorer's folder views do. Absent (None) when the shell refuses.
pub struct DropTargetHelper(IDropTargetHelper);

impl DropTargetHelper {
    pub fn new() -> Option<Self> {
        // SAFETY: plain CoCreateInstance on a thread that initialised OLE.
        let helper: IDropTargetHelper =
            unsafe { CoCreateInstance(&CLSID_DRAG_DROP_HELPER, None, CLSCTX_INPROC_SERVER) }
                .map_err(|e| tracing::debug!(error = %e, "DragDropHelper unavailable"))
                .ok()?;
        Some(Self(helper))
    }

    /// **Sends messages to the drag image window** (the source's thread, which is this one for
    /// fence-to-fence drags): the caller must hold no `RefCell` borrow.
    pub fn drag_enter(&self, hwnd: HWND, data: &IDataObject, pt: DragPoint, effect: DropEffect) {
        let p = POINT { x: pt.x, y: pt.y };
        // SAFETY: valid interface, hwnd and point.
        unsafe {
            let _ = self.0.DragEnter(hwnd, data, &p, effect.to_raw());
        }
    }

    /// See [`Self::drag_enter`] for the re-entrancy rule.
    pub fn drag_over(&self, pt: DragPoint, effect: DropEffect) {
        let p = POINT { x: pt.x, y: pt.y };
        // SAFETY: valid interface and point.
        unsafe {
            let _ = self.0.DragOver(&p, effect.to_raw());
        }
    }

    /// See [`Self::drag_enter`] for the re-entrancy rule.
    pub fn drag_leave(&self) {
        // SAFETY: valid interface.
        unsafe {
            let _ = self.0.DragLeave();
        }
    }

    /// See [`Self::drag_enter`] for the re-entrancy rule.
    pub fn drop(&self, data: &IDataObject, pt: DragPoint, effect: DropEffect) {
        let p = POINT { x: pt.x, y: pt.y };
        // SAFETY: valid interface and point.
        unsafe {
            let _ = self.0.Drop(data, &p, effect.to_raw());
        }
    }
}

#[cfg(test)]
mod effect_tests {
    use super::*;

    const MOVE: u32 = DROPEFFECT_MOVE as u32;
    const COPY: u32 = DROPEFFECT_COPY as u32;
    const LINK: u32 = DROPEFFECT_LINK as u32;

    /// A target never reports an effect the source did not offer; the fallback follows
    /// Explorer's Move > Copy > Link order, an empty mask (or a `None` wish) stays `None`.
    #[test]
    fn resolve_honours_the_source_mask() {
        assert_eq!(resolve(DropEffect::Move, COPY | LINK), DropEffect::Copy);
        assert_eq!(resolve(DropEffect::Copy, MOVE), DropEffect::Move);
        assert_eq!(resolve(DropEffect::Link, COPY), DropEffect::Copy);
        assert_eq!(resolve(DropEffect::Link, LINK | COPY), DropEffect::Link);
        assert_eq!(resolve(DropEffect::Move, 0), DropEffect::None);
        assert_eq!(resolve(DropEffect::None, ALL_EFFECTS), DropEffect::None);
        assert_eq!(resolve(DropEffect::Move, ALL_EFFECTS), DropEffect::Move);
        assert_eq!(resolve(DropEffect::Copy, ALL_EFFECTS), DropEffect::Copy);
    }

    /// Explorer's modifier table: Ctrl copy, Shift move, Alt or Ctrl+Shift link; the mouse
    /// button bits do not take part. A plain drag moves within a volume and copies across.
    #[test]
    fn modifier_table_matches_explorer() {
        assert_eq!(modifier_effect(0, true), DropEffect::Move);
        assert_eq!(modifier_effect(0, false), DropEffect::Copy);
        assert_eq!(modifier_effect(MK_SHIFT, true), DropEffect::Move);
        assert_eq!(modifier_effect(MK_SHIFT, false), DropEffect::Move);
        assert_eq!(modifier_effect(MK_CONTROL, true), DropEffect::Copy);
        assert_eq!(
            modifier_effect(MK_CONTROL | MK_SHIFT, true),
            DropEffect::Link
        );
        assert_eq!(modifier_effect(MK_ALT, false), DropEffect::Link);
        assert_eq!(modifier_effect(MK_ALT | MK_CONTROL, true), DropEffect::Link);
        assert_eq!(modifier_effect(MK_RBUTTON | 0x0001, true), DropEffect::Move);
        assert_eq!(
            modifier_effect(MK_RBUTTON | MK_CONTROL, true),
            DropEffect::Copy
        );
    }

    /// A drag out of a fence is carried by the button that started it: releasing it drops, Esc
    /// or the other button cancels, modifier keys change nothing.
    #[test]
    fn a_drag_is_carried_by_the_left_or_the_right_button() {
        assert_eq!(carrying_button(MK_LBUTTON), MK_LBUTTON);
        assert_eq!(carrying_button(MK_RBUTTON | MK_CONTROL), MK_RBUTTON);
        assert_eq!(carrying_button(MK_LBUTTON | MK_RBUTTON), MK_LBUTTON);
        assert_eq!(carrying_button(0), MK_LBUTTON);
        for (button, other) in [(MK_LBUTTON, MK_RBUTTON), (MK_RBUTTON, MK_LBUTTON)] {
            assert_eq!(drag_goes_on(button, false, button), Some(true));
            assert_eq!(drag_goes_on(button, false, button | MK_SHIFT), Some(true));
            assert_eq!(drag_goes_on(button, true, button), Some(false));
            assert_eq!(drag_goes_on(button, false, button | other), Some(false));
            assert_eq!(drag_goes_on(button, false, other), Some(false));
            assert_eq!(drag_goes_on(button, false, 0), None);
            assert_eq!(drag_goes_on(button, false, MK_CONTROL), None);
        }
    }

    /// Windows' DefaultDropEffect replaces the plain drag's effect on one volume and across
    /// volumes; Ctrl, Shift and Alt decide as without it (measured in File Explorer).
    #[test]
    fn default_drop_effect_replaces_the_plain_drag_only() {
        let (mv, cp) = (Some(DropEffect::Move), Some(DropEffect::Copy));
        assert_eq!(file_drag_effect(0, false, mv), DropEffect::Move);
        assert_eq!(file_drag_effect(0, true, mv), DropEffect::Move);
        assert_eq!(file_drag_effect(0, true, cp), DropEffect::Copy);
        assert_eq!(file_drag_effect(0, false, cp), DropEffect::Copy);
        assert_eq!(file_drag_effect(MK_LBUTTON, false, mv), DropEffect::Move);
        assert_eq!(file_drag_effect(MK_CONTROL, false, mv), DropEffect::Copy);
        assert_eq!(file_drag_effect(MK_ALT, false, mv), DropEffect::Link);
        assert_eq!(file_drag_effect(MK_SHIFT, true, cp), DropEffect::Move);
        assert_eq!(file_drag_effect(MK_ALT, true, cp), DropEffect::Link);
        assert_eq!(file_drag_effect(0, true, None), DropEffect::Move);
        assert_eq!(file_drag_effect(0, false, None), DropEffect::Copy);
    }

    #[test]
    fn volumes_compare_by_drive_or_share() {
        use std::path::Path;
        assert_eq!(volume_of(Path::new(r"C:\Users\me")).as_deref(), Some("c:"));
        assert_eq!(volume_of(Path::new(r"\\?\D:\x")).as_deref(), Some("d:"));
        assert_eq!(
            volume_of(Path::new(r"\\NAS\Share\a.txt")).as_deref(),
            Some(r"\\nas\share")
        );
        assert_eq!(volume_of(Path::new("relative")), None);
        let desk = Path::new(r"C:\Users\me\Desktop");
        assert!(same_volume(Some("c:"), Some(desk)));
        assert!(!same_volume(Some("e:"), Some(desk)));
        assert!(!same_volume(Some(r"\\nas\share"), Some(desk)));
        assert!(same_volume(None, Some(desk)));
        assert!(same_volume(Some("e:"), None));
    }

    /// A `CIDA` from the desktop: root items (type 0x1F, OneDrive, This PC) and files (0x32).
    #[test]
    fn root_items_are_found_in_a_desktop_ida() {
        // count 2; offsets: parent 16, items 18 and 24; parent = empty list; each item one ID
        // (cb 4: cb, type, one byte) and its terminator.
        let ida = |first: u8, second: u8, parent_cb: u8| {
            let mut b = vec![2, 0, 0, 0, 16, 0, 0, 0, 18, 0, 0, 0, 24, 0, 0, 0];
            b.extend([parent_cb, 0]);
            b.extend([4, 0, first, 0, 0, 0]);
            b.extend([4, 0, second, 0, 0, 0]);
            b
        };
        assert!(ida_has_root_item(&ida(0x1F, 0x1F, 0)));
        // OneDrive among files.
        assert!(ida_has_root_item(&ida(0x32, 0x1F, 0)));
        assert!(!ida_has_root_item(&ida(0x32, 0x31, 0)));
        // Not on the desktop (a folder's items): no root item there.
        assert!(!ida_has_root_item(&ida(0x1F, 0x1F, 4)));
        assert!(!ida_has_root_item(&[2, 0, 0]));
        assert!(!ida_has_root_item(&[0, 0, 0, 0, 8, 0, 0, 0, 0, 0]));
    }

    #[test]
    fn drop_image_maps_to_dropimagetype() {
        assert_eq!(DropImage::Invalid.to_raw(), -1);
        assert_eq!(DropImage::None.to_raw(), 0);
        assert_eq!(DropImage::Copy.to_raw(), 1);
        assert_eq!(DropImage::Move.to_raw(), 2);
        assert_eq!(DropImage::Link.to_raw(), 4);
        assert_eq!(DropImage::NoImage.to_raw(), 8);
        assert_eq!(DropImage::for_effect(DropEffect::Move), DropImage::Move);
        assert_eq!(DropImage::for_effect(DropEffect::None), DropImage::None);
        assert_eq!(size_of::<DROPDESCRIPTION>(), 4 + 2 * 260 * 2);
    }
}
