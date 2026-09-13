//! Fluent motion: the one place that knows durations, curves and how to animate.
//!
//! Two engines, picked by *where* the animated value lives:
//!
//! 1. **Compositor animations** ([`Motion`]) for properties of a [`Visual`] — `Opacity`,
//!    `Offset`, `Size`, `Scale`, clips. These are `Windows.UI.Composition` key-frame /
//!    implicit animations and run on DWM's compositor thread at display refresh, with zero
//!    work in this process once started. This is how WinUI / Explorer move things. The
//!    `windows-composition` wrapper exposes only part of that API, so this module reaches the
//!    rest through the generated [`crate::comp`] bindings (via the vendored `as_raw()`).
//! 2. **Client tweens** ([`Tween`]) for values that are rasterised *inside* a Direct2D
//!    surface — a hover highlight's alpha, the scroll offset of a list. The app redraws the
//!    surface each frame while a tween runs, ticked by the compositor clock
//!    (`pecofence_platform::frameclock`). Easing math comes from the `bezier_easing` crate
//!    (a port of gre/bezier-easing, the CSS `cubic-bezier()` solver).
//!
//! Both honour the user's "animation effects" switch: with animations off, [`Motion`] sets the
//! final value directly and [`Motion::tween`] returns a finished tween.
//!
//! Constants follow the Windows 11 motion guidance (Fluent "timing and easing" and the WinUI
//! `Common_themeresources` tokens): 83 / 167 / 250 / 333 ms, decelerate `(0,0,0,1)`,
//! accelerate `(1,0,1,1)`, point-to-point `(0.55,0.55,0,1)`, collapse `(1,1,0,1)`.

use crate::comp;
use crate::stack::RenderStack;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};
use windows_composition::Visual;
use windows_core::{HSTRING, Interface, Result};
use windows_numerics::{Vector2, Vector3};
use windows_time::TimeSpan;

/// `ControlFasterAnimationDuration`: fades (hover in/out, tooltip, scrollbar opacity).
pub const FASTER: Duration = Duration::from_millis(83);
/// `ControlFastAnimationDuration`: small moves, exits, collapse, scrollbar expand/contract.
pub const FAST: Duration = Duration::from_millis(167);
/// `ControlNormalAnimationDuration`: medium moves, entrances.
pub const NORMAL: Duration = Duration::from_millis(250);
/// Large entrances / expand (WinUI Expander opens in 333 ms).
pub const SLOW: Duration = Duration::from_millis(333);

/// Fluent easing curves. Each maps to a CSS-style `cubic-bezier(x1, y1, x2, y2)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Curve {
    Linear,
    /// Fast-out slow-in `(0,0,0,1)`: entrances, things arriving under the pointer.
    Decelerate,
    /// Slow-out fast-in `(1,0,1,1)`: exits, dismissals.
    Accelerate,
    /// `(0.55,0.55,0,1)`: a value moving from one resting place to another (scroll, reorder).
    PointToPoint,
    /// `(1,1,0,1)`: WinUI Expander collapse.
    Collapse,
    /// Zero velocity at both ends, for a liquid plate's continuous silhouette change.
    Smooth,
}

impl Curve {
    /// `None` for [`Curve::Linear`].
    pub const fn control_points(self) -> Option<[f32; 4]> {
        match self {
            Curve::Linear => None,
            Curve::Decelerate => Some([0.0, 0.0, 0.0, 1.0]),
            Curve::Accelerate => Some([1.0, 0.0, 1.0, 1.0]),
            Curve::PointToPoint => Some([0.55, 0.55, 0.0, 1.0]),
            Curve::Collapse => Some([1.0, 1.0, 0.0, 1.0]),
            Curve::Smooth => Some([0.42, 0.0, 0.58, 1.0]),
        }
    }

    const ALL: [Curve; 6] = [
        Curve::Linear,
        Curve::Decelerate,
        Curve::Accelerate,
        Curve::PointToPoint,
        Curve::Collapse,
        Curve::Smooth,
    ];
}

/// Eased progress for `t` in `0..=1` (clamped) — the client-side twin of the compositor's
/// `CubicBezierEasingFunction`.
pub fn ease(curve: Curve, t: f32) -> f32 {
    let t = if t.is_nan() { 1.0 } else { t.clamp(0.0, 1.0) };
    match curve.control_points() {
        None => t,
        Some(_) => EASINGS.with(|e| e[curve as usize].sample(t)),
    }
}

thread_local! {
    static EASINGS: Vec<bezier_easing::BezierEasing<f32>> = Curve::ALL
        .iter()
        .map(|c| {
            let [x1, y1, x2, y2] = c.control_points().unwrap_or([0.0, 0.0, 1.0, 1.0]);
            bezier_easing::BezierEasing::new(x1, y1, x2, y2).expect("valid Fluent curve")
        })
        .collect();
}

/// WinUI `ScrollPresenter` rule for a programmatic / wheel scroll: 5 ms per pixel, clamped to
/// 50..=1000 ms.
pub fn scroll_duration(distance_px: f32) -> Duration {
    let ms = (distance_px.abs() * 5.0).clamp(50.0, 1000.0);
    Duration::from_millis(ms as u64)
}

/// Animatable [`Visual`] properties addressed by name in the composition API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prop {
    Opacity,
    Offset,
    Size,
    Scale,
}

impl Prop {
    const fn name(self) -> &'static str {
        match self {
            Prop::Opacity => "Opacity",
            Prop::Offset => "Offset",
            Prop::Size => "Size",
            Prop::Scale => "Scale",
        }
    }
}

/// Compositor-side animation helper bound to the app's compositor.
pub struct Motion {
    raw: comp::Compositor,
    enabled: Cell<bool>,
    easings: RefCell<Vec<(Curve, comp::CompositionEasingFunction)>>,
}

fn timespan(d: Duration) -> TimeSpan {
    // Composition rejects zero durations; 1 ms is indistinguishable from a snap.
    TimeSpan::from_millis((d.as_millis() as i64).max(1))
}

fn raw_visual(v: &Visual) -> Result<comp::Visual> {
    v.as_raw().cast::<comp::Visual>()
}

impl Motion {
    pub fn new(stack: &RenderStack) -> Result<Self> {
        Ok(Self {
            raw: stack.compositor.as_raw().cast::<comp::Compositor>()?,
            enabled: Cell::new(true),
            easings: RefCell::new(Vec::new()),
        })
    }

    /// Mirror of `SPI_GETCLIENTAREAANIMATION`; the app sets it at start-up and on
    /// `WM_SETTINGCHANGE`. When false every animation degenerates to setting the end value.
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.set(enabled);
    }

    pub fn enabled(&self) -> bool {
        self.enabled.get()
    }

    fn easing(&self, curve: Curve) -> Result<comp::CompositionEasingFunction> {
        if let Some((_, e)) = self.easings.borrow().iter().find(|(c, _)| *c == curve) {
            return Ok(e.clone());
        }
        let e: comp::CompositionEasingFunction = match curve.control_points() {
            None => self.raw.CreateLinearEasingFunction()?.cast()?,
            Some([x1, y1, x2, y2]) => self
                .raw
                .CreateCubicBezierEasingFunction(Vector2::new(x1, y1), Vector2::new(x2, y2))?
                .cast()?,
        };
        self.easings.borrow_mut().push((curve, e.clone()));
        Ok(e)
    }

    fn scalar(
        &self,
        to: f32,
        dur: Duration,
        curve: Curve,
    ) -> Result<comp::ScalarKeyFrameAnimation> {
        let a = self.raw.CreateScalarKeyFrameAnimation()?;
        a.InsertKeyFrameWithEasingFunction(1.0, to, &self.easing(curve)?)?;
        a.SetDuration(timespan(dur))?;
        Ok(a)
    }

    fn vector3(
        &self,
        to: Vector3,
        dur: Duration,
        curve: Curve,
    ) -> Result<comp::Vector3KeyFrameAnimation> {
        let a = self.raw.CreateVector3KeyFrameAnimation()?;
        a.InsertKeyFrameWithEasingFunction(1.0, to, &self.easing(curve)?)?;
        a.SetDuration(timespan(dur))?;
        Ok(a)
    }

    fn vector2(
        &self,
        to: Vector2,
        dur: Duration,
        curve: Curve,
    ) -> Result<comp::Vector2KeyFrameAnimation> {
        let a = self.raw.CreateVector2KeyFrameAnimation()?;
        a.InsertKeyFrameWithEasingFunction(1.0, to, &self.easing(curve)?)?;
        a.SetDuration(timespan(dur))?;
        Ok(a)
    }

    /// Animates `Opacity` from its current (possibly mid-animation) value to `to`.
    pub fn fade_to(&self, v: &Visual, to: f32, dur: Duration, curve: Curve) -> Result<()> {
        if !self.enabled.get() {
            v.set_opacity(to);
            return Ok(());
        }
        raw_visual(v)?.StartAnimation(
            &HSTRING::from(Prop::Opacity.name()),
            &self.scalar(to, dur, curve)?,
        )
    }

    /// Animates `Offset` (z stays 0) from its current value to `(x, y)`.
    pub fn move_to(&self, v: &Visual, x: f32, y: f32, dur: Duration, curve: Curve) -> Result<()> {
        if !self.enabled.get() {
            v.set_offset(x, y, 0.0);
            return Ok(());
        }
        raw_visual(v)?.StartAnimation(
            &HSTRING::from(Prop::Offset.name()),
            &self.vector3(Vector3::new(x, y, 0.0), dur, curve)?,
        )
    }

    /// Animates `Size` from its current value to `(w, h)`.
    pub fn resize_to(&self, v: &Visual, w: f32, h: f32, dur: Duration, curve: Curve) -> Result<()> {
        if !self.enabled.get() {
            v.set_size(w, h);
            return Ok(());
        }
        raw_visual(v)?.StartAnimation(
            &HSTRING::from(Prop::Size.name()),
            &self.vector2(Vector2::new(w, h), dur, curve)?,
        )
    }

    /// Animates a uniform `Scale` (about the visual's centre point, which the caller sets).
    pub fn scale_to(&self, v: &Visual, s: f32, dur: Duration, curve: Curve) -> Result<()> {
        if !self.enabled.get() {
            v.set_scale(Vector3::new(s, s, 1.0));
            return Ok(());
        }
        raw_visual(v)?.StartAnimation(
            &HSTRING::from(Prop::Scale.name()),
            &self.vector3(Vector3::new(s, s, 1.0), dur, curve)?,
        )
    }

    /// Stops a running animation on `prop`, leaving the property at its current value.
    pub fn stop(&self, v: &Visual, prop: Prop) -> Result<()> {
        raw_visual(v)?.StopAnimation(&HSTRING::from(prop.name()))
    }

    /// Sets the pivot (in the visual's own units) that [`Motion::scale_to`] scales about.
    pub fn set_center(&self, v: &Visual, x: f32, y: f32) {
        v.set_center_point(Vector3::new(x, y, 0.0));
    }

    /// Sets a uniform scale immediately (the resting value before / after [`Motion::scale_to`]).
    pub fn set_scale(&self, v: &Visual, s: f32) {
        v.set_scale(Vector3::new(s, s, 1.0));
    }

    /// Installs implicit animations: from now on any `set_*` of the listed properties on `v`
    /// animates to the new value (`this.FinalValue`) instead of snapping. This is the WinUI
    /// way to animate layout changes without touching every call site.
    pub fn set_implicit(&self, v: &Visual, specs: &[(Prop, Duration, Curve)]) -> Result<()> {
        let raw = raw_visual(v)?;
        if !self.enabled.get() || specs.is_empty() {
            return raw.SetImplicitAnimations(None::<&comp::ImplicitAnimationCollection>);
        }
        let coll = self.raw.CreateImplicitAnimationCollection()?;
        let expr = HSTRING::from("this.FinalValue");
        for &(prop, dur, curve) in specs {
            let target = HSTRING::from(prop.name());
            let easing = self.easing(curve)?;
            let anim: comp::ICompositionAnimationBase = match prop {
                Prop::Opacity => {
                    let a = self.raw.CreateScalarKeyFrameAnimation()?;
                    a.InsertExpressionKeyFrameWithEasingFunction(1.0, &expr, &easing)?;
                    a.SetDuration(timespan(dur))?;
                    a.SetTarget(&target)?;
                    a.cast()?
                }
                Prop::Size => {
                    let a = self.raw.CreateVector2KeyFrameAnimation()?;
                    a.InsertExpressionKeyFrameWithEasingFunction(1.0, &expr, &easing)?;
                    a.SetDuration(timespan(dur))?;
                    a.SetTarget(&target)?;
                    a.cast()?
                }
                Prop::Offset | Prop::Scale => {
                    let a = self.raw.CreateVector3KeyFrameAnimation()?;
                    a.InsertExpressionKeyFrameWithEasingFunction(1.0, &expr, &easing)?;
                    a.SetDuration(timespan(dur))?;
                    a.SetTarget(&target)?;
                    a.cast()?
                }
            };
            coll.Insert(&target, &anim)?;
        }
        raw.SetImplicitAnimations(&coll)
    }

    /// Clips `v` to its bounds minus the given insets (DIPs of the visual's own space).
    /// Negative insets grow the clip. Use with `Size` animations to reveal / conceal content.
    pub fn clip_insets(
        &self,
        v: &Visual,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    ) -> Result<()> {
        let clip = self
            .raw
            .CreateInsetClipWithInsets(left, top, right, bottom)?;
        raw_visual(v)?.SetClip(&clip)
    }

    /// Animates the bottom inset of an existing inset clip on `v` (installed with
    /// [`Motion::clip_insets`]); used to reveal content top-down.
    pub fn animate_clip_bottom(
        &self,
        v: &Visual,
        to: f32,
        dur: Duration,
        curve: Curve,
    ) -> Result<()> {
        let clip = raw_visual(v)?.Clip()?.cast::<comp::InsetClip>()?;
        if !self.enabled.get() {
            return clip.SetBottomInset(to);
        }
        clip.StartAnimation(&HSTRING::from("BottomInset"), &self.scalar(to, dur, curve)?)
    }

    pub fn clear_clip(&self, v: &Visual) -> Result<()> {
        raw_visual(v)?.SetClip(None::<&comp::CompositionClip>)
    }

    /// Clip the entire fence, including scrolling content. Reuse its geometry on resize.
    /// Coordinates are physical composition pixels, not DIPs.
    pub fn clip_rounded(&self, v: &Visual, width: f32, height: f32, radius: f32) -> Result<()> {
        let visual = raw_visual(v)?;
        let existing = visual
            .Clip()
            .ok()
            .and_then(|c| c.cast::<comp::CompositionGeometricClip>().ok())
            .and_then(|c| c.Geometry().ok())
            .and_then(|g| g.cast::<comp::CompositionRoundedRectangleGeometry>().ok());
        let geometry = match existing {
            Some(g) => g,
            None => {
                let g = self.raw.CreateRoundedRectangleGeometry()?;
                visual.SetClip(&self.raw.CreateGeometricClipWithGeometry(&g)?)?;
                g
            }
        };
        let radius = radius.min(width.min(height) * 0.5).max(0.0);
        geometry.SetSize(Vector2::new(width, height))?;
        geometry.SetCornerRadius(Vector2::new(radius, radius))
    }

    /// Read back the actual clip properties for frame-by-frame geometry diagnostics.
    pub fn rounded_clip_bounds(&self, v: &Visual) -> Result<(Vector2, Vector2)> {
        let geometry = raw_visual(v)?
            .Clip()?
            .cast::<comp::CompositionGeometricClip>()?
            .Geometry()?
            .cast::<comp::CompositionRoundedRectangleGeometry>()?;
        Ok((geometry.Size()?, geometry.CornerRadius()?))
    }

    /// Runs `start` (which should start one or more animations) inside a scoped batch and
    /// calls `done` on the compositor's callback thread once every animation in the batch has
    /// finished. `done` must be thread-safe and cheap — post a message back to the window;
    /// never touch UI state from it. With animations disabled `start` still runs and `done`
    /// is invoked immediately on the calling thread.
    pub fn batch(
        &self,
        start: impl FnOnce() -> Result<()>,
        done: impl Fn() + Send + 'static,
    ) -> Result<()> {
        if !self.enabled.get() {
            start()?;
            done();
            return Ok(());
        }
        let batch = self
            .raw
            .CreateScopedBatch(comp::CompositionBatchTypes::Animation)?;
        // Subscribe before End: a batch may complete as soon as it is ended (no animations, or
        // animations disabled by the system), and `EventRevoker` unsubscribes on drop, so it is
        // forgotten (fire-and-forget) — the handler lives as long as the batch object.
        batch.Completed(move |_, _| done())?.forget();
        let result = start();
        batch.End()?;
        result
    }

    /// A client-side tween that respects the animations switch (zero duration when off).
    pub fn tween(&self, from: f32, to: f32, dur: Duration, curve: Curve, now: Instant) -> Tween {
        let dur = if self.enabled.get() {
            dur
        } else {
            Duration::ZERO
        };
        Tween::new(from, to, dur, curve, now)
    }
}

/// One eased `f32` value moving from `from` to `to` over `dur`. The caller samples it with
/// [`Tween::value_at`] each frame and redraws; when [`Tween::is_done`] it can be dropped.
#[derive(Clone, Copy, Debug)]
pub struct Tween {
    from: f32,
    to: f32,
    start: Instant,
    dur: Duration,
    curve: Curve,
}

impl Tween {
    pub fn new(from: f32, to: f32, dur: Duration, curve: Curve, now: Instant) -> Self {
        Self {
            from,
            to,
            start: now,
            dur,
            curve,
        }
    }

    /// A tween that is already at `value`.
    pub fn at(value: f32, now: Instant) -> Self {
        Self::new(value, value, Duration::ZERO, Curve::Linear, now)
    }

    pub fn target(&self) -> f32 {
        self.to
    }

    fn progress(&self, now: Instant) -> f32 {
        if self.dur.is_zero() {
            return 1.0;
        }
        (now.saturating_duration_since(self.start).as_secs_f32() / self.dur.as_secs_f32()).min(1.0)
    }

    pub fn value_at(&self, now: Instant) -> f32 {
        let p = ease(self.curve, self.progress(now));
        self.from + (self.to - self.from) * p
    }

    pub fn is_done(&self, now: Instant) -> bool {
        self.progress(now) >= 1.0
    }

    /// Redirects the tween towards a new target, starting from wherever it is *now* so the
    /// motion stays continuous (a hover that leaves halfway through its fade-in fades back
    /// from that alpha, not from 1). Asking for the target it already has is always a no-op:
    /// a running tween is not restarted and a finished one stays finished (WinUI implicit
    /// animations do nothing when FinalValue equals the current value).
    pub fn retarget(&mut self, to: f32, dur: Duration, curve: Curve, now: Instant) {
        if (to - self.to).abs() < f32::EPSILON {
            return;
        }
        self.from = self.value_at(now);
        self.to = to;
        self.start = now;
        self.dur = dur;
        self.curve = curve;
    }
}

/// A set of independent `0..=1` fades keyed by *what* is fading (an item index, a tab, a
/// header column, a named chrome control). This is the client-side twin of WinUI's per-state
/// brush transitions: the app keeps its logical `bool` / `Option` state for hit-testing and
/// syncs the fades from it at draw time with [`Fades::set`], then samples [`Fades::value`]
/// while painting. Absent keys rest at 0, so a key that faded out is simply dropped
/// ([`Fades::prune`]). Fade-in and fade-out may use different durations (the app currently
/// runs every state fade at 83 ms both ways); both collapse to a snap when animations are off.
pub struct Fades<K> {
    map: HashMap<K, Tween>,
    dur_in: Duration,
    dur_out: Duration,
    curve: Curve,
}

impl<K: Hash + Eq + Copy> Fades<K> {
    pub fn new(dur_in: Duration, dur_out: Duration, curve: Curve) -> Self {
        Self {
            map: HashMap::new(),
            dur_in,
            dur_out,
            curve,
        }
    }

    /// Drives `key` towards `to` (0 or 1), continuing from wherever it is now. A key that is
    /// absent rests at 0, so driving it to 0 is a no-op; driving it to 1 starts from 0.
    /// `animate` is [`Motion::enabled`]: false snaps to `to`.
    pub fn set(&mut self, animate: bool, key: K, to: f32, now: Instant) {
        match self.map.get_mut(&key) {
            Some(t) => {
                if (t.target() - to).abs() > f32::EPSILON {
                    let dur = if !animate {
                        Duration::ZERO
                    } else if to > t.target() {
                        self.dur_in
                    } else {
                        self.dur_out
                    };
                    t.retarget(to, dur, self.curve, now);
                }
            }
            None => {
                if to > 0.0 {
                    let dur = if animate { self.dur_in } else { Duration::ZERO };
                    self.map
                        .insert(key, Tween::new(0.0, to, dur, self.curve, now));
                }
            }
        }
    }

    /// Places `key` at `value` immediately (no transition), e.g. when the thing it describes
    /// was rebuilt and the old fade would be misleading.
    pub fn snap(&mut self, key: K, value: f32, now: Instant) {
        if value <= 0.0 {
            self.map.remove(&key);
        } else {
            self.map.insert(key, Tween::at(value, now));
        }
    }

    pub fn value(&self, key: &K, now: Instant) -> f32 {
        self.map.get(key).map_or(0.0, |t| t.value_at(now))
    }

    /// True while any fade is still moving (the caller keeps requesting frames).
    pub fn busy(&self, now: Instant) -> bool {
        self.map.values().any(|t| !t.is_done(now))
    }

    /// Drops fades that finished at 0 and any key `keep` rejects (indices past the end of a
    /// rebuilt list). Call once per frame after sampling.
    pub fn prune(&mut self, now: Instant, keep: impl Fn(&K) -> bool) {
        self.map
            .retain(|k, t| keep(k) && !(t.is_done(now) && t.target() <= 0.0));
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liquid_height_changes_smoothly_at_display_frame_intervals() {
        let start = Instant::now();
        let height = Tween::new(72.0, 282.0, SLOW, Curve::Smooth, start);
        assert!(height.value_at(start + Duration::from_millis(16)) < 74.0);
        let mut previous = 72.0;
        for ms in (8..=336).step_by(8) {
            let next = height.value_at(start + Duration::from_millis(ms));
            assert!(next >= previous);
            assert!(next - previous < 10.0, "silhouette jumped at {ms} ms");
            previous = next;
        }
        assert_eq!(previous, 282.0);
    }

    #[test]
    fn curves_hit_endpoints_and_stay_in_range() {
        for c in Curve::ALL {
            assert!((ease(c, 0.0)).abs() < 1e-4, "{c:?} start");
            assert!((ease(c, 1.0) - 1.0).abs() < 1e-4, "{c:?} end");
            let mut prev = 0.0f32;
            for i in 0..=100 {
                let v = ease(c, i as f32 / 100.0);
                assert!(
                    (-1e-4..=1.0 + 1e-4).contains(&v),
                    "{c:?} out of range at {i}: {v}"
                );
                assert!(v + 1e-3 >= prev, "{c:?} not monotonic at {i}");
                prev = v;
            }
        }
        assert_eq!(ease(Curve::Linear, 0.25), 0.25);
        assert_eq!(ease(Curve::Decelerate, f32::NAN), 1.0);
        assert_eq!(ease(Curve::Decelerate, 7.0), 1.0);
    }

    #[test]
    fn decelerate_is_front_loaded_and_accelerate_back_loaded() {
        assert!(ease(Curve::Decelerate, 0.5) > 0.8);
        assert!(ease(Curve::Accelerate, 0.5) < 0.2);
        let p = ease(Curve::PointToPoint, 0.5);
        assert!(p > 0.5 && p < 0.95, "{p}");
    }

    #[test]
    fn scroll_duration_clamps() {
        assert_eq!(scroll_duration(1.0), Duration::from_millis(50));
        assert_eq!(scroll_duration(-40.0), Duration::from_millis(200));
        assert_eq!(scroll_duration(10_000.0), Duration::from_millis(1000));
    }

    #[test]
    fn fades_sync_from_logical_state_and_prune() {
        let t0 = Instant::now();
        let mut f: Fades<usize> = Fades::new(
            Duration::from_millis(100),
            Duration::from_millis(200),
            Curve::Linear,
        );
        // Absent keys rest at 0 and driving them to 0 allocates nothing.
        f.set(true, 3, 0.0, t0);
        assert!(f.map.is_empty());
        assert_eq!(f.value(&3, t0), 0.0);
        // Hover in: 0 -> 1 over dur_in.
        f.set(true, 3, 1.0, t0);
        assert!(f.busy(t0));
        let mid = t0 + Duration::from_millis(50);
        assert!((f.value(&3, mid) - 0.5).abs() < 1e-3);
        // Same target again: no restart.
        f.set(true, 3, 1.0, mid);
        assert!((f.value(&3, mid) - 0.5).abs() < 1e-3);
        // Hover out halfway: continues from 0.5 over dur_out (200 ms).
        f.set(true, 3, 0.0, mid);
        let q = mid + Duration::from_millis(100);
        assert!((f.value(&3, q) - 0.25).abs() < 1e-3);
        let end = mid + Duration::from_millis(200);
        assert!(!f.busy(end + Duration::from_millis(1)));
        // Prune drops finished fade-outs and rejected keys.
        f.set(true, 9, 1.0, end);
        f.prune(end + Duration::from_millis(1), |k| *k < 5);
        assert!(f.map.is_empty());
        // Animations off: snaps.
        f.set(false, 1, 1.0, t0);
        assert_eq!(f.value(&1, t0), 1.0);
        assert!(!f.busy(t0));
        f.snap(1, 0.0, t0);
        assert!(f.map.is_empty());
    }

    #[test]
    fn retarget_to_same_value_when_done_stays_done() {
        let t0 = Instant::now();
        let mut t = Tween::new(0.0, 1.0, Duration::from_millis(100), Curve::Linear, t0);
        let later = t0 + Duration::from_millis(150);
        assert!(t.is_done(later));
        t.retarget(1.0, Duration::from_millis(250), Curve::PointToPoint, later);
        assert!(t.is_done(later), "same-target retarget must not restart");
        assert_eq!(t.value_at(later + Duration::from_millis(1)), 1.0);
        // A real retarget from the done state continues from the finished value.
        t.retarget(0.0, Duration::from_millis(100), Curve::Linear, later);
        assert!(!t.is_done(later));
        assert_eq!(t.value_at(later), 1.0);
        assert!((t.value_at(later + Duration::from_millis(50)) - 0.5).abs() < 1e-3);
    }

    #[test]
    fn tween_progresses_and_retargets_continuously() {
        let t0 = Instant::now();
        let mut tw = Tween::new(0.0, 1.0, Duration::from_millis(100), Curve::Linear, t0);
        assert_eq!(tw.value_at(t0), 0.0);
        let mid = t0 + Duration::from_millis(50);
        assert!((tw.value_at(mid) - 0.5).abs() < 1e-3);
        assert!(!tw.is_done(mid));
        assert!(tw.is_done(t0 + Duration::from_millis(100)));
        assert_eq!(tw.value_at(t0 + Duration::from_secs(9)), 1.0);

        // Reverse halfway: starts from 0.5, not from 1.0.
        tw.retarget(0.0, Duration::from_millis(100), Curve::Linear, mid);
        assert!((tw.value_at(mid) - 0.5).abs() < 1e-3);
        assert!((tw.value_at(mid + Duration::from_millis(50)) - 0.25).abs() < 1e-3);
        assert_eq!(tw.target(), 0.0);

        // Same target while running: no restart.
        let before = tw.value_at(mid + Duration::from_millis(20));
        tw.retarget(
            0.0,
            Duration::from_millis(500),
            Curve::Decelerate,
            mid + Duration::from_millis(20),
        );
        assert!((tw.value_at(mid + Duration::from_millis(20)) - before).abs() < 1e-6);
        assert!(tw.is_done(mid + Duration::from_millis(100)));

        let done = Tween::at(3.0, t0);
        assert!(done.is_done(t0));
        assert_eq!(done.value_at(t0), 3.0);
        assert_eq!(
            Tween::new(0.0, 1.0, Duration::ZERO, Curve::Decelerate, t0).value_at(t0),
            1.0
        );
    }
}
