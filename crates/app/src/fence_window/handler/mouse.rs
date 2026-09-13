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
        }
        v.hide_tip();
        let chrome_changed = std::mem::take(&mut v.up_hovered)
            | std::mem::take(&mut v.chevron_hovered)
            | v.tab_hover.take().is_some();
        // Leaving the bar starts the 0.5 s contract delay, not an
        // instant collapse.
        v.set_scrollbar_pointer(None);
        let content_changed = v.header_hover.take().is_some();
        let (item_changed, title_changed) = v.set_hover(None, false);
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
        // Any new press cancels an armed slow-double-click rename.
        v.cancel_pending_rename();
        v.hide_tip();
        // Mouse interaction hides the keyboard focus ring (Explorer). The
        // ring lives on the content surface: arms that do not repaint it
        // anyway do so once below (one raster per click).
        let focus_cleared = std::mem::take(&mut v.focus_visible) && !v.rolled_up;
        // Up button / chevron: pressed look now, action on release inside.
        if v.up_button_at(x, y) || v.chevron_at(x, y) {
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
            let pointer_x = x as f32 / v.scale();
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
            v.pressed = Some(PressTarget::Tab(t));
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
        let in_content = !v.rolled_up && y >= v.title_h_px();
        if let Some(col) = v.divider_hit(x, y) {
            v.col_drag = Some(ColDrag {
                col,
                start_x_px: x,
                start_w: v.column_width(col),
            });
            v.header_hover = None;
            if focus_cleared {
                let _ = v.redraw_content();
            }
            drop(guard);
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
        let hit = v.hit_item(x, y);
        let mut capture = false;
        match hit {
            Some(i) => {
                // Explorer: a plain press keeps a multi-selection for
                // dragging and collapses it on a release without a drag; a
                // press on the sole selected item may become a slow
                // double-click rename.
                let plain = !shift && !ctrl && v.selected.contains(&i);
                let collapse_on_up = plain && v.selected.len() > 1;
                let was_sole_selected = plain && v.selected.len() == 1;
                if shift && let Some(a) = v.range_anchor {
                    // Explorer-style range from the anchor to the clicked item.
                    let (lo, hi) = (a.min(i), a.max(i));
                    if !ctrl {
                        v.selected.clear();
                    }
                    v.selected.extend(lo..=hi);
                } else if ctrl {
                    if !v.selected.remove(&i) {
                        v.selected.insert(i);
                    }
                    v.range_anchor = Some(i);
                } else if !v.selected.contains(&i) {
                    v.selected.clear();
                    v.selected.insert(i);
                    v.range_anchor = Some(i);
                }
                v.anchor_index = Some(i);
                v.drag = Some(DragState {
                    start: (x, y),
                    item: i,
                    collapse_on_up,
                    was_sole_selected,
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
        window::release_capture();
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
            return Some(0);
        }
        let rect = window::window_rect(drag.hwnd);
        queue.push(Command::FenceBoundsChanged {
            fence: drag.fence,
            rect,
        });
        if drag.merge_target != 0 {
            queue.push(Command::MergeHint {
                target: HWND(std::ptr::null_mut()),
                x: 0,
            });
        }
        if let Some(into) = merge_target_at(drag.hwnd, HWND::default(), (release.x, release.y)) {
            let x = release.x;
            queue.push(Command::MergeFence {
                fence: drag.fence,
                into,
                x,
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
                    (td.index, td.start_index, v.tabs.get(td.index).map(|t| t.id))
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
    if let Some((index, start, Some(tab))) = tab_drag
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
        Some((PressTarget::Item(_) | PressTarget::Header(_), _)) => {
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
            PressTarget::Up => queue.push(Command::PortalUp(v.active)),
            PressTarget::Header(col) => queue.push(Command::SortColumn {
                fence: v.active,
                sort: column_sort(col),
            }),
            PressTarget::Chevron if v.roll_anim.is_none() => {
                if v.peeking {
                    // Already expanded by peek: commit the expanded state.
                    v.peeking = false;
                    queue.push(Command::CommitExpanded(fence_id));
                } else {
                    queue.push(Command::ToggleRollUp(fence_id));
                }
            }
            _ => {}
        }
    }
    // A press that never became a drag (drags end inside the OLE loop).
    if let Some(d) = drag {
        if d.collapse_on_up {
            v.select_only(d.item);
            need_content = true;
        } else if d.was_sole_selected
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
                pecofence_platform::rawinput::double_click_time_ms(),
            );
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
            let rect = window::window_rect(rd.hwnd);
            queue.push(Command::FenceBoundsChanged {
                fence: rd.fence,
                rect,
            });
            if rd.merge_target != 0 {
                queue.push(Command::MergeHint {
                    target: HWND(std::ptr::null_mut()),
                    x: 0,
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
            if v.navigate_folders && item.is_folder {
                queue.push(Command::PortalEnter {
                    fence: v.active,
                    path: item.path.clone(),
                });
            } else {
                queue.push(Command::LaunchItem(item.id));
            }
        } else if v.tab_at(x, y).is_some() {
            queue.push(Command::ToggleRollUp(fence_id));
        }
    }
    Some(0)
}

// The other button during a tab drag / tear-off cancels it (as it does for
// an OLE drag); the WM_RBUTTONUP that follows must not open a menu.
pub(super) fn on_rbuttondown(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, queue, .. } = h;
    // A swallow left over from a cancel whose WM_RBUTTONUP went to another
    // window (tear-off: the pointer was over the torn-off window, which the
    // cancel destroyed) must not eat this click's menu.
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
    {
        v.rbutton_swallow = false;
    }
    if window::get_capture() != hwnd {
        return None;
    }
    let cancel = view.borrow_mut().as_mut().and_then(|v| {
        let c = v.cancel_tab_or_remote_drag();
        if c.is_some() {
            v.rbutton_swallow = true;
        }
        c
    });
    match cancel {
        Some(c) => {
            finish_drag_cancel(c, queue);
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
    let mut guard = view.borrow_mut();
    let Some(v) = guard.as_mut() else {
        return Some(0);
    };
    if std::mem::take(&mut v.rbutton_swallow) {
        return Some(0);
    }
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

pub(super) fn on_mousewheel(
    h: &HandlerCtx,
    _hwnd: HWND,
    wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
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
            // icon size, one step per whole notch (touchpads accumulate).
            if v.layout == ViewLayout::Icons {
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
        let step = wheel_step_dip(
            behavior.wheel_lines.get(),
            delta as f32 / msg::WHEEL_DELTA as f32,
            layout.row_step(),
            view_h,
            matches!(layout, ItemLayout::Grid(_)),
        );
        if step == 0.0 {
            return Some(0);
        }
        let cap = if delta.abs() < msg::WHEEL_DELTA {
            Duration::ZERO
        } else {
            motion::NORMAL
        };
        v.scroll_to(v.scroll_target() - step, cap);
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
