//! Keyboard focus between fences: Tab / Shift+Tab move it to the next / previous fence in
//! reading order, and during a Peek the first Tab or arrow key puts it into the first fence.
//! The desktop's own keys stay with Explorer: nothing here runs before a fence or the Peek
//! has the keyboard.

use super::*;

/// Fence tops closer than this (DIPs) are one row of the reading order, so a fence a few
/// pixels higher than its left neighbour does not jump ahead of it.
const ROW_SLACK_DIP: f32 = 24.0;

impl App {
    /// `Command::FocusNextFence`: from fence window `from` (null: the Peek, no fence yet) to
    /// the next one (`back`: the previous one) that can take the keyboard, wrapping around.
    pub(super) fn focus_next_fence(&mut self, from: HWND, back: bool) {
        let order = self.keyboard_order();
        let peeking = self.peek.as_ref().is_some_and(|p| !p.is_closing());
        let Some(next) = next_in_order(&order, from, back) else {
            tracing::info!(
                back,
                peeking,
                candidates = order.len(),
                "keyboard focus stays: no other fence takes it"
            );
            return;
        };
        let from_index = order.iter().position(|h| *h == from);
        let to_index = order.iter().position(|h| *h == next);
        tracing::info!(
            ?from_index,
            ?to_index,
            back,
            peeking,
            candidates = order.len(),
            "keyboard focus moves to another fence"
        );
        // As a click in it would: the other fences' icons lose their selection.
        self.end_item_selection(next);
        if let Some(w) = self.fences.values().find(|w| w.hwnd() == next) {
            w.focus_by_keyboard();
        }
    }

    /// The fence windows that can take the keyboard, in reading order: monitor by monitor from
    /// left to right, rows from top to bottom, each row from left to right.
    fn keyboard_order(&self) -> Vec<HWND> {
        let mut monitors: Vec<RECT> = monitors::enumerate().iter().map(|m| m.bounds).collect();
        monitors.sort_by_key(|m| (m.left, m.top));
        let slack = (ROW_SLACK_DIP * monitors::system_dpi().max(96) as f32 / 96.0) as i32;
        let windows: Vec<(HWND, usize, RECT)> = self
            .fences
            .values()
            .filter(|w| w.takes_keyboard())
            .map(|w| {
                let r = w.rect();
                let (cx, cy) = ((r.left + r.right) / 2, (r.top + r.bottom) / 2);
                let monitor = monitors
                    .iter()
                    .position(|m| cx >= m.left && cx < m.right && cy >= m.top && cy < m.bottom)
                    .unwrap_or(monitors.len());
                (w.hwnd(), monitor, r)
            })
            .collect();
        reading_order(windows, slack)
    }
}

/// Sorts `(id, monitor rank, rect)` into reading order: by monitor rank, then rows (a fence
/// whose top is within `row_slack` px of the row's first top joins that row), each row by left
/// edge. Ties keep a fixed order (top, then left).
pub(super) fn reading_order<T: Copy>(mut windows: Vec<(T, usize, RECT)>, row_slack: i32) -> Vec<T> {
    windows.sort_by_key(|(_, m, r)| (*m, r.top, r.left));
    let mut rows: Vec<Vec<(T, usize, RECT)>> = Vec::new();
    for w in windows {
        match rows.last_mut() {
            Some(row) if row[0].1 == w.1 && w.2.top - row[0].2.top <= row_slack => row.push(w),
            _ => rows.push(vec![w]),
        }
    }
    rows.into_iter()
        .flat_map(|mut row| {
            row.sort_by_key(|(_, _, r)| (r.left, r.top));
            row.into_iter().map(|(id, _, _)| id)
        })
        .collect()
}

/// The window after `from` in `order` (before it with `back`), wrapping around; the first one
/// (the last with `back`) when `from` is not in it. None when no other window can take the
/// keyboard.
pub(super) fn next_in_order<T: Copy + PartialEq>(order: &[T], from: T, back: bool) -> Option<T> {
    let n = order.len();
    match order.iter().position(|h| *h == from) {
        Some(_) if n < 2 => None,
        Some(i) => Some(order[if back { (i + n - 1) % n } else { (i + 1) % n }]),
        None if back => order.last().copied(),
        None => order.first().copied(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(left: i32, top: i32) -> RECT {
        RECT {
            left,
            top,
            right: left + 200,
            bottom: top + 150,
        }
    }

    #[test]
    fn reading_order_goes_row_by_row_and_left_to_right() {
        // Two columns; the right column's tops are a few pixels off the left one's.
        let order = reading_order(
            vec![
                ('d', 0, r(300, 410)),
                ('a', 0, r(20, 20)),
                ('b', 0, r(300, 12)),
                ('c', 0, r(20, 400)),
            ],
            24,
        );
        assert_eq!(order, vec!['a', 'b', 'c', 'd']);
    }

    #[test]
    fn reading_order_finishes_a_monitor_before_the_next() {
        let order = reading_order(
            vec![
                ('x', 1, r(2000, 10)),
                ('b', 0, r(20, 600)),
                ('a', 0, r(900, 10)),
            ],
            24,
        );
        assert_eq!(order, vec!['a', 'b', 'x']);
    }

    #[test]
    fn next_in_order_wraps_and_enters_from_outside() {
        let order = ['a', 'b', 'c'];
        assert_eq!(next_in_order(&order, 'a', false), Some('b'));
        assert_eq!(next_in_order(&order, 'c', false), Some('a'));
        assert_eq!(next_in_order(&order, 'a', true), Some('c'));
        assert_eq!(next_in_order(&order, 'z', false), Some('a'));
        assert_eq!(next_in_order(&order, 'z', true), Some('c'));
        assert_eq!(next_in_order(&['a'], 'a', false), None);
        assert_eq!(next_in_order(&['a'], 'z', false), Some('a'));
        assert_eq!(next_in_order::<char>(&[], 'z', false), None);
    }
}
