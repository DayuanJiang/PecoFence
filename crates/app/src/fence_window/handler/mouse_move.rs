//! WM_MOUSEMOVE: marquee, scrollbar drag, remote and tab drags, item drag → OLE, column drag, pressed-button tracking, hover / tip / peek.

use super::*;

pub(super) fn on_mousemove(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx {
        view,
        queue,
        behavior,
        peek_armed,
        menu_open,
        in_size_move,
        ..
    } = h;
    let x = msg::lo_i16(lparam);
    let y = msg::hi_i16(lparam);
    let mut guard = view.borrow_mut();
    let Some(v) = guard.as_mut() else {
        return Some(0);
    };
    v.set_mouse_inside(true);
    if v.marquee.is_some() {
        // Pointer outside the item area: auto-scroll per frame while the
        // button is down (`marquee_autoscroll` in `tick`).
        let (top, bottom) = v.marquee_band_px();
        let outside = y < top || y > bottom;
        if let Some(m) = v.marquee.as_mut() {
            m.cur = (x, y);
            m.pointer = (x, y);
            if !outside {
                m.auto_last = None;
            } else if m.auto_last.is_none() {
                m.auto_last = Some(Instant::now());
            }
        }
        if outside {
            v.frames.request();
        }
        v.update_marquee_selection();
        let _ = v.redraw();
        return Some(0);
    }
    if let Some(sd) = v.scroll_drag.as_ref() {
        let (start_y, start_scroll) = (sd.start_y_px, sd.start_scroll);
        if let Some(g) = v.scrollbar_geometry()
            && g.track_h > g.thumb_h
        {
            let dy = (y - start_y) as f32 / v.scale();
            // Direct manipulation: the thumb drives the offset, no glide;
            // every drag frame lands on a whole device pixel (Explorer
            // tracks the thumb in integer positions).
            v.scroll_anim = None;
            let next = snap_offset(
                start_scroll + dy * (g.content_h - g.view_h) / (g.track_h - g.thumb_h),
                v.scale(),
            );
            if (next - v.scroll_y).abs() > 0.01 {
                v.dismiss_rename_for_scroll();
            }
            v.scroll_y = next;
        }
        // Keep the hot zone current so a release with the pointer off
        // the bar starts the contract delay at once.
        let hit = v.scrollbar_hit(x, y).map(|(h, _)| h);
        v.set_scrollbar_pointer(hit);
        v.note_scroll_activity();
        let _ = v.redraw_content();
        return Some(0);
    }
    if v.page_repeat.is_some() {
        // Holding the track / an arrow: only the hot zone and the button
        // states matter (the repeat timer hit-tests the pointer itself).
        let hit = v.scrollbar_hit(x, y).map(|(h, _)| h);
        v.set_scrollbar_pointer(hit);
        return Some(0);
    }
    if v.remote_drag.is_some() && window::key_down(msg::VK_ESCAPE) {
        // Esc pressed while the focus session went elsewhere: cancel here.
        let cancel = v.cancel_tab_or_remote_drag();
        drop(guard);
        if let Some(c) = cancel {
            finish_drag_cancel(c, queue);
        }
        return Some(0);
    }
    if let Some(drag) = v.remote_drag.as_mut() {
        let pt = window::cursor_pos();
        let point = (pt.x, pt.y);
        drag.track_pointer(point, window::drag_threshold());
        if drag.pending_pointer.is_some() {
            v.frames.request();
        }
        return Some(0);
    }
    if let Some(td) = v.tab_drag.as_ref() {
        // A dragged tab is the drag handle: it keeps its pressed look while
        // it follows the pointer along the strip (TabView), so press_inside
        // is not tracked for PressTarget::Tab; tear-off clears it below.
        let far = (TAB_DETACH_DIP * v.scale()) as i32;
        let sy = td.start.1;
        let index = td.index;
        let (w_px, _) = v.chrome_panel.size_px();
        // Leaving the title row vertically (or the window sideways) tears the
        // tab off; moving along the strip reorders it.
        if (y - sy).abs() > far || x < 0 || x >= w_px {
            let tab = v.tabs.get(index).map(|t| t.id);
            v.tab_drag = None;
            v.tab_slide.clear();
            v.tab_settle = None;
            v.tab_hover = None;
            v.pressed = None;
            v.press_inside = false;
            v.hide_tip();
            let _ = v.redraw_chrome_only();
            if let Some(tab) = tab {
                // Keep the capture: once the App has created the new
                // window it calls begin_remote_drag and this window keeps
                // moving it with the cursor until the button is released.
                v.detach_pending = true;
                v.detach_cancelled = false;
                let pt = window::cursor_pos();
                tracing::debug!(?hwnd, "tab torn off; waiting for the new window");
                queue.push(Command::DetachTab {
                    tab,
                    x: pt.x,
                    y: pt.y,
                    from_drag: true,
                });
            }
            return Some(0);
        }
        // Inside the system drag threshold the press is still a click: the
        // pill stays in its slot (the SwitchTab pushed on button-down glides
        // the active pill exactly as a click does) and nothing is redrawn.
        if !td.moved {
            let (tx, ty) = window::drag_threshold();
            let sx = td.start.0;
            if (x - sx).abs() <= tx && (y - sy).abs() <= ty {
                return Some(0);
            }
        }
        // The pill rides under the pointer (TabView / Edge); once its
        // centre passes a neighbour's centre the two swap and the neighbour
        // slides into the vacated slot (167 ms point-to-point). One slot per
        // step, repeated so a fast sweep can cross several neighbours in one
        // message; `tab_reorder_step` is stable for any pair of widths.
        let now = Instant::now();
        let pointer_x = x as f32 / v.scale();
        if let Some(td) = v.tab_drag.as_mut() {
            td.moved = true;
            td.pointer_x = pointer_x;
        }
        let prev: Vec<(FenceId, f32)> = v
            .tabs
            .iter()
            .map(|t| t.id)
            .zip(v.tab_draw_xs(now))
            .collect();
        let mut index = index;
        let grab_dx = v.tab_drag.as_ref().map_or(0.0, |td| td.grab_dx);
        let mut reordered = false;
        for _ in 0..v.tabs.len() {
            let rects = v.tab_rects();
            let over = tab_reorder_at_pointer(index, pointer_x, grab_dx, &rects);
            let Some(over) = over.filter(|o| *o < v.tabs.len()) else {
                break;
            };
            let t = v.tabs.remove(index);
            v.tabs.insert(over, t);
            if let Some(td) = v.tab_drag.as_mut() {
                td.index = over;
            }
            index = over;
            reordered = true;
        }
        if reordered {
            v.tab_hover = Some(index);
            v.pressed = Some(PressTarget::Tab(index));
            v.hide_tip();
            v.sync_tab_slides(&prev, now);
        }
        // The chrome is rasterised once per frame by `tick`, not once per
        // mouse message (a 1000 Hz mouse would otherwise redraw it 16 times
        // per displayed frame on top of the slide tweens' own draw).
        v.chrome_busy = true;
        v.frames.request();
        return Some(0);
    }
    if let Some(d) = v.drag.as_ref() {
        let (tx, ty) = window::drag_threshold();
        let (sx, sy) = d.start;
        let item = d.item;
        if (x - sx).abs() <= tx && (y - sy).abs() <= ty {
            // Inside the drag threshold the press is still a click: the
            // pressed fill follows the pointer like a button's (a tab being
            // dragged keeps its look instead, see the tab_drag arm above).
            let p = PressTarget::Item(item);
            let inside = v.press_hit(p, x, y);
            if v.pressed == Some(p) && inside != v.press_inside {
                v.press_inside = inside;
                v.redraw_for_press(p);
            }
            return Some(0);
        }
        // Threshold crossed: hand the selection to an OLE drag carrying the
        // shell's own data object, so Explorer, the Recycle Bin, other
        // programs and our other fences all receive real files (with the
        // shell drag image and Move / Copy / Link cursors).
        v.drag = None;
        v.pressed = None;
        v.press_inside = false;
        v.cancel_pending_rename();
        v.hide_tip();
        if !v.selected.contains(&item) {
            v.selected.clear();
            v.selected.insert(item);
        }
        let ids = v.selected_ids();
        let paths = v.selected_paths();
        let from = v.active;
        let from_portal = v.is_portal;
        v.ole_drag = true;
        v.ole_drag_press = (sx, sy);
        let _ = v.redraw();
        drop(guard);
        // ReleaseCapture sends WM_CAPTURECHANGED synchronously: no borrow held.
        window::release_capture();
        let refs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
        let mut drag_returned: Option<Instant> = None;
        let effect = match dragdrop::data_object_for_paths(&refs) {
            Ok(obj) => {
                INTERNAL_DRAG.with(|s| {
                    *s.borrow_mut() = Some(InternalDrag {
                        from,
                        items: ids.clone(),
                        paths: paths.clone(),
                    })
                });
                // Nested message loop until the button is released.
                let effect = dragdrop::do_drag_drop(hwnd, &obj, dragdrop::ALL_EFFECTS);
                drag_returned = Some(Instant::now());
                INTERNAL_DRAG.with(|s| *s.borrow_mut() = None);
                Some(effect)
            }
            Err(e) => {
                tracing::warn!(error = %e, "shell data object for drag failed");
                None
            }
        };
        // Where it landed, resolved before re-borrowing: WindowFromPoint
        // sends WM_NCHITTEST to our own windows synchronously.
        let pt = window::cursor_pos();
        let target = desktop::root_ancestor(desktop::window_from_point(pt.x, pt.y));
        let class = desktop::class_name(target);
        let on_desktop = matches!(
            class.as_str(),
            "Progman" | "WorkerW" | "SHELLDLL_DefView" | "SysListView32"
        );
        let mut guard = view.borrow_mut();
        let Some(v) = guard.as_mut() else {
            return Some(0);
        };
        v.ole_drag = false;
        v.drag_scroll = None;
        if v.snap_scroll() {
            let _ = v.redraw_content();
        }
        v.apply_drop_feedback(None);
        window::set_standard_cursor(StandardCursor::Arrow);
        // Info level on purpose: one line per drag, and it is the first thing needed when a
        // user reports "the icon took a while to land".
        tracing::info!(
            outcome = ?effect.as_ref().map(|r| (r.dropped, r.effect)),
            on_desktop,
            %class,
            from_portal,
            "drag-out finished"
        );
        match effect {
            // Esc / right button: Explorer cancels outright, so do we.
            Some(r) if !r.dropped => {}
            // Bare desktop (its hidden ListView accepts nothing, so the
            // effect is None): Explorer's same-folder drop leaves desktop
            // items alone, so keep "back to the inbox". A Ctrl / Alt drop
            // means duplicate, not relocate. A portal's files really moved
            // there; the desktop watcher files them.
            Some(r)
                if on_desktop
                    && !from_portal
                    && matches!(r.effect, DropEffect::None | DropEffect::Move) =>
            {
                queue.push(Command::MoveItemsToInbox { items: ids });
            }
            Some(r)
                if matches!(r.effect, DropEffect::Move | DropEffect::Copy)
                    && class != anchor::FENCE_CLASS =>
            {
                queue.push(Command::DragOutFinished);
            }
            _ => {}
        }
        let _ = v.redraw();
        // Everything between DoDragDrop's return and here runs before the first frame of the
        // drop's layout motion can be shown.
        if let Some(t) = drag_returned {
            let spent = t.elapsed();
            if spent > Duration::from_millis(8) {
                tracing::info!(ms = spent.as_secs_f32() * 1000.0, "slow drag-out post work");
            }
        }
        return Some(0);
    }
    if let Some(cd) = v.col_drag.as_ref() {
        // Divider sits on the column's left edge: dragging right shrinks it.
        let dx = (x - cd.start_x_px) as f32 / v.scale();
        let (col, w) = (cd.col, cd.start_w - dx);
        v.set_column_width(col, w);
        window::set_standard_cursor(StandardCursor::SizeWE);
        let _ = v.redraw_content();
        return Some(0);
    }
    // A release-fired button is held: track whether the pointer is still
    // over it (Win32 button semantics: pressed fill and hover fill both go
    // when the pointer leaves, both return with it); no tip updates
    // meanwhile.
    if let Some(p) = v.pressed
        && matches!(
            p,
            PressTarget::Up | PressTarget::Chevron | PressTarget::Header(_)
        )
    {
        if p == PressTarget::Chevron {
            // The chevron box sits where the title row used to be grabbed:
            // a press that turns into a drag still moves the fence (the
            // click-to-expand press does the same in WM_NCMOUSEMOVE).
            let (tx, ty) = window::drag_threshold();
            let (sx, sy) = v.press_origin;
            if (x - sx).abs() > tx || (y - sy).abs() > ty {
                v.pressed = None;
                v.press_inside = false;
                v.chevron_hovered = false;
                let _ = v.redraw_chrome_only();
                drop(guard);
                // ReleaseCapture sends WM_CAPTURECHANGED synchronously.
                window::release_capture();
                let pt = window::cursor_pos();
                window::send_message(
                    hwnd,
                    msg::WM_SYSCOMMAND,
                    SC_MOVE_CAPTION,
                    msg::make_lparam(pt.x, pt.y),
                );
                return Some(0);
            }
        }
        let inside = v.press_hit(p, x, y);
        if inside != v.press_inside {
            v.press_inside = inside;
            match p {
                PressTarget::Up => v.up_hovered = inside,
                PressTarget::Chevron => v.chevron_hovered = inside,
                PressTarget::Header(c) => v.header_hover = inside.then_some(c),
                _ => {}
            }
            v.redraw_for_press(p);
        }
        return Some(0);
    }
    // Title-row controls (chrome surface) and content hovers (content
    // surface) are resolved together so each surface is redrawn at most
    // once, and only when its own state changed.
    let up = v.up_button_at(x, y);
    let chev = v.chevron_at(x, y);
    let tab = v.tab_at(x, y);
    let chrome_changed = std::mem::replace(&mut v.up_hovered, up) != up
        || std::mem::replace(&mut v.chevron_hovered, chev) != chev
        || std::mem::replace(&mut v.tab_hover, tab) != tab;
    let on_control = up || chev || tab.is_some();
    // The divider's SizeWE cursor is set by WM_SETCURSOR before this message.
    let divider = v.divider_hit(x, y);
    // The scrollbar's hot zone wins over the item under it. Nothing
    // changes visually at this instant: the thumb widens after the WinUI
    // 0.4 s expand delay (set_scrollbar_zone arms it).
    let sb_hit = v.scrollbar_hit(x, y).map(|(h, _)| h);
    let sb_hot = v.set_scrollbar_pointer(sb_hit);
    let hit = if sb_hot || on_control {
        None
    } else {
        v.hit_item(x, y)
    };
    // Infotip: re-arm when the pointer settles on something else (fed the
    // same filtered hit as the hover, so no tip over the scrollbar strip).
    let tip = v.tip_target_at(x, y, hit);
    v.update_tip_target(tip, x, y);
    let header = if divider.is_some() || on_control {
        None
    } else {
        v.header_hit(x, y)
    };
    let header_changed = std::mem::replace(&mut v.header_hover, header) != header;
    // Client-area pointer inside the title row (a control's box) still
    // counts as hovering the row, so the pill / chevron do not drop.
    let in_title = y < v.title_h_px();
    let (item_changed, title_changed) = v.set_hover(hit, in_title);
    if chrome_changed || title_changed {
        let _ = v.redraw_chrome_only();
    }
    if (item_changed || header_changed) && !v.rolled_up {
        let _ = v.redraw_content();
    }
    // Hovering the title row of a rolled fence opens the peek, whether the
    // pointer is on the caption (WM_NCMOUSEMOVE) or on one of the row's
    // client-area controls (chevron box, tabs) handled here.
    if in_title
        && v.rolled_up
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
    window::track_mouse_leave(hwnd);
    Some(0)
}
