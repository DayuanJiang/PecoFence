//! The filter box of a folder fence (筛选): a small activatable popup hosting an EDIT control in
//! the fence's title row, as the inline rename does. It looks like Windows 11's search box:
//! rounded, a magnifier at the left, the grey hint 筛选“<name>”, then at the right how many
//! files show ("4 / 31") and × to clear the text, with the accent line of a focused text box
//! at the bottom. Every change filters the fence at once; Enter or a click elsewhere closes
//! the box and keeps the filter, Esc clears it.

use crate::commands::{Command, CommandQueue};
use pecofence_core::FenceId;
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, RECT, dwm, edit, fonts, monitors, msg, paint};
use pecofence_render::ThemeMode;
use std::cell::RefCell;

const FILTER_CLASS: &str = "PecoFence.Filter";

// Layout in DIPs.
const BOX_H: f32 = 28.0;
const GLYPH_W: f32 = 30.0;
const COUNT_W: f32 = 56.0;
const CLEAR_W: f32 = 28.0;
const EDIT_H: f32 = 20.0;

thread_local! {
    static ACTIVE: RefCell<Option<FilterSession>> = const { RefCell::new(None) };
    static CLASS: RefCell<Option<WindowClass>> = const { RefCell::new(None) };
}

struct FilterSession {
    window: Window,
    edit: edit::EditControl,
    fence: FenceId,
    queue: CommandQueue,
    dpi: u32,
    /// How many files show, at the right ("4 / 31", or all of them while the text is empty).
    count: String,
    glyph_font: paint::Font,
    count_font: paint::Font,
    clear_font: paint::Font,
}

impl FilterSession {
    fn px(&self, dip: f32) -> i32 {
        (dip * self.dpi as f32 / 96.0).round() as i32
    }
}

/// Opens (or moves) the filter box over `fence_hwnd`'s title row with `current` in it; `cue`
/// is the grey hint while it is empty.
pub fn open(
    fence_hwnd: HWND,
    fence: FenceId,
    current: &str,
    cue: &str,
    queue: CommandQueue,
    mode: ThemeMode,
) {
    close();
    let r = window::window_rect(fence_hwnd);
    let dpi = monitors::dpi_for_window(fence_hwnd).max(96);
    let px = |dip: f32| (dip * dpi as f32 / 96.0).round() as i32;
    // Over the whole title row, as the rename box: the roll-up chevron stays clear.
    let left = r.left + px(8.0);
    let right = (r.right - px(44.0)).max(left + px(120.0));
    let top = r.top + px(4.0);
    let (w, h) = (right - left, px(BOX_H));
    CLASS.with(|c| {
        if c.borrow().is_none() {
            *c.borrow_mut() = WindowClass::register(FILTER_CLASS, ClassOptions::default()).ok();
        }
    });
    let handler: MessageHandler = Box::new(
        move |hwnd: HWND, message: u32, wparam: usize, lparam: isize| -> Option<isize> {
            match message {
                msg::WM_COMMAND if msg::hi_u16(wparam) == edit::EN_CHANGE => {
                    ACTIVE.with(|a| {
                        if let Some(s) = a.try_borrow().ok().as_ref().and_then(|a| a.as_ref()) {
                            s.queue.push(Command::FenceFilterText {
                                fence: s.fence,
                                text: s.edit.text(),
                            });
                        }
                    });
                    Some(0)
                }
                msg::WM_COMMAND if msg::hi_u16(wparam) == edit::EN_KILLFOCUS => {
                    close();
                    Some(0)
                }
                edit::WM_CTLCOLOREDIT_MSG => {
                    ACTIVE.with(|a| a.borrow().as_ref().map(|s| s.edit.ctl_color(wparam)))
                }
                msg::WM_ERASEBKGND => Some(1),
                msg::WM_PAINT => {
                    paint_box(hwnd);
                    Some(0)
                }
                // × clears the text; the box stays open.
                msg::WM_LBUTTONDOWN => {
                    let (w, _) = window::window_rect_size(hwnd);
                    ACTIVE.with(|a| {
                        if let Some(s) = a.try_borrow().ok().as_ref().and_then(|a| a.as_ref())
                            && msg::lo_i16(lparam) >= w - s.px(CLEAR_W + 2.0)
                        {
                            tracing::info!(fence = %s.fence, "filter box: × clears the text");
                            s.edit.set_text("");
                            s.edit.focus_at_end();
                        }
                    });
                    Some(0)
                }
                msg::WM_DESTROY => Some(0),
                _ => None,
            }
        },
    );
    let created = CLASS.with(|c| {
        let c = c.borrow();
        let class = c.as_ref()?;
        // Topmost for the same reason as the rename popup (#99): during Peek and Win+D the
        // fences float in the topmost band.
        WindowBuilder::new(class)
            .title("filter")
            .style(style::POPUP | style::CLIPCHILDREN)
            .ex_style(style::EX_TOOLWINDOW | style::EX_TOPMOST)
            .bounds(left, top, w, h)
            .create(handler)
            .ok()
    });
    let Some(window) = created else { return };
    // Rounded as a Fluent text box; Windows draws the corners where it can (a GPU).
    let _ = dwm::set_corner_preference(window.hwnd(), dwm::CornerPreference::RoundSmall);
    let edit_x = px(GLYPH_W);
    let edit_w = (w - edit_x - px(COUNT_W + CLEAR_W + 4.0)).max(px(40.0));
    let Ok(ctl) = edit::EditControl::create(
        window.hwnd(),
        edit_w,
        px(EDIT_H),
        dpi,
        14.0,
        current,
        mode == ThemeMode::Dark,
        on_key,
    ) else {
        return;
    };
    ctl.set_bounds(edit_x, (h - px(EDIT_H)) / 2, edit_w, px(EDIT_H));
    ctl.set_cue(cue);
    window::show_normal(window.hwnd());
    window::bring_to_front(window.hwnd());
    ctl.focus_at_end();
    ACTIVE.with(|a| {
        *a.borrow_mut() = Some(FilterSession {
            window,
            edit: ctl,
            fence,
            queue,
            dpi,
            count: String::new(),
            glyph_font: paint::Font::new(fonts::icon_face(), 13.0, dpi),
            count_font: paint::Font::new(fonts::text_face(), 12.0, dpi),
            clear_font: paint::Font::new(fonts::icon_face(), 10.0, dpi),
        })
    });
}

/// The box around the text: border, accent line, magnifier, count and ×.
fn paint_box(hwnd: HWND) {
    let p = paint::Paint::begin(hwnd);
    ACTIVE.with(|a| {
        let Ok(a) = a.try_borrow() else { return };
        let Some(s) = a.as_ref() else { return };
        let (w, h) = window::window_rect_size(hwnd);
        let back = s.edit.back_color();
        let dark = back < 0x0080_8080;
        let border = if dark { 0x0050_5050 } else { 0x00C8_C8C8 };
        let dim = if dark { 0x00C5_C5C5 } else { 0x005F_5F5F };
        let all = RECT {
            left: 0,
            top: 0,
            right: w,
            bottom: h,
        };
        p.fill(all, back);
        p.round_rect(all, s.px(4.0), back, border);
        // A focused Fluent text box: 2 px of the accent along the bottom.
        let accent = pecofence_platform::theme::accent_palette().map_or(0x00B8_5F00, |c| {
            let rgb = if dark { c.light2 } else { c.dark1 };
            u32::from_le_bytes([rgb[0], rgb[1], rgb[2], 0])
        });
        p.fill(
            RECT {
                left: s.px(3.0),
                top: h - s.px(2.0),
                right: w - s.px(3.0),
                bottom: h,
            },
            accent,
        );
        let row = |left: i32, right: i32| RECT {
            left,
            top: 0,
            right,
            bottom: h,
        };
        p.text(
            row(s.px(6.0), s.px(GLYPH_W - 2.0)),
            "\u{E721}",
            &s.glyph_font,
            dim,
            paint::Align::Center,
        );
        p.text(
            row(w - s.px(COUNT_W + CLEAR_W + 2.0), w - s.px(CLEAR_W + 2.0)),
            &s.count,
            &s.count_font,
            dim,
            paint::Align::Right,
        );
        // Cancel (E711): clears the text.
        p.text(
            row(w - s.px(CLEAR_W), w - s.px(4.0)),
            "\u{E711}",
            &s.clear_font,
            dim,
            paint::Align::Center,
        );
    });
}

/// How many files the fence shows now, at the right of the box ("4 / 31").
pub fn set_count(text: &str) {
    let hwnd = ACTIVE.with(|a| {
        let mut a = a.try_borrow_mut().ok()?;
        let s = a.as_mut()?;
        if s.count == text {
            return None;
        }
        s.count = text.to_string();
        Some(s.window.hwnd())
    });
    if let Some(h) = hwnd {
        paint::invalidate(h);
    }
}

fn on_key(vk: u32) -> bool {
    match vk {
        k if k == msg::VK_RETURN => {
            close();
            true
        }
        k if k == msg::VK_ESCAPE => {
            let session = ACTIVE.with(|a| a.borrow_mut().take());
            if let Some(s) = session {
                s.queue.push(Command::FenceFilterText {
                    fence: s.fence,
                    text: String::new(),
                });
                s.queue.push(Command::FenceFilterClosed(s.fence));
            }
            true
        }
        _ => false,
    }
}

/// The fence whose filter box is open, if one is.
pub fn active_fence() -> Option<FenceId> {
    ACTIVE.with(|a| a.try_borrow().ok()?.as_ref().map(|s| s.fence))
}

/// Closes the box; the filter stays as typed.
pub fn close() {
    let session = ACTIVE.with(|a| a.try_borrow_mut().ok()?.take());
    if let Some(s) = session {
        s.queue.push(Command::FenceFilterClosed(s.fence));
        drop(s.edit);
        drop(s.window);
    }
}
