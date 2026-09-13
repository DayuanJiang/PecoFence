//! Tab strip layout, pill slides / settle, tab drag reorder, merge gap, torn-off (remote) drag cancel and the tab-switch content cross-fade.

use super::*;

pub(super) struct TabDrag {
    pub(super) start: (i32, i32),
    /// The pointer left the system drag threshold: until then the press is a click (the pill
    /// stays in its slot, keeps the pressed caption and lets the SwitchTab pill glide play).
    pub(super) moved: bool,
    /// Current strip position of the dragged tab (moves as it passes its neighbours).
    pub(super) index: usize,
    pub(super) start_index: usize,
    /// Pointer x (DIPs) minus the pressed pill's slot x at button-down: the pill rides under
    /// the pointer at `pointer_x - grab_dx` (TabView / Edge), clamped to the strip.
    pub(super) grab_dx: f32,
    pub(super) pointer_x: f32,
}

/// Where a tab pill is painted: `pointer_x - grab_dx` clamped so the pill stays inside the
/// strip (`first_x` = first slot's x, `strip_end` = last slot's right edge).
pub(super) fn dragged_tab_x(
    pointer_x: f32,
    grab_dx: f32,
    w: f32,
    first_x: f32,
    strip_end: f32,
) -> f32 {
    let max = (strip_end - w).max(first_x);
    (pointer_x - grab_dx).clamp(first_x, max)
}

/// One reorder step for the dragged pill at strip `index` (drawn at `x`, `w` wide): the
/// neighbour it should swap with, or None when it stays. The pill moves right once its centre
/// passes the right neighbour's centre and left once it passes the left neighbour's centre —
/// never "the slot the centre is in", which oscillates when a narrow pill crosses a wide one
/// (after that swap the narrow pill's centre would still lie inside the wide neighbour's new
/// slot). Comparing centres is stable for any pair of widths: after a swap the neighbour's
/// centre lands on the far side of the dragged pill, so the reverse test cannot fire until the
/// pointer really comes back.
pub(super) fn tab_reorder_step(
    index: usize,
    x: f32,
    w: f32,
    slots: &[(f32, f32)],
) -> Option<usize> {
    let centre = |(sx, sw): (f32, f32)| sx + sw / 2.0;
    let c = x + w / 2.0;
    if let Some(&right) = slots.get(index + 1)
        && c >= centre(right)
    {
        return Some(index + 1);
    }
    if index > 0
        && let Some(&left) = slots.get(index - 1)
        && c <= centre(left)
    {
        return Some(index - 1);
    }
    None
}

/// Ordering follows the unclipped pill or the pointer crossing a neighbour. The
/// pointer fallback matters when a wide tab is grabbed near its edge: requiring
/// only its centre to cross can demand a pointer outside the window (a tear-off).
pub(super) fn tab_reorder_at_pointer(
    index: usize,
    pointer_x: f32,
    grab_dx: f32,
    slots: &[(f32, f32)],
) -> Option<usize> {
    let (_, width) = *slots.get(index)?;
    tab_reorder_step(index, pointer_x - grab_dx, width, slots)
        .or_else(|| tab_reorder_step(index, pointer_x - width * 0.5, width, slots))
}

/// What a cancelled drag (Esc / right button) needs done once the borrow is released.
pub(super) enum DragCancel {
    /// A tab drag along the strip: the order was restored in place.
    Tab,
    /// A torn-off window being dragged: the App restores the original group.
    Remote {
        change: Box<pecofence_core::TabDetach>,
        hinted: bool,
    },
    Window {
        hwnd: HWND,
        rect: RECT,
        hinted: bool,
    },
}

pub(super) enum WindowDragOrigin {
    Tab(Box<pecofence_core::TabDetach>),
    Caption { rect: RECT, click_expand: bool },
}

/// The original window keeps capture and moves the detached HWND (which can be itself).
pub(super) struct RemoteDrag {
    pub(super) hwnd: HWND,
    pub(super) fence: FenceId,
    /// Cursor position relative to the dragged window's top-left.
    pub(super) offset: (i32, i32),
    /// Merge target currently highlighted (0 = none).
    pub(super) merge_target: isize,
    /// Pointer screen x last sent with the hint (the target's insertion gap follows it).
    pub(super) merge_x: i32,
    pub(super) moved_once: bool,
    /// Only the ownership and geometry changed by this gesture; used when cancelling.
    pub(super) origin: WindowDragOrigin,
    pub(super) press: (i32, i32),
    pub(super) last_pointer: (i32, i32),
    pub(super) pending_pointer: Option<(i32, i32)>,
    pub(super) started: bool,
    pub(super) requests: u32,
    pub(super) applied: u32,
}

/// WinUI TabView's insertion rule for a tab dragged over a strip: the slot of the pill under
/// `x` (DIPs) when `x` is left of its centre, the next slot when right of it; left of the first
/// pill = 0, past the last (or between pills) = the nearest following slot. `rects` are the
/// strip's (x, w) pills as drawn. When they were laid out around an open insertion gap
/// (`tab_strip_rects` with `gap = Some(g)`), a pointer anywhere in the gap — after pill g-1's
/// centre and before pill g's — keeps slot g, so the gap only moves once the pointer crosses
/// the centre of a real pill: WinUI's hysteresis, and oscillation-free because the pills keep
/// their widths and only the ones between the old and new slot shift.
pub(super) fn merge_slot_for(x: f32, rects: &[(f32, f32)]) -> usize {
    for (i, (tx, tw)) in rects.iter().enumerate() {
        if x < tx + tw / 2.0 {
            return i;
        }
    }
    rects.len()
}

/// Lays the tab strip out along the title row: `natural` are the pills' natural widths (DIPs,
/// already clamped), `available` the room between `TAB_LEFT` and the chevron zone. A phantom
/// pill of `TAB_MERGE_GAP_W` at slot `gap` takes part in the proportional shrink and the
/// spacing but is not emitted, so the result has exactly one (x, w) per entry of `natural`.
pub(super) fn tab_strip_rects(
    natural: &[f32],
    gap: Option<usize>,
    available: f32,
) -> Vec<(f32, f32)> {
    // (natural width, is the phantom gap)
    let mut natural: Vec<(f32, bool)> = natural.iter().map(|n| (*n, false)).collect();
    if let Some(g) = gap {
        natural.insert(g.min(natural.len()), (TAB_MERGE_GAP_W, true));
    }
    let available = available.max(0.0);
    let gap_w = TAB_GAP.min(available / natural.len().max(1) as f32 * 0.2);
    let gaps = gap_w * natural.len().saturating_sub(1) as f32;
    let total: f32 = natural.iter().map(|(n, _)| *n).sum::<f32>() + gaps;
    let factor = if total > available {
        ((available - gaps) / (total - gaps).max(0.001)).max(0.0)
    } else {
        1.0
    };
    let mut x = TAB_LEFT;
    natural
        .iter()
        .filter_map(|(n, phantom)| {
            let w = (n * factor).max(0.0);
            let r = (x, w);
            x += w + gap_w;
            (!phantom).then_some(r)
        })
        .collect()
}

impl FenceViewState {
    /// Tab switch, Fluent Direct Exit / Entrance for a content swap: the old content stays on
    /// its panel and fades out (83 ms linear) while a second panel — which every later redraw
    /// addresses — fades in (83 ms) and slides 24 DIP from the side of the new tab (167 ms
    /// decelerate). `dir` = +1 when the new tab lies to the right of the old one. The panel is
    /// drawn once right away so it never shows stale pixels; `set_items` redraws it next. A
    /// switch that lands while the previous swap is still moving continues from the current
    /// state: the half-faded page keeps fading from its current opacity (its slide finishes
    /// under the fade) and the new page starts as usual — nothing pops to rest first.
    pub(super) fn begin_content_swap(&mut self, dir: i8) {
        if self.outgoing.len() >= 2 {
            // Third switch inside 167 ms: the oldest page is (nearly) transparent, park it so
            // live panels stay bounded at one content + two outgoing.
            let oldest = self.outgoing.remove(0);
            self.park_panel(oldest);
        }
        let mut incoming = match self
            .spares
            .pop()
            .map(Ok)
            .unwrap_or_else(|| Panel::new(&self.stack))
        {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(error = %e, "spare content panel");
                return;
            }
        };
        let (cw, ch) = self.content_panel.size_px();
        if incoming.resize(cw, ch).is_err() {
            return;
        }
        let title_h = self.title_h_px() as f32;
        let dx = dir.signum() as f32 * TAB_SLIDE_DIP * self.scale();
        incoming.visual.set_offset(dx, title_h, 0.0);
        incoming.visual.set_opacity(0.0);
        incoming.visual.set_visible(true);
        self.root.children().insert_at_top(&incoming.visual);
        let outgoing = std::mem::replace(&mut self.content_panel, incoming);
        let (o, n) = (outgoing.visual.clone(), self.content_panel.visual.clone());
        self.outgoing.push(outgoing);
        self.swap_gen += 1;
        self.swap_deadline =
            Some(Instant::now() + Duration::from_millis(COMPOSITION_FINISH_MS as u64));
        window::set_timer(self.hwnd, TIMER_CONTENT_FINISH, COMPOSITION_FINISH_MS);
        let motion = self.motion.clone();
        let done = self.poster(WM_APP_TAB_SWAP_DONE, self.swap_gen);
        let started = motion.batch(
            || {
                motion.fade_to(&o, 0.0, motion::FASTER, Curve::Linear)?;
                motion.fade_to(&n, 1.0, motion::FASTER, Curve::Linear)?;
                motion.move_to(&n, 0.0, title_h, motion::FAST, Curve::Decelerate)
            },
            done,
        );
        if let Err(e) = started {
            // No completion will come: rest the panels now rather than leaving the new content
            // transparent and offset.
            tracing::warn!(error = %e, "tab swap batch failed; snapping");
            self.finish_content_swap_now();
        }
        let _ = self.redraw_content();
    }

    /// Ends the content swaps at once: every outgoing panel is parked as a spare and the new
    /// panel rests at its final place. Called on completion (of the newest swap; older
    /// batches use the same durations, so they have finished by then), and by anything that
    /// re-lays the panels out or rolls the fence while a swap is still moving.
    pub(super) fn finish_content_swap_now(&mut self) {
        window::kill_timer(self.hwnd, TIMER_CONTENT_FINISH);
        self.swap_deadline = None;
        if self.outgoing.is_empty() {
            return;
        }
        for o in std::mem::take(&mut self.outgoing) {
            self.park_panel(o);
        }
        let n = &self.content_panel.visual;
        let _ = self.motion.stop(n, Prop::Opacity);
        let _ = self.motion.stop(n, Prop::Offset);
        n.set_opacity(1.0);
        n.set_offset(0.0, self.title_h_px() as f32, 0.0);
    }

    /// Takes an outgoing panel out of the tree and keeps it (at most two) for the next swap.
    pub(super) fn park_panel(&mut self, o: Panel) {
        let _ = self.motion.stop(&o.visual, Prop::Opacity);
        let _ = self.motion.stop(&o.visual, Prop::Offset);
        self.root.children().remove(&o.visual);
        o.visual.set_opacity(1.0);
        o.visual.set_visible(false);
        if self.spares.len() < 2 {
            self.spares.push(o);
        }
    }

    /// Esc / right button while a tab is dragged along the strip or a torn-off window is being
    /// dragged: puts the tab back (the strip slides to its original order) or hands the torn-off
    /// fence back to the App. The caller releases the capture and pushes the commands with no
    /// borrow held. Returns None when neither drag is running.
    pub(super) fn cancel_tab_or_remote_drag(&mut self) -> Option<DragCancel> {
        if self.tab_drag.is_some() {
            let now = Instant::now();
            let prev: Vec<(FenceId, f32)> = self
                .tabs
                .iter()
                .map(|t| t.id)
                .zip(self.tab_draw_xs(now))
                .collect();
            let td = self.tab_drag.take()?;
            if td.index < self.tabs.len() {
                let t = self.tabs.remove(td.index);
                self.tabs.insert(td.start_index.min(self.tabs.len()), t);
            }
            self.pressed = None;
            self.press_inside = false;
            self.tab_hover = None;
            self.hide_tip();
            // The released pill slides from under the pointer back into its original slot along
            // with the neighbours it had displaced.
            self.sync_tab_slides(&prev, now);
            let _ = self.redraw_chrome_only();
            return Some(DragCancel::Tab);
        }
        if self.detach_pending {
            self.detach_pending = false;
            self.detach_cancelled = true;
            return Some(DragCancel::Tab);
        }
        let rd = self.remote_drag.take()?;
        self.detach_pending = false;
        Some(match rd.origin {
            WindowDragOrigin::Tab(change) => DragCancel::Remote {
                change,
                hinted: rd.merge_target != 0,
            },
            WindowDragOrigin::Caption { rect, .. } => DragCancel::Window {
                hwnd: rd.hwnd,
                rect,
                hinted: rd.merge_target != 0,
            },
        })
    }

    /// Tab header rects (x, w) in DIPs along the title row; empty unless there are 2+ tabs.
    /// Natural width = caption + 16 padding, clamped to [48, 160]; when the strip does not fit
    /// beside the chevron zone every tab shrinks proportionally (captions ellipsize).
    pub(super) fn tab_rects(&self) -> Vec<(f32, f32)> {
        self.tab_rects_with_gap(self.merge_gap)
    }

    /// `tab_rects` with an explicit insertion gap (`tab_strip_rects`): pure arithmetic over the
    /// cached natural widths, so it is cheap enough for every pointer move of a merge drag.
    pub(super) fn tab_rects_with_gap(&self, gap: Option<usize>) -> Vec<(f32, f32)> {
        if self.tabs.len() < 2 {
            return Vec::new();
        }
        let (_, end) = self.tab_strip_bounds();
        let available = (end - TAB_LEFT).max(0.0);
        tab_strip_rects(&self.tab_natural_widths(), gap, available)
    }

    fn tab_strip_bounds(&self) -> (f32, f32) {
        let width = self.chrome_panel.size_px().0 as f32 / self.scale();
        // Rolled: the item count sits before the chevron; the strip must end before it. The
        // target state, so the pills slide there during the roll (`start_roll_anim`).
        let reserve = if self.roll_target() {
            TAB_RIGHT_RESERVE
                + pecofence_render::fence_chrome::header_count_width(
                    width,
                    self.tabs.len(),
                    self.deco.title_x(),
                )
        } else {
            TAB_RIGHT_RESERVE
        };
        (TAB_LEFT, (width - reserve).max(TAB_LEFT))
    }

    /// Natural pill width per tab (caption + 16 padding, clamped to [48, 160] DIPs). Measuring
    /// a caption builds a DirectWrite layout, so widths are cached by caption: a reorder, a
    /// merge hint or a redraw re-uses them and only a new / renamed caption is measured. The
    /// cache also tracks title size and the theme's control padding.
    pub(super) fn tab_natural_widths(&self) -> Vec<f32> {
        let mut cache = self.tab_natural_w.borrow_mut();
        let hit = cache.len() == self.tabs.len()
            && cache
                .iter()
                .zip(&self.tabs)
                .all(|((c, size, glass, _), t)| {
                    *c == t.title && *size == t.title_size && *glass == self.theme.liquid_glass
                });
        if !hit {
            let fresh: Vec<_> = self
                .tabs
                .iter()
                .map(|t| {
                    let w = cache
                        .iter()
                        .find(|(c, size, glass, _)| {
                            *c == t.title
                                && *size == t.title_size
                                && *glass == self.theme.liquid_glass
                        })
                        .map_or_else(
                            || {
                                (pecofence_render::text::measure_width(
                                    &t.title,
                                    self.chrome.tab_format(t.title_size),
                                ) + pecofence_render::fence_chrome::tab_text_padding(&self.theme))
                                .clamp(TAB_MIN_W, TAB_MAX_W)
                            },
                            |(_, _, _, w)| *w,
                        );
                    (t.title.clone(), t.title_size, self.theme.liquid_glass, w)
                })
                .collect();
            *cache = fresh;
        }
        cache.iter().map(|(_, _, _, w)| *w).collect()
    }

    /// The strip slot a fence dragged over this title at screen `x` would be inserted at (WinUI
    /// TabView's GetTabInsertionIndex rule, see `merge_slot_for`); `tabs.len()` = append. A
    /// plain title (fewer than two tabs) always appends. Hit-tests the pills where they are
    /// drawn — around the open gap, shifted and shrunk by it — so the gap sits exactly where
    /// the pointer is over the strip and stays put while the pointer is inside it.
    pub(super) fn merge_slot_at(&self, x_screen: i32) -> usize {
        // Menu/script merges have no pointer position and append. Handle the sentinel
        // before screen→client subtraction; MIN - positive_left overflows in debug.
        if self.tabs.len() < 2 || x_screen == i32::MIN {
            return self.tabs.len();
        }
        let left = window::window_rect(self.hwnd).left;
        let x = x_screen.saturating_sub(left) as f32 / self.scale();
        merge_slot_for(x, &self.tab_rects())
    }

    /// Opens (or closes) the insertion gap at screen `x` for a drag-to-merge hint; the
    /// neighbours slide aside / back over 167 ms point-to-point (`sync_tab_slides`). Returns
    /// true when the strip changed.
    pub(super) fn set_merge_gap(&mut self, x_screen: Option<i32>) -> bool {
        let gap = x_screen
            .filter(|_| self.tabs.len() >= 2)
            .map(|x| self.merge_slot_at(x));
        if gap == self.merge_gap {
            return false;
        }
        let now = Instant::now();
        let prev: Vec<(FenceId, f32)> = self
            .tabs
            .iter()
            .map(|t| t.id)
            .zip(self.tab_draw_xs(now))
            .collect();
        self.merge_gap = gap;
        self.sync_tab_slides(&prev, now);
        true
    }

    /// Tab header under a client-pixel point.
    pub(super) fn tab_at(&self, x_px: i32, y_px: i32) -> Option<usize> {
        if y_px < 0 || y_px >= self.title_h_px() {
            return None;
        }
        let x = x_px as f32 / self.scale();
        let rects = self.tab_rects();
        let xs = self.tab_draw_xs(Instant::now());
        if let Some(drag) = &self.tab_drag
            && drag.moved
            && drag.index < rects.len()
            && x >= xs[drag.index]
            && x < xs[drag.index] + rects[drag.index].1
        {
            return Some(drag.index);
        }
        (0..rects.len())
            .rev()
            .find(|&i| x >= xs[i] && x < xs[i] + rects[i].1)
    }

    /// Where each tab pill is painted this frame (DIPs): its slot, or the tween sliding it into
    /// its slot, or — for the tab being dragged — the pointer minus the grab offset, clamped to
    /// the strip. Hit-testing (`tab_at`) follows these same painted positions.
    pub(super) fn tab_draw_xs(&self, now: Instant) -> Vec<f32> {
        let rects = self.tab_rects();
        // `rects` is empty for a single tab (no strip), while a slide keyed by id could in
        // principle outlive a shrink: index by `get_mut`, never by position.
        let mut xs: Vec<f32> = rects.iter().map(|(x, _)| *x).collect();
        let index_of = |id: FenceId| self.tabs.iter().position(|t| t.id == id);
        for (id, t) in &self.tab_slide {
            if let Some(x) = index_of(*id).and_then(|i| xs.get_mut(i)) {
                *x = t.value_at(now);
            }
        }
        if let Some((id, t)) = &self.tab_settle
            && let Some(x) = index_of(*id).and_then(|i| xs.get_mut(i))
        {
            *x = t.value_at(now);
        }
        if let Some(td) = &self.tab_drag
            && let Some(x) = self.dragged_tab_x(td, &rects)
            && let Some(slot) = xs.get_mut(td.index)
        {
            *slot = x;
        }
        xs
    }

    /// The dragged pill's painted x (None when the strip is gone, or while the press has not
    /// left the drag threshold: the pill sits in its slot like a clicked tab's).
    pub(super) fn dragged_tab_x(&self, td: &TabDrag, rects: &[(f32, f32)]) -> Option<f32> {
        if !td.moved {
            return None;
        }
        let (_, w) = *rects.get(td.index)?;
        let (first, end) = self.tab_strip_bounds();
        Some(dragged_tab_x(td.pointer_x, td.grab_dx, w, first, end))
    }

    /// After the strip order changed: every tab whose painted x (`prev`, by id) differs from
    /// its new slot slides there over 167 ms point-to-point (TabView / Edge neighbours moving
    /// into the vacated slot); the tab being dragged keeps following the pointer. Snaps when
    /// animations are off. PointToPoint (the plan's layout-motion curve) rather than the
    /// Decelerate its tab row names: a pill moving a full slot is a translation between two
    /// resting places, and Decelerate reads as a snap-then-drift over 60 DIP.
    pub(super) fn sync_tab_slides(&mut self, prev: &[(FenceId, f32)], now: Instant) {
        let rects = self.tab_rects();
        let dragged = self.tab_drag.as_ref().map(|d| d.index);
        let dur = self.anim_dur(motion::FAST);
        let mut live = false;
        for (i, tab) in self.tabs.iter().enumerate() {
            let Some(&(slot, _)) = rects.get(i) else {
                continue;
            };
            let slot_entry = self.tab_slide.iter().position(|(id, _)| *id == tab.id);
            if dragged == Some(i) {
                if let Some(k) = slot_entry {
                    self.tab_slide.remove(k);
                }
                continue;
            }
            let from = prev
                .iter()
                .find(|(id, _)| *id == tab.id)
                .map_or(slot, |(_, x)| *x);
            if (from - slot).abs() < 0.5 {
                if let Some(k) = slot_entry {
                    self.tab_slide.remove(k);
                }
                continue;
            }
            match slot_entry {
                Some(k) => self.tab_slide[k]
                    .1
                    .retarget(slot, dur, Curve::PointToPoint, now),
                None => self.tab_slide.push((
                    tab.id,
                    Tween::new(from, slot, dur, Curve::PointToPoint, now),
                )),
            }
            live = true;
        }
        if let Some((id, _)) = &self.tab_settle
            && self.tab_slide.iter().any(|(sid, _)| sid == id)
        {
            self.tab_settle = None;
        }
        if live {
            self.frames.request();
        }
    }

    /// The dragged pill was released (or the capture lost): it settles from under the pointer
    /// into its slot over 167 ms point-to-point.
    pub(super) fn settle_dragged_tab(&mut self, td: &TabDrag) {
        let rects = self.tab_rects();
        let (Some(from), Some(&(slot, _)), Some(tab)) = (
            self.dragged_tab_x(td, &rects),
            rects.get(td.index),
            self.tabs.get(td.index),
        ) else {
            return;
        };
        let id = tab.id;
        self.tab_slide.retain(|(sid, _)| *sid != id);
        if (from - slot).abs() < 0.5 {
            self.tab_settle = None;
            return;
        }
        let now = Instant::now();
        self.tab_settle = Some((
            id,
            self.motion
                .tween(from, slot, motion::FAST, Curve::PointToPoint, now),
        ));
        self.frames.request();
    }

    pub(super) fn active_index(&self) -> usize {
        self.tabs
            .iter()
            .position(|t| t.id == self.active)
            .unwrap_or(0)
    }
}
