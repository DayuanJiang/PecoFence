//! Tab strip model on the app side: tab view list, reorder, detach (tear-off), switch.

use super::*;

/// Where the cursor holds a fence torn off a tab strip, from the window's top-left corner: the
/// middle of its title row, at the top or the bottom of a window `w` x `h` px, or the top of its
/// title column.
pub(super) fn title_grip(
    edge: pecofence_core::TitleEdge,
    w: i32,
    h: i32,
    title_h: i32,
) -> (i32, i32) {
    match edge {
        pecofence_core::TitleEdge::Bottom => (w / 2, h - title_h / 2),
        pecofence_core::TitleEdge::Top => (w / 2, title_h / 2),
        pecofence_core::TitleEdge::Left => (title_h / 2, title_h / 2),
        pecofence_core::TitleEdge::Right => (w - title_h / 2, title_h / 2),
    }
}

impl App {
    /// Tab strip of a host window: the host first, then its hosted fences.
    pub(super) fn tab_views(&self, host: FenceId) -> Vec<TabView> {
        self.state
            .tabs_of(host)
            .into_iter()
            .filter_map(|id| self.state.fence(id))
            .map(|f| TabView {
                id: f.id,
                folder: self.state.portal_path(f.id),
                // A navigated portal's tab names the subfolder actually shown / dropped into.
                title: self.state.display_title(f),
                color: f.appearance.as_ref().and_then(|a| a.tab_rgb.or(a.tint_rgb)),
                title_size: fence_style_for(f).title_size,
                title_color: fence_style_for(f).title_color,
                // A regular fence showing a folder opened in it gets the folder icon too, so its
                // tab has the up arrow that takes it back to its own icons.
                folder_icon: !f.hide_title_icon
                    && (f.kind == FenceKind::FolderPortal || self.state.can_go_up(f.id)),
                up_button: self.state.can_go_up(f.id),
            })
            .collect()
    }

    /// Tab dragged along the strip / menu 左移 右移: persist the order and redraw the strip.
    pub(super) fn reorder_tab(&mut self, host: FenceId, tab: FenceId, to: usize) {
        let host = self.state.host_of(host);
        if self.state.reorder_tab(host, tab, to) {
            self.refresh_fence(host);
            self.schedule_save();
        }
    }

    /// Splits a tab out into its own window: back at its saved geometry when that still fits
    /// and is free (Fence.geometry is kept while it is a tab), else at its own size beside
    /// its former host / on free space. Torn off by dragging (`from_drag`): the new window
    /// appears under the pointer and keeps following it until the button is released, like a
    /// browser tab; dropping it on another fence's title merges it there.
    pub(super) fn detach_tab(&mut self, tab: FenceId, x: i32, y: i32, from_drag: bool) {
        let source = self.state.host_of(tab);
        if from_drag
            && self
                .fences
                .get(&source)
                .is_some_and(|w| w.take_cancelled_detach())
        {
            return;
        }
        if self.state.tabs_of(source).len() < 2 {
            return;
        }
        if let Some(window) = self.fences.get(&source) {
            let (rect, rolled, expanded) = (
                window.rect(),
                window.is_rolled(),
                window.expanded_height_px(),
            );
            self.record_bounds(source, rect, rolled, expanded);
        }
        let Some(change) = self.state.detach_tab(tab) else {
            return;
        };
        let host = change.remaining_host;
        let stored = self
            .state
            .fence(tab)
            .map(|f| (self.state.fence_px_rect(f), f.geometry.w, f.geometry.h));
        let rect = match stored {
            Some(_) | None if from_drag => {
                let work = self.work_area_at(x, y);
                let scale = work.as_ref().map(|w| w.scale()).unwrap_or(1.0);
                let (w, h) = match stored {
                    Some((_, w_dip, h_dip)) if w_dip > 0.0 && h_dip > 0.0 => {
                        ((w_dip * scale) as i32, (h_dip * scale) as i32)
                    }
                    _ => {
                        let r = self.place_new_fence(3, 200.0, x, y, Some(host));
                        (r.right - r.left, r.bottom - r.top)
                    }
                };
                let title_h = (36.0 * scale) as i32;
                let edge = self
                    .state
                    .fence(tab)
                    .map_or(pecofence_core::TitleEdge::Top, |f| f.title_edge());
                let (gx, gy) = title_grip(edge, w, h, title_h);
                let mut left = x - gx;
                let mut top = y - gy;
                if let Some(work) = &work {
                    left = left.clamp(work.left, (work.right - w).max(work.left));
                    top = top.clamp(work.top, (work.bottom - h).max(work.top));
                }
                RECT {
                    left,
                    top,
                    right: left + w,
                    bottom: top + h,
                }
            }
            Some((px, w_dip, h_dip)) => {
                let r = RECT {
                    left: px.left,
                    top: px.top,
                    right: px.right,
                    bottom: px.bottom,
                };
                let (cx, cy) = px.center();
                // work_area_at also rejects the 1920x1040 fallback fence_px_rect returns when
                // the geometry's monitor is gone (the rect then lies outside every work area).
                let free = self
                    .work_area_at(cx, cy)
                    .is_some_and(|work| self.rect_free(&r, &work));
                if free {
                    r
                } else {
                    self.place_new_fence_dip(w_dip, h_dip, x, y, Some(host))
                }
            }
            None => self.place_new_fence(3, 200.0, x, y, Some(host)),
        };
        let h = rect.bottom - rect.top;
        self.record_bounds(tab, rect, false, h);
        let started = Instant::now();
        self.resync_windows();
        let resynced_ms = started.elapsed().as_millis();
        self.apply_fence_view(host);
        self.apply_fence_view(tab);
        if let Some(w) = self.fences.get(&tab) {
            // A departing host already owns this HWND; it must move too, even when
            // the mouse was released before the queued detach command ran.
            w.restore_geometry(rect, false, h);
            // Fit at the destination (including its work-area limit). The source
            // window may have had another tab's content/position during resync.
            if let Some(fit) = w.auto_height_px() {
                w.restore_geometry(w.rect(), false, fit);
            }
            let rect = self.ctx.behavior.resting_rect(w.hwnd(), w.rect());
            self.state
                .set_fence_bounds(tab, rect, false, w.expanded_height_px(), false, false);
            self.queue.push(Command::RaiseFence(w.hwnd()));
            tracing::debug!(
                from_drag,
                resynced_ms,
                total_ms = started.elapsed().as_millis(),
                "detach_tab: window created"
            );
            if from_drag && window::key_down(msg::VK_LBUTTON) {
                // The cursor has moved on since DetachTab was queued: put the new window's
                // title centre under it NOW, then let the host (which still holds the mouse
                // capture) drive it from there (see begin_remote_drag).
                let new_hwnd = w.hwnd();
                let r = w.rect();
                let (rw, rh) = (r.right - r.left, r.bottom - r.top);
                let pt = window::cursor_pos();
                let scale = monitors::dpi_for_window(new_hwnd).max(96) as f32 / 96.0;
                let title_h = (36.0 * scale) as i32;
                let edge = self
                    .state
                    .fence(tab)
                    .map_or(pecofence_core::TitleEdge::Top, |f| f.title_edge());
                let (gx, gy) = title_grip(edge, rw, rh, title_h);
                let placed = RECT {
                    left: pt.x - gx,
                    top: pt.y - gy,
                    right: pt.x - gx + rw,
                    bottom: pt.y - gy + rh,
                };
                w.set_bounds(placed);
                // Keep capture on the original HWND. For a departing host this is
                // also the moving HWND; the handler's logical identity stays valid.
                if let Some(source_w) = self.fences.get(&change.source_host) {
                    source_w.begin_remote_drag(new_hwnd, change);
                    // Deterministic integration coverage while a real mouse drag is held.
                    // Release builds never synthesize input; the message follows the same
                    // Escape/right-button handler as a user's cancellation.
                    #[cfg(debug_assertions)]
                    if pecofence_core::brand::var_os("PECOFENCE_UI_TEST_WINDOWS").is_some()
                        && let Ok(cancel) =
                            pecofence_core::brand::var("PECOFENCE_TEST_CANCEL_DETACH")
                        && matches!(cancel.as_str(), "escape" | "right")
                    {
                        match cancel.as_str() {
                            "escape" => window::post_message(
                                source_w.hwnd(),
                                msg::WM_KEYDOWN,
                                msg::VK_ESCAPE as usize,
                                0,
                            ),
                            "right" => {
                                window::post_message(source_w.hwnd(), msg::WM_RBUTTONDOWN, 0, 0)
                            }
                            _ => {}
                        }
                        tracing::debug!(%cancel, "test cancellation posted after detach");
                    }
                }
            }
        }
        self.schedule_save();
    }

    pub(super) fn cancel_tab_detach(&mut self, change: pecofence_core::TabDetach) {
        for window in self.fences.values() {
            window.set_merge_hint(false, (0, 0));
        }
        if !self.state.cancel_tab_detach(&change) {
            return;
        }
        self.resync_windows();
        self.apply_fence_view(change.source_host);
        if let (Some(fence), Some(window)) = (
            self.state.fence(change.source_host),
            self.fences.get(&change.source_host),
        ) {
            let px = self.state.fence_px_rect(fence);
            // A bottom row or title column rolled up keeps the full height its saved rect was
            // cut from.
            let expanded = if fence.title_edge() == pecofence_core::TitleEdge::Top {
                px.height()
            } else {
                self.state.fence_expanded_px(fence)
            };
            window.restore_geometry(
                RECT {
                    left: px.left,
                    top: px.top,
                    right: px.right,
                    bottom: px.bottom,
                },
                fence.rolled_up,
                expanded,
            );
            if fence.title_edge().is_side() {
                window.set_expanded_width_px(self.state.fence_expanded_w_px(fence));
            }
            self.queue.push(Command::RaiseFence(window.hwnd()));
        }
        self.schedule_save();
    }

    /// Shows `tab`'s items in its host window.
    pub(super) fn switch_tab(&mut self, host: FenceId, tab: FenceId) {
        let host = self.state.host_of(host);
        if !self.state.tabs_of(host).contains(&tab) {
            return;
        }
        self.state.set_active_tab(host, tab);
        self.refresh_fence(tab);
        // The host's frame belongs to all tabs. Snapping it to each tab's different row
        // height made repeated Icons/Details switches grow the window and lose scroll.
        self.apply_fence_view(host);
        self.schedule_save();
    }
}
