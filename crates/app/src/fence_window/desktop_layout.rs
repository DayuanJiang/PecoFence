//! 像桌面一样排列 (`Fence::desktop_layout`): the 「桌面」 fence draws its icons like the Windows
//! desktop. They stand down columns from the top-left of its monitor's work area, the cells
//! under other fences stay free, and the window covers just the cells its icons take.

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

impl FenceViewState {
    /// Lays the items out for `self.desktop` and returns the window rect that holds their
    /// cells (screen px), and those cells relative to it (the window's shape: clicks between
    /// and beside the icons reach the desktop); None for a normal fence. The cells are kept in
    /// content-local DIPs for `layout`; `changed` says whether they moved.
    pub(super) fn arrange_desktop(&mut self) -> Option<(RECT, Vec<RECT>, bool)> {
        let area = self.desktop.clone()?;
        let scale = self.scale();
        let m = self.grid_metrics();
        let cell = (
            (m.cell_w * scale).round() as i32,
            (m.cell_h * scale).round() as i32,
        );
        let edge = (EDGE_DIP * scale).round() as i32;
        let n = self.items.len();
        let cells = desktop_cells(&area.work, &area.covered, cell, edge, n);
        // The window's shape: the icon cells, and under an icon with no icon below it room
        // for its selected full name (up to `MAX_UNFOLD_LINES` lines), which would be cut off
        // at the cell's bottom edge otherwise.
        let unfold =
            (MAX_UNFOLD_LINES.saturating_sub(u32::from(m.label_lines)) as f32 * m.line_h * scale)
                .round() as i32;
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
        let changed = slots != self.desktop_slots;
        self.desktop_slots = slots;
        let shape = shapes
            .iter()
            .map(|r| RECT {
                left: r.left - rect.left,
                top: r.top - rect.top,
                right: r.right - rect.left,
                bottom: r.bottom - rect.top,
            })
            .collect();
        Some((rect, shape, changed))
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

    #[test]
    fn an_empty_desktop_still_has_the_cell_a_drop_takes() {
        let cells = desktop_cells(&rect(0, 0, 500, 500), &[], (80, 100), 4, 0);
        assert_eq!(cells, vec![rect(4, 4, 84, 104)]);
    }
}
