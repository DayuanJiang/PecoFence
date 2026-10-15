//! The Peek search box: typing during a Peek opens it in the upper middle of the pointer's
//! screen. The
//! fences then dim every icon whose name does not contain the text and highlight the matches
//! (`App::update_peek_search`). Up / Down step through the matches, Enter opens the highlighted
//! one, Esc ends the Peek.
//!
//! A small activatable popup hosting an EDIT control, like the rename popup, so IME input,
//! the caret and clipboard keys work as in any text box. The popup paints its background, the
//! search glyph and the match count itself.

use crate::commands::{Command, CommandQueue};
use pecofence_platform::paint::{self, Align, Font};
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, RECT, desktop, dwm, edit, fonts, monitors, msg};
use std::cell::RefCell;

pub const SEARCH_CLASS: &str = "PecoFence.PeekSearch";
const WA_INACTIVE: usize = 0;
const VK_UP: u32 = 0x26;
const VK_DOWN: u32 = 0x28;
/// Search in Segoe Fluent Icons / Segoe MDL2 Assets.
const GLYPH: &str = "\u{E721}";

// Layout in DIPs.
const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 56.0;
/// The box sits this far down the free height of the work area (0 = top, 0.5 = centre).
const TOP_FRACTION: f32 = 0.3;
const PAD: f32 = 18.0;
const GLYPH_W: f32 = 30.0;
const GLYPH_SIZE: f32 = 20.0;
const STATUS_W: f32 = 170.0;
const STATUS_SIZE: f32 = 15.0;
const TEXT_SIZE: f32 = 20.0;
const EDIT_H: f32 = 28.0;

thread_local! {
    static ACTIVE: RefCell<Option<Session>> = const { RefCell::new(None) };
    static CLASS: RefCell<Option<WindowClass>> = const { RefCell::new(None) };
}

struct Session {
    window: Window,
    edit: edit::EditControl,
    queue: CommandQueue,
    dpi: u32,
    /// Glyph and match count colour (COLORREF).
    dim_color: u32,
    glyph_font: Font,
    status_font: Font,
    /// Match count shown at the right ("2 / 5", or that nothing matches).
    status: String,
}

/// Opens the box with `text` typed in and gives it the keyboard.
pub fn open(text: &str, queue: CommandQueue, dark: bool) {
    close();
    CLASS.with(|c| {
        if c.borrow().is_none() {
            *c.borrow_mut() = WindowClass::register(SEARCH_CLASS, ClassOptions::default()).ok();
        }
    });
    // On the screen the pointer is on, where the user is looking.
    let pt = window::cursor_pos();
    let Some(m) = monitors::query(monitors::monitor_from_point(pt.x, pt.y)) else {
        return;
    };
    let dpi = m.dpi.max(96);
    let px = |dip: f32| (dip * dpi as f32 / 96.0).round() as i32;
    let (w, h) = (px(WIDTH), px(HEIGHT));
    let wa = m.work_area;
    let x = (wa.left + wa.right) / 2 - w / 2;
    let y = wa.top + ((wa.bottom - wa.top - h) as f32 * TOP_FRACTION) as i32;

    let handler: MessageHandler = {
        let queue = queue.clone();
        Box::new(
            move |hwnd: HWND, message: u32, wparam: usize, lparam: isize| -> Option<isize> {
                match message {
                    msg::WM_COMMAND if msg::hi_u16(wparam) == edit::EN_CHANGE => {
                        queue.push(Command::PeekSearchChanged);
                        Some(0)
                    }
                    edit::WM_CTLCOLOREDIT_MSG => ACTIVE.with(|a| {
                        a.try_borrow()
                            .ok()?
                            .as_ref()
                            .map(|s| s.edit.ctl_color(wparam))
                    }),
                    // Activation focuses the popup itself: the keyboard belongs to the edit.
                    msg::WM_SETFOCUS => {
                        ACTIVE.with(|a| {
                            if let Ok(a) = a.try_borrow()
                                && let Some(s) = a.as_ref()
                            {
                                s.edit.focus();
                            }
                        });
                        Some(0)
                    }
                    msg::WM_ERASEBKGND => Some(1),
                    msg::WM_PAINT => {
                        paint_box(hwnd);
                        Some(0)
                    }
                    msg::WM_ACTIVATE if wparam & 0xFFFF == WA_INACTIVE => {
                        let other = HWND(lparam as *mut core::ffi::c_void);
                        if !crate::peek::keeps_peek(other) {
                            queue.push(Command::EndPeek("search box lost the focus"));
                        }
                        None
                    }
                    msg::WM_CLOSE => {
                        queue.push(Command::EndPeek("search box closed"));
                        Some(0)
                    }
                    _ => None,
                }
            },
        )
    };
    let created = CLASS.with(|c| {
        let c = c.borrow();
        let class = c.as_ref()?;
        WindowBuilder::new(class)
            .title("PecoFence search")
            .style(style::POPUP | style::CLIPCHILDREN)
            .ex_style(style::EX_TOOLWINDOW | style::EX_TOPMOST)
            .bounds(x, y, w, h)
            .create(handler)
            .ok()
    });
    let Some(window) = created else { return };
    let _ = dwm::set_corner_preference(window.hwnd(), dwm::CornerPreference::Round);
    let edit_x = px(PAD + GLYPH_W + 8.0);
    let edit_w = w - edit_x - px(STATUS_W + PAD + 8.0);
    let Ok(ctl) = edit::EditControl::create(
        window.hwnd(),
        edit_w,
        px(EDIT_H),
        dpi,
        TEXT_SIZE,
        text,
        dark,
        on_key,
    ) else {
        return;
    };
    ctl.set_bounds(edit_x, (h - px(EDIT_H)) / 2, edit_w, px(EDIT_H));
    let hwnd = window.hwnd();
    ACTIVE.with(|a| {
        *a.borrow_mut() = Some(Session {
            window,
            edit: ctl,
            queue,
            dpi,
            dim_color: if dark { 0x00C5_C5C5 } else { 0x005F_5F5F },
            glyph_font: Font::new(fonts::icon_face(), GLYPH_SIZE, dpi),
            status_font: Font::new(fonts::text_face(), STATUS_SIZE, dpi),
            status: String::new(),
        })
    });
    window::show_normal(hwnd);
    window::bring_to_front(hwnd);
    ACTIVE.with(|a| {
        if let Some(s) = a.borrow().as_ref() {
            // Caret after the typed text, ready for the next character.
            let end = text.encode_utf16().count() as i32;
            s.edit.focus_and_select(end, end);
        }
    });
}

fn paint_box(hwnd: HWND) {
    let p = paint::Paint::begin(hwnd);
    ACTIVE.with(|a| {
        let Ok(a) = a.try_borrow() else { return };
        let Some(s) = a.as_ref() else { return };
        let (w, h) = window::window_rect_size(hwnd);
        let px = |dip: f32| (dip * s.dpi as f32 / 96.0).round() as i32;
        p.fill(
            RECT {
                left: 0,
                top: 0,
                right: w,
                bottom: h,
            },
            s.edit.back_color(),
        );
        p.text(
            RECT {
                left: px(PAD),
                top: 0,
                right: px(PAD + GLYPH_W),
                bottom: h,
            },
            GLYPH,
            &s.glyph_font,
            s.dim_color,
            Align::Left,
        );
        p.text(
            RECT {
                left: w - px(PAD + STATUS_W),
                top: 0,
                right: w - px(PAD),
                bottom: h,
            },
            &s.status,
            &s.status_font,
            s.dim_color,
            Align::Right,
        );
    });
}

fn on_key(vk: u32) -> bool {
    let cmd = match vk {
        k if k == msg::VK_RETURN => Command::PeekSearchOpen,
        k if k == msg::VK_ESCAPE => Command::EndPeek("Esc pressed in the search box"),
        VK_UP => Command::PeekSearchStep(-1),
        VK_DOWN => Command::PeekSearchStep(1),
        _ => return false,
    };
    ACTIVE.with(|a| {
        if let Ok(a) = a.try_borrow()
            && let Some(s) = a.as_ref()
        {
            s.queue.push(cmd);
        }
    });
    true
}

pub fn is_open() -> bool {
    ACTIVE.with(|a| a.borrow().is_some())
}

/// The text in the box (None when it is closed).
pub fn query() -> Option<String> {
    ACTIVE.with(|a| a.borrow().as_ref().map(|s| s.edit.text()))
}

/// Adds `text` after what is in the box (characters that reached the dimmer before the box
/// took the keyboard).
pub fn append(text: &str) {
    ACTIVE.with(|a| {
        if let Some(s) = a.borrow().as_ref() {
            s.edit.set_text(&(s.edit.text() + text));
        }
    });
}

/// Shows `text` at the right end of the box.
pub fn set_status(text: &str) {
    let hwnd = ACTIVE.with(|a| {
        let mut a = a.borrow_mut();
        let s = a.as_mut()?;
        if s.status == text {
            return None;
        }
        s.status = text.to_string();
        Some(s.window.hwnd())
    });
    if let Some(hwnd) = hwnd {
        paint::invalidate(hwnd);
    }
}

/// Puts the box back on top of the fences (the Peek band re-raises them).
pub fn raise() {
    if let Some(hwnd) = ACTIVE.with(|a| a.borrow().as_ref().map(|s| s.window.hwnd())) {
        let _ = desktop::raise_in_band(hwnd);
    }
}

pub fn close() {
    let session = ACTIVE.with(|a| a.borrow_mut().take());
    if let Some(s) = session {
        drop(s.edit);
        drop(s.window);
    }
}
