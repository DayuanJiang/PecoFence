//! Scroll offset and glide, overlay scrollbar geometry / hit / draw / press, wheel step, and the per-frame auto-scrolls (marquee edge, OLE drag edge).

use super::*;

/// Wheel distance (DIPs, positive = content moves up) for `notches` notches (fractional for
/// touchpads) under the user's `SPI_GETWHEELSCROLLLINES` setting: `WHEEL_PAGESCROLL`
/// (`u32::MAX`) scrolls a viewport, 0 disables the wheel, otherwise List / Details scroll
/// `lines` rows per notch (Explorer's default 3) while the icon grid scrolls one grid row per
/// notch (comctl32's icon-view behaviour; a grid row is already several list rows tall).
pub(super) fn wheel_step_dip(
    lines: u32,
    notches: f32,
    row_step: f32,
    view_h: f32,
    grid: bool,
) -> f32 {
    if lines == 0 {
        return 0.0;
    }
    let per_notch = if lines == u32::MAX {
        view_h
    } else if grid {
        row_step
    } else {
        row_step * lines as f32
    };
    per_notch * notches
}

/// Duration of an animated scroll over `distance_dip` (WinUI: 5 ms per pixel, 50..=1000 ms),
/// capped so short wheel notches and page jumps stay snappy: 250 ms for wheel notches, 333 ms
/// for keyboard / page jumps, 83 ms for track / arrow auto-repeat ticks, zero (next frame)
/// for precision-touchpad sub-notch deltas.
pub(super) fn scroll_anim_duration(distance_dip: f32, cap: Duration) -> Duration {
    motion::scroll_duration(distance_dip).min(cap)
}

/// Marquee auto-scroll speed (DIP/s) for a pointer `overshoot_dip` outside the item area.
pub(super) fn marquee_scroll_speed(overshoot_dip: f32) -> f32 {
    (overshoot_dip.abs() * MARQUEE_SCROLL_GAIN).clamp(MARQUEE_SCROLL_MIN, MARQUEE_SCROLL_MAX)
}

/// OLE drag auto-scroll: direction (-1 up / +1 down) and depth into the edge zone (0 at the
/// zone's inner boundary, 1 at the band edge) for a client-px pointer inside the item band
/// `top..=bottom`; None outside both zones (a pointer above the band is on the title / header,
/// whose hover has its own meaning).
pub(super) fn drag_scroll_zone(cy: i32, top: i32, bottom: i32, zone_px: i32) -> Option<(i8, f32)> {
    let zone_px = zone_px.max(1);
    if cy < top || cy > bottom {
        return None;
    }
    if cy < top + zone_px {
        let depth = (top + zone_px - cy) as f32 / zone_px as f32;
        return Some((-1, depth.clamp(0.0, 1.0)));
    }
    if cy > bottom - zone_px {
        let depth = (cy - (bottom - zone_px)) as f32 / zone_px as f32;
        return Some((1, depth.clamp(0.0, 1.0)));
    }
    None
}

/// OLE drag auto-scroll speed (DIP/s): ramps from a crawl at the zone boundary to about one
/// row per 50 ms at the very edge (Explorer's cadence), clamped so large icon cells do not fly.
pub(super) fn drag_scroll_speed(depth: f32, row_step: f32) -> f32 {
    let max = (row_step / 0.05).clamp(DRAG_SCROLL_MIN, DRAG_SCROLL_MAX);
    DRAG_SCROLL_MIN + (max - DRAG_SCROLL_MIN) * depth.clamp(0.0, 1.0)
}

/// Overlay scrollbar geometry in content DIPs (WinUI ScrollBar): the bar spans the scrollable
/// band (`band_top`, `band_h`); a 12 DIP arrow button sits at each end and the thumb travels
/// the track between them.
#[derive(Clone, Copy, Debug)]
pub(super) struct ScrollbarGeom {
    pub(super) band_top: f32,
    pub(super) band_h: f32,
    pub(super) track_top: f32,
    pub(super) track_h: f32,
    pub(super) thumb_y: f32,
    pub(super) thumb_h: f32,
    pub(super) max_scroll: f32,
    pub(super) view_h: f32,
    pub(super) content_h: f32,
}

/// Thumb travel inside a scrollable band `view_h` tall below `fixed_top`: the arrow buttons
/// (12 DIP each, always in layout so the thumb never jumps when the bar expands) are excluded.
pub(super) fn scrollbar_track(fixed_top: f32, view_h: f32) -> (f32, f32) {
    let button = pecofence_render::fence_chrome::ScrollbarDraw::BUTTON_H;
    (fixed_top + button, (view_h - 2.0 * button).max(1.0))
}

/// Thumb length for a viewport `view_h` over `content_h` on a `track_h` track.
pub(super) fn scrollbar_thumb_h(track_h: f32, view_h: f32, content_h: f32) -> f32 {
    (track_h * view_h / content_h)
        .max(SCROLLBAR_THUMB_MIN)
        .min(track_h)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum ScrollHit {
    Thumb,
    TrackAbove,
    TrackBelow,
    /// The RepeatButton above the track (one row per tick).
    ArrowUp,
    ArrowDown,
}

/// Offset change one press / repeat tick of `hit` makes: a viewport on the track (SB_PAGEUP /
/// SB_PAGEDOWN), one row on the arrows (SB_LINEUP / SB_LINEDOWN); 0 for the thumb.
pub(super) fn scroll_step_for(hit: ScrollHit, view_h: f32, row_step: f32) -> f32 {
    match hit {
        ScrollHit::Thumb => 0.0,
        ScrollHit::TrackAbove => -view_h,
        ScrollHit::TrackBelow => view_h,
        ScrollHit::ArrowUp => -row_step,
        ScrollHit::ArrowDown => row_step,
    }
}

/// Thumb being dragged.
pub(super) struct ScrollDrag {
    pub(super) start_y_px: i32,
    pub(super) start_scroll: f32,
}

/// Edge auto-scroll while an OLE drag (files from outside or our own items) hovers near the
/// top / bottom of the item area. Ticked by the frame clock, not by `DragOver` (OLE's ~50 ms
/// cadence would stutter).
#[derive(Clone, Copy, Debug)]
pub(super) struct DragScroll {
    /// -1 scrolls up, +1 down.
    pub(super) dir: i8,
    /// Depth into the edge zone, 0..=1 (drives the speed).
    pub(super) depth: f32,
    /// When the pointer entered this zone (the scroll starts after `DRAG_SCROLL_ARM`).
    pub(super) armed_at: Instant,
    /// Previous frame the scroll advanced.
    pub(super) last: Instant,
    /// Last screen point from DragEnter / DragOver: the drop highlight is re-resolved there as
    /// the content moves under a stationary pointer.
    pub(super) pt: DragPoint,
}

/// Rounds a DIP offset to a whole device pixel at `scale` (mirrors `Px::snap`): the content
/// surface then rasterises cells on pixel boundaries, so icons and 1 px rings stay crisp.
pub(super) fn snap_offset(v: f32, scale: f32) -> f32 {
    (v * scale).round() / scale
}

impl FenceViewState {
    /// Scrollable layout and viewport height (DIPs) of the content panel. While the fence is
    /// rolled up the panel is 1 px tall, so the viewport is the remembered expanded height
    /// instead: the offset the user left stays within the real range and the fence unrolls
    /// exactly where it was (a collapsed container has no viewport of its own).
    pub(super) fn scroll_extent(&self) -> (ItemLayout, f32) {
        let scale = self.scale();
        let (cw, ch) = self.content_size_px();
        let layout = self.layout(cw as f32 / scale);
        let ch = if self.rolled_up && self.roll_anim.is_none() {
            let title_h = self.title_h_px();
            self.expanded_h_px.max(title_h + 40) - title_h
        } else {
            ch
        };
        let view_h = (ch as f32 / scale - layout.fixed_top()).max(1.0);
        (layout, view_h)
    }

    pub(super) fn scroll_max(&self) -> f32 {
        let (layout, view_h) = self.scroll_extent();
        layout.max_scroll(view_h)
    }

    /// Where the scroll offset is heading: the glide's destination while one runs, else the
    /// offset itself. Input that adds to the scroll adds to this, so notches accumulate.
    pub(super) fn scroll_target(&self) -> f32 {
        self.scroll_anim.map_or(self.scroll_y, |t| t.target())
    }

    /// Animated scroll to `target` (clamped to the content): WinUI ScrollPresenter's glide,
    /// 5 ms per DIP within 50..=1000 ms then capped at `cap`, decelerate (0,0,0,1). A running
    /// glide is retargeted from its current value so the motion stays continuous. A zero `cap`
    /// means "apply on the next frame, no glide" (precision-touchpad deltas tracking 1:1); the
    /// frame clock still coalesces a 120 Hz stream into one redraw per compositor frame. With
    /// animations off the offset snaps at once and `scroll_anim` stays None (callers redraw).
    pub(super) fn scroll_to(&mut self, target: f32, cap: Duration) {
        if self.rolled_up && self.roll_anim.is_none() {
            // Nothing to scroll: the content is hidden and must reappear where it was.
            return;
        }
        // Every glide ends on a whole device pixel (WinUI ScrollPresenter pixel-snaps at
        // rest), so icons and hairlines are crisp once the motion stops.
        let target = snap_offset(target, self.scale()).clamp(0.0, self.scroll_max());
        if (target - self.scroll_target()).abs() > 0.01 {
            self.dismiss_rename_for_scroll();
        }
        let now = Instant::now();
        let dist = (target - self.scroll_y).abs();
        // Every scroll input counts as activity, including one pinned at an end: WinUI shows
        // the indicator on any wheel notch, so a hidden bar reappears even at the boundary.
        self.note_scroll_activity();
        if !self.motion.enabled() {
            self.scroll_anim = None;
            if dist > 0.0 {
                self.scroll_y = target;
                self.request_hover_refresh();
            }
            return;
        }
        if self.scroll_anim.is_none() && dist < 0.01 {
            return;
        }
        let dur = scroll_anim_duration(dist, cap);
        match self.scroll_anim.as_mut() {
            Some(t) if !t.is_done(now) => t.retarget(target, dur, Curve::Decelerate, now),
            _ => {
                self.scroll_anim = Some(Tween::new(
                    self.scroll_y,
                    target,
                    dur,
                    Curve::Decelerate,
                    now,
                ));
            }
        }
        self.frames.request();
    }

    /// The user scrolled the content while the inline rename edit was open: commit it
    /// (comctl32 ListView dismisses label edit on scroll) instead of leaving the popup
    /// floating over the wrong row. Taking `renaming` here keeps a second notch from posting
    /// again; the popup itself closes from the command loop, outside this borrow.
    pub(super) fn dismiss_rename_for_scroll(&mut self) {
        if self.renaming.take().is_some() {
            self.queue.push(Command::EndItemRename { commit: true });
        }
    }

    /// Rounds the offset to a whole device pixel and clamps it (WinUI ScrollPresenter rests on
    /// pixel boundaries). Returns whether it changed.
    pub(super) fn snap_scroll(&mut self) -> bool {
        let old = self.scroll_y;
        self.scroll_y = snap_offset(old, self.scale()).clamp(0.0, self.scroll_max());
        self.scroll_y != old
    }

    /// Scrolls so item `index` is fully visible (relative to where the offset is heading):
    /// animated for keyboard navigation, immediate when the caller needs final geometry now.
    pub(super) fn scroll_into_view(&mut self, index: usize, animate: bool) {
        let (layout, view_h) = self.scroll_extent();
        let cell = layout.cell(index);
        // The first item of a "按时间分组" section brings its header along.
        let top = layout.group_header_of(index).map_or(cell.y, |h| h.y);
        let base = self.scroll_target();
        let target = if top < base {
            top - layout.top_pad()
        } else if cell.y + cell.h > base + view_h {
            cell.y + cell.h + layout.top_pad() - view_h
        } else {
            return;
        };
        if animate {
            self.scroll_to(target, motion::SLOW);
        } else {
            self.scroll_anim = None;
            self.scroll_y = target.clamp(0.0, layout.max_scroll(view_h));
            self.note_scroll_activity();
        }
    }

    /// Scroll / scrollbar activity: restarts the inactive-scrollbar countdown (2 s, WinUI
    /// ScrollView) and fades the indicator in if it was hidden. `SetTimer` neither re-enters
    /// the window procedure nor touches geometry, so `tick` may call this.
    pub(super) fn note_scroll_activity(&mut self) {
        self.last_scroll = Some(Instant::now());
        if self.behavior.hide_inactive_scrollbar.get() {
            window::set_timer(self.hwnd, TIMER_SCROLLBAR, SCROLLBAR_LINGER_MS + 50);
        }
        self.sync_scrollbar_alpha();
    }

    /// Overlay scrollbar geometry (None when nothing overflows / rolled).
    pub(super) fn scrollbar_geometry(&self) -> Option<ScrollbarGeom> {
        self.scrollbar_geometry_for(self.scroll_y)
    }

    /// Geometry with the thumb placed for `scroll_y` (the glide's destination, for the
    /// track-repeat hit test).
    pub(super) fn scrollbar_geometry_for(&self, scroll_y: f32) -> Option<ScrollbarGeom> {
        if self.rolled_up {
            return None;
        }
        let (layout, view_h) = self.scroll_extent();
        let max_scroll = layout.max_scroll(view_h);
        if max_scroll <= 0.0 {
            return None;
        }
        let band_top = layout.fixed_top();
        let (track_top, track_h) = scrollbar_track(band_top, view_h);
        let content_h = layout.content_height();
        let thumb_h = scrollbar_thumb_h(track_h, view_h, content_h);
        let thumb_y =
            track_top + (track_h - thumb_h) * (scroll_y.clamp(0.0, max_scroll) / max_scroll);
        Some(ScrollbarGeom {
            band_top,
            band_h: view_h,
            track_top,
            track_h,
            thumb_y,
            thumb_h,
            max_scroll,
            view_h,
            content_h,
        })
    }

    /// Scrollbar part under a client-pixel point (the 12 DIP strip along the right edge).
    pub(super) fn scrollbar_hit(&self, x_px: i32, y_px: i32) -> Option<(ScrollHit, ScrollbarGeom)> {
        self.scrollbar_hit_for(x_px, y_px, self.scroll_y)
    }

    pub(super) fn scrollbar_hit_for(
        &self,
        x_px: i32,
        y_px: i32,
        scroll_y: f32,
    ) -> Option<(ScrollHit, ScrollbarGeom)> {
        let title_h = self.title_h_px();
        if y_px < title_h {
            return None;
        }
        let g = self.scrollbar_geometry_for(scroll_y)?;
        let scale = self.scale();
        let (cw, _) = self.content_size_px();
        let x = x_px as f32 / scale;
        let y = (y_px - title_h) as f32 / scale;
        if x < cw as f32 / scale - SCROLLBAR_HOT_DIP || y < g.band_top || y > g.band_top + g.band_h
        {
            return None;
        }
        // The arrow buttons are hit even while the bar rests thin (WinUI keeps them in the
        // tree at opacity 0); the press itself expands the bar at once.
        let hit = if y < g.track_top {
            ScrollHit::ArrowUp
        } else if y > g.track_top + g.track_h {
            ScrollHit::ArrowDown
        } else if y < g.thumb_y {
            ScrollHit::TrackAbove
        } else if y > g.thumb_y + g.thumb_h {
            ScrollHit::TrackBelow
        } else {
            ScrollHit::Thumb
        };
        Some((hit, g))
    }

    /// The bar to draw: width, alpha, the track / arrow reveal and the arrow hover fades
    /// sampled from their tweens (None once faded out).
    pub(super) fn scrollbar_draw(&self) -> Option<ScrollbarDraw> {
        let g = self.scrollbar_geometry()?;
        let now = Instant::now();
        let alpha = self.sb_alpha.value_at(now);
        if alpha <= 0.0 {
            return None;
        }
        // Win32 button semantics: the pressed fill shows only while the pointer is on the
        // held button (the repeat pauses off it too).
        let pressed = |h: ScrollHit| self.page_repeat == Some(h) && self.sb_pointer == Some(h);
        Some(ScrollbarDraw {
            thumb_y: g.thumb_y,
            thumb_h: g.thumb_h,
            width: self.sb_width.value_at(now),
            alpha,
            parts: self.sb_parts.value_at(now),
            band_top: g.band_top,
            band_h: g.band_h,
            arrow_hover: [
                self.sb_arrow_fades.value(&ScrollHit::ArrowUp, now),
                self.sb_arrow_fades.value(&ScrollHit::ArrowDown, now),
            ],
            arrow_pressed: [pressed(ScrollHit::ArrowUp), pressed(ScrollHit::ArrowDown)],
        })
    }

    /// Pointer over scrollbar part `hit` (None = off the bar): keeps the hot zone (expand /
    /// contract delays) and the arrow buttons' PointerOver fades (83 ms) current. Returns
    /// whether the pointer is on the bar.
    pub(super) fn set_scrollbar_pointer(&mut self, hit: Option<ScrollHit>) -> bool {
        self.set_scrollbar_zone(hit.is_some());
        let now = Instant::now();
        let animate = self.motion.enabled();
        for arrow in [ScrollHit::ArrowUp, ScrollHit::ArrowDown] {
            // Hover only while nothing else on the bar is held (a thumb drag passing over a
            // button must not light it).
            let on = hit == Some(arrow)
                && self.scroll_drag.is_none()
                && self.page_repeat.is_none_or(|h| h == arrow);
            self.sb_arrow_fades
                .set(animate, arrow, on as u8 as f32, now);
        }
        if std::mem::replace(&mut self.sb_pointer, hit) != hit
            && (self.page_repeat.is_some() || self.sb_arrow_fades.busy(now))
        {
            self.start_scrollbar_anim();
        }
        hit.is_some()
    }

    /// Thumb held (drag) or track held (page repeat): WinUI Pressed = expanded, no delays.
    pub(super) fn scrollbar_pressed(&self) -> bool {
        self.scroll_drag.is_some() || self.page_repeat.is_some()
    }

    pub(super) fn start_scrollbar_anim(&mut self) {
        self.sb_busy = true;
        self.frames.request();
    }

    /// Fades the indicator towards what `scrollbar_visible` says (83 ms linear, WinUI
    /// ScrollBarOpacityChangeDuration). Cheap when nothing changes: called at draw time too.
    pub(super) fn sync_scrollbar_alpha(&mut self) {
        let want = if self.scrollbar_visible() { 1.0 } else { 0.0 };
        if (self.sb_alpha.target() - want).abs() < f32::EPSILON {
            return;
        }
        let dur = self.anim_dur(motion::FASTER);
        self.sb_alpha
            .retarget(want, dur, Curve::Linear, Instant::now());
        self.start_scrollbar_anim();
    }

    /// Widens / narrows the thumb (167 ms decelerate, WinUI ScrollBarExpand/ContractDuration);
    /// the track fill and arrow buttons fade in / out alongside (83 ms linear,
    /// ScrollBarOpacityChangeDuration).
    pub(super) fn scrollbar_width_to(&mut self, width: f32) {
        if (self.sb_width.target() - width).abs() < f32::EPSILON {
            return;
        }
        let now = Instant::now();
        let dur = self.anim_dur(motion::FAST);
        self.sb_width.retarget(width, dur, Curve::Decelerate, now);
        let parts = if width >= ScrollbarDraw::EXPANDED_WIDTH {
            1.0
        } else {
            0.0
        };
        self.sb_parts
            .retarget(parts, self.anim_dur(motion::FASTER), Curve::Linear, now);
        self.start_scrollbar_anim();
    }

    /// Pointer entered / left the 12 DIP hot zone: arms the WinUI begin delays (0.4 s before
    /// expanding, 0.5 s before contracting); re-entering within the contract delay cancels it
    /// so the bar stays wide. Safe while the bar is pressed: a press keeps it wide, so
    /// leaving the zone mid-drag only records the fact (`end_scrollbar_press` uses it). With
    /// animations off the width changes at once: the delays belong to the XAML expand /
    /// contract storyboards, which are disabled along with the motion.
    pub(super) fn set_scrollbar_zone(&mut self, inside: bool) {
        if self.scrollbar_hot == inside {
            return;
        }
        self.scrollbar_hot = inside;
        let immediate = !self.motion.enabled();
        if inside {
            window::kill_timer(self.hwnd, TIMER_SCROLLBAR_CONTRACT);
            if self.sb_width.target() < ScrollbarDraw::EXPANDED_WIDTH
                && self.scrollbar_geometry().is_some()
            {
                if immediate {
                    self.scrollbar_width_to(ScrollbarDraw::EXPANDED_WIDTH);
                } else {
                    window::set_timer(self.hwnd, TIMER_SCROLLBAR_EXPAND, SCROLLBAR_EXPAND_BEGIN_MS);
                }
            }
        } else {
            window::kill_timer(self.hwnd, TIMER_SCROLLBAR_EXPAND);
            if !self.scrollbar_pressed() && self.sb_width.target() > ScrollbarDraw::REST_WIDTH {
                if immediate {
                    self.scrollbar_width_to(ScrollbarDraw::REST_WIDTH);
                } else {
                    window::set_timer(
                        self.hwnd,
                        TIMER_SCROLLBAR_CONTRACT,
                        SCROLLBAR_CONTRACT_BEGIN_MS,
                    );
                }
            }
        }
        self.sync_scrollbar_alpha();
    }

    /// Thumb or track pressed: expands at once (a press must not wait out the hover delay).
    pub(super) fn scrollbar_press(&mut self) {
        window::kill_timer(self.hwnd, TIMER_SCROLLBAR_EXPAND);
        window::kill_timer(self.hwnd, TIMER_SCROLLBAR_CONTRACT);
        self.scrollbar_width_to(ScrollbarDraw::EXPANDED_WIDTH);
        self.note_scroll_activity();
    }

    /// Press on the overlay scrollbar (WM_LBUTTONDOWN, or the second click of a rapid pair
    /// arriving as WM_LBUTTONDBLCLK): starts a thumb drag, or pages once on the track and
    /// arms the auto-repeat. Returns None when the point is off the bar; otherwise whether the
    /// caller — with no borrow held — must start the repeat timer besides taking capture.
    pub(super) fn press_scrollbar(&mut self, x_px: i32, y_px: i32) -> Option<bool> {
        let (hit, g) = self.scrollbar_hit(x_px, y_px)?;
        let repeat = match hit {
            ScrollHit::Thumb => {
                // A glide in flight would fight the thumb: stop it.
                self.scroll_anim = None;
                self.snap_scroll();
                self.scroll_drag = Some(ScrollDrag {
                    start_y_px: y_px,
                    start_scroll: self.scroll_y,
                });
                false
            }
            // Track: one animated page now, then auto-repeat while held (TIMER_SCROLL_REPEAT)
            // until the thumb reaches the pointer. Arrow buttons: one row, repeating while
            // held (RepeatButton, 50 ms interval).
            ScrollHit::TrackAbove
            | ScrollHit::TrackBelow
            | ScrollHit::ArrowUp
            | ScrollHit::ArrowDown => {
                let step = scroll_step_for(hit, g.view_h, self.scroll_extent().0.row_step());
                self.scroll_to(self.scroll_target() + step, motion::SLOW);
                self.page_repeat = Some(hit);
                self.set_scrollbar_pointer(Some(hit));
                true
            }
        };
        self.scrollbar_hot = true;
        self.scrollbar_press();
        let _ = self.redraw_content();
        Some(repeat)
    }

    /// Ends a thumb drag / track repeat (release, capture loss, Escape): the contract delay
    /// starts now unless the pointer is still on the bar, and the hide countdown restarts.
    /// Returns whether one was running.
    pub(super) fn end_scrollbar_press(&mut self) -> bool {
        let had = self.scroll_drag.take().is_some() | self.page_repeat.take().is_some();
        if had {
            self.snap_scroll();
            window::kill_timer(self.hwnd, TIMER_SCROLL_REPEAT);
            if !self.scrollbar_hot {
                window::set_timer(
                    self.hwnd,
                    TIMER_SCROLLBAR_CONTRACT,
                    SCROLLBAR_CONTRACT_BEGIN_MS,
                );
            }
            self.note_scroll_activity();
        }
        had
    }

    /// Edge auto-scroll bookkeeping for a drag hovering at screen point `pt` (see
    /// `DragScroll`): enters / leaves / re-arms the zone; the scrolling itself runs in `tick`.
    pub(super) fn update_drag_scroll(&mut self, pt: DragPoint) {
        let r = window::window_rect(self.hwnd);
        let cy = pt.y - r.top;
        let (top, bottom) = self.marquee_band_px();
        let zone_px = (DRAG_SCROLL_ZONE_DIP * self.scale()) as i32;
        let zone = drag_scroll_zone(cy, top, bottom, zone_px).filter(|_| self.scroll_max() > 0.0);
        let Some((dir, depth)) = zone else {
            if self.drag_scroll.take().is_some() && self.snap_scroll() {
                // Auto-scroll ended between pixels: `content_busy` makes the next tick redraw
                // the content at the snapped offset (nothing else flags the content then).
                self.content_busy = true;
                self.frames.request();
            }
            return;
        };
        match self.drag_scroll.as_mut() {
            Some(ds) if ds.dir == dir => {
                ds.depth = depth;
                ds.pt = pt;
            }
            _ => {
                let now = Instant::now();
                self.drag_scroll = Some(DragScroll {
                    dir,
                    depth,
                    armed_at: now,
                    last: now,
                    pt,
                });
            }
        }
        self.frames.request();
    }

    /// One frame of OLE drag edge auto-scroll. Returns true while it keeps scrolling; pinned
    /// at an end it stops ticking (the next DragOver re-requests a frame).
    pub(super) fn drag_autoscroll(&mut self, now: Instant) -> bool {
        let Some(ds) = self.drag_scroll else {
            return false;
        };
        if now.saturating_duration_since(ds.armed_at) < DRAG_SCROLL_ARM {
            if let Some(d) = self.drag_scroll.as_mut() {
                d.last = now;
            }
            // Still arming: nothing moved, so no redraw — only keep the clock ticking.
            self.frames.request();
            return false;
        }
        let dt = now
            .saturating_duration_since(ds.last)
            .as_secs_f32()
            .min(0.05);
        if let Some(d) = self.drag_scroll.as_mut() {
            d.last = now;
        }
        let (layout, view_h) = self.scroll_extent();
        let speed = drag_scroll_speed(ds.depth, layout.row_step());
        let old = self.scroll_y;
        self.scroll_anim = None;
        self.scroll_y = (old + ds.dir as f32 * speed * dt).clamp(0.0, layout.max_scroll(view_h));
        if self.scroll_y == old {
            return false;
        }
        // The content moved under a (possibly stationary) pointer: the folder highlight /
        // insertion caret follow the newly exposed rows. The frame's redraw shows it.
        let spot = self.drop_spot_at(ds.pt.x, ds.pt.y, FenceDropHandler::internal().as_ref());
        let (_, chrome_changed) = self.set_drop_feedback(Some(spot));
        if chrome_changed {
            let _ = self.redraw_chrome_only();
        }
        self.note_scroll_activity();
        true
    }

    /// One frame of marquee auto-scroll: a velocity proportional to how far the pointer is
    /// outside the item area, applied per frame so band and items glide. Returns true while
    /// it moved (pinned at an end it stops ticking; the next WM_MOUSEMOVE re-arms).
    pub(super) fn marquee_autoscroll(&mut self, now: Instant) -> bool {
        let Some(m) = self.marquee.as_ref() else {
            return false;
        };
        let Some(last) = m.auto_last else {
            return false;
        };
        let (px, py) = m.pointer;
        let (top, bottom) = self.marquee_band_px();
        let overshoot = if py < top {
            py - top
        } else if py > bottom {
            py - bottom
        } else {
            0
        };
        if overshoot == 0 {
            if let Some(m) = self.marquee.as_mut() {
                m.auto_last = None;
            }
            // Auto-scroll came to rest: land on a whole device pixel (one more redraw).
            return self.snap_scroll();
        }
        let dt = now.saturating_duration_since(last).as_secs_f32().min(0.05);
        let speed = marquee_scroll_speed(overshoot as f32 / self.scale());
        let (layout, view_h) = self.scroll_extent();
        let old = self.scroll_y;
        self.scroll_anim = None;
        self.scroll_y =
            (old + overshoot.signum() as f32 * speed * dt).clamp(0.0, layout.max_scroll(view_h));
        if let Some(m) = self.marquee.as_mut() {
            m.auto_last = Some(now);
            m.cur = (px, py.clamp(top, bottom));
        }
        if self.scroll_y == old {
            return false;
        }
        self.note_scroll_activity();
        true
    }

    /// Whether the inactive-scrollbar rule wants the indicator shown: pointer over the fence
    /// or the bar, bar held, or activity within the last 2 s (`sync_scrollbar_alpha` fades
    /// towards this).
    pub(super) fn scrollbar_visible(&self) -> bool {
        if !self.behavior.hide_inactive_scrollbar.get() {
            return true;
        }
        self.mouse_inside
            || self.scrollbar_hot
            || self.scrollbar_pressed()
            || self
                .last_scroll
                .is_some_and(|t| t.elapsed().as_millis() < SCROLLBAR_LINGER_MS as u128)
    }
}
