//! Double-click on a fence's empty background: the fence grows to a centred box on its
//! monitor and every icon in it grows with it; a second double-click on the background puts
//! it back, icons included.
//!
//! The zoom is a window-local, transient state: the window moves itself and never tells the
//! app's layout about it, so no other fence is pushed, moved or re-saved while it runs. The
//! saved rect of the fence stays the one it had, which is also the rect the next close
//! animates back to (`zoom_home`).
//!
//! Icons follow the window because [`FenceViewState::icon_dip`] multiplies the fence's icon
//! size by `zoom_icon`: the cell, the grid and every hit test are computed from that one
//! number, so the layout stays the same shape (same columns, same order) at any factor. The
//! factor stops where the 图标大小 setting stops (256 DIP), so a zoom never asks the icon
//! cache for a size the rest of the app cannot be set to.
//!
//! The tween is interruptible: a second double-click during it turns the same movement
//! around from wherever the fence is, instead of being swallowed or restarting from the old
//! end. Only the fence that started the zoom claims it (`zoom_owner`); any gesture on
//! another fence claims it in turn and the app closes the one before.

use super::*;

/// The share of the work area a zoomed fence takes, width and height.
pub(super) const ZOOM_RATIO: f32 = 0.72;
/// Length of the zoom tween. `motion::NORMAL` (250 ms) is Fluent's medium move; a window
/// that also scales its contents reads better a touch longer.
pub(super) const ZOOM_MS: u64 = 280;
/// The largest icon edge the 图标大小 setting offers (24–256 DIP). A zoom stops here too.
const ICON_MAX_DIP: f32 = 256.0;

/// The running zoom: a rect and an icon factor, eased by one tween so they stay in step
/// (the icon factor is the width factor, which keeps the column count constant).
pub(super) struct ZoomAnim {
    from: RECT,
    to: RECT,
    k_from: f32,
    k_to: f32,
    tween: Tween,
    /// Playing outwards (towards the centred box); `false` plays back to `zoom_home`.
    opening: bool,
}

impl FenceViewState {
    /// The fence is in a zoom cycle: opening, zoomed at rest, or closing.
    pub(super) fn zoom_busy(&self) -> bool {
        self.zoom_home.is_some()
    }

    /// The fence is zoomed or on its way out there (not closing): it may own the screen.
    pub(super) fn zoom_open(&self) -> bool {
        self.zoom_home.is_some() && self.zoom_anim.as_ref().is_none_or(|a| a.opening)
    }

    /// The pointer is on the fence's empty background: no item, no title row, no chrome.
    ///
    /// The double-click handler has already returned for every piece of chrome it knows
    /// (divider, chevron, note, script row, button, scrollbar, tab) before it gets here; the
    /// checks below are the ones that also matter for a double-click that reached the end.
    pub(super) fn zoom_blank_at(&self, x: i32, y: i32) -> bool {
        // 双击空白放大 (`zoom.enabled`), off by default, refuses a new zoom; one that is open
        // still answers the double-click that puts it back.
        if !self.behavior.zoom_enabled.get() && !self.zoom_busy() {
            return false;
        }
        // 像桌面一样排列 (`desktop_layout`) is laid out by the app around the other fences; a
        // zoom would be fought by that pass every frame, so the gesture is off there.
        // A rolled fence that hover-peeks is open only while the pointer stays: not zoomed.
        // A roll, a height glide or a move in flight would carry on after the zoom and land
        // with the zoomed rect: no zoom until it is over.
        !self.bare()
            && !self.rolled_up
            && !self.peeking
            && self.roll_anim.is_none()
            && self.height_anim.is_none()
            && self.glide.is_none()
            && self.hit_item(x, y).is_none()
            && !self.title_row_at(x, y)
            && !self.chevron_at(x, y)
            && !self.up_button_at(x, y)
            && !self.count_capsule_at(x, y)
            && !self.lock_glyph_at(x, y)
            && !self.in_note_text(x, y)
            && self.tab_at(x, y).is_none()
    }

    /// A double-click on the background. Opening, zoomed at rest and closing all answer to
    /// it: the gesture always turns the fence towards the other end of the same movement,
    /// starting from where the fence is now (an interrupted tween reverses on the spot).
    pub(super) fn zoom_toggle(&mut self, hwnd: HWND, now: Instant) {
        match self.zoom_home {
            // Closing and clicked again: turn back outwards, to the box it was heading for.
            Some(_) if self.zoom_anim.as_ref().is_some_and(|a| !a.opening) => {
                let (to, k) = match self.zoom_far {
                    Some(far) => far,
                    None => return,
                };
                self.begin(hwnd, to, k, true, now);
            }
            // Open (at rest or on the way out): go back.
            Some(home) => self.begin(hwnd, home, 1.0, false, now),
            // Not zoomed: grow to the centred box of this monitor.
            None => {
                let Some((to, k)) = centred_target(self, hwnd) else {
                    return;
                };
                self.zoom_far = Some((to, k));
                self.begin(hwnd, to, k, true, now);
            }
        }
    }

    fn begin(&mut self, hwnd: HWND, to: RECT, k_to: f32, opening: bool, now: Instant) {
        let from = window::window_rect(hwnd);
        let first = opening && self.zoom_home.is_none();
        if opening {
            // The rect to go back to, kept for the whole cycle so a reversal still knows it.
            if first {
                self.zoom_home = Some(from);
            }
            // Above the other fences: the double-click's first press raised it already.
            *self.behavior.zoom_owner.borrow_mut() = Some(self.fence_id);
        }
        tracing::debug!(
            fence = %self.fence_id,
            from = ?(from.left, from.top, from.right, from.bottom),
            to = ?(to.left, to.top, to.right, to.bottom),
            k = k_to,
            opening,
            "zoom tween"
        );
        self.zoom_anim = Some(ZoomAnim {
            from,
            to,
            k_from: self.zoom_icon,
            k_to,
            tween: Tween::new(
                0.0,
                1.0,
                // 降低动效 (Windows "show animations" off) applies the ends at once.
                if self.motion.enabled() {
                    Duration::from_millis(ZOOM_MS)
                } else {
                    Duration::ZERO
                },
                Curve::Decelerate,
                now,
            ),
            opening,
        });
        self.frames.request();
    }

    /// The app closes a zoom another fence took over: the same gesture in reverse, from
    /// wherever this fence is (a tween still opening is turned around, not restarted).
    pub(super) fn close_zoom(&mut self, hwnd: HWND) {
        let Some(home) = self.zoom_home else {
            return;
        };
        if self.zoom_anim.as_ref().is_some_and(|a| !a.opening) {
            return;
        }
        let now = Instant::now();
        self.begin(hwnd, home, 1.0, false, now);
    }

    /// One frame of the zoom. Returns the window rect to apply and whether the tween ended;
    /// `zoom_icon` is already updated, so the repaint that follows the resize lays the
    /// content out at the new scale.
    pub(super) fn zoom_step(&mut self, hwnd: HWND, now: Instant) -> Option<(RECT, bool)> {
        let z = self.zoom_anim.take()?;
        let t = z.tween.value_at(now);
        let done = z.tween.is_done(now);
        let opening = z.opening;
        let rect = lerp_rect(z.from, z.to, t);
        self.zoom_icon = z.k_from + (z.k_to - z.k_from) * t;
        if done {
            self.zoom_icon = if z.opening { z.k_to } else { 1.0 };
            tracing::debug!(
                fence = %self.fence_id,
                rect = ?(rect.left, rect.top, rect.right, rect.bottom),
                k = self.zoom_icon,
                cell_w = self.grid_metrics().cell_w,
                icon_dip = self.icon_dip(),
                opacity = self.opacity,
                opening = z.opening,
                "zoom settled"
            );
            if !z.opening {
                self.zoom_closed();
            }
        } else {
            self.zoom_anim = Some(z);
        }
        if done && opening && self.zoom_press.is_none() {
            // A click on the desktop, outside this fence, puts it back as well.
            self.zoom_press = window::PressWatch::start(hwnd, WM_APP_ZOOM_PRESS);
        }
        Some((rect, done))
    }

    /// The zoom is over: the press watch ends, no fence owns the screen any more, and the app
    /// measures the fence's auto height again (it waited while the fence was zoomed).
    fn zoom_closed(&mut self) {
        self.zoom_press = None;
        self.zoom_home = None;
        self.zoom_far = None;
        self.zoom_icon = 1.0;
        if *self.behavior.zoom_owner.borrow() == Some(self.fence_id) {
            *self.behavior.zoom_owner.borrow_mut() = None;
        }
        self.queue.push(Command::ZoomEnded(self.fence_id));
    }

    /// Ends the zoom at once, opening, at rest or closing: the app is about to place or size
    /// the fence (a display change, a roll, an undo, the command line, 适应内容大小...), and
    /// what it measures and saves must be the fence's own rect and icons, never the zoomed
    /// ones. Returns the rect the window goes back to, for the caller to apply with no borrow
    /// held; `None` when the fence is not zoomed.
    pub(super) fn end_zoom_now(&mut self) -> Option<RECT> {
        let home = self.zoom_home?;
        tracing::debug!(
            fence = %self.fence_id,
            home = ?(home.left, home.top, home.right, home.bottom),
            "zoom ended at once: the fence is placed or sized"
        );
        self.zoom_anim = None;
        self.zoom_closed();
        Some(home)
    }

    /// The icon edge pictures are fetched at: during the tween the size it ends at, so a zoom
    /// asks the icon cache for one new size instead of one per frame (the frames in between
    /// draw the picture they have, scaled).
    pub(super) fn icon_fetch_dip(&self) -> f32 {
        let k = self
            .zoom_anim
            .as_ref()
            .map_or(self.zoom_icon, |a| if a.opening { a.k_to } else { 1.0 });
        crate::layout::icon_dip(self.layout, self.icon_size) * k
    }
}

impl FenceViewState {
    /// A mouse button went down at `pt` (screen) while the fence is zoomed: a press that
    /// lands outside it, above the taskbar, closes the zoom. Returns whether it did.
    pub(super) fn zoom_press_at(&mut self, hwnd: HWND, pt: pecofence_platform::POINT) -> bool {
        if self.zoom_press.is_none() {
            return false;
        }
        let r = window::window_rect(hwnd);
        if pt.x >= r.left && pt.x < r.right && pt.y >= r.top && pt.y < r.bottom {
            // Inside the zoomed fence: its own clicks, not a dismissal.
            return false;
        }
        // The taskbar and the other shell bands of the monitor pressed on are not the desktop;
        // the desktop of any monitor is.
        if let Some(mon) = monitors::query(monitors::monitor_from_point(pt.x, pt.y)) {
            let w = mon.work_area;
            if pt.y < w.top || pt.y >= w.bottom || pt.x < w.left || pt.x >= w.right {
                return false;
            }
        }
        self.close_zoom(hwnd);
        true
    }
}

/// The centred box of the fence's monitor and the icon factor that fills it with the same
/// column count: the width ratio, held to what the 图标大小 setting can express.
fn centred_target(v: &FenceViewState, hwnd: HWND) -> Option<(RECT, f32)> {
    let cur = window::window_rect(hwnd);
    let (cw, ch) = (cur.right - cur.left, cur.bottom - cur.top);
    if cw <= 0 || ch <= 0 {
        return None;
    }
    let mon = monitors::query(monitors::monitor_from_window(hwnd))?;
    let to = centred_box(mon.work_area, ZOOM_RATIO);
    let ratio = (to.right - to.left) as f32 / cw as f32;
    // The built-in icon size stops at 256 DIP; past that the fence would show an icon size
    // no fence can be set to, and the cache would have nothing that large to give.
    let base = crate::layout::icon_dip(v.layout, v.icon_size).max(1.0);
    let room = (ICON_MAX_DIP / base).max(1.0);
    let k = ratio.clamp(1.05, room.min(4.0));
    // The box is as wide as the icons grow: wider, the grid would take more columns.
    Some((narrowed(to, (cw as f32 * k).round() as i32), k))
}

/// `b` at most `w` wide, about the same centre.
fn narrowed(b: RECT, w: i32) -> RECT {
    if w >= b.right - b.left {
        return b;
    }
    let left = b.left + (b.right - b.left - w) / 2;
    RECT {
        left,
        right: left + w,
        ..b
    }
}

/// A box of `ratio` of the work area, centred on it.
fn centred_box(work: RECT, ratio: f32) -> RECT {
    let (ww, wh) = (work.right - work.left, work.bottom - work.top);
    let w = (ww as f32 * ratio).round() as i32;
    let h = (wh as f32 * ratio).round() as i32;
    let left = work.left + (ww - w) / 2;
    let top = work.top + (wh - h) / 2;
    RECT {
        left,
        top,
        right: left + w,
        bottom: top + h,
    }
}

fn lerp_rect(a: RECT, b: RECT, t: f32) -> RECT {
    let mix = |x: i32, y: i32| x + ((y - x) as f32 * t).round() as i32;
    RECT {
        left: mix(a.left, b.left),
        top: mix(a.top, b.top),
        right: mix(a.right, b.right),
        bottom: mix(a.bottom, b.bottom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(l: i32, t: i32, rr: i32, b: i32) -> RECT {
        RECT {
            left: l,
            top: t,
            right: rr,
            bottom: b,
        }
    }

    fn tup(x: RECT) -> (i32, i32, i32, i32) {
        (x.left, x.top, x.right, x.bottom)
    }

    #[test]
    fn the_zoom_box_is_a_centred_share_of_the_work_area() {
        let work = r(0, 0, 1920, 1020);
        let b = centred_box(work, ZOOM_RATIO);
        assert_eq!((b.right - b.left, b.bottom - b.top), (1382, 734));
        assert_eq!(
            (b.left + b.right) / 2,
            (work.left + work.right) / 2,
            "the box is not centred horizontally"
        );
        assert_eq!(
            (b.top + b.bottom) / 2,
            (work.top + work.bottom) / 2,
            "the box is not centred vertically"
        );
    }

    #[test]
    fn a_second_monitor_centres_in_its_own_work_area() {
        // A work area that does not start at the origin (a monitor to the right, a taskbar
        // on the left): the box stays inside it and keeps its centre.
        let work = r(1920, 40, 3840, 1060);
        let b = centred_box(work, ZOOM_RATIO);
        assert!(b.left >= work.left && b.right <= work.right);
        assert!(b.top >= work.top && b.bottom <= work.bottom);
        assert_eq!((b.left + b.right) / 2, (work.left + work.right) / 2);
        assert_eq!((b.top + b.bottom) / 2, (work.top + work.bottom) / 2);
    }

    #[test]
    fn a_box_the_icons_cannot_fill_is_narrowed_about_its_centre() {
        let b = r(224, 119, 1376, 732);
        assert_eq!(tup(narrowed(b, 400)), (600, 119, 1000, 732));
        assert_eq!(tup(narrowed(b, 2000)), tup(b));
    }

    #[test]
    fn the_tween_covers_the_two_rects_exactly() {
        let home = r(200, 200, 700, 500);
        let far = r(269, 143, 1651, 877);
        assert_eq!(tup(lerp_rect(home, far, 0.0)), tup(home));
        assert_eq!(tup(lerp_rect(home, far, 1.0)), tup(far));
        let mid = lerp_rect(home, far, 0.5);
        assert!(mid.left > home.left && mid.left < far.left);
        assert!(mid.bottom > home.bottom && mid.bottom < far.bottom);
    }
}
