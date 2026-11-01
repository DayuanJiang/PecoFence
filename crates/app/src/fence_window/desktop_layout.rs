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
    /// cells (screen px); None for a normal fence. The cells are kept in content-local DIPs
    /// for `layout`; `changed` says whether they moved.
    pub(super) fn arrange_desktop(&mut self) -> Option<(RECT, bool)> {
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
        let used = &cells[..n.max(1).min(cells.len())];
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
        Some((rect, changed))
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
