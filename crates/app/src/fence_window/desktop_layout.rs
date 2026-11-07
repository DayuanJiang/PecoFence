//! 像桌面一样排列 (`Fence::desktop_layout`): the 「桌面」 fence draws its icons like the Windows
//! desktop. They stand down columns from the top-left of its monitor's work area, the cells
//! under other fences stay free, and the window covers just the cells its icons take.
//! With 自由摆放 (`Fence::free_placement`) each icon keeps the grid cell it was dropped in
//! instead, as on a Windows desktop with 自动排列图标 off and 将图标与网格对齐 on.

use super::*;

/// Gap (DIPs) between the work-area edges and the first column and row.
const EDGE_DIP: f32 = 4.0;

/// Columns tried at most (a very full desktop runs off the right edge, as Explorer's does).
const MAX_COLUMNS: i32 = 256;

fn overlaps(a: &RECT, b: &RECT) -> bool {
    a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom
}

/// Cells (screen px) for `count` items plus one more for a drop after the last: down each
/// column of `work`, then the next column to the right, skipping cells that touch `covered`.
pub(super) fn desktop_cells(
    work: &RECT,
    covered: &[RECT],
    cell: (i32, i32),
    edge: i32,
    count: usize,
) -> Vec<RECT> {
    let (cw, ch) = (cell.0.max(1), cell.1.max(1));
    let rows = ((work.bottom - work.top - edge) / ch).max(1);
    let want = count + 1;
    let mut cells = Vec::with_capacity(want);
    for col in 0..MAX_COLUMNS {
        let left = work.left + edge + col * cw;
        for row in 0..rows {
            let top = work.top + edge + row * ch;
            let r = RECT {
                left,
                top,
                right: left + cw,
                bottom: top + ch,
            };
            if covered.iter().any(|c| overlaps(c, &r)) {
                continue;
            }
            cells.push(r);
            if cells.len() == want {
                return cells;
            }
        }
    }
    cells
}

/// 自由摆放: the grid of whole cells on the work area (screen px), `edge` in from its top-left
/// corner like the arranged columns, and the cells other fences cover.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct FreeGrid {
    work: RECT,
    cell: (i32, i32),
    edge: i32,
    pub(super) cols: i32,
    pub(super) rows: i32,
    /// Per cell (column by column): under another fence.
    blocked: Vec<bool>,
}

impl FreeGrid {
    pub(super) fn new(work: &RECT, covered: &[RECT], cell: (i32, i32), edge: i32) -> Self {
        let (cw, ch) = (cell.0.max(1), cell.1.max(1));
        let mut grid = Self {
            work: *work,
            cell: (cw, ch),
            edge,
            cols: ((work.right - work.left - edge) / cw).max(1),
            rows: ((work.bottom - work.top - edge) / ch).max(1),
            blocked: Vec::new(),
        };
        let blocked: Vec<bool> = (0..grid.cols)
            .flat_map(|c| (0..grid.rows).map(move |r| (c, r)))
            .map(|c| {
                let r = grid.rect(c);
                covered.iter().any(|o| overlaps(o, &r))
            })
            .collect();
        grid.blocked = blocked;
        grid
    }

    /// The cell's rect (screen px). Columns past the right edge are where icons go when every
    /// cell is taken (off the screen, as on a full Windows desktop).
    pub(super) fn rect(&self, (c, r): (i32, i32)) -> RECT {
        let left = self.work.left + self.edge + c * self.cell.0;
        let top = self.work.top + self.edge + r * self.cell.1;
        RECT {
            left,
            top,
            right: left + self.cell.0,
            bottom: top + self.cell.1,
        }
    }

    fn inside(&self, (c, r): (i32, i32)) -> bool {
        (0..self.cols).contains(&c) && (0..self.rows).contains(&r)
    }

    fn blocked(&self, (c, r): (i32, i32)) -> bool {
        self.inside((c, r)) && self.blocked[(c * self.rows + r) as usize]
    }

    fn clamp(&self, (c, r): (i32, i32)) -> (i32, i32) {
        (c.clamp(0, self.cols - 1), r.clamp(0, self.rows - 1))
    }

    /// The cell under a screen point, or the nearest one inside the grid.
    pub(super) fn cell_at(&self, x: i32, y: i32) -> (i32, i32) {
        self.clamp((
            (x - self.work.left - self.edge).div_euclid(self.cell.0),
            (y - self.work.top - self.edge).div_euclid(self.cell.1),
        ))
    }

    /// Takes a cell for one icon: `want` (moved into the grid) when it is free, else the free
    /// cell nearest to it; without `want` the first free cell down the columns. When the grid
    /// is full, the first free cell in the columns past its right edge.
    pub(super) fn claim(
        &self,
        taken: &mut HashSet<(i32, i32)>,
        want: Option<(i32, i32)>,
    ) -> (i32, i32) {
        let free = |c: (i32, i32)| !self.blocked(c) && !taken.contains(&c);
        let all = || (0..self.cols).flat_map(|c| (0..self.rows).map(move |r| (c, r)));
        let picked = match want.map(|w| self.clamp(w)) {
            Some(w) if free(w) => Some(w),
            Some(w) => all().filter(|c| free(*c)).min_by_key(|&(c, r)| {
                let dx = i64::from((c - w.0) * self.cell.0);
                let dy = i64::from((r - w.1) * self.cell.1);
                dx * dx + dy * dy
            }),
            None => all().find(|c| free(*c)),
        };
        let cell = picked.unwrap_or_else(|| {
            (self.cols..self.cols + MAX_COLUMNS)
                .flat_map(|c| (0..self.rows).map(move |r| (c, r)))
                .find(|c| free(*c))
                .unwrap_or((self.cols, 0))
        });
        taken.insert(cell);
        cell
    }

    /// The cells icons dropped with their centres at `drops` (screen px) take, around the
    /// `taken` cells of the icons that stay: the one nearest `cursor` (the grabbed icon) first,
    /// each the cell under its centre or the free cell nearest to it.
    pub(super) fn drop_cells(
        &self,
        mut taken: HashSet<(i32, i32)>,
        cursor: (i32, i32),
        drops: &[(ItemId, (i32, i32))],
    ) -> Vec<(ItemId, [u16; 2])> {
        let mut order: Vec<&(ItemId, (i32, i32))> = drops.iter().collect();
        order.sort_by_key(|(_, (x, y))| {
            let (dx, dy) = (i64::from(x - cursor.0), i64::from(y - cursor.1));
            dx * dx + dy * dy
        });
        order
            .into_iter()
            .map(|(id, (x, y))| {
                let c = self.claim(&mut taken, Some(self.cell_at(*x, *y)));
                (*id, [c.0.max(0) as u16, c.1.max(0) as u16])
            })
            .collect()
    }

    /// The cells of icons that saved `wanted` (None: a new icon). An icon whose saved cell is
    /// inside the grid, not under a fence and not taken keeps it; the others then take the
    /// free cell nearest to theirs, and the new ones the first free cells down the columns.
    pub(super) fn place(&self, wanted: &[Option<(i32, i32)>]) -> Vec<(i32, i32)> {
        let mut taken = HashSet::new();
        let mut cells: Vec<Option<(i32, i32)>> = wanted
            .iter()
            .map(|w| w.filter(|&c| self.inside(c) && !self.blocked(c) && taken.insert(c)))
            .collect();
        for pass_new in [false, true] {
            for (i, w) in wanted.iter().enumerate() {
                if cells[i].is_none() && w.is_none() == pass_new {
                    cells[i] = Some(self.claim(&mut taken, *w));
                }
            }
        }
        cells.into_iter().flatten().collect()
    }
}

/// Height (DIPs) a selected full name may take below a cell of `m`. Without labels nothing
/// unfolds, so there is none.
fn unfold_room_dip(m: &GridMetrics) -> f32 {
    if m.label_lines == 0 {
        return 0.0;
    }
    MAX_UNFOLD_LINES.saturating_sub(u32::from(m.label_lines)) as f32 * m.line_h
}

impl FenceViewState {
    /// Lays the items out for `self.desktop` and returns the window rect that holds their
    /// cells (screen px), with room under the last icon of each column for a selected full
    /// name; None for a normal fence. The cells are kept in content-local DIPs for `layout`;
    /// `changed` says whether they moved or took another scale. The window's shape is
    /// `desktop_shape`.
    pub(super) fn arrange_desktop(&mut self) -> Option<(RECT, bool)> {
        let area = self.desktop.clone()?;
        // The cells are measured for the scale of the monitor they stand on, so a scale change
        // there or a move onto a monitor with another scale is taken first. Windows sends no
        // WM_DPICHANGED for a scale change by itself, only inside the window's next move or
        // resize: cells measured at the old scale would keep the rect, and the old DPI with it.
        let (cx, cy) = (
            (area.work.left + area.work.right) / 2,
            (area.work.top + area.work.bottom) / 2,
        );
        let rescaled = monitors::query(monitors::monitor_from_point(cx, cy))
            .is_some_and(|m| self.apply_dpi(m.dpi));
        let scale = self.scale();
        let m = self.grid_metrics();
        let cell = (
            (m.cell_w * scale).round() as i32,
            (m.cell_h * scale).round() as i32,
        );
        let edge = (EDGE_DIP * scale).round() as i32;
        let n = self.items.len();
        let cells = match area.free.as_ref() {
            Some(saved) => self.place_free(&area, saved, cell, edge),
            None => {
                self.desktop_free = None;
                desktop_cells(&area.work, &area.covered, cell, edge, n)
            }
        };
        // Under an icon with no icon below it the window has room for its selected full name
        // (up to `MAX_UNFOLD_LINES` lines), which would be cut off at the cell's bottom edge.
        let unfold = self.desktop_unfold_px();
        let icons = &cells[..n.min(cells.len())];
        let shapes: Vec<RECT> = icons
            .iter()
            .map(|r| {
                let below = icons.iter().any(|o| o.left == r.left && o.top == r.bottom);
                RECT {
                    bottom: if below { r.bottom } else { r.bottom + unfold },
                    ..*r
                }
            })
            .collect();
        let used = if shapes.is_empty() {
            &cells[..1.min(cells.len())]
        } else {
            &shapes[..]
        };
        let rect = RECT {
            left: area.work.left,
            top: area.work.top,
            right: used.iter().map(|r| r.right).max()? + edge,
            bottom: used.iter().map(|r| r.bottom).max()? + edge,
        };
        let slots: Vec<CellRect> = cells
            .iter()
            .map(|r| CellRect {
                x: (r.left - rect.left) as f32 / scale,
                y: (r.top - rect.top) as f32 / scale,
                w: m.cell_w,
                h: m.cell_h,
            })
            .collect();
        let changed = rescaled || slots != self.desktop_slots;
        self.desktop_slots = slots;
        Some((rect, changed))
    }

    /// 自由摆放: the cells (screen px) of the items from their saved cells, plus one free cell
    /// for a drop. Cells that changed (a new icon, one moved off a fence or into a smaller
    /// work area) go to the app to be saved.
    fn place_free(
        &mut self,
        area: &DesktopArea,
        saved: &BTreeMap<ItemId, [u16; 2]>,
        cell: (i32, i32),
        edge: i32,
    ) -> Vec<RECT> {
        let grid = FreeGrid::new(&area.work, &area.covered, cell, edge);
        let wanted: Vec<Option<(i32, i32)>> = self
            .items
            .iter()
            .map(|it| {
                saved
                    .get(&it.id)
                    .map(|c| (i32::from(c[0]), i32::from(c[1])))
            })
            .collect();
        let placed = grid.place(&wanted);
        let mut taken: HashSet<(i32, i32)> = placed.iter().copied().collect();
        let spare = grid.claim(&mut taken, None);
        let rects = placed
            .iter()
            .chain(std::iter::once(&spare))
            .map(|c| grid.rect(*c))
            .collect();
        // The icons that did not get their saved cell. The app reads them from the latest pass
        // (`desktop_free_changes`) when it gets the note, so a pass that a later one replaced
        // (other fences not yet moved for new monitors, say) is never saved. One note per
        // change, not one on every pass that finds it.
        let saved_form = |c: (i32, i32)| [c.0.max(0) as u16, c.1.max(0) as u16];
        let changes: Vec<(ItemId, [u16; 2])> = self
            .items
            .iter()
            .zip(&wanted)
            .zip(&placed)
            .filter(|((_, w), p)| **w != Some(**p))
            .map(|((it, _), p)| (it.id, saved_form(*p)))
            .collect();
        if !changes.is_empty() && self.desktop_free_sent.as_ref() != Some(&changes) {
            let added = wanted.iter().filter(|w| w.is_none()).count();
            tracing::info!(
                fence = %self.fence_id,
                icons = placed.len(),
                added,
                moved = changes.len() - added,
                cols = grid.cols,
                rows = grid.rows,
                covered = area.covered.len(),
                first = ?changes.first().map(|c| c.1),
                "free placement: cells placed"
            );
            self.queue.push(Command::DesktopCellsPlaced {
                fence: self.fence_id,
            });
        }
        self.desktop_free_sent = (!changes.is_empty()).then_some(changes);
        self.desktop_free = Some((grid, placed));
        rects
    }

    /// 自由摆放: where icons dropped on the desktop go. `drops` are the items and where their
    /// centres landed (screen px); the one dropped nearest `cursor` (the icon that was grabbed)
    /// takes its cell first, the others the cell under their centre or the nearest free one,
    /// around the icons that stay. None when the fence does not place its icons freely.
    pub(super) fn desktop_drop_cells(
        &self,
        cursor: (i32, i32),
        drops: &[(ItemId, (i32, i32))],
    ) -> Option<Vec<(ItemId, [u16; 2])>> {
        let (grid, placed) = self.desktop_free.as_ref()?;
        let moving: HashSet<ItemId> = drops.iter().map(|(id, _)| *id).collect();
        let taken: HashSet<(i32, i32)> = self
            .items
            .iter()
            .zip(placed)
            .filter(|(it, _)| !moving.contains(&it.id))
            .map(|(_, c)| *c)
            .collect();
        Some(grid.drop_cells(taken, cursor, drops))
    }

    /// Each selected item's centre less the press point (screen px), in `selected_ids` order,
    /// as a drag starts: a 「桌面」 that places its icons freely puts them where they land.
    pub(super) fn drag_offsets(&self) -> Vec<(ItemId, (i32, i32))> {
        let scale = self.scale();
        let (cw, _) = self.content_size_px();
        let width_dip = cw as f32 / scale;
        let layout = self.layout(width_dip);
        let fixed = layout.fixed_top();
        let (px, py) = self.ole_drag_press;
        let mut idx: Vec<usize> = self.selected.iter().copied().collect();
        idx.sort_unstable();
        idx.into_iter()
            .filter_map(|i| {
                let it = self.items.get(i)?;
                let c = layout.cell(i);
                let (x, w) = match layout {
                    ItemLayout::Grid(_) => (c.x, c.w),
                    ItemLayout::Rows { .. } => (0.0, width_dip),
                };
                let cx = self.content_x + ((x + w / 2.0) * scale).round() as i32;
                let cy = self.content_y
                    + ((c.y - self.scroll_y + fixed + c.h / 2.0) * scale).round() as i32;
                Some((it.id, (cx - px, cy - py)))
            })
            .collect()
    }

    /// 自由摆放: the cells the latest pass gave icons other than their saved ones, and the saved
    /// cells it placed them from; None when every icon has its saved cell.
    pub(super) fn desktop_free_changes(
        &self,
    ) -> Option<(BTreeMap<ItemId, [u16; 2]>, Vec<(ItemId, [u16; 2])>)> {
        let from = self.desktop.as_ref()?.free.clone()?;
        Some((from, self.desktop_free_sent.clone()?))
    }

    /// Whether the icons stay where they are dropped (自由摆放 on an arranged 「桌面」).
    pub(super) fn places_freely(&self) -> bool {
        self.desktop.as_ref().is_some_and(|d| d.free.is_some())
    }

    /// Height (px) a selected full name may take below its cell.
    fn desktop_unfold_px(&self) -> i32 {
        (unfold_room_dip(&self.grid_metrics()) * self.scale()).round() as i32
    }

    /// The selected icon whose shortened name unfolds (the anchor), and how many lines below
    /// its cell the full name takes (`unfold_cache`, filled by the redraw before the shape is
    /// made; the most lines while it is not filled yet).
    fn desktop_unfolded(&self) -> Option<(usize, u32)> {
        let a = self
            .anchor_index
            .filter(|a| *a < self.items.len() && self.selected.contains(a))?;
        let it = &self.items[a];
        if it.label.as_deref().unwrap_or(&it.name) == it.name {
            return None;
        }
        let lines = self
            .unfold_cache
            .as_ref()
            .filter(|c| c.id == it.id)
            .map_or(MAX_UNFOLD_LINES, |c| c.lines);
        let extra = lines.saturating_sub(u32::from(self.label_lines));
        (extra > 0).then_some((a, extra))
    }

    /// The window's shape (window px): the icon cells, plus room under the selected icon for
    /// its full name; clicks anywhere else reach the desktop.
    fn desktop_shape(&self) -> Vec<RECT> {
        let scale = self.scale();
        let px = |v: f32| (v * scale).round() as i32;
        let unfolded = self.desktop_unfolded();
        let line_h = self.grid_metrics().line_h;
        self.desktop_slots
            .iter()
            .take(self.items.len())
            .enumerate()
            .map(|(i, c)| {
                let room = match unfolded {
                    Some((a, extra)) if a == i => px(extra as f32 * line_h),
                    _ => 0,
                };
                RECT {
                    left: px(c.x),
                    top: px(c.y),
                    right: px(c.x + c.w),
                    bottom: px(c.y + c.h) + room,
                }
            })
            .collect()
    }

    /// Gives the window its shape when the cells or the selected icon changed (`force`: the
    /// cells were placed again).
    pub(super) fn sync_desktop_shape(&mut self, force: bool) {
        if !self.bare() {
            return;
        }
        let key = (self.desktop_unfolded(), self.items.len());
        if !force && self.desktop_shape_for == Some(key) {
            return;
        }
        self.desktop_shape_for = Some(key);
        let first = self.desktop_shape_pending.is_none();
        self.desktop_shape_pending = Some(self.desktop_shape());
        if first {
            window::post_message(self.hwnd, WM_APP_DESKTOP_SHAPE, 0, 0);
        }
    }

    /// A rectangle drawn on the empty desktop (screen px) selects the icons whose cells it
    /// touches, as Explorer's rubber band selects desktop icons; with Ctrl or Shift the icons
    /// selected when it started stay selected. Returns whether any icon is in the band.
    pub(super) fn band_desktop_items(&mut self, band: RECT) -> bool {
        if !self.bare() || self.desktop_slots.is_empty() {
            return false;
        }
        let win = window::window_rect(self.hwnd);
        let scale = self.scale();
        let mut hit = HashSet::new();
        for (i, c) in self.desktop_slots.iter().take(self.items.len()).enumerate() {
            let left = win.left + (c.x * scale).round() as i32;
            let top = win.top + (c.y * scale).round() as i32;
            let right = left + (c.w * scale).round() as i32;
            let bottom = top + (c.h * scale).round() as i32;
            if left < band.right && band.left < right && top < band.bottom && band.top < bottom {
                hit.insert(i);
            }
        }
        let any = !hit.is_empty();
        let base = self.desktop_band_base.clone().unwrap_or_default();
        let selected: HashSet<usize> = base.union(&hit).copied().collect();
        if selected != self.selected {
            self.selected = selected;
            let _ = self.redraw_content();
        }
        any
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn cells_go_down_each_column_and_skip_the_ones_under_other_fences() {
        let work = rect(0, 0, 1000, 400);
        // Three rows of 100 px fit in 400 - 4; a fence covers the second row of column one.
        let covered = [rect(10, 150, 300, 190)];
        let cells = desktop_cells(&work, &covered, (80, 100), 4, 4);
        let tops: Vec<(i32, i32)> = cells.iter().map(|c| (c.left, c.top)).collect();
        assert_eq!(
            tops,
            vec![(4, 4), (4, 204), (84, 4), (84, 204), (164, 4)],
            "column one, its third row, then column two below the fence, then column three"
        );
    }

    /// The cells come from the fence's grid: a smaller icon, a narrower spacing or hidden
    /// labels fit more icons in a column, and hidden labels leave no room for a full name.
    #[test]
    fn desktop_cells_follow_the_icon_grid() {
        use crate::layout::GridSpec;
        let spec = |icon_size, label_lines, spacing| GridSpec {
            icon_size,
            label_lines,
            spacing,
            icon_align: pecofence_core::IconAlign::Left,
            layout: ViewLayout::Icons,
        };
        let work = rect(0, 0, 1000, 404);
        let rows = |g: GridSpec| {
            let m = g.metrics(16.0);
            let cells = desktop_cells(&work, &[], (m.cell_w as i32, m.cell_h as i32), 4, 40);
            cells.iter().filter(|c| c.left == 4).count()
        };
        // 48 px with two label lines: 80 x 96 cells, four to a column of 400 px.
        assert_eq!(rows(spec(48, 2, 0)), 4);
        assert_eq!(rows(spec(24, 2, 0)), 5, "72 px cells");
        assert_eq!(rows(spec(48, 2, -12)), 4, "88 px cells");
        assert_eq!(rows(spec(48, 0, 0)), 6, "64 px squares");
        assert_eq!(rows(spec(48, 0, -12)), 7, "56 px squares");
        assert_eq!(unfold_room_dip(&spec(48, 0, 0).metrics(16.0)), 0.0);
        assert_eq!(
            unfold_room_dip(&spec(48, 2, 0).metrics(16.0)),
            (MAX_UNFOLD_LINES - 2) as f32 * 16.0
        );
        // Windows' text size makes the label lines, and so the room, taller.
        assert_eq!(
            unfold_room_dip(&spec(48, 1, 0).metrics(20.0)),
            (MAX_UNFOLD_LINES - 1) as f32 * 20.0
        );
    }

    /// As on the Windows desktop, a press takes an icon only on the icon or its name: the top
    /// right of the cell (where a fence with a title has its roll-up arrow) is desktop.
    #[test]
    fn only_the_icon_and_its_name_take_a_press() {
        use crate::layout::GridSpec;
        let m = GridSpec {
            icon_size: 48,
            label_lines: 2,
            spacing: 0,
            icon_align: pecofence_core::IconAlign::Left,
            layout: ViewLayout::Icons,
        }
        .metrics(16.0);
        let cell = CellRect {
            x: 0.0,
            y: 0.0,
            w: m.cell_w,
            h: m.cell_h,
        };
        assert!(
            super::super::hit::on_icon_or_label(&m, &cell, 40.0, 30.0),
            "the icon"
        );
        assert!(
            super::super::hit::on_icon_or_label(&m, &cell, 40.0, 70.0),
            "the name"
        );
        assert!(
            !super::super::hit::on_icon_or_label(&m, &cell, 76.0, 4.0),
            "top right"
        );
        assert!(
            !super::super::hit::on_icon_or_label(&m, &cell, 4.0, 20.0),
            "beside the icon"
        );
    }

    /// A 1000 x 400 work area of 80 x 100 px cells, 4 px in: 12 columns of 3 rows.
    fn free_grid(covered: &[RECT]) -> FreeGrid {
        FreeGrid::new(&rect(0, 0, 1000, 400), covered, (80, 100), 4)
    }

    #[test]
    fn free_icons_keep_their_cells_and_new_ones_take_the_first_free_cells() {
        let g = free_grid(&[]);
        assert_eq!((g.cols, g.rows), (12, 3));
        assert_eq!(g.rect((2, 1)), rect(164, 104, 244, 204));
        let placed = g.place(&[Some((5, 2)), None, Some((0, 0)), None, Some((11, 1))]);
        assert_eq!(
            placed,
            vec![(5, 2), (0, 1), (0, 0), (0, 2), (11, 1)],
            "saved cells stay; new icons fill column one from the top around them"
        );
    }

    #[test]
    fn a_fence_over_an_icon_moves_it_to_the_nearest_free_cell() {
        // The fence covers columns 4 and 5 of the middle row.
        let g = free_grid(&[rect(330, 150, 470, 190)]);
        let placed = g.place(&[Some((4, 1)), Some((5, 1)), Some((3, 1))]);
        assert_eq!(placed[2], (3, 1), "an icon beside the fence stays");
        // (4, 1) has a taken cell and the fence beside it: up a row (100 px). (5, 1) goes
        // right, 80 px, the nearest.
        assert_eq!(placed[0], (4, 0));
        assert_eq!(placed[1], (6, 1));
        let mut unique = placed.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), placed.len(), "never two icons in one cell");
    }

    #[test]
    fn a_smaller_work_area_moves_icons_inside_without_stacking_them() {
        // Saved on a larger screen: column 20 and row 7 are past this grid's edges.
        let g = free_grid(&[]);
        let placed = g.place(&[Some((20, 0)), Some((11, 0)), Some((3, 7)), Some((11, 2))]);
        assert_eq!(
            placed[1],
            (11, 0),
            "an icon whose cell is still here keeps it"
        );
        assert_eq!(
            placed[0],
            (10, 0),
            "the nearest free cell inside, beside the one that kept its cell"
        );
        assert_eq!(placed[2], (3, 2), "the bottom row of its column");
        assert_eq!(placed[3], (11, 2));
    }

    #[test]
    fn a_full_grid_puts_the_rest_past_the_right_edge() {
        let g = FreeGrid::new(&rect(0, 0, 168, 208), &[], (80, 100), 4);
        assert_eq!((g.cols, g.rows), (2, 2));
        let placed = g.place(&[None, None, None, None, None, Some((0, 0))]);
        assert_eq!(placed[5], (0, 0), "the saved cell wins over the new icons");
        assert_eq!(&placed[..5], &[(0, 1), (1, 0), (1, 1), (2, 0), (2, 1)]);
    }

    #[test]
    fn dropped_icons_land_under_their_centres_around_the_icons_that_stay() {
        let g = free_grid(&[]);
        let (a, b, c) = (ItemId::new_v4(), ItemId::new_v4(), ItemId::new_v4());
        // An icon stays in (7, 1); A is grabbed at (500, 160), B was one cell below it and C
        // one cell to its right, where the icon that stays is.
        let taken = HashSet::from([(7, 1)]);
        let cells = g.drop_cells(
            taken,
            (500, 160),
            &[(b, (500, 254)), (c, (580, 154)), (a, (504, 154))],
        );
        let find = |id| cells.iter().find(|(i, _)| *i == id).unwrap().1;
        assert_eq!(
            find(a),
            [6, 1],
            "the grabbed icon takes the cell under the cursor"
        );
        assert_eq!(find(b), [6, 2]);
        assert_eq!(
            find(c),
            [8, 1],
            "beside the icon that stays (80 px away, not 100)"
        );
        assert_eq!(cells[0].0, a, "the grabbed icon claims its cell first");
        // A point off the work area lands in the nearest cell inside it.
        assert_eq!(g.cell_at(-50, 900), (0, 2));
        assert_eq!(g.cell_at(5000, 10), (11, 0));
    }

    #[test]
    fn an_empty_desktop_still_has_the_cell_a_drop_takes() {
        let cells = desktop_cells(&rect(0, 0, 500, 500), &[], (80, 100), 4, 0);
        assert_eq!(cells, vec![rect(4, 4, 84, 104)]);
    }
}
