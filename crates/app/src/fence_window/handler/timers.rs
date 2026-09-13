//! WM_TIMER arms (one per TIMER_* id) and the menu-loop bracket that gates the peek timers.

use super::*;

pub(super) fn on_timer_composition_finish(
    h: &HandlerCtx,
    hwnd: HWND,
    timer: usize,
    _lparam: isize,
) -> Option<isize> {
    let Ok(guard) = h.view.try_borrow() else {
        // An owned popup can re-enter us while the view is borrowed.
        window::set_timer(hwnd, timer, 16);
        return Some(0);
    };
    let Some(v) = guard.as_ref() else {
        return Some(0);
    };
    let (deadline, generation) = if timer == TIMER_VISIBILITY_FINISH {
        (v.vis_deadline, v.vis_gen)
    } else {
        (v.swap_deadline, v.swap_gen)
    };
    let Some(deadline) = deadline else {
        window::kill_timer(hwnd, timer);
        return Some(0);
    };
    let remaining = deadline.saturating_duration_since(Instant::now());
    if !remaining.is_zero() {
        // A timer message queued before a reversal must not finish the new transition.
        window::set_timer(hwnd, timer, remaining.as_millis() as u32 + 1);
        return Some(0);
    }
    drop(guard);
    if timer == TIMER_VISIBILITY_FINISH {
        super::misc::on_app_fade_done(h, hwnd, generation, 0)
    } else {
        super::misc::on_app_tab_swap_done(h, hwnd, generation, 0)
    }
}

pub(super) fn on_timer_rename(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, queue, .. } = h;
    window::kill_timer(hwnd, TIMER_RENAME);
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
        && let Some(id) = v.pending_rename.take()
        && let Some(i) = v.items.iter().position(|it| it.id == id)
        && v.selected.len() == 1
        && v.selected.contains(&i)
        && !window::key_down(msg::VK_LBUTTON)
    {
        queue.push(Command::RenameItem {
            fence: v.active,
            item: id,
        });
    }
    Some(0)
}

pub(super) fn on_timer_tip(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    window::kill_timer(hwnd, TIMER_TIP);
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
    {
        // Fired (or suppressed by a gesture): no longer armed either way.
        v.tip_armed = false;
        if v.tip_target.is_some()
            && v.drag.is_none()
            && v.marquee.is_none()
            && v.tab_drag.is_none()
            && v.scroll_drag.is_none()
            && !v.ole_drag
        {
            v.show_tip();
        }
    }
    Some(0)
}

pub(super) fn on_timer_tip_hide(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    window::kill_timer(hwnd, TIMER_TIP_HIDE);
    // try_borrow_mut: TTM_TRACKACTIVATE sends TTN_POP synchronously.
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
    {
        v.autopop_tip();
    }
    Some(0)
}

pub(super) fn on_timer_scroll_repeat(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    let Ok(mut guard) = view.try_borrow_mut() else {
        return Some(0);
    };
    let Some(v) = guard.as_mut() else {
        return Some(0);
    };
    let Some(side) = v.page_repeat else {
        window::kill_timer(hwnd, TIMER_SCROLL_REPEAT);
        return Some(0);
    };
    // Hit-test against the thumb at the glide's destination, so paging
    // stops as soon as the thumb is going to sit under the pointer (Win32
    // stops when the hit test changes; the pointer sliding off the bar
    // pauses and resumes). An arrow keeps stepping one row while the
    // pointer stays on it. Repeat steps use the 83 ms cap so the offset
    // reaches each page / row before the next 50 ms tick (Win32 pages at
    // once); a 333 ms glide retargeted every tick would trail the thumb by
    // pages and keep moving after the button is released.
    let pt = window::screen_to_client(hwnd, window::cursor_pos());
    let target = v.scroll_target();
    let row_step = v.scroll_extent().0.row_step();
    match v.scrollbar_hit_for(pt.x, pt.y, target) {
        Some((hit, g)) if hit == side => {
            let step = scroll_step_for(hit, g.view_h, row_step);
            let at_end = (step < 0.0 && target <= 0.0) || (step > 0.0 && target >= g.max_scroll);
            if at_end {
                window::kill_timer(hwnd, TIMER_SCROLL_REPEAT);
            } else {
                v.scroll_to(target + step, motion::FASTER);
                if v.scroll_anim.is_none() {
                    let _ = v.redraw_content();
                }
                window::set_timer(hwnd, TIMER_SCROLL_REPEAT, SCROLL_PAGE_REPEAT_MS);
            }
        }
        Some(_) => window::kill_timer(hwnd, TIMER_SCROLL_REPEAT),
        None => window::set_timer(hwnd, TIMER_SCROLL_REPEAT, SCROLL_PAGE_REPEAT_MS),
    }
    Some(0)
}

pub(super) fn on_timer_scrollbar_expand(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    window::kill_timer(hwnd, TIMER_SCROLLBAR_EXPAND);
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
        && v.scrollbar_hot
        && v.scrollbar_geometry().is_some()
    {
        v.scrollbar_width_to(ScrollbarDraw::EXPANDED_WIDTH);
    }
    Some(0)
}

pub(super) fn on_timer_scrollbar_contract(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    window::kill_timer(hwnd, TIMER_SCROLLBAR_CONTRACT);
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
        && !v.scrollbar_hot
        && !v.scrollbar_pressed()
    {
        v.scrollbar_width_to(ScrollbarDraw::REST_WIDTH);
    }
    Some(0)
}

pub(super) fn on_timer_peek_open(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view,
        peek_armed,
        in_size_move,
        ..
    } = h;
    window::kill_timer(hwnd, TIMER_PEEK_OPEN);
    peek_armed.set(false);
    if in_size_move.get() || window::key_down(msg::VK_LBUTTON) {
        // Modal move/size loop (or a press about to start one) owns the
        // window rect: do not fight it with a peek animation.
        return Some(0);
    }
    {
        let mut guard = view.borrow_mut();
        if let Some(v) = guard.as_mut()
            && !v.retired
            && v.rolled_up
            && !v.peeking
            && v.roll_anim.is_none()
            && v.title_hover
        {
            v.peeking = true;
            if v.expanded_h_px <= v.title_h_px() {
                v.expanded_h_px = v.title_h_px() + (200.0 * v.scale()) as i32;
            }
            v.start_roll_anim(false);
        }
    }
    Some(0)
}

// A drag rested on a rolled fence for the hover delay: unroll it (peek) so
// the drop can target its folders / insertion slots. No button / title
// hover guards here — OLE owns the pointer, which is exactly why
// TIMER_PEEK_OPEN cannot be reused.
pub(super) fn on_timer_drag_peek(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view, in_size_move, ..
    } = h;
    window::kill_timer(hwnd, TIMER_DRAG_PEEK);
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
    {
        v.drag_peek_armed = false;
        if !v.retired
            && v.rolled_up
            && !v.peeking
            && v.roll_anim.is_none()
            && (v.drop_hover || v.drop_tab.is_some())
            && !in_size_move.get()
        {
            v.peeking = true;
            if v.expanded_h_px <= v.title_h_px() {
                v.expanded_h_px = v.title_h_px() + (200.0 * v.scale()) as i32;
            }
            v.start_roll_anim(false);
        }
    }
    Some(0)
}

pub(super) fn on_timer_scrollbar(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    window::kill_timer(hwnd, TIMER_SCROLLBAR);
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
    {
        // Linger over: fade the indicator out (83 ms) unless something
        // still wants it shown.
        v.sync_scrollbar_alpha();
    }
    Some(0)
}

pub(super) fn on_timer_shadow(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    window::kill_timer(hwnd, TIMER_SHADOW);
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
    {
        v.update_shadow_now();
    }
    Some(0)
}

pub(super) fn on_entermenuloop(
    h: &HandlerCtx,
    _hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { menu_open, .. } = h;
    menu_open.set(true);
    None
}

pub(super) fn on_exitmenuloop(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view, menu_open, ..
    } = h;
    menu_open.set(false);
    // The popup was part of the peeked fence: its close countdown starts
    // now, not while the menu was up (WM_MOUSEMOVE inside still cancels).
    let peeking = view
        .try_borrow()
        .ok()
        .is_some_and(|g| g.as_ref().is_some_and(|v| v.peeking));
    if peeking {
        window::kill_timer(hwnd, TIMER_PEEK_CLOSE);
        window::set_timer(hwnd, TIMER_PEEK_CLOSE, PEEK_CLOSE_MS);
    }
    None
}

pub(super) fn on_timer_peek_close(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view,
        menu_open,
        in_size_move,
        ..
    } = h;
    window::kill_timer(hwnd, TIMER_PEEK_CLOSE);
    if in_size_move.get() || window::key_down(msg::VK_LBUTTON) {
        // Never roll up under the user's hands (resize/drag in progress).
        window::set_timer(hwnd, TIMER_PEEK_CLOSE, PEEK_CLOSE_MS);
        return Some(0);
    }
    // A spring-loaded container never closes under its own popup: the
    // context menu and the rename edit count as part of the fence
    // (WinUI keeps the expanded state while a flyout is open).
    let popup = menu_open.get()
        || view.try_borrow().ok().is_some_and(|g| {
            g.as_ref()
                .is_some_and(|v| v.renaming.is_some() || v.title_renaming)
        });
    let pt = window::cursor_pos();
    let r = window::window_rect(hwnd);
    let inside = pt.x >= r.left && pt.x < r.right && pt.y >= r.top && pt.y < r.bottom;
    if inside || popup {
        window::set_timer(hwnd, TIMER_PEEK_CLOSE, PEEK_CLOSE_MS);
        return Some(0);
    }
    {
        let mut guard = view.borrow_mut();
        if let Some(v) = guard.as_mut()
            && !v.retired
            && v.peeking
            && v.roll_anim.is_none()
            && v.drag.is_none()
            && !v.ole_drag
            && v.marquee.is_none()
        {
            v.peeking = false;
            v.start_roll_anim(true);
        }
    }
    Some(0)
}
