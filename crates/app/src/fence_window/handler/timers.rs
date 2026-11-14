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
        super::misc::on_app_fade_done(h, hwnd, generation, super::misc::FADE_DONE_BY_TIMER)
    } else {
        super::misc::on_app_tab_swap_done(h, hwnd, generation, 0)
    }
}

/// `TIMER_ACTIVE_SYNC` (`last` false): is the file operation of our own over? Then
/// `TIMER_ACTIVE_SYNC_END` (`last` true) fires once, half a second later, and the selection
/// colour follows the real activation again: the fence itself, its name box or a menu in
/// front means active.
pub(super) fn on_timer_active_sync(h: &HandlerCtx, hwnd: HWND, last: bool) -> Option<isize> {
    if !last {
        if !crate::fence_window::own_file_op_running() {
            window::kill_timer(hwnd, TIMER_ACTIVE_SYNC);
            window::set_timer(hwnd, TIMER_ACTIVE_SYNC_END, ACTIVE_SYNC_MS);
            return Some(0);
        }
        // The operation still runs (a large copy, a question nobody answered) and the user
        // went on to another program: the selection is not the active one for all that time.
        // The shell's own progress and question windows belong to this process.
        let front = desktop::root_ancestor(desktop::foreground_window());
        let elsewhere = !front.0.is_null()
            && desktop::window_pid(front) != pecofence_platform::process::current_pid();
        if elsewhere {
            window::kill_timer(hwnd, TIMER_ACTIVE_SYNC);
            if let Ok(mut guard) = h.view.try_borrow_mut()
                && let Some(v) = guard.as_mut()
            {
                tracing::debug!(
                    "file operation still running, another program in front: the selection colour follows"
                );
                if v.set_active(false) {
                    let _ = v.redraw_content();
                }
            }
        }
        return Some(0);
    }
    window::kill_timer(hwnd, TIMER_ACTIVE_SYNC_END);
    let front = desktop::root_ancestor(desktop::foreground_window());
    let cls = desktop::class_name(front);
    let active = front == hwnd || cls == "PecoFence.Rename" || cls == "#32768";
    if let Ok(mut guard) = h.view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
    {
        tracing::debug!(
            active,
            "file operation over: the selection colour follows the activation again"
        );
        if v.set_active(active) {
            let _ = v.redraw_content();
        }
    }
    Some(0)
}

/// A fence just made is at full strength (the 167 ms of its fade-in are over): its name box
/// opens now, unless a hide has begun meanwhile (the fade's end then drops the request). A
/// view that is borrowed right now leaves the request to the fade's end, as before.
pub(super) fn on_timer_name_box(h: &HandlerCtx, hwnd: HWND) -> Option<isize> {
    window::kill_timer(hwnd, TIMER_NAME_BOX);
    if let Ok(mut guard) = h.view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
        && !matches!(v.vis_fade, Some(VisFade::Hiding { .. }))
        && let Some(asked) = v.rename_after_fade.take()
    {
        tracing::debug!(
            title = %v.title,
            waited_ms = asked.elapsed().as_millis() as u64,
            fade = ?v.vis_fade,
            "new fence at full strength: name box"
        );
        v.queue.push(Command::BeginFenceRename(h.fence_id));
    }
    Some(0)
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
    // pauses as soon as the thumb is going to sit under the pointer (Win32
    // pauses while the hit test differs, the pointer sliding off the bar
    // included, and resumes). An arrow keeps stepping one row while the
    // pointer stays on it. Repeat steps use the 83 ms cap so the offset
    // reaches each page / row before the next 50 ms tick (Win32 pages at
    // once); a 333 ms glide retargeted every tick would trail the thumb by
    // pages and keep moving after the button is released.
    let pt = window::screen_to_client(hwnd, window::cursor_pos());
    let target = v.scroll_target();
    let row_step = v.scroll_row_step(&v.scroll_extent().0);
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
        // Another part or off the bar: paused, it resumes once the pointer is back on `side`
        // (the thumb that caught up with the pointer, or the pointer dragged further along).
        _ => window::set_timer(hwnd, TIMER_SCROLL_REPEAT, SCROLL_PAGE_REPEAT_MS),
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
            // The rolled fences the pointer comes to next follow after the short wait.
            h.behavior.rolled_fence_hover_expanded();
            // Above the other fences while it is open: a neighbour the push cannot clear (it
            // stops at the bottom of the screen) must not cover it (#197).
            v.hover_raised = true;
            h.queue.push(Command::RaiseForHover(hwnd));
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
            v.hover_raised = true;
            h.queue.push(Command::RaiseForHover(hwnd));
        }
    }
    Some(0)
}

/// The pointer rested on another tab for the hover delay: switch to it the way a wheel notch
/// does, unless it moved off, a press or drag began, or the setting went off meanwhile.
pub(super) fn on_timer_tab_hover(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view,
        fence_id,
        queue,
        behavior,
        in_size_move,
        ..
    } = h;
    window::kill_timer(hwnd, TIMER_TAB_HOVER);
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
        && let Some(i) = v.tab_hover_switch.take()
    {
        let target = hover_switch_target(
            v.tab_hover.filter(|&i| v.tab_fully_shown(i)),
            v.active_index(),
            v.tabs.len(),
            behavior.switch_tab_on_hover.get(),
            v.title_renaming,
        );
        if !v.retired
            && target == Some(i)
            && v.pressed.is_none()
            && v.tab_drag.is_none()
            && v.roll_anim.is_none()
            && !in_size_move.get()
            && !window::key_down(msg::VK_LBUTTON)
        {
            // A switch that follows another hover switch shows its content at once: the swap's
            // fade and slide would run into the next tab's during a sweep along the strip.
            v.tab_swap_at_once = v.tab_hover_quick;
            v.tab_hover_quick = true;
            v.tab_strip_left = None;
            tracing::debug!(tab = i, at_once = v.tab_swap_at_once, "tab hover switch");
            queue.push(Command::SwitchTab {
                host: *fence_id,
                tab: v.tabs[i].id,
            });
        }
    }
    Some(0)
}

// Explorer's single-click open: the pointer rested on an item for the hover time, so it is
// selected as on the desktop (Ctrl toggles it, Shift selects the range from the anchor). The
// fences and Explorer's desktop are one desktop to the user: the fence under the pointer
// selects while it, another fence or the desktop has the keyboard, and takes the keyboard as a
// press on the item would, so F2 renames what the pointer selected (#184). Resting on the
// title row takes the keyboard the same way, for F2 there (the fence's name box). Another
// program in front keeps the keyboard, as it does over the desktop. Not while a button is
// held, a gesture runs, a rename is open or a note is edited.
pub(super) fn on_timer_hover_select(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view,
        behavior,
        queue,
        ..
    } = h;
    window::kill_timer(hwnd, TIMER_HOVER_SELECT);
    // Some: this fence takes the keyboard from another fence or the desktop; true when an
    // item was selected for it (the selection in the other fences then ends, as on a press).
    let mut take = None;
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
        && behavior.single_click.get()
        && !v.retired
        && v.pressed.is_none()
        && v.drag.is_none()
        && v.marquee.is_none()
        && !v.ole_drag
        && !window::key_down(msg::VK_LBUTTON)
        && crate::rename::active_target().is_none()
    {
        // During a Peek the dimmer has the keyboard and hands the keys to the fence last used
        // (`focus_session`): no fence is the active window then, and each of them selects.
        let elsewhere = !v.win_active
            && !crate::note_edit::is_open()
            && (behavior.floating.get() || keyboard_on_desktop());
        if let Some(i) = v.hover
            && !v.rolled_up
            && (v.win_active || elsewhere)
        {
            let changed = hover_item(
                &mut v.selected,
                &mut v.range_anchor,
                i,
                window::key_down(msg::VK_SHIFT),
                window::key_down(msg::VK_CONTROL),
            );
            v.anchor_index = Some(i);
            let woke = elsewhere && v.activate_by_pointer();
            if changed || woke {
                let _ = v.redraw_content();
            }
            take = elsewhere.then_some(true);
        } else if v.hover.is_none() && v.title_hover && elsewhere {
            take = Some(false);
        }
    }
    // No borrow held: taking the foreground re-enters the window procedure.
    if let Some(item) = take {
        if item {
            queue.push(Command::EndItemSelection { except: hwnd });
        }
        // From Explorer's desktop a plain request is refused (a timer is no input of ours).
        // During a Peek the foreground stays with the dimmer (taking it would end the Peek).
        let peek = behavior.floating.get();
        let got = !peek && window::claim_foreground(hwnd);
        tracing::debug!(
            fence = %h.fence_id,
            item,
            peek,
            got,
            "hover select: the fence under the pointer takes the keyboard"
        );
        focus_session(view, hwnd);
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
    match view.try_borrow_mut() {
        // The view is busy: the timer stays and comes again. (`update_shadow` arms it once per
        // wait and counts on it to fire, so it must not be dropped here.)
        Err(_) => {}
        Ok(mut guard) => match guard.as_mut() {
            // Ends the timer.
            Some(v) => v.update_shadow_now(),
            None => window::kill_timer(hwnd, TIMER_SHADOW),
        },
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
                // A note that is being edited keeps its fence open too: the editor draws
                // nothing itself, so a fence that rolled up left the typing unseen.
                .is_some_and(|v| v.renaming.is_some() || v.title_renaming || v.note_edit.is_some())
        });
    // A Peek search holds it open; ending the search restarts this countdown.
    if view
        .try_borrow()
        .ok()
        .is_some_and(|g| g.as_ref().is_some_and(|v| v.search_expanded))
    {
        return Some(0);
    }
    let pt = window::cursor_pos();
    let r = window::window_rect(hwnd);
    // A fence this one pushed out of its way counts as inside: moving onto it must not make it
    // slide back from under the pointer.
    let inside = (pt.x >= r.left && pt.x < r.right && pt.y >= r.top && pt.y < r.bottom)
        || h.behavior.holds_open(hwnd, pt);
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

/// 时钟与日期: the minute (or second) turned; draw the new time and wait for the next one.
/// `TIMER_FIT_AHEAD`: see `FenceViewState::fit_ahead_tick`. A view that is busy right now
/// (a popup re-entered us) is asked again after the rest time.
pub(super) fn on_timer_fit_ahead(h: &HandlerCtx, hwnd: HWND) -> Option<isize> {
    match h.view.try_borrow_mut() {
        Ok(mut g) => {
            if let Some(v) = g.as_mut() {
                v.fit_ahead_tick();
            } else {
                window::kill_timer(hwnd, TIMER_FIT_AHEAD);
            }
        }
        Err(_) => window::set_timer(hwnd, TIMER_FIT_AHEAD, FIT_AHEAD_REST_MS),
    }
    Some(0)
}

pub(super) fn on_timer_clock(h: &HandlerCtx) -> Option<isize> {
    if let Ok(mut g) = h.view.try_borrow_mut()
        && let Some(v) = g.as_mut()
    {
        if !v.rolled_up && !v.retired && pecofence_platform::desktop::is_visible(v.hwnd) {
            // A tick is nobody using PecoFence: it does not hold back the memory housekeeping.
            let _ = pecofence_render::stack::quiet_draws(|| v.redraw_content());
        }
        v.arm_clock();
    }
    Some(0)
}
