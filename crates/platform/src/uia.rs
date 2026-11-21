//! Where the keyboard focus and the text insertion point are, for the Magnifier's Follow the
//! keyboard focus and Follow the text insertion point.
//!
//! The answers come from UI Automation, as the Windows Magnifier gets them: the element that has
//! the focus and its rectangle, and the caret of the text control that has it, from the most exact
//! source down (the caret range of a TextPattern2, the caret object of the focused window that
//! Chromium keeps for magnifiers, the end of the selection of a TextPattern, the system caret).
//! These calls go to the other program and wait for it, and one that does not answer would hold
//! the caller, so they are made on a worker thread of their own that polls, and the caller only
//! reads the last answer ([`FocusWatcher::latest`]), which never waits.

use crate::bindings::{ClientToScreen, GUITHREADINFO, GetGUIThreadInfo, POINT};
use crate::com::MtaGuard;
use crate::uia_bindings as b;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use windows_core::{IUnknown, Interface};

/// `UIA_TextPatternId` and `UIA_TextPattern2Id`.
const TEXT_PATTERN: i32 = 10014;
const TEXT_PATTERN2: i32 = 10024;
/// `VT_I4`.
const VT_I4: u16 = 3;
/// `TextPatternRangeEndpoint_Start`.
const RANGE_START: i32 = 0;
/// `TextUnit_Character`.
const TEXT_UNIT_CHARACTER: i32 = 0;

/// How long the worker waits between two polls while something is asked for, and while nothing is.
const POLL: Duration = Duration::from_millis(50);
const IDLE: Duration = Duration::from_millis(250);

/// A rectangle on the screen in pixels: `[left, top, right, bottom]`.
pub type ScreenRect = [i32; 4];

/// The last answers. A `*_seq` grows each time its rectangle changes, so the caller sees a move
/// without comparing rectangles.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Places {
    /// The element that has the keyboard focus.
    pub focus: Option<ScreenRect>,
    pub focus_seq: u64,
    /// The text insertion point (a thin rectangle) of the focused text control.
    pub caret: Option<ScreenRect>,
    pub caret_seq: u64,
}

struct Shared {
    places: Mutex<Places>,
    want_focus: AtomicBool,
    want_caret: AtomicBool,
    /// The [`FocusWatcher`] is gone: the worker ends.
    stop: AtomicBool,
}

/// Watches the focus and the caret on a worker thread while one of them is asked for
/// ([`FocusWatcher::want`]). Dropping it ends the worker (at its next poll: it never waits for
/// it, so a program that does not answer holds only the worker).
pub struct FocusWatcher {
    shared: Arc<Shared>,
}

impl FocusWatcher {
    /// Starts the worker. It asks for nothing until [`FocusWatcher::want`] says so.
    pub fn start() -> Self {
        let shared = Arc::new(Shared {
            places: Mutex::new(Places::default()),
            want_focus: AtomicBool::new(false),
            want_caret: AtomicBool::new(false),
            stop: AtomicBool::new(false),
        });
        let worker = shared.clone();
        let spawned = std::thread::Builder::new()
            .name("uia-focus".into())
            .spawn(move || run(&worker));
        if let Err(e) = spawned {
            tracing::warn!(error = %e, "focus watcher: the worker thread did not start");
        }
        Self { shared }
    }

    /// What the worker is to look for: the focused element, the caret. Nothing keeps it idle.
    pub fn want(&self, focus: bool, caret: bool) {
        self.shared.want_focus.store(focus, Ordering::Relaxed);
        self.shared.want_caret.store(caret, Ordering::Relaxed);
    }

    /// The last answers; never waits for another program.
    pub fn latest(&self) -> Places {
        *self.shared.places.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl Drop for FocusWatcher {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Relaxed);
    }
}

fn run(shared: &Shared) {
    let Ok(_mta) = MtaGuard::init() else {
        tracing::warn!("focus watcher: COM did not start");
        return;
    };
    let mut automation: Option<b::IUIAutomation> = None;
    while !shared.stop.load(Ordering::Relaxed) {
        let focus = shared.want_focus.load(Ordering::Relaxed);
        let caret = shared.want_caret.load(Ordering::Relaxed);
        if !focus && !caret {
            std::thread::sleep(IDLE);
            continue;
        }
        if automation.is_none() {
            // SAFETY: plain COM call; the class is in-process in Windows itself.
            automation = unsafe {
                b::CoCreateInstance(
                    &b::CUIAutomation,
                    None::<&IUnknown>,
                    b::CLSCTX_INPROC_SERVER,
                )
            }
            .map_err(|e| tracing::warn!(error = %e, "focus watcher: UI Automation did not start"))
            .ok();
        }
        if let Some(ui) = &automation {
            let (focus_rect, caret_rect) = look(ui, focus, caret);
            let mut guard = shared.places.lock().unwrap_or_else(|e| e.into_inner());
            let places = &mut *guard;
            update(&mut places.focus, &mut places.focus_seq, focus_rect);
            update(&mut places.caret, &mut places.caret_seq, caret_rect);
        }
        std::thread::sleep(if automation.is_some() { POLL } else { IDLE });
    }
}

/// Keeps the last rectangle when there is no new one (the focus is on something with no place,
/// or a program that is slow); counts a rectangle that differs.
fn update(old: &mut Option<ScreenRect>, seq: &mut u64, new: Option<ScreenRect>) {
    if let Some(new) = new
        && *old != Some(new)
    {
        *old = Some(new);
        *seq += 1;
    }
}

/// The focused element's rectangle and the caret's, as far as asked for. Focus in one of
/// PecoFence's own windows (a click in the magnifier's fence, the settings) says nothing: the
/// picture would jump to the fence that shows it.
fn look(
    ui: &b::IUIAutomation,
    focus: bool,
    caret: bool,
) -> (Option<ScreenRect>, Option<ScreenRect>) {
    // SAFETY: plain COM calls on a live interface and element of this thread.
    let element = unsafe {
        let Ok(element) = ui.GetFocusedElement() else {
            return (None, None);
        };
        if element
            .CurrentProcessId()
            .is_ok_and(|pid| pid as u32 == std::process::id())
        {
            return (None, None);
        }
        element
    };
    let focus_rect = focus.then(|| element_rect(&element)).flatten();
    let caret_rect = caret.then(|| caret_rect(&element)).flatten();
    (focus_rect, caret_rect)
}

fn element_rect(element: &b::IUIAutomationElement) -> Option<ScreenRect> {
    // SAFETY: plain COM calls on a live element.
    unsafe {
        if element.CurrentIsOffscreen().ok()?.as_bool() {
            return None;
        }
        let r = element.CurrentBoundingRectangle().ok()?;
        usable([r.left, r.top, r.right, r.bottom])
    }
}

/// A rectangle with an area or at least a place: an empty one at 0,0 is "nowhere".
fn usable(r: ScreenRect) -> Option<ScreenRect> {
    (r[2] >= r[0] && r[3] >= r[1] && (r[2] > r[0] || r[3] > r[1] || r[0] != 0 || r[1] != 0))
        .then_some(r)
}

/// The caret, from the most exact source down: the caret range of a TextPattern2; the caret
/// object of the focused window (MSAA: what Chromium keeps for screen magnifiers, and right where
/// its text pattern is not); the end of the selection of a TextPattern; the system caret of the
/// foreground thread.
fn caret_rect(element: &b::IUIAutomationElement) -> Option<ScreenRect> {
    uia_caret2(element)
        .or_else(msaa_caret)
        .or_else(|| uia_selection_end(element))
        .or_else(system_caret)
}

/// What the foreground thread has: its focus and caret windows and its system caret.
fn gui_info() -> Option<GUITHREADINFO> {
    let mut info = GUITHREADINFO {
        cbSize: size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: plain FFI call with a sized out-structure.
    unsafe { GetGUIThreadInfo(0, &mut info).as_bool() }.then_some(info)
}

/// The caret object (`OBJID_CARET`) of the window that has the focus: what screen magnifiers ask
/// for. Chromium keeps one for its pages once an accessibility client has been seen.
fn msaa_caret() -> Option<ScreenRect> {
    let info = gui_info()?;
    let hwnd = [info.hwndFocus, info.hwndCaret, info.hwndActive]
        .into_iter()
        .find(|h| !h.0.is_null())?;
    // SAFETY: plain COM calls on a window handle the system just gave; the answers are copied.
    unsafe {
        let caret: b::IAccessible =
            b::AccessibleObjectFromWindow(b::HWND(hwnd.0), b::OBJID_CARET as u32).ok()?;
        let (mut x, mut y, mut w, mut h) = (0, 0, 0, 0);
        // The caret itself: child id 0 (`CHILDID_SELF`), a VT_I4 variant.
        let mut own = b::VARIANT::default();
        core::ptr::write(
            &mut own.Anonymous.Anonymous,
            core::mem::ManuallyDrop::new(b::VARIANT_0_0 {
                vt: b::VARTYPE(VT_I4),
                ..Default::default()
            }),
        );
        caret
            .accLocation(&mut x, &mut y, &mut w, &mut h, &own)
            .ok()
            .ok()?;
        usable([x, y, x + w, y + h])
    }
}

/// The system caret of the foreground thread: programs with no text pattern that still keep a
/// caret for the system (a classic edit control, an editor that follows its caret for screen
/// readers).
fn system_caret() -> Option<ScreenRect> {
    let info = gui_info().filter(|i| !i.hwndCaret.0.is_null())?;
    // SAFETY: plain FFI calls with local points.
    unsafe {
        let r = info.rcCaret;
        if r.right <= r.left && r.bottom <= r.top {
            return None;
        }
        let mut top_left = POINT {
            x: r.left,
            y: r.top,
        };
        let mut bottom_right = POINT {
            x: r.right,
            y: r.bottom,
        };
        if !ClientToScreen(info.hwndCaret, &mut top_left).as_bool()
            || !ClientToScreen(info.hwndCaret, &mut bottom_right).as_bool()
        {
            return None;
        }
        usable([top_left.x, top_left.y, bottom_right.x, bottom_right.y])
    }
}

/// The caret of a TextPattern2: its caret range, when the control has the keyboard focus.
fn uia_caret2(element: &b::IUIAutomationElement) -> Option<ScreenRect> {
    // SAFETY: plain COM calls on live interfaces; the pattern is asked of the focused element.
    unsafe {
        let p = element
            .GetCurrentPattern(b::PATTERNID(TEXT_PATTERN2))
            .ok()?;
        let pattern = p.cast::<b::IUIAutomationTextPattern2>().ok()?;
        let mut active = windows_core::BOOL(0);
        let range = pattern.GetCaretRange(&mut active).ok()?;
        active.as_bool().then(|| range_end(&range)).flatten()
    }
}

/// The end of the selection of a TextPattern: where the caret is when the selection was made
/// forward, and the insertion point itself when it is empty.
fn uia_selection_end(element: &b::IUIAutomationElement) -> Option<ScreenRect> {
    // SAFETY: plain COM calls on live interfaces; the pattern is asked of the focused element.
    unsafe {
        let p = element.GetCurrentPattern(b::PATTERNID(TEXT_PATTERN)).ok()?;
        let pattern = p.cast::<b::IUIAutomationTextPattern>().ok()?;
        let selection = pattern.GetSelection().ok()?;
        if selection.Length().ok()? < 1 {
            return None;
        }
        range_end(&selection.GetElement(0).ok()?)
    }
}

/// A thin rectangle at the end of a text range: the right edge of the last rectangle it covers
/// (a selection of several words or lines ends where the caret is, not in its middle). A range
/// with none (the insertion point between two characters) takes the left edge of the character
/// after it, or at the end of the text, where typing goes, the right edge of the one before.
///
/// # Safety
/// `range` is a live text range.
unsafe fn range_end(range: &b::IUIAutomationTextRange) -> Option<ScreenRect> {
    // SAFETY: the caller's.
    unsafe {
        if let Some(last) = rects_of(range).last() {
            return usable([last[2], last[1], last[2], last[3]]);
        }
        // The character after the insertion point (on a copy: the range stays as it is).
        if let Ok(after) = range.Clone() {
            let _ = after.ExpandToEnclosingUnit(TEXT_UNIT_CHARACTER);
            if let Some(first) = rects_of(&after).first() {
                return usable([first[0], first[1], first[0], first[3]]);
            }
        }
        // None: the end of the text. The character before it ends where the caret is.
        let before = range.Clone().ok()?;
        before
            .MoveEndpointByUnit(RANGE_START, TEXT_UNIT_CHARACTER, -1)
            .ok()?;
        let last = *rects_of(&before).last()?;
        usable([last[2], last[1], last[2], last[3]])
    }
}

/// The rectangles of a range, one per line (a SAFEARRAY of doubles: left, top, width, height, ...).
///
/// # Safety
/// `range` is a live text range.
unsafe fn rects_of(range: &b::IUIAutomationTextRange) -> Vec<ScreenRect> {
    let mut out = Vec::new();
    // SAFETY: the array is the range's to give; it is read between a lock and an unlock, and
    // destroyed once, as the caller owns it.
    unsafe {
        let Ok(array) = range.GetBoundingRectangles() else {
            return out;
        };
        if array.is_null() {
            return out;
        }
        let low = b::SafeArrayGetLBound(array, 1);
        let high = b::SafeArrayGetUBound(array, 1);
        let mut data: *mut core::ffi::c_void = core::ptr::null_mut();
        if let (Ok(low), Ok(high)) = (low, high)
            && high >= low
            && b::SafeArrayAccessData(array, &mut data).is_ok()
        {
            let count = (high - low + 1) as usize;
            let values = std::slice::from_raw_parts(data as *const f64, count);
            out.extend(values.as_chunks::<4>().0.iter().filter_map(|q| {
                usable([
                    q[0].floor() as i32,
                    q[1].floor() as i32,
                    (q[0] + q[2]).ceil() as i32,
                    (q[1] + q[3]).ceil() as i32,
                ])
            }));
            let _ = b::SafeArrayUnaccessData(array);
        }
        let _ = b::SafeArrayDestroy(array);
    }
    out
}

/// The point the Magnifier centres on for a focused element: its middle, or for an element larger
/// than `limit` pixels in a direction (a document, a whole window) the middle of its first
/// `limit` pixels there, which is where a reader's eyes are.
pub fn focus_point(r: ScreenRect, limit: i32) -> (i32, i32) {
    let reach = |from: i32, to: i32| from + ((to - from).min(limit)) / 2;
    (reach(r[0], r[2]), reach(r[1], r[3]))
}

/// What each way of finding the caret says for the focused element, as text: for the probe
/// (`examples/uia_probe.rs`), to see which one a program answers to.
pub fn describe_focus() -> String {
    let Ok(_mta) = MtaGuard::init() else {
        return "COM did not start".into();
    };
    // SAFETY: plain COM calls on interfaces made and used on this thread.
    unsafe {
        let Ok(ui) = b::CoCreateInstance::<_, b::IUIAutomation>(
            &b::CUIAutomation,
            None::<&IUnknown>,
            b::CLSCTX_INPROC_SERVER,
        ) else {
            return "UI Automation did not start".into();
        };
        let Ok(element) = ui.GetFocusedElement() else {
            return "no focused element".into();
        };
        let name = element
            .CurrentName()
            .map(|n| n.display().to_string())
            .unwrap_or_default();
        let class = element
            .CurrentClassName()
            .map(|n| n.display().to_string())
            .unwrap_or_default();
        let kind = element.CurrentControlType().map(|t| t.0).unwrap_or(0);
        let has = |id: i32| element.GetCurrentPattern(b::PATTERNID(id)).is_ok();
        format!(
            "focus {:?} class {class:?} type {kind} | rect {:?} | TextPattern {} TextPattern2 {} | caret2 {:?} | msaa {:?} | selection end {:?} | system {:?} | used {:?}",
            name.chars().take(30).collect::<String>(),
            element_rect(&element),
            has(TEXT_PATTERN),
            has(TEXT_PATTERN2),
            uia_caret2(&element),
            msaa_caret(),
            uia_selection_end(&element),
            system_caret(),
            caret_rect(&element),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_small_element_is_centred_and_a_large_one_by_its_first_part() {
        assert_eq!(focus_point([100, 200, 140, 220], 400), (120, 210));
        assert_eq!(focus_point([0, 0, 1900, 1000], 400), (200, 200));
        assert_eq!(focus_point([10, 10, 10, 10], 400), (10, 10));
    }

    #[test]
    fn nowhere_is_not_a_place() {
        assert_eq!(usable([0, 0, 0, 0]), None);
        assert_eq!(usable([5, 5, 2, 9]), None);
        assert_eq!(usable([300, 40, 301, 58]), Some([300, 40, 301, 58]));
        assert_eq!(usable([300, 40, 300, 40]), Some([300, 40, 300, 40]));
    }

    #[test]
    fn a_move_counts_and_a_missing_answer_keeps_the_last_place() {
        let (mut rect, mut seq) = (None, 0);
        update(&mut rect, &mut seq, Some([1, 2, 3, 4]));
        update(&mut rect, &mut seq, Some([1, 2, 3, 4]));
        update(&mut rect, &mut seq, None);
        assert_eq!((rect, seq), (Some([1, 2, 3, 4]), 1));
        update(&mut rect, &mut seq, Some([5, 6, 7, 8]));
        assert_eq!((rect, seq), (Some([5, 6, 7, 8]), 2));
    }
}
