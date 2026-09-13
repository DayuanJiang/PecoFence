//! Window procedure: `HandlerCtx` (the closure state) and the message dispatch; the per-message bodies live in the child modules.

use super::*;

mod keyboard;
mod misc;
mod mouse;
mod mouse_move;
mod nc;
mod timers;

/// Grants the fence the focus session (SetForegroundWindow) and reconciles the optimistic
/// accent the click already painted: a WS_EX_NOACTIVATE tool window the system refuses to
/// activate never receives WM_ACTIVATE, so without this check its selection would stay in
/// the accent while another window (or another fence) is the active one. Call with no borrow
/// of `view` held: WM_ACTIVATE re-enters the window procedure.
pub(super) fn focus_session(view: &RefCell<Option<FenceViewState>>, hwnd: HWND) {
    window::bring_to_front(hwnd);
    if desktop::foreground_window() == hwnd {
        return;
    }
    if let Ok(mut guard) = view.try_borrow_mut()
        && let Some(v) = guard.as_mut()
        && v.set_active(false)
    {
        let _ = v.redraw_content();
    }
}

/// Second half of a cancelled tab / tear-off drag, with no borrow held: releases the capture
/// (WM_CAPTURECHANGED then finds no drag to commit) and hands a torn-off fence back to its host.
pub(super) fn finish_drag_cancel(cancel: DragCancel, queue: &CommandQueue) {
    window::release_capture();
    let clear_hint = |hinted| {
        if hinted {
            queue.push(Command::MergeHint {
                target: HWND(std::ptr::null_mut()),
                x: 0,
            });
        }
    };
    match cancel {
        DragCancel::Remote { change, hinted } => {
            clear_hint(hinted);
            queue.push(Command::CancelDetach { change });
        }
        DragCancel::Window { hwnd, rect, hinted } => {
            clear_hint(hinted);
            let _ = window::set_window_bounds(
                hwnd,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
            );
        }
        DragCancel::Tab => {}
    }
}

/// State of one fence window's message handler: what the window-procedure closure used to
/// capture. One instance is moved into the boxed closure; every per-message function receives
/// it as `h`.
pub(super) struct HandlerCtx {
    pub(super) fence_id: FenceId,
    pub(super) view: ViewCell,
    pub(super) anchor: AnchorCell,
    pub(super) taskbar_created: u32,
    pub(super) di_getdragimage: u32,
    pub(super) queue: CommandQueue,
    pub(super) behavior: Rc<Behavior>,
    /// True while TIMER_PEEK_OPEN is armed: WM_NCMOUSEMOVE must not keep restarting it.
    pub(super) peek_armed: Cell<bool>,
    /// True between WM_ENTERMENULOOP and WM_EXITMENULOOP: one of our own context menus is
    /// up (TrackPopupMenuEx with this window as owner pumps WM_TIMER to us meanwhile). A
    /// hover peek stays expanded under its own popup and no new peek arms.
    pub(super) menu_open: Cell<bool>,
    /// True between WM_ENTERSIZEMOVE and WM_EXITSIZEMOVE: hover-peek must not start/stop while
    /// the modal move/size loop owns the window rect (it dispatches WM_TIMER to us).
    pub(super) in_size_move: Cell<bool>,
    /// Window rect when the modal loop began: an unchanged size at exit = the loop was a
    /// move, and a move that ends over another fence's title merges into it (Fences
    /// drag-to-merge); an unchanged full rect = the move was cancelled (Esc restores it
    /// exactly while the cursor stays where it is) and must not merge.
    pub(super) rect_at_enter: Cell<RECT>,
    /// Original caption press carried by SC_MOVE. The cursor can already be farther along
    /// the drag by the time Windows enters its modal move loop.
    pub(super) move_start: Cell<Option<(i32, i32)>>,
    /// Cursor position relative to the window's top-left when the move began. WM_MOVING
    /// rebuilds the rect from it every frame, so a snap displacement never accumulates (the
    /// system would otherwise keep adding mouse deltas to the *snapped* rect and the window
    /// would trail the cursor by the snap distance for the rest of the drag).
    pub(super) grab_offset: Cell<(i32, i32)>,
    pub(super) merge_target: Cell<isize>,
    /// Pointer screen x last reported with the merge hint (the target strip opens its
    /// insertion gap at that slot; re-sent only when the slot could have changed).
    pub(super) merge_x: Cell<i32>,
}

impl FenceWindow {
    pub(super) fn make_handler(
        fctx: &FenceContext,
        fence_id: FenceId,
        view: Rc<RefCell<Option<FenceViewState>>>,
    ) -> MessageHandler {
        let ctx = HandlerCtx {
            fence_id,
            view,
            anchor: fctx.anchor.clone(),
            taskbar_created: fctx.taskbar_created,
            di_getdragimage: dragdrop::di_getdragimage_msg(),
            queue: fctx.queue.clone(),
            behavior: fctx.behavior.clone(),
            peek_armed: Cell::new(false),
            menu_open: Cell::new(false),
            in_size_move: Cell::new(false),
            rect_at_enter: Cell::new(RECT::default()),
            move_start: Cell::new(None),
            grab_offset: Cell::new((0, 0)),
            merge_target: Cell::new(0isize),
            merge_x: Cell::new(i32::MIN),
        };
        let trace_test_input = cfg!(debug_assertions)
            && pecofence_core::brand::var_os("PECOFENCE_UI_TEST_WINDOWS").is_some();
        Box::new(
            move |hwnd: HWND, message: u32, wparam: usize, lparam: isize| -> Option<isize> {
                let h = &ctx;
                if trace_test_input
                    && matches!(
                        message,
                        msg::WM_LBUTTONDOWN
                            | msg::WM_LBUTTONUP
                            | msg::WM_NCLBUTTONDOWN
                            | WM_NCLBUTTONUP
                            | msg::WM_ENTERSIZEMOVE
                            | msg::WM_EXITSIZEMOVE
                    )
                {
                    tracing::info!(
                        target: "pecofence::input_audit",
                        ?hwnd, message, wparam, lparam,
                        "native pointer event"
                    );
                }
                match message {
                    msg::WM_MOUSEACTIVATE => Some(msg::MA_NOACTIVATE),
                    // Alt+F4 / SC_CLOSE while the fence is the active window: fences are
                    // furniture, they never close (DefWindowProc would DestroyWindow).
                    msg::WM_CLOSE => Some(0),
                    msg::WM_ERASEBKGND => Some(1),
                    msg::WM_NCCALCSIZE => Some(0),
                    // Both materials draw their own frame. With DWM nonclient rendering
                    // disabled, DefWindowProc would otherwise paint a classic square frame.
                    msg::WM_NCPAINT => Some(0),
                    msg::WM_NCACTIVATE if h.behavior.backdrop.get() == BackdropMode::Acrylic => {
                        nc::on_ncactivate_acrylic(h, hwnd, wparam, lparam)
                    }
                    msg::WM_WINDOWPOSCHANGING => nc::on_windowposchanging(h, hwnd, wparam, lparam),
                    msg::WM_NCHITTEST => nc::on_nchittest(h, hwnd, wparam, lparam),
                    msg::WM_SETCURSOR => mouse::on_setcursor(h, hwnd, wparam, lparam),
                    m if m == h.di_getdragimage && m != 0 => {
                        misc::on_getdragimage(h, hwnd, wparam, lparam)
                    }
                    msg::WM_NCMOUSEMOVE => nc::on_ncmousemove(h, hwnd, wparam, lparam),
                    msg::WM_NCMOUSELEAVE => nc::on_ncmouseleave(h, hwnd, wparam, lparam),
                    msg::WM_ACTIVATE => keyboard::on_activate(h, hwnd, wparam, lparam),
                    msg::WM_KILLFOCUS => keyboard::on_killfocus(h, hwnd, wparam, lparam),
                    msg::WM_NCLBUTTONDOWN => nc::on_nclbuttondown(h, hwnd, wparam, lparam),
                    WM_NCLBUTTONUP => nc::on_nclbuttonup(h, hwnd, wparam, lparam),
                    msg::WM_NCLBUTTONDBLCLK => nc::on_nclbuttondblclk(h, hwnd, wparam, lparam),
                    msg::WM_NCRBUTTONUP => nc::on_ncrbuttonup(h, hwnd, wparam, lparam),
                    msg::WM_MOUSEMOVE => mouse_move::on_mousemove(h, hwnd, wparam, lparam),
                    msg::WM_MOUSELEAVE => mouse::on_mouseleave(h, hwnd, wparam, lparam),
                    msg::WM_LBUTTONDOWN => mouse::on_lbuttondown(h, hwnd, wparam, lparam),
                    msg::WM_LBUTTONUP => mouse::on_lbuttonup(h, hwnd, wparam, lparam),
                    msg::WM_CAPTURECHANGED => mouse::on_capturechanged(h, hwnd, wparam, lparam),
                    msg::WM_KEYDOWN => keyboard::on_keydown(h, hwnd, wparam, lparam),
                    msg::WM_CHAR => keyboard::on_char(h, hwnd, wparam, lparam),
                    msg::WM_SYSKEYDOWN => keyboard::on_syskeydown(h, hwnd, wparam, lparam),
                    msg::WM_CONTEXTMENU => keyboard::on_contextmenu(h, hwnd, wparam, lparam),
                    msg::WM_LBUTTONDBLCLK => mouse::on_lbuttondblclk(h, hwnd, wparam, lparam),
                    msg::WM_RBUTTONDOWN => mouse::on_rbuttondown(h, hwnd, wparam, lparam),
                    msg::WM_RBUTTONUP => mouse::on_rbuttonup(h, hwnd, wparam, lparam),
                    msg::WM_MOUSEWHEEL => mouse::on_mousewheel(h, hwnd, wparam, lparam),
                    msg::WM_SIZE => nc::on_size(h, hwnd, wparam, lparam),
                    msg::WM_MOVE => nc::on_move(h, hwnd, wparam, lparam),
                    msg::WM_SHOWWINDOW => misc::on_showwindow(h, hwnd, wparam, lparam),
                    WM_APP_SET_VISIBLE => misc::on_app_set_visible(h, hwnd, wparam, lparam),
                    WM_APP_TAB_SWAP_DONE => misc::on_app_tab_swap_done(h, hwnd, wparam, lparam),
                    WM_APP_FADE_DONE => misc::on_app_fade_done(h, hwnd, wparam, lparam),
                    msg::WM_ENTERSIZEMOVE => nc::on_entersizemove(h, hwnd, wparam, lparam),
                    msg::WM_EXITSIZEMOVE => nc::on_exitsizemove(h, hwnd, wparam, lparam),
                    msg::WM_TIMER if wparam == TIMER_RENAME => {
                        timers::on_timer_rename(h, hwnd, wparam, lparam)
                    }
                    msg::WM_TIMER
                        if wparam == TIMER_VISIBILITY_FINISH || wparam == TIMER_CONTENT_FINISH =>
                    {
                        timers::on_timer_composition_finish(h, hwnd, wparam, lparam)
                    }
                    msg::WM_TIMER if wparam == TIMER_TIP => {
                        timers::on_timer_tip(h, hwnd, wparam, lparam)
                    }
                    msg::WM_TIMER if wparam == TIMER_TIP_HIDE => {
                        timers::on_timer_tip_hide(h, hwnd, wparam, lparam)
                    }
                    msg::WM_TIMER if wparam == TIMER_SCROLL_REPEAT => {
                        timers::on_timer_scroll_repeat(h, hwnd, wparam, lparam)
                    }
                    msg::WM_TIMER if wparam == TIMER_SCROLLBAR_EXPAND => {
                        timers::on_timer_scrollbar_expand(h, hwnd, wparam, lparam)
                    }
                    msg::WM_TIMER if wparam == TIMER_SCROLLBAR_CONTRACT => {
                        timers::on_timer_scrollbar_contract(h, hwnd, wparam, lparam)
                    }
                    msg::WM_TIMER if wparam == TIMER_PEEK_OPEN => {
                        timers::on_timer_peek_open(h, hwnd, wparam, lparam)
                    }
                    msg::WM_TIMER if wparam == TIMER_DRAG_PEEK => {
                        timers::on_timer_drag_peek(h, hwnd, wparam, lparam)
                    }

                    msg::WM_TIMER if wparam == TIMER_SCROLLBAR => {
                        timers::on_timer_scrollbar(h, hwnd, wparam, lparam)
                    }
                    msg::WM_TIMER if wparam == TIMER_SHADOW => {
                        timers::on_timer_shadow(h, hwnd, wparam, lparam)
                    }
                    msg::WM_ENTERMENULOOP => timers::on_entermenuloop(h, hwnd, wparam, lparam),
                    msg::WM_EXITMENULOOP => timers::on_exitmenuloop(h, hwnd, wparam, lparam),
                    msg::WM_TIMER if wparam == TIMER_PEEK_CLOSE => {
                        timers::on_timer_peek_close(h, hwnd, wparam, lparam)
                    }
                    msg::WM_SYSCOMMAND => nc::on_syscommand(h, hwnd, wparam, lparam),
                    msg::WM_SIZING => nc::on_sizing(h, hwnd, wparam, lparam),
                    msg::WM_MOVING => nc::on_moving(h, hwnd, wparam, lparam),
                    msg::WM_DPICHANGED => nc::on_dpichanged(h, hwnd, wparam, lparam),
                    m if m == h.taskbar_created && m != 0 => {
                        misc::on_taskbar_created(h, hwnd, wparam, lparam)
                    }
                    msg::WM_DESTROY => misc::on_destroy(h, hwnd, wparam, lparam),
                    _ => None,
                }
            },
        )
    }
}
