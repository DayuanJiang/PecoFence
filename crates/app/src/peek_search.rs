//! The Peek search box: typing during a Peek (or pressing Ctrl twice, when that is on) opens
//! it in the upper middle of the pointer's screen. The fences dim every icon that does not
//! match and highlight the matches (`App::update_peek_search`); a list under the box shows the
//! best matches, the fences' first and then what Everything finds on the whole computer, as
//! Listary's launcher does. Up / Down (or Ctrl+N / Ctrl+P) move through the list, Enter opens,
//! Ctrl+Enter opens the containing folder, Right (with the caret at the end), Ctrl+O, the menu
//! key or Shift+F10 show the actions, Ctrl+1 to Ctrl+9 open a row directly, Esc ends the Peek.
//! A right-click on a row highlights it and shows its actions at the pointer, as in Explorer;
//! a right-click on the typed text gives the text box's own menu.
//!
//! A small activatable popup hosting an EDIT control, like the rename popup, so IME input,
//! the caret and clipboard keys work as in any text box. The popup paints its background, the
//! search glyph, the match count and the list itself (GDI). Everything's answers come to this
//! window as `WM_COPYDATA`.

use crate::commands::{Command, CommandQueue};
use pecofence_platform::fileinfo::{self, FileIcon};
use pecofence_platform::paint::{self, Align, Font};
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, POINT, RECT, desktop, dwm, edit, everything, fonts, monitors, msg};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;

pub const SEARCH_CLASS: &str = "PecoFence.PeekSearch";
const WA_INACTIVE: usize = 0;
const VK_UP: u32 = 0x26;
const VK_DOWN: u32 = 0x28;
const VK_RIGHT: u32 = 0x27;
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
const LIST_GAP: f32 = 4.0;
const HEADING_H: f32 = 24.0;
const ROW_H: f32 = 44.0;
const ROW_PAD: f32 = 16.0;
const ICON: f32 = 20.0;
const NAME_SIZE: f32 = 14.0;
const DETAIL_SIZE: f32 = 11.5;
const HEADING_SIZE: f32 = 11.5;

/// One line of the result list.
pub struct Row {
    pub name: String,
    /// Characters of `name` to show in the accent colour (char indices).
    pub marks: Vec<Range<usize>>,
    /// The fence it is in, or its folder.
    pub detail: String,
    /// For the icon; `None` = no icon (the "show all in Everything" row).
    pub icon_path: Option<PathBuf>,
    pub is_folder: bool,
    /// Read the file's own icon (local drives); otherwise its type's icon.
    pub exact_icon: bool,
    /// Heading above this row; `None` = same group as the row before.
    pub heading: Option<String>,
    /// The digit of its Ctrl+digit shortcut.
    pub shortcut: Option<u8>,
}

thread_local! {
    static ACTIVE: RefCell<Option<Session>> = const { RefCell::new(None) };
    static CLASS: RefCell<Option<WindowClass>> = const { RefCell::new(None) };
}

struct Session {
    window: Window,
    edit: edit::EditControl,
    queue: CommandQueue,
    dpi: u32,
    dark: bool,
    /// Window position and width; the height follows the list.
    x: i32,
    y: i32,
    w: i32,
    /// Bottom of the work area: the list stops above it.
    work_bottom: i32,
    /// Glyph and match count colour (COLORREF).
    dim_color: u32,
    glyph_font: Font,
    status_font: Font,
    name_font: Font,
    detail_font: Font,
    heading_font: Font,
    /// Match count shown at the right ("2 / 5", or that nothing matches).
    status: String,
    rows: Vec<Row>,
    selected: Option<usize>,
    /// Top and bottom (client pixels) of each row, from the last layout.
    row_spans: Vec<(i32, i32)>,
    icons: HashMap<PathBuf, Option<FileIcon>>,
}

impl Session {
    fn px(&self, dip: f32) -> i32 {
        (dip * self.dpi as f32 / 96.0).round() as i32
    }

    /// Lays the rows out under the box; returns the window height.
    fn layout(&mut self) -> i32 {
        let mut y = self.px(HEIGHT);
        if self.rows.is_empty() {
            self.row_spans.clear();
            return y;
        }
        y += self.px(LIST_GAP);
        let mut spans = Vec::with_capacity(self.rows.len());
        for row in &self.rows {
            if row.heading.is_some() {
                y += self.px(HEADING_H);
            }
            let top = y;
            y += self.px(ROW_H);
            spans.push((top, y));
        }
        self.row_spans = spans;
        y + self.px(LIST_GAP * 2.0)
    }

    fn row_at(&self, y: i32) -> Option<usize> {
        self.row_spans.iter().position(|&(t, b)| y >= t && y < b)
    }
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
                    msg::WM_LBUTTONDOWN => {
                        let y = msg::hi_i16(lparam);
                        let row = ACTIVE.with(|a| a.try_borrow().ok()?.as_ref()?.row_at(y));
                        if let Some(row) = row {
                            queue.push(Command::PeekSearchOpenRow(row));
                        }
                        Some(0)
                    }
                    // DefWindowProc sends it on a right-click's button up (screen point). A row
                    // gets its actions at the pointer; elsewhere (around the box, a heading)
                    // there is no menu, as on an empty spot of a Windows list. The edit box
                    // never sends it here: it shows its own text menu.
                    msg::WM_CONTEXTMENU if lparam as u32 != u32::MAX => {
                        let (x, y) = (msg::lo_i16(lparam), msg::hi_i16(lparam));
                        let client = window::screen_to_client(hwnd, POINT { x, y });
                        let row = ACTIVE.with(|a| a.try_borrow().ok()?.as_ref()?.row_at(client.y));
                        match row {
                            Some(row) => queue.push(Command::PeekSearchRowMenu { row, x, y }),
                            None => tracing::info!(
                                x = client.x,
                                y = client.y,
                                "search: right-click outside the result rows, no menu"
                            ),
                        }
                        Some(0)
                    }
                    everything::WM_COPYDATA_MSG => {
                        if let Some((kind, bytes)) = everything::copydata(lparam) {
                            queue.push(Command::EverythingResults { kind, bytes });
                        }
                        Some(1)
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
            dark,
            x,
            y,
            w,
            work_bottom: wa.bottom,
            dim_color: if dark { 0x00C5_C5C5 } else { 0x005F_5F5F },
            glyph_font: Font::new(fonts::icon_face(), GLYPH_SIZE, dpi),
            status_font: Font::new(fonts::text_face(), STATUS_SIZE, dpi),
            name_font: Font::new(fonts::text_face(), NAME_SIZE, dpi),
            detail_font: Font::new(fonts::text_face(), DETAIL_SIZE, dpi),
            heading_font: Font::new(fonts::text_face(), HEADING_SIZE, dpi),
            status: String::new(),
            rows: Vec::new(),
            selected: None,
            row_spans: Vec::new(),
            icons: HashMap::new(),
        })
    });
    window::show_normal(hwnd);
    if !window::claim_foreground(hwnd) {
        tracing::debug!("search box: the foreground switch was refused");
    }
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
        let Ok(mut a) = a.try_borrow_mut() else {
            return;
        };
        let Some(s) = a.as_mut() else { return };
        let (w, h) = window::window_rect_size(hwnd);
        let px = |dip: f32| (dip * s.dpi as f32 / 96.0).round() as i32;
        let back = s.edit.back_color();
        p.fill(
            RECT {
                left: 0,
                top: 0,
                right: w,
                bottom: h,
            },
            back,
        );
        let box_h = px(HEIGHT);
        p.text(
            RECT {
                left: px(PAD),
                top: 0,
                right: px(PAD + GLYPH_W),
                bottom: box_h,
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
                bottom: box_h,
            },
            &s.status,
            &s.status_font,
            s.dim_color,
            Align::Right,
        );
        if s.rows.is_empty() {
            return;
        }
        // A hairline between the box and the list.
        let line = if s.dark { 0x0040_4040 } else { 0x00E0_E0E0 };
        p.fill(
            RECT {
                left: 0,
                top: box_h,
                right: w,
                bottom: box_h + 1.max(px(1.0)),
            },
            line,
        );
        let text_color = if s.dark { 0x00F0_F0F0 } else { 0x001E_1E1E };
        let accent = if s.dark { 0x00FF_C060 } else { 0x00D7_5B2A };
        let selected_back = if s.dark { 0x0048_3A2E } else { 0x00FA_EBE3 };
        let (pad, icon) = (px(ROW_PAD), px(ICON));
        let hint_w = px(56.0);
        for i in 0..s.rows.len() {
            let (top, bottom) = s.row_spans[i];
            let row = &s.rows[i];
            if let Some(heading) = &row.heading {
                p.text(
                    RECT {
                        left: pad,
                        top: top - px(HEADING_H),
                        right: w - pad,
                        bottom: top,
                    },
                    heading,
                    &s.heading_font,
                    s.dim_color,
                    Align::Left,
                );
            }
            if s.selected == Some(i) {
                p.fill(
                    RECT {
                        left: px(6.0),
                        top,
                        right: w - px(6.0),
                        bottom,
                    },
                    selected_back,
                );
            }
            let mid = (top + bottom) / 2;
            if let Some(path) = row.icon_path.clone() {
                let (is_folder, exact) = (row.is_folder, row.exact_icon);
                let ico = s
                    .icons
                    .entry(path.clone())
                    .or_insert_with(|| fileinfo::small_icon(&path, is_folder, exact));
                if let Some(ico) = ico {
                    p.icon(pad, mid - icon / 2, icon, ico);
                }
            }
            let row = &s.rows[i];
            let text_left = pad + icon + px(12.0);
            let text_right = w - pad - hint_w;
            let name_rect = RECT {
                left: text_left,
                top: top + px(4.0),
                right: text_right,
                bottom: mid + px(2.0),
            };
            let (name, marks) = fit_marks(
                &p,
                &row.name,
                &row.marks,
                &s.name_font,
                text_right - text_left,
            );
            p.text(name_rect, &name, &s.name_font, text_color, Align::Left);
            // The matched characters again in the accent colour, at their measured offsets.
            let chars: Vec<char> = name.chars().collect();
            for r in &marks {
                if r.start >= chars.len() {
                    continue;
                }
                let end = r.end.min(chars.len());
                let before: String = chars[..r.start].iter().collect();
                let part: String = chars[r.start..end].iter().collect();
                let x = text_left + p.text_width(&before, &s.name_font);
                if x >= text_right {
                    continue;
                }
                p.text(
                    RECT {
                        left: x,
                        right: text_right,
                        ..name_rect
                    },
                    &part,
                    &s.name_font,
                    accent,
                    Align::Left,
                );
            }
            p.text(
                RECT {
                    left: text_left,
                    top: mid,
                    right: text_right,
                    bottom: bottom - px(4.0),
                },
                &row.detail,
                &s.detail_font,
                s.dim_color,
                Align::Left,
            );
            if let Some(d) = row.shortcut {
                p.text(
                    RECT {
                        left: w - pad - hint_w,
                        top,
                        right: w - pad,
                        bottom,
                    },
                    &format!("Ctrl+{d}"),
                    &s.detail_font,
                    s.dim_color,
                    Align::Right,
                );
            }
        }
    });
}

/// A name too long for its row whose matched letters lie past what fits: its start gives way
/// to "…" so they stay in sight. Returns the name to draw and its marks.
fn fit_marks(
    p: &paint::Paint,
    name: &str,
    marks: &[Range<usize>],
    font: &Font,
    room: i32,
) -> (String, Vec<Range<usize>>) {
    let chars: Vec<char> = name.chars().collect();
    let last = marks
        .iter()
        .map(|r| r.end.min(chars.len()))
        .max()
        .unwrap_or(0);
    let width = |from: usize, lead: bool| {
        let text: String = lead
            .then_some('…')
            .into_iter()
            .chain(chars[from..last].iter().copied())
            .collect();
        p.text_width(&text, font)
    };
    if last == 0 || width(0, false) <= room {
        return (name.to_string(), marks.to_vec());
    }
    let mut cut = 1;
    while cut < last - 1 && width(cut, true) > room {
        cut += 1;
    }
    let shown = std::iter::once('…')
        .chain(chars[cut..].iter().copied())
        .collect();
    let marks = marks
        .iter()
        .filter(|r| r.end > cut)
        .map(|r| r.start.max(cut) - cut + 1..r.end.min(chars.len()) - cut + 1)
        .collect();
    (shown, marks)
}

fn on_key(vk: u32) -> bool {
    let ctrl = window::key_down(msg::VK_CONTROL);
    let cmd = match vk {
        k if k == msg::VK_RETURN && ctrl => Command::PeekSearchOpenFolder,
        k if k == msg::VK_RETURN => Command::PeekSearchOpen,
        k if k == msg::VK_ESCAPE => Command::EndPeek("Esc pressed in the search box"),
        VK_UP => Command::PeekSearchStep(-1),
        VK_DOWN => Command::PeekSearchStep(1),
        0x50 if ctrl => Command::PeekSearchStep(-1),
        0x4E if ctrl => Command::PeekSearchStep(1),
        0x4F if ctrl => Command::PeekSearchActions,
        // The menu key or Shift+F10: the highlighted row's actions, as a right-click on it
        // shows them. With no row the box keeps its own text menu.
        edit::KEY_CONTEXT_MENU => {
            let has_row =
                ACTIVE.with(|a| a.borrow().as_ref().is_some_and(|s| s.selected.is_some()));
            if !has_row {
                tracing::info!(
                    "search: menu key or Shift+F10 with no result row, the text box menu shows"
                );
                return false;
            }
            Command::PeekSearchActions
        }
        // Ctrl+1 .. Ctrl+9: the row showing that digit.
        0x31..=0x39 if ctrl => {
            let digit = (vk - 0x30) as u8;
            let row = ACTIVE.with(|a| {
                a.borrow()
                    .as_ref()?
                    .rows
                    .iter()
                    .position(|r| r.shortcut == Some(digit))
            });
            match row {
                Some(r) => Command::PeekSearchOpenRow(r),
                None => return true,
            }
        }
        VK_RIGHT => {
            let at_end = ACTIVE.with(|a| {
                a.borrow()
                    .as_ref()
                    .is_some_and(|s| s.selected.is_some() && s.edit.caret_at_end())
            });
            if !at_end {
                return false;
            }
            Command::PeekSearchActions
        }
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

/// The popup window (Everything sends its answers here).
pub fn hwnd() -> Option<HWND> {
    ACTIVE.with(|a| a.borrow().as_ref().map(|s| s.window.hwnd()))
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

/// Replaces the text in the box (`pecofence-cli peek start --search`).
pub fn set_text(text: &str) {
    ACTIVE.with(|a| {
        if let Some(s) = a.borrow().as_ref() {
            s.edit.set_text(text);
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

/// Replaces the list under the box and the highlighted row; the window grows or shrinks to fit.
pub fn set_rows(rows: Vec<Row>, selected: Option<usize>) {
    let resize = ACTIVE.with(|a| {
        let mut a = a.borrow_mut();
        let s = a.as_mut()?;
        s.rows = rows;
        s.selected = selected.filter(|&i| i < s.rows.len());
        // Icons of rows that went away are dropped (they are cheap to get again).
        let keep: Vec<PathBuf> = s.rows.iter().filter_map(|r| r.icon_path.clone()).collect();
        s.icons.retain(|p, _| keep.contains(p));
        let h = s.layout();
        Some((s.window.hwnd(), s.x, s.y, s.w, h))
    });
    if let Some((hwnd, x, y, w, h)) = resize {
        let (_, now) = window::window_rect_size(hwnd);
        if now != h {
            ACTIVE.with(|a| {
                if let Some(s) = a.borrow().as_ref() {
                    let _ = s.window.set_bounds(x, y, w, h);
                }
            });
        }
        paint::invalidate(hwnd);
    }
}

/// How many rows fit between the box and the bottom of the work area when `headings` group
/// headings are among them (a small screen at a high scale holds fewer than nine).
pub fn row_room(headings: usize) -> usize {
    ACTIVE.with(|a| {
        let a = a.borrow();
        let Some(s) = a.as_ref() else {
            return usize::MAX;
        };
        let free = s.work_bottom
            - s.y
            - s.px(HEIGHT)
            - s.px(LIST_GAP * 3.0)
            - headings as i32 * s.px(HEADING_H);
        (free.max(0) / s.px(ROW_H).max(1)) as usize
    })
}

/// Screen position under the highlighted row (for its action menu).
pub fn selected_row_point() -> Option<(i32, i32)> {
    ACTIVE.with(|a| {
        let a = a.borrow();
        let s = a.as_ref()?;
        let (_, bottom) = *s.row_spans.get(s.selected?)?;
        Some((s.x + s.px(ROW_PAD), s.y + bottom))
    })
}

/// Gives the box the keyboard again, if it is open.
pub fn focus() {
    let Some(hwnd) = ACTIVE.with(|a| a.borrow().as_ref().map(|s| s.window.hwnd())) else {
        return;
    };
    if !window::claim_foreground(hwnd) {
        tracing::debug!("search box: the foreground switch was refused");
    }
    ACTIVE.with(|a| {
        if let Some(s) = a.borrow().as_ref() {
            s.edit.focus();
        }
    });
    tracing::debug!("search box has the keyboard again");
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
