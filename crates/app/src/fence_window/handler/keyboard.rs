//! Keyboard, char, context-menu key and activation / focus messages.

use super::*;

// Selection colour follows activation (Explorer: accent only in the active
// view). Our own rename popup and a context menu keep the fence "active".
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
// Delete → shell delete (Shift+Delete = permanent), Esc clears the selection,
// Ctrl+C / X / V clipboard, Ctrl+Shift+N new folder, F5 refresh, Ctrl+Tab.
pub(super) fn on_keydown(
    h: &HandlerCtx,
    _hwnd: HWND,
    wparam: usize,
    _lparam: isize,
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
    const VK_N: u32 = 0x4E;
    const VK_V: u32 = 0x56;
    const VK_X: u32 = 0x58;
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
        VK_F5 => queue.push(Command::RefreshFence(v.active)),
        k if k == msg::VK_F2 => {
            if let [item] = ids[..] {
                queue.push(Command::RenameItem {
                    fence: v.active,
                    item,
                });
            }
        }
        k if k == msg::VK_RETURN => {
            let folder = (v.navigate_folders && ids.len() == 1)
                .then(|| {
                    v.items
                        .iter()
                        .find(|it| it.id == ids[0] && it.is_folder)
                        .map(|it| it.path.clone())
                })
                .flatten();
            match folder {
                Some(path) => queue.push(Command::PortalEnter {
                    fence: v.active,
                    path,
                }),
                None => {
                    for id in ids {
                        queue.push(Command::LaunchItem(id));
                    }
                }
            }
        }
        VK_BACK if v.deco.up_button => {
            queue.push(Command::PortalUp(v.active));
        }
        k if k == msg::VK_DELETE => {
            if !ids.is_empty() {
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
                finish_drag_cancel(c, queue);
                return Some(0);
            }
            v.selected.clear();
            v.type_ahead.clear();
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
pub(super) fn on_char(h: &HandlerCtx, _hwnd: HWND, wparam: usize, _lparam: isize) -> Option<isize> {
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
    if v.type_ahead_char(ch) {
        let _ = v.redraw_content();
    }
    Some(0)
}

// Alt+Enter = 属性, Alt+Up = portal up (bit 29 of lParam: Alt was down).
pub(super) fn on_syskeydown(
    h: &HandlerCtx,
    _hwnd: HWND,
    wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, queue, .. } = h;
    const VK_UP: u32 = 0x26;
    let vk = wparam as u32;
    let alt = (lparam >> 29) & 1 == 1;
    if !alt {
        return None;
    }
    let guard = view.borrow();
    let v = guard.as_ref()?;
    if vk == msg::VK_RETURN {
        let ids = v.selected_ids();
        if !ids.is_empty() {
            queue.push(Command::ItemProperties {
                fence: v.active,
                items: ids,
            });
        }
        return Some(0);
    }
    if vk == VK_UP && v.deco.up_button {
        queue.push(Command::PortalUp(v.active));
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
    if ids.is_empty() {
        // Background menu at the content's top-left.
        let win = window::window_rect(hwnd);
        let pad = (8.0 * v.scale()) as i32;
        queue.push(Command::FenceMenu {
            fence: v.active,
            x: win.left + pad,
            y: win.top + v.title_h_px() + pad,
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
