//! Icon grid layout in DIPs: cell geometry, hit testing, scrolling extents.

use pecofence_core::DateBucket;

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

/// Height of a "按时间分组" section header band (DIPs): 24 keeps the 4 px grid and sits just
/// under the 28 DIP details header and row height.
pub const GROUP_HEADER_H: f32 = 24.0;

/// A contiguous run of the (already sorted) items shown as one section: under a dated header,
/// or — for the leading namespace items (Recycle Bin …) that have no date — under none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupSpan {
    pub first: usize,
    pub len: usize,
    pub bucket: Option<DateBucket>,
}

impl GroupSpan {
    pub fn has_header(&self) -> bool {
        self.bucket.is_some()
    }
}

/// Splits the items into runs of equal bucket, in delivered order (`reverse` may flip the
/// sort, so runs are detected rather than assumed descending). Empty input = no spans.
pub fn group_spans(buckets: impl IntoIterator<Item = Option<DateBucket>>) -> Vec<GroupSpan> {
    let mut spans: Vec<GroupSpan> = Vec::new();
    for (i, bucket) in buckets.into_iter().enumerate() {
        match spans.last_mut() {
            Some(last) if last.bucket == bucket => last.len += 1,
            _ => spans.push(GroupSpan {
                first: i,
                len: 1,
                bucket,
            }),
        }
    }
    spans
}

/// A section header to draw: its band in scrollable-content DIPs and the group it heads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupHeader {
    pub rect: CellRect,
    pub group: usize,
    pub bucket: DateBucket,
}

/// Vertical structure shared by the icon grid and the row layouts: every group is an optional
/// header band followed by a block of whole rows of `cols` cells, `row_h` tall. Ungrouped =
/// one headerless group holding everything, which reproduces the plain grid exactly.
#[derive(Clone, Debug)]
pub struct Bands {
    cols: usize,
    row_h: f32,
    count: usize,
    groups: Vec<Band>,
    content_height: f32,
}

#[derive(Clone, Copy, Debug)]
struct Band {
    span: GroupSpan,
    /// Top of the header band (== `rows_y` without a header).
    top: f32,
    rows_y: f32,
    rows: usize,
}

impl Band {
    fn end(&self) -> usize {
        self.span.first + self.span.len
    }

    fn bottom(&self, row_h: f32) -> f32 {
        self.rows_y + self.rows as f32 * row_h
    }
}

impl Bands {
    fn new(cols: usize, row_h: f32, pad_y: f32, count: usize, spans: &[GroupSpan]) -> Self {
        let cols = cols.max(1);
        let single = [GroupSpan {
            first: 0,
            len: count,
            bucket: None,
        }];
        // The spans must tile 0..count; anything else falls back to the plain layout.
        let tiles = spans.first().is_some_and(|s| s.first == 0)
            && spans
                .windows(2)
                .all(|w| w[0].first + w[0].len == w[1].first)
            && spans.last().is_some_and(|s| s.first + s.len == count);
        let spans: &[GroupSpan] = if tiles { spans } else { &single };
        let mut y = pad_y;
        let mut groups = Vec::with_capacity(spans.len());
        for &span in spans {
            let top = y;
            if span.has_header() {
                y += GROUP_HEADER_H;
            }
            let rows = span.len.div_ceil(cols);
            groups.push(Band {
                span,
                top,
                rows_y: y,
                rows,
            });
            y += rows as f32 * row_h;
        }
        Self {
            cols,
            row_h,
            count,
            groups,
            content_height: y + pad_y,
        }
    }

    fn total_rows(&self) -> usize {
        self.groups.iter().map(|g| g.rows).sum()
    }

    fn is_grouped(&self) -> bool {
        self.groups.len() > 1 || self.groups[0].span.has_header()
    }

    /// Index of the band holding `index`; indices past the end land in the last band (the
    /// insertion caret after the last item).
    fn band_index(&self, index: usize) -> usize {
        self.groups
            .iter()
            .position(|g| index < g.end())
            .unwrap_or(self.groups.len() - 1)
    }

    /// (band index, row in the band, column) of `index`.
    fn place(&self, index: usize) -> (usize, usize, usize) {
        let gi = self.band_index(index);
        let local = index.saturating_sub(self.groups[gi].span.first);
        (gi, local / self.cols, local % self.cols)
    }

    fn cell_y(&self, index: usize) -> f32 {
        let (gi, row, _) = self.place(index);
        self.groups[gi].rows_y + row as f32 * self.row_h
    }

    fn band_at(&self, y: f32) -> Option<&Band> {
        self.groups
            .iter()
            .find(|g| y >= g.top && y < g.bottom(self.row_h))
    }

    /// Item at column `col` under `y`: None inside a header band, in the empty tail of a
    /// group's last row, or outside every band (so a point never maps into another group).
    fn hit(&self, col: usize, y: f32) -> Option<usize> {
        let g = self.band_at(y)?;
        if y < g.rows_y || col >= self.cols {
            return None;
        }
        let row = ((y - g.rows_y) / self.row_h).floor() as usize;
        let i = g.span.first + row * self.cols + col;
        (row < g.rows && i < g.end()).then_some(i)
    }

    /// Band for an insertion point: the one under `y`, else the first / last.
    fn band_near(&self, y: f32) -> &Band {
        self.band_at(y).unwrap_or_else(|| {
            if y < self.groups[0].top {
                &self.groups[0]
            } else {
                &self.groups[self.groups.len() - 1]
            }
        })
    }

    /// Insertion slot from a floored row and the caller's rounded column, clamped to the
    /// group under the point (grouping never applies to Manual sort, so this only matters for
    /// the single headerless group, where it is the plain-grid formula).
    fn insertion(&self, y: f32, col: usize) -> usize {
        let g = self.band_near(y);
        let row = ((y - g.rows_y) / self.row_h).floor().max(0.0) as usize;
        (g.span.first + row * self.cols + col.min(self.cols)).min(g.end())
    }

    /// Row variant: the nearest row boundary.
    fn insertion_rounded(&self, y: f32) -> usize {
        let g = self.band_near(y);
        let row = ((y - g.rows_y) / self.row_h).round().max(0.0) as usize;
        (g.span.first + row).min(g.end())
    }

    fn headers(&self, width: f32) -> Vec<GroupHeader> {
        self.groups
            .iter()
            .enumerate()
            .filter_map(|(i, g)| {
                Some(GroupHeader {
                    rect: CellRect {
                        x: 0.0,
                        y: g.top,
                        w: width,
                        h: GROUP_HEADER_H,
                    },
                    group: i,
                    bucket: g.span.bucket?,
                })
            })
            .collect()
    }

    /// Header band of the group `index` opens, if it has one.
    fn header_of_first(&self, index: usize, width: f32) -> Option<CellRect> {
        let g = self.groups.iter().find(|g| index < g.end())?;
        (g.span.first == index && g.span.has_header()).then_some(CellRect {
            x: 0.0,
            y: g.top,
            w: width,
            h: GROUP_HEADER_H,
        })
    }

    /// The row `steps` rows (signed) away from the row of `index`, crossing group boundaries:
    /// (band index, row), or None when the list ends before all steps are taken.
    fn row_step(&self, index: usize, steps: i32) -> Option<(usize, usize)> {
        let (mut gi, mut row, _) = self.place(index);
        for _ in 0..steps.unsigned_abs() {
            if steps > 0 {
                if row + 1 < self.groups[gi].rows {
                    row += 1;
                } else {
                    gi = (gi + 1..self.groups.len()).find(|&j| self.groups[j].rows > 0)?;
                    row = 0;
                }
            } else if row > 0 {
                row -= 1;
            } else {
                gi = (0..gi).rev().find(|&j| self.groups[j].rows > 0)?;
                row = self.groups[gi].rows - 1;
            }
        }
        Some((gi, row))
    }

    /// Item indices in row `row` of band `gi`.
    fn row_items(&self, gi: usize, row: usize) -> std::ops::Range<usize> {
        let g = &self.groups[gi];
        let start = g.span.first + row * self.cols;
        start..(start + self.cols).min(g.end())
    }

    fn neighbour(
        &self,
        index: usize,
        dx: i32,
        dy: i32,
        count: usize,
        centre_x: impl Fn(usize) -> f32,
    ) -> Option<usize> {
        let count = count.min(self.count);
        if count == 0 || index >= count {
            return None;
        }
        if dy == 0 {
            let last = count as i64 - 1;
            return Some((index as i64 + i64::from(dx)).clamp(0, last) as usize);
        }
        let (gi, row) = self.row_step(index, dy)?;
        let cx = centre_x(index);
        self.row_items(gi, row)
            .filter(|&i| i < count)
            .min_by(|&a, &b| {
                (centre_x(a) - cx)
                    .abs()
                    .total_cmp(&(centre_x(b) - cx).abs())
            })
    }
}

/// Lays out `count` items inside a content area of `width` DIPs (below the title bar), with
/// vertical scroll offset `scroll_y`. Returns cell rects in content-local DIPs (already scrolled).
/// With group spans, every group starts a fresh row block under its header band.
pub struct Grid {
    pub metrics: GridMetrics,
    pub columns: usize,
    /// Rows over all groups.
    #[allow(dead_code)]
    pub rows: usize,
    pub content_height: f32,
    bands: Bands,
}

impl Grid {
    #[cfg(test)]
    pub fn new(metrics: GridMetrics, width: f32, count: usize) -> Self {
        Self::grouped(metrics, width, count, &[])
    }

    pub fn grouped(metrics: GridMetrics, width: f32, count: usize, spans: &[GroupSpan]) -> Self {
        let usable = (width - metrics.pad_x * 2.0).max(metrics.cell_w);
        let columns = ((usable / metrics.cell_w).floor() as usize).max(1);
        let bands = Bands::new(columns, metrics.cell_h, metrics.pad_y, count, spans);
        Self {
            metrics,
            columns,
            rows: bands.total_rows(),
            content_height: bands.content_height,
            bands,
        }
    }

    /// Rect of item `index` in content coordinates (before scrolling).
    pub fn cell(&self, index: usize) -> CellRect {
        let (gi, row, col) = self.bands.place(index);
        CellRect {
            x: self.metrics.pad_x + col as f32 * self.metrics.cell_w,
            y: self.bands.groups[gi].rows_y + row as f32 * self.metrics.cell_h,
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
        if col >= self.columns {
            return None;
        }
        let idx = self.bands.hit(col, y)?;
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
        let colf = ((x - self.metrics.pad_x) / self.metrics.cell_w).max(0.0);
        self.bands.insertion(y, colf.round() as usize).min(count)
    }

    fn centre_x(&self, index: usize) -> f32 {
        let c = self.cell(index);
        c.x + c.w / 2.0
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
        bands: Bands,
    },
}

impl ItemLayout {
    #[cfg(test)]
    pub fn rows(metrics: RowMetrics, width: f32, count: usize) -> Self {
        Self::rows_grouped(metrics, width, count, &[])
    }

    pub fn rows_grouped(
        metrics: RowMetrics,
        width: f32,
        count: usize,
        spans: &[GroupSpan],
    ) -> Self {
        Self::Rows {
            metrics,
            width,
            bands: Bands::new(1, metrics.row_h, metrics.pad_y, count, spans),
        }
    }

    fn bands(&self) -> &Bands {
        match self {
            Self::Grid(g) => &g.bands,
            Self::Rows { bands, .. } => bands,
        }
    }

    /// Full content width the header bands span (rows know it; the grid's headers run from
    /// its left pad to the right edge of the last column).
    fn header_width(&self) -> f32 {
        match self {
            Self::Grid(g) => g.metrics.pad_x * 2.0 + g.columns as f32 * g.metrics.cell_w,
            Self::Rows { width, .. } => *width,
        }
    }

    #[cfg(test)]
    pub fn columns(&self) -> usize {
        match self {
            Self::Grid(g) => g.columns,
            Self::Rows { .. } => 1,
        }
    }

    #[cfg(test)]
    pub fn row_count(&self) -> usize {
        self.bands().total_rows()
    }

    /// Items are shown under "按时间分组" section headers.
    pub fn is_grouped(&self) -> bool {
        self.bands().is_grouped()
    }

    /// Section headers in scrollable-content coordinates (below the fixed details header).
    pub fn headers(&self) -> Vec<GroupHeader> {
        self.bands().headers(self.header_width())
    }

    /// The header band item `index` opens a group under, if any (scroll-into-view reveals it).
    pub fn group_header_of(&self, index: usize) -> Option<CellRect> {
        self.bands().header_of_first(index, self.header_width())
    }

    /// Scrollable content height, excluding a fixed details header.
    pub fn content_height(&self) -> f32 {
        self.bands().content_height
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
            Self::Rows {
                metrics,
                width,
                bands,
            } => CellRect {
                x: 0.0,
                y: bands.cell_y(index),
                w: *width,
                h: metrics.row_h,
            },
        }
    }

    /// Keyboard neighbour of `index`: `dx` steps along the list (clamped to its ends); `dy`
    /// rows up / down — the item of the target row whose centre is nearest horizontally,
    /// crossing group headers — or None when there is no such row (no wrap between rows).
    pub fn neighbour(&self, index: usize, dx: i32, dy: i32, count: usize) -> Option<usize> {
        match self {
            Self::Grid(g) => g.bands.neighbour(index, dx, dy, count, |i| g.centre_x(i)),
            Self::Rows { bands, .. } => bands.neighbour(index, dx, dy, count, |_| 0.0),
        }
    }

    /// Insertion index (0..=count) for a drop at a scrollable-content point.
    pub fn insertion_index(&self, x: f32, y: f32, count: usize) -> usize {
        match self {
            Self::Grid(g) => g.insertion_index(x, y, count),
            Self::Rows { bands, .. } => bands.insertion_rounded(y).min(count),
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
                let (_, _, col) = g.bands.place(index);
                if index < count || col != 0 {
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
                    self.cell(index).y
                } else if count == 0 {
                    metrics.pad_y
                } else {
                    let c = self.cell(count - 1);
                    c.y + c.h
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
            Self::Rows {
                metrics,
                width,
                bands,
            } => {
                if y < metrics.pad_y || x < 0.0 || x >= *width {
                    return None;
                }
                bands.hit(0, y).filter(|&i| i < count)
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

    fn overlaps(a: CellRect, b: CellRect) -> bool {
        a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
    }

    /// Two headerless namespace items, five 今天, one 更早, in a 4-column grid.
    fn grouped_grid() -> (GridMetrics, ItemLayout) {
        let m = GridMetrics::for_icon_size(48, 2);
        let spans = group_spans(
            [None, None]
                .into_iter()
                .chain(std::iter::repeat_n(Some(DateBucket::Today), 5))
                .chain([Some(DateBucket::Earlier)]),
        );
        assert_eq!(spans.len(), 3);
        (m, ItemLayout::Grid(Grid::grouped(m, 320.0, 8, &spans)))
    }

    #[test]
    fn group_spans_detects_runs_in_delivered_order() {
        let spans = group_spans([
            None,
            Some(DateBucket::Earlier),
            Some(DateBucket::Earlier),
            Some(DateBucket::Today),
        ]);
        assert_eq!(
            spans,
            vec![
                GroupSpan {
                    first: 0,
                    len: 1,
                    bucket: None
                },
                GroupSpan {
                    first: 1,
                    len: 2,
                    bucket: Some(DateBucket::Earlier)
                },
                GroupSpan {
                    first: 3,
                    len: 1,
                    bucket: Some(DateBucket::Today)
                },
            ]
        );
        assert!(group_spans(std::iter::empty()).is_empty());
    }

    #[test]
    fn grouped_grid_stacks_header_bands_and_row_blocks() {
        let (m, l) = grouped_grid();
        assert!(l.is_grouped());
        assert_eq!(l.columns(), 4);
        let headers = l.headers();
        assert_eq!(headers.len(), 2, "the namespace run has no header");
        // Namespace run: one row at the top pad; then the 今天 header, two rows (4 + 1);
        // then the 更早 header and its single row.
        assert_eq!(l.cell(0).y, m.pad_y);
        assert_eq!(l.cell(1).x, m.pad_x + m.cell_w);
        assert_eq!(headers[0].rect.y, m.pad_y + m.cell_h);
        assert_eq!(headers[0].rect.h, GROUP_HEADER_H);
        assert_eq!(headers[0].bucket, DateBucket::Today);
        assert_eq!(headers[0].group, 1);
        assert_eq!(l.cell(2).y, headers[0].rect.y + GROUP_HEADER_H);
        assert_eq!(l.cell(2).x, m.pad_x, "each group starts a fresh row");
        assert_eq!(l.cell(5).x, m.pad_x + 3.0 * m.cell_w);
        assert_eq!(l.cell(6).y, l.cell(2).y + m.cell_h);
        assert_eq!(l.cell(6).x, m.pad_x);
        assert_eq!(headers[1].rect.y, l.cell(6).y + m.cell_h);
        assert_eq!(headers[1].bucket, DateBucket::Earlier);
        assert_eq!(l.cell(7).y, headers[1].rect.y + GROUP_HEADER_H);
        assert_eq!(l.row_count(), 4);
        assert_eq!(
            l.content_height(),
            m.pad_y * 2.0 + 4.0 * m.cell_h + 2.0 * GROUP_HEADER_H
        );
        assert_eq!(l.max_scroll(100.0), l.content_height() - 100.0);
        for i in 0..8 {
            for j in 0..8 {
                assert!(i == j || !overlaps(l.cell(i), l.cell(j)), "{i} vs {j}");
            }
            for h in &headers {
                assert!(!overlaps(l.cell(i), h.rect), "{i} under a header");
            }
        }
    }

    #[test]
    fn grouped_grid_hit_tests_never_cross_groups() {
        let (m, l) = grouped_grid();
        for i in 0..8 {
            let c = l.cell(i);
            assert_eq!(l.hit_test(c.x + c.w / 2.0, c.y + c.h / 2.0, 8), Some(i));
            assert_eq!(l.hit_test(c.x + 1.0, c.y + 1.0, 8), Some(i));
        }
        for h in l.headers() {
            assert_eq!(l.hit_test(10.0, h.rect.y + 1.0, 8), None);
            assert_eq!(l.hit_test(300.0, h.rect.y + h.rect.h - 0.5, 8), None);
        }
        // Empty tail of a short row: right of the two namespace items, right of item 6.
        let c0 = l.cell(0);
        assert_eq!(l.hit_test(c0.x + 2.0 * c0.w + 5.0, c0.y + 5.0, 8), None);
        let c6 = l.cell(6);
        assert_eq!(l.hit_test(c6.x + c6.w + 5.0, c6.y + 5.0, 8), None);
        assert_eq!(l.hit_test(c6.x + 3.0 * c6.w + 5.0, c6.y + 5.0, 8), None);
        // Below everything.
        assert_eq!(l.hit_test(10.0, l.content_height() + 1.0, 8), None);
        let _ = m;
    }

    #[test]
    fn neighbour_crosses_group_headers_by_nearest_column() {
        let (_, l) = grouped_grid();
        assert_eq!(l.neighbour(0, 0, 1, 8), Some(2), "down into 今天 row 0");
        assert_eq!(l.neighbour(1, 0, 1, 8), Some(3));
        assert_eq!(
            l.neighbour(5, 0, 1, 8),
            Some(6),
            "short row: nearest column"
        );
        assert_eq!(l.neighbour(3, 0, -1, 8), Some(1));
        assert_eq!(
            l.neighbour(4, 0, -1, 8),
            Some(1),
            "nearest when the row above is short"
        );
        assert_eq!(l.neighbour(6, 0, -1, 8), Some(2));
        assert_eq!(l.neighbour(7, 0, -1, 8), Some(6));
        assert_eq!(l.neighbour(7, 0, 1, 8), None, "no wrap past the last row");
        assert_eq!(l.neighbour(0, 0, -1, 8), None);
        assert_eq!(
            l.neighbour(1, 1, 0, 8),
            Some(2),
            "right crosses into the next group"
        );
        assert_eq!(l.neighbour(7, 1, 0, 8), Some(7), "clamped at the end");
        assert_eq!(l.neighbour(0, -1, 0, 8), Some(0));
        assert_eq!(l.neighbour(0, 0, 3, 8), Some(7), "three rows down");
        assert_eq!(l.neighbour(0, 0, 4, 8), None, "page past the end");
        assert_eq!(l.neighbour(7, 0, -3, 8), Some(0));
        assert_eq!(l.neighbour(8, 0, 1, 8), None);
        // Plain grid: the old `cur ± cols` arithmetic.
        let plain = ItemLayout::Grid(Grid::new(GridMetrics::for_icon_size(48, 2), 320.0, 10));
        assert_eq!(plain.neighbour(1, 0, 1, 10), Some(5));
        assert_eq!(plain.neighbour(6, 0, -1, 10), Some(2));
        // A short last row takes the nearest column instead of refusing to move.
        assert_eq!(plain.neighbour(7, 0, 1, 10), Some(9));
        assert_eq!(plain.neighbour(9, 0, 1, 10), None);
        assert_eq!(plain.neighbour(9, 1, 0, 10), Some(9));
        let rows = ItemLayout::rows(RowMetrics::list(), 200.0, 3);
        assert_eq!(rows.neighbour(0, 0, 1, 3), Some(1));
        assert_eq!(rows.neighbour(2, 0, 1, 3), None);
    }

    #[test]
    fn plain_grid_is_one_headerless_group() {
        let m = GridMetrics::for_icon_size(48, 2);
        let plain = Grid::new(m, 320.0, 7);
        let one = Grid::grouped(
            m,
            320.0,
            7,
            &[GroupSpan {
                first: 0,
                len: 7,
                bucket: None,
            }],
        );
        // Spans that do not tile the items are ignored rather than trusted.
        let broken = Grid::grouped(
            m,
            320.0,
            7,
            &[GroupSpan {
                first: 0,
                len: 3,
                bucket: Some(DateBucket::Today),
            }],
        );
        for g in [&plain, &one, &broken] {
            assert_eq!(g.columns, 4);
            assert_eq!(g.rows, 2);
            assert_eq!(g.content_height, m.pad_y * 2.0 + 2.0 * m.cell_h);
            for i in 0..7 {
                assert_eq!(g.cell(i), plain.cell(i));
                assert_eq!(
                    g.cell(i),
                    CellRect {
                        x: m.pad_x + (i % 4) as f32 * m.cell_w,
                        y: m.pad_y + (i / 4) as f32 * m.cell_h,
                        w: m.cell_w,
                        h: m.cell_h,
                    }
                );
            }
        }
        let l = ItemLayout::Grid(one);
        assert!(!l.is_grouped());
        assert!(l.headers().is_empty());
        assert_eq!(l.group_header_of(0), None);
        let empty = ItemLayout::Grid(Grid::new(m, 320.0, 0));
        assert_eq!(empty.content_height(), m.pad_y * 2.0);
        assert_eq!(empty.insertion_caret(0, 0).y, m.pad_y);
    }

    #[test]
    fn grouped_rows_layout_interleaves_header_bands() {
        let spans = group_spans([
            Some(DateBucket::Today),
            Some(DateBucket::Today),
            Some(DateBucket::Yesterday),
        ]);
        let l = ItemLayout::rows_grouped(RowMetrics::details(), 300.0, 3, &spans);
        let m = RowMetrics::details();
        let headers = l.headers();
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].rect.y, m.pad_y);
        assert_eq!(headers[0].rect.w, 300.0);
        assert_eq!(l.cell(0).y, m.pad_y + GROUP_HEADER_H);
        assert_eq!(l.cell(1).y, l.cell(0).y + m.row_h);
        assert_eq!(headers[1].rect.y, l.cell(1).y + m.row_h);
        assert_eq!(l.cell(2).y, headers[1].rect.y + GROUP_HEADER_H);
        assert_eq!(
            l.content_height(),
            m.pad_y * 2.0 + 3.0 * m.row_h + 2.0 * GROUP_HEADER_H
        );
        assert_eq!(l.fixed_top(), m.header_h);
        for h in &headers {
            assert_eq!(l.hit_test(10.0, h.rect.y + 2.0, 3), None);
        }
        for i in 0..3 {
            let c = l.cell(i);
            assert_eq!(l.hit_test(10.0, c.y + c.h / 2.0, 3), Some(i));
        }
        assert_eq!(l.group_header_of(0), Some(headers[0].rect));
        assert_eq!(l.group_header_of(1), None);
        assert_eq!(l.group_header_of(2), Some(headers[1].rect));
        assert_eq!(l.insertion_caret(3, 3).y, l.cell(2).y + m.row_h - 1.0);
        assert_eq!(
            l.neighbour(1, 0, 1, 3),
            Some(2),
            "down across the 昨天 header"
        );
        assert_eq!(l.neighbour(2, 0, -1, 3), Some(1));
    }
}
