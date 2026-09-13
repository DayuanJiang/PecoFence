//! Infotip target / arming / show / hide (comctl32 TME_HOVER + TTDT_RESHOW + auto-pop emulation).

use super::*;

/// Infotip delay: the full hover delay normally; the short comctl32 TTDT_RESHOW delay when a
/// tip was hidden less than one hover delay ago (sweeping across neighbouring items).
pub(super) fn tip_delay_ms(hover_ms: u32, reshow_ms: u32, hidden_ago: Option<Duration>) -> u32 {
    match hidden_ago {
        Some(ago) if ago < Duration::from_millis(hover_ms as u64) => reshow_ms,
        _ => hover_ms,
    }
}

/// What the infotip describes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TipTarget {
    Item(usize),
    Tab(usize),
    Up,
}

/// TME_HOVER rectangle test: has the pointer left the `rect` (SPI_GETMOUSEHOVERWIDTH x
/// SPI_GETMOUSEHOVERHEIGHT, physical px) centred on where the infotip timer was armed? The delay
/// restarts when it has, so a tip appears only once the pointer rests.
pub(super) fn tip_hover_moved(anchor: (i32, i32), pt: (i32, i32), rect: (i32, i32)) -> bool {
    (pt.0 - anchor.0).abs() >= (rect.0 / 2).max(1) || (pt.1 - anchor.1).abs() >= (rect.1 / 2).max(1)
}

impl FenceViewState {
    /// What an infotip at a client-pixel point would describe. `hit` is the hover-filtered
    /// item hit (None over the scrollbar hot zone / title controls), so the tip can never
    /// name an item the hover logic does not consider under the pointer.
    pub(super) fn tip_target_at(
        &self,
        x_px: i32,
        y_px: i32,
        hit: Option<usize>,
    ) -> Option<TipTarget> {
        if self.up_button_at(x_px, y_px) {
            return Some(TipTarget::Up);
        }
        if let Some(t) = self.tab_at(x_px, y_px) {
            return Some(TipTarget::Tab(t));
        }
        hit.map(TipTarget::Item)
    }

    /// NOTE: `TTM_TRACKACTIVATE` makes comctl32 send `WM_NOTIFY` (TTN_SHOW / TTN_POP) back to
    /// this window synchronously while the view is borrowed. That is only safe because the
    /// wndproc has no `WM_NOTIFY` arm (it falls through to DefWindowProc); do not add one that
    /// borrows `view`.
    pub(super) fn hide_tip(&mut self) {
        window::kill_timer(self.hwnd, TIMER_TIP);
        window::kill_timer(self.hwnd, TIMER_TIP_HIDE);
        if self.tip_shown {
            if let Some(t) = self.tip.as_mut() {
                t.hide();
            }
            self.tip_shown = false;
            self.tip_hidden_at = Some(Instant::now());
        }
        self.tip_target = None;
        self.tip_armed = false;
    }

    /// Auto-pop (TIMER_TIP_HIDE): the tip goes away but the target stays, so it does not come
    /// back until the pointer moves onto something else (comctl32 after TTDT_AUTOPOP).
    pub(super) fn autopop_tip(&mut self) {
        if self.tip_shown {
            if let Some(t) = self.tip.as_mut() {
                t.hide();
            }
            self.tip_shown = false;
            self.tip_hidden_at = Some(Instant::now());
        }
    }

    /// Pointer moved to client px (`x`, `y`): re-arm the infotip when it settles on something
    /// else, and restart the delay while it keeps moving over the same thing (TME_HOVER: the
    /// tip appears only after the pointer has rested inside the SM_C[XY]MOUSEHOVER box for
    /// SPI_GETMOUSEHOVERTIME). Moving straight from one tipped thing to the next re-shows
    /// after the short TTDT_RESHOW delay. A tip already shown / auto-popped never re-arms.
    pub(super) fn update_tip_target(&mut self, target: Option<TipTarget>, x: i32, y: i32) {
        if target == self.tip_target {
            if self.tip_armed
                && tip_hover_moved(
                    self.tip_anchor,
                    (x, y),
                    pecofence_platform::sysparams::mouse_hover_rect(),
                )
            {
                window::kill_timer(self.hwnd, TIMER_TIP);
                self.arm_tip_timer(x, y);
            }
            return;
        }
        self.hide_tip();
        self.tip_target = target;
        if target.is_some() {
            self.arm_tip_timer(x, y);
        }
    }

    pub(super) fn arm_tip_timer(&mut self, x: i32, y: i32) {
        let delay = tip_delay_ms(
            Tooltip::hover_delay_ms(),
            Tooltip::reshow_delay_ms(),
            self.tip_hidden_at.map(|t| t.elapsed()),
        );
        window::set_timer(self.hwnd, TIMER_TIP, delay);
        self.tip_anchor = (x, y);
        self.tip_armed = true;
    }

    /// Infotip text: items get Explorer's name / 类型 / 大小 / 修改日期, a shrunken tab pill its
    /// full caption, the portal up button "向上".
    pub(super) fn tip_text(&self) -> Option<String> {
        match self.tip_target? {
            TipTarget::Up => Some(pecofence_core::i18n::text("向上").to_string()),
            TipTarget::Tab(i) => {
                let t = self.tabs.get(i)?;
                let (_, w) = *self.tab_rects().get(i)?;
                (!self
                    .chrome
                    .tab_title_fits(&t.title, w, t.title_size, &self.theme))
                .then(|| t.title.clone())
            }
            TipTarget::Item(i) => {
                let it = self.items.get(i)?;
                let mut s = it.name.clone();
                if let Some(t) = fileinfo::type_name(&it.path, it.is_folder) {
                    s.push_str(&pecofence_core::i18n::format(
                        "\n类型: {0}",
                        std::slice::from_ref(&t),
                    ));
                }
                if !it.is_folder {
                    s.push_str(&pecofence_core::i18n::format(
                        "\n大小: {0}",
                        &[fileinfo::format_size_kb(it.size, false).to_string()],
                    ));
                }
                s.push_str(&pecofence_core::i18n::format(
                    "\n修改日期: {0}",
                    &[fileinfo::format_local_datetime(it.mtime).to_string()],
                ));
                Some(s)
            }
        }
    }

    pub(super) fn show_tip(&mut self) {
        let Some(text) = self.tip_text() else {
            return;
        };
        if self.tip.is_none() {
            let dark = matches!(self.theme.mode, pecofence_render::ThemeMode::Dark);
            self.tip = Tooltip::create(self.hwnd, dark)
                .map_err(|e| tracing::warn!(error = %e, "tooltip window failed"))
                .ok();
        }
        let pt = window::cursor_pos();
        let scale = self.scale();
        if let Some(t) = self.tip.as_mut() {
            t.show(
                &text,
                pt.x + (12.0 * scale).round() as i32,
                pt.y + (20.0 * scale).round() as i32,
            );
            self.tip_shown = true;
            self.tip_armed = false;
            window::set_timer(
                self.hwnd,
                TIMER_TIP_HIDE,
                pecofence_platform::sysparams::message_duration_secs() * 1000,
            );
        }
    }
}
