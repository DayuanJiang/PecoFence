//! Where the keyboard focus and the text caret are on screen, for plugins (`pf.focus`,
//! `pf.caret`), the light way: the focus window and the caret Windows keeps for the foreground
//! thread (`GetGUIThreadInfo`), and through MSAA (oleacc) the caret object and the focused element
//! of that window. A classic Win32 control has the system caret; Chromium, Edge and Electron serve
//! the MSAA caret for magnifiers. No UI Automation (its client turns on more of a browser's
//! accessibility and needs thousands of lines of bindings).
//!
//! Call from a worker thread with COM initialized ([`crate::com::MtaGuard`]): MSAA asks the
//! focused program, and a program that does not answer must not hold up the UI thread.

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

/// What [`read`] found.
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

/// The focus and the caret of the foreground thread, asked of the window in front now.
pub fn read() -> Reading {
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
    if crate::desktop::window_pid(window) == std::process::id() {
        return Reading::Own;
    }
    let top = crate::desktop::root_ancestor(window);
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
    let (focus, focus_by) = match msaa_focus(window) {
        Some(r) => (Some(r), "msaa"),
        None => (
            ScreenRect::from_rect(crate::window::window_rect(window)),
            "window",
        ),
    };
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
    location(&acc, CHILDID_SELF)
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
