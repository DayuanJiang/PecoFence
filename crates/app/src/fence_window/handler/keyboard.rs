//! Keyboard, char, context-menu key and activation / focus messages.

use super::*;
use pecofence_platform::{quicklook, seer};

// Selection colour follows activation (Explorer: accent only in the active
// view). Our own rename popup and a context menu keep the fence "active".
// Another window keeps the selection in grey; a press on the desktop or in
// another fence ends it (`Command::EndItemSelection`).
pub(super) fn on_activate(
    h: &HandlerCtx,
    _hwnd: HWND,
    wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    let inactive = wparam & 0xFFFF == 0;
    let stays = inactive && {
        let other = HWND(lparam as *mut core::ffi::c_void);
        !other.0.is_null() && {
            let cls = desktop::class_name(desktop::root_ancestor(other));
            cls == "PecoFence.Rename" || cls == "#32768"
        }
    };
    if !stays
        && let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
        && v.set_active(!inactive)
    {
        let _ = v.redraw_content();
    }
    None
}

pub(super) fn on_killfocus(
    h: &HandlerCtx,
    _hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
        && std::mem::take(&mut v.focus_visible)
        && !v.rolled_up
    {
        let _ = v.redraw_content();
    }
    None
}

// Keyboard during a focus session (plan §5.12): F2 rename, Enter open,
// Delete or Ctrl+D → shell delete (Shift+Delete = permanent), Esc clears the selection,
// Ctrl+C / X / V clipboard, Ctrl+Shift+N new folder, F5 refresh, Ctrl+Tab, Tab to the next
// fence.
pub(super) fn on_keydown(
    h: &HandlerCtx,
    hwnd: HWND,
    wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, queue, .. } = h;
    let fence_id = h.fence_id;
    let vk = wparam as u32;
    let mut guard = view.borrow_mut();
    let Some(v) = guard.as_mut() else {
        return Some(0);
    };
    v.cancel_pending_rename();
    v.hide_tip();
    let ids = v.selected_ids();
    let mut release = false;
    let shift = window::key_down(msg::VK_SHIFT);
    let ctrl = window::key_down(msg::VK_CONTROL);
    // Any other key ends the wait of an earlier Delete for its icons to go (`UserDelete`);
    // Shift, Ctrl and Alt alone may be the start of the next Delete, whose repeats count too.
    let deletes = vk == msg::VK_DELETE || (vk == VK_D && ctrl && !shift);
    if !deletes && !matches!(vk, msg::VK_SHIFT | msg::VK_CONTROL | msg::VK_MENU) {
        v.forget_user_delete("another key was pressed");
    }
    // F2 while the pointer is on the title row and no icon is selected (none to see, in a
    // rolled-up fence): the name box of the fence, or of the tab under the pointer (#184).
    // With Explorer's single-click open, where pointing is selecting, the title the pointer
    // is on comes before an icon selected earlier.
    let pointing = h.behavior.single_click.get();
    if vk == msg::VK_F2 && v.title_hover && (ids.is_empty() || v.roll_target() || pointing) {
        let pt = window::screen_to_client(hwnd, window::cursor_pos());
        let fence = v
            .tab_at(pt.x, pt.y)
            .and_then(|i| v.tabs.get(i))
            .map_or(v.active, |t| t.id);
        tracing::debug!(
            %fence,
            rolled = v.roll_target(),
            selected = ids.len(),
            pointing,
            "F2 on the title row: fence name box"
        );
        queue.push(Command::EditFenceName(fence));
        return Some(0);
    }
    // A rolled-up fence (from the moment it starts to roll up) shows none of its items: a key
    // must not act on a selection nobody can see (Delete sent a hidden file to the Recycle Bin).
    if v.roll_target() && acts_on_items(wparam as u32, ctrl) {
        tracing::debug!(
            vk = wparam as u32,
            ctrl,
            selected = ids.len(),
            "key ignored: the fence is rolled up, its items are hidden"
        );
        return Some(0);
    }
    const VK_BACK: u32 = 0x08;
    const VK_TAB: u32 = 0x09;
    const VK_PRIOR: u32 = 0x21;
    const VK_NEXT: u32 = 0x22;
    const VK_END: u32 = 0x23;
    const VK_HOME: u32 = 0x24;
    const VK_LEFT: u32 = 0x25;
    const VK_UP: u32 = 0x26;
    const VK_RIGHT: u32 = 0x27;
    const VK_DOWN: u32 = 0x28;
    const VK_A: u32 = 0x41;
    const VK_C: u32 = 0x43;
    const VK_D: u32 = 0x44;
    const VK_F: u32 = 0x46;
    const VK_N: u32 = 0x4E;
    const VK_V: u32 = 0x56;
    const VK_X: u32 = 0x58;
    const VK_Z: u32 = 0x5A;
    const VK_F5: u32 = 0x74;
    const VK_SPACE: u32 = 0x20;
    let mode = cursor_mode(shift, ctrl);
    // Navigation keys are swallowed while the scrollbar thumb / track /
    // arrow is held: the direct manipulation owns the offset (Esc below
    // still cancels the press).
    let sb_held = v.scrollbar_pressed();
    match vk {
        VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN | VK_PRIOR | VK_NEXT | VK_HOME | VK_END if sb_held => {
        }
        VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN => {
            let (dx, dy) = match vk {
                VK_LEFT => (-1, 0),
                VK_RIGHT => (1, 0),
                VK_UP => (0, -1),
                _ => (0, 1),
            };
            v.move_cursor(dx, dy, mode);
            if let Some(path) = v.preview_path() {
                seer::switch(path);
                quicklook::switch(path);
            }
            let _ = v.redraw_content();
        }
        VK_SPACE if ctrl => {
            v.toggle_focused();
            let _ = v.redraw_content();
        }
        VK_PRIOR | VK_NEXT if !v.items.is_empty() => {
            v.page_cursor(if vk == VK_PRIOR { -1 } else { 1 }, mode);
            let _ = v.redraw_content();
        }
        VK_HOME | VK_END if !v.items.is_empty() => {
            let target = if vk == VK_HOME { 0 } else { v.items.len() - 1 };
            v.cursor_to(target, mode);
            let _ = v.redraw_content();
        }
        VK_A if ctrl => {
            v.selected = (0..v.items.len()).collect();
            v.focus_visible = v.anchor_index.is_some();
            let _ = v.redraw_content();
        }
        VK_C | VK_X if ctrl => {
            if !ids.is_empty() {
                queue.push(Command::ClipboardVerb {
                    fence: v.active,
                    items: ids,
                    cut: vk == VK_X,
                });
            }
        }
        VK_V if ctrl => queue.push(Command::Paste { fence: v.active }),
        VK_F if ctrl => queue.push(Command::OpenFenceFilter(v.active)),
        // Ctrl+Z undoes the last fence move, resize, icon move or fence delete (app/undo.rs);
        // a held key undoes once.
        VK_Z if ctrl && !shift && lparam & (1 << 30) == 0 => queue.push(Command::Undo),
        VK_N if ctrl && shift => queue.push(Command::NewFolder { fence: v.active }),
        VK_TAB if ctrl && v.tabs.len() > 1 => {
            // Ctrl+Tab / Ctrl+Shift+Tab cycle the tabs (Explorer).
            let n = v.tabs.len();
            let cur = v.active_index();
            let next = if shift {
                (cur + n - 1) % n
            } else {
                (cur + 1) % n
            };
            queue.push(Command::SwitchTab {
                host: fence_id,
                tab: v.tabs[next].id,
            });
        }
        // Tab / Shift+Tab: the keyboard moves to the next / previous fence in reading order
        // (Explorer's Tab between panes). Ctrl+Tab stays with the tabs above.
        VK_TAB if !ctrl => queue.push(Command::FocusNextFence {
            from: hwnd,
            back: shift,
        }),
        VK_F5 => queue.push(Command::RefreshFence(v.active)),
        k if k == msg::VK_F2 => {
            // Several selected: the focused one is renamed, as in Explorer (the first of them
            // when the focus is on none).
            let focused = v
                .anchor_index
                .filter(|i| v.selected.contains(i))
                .and_then(|i| v.items.get(i))
                .map(|it| it.id);
            if let Some(item) = focused.or_else(|| ids.first().copied()) {
                tracing::debug!(
                    selected = ids.len(),
                    focused = focused.is_some(),
                    "F2 rename"
                );
                queue.push(Command::RenameItem {
                    fence: v.active,
                    item,
                });
            }
        }
        // During the Peek (the dimmer passed the key on), as Enter in the search box does: the
        // Peek ends and the items open, folders in Explorer (opened inside the fence, a folder
        // would sit behind the windows the Peek uncovered).
        k if k == msg::VK_RETURN && h.behavior.floating.get() && !ids.is_empty() => {
            queue.push(Command::EndPeek("items opened with Enter"));
            for item in ids {
                queue.push(Command::LaunchItem {
                    item,
                    from: v.active,
                    inside: false,
                });
            }
        }
        k if k == msg::VK_RETURN => open_items(v, queue, ids),
        VK_BACK if v.deco.up_button => {
            // Keyboard navigation: the folder it returns to shows the focus ring.
            v.focus_visible = true;
            queue.push(Command::PortalUp(v.active));
        }
        // Ctrl+D deletes to the Recycle Bin, as on the desktop and in Explorer. A held key
        // deletes once: its repeats (bit 30, the key was already down) asked the shell again
        // for the same files (feedback #108).
        k if k == msg::VK_DELETE || (k == VK_D && ctrl && !shift) => {
            if lparam & (1 << 30) != 0 {
                tracing::debug!("delete key repeat ignored");
            } else if !ids.is_empty() {
                // From the keyboard: the neighbour that takes the focus shows the focus ring.
                v.user_delete = Some(UserDelete::new(ids.iter().copied().collect(), true));
                queue.push(Command::DeleteItems {
                    fence: v.active,
                    items: ids,
                    permanent: shift,
                });
            }
        }
        k if k == msg::VK_ESCAPE => {
            // During a tab drag / tear-off Esc only cancels the drag (an OLE
            // drag's Esc leaves the selection alone too).
            if let Some(c) = v.cancel_tab_or_remote_drag() {
                drop(guard);
                finish_drag_cancel(c, &h.view, queue, &h.behavior);
                return Some(0);
            }
            // A divider drag: the column gets its width back and the selection stays
            // (Explorer).
            if let Some(cd) = v.col_drag.take() {
                v.set_column_width(cd.col, cd.start_w);
                let _ = v.redraw_content();
                drop(guard);
                window::release_capture();
                return Some(0);
            }
            // A filtered fence: Esc clears the filter first and leaves the selection, which
            // the next Esc clears.
            if v.filter_active {
                tracing::debug!(selected = ids.len(), "Esc: filter cleared, selection kept");
                queue.push(Command::ClearFenceFilter(v.active));
            } else {
                v.selected.clear();
                v.type_ahead.clear();
            }
            seer::close();
            quicklook::close();
            if !h.behavior.selection.borrow().is_empty() {
                queue.push(Command::ClearFenceSelection);
            }
            release = v.end_marquee()
                || v.drag.take().is_some()
                || v.end_scrollbar_press()
                || v.clear_pressed().is_some();
            let _ = v.redraw_content();
        }
        _ => return None,
    }
    drop(guard);
    if release {
        window::release_capture();
    }
    Some(0)
}

// Type-to-select: printable characters accumulate into a prefix (control
// codes — Ctrl+letter, Enter, Esc, Backspace, Tab — were handled as keys).
pub(super) fn on_char(h: &HandlerCtx, _hwnd: HWND, wparam: usize, lparam: isize) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    let code = wparam as u32;
    if code < 0x20 || code == 0x7F {
        return None;
    }
    // Ctrl+Space toggles the focused item (WM_KEYDOWN); not a type-ahead space.
    if code == 0x20 && window::key_down(msg::VK_CONTROL) {
        return Some(0);
    }
    let Some(ch) = char::from_u32(code) else {
        return Some(0);
    };
    let mut guard = view.borrow_mut();
    let Some(v) = guard.as_mut() else {
        return Some(0);
    };
    // Space outside a prefix opens or closes the preview of the focused item in Seer or
    // QuickLook, whichever runs (Seer when both do), as on the desktop and in Explorer (nothing
    // without them); a held Space does not repeat it.
    if ch == ' ' && !v.typing_ahead() && !window::key_down(msg::VK_SHIFT) {
        if lparam & (1 << 30) == 0
            && let Some(path) = v.preview_path()
            && !seer::toggle(path)
        {
            quicklook::toggle(path);
        }
        return Some(0);
    }
    if v.type_ahead_char(ch) {
        let _ = v.redraw_content();
    }
    Some(0)
}

// Alt+Enter = 属性, Alt+Up = portal up, Alt+Left / Alt+Right = back / forward as in Explorer
// (back is up, as the mouse's back button; bit 29 of lParam: Alt was down).
pub(super) fn on_syskeydown(
    h: &HandlerCtx,
    _hwnd: HWND,
    wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, queue, .. } = h;
    const VK_LEFT: u32 = 0x25;
    const VK_UP: u32 = 0x26;
    const VK_RIGHT: u32 = 0x27;
    let vk = wparam as u32;
    let alt = (lparam >> 29) & 1 == 1;
    if !alt {
        return None;
    }
    let mut guard = view.borrow_mut();
    let v = guard.as_mut()?;
    if vk == msg::VK_RETURN {
        let ids = v.selected_ids();
        // Not for the hidden items of a rolled-up fence (see `acts_on_items`).
        if !ids.is_empty() && !v.roll_target() {
            queue.push(Command::ItemProperties {
                fence: v.active,
                items: ids,
            });
        }
        return Some(0);
    }
    if (vk == VK_UP || vk == VK_LEFT) && v.deco.up_button {
        v.focus_visible = true;
        queue.push(Command::PortalUp(v.active));
        return Some(0);
    }
    // Like the mouse's forward button: the app goes nowhere when no up came before.
    if vk == VK_RIGHT {
        v.focus_visible = true;
        queue.push(Command::PortalForward(v.active));
        return Some(0);
    }
    None
}

// Menu key / Shift+F10: DefWindowProc turns them into WM_CONTEXTMENU with
// lParam = -1 (mouse right-clicks never reach here: WM_RBUTTONUP is consumed).
pub(super) fn on_contextmenu(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, queue, .. } = h;
    if lparam != -1 && lparam as u32 != u32::MAX {
        return Some(0);
    }
    let mut guard = view.borrow_mut();
    let Some(v) = guard.as_mut() else {
        return Some(0);
    };
    let ids = v.selected_ids();
    // A rolled-up fence shows no items: the Menu key opens the fence menu there.
    if ids.is_empty() || v.roll_target() {
        // Background menu at the content's top-left.
        let win = window::window_rect(hwnd);
        let pad = (8.0 * v.scale()) as i32;
        queue.push(Command::FenceMenu {
            fence: v.active,
            x: win.left + v.content_x + pad,
            y: win.top + v.content_y + pad,
        });
        return Some(0);
    }
    // Anchor at the focused item's label (Explorer: bottom-left of the item).
    let idx = v
        .anchor_index
        .filter(|i| v.selected.contains(i))
        .or_else(|| v.selected.iter().min().copied())
        .unwrap_or(0);
    // The menu anchors at the label's final position: no glide here.
    v.scroll_into_view(idx, false);
    let _ = v.redraw();
    let item = v.items[idx].id;
    let (x, y) = match v.item_label_rect(item) {
        Some(r) => (r.left, r.bottom),
        None => {
            let p = window::cursor_pos();
            (p.x, p.y)
        }
    };
    queue.push(Command::ItemMenu {
        fence: v.active,
        items: ids,
        x,
        y,
    });
    Some(0)
}
