//! The filter box of a folder fence (筛选): a small activatable popup hosting an EDIT control in
//! the fence's title row, as the inline rename does. It looks like Windows 11's search box:
//! rounded, a magnifier at the left, the grey hint 筛选“<name>”, then at the right how many
//! files show ("4 / 31") and × to clear the text, with the accent line of a focused text box
//! at the bottom. Every change filters the fence at once; Enter or a click elsewhere closes
//! the box and keeps the filter, Esc clears it.
//!
//! A type as Finder takes it: a text that names one (pdf, 图片) offers 「类型：PDF」 at the right,
//! Tab makes it a chip before the text (`kind:` or `类型:` words become one too), Backspace in
//! an empty box takes the last chip back. The fence is filtered by `kind:<type>` words in
//! front of the text.

use crate::commands::{Command, CommandQueue};
use pecofence_core::FenceId;
use pecofence_core::type_filter::{self, TypeToken};
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, RECT, dwm, edit, fonts, monitors, msg, paint};
use pecofence_render::ColorF;
use std::cell::{Cell, RefCell};

const FILTER_CLASS: &str = "PecoFence.Filter";

// Layout in DIPs.
const BOX_H: f32 = 28.0;
const GLYPH_W: f32 = 30.0;
const COUNT_W: f32 = 56.0;
const CLEAR_W: f32 = 28.0;
const EDIT_H: f32 = 20.0;
const SUGGEST_W: f32 = 140.0;

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
    /// The type chips before the text, the type the text names (Tab takes it), and the text
    /// box's left edge and width as last placed.
    tokens: Vec<TypeToken>,
    suggest: Option<TypeToken>,
    edit_at: Cell<(i32, i32)>,
}

impl FilterSession {
    fn px(&self, dip: f32) -> i32 {
        (dip * self.dpi as f32 / 96.0).round() as i32
    }

    /// What the fence is filtered by: the chips as `kind:` words, then the text.
    fn combined(&self) -> String {
        let mut words: Vec<String> = self.tokens.iter().map(type_filter::word).collect();
        words.push(self.edit.text());
        words.join(" ").trim().to_string()
    }
}

/// The finished `kind:` / `类型:` words of `text` (a space after them; the last word may still
/// grow, kind:doc into kind:docx) and the text without them.
fn finished_types(text: &str) -> (Vec<TypeToken>, String) {
    let all_done = text.ends_with(char::is_whitespace);
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut found = Vec::new();
    let mut rest = Vec::new();
    for (i, w) in words.iter().enumerate() {
        match type_filter::parse_word(w).filter(|_| all_done || i + 1 < words.len()) {
            Some(t) => found.push(t),
            None => rest.push(*w),
        }
    }
    (found, rest.join(" "))
}

/// The text changed: finished `kind:` / `类型:` words become chips, a text that names a type
/// offers it (Tab takes either).
fn type_change() {
    let rest = ACTIVE.with(|a| {
        let mut a = a.try_borrow_mut().ok()?;
        let s = a.as_mut()?;
        let (found, rest) = finished_types(&s.edit.text());
        for t in found.iter().cloned() {
            if !s.tokens.contains(&t) {
                tracing::info!(fence = %s.fence, token = %type_filter::word(&t), "filter box: a typed type word became a chip");
                s.tokens.push(t);
            }
        }
        let suggest = type_filter::suggest(rest.trim()).filter(|t| !s.tokens.contains(t));
        if suggest != s.suggest {
            tracing::info!(fence = %s.fence, offered = ?suggest.as_ref().map(type_filter::word), "filter box: the text names a type");
            s.suggest = suggest;
        }
        paint::invalidate(s.window.hwnd());
        (!found.is_empty()).then_some(rest)
    });
    // The words leave the text once the borrow is over: that change pushes the filter again.
    if let Some(rest) = rest {
        ACTIVE.with(|a| {
            if let Some(s) = a.borrow().as_ref() {
                s.edit.set_text(&rest);
                s.edit.focus_at_end();
            }
        });
    }
}

/// Opens (or moves) the filter box over `fence_hwnd`'s title row with `current` in it; `cue`
/// is the grey hint while it is empty. `ink` is the colour the fence draws its title in: the
/// box takes it for its text, on the field that colour reads on, as a name box does.
pub fn open(
    fence_hwnd: HWND,
    fence: FenceId,
    current: &str,
    cue: &str,
    queue: CommandQueue,
    ink: ColorF,
) {
    close();
    let (dark, text_color) = crate::rename::box_colors(ink);
    tracing::debug!(%fence, dark_field = dark, text = format_args!("{text_color:06x}"), "filter box colours");
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
                    type_change();
                    ACTIVE.with(|a| {
                        if let Some(s) = a.try_borrow().ok().as_ref().and_then(|a| a.as_ref()) {
                            s.queue.push(Command::FenceFilterText {
                                fence: s.fence,
                                text: s.combined(),
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
                    // × clears the text and the chips; the box stays open.
                    let cleared = ACTIVE.with(|a| {
                        let mut a = a.try_borrow_mut().ok()?;
                        let s = a.as_mut()?;
                        if msg::lo_i16(lparam) < w - s.px(CLEAR_W + 2.0) {
                            return None;
                        }
                        tracing::info!(fence = %s.fence, chips = s.tokens.len(), "filter box: × clears the text");
                        s.tokens.clear();
                        s.suggest = None;
                        s.queue.push(Command::FenceFilterText {
                            fence: s.fence,
                            text: String::new(),
                        });
                        paint::invalidate(s.window.hwnd());
                        Some(())
                    });
                    if cleared.is_some() {
                        ACTIVE.with(|a| {
                            if let Some(s) = a.borrow().as_ref() {
                                s.edit.set_text("");
                                s.edit.focus_at_end();
                            }
                        });
                    }
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
    // A type in the current filter comes back as its chip.
    let (tokens, typed) = type_filter::split(current);
    let Ok(mut ctl) = edit::EditControl::create(
        window.hwnd(),
        edit_w,
        px(EDIT_H),
        dpi,
        14.0,
        &typed,
        dark,
        on_key,
    ) else {
        return;
    };
    ctl.set_text_color(text_color);
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
            tokens,
            suggest: None,
            edit_at: Cell::new((edit_x, edit_w)),
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
        // The type chips, then the text box after them.
        let tint = if dark { 0x0060_4A33 } else { 0x00F7_E8D9 };
        let accent_text = if dark { 0x00FF_C060 } else { 0x00B8_5F00 };
        let mut x = s.px(GLYPH_W);
        for t in &s.tokens {
            let label = type_filter::label(t);
            let cw = p.text_width(&label, &s.count_font) + s.px(16.0);
            let r = RECT {
                left: x,
                top: s.px(4.0),
                right: x + cw,
                bottom: h - s.px(5.0),
            };
            p.round_rect(r, (r.bottom - r.top) / 2, tint, tint);
            p.text(r, &label, &s.count_font, accent_text, paint::Align::Center);
            x = r.right + s.px(4.0);
        }
        // The count takes the room its text needs: 「Elemente: 128」 is wider than 「128 项」,
        // and cut short it lost its number.
        let count_w = s
            .px(COUNT_W)
            .max(p.text_width(&s.count, &s.count_font) + s.px(8.0));
        // The suggestion needs more room at the right than the count.
        let right = if s.suggest.is_some() {
            s.px(SUGGEST_W + CLEAR_W + 4.0)
        } else {
            count_w + s.px(CLEAR_W + 4.0)
        };
        let ew = (w - x - right).max(s.px(30.0));
        if (x, ew) != s.edit_at.get() {
            s.edit_at.set((x, ew));
            s.edit
                .set_bounds(x, (h - s.px(EDIT_H)) / 2, ew, s.px(EDIT_H));
        }
        match &s.suggest {
            // The type the text names, taken with Tab, in place of the count.
            Some(t) => p.text(
                row(w - s.px(SUGGEST_W + CLEAR_W + 2.0), w - s.px(CLEAR_W + 2.0)),
                &format!("{}  Tab", type_filter::label(t)),
                &s.count_font,
                accent_text,
                paint::Align::Right,
            ),
            None => p.text(
                row(w - count_w - s.px(CLEAR_W + 2.0), w - s.px(CLEAR_W + 2.0)),
                &s.count,
                &s.count_font,
                dim,
                paint::Align::Right,
            ),
        }
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
        tracing::debug!(fence = %s.fence, count = text, "filter box count");
        Some(s.window.hwnd())
    });
    if let Some(h) = hwnd {
        paint::invalidate(h);
    }
}

fn on_key(vk: u32) -> bool {
    // Tab takes the type the text names; Backspace in an empty box takes the last chip back.
    let took = ACTIVE.with(|a| {
        let mut a = a.try_borrow_mut().ok()?;
        let s = a.as_mut()?;
        if vk == 0x09 && s.suggest.is_some() {
            let t = s.suggest.take()?;
            tracing::info!(fence = %s.fence, token = %type_filter::word(&t), "filter box: Tab takes the type");
            s.tokens.push(t);
            paint::invalidate(s.window.hwnd());
            return Some(Some(String::new()));
        }
        // Tab after a kind: word still being typed finishes it.
        if vk == 0x09 {
            let text = s.edit.text();
            let (found, rest) = finished_types(&format!("{text} "));
            if !found.is_empty() {
                tracing::info!(fence = %s.fence, chips = found.len(), "filter box: Tab finishes the kind: word");
                for t in found {
                    if !s.tokens.contains(&t) {
                        s.tokens.push(t);
                    }
                }
                paint::invalidate(s.window.hwnd());
                return Some(Some(rest));
            }
        }
        if vk == 0x08 && s.edit.text().is_empty() && s.tokens.pop().is_some() {
            tracing::info!(fence = %s.fence, "filter box: Backspace takes the last chip back");
            s.queue.push(Command::FenceFilterText {
                fence: s.fence,
                text: s.combined(),
            });
            paint::invalidate(s.window.hwnd());
            return Some(None);
        }
        None
    });
    match took {
        // The text leaves for the chip after the borrow: its change pushes the filter.
        Some(Some(rest)) => {
            ACTIVE.with(|a| {
                if let Some(s) = a.borrow().as_ref() {
                    s.edit.set_text(&rest);
                    s.edit.focus_at_end();
                }
            });
            return true;
        }
        Some(None) => return true,
        None => {}
    }
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
