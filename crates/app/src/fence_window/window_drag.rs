//! Latest-pointer window movement, applied once per compositor tick.
use super::*;

pub(super) fn passed_drag_threshold(
    start: (i32, i32),
    point: (i32, i32),
    threshold: (i32, i32),
) -> bool {
    point.0.abs_diff(start.0) > threshold.0 as u32 || point.1.abs_diff(start.1) > threshold.1 as u32
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
    let (target_hwnd, point, offset, snapping, override_height) = {
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
            drag.offset,
            v.behavior.snapping.get(),
            (drag.hwnd == controller).then_some(height).flatten(),
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
    let target = merge_target_at(target_hwnd, HWND::default(), point).map_or(0, |h| h.0 as isize);
    let target_done = Instant::now();
    if snapping && target == 0 {
        let scale = monitors::dpi_for_window(target_hwnd).max(96) as f32 / 96.0;
        snap_rect(
            &mut rect,
            target_hwnd,
            (SNAP_GAP_DIP as f32 * scale) as i32,
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
        let changed = drag.merge_target != target || (target != 0 && drag.merge_x != point.0);
        drag.merge_target = target;
        drag.merge_x = point.0;
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
    if hint_changed {
        queue.push(Command::MergeHint {
            target: HWND(target as *mut core::ffi::c_void),
            x: point.0,
        });
    }
    tracing::trace!(target: "pecofence::drag_frame", requests, applied, finish = release.is_some(),
        target_us = target_done.duration_since(began).as_micros(),
        snap_us = snap_done.duration_since(target_done).as_micros(),
        move_us = snap_done.elapsed().as_micros(), "window drag frame");
    moved && target_hwnd == controller
}
