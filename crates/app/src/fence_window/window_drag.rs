//! Latest-pointer window movement, applied once per compositor tick.
use super::*;

pub(super) fn passed_drag_threshold(
    start: (i32, i32),
    point: (i32, i32),
    threshold: (i32, i32),
) -> bool {
    point.0.abs_diff(start.0) > threshold.0 as u32 || point.1.abs_diff(start.1) > threshold.1 as u32
}

/// A grab offset measured at `from_dpi`, for a window now at `to_dpi`: a drag onto a monitor
/// with another scale resizes the window (WM_DPICHANGED), so the cursor keeps holding the same
/// point of it only if the offset scales too. Unknown DPI (0) leaves it as it is.
pub(super) fn grab_offset_for_dpi(offset: (i32, i32), from_dpi: u32, to_dpi: u32) -> (i32, i32) {
    if from_dpi == 0 || to_dpi == 0 || from_dpi == to_dpi {
        return offset;
    }
    let k = to_dpi as f32 / from_dpi as f32;
    (
        (offset.0 as f32 * k).round() as i32,
        (offset.1 as f32 * k).round() as i32,
    )
}

impl RemoteDrag {
    /// Keep only the newest sample. Mouse-up uses this too, so a fast gesture that
    /// finishes before the frame callback still reaches its final position.
    pub(super) fn track_pointer(&mut self, point: (i32, i32), threshold: (i32, i32)) {
        if point == self.last_pointer {
            return;
        }
        self.last_pointer = point;
        self.requests += 1;
        if !self.started && passed_drag_threshold(self.press, point, threshold) {
            self.started = true;
        }
        if self.started {
            self.pending_pointer = Some(point);
        }
    }
}

/// HWND placement and its wallpaper update run together after the frame signal.
/// A final release point is supplied by WM_LBUTTONUP, never sampled from a cursor
/// that may already have moved on while the release waited in the message queue.
pub(super) fn flush_window_drag(
    view: &ViewCell,
    controller: HWND,
    now: Instant,
    release: Option<(i32, i32)>,
) -> bool {
    let (target_hwnd, point, offset, snapping, gap_dip, override_height, behavior, radius, group) = {
        let mut guard = view.borrow_mut();
        let Some(v) = guard.as_mut() else {
            return false;
        };
        let height = v
            .roll_anim
            .as_ref()
            .map(|a| a.height.value_at(now).round() as i32);
        let Some(drag) = v.remote_drag.as_mut() else {
            return false;
        };
        if let Some(point) = release {
            drag.track_pointer(point, window::drag_threshold());
        }
        if !drag.started {
            return false;
        }
        let Some(point) = drag.pending_pointer.take() else {
            return false;
        };
        (
            drag.hwnd,
            point,
            grab_offset_for_dpi(
                drag.offset,
                drag.offset_dpi,
                monitors::dpi_for_window(drag.hwnd),
            ),
            v.behavior.snapping.get(),
            v.behavior.snap_gap_dip.get(),
            (drag.hwnd == controller).then_some(height).flatten(),
            v.behavior.clone(),
            v.theme.corner_radius,
            match &drag.origin {
                WindowDragOrigin::Caption { rect, .. } if !drag.group.is_empty() => {
                    Some((*rect, drag.group.clone()))
                }
                _ => None,
            },
        )
    };
    let began = Instant::now();
    let old = window::window_rect(target_hwnd);
    let mut rect = RECT {
        left: point.0 - offset.0,
        top: point.1 - offset.1,
        right: point.0 - offset.0 + old.right - old.left,
        bottom: point.1 - offset.1 + override_height.unwrap_or(old.bottom - old.top),
    };
    // A group drag never joins a tab strip.
    let merge = if group.is_none() {
        merge_target_at(target_hwnd, HWND::default(), point)
    } else {
        None
    };
    let target = merge.map_or(0, |h| h.0 as isize);
    let target_done = Instant::now();
    let mut guides = Vec::new();
    let mut placed = Vec::new();
    if let Some((start, members)) = &group {
        let scale = monitors::dpi_for_window(target_hwnd).max(96) as f32 / 96.0;
        let snap = (snapping && !snap_paused()).then_some((
            (gap_dip as f32 * scale) as i32,
            (SNAP_DIST_DIP as f32 * scale) as i32,
        ));
        let others = fence_rects_outside(target_hwnd, members);
        (placed, guides) = place_group(&mut rect, start, members, &others, work_area_under, snap);
    } else if snapping && target == 0 && !snap_paused() {
        let scale = monitors::dpi_for_window(target_hwnd).max(96) as f32 / 96.0;
        guides = snap_rect(
            &mut rect,
            target_hwnd,
            (gap_dip as f32 * scale) as i32,
            (SNAP_DIST_DIP as f32 * scale) as i32,
        );
    }
    let snap_done = Instant::now();
    let (queue, hint_changed, requests, applied) = {
        let mut guard = view.borrow_mut();
        let Some(v) = guard.as_mut() else {
            return false;
        };
        let Some(drag) = v.remote_drag.as_mut().filter(|d| d.hwnd == target_hwnd) else {
            return false;
        };
        let changed = drag.merge_target != target || (target != 0 && drag.merge_at != point);
        drag.merge_target = target;
        drag.merge_at = point;
        if rect != old {
            drag.applied += 1;
        }
        drag.moved_once = true;
        (v.queue.clone(), changed, drag.requests, drag.applied)
    };
    let moved = rect != old;
    if moved {
        let _ = window::set_window_bounds(
            target_hwnd,
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
        );
    }
    match &group {
        Some((_, members)) => {
            move_members(members, &placed);
            show_guides(&behavior, target_hwnd, &guides);
        }
        None => update_drag_feedback(&behavior, target_hwnd, &rect, merge, &guides, radius),
    }
    if hint_changed {
        queue.push(Command::MergeHint {
            target: HWND(target as *mut core::ffi::c_void),
            at: point,
        });
    }
    tracing::trace!(target: "pecofence::drag_frame", requests, applied, finish = release.is_some(),
        members = placed.len(),
        target_us = target_done.duration_since(began).as_micros(),
        snap_us = snap_done.duration_since(target_done).as_micros(),
        move_us = snap_done.elapsed().as_micros(), "window drag frame");
    moved && target_hwnd == controller
}
