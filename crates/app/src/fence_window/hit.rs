//! Client-pixel hit-testing and item / grid / details-column geometry helpers.

use super::*;

/// Details header divider being dragged.
pub(super) struct ColDrag {
    pub(super) col: DetailColumn,
    pub(super) start_x_px: i32,
    pub(super) start_w: f32,
}

pub(super) fn bounded_auto_height(
    desired: i32,
    title_height: i32,
    top: i32,
    work_bottom: i32,
) -> i32 {
    // Preserve a client pixel even when the title was dragged almost offscreen. In normal
    // positions, excess rows stay accessible via the existing scrollbar above the taskbar.
    desired.min(work_bottom.saturating_sub(top).max(title_height + 1))
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
        let mut w = pecofence_render::text::measure_width(caption, fmt) + 12.0;
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
            w = w.max(pecofence_render::text::measure_width(&label, fmt));
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
        if x_px < 0 || x_px >= cw {
            return false;
        }
        let y = (y_px - self.title_h_px()) as f32 / self.scale();
        y >= 0.0 && y < RowMetrics::details().header_h
    }

    /// Is the client-pixel point on the portal "up" button?
    pub(super) fn up_button_at(&self, x_px: i32, y_px: i32) -> bool {
        if !self.deco.up_button || self.tabs.len() > 1 {
            return false;
        }
        let scale = self.scale();
        let (x, y) = (x_px as f32 / scale, y_px as f32 / scale);
        let r = TitleDeco::up_box(self.theme.title_height);
        x >= r.left && x < r.right && y >= r.top && y < r.bottom
    }

    /// Is the client-pixel point on the roll-up chevron button (right end of the title row)?
    pub(super) fn chevron_at(&self, x_px: i32, y_px: i32) -> bool {
        let scale = self.scale();
        let (w_px, _) = self.chrome_panel.size_px();
        let r = TitleDeco::chevron_box(w_px as f32 / scale, self.theme.title_height);
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

    pub(super) fn layout(&self, width_dip: f32) -> ItemLayout {
        let spans = self.group_spans();
        let n = self.items.len();
        match self.layout {
            ViewLayout::Icons => {
                ItemLayout::Grid(Grid::grouped(self.grid_metrics(), width_dip, n, &spans))
            }
            ViewLayout::List => ItemLayout::rows_grouped(RowMetrics::list(), width_dip, n, &spans),
            ViewLayout::Details => {
                ItemLayout::rows_grouped(RowMetrics::details(), width_dip, n, &spans)
            }
        }
    }

    /// Icon edge in DIPs for the current layout (rows use a small 20 DIP icon).
    pub(super) fn icon_dip(&self) -> f32 {
        match self.layout {
            ViewLayout::Icons => self.icon_size as f32,
            ViewLayout::List => RowMetrics::list().icon,
            ViewLayout::Details => RowMetrics::details().icon,
        }
    }

    pub(super) fn row_metrics(&self) -> Option<RowMetrics> {
        match self.layout {
            ViewLayout::Icons => None,
            ViewLayout::List => Some(RowMetrics::list()),
            ViewLayout::Details => Some(RowMetrics::details()),
        }
    }

    /// Details header column under a client-pixel point.
    pub(super) fn header_hit(&self, x_px: i32, y_px: i32) -> Option<DetailColumn> {
        if self.layout != ViewLayout::Details || self.rolled_up {
            return None;
        }
        let scale = self.scale();
        let title_h = self.title_h_px();
        let y = (y_px - title_h) as f32 / scale;
        if y < 0.0 || y >= RowMetrics::details().header_h {
            return None;
        }
        let (cw, _) = self.content_size_px();
        self.columns_for(cw as f32 / scale)
            .column_at(x_px as f32 / scale)
    }

    /// Details header divider under a client-pixel point (resize grip).
    pub(super) fn divider_hit(&self, x_px: i32, y_px: i32) -> Option<DetailColumn> {
        if self.layout != ViewLayout::Details || self.rolled_up {
            return None;
        }
        let scale = self.scale();
        let y = (y_px - self.title_h_px()) as f32 / scale;
        if y < 0.0 || y >= RowMetrics::details().header_h {
            return None;
        }
        let (cw, _) = self.content_size_px();
        self.columns_for(cw as f32 / scale)
            .divider_at(x_px as f32 / scale)
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
        let title_h = self.title_h_px();
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
                let top = cell.y - self.scroll_y + metrics.header_h + 2.0;
                (
                    text_x - 4.0,
                    top,
                    cols.name_x + cols.name_w,
                    top + metrics.row_h - 4.0,
                )
            }
        };
        Some(RECT {
            left: win.left + (left * scale) as i32,
            top: win.top + title_h + (top * scale) as i32,
            right: win.left + (right * scale) as i32,
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
        let scale = self.scale();
        let (cw, _) = self.content_size_px();
        let layout = self.layout(cw as f32 / scale);
        // Header bands and per-group row blocks included; an empty fence keeps one row.
        let content = layout.fixed_top()
            + layout
                .content_height()
                .max(layout.top_pad() * 2.0 + layout.row_step());
        let desired = self.title_h_px() + (content * scale).ceil() as i32 + 2;
        let rect = window::window_rect(self.hwnd);
        monitors::query(monitors::monitor_from_window(self.hwnd)).map_or(desired, |monitor| {
            bounded_auto_height(
                desired,
                self.title_h_px(),
                rect.top,
                monitor.work_area.bottom,
            )
        })
    }

    /// Item index under a client-pixel point, if any.
    pub(super) fn hit_item(&self, x_px: i32, y_px: i32) -> Option<usize> {
        if self.rolled_up {
            return None;
        }
        let scale = self.scale();
        let title_h = self.title_h_px();
        if y_px < title_h {
            return None;
        }
        let (cw, _) = self.content_size_px();
        let layout = self.layout(cw as f32 / scale);
        let x = x_px as f32 / scale;
        let y = (y_px - title_h) as f32 / scale - layout.fixed_top();
        if y < 0.0 {
            return None;
        }
        layout.hit_test(x, y + self.scroll_y, self.items.len())
    }
}
