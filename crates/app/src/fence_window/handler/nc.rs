//! Non-client / title-row / sizing / moving / geometry messages.

use super::*;

// Keep DWM materials in their "active" state even though we never activate.
pub(super) fn on_ncactivate_acrylic(
    _h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    Some(window::def_window_proc(hwnd, msg::WM_NCACTIVATE, 1, -1))
}

pub(super) fn on_windowposchanging(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    lparam: isize,
) -> Option<isize> {
    if !anchor::is_anchoring() {
        // SAFETY: lparam is the WINDOWPOS of the message being processed.
        unsafe { window::windowpos_deny_zorder_change(lparam) };
    }
    // Prepare the new sampling transform before the HWND moves. The following WM_MOVE
    // finds the same rectangle and only moves the shadow. Resizes remain in WM_SIZE,
    // after the composition surfaces and their rounded clip have the new dimensions.
    if let Ok(mut guard) = h.view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
        && v.theme.liquid_glass
        // SAFETY: this is the WINDOWPOS of the message; fences are top-level windows.
        && let Some(next) = unsafe {
            window::windowpos_move_rect(lparam, window::window_rect(hwnd))
        }
    {
        let started = Instant::now();
        let old_ink = v.material_foreground().text_primary;
        if v.update_backdrop_at(next) {
            if old_ink != v.material_foreground().text_primary || v.row_metrics().is_some() {
                let _ = v.redraw();
            } else {
                let _ = v.redraw_chrome_only();
            }
            tracing::trace!(
                us = started.elapsed().as_micros(),
                "GPU glass move preparation"
            );
        }
    }
    None
}

pub(super) fn on_nchittest(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    let x = msg::lo_i16(lparam);
    let y = msg::hi_i16(lparam);
    // WindowFromPoint & friends send WM_NCHITTEST synchronously while another
    // arm may hold the borrow: degrade to HTCLIENT instead of panicking.
    let Ok(guard) = view.try_borrow() else {
        return Some(msg::HTCLIENT);
    };
    let Some(v) = guard.as_ref() else {
        return Some(msg::HTCLIENT);
    };
    let scale = v.scale();
    let border = (RESIZE_BORDER_DIP * scale).round() as i32;
    // Beside a left tab column there is no title row: the column is the title bar.
    let title_h = if v.tabs_on_rail() { 0 } else { v.title_h_px() };
    let rect = window::window_rect(hwnd);
    let left = x - rect.left;
    let top = y - rect.top;
    let right = rect.right - x;
    let bottom = rect.bottom - y;
    let w = rect.right - rect.left;
    let h = rect.bottom - rect.top;
    if left < 0 || top < 0 || left >= w || top >= h {
        return Some(msg::HTNOWHERE);
    }
    let rolled = v.rolled_up;
    // Tab headers and the title bar's buttons take clicks (switch / drag out,
    // up, roll); the rest of the title row moves the window, and so does the
    // free part of a left tab column. A fence collapsed to its column has no
    // resize edges.
    let in_rail = left < v.content_x;
    let on_control =
        v.tab_at(left, top).is_some() || v.up_button_at(left, top) || v.chevron_at(left, top);
    let clear_of_edges = if in_rail {
        rolled || (left >= border && bottom > border)
    } else {
        top < title_h && left >= border && right > border
    };
    if on_control && clear_of_edges {
        return Some(msg::HTCLIENT);
    }
    if v.locked {
        // Locked: no resize edges; the title still takes clicks/menus.
        return Some(if top < title_h || in_rail {
            msg::HTCAPTION
        } else {
            msg::HTCLIENT
        });
    }
    if rolled && v.tabs_on_rail() {
        // Collapsed to the column: it only moves.
        return Some(msg::HTCAPTION);
    }
    if rolled {
        // Title-only (Fences): only the width resizes; the rest of the strip,
        // corners included, moves the window.
        return Some(if left < border {
            msg::HTLEFT
        } else if right <= border {
            msg::HTRIGHT
        } else {
            msg::HTCAPTION
        });
    }
    let code = match (
        left < border,
        right <= border,
        top < border,
        bottom <= border,
    ) {
        (true, _, true, _) => msg::HTTOPLEFT,
        (_, true, true, _) => msg::HTTOPRIGHT,
        (true, _, _, true) => msg::HTBOTTOMLEFT,
        (_, true, _, true) => msg::HTBOTTOMRIGHT,
        (true, _, _, _) => msg::HTLEFT,
        (_, true, _, _) => msg::HTRIGHT,
        (_, _, true, _) => msg::HTTOP,
        (_, _, _, true) => msg::HTBOTTOM,
        _ if top < title_h || in_rail => msg::HTCAPTION,
        _ => msg::HTCLIENT,
    };
    Some(code)
}

pub(super) fn on_ncmousemove(
    h: &HandlerCtx,
    hwnd: HWND,
    wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view,
        behavior,
        peek_armed,
        menu_open,
        in_size_move,
        ..
    } = h;
    let mut guard = view.borrow_mut();
    if let Some(v) = guard.as_mut() {
        v.set_mouse_inside(true);
        // Beside a left tab column the column is the title bar (the only caption area there).
        let top = window::window_rect(hwnd).top;
        let is_title = wparam as isize == msg::HTCAPTION
            && (v.tabs_on_rail() || msg::hi_i16(lparam) - top < v.title_h_px());
        let (item, title) = v.set_hover(None, is_title);
        if title {
            let _ = v.redraw_chrome_only();
        }
        if item && !v.rolled_up {
            let _ = v.redraw_content();
        }
        // Click-to-expand pressed and now moving: hand the drag to the
        // system move loop (the press itself was swallowed).
        if let Some((px, py)) = v.click_pending
            && window::key_down(msg::VK_LBUTTON)
        {
            let (tx, ty) = window::drag_threshold();
            let (cx, cy) = (msg::lo_i16(lparam), msg::hi_i16(lparam));
            if (cx - px).abs() > tx || (cy - py).abs() > ty {
                v.click_pending = None;
                drop(guard);
                window::send_message(
                    hwnd,
                    msg::WM_SYSCOMMAND,
                    SC_MOVE_CAPTION,
                    msg::make_lparam(px, py),
                );
                return Some(0);
            }
        }
        if v.rolled_up
            && !v.peeking
            && v.roll_anim.is_none()
            && behavior.hover_peek.get()
            && !behavior.click_to_expand.get()
            && !in_size_move.get()
            && !menu_open.get()
            && !peek_armed.replace(true)
        {
            window::set_timer(hwnd, TIMER_PEEK_OPEN, Tooltip::hover_delay_ms());
        }
        if v.peeking {
            window::kill_timer(hwnd, TIMER_PEEK_CLOSE);
        }
        window::track_mouse_leave_nc(hwnd);
    }
    None
}

pub(super) fn on_ncmouseleave(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view, peek_armed, ..
    } = h;
    // Leaving the title into the content is not leaving the window; leaving
    // onto a menu / an overlapping fence is, even while the cursor is still
    // inside our RECT. Hit-test before the borrow: WindowFromPoint sends
    // WM_NCHITTEST synchronously.
    let pt = window::cursor_pos();
    let inside = desktop::root_ancestor(desktop::window_from_point(pt.x, pt.y)) == hwnd;
    if !inside {
        // Still over our own client area (the chevron box, a tab): the
        // pending hover-peek keeps counting; WM_MOUSEMOVE takes over.
        window::kill_timer(hwnd, TIMER_PEEK_OPEN);
        peek_armed.set(false);
    }
    let mut guard = view.borrow_mut();
    if let Some(v) = guard.as_mut() {
        if !inside {
            v.set_mouse_inside(false);
            v.click_pending = None;
        }
        if v.title_hover {
            v.title_hover = false;
            let _ = v.redraw_chrome_only();
        }
        if v.peeking {
            window::set_timer(hwnd, TIMER_PEEK_CLOSE, PEEK_CLOSE_MS);
        }
    }
    None
}

pub(super) fn on_nclbuttondown(
    h: &HandlerCtx,
    hwnd: HWND,
    wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view,
        queue,
        behavior,
        peek_armed,
        ..
    } = h;
    // Title press (drag start / click): raise above the other fences once the
    // modal move loop returns. A pending hover-peek must not open inside that
    // loop (SC_MOVE caption press or SC_SIZE edge press alike).
    window::kill_timer(hwnd, TIMER_PEEK_OPEN);
    peek_armed.set(false);
    if let Some(v) = view.borrow_mut().as_mut() {
        v.suppress_dblclk = false;
        v.hide_tip();
        v.cancel_pending_rename();
    }
    queue.push(Command::RaiseFence(hwnd));
    if wparam as isize == msg::HTCAPTION {
        let start = (msg::lo_i16(lparam), msg::hi_i16(lparam));
        let rect = window::window_rect(hwnd);
        let custom = {
            let mut guard = view.borrow_mut();
            if let Some(v) = guard.as_mut() {
                if v.locked {
                    return Some(0);
                }
                if v.theme.liquid_glass {
                    v.height_anim = None;
                    v.remote_drag = Some(RemoteDrag {
                        hwnd,
                        fence: v.fence_id,
                        offset: (start.0 - rect.left, start.1 - rect.top),
                        offset_dpi: monitors::dpi_for_window(hwnd),
                        merge_target: 0,
                        merge_at: (i32::MIN, i32::MIN),
                        moved_once: false,
                        origin: WindowDragOrigin::Caption {
                            rect,
                            click_expand: behavior.click_to_expand.get()
                                && v.rolled_up
                                && !v.peeking
                                && v.roll_anim.is_none(),
                        },
                        group: group_members(behavior, hwnd),
                        press: start,
                        last_pointer: start,
                        pending_pointer: None,
                        started: false,
                        requests: 0,
                        applied: 0,
                    });
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        if custom {
            focus_session(view, hwnd);
            window::set_capture(hwnd);
            return Some(0);
        }
    }
    if wparam as isize == msg::HTCAPTION && behavior.click_to_expand.get() {
        // Swallow the press: a release without a drag expands the rolled
        // fence (WM_NCLBUTTONUP); a drag hands over to SC_MOVE.
        let mut guard = view.borrow_mut();
        if let Some(v) = guard.as_mut()
            && v.rolled_up
            && !v.peeking
            && v.roll_anim.is_none()
        {
            v.click_pending = Some((msg::lo_i16(lparam), msg::hi_i16(lparam)));
            return Some(0);
        }
    }
    None
}

pub(super) fn on_nclbuttonup(
    h: &HandlerCtx,
    _hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, queue, .. } = h;
    let fence_id = h.fence_id;
    let expand = view.borrow_mut().as_mut().is_some_and(|v| {
        let e = v.click_pending.take().is_some();
        if e {
            // The next press may arrive as WM_NCLBUTTONDBLCLK; it must not
            // re-roll the fence this click just expanded.
            v.suppress_dblclk = true;
        }
        e
    });
    if expand {
        queue.push(Command::ToggleRollUp(fence_id));
        return Some(0);
    }
    None
}

pub(super) fn on_nclbuttondblclk(
    h: &HandlerCtx,
    _hwnd: HWND,
    wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, queue, .. } = h;
    let fence_id = h.fence_id;
    if wparam as isize == msg::HTCAPTION {
        let suppressed = view
            .borrow_mut()
            .as_mut()
            .is_some_and(|v| std::mem::take(&mut v.suppress_dblclk));
        if suppressed {
            // Second half of a double-click on a title that the first click
            // already expanded (click-to-expand): same click, nothing to do.
            return Some(0);
        }
        let was_peeking = view.borrow_mut().as_mut().map(|v| {
            let p = v.peeking;
            v.peeking = false;
            p
        });
        if was_peeking == Some(true) {
            // Already expanded by peek: commit the expanded state.
            queue.push(Command::CommitExpanded(fence_id));
        } else {
            queue.push(Command::ToggleRollUp(fence_id));
        }
        return Some(0);
    }
    None
}

pub(super) fn on_ncrbuttonup(
    h: &HandlerCtx,
    hwnd: HWND,
    wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view,
        queue,
        peek_armed,
        ..
    } = h;
    let fence_id = h.fence_id;
    if wparam as isize == msg::HTCAPTION {
        // The peek must not open under the TrackPopupMenuEx loop.
        window::kill_timer(hwnd, TIMER_PEEK_OPEN);
        peek_armed.set(false);
        let fence = view
            .try_borrow()
            .ok()
            .and_then(|g| g.as_ref().map(|v| v.active))
            .unwrap_or(fence_id);
        queue.push(Command::FenceMenu {
            fence,
            x: msg::lo_i16(lparam),
            y: msg::hi_i16(lparam),
        });
        return Some(0);
    }
    None
}

pub(super) fn on_size(h: &HandlerCtx, hwnd: HWND, _wparam: usize, lparam: isize) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    let started = Instant::now();
    let w = msg::lo_i16(lparam);
    let h = msg::hi_i16(lparam);
    let mut guard = view.borrow_mut();
    if let Some(v) = guard.as_mut() {
        // A window created during early logon may have reported 96 DPI
        // before the display settled; re-read it on every size change.
        v.apply_dpi(monitors::dpi_for_window(hwnd));
        v.update_backdrop();
        let crop_done = Instant::now();
        let drawn = if v.height_animating() {
            // Frame-clock resize: the content surface is frozen at its
            // expanded size; only the chrome (backdrop, rim) follows.
            v.update_shape_clip(w, h)
                .and_then(|_| v.chrome_panel.resize(w, h))
                .and_then(|_| v.draw_chrome_panel())
        } else {
            // The cell grid is re-laid out for the new width: old origins
            // would glide to the wrong places.
            v.snap_item_motion();
            v.layout_panels(w, h).and_then(|_| v.redraw())
        };
        if let Err(e) = drawn {
            tracing::error!(error = %e, "fence resize/redraw failed");
        }
        let draw_done = Instant::now();
        v.update_shadow();
        if tracing::enabled!(target: "pecofence::frame_shape", tracing::Level::TRACE)
            && v.theme.liquid_glass
        {
            let r = window::window_rect(hwnd);
            let origin = window::screen_to_client(
                hwnd,
                pecofence_platform::POINT {
                    x: r.left,
                    y: r.top,
                },
            );
            if let Ok((clip, radius)) = v.motion.rounded_clip_bounds(&v.root) {
                tracing::trace!(
                    target: "pecofence::frame_shape",
                    title = %v.title,
                    window_w = r.right - r.left, window_h = r.bottom - r.top,
                    client_w = w, client_h = h,
                    inset_x = -origin.x, inset_y = -origin.y,
                    chrome_w = v.chrome_panel.size_px().0,
                    chrome_h = v.chrome_panel.size_px().1,
                    clip_w = clip.x, clip_h = clip.y,
                    radius_x = radius.x, radius_y = radius.y,
                    peeking = v.peeking,
                    "shape frame"
                );
            }
        }
        tracing::trace!(
            target: "pecofence::render_timing",
            w, h,
            crop_us = crop_done.duration_since(started).as_micros(),
            draw_us = draw_done.duration_since(crop_done).as_micros(),
            shadow_us = draw_done.elapsed().as_micros(),
            "resize stages"
        );
    }
    tracing::trace!(w, h, us = started.elapsed().as_micros(), "WM_SIZE frame");
    Some(0)
}

pub(super) fn on_move(
    h: &HandlerCtx,
    _hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    let mut guard = view.borrow_mut();
    if let Some(v) = guard.as_mut() {
        let old_ink = v.material_foreground().text_primary;
        if v.update_backdrop() {
            if old_ink != v.material_foreground().text_primary
                || (v.theme.liquid_glass && v.row_metrics().is_some())
            {
                // Moving clear glass across a light/dark boundary also changes the icon labels.
                let _ = v.redraw();
            } else {
                let _ = v.redraw_chrome_only();
            }
        }
        v.update_shadow();
    }
    Some(0)
}

pub(super) fn on_entersizemove(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view,
        peek_armed,
        in_size_move,
        rect_at_enter,
        move_start,
        grab_offset,
        merge_target,
        merge_at,
        ..
    } = h;
    in_size_move.set(true);
    // The user owns the rect now: an auto-height settle must not fight it.
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
    {
        v.height_anim = None;
    }
    let (r, pt) = (window::window_rect(hwnd), window::cursor_pos());
    let caption = move_start.take();
    let (start_x, start_y) = caption.unwrap_or((pt.x, pt.y));
    // A title drag (not a resize) of a selected fence takes the rest of the selection along.
    *h.group.borrow_mut() = if caption.is_some() {
        group_members(&h.behavior, hwnd)
    } else {
        Vec::new()
    };
    // The push pass leaves the windows of the loop where the user puts them.
    *h.behavior.size_move.borrow_mut() = Some(SizeMove {
        hwnd,
        moving: caption.is_some(),
        group: h.group.borrow().iter().map(|m| m.hwnd).collect(),
    });
    rect_at_enter.set(r);
    grab_offset.set((start_x - r.left, start_y - r.top));
    h.grab_dpi.set(monitors::dpi_for_window(hwnd));
    merge_target.set(0);
    merge_at.set((i32::MIN, i32::MIN));
    window::kill_timer(hwnd, TIMER_PEEK_OPEN);
    window::kill_timer(hwnd, TIMER_PEEK_CLOSE);
    peek_armed.set(false);
    None
}

pub(super) fn on_exitsizemove(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view,
        queue,
        in_size_move,
        rect_at_enter,
        merge_target,
        ..
    } = h;
    let fence_id = h.fence_id;
    in_size_move.set(false);
    // A title drag, not a resize. (Comparing sizes would call a move onto a monitor with
    // another scale a resize: WM_DPICHANGED rescales the window inside the loop.)
    let moved = h
        .behavior
        .size_move
        .replace(None)
        .is_some_and(|m| m.moving && m.hwnd == hwnd);
    hide_drag_feedback(&h.behavior);
    let rect = window::window_rect(hwnd);
    let at_enter = rect_at_enter.get();
    // Esc in the system move loop restores the rect but leaves the cursor
    // where it is (possibly on another fence's title): never merge then.
    let cancelled = rect == at_enter || window::key_down(msg::VK_ESCAPE);
    let group = std::mem::take(&mut *h.group.borrow_mut());
    let merge_into = if moved && !cancelled && group.is_empty() {
        merge_target_under_cursor(hwnd)
    } else {
        None
    };
    if !group.is_empty() && cancelled {
        restore_members(&group);
        if rect == at_enter && !window::key_down(msg::VK_ESCAPE) {
            // A title click without a drag: the selection ends, as a click elsewhere would.
            queue.push(Command::ClearFenceSelection);
        }
    }
    if merge_target.replace(0) != 0 {
        queue.push(Command::MergeHint {
            target: HWND(std::ptr::null_mut()),
            at: (0, 0),
        });
    }
    let peeking = {
        let mut guard = view.borrow_mut();
        match guard.as_mut() {
            Some(v) => {
                // Record the user's height, but never a transient
                // mid-animation one (it would be persisted as expanded_h).
                if !v.rolled_up && !v.height_animating() {
                    v.expanded_h_px = rect.bottom - rect.top;
                }
                v.peeking
            }
            None => false,
        }
    };
    if moved && !cancelled {
        // Moved: where it was dropped is where it rests.
        h.behavior.release_push(hwnd);
    }
    queue_bounds_changed(queue, &h.behavior, fence_id, hwnd);
    if let Some(into) = merge_into {
        let pt = window::cursor_pos();
        queue.push(Command::MergeFence {
            fence: fence_id,
            into,
            at: (pt.x, pt.y),
        });
    } else if moved && !cancelled {
        queue.push(Command::FenceDropped(fence_id));
        commit_members(queue, &h.behavior, &group);
    }
    if peeking {
        // The close timer was cancelled on WM_ENTERSIZEMOVE; its arm
        // re-checks the cursor and re-arms while it is still inside.
        window::set_timer(hwnd, TIMER_PEEK_CLOSE, PEEK_CLOSE_MS);
    }
    Some(0)
}

pub(super) fn on_syscommand(
    h: &HandlerCtx,
    _hwnd: HWND,
    wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    let cmd = (wparam & 0xFFF0) as u32;
    let locked = view
        .try_borrow()
        .ok()
        .and_then(|g| g.as_ref().map(|v| v.locked))
        .unwrap_or(false);
    if locked && (cmd == msg::SC_MOVE || cmd == msg::SC_SIZE) {
        return Some(0);
    }
    if cmd == msg::SC_MOVE || cmd == msg::SC_SIZE {
        h.move_start
            .set((wparam == SC_MOVE_CAPTION).then(|| (msg::lo_i16(lparam), msg::hi_i16(lparam))));
    }
    None
}

pub(super) fn on_sizing(h: &HandlerCtx, hwnd: HWND, wparam: usize, lparam: isize) -> Option<isize> {
    let HandlerCtx { view, behavior, .. } = h;
    // 「调整大小时按整行整列对齐」 (snapping.sizeToCells, Fences' "sized to even multiples of
    // icons"): the width takes whole icon columns and the height whole rows, except where an
    // edge lines up with another fence (the columns then share the rest of the width, the rest
    // of the height stays empty under the whole rows). Off, sizes are free (one column / one
    // row at least).
    let cells = behavior.size_to_cells.get();
    let guard = view.try_borrow().ok()?;
    let v = guard.as_ref()?;
    let scale = v.scale();
    let metrics = v.grid_metrics();
    // Above the rows: the title row, nothing beside a left tab column.
    let title_h = v.head_px();
    let slack = FIT_SLACK_PX as f32;
    // Rows (List / Details): rows below the title (+ fixed header), free width.
    let (cols, rows, min_w) = match v.row_metrics() {
        Some(rm) => (
            None,
            Cells {
                fixed: title_h as f32 + (rm.header_h + rm.pad_y * 2.0) * scale + slack,
                step: (rm.row_h * scale).max(1.0),
            },
            1,
        ),
        None => {
            // A left tab column sits before the first column.
            let cols = Cells {
                fixed: metrics.pad_x * 2.0 * scale + v.content_x as f32,
                step: (metrics.cell_w * scale).max(1.0),
            };
            (
                Some(cols),
                Cells {
                    fixed: title_h as f32 + metrics.pad_y * 2.0 * scale + slack,
                    step: (metrics.cell_h * scale).max(1.0),
                },
                (cols.fixed + cols.step).round() as i32,
            )
        }
    };
    // "按时间分组" interleaves header bands with the rows: no whole-row rhythm to keep.
    let grouped = v.group_by_date && {
        let (cw, _) = v.content_size_px();
        v.layout(cw as f32 / scale).is_grouped()
    };
    let rules = SizeRules {
        cols: cols.filter(|_| cells),
        rows: (cells && !grouped).then_some(rows),
        min_w,
        min_h: (rows.fixed + rows.step).round() as i32,
        rolled_h: v.rolled_up.then(|| v.rolled_h_px()),
    };
    drop(guard);
    // SAFETY: lParam is the RECT* being sized for this message.
    let rect = unsafe { &mut *(lparam as *mut RECT) };
    let others = if behavior.snapping.get() {
        other_fence_rects(hwnd)
    } else {
        Vec::new()
    };
    let work = work_area_under(rect);
    // Snapping on: edges line up with other fences (Alt held: free) and stop at them.
    let near = behavior.snapping.get().then(|| Neighbours {
        others: &others,
        work: work.as_ref(),
        gap: (behavior.snap_gap_dip.get() as f32 * scale).round() as i32,
        dist: (SNAP_DIST_DIP as f32 * scale) as i32,
        align: !snap_paused(),
    });
    let guides = size_among(
        rect,
        &window::window_rect(hwnd),
        SizingEdges::from_wmsz(wparam),
        &rules,
        near.as_ref(),
    );
    show_guides(behavior, hwnd, &guides);
    Some(1)
}

pub(super) fn on_moving(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view,
        queue,
        behavior,
        grab_offset,
        merge_target,
        merge_at,
        rect_at_enter,
        ..
    } = h;
    let group = h.group.borrow().clone();
    let grab_dpi = h.grab_dpi.get();
    // Drag-to-merge: light up the title of the fence under the cursor.
    let merge = if group.is_empty() {
        merge_target_under_cursor(hwnd)
    } else {
        None
    };
    let target = merge.map_or(0, |h| h.0 as isize);
    // SAFETY: lParam is the RECT* of the window being dragged for this
    // message.
    let rect = unsafe { &mut *(lparam as *mut RECT) };
    // Start from where the cursor says the window is (grab offset kept), not
    // from the previous, possibly snapped, position.
    let pt = window::cursor_pos();
    let (w, h) = (rect.right - rect.left, rect.bottom - rect.top);
    let (gx, gy) = grab_offset_for_dpi(grab_offset.get(), grab_dpi, monitors::dpi_for_window(hwnd));
    rect.left = pt.x - gx;
    rect.top = pt.y - gy;
    rect.right = rect.left + w;
    rect.bottom = rect.top + h;
    if !group.is_empty() {
        let scale = monitors::dpi_for_window(hwnd).max(96) as f32 / 96.0;
        let snap = (behavior.snapping.get() && !snap_paused()).then(|| {
            (
                (behavior.snap_gap_dip.get() as f32 * scale) as i32,
                (SNAP_DIST_DIP as f32 * scale) as i32,
            )
        });
        let others = fence_rects_outside(hwnd, &group);
        let (placed, guides) = place_group(
            rect,
            &rect_at_enter.get(),
            &group,
            &others,
            work_area_under,
            snap,
        );
        move_members(&group, &placed);
        show_guides(behavior, hwnd, &guides);
        return Some(1);
    }
    // Snapping and merging fight: the 8 px snap would hold the dragged fence
    // at the target's outer edge while the cursor is already on its title.
    // Follow the cursor freely over a target, and while Alt is held.
    let mut guides = Vec::new();
    if behavior.snapping.get() && target == 0 && !snap_paused() {
        // Snap to other fences and the work area (snapping.gapPx gap, 10 px capture).
        let scale = monitors::dpi_for_window(hwnd).max(96) as f32 / 96.0;
        guides = snap_rect(
            rect,
            hwnd,
            (behavior.snap_gap_dip.get() as f32 * scale) as i32,
            (SNAP_DIST_DIP as f32 * scale) as i32,
        );
    }
    let radius = view
        .try_borrow()
        .ok()
        .and_then(|g| g.as_ref().map(|v| v.theme.corner_radius))
        .unwrap_or(8.0);
    update_drag_feedback(behavior, hwnd, rect, merge, &guides, radius);
    // Re-sent while the pointer moves along a target's strip so the
    // insertion gap follows it (cheap: the target ignores an unchanged slot).
    let target_changed = merge_target.replace(target) != target;
    if target_changed || (target != 0 && merge_at.replace((pt.x, pt.y)) != (pt.x, pt.y)) {
        merge_at.set((pt.x, pt.y));
        queue.push(Command::MergeHint {
            target: HWND(target as *mut core::ffi::c_void),
            at: (pt.x, pt.y),
        });
    }
    Some(1)
}

pub(super) fn on_dpichanged(
    h: &HandlerCtx,
    hwnd: HWND,
    wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    let new_dpi = msg::lo_u16(wparam);
    // SAFETY: WM_DPICHANGED's lParam points to a RECT owned by the system for the
    // duration of the message.
    let suggested = unsafe { *(lparam as *const RECT) };
    {
        let mut guard = view.borrow_mut();
        if let Some(v) = guard.as_mut() {
            v.apply_dpi(new_dpi as u32);
        }
    }
    let _ = window::set_window_bounds(
        hwnd,
        suggested.left,
        suggested.top,
        suggested.right - suggested.left,
        suggested.bottom - suggested.top,
    );
    // Windows need not send WM_SIZE when the recommended physical size is unchanged.
    // DPI still changes the optical bezel, root clip and content layout in that case.
    if let Some(v) = view.borrow_mut().as_mut() {
        v.update_backdrop();
        let (w, h) = window::window_rect_size(hwnd);
        if let Err(error) = v.layout_panels(w, h).and_then(|_| v.redraw()) {
            tracing::error!(%error, "redraw after DPI change failed");
        }
        v.update_shadow();
    }
    Some(0)
}
