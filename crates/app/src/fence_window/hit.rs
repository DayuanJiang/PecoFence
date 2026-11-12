//! Client-pixel hit-testing and item / grid / details-column geometry helpers.

use super::*;

/// Details header divider being dragged.
pub(super) struct ColDrag {
    pub(super) col: DetailColumn,
    pub(super) start_x_px: i32,
    pub(super) start_w: f32,
    /// The widest the column may get in this drag (`DetailColumns::drag_limit`).
    pub(super) max_w: f32,
}

pub(super) fn bounded_auto_height(desired: i32, title_height: i32, top: i32, bottom: i32) -> i32 {
    // Preserve a client pixel even when the title was dragged almost offscreen. In normal
    // positions, excess rows stay accessible via the existing scrollbar above the taskbar.
    desired.min(bottom.saturating_sub(top).max(title_height + 1))
}

/// The resize hit code of a point `inside` (px from the window's left, top, right and bottom
/// edges) of a window `size` px large, or None off the resize edges. An edge is `border` px
/// deep (`right_border` on the right, thinner beside a scrollbar). A corner is the part of an
/// edge within `corner` px of the adjacent edge, as DefWindowProc gives a normal window, so the
/// diagonal cursor answers along 16 DIPs of each edge and not only in the 6 x 6 square where
/// two edges cross. On a small window a corner takes at most a third of the edge.
pub(super) fn resize_edge_at(
    inside: (i32, i32, i32, i32),
    size: (i32, i32),
    border: i32,
    right_border: i32,
    corner: i32,
) -> Option<isize> {
    let (left, top, right, bottom) = inside;
    let (on_left, on_right) = (left < border, right <= right_border);
    let (on_top, on_bottom) = (top < border, bottom <= border);
    let (reach_x, reach_y) = (
        corner.min(size.0 / 3).max(border),
        corner.min(size.1 / 3).max(border),
    );
    let (near_left, near_right) = (left < reach_x, right <= reach_x);
    let (near_top, near_bottom) = (top < reach_y, bottom <= reach_y);
    let code = if (on_left && near_top) || (on_top && near_left) {
        msg::HTTOPLEFT
    } else if (on_right && near_top) || (on_top && near_right) {
        msg::HTTOPRIGHT
    } else if (on_left && near_bottom) || (on_bottom && near_left) {
        msg::HTBOTTOMLEFT
    } else if (on_right && near_bottom) || (on_bottom && near_right) {
        msg::HTBOTTOMRIGHT
    } else if on_left {
        msg::HTLEFT
    } else if on_right {
        msg::HTRIGHT
    } else if on_top {
        msg::HTTOP
    } else if on_bottom {
        msg::HTBOTTOM
    } else {
        return None;
    };
    Some(code)
}

impl FenceViewState {
    /// Is the client-pixel point on item `index`'s label text (not its icon)? Never in an
    /// icon grid without labels, whose label rect has no height.
    pub(super) fn label_hit(&self, x_px: i32, y_px: i32, index: usize) -> bool {
        let Some(id) = self.items.get(index).map(|i| i.id) else {
            return false;
        };
        let Some(r) = self.item_label_rect(id) else {
            return false;
        };
        let win = crate::fence_window::shown_rect(self.hwnd);
        let (sx, sy) = (x_px + win.left, y_px + win.top);
        sx >= r.left && sx < r.right && sy >= r.top && sy < r.bottom
    }

    /// Double-click on a header divider: size the column to its widest cell (Explorer).
    pub(super) fn auto_fit_column(&mut self, col: DetailColumn) {
        if col == DetailColumn::Name || self.layout != ViewLayout::Details {
            return;
        }
        let caption = match col {
            DetailColumn::Date => pecofence_core::i18n::text("修改日期"),
            DetailColumn::Type => pecofence_core::i18n::text("类型"),
            DetailColumn::Size => pecofence_core::i18n::text("大小"),
            DetailColumn::Name => return,
        };
        let chrome = self.chrome.clone();
        let fmt = chrome.row_format();
        // 12 DIPs of caption breathing room (the sort chevron sits above the caption).
        let mut w = pecofence_render::text::measure_width(caption, &fmt) + 12.0;
        for item in &mut self.items {
            let label = match col {
                DetailColumn::Date => {
                    let text = item.date_text();
                    item.date_label.get_or_insert(text).clone()
                }
                // Rows that were never drawn have no type text yet: asked once per extension
                // here too (`ensure_texts`), not once per file.
                DetailColumn::Type => item
                    .type_label
                    .get_or_insert_with(|| {
                        self.type_names
                            .get(&item.path, item.is_folder)
                            .unwrap_or_default()
                    })
                    .clone(),
                _ => {
                    let text = item.size_text();
                    item.size_label.get_or_insert(text).clone()
                }
            };
            w = w.max(pecofence_render::text::measure_width(&label, &fmt));
        }
        // 8 DIPs of cell padding; MIN/MAX clamping happens in set_column_width.
        self.set_column_width(col, w + 8.0);
    }

    /// Is the client-pixel point anywhere in the Details header band (column menu)?
    pub(super) fn header_band_hit(&self, x_px: i32, y_px: i32) -> bool {
        if self.layout != ViewLayout::Details || self.rolled_up {
            return false;
        }
        let (cw, _) = self.content_size_px();
        let x_px = x_px - self.content_x;
        if x_px < 0 || x_px >= cw {
            return false;
        }
        let y = (y_px - self.content_y) as f32 / self.scale();
        y >= 0.0 && y < self.row_metrics_of(ViewLayout::Details).header_h
    }

    /// At its root folder a portal's title glyph opens that folder in Explorer (#192); inside
    /// a subfolder the same glyph is the up button. A left tab column has no glyph button at
    /// the root, and a folder that cannot be found opens nothing.
    pub(super) fn glyph_opens_folder(&self) -> bool {
        self.is_portal
            && self.deco.folder_icon
            && !self.deco.up_button
            && !self.portal_missing
            && !self.tabs_on_rail()
    }

    /// Is the client-pixel point on the portal "up" button? A tab strip in the title row has it
    /// on the shown tab's folder glyph (`FenceChrome::tab_up_box`); a left tab column at the end
    /// of the shown tab's row (`rail_up_box`); a single title on its folder glyph
    /// (`FenceChrome::lone_up_box`), or at the far left while the glyph is hidden. At the
    /// portal's root the glyph is the same button, opening the folder instead
    /// ([`Self::glyph_opens_folder`]).
    pub(super) fn up_button_at(&self, x_px: i32, y_px: i32) -> bool {
        let opens = self.glyph_opens_folder();
        if !self.deco.up_button && !opens {
            return false;
        }
        let scale = self.scale();
        let (x, y) = (x_px as f32 / scale, y_px as f32 / scale);
        let r = if self.title_row_strip() {
            // The pill is where it is drawn (sliding, dragged); a scrolled strip's cut-off part
            // takes no click (`tab_at`). A tab without its folder glyph has no button.
            let i = self.active_index();
            let Some(t) = self
                .tabs
                .get(i)
                .filter(|t| (t.up_button || opens) && t.folder_icon)
            else {
                return false;
            };
            let (Some(&tab_x), Some(&(_, tab_w))) = (
                self.tab_draw_xs(Instant::now()).get(i),
                self.tab_rects().get(i),
            ) else {
                return false;
            };
            if self.tab_at(x_px, y_px) != Some(i) {
                return false;
            }
            let badge = self.tab_badge(t.id);
            self.chrome.tab_up_box(
                &self.theme,
                tab_x,
                tab_w,
                &t.title,
                t.title_size,
                t.desktop_icon,
                badge.as_deref(),
            )
        } else if self.tabs_on_rail() {
            match self.rail_up_box() {
                Some(r) => r,
                None => return false,
            }
        } else if self.deco.folder_icon {
            // The title moves aside for the rolled item count and the filter tag, as drawn;
            // the tag over the glyph of a narrow fence takes the click.
            if self.on_filter_tag(x, y) {
                return false;
            }
            let width = self.chrome_panel.size_px().0 as f32 / scale;
            let reserve = if self.roll_target()
                && !self.behavior.hide_count_and_chevron.get()
                && self.filter_tag.is_none()
            {
                pecofence_render::fence_chrome::header_count_width(
                    width - self.laid_deco().lock_w(),
                    1,
                    self.deco.title_x(),
                    self.rolled_count_room(),
                )
            } else {
                0.0
            };
            self.chrome.lone_up_box(
                &self.title,
                self.filter_tag_ref(),
                self.style.title_size,
                self.behavior.title_align.get(),
                width,
                (self.deco.desktop_icon, self.laid_deco().locked),
                reserve,
                self.theme.title_height,
            )
        } else {
            TitleDeco::up_box(self.theme.title_height)
        };
        x >= r.left && x < r.right && y >= r.top && y < r.bottom
    }

    /// Is the client-pixel point on the Task View glyph of a single title (a window bound to
    /// virtual desktops; its infotip names them)? A tab strip has it on every pill, where the
    /// tab's own infotip names them.
    pub(super) fn desktop_glyph_at(&self, x_px: i32, y_px: i32) -> bool {
        if !self.deco.desktop_icon || self.title_row_strip() || self.tabs_on_rail() {
            return false;
        }
        let scale = self.scale();
        let (x, y) = (x_px as f32 / scale, y_px as f32 / scale);
        let width = self.chrome_panel.size_px().0 as f32 / scale;
        if self.on_filter_tag(x, y) {
            return false;
        }
        // As drawn: a smart fence's count is its capsule and a filter tag carries one, so no
        // rolled count at the right.
        let reserve = if self.roll_target()
            && !self.behavior.hide_count_and_chevron.get()
            && self.title_count.is_none()
            && self.filter_tag.is_none()
        {
            pecofence_render::fence_chrome::header_count_width(
                width - self.laid_deco().lock_w(),
                1,
                self.deco.title_x(),
                self.rolled_count_room(),
            )
        } else {
            0.0
        };
        let r = self.chrome.lone_desktop_box(
            &self.title,
            self.title_count.as_deref(),
            self.filter_tag_ref(),
            self.style.title_size,
            self.behavior.title_align.get(),
            width,
            self.laid_deco(),
            reserve,
            self.theme.title_height,
        );
        x >= r.left && x < r.right && y >= r.top && y < r.bottom
    }

    /// The row a single title is laid out in, as drawn (see `desktop_glyph_at`): its width in
    /// DIPs and the rolled count's room at the right.
    fn lone_title_row(&self) -> (f32, f32) {
        let width = self.chrome_panel.size_px().0 as f32 / self.scale();
        // As drawn (`render`, `FenceChrome::draw`): a note, a clock and a script fence show
        // no count, and a left-aligned title leaves only the count's text free. With the
        // whole box always kept, a title shown whole got a tip that repeated it, and the name
        // box of a tool fence's centred or right-aligned title stood up to 64 DIPs left.
        let reserve = if self.roll_target()
            && !self.behavior.hide_count_and_chevron.get()
            && self.shows_rolled_count()
        {
            let caption = self.rolled_count_caption();
            let count_w = pecofence_render::fence_chrome::header_count_width(
                width - self.laid_deco().lock_w(),
                1,
                self.deco.title_x(),
                self.chrome.rolled_count_room(&caption),
            );
            self.chrome
                .rolled_count_reserve(&caption, count_w, self.behavior.title_align.get(), 1)
        } else {
            0.0
        };
        (width, reserve)
    }

    /// A centred or right-aligned single title as drawn (DIPs): where its text starts and
    /// ends, and the room it is aligned in. Its name box takes that room.
    pub(super) fn lone_title_text_span(&self) -> (f32, f32, (f32, f32)) {
        let (width, reserve) = self.lone_title_row();
        let deco = self.laid_deco();
        let (x, w, room_w) = self.chrome.lone_title_text_span(
            &self.title,
            self.title_count.as_deref(),
            self.filter_tag_ref(),
            self.style.title_size,
            self.behavior.title_align.get(),
            width,
            deco,
            reserve,
        );
        (x, x + w, (deco.title_x(), deco.title_x() + room_w))
    }

    /// Is the single title drawn cut short (it ends in an ellipsis)? Its infotip then shows the
    /// whole title, as a cut tab caption's does. Never for a tab strip, a left tab column (the
    /// tabs have their own infotips) or the desktop fence without a title row.
    pub(super) fn lone_title_cut(&self) -> bool {
        if self.title_row_strip() || self.tabs_on_rail() || self.bare() {
            return false;
        }
        let (width, reserve) = self.lone_title_row();
        self.chrome.lone_title_cut(
            &self.title,
            self.title_count.as_deref(),
            self.filter_tag_ref(),
            self.style.title_size,
            self.behavior.title_align.get(),
            width,
            self.laid_deco(),
            reserve,
        )
    }

    /// The portal up button in a left tab column (DIPs): at the end of the shown tab's row, as
    /// drawn (`TitleDeco::rail_up_box`). A column the fence is collapsed to has none.
    pub(super) fn rail_up_box(&self) -> Option<Rect> {
        if self.column_collapsed {
            return None;
        }
        let i = self.active_index();
        let (_, h) = *self.tab_rects().get(i)?;
        let y = *self.tab_draw_xs(Instant::now()).get(i)?;
        let row = Rect::from_xywh(
            RAIL_ROW_INSET,
            y,
            (self.rail_w() - RAIL_ROW_INSET - RAIL_ROW_RIGHT).max(0.0),
            h,
        );
        Some(TitleDeco::rail_up_box(row))
    }

    /// Is the client-pixel point in the title row (the left tab column beside one)? Its
    /// caption and its controls (tabs, the chevron box) are one row for the hover.
    pub(super) fn title_row_at(&self, x_px: i32, y_px: i32) -> bool {
        if self.tabs_on_rail() {
            x_px >= 0 && x_px < self.content_x
        } else {
            y_px >= 0 && y_px < self.title_h_px()
        }
    }

    /// The filter tag of a single title (筛选), as drawn.
    fn filter_tag_ref(&self) -> Option<(&str, &str, &str)> {
        self.filter_tag
            .as_ref()
            .map(|(k, t, c)| (k.as_str(), t.as_str(), c.as_str()))
    }

    /// What of the filter tag a single title shows, as drawn (None: no tag, or not even its
    /// magnifier and × fit).
    pub(super) fn filter_tag_fit(&self) -> Option<pecofence_render::fence_chrome::TagFit> {
        let tag = self.filter_tag_ref()?;
        if self.title_row_strip() || self.tabs_on_rail() || self.bare() {
            return None;
        }
        // As drawn: a locked fence keeps the lock glyph's slot before the chevron.
        let width = self.chrome_panel.size_px().0 as f32 / self.scale() - self.laid_deco().lock_w();
        self.chrome.filter_tag_fit(
            width,
            self.theme.title_height,
            tag,
            self.title_count.as_deref(),
        )
    }

    /// The filter tag's box (DIPs) when a single title shows one.
    fn filter_tag_box(&self) -> Option<Rect> {
        self.filter_tag_fit().map(|f| f.rect)
    }

    /// Is the DIP point on the filter tag?
    fn on_filter_tag(&self, x: f32, y: f32) -> bool {
        self.filter_tag_box()
            .is_some_and(|r| x >= r.left && x < r.right && y >= r.top && y < r.bottom)
    }

    /// Is the client-pixel point on a filter tag that shows only part of the filter (its
    /// infotip names all of it)?
    pub(super) fn cut_filter_tag_at(&self, x_px: i32, y_px: i32) -> bool {
        let scale = self.scale();
        let (x, y) = (x_px as f32 / scale, y_px as f32 / scale);
        match (self.filter_tag_fit(), self.filter_tag_ref()) {
            (Some(f), Some(tag)) => {
                !f.whole(tag)
                    && x >= f.rect.left
                    && x < f.rect.right
                    && y >= f.rect.top
                    && y < f.rect.bottom
            }
            _ => false,
        }
    }

    /// Is the client-pixel point on the × of the filter tag (筛选, a single title)?
    pub(super) fn filter_tag_clear_at(&self, x_px: i32, y_px: i32) -> bool {
        let scale = self.scale();
        let (x, y) = (x_px as f32 / scale, y_px as f32 / scale);
        self.filter_tag_box().is_some_and(|r| {
            x >= r.right - pecofence_render::fence_chrome::TAG_CLEAR_W
                && x < r.right
                && y >= r.top
                && y < r.bottom
        })
    }

    /// Is the client-pixel point on the roll-up chevron button (right end of the title row)?
    pub(super) fn chevron_at(&self, x_px: i32, y_px: i32) -> bool {
        let scale = self.scale();
        let (w_px, h_px) = self.chrome_panel.size_px();
        // Under the tabs of a left tab column (the fence's title bar). Collapsed to the column
        // there is none: a tab click expands the fence. Hidden by the user, its spot is title.
        // 像桌面一样排列 has no title row: the spot is part of an icon's cell, where a press
        // starts a rectangle (a drag from it moved the whole 「桌面」 fence).
        if self.column_collapsed || self.behavior.hide_count_and_chevron.get() || self.bare() {
            return false;
        }
        // 鼠标移入时保持透明 with the title row folded away: the chevron is not drawn, and a
        // click on a button nobody sees would roll the fence up. Its spot is title row.
        if self.title_row_unseen() {
            return false;
        }
        let r = if self.tabs_on_rail() {
            let rows_end = self.rail_rows_end();
            TitleDeco::rail_chevron_box(self.rail_w(), h_px as f32 / scale, rows_end)
        } else {
            TitleDeco::chevron_box(w_px as f32 / scale, self.theme.title_height)
        };
        let (x, y) = (x_px as f32 / scale, y_px as f32 / scale);
        x >= r.left && x < r.right && y >= r.top && y < r.bottom
    }

    /// The title-row decorations as they are laid out and drawn now: the stored ones, except
    /// that the lock glyph of a single title gives way when the row has no room for it beside
    /// the start of the name (`FenceChrome::lone_deco_in`: a fence sized to its narrowest and
    /// locked afterwards). Drawing, the hit tests and the infotip all go by this.
    pub(super) fn laid_deco(&self) -> TitleDeco {
        if !self.deco.locked || self.title_row_strip() || self.tabs_on_rail() {
            return self.deco;
        }
        let width = self.chrome_panel.size_px().0 as f32 / self.scale();
        self.chrome.lone_deco_in(
            &self.title,
            self.title_count.as_deref(),
            self.style.title_size,
            width,
            self.deco,
        )
    }

    /// Is the client-pixel point on the lock glyph of a locked fence (its infotip says how to
    /// unlock)? The glyph's slot before the chevron, as drawn; a fence collapsed to its left
    /// tab column shows none.
    pub(super) fn lock_glyph_at(&self, x_px: i32, y_px: i32) -> bool {
        if !self.laid_deco().locked || self.bare() || self.column_collapsed {
            return false;
        }
        let scale = self.scale();
        let (w_px, h_px) = self.chrome_panel.size_px();
        let r = if self.tabs_on_rail() {
            let rows_end = self.rail_rows_end();
            TitleDeco::rail_lock_box(self.rail_w(), h_px as f32 / scale, rows_end)
        } else {
            TitleDeco::lock_box(w_px as f32 / scale, self.theme.title_height)
        };
        let (x, y) = (x_px as f32 / scale, y_px as f32 / scale);
        x >= r.left && x < r.right && y >= r.top && y < r.bottom
    }

    /// Is the client-pixel point on the count capsule of a smart fence's single title, as drawn
    /// (its infotip says of how many files the fence shows the newest)?
    pub(super) fn count_capsule_at(&self, x_px: i32, y_px: i32) -> bool {
        let Some(count) = self.title_count.as_deref() else {
            return false;
        };
        if self.title_row_strip() || self.tabs_on_rail() || self.bare() || self.column_collapsed {
            return false;
        }
        let scale = self.scale();
        let width = self.chrome_panel.size_px().0 as f32 / scale - self.laid_deco().lock_w();
        let r = self
            .chrome
            .count_capsule_box(count, width, self.theme.title_height);
        let (x, y) = (x_px as f32 / scale, y_px as f32 / scale);
        x >= r.left && x < r.right && y >= r.top && y < r.bottom
    }

    /// Is the pointer (client px) over the pressed control?
    pub(super) fn press_hit(&self, p: PressTarget, x_px: i32, y_px: i32) -> bool {
        match p {
            PressTarget::Up => self.up_button_at(x_px, y_px),
            PressTarget::Chevron => self.chevron_at(x_px, y_px),
            PressTarget::Header(c) => self.header_hit(x_px, y_px) == Some(c),
            PressTarget::Item(i) => self.hit_item(x_px, y_px) == Some(i),
            PressTarget::Tab(t) => self.tab_at(x_px, y_px) == Some(t),
            PressTarget::NoteBox(l) => self.note_box_at(x_px, y_px) == Some(l),
            PressTarget::NoteLink(l, s) => self.note_link_at(x_px, y_px) == Some((l, s)),
            PressTarget::ScriptRow(r) => self.script_row_at(x_px, y_px) == Some(r),
        }
    }

    pub(super) fn grid_metrics(&self) -> GridMetrics {
        self.grid().metrics(self.chrome.label_line_h())
    }

    /// "按时间分组" sections of the current items as delivered (sorted, possibly reversed):
    /// contiguous runs of one date bucket; the leading namespace items form a headerless run.
    /// Empty when grouping is off.
    pub(super) fn group_spans(&self) -> Vec<GroupSpan> {
        if !self.group_by_date || self.items.is_empty() {
            return Vec::new();
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64);
        let Some((y, m, d)) = fileinfo::local_civil_date(now) else {
            return Vec::new();
        };
        let today = CivilDate::new(y, m, d);
        group_spans(
            self.items
                .iter()
                .map(|it| it.local_date.map(|d| date_bucket(d, today))),
        )
    }

    /// The item layout at `width_dip`. Its rows keep their pitch at any fence height; with
    /// 「调整大小时按整行整列对齐」 the viewport shows only the whole rows (`scroll_view_h`).
    pub(super) fn layout(&self, width_dip: f32) -> ItemLayout {
        if self.scrolls_text() {
            // A note or a script fence scrolls its text: an empty grid as tall as that text
            // gives the wheel, the glide and the scrollbar their range.
            let mut grid = Grid::grouped(self.grid_metrics(), width_dip, 0, &[]);
            grid.content_height = self.tool_text_h;
            return ItemLayout::Grid(grid);
        }
        let spans = self.group_spans();
        let n = self.items.len();
        match self.row_metrics() {
            Some(rm) => ItemLayout::rows_grouped(rm, width_dip, n, &spans),
            None if self.bare() && !self.desktop_slots.is_empty() => {
                // 像桌面一样排列: the cells `arrange_desktop` placed down the columns.
                let grid = Grid::grouped(self.grid_metrics(), width_dip, n, &[]);
                ItemLayout::Grid(
                    grid.with_slots(self.desktop_slots.clone(), n)
                        .placed_freely(self.places_freely()),
                )
            }
            None => {
                ItemLayout::Grid(
                    self.grid()
                        .grid(self.chrome.label_line_h(), width_dip, n, &spans),
                )
            }
        }
    }

    /// Icon edge in DIPs for the current layout (rows use a small 20 DIP icon).
    pub(super) fn icon_dip(&self) -> f32 {
        crate::layout::icon_dip(self.layout, self.icon_size)
    }

    pub(super) fn row_metrics(&self) -> Option<RowMetrics> {
        match self.layout {
            ViewLayout::Icons => None,
            layout => Some(self.row_metrics_of(layout)),
        }
    }

    /// List or Details rows for the current row text (the icon-label font).
    fn row_metrics_of(&self, layout: ViewLayout) -> RowMetrics {
        let line_h = self.chrome.label_line_h();
        match layout {
            ViewLayout::Details => RowMetrics::details(line_h),
            _ => RowMetrics::list(line_h),
        }
    }

    /// Details header column under a client-pixel point.
    pub(super) fn header_hit(&self, x_px: i32, y_px: i32) -> Option<DetailColumn> {
        if self.layout != ViewLayout::Details || self.rolled_up {
            return None;
        }
        let scale = self.scale();
        let y = (y_px - self.content_y) as f32 / scale;
        if y < 0.0 || y >= self.row_metrics_of(ViewLayout::Details).header_h {
            return None;
        }
        let (cw, _) = self.content_size_px();
        self.columns_for(cw as f32 / scale)
            .column_at((x_px - self.content_x) as f32 / scale)
    }

    /// Details header divider under a client-pixel point (resize grip).
    pub(super) fn divider_hit(&self, x_px: i32, y_px: i32) -> Option<DetailColumn> {
        if self.layout != ViewLayout::Details || self.rolled_up {
            return None;
        }
        let scale = self.scale();
        let y = (y_px - self.content_y) as f32 / scale;
        if y < 0.0 || y >= self.row_metrics_of(ViewLayout::Details).header_h {
            return None;
        }
        let (cw, _) = self.content_size_px();
        self.columns_for(cw as f32 / scale)
            .divider_at((x_px - self.content_x) as f32 / scale)
    }

    pub(super) fn column_width(&self, col: DetailColumn) -> f32 {
        match col {
            DetailColumn::Date => self.column_widths[0],
            DetailColumn::Type => self.column_widths[1],
            DetailColumn::Size => self.column_widths[2],
            DetailColumn::Name => 0.0,
        }
    }

    pub(super) fn set_column_width(&mut self, col: DetailColumn, w: f32) {
        let w = w.clamp(DetailColumns::MIN_COL_W, DetailColumns::MAX_COL_W);
        match col {
            DetailColumn::Date => self.column_widths[0] = w,
            DetailColumn::Type => self.column_widths[1] = w,
            DetailColumn::Size => self.column_widths[2] = w,
            DetailColumn::Name => {}
        }
    }

    /// Screen rectangle of an item's whole cell (icon and name; its row in the list views):
    /// what a menu opened for it with the keyboard keeps clear of.
    pub(super) fn item_cell_rect(&self, item: ItemId) -> Option<RECT> {
        let index = self.items.iter().position(|i| i.id == item)?;
        let scale = self.scale();
        let (cw, _) = self.content_size_px();
        let layout = self.layout(cw as f32 / scale);
        let cell = layout.cell(index);
        let header = match &layout {
            ItemLayout::Grid(_) => 0.0,
            ItemLayout::Rows { metrics, .. } => metrics.header_h,
        };
        let win = crate::fence_window::shown_rect(self.hwnd);
        let (left, top) = (win.left + self.content_x, win.top + self.content_y);
        let y = cell.y - self.scroll_y + header;
        Some(RECT {
            left: left + (cell.x * scale) as i32,
            top: top + (y * scale) as i32,
            right: left + ((cell.x + cell.w) * scale) as i32,
            bottom: top + ((y + cell.h) * scale) as i32,
        })
    }

    /// Screen rectangle of an item's label (for the inline rename popup).
    pub(super) fn item_label_rect(&self, item: ItemId) -> Option<RECT> {
        let index = self.items.iter().position(|i| i.id == item)?;
        let scale = self.scale();
        let (cw, _) = self.content_size_px();
        let width_dip = cw as f32 / scale;
        let layout = self.layout(width_dip);
        let cell = layout.cell(index);
        let win = crate::fence_window::shown_rect(self.hwnd);
        let title_h = self.content_y;
        let (left, top, right, bottom) = match &layout {
            ItemLayout::Grid(grid) => {
                let r = grid.metrics.label_rect(cell);
                let top = r.y - self.scroll_y;
                (r.x, top, r.x + r.w, top + r.h)
            }
            ItemLayout::Rows { metrics, .. } => {
                let cols = self.columns_for(width_dip);
                let text_x = cols.name_x + metrics.icon + 8.0;
                // The rename box (`begin_item_rename`) centred on the row.
                let box_h = crate::rename::item_rename_h(crate::layout::row_text_scale(
                    self.chrome.label_size(),
                ));
                let top = cell.y - self.scroll_y + metrics.header_h + (metrics.row_h - box_h) / 2.0;
                (
                    text_x - 4.0,
                    top,
                    cols.name_x + cols.name_w,
                    top + metrics.row_h - 4.0,
                )
            }
        };
        let win_left = win.left + self.content_x;
        Some(RECT {
            left: win_left + (left * scale) as i32,
            top: win.top + title_h + (top * scale) as i32,
            right: win_left + (right * scale) as i32,
            bottom: win.top + title_h + (bottom * scale) as i32,
        })
    }

    /// Window height (device px) that fits every row of the grid at the current width, when
    /// auto-height applies (off while rolled or rolling).
    pub(super) fn auto_height_px(&self) -> Option<i32> {
        if !self.auto_height || self.rolled_up || self.roll_anim.is_some() {
            return None;
        }
        Some(self.fitting_height_px())
    }

    /// Window height (device px) that fits every row of the grid at the current width,
    /// regardless of the roll state (the expand animation targets it directly).
    pub(super) fn fitting_height_px(&self) -> i32 {
        self.bounded_height_px(self.wanted_height_px())
    }

    /// The height that shows every row (and the left tab column), with no room limit.
    pub(super) fn wanted_height_px(&self) -> i32 {
        let desired = self.desired_height_px(&self.current_layout());
        desired.max(self.rail_fit_height_px())
    }

    /// The item layout at the content surface's current width. A fence collapsed (or
    /// collapsing) to its left tab column uses the width it expands to.
    fn current_layout(&self) -> ItemLayout {
        let (mut cw, _) = self.content_size_px();
        if self.tabs_on_rail() && (self.rolled_up || self.roll_anim.is_some()) {
            cw = (self.expanded_w_px - self.rail_px()).max(1);
        }
        self.layout(cw as f32 / self.scale())
    }

    /// Window height (device px) that shows every row of `layout`, ignoring the work area.
    fn desired_height_px(&self, layout: &ItemLayout) -> i32 {
        // Header bands and per-group row blocks included; an empty fence keeps one row.
        let content = layout.fixed_top()
            + layout
                .content_height()
                .max(layout.top_pad() * 2.0 + layout.row_step());
        self.head_px() + (content * self.scale()).ceil() as i32 + FIT_SLACK_PX
    }

    /// Window height (device px) that shows every row of the left tab column, the free strip
    /// above them and the chevron under them; 0 without a column.
    pub(super) fn rail_fit_height_px(&self) -> i32 {
        self.rail_rows_height_px(RAIL_BOTTOM)
    }

    /// `rail_fit_height_px` with `bottom` DIPs under the last row instead of the chevron's room.
    pub(super) fn rail_rows_height_px(&self, bottom: f32) -> i32 {
        if !self.tabs_on_rail() {
            return 0;
        }
        (rail_len_for(self.tabs.len(), bottom) * self.scale()).ceil() as i32
    }

    /// Once after the left tab column appeared or grew: the expanded height (device px) that
    /// shows all of it, when the fence is shorter. Auto-height fences already fit it, and a
    /// fence is never made shorter; it can still be sized smaller by hand afterwards (the
    /// pills then shrink). A rolled fence keeps the request until it is expanded.
    pub(super) fn take_rail_growth_px(&mut self) -> Option<i32> {
        if self.rolled_up || self.roll_anim.is_some() || !std::mem::take(&mut self.rail_grow) {
            return None;
        }
        if self.auto_height || !self.tabs_on_rail() {
            // Auto height fits the column by itself.
            return None;
        }
        let want = self.bounded_height_px(self.rail_fit_height_px());
        (want > self.expanded_h_px).then_some(want)
    }

    /// Columns, rows and the fitting height of the shown items at the current width, and
    /// whether the expanded height cuts them off (the CLI's `FenceDto.fit`).
    pub(super) fn fit_report(&self) -> FitReport {
        let layout = self.current_layout();
        let desired = self.desired_height_px(&layout);
        FitReport {
            columns: layout.columns(),
            rows: layout.row_count(),
            // The fitting height shows a left tab column too; `overflow` stays about items.
            fitting_height_px: self.bounded_height_px(desired.max(self.rail_fit_height_px())),
            overflow: desired > self.expanded_h_px,
        }
    }

    /// `desired` capped so the window ends at the bottom of its monitor's work area, or starts
    /// at its top when the title row is at the bottom: the room on the side the fence grows
    /// into, with 向上展开 also only as far as the fences there can make room. The edge that stays
    /// put is the one measured from, so the cap does not grow as the fence does.
    fn bounded_height_px(&self, desired: i32) -> i32 {
        let rect = crate::fence_window::shown_rect(self.hwnd);
        monitors::query(monitors::monitor_from_window(self.hwnd)).map_or(desired, |monitor| {
            let (above, below) = if self.expand_upward {
                self.expand_rooms_px()
            } else {
                // With pushNeighbors: as far as the fences below can slide down. A hover peek
                // opens in full, above the fences it cannot push.
                let floor = self.title_h_px() + 40;
                (
                    i32::MAX,
                    self.kept_room_below_px()
                        .filter(|_| !self.peeking)
                        .map_or(i32::MAX, |room| room.max(floor)),
                )
            };
            let (top, bottom) = if self.row_bottom {
                let top = rect.bottom.saturating_sub(above);
                (monitor.work_area.top.max(top), rect.bottom)
            } else {
                let bottom = rect.top.saturating_add(below);
                (rect.top, monitor.work_area.bottom.min(bottom))
            };
            bounded_auto_height(desired, self.title_h_px(), top, bottom)
        })
    }

    /// Room (device px) above the window's bottom edge and below its top edge for a fence with
    /// 向上展开: as far as the fences there can make room (they slide with
    /// `rollUp.pushNeighbors`, else the fence stops short of them) inside the work area. Kept
    /// by the App's push pass; the bare work area until it has run.
    pub(super) fn expand_rooms_px(&self) -> (i32, i32) {
        let kept = self
            .behavior
            .expand_room
            .borrow()
            .iter()
            .find(|(hwnd, ..)| *hwnd == self.hwnd)
            .map(|&(_, above, below)| (above, below));
        kept.unwrap_or_else(|| {
            let rect = crate::fence_window::shown_rect(self.hwnd);
            monitors::query(monitors::monitor_from_window(self.hwnd))
                .map_or((i32::MAX, i32::MAX), |m| {
                    (rect.bottom - m.work_area.top, m.work_area.bottom - rect.top)
                })
        })
    }

    /// Room (device px) below the window's top edge that the App's push pass kept for it: with
    /// `rollUp.pushNeighbors`, down to where the fences below it, slid as far as they can go,
    /// leave room. None when the pass keeps none (pushNeighbors off, a fence in the user's
    /// hands): the fence then opens as tall as it is saved.
    pub(super) fn kept_room_below_px(&self) -> Option<i32> {
        self.behavior
            .expand_room
            .borrow()
            .iter()
            .find(|(hwnd, ..)| *hwnd == self.hwnd)
            .map(|&(_, _, below)| below)
    }

    /// Test hook: the screen point over the first item whose name contains `name` (a drop onto
    /// a folder item), the inverse of `hit_item`; `tab:<title>` is the middle of that tab's pill.
    #[cfg(debug_assertions)]
    pub(super) fn test_item_point(&self, name: &str) -> Option<(i32, i32)> {
        let scale = self.scale();
        let r = crate::fence_window::shown_rect(self.hwnd);
        if let Some(title) = name.strip_prefix("tab:") {
            let i = self.tabs.iter().position(|t| t.title.contains(title))?;
            let (x, w) = *self.tab_rects().get(i)?;
            let along = ((x + w / 2.0) * scale).round() as i32;
            return Some(if self.tabs_on_rail() {
                (r.left + self.content_x / 2, r.top + along)
            } else {
                (r.left + along, r.top + self.title_h_px() / 2)
            });
        }
        let i = self.items.iter().position(|it| it.name.contains(name))?;
        let (cw, _) = self.content_size_px();
        let layout = self.layout(cw as f32 / scale);
        let c = layout.cell(i);
        let x = c.x + c.w.min(40.0) / 2.0;
        let y = c.y - self.scroll_y + layout.fixed_top() + c.h / 2.0;
        Some((
            r.left + self.content_x + (x * scale).round() as i32,
            r.top + self.content_y + (y * scale).round() as i32,
        ))
    }

    /// Item index under a client-pixel point, if any.
    pub(super) fn hit_item(&self, x_px: i32, y_px: i32) -> Option<usize> {
        if self.rolled_up {
            return None;
        }
        let scale = self.scale();
        let title_h = self.content_y;
        if y_px < title_h || x_px < self.content_x {
            return None;
        }
        let (cw, _) = self.content_size_px();
        let layout = self.layout(cw as f32 / scale);
        let x = (x_px - self.content_x) as f32 / scale;
        let y = (y_px - title_h) as f32 / scale - layout.fixed_top();
        // Under the whole rows (「调整大小时按整行整列对齐」) nothing shows.
        if y < 0.0 || y >= self.scroll_view_h(&layout) {
            return None;
        }
        let y = y + self.scroll_y;
        if self.bare() {
            // 像桌面一样排列: as on the Windows desktop only the icon and its name take a press;
            // the rest of a cell is desktop, where a press starts a rectangle (a press at the top
            // right of a cell dragged the whole 「桌面」 fence, then the Recycle Bin).
            let m = self.grid_metrics();
            return layout
                .hit_test(x, y, self.items.len())
                .filter(|&i| on_icon_or_label(&m, &layout.cell(i), x, y));
        }
        if self.item_motion.is_empty() {
            return layout.hit_test(x, y, self.items.len());
        }
        // Layout motion running: every item is hit where it is drawn, in paint order (later
        // items on top), so a gliding item passing under a resting one does not take its click.
        let now = Instant::now();
        (0..self.items.len()).rev().find(|&i| {
            let mut c = layout.cell(i);
            if let Some(m) = self.item_motion.get(&self.items[i].id) {
                (c.x, c.y) = (m.x.value_at(now), m.y.value_at(now));
            }
            c.contains(x, y)
        })
    }
}

/// The icon of `cell` (with a few DIPs of slack) or the name under it, in content DIPs.
pub(super) fn on_icon_or_label(m: &GridMetrics, cell: &CellRect, x: f32, y: f32) -> bool {
    const SLACK: f32 = 4.0;
    let left = cell.x + (cell.w - m.icon) / 2.0 - SLACK;
    let top = cell.y + m.icon_top - SLACK;
    let size = m.icon + 2.0 * SLACK;
    let on_icon = x >= left && x < left + size && y >= top && y < top + size;
    on_icon || m.label_rect(*cell).contains(x, y)
}
