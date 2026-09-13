//! Window-to-window geometry: drag snapping against other fences / the work area, and the drag-to-merge target under the cursor.

use super::*;

/// Another fence's window whose title row is under the cursor (drag-to-merge target).
pub(super) fn merge_target_under_cursor(me: HWND) -> Option<HWND> {
    merge_target_under_cursor_excluding(me, HWND(std::ptr::null_mut()))
}

pub(super) fn merge_target_under_cursor_excluding(me: HWND, also: HWND) -> Option<HWND> {
    let pt = window::cursor_pos();
    merge_target_at(me, also, (pt.x, pt.y))
}

pub(super) fn merge_target_at(me: HWND, also: HWND, point: (i32, i32)) -> Option<HWND> {
    desktop::top_level_windows().into_iter().find(|&w| {
        if w == me
            || w == also
            || desktop::class_name(w) != anchor::FENCE_CLASS
            || !desktop::is_visible(w)
        {
            return false;
        }
        let r = window::window_rect(w);
        let title_h = (36.0 * monitors::dpi_for_window(w).max(96) as f32 / 96.0) as i32;
        point.0 >= r.left && point.0 < r.right && point.1 >= r.top && point.1 < r.top + title_h
    })
}

/// Snaps `rect` (being dragged) to the edges of other fence windows and the monitor work area.
pub(super) fn snap_rect(rect: &mut RECT, me: HWND, gap: i32, dist: i32) {
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
    for other in desktop::top_level_windows() {
        if other == me
            || desktop::class_name(other) != anchor::FENCE_CLASS
            || !desktop::is_visible(other)
        {
            continue;
        }
        let o = window::window_rect(other);
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
    // Work area of the monitor under the rect centre.
    let mon =
        monitors::monitor_from_point((rect.left + rect.right) / 2, (rect.top + rect.bottom) / 2);
    if let Some(info) = monitors::query(mon) {
        let wa = info.work_area;
        consider_x(wa.left + gap);
        consider_x(wa.right - gap - w);
        consider_y(wa.top + gap);
        consider_y(wa.bottom - gap - h);
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
