//! Push neighbours (`rollUp.pushNeighbors`): while a fence is expanded (a click, a hover peek, or
//! a taller height from the CLI), the fences stacked below it slide down out of its way, and
//! slide back when it rolls up. Edge drags and the automatic size changes still stop short of
//! the fence below as it shows.
//!
//! The layout keeps the resting positions. The pass moves the windows and notes each shift in
//! `Behavior::pushed`; every rectangle the layout records has the shift taken out first
//! ([`App::record_bounds`] here, `queue_bounds_changed` where the windows report one). Which
//! fence sits below which, and the heights they rest at, come from the roll states, so a fence
//! saved expanded pushes the same way after a restart. A shift stands only while the window is
//! still at the top the pass gave it, and a fence the user drags rests where it is dropped.

use super::*;
use crate::fence_window::Pushed;

/// Who may move a fence in this pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    /// Pushed out of the way, also when locked (feedback #52): a push only moves the window
    /// for a while, and the fence keeps its saved place.
    Free,
    /// No spot on these monitors yet: stays at its resting place.
    Settling,
    /// Stays where it is: in the user's hands (dragged, or resized when `pushes`), or pushed by
    /// a fence that is being dragged (it makes room again once that fence is dropped).
    Held,
}

#[derive(Debug)]
struct Slot {
    hwnd: HWND,
    id: FenceId,
    left: i32,
    right: i32,
    /// Window top now, and where it rests.
    top: i32,
    base: i32,
    /// Window height now, its title row's and the one it settles at.
    h: i32,
    title_h: i32,
    rest_h: i32,
    gap: i32,
    /// Work area (index, bottom) the fence rests on.
    work: Option<(usize, i32)>,
    role: Role,
    /// Fences below it make room for it (not while the user drags it around).
    pushes: bool,
    /// Top this pass gives it, and the fences that pushed it there (a held fence keeps the ones
    /// from the last pass).
    y: i32,
    by: Vec<HWND>,
}

/// Gives every slot its top for the heights the windows have now. A free fence is pushed down
/// just far enough to clear each pushing fence above it on the same screen that overlaps it
/// horizontally, keeping the gap the resting layout had (at most `gap`), so a layout at rest
/// never moves; it stays inside its work area. `enabled == false` rests everything.
fn stack(slots: &mut [Slot], enabled: bool) {
    slots.sort_by_key(|s| (s.base, s.left, s.id));
    for j in 0..slots.len() {
        let b = &slots[j];
        let mut y = match b.role {
            Role::Held => b.top,
            Role::Settling => b.base,
            Role::Free => b.base,
        };
        let mut binding = None;
        if enabled && b.role == Role::Free {
            for (i, a) in slots[..j].iter().enumerate() {
                // `b` is below `a` when it rests under `a`'s title row.
                let below = a.pushes
                    && a.work.map(|w| w.0) == b.work.map(|w| w.0)
                    && a.left < b.right
                    && b.left < a.right
                    && b.base >= a.base + a.title_h;
                if !below {
                    continue;
                }
                // The resting gap under `a`, or under its title row when `b` rests inside `a`'s
                // resting height (a fence saved expanded over the ones it pushes).
                let rest_h = if b.base >= a.base + a.rest_h {
                    a.rest_h
                } else {
                    a.title_h
                };
                let slack = (b.base - (a.base + rest_h)).clamp(0, b.gap);
                let need = a.y + a.h + slack;
                if need > y {
                    y = need;
                    binding = Some(i);
                }
            }
            if let Some((_, bottom)) = b.work {
                y = y.min(bottom - b.h).max(b.base);
            }
        }
        if slots[j].role != Role::Held {
            slots[j].by = match binding {
                Some(i) if y != slots[j].base => std::iter::once(slots[i].hwnd)
                    .chain(slots[i].by.iter().copied())
                    .collect(),
                _ => Vec::new(),
            };
        }
        slots[j].y = y;
    }
}

impl App {
    /// Lays the fences out for the heights they have now: after every command batch and on
    /// every animation frame (a rectangle read per window; windows move only when their top
    /// changes).
    pub(super) fn reflow_pushed(&mut self) {
        self.reflow_pushed_except(&[]);
    }

    /// [`Self::reflow_pushed`] with the `settling` fences (no spot on these monitors yet) left
    /// at rest and pushing nothing: one that lands inside another fence is then moved out of the
    /// way rather than taken for a fence that one pushes.
    pub(super) fn reflow_pushed_except(&mut self, settling: &[FenceId]) {
        let enabled = self.state.config.settings.roll_up.push_neighbors;
        if !enabled && self.ctx.behavior.pushed.borrow().is_empty() {
            return;
        }
        let mut dragged: Vec<HWND> = self
            .fences
            .values()
            .flat_map(|w| w.dragged_windows())
            .collect();
        let mut sized = None;
        if let Some(sm) = self.ctx.behavior.size_move.borrow().as_ref() {
            if sm.moving {
                dragged.push(sm.hwnd);
                dragged.extend(&sm.group);
            } else {
                sized = Some(sm.hwnd);
            }
        }
        let mut slots: Vec<Slot> = Vec::with_capacity(self.fences.len());
        {
            let pushed = self.ctx.behavior.pushed.borrow();
            for (&id, w) in &self.fences {
                let hwnd = w.hwnd();
                let Some((title_h, rest_h)) = w.resting_heights_px() else {
                    continue;
                };
                let r = w.rect();
                let h = r.bottom - r.top;
                if h <= 0 {
                    continue;
                }
                let prev = pushed.iter().find(|p| p.hwnd == hwnd && p.top == r.top);
                let base = prev.map_or(r.top, |p| r.top - p.dy);
                let by = prev.map(|p| p.by.clone()).unwrap_or_default();
                let (cx, cy) = ((r.left + r.right) / 2, base + title_h / 2);
                let work = self
                    .state
                    .work_areas
                    .iter()
                    .position(|w| cx >= w.left && cx < w.right && cy >= w.top && cy < w.bottom)
                    .map(|i| (i, self.state.work_areas[i].bottom));
                let is_dragged = dragged.contains(&hwnd);
                // A press on the title of a fence that pushes others already counts as a drag:
                // what it pushed stays put rather than sliding back under it.
                let frozen = by.iter().any(|h| dragged.contains(h));
                let settles = settling.contains(&id);
                let role = if is_dragged || frozen || sized == Some(hwnd) {
                    Role::Held
                } else if settles {
                    Role::Settling
                } else {
                    Role::Free
                };
                slots.push(Slot {
                    hwnd,
                    id,
                    left: r.left,
                    right: r.right,
                    top: r.top,
                    base,
                    h,
                    title_h,
                    rest_h,
                    gap: self.gap_px(hwnd),
                    work,
                    role,
                    pushes: !is_dragged && !settles,
                    y: r.top,
                    by,
                });
            }
        }
        stack(&mut slots, enabled);
        // No borrow held: SetWindowPos runs the fences' handlers synchronously.
        let mut next = Vec::new();
        for s in slots {
            if s.y != s.top {
                let _ = window::move_window_to(s.hwnd, s.left, s.y);
            }
            if s.y != s.base {
                next.push(Pushed {
                    hwnd: s.hwnd,
                    top: s.y,
                    dy: s.y - s.base,
                    by: s.by,
                });
            }
        }
        *self.ctx.behavior.pushed.borrow_mut() = next;
    }

    /// Records host `id` at `rect`, a window rectangle, without the push shift (see the module
    /// docs): every path that saves a fence window's geometry goes through here.
    pub(super) fn record_bounds(&mut self, id: FenceId, rect: RECT, rolled: bool, expanded: i32) {
        let rect = match self.fences.get(&id) {
            Some(w) => {
                let mut rect = self.ctx.behavior.resting_rect(w.hwnd(), rect);
                // Collapsed to its left tab column: the fence keeps the width it expands to.
                if rolled && let Some(ew) = w.collapsed_expanded_width_px() {
                    rect.right = rect.left + ew;
                }
                rect
            }
            None => rect,
        };
        self.state.set_fence_bounds(id, rect, rolled, expanded);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TITLE: i32 = 36;

    fn hwnd(n: u128) -> HWND {
        HWND((0x1000 + n as usize) as *mut _)
    }

    /// A 240 px wide fence at (`x`, `y`) that is `h` tall now and rests at `rest_h`.
    fn slot(n: u128, x: i32, y: i32, h: i32, rest_h: i32) -> Slot {
        Slot {
            hwnd: hwnd(n),
            id: FenceId::from_u128(n),
            left: x,
            right: x + 240,
            top: y,
            base: y,
            h,
            title_h: TITLE,
            rest_h,
            gap: 8,
            work: Some((0, 1000)),
            role: Role::Free,
            pushes: true,
            y,
            by: Vec::new(),
        }
    }

    fn ys(slots: &[Slot], ids: &[u128]) -> Vec<i32> {
        ids.iter()
            .map(|n| {
                slots
                    .iter()
                    .find(|s| s.id == FenceId::from_u128(*n))
                    .unwrap()
                    .y
            })
            .collect()
    }

    fn get(slots: &mut [Slot], n: u128) -> &mut Slot {
        slots
            .iter_mut()
            .find(|s| s.id == FenceId::from_u128(n))
            .unwrap()
    }

    #[test]
    fn resting_stack_does_not_move() {
        // Three rolled fences 10 px apart, one 4 px under them, and an expanded one with a
        // fence 4 px under its bottom.
        let mut s = vec![
            slot(1, 0, 100, TITLE, TITLE),
            slot(2, 0, 146, TITLE, TITLE),
            slot(3, 0, 192, TITLE, TITLE),
            slot(4, 0, 232, TITLE, TITLE),
            slot(5, 300, 100, 300, 300),
            slot(6, 300, 404, TITLE, TITLE),
        ];
        stack(&mut s, true);
        assert_eq!(
            ys(&s, &[1, 2, 3, 4, 5, 6]),
            vec![100, 146, 192, 232, 100, 404]
        );
    }

    #[test]
    fn peek_pushes_the_ones_below_and_they_come_back() {
        // A hover peek: rolled at rest, 150 px tall now.
        let mut s = vec![
            slot(1, 0, 100, 150, TITLE),
            slot(2, 0, 146, TITLE, TITLE),
            slot(3, 0, 192, TITLE, TITLE),
        ];
        stack(&mut s, true);
        // 100 + 150 + the 8 px gap (10 px at rest, at most 8), then 36 + 8 further down.
        assert_eq!(ys(&s, &[1, 2, 3]), vec![100, 258, 302]);
        assert_eq!(get(&mut s, 3).by, vec![hwnd(2), hwnd(1)]);
        get(&mut s, 1).h = TITLE;
        stack(&mut s, true);
        assert_eq!(ys(&s, &[1, 2, 3]), vec![100, 146, 192]);
    }

    #[test]
    fn a_fence_saved_expanded_keeps_pushing() {
        // Expanded for good (rests at 300): the fence under its title row stays pushed, with
        // the same gap a peek gives it, so the layout looks the same after a restart.
        let mut s = vec![slot(1, 0, 100, 300, 300), slot(2, 0, 146, TITLE, TITLE)];
        stack(&mut s, true);
        assert_eq!(ys(&s, &[2]), vec![408]);
        // Rolling it up brings the fence back.
        get(&mut s, 1).h = TITLE;
        get(&mut s, 1).rest_h = TITLE;
        stack(&mut s, true);
        assert_eq!(ys(&s, &[2]), vec![146]);
    }

    #[test]
    fn only_overlapping_columns_below_on_the_same_screen_are_pushed() {
        let mut s = vec![
            slot(1, 0, 100, 300, TITLE),
            slot(2, 500, 146, TITLE, TITLE), // other column
            slot(3, 0, 20, TITLE, TITLE),    // above
            slot(4, 0, 160, TITLE, TITLE),   // another screen
        ];
        get(&mut s, 4).work = Some((1, 1000));
        stack(&mut s, true);
        assert_eq!(ys(&s, &[1, 2, 3, 4]), vec![100, 146, 20, 160]);
    }

    #[test]
    fn a_big_gap_absorbs_the_growth() {
        let mut s = vec![slot(1, 0, 100, 150, TITLE), slot(2, 0, 400, TITLE, TITLE)];
        stack(&mut s, true);
        assert_eq!(ys(&s, &[2]), vec![400]);
    }

    #[test]
    fn pushed_fences_stay_inside_the_work_area() {
        let mut s = vec![slot(1, 0, 600, 380, TITLE), slot(2, 0, 646, 200, 200)];
        stack(&mut s, true);
        assert_eq!(ys(&s, &[2]), vec![800]);
    }

    #[test]
    fn a_held_fence_is_not_moved() {
        let mut s = vec![slot(1, 0, 100, 300, TITLE), slot(2, 0, 146, TITLE, TITLE)];
        get(&mut s, 2).role = Role::Held;
        get(&mut s, 2).top = 250;
        stack(&mut s, true);
        assert_eq!(ys(&s, &[2]), vec![250]);
    }

    #[test]
    fn what_a_dragged_fence_pushed_stays_put() {
        // A expanded and pushing B, then pressed on its title: B is held where it is.
        let mut s = vec![slot(1, 0, 100, 300, TITLE), slot(2, 0, 146, TITLE, TITLE)];
        get(&mut s, 1).role = Role::Held;
        get(&mut s, 1).pushes = false;
        let b = get(&mut s, 2);
        b.role = Role::Held;
        b.top = 408;
        b.by = vec![hwnd(1)];
        stack(&mut s, true);
        assert_eq!(ys(&s, &[1, 2]), vec![100, 408]);
        assert_eq!(get(&mut s, 2).by, vec![hwnd(1)]);
    }

    #[test]
    fn a_dragged_fence_pushes_nothing() {
        let mut s = vec![slot(1, 0, 100, 300, TITLE), slot(2, 0, 146, TITLE, TITLE)];
        get(&mut s, 1).role = Role::Held;
        get(&mut s, 1).pushes = false;
        stack(&mut s, true);
        assert_eq!(ys(&s, &[2]), vec![146]);
        // A resized one still does.
        get(&mut s, 1).pushes = true;
        stack(&mut s, true);
        assert_eq!(ys(&s, &[2]), vec![408]);
    }

    #[test]
    fn a_settling_fence_is_neither_pushed_nor_pushes() {
        // A fence from another screen setup lands under the title row of an expanded one: it
        // stays where it landed for the overlap pass to move, and the fence below stays put
        // (pushed along, it would go from 420 to 452).
        let mut s = vec![
            slot(1, 0, 100, 300, 300),
            slot(2, 0, 146, TITLE, TITLE),
            slot(3, 0, 420, TITLE, TITLE),
        ];
        let m = get(&mut s, 2);
        m.role = Role::Settling;
        m.pushes = false;
        stack(&mut s, true);
        assert_eq!(ys(&s, &[2, 3]), vec![146, 420]);
    }

    #[test]
    fn disabled_rests_everything() {
        let mut s = vec![slot(1, 0, 100, 150, TITLE), slot(2, 0, 146, TITLE, TITLE)];
        get(&mut s, 2).top = 258;
        stack(&mut s, false);
        assert_eq!(ys(&s, &[2]), vec![146]);
    }
}
