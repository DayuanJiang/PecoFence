//! Window-to-window geometry: drag snapping against other fences, the work area and its centre
//! lines, the alignment guides and white outline shown meanwhile, and the drag-to-merge target
//! under the cursor.

use super::*;
use crate::drag_guides::GuideLine;

/// Another fence's window whose title row is under the cursor (drag-to-merge target).
pub(super) fn merge_target_under_cursor(me: HWND) -> Option<HWND> {
    merge_target_under_cursor_excluding(me, HWND(std::ptr::null_mut()))
}

pub(super) fn merge_target_under_cursor_excluding(me: HWND, also: HWND) -> Option<HWND> {
    let pt = window::cursor_pos();
    merge_target_at(me, also, (pt.x, pt.y))
}

pub(super) fn merge_target_at(me: HWND, also: HWND, point: (i32, i32)) -> Option<HWND> {
    desktop::windows_of_class(anchor::FENCE_CLASS)
        .into_iter()
        .find(|&w| {
            if w == me || w == also || !desktop::is_visible(w) {
                return false;
            }
            let r = window::window_rect(w);
            let title_h = (36.0 * monitors::dpi_for_window(w).max(96) as f32 / 96.0) as i32;
            point.0 >= r.left && point.0 < r.right && point.1 >= r.top && point.1 < r.top + title_h
        })
}

thread_local! {
    /// Alt was down at the last `snap_paused` of this drag (its release is already masked).
    static ALT_SEEN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Alt held while dragging or resizing pauses snapping and its guides. Each time Alt is first
/// seen down, an unassigned key is tapped so that releasing Alt does not reach the foreground
/// window (fences never activate) as a lone Alt press, which would open its menu bar.
pub(super) fn snap_paused() -> bool {
    let down = window::key_down_async(msg::VK_MENU);
    if down && !ALT_SEEN.with(|s| s.get()) {
        desktop::mask_alt_release();
    }
    ALT_SEEN.with(|s| s.set(down));
    down
}

/// While `hwnd` is dragged to `rect`: the white outline around the fence it would join as a tab
/// (`merge`), else around the spot it will move to on release when it covers another fence
/// (`FenceDropped`). The alignment `guides` show only while neither applies. `radius_dip` is
/// the fences' corner radius.
pub(super) fn update_drag_feedback(
    behavior: &Behavior,
    hwnd: HWND,
    rect: &RECT,
    merge: Option<HWND>,
    guides: &[GuideLine],
    radius_dip: f32,
) {
    let dpi = monitors::dpi_for_window(hwnd).max(96);
    let gap = (behavior.snap_gap_dip.get() as f32 * dpi as f32 / 96.0).round() as i32;
    let (covering, outline) = match merge {
        Some(target) => (true, Some(window::window_rect(target))),
        None => {
            let others = other_fence_rects(hwnd);
            if others
                .iter()
                .any(|o| crate::layout::clearance::overlaps(rect, o, gap))
            {
                let spot = work_area_under(rect).and_then(|work| {
                    crate::layout::clearance::nearest_free(rect, &others, &work, gap)
                });
                (true, spot)
            } else {
                (false, None)
            }
        }
    };
    if let Ok(mut preview) = behavior.drop_preview.try_borrow_mut()
        && let Some(preview) = preview.as_mut()
    {
        match outline {
            Some(r) => preview.show_at(r, dpi, radius_dip, hwnd),
            None => preview.hide(),
        }
    }
    show_guides(behavior, hwnd, if covering { &[] } else { guides });
}

/// Shows `lines` above `hwnd` (hidden when empty or `snapping.guideLines` is off).
pub(super) fn show_guides(behavior: &Behavior, hwnd: HWND, lines: &[GuideLine]) {
    let Ok(mut guides) = behavior.drag_guides.try_borrow_mut() else {
        return;
    };
    let Some(guides) = guides.as_mut() else {
        return;
    };
    if lines.is_empty() || !behavior.guide_lines.get() {
        guides.hide();
    } else {
        guides.show(lines, monitors::dpi_for_window(hwnd).max(96), hwnd);
    }
}

/// Drag or resize over: outline and guides go away (and the next Alt press is masked again).
pub(super) fn hide_drag_feedback(behavior: &Behavior) {
    ALT_SEEN.with(|s| s.set(false));
    if let Ok(mut preview) = behavior.drop_preview.try_borrow_mut()
        && let Some(preview) = preview.as_mut()
    {
        preview.hide();
    }
    if let Ok(mut guides) = behavior.drag_guides.try_borrow_mut()
        && let Some(guides) = guides.as_mut()
    {
        guides.hide();
    }
}

/// Screen rectangles of the other fence windows that are showing.
pub(super) fn other_fence_rects(me: HWND) -> Vec<RECT> {
    desktop::windows_of_class(anchor::FENCE_CLASS)
        .into_iter()
        .filter(|&w| w != me && desktop::is_visible(w))
        .map(window::window_rect)
        .collect()
}

/// Work area of the monitor under `rect`'s centre.
pub(super) fn work_area_under(rect: &RECT) -> Option<RECT> {
    let mon =
        monitors::monitor_from_point((rect.left + rect.right) / 2, (rect.top + rect.bottom) / 2);
    monitors::query(mon).map(|info| info.work_area)
}

/// Snaps `rect` (being dragged) to the other fence windows and the work area of the monitor
/// under its centre; returns the alignment guides of the snapped position.
pub(super) fn snap_rect(rect: &mut RECT, me: HWND, gap: i32, dist: i32) -> Vec<GuideLine> {
    let others = other_fence_rects(me);
    let work = work_area_under(rect);
    snap_among(rect, &others, work.as_ref(), gap, dist);
    alignment_guides(rect, &others, work.as_ref(), dist)
}

/// Moves `rect` by the shortest pull within `dist` on each axis, towards: outer edges a `gap`
/// from a nearby fence's, edges level with a nearby fence's, the work-area edges (`gap`
/// inside) and the work area's centre lines.
pub(super) fn snap_among(
    rect: &mut RECT,
    others: &[RECT],
    work: Option<&RECT>,
    gap: i32,
    dist: i32,
) {
    let w = rect.right - rect.left;
    let h = rect.bottom - rect.top;
    let mut best_dx: Option<i32> = None;
    let mut best_dy: Option<i32> = None;
    let mut consider_x = |candidate_left: i32| {
        let d = candidate_left - rect.left;
        if d.abs() <= dist && best_dx.is_none_or(|b| d.abs() < b.abs()) {
            best_dx = Some(d);
        }
    };
    let mut consider_y = |candidate_top: i32| {
        let d = candidate_top - rect.top;
        if d.abs() <= dist && best_dy.is_none_or(|b| d.abs() < b.abs()) {
            best_dy = Some(d);
        }
    };
    // Other fences: align outer edges with a gap, and align same edges.
    for o in others {
        let overlap_y = rect.top < o.bottom + dist && rect.bottom > o.top - dist;
        let overlap_x = rect.left < o.right + dist && rect.right > o.left - dist;
        if overlap_y {
            consider_x(o.right + gap); // my left to their right
            consider_x(o.left - gap - w); // my right to their left
            consider_x(o.left); // same left
            consider_x(o.right - w); // same right
        }
        if overlap_x {
            consider_y(o.bottom + gap);
            consider_y(o.top - gap - h);
            consider_y(o.top);
            consider_y(o.bottom - h);
        }
    }
    if let Some(wa) = work {
        consider_x(wa.left + gap);
        consider_x(wa.right - gap - w);
        consider_x((wa.left + wa.right) / 2 - w / 2); // centred on the screen
        consider_y(wa.top + gap);
        consider_y(wa.bottom - gap - h);
        consider_y((wa.top + wa.bottom) / 2 - h / 2);
    }
    if let Some(dx) = best_dx {
        rect.left += dx;
        rect.right += dx;
    }
    if let Some(dy) = best_dy {
        rect.top += dy;
        rect.bottom += dy;
    }
}

/// Which edges a WM_SIZING drag moves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SizingEdges {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
}

impl SizingEdges {
    /// From WM_SIZING's `wParam` (WMSZ_LEFT = 1 … WMSZ_BOTTOMRIGHT = 8).
    pub fn from_wmsz(wmsz: usize) -> Self {
        Self {
            left: matches!(wmsz, 1 | 4 | 7),
            right: matches!(wmsz, 2 | 5 | 8),
            top: matches!(wmsz, 3..=5),
            bottom: matches!(wmsz, 6..=8),
        }
    }
}

/// Whole-cell sizes along one axis (device px): `fixed + n * step`, n ≥ 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Cells {
    pub fixed: f32,
    pub step: f32,
}

impl Cells {
    fn nearest(self, size: i32) -> i32 {
        let n = ((size as f32 - self.fixed) / self.step).round().max(1.0);
        (self.fixed + n * self.step).round() as i32
    }
}

/// How a fence window may be resized (device px).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SizeRules {
    /// Whole icon columns / rows (「调整大小时按整行整列对齐」); None = any size.
    pub cols: Option<Cells>,
    pub rows: Option<Cells>,
    pub min_w: i32,
    pub min_h: i32,
    /// Rolled up: exactly this tall (the title bar), whichever edge is dragged.
    pub rolled_h: Option<i32>,
}

/// The other fences a resized edge lines up with and stops at (snapping on).
pub(super) struct Neighbours<'a> {
    pub others: &'a [RECT],
    pub work: Option<&'a RECT>,
    pub gap: i32,
    pub dist: i32,
    /// Alt is not held: edges snap to the alignment targets and guides show.
    pub align: bool,
}

/// WM_SIZING: places the dragged `edges` of `rect` (where the pointer puts them; `cur` is the
/// window now). With `near`, an edge within `dist` of an alignment target goes there, whatever
/// the icon sizes: another fence's edge, a gap outside a fence across from it, or the work
/// area's edge a gap inside. Otherwise it takes whole cells when `rules` has them. An edge never
/// passes a fence lying beyond it (it stops a gap short). Returns the guides along dragged edges
/// that are level with another fence's edge.
pub(super) fn size_among(
    rect: &mut RECT,
    cur: &RECT,
    edges: SizingEdges,
    rules: &SizeRules,
    near: Option<&Neighbours>,
) -> Vec<GuideLine> {
    let align = near.filter(|n| n.align);
    let dist = align.map_or(0, |n| n.dist);
    let targets = |vertical: bool, max_side: bool, rect: &RECT| {
        align.map_or_else(Vec::new, |n| edge_targets(n, rect, vertical, max_side))
    };
    if edges.right {
        let t = targets(true, true, rect);
        rect.right = place_edge(rect.left, rect.right, 1, rules.cols, rules.min_w, &t, dist);
    } else if edges.left {
        let t = targets(true, false, rect);
        rect.left = place_edge(rect.right, rect.left, -1, rules.cols, rules.min_w, &t, dist);
    }
    if let Some(h) = rules.rolled_h {
        // Keyboard SC_SIZE can still send top / bottom codes to a rolled fence.
        if edges.top {
            rect.top = rect.bottom - h;
        } else {
            rect.bottom = rect.top + h;
        }
    } else if edges.bottom {
        let t = targets(false, true, rect);
        rect.bottom = place_edge(rect.top, rect.bottom, 1, rules.rows, rules.min_h, &t, dist);
    } else if edges.top {
        let t = targets(false, false, rect);
        rect.top = place_edge(rect.bottom, rect.top, -1, rules.rows, rules.min_h, &t, dist);
    }
    let Some(n) = near else {
        return Vec::new();
    };
    // Exactly a gap short of a fence beyond the edge (fences it already overlaps do not count),
    // not a whole cell short: stacked fences keep even gaps.
    let lim = crate::layout::clearance::sizing_limits(n.others, cur, rect, n.gap);
    if edges.right
        && let Some(limit) = lim.right
    {
        rect.right = rect.right.min(limit);
    }
    if edges.left
        && let Some(limit) = lim.left
    {
        rect.left = rect.left.max(limit);
    }
    if rules.rolled_h.is_none() {
        if edges.bottom
            && let Some(limit) = lim.bottom
        {
            rect.bottom = rect.bottom.min(limit);
        }
        if edges.top
            && let Some(limit) = lim.top
        {
            rect.top = rect.top.max(limit);
        }
    }
    if !n.align {
        return Vec::new();
    }
    alignment_guides(rect, n.others, None, i32::MAX / 4)
        .into_iter()
        .filter(|g| {
            if g.vertical {
                (edges.right && g.at == rect.right) || (edges.left && g.at == rect.left)
            } else {
                (edges.bottom && g.at == rect.bottom) || (edges.top && g.at == rect.top)
            }
        })
        .collect()
}

/// Where a dragged edge goes: `anchor` is the opposite edge, `edge` the pointer's, `sign` +1 for
/// a right / bottom edge and -1 for a left / top one. The nearest target within `dist` that
/// leaves at least `min` wins; else whole `cells`, else the pointer (at least `min`).
fn place_edge(
    anchor: i32,
    edge: i32,
    sign: i32,
    cells: Option<Cells>,
    min: i32,
    targets: &[i32],
    dist: i32,
) -> i32 {
    targets
        .iter()
        .copied()
        .filter(|&t| (t - edge).abs() <= dist && (t - anchor) * sign >= min)
        .min_by_key(|&t| (t - edge).abs())
        .unwrap_or_else(|| {
            let size = (edge - anchor) * sign;
            anchor + sign * cells.map_or(size, |c| c.nearest(size)).max(min)
        })
}

/// Alignment targets for a dragged edge of `rect`: a vertical (left / right) edge when
/// `vertical`, the right / bottom one when `max_side`. Every other fence's two edges on that
/// axis; the edge a gap outside a fence across from `rect` (overlapping it on the other axis
/// within `dist`); the work area's edge a gap inside.
fn edge_targets(n: &Neighbours, rect: &RECT, vertical: bool, max_side: bool) -> Vec<i32> {
    let mut targets = Vec::with_capacity(n.others.len() * 3 + 1);
    for o in n.others {
        let (lo, hi, across) = if vertical {
            let across = rect.top < o.bottom + n.dist && rect.bottom > o.top - n.dist;
            (o.left, o.right, across)
        } else {
            let across = rect.left < o.right + n.dist && rect.right > o.left - n.dist;
            (o.top, o.bottom, across)
        };
        targets.extend([lo, hi]);
        if across {
            targets.push(if max_side { lo - n.gap } else { hi + n.gap });
        }
    }
    if let Some(w) = n.work {
        targets.push(match (vertical, max_side) {
            (true, true) => w.right - n.gap,
            (true, false) => w.left + n.gap,
            (false, true) => w.bottom - n.gap,
            (false, false) => w.top + n.gap,
        });
    }
    targets
}

/// Guides for `rect` as placed: each of its edges that lies exactly on an edge of a fence within
/// `dist` across (a line spanning both), and the work area's centre line through its centre
/// (spanning the work area).
pub(super) fn alignment_guides(
    rect: &RECT,
    others: &[RECT],
    work: Option<&RECT>,
    dist: i32,
) -> Vec<GuideLine> {
    let mut lines: Vec<GuideLine> = Vec::new();
    let mut add = |vertical: bool, at: i32, from: i32, to: i32| match lines
        .iter_mut()
        .find(|g| g.vertical == vertical && g.at == at)
    {
        Some(g) => {
            g.from = g.from.min(from);
            g.to = g.to.max(to);
        }
        None => lines.push(GuideLine {
            vertical,
            at,
            from,
            to,
        }),
    };
    for o in others {
        if rect.top < o.bottom + dist && rect.bottom > o.top - dist {
            let (from, to) = (rect.top.min(o.top), rect.bottom.max(o.bottom));
            for x in [rect.left, rect.right] {
                if x == o.left || x == o.right {
                    add(true, x, from, to);
                }
            }
        }
        if rect.left < o.right + dist && rect.right > o.left - dist {
            let (from, to) = (rect.left.min(o.left), rect.right.max(o.right));
            for y in [rect.top, rect.bottom] {
                if y == o.top || y == o.bottom {
                    add(false, y, from, to);
                }
            }
        }
    }
    if let Some(wa) = work {
        let (cx, cy) = ((wa.left + wa.right) / 2, (wa.top + wa.bottom) / 2);
        if rect.left == cx - (rect.right - rect.left) / 2 {
            add(true, cx, wa.top, wa.bottom);
        }
        if rect.top == cy - (rect.bottom - rect.top) / 2 {
            add(false, cy, wa.left, wa.right);
        }
    }
    lines
}
