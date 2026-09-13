//! Icon grid layout in DIPs: cell geometry, hit testing, scrolling extents.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridMetrics {
    pub icon: f32,
    pub cell_w: f32,
    pub cell_h: f32,
    pub pad_x: f32,
    pub pad_y: f32,
    pub label_lines: u8,
    pub line_h: f32,
    /// Top pad above the icon and gap between icon and label; the renderer positions the icon
    /// and label from these so a spacing change cannot push the label out of the cell.
    pub icon_top: f32,
    pub label_gap: f32,
}

impl GridMetrics {
    pub fn for_icon_size(icon: u32, label_lines: u8) -> Self {
        let icon = icon as f32;
        let line_h = 16.0;
        Self {
            icon,
            // 4 px grid: icon + 32 wide (32→64, 48→80, 64→96, 96→128) so a 72 DIP label holds
            // six 12 px CJK glyphs (the 96 px desktop "large" size gets a 120 DIP label); no
            // side padding, so the first icon's left edge lands on the title's x = 16. 8 top
            // pad, 4 icon→label gap, two 16 px label lines, 4 bottom.
            cell_w: icon + 32.0,
            cell_h: icon + 8.0 + 4.0 + line_h * label_lines as f32 + 4.0,
            pad_x: 0.0,
            pad_y: 8.0,
            label_lines,
            line_h,
            icon_top: 8.0,
            label_gap: 4.0,
        }
    }

    /// Label line height from the system icon-title font (Accessibility text scaling): the
    /// cell grows with it, rounded up to the 4 px grid so cells stay on the grid (16 for the
    /// default 12 px font, i.e. a no-op). Apply before `with_spacing`.
    pub fn with_line_h(mut self, line_h: f32) -> Self {
        let line_h = (line_h.max(1.0) / 4.0).ceil() * 4.0;
        self.line_h = line_h;
        self.cell_h = self.icon + 8.0 + 4.0 + line_h * self.label_lines as f32 + 4.0;
        self
    }

    /// Fences "icon spacing": compact pulls cells in by 12/8 DIPs, loose pushes them out by
    /// 16/12 (4 px grid kept).
    pub fn with_spacing(mut self, spacing: pecofence_core::Spacing) -> Self {
        match spacing {
            pecofence_core::Spacing::Compact => {
                self.cell_w -= 12.0;
                // 4 + icon + 2 + label + 2 == cell_h: the label still ends inside the cell.
                self.cell_h -= 8.0;
                self.icon_top = 4.0;
                self.label_gap = 2.0;
            }
            pecofence_core::Spacing::Normal => {}
            pecofence_core::Spacing::Loose => {
                self.cell_w += 16.0;
                self.cell_h += 12.0;
            }
        }
        self
    }

    /// Outer fence width in DIPs holding exactly `cols` icon columns; this is the value column
    /// snapping converges to, so new fences must be created at it.
    pub fn width_for_columns(&self, cols: u32) -> f32 {
        self.pad_x * 2.0 + cols.max(1) as f32 * self.cell_w
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl CellRect {
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && py >= self.y && px < self.x + self.w && py < self.y + self.h
    }
}

/// Lays out `count` items inside a content area of `width` DIPs (below the title bar), with
/// vertical scroll offset `scroll_y`. Returns cell rects in content-local DIPs (already scrolled).
pub struct Grid {
    pub metrics: GridMetrics,
    pub columns: usize,
    #[allow(dead_code)]
    pub rows: usize,
    pub content_height: f32,
}

impl Grid {
    pub fn new(metrics: GridMetrics, width: f32, count: usize) -> Self {
        let usable = (width - metrics.pad_x * 2.0).max(metrics.cell_w);
        let columns = ((usable / metrics.cell_w).floor() as usize).max(1);
        let rows = count.div_ceil(columns);
        let content_height = metrics.pad_y * 2.0 + rows as f32 * metrics.cell_h;
        Self {
            metrics,
            columns,
            rows,
            content_height,
        }
    }

    /// Rect of item `index` in content coordinates (before scrolling).
    pub fn cell(&self, index: usize) -> CellRect {
        let col = index % self.columns;
        let row = index / self.columns;
        CellRect {
            x: self.metrics.pad_x + col as f32 * self.metrics.cell_w,
            y: self.metrics.pad_y + row as f32 * self.metrics.cell_h,
            w: self.metrics.cell_w,
            h: self.metrics.cell_h,
        }
    }

    /// Index of the item under a content-local point (already adjusted for scroll).
    pub fn hit_test(&self, x: f32, y: f32, count: usize) -> Option<usize> {
        if x < self.metrics.pad_x || y < self.metrics.pad_y {
            return None;
        }
        let col = ((x - self.metrics.pad_x) / self.metrics.cell_w).floor() as usize;
        let row = ((y - self.metrics.pad_y) / self.metrics.cell_h).floor() as usize;
        if col >= self.columns {
            return None;
        }
        let idx = row * self.columns + col;
        (idx < count && self.cell(idx).contains(x, y)).then_some(idx)
    }

    /// Maximum scroll offset for a viewport of `view_height` DIPs.
    pub fn max_scroll(&self, view_height: f32) -> f32 {
        (self.content_height - view_height).max(0.0)
    }

    /// Insertion index for a drop at content-local point (nearest cell boundary).
    pub fn insertion_index(&self, x: f32, y: f32, count: usize) -> usize {
        if count == 0 {
            return 0;
        }
        let row = (((y - self.metrics.pad_y) / self.metrics.cell_h)
            .floor()
            .max(0.0)) as usize;
        let colf = ((x - self.metrics.pad_x) / self.metrics.cell_w).max(0.0);
        let col = colf.round() as usize;
        (row * self.columns + col.min(self.columns)).min(count)
    }
}

/// Row metrics for the List / Details layouts (DIPs). Rows are 28 tall with a 20 DIP icon;
/// Details adds a 28 DIP header that stays put while the rows scroll.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowMetrics {
    pub row_h: f32,
    pub icon: f32,
    pub header_h: f32,
    pub pad_y: f32,
}

impl RowMetrics {
    pub const fn list() -> Self {
        Self {
            row_h: 28.0,
            icon: 20.0,
            header_h: 0.0,
            pad_y: 4.0,
        }
    }

    pub const fn details() -> Self {
        Self {
            row_h: 28.0,
            icon: 20.0,
            header_h: 28.0,
            pad_y: 4.0,
        }
    }
}

/// Column split of the details view for a content width (DIPs). Narrow fences drop the type,
/// then the date column, so the name always keeps at least ~120 DIPs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DetailColumns {
    pub name_x: f32,
    pub name_w: f32,
    pub date: Option<(f32, f32)>,
    pub type_: Option<(f32, f32)>,
    pub size: Option<(f32, f32)>,
}

impl DetailColumns {
    pub const DATE_W: f32 = 136.0;
    pub const TYPE_W: f32 = 108.0;
    pub const SIZE_W: f32 = 76.0;
    pub const GUTTER: f32 = 8.0;

    pub const DEFAULT_WIDTHS: [f32; 3] = [Self::DATE_W, Self::TYPE_W, Self::SIZE_W];
    pub const MIN_COL_W: f32 = 40.0;
    pub const MAX_COL_W: f32 = 480.0;
    /// Half-width of the grab zone on a column's left edge (DIPs).
    pub const DIVIDER_GRIP: f32 = 5.0;

    #[cfg(test)]
    pub fn for_width(width: f32) -> Self {
        Self::for_width_with(width, Self::DEFAULT_WIDTHS, [true; 3])
    }

    /// `widths` = user-sized (修改日期, 类型, 大小) column widths; `visible` = which of them the
    /// user shows (header context menu). A hidden column is neither placed nor reserved.
    pub fn for_width_with(width: f32, widths: [f32; 3], visible: [bool; 3]) -> Self {
        let clamp = |w: f32| w.clamp(Self::MIN_COL_W, Self::MAX_COL_W);
        let (date_w, type_w, size_w) = (clamp(widths[0]), clamp(widths[1]), clamp(widths[2]));
        let name_x = 8.0;
        let min_name = 120.0;
        let mut right = width - Self::GUTTER;
        let mut size = None;
        let mut date = None;
        let mut type_ = None;
        // Drop order when narrow: type first, then date, then size; Explorer order on screen
        // is 名称, 修改日期, 类型, 大小, so lay the survivors out right-to-left in that order.
        // Each column only costs its width (+ gutter after the first) when it is shown.
        let mut avail = right - name_x - min_name;
        let mut include = |shown: bool, w: f32| -> bool {
            if !shown {
                return false;
            }
            let cost = if avail < right - name_x - min_name {
                w + Self::GUTTER
            } else {
                w
            };
            if avail >= cost {
                avail -= cost;
                true
            } else {
                false
            }
        };
        let want_size = include(visible[2], size_w);
        let want_date = include(visible[0], date_w);
        let want_type = include(visible[1], type_w);
        if want_size {
            size = Some((right - size_w, size_w));
            right -= size_w + Self::GUTTER;
        }
        if want_type {
            type_ = Some((right - type_w, type_w));
            right -= type_w + Self::GUTTER;
        }
        if want_date {
            date = Some((right - date_w, date_w));
            right -= date_w + Self::GUTTER;
        }
        Self {
            name_x,
            name_w: (right - name_x).max(40.0),
            date,
            type_,
            size,
        }
    }

    /// The column whose left-edge divider is under a content-local x (header resize grip).
    /// Dragging that divider resizes this column (Explorer resizes the column to the LEFT of a
    /// divider; here the name column is elastic, so the right-hand column takes the drag).
    pub fn divider_at(&self, x: f32) -> Option<DetailColumn> {
        let near = |c: Option<(f32, f32)>| {
            c.is_some_and(|(cx, _)| (x - (cx - Self::GUTTER / 2.0)).abs() <= Self::DIVIDER_GRIP)
        };
        if near(self.date) {
            Some(DetailColumn::Date)
        } else if near(self.type_) {
            Some(DetailColumn::Type)
        } else if near(self.size) {
            Some(DetailColumn::Size)
        } else {
            None
        }
    }

    /// Which column a content-local x falls in (header hit test).
    pub fn column_at(&self, x: f32) -> Option<DetailColumn> {
        let hit = |c: Option<(f32, f32)>| c.is_some_and(|(cx, cw)| x >= cx && x < cx + cw);
        if hit(self.size) {
            Some(DetailColumn::Size)
        } else if hit(self.type_) {
            Some(DetailColumn::Type)
        } else if hit(self.date) {
            Some(DetailColumn::Date)
        } else if x >= self.name_x && x < self.name_x + self.name_w {
            Some(DetailColumn::Name)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetailColumn {
    Name,
    Date,
    Type,
    Size,
}

/// The one layout abstraction fence windows use: an icon grid, or rows (List / Details).
pub enum ItemLayout {
    Grid(Grid),
    Rows {
        metrics: RowMetrics,
        width: f32,
        count: usize,
    },
}

impl ItemLayout {
    pub fn rows(metrics: RowMetrics, width: f32, count: usize) -> Self {
        Self::Rows {
            metrics,
            width,
            count,
        }
    }

    pub fn columns(&self) -> usize {
        match self {
            Self::Grid(g) => g.columns,
            Self::Rows { .. } => 1,
        }
    }

    pub fn row_count(&self) -> usize {
        match self {
            Self::Grid(g) => g.rows,
            Self::Rows { count, .. } => *count,
        }
    }

    /// Scrollable content height, excluding a fixed details header.
    pub fn content_height(&self) -> f32 {
        match self {
            Self::Grid(g) => g.content_height,
            Self::Rows { metrics, count, .. } => {
                metrics.pad_y * 2.0 + *count as f32 * metrics.row_h
            }
        }
    }

    /// Height of the part that does not scroll (details header).
    pub fn fixed_top(&self) -> f32 {
        match self {
            Self::Grid(_) => 0.0,
            Self::Rows { metrics, .. } => metrics.header_h,
        }
    }

    /// One wheel notch / one keyboard row.
    pub fn row_step(&self) -> f32 {
        match self {
            Self::Grid(g) => g.metrics.cell_h,
            Self::Rows { metrics, .. } => metrics.row_h,
        }
    }

    pub fn top_pad(&self) -> f32 {
        match self {
            Self::Grid(g) => g.metrics.pad_y,
            Self::Rows { metrics, .. } => metrics.pad_y,
        }
    }

    /// Rect of item `index` in scrollable-content coordinates (below the fixed header).
    pub fn cell(&self, index: usize) -> CellRect {
        match self {
            Self::Grid(g) => g.cell(index),
            Self::Rows { metrics, width, .. } => CellRect {
                x: 0.0,
                y: metrics.pad_y + index as f32 * metrics.row_h,
                w: *width,
                h: metrics.row_h,
            },
        }
    }

    /// Insertion index (0..=count) for a drop at a scrollable-content point.
    pub fn insertion_index(&self, x: f32, y: f32, count: usize) -> usize {
        match self {
            Self::Grid(g) => g.insertion_index(x, y, count),
            Self::Rows { metrics, .. } => {
                let row = ((y - metrics.pad_y) / metrics.row_h).round().max(0.0) as usize;
                row.min(count)
            }
        }
    }

    /// 2-DIP insertion caret before display index `index` (`index == count` = after the last
    /// item), in scrollable-content coordinates.
    pub fn insertion_caret(&self, index: usize, count: usize) -> CellRect {
        match self {
            Self::Grid(g) => {
                if count == 0 {
                    let c = g.cell(0);
                    return CellRect {
                        x: c.x,
                        y: c.y,
                        w: 2.0,
                        h: c.h,
                    };
                }
                if index < count || !index.is_multiple_of(g.columns) {
                    let c = g.cell(index);
                    CellRect {
                        x: (c.x - 1.0).max(0.0),
                        y: c.y,
                        w: 2.0,
                        h: c.h,
                    }
                } else {
                    // After a full last row: right edge of the last cell.
                    let c = g.cell(count - 1);
                    CellRect {
                        x: c.x + c.w - 1.0,
                        y: c.y,
                        w: 2.0,
                        h: c.h,
                    }
                }
            }
            Self::Rows { metrics, width, .. } => {
                let y = if index < count {
                    metrics.pad_y + index as f32 * metrics.row_h
                } else {
                    metrics.pad_y + count as f32 * metrics.row_h
                };
                CellRect {
                    x: 0.0,
                    y: y - 1.0,
                    w: *width,
                    h: 2.0,
                }
            }
        }
    }

    /// Index under a scrollable-content point (already adjusted for scroll and header).
    pub fn hit_test(&self, x: f32, y: f32, count: usize) -> Option<usize> {
        match self {
            Self::Grid(g) => g.hit_test(x, y, count),
            Self::Rows { metrics, width, .. } => {
                if y < metrics.pad_y || x < 0.0 || x >= *width {
                    return None;
                }
                let i = ((y - metrics.pad_y) / metrics.row_h).floor() as usize;
                (i < count).then_some(i)
            }
        }
    }

    /// Maximum scroll offset for a scrollable viewport of `view_height` DIPs.
    pub fn max_scroll(&self, view_height: f32) -> f32 {
        (self.content_height() - view_height).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default icon-title font (12 px, 16 line) reproduces today's cells exactly; a
    /// scaled font grows the cell on the 4 px grid, before spacing is applied.
    #[test]
    fn line_height_follows_the_icon_title_font_on_the_grid() {
        let base = GridMetrics::for_icon_size(48, 2);
        assert_eq!(base.with_line_h(16.0), base);
        assert_eq!(base.with_line_h(15.0), base, "rounds up to the 4 px grid");
        let big = GridMetrics::for_icon_size(48, 2).with_line_h(20.0);
        assert_eq!(big.line_h, 20.0);
        assert_eq!(big.cell_h, base.cell_h + 8.0);
        assert_eq!(big.cell_w, base.cell_w);
        let odd = GridMetrics::for_icon_size(48, 1).with_line_h(17.0);
        assert_eq!(odd.line_h, 20.0);
        assert_eq!(odd.cell_h % 4.0, 0.0);
        let compact = GridMetrics::for_icon_size(48, 2)
            .with_line_h(20.0)
            .with_spacing(pecofence_core::Spacing::Compact);
        assert_eq!(compact.cell_h, big.cell_h - 8.0);
    }

    #[test]
    fn columns_and_hit_test() {
        let g = Grid::new(GridMetrics::for_icon_size(48, 2), 320.0, 7);
        assert_eq!(g.columns, 4);
        assert_eq!(g.rows, 2);
        let c = g.cell(4);
        assert_eq!(g.hit_test(c.x + 1.0, c.y + 1.0, 7), Some(4));
        assert_eq!(g.hit_test(1.0, 1.0, 7), None);
        assert_eq!(g.hit_test(c.x + 1.0, 10_000.0, 7), None);
    }

    /// New fences are created at `width_for_columns`, which must be a fixed point of the
    /// column snap (`round((w - 2*pad) / cell)` columns → same width) or they would grow on
    /// the first sync and drift on every start.
    #[test]
    fn default_fence_width_is_a_column_snap_fixed_point() {
        let m = GridMetrics::for_icon_size(48, 2);
        assert_eq!(m.width_for_columns(3), 240.0);
        assert_eq!(m.width_for_columns(4), 320.0);
        for cols in 1..8u32 {
            let w = m.width_for_columns(cols);
            let snapped_cols = ((w - m.pad_x * 2.0) / m.cell_w).round();
            assert_eq!(snapped_cols, cols as f32);
            assert_eq!(m.pad_x * 2.0 + snapped_cols * m.cell_w, w);
        }
    }

    #[test]
    fn detail_columns_drop_from_the_right_when_narrow() {
        let wide = DetailColumns::for_width(480.0);
        assert!(wide.size.is_some() && wide.date.is_some() && wide.type_.is_some());
        // Explorer order: name, date, type, size.
        let (dx, _) = wide.date.unwrap();
        let (tx, _) = wide.type_.unwrap();
        let (sx, _) = wide.size.unwrap();
        assert!(wide.name_x < dx && dx < tx && tx < sx);
        assert_eq!(wide.column_at(sx + 1.0), Some(DetailColumn::Size));
        assert_eq!(wide.column_at(10.0), Some(DetailColumn::Name));
        let narrow = DetailColumns::for_width(200.0);
        assert!(narrow.type_.is_none() && narrow.date.is_none());
        assert!(narrow.name_w >= 40.0);
    }

    #[test]
    fn label_stays_inside_the_cell_for_every_spacing() {
        for icon in [32u32, 48, 64, 96] {
            for lines in [1u8, 2] {
                for spacing in [
                    pecofence_core::Spacing::Compact,
                    pecofence_core::Spacing::Normal,
                    pecofence_core::Spacing::Loose,
                ] {
                    let m = GridMetrics::for_icon_size(icon, lines).with_spacing(spacing);
                    // Every size stays on the 4 px grid the column snap relies on.
                    assert_eq!(
                        m.cell_w % 4.0,
                        0.0,
                        "{spacing:?} icon {icon}: cell_w {}",
                        m.cell_w
                    );
                    let label_bottom = m.icon_top + m.icon + m.label_gap + m.line_h * lines as f32;
                    assert!(
                        label_bottom <= m.cell_h - 2.0,
                        "{spacing:?} icon {icon} lines {lines}: label bottom {label_bottom} vs cell {}",
                        m.cell_h
                    );
                }
            }
        }
    }

    #[test]
    fn detail_columns_skip_hidden_columns() {
        let all = DetailColumns::for_width_with(480.0, DetailColumns::DEFAULT_WIDTHS, [true; 3]);
        let no_type = DetailColumns::for_width_with(
            480.0,
            DetailColumns::DEFAULT_WIDTHS,
            [true, false, true],
        );
        assert!(no_type.type_.is_none());
        assert!(no_type.date.is_some() && no_type.size.is_some());
        assert!(no_type.name_w > all.name_w);
        // Hiding the widest column lets a narrow fence keep the others (340 DIP: 大小 and
        // 类型 + gutter fit once 修改日期 is hidden).
        let narrow = DetailColumns::for_width_with(
            340.0,
            DetailColumns::DEFAULT_WIDTHS,
            [false, true, true],
        );
        assert!(narrow.date.is_none() && narrow.type_.is_some() && narrow.size.is_some());
    }

    #[test]
    fn insertion_index_and_caret_follow_cells() {
        let l = ItemLayout::Grid(Grid::new(GridMetrics::for_icon_size(48, 2), 320.0, 6));
        let c1 = l.cell(1);
        // Left half of cell 1 -> before it; right half -> after it.
        assert_eq!(l.insertion_index(c1.x + 4.0, c1.y + 4.0, 6), 1);
        assert_eq!(l.insertion_index(c1.x + c1.w - 4.0, c1.y + 4.0, 6), 2);
        assert_eq!(l.insertion_index(10_000.0, 10_000.0, 6), 6);
        let caret = l.insertion_caret(1, 6);
        assert_eq!(caret.x, c1.x - 1.0);
        assert_eq!(caret.w, 2.0);
        // After the last item of a full row: the last cell's right edge.
        let last = l.cell(3);
        let end = l.insertion_caret(4, 4);
        assert_eq!(end.x, last.x + last.w - 1.0);
        let rows = ItemLayout::rows(RowMetrics::list(), 200.0, 3);
        assert_eq!(rows.insertion_index(10.0, 4.0 + 28.0 * 1.6, 3), 2);
        assert_eq!(rows.insertion_index(10.0, 9_999.0, 3), 3);
        assert_eq!(rows.insertion_caret(3, 3).y, 4.0 + 3.0 * 28.0 - 1.0);
    }

    #[test]
    fn detail_columns_honour_user_widths_and_expose_dividers() {
        let c = DetailColumns::for_width_with(600.0, [200.0, 60.0, 100.0], [true; 3]);
        assert_eq!(c.date.unwrap().1, 200.0);
        assert_eq!(c.type_.unwrap().1, 60.0);
        assert_eq!(c.size.unwrap().1, 100.0);
        let (dx, _) = c.date.unwrap();
        assert_eq!(c.divider_at(dx - 4.0), Some(DetailColumn::Date));
        assert_eq!(c.divider_at(dx + 30.0), None);
        // Widths clamp so a dragged column can never eat the whole fence.
        let tiny = DetailColumns::for_width_with(600.0, [1.0, 1.0, 1.0], [true; 3]);
        assert_eq!(tiny.size.unwrap().1, DetailColumns::MIN_COL_W);
    }

    #[test]
    fn rows_layout_hit_tests_by_row() {
        let l = ItemLayout::rows(RowMetrics::details(), 300.0, 5);
        assert_eq!(l.columns(), 1);
        assert_eq!(l.fixed_top(), 28.0);
        let c = l.cell(2);
        assert_eq!(l.hit_test(10.0, c.y + 1.0, 5), Some(2));
        assert_eq!(l.hit_test(10.0, 1.0, 5), None);
        assert_eq!(l.hit_test(10.0, c.y + 1000.0, 5), None);
        assert_eq!(l.content_height(), 4.0 * 2.0 + 5.0 * 28.0);
    }
}
