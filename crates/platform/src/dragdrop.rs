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
pub const MK_RBUTTON: u32 = 0x0002;
pub const MK_SHIFT: u32 = 0x0004;
pub const MK_CONTROL: u32 = 0x0008;
pub const MK_ALT: u32 = 0x0020;

/// Explorer's modifier table for a file drag: Ctrl = Copy, Alt or Ctrl+Shift = Link (create
/// shortcut), Shift alone forces Move, no modifier = Move (the default the target prefers).
pub fn modifier_effect(key_state: u32) -> DropEffect {
    let ctrl = key_state & MK_CONTROL != 0;
    let shift = key_state & MK_SHIFT != 0;
    let alt = key_state & MK_ALT != 0;
    if (ctrl && shift) || alt {
        DropEffect::Link
    } else if ctrl {
        DropEffect::Copy
    } else {
        DropEffect::Move
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

/// A handler that only logs. Useful for spikes and for windows that accept nothing.
pub struct LoggingDropHandler;

impl DropHandler for LoggingDropHandler {
    fn drag_enter(
        &mut self,
        _data: &IDataObject,
        key_state: u32,
        pt: DragPoint,
        allowed: u32,
    ) -> DropEffect {
        tracing::info!(key_state, ?pt, allowed, "drag enter");
        resolve(DropEffect::Link, allowed)
    }

    fn drag_over(&mut self, _key_state: u32, _pt: DragPoint, allowed: u32) -> DropEffect {
        resolve(DropEffect::Link, allowed)
    }

    fn drag_leave(&mut self) {
        tracing::info!("drag leave");
    }

    fn drop(
        &mut self,
        _data: &IDataObject,
        key_state: u32,
        pt: DragPoint,
        allowed: u32,
    ) -> DropEffect {
        tracing::info!(key_state, ?pt, "drop");
        resolve(DropEffect::Link, allowed)
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

/// Does the data object carry a browser link (`UniformResourceLocatorW`)?
pub fn has_inet_url(data: &IDataObject) -> bool {
    let fmt = inet_url_format();
    // SAFETY: COM call with a fully initialized FORMATETC.
    unsafe { data.QueryGetData(&fmt).is_ok() }
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

/// The file name a browser suggests for a dragged link (`FileGroupDescriptorW`, first entry;
/// usually `<page title>.url`).
pub fn file_group_descriptor_name(data: &IDataObject) -> Option<String> {
    let fmt = hglobal_format(registered_format(
        "FileGroupDescriptorW",
        &FILE_GROUP_FORMAT,
    ));
    let bytes = hglobal_bytes(data, &fmt)?;
    if bytes.len() < std::mem::size_of::<FILEGROUPDESCRIPTORW>() {
        return None;
    }
    // SAFETY: the buffer is at least one FILEGROUPDESCRIPTORW long (checked above); the struct
    // is plain data, so an unaligned read of a copy is fine.
    let desc: FILEGROUPDESCRIPTORW =
        unsafe { std::ptr::read_unaligned(bytes.as_ptr() as *const FILEGROUPDESCRIPTORW) };
    if desc.cItems == 0 {
        return None;
    }
    let name: [u16; 260] = desc.fgd[0].cFileName;
    let n = name.iter().position(|&c| c == 0).unwrap_or(name.len());
    let s = String::from_utf16_lossy(&name[..n]);
    (!s.is_empty()).then_some(s)
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

/// Starts an OLE drag of `data` from `hwnd` with the shell's default drop source (Esc / right
/// button cancel, standard Move / Copy / Link cursors) and the shell drag image.
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

/// **Runs a nested message loop** until the button is released: the caller must hold no
/// `RefCell` borrow and must have released mouse capture. Distinguishes a cancelled drag from
/// a drop nobody accepted (the generated binding folds both into `Ok(0)`).
pub fn do_drag_drop(hwnd: HWND, data: &IDataObject, allowed: u32) -> DragOutcome {
    windows_core::link!("shell32.dll" "system" fn SHDoDragDrop(hwnd: HWND, pdata: *mut core::ffi::c_void, pdsrc: *mut core::ffi::c_void, dweffect: u32, pdweffect: *mut u32) -> windows_core::HRESULT);
    let mut effect: u32 = 0;
    // SAFETY: valid HWND and interface; a null IDropSource asks the shell for its default one.
    let hr = unsafe {
        SHDoDragDrop(
            hwnd,
            data.as_raw(),
            core::ptr::null_mut(),
            allowed,
            &mut effect,
        )
    };
    if hr.is_err() {
        tracing::warn!(error = %windows_core::Error::from_hresult(hr), "SHDoDragDrop failed");
    }
    DragOutcome {
        dropped: hr.0 == DRAGDROP_S_DROP,
        effect: DropEffect::from_raw(effect),
    }
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
    /// button bits do not take part.
    #[test]
    fn modifier_table_matches_explorer() {
        assert_eq!(modifier_effect(0), DropEffect::Move);
        assert_eq!(modifier_effect(MK_SHIFT), DropEffect::Move);
        assert_eq!(modifier_effect(MK_CONTROL), DropEffect::Copy);
        assert_eq!(modifier_effect(MK_CONTROL | MK_SHIFT), DropEffect::Link);
        assert_eq!(modifier_effect(MK_ALT), DropEffect::Link);
        assert_eq!(modifier_effect(MK_ALT | MK_CONTROL), DropEffect::Link);
        assert_eq!(modifier_effect(MK_RBUTTON | 0x0001), DropEffect::Move);
        assert_eq!(modifier_effect(MK_RBUTTON | MK_CONTROL), DropEffect::Copy);
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
