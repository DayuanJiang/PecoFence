//! Fence windows and geometry: create / sync / resync windows, bounds, cell snap, auto height,
//! display changes, monitors, new-fence placement, roll, title rename, hwnd lookup.

use super::*;

#[cfg(debug_assertions)]
thread_local! {
    /// `--test-script` `monitors`: stands in for the connected monitors.
    pub(super) static TEST_MONITORS: std::cell::RefCell<Option<Vec<WorkArea>>> =
        const { std::cell::RefCell::new(None) };
}

/// One log line per monitor, at startup and after every display change.
pub(super) fn log_monitors(areas: &[WorkArea]) {
    for w in areas {
        tracing::info!(
            name = %w.device_path,
            dpi = w.dpi,
            size = ?(w.mon_right - w.mon_left, w.mon_bottom - w.mon_top),
            work = ?(w.left, w.top, w.right, w.bottom),
            "monitor"
        );
    }
}

pub(super) fn work_areas() -> Vec<WorkArea> {
    #[cfg(debug_assertions)]
    if let Some(areas) = TEST_MONITORS.with_borrow(Clone::clone) {
        return areas;
    }
    let targets = monitors::display_targets();
    // The primary monitor first: the first-run column and every "no monitor matched" fallback
    // take `first()`, and Windows enumerates in display-number order.
    let mut found = monitors::enumerate();
    found.sort_by_key(|m| !m.primary);
    found
        .into_iter()
        .map(|m| WorkArea {
            device_path: monitor_id(
                &m.device_name,
                targets.iter().find(|t| t.gdi_name == m.device_name),
            ),
            gdi_name: m.device_name.clone(),
            left: m.work_area.left,
            top: m.work_area.top,
            right: m.work_area.right,
            bottom: m.work_area.bottom,
            dpi: m.dpi,
            mon_left: m.bounds.left,
            mon_top: m.bounds.top,
            mon_right: m.bounds.right,
            mon_bottom: m.bounds.bottom,
        })
        .collect()
}

/// See [`WorkArea::device_path`].
fn monitor_id(gdi_name: &str, target: Option<&monitors::DisplayTarget>) -> String {
    match target {
        Some(t) if !t.instance.is_empty() => t.instance.clone(),
        Some(monitors::DisplayTarget {
            model: Some(model),
            connector: (tech, index),
            ..
        }) => format!("{model}#{tech}.{index}"),
        _ => gdi_name.to_string(),
    }
}

impl App {
    /// Creates/destroys/updates fence windows to match the state.
    pub(super) fn sync_fence_windows(&mut self) {
        // Portal folders are re-read by their watchers / on_fs_changed / startup, not here:
        // enumerating every portal (shell display names per entry) made a tab tear-off take
        // hundreds of milliseconds before the new window appeared.
        self.ensure_portal_watchers();
        self.state.normalize_tabs();
        // Only host fences own a window; fences hosted as tabs live inside their host's. An
        // empty fence set to 为空时自动隐藏 has none until something lands in it, and an inbox
        // hidden beside the Windows icons none until they are hidden again.
        let hidden = self.state.hidden_fences();
        let mut hosts = self.state.host_fences();
        hosts.retain(|f| !hidden.contains(&f.id));
        let ids: Vec<FenceId> = hosts.iter().map(|f| f.id).collect();
        let gone: Vec<FenceId> = self
            .fences
            .iter()
            .filter(|(id, w)| !ids.contains(id) || !window::is_window(w.hwnd()))
            .map(|(id, _)| *id)
            .collect();
        for id in gone {
            if let Some(w) = self.fences.remove(&id) {
                self.retire_window(w);
            }
        }
        for fence in hosts {
            let active = self.state.active_tab_of(fence.id);
            let shown = self.state.fence(active).cloned().unwrap_or(fence.clone());
            let items = self.item_views(&shown);
            let tabs = self.tab_views(fence.id);
            if let Some(w) = self.fences.get(&fence.id) {
                w.set_tabs(tabs, active);
                w.set_group_by_date(shown.view.group_by_date);
                w.set_items(items);
                // The window may have been kept across a layout switch (same FenceId, other
                // per-fence view flags): push every persisted flag, not just items/title.
                self.apply_fence_view(fence.id);
                continue;
            }
            let px = self.state.fence_px_rect(&fence);
            let rect = RECT {
                left: px.left,
                top: px.top,
                right: px.right,
                bottom: px.bottom,
            };
            match FenceWindow::create(
                &self.ctx,
                fence.id,
                &fence.title,
                shown.kind == FenceKind::Inbox,
                fence.rolled_up,
                fence.expand_upward,
                fence.bottom_anchored,
                shown.view.icon_size,
                shown.view.label_lines,
                fence.view.auto_height,
                rect,
                self.state.fence_expanded_px(&fence),
                items,
            ) {
                Ok(w) => {
                    w.set_tabs(tabs, active);
                    self.fences.insert(fence.id, w);
                    self.apply_fence_view(fence.id);
                }
                Err(e) => tracing::error!(title = %fence.title, error = %e, "fence window failed"),
            }
        }
    }

    /// The window a fence is shown in (its own, or its tab host's).
    pub(super) fn window_for(&self, fence: FenceId) -> Option<&FenceWindow> {
        self.fences.get(&self.state.host_of(fence))
    }

    /// After tabs/fences were added, removed, merged or split: create/destroy windows, put new
    /// ones into the desktop z-band and show them.
    pub(super) fn resync_windows(&mut self) {
        let before: Vec<HWND> = self.fences.values().map(|w| w.hwnd()).collect();
        self.sync_fence_windows();
        if let Some(a) = self.anchor.borrow_mut().as_mut() {
            a.reanchor("windows resynced");
        }
        let mut shown = false;
        for w in self.fences.values() {
            if !before.contains(&w.hwnd()) {
                w.show(true);
                shown = true;
            }
        }
        if shown && let Some(a) = self.anchor.borrow_mut().as_mut() {
            a.reanchor("new windows shown");
        }
        // A fence that came (shown just now) or went changes the cells the desktop-arranged
        // 「桌面」 fence may use.
        self.arrange_desktop_fence();
        self.push_settings_state();
    }

    pub(super) fn refresh_fence(&mut self, id: FenceId) {
        let host = self.state.host_of(id);
        let active = self.state.active_tab_of(host);
        if let Some(w) = self.fences.get(&host) {
            let switching = w.active_fence() != active;
            w.set_tabs(self.tab_views(host), active);
            if active == id
                && let Some(f) = self.state.fence(id)
            {
                // Grouping first: the layout glide must target the new sections.
                w.set_group_by_date(f.view.group_by_date);
                if switching {
                    // After `set_tabs`, whose content swap draws the outgoing items once more.
                    w.set_incoming_tab_view(GridSpec::of(&f.view));
                }
                w.set_items(self.item_views(f));
                w.set_sort_indicator(f.view.sort, f.view.reverse);
            }
        }
        self.apply_portal_deco(host);
        self.apply_auto_height(host);
        self.sync_hidden_fences();
    }

    /// 为空时自动隐藏: a fence's window goes with its last item and comes back with the next
    /// one (or when the flag is switched off). Also when the inbox's hiding beside the Windows
    /// icons starts or ends.
    pub(super) fn sync_hidden_fences(&mut self) {
        let hidden = self.state.hidden_fences();
        let changed: Vec<FenceId> = self
            .state
            .host_fences()
            .iter()
            .map(|f| f.id)
            .filter(|id| self.fences.contains_key(id) == hidden.contains(id))
            .collect();
        if changed.is_empty() {
            return;
        }
        self.resync_windows();
        // While it had no window, another fence may have grown or moved over its spot.
        for id in changed {
            if self.fences.contains_key(&id) {
                self.move_out_of_overlap(id);
            }
        }
    }

    /// Pushes the persisted per-fence flags (lock, expand upward, quick-hide exclusion,
    /// appearance override, title on hover) into the window and the anchor. No geometry: the
    /// window keeps its rect.
    pub(super) fn apply_fence_appearance(&mut self, id: FenceId) {
        let id = self.state.host_of(id);
        let Some(f) = self.state.fence(id).cloned() else {
            return;
        };
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        w.set_locked(f.locked);
        // The flag only: a window rolled up at rest is where its saved rect says (the callers
        // that change it place the window from the state afterwards).
        w.set_expand_upward(f.expand_upward);
        let (bd, op) = f
            .appearance
            .as_ref()
            .map(|a| (a.backdrop.map(backdrop_mode_for), a.opacity.unwrap_or(1.0)))
            .unwrap_or((None, 1.0));
        w.set_appearance(bd, op);
        w.set_title_on_hover(f.appearance.as_ref().and_then(|a| a.title_on_hover));
        w.set_style(fence_style_for(&f));
        if let Some(a) = self.anchor.borrow_mut().as_mut() {
            a.set_quick_hide_excluded(w.hwnd(), f.exclude_from_quick_hide);
        }
    }

    /// Pushes every persisted per-fence view flag (icon size, auto height, lock/appearance,
    /// rolled state) into an existing window and re-applies auto height. This is the one place
    /// that makes a window agree with its `Fence`, whatever path changed the state (startup,
    /// layout switch, display change, tabs, restore). The saved rectangle is kept: whole cells
    /// are only re-taken by the explicit changes (`apply_cell_snap`), so fences the user lined
    /// up stay lined up.
    pub(super) fn apply_fence_view(&mut self, id: FenceId) {
        // Geometry-ish flags (rolled, auto height, lock, appearance) belong to the host window;
        // content flags (icon size, layout, sort) to the tab being shown.
        let id = self.state.host_of(id);
        let Some(f) = self.state.fence(id).cloned() else {
            return;
        };
        let shown = self
            .state
            .fence(self.state.active_tab_of(id))
            .cloned()
            .unwrap_or_else(|| f.clone());
        {
            let Some(w) = self.fences.get(&id) else {
                return;
            };
            w.set_grid(GridSpec::of(&shown.view));
            w.set_is_inbox(shown.kind == FenceKind::Inbox);
            w.set_column_widths(
                shown
                    .view
                    .column_widths
                    .unwrap_or(crate::layout::DetailColumns::DEFAULT_WIDTHS),
            );
            w.set_columns_visible(shown.view.columns_visible.unwrap_or([true; 3]));
            w.set_group_by_date(shown.view.group_by_date);
            w.set_sort_indicator(shown.view.sort, shown.view.reverse);
            w.set_auto_height(f.view.auto_height);
            if w.is_rolled() != f.rolled_up {
                w.set_rolled(f.rolled_up);
            }
        }
        self.apply_fence_appearance(id); // locked + backdrop/opacity + quick-hide exclusion
        self.apply_portal_deco(id);
        self.apply_auto_height(id);
        self.arrange_desktop_fence();
    }

    pub(super) fn refresh_all(&mut self) {
        let ids: Vec<FenceId> = self.fences.keys().copied().collect();
        for id in ids {
            let active = self.state.active_tab_of(id);
            if let Some(f) = self.state.fence(active)
                && let Some(w) = self.fences.get(&id)
            {
                // A fence the change did not touch keeps its view: no redraw, and its hover,
                // press and type-ahead stay (asked before `set_tabs`, which may switch tabs).
                let items = self.item_views(f);
                let unchanged = w.active_fence() == active && w.shows_items(&items);
                w.set_tabs(self.tab_views(id), active);
                w.set_group_by_date(f.view.group_by_date);
                if !unchanged {
                    w.set_items(items);
                }
            }
            self.apply_auto_height(id);
        }
        self.sync_hidden_fences();
        self.arrange_desktop_fence();
        // Cut items that were moved away (Explorer pasted them) stop being dimmed.
        let before = self.cut_items.len();
        self.cut_items.retain(|id| self.state.item(*id).is_some());
        if self.cut_items.len() != before {
            self.push_cut_items();
        }
    }

    /// 「调整大小时按整行整列对齐」 (`snapping.sizeToCells`, Fences' "sized to even
    /// multiples of icons"): the width becomes whole icon columns and a fixed height whole rows.
    /// Interactive resizing snaps in WM_SIZING; this applies the same rule to the explicit
    /// changes of the cell size (icon size, spacing, layout, label lines). Other sizes still
    /// show whole icons (the columns share the rest, the rows leave it empty), so loading, DPI
    /// changes, tabs and switching the setting on keep the saved rectangle. Off, sizes are free.
    pub(super) fn apply_cell_snap(&mut self, id: FenceId) {
        if self.snap_held {
            return;
        }
        let id = self.state.host_of(id);
        let Some(f) = self.state.fence(id).cloned() else {
            return;
        };
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        if w.is_desktop_arranged() {
            return;
        }
        let r = w.rect();
        let scale = monitors::dpi_for_window(w.hwnd()).max(96) as f32 / 96.0;
        let Some(steps) = self.grid_steps(id, scale) else {
            return;
        };
        // Collapsed to its left tab column, the window is as wide as the column.
        if !steps.cells || w.collapsed_expanded_width_px().is_some() {
            return;
        }
        let mut width = r.right - r.left;
        let mut left = r.left;
        if let Some((cell, pad)) = steps.cols {
            let cols = ((width as f32 - pad) / cell).round().max(1.0);
            width = (pad + cols * cell).round() as i32;
            // Rounding up never pushes a fence past the work-area edge or onto a neighbour:
            // shift it left, else round down.
            let (cx, cy) = ((r.left + r.right) / 2, (r.top + r.bottom) / 2);
            if let Some(wa) = self.work_area_at(cx, cy)
                && left + width > wa.right
            {
                left = (wa.right - width).max(wa.left);
            }
            let grown = RECT {
                left,
                top: r.top,
                right: left + width,
                bottom: r.bottom,
            };
            let gap = self.gap_px(w.hwnd());
            if width > r.right - r.left
                && self.other_fence_rects(id).iter().any(|o| {
                    crate::layout::clearance::overlaps(&grown, o, gap)
                        && !crate::layout::clearance::overlaps(&r, o, gap)
                })
            {
                left = r.left;
                width = steps.floor_width(r.right - r.left);
            }
        }
        // Rolled fences keep their title-only height; auto-height ones get an exact row count.
        let rolled = w.is_rolled();
        let mut bottom = r.bottom;
        if !rolled && !f.view.auto_height {
            let h = (r.bottom - r.top) as f32;
            let rows = ((h - steps.fixed) / steps.row).round().max(1.0);
            // A fence that grows downward stops at the fence below, one that grows upward (its
            // title row at the bottom) at the fence above.
            let limit = self
                .growth_limit_below(id, &r)
                .or_else(|| self.growth_limit_above(id, &r));
            let h = (steps.fixed + rows * steps.row).round() as i32;
            bottom = r.top + steps.height_up_to(h, &r, limit, w.row_at_bottom());
        }
        if (left, width, bottom) == (r.left, r.right - r.left, r.bottom) {
            return;
        }
        // A fence whose title row is at its bottom keeps that edge: the rows come and go at
        // the top.
        let (top, bottom) = if w.row_at_bottom() {
            (r.bottom - (bottom - r.top), r.bottom)
        } else {
            (r.top, bottom)
        };
        let rect = RECT {
            left,
            top,
            right: left + width,
            bottom,
        };
        w.set_bounds(rect);
        let expanded = if rolled {
            w.expanded_height_px()
        } else {
            let h = rect.bottom - rect.top;
            w.apply_height(h, false);
            h
        };
        self.record_bounds(id, rect, rolled, expanded);
        self.schedule_save();
    }

    /// 像桌面一样排列 (`Fence::desktop_layout`): hands the 「桌面」 fence the work area of its
    /// monitor and the other fences' rects; it lays its icons out down the columns around
    /// them and lies under every other fence. Called whenever a fence moves, resizes, comes or
    /// goes, and when the items change.
    pub(super) fn arrange_desktop_fence(&mut self) {
        let Some((id, on)) = self
            .state
            .fences()
            .iter()
            .find(|f| f.kind == FenceKind::Inbox && f.tab_host.is_none())
            .map(|f| (f.id, f.desktop_layout))
        else {
            return;
        };
        // A 「桌面」 that holds tabs stays a fence (its tab strip has nowhere to go).
        let on = on && self.state.tabs_of(id).len() <= 1;
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        if !on {
            if w.is_desktop_arranged() {
                w.set_desktop_area(None);
                // Back where the fence was before (its saved rect was kept meanwhile), out of the
                // way of fences moved there since.
                if let Some(f) = self.state.fence(id).cloned() {
                    self.place_from_state(&f);
                    self.move_out_of_overlap(id);
                }
            }
            return;
        }
        let r = w.rect();
        let Some(wa) = self.work_area_at((r.left + r.right) / 2, (r.top + r.bottom) / 2) else {
            return;
        };
        let work = RECT {
            left: wa.left,
            top: wa.top,
            right: wa.right,
            bottom: wa.bottom,
        };
        // Where the other fences rest: one hover-peeking open over the icons does not move them.
        // Windows not shown yet count too (startup creates them hidden), so the first arrange
        // is already the right one; a fence with no window (为空时自动隐藏) takes no cells.
        let covered: Vec<RECT> = self
            .fences
            .iter()
            .filter(|(fid, _)| **fid != id)
            .map(|(_, o)| o.resting_rect())
            .collect();
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        w.set_desktop_area(Some(crate::fence_window::DesktopArea { work, covered }));
        if let Some(a) = self.anchor.borrow_mut().as_mut() {
            a.lower_desktop_fence();
        }
    }

    pub(super) fn apply_auto_height(&mut self, id: FenceId) {
        let id = self.state.host_of(id);
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        // A rolled fence follows its tabs: collapsed to the left tab column, or back to a title
        // row at its expanded width.
        if let Some(rect) = w.settle_rolled_size() {
            let expanded = w.expanded_height_px();
            self.record_bounds(id, rect, true, expanded);
            self.schedule_save();
        }
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        // A fence without auto height still grows once to show a new left tab column (taken
        // on every pass, so a request left by an auto-height fence does not fire later).
        let rail = w.take_rail_growth_px();
        let Some(mut h) = w.auto_height_px().or(rail) else {
            return;
        };
        let r = w.rect();
        // Growing stops short of a fence below; the rest of the items scroll. One whose title
        // row is at its bottom keeps that edge and stops short of a fence above instead (while
        // it is expanded, the push pass has lifted those out of its way already).
        if let Some(limit) = self.growth_limit_below(id, &r) {
            h = h.min(limit.max(r.bottom) - r.top);
        }
        if let Some(limit) = self.growth_limit_above(id, &r) {
            h = h.min(r.bottom - limit.min(r.top));
        }
        if r.bottom - r.top == h {
            return;
        }
        // is_rolled() = rolled_up || peeking: during a hover peek this keeps f.rolled_up = true
        // (only the expanded height is recorded), so an auto-height pass can never persist a
        // transient peek as "expanded".
        let rolled = w.is_rolled();
        let rect = w.apply_height(h, true);
        self.record_bounds(id, rect, rolled, h);
        self.schedule_save();
    }

    pub(super) fn on_display_changed(&mut self) {
        self.end_peek_now();
        self.state.work_areas = work_areas();
        let layouts = self.state.config.layouts.len();
        self.state.ensure_layout();
        log_monitors(&self.state.work_areas);
        tracing::info!(
            monitors = self.state.work_areas.len(),
            new_setup = self.state.config.layouts.len() > layouts,
            layout = self.state.layout,
            fences_to_place = self.state.unplaced.len(),
            "display changed"
        );
        // A different monitor set may select a different layout: sync windows first.
        self.sync_fence_windows();
        for fence in self.state.fences().to_vec() {
            if self.fences.contains_key(&fence.id) {
                // State, not the window: sync_fence_windows has just made the window agree, and
                // a fence rolled in the old layout but expanded in the new one needs px.height().
                self.place_from_state(&fence);
            }
        }
        // Fences new to these monitors (or whose monitor is gone) move out of the others' way;
        // the rest stay where this layout saved them.
        self.settle_unplaced(false);
        for w in self.fences.values() {
            w.repaint_shadow();
        }
        // Wallpaper regions moved with the monitors: rebuild the backdrops unconditionally.
        self.refresh_visuals(true);
        // 像桌面一样排列: the new work area (and the fences placed again on it).
        self.arrange_desktop_fence();
    }

    pub fn toggle_all_fences(&mut self) {
        // 「显示所有栅栏」 while 「临时显示 Windows 桌面」 is on ends it (the icons go again).
        if self.windows_desktop {
            self.set_windows_desktop(false, "fences shown");
            return;
        }
        self.end_peek_now();
        if let Some(a) = self.anchor.borrow_mut().as_mut() {
            a.toggle_fences_hidden();
        }
    }

    /// Human label for a monitor: "显示器 1 (2560×1440)".
    pub(super) fn monitor_labels(&self) -> Vec<(String, String)> {
        self.state
            .work_areas
            .iter()
            .enumerate()
            .map(|(i, w)| {
                (
                    w.device_path.clone(),
                    pecofence_core::i18n::format(
                        "显示器 {0} ({1}×{2})",
                        &[
                            format!("{}", i + 1),
                            format!("{}", w.mon_right - w.mon_left),
                            format!("{}", w.mon_bottom - w.mon_top),
                        ],
                    ),
                )
            })
            .collect()
    }

    pub(super) fn swap_monitors(&mut self, a: &str, b: &str) {
        let n = self.state.swap_monitors(a, b);
        if n == 0 {
            self.settings_toast(pecofence_core::i18n::text("这两个显示器上没有栅栏"));
            return;
        }
        self.end_peek_now();
        self.relayout_from_state();
        self.schedule_save();
        self.settings_toast(&pecofence_core::i18n::format(
            "已在两个显示器之间交换 {0} 个栅栏",
            &[n.to_string()],
        ));
    }

    /// Puts `fence`'s window where the state has it: the saved rect, and rolled up its title row
    /// at the saved end of it, ready to open to its full height. The window takes the saved roll
    /// state and height at once, ending a roll or hover expansion still running toward the other
    /// state: one finishing later would roll the fence back up and record that, as after a
    /// `fence roll` just before a snapshot restore (feedback #76).
    fn place_from_state(&self, fence: &pecofence_core::Fence) {
        let Some(w) = self.fences.get(&fence.id) else {
            return;
        };
        let px = self.state.fence_px_rect(fence);
        let rect = RECT {
            left: px.left,
            top: px.top,
            right: px.right,
            bottom: px.bottom,
        };
        w.set_row_at_bottom(fence.bottom_anchored);
        w.restore_geometry(rect, fence.rolled_up, self.state.fence_expanded_px(fence));
    }

    /// Re-applies every fence's saved geometry to its window (after a swap / restore / import).
    pub(super) fn relayout_from_state(&mut self) {
        self.state.ensure_layout();
        self.resync_windows();
        for fence in self.state.host_fences() {
            if self.fences.contains_key(&fence.id) {
                self.place_from_state(&fence);
            }
            self.apply_fence_view(fence.id);
        }
        self.settle_unplaced(false);
        self.refresh_all();
        self.push_settings_state();
    }

    pub(super) fn fence_of_hwnd(&self, hwnd: HWND) -> Option<FenceId> {
        self.fences
            .iter()
            .find(|(_, w)| w.hwnd() == hwnd)
            .map(|(id, _)| *id)
    }

    /// Shows `fence`'s grid (icon size, label lines, spacing, alignment, layout) in its window,
    /// when it is the tab on show.
    pub(super) fn show_grid(&self, fence: FenceId) {
        if let Some(w) = self.window_for(fence)
            && w.active_fence() == fence
            && let Some(f) = self.state.fence(fence)
        {
            w.set_grid(GridSpec::of(&f.view));
        }
    }

    /// Menu, Ctrl+wheel, settings page and CLI: one place that changes a fence's icon size and
    /// everything derived from it (window redraw, cell snap, height fit, auto height, save).
    pub(super) fn apply_icon_size(&mut self, fence: FenceId, size: u32) {
        self.state.set_icon_size(fence, size);
        self.show_grid(fence);
        // The old size's extractions still queued would delay the new size's.
        self.drop_unneeded_icon_requests();
        self.apply_cell_snap(fence);
        self.fit_height_to_content(fence);
        self.apply_auto_height(fence);
        self.schedule_save();
    }

    /// Ctrl+wheel: 32 → 48 → 64 → 96, stopping at the ends. A size between them (from the
    /// settings slider or the CLI) steps to the next preset. See `ctrl_wheel_resizes` for a
    /// locked fence.
    pub(super) fn step_icon_size(&mut self, fence: FenceId, larger: bool) {
        let host = self.state.host_of(fence);
        let arranged = self
            .fences
            .get(&host)
            .is_some_and(|w| w.is_desktop_arranged());
        let locked = self.state.fence(host).is_some_and(|h| h.locked);
        if !ctrl_wheel_resizes(locked, arranged) {
            return;
        }
        let Some(cur) = self.state.fence(fence).map(|f| f.view.icon_size) else {
            return;
        };
        if let Some(next) = crate::layout::step_icon_preset(cur, larger) {
            self.apply_icon_size(fence, next);
            self.push_settings_state();
        }
    }

    pub(super) fn begin_rename(&mut self, fence: FenceId) {
        // Inline rename lands with the FocusSession task; until then use the tray notification
        // to explain. Kept as a queue command so the UI is already wired.
        if let Some(w) = self.window_for(fence) {
            w.set_title_renaming(true);
            crate::rename::begin_inline_rename(
                w.hwnd(),
                fence,
                self.state
                    .fence(fence)
                    .map(|f| f.title.clone())
                    .unwrap_or_default(),
                w.rail_row_screen_rect(fence),
                self.queue.clone(),
                self.theme_mode,
            );
        }
    }

    /// "快速添加" from the settings page: a template's fence beside the inbox plus its rule,
    /// applied to the desktop right away. A template added before just gets shown.
    pub(super) fn add_template(&mut self, template: pecofence_core::rules::Template) {
        // The inbox may be a tab: its window is the host's.
        let inbox = self.state.inbox_id().map(|id| self.state.host_of(id));
        let centre = |r: RECT| ((r.left + r.right) / 2, (r.top + r.bottom) / 2);
        let (x, y) = inbox
            .and_then(|id| self.fences.get(&id))
            .map(|w| centre(w.rect()))
            .or_else(|| {
                self.state
                    .work_areas
                    .first()
                    .map(|w| ((w.left + w.right) / 2, (w.top + w.bottom) / 2))
            })
            .unwrap_or((0, 0));
        if self.state.fence_limit_reached() {
            self.settings_toast(&fence_limit_text());
            return;
        }
        let rect = self.place_new_fence(3, 200.0, x, y, inbox);
        match self.state.add_template(template, rect) {
            Ok(id) => {
                self.resync_windows();
                if let Some(w) = self.fences.get(&id) {
                    w.show(true);
                }
                let entries = shell::enumerate_desktop();
                let moved = self.state.apply_rules_all(&entries);
                self.refresh_all();
                self.schedule_save();
                self.push_settings_state();
                self.settings_toast(&pecofence_core::i18n::format(
                    "已添加“{0}”，整理了 {1} 个项目",
                    &[template.title(), moved.to_string()],
                ));
            }
            Err(existing) => {
                self.settings_toast(&pecofence_core::i18n::format(
                    "“{0}”已存在",
                    &[template.title()],
                ));
                if let Some(id) = existing {
                    self.post_show_fence(id);
                }
            }
        }
    }

    /// Creates a virtual fence at `rect` (physical px); `title` defaults to the localized
    /// "新栅栏". Returns the new id (None when there is no monitor to place it on).
    pub(super) fn create_fence_at(&mut self, rect: RECT, title: Option<&str>) -> Option<FenceId> {
        let title = title
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| pecofence_core::i18n::text("新栅栏"));
        if !self.fence_room_or_notice() {
            return None;
        }
        let id = self.state.new_fence(title, rect)?;
        self.resync_windows();
        if let Some(w) = self.fences.get(&id) {
            w.show(true);
        }
        self.schedule_save();
        Some(id)
    }

    /// A fence made from a menu: its title opens for editing, as Explorer's 新建文件夹 does.
    pub(super) fn create_fence_with_rename(&mut self, rect: RECT) {
        if let Some(id) = self.create_fence_at(rect, None) {
            self.begin_rename(id);
        }
    }

    /// False (with a tray notice) when the layout already holds `MAX_FENCES`.
    pub(super) fn fence_room_or_notice(&self) -> bool {
        if !self.state.fence_limit_reached() {
            return true;
        }
        tracing::warn!("fence limit reached; new fence refused");
        if let Some(t) = &self.tray {
            t.show_info("PecoFence", &fence_limit_text(), true);
        }
        false
    }

    /// The fence menu's 删除栅栏: asks first when icons or rules would go with the fence.
    pub(super) fn delete_fence_from_menu(&mut self, fence: FenceId) {
        if let Some(prompt) = self.delete_fence_prompt(fence) {
            let owner = self
                .window_for(fence)
                .map(|w| w.hwnd())
                .unwrap_or(self.control.hwnd());
            if !window::confirm(owner, &prompt, pecofence_core::i18n::text("删除栅栏")) {
                return;
            }
        }
        self.delete_fence(fence);
    }

    /// What deleting `fence` takes with it, as a question; None when nothing would be lost (no
    /// icons of its own, no rule pointing at it). A folder portal's items are the folder's
    /// files and stay on disk, so only its rules count.
    fn delete_fence_prompt(&self, fence: FenceId) -> Option<String> {
        use pecofence_core::i18n;
        let f = self.state.fence(fence)?;
        let icons = if f.kind == FenceKind::FolderPortal {
            0
        } else {
            self.state.own_item_count(f)
        };
        let rules = self
            .state
            .config
            .rules
            .list
            .iter()
            .filter(|r| r.target == Target::Fence(fence))
            .count();
        if icons == 0 && rules == 0 {
            return None;
        }
        let mut text = i18n::format("要删除栅栏“{0}”吗？", std::slice::from_ref(&f.title));
        if icons > 0 {
            let inbox = self
                .state
                .inbox_id()
                .and_then(|id| self.state.fence(id))
                .map(|f| f.title.clone())
                .unwrap_or_default();
            text.push_str("\n\n");
            text.push_str(&i18n::format(
                "其中的 {0} 个图标会回到“{1}”栅栏，文件本身不受影响。",
                &[icons.to_string(), inbox],
            ));
        }
        if rules > 0 {
            text.push_str("\n\n");
            text.push_str(&i18n::format(
                "指向它的 {0} 条规则也会一并删除。",
                &[rules.to_string()],
            ));
        }
        Some(text)
    }

    /// Deletes a fence (or a tab); a deleted host's next tab takes its place and keeps the rest.
    pub(super) fn delete_fence(&mut self, fence: FenceId) {
        if self.state.delete_fence(fence) {
            if let Some(w) = self.fences.remove(&fence) {
                self.retire_window(w);
            }
            self.resync_windows();
            self.refresh_all();
            self.schedule_save();
        }
    }

    /// Fences "dock to the top of the screen": the fence moves flush against the top of its
    /// work area, rolls up and expands on hover — a pull-down shelf. Its title row is the top
    /// of it from now on (with 向上展开 there is no room above it either, so it opens downward).
    pub(super) fn dock_to_top(&mut self, fence: FenceId) {
        let Some(w) = self.fences.get(&fence) else {
            return;
        };
        w.set_row_at_bottom(false);
        let r = w.rect();
        let Some(wa) = self.work_area_at((r.left + r.right) / 2, (r.top + r.bottom) / 2) else {
            return;
        };
        let h = r.bottom - r.top;
        let rect = RECT {
            left: r
                .left
                .clamp(wa.left, (wa.right - (r.right - r.left)).max(wa.left)),
            top: wa.top,
            right: r
                .left
                .clamp(wa.left, (wa.right - (r.right - r.left)).max(wa.left))
                + (r.right - r.left),
            bottom: wa.top + h,
        };
        w.set_bounds(rect);
        let expanded = if w.is_rolled() {
            w.expanded_height_px()
        } else {
            h
        };
        let rolled = w.is_rolled();
        self.record_bounds(fence, rect, rolled, expanded);
        if !rolled {
            self.toggle_roll(fence);
        }
        self.schedule_save();
    }

    pub(super) fn new_fence_near(&mut self, x: i32, y: i32) {
        let rect = self.place_new_fence(3, 200.0, x, y, None);
        self.create_fence_with_rename(rect);
    }

    /// Work area (device px) containing the point, else the primary one.
    pub(super) fn work_area_at(&self, x: i32, y: i32) -> Option<WorkArea> {
        self.state
            .work_areas
            .iter()
            .find(|w| x >= w.left && x < w.right && y >= w.top && y < w.bottom)
            .or(self.state.work_areas.first())
            .cloned()
    }

    /// Picks a screen rectangle `cols` icon columns wide and `h_dip` tall for a new fence that
    /// does not overlap any existing fence: beside `near` (right, left, below, above) first,
    /// then a coarse scan of the work area, finally centred on the point. Result is clamped
    /// into the work area. The width is taken from the grid metrics, so a new fence shows
    /// exactly `cols` columns.
    pub(super) fn place_new_fence(
        &self,
        cols: u32,
        h_dip: f32,
        x: i32,
        y: i32,
        near: Option<FenceId>,
    ) -> RECT {
        let view = pecofence_core::FenceView::default();
        let w_dip = crate::layout::GridMetrics::for_icon_size(
            self.state.config.settings.icon_size,
            view.label_lines,
        )
        .width_for_columns(cols);
        self.place_new_fence_dip(w_dip, h_dip, x, y, near)
    }

    /// Does `r` lie inside `work` without overlapping an existing fence window (gap/2 slack)?
    pub(super) fn rect_free(&self, r: &RECT, work: &WorkArea) -> bool {
        let fits = r.left >= work.left
            && r.top >= work.top
            && r.right <= work.right
            && r.bottom <= work.bottom;
        fits && !self.overlaps_fence(r, (12.0 * work.scale()) as i32)
    }

    pub(super) fn overlaps_fence(&self, r: &RECT, gap: i32) -> bool {
        self.fences.values().any(|f| {
            let o = f.footprint_rect();
            r.left < o.right - gap / 2
                && r.right > o.left + gap / 2
                && r.top < o.bottom - gap / 2
                && r.bottom > o.top + gap / 2
        })
    }

    /// `place_new_fence` for an explicit DIP size.
    pub(super) fn place_new_fence_dip(
        &self,
        w_dip: f32,
        h_dip: f32,
        x: i32,
        y: i32,
        near: Option<FenceId>,
    ) -> RECT {
        let Some(work) = self.work_area_at(x, y) else {
            return RECT {
                left: x,
                top: y,
                right: x + w_dip as i32,
                bottom: y + h_dip as i32,
            };
        };
        let scale = work.scale();
        let (w, h) = ((w_dip * scale) as i32, (h_dip * scale) as i32);
        let gap = (12.0 * scale) as i32;
        let at = |left: i32, top: i32| RECT {
            left,
            top,
            right: left + w,
            bottom: top + h,
        };
        let mut candidates = Vec::new();
        if let Some(src) = near
            .and_then(|id| self.fences.get(&id))
            .map(|f| f.footprint_rect())
        {
            candidates.push(at(src.right + gap, src.top));
            candidates.push(at(src.left - gap - w, src.top));
            candidates.push(at(src.left, src.bottom + gap));
            candidates.push(at(src.left, src.top - gap - h));
        }
        candidates.push(at(x - w / 2, y - h / 2));
        if let Some(r) = candidates.iter().find(|r| self.rect_free(r, &work)) {
            return *r;
        }
        // Coarse scan, top-left to bottom-right.
        let step = (24.0 * scale) as i32;
        let mut top = work.top + gap;
        while top + h <= work.bottom {
            let mut left = work.left + gap;
            while left + w <= work.right {
                let r = at(left, top);
                if !self.overlaps_fence(&r, gap) {
                    return r;
                }
                left += step;
            }
            top += step;
        }
        // Everything is covered: centre on the point, clamped into the work area.
        let left = (x - w / 2).clamp(work.left, (work.right - w).max(work.left));
        let top = (y - h / 2).clamp(work.top, (work.bottom - h).max(work.top));
        at(left, top)
    }

    pub(super) fn toggle_roll(&mut self, fence: FenceId) {
        let fence = self.state.host_of(fence);
        if let Some(w) = self.fences.get(&fence) {
            let rolled = !w.is_rolled();
            self.set_roll(fence, rolled);
        }
    }

    /// Rolls up / expands the host window to an explicit state. Unlike [`Self::toggle_roll`]
    /// this does not read the window's animated flag, so two quick requests (the CLI) cannot
    /// undo each other while the 167 ms roll animation is still running.
    pub(super) fn set_roll(&mut self, fence: FenceId, rolled: bool) {
        let fence = self.state.host_of(fence);
        // 像桌面一样排列 has no title row to roll up to.
        if self
            .fences
            .get(&fence)
            .is_some_and(|w| w.is_desktop_arranged())
        {
            return;
        }
        // With pushNeighbors on the fences below make room instead.
        if !rolled
            && !self.state.config.settings.roll_up.push_neighbors
            && self.fences.get(&fence).is_some_and(|w| w.is_rolled())
        {
            self.clamp_expand_height(fence);
        }
        if let Some(w) = self.fences.get(&fence) {
            w.set_rolled(rolled);
            if let Some(f) = self.state.fence_mut(fence) {
                f.rolled_up = rolled;
            }
            self.state.mark_dirty();
            self.schedule_save();
        }
    }
}

fn fence_limit_text() -> String {
    pecofence_core::i18n::format(
        "最多只能有 {0} 个栅栏（含标签页），请先删除或合并一些。",
        &[pecofence_core::config_store::MAX_FENCES.to_string()],
    )
}

/// Whether Ctrl+wheel changes the icon size. A locked fence keeps its size: a stray Ctrl+wheel
/// would otherwise resize a fence locked against resizing. 像桌面一样排列 places its window
/// itself and has no lock, so its icons change size as the Windows desktop's do.
pub(super) fn ctrl_wheel_resizes(locked: bool, desktop_arranged: bool) -> bool {
    desktop_arranged || !locked
}
