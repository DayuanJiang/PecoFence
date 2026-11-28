//! Keeping fences out of each other's way: growth limits for the automatic size changes (expand,
//! icon size, auto height, cell snap), moving a dropped fence off another one and pulling
//! overlaps apart at startup.

use super::*;
use crate::layout::{FIT_SLACK_PX, clearance};

/// Device-px cell rhythm of the fence a host window shows: icon columns (none for the List /
/// Details rows) and rows below the fixed title / header part. With `cells` off
/// (`snapping.sizeToCells`) sizes are free and only the one-row minimum applies; on, the
/// automatic size changes take whole cells, but a height held short by a fence below is kept
/// exactly (the rest stays empty under the whole rows, see `ItemLayout::whole_rows_h`).
#[derive(Clone, Copy, Debug)]
pub(super) struct GridSteps {
    /// Column width and total horizontal padding.
    pub cols: Option<(f32, f32)>,
    pub row: f32,
    pub fixed: f32,
    pub cells: bool,
}

impl GridSteps {
    /// Height for a fence that may grow to `h`: `h` itself, one row at least. Whole-cell
    /// fences stop there too, exactly a gap above the fence below.
    pub fn height_within(&self, h: i32) -> i32 {
        h.max((self.fixed + self.row).round() as i32)
    }

    /// Height `h` for a window at `r` whose edge away from its title row stops at `limit`: the
    /// bottom edge, or the top one for a window that keeps its bottom (`bottom_anchored`). A
    /// window already past the limit may keep its height.
    pub fn height_up_to(&self, h: i32, r: &RECT, limit: Option<i32>, bottom_anchored: bool) -> i32 {
        let Some(limit) = limit else {
            return h;
        };
        let room = if bottom_anchored {
            r.bottom - limit.min(r.top)
        } else {
            limit.max(r.bottom) - r.top
        };
        if h > room {
            self.height_within(room)
        } else {
            h
        }
    }

    /// Height of `rows` whole rows.
    pub fn rows_height(&self, rows: u32) -> i32 {
        (self.fixed + rows as f32 * self.row).round() as i32
    }

    /// Widest width that fits in `w`: whole columns with `cells`, else `w`; one column at least.
    pub fn floor_width(&self, w: i32) -> i32 {
        let Some((cell, pad)) = self.cols else {
            return w;
        };
        if !self.cells {
            return w.max((pad + cell).round() as i32);
        }
        (pad + ((w as f32 - pad) / cell).floor().max(1.0) * cell).round() as i32
    }
}

impl App {
    /// Grid rhythm of host `id`'s shown fence at `scale`.
    pub(super) fn grid_steps(&self, id: FenceId, scale: f32) -> Option<GridSteps> {
        let shown = self.state.fence(self.state.active_tab_of(id))?;
        let metrics = GridSpec::of(&shown.view).metrics(self.ctx.chrome.label_line_h());
        // A left tab column sits before the first icon column, and replaces the title row.
        let rail = self.fences.get(&id).map_or(0.0, |w| w.tab_rail_dip());
        let title_h = if rail > 0.0 {
            0.0
        } else {
            (self.ctx.theme.borrow().title_height * scale).round()
        };
        let cells = self.state.config.settings.snapping.size_to_cells;
        let line_h = self.ctx.chrome.label_line_h();
        let rows = match shown.view.layout {
            ViewLayout::Icons => None,
            ViewLayout::List => Some(crate::layout::RowMetrics::list(line_h)),
            ViewLayout::Details => Some(crate::layout::RowMetrics::details(line_h)),
        }
        .map(|rm| rm.with_icon_size(shown.view.icon_size));
        Some(match rows {
            Some(rm) => GridSteps {
                cols: None,
                cells,
                row: rm.row_h * scale,
                fixed: title_h + (rm.header_h + rm.pad_y * 2.0) * scale + FIT_SLACK_PX as f32,
            },
            None => GridSteps {
                cols: Some((metrics.cell_w * scale, (metrics.pad_x * 2.0 + rail) * scale)),
                cells,
                row: metrics.cell_h * scale,
                fixed: title_h + metrics.pad_y * 2.0 * scale + FIT_SLACK_PX as f32,
            },
        })
    }

    /// The snapping gap (`snapping.gapPx`) in device px for `hwnd`'s monitor.
    pub(super) fn gap_px(&self, hwnd: HWND) -> i32 {
        let scale = monitors::dpi_for_window(hwnd).max(96) as f32 / 96.0;
        crate::fence_window::snap_gap_px(
            settings::snap_gap_dip(&self.state.config.settings.snapping),
            scale,
        )
    }

    /// Screen rectangles of the other fence windows that are showing, and that `id` is ever
    /// on screen with (a fence of another virtual desktop may take the same spot).
    pub(super) fn other_fence_rects(&self, id: FenceId) -> Vec<RECT> {
        self.fences
            .iter()
            .filter(|(fid, w)| {
                **fid != id
                    && pecofence_platform::desktop::is_visible(w.hwnd())
                    && !crate::anchor::is_desktop_fence(w.hwnd())
                    && self.share_a_desktop(id, **fid)
            })
            .map(|(_, w)| w.rect())
            .collect()
    }

    /// Where the other showing fences rest: without a push shift, and a rolled one (also while
    /// it hover-peeks or rolls up) as its title row. A drop is judged against these, so a spot
    /// a push or a peek only lends for a while does not count as free.
    pub(super) fn resting_fence_rects(&self, id: FenceId) -> Vec<RECT> {
        self.fences
            .iter()
            .filter(|(fid, w)| {
                **fid != id
                    && pecofence_platform::desktop::is_visible(w.hwnd())
                    && !crate::anchor::is_desktop_fence(w.hwnd())
            })
            .map(|(_, w)| self.ctx.behavior.resting_rect(w.hwnd(), w.resting_rect()))
            .collect()
    }

    /// Moves host `id` so that its resting rect `from` goes to `to`, and records it there (it
    /// rests there now). The window keeps its size: a hover peek or a roll in flight goes on
    /// from the new spot.
    /// `glide`: the user just dropped it, so the window travels there (`FenceWindow::glide_to`)
    /// instead of jumping.
    fn relocate(&mut self, id: FenceId, from: RECT, to: RECT, glide: bool) {
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        self.ctx.behavior.release_push(w.hwnd());
        let (dx, dy) = (to.left - from.left, to.top - from.top);
        let r = w.rect();
        let rolled = w.is_rolled();
        let mut rect = RECT {
            left: r.left + dx,
            top: r.top + dy,
            right: r.right + dx,
            bottom: r.bottom + dy,
        };
        // Made shorter to fit a spot (`clearance::pull_apart`): an open fence takes the spot's
        // height; a rolled-up one is its title row and keeps it.
        if !rolled {
            rect.bottom = rect.top + (to.bottom - to.top);
        }
        if !(glide && w.glide_to(rect)) {
            w.set_bounds(rect);
        }
        // Made shorter to fit a spot (`clearance::pull_apart`): the height it opens to as well.
        // One that is open only as far as the fences below leave room keeps its full height.
        if !rolled
            && w.capped_full_height_px().is_none()
            && w.expanded_height_px() != rect.bottom - rect.top
        {
            w.apply_height(rect.bottom - rect.top, false);
        }
        let expanded = if rolled {
            w.expanded_height_px()
        } else {
            rect.bottom - rect.top
        };
        self.record_bounds(id, rect, rolled, expanded, "overlap");
        self.schedule_save();
    }

    /// A fence the user dropped onto another one moves the shortest way to a free spot on its
    /// monitor (the snapping gap kept); with no free spot it stays where it was dropped. One
    /// dropped partly off the screen (a torn-off tab follows the pointer to the screen edge)
    /// is pulled back in the same way, where a restart would place it anyway. A rolled-up fence
    /// counts with the room it opens into ([`clearance::landing_spot`]): let go near the
    /// bottom of the screen, it moves up to where it opens above the taskbar, and its saved
    /// rect is where its window is.
    pub(super) fn move_out_of_overlap(&mut self, fence: FenceId) {
        let id = self.state.host_of(fence);
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        // 像桌面一样排列 places itself around the others.
        if w.is_desktop_arranged() {
            return;
        }
        let r = w.resting_rect();
        let hang = w.open_hang_px();
        let gap = self.gap_px(w.hwnd());
        let others = self.resting_fence_rects(id);
        let Some(wa) = self.work_area_at((r.left + r.right) / 2, (r.top + r.bottom) / 2) else {
            return;
        };
        let work = RECT {
            left: wa.left,
            top: wa.top,
            right: wa.right,
            bottom: wa.bottom,
        };
        // Off the screen means outside every work area: a fence dropped across two monitors
        // is left where it is.
        let foot = RECT {
            right: r.right + hang.0,
            bottom: r.bottom + hang.1,
            ..r
        };
        let on_screen = |x: i32, y: i32| {
            self.state
                .work_areas
                .iter()
                .any(|w| x >= w.left && x < w.right && y >= w.top && y < w.bottom)
        };
        let off_screen = !(on_screen(foot.left, foot.top)
            && on_screen(foot.right - 1, foot.top)
            && on_screen(foot.left, foot.bottom - 1)
            && on_screen(foot.right - 1, foot.bottom - 1));
        let Some(spot) = clearance::landing_spot(&r, hang, &others, &work, !off_screen, gap) else {
            if off_screen {
                tracing::info!(%id, rect = ?(r.left, r.top, r.right, r.bottom), ?hang, "dropped fence off the screen stays: no spot");
            }
            return;
        };
        tracing::info!(%id, off_screen, ?hang, from = ?(r.left, r.top), to = ?(spot.left, spot.top), "fence moved out of an overlap");
        self.relocate(id, r, spot, true);
    }

    /// Pulls overlapping fences apart ([`clearance::pull_apart`]): any fence with `all` (at
    /// startup), else only `movers`. Fences at their own spot are kept first, then the movers
    /// whose monitor is connected, then those whose monitor is gone.
    fn resolve_overlaps(&mut self, movers: &[FenceId], all: bool) {
        // 像桌面一样排列 lies under the others and flows around them: not a fence to pull apart.
        // Fences of other virtual desktops are never on screen with these: they may share a spot.
        let fences: Vec<(FenceId, clearance::Resting)> = self
            .fences
            .iter()
            .filter(|(id, w)| !w.is_desktop_arranged() && self.on_current_desktop(**id))
            .map(|(id, w)| {
                // A height animation in flight counts with the height it ends at: at startup an
                // open fence fitted to the gap above the fence below (`rooms_without_push`)
                // is still on its way there.
                let mut rect = w.rect();
                match w.animation_end_height_px() {
                    Some((h, true)) => rect.top = rect.bottom - h,
                    Some((h, false)) => rect.bottom = rect.top + h,
                    None => {}
                }
                let connected = |f: &pecofence_core::Fence| {
                    let on = |a: &WorkArea| a.device_path == f.geometry.monitor;
                    self.state.work_areas.iter().any(on)
                };
                let rank = match self.state.fence(*id) {
                    _ if !movers.contains(id) => 0,
                    Some(f) if connected(f) => 1,
                    _ => 2,
                };
                let work = self
                    .work_area_at((rect.left + rect.right) / 2, (rect.top + rect.bottom) / 2)
                    .map(|wa| RECT {
                        left: wa.left,
                        top: wa.top,
                        right: wa.right,
                        bottom: wa.bottom,
                    });
                // Where no spot has room for it, a fence may get as short as its title row and
                // one row of icons; rolled up it is its title row already.
                let h = rect.bottom - rect.top;
                let scale = monitors::dpi_for_window(w.hwnd()).max(96) as f32 / 96.0;
                let min_h = match self.grid_steps(*id, scale) {
                    Some(steps) if !w.is_rolled() => steps.rows_height(1).min(h),
                    _ => h,
                };
                let rest = clearance::Resting {
                    rect,
                    work,
                    gap: self.gap_px(w.hwnd()),
                    rank,
                    // A locked fence keeps its place at startup, also when a fence saved expanded
                    // above it could not push it clear; one with no spot on these monitors moves.
                    movable: movers.contains(id)
                        || (all && !self.state.fence(*id).is_some_and(|f| f.locked)),
                    min_h,
                    hang: w.open_hang_px(),
                };
                (*id, rest)
            })
            .collect();
        let rests: Vec<clearance::Resting> = fences.iter().map(|(_, r)| *r).collect();
        for (i, spot) in clearance::pull_apart(&rests) {
            let (id, from) = (fences[i].0, fences[i].1.rect);
            let (from_h, to_h) = (from.bottom - from.top, spot.bottom - spot.top);
            // `shorter`: no spot had room for its height (a screen made smaller).
            tracing::info!(%id, from = ?(from.left, from.top), to = ?(spot.left, spot.top), from_h, to_h, shorter = to_h < from_h, hang = ?fences[i].1.hang, "overlapping fence moved");
            self.relocate(id, from, spot, false);
        }
    }

    /// Startup (`all`: every overlap the layout brings is pulled apart) and new monitors: the
    /// fences with no spot on these monitors yet ([`AppState::unplaced`]) move out of the
    /// others' way and are recorded where they end up, on the monitor they are on now.
    pub(super) fn settle_unplaced(&mut self, all: bool) {
        let movers = std::mem::take(&mut self.state.unplaced);
        if movers.is_empty() && !all {
            return;
        }
        // A fence saved expanded pushes the ones below it before overlaps are judged; the movers
        // sit this pass out, so one that lands inside a fence moves to free space instead.
        self.reflow_pushed_except(&movers);
        self.resolve_overlaps(&movers, all);
        for id in movers {
            let Some(w) = self.fences.get(&id) else {
                continue;
            };
            let (rect, rolled, expanded) = (w.rect(), w.is_rolled(), w.expanded_height_px());
            self.record_bounds(id, rect, rolled, expanded, "unplaced");
        }
        self.schedule_save();
    }

    /// Lowest bottom edge host `id` at `r` may grow to without running into a fence below. One
    /// whose title row is at its bottom (a 向上展开 fence that opened upward) keeps that edge,
    /// so nothing below limits it.
    pub(super) fn growth_limit_below(&self, id: FenceId, r: &RECT) -> Option<i32> {
        let w = self.fences.get(&id)?;
        if w.row_at_bottom() {
            return None;
        }
        clearance::limit_below(&self.other_fence_rects(id), r, self.gap_px(w.hwnd()))
    }

    /// Highest top edge host `id` at `r` may grow to without running into a fence above, for a
    /// fence whose title row is at its bottom (None for the others).
    pub(super) fn growth_limit_above(&self, id: FenceId, r: &RECT) -> Option<i32> {
        let w = self.fences.get(&id)?;
        if !w.row_at_bottom() {
            return None;
        }
        clearance::limit_above(&self.other_fence_rects(id), r, self.gap_px(w.hwnd()))
    }

    /// With `rollUp.pushNeighbors` off, the gap under host `id` (not one with 向上展开): the room
    /// below its top edge down to the next fence below, and the height of its title row and one
    /// row of icons, and the snapping gap. None when no fence below limits it. `obstacles`: the
    /// fences it can run into ([`Self::gap_obstacles`]).
    fn gap_below(&self, id: FenceId, obstacles: &[(FenceId, RECT)]) -> Option<(i32, i32, i32)> {
        if self.state.fence(id).is_some_and(|f| f.expand_upward) {
            return None;
        }
        let w = self.fences.get(&id)?;
        // Collapsed to its left tab column, the fence expands to its own width too. Measured
        // from its title row, as when it is rolled: open in full over a fence below (after a
        // restart), that one still counts.
        let mut r = w.footprint_rect();
        r.bottom = r.top + w.resting_heights_px()?.0;
        if w.row_at_bottom() {
            return None;
        }
        let others: Vec<RECT> = (obstacles.iter())
            .filter(|(fid, _)| *fid != id)
            .map(|&(_, r)| r)
            .collect();
        let gap = self.gap_px(w.hwnd());
        let limit = clearance::limit_below(&others, &r, gap)?;
        let scale = monitors::dpi_for_window(w.hwnd()).max(96) as f32 / 96.0;
        let steps = self.grid_steps(id, scale)?;
        Some((limit - r.top, steps.height_within(0), gap))
    }

    /// What a gap is measured against, read once per pass: every fence of the current desktop,
    /// shown yet or not (at startup none is), as the push pass counts them; one in the user's
    /// hands (`held`) limits nothing until it is let go.
    fn gap_obstacles(&self, held: &[HWND]) -> Vec<(FenceId, RECT)> {
        (self.fences.iter())
            .filter(|(fid, o)| {
                !o.is_desktop_arranged()
                    && !held.contains(&o.hwnd())
                    && self.on_current_desktop(**fid)
            })
            .map(|(&fid, o)| (fid, o.rect()))
            .collect()
    }

    /// How a fence of `full` px opens over the gap below it (`gap_below`): `Some(true)` above
    /// the fences there (the gap is under one row), `Some(false)` held short of them, None in
    /// full. Only a fence whose full height reaches into the fence below is limited: one that
    /// rests within the snapping gap above it keeps its height, as in 0.5.0.
    fn gap_limits(full: i32, (room, row, gap): (i32, i32, i32)) -> Option<bool> {
        (full > room + gap).then_some(room < row)
    }

    /// The windows in the user's hands (moved or resized), which keep the height they have.
    pub(super) fn windows_in_hand(&self) -> Vec<HWND> {
        let mut held = Vec::new();
        if let Some(sm) = self.ctx.behavior.size_move.borrow().as_ref() {
            held.push(sm.hwnd);
            held.extend(&sm.group);
        }
        held.extend(
            self.fences
                .values()
                .filter(|w| w.zoom_busy())
                .map(|w| w.hwnd()),
        );
        // A tab torn off and dragged, as the push pass counts it.
        held.extend(self.fences.values().flat_map(|w| w.dragged_windows()));
        held
    }

    /// With `rollUp.pushNeighbors` off, how far each fence opens below its top edge: it stops
    /// short of the next fence below while that gap holds a row of icons, and is held there
    /// only for as long as it is open, as with the push on: the saved height stays its own, and
    /// a hover peek and the next open take it (feedback #207: a stop short was saved, so the
    /// top fence of a stack opened one row from then on). With a fence closer than one row it
    /// opens to its saved height above the fences there, as a hover peek does.
    pub(super) fn rooms_without_push(&self) -> Vec<(HWND, i32)> {
        let held = self.windows_in_hand();
        let obstacles = self.gap_obstacles(&held);
        self.fences
            .iter()
            .filter(|(_, w)| !w.is_desktop_arranged() && !held.contains(&w.hwnd()))
            .filter_map(|(&id, w)| {
                let gap = self.gap_below(id, &obstacles)?;
                let on_top = Self::gap_limits(w.expanded_height_px(), gap)?;
                (!on_top).then_some((w.hwnd(), gap.0))
            })
            .collect()
    }

    /// A rolled fence opens with `rollUp.pushNeighbors` off: says how far (the decision the
    /// opening takes from [`Self::rooms_without_push`]) and, when it covers the fences under it,
    /// raises it above them until it rolls up again.
    pub(super) fn open_without_push(&self, id: FenceId) {
        let obstacles = self.gap_obstacles(&self.windows_in_hand());
        let (Some(gap), Some(w)) = (self.gap_below(id, &obstacles), self.fences.get(&id)) else {
            return;
        };
        let full = w.expanded_height_px();
        let on_top = Self::gap_limits(full, gap);
        let outcome = match on_top {
            None => "in full: it does not reach the fence below",
            Some(false) => "stops short of the fence below",
            Some(true) => "in full, above the fences below: the gap is under one row",
        };
        tracing::info!(%id, gap_px = gap.0, row_px = gap.1, full_px = full, outcome, "fence opens with pushing off");
        if on_top == Some(true) {
            w.raise_while_open();
        }
    }

    /// Startup with `rollUp.pushNeighbors` off: a fence left open over a gap under one row lay
    /// above the fences under it only while it was raised, which a restart does not keep. It
    /// comes back rolled up, as a hover peek would, so it covers nothing and no overlap is
    /// pulled apart.
    pub(super) fn roll_up_opened_on_top(&mut self) {
        if self.state.config.settings.roll_up.push_neighbors {
            return;
        }
        let open: Vec<FenceId> = (self.fences.iter())
            .filter(|(_, w)| !w.is_rolled() && !w.is_desktop_arranged())
            .map(|(&id, _)| id)
            .collect();
        let obstacles = self.gap_obstacles(&[]);
        for id in open {
            let full = self.fences.get(&id).map_or(0, |w| w.expanded_height_px());
            if let Some(gap) = self.gap_below(id, &obstacles)
                && Self::gap_limits(full, gap) == Some(true)
            {
                tracing::info!(%id, gap_px = gap.0, row_px = gap.1, "startup: a fence open over the ones under it is rolled up");
                self.set_roll(id, true);
            }
        }
    }

    /// Where a notice in the corner (`crate::notice`) must not go: the fence windows on show,
    /// but for 「桌面」 arranged like the desktop, which lies under everything as the desktop does.
    pub(super) fn notice_avoid_rects(&self) -> Vec<RECT> {
        self.fences
            .values()
            .filter(|w| {
                pecofence_platform::desktop::is_visible(w.hwnd()) && !w.is_desktop_arranged()
            })
            .map(|w| w.rect())
            .collect()
    }

    /// First run: the default column is sized to what the rules sorted into it, in whole rows,
    /// and restacked from the top. Fixed heights left a half-cut row in one fence and mostly
    /// empty plates in the others. Every fence shows all its rows: where the column has no room
    /// for the next one (a short screen, many icons), that one starts another column to the left
    /// of it; only a fence taller than the whole screen scrolls.
    pub(super) fn fit_first_run_column(&mut self) {
        // 像桌面一样排列: the 「桌面」 fence stands on the desktop, not in the column.
        let ids: Vec<FenceId> = self
            .state
            .fences()
            .iter()
            .map(|f| f.id)
            .filter(|id| !self.fences.get(id).is_some_and(|w| w.is_desktop_arranged()))
            .collect();
        let Some(hwnd) = ids
            .first()
            .and_then(|id| self.fences.get(id))
            .map(|w| w.hwnd())
        else {
            return;
        };
        let Some(monitor) = monitors::query(monitors::monitor_from_window(hwnd)) else {
            return;
        };
        let scale = monitors::dpi_for_window(hwnd).max(96) as f32 / 96.0;
        // The snapping gap, as between fences the user drags together later.
        let gap = self.gap_px(hwnd);
        let work = monitor.work_area;
        let column_h = work.bottom - work.top - gap * 2;
        let mut heights = Vec::new();
        let mut rows = Vec::new();
        for &id in &ids {
            let (Some(w), Some(steps)) = (self.fences.get(&id), self.grid_steps(id, scale)) else {
                return;
            };
            let Some(fit) = w.fit_report() else {
                return;
            };
            // Every row, not `fitting_height_px`: that stops at the work area's bottom from
            // where the fence sits before the restack. Taller than the column: the rows that
            // fit in it.
            let fit_rows = fit.rows.max(1) as u32;
            let room = ((column_h as f32 - steps.fixed) / steps.row)
                .floor()
                .max(1.0) as u32;
            heights.push(steps.rows_height(fit_rows.min(room)));
            rows.push(fit_rows);
        }
        let mut top = work.top + gap;
        let mut shift = 0;
        let mut columns = 1;
        for (&id, &h) in ids.iter().zip(&heights) {
            let Some(w) = self.fences.get(&id) else {
                continue;
            };
            let r = w.rect();
            if top + h > work.bottom - gap && top > work.top + gap {
                shift += r.right - r.left + gap;
                top = work.top + gap;
                columns += 1;
            }
            let rect = RECT {
                left: r.left - shift,
                top,
                right: r.right - shift,
                bottom: top + h,
            };
            w.set_bounds(rect);
            w.apply_height(h, false);
            self.record_bounds(id, rect, false, h, "first run");
            top += h + gap;
        }
        tracing::info!(
            fences = ids.len(),
            ?rows,
            ?heights,
            column_h,
            columns,
            "first run: fences laid out in columns"
        );
        self.schedule_save();
    }

    /// Icon size, spacing or label lines changed on a fixed-height fence, or 适应内容大小: the
    /// height follows the content (no band of empty rows after shrinking the cells, no
    /// scrollbar after growing them). With 推开下方的栅栏 the fences below make room, as for an
    /// expand, and come back when it shrinks; without it the fence grows only as far as the next
    /// fence below. The bottom edge glides there like an auto-height fence's. Returns the new
    /// height (None: nothing changed).
    pub(super) fn fit_height_to_content(&mut self, id: FenceId) -> Option<i32> {
        let id = self.state.host_of(id);
        // A tool fence keeps the height the user gave it (it has no rows of icons).
        if self
            .state
            .fence(id)
            .is_none_or(|f| f.view.auto_height || f.tool.is_some())
        {
            return None;
        }
        let w = self.fences.get(&id)?;
        if w.is_rolled() {
            return None;
        }
        // Measured at the fence's own size, never at a zoom's (`fence_window/zoom.rs`).
        w.end_zoom();
        let fit = w.fit_report()?;
        let r = w.rect();
        let scale = monitors::dpi_for_window(w.hwnd()).max(96) as f32 / 96.0;
        let mut h = fit.fitting_height_px;
        let push = self.state.config.settings.roll_up.push_neighbors;
        if !push
            && let Some(limit) = self.growth_limit_below(id, &r)
            && let Some(steps) = self.grid_steps(id, scale)
        {
            let limit = limit.max(r.bottom);
            if r.top + h > limit {
                h = steps.height_within(limit - r.top);
            }
        }
        // A fence that expands upward grows only as far as the next fence above.
        if let Some(limit) = self.growth_limit_above(id, &r)
            && let Some(steps) = self.grid_steps(id, scale)
        {
            let limit = limit.min(r.top);
            if r.bottom - h < limit {
                h = steps.height_within(r.bottom - limit);
            }
        }
        if h == r.bottom - r.top {
            return None;
        }
        let rect = w.apply_height(h, !self.fits_at_once);
        self.record_bounds(id, rect, false, h, "fit");
        self.schedule_save();
        Some(h)
    }

    /// 适应内容大小 (fence menu) and a double-click on the bottom edge: the height that shows
    /// every icon (`fit_height_to_content`). Nothing for a fence placed like the desktop, a
    /// locked, rolled or auto-height one (it fits already), or a tool fence.
    pub(super) fn fit_to_content_by_user(&mut self, fence: FenceId, by: &'static str) {
        let host = self.state.host_of(fence);
        let Some(w) = self.fences.get(&host) else {
            return;
        };
        let from_h = w.rect().bottom - w.rect().top;
        let f = self.state.fence(host);
        let skip = if w.is_desktop_arranged() {
            Some("desktop layout")
        } else if f.is_some_and(|f| f.locked) {
            Some("locked")
        } else if w.is_rolled() {
            Some("rolled")
        } else if f.is_some_and(|f| f.view.auto_height) {
            Some("auto height")
        } else if self.state.fence(fence).is_some_and(|f| f.tool.is_some()) {
            Some("tool fence")
        } else {
            None
        };
        if let Some(reason) = skip {
            tracing::info!(%fence, by, reason, "fit to content skipped");
            return;
        }
        // The height the user asks for now is the one an icon-size step comes back to.
        self.icon_size_heights.remove(&host);
        let to_h = self.fit_height_to_content(fence);
        tracing::info!(%fence, by, from_h, ?to_h, "fit to content");
        self.push_settings_state();
    }
}
