//! Client-pixel hit-testing and item / grid / details-column geometry helpers.

use super::*;

/// Details header divider being dragged.
pub(super) struct ColDrag {
    pub(super) col: DetailColumn,
    pub(super) start_x_px: i32,
    pub(super) start_w: f32,
}

pub(super) fn bounded_auto_height(desired: i32, title_height: i32, top: i32, bottom: i32) -> i32 {
    // Preserve a client pixel even when the title was dragged almost offscreen. In normal
    // positions, excess rows stay accessible via the existing scrollbar above the taskbar.
    desired.min(bottom.saturating_sub(top).max(title_height + 1))
}

impl FenceViewState {
    /// Is the client-pixel point on item `index`'s label text (not its icon)?
    pub(super) fn label_hit(&self, x_px: i32, y_px: i32, index: usize) -> bool {
        let Some(id) = self.items.get(index).map(|i| i.id) else {
            return false;
        };
        let Some(r) = self.item_label_rect(id) else {
            return false;
        };
        let win = window::window_rect(self.hwnd);
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
                DetailColumn::Type => item
                    .type_label
                    .get_or_insert_with(|| {
                        fileinfo::type_name(&item.path, item.is_folder).unwrap_or_default()
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

    /// Is the client-pixel point on the portal "up" button? A tab strip in the title row has it
    /// on the shown tab's folder glyph (`FenceChrome::tab_up_box`); a title column of tabs at the
    /// end of the shown tab (`rail_up_box`); a single title column above its title; a single
    /// title row on its folder glyph (`FenceChrome::lone_up_box`), or at the far left while the
    /// glyph is hidden.
    pub(super) fn up_button_at(&self, x_px: i32, y_px: i32) -> bool {
        if !self.deco.up_button {
            return false;
        }
        let scale = self.scale();
        let r = if self.title_row_strip() {
            // The pill is where it is drawn (sliding, dragged); a scrolled strip's cut-off part
            // takes no click (`tab_at`). A tab without its folder glyph has no button.
            let i = self.active_index();
            let Some(t) = self.tabs.get(i).filter(|t| t.up_button && t.folder_icon) else {
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
                badge.as_deref(),
            )
        } else if self.tabs_on_rail() {
            match self.rail_up_box() {
                Some(r) => r,
                None => return false,
            }
        } else if self.title_column() {
            TitleDeco::column_up_box(self.rail_w(), self.column_top())
        } else if self.deco.folder_icon {
            // The title moves aside for the rolled item count, as drawn.
            let width = self.chrome_panel.size_px().0 as f32 / scale;
            let reserve = if self.roll_target() && !self.behavior.hide_count_and_chevron.get() {
                pecofence_render::fence_chrome::header_count_width(width, 1, self.deco.title_x())
            } else {
                0.0
            };
            self.chrome.lone_up_box(
                &self.title,
                self.style.title_size,
                self.title_align(),
                width,
                reserve,
                self.theme.title_height,
                self.chevron_leads(),
            )
        } else {
            TitleDeco::up_box(self.theme.title_height)
        };
        // Boxes are measured from the title column's left, or the title row's top.
        let (x_px, y_px) = if self.title_column() {
            (x_px - self.column_x_px(), y_px)
        } else {
            (x_px, y_px - self.title_top_px())
        };
        let (x, y) = (x_px as f32 / scale, y_px as f32 / scale);
        x >= r.left && x < r.right && y >= r.top && y < r.bottom
    }

    /// The portal up button in a title column (DIPs): at the end of the shown tab's row, as
    /// drawn (`TitleDeco::rail_up_box`). A column the fence is collapsed to has none.
    pub(super) fn rail_up_box(&self) -> Option<Rect> {
        if self.column_collapsed() {
            return None;
        }
        let i = self.active_index();
        let (_, h) = *self.tab_rects().get(i)?;
        let y = *self.tab_draw_xs(Instant::now()).get(i)?;
        if self.vertical_tabs() {
            return Some(TitleDeco::vertical_up_box(TitleDeco::vertical_pill(
                &self.theme,
                0.0,
                self.rail_w(),
                y,
                h,
            )));
        }
        let row = Rect::from_xywh(
            RAIL_ROW_INSET,
            y,
            (self.rail_w() - RAIL_ROW_INSET - RAIL_ROW_RIGHT).max(0.0),
            h,
        );
        Some(TitleDeco::rail_up_box(row))
    }

    /// Is the client-pixel point in the title row (the title column beside one)? Its
    /// caption and its controls (tabs, the chevron box) are one row for the hover.
    pub(super) fn title_row_at(&self, x_px: i32, y_px: i32) -> bool {
        if self.title_column() {
            self.in_column(x_px)
        } else {
            self.in_title_row(y_px)
        }
    }

    /// Is the client-pixel point on the roll-up chevron button (right end of the title row)?
    pub(super) fn chevron_at(&self, x_px: i32, y_px: i32) -> bool {
        let scale = self.scale();
        let (w_px, h_px) = self.chrome_panel.size_px();
        // Under the tabs of a title column (the fence's title bar). Collapsed to the column
        // there is none: a tab click expands the fence. Hidden by the user, its spot is title.
        if (self.column_collapsed() && self.tabs_on_rail())
            || self.behavior.hide_count_and_chevron.get()
        {
            return false;
        }
        let r = if self.title_column() {
            // A single fence's chevron stays under the start of its title, where a fence too
            // short for the title draws it.
            let rows_end = if self.tabs_on_rail() {
                self.tab_rects()
                    .iter()
                    .map(|(y, h)| y + h)
                    .fold(0.0, f32::max)
            } else {
                self.column_title_start()
            };
            if self.chevron_leads() {
                TitleDeco::rail_chevron_box_top(self.rail_w())
            } else {
                TitleDeco::rail_chevron_box(self.rail_w(), h_px as f32 / scale, rows_end)
            }
        } else if self.chevron_leads() {
            let up_w = if self.title_row_strip() {
                0.0
            } else {
                self.deco.lead_up_w()
            };
            TitleDeco::lead_chevron_box(self.theme.title_height, up_w)
        } else {
            TitleDeco::chevron_box(w_px as f32 / scale, self.theme.title_height)
        };
        let (x_px, y_px) = if self.title_column() {
            (x_px - self.column_x_px(), y_px)
        } else {
            (x_px, y_px - self.title_top_px())
        };
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
        }
    }

    pub(super) fn grid_metrics(&self) -> GridMetrics {
        GridMetrics::for_icon_size(self.icon_size, self.label_lines)
            .with_line_h(self.chrome.label_line_h())
            .with_spacing(self.spacing)
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
        let spans = self.group_spans();
        let n = self.items.len();
        match self.row_metrics() {
            Some(rm) => ItemLayout::rows_grouped(rm, width_dip, n, &spans),
            None if self.bare() && !self.desktop_slots.is_empty() => {
                // 像桌面一样排列: the cells `arrange_desktop` placed down the columns.
                let grid = Grid::grouped(self.grid_metrics(), width_dip, n, &[]);
                ItemLayout::Grid(grid.with_slots(self.desktop_slots.clone(), n))
            }
            None => {
                let grid = Grid::grouped(self.grid_metrics(), width_dip, n, &spans);
                ItemLayout::Grid(grid.centred(self.icon_align == IconAlign::Center))
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

    /// Screen rectangle of an item's label (for the inline rename popup).
    pub(super) fn item_label_rect(&self, item: ItemId) -> Option<RECT> {
        let index = self.items.iter().position(|i| i.id == item)?;
        let scale = self.scale();
        let (cw, _) = self.content_size_px();
        let width_dip = cw as f32 / scale;
        let layout = self.layout(width_dip);
        let cell = layout.cell(index);
        let win = window::window_rect(self.hwnd);
        let title_h = self.content_y;
        let (left, top, right, bottom) = match &layout {
            ItemLayout::Grid(grid) => {
                let label_top = cell.y - self.scroll_y
                    + grid.metrics.icon_top
                    + self.icon_size as f32
                    + grid.metrics.label_gap;
                let label_h = grid.metrics.line_h * self.label_lines as f32;
                (cell.x, label_top, cell.x + cell.w, label_top + label_h)
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

    /// [`Self::fitting_height_px`] for a window `window_w` px wide (a title column opening
    /// narrower than the fence).
    pub(super) fn fitting_height_at_px(&self, window_w: i32) -> i32 {
        let content_w = (window_w - self.rail_px()).max(1);
        let layout = self.layout(content_w as f32 / self.scale());
        self.bounded_height_px(
            self.desired_height_px(&layout)
                .max(self.rail_fit_height_px()),
        )
    }

    /// The height that shows every row (and the left tab column), with no room limit.
    pub(super) fn wanted_height_px(&self) -> i32 {
        let desired = self.desired_height_px(&self.current_layout());
        desired.max(self.rail_fit_height_px())
    }

    /// The item layout at the content surface's current width. A fence collapsed (or
    /// collapsing) narrower than it expands uses the width it expands to.
    fn current_layout(&self) -> ItemLayout {
        let (mut cw, _) = self.content_size_px();
        if self.collapses_width() && (self.rolled_up || self.roll_anim.is_some()) {
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
        self.head_px() + self.foot_px() + (content * self.scale()).ceil() as i32 + FIT_SLACK_PX
    }

    /// Window height (device px) that shows every row of the title column, the free strip
    /// above them and the chevron under them; 0 without a column.
    pub(super) fn rail_fit_height_px(&self) -> i32 {
        self.rail_rows_height_px(RAIL_BOTTOM)
    }

    /// `rail_fit_height_px` with `bottom` DIPs under the last row instead of the chevron's room.
    pub(super) fn rail_rows_height_px(&self, bottom: f32) -> i32 {
        if !self.tabs_on_rail() {
            return 0;
        }
        (rail_len_of(&self.tab_natural_widths(), bottom) * self.scale()).ceil() as i32
    }

    /// Once after the title column appeared or grew: the expanded height (device px) that
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
            // The fitting height shows a title column too; `overflow` stays about items.
            fitting_height_px: self.bounded_height_px(desired.max(self.rail_fit_height_px())),
            overflow: desired > self.expanded_h_px,
        }
    }

    /// `desired` capped so the window ends at the bottom of its monitor's work area, or starts
    /// at its top when the title row is at the bottom: the room on the side the fence grows
    /// into, with 向上展开 also only as far as the fences there can make room. The edge that stays
    /// put is the one measured from, so the cap does not grow as the fence does.
    fn bounded_height_px(&self, desired: i32) -> i32 {
        let rect = window::window_rect(self.hwnd);
        monitors::query(monitors::monitor_from_window(self.hwnd)).map_or(desired, |monitor| {
            let (above, below) = if self.chooses_side() {
                self.expand_rooms_px()
            } else {
                (i32::MAX, i32::MAX)
            };
            let (top, bottom) = if self.row_at_bottom() {
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
            let rect = window::window_rect(self.hwnd);
            monitors::query(monitors::monitor_from_window(self.hwnd))
                .map_or((i32::MAX, i32::MAX), |m| {
                    (rect.bottom - m.work_area.top, m.work_area.bottom - rect.top)
                })
        })
    }

    /// Test hook: the screen point over the first item whose name contains `name` (a drop onto
    /// a folder item), the inverse of `hit_item`; `tab:<title>` is the middle of that tab's pill.
    #[cfg(debug_assertions)]
    pub(super) fn test_item_point(&self, name: &str) -> Option<(i32, i32)> {
        let scale = self.scale();
        let r = window::window_rect(self.hwnd);
        if let Some(title) = name.strip_prefix("tab:") {
            let i = self.tabs.iter().position(|t| t.title.contains(title))?;
            let (x, w) = *self.tab_rects().get(i)?;
            let along = ((x + w / 2.0) * scale).round() as i32;
            return Some(if self.tabs_on_rail() {
                (
                    r.left + self.column_x_px() + self.rail_px() / 2,
                    r.top + along,
                )
            } else {
                (
                    r.left + along,
                    r.top + self.title_top_px() + self.title_h_px() / 2,
                )
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
