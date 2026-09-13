//! Tab strip model on the app side: tab view list, reorder, detach (tear-off), switch.

use super::*;

impl App {
    /// Tab strip of a host window: the host first, then its hosted fences.
    pub(super) fn tab_views(&self, host: FenceId) -> Vec<TabView> {
        self.state
            .tabs_of(host)
            .into_iter()
            .filter_map(|id| self.state.fence(id))
            .map(|f| TabView {
                id: f.id,
                // A navigated portal's tab names the subfolder actually shown / dropped into.
                title: self.state.display_title(f),
                color: f.appearance.as_ref().and_then(|a| a.tint_rgb),
                title_size: fence_style_for(f).title_size,
                title_color: fence_style_for(f).title_color,
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
            self.state.set_fence_bounds(
                source,
                window.rect(),
                window.is_rolled(),
                window.expanded_height_px(),
            );
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
                let mut left = x - w / 2;
                let mut top = y - title_h / 2;
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
        self.state.set_fence_bounds(tab, rect, false, h);
        let started = Instant::now();
        self.resync_windows();
        let resynced_ms = started.elapsed().as_millis();
        self.apply_fence_view_with_snap(host, false);
        self.apply_fence_view_with_snap(tab, false);
        if let Some(w) = self.fences.get(&tab) {
            // A departing host already owns this HWND; it must move too, even when
            // the mouse was released before the queued detach command ran.
            w.restore_geometry(rect, false, h);
            // Fit at the destination (including its work-area limit). The source
            // window may have had another tab's content/position during resync.
            if let Some(fit) = w.auto_height_px() {
                w.restore_geometry(w.rect(), false, fit);
            }
            self.state
                .set_fence_bounds(tab, w.rect(), false, w.expanded_height_px());
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
                let placed = RECT {
                    left: pt.x - rw / 2,
                    top: pt.y - title_h / 2,
                    right: pt.x - rw / 2 + rw,
                    bottom: pt.y - title_h / 2 + rh,
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
            window.set_merge_hint(false, 0);
        }
        if !self.state.cancel_tab_detach(&change) {
            return;
        }
        self.resync_windows();
        self.apply_fence_view_with_snap(change.source_host, false);
        if let (Some(fence), Some(window)) = (
            self.state.fence(change.source_host),
            self.fences.get(&change.source_host),
        ) {
            let px = self.state.fence_px_rect(fence);
            window.restore_geometry(
                RECT {
                    left: px.left,
                    top: px.top,
                    right: px.right,
                    bottom: px.bottom,
                },
                fence.rolled_up,
                px.height(),
            );
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
        self.apply_fence_view_with_snap(host, false);
        self.schedule_save();
    }
}
