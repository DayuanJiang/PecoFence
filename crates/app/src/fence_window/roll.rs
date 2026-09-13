//! Roll-up / expand height tween, auto-height settle, hover-peek close, whole-window show / hide fades and shadow coordination.

use super::*;

/// The rolled state the chrome shows: the end state of a roll in flight, else the resting one.
pub(super) fn roll_target_of(rolling_to: Option<bool>, rolled_up: bool) -> bool {
    rolling_to.unwrap_or(rolled_up)
}

/// Roll-up / expand: the window height itself moves, which only the app can animate (an
/// HWND has no compositor properties), so this is a client tween ticked by the frame clock.
/// While it runs nothing is re-rasterised but the chrome (the content surface is frozen at its
/// expanded size and simply revealed / concealed by the shrinking HWND while its opacity
/// cross-fades on the compositor).
pub(super) struct RollAnim {
    pub(super) height: Tween,
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

/// Shows a fence window with a 167 ms fade (`entrance` adds the 0.97 → 1 scale of a new
/// window); the shadow is shown at once at alpha 0 and fades with the plate. Already-visible
/// windows are left alone. Borrow-free around `ShowWindow`, which re-enters the handler with
/// WM_SHOWWINDOW.
pub(super) fn show_with_fade(view: &ViewCell, hwnd: HWND, entrance: bool) {
    let visible = desktop::is_visible(hwnd);
    let animate = {
        let mut guard = view.borrow_mut();
        match guard.as_mut() {
            // A retired window is on its way out: nothing may bring it back.
            Some(v) if v.retired => return,
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
        let r = window::window_rect(self.hwnd);
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
        if let Some(t) = self.shadow_uploaded
            && now.duration_since(t).as_millis() < SHADOW_MIN_INTERVAL_MS
        {
            // Too soon after the previous upload: let the body render at full rate and refresh
            // the shadow with a trailing timer instead.
            window::set_timer(self.hwnd, TIMER_SHADOW, SHADOW_MIN_INTERVAL_MS as u32);
            return;
        }
        self.update_shadow_now();
    }

    pub(super) fn update_shadow_now(&mut self) {
        window::kill_timer(self.hwnd, TIMER_SHADOW);
        let r = window::window_rect(self.hwnd);
        self.shadow.update(r, self.dpi);
        self.shadow_uploaded = Some(Instant::now());
    }

    /// The window height is being animated (roll-up / expand or an auto-height settle): the
    /// content surface is frozen and only the chrome and the shadow follow the bottom edge.
    pub(super) fn height_animating(&self) -> bool {
        self.roll_anim.is_some() || self.height_anim.is_some()
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
        if !self.motion.enabled() {
            self.vis_fade = None;
            self.reset_root();
            return false;
        }
        self.vis_gen += 1;
        let now = Instant::now();
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
        self.hide_tip();
        // A retired window must always end in `FadeOutDone`, whoever hides it (quick hide,
        // Peek) while its exit fade is still running.
        let destroy = destroy || self.retired;
        if !self.motion.enabled() {
            self.vis_fade = None;
            return false;
        }
        self.vis_gen += 1;
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
        let _ = self.motion.stop(&self.root, Prop::Opacity);
        let _ = self.motion.stop(&self.root, Prop::Scale);
        self.root.set_opacity(1.0);
        self.motion.set_scale(&self.root, 1.0);
        self.shadow_alpha = Tween::at(1.0, Instant::now());
        self.shadow_fading = false;
        self.shadow.set_alpha(255);
    }

    /// Advances the shadow's fade for one frame (see `shadow_alpha`); true while it still moves.
    pub(super) fn tick_shadow_alpha(&mut self, now: Instant) -> bool {
        if !self.shadow_fading {
            return false;
        }
        let a = self.shadow_alpha.value_at(now).clamp(0.0, 1.0);
        self.shadow.set_alpha((a * 255.0).round() as u8);
        if self.shadow_alpha.is_done(now) {
            self.shadow_fading = false;
            return false;
        }
        true
    }

    /// Whether the title row is shown right now (title-on-hover rule).
    pub(super) fn title_visible_target(&self) -> bool {
        !self.behavior.title_on_hover.get() || self.mouse_inside || self.roll_target()
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

    pub(super) fn start_roll_anim(&mut self, rolled: bool) {
        self.hide_tip();
        let now = Instant::now();
        // Painted x per tab before the target state changes: the pills slide the 64 DIP the
        // item-count reserve takes (or frees) instead of jumping there.
        let prev: Vec<(FenceId, f32)> = self
            .tabs
            .iter()
            .map(|t| t.id)
            .zip(self.tab_draw_xs(now))
            .collect();
        // The roll owns the height (and the content panel) from here on.
        self.height_anim = None;
        self.finish_content_swap_now();
        let cur_h = window::window_rect(self.hwnd);
        let cur_h = cur_h.bottom - cur_h.top;
        let to_h = if rolled {
            self.title_h_px()
        } else if self.auto_height {
            // Items may have changed while rolled (the remembered height is stale then): expand
            // straight to the fitting height so the settle does not add a second glide.
            let h = self.fitting_height_px();
            self.expanded_h_px = h;
            h
        } else {
            self.expanded_h_px.max(self.title_h_px() + 40)
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
            let (w, _) = self.chrome_panel.size_px();
            let title_h = self.title_h_px();
            let _ = self.content_panel.resize(w, (to_h - title_h).max(1));
            self.content_panel
                .visual
                .set_offset(0.0, title_h as f32, 0.0);
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
        // WinUI Expander: open 333 ms decelerate, close 167 ms with the collapse curve.
        let (dur, curve) = if rolled {
            (motion::FAST, Curve::Collapse)
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
            rolled_at_end: rolled,
        });
        // Chevron angle (and anything else reading the roll progress) moves with the height:
        // same duration and curve, continuous when a roll is reversed mid-flight.
        self.roll_t
            .retarget(rolled as u8 as f32, self.anim_dur(dur), curve, now);
        if self.tabs.len() > 1 && self.tab_drag.is_none() {
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
        let target_ch = (to_h - self.title_h_px()).max(1);
        if target_ch > ch {
            let _ = self.content_panel.resize(w, target_ch);
            let _ = self.redraw_content();
        }
        self.frames.request();
        true
    }
}
