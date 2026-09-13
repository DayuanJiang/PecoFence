//! Selection, keyboard cursor, type-ahead, marquee, press / hover bookkeeping, activation colour state and the slow-double-click rename.

use super::*;

/// A press on an item that may become a drag (OLE drag once the threshold is crossed).
pub(super) struct DragState {
    pub(super) start: (i32, i32),
    pub(super) item: usize,
    /// Plain press on an already-selected item of a multi-selection: a release that never
    /// became a drag collapses the selection to this item (Explorer).
    pub(super) collapse_on_up: bool,
    /// The pressed item was the sole selected item before the press: a release on its label
    /// arms the slow-double-click rename.
    pub(super) was_sole_selected: bool,
}

/// The control the mouse button is held on. Items and tabs act on press (selection / switch,
/// as in Explorer); the up button, the roll-up chevron and header cells are buttons that fire
/// on release inside the same control and show the WinUI pressed fill meanwhile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PressTarget {
    Item(usize),
    Tab(usize),
    Header(DetailColumn),
    Up,
    Chevron,
}

/// How a keyboard navigation key moves the cursor (Explorer): plain = select the target,
/// Shift = extend the range from the anchor, Ctrl = move the focus only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CursorMode {
    Select,
    Extend,
    FocusOnly,
}

pub(super) fn cursor_mode(shift: bool, ctrl: bool) -> CursorMode {
    if shift {
        CursorMode::Extend
    } else if ctrl {
        CursorMode::FocusOnly
    } else {
        CursorMode::Select
    }
}

/// Apply a keyboard move without involving rendering. A Shift range keeps its original
/// anchor even when it began after a focus-only move into an unselected view.
pub(super) fn apply_cursor_selection(
    selected: &mut HashSet<usize>,
    anchor: &mut Option<usize>,
    range_anchor: &mut Option<usize>,
    next: usize,
    mode: CursorMode,
) {
    match mode {
        CursorMode::Extend => {
            let from = range_anchor.or(*anchor).unwrap_or(next);
            *range_anchor = Some(from);
            *selected = (from.min(next)..=from.max(next)).collect();
        }
        CursorMode::FocusOnly => {}
        CursorMode::Select => {
            selected.clear();
            selected.insert(next);
            *range_anchor = Some(next);
        }
    }
    *anchor = Some(next);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MarqueeMode {
    Replace,
    /// Shift: add to the existing selection.
    Add,
    /// Ctrl: toggle the banded items.
    Toggle,
}

/// Rubber-band selection inside one fence (plan §5.9: MVP marquee is per fence).
pub(super) struct MarqueeState {
    /// Window px at press; anchored to the content via `start_scroll`.
    pub(super) start: (i32, i32),
    pub(super) start_scroll: f32,
    pub(super) cur: (i32, i32),
    /// Last raw pointer position (capture is held, so moves arrive while outside the window);
    /// `cur` is clamped to the item band while auto-scrolling, this one is not.
    pub(super) pointer: (i32, i32),
    /// Previous auto-scroll frame while the pointer is outside the item area (None = not
    /// auto-scrolling).
    pub(super) auto_last: Option<Instant>,
    /// Selection when the drag began (Shift / Ctrl modes).
    pub(super) base: HashSet<usize>,
    pub(super) mode: MarqueeMode,
}

impl FenceViewState {
    /// Moves the keyboard cursor to `next`: selecting it, extending the range from the anchor
    /// (Shift) or moving the focus alone (Ctrl); shows the focus ring and scrolls into view.
    pub(super) fn cursor_to(&mut self, next: usize, mode: CursorMode) {
        apply_cursor_selection(
            &mut self.selected,
            &mut self.anchor_index,
            &mut self.range_anchor,
            next,
            mode,
        );
        self.focus_visible = true;
        self.scroll_into_view(next, true);
    }

    /// Ctrl+Space: toggles the focused item's membership in the selection.
    pub(super) fn toggle_focused(&mut self) {
        let Some(a) = self.anchor_index.filter(|a| *a < self.items.len()) else {
            return;
        };
        if !self.selected.remove(&a) {
            self.selected.insert(a);
        }
        self.range_anchor = Some(a);
        self.focus_visible = true;
    }

    /// Moves the selection anchor by (dx, dy) cells (see [`CursorMode`]).
    pub(super) fn move_cursor(&mut self, dx: i32, dy: i32, mode: CursorMode) {
        if self.items.is_empty() {
            return;
        }
        let scale = self.scale();
        let (cw, _) = self.content_size_px();
        let cols = self.layout(cw as f32 / scale).columns().max(1) as i32;
        let n = self.items.len() as i32;
        let cur = match self.anchor_index {
            Some(i) => i as i32,
            None => {
                self.cursor_to(0, mode);
                return;
            }
        };
        let mut next = cur + dx + dy * cols;
        if dy != 0 && (next < 0 || next >= n) {
            next = cur; // no wrap between rows
        }
        let next = next.clamp(0, n - 1) as usize;
        self.cursor_to(next, mode);
    }

    pub(super) fn select_only(&mut self, index: usize) {
        self.selected.clear();
        self.selected.insert(index);
        self.anchor_index = Some(index);
        self.range_anchor = Some(index);
    }

    /// Ends a marquee (button release, capture loss, Esc), returning whether one was running.
    /// Releasing while the band was still edge-auto-scrolling leaves the offset between
    /// pixels, so it snaps here; the caller's redraw paints the snapped offset.
    pub(super) fn end_marquee(&mut self) -> bool {
        let had = self.marquee.take().is_some();
        if had {
            self.snap_scroll();
        }
        had
    }

    /// PageUp / PageDown: moves the anchor one viewport of rows (same column), scrolling a
    /// whole page; Shift extends the range, Ctrl moves the focus only. Clamps to the first /
    /// last item.
    pub(super) fn page_cursor(&mut self, dir: i32, mode: CursorMode) {
        if self.items.is_empty() {
            return;
        }
        let scale = self.scale();
        let (cw, ch) = self.content_size_px();
        let layout = self.layout(cw as f32 / scale);
        let cols = layout.columns().max(1) as i32;
        let view_h = ch as f32 / scale - layout.fixed_top();
        let rows_per_page = ((view_h / layout.row_step()).floor() as i32).max(1);
        let n = self.items.len() as i32;
        let cur = match self.anchor_index {
            Some(i) => i as i32,
            None => {
                let t = if dir > 0 { 0 } else { (n - 1) as usize };
                self.cursor_to(t, mode);
                return;
            }
        };
        let target = (cur + dir * rows_per_page * cols).clamp(0, n - 1) as usize;
        // One viewport of rows, animated; `cursor_to` then only retargets if the item still
        // ends up outside the destination viewport.
        let page = dir as f32 * rows_per_page as f32 * layout.row_step();
        self.scroll_to(self.scroll_target() + page, motion::SLOW);
        self.cursor_to(target, mode);
    }

    /// Type-to-select (Explorer ListView): characters typed within ~1 s form a prefix; the next
    /// item (cyclically from the anchor) whose name starts with it is selected and scrolled into
    /// view. One letter pressed repeatedly cycles through the items starting with it; a longer
    /// prefix stays on the current item while it still matches. Returns true when the selection
    /// moved.
    pub(super) fn type_ahead_char(&mut self, ch: char) -> bool {
        if self.items.is_empty() || self.rolled_up {
            return false;
        }
        if self
            .type_ahead_at
            .is_none_or(|t| t.elapsed() > TYPE_AHEAD_TIMEOUT)
        {
            self.type_ahead.clear();
        }
        self.type_ahead_at = Some(Instant::now());
        if self.type_ahead.is_empty() && ch == ' ' {
            return false;
        }
        self.type_ahead.push(ch);
        let lower = self.type_ahead.to_lowercase();
        let first = lower.chars().next().unwrap_or(ch);
        let single = lower.chars().count() == 1;
        let repeated = !single && lower.chars().all(|c| c == first);
        let n = self.items.len();
        let (needle, start) = if single || repeated {
            (
                first.to_string(),
                self.anchor_index.map_or(0, |a| (a + 1) % n),
            )
        } else {
            (lower, self.anchor_index.unwrap_or(0).min(n - 1))
        };
        for k in 0..n {
            let i = (start + k) % n;
            if self.items[i].name.to_lowercase().starts_with(&needle) {
                self.cursor_to(i, CursorMode::Select);
                return true;
            }
        }
        false
    }

    /// The content moved under a (possibly stationary) pointer — a glide ended or the offset
    /// snapped — so the item under it, its infotip and the cursor must follow. Posts a
    /// synthetic WM_MOUSEMOVE at the pointer's current client position: the full hover / tip /
    /// cursor logic then re-runs from the message loop instead of being duplicated here.
    /// `PostMessage` is asynchronous, so this is safe under the borrow and from `tick`
    /// (`hide_tip` is not: TTM_TRACKACTIVATE is a SendMessage). Skipped while a gesture owns
    /// the pointer, as those branches track the offset themselves.
    pub(super) fn request_hover_refresh(&self) {
        if !self.mouse_inside
            || self.marquee.is_some()
            || self.drag.is_some()
            || self.ole_drag
            || self.scrollbar_pressed()
        {
            return;
        }
        let pt = window::screen_to_client(self.hwnd, window::cursor_pos());
        window::post_message(
            self.hwnd,
            msg::WM_MOUSEMOVE,
            0,
            msg::make_lparam(pt.x, pt.y),
        );
    }

    /// Cancels an armed slow-double-click rename.
    pub(super) fn cancel_pending_rename(&mut self) {
        if self.pending_rename.take().is_some() {
            window::kill_timer(self.hwnd, TIMER_RENAME);
        }
    }

    pub(super) fn selected_paths(&self) -> Vec<PathBuf> {
        let mut v: Vec<usize> = self.selected.iter().copied().collect();
        v.sort_unstable();
        v.into_iter()
            .filter_map(|i| self.items.get(i).map(|it| it.path.clone()))
            .collect()
    }

    /// Redraws the surface that shows `p`'s pressed look.
    pub(super) fn redraw_for_press(&mut self, p: PressTarget) {
        match p {
            PressTarget::Item(_) | PressTarget::Header(_) => {
                if !self.rolled_up {
                    let _ = self.redraw_content();
                }
            }
            PressTarget::Tab(_) | PressTarget::Up | PressTarget::Chevron => {
                let _ = self.redraw_chrome_only();
            }
        }
    }

    /// Button-down on one of the release-fired controls (up button, chevron, header cell):
    /// records the press and paints it. The caller takes the mouse capture afterwards (with no
    /// borrow held) so the release always arrives here.
    pub(super) fn press_button_at(&mut self, x_px: i32, y_px: i32) -> Option<PressTarget> {
        let p = if self.up_button_at(x_px, y_px) {
            PressTarget::Up
        } else if self.chevron_at(x_px, y_px) {
            PressTarget::Chevron
        } else if self.divider_hit(x_px, y_px).is_none()
            && let Some(col) = self.header_hit(x_px, y_px)
        {
            PressTarget::Header(col)
        } else {
            return None;
        };
        self.pressed = Some(p);
        self.press_inside = true;
        self.press_origin = (x_px, y_px);
        self.redraw_for_press(p);
        Some(p)
    }

    /// Takes the pressed control without repainting: the caller folds the repaint into its own
    /// single redraw of the affected surface.
    pub(super) fn take_pressed(&mut self) -> Option<(PressTarget, bool)> {
        let p = self.pressed.take()?;
        let inside = std::mem::replace(&mut self.press_inside, false);
        Some((p, inside))
    }

    /// Clears a pressed control (capture loss, Esc), repainting it at rest.
    pub(super) fn clear_pressed(&mut self) -> Option<(PressTarget, bool)> {
        let r = self.take_pressed()?;
        self.redraw_for_press(r.0);
        Some(r)
    }

    /// Activation state for the selection colour (accent while active, neutral otherwise).
    /// Returns true when the content surface shows the change and needs a redraw (the caller
    /// folds it into its own redraw so a click rasterises the content once).
    #[must_use]
    pub(super) fn set_active(&mut self, on: bool) -> bool {
        if self.win_active == on {
            return false;
        }
        self.win_active = on;
        let to = if on { 0.0 } else { 1.0 };
        let now = Instant::now();
        if self.selected.is_empty() || !self.motion.enabled() {
            self.selection_inactive = Tween::at(to, now);
        } else {
            self.selection_inactive
                .retarget(to, motion::FAST, Curve::Linear, now);
        }
        !self.selected.is_empty() && !self.rolled_up
    }

    /// Pointer-initiated activation. Explorer processes WM_MOUSEACTIVATE / focus before the
    /// button message, so a clicked item is accent on its first frame; the 167 ms cross-fade
    /// is reserved for WM_ACTIVATE-driven changes (focus leaving / returning without a click).
    /// Returns true when the content surface shows the change and needs a redraw.
    #[must_use]
    pub(super) fn activate_by_pointer(&mut self) -> bool {
        let now = Instant::now();
        let changed = !self.win_active
            || !self.selection_inactive.is_done(now)
            || self.selection_inactive.target() != 0.0;
        self.win_active = true;
        self.selection_inactive = Tween::at(0.0, now);
        changed && !self.selected.is_empty() && !self.rolled_up
    }

    /// Tracks the mouse entering / leaving the window as a whole; redraws the chrome when the
    /// title-on-hover rule changes what is shown.
    pub(super) fn set_mouse_inside(&mut self, inside: bool) {
        if self.mouse_inside == inside {
            return;
        }
        self.mouse_inside = inside;
        if self.behavior.title_on_hover.get() {
            let _ = self.redraw_chrome_only();
        }
        if self.behavior.hide_inactive_scrollbar.get() {
            if inside {
                self.sync_scrollbar_alpha();
            } else {
                // Leaving counts as activity: the indicator fades 2 s later (WinUI
                // ScrollView no-indicator countdown), not the instant the pointer is gone.
                self.note_scroll_activity();
            }
        }
    }

    /// Marquee rectangle in content-local DIPs (visual coordinates, already scrolled).
    pub(super) fn marquee_rect_dip(&self) -> Option<Rect> {
        let m = self.marquee.as_ref()?;
        let scale = self.scale();
        let title_h = self.title_h_px() as f32;
        let (cw, _) = self.content_size_px();
        let fixed = self.layout(cw as f32 / scale).fixed_top();
        let to_dip = |(x, y): (i32, i32)| (x as f32 / scale, (y as f32 - title_h) / scale - fixed);
        let (x0, y0) = to_dip(m.start);
        // The start corner is anchored to the content, so scrolling under the band keeps it on
        // the same items.
        let y0 = y0 + m.start_scroll - self.scroll_y;
        let (x1, y1) = to_dip(m.cur);
        Some(Rect {
            left: x0.min(x1),
            top: y0.min(y1),
            right: x0.max(x1),
            bottom: y0.max(y1),
        })
    }

    /// Selects the items whose cells intersect the marquee: replacing the selection, adding to
    /// it (Shift) or toggling the banded items (Ctrl), like Explorer.
    pub(super) fn update_marquee_selection(&mut self) {
        let Some(r) = self.marquee_rect_dip() else {
            return;
        };
        let scale = self.scale();
        let (cw, _) = self.content_size_px();
        let grid = self.layout(cw as f32 / scale);
        let scroll = self.scroll_y;
        let (base, mode) = match self.marquee.as_ref() {
            Some(m) => (m.base.clone(), m.mode),
            None => return,
        };
        let mut band = HashSet::new();
        for i in 0..self.items.len() {
            let c = grid.cell(i);
            let (cy0, cy1) = (c.y - scroll, c.y - scroll + c.h);
            if c.x < r.right && c.x + c.w > r.left && cy0 < r.bottom && cy1 > r.top {
                band.insert(i);
            }
        }
        self.selected = match mode {
            MarqueeMode::Replace => band,
            MarqueeMode::Add => base.union(&band).copied().collect(),
            MarqueeMode::Toggle => base.symmetric_difference(&band).copied().collect(),
        };
    }

    /// Item area (client px) for the marquee auto-scroll test: (top, bottom).
    pub(super) fn marquee_band_px(&self) -> (i32, i32) {
        let scale = self.scale();
        let (cw, ch) = self.content_size_px();
        let fixed = self.layout(cw as f32 / scale).fixed_top();
        let top = self.title_h_px() + (fixed * scale) as i32;
        (top, self.title_h_px() + ch)
    }

    /// Updates the item / title-row hover; returns `(item_changed, title_changed)` so the
    /// caller redraws only the surface that shows the change.
    pub(super) fn set_hover(&mut self, hover: Option<usize>, title_hover: bool) -> (bool, bool) {
        if self.retired {
            // Fading out for good: no hover fades (they would only request frames nobody ticks).
            return (false, false);
        }
        let item = self.hover != hover;
        let title = self.title_hover != title_hover;
        self.hover = hover;
        self.title_hover = title_hover;
        (item, title)
    }

    pub(super) fn selected_ids(&self) -> Vec<ItemId> {
        let mut v: Vec<usize> = self.selected.iter().copied().collect();
        v.sort_unstable();
        v.into_iter()
            .filter_map(|i| self.items.get(i).map(|it| it.id))
            .collect()
    }
}
