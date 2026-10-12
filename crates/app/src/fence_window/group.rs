//! Fences selected together with a marquee on the empty desktop (`Behavior::selection`, owned by
//! the App): a title drag of one of them moves the others along, each keeping its offset, and
//! the group's bounding box snaps to the fences outside it and the work area like a single
//! fence does. Group drags never merge into a tab strip.

use super::*;
use crate::drag_guides::GuideLine;

/// Another selected fence taking part in a group drag, with its rect when the drag began.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GroupMember {
    pub hwnd: HWND,
    pub fence: FenceId,
    pub start: RECT,
}

/// The rest of the selection when `me` starts a title drag and is part of it; empty otherwise.
/// Windows that are gone or hidden meanwhile are left out.
pub(super) fn group_members(behavior: &Behavior, me: HWND) -> Vec<GroupMember> {
    let selection = behavior.selection.borrow();
    if !selection.iter().any(|(h, _)| *h == me) {
        return Vec::new();
    }
    selection
        .iter()
        .filter(|(h, _)| *h != me && window::is_window(*h) && desktop::is_visible(*h))
        .map(|&(hwnd, fence)| GroupMember {
            hwnd,
            fence,
            start: window::window_rect(hwnd),
        })
        .collect()
}

/// A press on this fence ends the selection, except a title press on a selected fence (it may
/// start a group drag; a title click without a drag ends the selection on release).
pub(super) fn press_ends_selection(
    behavior: &Behavior,
    hwnd: HWND,
    message: u32,
    wparam: usize,
) -> bool {
    let press = matches!(
        message,
        msg::WM_NCLBUTTONDOWN
            | msg::WM_NCLBUTTONDBLCLK
            | WM_NCRBUTTONDOWN
            | msg::WM_LBUTTONDOWN
            | msg::WM_LBUTTONDBLCLK
            | msg::WM_RBUTTONDOWN
    );
    if !press {
        return false;
    }
    let Ok(selection) = behavior.selection.try_borrow() else {
        return false;
    };
    if selection.is_empty() {
        return false;
    }
    let group_drag = message == msg::WM_NCLBUTTONDOWN
        && wparam as isize == msg::HTCAPTION
        && selection.iter().any(|(h, _)| *h == hwnd);
    !group_drag
}

/// Screen rectangles of the showing fence windows outside the group (`me` and `members`).
pub(super) fn fence_rects_outside(me: HWND, members: &[GroupMember]) -> Vec<RECT> {
    desktop::windows_of_class(anchor::FENCE_CLASS)
        .into_iter()
        .filter(|&w| w != me && !members.iter().any(|m| m.hwnd == w) && desktop::is_visible(w))
        .map(window::window_rect)
        .collect()
}

fn shifted(r: &RECT, dx: i32, dy: i32) -> RECT {
    RECT {
        left: r.left + dx,
        top: r.top + dy,
        right: r.right + dx,
        bottom: r.bottom + dy,
    }
}

/// Where the group goes when the dragged fence, at `start` when the drag began, is placed at
/// `rect`: every member keeps its offset to it. With `snap` (gap, capture distance) the group's
/// bounding box snaps to `others` (the fences outside the group) and the work area `work_for`
/// finds under the box; `rect` takes the same shift. Returns the members' rects (in order) and
/// the guides of the snapped box.
pub(super) fn place_group(
    rect: &mut RECT,
    start: &RECT,
    members: &[GroupMember],
    others: &[RECT],
    work_for: impl Fn(&RECT) -> Option<RECT>,
    snap: Option<(i32, i32)>,
) -> (Vec<RECT>, Vec<GuideLine>) {
    let (dx, dy) = (rect.left - start.left, rect.top - start.top);
    let bbox = members.iter().fold(*rect, |b, m| {
        let r = shifted(&m.start, dx, dy);
        RECT {
            left: b.left.min(r.left),
            top: b.top.min(r.top),
            right: b.right.max(r.right),
            bottom: b.bottom.max(r.bottom),
        }
    });
    let mut guides = Vec::new();
    let (mut sx, mut sy) = (0, 0);
    if let Some((gap, dist)) = snap {
        let work = work_for(&bbox);
        let mut snapped = bbox;
        snap_among(&mut snapped, others, work.as_ref(), gap, dist);
        guides = alignment_guides(&snapped, others, work.as_ref(), dist);
        (sx, sy) = (snapped.left - bbox.left, snapped.top - bbox.top);
    }
    *rect = shifted(rect, sx, sy);
    let placed = members
        .iter()
        .map(|m| shifted(&m.start, dx + sx, dy + sy))
        .collect();
    (placed, guides)
}

/// Moves each member's top-left to its rect in `placed` (sizes stay), skipping the ones already
/// there. Call with no view borrow held: the members' handlers run synchronously.
pub(super) fn move_members(members: &[GroupMember], placed: &[RECT]) {
    for (m, r) in members.iter().zip(placed) {
        let now = window::window_rect(m.hwnd);
        if (now.left, now.top) != (r.left, r.top) {
            let _ = window::move_window_to(m.hwnd, r.left, r.top);
        }
    }
}

/// Puts the members back where the drag found them (Esc / right button).
pub(super) fn restore_members(members: &[GroupMember]) {
    let starts: Vec<RECT> = members.iter().map(|m| m.start).collect();
    move_members(members, &starts);
}

/// A finished group drag: every member's new rect is recorded (where it was dropped is where it
/// rests now) and, like a single fence let go over another one, moved out of an overlap.
pub(super) fn commit_members(queue: &CommandQueue, behavior: &Behavior, members: &[GroupMember]) {
    for m in members {
        behavior.release_push(m.hwnd);
        queue_bounds_changed(queue, behavior, m.fence, m.hwnd);
    }
    for m in members {
        queue.push(Command::FenceDropped(m.fence));
    }
}
