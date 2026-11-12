//! Client-area mouse buttons, wheel, cursor and leave messages.

use super::*;

// The cursor is decided here, before each mouse message (Explorer's header
// shows IDC_SIZEWE the moment the pointer is over a divider and keeps it):
// DefWindowProc would otherwise reset the class arrow first and the move
// handler re-apply the resize cursor a message later (a one-frame flicker).
pub(super) fn on_setcursor(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    if msg::lo_i16(lparam) as isize != msg::HTCLIENT {
        return None;
    }
    // try_borrow: this arrives re-entrantly from nested loops too.
    let guard = view.try_borrow().ok()?;
    let v = guard.as_ref()?;
    let pt = window::screen_to_client(hwnd, window::cursor_pos());
    if v.col_drag.is_some() || v.divider_hit(pt.x, pt.y).is_some() {
        drop(guard);
        window::set_standard_cursor(StandardCursor::SizeWE);
        return Some(1);
    }
    // The text of a note that is being edited: the text cursor.
    if v.note_edit.is_some() && v.in_note_text(pt.x, pt.y) {
        drop(guard);
        window::set_standard_cursor(StandardCursor::IBeam);
        return Some(1);
    }
    // A script fence's clickable row, the empty state's action row: the hand, as over a link.
    if v.script_row_at(pt.x, pt.y).is_some() || v.empty_action_at(pt.x, pt.y) {
        drop(guard);
        window::set_standard_cursor(StandardCursor::Hand);
        return Some(1);
    }
    // Explorer's single-click open: the hand over an item, as on the desktop.
    if h.behavior.single_click.get()
        && v.scrollbar_hit(pt.x, pt.y).is_none()
        && v.hit_item(pt.x, pt.y).is_some()
    {
        drop(guard);
        window::set_standard_cursor(StandardCursor::Hand);
        return Some(1);
    }
    None
}

pub(super) fn on_mouseleave(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view, peek_armed, ..
    } = h;
    // Leaving the client area into our own title bar is not leaving the
    // window; leaving onto a menu / an overlapping fence is, even while the
    // cursor is still inside our RECT (see WM_NCMOUSELEAVE).
    let pt = window::cursor_pos();
    let inside = desktop::root_ancestor(desktop::window_from_point(pt.x, pt.y)) == hwnd;
    if !inside {
        // A peek armed over the chevron / a tab must not open once the
        // pointer has left the fence.
        window::kill_timer(hwnd, TIMER_PEEK_OPEN);
        peek_armed.set(false);
    }
    let mut guard = view.borrow_mut();
    if let Some(v) = guard.as_mut() {
        if !inside {
            v.set_mouse_inside(false);
            v.lift_peek_block("the pointer left the fence");
        }
        // A tip armed over a spot of the caption (the Task View glyph, a filter tag, the lock
        // glyph) is WM_NCMOUSEMOVE's, which can come before this message when the pointer goes
        // from the content straight onto it: it stays while the pointer is still in the window.
        if !inside
            || !matches!(
                v.tip_target,
                Some(TipTarget::Desktops | TipTarget::FilterTag | TipTarget::Lock)
            )
        {
            v.hide_tip();
        }
        let chrome_changed = std::mem::take(&mut v.up_hovered)
            | std::mem::take(&mut v.chevron_hovered)
            | v.tab_hover.take().is_some();
        // Leaving the bar starts the 0.5 s contract delay, not an
        // instant collapse.
        v.set_scrollbar_pointer(None);
        let content_changed = v.header_hover.take().is_some()
            | v.script_hover.take().is_some()
            | v.note_box_hover.take().is_some()
            | v.hover_empty_action(None);
        // Onto the caption part of the title row (a gap between tabs): still the row, as in
        // WM_NCMOUSELEAVE; this message can come after the WM_NCMOUSEMOVE there.
        let c = window::screen_to_client(hwnd, pt);
        let on_row = inside && v.title_row_at(c.x, c.y);
        let (item_changed, title_changed) = v.set_hover(None, on_row && v.title_hover);
        if chrome_changed || title_changed {
            let _ = v.redraw_chrome_only();
        }
        if (content_changed || item_changed) && !v.rolled_up {
            let _ = v.redraw_content();
        }
        if v.peeking {
            window::set_timer(hwnd, TIMER_PEEK_CLOSE, PEEK_CLOSE_MS);
        }
    }
    Some(0)
}

pub(super) fn on_lbuttondown(
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
    let x = msg::lo_i16(lparam);
    let y = msg::hi_i16(lparam);
    let ctrl = wparam & msg::MK_CONTROL as usize != 0;
    let shift = wparam & msg::MK_SHIFT as usize != 0;
    // Decide and mutate under the borrow; every call that can re-enter the
    // window procedure (SetForegroundWindow, SetCapture) happens after it.
    let (focus, capture) = {
        let mut guard = view.borrow_mut();
        let Some(v) = guard.as_mut() else {
            return Some(0);
        };
        // Any new press cancels an armed slow-double-click rename. A plain press is no
        // double-click: a click that expanded the fence no longer swallows the next one.
        v.cancel_pending_rename();
        v.suppress_dblclk = false;
        v.hide_tip();
        // A press where the fence would resize without its lock (an edge band inside the
        // plate). It is remembered for the lock glyph's pulse only further down, once nothing
        // else has taken the press: the scrollbar, a tab, a header, a tool's row and an icon
        // all return or go on before that.
        v.lock_press = None;
        let on_locked_edge = v.locked && {
            let pt = window::client_to_screen(hwnd, pecofence_platform::POINT { x, y });
            super::nc::lock_refuses(v, &window::window_rect(hwnd), pt.x, pt.y)
        };
        let lock_press = |leave_only: bool| {
            let pt = window::client_to_screen(hwnd, pecofence_platform::POINT { x, y });
            on_locked_edge.then_some(LockPress {
                at: (pt.x, pt.y),
                leave_only,
            })
        };
        // A press decides the selection itself: a pending hover select must not follow it.
        window::kill_timer(hwnd, TIMER_HOVER_SELECT);
        // Mouse interaction hides the keyboard focus ring (Explorer). The
        // ring lives on the content surface: arms that do not repaint it
        // anyway do so once below (one raster per click).
        let focus_cleared = std::mem::take(&mut v.focus_visible) && !v.rolled_up;
        // 筛选: the × of the filter tag clears the filter.
        if v.filter_tag_clear_at(x, y) {
            let fence = v.active;
            drop(guard);
            tracing::info!(%fence, "filter tag: × clears the filter");
            queue.push(Command::ClearFenceFilter(fence));
            return Some(0);
        }
        // Up button / chevron: pressed look now, action on release inside. In a title-row tab
        // strip the up button is the shown tab's folder glyph, pressed as part of its tab below.
        let up = v.up_button_at(x, y);
        let tab_up = up && v.title_row_strip();
        if (up && !tab_up) || v.chevron_at(x, y) {
            // A press on the chevron is about to toggle the roll-up: a
            // pending hover-peek must not fire under it.
            window::kill_timer(hwnd, TIMER_PEEK_OPEN);
            peek_armed.set(false);
            v.press_button_at(x, y);
            if focus_cleared {
                let _ = v.redraw_content();
            }
            drop(guard);
            window::set_capture(hwnd);
            queue.push(Command::RaiseFence(hwnd));
            return Some(0);
        }
        if let Some(t) = v.tab_at(x, y) {
            if let Some(tab) = v.tabs.get(t)
                && tab.id != v.active
            {
                queue.push(Command::SwitchTab {
                    host: fence_id,
                    tab: tab.id,
                });
            }
            let pointer_x = v.tab_axis(x, y);
            let slot_x = v
                .tab_draw_xs(Instant::now())
                .get(t)
                .copied()
                .unwrap_or(pointer_x);
            v.tab_drag = Some(TabDrag {
                start: (x, y),
                moved: false,
                index: t,
                start_index: t,
                grab_dx: pointer_x - slot_x,
                pointer_x,
            });
            v.tab_settle = None;
            // On the up button the press is the button's (up on a release); a drag from it
            // still moves the tab (`on_mousemove`).
            v.pressed = Some(if tab_up {
                PressTarget::Up
            } else {
                PressTarget::Tab(t)
            });
            v.press_inside = true;
            // The click grants activation below; draw the accent now.
            if v.activate_by_pointer() || focus_cleared {
                let _ = v.redraw_content();
            }
            let _ = v.redraw_chrome_only();
            drop(guard);
            // The tab strip is client area: like a content click it grants
            // the focus session (plan §5.12) so Ctrl+A / Delete / F2 work on
            // the tab just switched to. Focus, then capture, then raise.
            focus_session(view, hwnd);
            window::set_capture(hwnd);
            queue.push(Command::RaiseFence(hwnd));
            return Some(0);
        }
        let in_content = !v.rolled_up && y >= v.content_y && x >= v.content_x;
        if let Some(col) = v.divider_hit(x, y) {
            let start_w = v.column_width(col);
            let (cw, _) = v.content_size_px();
            let max_w = v.columns_for(cw as f32 / v.scale()).drag_limit(start_w);
            tracing::debug!(?col, start_w, max_w, "column divider drag starts");
            v.col_drag = Some(ColDrag {
                col,
                start_x_px: x,
                start_w,
                max_w,
            });
            v.header_hover = None;
            if focus_cleared {
                let _ = v.redraw_content();
            }
            drop(guard);
            // Like a content click, the press grants the focus session: Esc cancelling the
            // drag has to reach this window.
            focus_session(view, hwnd);
            window::set_capture(hwnd);
            window::set_standard_cursor(StandardCursor::SizeWE);
            return Some(0);
        }
        if v.header_hit(x, y).is_some() {
            // Header cell: pressed fill now (that repaint also drops the
            // focus ring), sort on release inside it.
            v.press_button_at(x, y);
            drop(guard);
            window::set_capture(hwnd);
            return Some(0);
        }
        // Overlay scrollbar: drag the thumb, or page on the track.
        if let Some(repeat) = v.press_scrollbar(x, y) {
            drop(guard);
            window::set_capture(hwnd);
            if repeat {
                window::set_timer(hwnd, TIMER_SCROLL_REPEAT, SCROLL_PAGE_FIRST_MS);
            }
            queue.push(Command::RaiseFence(hwnd));
            return Some(0);
        }
        // A note's check box, a script fence's clickable row: pressed look now, the action on
        // a release over it.
        if v.press_tool_at(x, y, false).is_some() {
            drop(guard);
            window::set_capture(hwnd);
            queue.push(Command::RaiseFence(hwnd));
            return Some(0);
        }
        if let Some(cmd) = v.note_press(x, y).or_else(|| v.empty_action_press(x, y)) {
            drop(guard);
            queue.push(cmd);
            queue.push(Command::RaiseFence(hwnd));
            return Some(0);
        }
        // Other tools (a clock) take no clicks in their content: no marquee, no selection.
        if v.shows_tool() && in_content {
            // Nothing else takes the press: on the edge band a drag is one the lock refuses.
            v.lock_press = lock_press(false);
            drop(guard);
            queue.push(Command::RaiseFence(hwnd));
            return Some(0);
        }
        let hit = v.hit_item(x, y);
        let mut capture = false;
        match hit {
            Some(i) => {
                // Explorer: a plain press keeps a multi-selection for
                // dragging and collapses it on a release without a drag; a
                // press on the sole selected item may become a slow
                // double-click rename.
                let press = press_item(&mut v.selected, &mut v.range_anchor, i, shift, ctrl);
                v.anchor_index = Some(i);
                v.drag = Some(DragState {
                    start: (x, y),
                    item: i,
                    collapse_on_up: press.collapse_on_up,
                    was_sole_selected: press.was_sole_selected,
                    deselect_on_up: press.deselect_on_up,
                    open_on_up: h.behavior.single_click.get() && !ctrl && !shift,
                });
                v.pressed = Some(PressTarget::Item(i));
                v.press_inside = true;
                capture = true;
            }
            None => {
                if !ctrl && !shift {
                    v.selected.clear();
                }
                if in_content {
                    // Ctrl toggles, Shift adds (Ctrl wins when both are held).
                    let mode = if ctrl {
                        MarqueeMode::Toggle
                    } else if shift {
                        MarqueeMode::Add
                    } else {
                        MarqueeMode::Replace
                    };
                    v.marquee = Some(MarqueeState {
                        start: (x, y),
                        start_scroll: v.scroll_y,
                        cur: (x, y),
                        pointer: (x, y),
                        auto_last: None,
                        base: if ctrl || shift {
                            v.selected.clone()
                        } else {
                            HashSet::new()
                        },
                        mode,
                    });
                    capture = true;
                    // On the edge band of a locked fence the rectangle begins all the same; a
                    // drag out of the fence from there is a resize the lock refuses.
                    v.lock_press = lock_press(true);
                    if v.lock_press.is_some() {
                        tracing::debug!(
                            fence = %fence_id,
                            "press on a locked fence's edge over empty content: a rectangle \
                             begins; the lock glyph pulses only if the drag leaves the fence"
                        );
                    }
                }
            }
        }
        if in_content {
            // The click grants activation below; draw the accent now (the
            // redraw that follows shows it, focus_session reconciles).
            let _ = v.activate_by_pointer();
        }
        if !v.rolled_up {
            let _ = v.redraw_content();
        }
        (in_content, capture)
    };
    // Focus session (plan §5.12): a click inside the content grants us the
    // foreground, so F2 / Enter / Delete / Esc reach this window. The z-order
    // guard in WM_WINDOWPOSCHANGING keeps the fence pinned above the icon host.
    if focus {
        focus_session(view, hwnd);
    }
    if capture {
        window::set_capture(hwnd);
    }
    queue.push(Command::RaiseFence(hwnd));
    Some(0)
}

pub(super) fn on_lbuttonup(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    lparam: isize,
) -> Option<isize> {
    // The release of a click-to-expand title press (it captured the mouse).
    if super::nc::release_click_pending(h, hwnd) {
        return Some(0);
    }
    let HandlerCtx { view, queue, .. } = h;
    let fence_id = h.fence_id;
    let x = msg::lo_i16(lparam);
    let y = msg::hi_i16(lparam);
    let release = window::client_to_screen(hwnd, pecofence_platform::POINT { x, y });
    flush_window_drag(view, hwnd, Instant::now(), Some((release.x, release.y)));
    let remote = view.borrow_mut().as_mut().and_then(|v| {
        v.detach_pending = false;
        v.remote_drag.take()
    });
    if let Some(drag) = remote {
        let r = window::window_rect(drag.hwnd);
        tracing::debug!(
            started = drag.started,
            release = ?(release.x, release.y),
            rect = ?(r.left, r.top, r.right - r.left, r.bottom - r.top),
            requests = drag.requests,
            applied = drag.applied,
            "title drag released"
        );
        window::release_capture();
        hide_drag_feedback(&h.behavior);
        if !drag.started {
            if matches!(
                drag.origin,
                WindowDragOrigin::Caption {
                    click_expand: true,
                    ..
                }
            ) {
                if let Some(v) = view.borrow_mut().as_mut() {
                    v.suppress_dblclk = true;
                }
                queue.push(Command::ToggleRollUp(drag.fence));
            }
            // A title click without a drag ends the desktop selection, as a click elsewhere
            // would.
            if !h.behavior.selection.borrow().is_empty() {
                queue.push(Command::ClearFenceSelection);
            }
            return Some(0);
        }
        h.behavior.release_push(drag.hwnd);
        queue_bounds_changed(queue, &h.behavior, drag.fence, drag.hwnd);
        if !drag.group.is_empty() {
            queue.push(Command::FenceDropped(drag.fence));
            commit_members(queue, &h.behavior, &drag.group);
        } else if let Some(into) =
            merge_target_at(drag.hwnd, HWND::default(), (release.x, release.y))
        {
            queue.push(Command::MergeFence {
                fence: drag.fence,
                into,
                at: (release.x, release.y),
            });
        } else {
            queue.push(Command::FenceDropped(drag.fence));
        }
        // After the merge, which reads the slot the gap sits at: without the gap a scrolled
        // strip moves back to its active tab and the pointer would point at another slot.
        if drag.merge_target != 0 {
            queue.push(Command::MergeHint {
                target: HWND(std::ptr::null_mut()),
                at: (0, 0),
            });
        }
        return Some(0);
    }
    let (drag, had_marquee, tab_drag, col_done, had_scroll, press) = {
        let mut guard = view.borrow_mut();
        match guard.as_mut() {
            Some(v) => {
                let tab_drag = v.tab_drag.take().map(|td| {
                    // The pill settles from under the pointer into its slot.
                    v.settle_dragged_tab(&td);
                    (
                        td.index,
                        td.start_index,
                        v.tabs.get(td.index).map(|t| t.id),
                        td.moved,
                    )
                });
                (
                    v.drag.take(),
                    v.end_marquee(),
                    tab_drag,
                    v.col_drag.take().map(|_| (v.active, v.column_widths)),
                    v.end_scrollbar_press(),
                    // Not repainted here: folded into the single redraw
                    // below, after the capture is released.
                    v.take_pressed(),
                )
            }
            None => (None, false, None, None, false, None),
        }
    };
    if let Some((fence, widths)) = col_done {
        queue.push(Command::SetColumnWidths { fence, widths });
    }
    // A tab dragged past its neighbours: persist the new strip order.
    if let Some((index, start, Some(tab), _)) = tab_drag
        && index != start
    {
        queue.push(Command::ReorderTab {
            host: fence_id,
            tab,
            to: index,
        });
    }
    if drag.is_some()
        || had_marquee
        || tab_drag.is_some()
        || col_done.is_some()
        || had_scroll
        || press.is_some()
        || window::get_capture() == hwnd
    {
        // ReleaseCapture sends WM_CAPTURECHANGED synchronously: no borrow held.
        window::release_capture();
    }
    let mut guard = view.borrow_mut();
    let Some(v) = guard.as_mut() else {
        return Some(0);
    };
    // Each surface is rasterised at most once for the release: the flags
    // collect what changed, the redraws come last.
    let mut need_content = had_marquee || had_scroll;
    let mut need_chrome = false;
    match press {
        Some((
            PressTarget::Item(_)
            | PressTarget::Header(_)
            | PressTarget::NoteBox(_)
            | PressTarget::ScriptRow(_),
            _,
        )) => {
            need_content = true;
        }
        Some((PressTarget::Tab(_) | PressTarget::Up | PressTarget::Chevron, _)) => {
            need_chrome = true;
        }
        None => {}
    }
    // Release-fired controls act only when released over the same control.
    if let Some((p, true)) = press {
        match p {
            // At the portal's root its glyph opens the folder in Explorer (#192).
            PressTarget::Up if !v.deco.up_button => queue.push(Command::OpenPortalFolder(v.active)),
            PressTarget::Up => queue.push(Command::PortalUp(v.active)),
            PressTarget::Header(col) => queue.push(Command::SortColumn {
                fence: v.active,
                sort: column_sort(col),
            }),
            PressTarget::NoteBox(line) => queue.push(Command::ToggleNoteLine {
                fence: v.active,
                line,
            }),
            PressTarget::ScriptRow(row) => queue.push(Command::ScriptRowClicked {
                fence: v.active,
                row,
            }),
            // Also while a roll is still in flight: the toggle goes against the state it is
            // rolling to (`is_rolled`), so the roll turns round.
            PressTarget::Chevron => {
                tracing::info!(
                    fence = %fence_id,
                    peeking = v.peeking,
                    rolling = v.roll_anim.is_some(),
                    rolling_to = ?v.roll_anim.as_ref().map(|a| a.rolled_at_end),
                    "chevron clicked"
                );
                if v.peeking {
                    // Already expanded by peek: commit the expanded state.
                    v.peeking = false;
                    queue.push(Command::CommitExpanded(fence_id));
                } else {
                    queue.push(Command::ToggleRollUp(fence_id));
                }
            }
            // A fence collapsed to its left tab column has no chevron: a click on a tab (not
            // a drag) expands it on that tab, which the press already switched to.
            PressTarget::Tab(_)
                if matches!(tab_drag, Some((.., false)))
                    && v.column_collapsed
                    && v.rolled_up
                    && v.roll_anim.is_none() =>
            {
                // The next press may arrive as WM_LBUTTONDBLCLK; it must not collapse the
                // fence this click just expanded.
                v.suppress_dblclk = true;
                queue.push(Command::ToggleRollUp(fence_id));
            }
            _ => {}
        }
    }
    // A press that never became a drag (drags end inside the OLE loop).
    if let Some(d) = drag {
        if d.collapse_on_up {
            v.select_only(d.item);
            need_content = true;
        } else if d.deselect_on_up {
            release_item(
                &mut v.selected,
                d.item,
                ItemPress {
                    deselect_on_up: true,
                    ..Default::default()
                },
            );
            need_content = true;
        } else if d.was_sole_selected
            && !d.open_on_up
            && v.selected.len() == 1
            && v.selected.contains(&d.item)
            && v.label_hit(x, y, d.item)
            && let Some(it) = v.items.get(d.item)
        {
            // Explorer's slow double-click: a second click on the label of
            // the already-selected item renames it once the double-click
            // interval has passed (a real double-click cancels this).
            v.pending_rename = Some(it.id);
            window::set_timer(
                hwnd,
                TIMER_RENAME,
                pecofence_platform::sysparams::double_click_time_ms(),
            );
        }
        // Explorer's single-click open: a click without a drag opens the item.
        if d.open_on_up
            && let Some(it) = v.items.get(d.item)
        {
            open_items(v, queue, vec![it.id]);
        }
    }
    if need_content && !v.rolled_up {
        let _ = v.redraw_content();
    }
    if need_chrome {
        let _ = v.redraw_chrome_only();
    }
    Some(0)
}

pub(super) fn on_capturechanged(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, queue, .. } = h;
    let fence_id = h.fence_id;
    let mut guard = view.borrow_mut();
    if let Some(v) = guard.as_mut() {
        v.drag = None;
        v.click_pending = None;
        v.rbutton_press = None;
        v.clear_pressed();
        if v.detach_pending || v.remote_drag.is_some() {
            tracing::info!(
                ?hwnd,
                pending = v.detach_pending,
                remote = v.remote_drag.is_some(),
                now = ?window::get_capture(),
                "capture lost during tab tear-off"
            );
        }
        v.detach_pending = false;
        if let Some(rd) = v.remote_drag.take() {
            hide_drag_feedback(&v.behavior);
            for (fence, hwnd) in std::iter::once((rd.fence, rd.hwnd))
                .chain(rd.group.iter().map(|m| (m.fence, m.hwnd)))
            {
                if rd.started {
                    v.behavior.release_push(hwnd);
                }
                queue_bounds_changed(queue, &v.behavior, fence, hwnd);
            }
            if rd.merge_target != 0 {
                queue.push(Command::MergeHint {
                    target: HWND(std::ptr::null_mut()),
                    at: (0, 0),
                });
            }
        }
        let marquee = v.end_marquee();
        // Losing capture mid tab-drag still commits the order shown.
        if let Some(td) = v.tab_drag.take() {
            v.settle_dragged_tab(&td);
            let _ = v.redraw_chrome_only();
            if td.index != td.start_index
                && let Some(tab) = v.tabs.get(td.index).map(|t| t.id)
            {
                queue.push(Command::ReorderTab {
                    host: fence_id,
                    tab,
                    to: td.index,
                });
            }
        }
        v.col_drag = None;
        let scroll = v.end_scrollbar_press();
        if marquee || scroll {
            let _ = v.redraw();
        }
    }
    Some(0)
}

pub(super) fn on_lbuttondblclk(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, queue, .. } = h;
    let fence_id = h.fence_id;
    let x = msg::lo_i16(lparam);
    let y = msg::hi_i16(lparam);
    let mut guard = view.borrow_mut();
    if let Some(v) = guard.as_mut() {
        // A real double-click launches; it never renames.
        v.cancel_pending_rename();
        v.hide_tip();
        v.focus_visible = false;
        if let Some(col) = v.divider_hit(x, y) {
            // Explorer: double-clicking a divider sizes the column to fit.
            v.col_drag = None;
            v.auto_fit_column(col);
            let (fence, widths) = (v.active, v.column_widths);
            let _ = v.redraw_content();
            queue.push(Command::SetColumnWidths { fence, widths });
            return Some(0);
        }
        // The chevron's toggle is one action per click pair: the second
        // click of a double-click is swallowed, whether or not the roll
        // animation from the first is still running (with animations off it
        // would otherwise toggle straight back).
        if v.chevron_at(x, y) {
            return Some(0);
        }
        // In a note the second click is another press: a check box ticks back.
        if v.press_tool_at(x, y, true).is_some() {
            drop(guard);
            window::set_capture(hwnd);
            return Some(0);
        }
        if let Some(cmd) = v.note_press(x, y) {
            drop(guard);
            queue.push(cmd);
            return Some(0);
        }
        // A script row acts once per double-click: a link opens one tab. So does the empty
        // state's action row.
        if v.script_row_at(x, y).is_some() || v.empty_action_at(x, y) {
            return Some(0);
        }
        // The folder glyph at a portal's root opens Explorer once per double-click.
        if v.glyph_opens_folder() && v.up_button_at(x, y) {
            return Some(0);
        }
        // The second click of a rapid double-click on a button is another
        // press (a header toggles the sort twice, like Explorer).
        if v.press_button_at(x, y).is_some() {
            drop(guard);
            window::set_capture(hwnd);
            return Some(0);
        }
        // Likewise on the scrollbar: clicking the track quickly pages
        // again (it must never launch the item under the strip).
        if let Some(repeat) = v.press_scrollbar(x, y) {
            drop(guard);
            window::set_capture(hwnd);
            if repeat {
                window::set_timer(hwnd, TIMER_SCROLL_REPEAT, SCROLL_PAGE_FIRST_MS);
            }
            queue.push(Command::RaiseFence(hwnd));
            return Some(0);
        }
        if let Some(i) = v.hit_item(x, y)
            && let Some(item) = v.items.get(i)
        {
            // With single-click open the first click of the pair opened it already.
            if !h.behavior.single_click.get() {
                open_items(v, queue, vec![item.id]);
            }
        } else if v.tab_at(x, y).is_some() && !std::mem::take(&mut v.suppress_dblclk) {
            queue.push(Command::ToggleRollUp(fence_id));
        }
    }
    Some(0)
}

// A plain press (WM_RBUTTONDOWN, or WM_RBUTTONDBLCLK for the second press of
// a double-click) takes the capture so the release pairs with it (see
// `rbutton_press`). The other button during a tab drag / tear-off cancels it
// (as it does for an OLE drag); the WM_RBUTTONUP that follows then finds no
// press and opens no menu.
pub(super) fn on_rbuttondown(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, queue, .. } = h;
    if h.in_size_move.get() {
        // A title drag in the system move loop: the right button cancels it (as it does the
        // Liquid Glass drag below); the release finds no press and opens no menu.
        super::nc::cancel_move(h, hwnd);
        return Some(0);
    }
    if window::get_capture() != hwnd {
        if let Some(v) = view.borrow_mut().as_mut() {
            v.rbutton_press = Some((msg::lo_i16(lparam), msg::hi_i16(lparam)));
        }
        window::set_capture(hwnd);
        return Some(0);
    }
    let cancel = view
        .borrow_mut()
        .as_mut()
        .and_then(|v| v.cancel_tab_or_remote_drag());
    match cancel {
        Some(c) => {
            finish_drag_cancel(c, &h.view, queue, &h.behavior);
            Some(0)
        }
        None => None,
    }
}

pub(super) fn on_rbuttonup(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, queue, .. } = h;
    let x = msg::lo_i16(lparam);
    let y = msg::hi_i16(lparam);
    let press = view
        .borrow_mut()
        .as_mut()
        .and_then(|v| v.rbutton_press.take());
    let Some((px, py)) = press else {
        return Some(0);
    };
    if window::get_capture() == hwnd {
        // ReleaseCapture sends WM_CAPTURECHANGED synchronously: no borrow held.
        window::release_capture();
    }
    let (tx, ty) = window::drag_threshold();
    if (x - px).abs() > tx || (y - py).abs() > ty {
        return Some(0);
    }
    let mut guard = view.borrow_mut();
    let Some(v) = guard.as_mut() else {
        return Some(0);
    };
    let pt = window::cursor_pos();
    v.cancel_pending_rename();
    v.hide_tip();
    if v.header_band_hit(x, y) {
        // Explorer: right-click on the column header = column chooser.
        queue.push(Command::HeaderMenu {
            fence: v.active,
            x: pt.x,
            y: pt.y,
        });
        return Some(0);
    }
    if let Some(t) = v.tab_at(x, y) {
        let fence = v.tabs.get(t).map(|t| t.id).unwrap_or(v.active);
        queue.push(Command::FenceMenu {
            fence,
            x: pt.x,
            y: pt.y,
        });
        return Some(0);
    }
    v.focus_visible = false;
    match v.hit_item(x, y) {
        Some(i) => {
            if !v.selected.contains(&i) {
                v.selected.clear();
                v.selected.insert(i);
                v.anchor_index = Some(i);
                v.range_anchor = Some(i);
            }
            // Like a left click, a right click on an item grants the focus
            // session (Explorer activates the view), so the selection shows
            // in the accent under the menu.
            let _ = v.activate_by_pointer();
            let _ = v.redraw_content();
            let cmd = Command::ItemMenu {
                fence: v.active,
                items: v.selected_ids(),
                x: pt.x,
                y: pt.y,
            };
            drop(guard);
            focus_session(view, hwnd);
            queue.push(cmd);
        }
        None => queue.push(Command::FenceMenu {
            fence: v.active,
            x: pt.x,
            y: pt.y,
        }),
    }
    Some(0)
}

pub(super) const WM_XBUTTONDOWN: u32 = 0x020B;
pub(super) const WM_XBUTTONUP: u32 = 0x020C;

/// The mouse's back and forward buttons (feedback #116): back goes up a folder as Backspace
/// does, forward goes into the folder the last back left. Handled (TRUE) either way, so the
/// buttons do nothing else in a fence.
pub(super) fn on_xbuttonup(
    h: &HandlerCtx,
    _hwnd: HWND,
    wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    const XBUTTON1: u32 = 1;
    const XBUTTON2: u32 = 2;
    let HandlerCtx { view, queue, .. } = h;
    let view = view.borrow();
    let Some(v) = view.as_ref() else {
        return Some(1);
    };
    match msg::hi_u16(wparam) {
        XBUTTON1 if v.deco.up_button => queue.push(Command::PortalUp(v.active)),
        XBUTTON2 => queue.push(Command::PortalForward(v.active)),
        _ => {}
    }
    Some(1)
}

/// A press in the text of the note that is being edited in this window: it places the
/// caret, so the editor stays open.
pub(super) fn press_in_note_editor(h: &HandlerCtx, message: u32, lparam: isize) -> bool {
    matches!(
        message,
        msg::WM_LBUTTONDOWN | msg::WM_LBUTTONDBLCLK | msg::WM_RBUTTONDOWN | WM_RBUTTONDBLCLK
    ) && h.view.try_borrow().is_ok_and(|view| {
        view.as_ref().is_some_and(|v| {
            v.note_edit.is_some() && v.in_note_text(msg::lo_i16(lparam), msg::hi_i16(lparam))
        })
    })
}

/// The mouse in the text of a note that is being edited (see `note_text`): a press places
/// the caret (Shift extends the selection, a double-click takes the word), moving with the
/// button held selects, a right-click opens the editor's own text menu. None for every other
/// message: the usual handling goes on.
pub(super) fn note_edit_mouse(
    h: &HandlerCtx,
    hwnd: HWND,
    message: u32,
    wparam: usize,
    lparam: isize,
) -> Option<isize> {
    if !matches!(
        message,
        msg::WM_LBUTTONDOWN
            | msg::WM_LBUTTONDBLCLK
            | msg::WM_MOUSEMOVE
            | msg::WM_LBUTTONUP
            | msg::WM_RBUTTONDOWN
            | WM_RBUTTONDBLCLK
            | msg::WM_RBUTTONUP
    ) {
        return None;
    }
    let x = msg::lo_i16(lparam);
    let y = msg::hi_i16(lparam);
    let mut guard = h.view.try_borrow_mut().ok()?;
    let v = guard.as_mut()?;
    v.note_edit.as_ref()?;
    let fence = v.active;
    match message {
        msg::WM_LBUTTONDOWN | msg::WM_LBUTTONDBLCLK if v.in_note_text(x, y) => {
            let offset = v.note_offset_at(x, y)?;
            v.note_drag = true;
            drop(guard);
            window::set_capture(hwnd);
            h.queue.push(Command::NoteCaret {
                fence,
                offset,
                extend: wparam & msg::MK_SHIFT as usize != 0,
                word: message == msg::WM_LBUTTONDBLCLK,
            });
            Some(0)
        }
        msg::WM_MOUSEMOVE if v.note_drag => {
            if wparam & msg::MK_LBUTTON as usize == 0 {
                // The release went elsewhere.
                v.note_drag = false;
                return None;
            }
            let offset = v.note_offset_at(x.max(v.content_x), y.max(v.content_y))?;
            drop(guard);
            h.queue.push(Command::NoteCaret {
                fence,
                offset,
                extend: true,
                word: false,
            });
            Some(0)
        }
        msg::WM_LBUTTONUP if v.note_drag => {
            v.note_drag = false;
            drop(guard);
            window::release_capture();
            Some(0)
        }
        msg::WM_RBUTTONDOWN | WM_RBUTTONDBLCLK if v.in_note_text(x, y) => Some(0),
        msg::WM_RBUTTONUP if v.in_note_text(x, y) => {
            drop(guard);
            let at = window::client_to_screen(hwnd, pecofence_platform::POINT { x, y });
            h.queue.push(Command::NoteEditMenu { x: at.x, y: at.y });
            Some(0)
        }
        _ => None,
    }
}

pub(super) fn on_mousewheel(
    h: &HandlerCtx,
    hwnd: HWND,
    wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        fence_id,
        view,
        queue,
        behavior,
        ..
    } = h;
    let delta = msg::hi_u16(wparam) as u16 as i16 as i32;
    let ctrl = wparam & msg::MK_CONTROL as usize != 0;
    let mut guard = view.borrow_mut();
    if let Some(v) = guard.as_mut() {
        v.hide_tip();
        if v.scrollbar_pressed() {
            // The thumb drag / track or arrow repeat owns the offset (Win32
            // SB_THUMBTRACK ignores the wheel): a notch would start a glide
            // the next pointer move snaps back, and Ctrl+wheel would relayout
            // under the held thumb.
            return Some(0);
        }
        if ctrl {
            // Ctrl+wheel never scrolls (Explorer); in Icons view it steps the
            // icon size, one step per whole notch (touchpads accumulate). A tool
            // fence has no icons to size.
            if v.layout == ViewLayout::Icons && !v.shows_tool() {
                v.wheel_zoom_accum += delta;
                while v.wheel_zoom_accum.abs() >= msg::WHEEL_DELTA {
                    let larger = v.wheel_zoom_accum > 0;
                    v.wheel_zoom_accum -= if larger {
                        msg::WHEEL_DELTA
                    } else {
                        -msg::WHEEL_DELTA
                    };
                    queue.push(Command::StepIconSize {
                        fence: v.active,
                        larger,
                    });
                }
            }
            return Some(0);
        }
        // Wheel over the title row (or the left tab column) of a tabbed fence: one tab per
        // whole notch, accumulated for touchpads. A held left button (tab, column or title
        // drag, marquee) and a tab title rename keep the tab where it is.
        let pt = window::screen_to_client(
            hwnd,
            pecofence_platform::POINT {
                x: msg::lo_i16(lparam),
                y: msg::hi_i16(lparam),
            },
        );
        // A left tab column is the title bar (it stays when the fence collapses to it).
        let over_title = if v.tabs_on_rail() {
            pt.x >= 0 && pt.x < v.content_x && pt.y >= 0 && pt.y < v.chrome_panel.size_px().1
        } else {
            pt.y >= 0 && pt.y < v.title_h_px()
        };
        let lbutton = wparam & msg::MK_LBUTTON as usize != 0;
        if v.tabs.len() > 1 && over_title && !lbutton && !v.title_renaming && v.roll_anim.is_none()
        {
            v.wheel_tab_accum += delta;
            let notches = v.wheel_tab_accum / msg::WHEEL_DELTA;
            v.wheel_tab_accum -= notches * msg::WHEEL_DELTA;
            let cur = v.active_index();
            let next = wheel_tab_target(cur, v.tabs.len(), notches);
            if next != cur {
                // An open item rename closes, as it does when the content scrolls.
                v.dismiss_rename_for_scroll();
                queue.push(Command::SwitchTab {
                    host: *fence_id,
                    tab: v.tabs[next].id,
                });
            }
            return Some(0);
        }
        v.wheel_tab_accum = 0;
        if v.rolled_up || v.roll_anim.is_some() {
            // A collapsed container has no viewport (WinUI Expander):
            // the wheel must not scroll the hidden content, and the
            // frozen surface of a rolling fence stays put.
            return Some(0);
        }
        // Windows "lines per notch" (SPI_GETWHEELSCROLLLINES) rows in
        // List / Details, one grid row in Icons, a viewport when set to
        // "one screen at a time". Whole notches (|delta| >= 120) glide
        // with the 5 ms/DIP decelerate curve: each notch retargets the
        // glide from its anticipated end (WinUI ScrollPresenter). The
        // 250 ms cap is a deliberate departure from the uncapped rule (a
        // 3-row notch would take ~480 ms): Explorer's list view settles
        // in roughly 150-250 ms per notch, and notches keep accumulating
        // into the target either way. Fractional deltas — a precision
        // touchpad (which synthesises its own inertia tail as wheel
        // messages) or a high-resolution wheel — scroll proportionally
        // and are applied 1:1 on the next frame with no glide, like
        // DirectManipulation tracking the fingers; easing each ~5 DIP
        // message would leave the content a glide behind the finger.
        let (layout, view_h) = v.scroll_extent();
        // A note or a script fence scrolls its lines of text as a list scrolls its rows.
        let text = v.scrolls_text();
        let step = wheel_step_dip(
            behavior.wheel_lines.get(),
            delta as f32 / msg::WHEEL_DELTA as f32,
            v.scroll_row_step(&layout),
            view_h,
            matches!(layout, ItemLayout::Grid(_)) && !text,
        );
        if step == 0.0 {
            return Some(0);
        }
        let cap = if delta.abs() < msg::WHEEL_DELTA {
            Duration::ZERO
        } else {
            motion::NORMAL
        };
        let mut target = v.scroll_target() - step;
        if text && delta.abs() >= msg::WHEEL_DELTA {
            // A whole notch ends with a whole line at the top edge (the last position is one
            // too, see `take_tool_extent`). A step shorter than the way to the next line
            // keeps its own target, so the wheel always moves the text.
            let line = v.tool_line_snap(target);
            if (line - v.scroll_target()).abs() > 0.5 {
                target = line;
            }
            tracing::debug!(fence = %v.active, from = v.scroll_target(), step, to = target, "wheel over a tool fence's text: to a line's top");
        }
        v.scroll_to_by_wheel(target, cap);
        if v.scroll_anim.is_none() {
            // Animations off: the offset snapped, draw it now.
            if v.marquee.is_some() {
                // The band is anchored to the content: re-select under it.
                v.update_marquee_selection();
                let _ = v.redraw();
            } else {
                let _ = v.redraw_content();
            }
        }
    }
    Some(0)
}
