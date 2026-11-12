//! Per-frame plumbing: the client-tween tick and the window-height animation phases.

use super::*;

impl FenceViewState {
    /// Advances client-side tweens and per-frame auto-scroll for one frame: the scroll glide,
    /// marquee / drag edge auto-scroll, the scrollbar's width and alpha, and the state fades.
    /// Each surface is redrawn at most once, only when something on it moved. Returns true
    /// while anything still moves (the caller then requests another frame). Must not change
    /// window geometry or send messages synchronously (posting is fine) — the caller does
    /// that with no borrow held.
    pub(super) fn tick(&mut self, now: Instant, draw_chrome: bool) -> bool {
        let mut more = self.tick_shadow_alpha(now);
        let mut content = false;
        let mut glide_done = false;
        if let Some(t) = self.scroll_anim {
            self.scroll_y = t.value_at(now);
            let done = t.is_done(now);
            if done {
                // Exactly the (pixel-snapped) destination, whatever the eased sample was.
                self.scroll_y = t.target();
                self.scroll_anim = None;
                glide_done = true;
            }
            content = true;
            more |= !done;
        }
        if self.marquee_autoscroll(now) {
            content = true;
            more = true;
        }
        if self.drag_autoscroll(now) {
            content = true;
            more = true;
        }
        if self.tab_strip_autoscroll(now) {
            more = true;
        }
        if self.sb_busy {
            content = true;
            self.sb_arrow_fades.prune(now, |_| true);
            self.sb_busy = !self.sb_width.is_done(now)
                || !self.sb_alpha.is_done(now)
                || !self.sb_parts.is_done(now)
                || self.sb_arrow_fades.busy(now);
            more |= self.sb_busy;
        }
        if self.content_busy {
            content = true;
        }
        if self.rolled_up && self.roll_anim.is_none() {
            // Rolled: nothing is visible. A glide caught by the roll ends on its destination
            // at once: the fence opens where its content was heading (back to its place after
            // a Peek search, the row a wheel notch asked for), not part of the way there.
            self.content_busy = false;
            if let Some(glide) = self.scroll_anim.take() {
                self.scroll_y = glide.target();
            }
            content = false;
        }
        if content {
            if glide_done {
                self.note_scroll_activity();
                self.request_hover_refresh();
            }
            if self.marquee.is_some() {
                self.update_marquee_selection();
            }
            let before = self.scroll_y;
            let _ = self.redraw_content();
            if self.scroll_y != before {
                // The draw clamped the offset (content shrank mid-glide): drop the glide.
                self.scroll_anim = None;
            }
            more |= self.content_busy;
        }
        if self.chrome_busy && draw_chrome {
            let _ = self.draw_chrome_panel();
            more |= self.chrome_busy;
        }
        more
    }
}

/// What a frame needs of a fence window, so that its animations can go on while a popup menu
/// keeps the App borrowed (see `App::track_menu`).
pub struct FrameHandle {
    id: FenceId,
    hwnd: HWND,
    view: Rc<RefCell<Option<FenceViewState>>>,
}

impl FenceWindow {
    pub fn frame_handle(&self) -> FrameHandle {
        FrameHandle {
            id: self.id,
            hwnd: self.hwnd(),
            view: self.view.clone(),
        }
    }

    /// One compositor frame, see [`FrameHandle::on_frame`].
    pub fn on_frame(&self, now: Instant) -> bool {
        self.frame_handle().on_frame(now)
    }
}

impl FrameHandle {
    fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// One step of a dropped window's glide to its landing place (`FenceWindow::glide_to`).
    /// The window moves with no borrow held (WM_MOVE re-enters the handler). When it has
    /// arrived, the app hears the bounds once more: what waits for a fence at rest (the loose
    /// icons of 像桌面一样排列, the push pass) is laid out for where it is now. Returns whether
    /// the window moved in this frame and whether the glide goes on.
    fn tick_glide(&self, now: Instant) -> (bool, bool) {
        let hwnd = self.hwnd();
        let Some((to, done)) = self.view.borrow_mut().as_mut().and_then(|v| {
            let g = v.glide.as_mut()?;
            // The glide's clock starts with its first frame: what the caller did after asking
            // for it (an import saves and repaints first) does not eat the motion.
            if !std::mem::replace(&mut g.begun, true) {
                g.t = v
                    .motion
                    .tween(0.0, 1.0, motion::FAST, Curve::Decelerate, now);
            }
            Some(g.at(now))
        }) else {
            return (false, false);
        };
        let r = window::window_rect(hwnd);
        let moved = (r.left, r.top) != to;
        if moved {
            let _ = window::move_window_to(hwnd, to.0, to.1);
        }
        if done {
            let handles = self.view.borrow_mut().as_mut().and_then(|v| {
                let notify = v.glide.take().is_some_and(|g| g.notify);
                notify.then(|| (v.queue.clone(), v.behavior.clone()))
            });
            if let Some((queue, behavior)) = handles {
                queue_bounds_changed(&queue, &behavior, self.id, hwnd);
            }
        }
        (moved, !done)
    }

    /// One compositor frame: advances the roll / auto-height animation and every other client
    /// tween. Returns true while something still moves (the app then asks the frame clock for
    /// the next frame). Re-entrancy: the window is resized with no borrow held.
    pub fn on_frame(&self, now: Instant) -> bool {
        let hwnd = self.hwnd();
        let (glided, gliding) = self.tick_glide(now);
        let moved = flush_window_drag(&self.view, hwnd, now, None) | glided;
        // A title drag moved the window on mouse moves since the last frame: the backdrop
        // catches up once for where it is now.
        if let Some(v) = self.view.borrow_mut().as_mut()
            && v.backdrop_stale
        {
            v.refresh_backdrop();
        }
        // Phase 1: compute under a short borrow.
        let (height, more, upward) = {
            let mut guard = self.view.borrow_mut();
            let Some(v) = guard.as_mut() else {
                return false;
            };
            let upward = v.anim_upward;
            // A collapse to a left tab column (or an expand from it) also moves the width.
            let width = v
                .roll_anim
                .as_ref()
                .and_then(|a| a.width)
                .map(|t| t.value_at(now).round() as i32);
            let height = v
                .roll_anim
                .as_ref()
                .map(|a| (a.height, Some(a.rolled_at_end)))
                .or_else(|| v.height_anim.map(|t| (t, None)))
                .map(|(t, rolled)| (t.value_at(now).round() as i32, t.is_done(now), rolled));
            // If the size changes, WM_SIZE will render the new chrome. Drawing hover
            // feedback here would first publish the same frame at the previous size.
            let draw_chrome = !moved
                && height.is_none_or(|(h, _, _)| {
                    let (cur_w, cur_h) = window::window_rect_size(hwnd);
                    h == cur_h && width.is_none_or(|w| w == cur_w)
                });
            let more = v.tick(now, draw_chrome) || gliding;
            (
                height.map(|(h, done, rolled)| (width, h, done, rolled)),
                more,
                upward,
            )
        };
        // Phase 2: window geometry with no borrow held (WM_SIZE re-enters the handler; while
        // an animation runs it only redraws the chrome and moves the shadow).
        let Some((w, h, done, rolled_at_end)) = height else {
            return more;
        };
        let r = window::window_rect(hwnd);
        let w = w.unwrap_or(r.right - r.left);
        let top = top_for_height(upward, &r, h);
        // Inside the app's frame a size that makes the window smaller waits for the push pass:
        // the window, its shadow and the fences it pushes then move in one batch, and the
        // animation lands after it. (A refresh that fell between the resize and the moves
        // showed a bare strip of wallpaper between the shortened plate and a fence below
        // that had not followed yet.) A window that grows is resized at once, as before: its
        // plate is filled for the new size in WM_SIZE, a few ms after the resize, and the
        // fences it pushes move after that, out from under it. Moved in one batch with the
        // resize, they stood a step away from the plate's old edge until the fill came.
        let behavior = self.view.borrow().as_ref().map(|v| v.behavior.clone());
        let pos = window::WindowPos {
            hwnd,
            x: r.left,
            y: top,
            size: Some((w, h)),
        };
        if w <= r.right - r.left
            && h <= r.bottom - r.top
            && let Some(behavior) = behavior
            && behavior.defer_position(pos)
        {
            if done {
                behavior
                    .frame_landings
                    .borrow_mut()
                    .push((hwnd, rolled_at_end));
            }
            return more || !done;
        }
        let _ = window::set_window_bounds(hwnd, r.left, top, w, h);
        if !done {
            return true;
        }
        self.land(rolled_at_end);
        more
    }

    /// Before the frame's batch of window positions is applied. For a window the batch only
    /// moves, the shadow window comes back: it moves in the batch. A window the batch resizes
    /// (it gets smaller) needs a new shadow bitmap, which no batch carries, and Windows spends
    /// about 5 ms between two resizes of windows (measured on a PC with a GPU): its shadow
    /// shrinks now, just before the batch, behind the plate that is still at its old size.
    /// A rolled-up fence left its old shadow outline standing on the wallpaper for that time.
    pub fn shadow_before_batch(&self, resized: bool) -> Option<HWND> {
        let mut guard = self.view.borrow_mut();
        let v = guard.as_mut()?;
        if resized {
            v.update_shadow();
            return None;
        }
        Some(v.shadow.hwnd())
    }

    /// The end of a height animation, once the window has its final size.
    pub fn land(&self, rolled_at_end: Option<bool>) {
        let hwnd = self.hwnd();
        // Phase 3: final geometry — lay the frozen content out once and rasterise everything.
        let (queue, behavior) = {
            let mut guard = self.view.borrow_mut();
            let Some(v) = guard.as_mut() else {
                return;
            };
            v.roll_anim = None;
            v.height_anim = None;
            if let Some(rolled) = rolled_at_end {
                v.rolled_up = rolled;
                v.content_panel.visual.set_opacity(1.0);
                v.content_panel.visual.set_visible(!rolled);
                // Rolled up after a hover peek raised it: back among the other fences.
                if rolled && std::mem::take(&mut v.hover_raised) {
                    v.queue.push(Command::EndHoverRaise(hwnd));
                }
                // Nothing of the content shows from here on: a selection nobody can see is
                // dropped, so no later key or command acts on it.
                if rolled {
                    let had = v.selected.len();
                    let _ = v.end_selection();
                    v.focus_visible = false;
                    v.type_ahead.clear();
                    if had > 0 {
                        tracing::debug!(had, "roll-up landed: the hidden selection is dropped");
                    }
                }
                // The title row it shrank to is no longer under the pointer (rolled up from
                // its menu, or with the pointer over the content): nothing to wait for.
                if rolled && v.peek_blocked && !v.pointer_in_rect() {
                    v.lift_peek_block("the rolled fence is not under the pointer");
                }
            }
            // The resting window gets a crop cut for its final rect (one crop + upload per
            // animation, in the pass that redraws everything anyway).
            v.update_backdrop();
            let (w, h) = window::window_rect_size(hwnd);
            if let Err(e) = v.layout_panels(w, h).and_then(|_| {
                // An icon size step made the fence shorter after its place was chosen: the
                // selected icon was left below the new lower edge.
                if std::mem::take(&mut v.reveal_after_height) && !v.rolled_up {
                    v.reveal_selection();
                }
                v.redraw()
            }) {
                tracing::error!(error = %e, "redraw after height animation failed");
            }
            v.update_shadow_now();
            // Rolled up with 向上展开 switched off meanwhile: the row stays, and from now on it
            // is the top of the fence again (the bounds recorded below hang the rect below it).
            if rolled_at_end == Some(true) && !v.expand_upward {
                v.row_bottom = false;
            }
            v.anim_upward = v.row_bottom;
            (v.queue.clone(), v.behavior.clone())
        };
        queue_bounds_changed(&queue, &behavior, self.id, hwnd);
    }
}
