//! 网页层 in the fence: the web plugin's page (`crate::web_plugin::WebFence`) lies over the
//! content panel, inside its visual, so it moves, clips, rolls up and fades with the fence. The
//! fence sends it the mouse over the page and its cursor, and gives it the keyboard on a press.

use super::handler::HandlerCtx;
use super::*;
use crate::web_plugin::{TIMER_WEB_INPUT, TIMER_WEB_READY, WebFence};

impl FenceViewState {
    /// The page takes the content panel's place (px in the window's client area) and scale.
    pub(super) fn layout_web(&self) {
        if let Some(web) = &self.web {
            let (w, h) = self.content_panel.size_px();
            let scale = f64::from(self.scale());
            web.set_bounds(self.content_x, self.content_y, w, h, scale);
        }
    }

    /// Whether the page shows: the window is (being) shown and the fence is not rolled up.
    pub(super) fn web_shown(&self, window_visible: bool) {
        if let Some(web) = &self.web {
            web.set_shown(window_visible && !self.rolled_up);
        }
    }

    /// The content panel under a page draws nothing: the page lies on the fence's glass.
    pub(super) fn redraw_web(&mut self) -> Result<()> {
        let ok = self.content_panel.draw(self.dpi, |session, _, _| {
            session.clear(ColorF {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            });
            Ok(())
        })?;
        self.note_draw_result(ok);
        Ok(())
    }
}

impl FenceWindow {
    /// 网页层: the fence shows `web`'s page in place of its content (None: no page).
    pub fn set_web(&self, web: Option<Rc<WebFence>>) {
        let mut old = None;
        self.with_view(|v| {
            if v.web.as_ref().map(Rc::as_ptr) == web.as_ref().map(Rc::as_ptr) {
                return;
            }
            if let Some(o) = v.web.take() {
                v.content_panel.visual.children().remove(&o.visual);
                old = Some(o);
            }
            if let Some(w) = &web {
                v.content_panel.visual.children().insert_at_top(&w.visual);
            }
            v.web = web;
            v.layout_web();
            v.web_shown(desktop::is_visible(v.hwnd));
            let _ = v.redraw_content();
        });
        // Closed with no borrow held: closing the browser may let messages in.
        drop(old);
    }

    /// The plugin whose page the fence shows.
    pub fn web_plugin(&self) -> Option<String> {
        self.web().map(|w| w.plugin.id.clone())
    }

    /// The page the fence shows.
    pub fn web(&self) -> Option<Rc<WebFence>> {
        self.view.borrow().as_ref()?.web.clone()
    }
}

fn web_of(h: &HandlerCtx) -> Option<Rc<WebFence>> {
    h.view.try_borrow().ok()?.as_ref()?.web.clone()
}

/// Mouse messages over the page go to it. A press there also gives it the keyboard; a move goes
/// on to the fence too (its hover, the idle fade).
pub(super) fn on_mouse(
    h: &HandlerCtx,
    hwnd: HWND,
    message: u32,
    wparam: usize,
    lparam: isize,
) -> Option<isize> {
    if !(0x0200..=0x020E).contains(&message) && message != msg::WM_MOUSELEAVE {
        return None;
    }
    let web = web_of(h)?;
    if !web.mouse(message, wparam, lparam) || message == msg::WM_MOUSEMOVE {
        return None;
    }
    if matches!(message, 0x0201 | 0x0204 | 0x0207 | 0x020B) {
        handler::focus_session(&h.view, hwnd);
        web.focus();
    }
    Some(0)
}

pub(super) fn on_setcursor(h: &HandlerCtx, lparam: isize) -> Option<isize> {
    let web = web_of(h)?;
    ((lparam & 0xFFFF) == msg::HTCLIENT && web.set_cursor()).then_some(1)
}

pub(super) fn on_capture_changed(h: &HandlerCtx) {
    if let Some(web) = web_of(h) {
        web.capture_lost();
    }
}

/// `TIMER_WEB_READY` (the browser is made), `TIMER_WEB_INPUT` and `TIMER_WEB_SUSPEND` (hidden
/// for 30 s).
pub(super) fn on_timer(h: &HandlerCtx, wparam: usize) -> Option<isize> {
    if let Some(web) = web_of(h) {
        match wparam {
            TIMER_WEB_READY => web.setup(),
            TIMER_WEB_INPUT => web.pass_mouse_through(),
            _ => web.suspend(),
        }
    }
    Some(0)
}

pub(super) fn on_moved(h: &HandlerCtx) {
    if let Some(web) = web_of(h) {
        web.parent_moved();
    }
}
