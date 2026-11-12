//! Roll-up / expand height tween, auto-height settle, hover-peek close, whole-window show / hide fades and shadow coordination.

use super::*;

/// The rolled state the chrome shows: the end state of a roll in flight, else the resting one.
pub(super) fn roll_target_of(rolling_to: Option<bool>, rolled_up: bool) -> bool {
    rolling_to.unwrap_or(rolled_up)
}

/// Top of window rect `r` once it is `h` tall: the top edge stays, or the bottom edge when the
/// title row is at the bottom (a 向上展开 fence that opened upward).
pub(super) fn top_for_height(row_bottom: bool, r: &RECT, h: i32) -> i32 {
    if row_bottom { r.bottom - h } else { r.top }
}

/// Roll-up / expand: the window height itself moves, which only the app can animate (an
/// HWND has no compositor properties), so this is a client tween ticked by the frame clock.
/// While it runs nothing is re-rasterised but the chrome (the content surface is frozen at its
/// expanded size and simply revealed / concealed by the shrinking HWND while its opacity
/// cross-fades on the compositor).
pub(super) struct RollAnim {
    pub(super) height: Tween,
    /// A fence with a left tab column collapses to that column: its width moves too.
    pub(super) width: Option<Tween>,
    pub(super) rolled_at_end: bool,
}

/// Whole-window fade (root visual opacity, plus a 0.97 scale for entrances / exits). The
/// completion arrives as `WM_APP_FADE_DONE` with the generation, so a fade that was reversed
/// or restarted in the meantime is ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VisFade {
    /// Fading in; the shadow is shown at once and fades with the plate.
    Showing,
    /// Fading out; the window is hidden when the fade completes (`destroy` = the app then
    /// drops the window).
    Hiding { destroy: bool },
}

/// What happened during the current whole-window fade, for the debug line at its end: how long
/// it took, what ended it, how evenly the frame clock stepped the shadow along, and how often
/// the anchor moved the window in the z-order meanwhile.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct FadeTrace {
    pub(super) started: Option<Instant>,
    /// Shadow alpha steps taken by the frame clock, and the longest wait between two.
    pub(super) shadow_steps: u32,
    pub(super) max_step_gap_ms: u32,
    pub(super) last_step: Option<Instant>,
    /// Z-order placements of the window by the anchor.
    pub(super) zorder_moves: u32,
}

impl FadeTrace {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            started: Some(now),
            ..Self::default()
        }
    }

    pub(super) fn shadow_step(&mut self, now: Instant) {
        if self.started.is_none() {
            return;
        }
        self.shadow_steps += 1;
        if let Some(prev) = self.last_step.replace(now) {
            let gap = now.duration_since(prev).as_millis() as u32;
            self.max_step_gap_ms = self.max_step_gap_ms.max(gap);
        }
    }

    /// Milliseconds since the fade started.
    pub(super) fn elapsed_ms(&self) -> u64 {
        self.started.map_or(0, |t| t.elapsed().as_millis() as u64)
    }
}

/// Shows a fence window with a 167 ms fade (`entrance` adds the 0.97 → 1 scale of a new
/// window); the shadow is shown at once at alpha 0 and fades with the plate. Already-visible
/// windows are left alone. Borrow-free around `ShowWindow`, which re-enters the handler with
/// WM_SHOWWINDOW.
pub(super) fn show_with_fade(view: &ViewCell, hwnd: HWND, entrance: bool) {
    let visible = desktop::is_visible(hwnd);
    let animate = {
        let mut guard = view.borrow_mut();
        match guard.as_mut() {
            // A retired window is on its way out: nothing may bring it back. One of another
            // virtual desktop waits for a switch there.
            Some(v) if v.retired || v.off_desktop => return,
            // Visible and not mid fade-out: nothing to do.
            Some(v) if visible && v.vis_fade.is_none() => return,
            Some(v) => v.prepare_fade_in(entrance),
            None => false,
        }
    };
    if !visible {
        desktop::show_no_activate(hwnd);
    }
    if !animate {
        return;
    }
    let shadow = {
        let mut guard = view.borrow_mut();
        let Some(v) = guard.as_mut() else {
            return;
        };
        // A refused batch rests the window (and the shadow's alpha) at once; otherwise the
        // shadow is still at alpha 0 and fades in under the frame clock.
        v.start_fade_in(entrance);
        v.shadow.hwnd()
    };
    show_shadow(shadow, hwnd);
}

/// Hides a fence window with a 167 ms fade; `exit` adds the 0.97 scale of a window going
/// away for good, `destroy` makes the completion post `Command::FadeOutDone` so the app can
/// drop the window. Returns false when the window was hidden at once (animations off, or not
/// visible) — the caller may then destroy it immediately. While the fade runs the shadow stays
/// visible and fades with the plate (its HWND is hidden on completion); on a plain hide it is
/// hidden here, with no borrow held: `ShowWindow` on it may re-enter the message loop.
pub(super) fn hide_with_fade(view: &ViewCell, hwnd: HWND, exit: bool, destroy: bool) -> bool {
    if !desktop::is_visible(hwnd) {
        return false;
    }
    let (animated, shadow) = {
        let mut guard = view.borrow_mut();
        match guard.as_mut() {
            Some(v) => (v.begin_fade_out(exit, destroy), Some(v.shadow.hwnd())),
            None => (false, None),
        }
    };
    if !animated {
        if let Some(shadow) = shadow {
            desktop::hide_window(shadow);
        }
        desktop::hide_window(hwnd);
    }
    animated
}

/// The shortest time between two shadow bitmaps while an edge is dragged, from what the last
/// one cost: six times that (the shadow then takes at most a sixth of the UI thread), never
/// under one frame and never over `SHADOW_MIN_INTERVAL_MS`. A fence of ordinary size gets a
/// new shadow with every frame, so it stays on the plate's edge; a very large one keeps the
/// old 40 ms.
pub(super) fn shadow_interval_ms(cost: Duration) -> u128 {
    (cost.as_micros() * 6 / 1000).clamp(SHADOW_FRAME_MS, SHADOW_MIN_INTERVAL_MS)
}

/// Shows a fence's shadow window just below it. Called with no view borrow held.
pub(super) fn show_shadow(shadow: HWND, fence: HWND) {
    desktop::show_no_activate(shadow);
    let _ = desktop::insert_after(shadow, fence);
}

impl FenceViewState {
    pub(super) fn update_shadow(&mut self) {
        if pecofence_core::brand::var_os("PECOFENCE_NO_SHADOW").is_some() {
            self.shadow.hide();
            return;
        }
        let r = self.plate_rect();
        if self.shadow.body_size() == Some((r.right - r.left, r.bottom - r.top)) {
            // Move only: cheap reposition, never throttled (a lagging shadow reads as a ghost).
            self.shadow.update(r, self.dpi);
            return;
        }
        if self.height_animating() {
            // Frame-clock driven resize: one upload per frame keeps the shadow on the plate.
            self.update_shadow_now();
            return;
        }
        let now = Instant::now();
        let min_interval = shadow_interval_ms(self.shadow_cost);
        if let Some(t) = self.shadow_uploaded {
            let since = now.duration_since(t).as_millis();
            if since < min_interval {
                // Too soon after the previous upload: let the body render at full rate and
                // refresh the shadow with a trailing timer, due when the interval since that
                // upload is over. A timer already waiting is left alone: restarted by every
                // size step, it let the shadow fall two intervals behind an edge that kept
                // moving.
                if !self.shadow_timer {
                    self.shadow_timer = true;
                    window::set_timer(
                        self.hwnd,
                        TIMER_SHADOW,
                        (min_interval - since).max(1) as u32,
                    );
                }
                return;
            }
        }
        self.update_shadow_now();
    }

    pub(super) fn update_shadow_now(&mut self) {
        window::kill_timer(self.hwnd, TIMER_SHADOW);
        self.shadow_timer = false;
        let r = self.plate_rect();
        let resized = self.shadow.body_size() != Some((r.right - r.left, r.bottom - r.top));
        let started = Instant::now();
        self.shadow.update(r, self.dpi);
        let now = Instant::now();
        if resized {
            // What a new bitmap for this size costs sets how often an edge drag may ask for one.
            self.shadow_cost = now.duration_since(started);
        }
        self.shadow_uploaded = Some(now);
    }

    /// Screen rect of the visible plate: the window, minus a title row that title on hover
    /// has folded away (the shadow casts from this).
    pub(super) fn plate_rect(&self) -> RECT {
        let mut r = window::window_rect(self.hwnd);
        r.top = (r.top + self.plate_inset_px).min(r.bottom - 1);
        r
    }

    /// After a chrome draw moved the plate's top edge or changed its alpha: the root clip and
    /// the shadow follow (per frame while the fold or the 全透明 fade runs).
    pub(super) fn sync_plate_shape(&mut self, inset_px: i32, alpha: f32) {
        if inset_px != self.plate_inset_px {
            self.plate_inset_px = inset_px;
            let (w, h) = self.chrome_panel.size_px();
            if let Err(error) = self.update_shape_clip(w, h) {
                tracing::warn!(%error, "plate clip update failed");
            }
            self.update_shadow_now();
        }
        if (alpha - self.plate_alpha).abs() > f32::EPSILON {
            self.plate_alpha = alpha;
            self.apply_shadow_alpha(Instant::now());
        }
    }

    /// The shadow's constant alpha: the whole-window fade times the plate's own alpha, so a
    /// fully transparent fence casts no shadow until its plate shows.
    pub(super) fn apply_shadow_alpha(&mut self, now: Instant) {
        let a = self.shadow_alpha.value_at(now).clamp(0.0, 1.0) * self.plate_alpha;
        self.shadow.set_alpha((a * 255.0).round() as u8);
    }

    /// The window height is being animated (roll-up / expand or an auto-height settle): the
    /// content surface is frozen and only the chrome and the shadow follow the bottom edge.
    pub(super) fn height_animating(&self) -> bool {
        self.roll_anim.is_some() || self.height_anim.is_some()
    }

    /// How far the fence reaches beyond its window rect `rest` once it opens, to the right and
    /// below (see `FenceWindow::open_hang_px`).
    pub(super) fn open_hang_px(&self, rest: &RECT) -> (i32, i32) {
        if !(self.roll_target() || self.peeking) {
            return (0, 0);
        }
        let right = if self.column_collapsed {
            (rest.left + self.expanded_w_px - rest.right).max(0)
        } else {
            0
        };
        let below = if self.expand_upward || self.row_bottom {
            0
        } else {
            (rest.top + self.expanded_h_px - rest.bottom).max(0)
        };
        (right, below)
    }

    pub(super) fn rolled_at_rest(&self) -> bool {
        self.rolled_up && !self.peeking && !self.height_animating() && !self.retired
    }

    /// The rolled state the chrome should show: the end state of a roll in flight, else the
    /// resting one. Chevron, item count and tab-strip layout follow this so they animate with
    /// the collapse rather than flipping when it lands (`rolled_up` itself is committed at the
    /// end so persistence and hit-testing see the settled state).
    pub(super) fn roll_target(&self) -> bool {
        roll_target_of(
            self.roll_anim.as_ref().map(|a| a.rolled_at_end),
            self.rolled_up,
        )
    }

    /// Step 1 of a fade-in, before `ShowWindow`: the root is made transparent (and shrunk to
    /// 0.97 for an entrance) so the first composed frame is invisible. Returns false when
    /// animations are off — the caller then shows the window plainly.
    pub(super) fn prepare_fade_in(&mut self, entrance: bool) -> bool {
        window::kill_timer(self.hwnd, TIMER_VISIBILITY_FINISH);
        self.vis_deadline = None;
        self.peek_faded = false;
        if !self.motion.enabled() {
            self.vis_fade = None;
            self.reset_root();
            return false;
        }
        self.vis_gen += 1;
        let now = Instant::now();
        tracing::debug!(
            title = %self.title,
            generation = self.vis_gen,
            replaces = ?self.vis_fade,
            dpi = self.dpi,
            "fade in started"
        );
        self.fade_trace = FadeTrace::new(now);
        // Reversing a fade-out continues from its current opacity instead of restarting at 0.
        if self.vis_fade.is_none() {
            self.root.set_opacity(0.0);
            if entrance {
                self.motion.set_scale(&self.root, ENTRANCE_SCALE);
            }
            self.shadow.set_alpha(0);
            self.shadow_alpha = Tween::at(0.0, now);
        }
        // The shadow fades with the plate: same 167 ms linear, continuous when reversing.
        self.shadow_alpha
            .retarget(1.0, motion::FAST, Curve::Linear, now);
        self.shadow_fading = true;
        self.frames.request();
        self.vis_fade = Some(VisFade::Showing);
        true
    }

    /// Step 2 of a fade-in, after `ShowWindow`: Fluent Direct Entrance — opacity 0 → 1 over
    /// 167 ms linear, scale 0.97 → 1 over 250 ms decelerate; the shadow's constant alpha
    /// follows the opacity on the frame clock. Returns false when the compositor refused the
    /// batch: the window (and its shadow) is then already at rest (opaque, unscaled).
    pub(super) fn start_fade_in(&mut self, entrance: bool) -> bool {
        let root = self.root.clone();
        let motion = self.motion.clone();
        let done = self.poster(WM_APP_FADE_DONE, self.vis_gen);
        let started = motion.batch(
            || {
                motion.fade_to(&root, 1.0, motion::FAST, Curve::Linear)?;
                if entrance {
                    motion.scale_to(&root, 1.0, motion::NORMAL, Curve::Decelerate)?;
                }
                Ok(())
            },
            done,
        );
        if let Err(e) = started {
            // No completion will ever arrive: finish synchronously or the fence stays invisible.
            tracing::warn!(error = %e, "fade-in batch failed; showing plainly");
            self.vis_fade = None;
            self.reset_root();
            return false;
        }
        self.vis_deadline =
            Some(Instant::now() + Duration::from_millis(COMPOSITION_FINISH_MS as u64));
        window::set_timer(self.hwnd, TIMER_VISIBILITY_FINISH, COMPOSITION_FINISH_MS);
        true
    }

    /// Fluent Direct Exit: opacity → 0 over 167 ms linear (Fluent fades are linear; the
    /// Direct Exit curve applies to the scale → 0.97 over 167 ms decelerate for an `exit`,
    /// i.e. a fence being deleted or merged away). The shadow's alpha fades alongside on the
    /// frame clock, so no frame shows a plate without its shadow or a shadow without its plate.
    /// The window is hidden when the fade completes (`WM_APP_FADE_DONE`). Returns false when
    /// animations are off or the compositor refused the batch: the caller hides plainly.
    pub(super) fn begin_fade_out(&mut self, exit: bool, destroy: bool) -> bool {
        window::kill_timer(self.hwnd, TIMER_VISIBILITY_FINISH);
        self.vis_deadline = None;
        self.peek_faded = false;
        self.hide_tip();
        // A retired window must always end in `FadeOutDone`, whoever hides it (quick hide,
        // Peek) while its exit fade is still running.
        let destroy = destroy || self.retired;
        if !self.motion.enabled() {
            self.vis_fade = None;
            return false;
        }
        self.vis_gen += 1;
        tracing::debug!(
            title = %self.title,
            generation = self.vis_gen,
            replaces = ?self.vis_fade,
            dpi = self.dpi,
            "fade out started"
        );
        self.fade_trace = FadeTrace::new(Instant::now());
        self.vis_fade = Some(VisFade::Hiding { destroy });
        let root = self.root.clone();
        let motion = self.motion.clone();
        let done = self.poster(WM_APP_FADE_DONE, self.vis_gen);
        let started = motion.batch(
            || {
                motion.fade_to(&root, 0.0, motion::FAST, Curve::Linear)?;
                if exit {
                    motion.scale_to(&root, ENTRANCE_SCALE, motion::FAST, Curve::Decelerate)?;
                }
                Ok(())
            },
            done,
        );
        if let Err(e) = started {
            tracing::warn!(error = %e, "fade-out batch failed; hiding plainly");
            self.vis_fade = None;
            return false;
        }
        let now = Instant::now();
        self.shadow_alpha
            .retarget(0.0, motion::FAST, Curve::Linear, now);
        self.shadow_fading = true;
        self.frames.request();
        self.vis_deadline = Some(now + Duration::from_millis(COMPOSITION_FINISH_MS as u64));
        window::set_timer(self.hwnd, TIMER_VISIBILITY_FINISH, COMPOSITION_FINISH_MS);
        true
    }

    /// Resting visual state of a shown window (after a fade or a plain show): opaque,
    /// unscaled, shadow at full alpha.
    pub(super) fn reset_root(&mut self) {
        window::kill_timer(self.hwnd, TIMER_VISIBILITY_FINISH);
        self.vis_deadline = None;
        self.peek_faded = false;
        let _ = self.motion.stop(&self.root, Prop::Opacity);
        let _ = self.motion.stop(&self.root, Prop::Scale);
        self.root.set_opacity(1.0);
        self.motion.set_scale(&self.root, 1.0);
        self.shadow_alpha = Tween::at(1.0, Instant::now());
        self.shadow_fading = false;
        self.apply_shadow_alpha(Instant::now());
    }

    /// Resting visual state of a hidden window: transparent, unscaled, shadow alpha 0. A fade-in
    /// shows the window before its opacity 0 reaches the compositor (that happens when the
    /// message loop runs again), so a window kept opaque while hidden flashed fully opaque for a
    /// frame and only then faded in; the same opacity 1, set as a fade-out ended, could also
    /// reach the screen before the window was gone. A plain show makes it opaque in
    /// `WM_SHOWWINDOW`. With animations off nothing fades: it rests opaque as before.
    pub(super) fn rest_hidden(&mut self) {
        if !self.motion.enabled() {
            self.reset_root();
            return;
        }
        window::kill_timer(self.hwnd, TIMER_VISIBILITY_FINISH);
        self.vis_deadline = None;
        self.peek_faded = false;
        let _ = self.motion.stop(&self.root, Prop::Opacity);
        let _ = self.motion.stop(&self.root, Prop::Scale);
        self.root.set_opacity(0.0);
        self.motion.set_scale(&self.root, 1.0);
        self.shadow_alpha = Tween::at(0.0, Instant::now());
        self.shadow_fading = false;
        self.apply_shadow_alpha(Instant::now());
    }

    /// Whether the Peek may fade this window: animations on, shown, and no show / hide fade
    /// owns the root.
    fn peek_fade_allowed(&self) -> bool {
        self.motion.enabled()
            && self.vis_fade.is_none()
            && !self.retired
            && !self.off_desktop
            && desktop::is_visible(self.hwnd)
    }

    /// Peek, before the fence comes over the window that hides it: transparent, its shadow
    /// too. The fence is raised only once this has reached the screen (a raise in the same
    /// turn showed it opaque for a frame), then `peek_fade_to(1.0)` fades it in. False: the
    /// Peek shows it at once, as it does with animations off.
    pub(super) fn peek_fade_prepare(&mut self) -> bool {
        if !self.peek_fade_allowed() {
            tracing::debug!(
                title = %self.title,
                animations = self.motion.enabled(),
                fade = ?self.vis_fade,
                "peek fade: fence left as it is"
            );
            return false;
        }
        let now = Instant::now();
        let _ = self.motion.stop(&self.root, Prop::Opacity);
        self.root.set_opacity(0.0);
        self.shadow_alpha = Tween::at(0.0, now);
        self.shadow_fading = false;
        self.apply_shadow_alpha(now);
        self.peek_faded = true;
        tracing::debug!(
            title = %self.title,
            "peek fade: fence made transparent before its raise"
        );
        true
    }

    /// Fades a fence the Peek took (`peek_fade_prepare`, or `take` for one that is opaque
    /// now and fades out as the Peek ends) to `to`, with the dimmer: 83 ms linear, no scale,
    /// continuing from wherever a fade in flight is; the shadow's alpha follows on the frame
    /// clock. Nothing waits for its end: the app calls `peek_fade_rest` when the Peek is over.
    pub(super) fn peek_fade_to(&mut self, to: f32, take: bool) -> bool {
        if take && !self.peek_faded {
            if !self.peek_fade_allowed() {
                return false;
            }
            self.peek_faded = true;
        }
        if !self.peek_faded {
            return false;
        }
        if let Err(e) = self
            .motion
            .fade_to(&self.root, to, motion::FASTER, Curve::Linear)
        {
            tracing::warn!(error = %e, "peek fade failed; opacity set at once");
            let _ = self.motion.stop(&self.root, Prop::Opacity);
            self.root.set_opacity(to);
        }
        let now = Instant::now();
        self.shadow_alpha
            .retarget(to, motion::FASTER, Curve::Linear, now);
        self.shadow_fading = true;
        self.frames.request();
        tracing::debug!(title = %self.title, to, "peek fade started");
        true
    }

    /// The Peek is over (or its fade was cancelled): opaque with its shadow again, unless a
    /// show / hide fade took the root meanwhile or the window was hidden, which rest it
    /// themselves.
    pub(super) fn peek_fade_rest(&mut self) {
        if !std::mem::take(&mut self.peek_faded) {
            return;
        }
        if self.vis_fade.is_some() || !desktop::is_visible(self.hwnd) {
            tracing::debug!(
                title = %self.title,
                fade = ?self.vis_fade,
                "peek fade: rest left to the window's own fade"
            );
            return;
        }
        self.reset_root();
        tracing::debug!(title = %self.title, "peek fade: fence opaque again");
    }

    /// Advances the shadow's fade for one frame (see `shadow_alpha`); true while it still moves.
    pub(super) fn tick_shadow_alpha(&mut self, now: Instant) -> bool {
        if !self.shadow_fading {
            return false;
        }
        self.apply_shadow_alpha(now);
        self.fade_trace.shadow_step(now);
        if self.shadow_alpha.is_done(now) {
            self.shadow_fading = false;
            return false;
        }
        true
    }

    /// Title on hover for this fence: its own choice, else the global setting. A fence whose
    /// tabs are on the left has no title row to fold away.
    pub(super) fn title_on_hover(&self) -> bool {
        !self.tabs_on_rail()
            && self
                .title_on_hover_override
                .unwrap_or_else(|| self.behavior.title_on_hover.get())
    }

    /// Whether the title row is shown right now (title-on-hover rule): also while another
    /// fence is dragged over it to merge and while the title is being renamed.
    pub(super) fn title_visible_target(&self) -> bool {
        !self.title_on_hover()
            || self.mouse_inside
            || self.roll_target()
            || self.merge_hint
            || self.title_renaming
    }

    /// 外观 → 全透明: no plate (glass, tint, rim, shadow) at rest.
    pub(super) fn is_clear(&self) -> bool {
        self.opacity <= 0.0 || self.bare()
    }

    /// Whether floating above other windows (Peek) draws this fence differently: a fully
    /// transparent plate shows while it floats (`plate_visible_target`), Liquid Glass floats
    /// at full opacity (`material_opacity`), and a Fluent fence in light mode that took light
    /// text from its plate shows dark text again (`material_foreground`). A fence arranged
    /// like the desktop looks the same either way.
    pub(super) fn floating_changes_look(&self) -> bool {
        !self.bare()
            && (self.opacity <= 0.0
                || (self.theme.liquid_glass && self.opacity < 1.0)
                || self.peek_changes_text())
    }

    /// Whether the plate is shown right now: always, except on a fully transparent fence,
    /// whose plate appears while the pointer is over it, while it is rolled up, floats above
    /// other windows (Peek) or is selected with a desktop marquee (the ring is part of the
    /// shadow), and while something is dropped on it or merged into it.
    pub(super) fn plate_visible_target(&self) -> bool {
        // 像桌面一样排列: never a plate, in Peek too; hover, selection and drops show the
        // items' own fills, as on the desktop.
        if self.bare() {
            return false;
        }
        !self.is_clear()
            || self.mouse_inside
            || self.shadow.ring().is_some()
            || self.roll_target()
            || self.behavior.floating.get()
            || self.merge_hint
            || self.drop_hover
            || self.drop_item.is_some()
            || self.drop_insert.is_some()
            || self.drop_tab.is_some()
            || self.title_renaming
    }

    /// End width (device px) of a collapse to (or expand from) a left tab column in flight.
    pub(super) fn width_target_px(&self) -> Option<i32> {
        self.roll_anim
            .as_ref()
            .and_then(|a| a.width)
            .map(|w| w.target().round() as i32)
    }

    /// End height (device px) of the roll / auto-height tween in flight.
    pub(super) fn height_target_px(&self) -> Option<i32> {
        self.roll_anim
            .as_ref()
            .map(|a| a.height.target())
            .or_else(|| self.height_anim.map(|t| t.target()))
            .map(|h| h.round() as i32)
    }

    /// A popup that kept a hover peek open has closed: the 400 ms close countdown starts
    /// now (a WM_MOUSEMOVE inside the fence cancels it as usual). `SetTimer` is safe under
    /// the borrow.
    pub(super) fn resume_peek_close(&self) {
        if self.peeking {
            window::kill_timer(self.hwnd, TIMER_PEEK_CLOSE);
            window::set_timer(self.hwnd, TIMER_PEEK_CLOSE, PEEK_CLOSE_MS);
        }
    }

    /// The pointer is inside the window's rectangle as it is now. No hit test, so it is safe
    /// under the view borrow (WindowFromPoint would send WM_NCHITTEST to this window).
    pub(super) fn pointer_in_rect(&self) -> bool {
        let pt = window::cursor_pos();
        let r = window::window_rect(self.hwnd);
        pt.x >= r.left && pt.x < r.right && pt.y >= r.top && pt.y < r.bottom
    }

    /// The pointer has left the fence (or the fence opens): a hover peek may start again.
    pub(super) fn lift_peek_block(&mut self, reason: &'static str) {
        if std::mem::take(&mut self.peek_blocked) {
            tracing::debug!(fence = %self.fence_id, reason, "hover peek allowed again");
        }
    }

    pub(super) fn start_roll_anim(&mut self, rolled: bool) {
        self.hide_tip();
        if !rolled {
            self.lift_peek_block("the fence opens");
        }
        let now = Instant::now();
        // Painted x per tab before the target state changes: the pills slide the 64 DIP the
        // item-count reserve takes (or frees) instead of jumping there.
        let prev: Vec<(FenceId, f32)> = self
            .tabs
            .iter()
            .map(|t| t.id)
            .zip(self.tab_draw_xs(now))
            .collect();
        // 向上展开: which way the fence opens is decided as it starts to expand, from the room
        // on either side of its title row, which stays where it is. A roll that takes over a
        // running height animation keeps moving the same edge.
        let side = (!rolled && self.expand_upward && !self.height_animating()).then(|| {
            let want = if self.auto_height {
                self.wanted_height_px()
            } else {
                self.expanded_h_px.max(self.title_h_px() + 40)
            };
            let (above, below) = self.expand_rooms_px();
            crate::layout::clearance::expand_side(want, above, below)
        });
        if let Some((up, _)) = side {
            self.row_bottom = up;
        }
        if !self.height_animating() {
            self.anim_upward = self.row_bottom;
        }
        // The roll owns the height (and the content panel) from here on.
        self.height_anim = None;
        self.finish_content_swap_now();
        let cur = window::window_rect(self.hwnd);
        let (cur_w, cur_h) = (cur.right - cur.left, cur.bottom - cur.top);
        // A fence with a left tab column collapses to the column (its title bar) rather than
        // to a title row: the width moves with the height.
        let to_w = self.tabs_on_rail().then(|| {
            if rolled {
                self.rolled_w_px()
            } else {
                self.expanded_w_px
            }
        });
        self.column_collapsed = rolled && to_w.is_some();
        self.capped_h_px = None;
        let to_h = if rolled {
            self.rolled_h_px()
        } else if let Some((_, h)) = side {
            let h = h.max(self.title_h_px() + 1);
            if self.auto_height {
                self.expanded_h_px = h;
            }
            h
        } else if self.auto_height {
            // Items may have changed while rolled (the remembered height is stale then): expand
            // straight to the fitting height so the settle does not add a second glide.
            let h = self.fitting_height_px();
            self.expanded_h_px = h;
            h
        } else {
            // With pushNeighbors the fences below slide down only to the bottom of the work
            // area: this open stops where they leave room (the content scrolls), and
            // `expanded_h_px` keeps the height the fence opens to once there is room again.
            // A hover peek opens in full, above the fences it cannot push (#197): it is gone
            // again when the pointer leaves.
            let floor = self.title_h_px() + 40;
            let full = self.expanded_h_px.max(floor);
            let h = self
                .kept_room_below_px()
                .filter(|_| !self.peeking)
                .map_or(full, |room| full.min(room.max(floor)));
            if h < full {
                tracing::debug!(
                    fence = %self.fence_id,
                    full,
                    opens_to = h,
                    "the fences below cannot go lower: opening to the room they leave"
                );
                self.capped_h_px = Some(h);
            }
            h
        };
        let mid_roll = self.roll_anim.is_some();
        if rolled {
            // WinUI Expander: the content cross-fades out (83 ms linear) while the height
            // collapses; the surface stays at its expanded size and the shrinking HWND
            // conceals it, so nothing is re-rasterised.
            let _ = self.motion.fade_to(
                &self.content_panel.visual,
                0.0,
                motion::FASTER,
                Curve::Linear,
            );
        } else {
            self.rolled_up = false;
            // Rasterise the content once at its final size; the growing HWND reveals it.
            let w = to_w.unwrap_or_else(|| self.chrome_panel.size_px().0);
            let top = self.content_y;
            let _ = self
                .content_panel
                .resize((w - self.content_x).max(1), (to_h - top).max(1));
            self.content_panel
                .visual
                .set_offset(self.content_x as f32, top as f32, 0.0);
            if !mid_roll {
                self.content_panel.visual.set_opacity(0.0);
            }
            self.content_panel.visual.set_visible(true);
            let _ = self.redraw_content();
            let _ = self.motion.fade_to(
                &self.content_panel.visual,
                1.0,
                motion::FASTER,
                Curve::Linear,
            );
        }
        // Open: WinUI Expander's 333 ms decelerate. Close: 167 ms eased in and out, as Liquid
        // Glass always closed. Expander's collapse curve (1,1,0,1) is almost vertical at
        // mid-time: a 560 px fence lost up to 288 px in one frame and read as a cut (88 px now).
        let (dur, curve) = if rolled {
            (motion::FAST, Curve::Smooth)
        } else {
            (motion::SLOW, Curve::Decelerate)
        };
        let curve = if self.theme.liquid_glass {
            Curve::Smooth
        } else {
            curve
        };
        self.roll_anim = Some(RollAnim {
            height: self
                .motion
                .tween(cur_h as f32, to_h as f32, dur, curve, now),
            width: to_w.map(|w| self.motion.tween(cur_w as f32, w as f32, dur, curve, now)),
            rolled_at_end: rolled,
        });
        // Chevron angle (and anything else reading the roll progress) moves with the height:
        // same duration and curve, continuous when a roll is reversed mid-flight.
        self.roll_t
            .retarget(rolled as u8 as f32, self.anim_dur(dur), curve, now);
        // A left tab column keeps its pills where they are (it has no item count to make room
        // for, and it is hidden while rolled).
        if self.tabs.len() > 1 && self.tab_drag.is_none() && !self.tabs_on_rail() {
            self.sync_tab_slides(&prev, now);
        }
        self.frames.request();
    }

    /// Starts (or retargets) the auto-height settle towards `to_h`: 250 ms point-to-point,
    /// the Fluent value for a container changing size in place. Returns false when the caller
    /// should snap instead (tiny change, animations off, window hidden, or a roll owns the
    /// height). The content is rasterised once at the larger of the two sizes so the moving
    /// bottom edge only ever reveals or conceals finished rows.
    pub(super) fn start_height_anim(&mut self, to_h: i32) -> bool {
        if self.roll_anim.is_some() {
            return false;
        }
        let cur_h = window::window_rect_size(self.hwnd).1;
        let now = Instant::now();
        if (to_h - cur_h).abs() < 2 || !self.motion.enabled() || !desktop::is_visible(self.hwnd) {
            return false;
        }
        match self.height_anim.as_mut() {
            Some(t) => t.retarget(to_h as f32, motion::NORMAL, Curve::PointToPoint, now),
            None => {
                self.anim_upward = self.row_bottom;
                self.height_anim = Some(self.motion.tween(
                    cur_h as f32,
                    to_h as f32,
                    motion::NORMAL,
                    Curve::PointToPoint,
                    now,
                ));
            }
        }
        let (w, ch) = self.content_panel.size_px();
        let target_ch = (to_h - self.content_y).max(1);
        if target_ch > ch {
            let _ = self.content_panel.resize(w, target_ch);
            let _ = self.redraw_content();
        }
        self.frames.request();
        true
    }
}
