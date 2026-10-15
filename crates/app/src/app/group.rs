//! Fences selected together by drawing a rectangle on the empty desktop: every fence the
//! marquee touches is selected as it is drawn (Explorer's rubber band), Ctrl or Shift adds to
//! the selection, and a click on the desktop or on any fence ends it. Selected fences wear a
//! white ring, like the other drag feedback; a title drag of one of them moves them all
//! (`fence_window::group`). A marquee that touches no fence and selects no Windows icon still
//! offers a new fence there.

use super::*;
use crate::anchor::{MARQUEE_MIN_PX, MarqueeEvent};

impl App {
    pub(super) fn on_desktop_marquee(&mut self, event: MarqueeEvent) {
        match event {
            MarqueeEvent::Pressed { additive } => {
                let kept = if additive {
                    self.ctx.behavior.selection.borrow().clone()
                } else {
                    Vec::new()
                };
                self.set_fence_selection(kept.clone());
                self.marquee_base = Some(kept);
                self.hide_marquee_band();
                self.marquee_draws_band = pecofence_platform::shell_icons::desktop_icons_hidden();
            }
            MarqueeEvent::Moved(rect) => {
                let selection = self.marquee_selection(rect);
                self.set_fence_selection(selection);
                if self.marquee_draws_band {
                    self.show_marquee_band(rect);
                }
            }
            MarqueeEvent::Released(rect) => {
                self.hide_marquee_band();
                let touched = !self.fences_touching(rect).is_empty();
                let selection = self.marquee_selection(rect);
                self.marquee_base = None;
                let count = selection.len();
                self.set_fence_selection(selection);
                if touched {
                    tracing::info!(count, "fences selected on the desktop");
                } else if rect.right - rect.left >= MARQUEE_MIN_PX
                    && rect.bottom - rect.top >= MARQUEE_MIN_PX
                    && !self.desktop_icons_selected()
                {
                    self.offer_new_fence(rect);
                }
            }
            MarqueeEvent::Cancelled => {
                self.hide_marquee_band();
                self.marquee_base = None;
            }
        }
    }

    /// The Windows icons show and Explorer's own marquee selected some of them: the drag was
    /// for those files, so the new-fence offer stays out of the way of their menu.
    fn desktop_icons_selected(&self) -> bool {
        if pecofence_platform::shell_icons::desktop_icons_hidden() {
            return false;
        }
        let def_view = self
            .anchor
            .borrow()
            .as_ref()
            .and_then(|a| a.host())
            .map(|h| h.def_view);
        let selected = def_view
            .and_then(pecofence_platform::shell_icons::desktop_selected_count)
            .unwrap_or(0);
        if selected > 0 {
            tracing::info!(
                selected,
                "desktop marquee selected icons; no new fence offer"
            );
        }
        selected > 0
    }

    fn show_marquee_band(&mut self, rect: RECT) {
        if self.marquee_band.is_none() {
            match crate::marquee_band::MarqueeBand::create() {
                Ok(band) => self.marquee_band = Some(band),
                Err(e) => {
                    tracing::warn!(error = %e, "marquee band unavailable");
                    self.marquee_draws_band = false;
                    return;
                }
            }
        }
        let host = self
            .anchor
            .borrow()
            .as_ref()
            .and_then(|a| a.host())
            .map(|h| h.host);
        let dpi = monitors::monitor_from_point(rect.left, rect.top);
        let dpi = monitors::query(dpi).map_or(monitors::system_dpi(), |m| m.dpi);
        if let Some(band) = self.marquee_band.as_mut() {
            band.show(rect, dpi, host);
        }
    }

    fn hide_marquee_band(&mut self) {
        if let Some(band) = self.marquee_band.as_mut() {
            band.hide();
        }
    }

    /// The marquee band's rect while it shows (test dumps).
    pub(super) fn marquee_band_shown(&self) -> Option<RECT> {
        self.marquee_band.as_ref().and_then(|b| b.shown())
    }

    /// The selection a marquee over `rect` stands for: what a Ctrl / Shift marquee kept, plus
    /// every fence it touches.
    fn marquee_selection(&self, rect: RECT) -> Vec<(HWND, FenceId)> {
        let mut selection = self.marquee_base.clone().unwrap_or_default();
        for hit in self.fences_touching(rect) {
            if !selection.contains(&hit) {
                selection.push(hit);
            }
        }
        selection
    }

    /// Showing, unlocked fence windows that `rect` (screen px) overlaps.
    fn fences_touching(&self, rect: RECT) -> Vec<(HWND, FenceId)> {
        let mut hits: Vec<(HWND, FenceId)> = self
            .fences
            .iter()
            .filter(|(id, w)| {
                let r = w.rect();
                desktop::is_visible(w.hwnd())
                    && !self.state.fence(**id).is_some_and(|f| f.locked)
                    && r.left < rect.right
                    && rect.left < r.right
                    && r.top < rect.bottom
                    && rect.top < r.bottom
            })
            .map(|(id, w)| (w.hwnd(), *id))
            .collect();
        // Stable order (the map's is not): top to bottom, then left to right.
        hits.sort_by_key(|(h, _)| {
            let r = window::window_rect(*h);
            (r.top, r.left)
        });
        hits
    }

    /// Replaces the selection, updating the rings of the fences that joined or left it.
    pub(super) fn set_fence_selection(&mut self, selection: Vec<(HWND, FenceId)>) {
        let old = self.ctx.behavior.selection.replace(selection.clone());
        if old == selection {
            return;
        }
        for (hwnd, id) in &old {
            if !selection.contains(&(*hwnd, *id))
                && self.settings_ring != Some(*id)
                && let Some(w) = self.fences.get(id)
                && w.hwnd() == *hwnd
            {
                w.set_group_selected(false);
            }
        }
        for (hwnd, id) in &selection {
            if !old.contains(&(*hwnd, *id))
                && let Some(w) = self.fences.get(id)
                && w.hwnd() == *hwnd
            {
                w.set_group_selected(true);
            }
        }
    }

    /// Settings › 「栅栏」 rings the fence it edits (`host`, None off the page or when Settings
    /// closes) without joining the desktop selection that group drags move.
    pub(super) fn set_settings_ring(&mut self, host: Option<FenceId>) {
        let selected = self.ctx.behavior.selection.borrow().clone();
        if let Some(old) = self.settings_ring.take()
            && host != Some(old)
            && !selected.iter().any(|(_, id)| *id == old)
            && let Some(w) = self.fences.get(&old)
        {
            w.set_group_selected(false);
        }
        if let Some(w) = host.and_then(|h| self.fences.get(&h)) {
            w.set_group_selected(true);
            self.settings_ring = host;
        }
    }

    /// A fence window going away leaves the selection.
    pub(super) fn forget_selected_window(&mut self, hwnd: HWND) {
        self.ctx
            .behavior
            .selection
            .borrow_mut()
            .retain(|(h, _)| *h != hwnd);
        if let Some(base) = self.marquee_base.as_mut() {
            base.retain(|(h, _)| *h != hwnd);
        }
    }
}
