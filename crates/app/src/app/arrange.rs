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
        let metrics =
            crate::layout::GridMetrics::for_icon_size(shown.view.icon_size, shown.view.label_lines)
                .with_line_h(self.ctx.chrome.label_line_h())
                .with_spacing(shown.view.spacing);
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
        };
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
        (settings::snap_gap_dip(&self.state.config.settings.snapping) as f32 * scale).round() as i32
    }

    /// Screen rectangles of the other fence windows that are showing.
    pub(super) fn other_fence_rects(&self, id: FenceId) -> Vec<RECT> {
        self.fences
            .iter()
            .filter(|(fid, w)| {
                **fid != id
                    && pecofence_platform::desktop::is_visible(w.hwnd())
                    && !crate::anchor::is_desktop_fence(w.hwnd())
            })
            .map(|(_, w)| w.rect())
            .collect()
    }

    /// Moves host `id` to `rect` and records it (it rests there now).
    fn relocate(&mut self, id: FenceId, rect: RECT) {
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        self.ctx.behavior.release_push(w.hwnd());
        w.set_bounds(rect);
        let rolled = w.is_rolled();
        let expanded = if rolled {
            w.expanded_height_px()
        } else {
            rect.bottom - rect.top
        };
        self.record_bounds(id, rect, rolled, expanded);
        self.schedule_save();
    }

    /// A fence the user dropped onto another one moves the shortest way to a free spot on its
    /// monitor (the snapping gap kept); with no free spot it stays where it was dropped. One
    /// dropped partly off the screen (a torn-off tab follows the pointer to the screen edge)
    /// is pulled back in the same way, where a restart would place it anyway.
    pub(super) fn move_out_of_overlap(&mut self, fence: FenceId) {
        let id = self.state.host_of(fence);
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        let r = w.rect();
        let gap = self.gap_px(w.hwnd());
        let others = self.other_fence_rects(id);
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
        let on_screen = |x: i32, y: i32| {
            self.state
                .work_areas
                .iter()
                .any(|w| x >= w.left && x < w.right && y >= w.top && y < w.bottom)
        };
        let off_screen = !(on_screen(r.left, r.top)
            && on_screen(r.right - 1, r.top)
            && on_screen(r.left, r.bottom - 1)
            && on_screen(r.right - 1, r.bottom - 1));
        if !off_screen && !others.iter().any(|o| clearance::overlaps(&r, o, gap)) {
            return;
        }
        if let Some(spot) = clearance::nearest_free(&r, &others, &work, gap) {
            tracing::info!(%id, off_screen, from = ?(r.left, r.top), to = ?(spot.left, spot.top), "fence moved out of an overlap");
            self.relocate(id, spot);
        }
    }

    /// Pulls overlapping fences apart ([`clearance::pull_apart`]): any fence with `all` (at
    /// startup), else only `movers`. Fences at their own spot are kept first, then the movers
    /// whose monitor is connected, then those whose monitor is gone.
    fn resolve_overlaps(&mut self, movers: &[FenceId], all: bool) {
        let fences: Vec<(FenceId, clearance::Resting)> = self
            .fences
            .iter()
            .map(|(id, w)| {
                let rect = w.rect();
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
                let rest = clearance::Resting {
                    rect,
                    work,
                    gap: self.gap_px(w.hwnd()),
                    rank,
                    // A locked fence keeps its place at startup, also when a fence saved expanded
                    // above it could not push it clear; one with no spot on these monitors moves.
                    movable: movers.contains(id)
                        || (all && !self.state.fence(*id).is_some_and(|f| f.locked)),
                };
                (*id, rest)
            })
            .collect();
        let rests: Vec<clearance::Resting> = fences.iter().map(|(_, r)| *r).collect();
        for (i, spot) in clearance::pull_apart(&rests) {
            let (id, from) = (fences[i].0, fences[i].1.rect);
            tracing::info!(%id, from = ?(from.left, from.top), to = ?(spot.left, spot.top), "overlapping fence moved");
            self.relocate(id, spot);
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
            self.record_bounds(id, rect, rolled, expanded);
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

    /// Expanding a rolled fence with `rollUp.pushNeighbors` off: the expanded height stops short
    /// of a fence placed under the title row meanwhile (the rest of the items scroll). A fence
    /// with 向上展开 picks its side and height from the room around its title row as it opens
    /// instead (`expand_rooms_px`), without changing the height it is saved with.
    pub(super) fn clamp_expand_height(&mut self, id: FenceId) {
        if self.state.fence(id).is_some_and(|f| f.expand_upward) {
            return;
        }
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        // Collapsed to its left tab column, the fence expands to its own width too.
        let r = w.footprint_rect();
        let expanded = w.expanded_height_px();
        let scale = monitors::dpi_for_window(w.hwnd()).max(96) as f32 / 96.0;
        let (Some(limit), Some(steps)) =
            (self.growth_limit_below(id, &r), self.grid_steps(id, scale))
        else {
            return;
        };
        if r.top + expanded <= limit {
            return;
        }
        let h = steps.height_within(limit - r.top);
        if h >= expanded {
            return;
        }
        w.set_expanded_height_px(h);
        self.record_bounds(id, r, true, h);
    }

    /// First run: the default column is sized to what the rules sorted into it, in whole rows,
    /// and restacked from the top. Fixed heights left a half-cut row in one fence and mostly
    /// empty plates in the others.
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
        let gap = (16.0 * scale).round() as i32;
        let mut want = Vec::new();
        for &id in &ids {
            let (Some(w), Some(steps)) = (self.fences.get(&id), self.grid_steps(id, scale)) else {
                return;
            };
            let Some(fit) = w.fit_report() else {
                return;
            };
            // Every row, not `fitting_height_px`: that stops at the work area's bottom from
            // where the fence sits before the restack.
            want.push((steps.rows_height(fit.rows.max(1) as u32), steps));
        }
        // One row each, then one more row each in turn while it fits: fences with a few
        // icons stop at their fitting height, the others share the rest of the column.
        let work = monitor.work_area;
        let mut left = work.bottom - work.top - gap * (ids.len() as i32 + 1);
        let mut rows = vec![1; ids.len()];
        let mut heights: Vec<i32> = want
            .iter()
            .map(|(fit, s)| s.rows_height(1).min(*fit))
            .collect();
        left -= heights.iter().sum::<i32>();
        let mut grew = true;
        while grew {
            grew = false;
            for ((h, n), (fit, steps)) in heights.iter_mut().zip(&mut rows).zip(&want) {
                let taller = steps.rows_height(*n + 1).min(*fit);
                if taller > *h && taller - *h <= left {
                    left -= taller - *h;
                    *h = taller;
                    *n += 1;
                    grew = true;
                }
            }
        }
        let mut top = work.top + gap;
        // Fences the column has no room for (a small screen at a high scale) start another
        // column to the left of it.
        let mut shift = 0;
        for (&id, h) in ids.iter().zip(heights) {
            let Some(w) = self.fences.get(&id) else {
                continue;
            };
            let r = w.rect();
            if top + h > work.bottom - gap && top > work.top + gap {
                shift += r.right - r.left + gap;
                top = work.top + gap;
            }
            let rect = RECT {
                left: r.left - shift,
                top,
                right: r.right - shift,
                bottom: top + h,
            };
            w.set_bounds(rect);
            w.apply_height(h, false);
            self.record_bounds(id, rect, false, h);
            top += h + gap;
        }
        self.schedule_save();
    }

    /// Icon-size change on a fixed-height fence: the height follows the content (no band of
    /// empty rows after shrinking the icons), growing only as far as the next fence below. The
    /// bottom edge glides there like an auto-height fence's.
    pub(super) fn fit_height_to_content(&mut self, id: FenceId) {
        let id = self.state.host_of(id);
        if self.state.fence(id).is_none_or(|f| f.view.auto_height) {
            return;
        }
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        if w.is_rolled() {
            return;
        }
        let Some(fit) = w.fit_report() else {
            return;
        };
        let r = w.rect();
        let scale = monitors::dpi_for_window(w.hwnd()).max(96) as f32 / 96.0;
        let mut h = fit.fitting_height_px;
        if let Some(limit) = self.growth_limit_below(id, &r)
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
            return;
        }
        let rect = w.apply_height(h, true);
        self.record_bounds(id, rect, false, h);
        self.schedule_save();
    }
}
