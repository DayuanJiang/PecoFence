//! Where the keyboard focus and the text caret are on screen, for plugins (`pf.focus`,
//! `pf.caret`): the focus window and the caret Windows keeps for the foreground thread
//! (`GetGUIThreadInfo`), and through MSAA (oleacc) the caret object and the focused element of
//! that window. A classic Win32 control has the system caret; Chromium, Edge and Electron serve
//! the MSAA caret for magnifiers. Only where MSAA's focus has no size (in Chromium it stops at the
//! document) UI Automation is asked for the focused element: two calls on interfaces written out
//! here, no generated bindings.
//!
//! A [`Reader`] lives on one worker thread with COM initialized ([`crate::com::MtaGuard`], made
//! before the reader so that it outlives it): MSAA and UI Automation ask the focused program, and
//! one that does not answer must not hold up the UI thread.

use crate::bindings::*;

/// A rectangle in screen pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl ScreenRect {
    fn from_rect(r: RECT) -> Option<Self> {
        (r.right > r.left || r.bottom > r.top).then_some(Self {
            x: r.left,
            y: r.top,
            width: r.right - r.left,
            height: r.bottom - r.top,
        })
    }
}

/// What [`Reader::read`] found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    /// The focused element and the caret of the window in front (each None when it has none).
    Seen {
        focus: Option<ScreenRect>,
        caret: Option<ScreenRect>,
        /// Where each came from, for the log: `gui`, `msaa`, `window` or `none`.
        focus_by: &'static str,
        caret_by: &'static str,
    },
    /// The window in front is one of this process's: what was seen before still holds.
    Own,
    /// No window in front, or one that does not answer (hung): nothing is asked of it.
    Unknown,
}

/// How often the focused element is looked up again while the window and the caret stay as they
/// were (a Tab onto a button moves the focus without a caret). Each lookup asks the program in
/// front (MSAA, maybe UI Automation) and makes it answer: measured in Edge on ec2-4 at 10 a
/// second, 2 to 3 % of a core in Edge and as much in PecoFence.
const FOCUS_AGAIN: std::time::Duration = std::time::Duration::from_millis(500);

/// What the focused element was last looked up for (window, caret), when, and what was found.
type FocusSeen = (
    isize,
    Option<ScreenRect>,
    std::time::Instant,
    Option<ScreenRect>,
    &'static str,
);

/// Reads the focus and the caret on one thread; holds that thread's UI Automation object (made
/// the first time MSAA's focus has no size) and the last focus found.
#[derive(Default)]
pub struct Reader {
    automation: Option<uia::Com>,
    last: Option<FocusSeen>,
}

impl Reader {
    /// The focus and the caret of the foreground thread, asked of the window in front now.
    pub fn read(&mut self) -> Reading {
        read(self)
    }

    /// The focused element (and where it came from), looked up again when the window or the
    /// caret changed or [`FOCUS_AGAIN`] after the last time: MSAA's, else UI Automation's when
    /// MSAA's has no size, else the window's own rectangle.
    fn focus(
        &mut self,
        window: HWND,
        caret: Option<ScreenRect>,
    ) -> (Option<ScreenRect>, &'static str) {
        if let Some((w, c, at, focus, by)) = self.last
            && w == window.0 as isize
            && c == caret
            && at.elapsed() < FOCUS_AGAIN
        {
            return (focus, by);
        }
        let (focus, by) = match msaa_focus(window) {
            Some(r) => (Some(r), "msaa"),
            None => match uia::focused(&mut self.automation) {
                Some(r) => (Some(r), "uia"),
                None => (
                    ScreenRect::from_rect(crate::window::window_rect(window)),
                    "window",
                ),
            },
        };
        let now = std::time::Instant::now();
        self.last = Some((window.0 as isize, caret, now, focus, by));
        (focus, by)
    }
}

fn read(reader: &mut Reader) -> Reading {
    let mut info = GUITHREADINFO {
        cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: `info` is a live GUITHREADINFO with its size set; thread 0 is the foreground one.
    if !unsafe { GetGUIThreadInfo(0, &mut info) }.as_bool() {
        return Reading::Unknown;
    }
    let window = if info.hwndFocus.0.is_null() {
        info.hwndActive
    } else {
        info.hwndFocus
    };
    if window.0.is_null() {
        return Reading::Unknown;
    }
    // One of ours by its top-level window: the settings page's focus is in WebView2's child
    // window, whose process is the browser's.
    let top = crate::desktop::root_ancestor(window);
    let ours = |h: HWND| !h.0.is_null() && crate::desktop::window_pid(h) == std::process::id();
    if ours(window) || ours(top) || ours(info.hwndActive) {
        return Reading::Own;
    }
    // SAFETY: plain FFI call on a window handle (a stale one gives false).
    if unsafe { IsHungAppWindow(top) }.as_bool() {
        return Reading::Unknown;
    }
    let (caret, caret_by) = match system_caret(&info) {
        Some(r) => (Some(r), "gui"),
        None => match msaa_caret(window) {
            Some(r) => (Some(r), "msaa"),
            None => (None, "none"),
        },
    };
    let (focus, focus_by) = reader.focus(window, caret);
    Reading::Seen {
        focus,
        caret,
        focus_by,
        caret_by,
    }
}

/// The caret Windows keeps for the thread (`rcCaret`, in its window's client coordinates).
fn system_caret(info: &GUITHREADINFO) -> Option<ScreenRect> {
    let r = info.rcCaret;
    if info.hwndCaret.0.is_null() || r.bottom <= r.top {
        return None;
    }
    let mut corners = [
        POINT {
            x: r.left,
            y: r.top,
        },
        POINT {
            x: r.right,
            y: r.bottom,
        },
    ];
    for p in &mut corners {
        // SAFETY: out-pointer to a local POINT; a stale window leaves it false.
        if !unsafe { ClientToScreen(info.hwndCaret, p) }.as_bool() {
            return None;
        }
    }
    ScreenRect::from_rect(RECT {
        left: corners[0].x,
        top: corners[0].y,
        right: corners[1].x,
        bottom: corners[1].y,
    })
}

fn child(id: i32) -> VARIANT {
    let mut v = VARIANT::default();
    // SAFETY: writing the tag and the matching union member of a zeroed VARIANT.
    unsafe {
        let inner = &mut *v.Anonymous.Anonymous;
        inner.vt = VARTYPE(VT_I4 as u16);
        inner.Anonymous.lVal = id;
    }
    v
}

fn location(acc: &IAccessible, id: i32) -> Option<ScreenRect> {
    let (mut x, mut y, mut w, mut h) = (0, 0, 0, 0);
    // SAFETY: out-pointers to locals; the VARIANT is a VT_I4 child id.
    unsafe { acc.accLocation(&mut x, &mut y, &mut w, &mut h, &child(id)) }
        .ok()
        .ok()?;
    (w > 0 || h > 0).then_some(ScreenRect {
        x,
        y,
        width: w,
        height: h,
    })
}

/// The MSAA caret object of the focus window (what Chromium keeps for magnifiers).
fn msaa_caret(window: HWND) -> Option<ScreenRect> {
    // SAFETY: plain FFI call; it asks the window's program (WM_GETOBJECT).
    let acc: IAccessible =
        unsafe { AccessibleObjectFromWindow(window, OBJID_CARET as u32) }.ok()?;
    // Chromium leaves its caret where the last text box had it, 0 wide, once the focus is on a
    // button: a caret that is not drawn is none.
    location(&acc, CHILDID_SELF).filter(|r| r.width > 0 && r.height > 0)
}

/// The focused element of the focus window: MSAA's focus, followed down through the objects
/// that hold it (at most 12 levels).
fn msaa_focus(window: HWND) -> Option<ScreenRect> {
    // SAFETY: plain FFI call; it asks the window's program (WM_GETOBJECT).
    let mut acc: IAccessible =
        unsafe { AccessibleObjectFromWindow(window, OBJID_CLIENT as u32) }.ok()?;
    for depth in 0..12 {
        // SAFETY: a live IAccessible; the VARIANT is ours to clear.
        let Ok(mut v) = (unsafe { acc.accFocus() }) else {
            return None;
        };
        // SAFETY: reading the tag, then the member it names.
        let (vt, next, id) = unsafe {
            let inner = &*v.Anonymous.Anonymous;
            let next = if inner.vt == VARTYPE(VT_DISPATCH as u16) {
                (*inner.Anonymous.pdispVal)
                    .as_ref()
                    .and_then(|d| windows_core::Interface::cast::<IAccessible>(d).ok())
            } else {
                None
            };
            (inner.vt.0, next, inner.Anonymous.lVal)
        };
        // SAFETY: the VARIANT came from accFocus; clearing releases its object.
        unsafe {
            let _ = VariantClear(&mut v as *mut VARIANT as *mut VARIANTARG);
        }
        match (vt, next) {
            (t, _) if t == VT_I4 as u16 => return location(&acc, id),
            (_, Some(next)) => acc = next,
            // Nothing below holds the focus: the object reached is the focused one (none at all
            // at the top: the window's own rectangle stands in).
            _ if depth > 0 => return location(&acc, CHILDID_SELF),
            _ => return None,
        }
    }
    location(&acc, CHILDID_SELF)
}

/// UI Automation's focused element, for a window whose MSAA focus has no size: two calls on the
/// interfaces written out by their vtable slots (from UIAutomationClient.h), no generated
/// bindings. The automation object is made once a [`Reader`] needs it and released with it.
mod uia {
    use super::*;
    use core::ffi::c_void;
    use windows_core::{GUID, HRESULT};

    const CLSID_CUIAUTOMATION: GUID = GUID::from_u128(0xff48dba4_60ef_4201_aa87_54103eef594e);
    const IID_IUIAUTOMATION: GUID = GUID::from_u128(0x30cbe57d_d9d0_452a_ab13_7ac5ac4825ee);
    /// The vtable slots used: IUnknown::Release, IUIAutomation::GetFocusedElement,
    /// IUIAutomationElement::get_CurrentBoundingRectangle.
    const RELEASE: usize = 2;
    const GET_FOCUSED_ELEMENT: usize = 8;
    const CURRENT_BOUNDING_RECTANGLE: usize = 43;

    /// A COM object this module holds one reference to.
    pub(super) struct Com(*mut c_void);

    /// Slot `i` of `this`'s vtable.
    ///
    /// # Safety
    /// `this` is a live COM object whose vtable has more than `i` slots.
    unsafe fn slot(this: *mut c_void, i: usize) -> *const c_void {
        // SAFETY: a COM object starts with a pointer to its vtable of function pointers.
        unsafe { *(*(this as *const *const *const c_void)).add(i) }
    }

    impl Drop for Com {
        fn drop(&mut self) {
            // SAFETY: `self.0` is a live object we hold one reference to; Release takes it.
            unsafe {
                let release: unsafe extern "system" fn(*mut c_void) -> u32 =
                    core::mem::transmute(slot(self.0, RELEASE));
                release(self.0);
            }
        }
    }

    /// The bounding rectangle of the element that has the keyboard focus, in screen pixels.
    pub(super) fn focused(automation: &mut Option<Com>) -> Option<ScreenRect> {
        if automation.is_none() {
            windows_core::link!("ole32.dll" "system" fn CoCreateInstance(rclsid : *const GUID, outer : *mut c_void, context : u32, riid : *const GUID, ppv : *mut *mut c_void) -> HRESULT);
            let mut made = core::ptr::null_mut();
            // SAFETY: out-pointer to a local; an in-process server, as UI Automation's client is.
            let hr = unsafe {
                CoCreateInstance(
                    &CLSID_CUIAUTOMATION,
                    core::ptr::null_mut(),
                    CLSCTX_INPROC_SERVER,
                    &IID_IUIAUTOMATION,
                    &mut made,
                )
            };
            if hr.is_err() || made.is_null() {
                tracing::debug!(%hr, "focus: no UI Automation object");
                return None;
            }
            tracing::info!(
                "focus: UI Automation asked for the focused element (MSAA's had no size)"
            );
            *automation = Some(Com(made));
        }
        let auto = automation.as_ref()?.0;
        let mut element = core::ptr::null_mut();
        // SAFETY: `auto` is a live IUIAutomation; slot 8 is GetFocusedElement(**element).
        let hr = unsafe {
            let get: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT =
                core::mem::transmute(slot(auto, GET_FOCUSED_ELEMENT));
            get(auto, &mut element)
        };
        if hr.is_err() || element.is_null() {
            return None;
        }
        let element = Com(element);
        let mut r = RECT::default();
        // SAFETY: a live IUIAutomationElement; slot 43 is get_CurrentBoundingRectangle(*RECT).
        let hr = unsafe {
            let rect: unsafe extern "system" fn(*mut c_void, *mut RECT) -> HRESULT =
                core::mem::transmute(slot(element.0, CURRENT_BOUNDING_RECTANGLE));
            rect(element.0, &mut r)
        };
        if hr.is_err() {
            return None;
        }
        ScreenRect::from_rect(r).filter(|s| s.width > 0 && s.height > 0)
    }
}
