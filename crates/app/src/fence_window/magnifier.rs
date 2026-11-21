//! 放大镜: the magnifier row of a plugin fence (`pecofence_plugin_host::view::Magnifier`). The
//! plugin says how (zoom, smoothing, following or holding, the pointer, staying on top); the
//! screen is captured and drawn by PecoFence on the graphics card (`pecofence_render::magnifier`),
//! so the plugin never sees a pixel.
//!
//! The runtime lives as long as the view has a magnifier row. A frame of the frame clock takes
//! the newest capture frame of the monitor the picture is centred on and redraws the content when
//! the picture or its centre changed. The centre is the pointer, the keyboard focus or the text
//! insertion point, whichever moved last among those the row follows ([`Follow`]); the focus and
//! the caret come from a [`FocusWatcher`] (UI Automation, on a thread of its own). A picture that
//! follows none of them stops the capture and keeps the last frame (and, with `shot`, is taken
//! again at the pointer's place).

use super::*;
use pecofence_platform::monitors;
use pecofence_platform::uia::{FocusWatcher, Places, ScreenRect, focus_point};
use pecofence_plugin_host::view::Magnifier as MagnifierSpec;
use pecofence_render::magnifier::Magnifier;

/// How long a monitor's size and scale are trusted before they are asked again.
const MONITOR_TTL: Duration = Duration::from_secs(2);

/// The place shown: a monitor (its `HMONITOR` as an integer), the point in that monitor's pixels
/// and its scale (1.0 at 96 DPI).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Point {
    pub monitor: isize,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

/// How far into a large focused element (a document, a whole window) the picture reaches: the
/// middle of its first part, in pixels.
const FOCUS_REACH: i32 = 400;

/// What the picture is centred on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Pointer,
    Focus,
    Caret,
}

/// Which of the pointer, the focus and the caret the picture goes to: the one that moved last
/// among those the row follows, as the Windows Magnifier does.
#[derive(Default)]
struct Follow {
    cursor: Option<(i32, i32)>,
    focus_seq: u64,
    caret_seq: u64,
    /// The first answers of the watcher were taken as where things already are.
    primed: bool,
    target: Option<Target>,
}

impl Follow {
    /// Notes what moved since the last frame and tells what to centre on. `want` is the row's
    /// (pointer, focus, caret). With none of them wanted (a picture taken at the pointer) it is
    /// the pointer. A target that is no longer wanted, or has no place yet, gives way to the
    /// next: the caret, then the focus, then the pointer.
    fn update(&mut self, want: (bool, bool, bool), cursor: (i32, i32), places: &Places) -> Target {
        let (pointer, focus, caret) = want;
        // The first answers of the watcher are where the focus and the caret already are, not a
        // move: with the pointer followed too, the picture stays at the pointer until one moves.
        if !self.primed && (places.focus_seq > 0 || places.caret_seq > 0) {
            self.primed = true;
            self.focus_seq = places.focus_seq;
            self.caret_seq = places.caret_seq;
        }
        let mut moved = None;
        if self.cursor != Some(cursor) {
            self.cursor = Some(cursor);
            if pointer {
                moved = Some(Target::Pointer);
            }
        }
        if self.focus_seq != places.focus_seq {
            self.focus_seq = places.focus_seq;
            if focus && places.focus.is_some() {
                moved = Some(Target::Focus);
            }
        }
        if self.caret_seq != places.caret_seq {
            self.caret_seq = places.caret_seq;
            if caret && places.caret.is_some() {
                moved = Some(Target::Caret);
            }
        }
        if moved.is_some() {
            self.target = moved;
        }
        let usable = |t: Target| match t {
            Target::Pointer => pointer || !(focus || caret),
            Target::Focus => focus && places.focus.is_some(),
            Target::Caret => caret && places.caret.is_some(),
        };
        self.target
            .filter(|t| usable(*t))
            .or_else(|| {
                [Target::Caret, Target::Focus, Target::Pointer]
                    .into_iter()
                    .find(|t| usable(*t))
            })
            .unwrap_or(Target::Pointer)
    }
}

/// The caret's middle, or the focused element's ([`focus_point`]).
fn centre_of(target: Target, cursor: (i32, i32), places: &Places) -> (i32, i32) {
    let middle = |r: ScreenRect| ((r[0] + r[2]) / 2, (r[1] + r[3]) / 2);
    match target {
        Target::Pointer => cursor,
        Target::Focus => places.focus.map_or(cursor, |r| focus_point(r, FOCUS_REACH)),
        Target::Caret => places.caret.map_or(cursor, middle),
    }
}

/// What the fence keeps for the magnifier row of its view.
pub(super) struct MagnifierRun {
    pub(super) magnifier: Rc<RefCell<Magnifier>>,
    /// The generation of the GPU device the captures were made for.
    generation: u32,
    /// Where the picture is held, or was at the last frame.
    pub(super) point: Option<Point>,
    /// The `shot` of the view; a new one asks for a picture at the pointer's place.
    shot: u32,
    /// A picture is to be taken at the pointer's place before the capture stops again.
    pending: bool,
    /// The capture could not start (protected content, no capture on this Windows): not tried
    /// again before the view changes.
    failed: bool,
    /// The monitor the pointer was on, asked again after [`MONITOR_TTL`].
    monitor: Option<MonitorCache>,
    /// Whether the captures ran at the last frame, for the log.
    live: bool,
    /// Finds the focus and the caret; there while the row follows one of them.
    watcher: Option<FocusWatcher>,
    follow: Follow,
    /// What the log last said the picture follows (a line each time it changes).
    told: Option<(Target, Places)>,
}

/// A monitor as the last tick saw it: its handle, the origin of its pixels, its scale, and when it
/// was asked.
#[derive(Clone, Copy)]
struct MonitorCache {
    handle: isize,
    origin: (i32, i32),
    scale: f32,
    at: Instant,
}

/// What the window must be told after the view changed: stay above the others (or not) and keep
/// out of the captures (or not). Done by the caller with no borrow of the view held: the calls
/// send messages to the window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct MagnifierWindow {
    pub float: Option<bool>,
    pub exclude: Option<bool>,
}

impl MagnifierRun {
    /// The origin and scale of `handle`, asked again after [`MONITOR_TTL`].
    fn monitor(&mut self, handle: monitors::HMONITOR, now: Instant) -> Option<MonitorCache> {
        let key = handle.0 as isize;
        if let Some(m) = self.monitor
            && m.handle == key
            && now.duration_since(m.at) < MONITOR_TTL
        {
            return Some(m);
        }
        let info = monitors::query(handle)?;
        let m = MonitorCache {
            handle: key,
            origin: (info.bounds.left, info.bounds.top),
            scale: info.scale(),
            at: now,
        };
        self.monitor = Some(m);
        Some(m)
    }
}

impl Drop for MagnifierRun {
    fn drop(&mut self) {
        let stats = self.magnifier.borrow().stats;
        tracing::debug!(
            frames = stats.frames,
            unseen = stats.unseen,
            draws = stats.draws,
            starts = stats.starts,
            "magnifier: stopped"
        );
    }
}

impl FenceViewState {
    /// The view's magnifier row, if it has one.
    pub(super) fn magnifier_spec(&self) -> Option<MagnifierSpec> {
        self.script.as_ref()?.rows.iter().find_map(|r| r.magnifier)
    }

    /// The view changed: starts, updates or drops the runtime and asks for frames.
    pub(super) fn sync_magnifier(&mut self) {
        let Some(spec) = self.magnifier_spec() else {
            if self.magnifier.take().is_some() {
                tracing::debug!(fence = %self.active, "magnifier: no more in the view");
            }
            return;
        };
        let generation = self.stack.gpu_generation();
        if self
            .magnifier
            .as_ref()
            .is_none_or(|r| r.generation != generation)
        {
            let (gpu, generation) = self.stack.gpu_device();
            self.magnifier = Some(MagnifierRun {
                magnifier: Rc::new(RefCell::new(Magnifier::new(gpu))),
                generation,
                point: None,
                shot: spec.shot,
                // A picture that does not follow shows one taken now.
                pending: !spec.follows(),
                failed: false,
                monitor: None,
                live: false,
                watcher: None,
                follow: Follow::default(),
                told: None,
            });
            tracing::info!(fence = %self.active, generation, "magnifier: started");
        }
        let Some(run) = self.magnifier.as_mut() else {
            return;
        };
        run.magnifier.borrow_mut().set_cursor(spec.pointer);
        if run.shot != spec.shot {
            run.shot = spec.shot;
            run.pending = true;
        }
        if spec.follows() {
            run.pending = false;
        }
        run.failed = false;
        self.frames.request();
    }

    /// What the window is to be told about the current view (see [`MagnifierWindow`]): only the
    /// changes since the last call.
    pub(super) fn magnifier_window(&mut self) -> MagnifierWindow {
        let spec = self.magnifier_spec();
        let float = spec.as_ref().is_some_and(|m| m.top);
        let exclude = spec.is_some();
        let mut told = MagnifierWindow::default();
        if float != self.magnifier_window_told.0 {
            self.magnifier_window_told.0 = float;
            told.float = Some(float);
        }
        if exclude != self.magnifier_window_told.1 {
            self.magnifier_window_told.1 = exclude;
            told.exclude = Some(exclude);
        }
        told
    }

    /// One frame: takes the newest capture frame and moves the picture to the pointer. Returns
    /// whether the content is to be drawn again and whether another frame is wanted.
    pub(super) fn tick_magnifier(&mut self) -> (bool, bool) {
        let Some(spec) = self.magnifier_spec() else {
            return (false, false);
        };
        let shown =
            !self.rolled_up && !self.off_desktop && !self.retired && desktop::is_visible(self.hwnd);
        // The GPU device was replaced (sleep, driver update): the captures were made for the old
        // one, so they are made again.
        if self
            .magnifier
            .as_ref()
            .is_some_and(|r| r.generation != self.stack.gpu_generation())
        {
            self.sync_magnifier();
        }
        let Some(run) = self.magnifier.as_mut() else {
            return (false, false);
        };
        if run.failed {
            return (false, false);
        }
        let live = shown && (spec.follows() || run.pending);
        // The focus and the caret are looked for only while the picture is live.
        if spec.focus || spec.caret {
            let watcher = run.watcher.get_or_insert_with(FocusWatcher::start);
            watcher.want(live && spec.focus, live && spec.caret);
        } else {
            run.watcher = None;
        }
        run.magnifier.borrow_mut().set_live(live);
        if live != run.live {
            run.live = live;
            tracing::debug!(
                fence = %self.active,
                live,
                follow = spec.follow,
                focus = spec.focus,
                caret = spec.caret,
                pending = run.pending,
                shown,
                zoom = spec.zoom,
                "magnifier: the capture runs or stops"
            );
        }
        if !live {
            return (false, false);
        }
        let now = Instant::now();
        let cursor = window::cursor_pos();
        let cursor = (cursor.x, cursor.y);
        let places = run.watcher.as_ref().map(|w| w.latest()).unwrap_or_default();
        let target = run
            .follow
            .update((spec.follow, spec.focus, spec.caret), cursor, &places);
        // A picture that is only taken (nothing followed) is taken at the pointer.
        let centre = centre_of(target, cursor, &places);
        if run.told != Some((target, places)) {
            run.told = Some((target, places));
            tracing::debug!(fence = %self.active, ?target, ?centre, ?places, "magnifier: what the picture follows");
        }
        let handle = monitors::monitor_from_point(centre.0, centre.1);
        let Some(info) = run.monitor(handle, now) else {
            return (false, true);
        };
        let point = Point {
            monitor: info.handle,
            x: (centre.0 - info.origin.0) as f32,
            y: (centre.1 - info.origin.1) as f32,
            scale: info.scale,
        };
        let moved = run.point != Some(point);
        run.point = Some(point);
        let polled = run
            .magnifier
            .borrow_mut()
            .poll(handle.0 as *mut _, Some((point.x, point.y)));
        let changed = match polled {
            Ok(changed) => changed,
            Err(error) => {
                tracing::warn!(fence = %self.active, %error, "magnifier: the screen cannot be captured");
                run.failed = true;
                return (true, false);
            }
        };
        let mut redraw = changed || moved;
        if run.pending && run.magnifier.borrow().has_picture() && changed {
            // The snapshot is taken: the capture stops, the picture stays.
            run.pending = false;
            run.live = false;
            run.magnifier.borrow_mut().set_live(false);
            redraw = true;
            tracing::debug!(fence = %self.active, point = ?run.point, "magnifier: the picture is taken and held");
        }
        (redraw, spec.follows() || run.pending)
    }
}

impl FenceWindow {
    /// Tells the window what the magnifier row asks of it (above every window, out of the
    /// captures), with no borrow of the view held.
    pub(super) fn apply_magnifier_window(&self) {
        let told = self
            .view
            .borrow_mut()
            .as_mut()
            .map(|v| v.magnifier_window())
            .unwrap_or_default();
        let hwnd = self.hwnd();
        if let Some(exclude) = told.exclude
            && !window::set_capture_excluded(hwnd, exclude)
            && exclude
        {
            tracing::info!(
                ?hwnd,
                "magnifier: this Windows cannot keep the fence out of the capture"
            );
        }
        if let Some(float) = told.float
            && let Ok(mut guard) = self.anchor.try_borrow_mut()
            && let Some(a) = guard.as_mut()
        {
            a.set_floating(hwnd, float);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn places(
        focus: Option<ScreenRect>,
        focus_seq: u64,
        caret: Option<ScreenRect>,
        caret_seq: u64,
    ) -> Places {
        Places {
            focus,
            focus_seq,
            caret,
            caret_seq,
        }
    }

    #[test]
    fn the_picture_goes_to_what_moved_last_among_what_is_followed() {
        let mut f = Follow::default();
        let all = (true, true, true);
        let none = places(None, 0, None, 0);
        assert_eq!(f.update(all, (10, 10), &none), Target::Pointer);
        // The watcher's first answers are where the focus and the caret already are: the picture
        // stays at the pointer.
        let first = places(Some([0, 0, 50, 50]), 1, Some([20, 20, 21, 40]), 1);
        assert_eq!(f.update(all, (10, 10), &first), Target::Pointer);
        // The caret moves, then the pointer, then the focus.
        let typed = places(Some([0, 0, 50, 50]), 1, Some([90, 20, 91, 40]), 2);
        assert_eq!(f.update(all, (10, 10), &typed), Target::Caret);
        assert_eq!(f.update(all, (10, 10), &typed), Target::Caret);
        assert_eq!(f.update(all, (300, 10), &typed), Target::Pointer);
        let tabbed = places(Some([60, 0, 110, 50]), 2, Some([90, 20, 91, 40]), 2);
        assert_eq!(f.update(all, (300, 10), &tabbed), Target::Focus);
    }

    #[test]
    fn what_is_not_followed_does_not_move_the_picture() {
        let mut f = Follow::default();
        let typed = places(Some([0, 0, 50, 50]), 1, Some([20, 20, 21, 40]), 1);
        // Only the focus is followed: the pointer and the caret do nothing.
        let focus_only = (false, true, false);
        assert_eq!(f.update(focus_only, (5, 5), &typed), Target::Focus);
        let more = places(Some([0, 0, 50, 50]), 1, Some([90, 20, 91, 40]), 2);
        assert_eq!(f.update(focus_only, (900, 5), &more), Target::Focus);
        // Nothing followed: a picture taken at the pointer.
        assert_eq!(
            f.update((false, false, false), (900, 5), &more),
            Target::Pointer
        );
    }

    #[test]
    fn a_target_without_a_place_gives_way_to_the_next() {
        let mut f = Follow::default();
        // The caret is followed but the program has none: the focus, then the pointer.
        let focus_only = places(Some([0, 0, 50, 50]), 1, None, 0);
        assert_eq!(
            f.update((false, true, true), (5, 5), &focus_only),
            Target::Focus
        );
        assert_eq!(
            f.update((false, false, true), (5, 5), &focus_only),
            Target::Pointer
        );
        // Only the focus and the caret are followed and neither has a place yet: the pointer.
        let mut g = Follow::default();
        assert_eq!(
            g.update((false, true, true), (5, 5), &places(None, 0, None, 0)),
            Target::Pointer
        );
    }

    #[test]
    fn the_centre_is_the_pointer_the_caret_or_the_focused_element() {
        let p = places(Some([0, 0, 1900, 1000]), 1, Some([100, 200, 102, 220]), 1);
        assert_eq!(centre_of(Target::Pointer, (7, 8), &p), (7, 8));
        assert_eq!(centre_of(Target::Caret, (7, 8), &p), (101, 210));
        assert_eq!(centre_of(Target::Focus, (7, 8), &p), (200, 200));
        assert_eq!(centre_of(Target::Caret, (7, 8), &Places::default()), (7, 8));
    }
}
