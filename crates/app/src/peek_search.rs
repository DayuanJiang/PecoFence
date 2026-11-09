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
//! Windows 11's look: rounded corners (by Windows, where it rounds), group headings (栅栏里,
//! 整台电脑, 操作), a glyph on the two action rows, the Ctrl+digit of the highlighted row only,
//! and a bottom bar with the keys and the number of results, as PowerToys' Command Palette.
//! Under the box a row of type tabs (全部, 文件夹, 文档, …), as Windows 11's Start search has:
//! a click or Ctrl+Tab / Ctrl+Shift+Tab picks one, and the fences' items and Everything's
//! results are only of that type.
//!
//! A small activatable popup hosting an EDIT control, like the rename popup, so IME input,
//! the caret and clipboard keys work as in any text box. The popup paints its background, the
//! search glyph, the match count and the list itself (GDI). Everything's answers come to this
//! window as `WM_COPYDATA`. A row shows its type's icon at once; its own icon (a program's, a
//! shortcut target's) is read on a worker thread and swapped in, since reading a cold file
//! (another drive, a sleeping disk) took from a tenth of a second to seconds.

use crate::commands::{Command, CommandQueue};
use pecofence_core::rules::TypeCategory;
use pecofence_core::type_filter;
use pecofence_platform::fileinfo::{self, FileIcon};
use pecofence_platform::paint::{self, Align, Font};
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, POINT, RECT, desktop, dwm, edit, everything, fonts, monitors, msg};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub const SEARCH_CLASS: &str = "PecoFence.PeekSearch";
/// A row's own icon is ready: the worker posts it to the box.
const WM_ROW_ICON: u32 = msg::WM_APP + 40;
/// A stage of the search slower than this is logged (its name and ms only; an own icon also
/// names its drive letter).
const SLOW_STAGE_MS: u128 = 30;

/// Logs `stage` when it took over [`SLOW_STAGE_MS`] (debug builds: `PECOFENCE_TEST_SLOW_MS`).
pub fn slow_stage(stage: &'static str, started: Instant) {
    log_stage(stage, started.elapsed());
}

fn log_stage(stage: &'static str, took: std::time::Duration) {
    let limit = slow_limit();
    if took.as_millis() > limit || limit == 0 {
        tracing::info!(
            stage,
            ms = took.as_millis() as u64,
            us = took.as_micros() as u64,
            "search: a stage was slow"
        );
    }
}

/// [`SLOW_STAGE_MS`], or in debug builds the kit's `PECOFENCE_TEST_SLOW_MS` (as the box's own).
pub(crate) fn slow_limit() -> u128 {
    if cfg!(debug_assertions) {
        edit::slow_ms()
    } else {
        SLOW_STAGE_MS
    }
}
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
const BAR_H: f32 = 36.0;
const TABS_H: f32 = 38.0;
const TAB_PAD: f32 = 12.0;

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
    /// An action row's glyph (Segoe Fluent), drawn in the accent where a file has its icon.
    pub glyph: Option<&'static str>,
}

thread_local! {
    static ACTIVE: RefCell<Option<Session>> = const { RefCell::new(None) };
    static CLASS: RefCell<Option<WindowClass>> = const { RefCell::new(None) };
    static ICON_WORKER: IconWorker = IconWorker::new();
    /// When the box's text changed and the rows for it are not shown yet.
    static CHANGED_AT: std::cell::Cell<Option<Instant>> = const { std::cell::Cell::new(None) };
}

/// Reads rows' own icons off the UI thread, one thread per drive (its own STA, as Explorer reads
/// icons): a drive spinning up for seconds holds only its own rows, which keep their type's icon
/// meanwhile. Each icon read is posted to the box.
struct IconWorker {
    /// The thread of each drive ("C:", "network", "other").
    threads: RefCell<HashMap<String, Sender<(PathBuf, bool, isize)>>>,
    /// The paths the rows show now: a request for a row gone meanwhile is skipped.
    wanted: Arc<Mutex<HashSet<PathBuf>>>,
    /// Icons read, waiting for the box to take them.
    done: Arc<Mutex<Vec<(PathBuf, Option<FileIcon>)>>>,
}

impl IconWorker {
    fn new() -> Self {
        Self {
            threads: RefCell::new(HashMap::new()),
            wanted: Arc::default(),
            done: Arc::default(),
        }
    }

    /// Asks the thread of `path`'s drive for its own icon (made on first use).
    fn ask(&self, path: PathBuf, is_folder: bool, hwnd: HWND) {
        self.wanted
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(path.clone());
        let drive = drive_of(&path);
        let mut threads = self.threads.borrow_mut();
        if !threads.contains_key(&drive) {
            match self.spawn(&drive) {
                Some(tx) => {
                    threads.insert(drive.clone(), tx);
                }
                None => return,
            }
        }
        if let Some(tx) = threads.get(&drive) {
            let _ = tx.send((path, is_folder, hwnd.0 as isize));
        }
    }

    fn spawn(&self, drive: &str) -> Option<Sender<(PathBuf, bool, isize)>> {
        let (tx, rx) = channel::<(PathBuf, bool, isize)>();
        let (w, d) = (self.wanted.clone(), self.done.clone());
        let spawned = std::thread::Builder::new()
            .name(format!("pecofence-peek-icons-{drive}"))
            .spawn(move || {
                let _sta = pecofence_platform::com::OleGuard::init();
                for (path, is_folder, hwnd) in rx {
                    if !w.lock().unwrap_or_else(|e| e.into_inner()).contains(&path) {
                        continue;
                    }
                    let started = Instant::now();
                    test_slow_read();
                    let icon = fileinfo::small_icon(&path, is_folder, true);
                    let took = started.elapsed();
                    if took.as_millis() > slow_limit() || slow_limit() == 0 {
                        tracing::info!(stage = "own_icon", ms = took.as_millis() as u64, us = took.as_micros() as u64, drive = %drive_of(&path), "search: a stage was slow");
                    }
                    d.lock().unwrap_or_else(|e| e.into_inner()).push((path, icon));
                    window::post_message(HWND(hwnd as *mut core::ffi::c_void), WM_ROW_ICON, 0, 0);
                }
            });
        match spawned {
            Ok(_) => {
                tracing::info!(drive, "search: a thread reads this drive's row icons");
                Some(tx)
            }
            Err(e) => {
                tracing::warn!(error = %e, "search: no thread for the rows' icons; rows keep their type's icon");
                None
            }
        }
    }
}

/// Debug builds: `PECOFENCE_TEST_SLOW_READ_MS` makes every read off the UI thread (a row's own
/// icon, a shortcut's target) take that long, standing in for a slow or sleeping drive (the
/// kit's peek-cold-icons and peek-chinese-input).
pub(crate) fn test_slow_read() {
    #[cfg(debug_assertions)]
    if let Some(ms) = std::env::var("PECOFENCE_TEST_SLOW_READ_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

/// The drive of `path` for the log ("C:", "network", "other"): never its name.
pub(crate) fn drive_of(path: &Path) -> String {
    let s = path.to_string_lossy();
    match s.as_bytes() {
        [l, b':', ..] if l.is_ascii_alphabetic() => {
            format!("{}:", (*l as char).to_ascii_uppercase())
        }
        [b'\\', b'\\', ..] => "network".into(),
        _ => "other".into(),
    }
}

/// The key a type's icon is kept under: a folder, or a file's extension.
fn type_key(path: &Path, is_folder: bool) -> String {
    if is_folder {
        "<folder>".into()
    } else {
        path.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default()
    }
}

/// Takes the icons the worker read: those of rows still shown replace the type's icon (their
/// rows repaint), the rest are dropped.
fn take_row_icons(hwnd: HWND) {
    ACTIVE.with(|a| {
        let Ok(mut a) = a.try_borrow_mut() else {
            // The box is busy (a nested loop): ask again once it is free.
            window::post_message(hwnd, WM_ROW_ICON, 0, 0);
            return;
        };
        let Some(s) = a.as_mut() else { return };
        let ready = ICON_WORKER
            .with(|w| std::mem::take(&mut *w.done.lock().unwrap_or_else(|e| e.into_inner())));
        let (w, _) = window::window_rect_size(hwnd);
        let mut taken = 0;
        for (path, icon) in ready {
            s.asked.remove(&path);
            let Some(i) = s
                .rows
                .iter()
                .position(|r| r.icon_path.as_ref() == Some(&path))
            else {
                continue;
            };
            s.icons.insert(path, icon);
            taken += 1;
            if let Some(&(top, bottom)) = s.row_spans.get(i) {
                paint::invalidate_rect(
                    hwnd,
                    RECT {
                        left: 0,
                        top,
                        right: w,
                        bottom,
                    },
                );
            }
        }
        tracing::debug!(taken, "search: rows' own icons swapped in");
    });
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
    /// The 14 px glyph of an action row.
    row_glyph_font: Font,
    /// Match count shown at the right ("2 / 5", or that nothing matches).
    status: String,
    rows: Vec<Row>,
    selected: Option<usize>,
    /// How many results the search has in all (more than the rows shown).
    total: usize,
    /// Top and bottom (client pixels) of each row, from the last layout.
    row_spans: Vec<(i32, i32)>,
    /// The type tab: 0 全部, else `type_filter::TABS[tab - 1]`; and each tab's left and right
    /// (client pixels) as last drawn.
    tab: usize,
    tab_spans: Vec<(i32, i32)>,
    /// Rows' own icons, read by the worker; `asked` are on their way.
    icons: HashMap<PathBuf, Option<FileIcon>>,
    asked: HashSet<PathBuf>,
    /// Types' icons by extension (or folder), which read no file.
    type_icons: HashMap<String, Option<FileIcon>>,
}

impl Session {
    fn px(&self, dip: f32) -> i32 {
        (dip * self.dpi as f32 / 96.0).round() as i32
    }

    /// Lays the rows out under the box; returns the window height.
    fn layout(&mut self) -> i32 {
        let mut y = self.px(HEIGHT) + self.px(TABS_H);
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
        y + self.px(LIST_GAP * 2.0) + self.px(BAR_H)
    }

    /// The type tab under client point (`x`, `y`).
    fn tab_at(&self, x: i32, y: i32) -> Option<usize> {
        let top = self.px(HEIGHT);
        if y < top || y >= top + self.px(TABS_H) {
            return None;
        }
        self.tab_spans.iter().position(|&(l, r)| x >= l && x < r)
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
    let (w, h) = (px(WIDTH), px(HEIGHT) + px(TABS_H));
    let wa = m.work_area;
    let x = (wa.left + wa.right) / 2 - w / 2;
    let y = wa.top + ((wa.bottom - wa.top - h) as f32 * TOP_FRACTION) as i32;

    let handler: MessageHandler = {
        let queue = queue.clone();
        Box::new(
            move |hwnd: HWND, message: u32, wparam: usize, lparam: isize| -> Option<isize> {
                match message {
                    msg::WM_COMMAND if msg::hi_u16(wparam) == edit::EN_CHANGE => {
                        CHANGED_AT.with(|c| {
                            if c.get().is_none() {
                                c.set(Some(Instant::now()));
                            }
                        });
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
                        let (x, y) = (msg::lo_i16(lparam), msg::hi_i16(lparam));
                        let tab = ACTIVE.with(|a| a.try_borrow().ok()?.as_ref()?.tab_at(x, y));
                        if let Some(t) = tab {
                            set_tab(t, "click");
                            return Some(0);
                        }
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
                    WM_ROW_ICON => {
                        take_row_icons(hwnd);
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
    // In the box above the tabs.
    ctl.set_bounds(edit_x, (px(HEIGHT) - px(EDIT_H)) / 2, edit_w, px(EDIT_H));
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
            name_font: Font::new(fonts::han_text_face(), NAME_SIZE, dpi),
            detail_font: Font::new(fonts::han_text_face(), DETAIL_SIZE, dpi),
            heading_font: Font::new(fonts::han_text_face(), HEADING_SIZE, dpi),
            row_glyph_font: Font::new(fonts::icon_face(), 14.0, dpi),
            status: String::new(),
            rows: Vec::new(),
            selected: None,
            total: 0,
            row_spans: Vec::new(),
            tab: 0,
            tab_spans: Vec::new(),
            icons: HashMap::new(),
            asked: HashSet::new(),
            type_icons: HashMap::new(),
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
    let started = Instant::now();
    // The rows' icons apart from the rest (text, its layout and the fonts' fallback).
    let mut icons_time = std::time::Duration::ZERO;
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
        let line = if s.dark { 0x0040_4040 } else { 0x00E0_E0E0 };
        let accent = if s.dark { 0x00FF_C060 } else { 0x00D7_5B2A };
        // The type tabs: the chosen one in an accent-tinted capsule, the keys at the right.
        let tabs_top = box_h;
        let tint = if s.dark { 0x0060_4A33 } else { 0x00F7_E8D9 };
        // The key in the UI language (Strg+Tab); the tabs give up some of their padding before
        // it runs out of room (long labels at 640 DIPs).
        let hint = format!("{}+Tab", pecofence_core::i18n::text("键名\x04Ctrl"));
        let tabs = type_filter::TABS.len() as i32 + 1;
        let labels_w: i32 = (0..=type_filter::TABS.len())
            .map(|i| p.text_width(tab_label(i), &s.detail_font))
            .sum();
        let spare = w - px(ROW_PAD) * 2 - p.text_width(&hint, &s.detail_font) - px(8.0)
            - px(2.0) * (tabs - 1)
            - labels_w;
        // The strip takes 2 pads per tab less half a pad at the left: 2 * spare / (4 * tabs - 1).
        let pad = (2 * spare / (4 * tabs - 1)).clamp(px(3.0), px(TAB_PAD));
        let mut x = px(ROW_PAD) - pad / 2;
        s.tab_spans.clear();
        for i in 0..=type_filter::TABS.len() {
            let label = tab_label(i);
            let tw = p.text_width(label, &s.detail_font) + pad * 2;
            let r = RECT {
                left: x,
                top: tabs_top + px(6.0),
                right: x + tw,
                bottom: tabs_top + px(TABS_H - 6.0),
            };
            let on = i == s.tab;
            if on {
                p.round_rect(r, (r.bottom - r.top) / 2, tint, tint);
            }
            p.text(
                r,
                label,
                &s.detail_font,
                if on { accent } else { s.dim_color },
                Align::Center,
            );
            s.tab_spans.push((r.left, r.right));
            x = r.right + px(2.0);
        }
        let hint_room = w - px(ROW_PAD) - x - p.text_width(&hint, &s.detail_font);
        tracing::debug!(spans = ?s.tab_spans, tab = tab_label(s.tab), pad, %hint, hint_room, "search tabs drawn");
        p.text(
            RECT {
                left: x,
                top: tabs_top,
                right: w - px(ROW_PAD),
                bottom: tabs_top + px(TABS_H),
            },
            &hint,
            &s.detail_font,
            s.dim_color,
            Align::Right,
        );
        // Hairlines under the box and under the tabs.
        for y in [box_h, tabs_top + px(TABS_H) - 1] {
            p.fill(
                RECT {
                    left: 0,
                    top: y,
                    right: w,
                    bottom: y + 1.max(px(1.0)),
                },
                line,
            );
        }
        if s.rows.is_empty() {
            return;
        }
        let text_color = if s.dark { 0x00F0_F0F0 } else { 0x001E_1E1E };
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
                let r = RECT {
                    left: px(6.0),
                    top,
                    right: w - px(6.0),
                    bottom,
                };
                p.round_rect(r, px(4.0), selected_back, selected_back);
            }
            let mid = (top + bottom) / 2;
            if let Some(path) = row.icon_path.clone() {
                let is_folder = row.is_folder;
                // Its own icon reads the file (local drives; a namespace item asks its folder):
                // the worker does that, and the type's icon, which reads nothing, stands in.
                let namespace = pecofence_platform::shell::is_namespace_path(&path);
                let icon_started = Instant::now();
                if (row.exact_icon || namespace)
                    && !s.icons.contains_key(&path)
                    && s.asked.insert(path.clone())
                {
                    ICON_WORKER.with(|w| w.ask(path.clone(), is_folder, hwnd));
                }
                let ico = match s.icons.get(&path) {
                    Some(i) => i.as_ref(),
                    None if namespace => None,
                    None => s
                        .type_icons
                        .entry(type_key(&path, is_folder))
                        .or_insert_with(|| fileinfo::small_icon(&path, is_folder, false))
                        .as_ref(),
                };
                if let Some(ico) = ico {
                    p.icon(pad, mid - icon / 2, icon, ico);
                }
                icons_time += icon_started.elapsed();
            } else if let Some(g) = row.glyph {
                p.text(
                    RECT {
                        left: pad,
                        top,
                        right: pad + icon,
                        bottom,
                    },
                    g,
                    &s.row_glyph_font,
                    accent,
                    Align::Center,
                );
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
            // Only the highlighted row's: every row's read as part of its name.
            if s.selected == Some(i)
                && let Some(d) = row.shortcut
            {
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
        // The bottom bar: what the keys do, and how many results in all (more than shown).
        let y0 = h - px(BAR_H);
        p.fill(
            RECT {
                left: 0,
                top: y0,
                right: w,
                bottom: y0 + 1.max(px(1.0)),
            },
            line,
        );
        let bar = RECT {
            left: pad,
            top: y0,
            right: w - pad,
            bottom: h,
        };
        p.text(bar, &bar_keys(), &s.detail_font, s.dim_color, Align::Left);
        p.text(
            bar,
            &pecofence_core::i18n::format("{0} 个结果", &[s.total.to_string()]),
            &s.detail_font,
            s.dim_color,
            Align::Right,
        );
    });
    drop(p);
    let text_time = started.elapsed().saturating_sub(icons_time);
    log_stage("paint_text", text_time);
    log_stage("paint_icons", icons_time);
}

/// The rows for the box's last text are shown: logs how long that took from its EN_CHANGE.
pub fn rows_shown_for_change() {
    if let Some(at) = CHANGED_AT.with(|c| c.take()) {
        slow_stage("change_to_rows", at);
    }
}

/// 「↵ 打开   Ctrl+Enter 打开所在文件夹   → 更多操作」 in the UI language (the key names as the
/// actions menu writes them).
fn bar_keys() -> String {
    let open = pecofence_core::i18n::text("打开");
    let folder = pecofence_core::i18n::text("打开所在文件夹\tCtrl+Enter");
    let (folder, folder_key) = folder.split_once('\t').unwrap_or((folder, "Ctrl+Enter"));
    let more = pecofence_core::i18n::text("更多操作");
    format!("↵ {open}      {folder_key} {folder}      → {more}")
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

/// Tab `i`'s caption: 全部, then the categories.
fn tab_label(i: usize) -> &'static str {
    match i.checked_sub(1).and_then(|t| type_filter::TABS.get(t)) {
        Some(c) => type_filter::category_label(*c),
        None => pecofence_core::i18n::text("类型\x04全部"),
    }
}

/// Picks type tab `t` (`by`: click, Ctrl+Tab, Ctrl+Shift+Tab); the search runs again.
fn set_tab(t: usize, by: &str) {
    let changed = ACTIVE.with(|a| {
        let mut a = a.try_borrow_mut().ok()?;
        let s = a.as_mut()?;
        if s.tab == t {
            return None;
        }
        s.tab = t;
        paint::invalidate(s.window.hwnd());
        Some(s.queue.clone())
    });
    if let Some(queue) = changed {
        tracing::info!(tab = tab_label(t), by, "search: type tab picked");
        queue.push(Command::PeekSearchChanged);
    }
}

/// The type the tabs keep the search to; None on 全部.
pub fn type_tab() -> Option<TypeCategory> {
    ACTIVE.with(|a| {
        let t = a.try_borrow().ok()?.as_ref()?.tab;
        type_filter::TABS.get(t.checked_sub(1)?).copied()
    })
}

fn on_key(vk: u32) -> bool {
    let ctrl = window::key_down(msg::VK_CONTROL);
    // Ctrl+Tab / Ctrl+Shift+Tab: the next / previous type tab, wrapping around.
    if vk == 0x09 && ctrl {
        let back = window::key_down(0x10);
        let n = type_filter::TABS.len() + 1;
        let now = ACTIVE.with(|a| a.borrow().as_ref().map_or(0, |s| s.tab));
        let next = if back {
            (now + n - 1) % n
        } else {
            (now + 1) % n
        };
        set_tab(next, if back { "Ctrl+Shift+Tab" } else { "Ctrl+Tab" });
        return true;
    }
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
pub fn set_rows(rows: Vec<Row>, selected: Option<usize>, total: usize) {
    let resize = ACTIVE.with(|a| {
        let mut a = a.borrow_mut();
        let s = a.as_mut()?;
        s.rows = rows;
        s.total = total;
        s.selected = selected.filter(|&i| i < s.rows.len());
        // Icons of rows that went away are dropped, and the worker skips them if not read yet.
        let keep: HashSet<PathBuf> = s.rows.iter().filter_map(|r| r.icon_path.clone()).collect();
        s.icons.retain(|p, _| keep.contains(p));
        s.asked.retain(|p| keep.contains(p));
        ICON_WORKER.with(|w| {
            let mut wanted = w.wanted.lock().unwrap_or_else(|e| e.into_inner());
            wanted.retain(|p| keep.contains(p));
        });
        let h = s.layout();
        tracing::debug!(
            rows = s.rows.len(),
            headings = s.rows.iter().filter(|r| r.heading.is_some()).count(),
            results = s.rows.iter().filter(|r| r.glyph.is_none()).count(),
            action_glyphs = s.rows.iter().filter(|r| r.glyph.is_some()).count(),
            selected = ?s.selected,
            total,
            bar = !s.rows.is_empty(),
            tab = tab_label(s.tab),
            "search list laid out"
        );
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
            - s.px(TABS_H)
            - s.px(LIST_GAP * 3.0)
            - s.px(BAR_H)
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
    CHANGED_AT.with(|c| c.set(None));
    // Nothing more to read; icons read meanwhile are dropped.
    ICON_WORKER.with(|w| {
        w.wanted.lock().unwrap_or_else(|e| e.into_inner()).clear();
        w.done.lock().unwrap_or_else(|e| e.into_inner()).clear();
    });
    if let Some(s) = session {
        drop(s.edit);
        drop(s.window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each drive gets a thread of its own (a spin-up holds only its rows); the log names the
    /// drive, never the file.
    #[test]
    fn rows_icons_go_by_drive_and_type() {
        assert_eq!(drive_of(Path::new(r"F:\Games\新年.exe")), "F:");
        assert_eq!(drive_of(Path::new(r"c:\Windows\notepad.exe")), "C:");
        assert_eq!(drive_of(Path::new(r"\\nas\share\a.exe")), "network");
        assert_eq!(
            drive_of(Path::new("::{645FF040-5081-101B-9F08-00AA002F954E}")),
            "other"
        );
        assert_eq!(type_key(Path::new(r"C:\a\季度报告.PDF"), false), "pdf");
        assert_eq!(type_key(Path::new(r"C:\a\下载"), true), "<folder>");
        assert_eq!(type_key(Path::new(r"C:\a\README"), false), "");
    }
}
