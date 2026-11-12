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

/// Where a tip beside the pointer goes at `scale`: its top-left (12 DIPs right of the pointer
/// and 20 below it, under the cursor's picture), and where its bottom edge goes when there is
/// no room below: 4 DIPs above the pointer.
pub(super) fn pointer_tip_place(scale: f32) -> (i32, i32, i32) {
    let pt = window::cursor_pos();
    let px = |dip: f32| (dip * scale).round() as i32;
    (pt.x + px(12.0), pt.y + px(20.0), pt.y - px(4.0))
}

/// What the infotip describes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TipTarget {
    Item(usize),
    Tab(usize),
    Up,
    /// The Task View glyph of a single title: the desktops the fence shows on.
    Desktops,
    /// A script fence's clickable row.
    ScriptRow(usize),
    /// A filter tag too narrow for all of the filter.
    FilterTag,
    /// The lock glyph of a locked fence: how to unlock it.
    Lock,
    /// A single title that is drawn cut short: all of it.
    Title,
    /// The arrow at the right of the title: what a click on it does (收起 / 展开).
    Chevron,
}

/// TME_HOVER rectangle test: has the pointer left the `rect` (SPI_GETMOUSEHOVERWIDTH x
/// SPI_GETMOUSEHOVERHEIGHT, physical px) centred on where the infotip timer was armed? The delay
/// restarts when it has, so a tip appears only once the pointer rests.
pub(super) fn tip_hover_moved(anchor: (i32, i32), pt: (i32, i32), rect: (i32, i32)) -> bool {
    (pt.0 - anchor.0).abs() >= (rect.0 / 2).max(1) || (pt.1 - anchor.1).abs() >= (rect.1 / 2).max(1)
}

/// The second line of a shortcut's tip, as File Explorer writes it: the shortcut's `comment`,
/// else 「位置: <target> (<its folder>)」. None when there is neither (the usual type, size and
/// date lines show then). A comment kept as a resource reference (`@shell32.dll,-22566`) is
/// no text to show.
pub(super) fn shortcut_tip_line(
    comment: Option<String>,
    target: Option<std::path::PathBuf>,
) -> Option<String> {
    if let Some(c) = comment.filter(|c| !c.starts_with('@')) {
        return Some(format!("\n{c}"));
    }
    let target = target?;
    let name = target
        .file_stem()
        .or_else(|| target.file_name())?
        .to_string_lossy()
        .into_owned();
    let place = match target.parent().filter(|p| !p.as_os_str().is_empty()) {
        Some(dir) => format!("{name} ({})", dir.display()),
        None => name,
    };
    Some(pecofence_core::i18n::format("\n位置: {0}", &[place]))
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
        if self.cut_filter_tag_at(x_px, y_px) {
            return Some(TipTarget::FilterTag);
        }
        if self.up_button_at(x_px, y_px) {
            return Some(TipTarget::Up);
        }
        if self.desktop_glyph_at(x_px, y_px) {
            return Some(TipTarget::Desktops);
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
    /// after the short TTDT_RESHOW delay. A tip already shown / auto-popped never re-arms. A
    /// move to the screen point of the previous one changes nothing (`tip_move_at`).
    pub(super) fn update_tip_target(&mut self, target: Option<TipTarget>, x: i32, y: i32) {
        let pt = window::client_to_screen(self.hwnd, pecofence_platform::POINT { x, y });
        if self.tip_move_at.replace((pt.x, pt.y)) == Some((pt.x, pt.y)) {
            return;
        }
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
    /// full caption, the portal up button "向上" (at the root, where its glyph opens the folder,
    /// 在资源管理器中打开文件夹).
    pub(super) fn tip_text(&self) -> Option<String> {
        // Both tips of the title's folder glyph end with the path of the folder shown: the
        // title is one name (the fence's own at the root, a subfolder's below it) and says
        // neither which folder that is nor how deep the fence has gone.
        let with_folder = |text: &str| match &self.portal_folder {
            Some(dir) => format!("{text}\n{}", dir.display()),
            None => text.to_string(),
        };
        match self.tip_target? {
            TipTarget::Up if self.glyph_opens_folder() => Some(with_folder(
                pecofence_core::i18n::text("在资源管理器中打开文件夹"),
            )),
            TipTarget::Up => Some(with_folder(pecofence_core::i18n::text("向上"))),
            TipTarget::Desktops => self.desktop_tip.clone(),
            TipTarget::Title => Some(self.title.clone()),
            TipTarget::Chevron => Some(
                pecofence_core::i18n::text(if self.roll_target() {
                    "展开"
                } else {
                    "收起"
                })
                .to_string(),
            ),
            TipTarget::Lock => {
                Some(pecofence_core::i18n::text("已锁定位置和大小，右键可解除锁定").to_string())
            }
            // The whole filter: the type, the text and the count.
            TipTarget::FilterTag => self.filter_tag.as_ref().map(|(kind, text, count)| {
                [kind, text, count]
                    .iter()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join("  ")
            }),
            TipTarget::ScriptRow(i) => self.script.as_ref()?.rows.get(i)?.tip.clone(),
            TipTarget::Tab(i) => {
                let t = self.tabs.get(i)?;
                let fits = if self.tabs_on_rail() {
                    // A row of the left column is as wide as the column, whatever its height.
                    let active = t.id == self.active;
                    let colored = self.tabs.iter().any(|t| t.color.is_some());
                    let folders = self.tabs.iter().any(|t| t.folder_icon);
                    let desktops = self.tabs.iter().any(|t| t.desktop_icon);
                    let badge = self.tab_badge(t.id);
                    let up = active && self.deco.up_button;
                    let room = self.chrome.rail_title_room(
                        self.rail_w(),
                        colored,
                        (folders, desktops),
                        badge.as_deref(),
                        up,
                    );
                    self.chrome
                        // Collapsed to the column, the shown tab is drawn like the others.
                        .rail_title_fits(
                            &t.title,
                            room,
                            t.title_size,
                            active && !self.column_collapsed,
                        )
                } else {
                    let (_, w) = *self.tab_rects().get(i)?;
                    self.chrome.tab_title_fits(
                        &t.title,
                        w,
                        t.title_size,
                        (t.folder_icon, t.desktop_icon),
                        &self.theme,
                    )
                };
                // A tab of a window bound to virtual desktops names them (its Task View glyph
                // is on the pill), after the caption when that is cut off.
                let caption = (!fits).then(|| t.title.clone());
                match (
                    caption,
                    self.desktop_tip.as_ref().filter(|_| t.desktop_icon),
                ) {
                    (Some(c), Some(d)) => Some(format!("{c}\n{d}")),
                    (c, d) => c.or_else(|| d.cloned()),
                }
            }
            TipTarget::Item(i) => {
                let it = self.items.get(i)?;
                let mut s = it.name.clone();
                // A shortcut: what it is for or where it leads, as in Explorer. The size and
                // the date of the link file itself say nothing about the program.
                let is_link = !it.is_folder
                    && it
                        .path
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("lnk"));
                if is_link
                    && let Some((comment, target)) =
                        pecofence_platform::shell::shortcut_tip(&it.path)
                    && let Some(line) = shortcut_tip_line(comment, target)
                {
                    s.push_str(&line);
                    return Some(s);
                }
                if let Some(t) = fileinfo::type_name(&it.path, it.is_folder) {
                    s.push_str(&pecofence_core::i18n::format(
                        "\n类型: {0}",
                        std::slice::from_ref(&t),
                    ));
                }
                // Namespace items (Recycle Bin, ...) have neither a size nor a date.
                if !pecofence_platform::shell::is_namespace_path(&it.path) {
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
                }
                Some(s)
            }
        }
    }

    pub(super) fn show_tip(&mut self) {
        let (x, y, above) = pointer_tip_place(self.scale());
        self.show_tip_at(x, y, above);
    }

    /// Keyboard focus (arrow keys, type-to-select, the Peek search's current match) moved to
    /// item `index` in an icon grid without labels: nothing on screen names it, so its infotip
    /// shows at once under the icon. `None` hides it.
    pub(super) fn tip_keyboard_focus(&mut self, index: Option<usize>) {
        if self.layout != ViewLayout::Icons || self.label_lines != 0 || self.rolled_up {
            return;
        }
        self.hide_tip();
        let Some(i) = index else {
            return;
        };
        let Some(r) = self.items.get(i).and_then(|it| self.item_label_rect(it.id)) else {
            return;
        };
        // The rect uses the current scroll offset. A keyboard glide is still heading for the
        // target, so the tip goes where the item stops.
        let glide = ((self.scroll_y - self.scroll_target()) * self.scale()).round() as i32;
        self.tip_target = Some(TipTarget::Item(i));
        // With no room below, above the icon (the name's place is right under it).
        let icon = ((self.icon_size as f32 + 8.0) * self.scale()).round() as i32;
        self.show_tip_at(r.left, r.bottom + glide, r.top + glide - icon);
    }

    fn show_tip_at(&mut self, x: i32, y: i32, above: i32) {
        let Some(text) = self.tip_text() else {
            return;
        };
        self.show_tip_text_at(&text, x, y, above);
    }

    /// `above`: where the tip's bottom edge goes when it has no room below (`Tooltip::show`).
    pub(super) fn show_tip_text_at(&mut self, text: &str, x: i32, y: i32, above: i32) {
        if self.tip.is_none() {
            let dark = matches!(self.theme.mode, pecofence_render::ThemeMode::Dark);
            self.tip = Tooltip::create(self.hwnd, dark)
                .map_err(|e| tracing::warn!(error = %e, "tooltip window failed"))
                .ok();
        }
        if let Some(t) = self.tip.as_mut() {
            t.show(text, x, y, above);
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
