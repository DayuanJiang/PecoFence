//! Fence windows and geometry: create / sync / resync windows, bounds, column snap, auto height,
//! display changes, monitors, new-fence placement, roll, title rename, hwnd lookup.

use super::*;

pub(super) fn work_areas() -> Vec<WorkArea> {
    monitors::enumerate()
        .into_iter()
        .map(|m| WorkArea {
            device_path: m.device_name.clone(),
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

impl App {
    /// Creates/destroys/updates fence windows to match the state.
    pub(super) fn sync_fence_windows(&mut self) {
        // Portal folders are re-read by their watchers / on_fs_changed / startup, not here:
        // enumerating every portal (shell display names per entry) made a tab tear-off take
        // hundreds of milliseconds before the new window appeared.
        self.ensure_portal_watchers();
        self.state.normalize_tabs();
        // Only host fences own a window; fences hosted as tabs live inside their host's.
        let hosts = self.state.host_fences();
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
                self.apply_fence_view_with_snap(fence.id, false);
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
                shown.view.icon_size,
                shown.view.label_lines,
                fence.view.auto_height,
                rect,
                px.height(),
                items,
            ) {
                Ok(w) => {
                    w.set_tabs(tabs, active);
                    self.fences.insert(fence.id, w);
                    // Loading/synchronizing a saved fence must preserve its rectangle.
                    // User resizing and explicit icon/spacing changes still snap normally.
                    self.apply_fence_view_with_snap(fence.id, false);
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
        for w in self.fences.values() {
            if !before.contains(&w.hwnd()) {
                w.show(true);
            }
        }
        self.push_settings_state();
    }

    pub(super) fn refresh_fence(&mut self, id: FenceId) {
        let host = self.state.host_of(id);
        let active = self.state.active_tab_of(host);
        if let Some(w) = self.fences.get(&host) {
            w.set_tabs(self.tab_views(host), active);
            if active == id
                && let Some(f) = self.state.fence(id)
            {
                // Grouping first: the layout glide must target the new sections.
                w.set_group_by_date(f.view.group_by_date);
                w.set_items(self.item_views(f));
                w.set_sort_indicator(f.view.sort, f.view.reverse);
            }
        }
        self.apply_portal_deco(host);
        self.apply_auto_height(host);
    }

    /// Pushes the persisted per-fence flags (lock, quick-hide exclusion, appearance override)
    /// into the window and the anchor.
    pub(super) fn apply_fence_appearance(&mut self, id: FenceId) {
        let id = self.state.host_of(id);
        let Some(f) = self.state.fence(id).cloned() else {
            return;
        };
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        w.set_locked(f.locked);
        let (bd, op) = f
            .appearance
            .as_ref()
            .map(|a| (a.backdrop.map(backdrop_mode_for), a.opacity.unwrap_or(1.0)))
            .unwrap_or((None, 1.0));
        w.set_appearance(bd, op);
        w.set_style(fence_style_for(&f));
        if let Some(a) = self.anchor.borrow_mut().as_mut() {
            a.set_quick_hide_excluded(w.hwnd(), f.exclude_from_quick_hide);
        }
    }

    /// Pushes every persisted per-fence view flag (icon size, auto height, lock/appearance,
    /// rolled state) into an existing window and re-applies the derived geometry rules. This is
    /// the one place that makes a window agree with its `Fence`, whatever path changed the
    /// state (startup, layout switch, display change).
    pub(super) fn apply_fence_view(&mut self, id: FenceId) {
        self.apply_fence_view_with_snap(id, true);
    }

    pub(super) fn apply_fence_view_with_snap(&mut self, id: FenceId, snap_geometry: bool) {
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
            if w.icon_size() != shown.view.icon_size {
                // Drops cached icons/labels, so only when it actually differs.
                w.set_icon_size(shown.view.icon_size);
            }
            w.set_layout(shown.view.layout);
            w.set_is_inbox(shown.kind == FenceKind::Inbox);
            w.set_spacing(shown.view.spacing);
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
        if snap_geometry {
            self.apply_column_snap(id);
        }
        self.apply_auto_height(id);
    }

    pub(super) fn refresh_all(&mut self) {
        let ids: Vec<FenceId> = self.fences.keys().copied().collect();
        for id in ids {
            let active = self.state.active_tab_of(id);
            if let Some(f) = self.state.fence(active)
                && let Some(w) = self.fences.get(&id)
            {
                w.set_tabs(self.tab_views(id), active);
                w.set_group_by_date(f.view.group_by_date);
                w.set_items(self.item_views(f));
            }
            self.apply_auto_height(id);
        }
        // Cut items that were moved away (Explorer pasted them) stop being dimmed.
        let before = self.cut_items.len();
        self.cut_items.retain(|id| self.state.item(*id).is_some());
        if self.cut_items.len() != before {
            self.push_cut_items();
        }
    }

    /// Auto-height (task 14): make the window exactly as tall as its grid needs.
    /// Column snapping (Fences): the width is a whole number of icon columns. Interactive
    /// resizing snaps in WM_SIZING; this applies the same rule to loaded fences, icon-size
    /// changes and DPI drift so a fence never sits at an unaligned width.
    pub(super) fn apply_column_snap(&mut self, id: FenceId) {
        let id = self.state.host_of(id);
        let Some(f) = self.state.fence(id).cloned() else {
            return;
        };
        let shown = self
            .state
            .fence(self.state.active_tab_of(id))
            .cloned()
            .unwrap_or_else(|| f.clone());
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        let r = w.rect();
        let scale = monitors::dpi_for_window(w.hwnd()).max(96) as f32 / 96.0;
        let metrics =
            crate::layout::GridMetrics::for_icon_size(shown.view.icon_size, shown.view.label_lines)
                .with_line_h(self.ctx.chrome.label_line_h())
                .with_spacing(shown.view.spacing);
        let rows_layout = match shown.view.layout {
            ViewLayout::Icons => None,
            ViewLayout::List => Some(crate::layout::RowMetrics::list()),
            ViewLayout::Details => Some(crate::layout::RowMetrics::details()),
        };
        let cell = metrics.cell_w * scale;
        let pad = metrics.pad_x * 2.0 * scale;
        let width = (r.right - r.left) as f32;
        let cols = ((width - pad) / cell).round().max(1.0);
        // List / Details rows have no column rhythm: the width stays as the user left it.
        let mut snapped = if rows_layout.is_some() {
            r.right - r.left
        } else {
            (pad + cols * cell).round() as i32
        };
        // The snap itself must never push a fence past the work-area edge: shift it left, and
        // if it would then leave the left edge instead, round the column count down.
        let (cx, cy) = ((r.left + r.right) / 2, (r.top + r.bottom) / 2);
        let mut left = r.left;
        if let Some(wa) = self.work_area_at(cx, cy)
            && left + snapped > wa.right
        {
            left = (wa.right - snapped).max(wa.left);
            if left + snapped > wa.right && cols >= 2.0 {
                snapped = (pad + (cols - 1.0) * cell).round() as i32;
                left = wa.right - snapped;
            }
        }
        // Height: whole rows too (auto-height fences already get an exact row count; rolled
        // fences keep their title-only height).
        let rolled = w.is_rolled();
        let mut bottom = r.bottom;
        if !rolled && !f.view.auto_height {
            let title_h = (self.ctx.theme.borrow().title_height * scale).round();
            let (row, fixed) = match rows_layout {
                Some(rm) => (
                    rm.row_h * scale,
                    title_h + (rm.header_h + rm.pad_y * 2.0) * scale + 2.0,
                ),
                None => (
                    metrics.cell_h * scale,
                    title_h + metrics.pad_y * 2.0 * scale + 2.0,
                ),
            };
            let h = (r.bottom - r.top) as f32;
            let rows = ((h - fixed) / row).round().max(1.0);
            bottom = r.top + (fixed + rows * row).round() as i32;
        }
        if snapped == r.right - r.left && left == r.left && bottom == r.bottom {
            return;
        }
        let rect = RECT {
            left,
            top: r.top,
            right: left + snapped,
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
        self.state.set_fence_bounds(id, rect, rolled, expanded);
        self.schedule_save();
    }

    pub(super) fn apply_auto_height(&mut self, id: FenceId) {
        let id = self.state.host_of(id);
        let Some(w) = self.fences.get(&id) else {
            return;
        };
        let Some(h) = w.auto_height_px() else { return };
        let r = w.rect();
        if r.bottom - r.top == h {
            return;
        }
        // is_rolled() = rolled_up || peeking: during a hover peek this keeps f.rolled_up = true
        // (only the expanded height is recorded), so an auto-height pass can never persist a
        // transient peek as "expanded".
        let rolled = w.is_rolled();
        let rect = w.apply_height(h, true);
        self.state.set_fence_bounds(id, rect, rolled, h);
        self.schedule_save();
    }

    pub(super) fn on_display_changed(&mut self) {
        self.end_peek_now();
        self.state.work_areas = work_areas();
        self.state.ensure_layout();
        // A different monitor set may select a different layout: sync windows first.
        self.sync_fence_windows();
        for fence in self.state.fences().to_vec() {
            if let Some(w) = self.fences.get(&fence.id) {
                let px = self.state.fence_px_rect(&fence);
                // State, not the window: sync_fence_windows has just made the window agree, and
                // a fence rolled in the old layout but expanded in the new one needs px.height().
                let h = if fence.rolled_up {
                    window::window_rect_size(w.hwnd()).1
                } else {
                    px.height()
                };
                w.set_bounds(RECT {
                    left: px.left,
                    top: px.top,
                    right: px.right,
                    bottom: px.top + h,
                });
            }
        }
        // Wallpaper regions moved with the monitors: rebuild the backdrops unconditionally.
        self.refresh_visuals(true);
    }

    pub fn toggle_all_fences(&mut self) {
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
            "已交换 {0} 个栅栏的显示器",
            &[n.to_string()],
        ));
    }

    /// Re-applies every fence's saved geometry to its window (after a swap / restore / import).
    pub(super) fn relayout_from_state(&mut self) {
        self.state.ensure_layout();
        self.resync_windows();
        for fence in self.state.host_fences() {
            if let Some(w) = self.fences.get(&fence.id) {
                let px = self.state.fence_px_rect(&fence);
                let h = if fence.rolled_up {
                    window::window_rect_size(w.hwnd()).1
                } else {
                    px.height()
                };
                w.set_bounds(RECT {
                    left: px.left,
                    top: px.top,
                    right: px.right,
                    bottom: px.top + h,
                });
            }
            self.apply_fence_view(fence.id);
        }
        self.refresh_all();
        self.push_settings_state();
    }

    pub(super) fn fence_of_hwnd(&self, hwnd: HWND) -> Option<FenceId> {
        self.fences
            .iter()
            .find(|(_, w)| w.hwnd() == hwnd)
            .map(|(id, _)| *id)
    }

    /// Menu / Ctrl+wheel: one place that changes a fence's icon size and everything derived
    /// from it (window redraw, column snap, auto height, save).
    pub(super) fn apply_icon_size(&mut self, fence: FenceId, size: u32) {
        self.state.set_icon_size(fence, size);
        if let Some(w) = self.window_for(fence)
            && w.active_fence() == fence
        {
            w.set_icon_size(size);
        }
        self.apply_column_snap(fence);
        self.apply_auto_height(fence);
        self.schedule_save();
    }

    /// Ctrl+wheel: 32 → 48 → 64 → 96, stopping at the ends. 32 / 48 / 96 are the desktop's
    /// small / medium / large; 64 stays as an intermediate step for existing configs.
    pub(super) fn step_icon_size(&mut self, fence: FenceId, larger: bool) {
        let Some(cur) = self.state.fence(fence).map(|f| f.view.icon_size) else {
            return;
        };
        const SIZES: [u32; 4] = [32, 48, 64, 96];
        let next = if larger {
            SIZES.iter().copied().find(|s| *s > cur)
        } else {
            SIZES.iter().rev().copied().find(|s| *s < cur)
        };
        if let Some(next) = next {
            self.apply_icon_size(fence, next);
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

    pub(super) fn create_fence_at(&mut self, rect: RECT) {
        if let Some(id) = self
            .state
            .new_fence(pecofence_core::i18n::text("新栅栏"), rect)
        {
            self.resync_windows();
            if let Some(w) = self.fences.get(&id) {
                w.show(true);
            }
            self.schedule_save();
        }
    }

    /// Deletes a fence (or a tab); a host's tabs become windows of their own again.
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
    /// work area, rolls up and expands on hover — a pull-down shelf.
    pub(super) fn dock_to_top(&mut self, fence: FenceId) {
        let Some(w) = self.fences.get(&fence) else {
            return;
        };
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
        self.state
            .set_fence_bounds(fence, rect, w.is_rolled(), expanded);
        if !w.is_rolled() {
            self.toggle_roll(fence);
        }
        self.schedule_save();
    }

    pub(super) fn new_fence_near(&mut self, x: i32, y: i32) {
        let rect = self.place_new_fence(3, 200.0, x, y, None);
        self.create_fence_at(rect);
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
    /// into the work area. The width is taken from the grid metrics so the column snap that
    /// runs right after creation is a no-op (otherwise it would grow the fence into a
    /// neighbour or past the work-area edge that was just checked here).
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
            let o = f.rect();
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
        if let Some(src) = near.and_then(|id| self.fences.get(&id)).map(|f| f.rect()) {
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
            w.set_rolled(rolled);
            if let Some(f) = self.state.fence_mut(fence) {
                f.rolled_up = rolled;
            }
            self.state.mark_dirty();
            self.schedule_save();
        }
    }
}
