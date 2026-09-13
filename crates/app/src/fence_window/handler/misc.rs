//! Lifecycle, app-posted and shell messages.

use super::*;

// SHDoDragDrop asks the source window for its drag image before the drag
// starts: answer with the selected items rendered as they are on screen so
// the translucent copy lifts off from their own places (Explorer's DefView).
// 0 = the shell falls back to its generic image.
pub(super) fn on_getdragimage(
    h: &HandlerCtx,
    _hwnd: HWND,
    _wparam: usize,
    lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    if lparam == 0 {
        return Some(0);
    }
    let image = {
        let Ok(guard) = view.try_borrow() else {
            return Some(0);
        };
        match guard.as_ref() {
            Some(v) if v.ole_drag => v.render_drag_image(),
            _ => None,
        }
    };
    match image {
        // SAFETY: lparam is the SHDRAGIMAGE* of this DI_GETDRAGIMAGE message.
        Some(img) => Some(unsafe { dragdrop::answer_drag_image(lparam, &img) } as isize),
        None => Some(0),
    }
}

pub(super) fn on_showwindow(
    h: &HandlerCtx,
    hwnd: HWND,
    wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    // State under the borrow; the shadow's ShowWindow after it.
    let (shadow, show) = {
        let mut guard = view.borrow_mut();
        let v = guard.as_mut()?;
        if wparam != 0 {
            // A fade-in shows the shadow itself (`show_with_fade`, at
            // alpha 0, fading with the plate).
            if v.vis_fade == Some(VisFade::Showing) {
                return None;
            }
            v.reset_root();
            (v.shadow.hwnd(), true)
        } else {
            // Hidden (possibly from outside, mid-fade): a stale completion
            // must not show the shadow of a hidden window — and a retiring
            // window whose exit fade was cut short still has to be dropped.
            if v.vis_fade.take() == Some(VisFade::Hiding { destroy: true }) {
                v.queue.push(Command::FadeOutDone(hwnd));
            }
            // Ready for a plain ShowWindow later: opaque, unscaled,
            // shadow alpha back at 255 (same rest as the fade-done path).
            v.reset_root();
            (v.shadow.hwnd(), false)
        }
    };
    if show {
        show_shadow(shadow, hwnd);
    } else {
        desktop::hide_window(shadow);
    }
    None
}

pub(super) fn on_app_set_visible(
    h: &HandlerCtx,
    hwnd: HWND,
    wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    // Desktop anchor (quick hide / show, Peek): fade rather than snap.
    if wparam != 0 {
        show_with_fade(view, hwnd, false);
    } else {
        hide_with_fade(view, hwnd, false, false);
    }
    Some(1)
}

pub(super) fn on_app_tab_swap_done(
    h: &HandlerCtx,
    _hwnd: HWND,
    wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    if let Some(v) = view.borrow_mut().as_mut()
        && v.swap_gen == wparam
    {
        v.finish_content_swap_now();
    }
    Some(0)
}

pub(super) fn on_app_fade_done(
    h: &HandlerCtx,
    hwnd: HWND,
    wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { view, .. } = h;
    // Compositor batch completed (posted from its callback thread). State
    // under the borrow; ShowWindow on the fence / shadow after it.
    let (shadow, hide) = {
        let mut guard = view.borrow_mut();
        let Some(v) = guard.as_mut() else {
            return Some(0);
        };
        if v.vis_gen != wparam {
            return Some(0);
        }
        window::kill_timer(hwnd, TIMER_VISIBILITY_FINISH);
        v.vis_deadline = None;
        match v.vis_fade.take() {
            None => return Some(0),
            Some(VisFade::Showing) => {
                v.reset_root();
                (v.shadow.hwnd(), None)
            }
            Some(VisFade::Hiding { destroy }) => {
                (v.shadow.hwnd(), Some((destroy, v.queue.clone())))
            }
        }
    };
    let Some((destroy, queue)) = hide else {
        if desktop::is_visible(hwnd) {
            show_shadow(shadow, hwnd);
        }
        return Some(0);
    };
    desktop::hide_window(hwnd);
    desktop::hide_window(shadow);
    if let Some(v) = view.borrow_mut().as_mut() {
        // Ready for a plain ShowWindow from the anchor later on.
        v.reset_root();
    }
    if destroy {
        queue.push(Command::FadeOutDone(hwnd));
    }
    Some(0)
}

pub(super) fn on_taskbar_created(
    h: &HandlerCtx,
    _hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { anchor, .. } = h;
    if let Ok(mut guard) = anchor.try_borrow_mut()
        && let Some(a) = guard.as_mut()
    {
        a.reanchor("TaskbarCreated");
    }
    Some(0)
}

pub(super) fn on_destroy(
    h: &HandlerCtx,
    hwnd: HWND,
    _wparam: usize,
    _lparam: isize,
) -> Option<isize> {
    let HandlerCtx { anchor, .. } = h;
    if let Ok(mut guard) = anchor.try_borrow_mut()
        && let Some(a) = guard.as_mut()
    {
        a.unregister_fence(hwnd);
    }
    Some(0)
}
