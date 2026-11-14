//! The Peek search box: typing during a Peek (or pressing Ctrl twice, when that is on) opens
//! it in the upper middle of the pointer's screen, or beside a fence it would cover there (it
//! takes its place as it opens and once more when the first matches are in, see [`place`]).
//! The fences dim every icon that does not
//! match and highlight the matches (`App::update_peek_search`); a list under the box shows the
//! best matches, the fences' first and then what Everything finds on the whole computer, as
//! Listary's launcher does. Up / Down (or Ctrl+N / Ctrl+P) move through the list, which scrolls
//! when it has more results than rows (the wheel, PageUp / PageDown, Home / End too), Enter opens,
//! Ctrl+Enter opens the containing folder, Right (with the caret at the end), Ctrl+O, the menu
//! key or Shift+F10 show the actions, Ctrl+1 to Ctrl+9 open a row directly, Esc ends the Peek.
//! A right-click on a row highlights it and shows its actions at the pointer, as in Explorer;
//! a right-click on the typed text gives the text box's own menu. The row or type tab under
//! the pointer gets a hover fill (it fades in and out as a fence item's), a row also the hand:
//! a click opens it as Enter does. The highlighted row stays where the keys put it. A
//! whole-computer row whose folder is too long for the line keeps the drive and the last
//! folders (the middle gives way to "…"); the pointer resting on such a row shows the full
//! path in a tip, as Explorer's infotips.
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
use pecofence_platform::com::{StaGuard, StaSender, sta_channel};
use pecofence_platform::fileinfo::{self, FileIcon};
use pecofence_platform::paint::{self, Align, Font};
use pecofence_platform::tooltip::Tooltip;
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, StandardCursor, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{
    HWND, POINT, RECT, desktop, dwm, edit, everything, fonts, monitors, msg, sysparams,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::{Path, PathBuf};
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
/// Steps the hover fills' fades.
const TIMER_HOVER: usize = 1;
/// Shows the full path of the whole-computer row the pointer rests on.
const TIMER_TIP: usize = 2;
/// A hover fill fades in and out this long, as a fence item's (the motion tokens' fastest).
const HOVER_FADE_MS: f32 = 83.0;
/// Steps the box's own fade: in once it is drawn for the first time, out when the search ends
/// (83 ms each, as the Peek's dimmer).
const TIMER_FADE: usize = 3;
const BOX_FADE_MS: f32 = 83.0;
/// Steps the window's height to the height of a new list.
const TIMER_GLIDE: usize = 4;
/// The window takes this long to reach a new list's height (the motion tokens' fastest, as
/// the fades around it), slowing down as it arrives.
const GLIDE_MS: f32 = 83.0;
const WA_INACTIVE: usize = 0;
const VK_UP: u32 = 0x26;
const VK_DOWN: u32 = 0x28;
const VK_RIGHT: u32 = 0x27;
const VK_PRIOR: u32 = 0x21;
const VK_NEXT: u32 = 0x22;
const VK_END: u32 = 0x23;
const VK_HOME: u32 = 0x24;
/// Search in Segoe Fluent Icons / Segoe MDL2 Assets.
const GLYPH: &str = "\u{E721}";

// Layout in DIPs.
const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 56.0;
/// The box sits this far down the free height of the work area (0 = top, 0.5 = centre).
const TOP_FRACTION: f32 = 0.3;
const PAD: f32 = 18.0;
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
/// The box with a full list (nine rows under one heading and the bottom bar): the room it
/// keeps clear of the fences.
const FULL_H: f32 = HEIGHT + TABS_H + LIST_GAP + HEADING_H + 9.0 * ROW_H + LIST_GAP * 2.0 + BAR_H;
/// Space left between the box and a fence it moved away from.
const AVOID_GAP: f32 = 16.0;

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

/// What the pointer can be over: a result row or a type tab (their indices).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hover {
    Row(usize),
    Tab(usize),
}

/// A list with more results than it shows: where the rows shown are in them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListScroll {
    /// Index of the first result shown, how many are shown and how many there are.
    pub first: usize,
    pub shown: usize,
    pub of: usize,
    /// Group headings of the results in all: the list keeps room for them, so it keeps its
    /// height when one scrolls out.
    pub headings: usize,
}

thread_local! {
    static ACTIVE: RefCell<Option<Session>> = const { RefCell::new(None) };
    /// The box of the search that just ended, while it fades out.
    static LEAVING: RefCell<Option<Session>> = const { RefCell::new(None) };
    /// A box made when the Peek started, hidden, for its first letter (`prepare`): a window
    /// that is new at the letter comes on screen a refresh later than one that only has to
    /// be shown. It is no open box: nothing but `open` and `drop_prepared` looks at it.
    static PREPARED: RefCell<Option<Session>> = const { RefCell::new(None) };
    /// The box made ahead is getting its first text (`adopt_prepared`): no change to report.
    static ADOPTING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
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
    threads: RefCell<HashMap<String, StaSender<(PathBuf, bool, isize)>>>,
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
            tx.send((path, is_folder, hwnd.0 as isize));
        }
    }

    fn spawn(&self, drive: &str) -> Option<StaSender<(PathBuf, bool, isize)>> {
        let (tx, rx) = sta_channel::<(PathBuf, bool, isize)>();
        let (w, d) = (self.wanted.clone(), self.done.clone());
        let spawned = std::thread::Builder::new()
            .name(format!("pecofence-peek-icons-{drive}"))
            .spawn(move || {
                let _sta = StaGuard::init();
                loop {
                    let (path, is_folder, hwnd) = rx.recv();
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
    /// The work area of its screen and its usual place there (the upper middle).
    work: RECT,
    home: (i32, i32),
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
    /// The room (px) the status has at the right of the text box: `STATUS_W`, or what a
    /// longer text needs.
    status_room: i32,
    rows: Vec<Row>,
    selected: Option<usize>,
    /// How many results the search has in all (more than the rows shown).
    total: usize,
    /// Top and bottom (client pixels) of each row, from the last layout.
    row_spans: Vec<(i32, i32)>,
    /// The list scrolls (more results than rows), top and bottom of its result rows (the
    /// thumb's track), and wheel movement not yet a whole row.
    scroll: Option<ListScroll>,
    list_span: (i32, i32),
    wheel: i32,
    /// The type tab: 0 全部, else `type_filter::TABS[tab - 1]`; and each tab's left and right
    /// (client pixels) as last drawn.
    tab: usize,
    tab_spans: Vec<(i32, i32)>,
    /// Rows' own icons, read by the worker; `asked` are on their way.
    icons: HashMap<PathBuf, Option<FileIcon>>,
    asked: HashSet<PathBuf>,
    /// Types' icons by extension (or folder), which read no file.
    type_icons: HashMap<String, Option<FileIcon>>,
    /// The row or tab under the pointer and how far its fill has faded in (0.0 to 1.0); the
    /// one the pointer left and how much of its fill is still there; when the fades last moved.
    hover: Option<Hover>,
    hover_level: f32,
    hover_out: Option<(Hover, f32)>,
    hover_tick: Instant,
    /// The tip with a whole-computer row's full path (made when first needed).
    tip: Option<Tooltip>,
    /// The box has been drawn once and shows (it opens invisible: before its first paint only
    /// the text field would show, a bare white strip).
    revealed: bool,
    /// How opaque the window is (0.0 to 1.0), where its fade is heading and when it last moved.
    alpha: f32,
    alpha_to: f32,
    alpha_tick: Instant,
    /// The height the rows ask for (`layout`): the window has it at rest and glides to it.
    list_h: i32,
    /// The window is on its way to `list_h`: the height it left and when (`step_glide`).
    glide: Option<(i32, Instant)>,
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
        // A list that scrolls keeps the height of its results: the rows shown and every
        // heading they have, also while one of the headings is scrolled out.
        let results_end = self.scroll.map_or(y, |sc| {
            y + self.px(ROW_H) * sc.shown as i32 + self.px(HEADING_H) * sc.headings as i32
        });
        self.list_span = (y, results_end);
        let mut results_done = false;
        let mut spans = Vec::with_capacity(self.rows.len());
        for row in &self.rows {
            // The action rows come under the results.
            if row.glyph.is_some() && !results_done {
                results_done = true;
                y = y.max(results_end);
            }
            if row.heading.is_some() {
                y += self.px(HEADING_H);
            }
            let top = y;
            y += self.px(ROW_H);
            spans.push((top, y));
        }
        self.row_spans = spans;
        if !results_done {
            y = y.max(results_end);
        }
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

    /// The highlight is on the first row of the whole list.
    fn on_first_row(&self) -> bool {
        self.selected == Some(0) && self.scroll.is_none_or(|sc| sc.first == 0)
    }

    /// The tab or row under client point (`x`, `y`).
    fn target_at(&self, x: i32, y: i32) -> Option<Hover> {
        self.tab_at(x, y)
            .map(Hover::Tab)
            .or_else(|| self.row_at(y).map(Hover::Row))
    }

    /// Asks for a repaint of `target`'s row, or of the tab strip (`w`: the window's width).
    fn invalidate_target(&self, target: Hover, w: i32) {
        let (top, bottom) = match target {
            Hover::Row(i) => match self.row_spans.get(i) {
                Some(&span) => span,
                None => return,
            },
            Hover::Tab(_) => (self.px(HEIGHT), self.px(HEIGHT) + self.px(TABS_H)),
        };
        paint::invalidate_rect(
            self.window.hwnd(),
            RECT {
                left: 0,
                top,
                right: w,
                bottom,
            },
        );
    }
}

/// A whole-computer row: the line under its name is the file's folder.
fn folder_detail(row: &Row) -> bool {
    row.icon_path
        .as_deref()
        .and_then(Path::parent)
        .is_some_and(|folder| folder.to_string_lossy() == row.detail)
}

/// A folder path wider than `room`: its middle gives way to "…", so the drive (or the server
/// of a network path) and as many of the last folders as fit stay, the last one always (the
/// end of the path is where results from one tree differ). `width` measures a text.
fn fit_path(path: &str, room: i32, width: impl Fn(&str) -> i32) -> String {
    if width(path) <= room {
        return path.to_string();
    }
    let parts: Vec<&str> = path.split('\\').collect();
    // "C:", or "\\server" (which splits into "", "", "server").
    let head = if path.starts_with(r"\\") { 3 } else { 1 };
    if parts.len() < head + 2 {
        return path.to_string();
    }
    let shown = |last: usize| {
        format!(
            "{}\\…\\{}",
            parts[..head].join("\\"),
            parts[parts.len() - last..].join("\\")
        )
    };
    let mut last = 1;
    while head + last + 1 < parts.len() && width(&shown(last + 1)) <= room {
        last += 1;
    }
    shown(last)
}

/// The pointer came to another row or left: the path tip goes, and a whole-computer row gets
/// one after the time Explorer waits for its infotips.
fn restart_tip(s: &mut Session, hwnd: HWND) {
    if let Some(tip) = s.tip.as_mut() {
        tip.hide();
    }
    window::kill_timer(hwnd, TIMER_TIP);
    if let Some(Hover::Row(i)) = s.hover
        && s.rows.get(i).is_some_and(folder_detail)
    {
        window::set_timer(hwnd, TIMER_TIP, Tooltip::hover_delay_ms());
    }
}

/// `TIMER_TIP`: the pointer rested on a whole-computer row: its full path under the pointer.
fn show_path_tip(hwnd: HWND) {
    window::kill_timer(hwnd, TIMER_TIP);
    ACTIVE.with(|a| {
        let Ok(mut a) = a.try_borrow_mut() else {
            return;
        };
        let Some(s) = a.as_mut() else { return };
        let Some(Hover::Row(i)) = s.hover else { return };
        let Some(path) = s
            .rows
            .get(i)
            .filter(|r| folder_detail(r))
            .and_then(|r| r.icon_path.clone())
        else {
            return;
        };
        if s.tip.is_none() {
            s.tip = Tooltip::create(hwnd, s.dark)
                .map_err(|e| tracing::warn!(error = %e, "search: no tip window for the path"))
                .ok();
        }
        let pt = window::cursor_pos();
        let (below, above) = (s.px(22.0), s.px(4.0));
        if let Some(tip) = s.tip.as_mut() {
            tip.show(&path.to_string_lossy(), pt.x, pt.y + below, pt.y - above);
            tracing::debug!(row = i, "search: the row's full path is shown in a tip");
        }
    });
}

/// `back` with `over` laid on it at `level` (0.0 nothing of it, 1.0 all of it), per channel.
fn mix(back: u32, over: u32, level: f32) -> u32 {
    let channel = |shift: u32| {
        let (b, o) = (
            ((back >> shift) & 0xFF) as f32,
            ((over >> shift) & 0xFF) as f32,
        );
        ((b + (o - b) * level).round() as u32) << shift
    };
    channel(0) | channel(8) | channel(16)
}

/// The pointer is at client point `at` (`None`: it left the box). The row or tab under it gets
/// the hover fill, fading in; the one it left fades out (both at once with Windows' animations
/// off). The highlighted row is not touched.
fn set_hover(hwnd: HWND, at: Option<(i32, i32)>) {
    ACTIVE.with(|a| {
        let Ok(mut a) = a.try_borrow_mut() else {
            return;
        };
        let Some(s) = a.as_mut() else { return };
        let target = at.and_then(|(x, y)| s.target_at(x, y));
        if target == s.hover {
            return;
        }
        let (w, _) = window::window_rect_size(hwnd);
        let animate = sysparams::client_area_animation();
        // Back on the one still fading out: on from what is left of its fill.
        let from = s
            .hover_out
            .filter(|(t, _)| Some(*t) == target)
            .map_or(0.0, |(_, level)| level);
        let left = s.hover.map(|t| (t, s.hover_level));
        for t in [left, s.hover_out].into_iter().flatten().map(|(t, _)| t) {
            s.invalidate_target(t, w);
        }
        s.hover = target;
        if let Some(t) = target {
            s.invalidate_target(t, w);
        }
        if animate {
            s.hover_out = left.filter(|(t, level)| *level > 0.0 && Some(*t) != target);
            s.hover_level = from;
            s.hover_tick = Instant::now();
            window::set_timer(hwnd, TIMER_HOVER, 15);
        } else {
            s.hover_out = None;
            s.hover_level = 1.0;
        }
        restart_tip(s, hwnd);
        tracing::debug!(over = ?target, left = ?left.map(|(t, _)| t), animate, "search: the pointer is over another row or tab");
    });
}

/// `TIMER_HOVER`: the hover fills a step further in their fades; the timer ends with them.
fn step_hover(hwnd: HWND) {
    ACTIVE.with(|a| {
        let Ok(mut a) = a.try_borrow_mut() else {
            return;
        };
        let Some(s) = a.as_mut() else { return };
        let now = Instant::now();
        let step = now.duration_since(s.hover_tick).as_secs_f32() * 1000.0 / HOVER_FADE_MS;
        s.hover_tick = now;
        let (w, _) = window::window_rect_size(hwnd);
        if let Some(t) = s.hover
            && s.hover_level < 1.0
        {
            s.hover_level = (s.hover_level + step).min(1.0);
            s.invalidate_target(t, w);
        }
        if let Some((t, level)) = s.hover_out {
            s.hover_out = (level > step).then_some((t, level - step));
            s.invalidate_target(t, w);
        }
        if s.hover_out.is_none() && (s.hover.is_none() || s.hover_level >= 1.0) {
            window::kill_timer(hwnd, TIMER_HOVER);
        }
    });
}

/// The box has been drawn: it shows from now on, fading in over [`BOX_FADE_MS`] (at once with
/// Windows' animations off). Only the first paint of a box does anything here.
fn reveal(hwnd: HWND) {
    ACTIVE.with(|a| {
        let Ok(mut a) = a.try_borrow_mut() else {
            return;
        };
        let Some(s) = a.as_mut() else { return };
        if std::mem::replace(&mut s.revealed, true) {
            return;
        }
        let animate = sysparams::client_area_animation();
        tracing::debug!(animate, "search box: drawn once, it shows");
        s.alpha_to = 1.0;
        if animate {
            // The fade starts at a fifth of its strength, set here: left at 0 until the fade's
            // timer first ran (15 to 31 ms later), the box missed the refresh it was drawn
            // for, and came one refresh later than it did before it learned to fade.
            s.alpha = s.alpha.max(pecofence_render::motion::FADE_IN_FIRST);
            window::set_layered_alpha(hwnd, (s.alpha * 255.0).round() as u8);
            tracing::debug!(alpha = s.alpha, next_refresh = ?dwm::next_refresh_ms(), "search box: shows");
            s.alpha_tick = Instant::now();
            window::set_timer(hwnd, TIMER_FADE, 15);
        } else {
            s.alpha = 1.0;
            window::set_layered_alpha(hwnd, 255);
        }
    });
}

/// `TIMER_FADE`: the box a step further in its fade, the open one's fade in or the fade out
/// of the one a search left behind. The timer ends with the fade; a box that has faded out is
/// hidden, and the app is asked to drop it.
fn step_fade(hwnd: HWND) {
    let step = |s: &mut Session| -> bool {
        let now = Instant::now();
        let d = now.duration_since(s.alpha_tick).as_secs_f32() * 1000.0 / BOX_FADE_MS;
        s.alpha_tick = now;
        s.alpha = if s.alpha_to > s.alpha {
            (s.alpha + d).min(s.alpha_to)
        } else {
            (s.alpha - d).max(s.alpha_to)
        };
        window::set_layered_alpha(hwnd, (s.alpha * 255.0).round() as u8);
        s.alpha == s.alpha_to
    };
    let open = ACTIVE.with(|a| {
        let Ok(mut a) = a.try_borrow_mut() else {
            return None;
        };
        let s = a.as_mut().filter(|s| s.window.hwnd() == hwnd)?;
        Some(step(s))
    });
    let left = match open {
        Some(_) => None,
        None => LEAVING.with(|l| {
            let mut l = l.try_borrow_mut().ok()?;
            let s = l.as_mut().filter(|s| s.window.hwnd() == hwnd)?;
            Some((step(s), s.queue.clone()))
        }),
    };
    match (open, left) {
        (Some(false), _) | (None, Some((false, _))) => {}
        (None, Some((true, queue))) => {
            window::kill_timer(hwnd, TIMER_FADE);
            desktop::hide_window(hwnd);
            // The app drops it: a window cannot be destroyed from its own timer message.
            queue.push(Command::PeekSearchFaded);
        }
        _ => window::kill_timer(hwnd, TIMER_FADE),
    }
}

/// The box that faded out is dropped (the app calls this for [`Command::PeekSearchFaded`]).
pub fn drop_faded() {
    drop(LEAVING.with(|l| l.borrow_mut().take_if(|s| s.alpha <= 0.0)));
}

/// Whether a box `w` x `h` at (`x`, `y`) covers any of `avoid`.
fn covers(x: i32, y: i32, w: i32, h: i32, avoid: &[RECT]) -> bool {
    avoid
        .iter()
        .any(|r| x < r.right && r.left < x + w && y < r.bottom && r.top < y + h)
}

/// Where the box goes: `home`, its usual place, unless it would cover one of `avoid` there
/// (fences, screen pixels). Then the nearest place beside, above or below one of them, `gap`
/// away, where the box (`w` x `h`, its list included) covers none and lies in the work area
/// `wa`; `home` after all when there is no such place.
fn place(home: (i32, i32), w: i32, h: i32, wa: RECT, avoid: &[RECT], gap: i32) -> (i32, i32) {
    if !covers(home.0, home.1, w, h, avoid) {
        return home;
    }
    let mut best: Option<((i32, i32), i64)> = None;
    for r in avoid {
        for (x, y) in [
            (r.left - gap - w, home.1),
            (r.right + gap, home.1),
            (home.0, r.top - gap - h),
            (home.0, r.bottom + gap),
        ] {
            let inside = x >= wa.left && x + w <= wa.right && y >= wa.top && y + h <= wa.bottom;
            if !inside || covers(x, y, w, h, avoid) {
                continue;
            }
            let (dx, dy) = ((x - home.0) as i64, (y - home.1) as i64);
            let d = dx * dx + dy * dy;
            if best.is_none_or(|(_, b)| d < b) {
                best = Some(((x, y), d));
            }
        }
    }
    best.map_or(home, |(to, _)| to)
}

/// Opens the box with `text` typed in and gives it the keyboard. `avoid`: the fences in
/// sight (screen pixels); the box opens beside them when its usual place would cover one.
pub fn open(text: &str, queue: CommandQueue, dark: bool, avoid: &[RECT]) {
    close();
    // A box still fading out, or hidden since its fade: gone before this one is made.
    drop(LEAVING.with(|l| l.borrow_mut().take()));
    let ready = adopt_prepared(text, dark, avoid);
    if !ready {
        let Some(s) = build(text, queue, dark, avoid) else {
            return;
        };
        ACTIVE.with(|a| *a.borrow_mut() = Some(s));
    }
    let Some(hwnd) = hwnd() else { return };
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
    if ready {
        // Drawn now, with the letter and before its first list, so that it is on screen
        // with the refresh that follows the letter (`reveal`).
        window::paint_now(hwnd);
    }
}

/// Makes the box's window before the first letter, hidden and without the keyboard, when the
/// Peek has started: `open` then only shows it. `avoid` as for `open`.
pub fn prepare(queue: CommandQueue, dark: bool, avoid: &[RECT]) {
    if is_open() {
        return;
    }
    let made = build("", queue, dark, avoid);
    tracing::debug!(
        made = made.is_some(),
        "search box: made ahead of the first letter"
    );
    drop(PREPARED.with(|p| p.replace(made)));
}

/// The box made ahead is not needed (the Peek ended without a letter): it goes unseen.
pub fn drop_prepared() {
    if let Some(s) = PREPARED.with(|p| p.borrow_mut().take()) {
        tracing::debug!("search box: the one made ahead was not needed");
        drop(s);
    }
}

/// Takes the box made ahead for the search that opens now with `text`, if it was made for the
/// screen the pointer is on and the colour mode of now. Its place is taken again by the rule
/// of `open`, with the fences as they stand now.
fn adopt_prepared(text: &str, dark: bool, avoid: &[RECT]) -> bool {
    let Some(mut s) = PREPARED.with(|p| p.borrow_mut().take()) else {
        return false;
    };
    let pt = window::cursor_pos();
    let same_screen = monitors::query(monitors::monitor_from_point(pt.x, pt.y))
        .is_some_and(|m| m.work_area == s.work && m.dpi.max(96) == s.dpi);
    if !same_screen || s.dark != dark {
        tracing::debug!(same_screen, "search box: the one made ahead no longer fits");
        return false;
    }
    let full = s.px(FULL_H).min(s.work.bottom - s.home.1);
    let (x, y) = place(s.home, s.w, full, s.work, avoid, s.px(AVOID_GAP));
    let moved = (x, y) != s.home;
    tracing::debug!(
        home = ?s.home,
        x,
        y,
        fences = avoid.len(),
        moved,
        ready = true,
        "search box: place taken as it opens"
    );
    if (x, y) != (s.x, s.y) {
        (s.x, s.y) = (x, y);
        let (_, h) = window::window_rect_size(s.window.hwnd());
        let _ = s.window.set_bounds(x, y, s.w, h);
    }
    ACTIVE.with(|a| *a.borrow_mut() = Some(s));
    ADOPTING.with(|a| a.set(true));
    set_text(text);
    ADOPTING.with(|a| a.set(false));
    true
}

/// The box's window, its text field and its session for `text`, not shown yet.
fn build(text: &str, queue: CommandQueue, dark: bool, avoid: &[RECT]) -> Option<Session> {
    CLASS.with(|c| {
        if c.borrow().is_none() {
            *c.borrow_mut() = WindowClass::register(SEARCH_CLASS, ClassOptions::default()).ok();
        }
    });
    // On the screen the pointer is on, where the user is looking.
    let pt = window::cursor_pos();
    let m = monitors::query(monitors::monitor_from_point(pt.x, pt.y))?;
    let dpi = m.dpi.max(96);
    let px = |dip: f32| (dip * dpi as f32 / 96.0).round() as i32;
    let (w, h) = (px(WIDTH), px(HEIGHT) + px(TABS_H));
    let wa = m.work_area;
    let home = (
        (wa.left + wa.right) / 2 - w / 2,
        wa.top + ((wa.bottom - wa.top - h) as f32 * TOP_FRACTION) as i32,
    );
    let full = px(FULL_H).min(wa.bottom - home.1);
    let (x, y) = place(home, w, full, wa, avoid, px(AVOID_GAP));
    tracing::debug!(
        ?home,
        x,
        y,
        fences = avoid.len(),
        moved = (x, y) != home,
        "search box: place taken as it opens"
    );

    let handler: MessageHandler = {
        let queue = queue.clone();
        Box::new(
            move |hwnd: HWND, message: u32, wparam: usize, lparam: isize| -> Option<isize> {
                match message {
                    msg::WM_COMMAND if msg::hi_u16(wparam) == edit::EN_CHANGE => {
                        if ADOPTING.with(|a| a.get()) {
                            return Some(0);
                        }
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
                    // The accent colour changed while the box is open: drawn again with it.
                    msg::WM_SETTINGCHANGE
                    | msg::WM_THEMECHANGED
                    | msg::WM_DWMCOLORIZATIONCOLORCHANGED => {
                        tracing::debug!(message, "search: colours may have changed, drawn again");
                        paint::invalidate(hwnd);
                        None
                    }
                    msg::WM_ERASEBKGND => Some(1),
                    msg::WM_PAINT => {
                        paint_box(hwnd);
                        Some(0)
                    }
                    msg::WM_MOUSEMOVE => {
                        set_hover(hwnd, Some((msg::lo_i16(lparam), msg::hi_i16(lparam))));
                        window::track_mouse_leave(hwnd);
                        Some(0)
                    }
                    msg::WM_MOUSELEAVE => {
                        set_hover(hwnd, None);
                        Some(0)
                    }
                    msg::WM_TIMER if wparam == TIMER_HOVER => {
                        step_hover(hwnd);
                        Some(0)
                    }
                    msg::WM_TIMER if wparam == TIMER_TIP => {
                        show_path_tip(hwnd);
                        Some(0)
                    }
                    msg::WM_TIMER if wparam == TIMER_FADE => {
                        step_fade(hwnd);
                        Some(0)
                    }
                    msg::WM_TIMER if wparam == TIMER_GLIDE => {
                        step_glide(hwnd);
                        Some(0)
                    }
                    // The hand over a result row, which a click opens (as over a script
                    // fence's clickable row); the arrow over a tab, as over a fence's tabs.
                    // The text box inside asks here too and keeps its own cursor. Windows asks
                    // before it sends the move, so the row is looked up at the pointer.
                    msg::WM_SETCURSOR
                        if wparam == hwnd.0 as usize
                            && msg::lo_i16(lparam) as isize == msg::HTCLIENT =>
                    {
                        let pt = window::screen_to_client(hwnd, window::cursor_pos());
                        let on_row = ACTIVE.with(|a| {
                            a.try_borrow().is_ok_and(|a| {
                                a.as_ref().is_some_and(|s| {
                                    matches!(s.target_at(pt.x, pt.y), Some(Hover::Row(_)))
                                })
                            })
                        });
                        if !on_row {
                            return None;
                        }
                        window::set_standard_cursor(StandardCursor::Hand);
                        Some(1)
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
                    // The wheel scrolls a list that has more results than rows, three rows a
                    // notch (a touchpad's small steps add up). Over the text box it comes here
                    // too: the box passes it on.
                    msg::WM_MOUSEWHEEL => {
                        let delta = msg::hi_u16(wparam) as u16 as i16 as i32;
                        let rows = ACTIVE.with(|a| {
                            let mut a = a.try_borrow_mut().ok()?;
                            let s = a.as_mut()?;
                            s.scroll?;
                            s.wheel += delta;
                            let rows = s.wheel / (msg::WHEEL_DELTA / 3);
                            s.wheel -= rows * (msg::WHEEL_DELTA / 3);
                            (rows != 0).then_some(rows)
                        });
                        if let Some(rows) = rows {
                            queue.push(Command::PeekSearchScroll(-rows));
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
            .ex_style(style::EX_TOOLWINDOW | style::EX_TOPMOST | style::EX_LAYERED)
            .bounds(x, y, w, h)
            .create(handler)
            .ok()
    });
    let window = created?;
    // Invisible until it has been drawn once (`reveal`): it takes the keyboard meanwhile.
    window::set_layered_alpha(window.hwnd(), 0);
    let _ = dwm::set_corner_preference(window.hwnd(), dwm::CornerPreference::Round);
    // The typed text starts where the rows' names start, the magnifier sits over their icons.
    let edit_x = px(ROW_PAD) + px(ICON) + px(12.0);
    let edit_w = w - edit_x - px(STATUS_W + PAD + 8.0);
    let Ok(mut ctl) = edit::EditControl::create(
        window.hwnd(),
        edit_w,
        px(EDIT_H),
        dpi,
        TEXT_SIZE,
        text,
        dark,
        on_key,
    ) else {
        return None;
    };
    // The list's font: Chinese typed here came out of GDI's fallback face on a Windows in
    // another language, beside names in Microsoft YaHei UI.
    ctl.set_font(fonts::han_text_face(), TEXT_SIZE, dpi);
    // No margins of its own: the text starts at the control's left edge.
    ctl.set_margins(0);
    // In the box above the tabs.
    ctl.set_bounds(edit_x, (px(HEIGHT) - px(EDIT_H)) / 2, edit_w, px(EDIT_H));
    Some(Session {
        window,
        edit: ctl,
        queue,
        dpi,
        dark,
        x,
        y,
        w,
        work_bottom: wa.bottom,
        work: wa,
        home,
        dim_color: if dark { 0x00C5_C5C5 } else { 0x005F_5F5F },
        glyph_font: Font::new(fonts::icon_face(), GLYPH_SIZE, dpi),
        status_font: Font::new(fonts::text_face(), STATUS_SIZE, dpi),
        name_font: Font::new(fonts::han_text_face(), NAME_SIZE, dpi),
        detail_font: Font::new(fonts::han_text_face(), DETAIL_SIZE, dpi),
        heading_font: Font::new(fonts::han_text_face(), HEADING_SIZE, dpi),
        row_glyph_font: Font::new(fonts::icon_face(), 14.0, dpi),
        status: String::new(),
        status_room: px(STATUS_W),
        rows: Vec::new(),
        selected: None,
        total: 0,
        row_spans: Vec::new(),
        scroll: None,
        list_span: (0, 0),
        wheel: 0,
        tab: 0,
        tab_spans: Vec::new(),
        icons: HashMap::new(),
        asked: HashSet::new(),
        type_icons: HashMap::new(),
        hover: None,
        hover_level: 0.0,
        hover_out: None,
        hover_tick: Instant::now(),
        tip: None,
        revealed: false,
        alpha: 0.0,
        alpha_to: 0.0,
        alpha_tick: Instant::now(),
        list_h: h,
        glide: None,
    })
}

fn paint_box(hwnd: HWND) {
    let started = Instant::now();
    // The rows' icons apart from the rest (text, its layout and the fonts' fallback).
    let mut icons_time = std::time::Duration::ZERO;
    // Into a memory bitmap, shown in one step: the first list of a Windows session takes over
    // a tenth of a second to draw (the fonts are cold), and its rows appeared one by one.
    let mut p = paint::Paint::begin_buffered(hwnd);
    let mut drawn = false;
    ACTIVE.with(|a| {
        let Ok(mut a) = a.try_borrow_mut() else {
            return;
        };
        let Some(s) = a.as_mut() else { return };
        drawn = true;
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
                left: px(ROW_PAD),
                top: 0,
                right: px(ROW_PAD) + px(ICON),
                bottom: box_h,
            },
            GLYPH,
            &s.glyph_font,
            s.dim_color,
            Align::Center,
        );
        // The status takes the room its text needs: 「Aucun fichier correspondant」 is wider
        // than 「没有匹配的文件」 and lost its end in 170 DIPs. The text box ends before it.
        let status_w = px(STATUS_W).max(p.text_width(&s.status, &s.status_font) + px(8.0));
        if status_w != s.status_room {
            s.status_room = status_w;
            let edit_x = px(ROW_PAD) + px(ICON) + px(12.0);
            s.edit.set_bounds(
                edit_x,
                (px(HEIGHT) - px(EDIT_H)) / 2,
                w - edit_x - status_w - px(PAD + 8.0),
                px(EDIT_H),
            );
            tracing::debug!(status_w, "search box: the text box ends before the status");
        }
        p.text(
            RECT {
                left: w - px(PAD) - status_w,
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
        // The Windows accent, as the fences mark the same matches with it (Dark1 on light,
        // Light2 on dark): the matched letters and the action glyphs; the chosen tab's capsule
        // and the highlighted row are the background tinted with it. The blue it had before
        // when the accent cannot be read.
        let (accent, tint, selected_back) = match pecofence_platform::theme::accent_palette() {
            Some(c) => {
                let rgb = if s.dark { c.light2 } else { c.dark1 };
                let accent = u32::from_le_bytes([rgb[0], rgb[1], rgb[2], 0]);
                let tint = mix(back, accent, if s.dark { 0.2 } else { 0.14 });
                (accent, tint, mix(back, accent, 0.1))
            }
            None if s.dark => (0x00FF_C060, 0x0060_4A33, 0x0048_3A2E),
            None => (0x00D7_5B2A, 0x00F7_E8D9, 0x00FA_EBE3),
        };
        // The type tabs: the chosen one in an accent-tinted capsule, the keys at the right.
        let tabs_top = box_h;
        // The fill under the pointer: the fences' subtle hover (WinUI's: 6 % white on dark,
        // 3.5 % black on light) at the strength its fade has reached.
        let hover_back = if s.dark {
            mix(back, 0x00FF_FFFF, 15.0 / 255.0)
        } else {
            mix(back, 0, 9.0 / 255.0)
        };
        let (hover, hover_level, hover_out) = (s.hover, s.hover_level, s.hover_out);
        let hover_of = |t: Hover| match hover_out {
            _ if hover == Some(t) => hover_level,
            Some((out, level)) if out == t => level,
            _ => 0.0,
        };
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
            let hovered = hover_of(Hover::Tab(i));
            if on {
                p.round_rect(r, (r.bottom - r.top) / 2, tint, tint);
            } else if hovered > 0.0 {
                let fill = mix(back, hover_back, hovered);
                p.round_rect(r, (r.bottom - r.top) / 2, fill, fill);
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
        // An outline, as the filter box and the rename box have: with the dimming off the
        // box had no edge over a white window (nor on Windows 10, where nothing rounds or
        // shades it).
        let border = if s.dark { 0x0050_5050 } else { 0x00C8_C8C8 };
        let t = 1.max(px(1.0));
        for (left, top, right, bottom) in [
            (0, 0, w, t),
            (0, h - t, w, h),
            (0, 0, t, h),
            (w - t, 0, w, h),
        ] {
            p.fill(
                RECT {
                    left,
                    top,
                    right,
                    bottom,
                },
                border,
            );
        }
        if s.rows.is_empty() {
            return;
        }
        let text_color = if s.dark { 0x00F0_F0F0 } else { 0x001E_1E1E };
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
            // The highlighted row keeps its stronger fill under the pointer too.
            let hovered = hover_of(Hover::Row(i));
            if s.selected == Some(i) || hovered > 0.0 {
                let r = RECT {
                    left: px(6.0),
                    top,
                    right: w - px(6.0),
                    bottom,
                };
                let fill = if s.selected == Some(i) {
                    selected_back
                } else {
                    mix(back, hover_back, hovered)
                };
                p.round_rect(r, px(4.0), fill, fill);
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
            // The line under the name runs to the right edge; only the highlighted row keeps
            // room for its Ctrl+digit. A folder too long for it is cut in the middle.
            let detail_right = if s.selected == Some(i) && row.shortcut.is_some() {
                text_right
            } else {
                w - pad
            };
            let detail = if folder_detail(row) {
                fit_path(&row.detail, detail_right - text_left, |t| {
                    p.text_width(t, &s.detail_font)
                })
            } else {
                row.detail.clone()
            };
            p.text(
                RECT {
                    left: text_left,
                    top: mid,
                    right: detail_right,
                    bottom: bottom - px(4.0),
                },
                &detail,
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
        // More results than rows: a thumb at the right edge, where the rows shown are in them
        // (the fences' resting scrollbar: a thin pill, no track).
        if let Some(sc) = s.scroll {
            let (top, bottom) = s.list_span;
            let track = bottom - top;
            let thumb = (track * sc.shown as i32 / sc.of.max(1) as i32)
                .max(px(16.0))
                .min(track);
            let steps = sc.of.saturating_sub(sc.shown).max(1) as i32;
            let y = top + (track - thumb) * (sc.first as i32).min(steps) / steps;
            let color = if s.dark { 0x00A3_A3A3 } else { 0x008B_8B8B };
            p.round_rect(
                RECT {
                    left: w - px(5.0),
                    top: y,
                    right: w - px(2.0),
                    bottom: y + thumb,
                },
                px(1.5),
                color,
                color,
            );
        }
        // The bottom bar: what the keys do, and how many results in all (more than shown).
        // It ends the list, under the last row. While the window is on its way to a shorter
        // list's height the bar is at its final place and the window's edge comes up to it;
        // on the way to a longer list it stays at the window's edge, over the rows the edge
        // has not reached yet, so the box ends with its bar in every frame.
        let bar_bottom = h.min(s.list_h);
        let y0 = bar_bottom - px(BAR_H);
        let edge = 1.max(px(1.0));
        if bar_bottom < s.list_h {
            p.fill(
                RECT {
                    left: edge,
                    top: y0,
                    right: w - edge,
                    bottom: bar_bottom - edge,
                },
                back,
            );
        }
        p.fill(
            RECT {
                left: 0,
                top: y0,
                right: w,
                bottom: y0 + edge,
            },
            line,
        );
        let bar = RECT {
            left: pad,
            top: y0,
            right: w - pad,
            bottom: bar_bottom,
        };
        // Ctrl+Enter and the actions need a file: the two action rows and system items (此电脑)
        // have none, and the keys do nothing there.
        let file_row = s.selected.and_then(|i| s.rows.get(i)).is_some_and(|r| {
            r.glyph.is_none()
                && r.icon_path
                    .as_deref()
                    .is_some_and(|p| !pecofence_platform::shell::is_namespace_path(p))
        });
        p.text(
            bar,
            &bar_keys(file_row),
            &s.detail_font,
            s.dim_color,
            Align::Left,
        );
        p.text(
            bar,
            &pecofence_core::i18n::format("{0} 个结果", &[s.total.to_string()]),
            &s.detail_font,
            s.dim_color,
            Align::Right,
        );
    });
    if !drawn {
        p.keep_window();
    }
    drop(p);
    if drawn {
        reveal(hwnd);
    }
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

/// 「Enter 打开   Ctrl+Enter 打开所在文件夹   → 更多操作」 in the UI language (the key names as the
/// actions menu writes them; Enter by its name, as the other keys: the ↵ sign came out of a
/// fallback font about 5 px high). Only 「Enter 打开」 when the highlighted row has no file
/// (`file_row` false).
fn bar_keys(file_row: bool) -> String {
    let enter = pecofence_core::i18n::text("键名\x04Enter");
    let open = pecofence_core::i18n::text("打开");
    if !file_row {
        return format!("{enter} {open}");
    }
    let folder = pecofence_core::i18n::text("打开所在文件夹\tCtrl+Enter");
    let (folder, folder_key) = folder.split_once('\t').unwrap_or((folder, "Ctrl+Enter"));
    let more = pecofence_core::i18n::text("更多操作");
    format!("{enter} {open}      {folder_key} {folder}      → {more}")
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
    // Not with Alt down: Windows reports AltGr as Ctrl and Alt, and AltGr with a digit or a
    // letter types a character on many layouts ([ ] { on the German one, @ # on the Spanish
    // one, ó ń on the Polish one). Those are text for the box, not Ctrl+8 or Ctrl+O.
    let ctrl = window::key_down(msg::VK_CONTROL) && !window::key_down(msg::VK_MENU);
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
        VK_PRIOR => Command::PeekSearchPage(-1),
        VK_NEXT => Command::PeekSearchPage(1),
        // Home / End: the first / last row of the list. Ctrl+Home / Ctrl+End always; without
        // Ctrl the text box comes first: Home moves the caret while the highlight is on the
        // first row, End moves it unless it already is at the end of the text. Shift+Home /
        // Shift+End select text.
        VK_HOME | VK_END => {
            let home = vk == VK_HOME;
            let list = ACTIVE.with(|a| {
                a.borrow().as_ref().is_some_and(|s| {
                    !s.rows.is_empty()
                        && (ctrl
                            || if home {
                                !s.on_first_row()
                            } else {
                                s.edit.caret_at_end()
                            })
                })
            });
            if !list || window::key_down(msg::VK_SHIFT) {
                return false;
            }
            Command::PeekSearchPage(if home { i32::MIN } else { i32::MAX })
        }
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

/// The hint in the empty box, in the colour of the glyph beside it: what can be searched
/// (`whole_pc`: Everything answers too).
pub fn set_cue(whole_pc: bool) {
    ACTIVE.with(|a| {
        if let Some(s) = a.borrow().as_ref() {
            let hint = if whole_pc {
                pecofence_core::i18n::text("搜索栅栏和整台电脑，可以用拼音")
            } else {
                pecofence_core::i18n::text("搜索栅栏里的文件，可以用拼音")
            };
            s.edit.set_hint(hint, s.dim_color);
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
/// `scroll`: the rows are a part of a longer list (a thumb shows where).
pub fn set_rows(rows: Vec<Row>, selected: Option<usize>, total: usize, scroll: Option<ListScroll>) {
    let resize = ACTIVE.with(|a| {
        let mut a = a.borrow_mut();
        let s = a.as_mut()?;
        s.rows = rows;
        s.total = total;
        s.scroll = scroll;
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
        s.list_h = h;
        // The rows changed under a pointer at rest: the fill goes to what is under it now.
        if s.hover.is_some() {
            let pt = window::screen_to_client(s.window.hwnd(), window::cursor_pos());
            s.hover = s.target_at(pt.x, pt.y);
            s.hover_out = None;
            let hwnd = s.window.hwnd();
            restart_tip(s, hwnd);
        }
        tracing::debug!(
            rows = s.rows.len(),
            headings = s.rows.iter().filter(|r| r.heading.is_some()).count(),
            results = s.rows.iter().filter(|r| r.glyph.is_none()).count(),
            action_glyphs = s.rows.iter().filter(|r| r.glyph.is_some()).count(),
            selected = ?s.selected,
            total,
            first = scroll.map_or(0, |sc| sc.first),
            of = scroll.map_or(s.rows.len(), |sc| sc.of),
            bar = !s.rows.is_empty(),
            tab = tab_label(s.tab),
            "search list laid out"
        );
        Some((s.window.hwnd(), s.x, s.y, s.w, h, s.revealed))
    });
    let Some((hwnd, x, y, w, h, revealed)) = resize else {
        return;
    };
    let (_, now) = window::window_rect_size(hwnd);
    paint::invalidate(hwnd);
    if now == h {
        // A glide on its way elsewhere ends where it is.
        ACTIVE.with(|a| {
            if let Some(s) = a.borrow_mut().as_mut() {
                s.glide = None;
            }
        });
        return;
    }
    if !revealed {
        // Not on screen yet: it takes its size before its first paint.
        ACTIVE.with(|a| {
            if let Some(s) = a.borrow().as_ref() {
                let _ = s.window.set_bounds(x, y, w, h);
            }
        });
        return;
    }
    let animate = sysparams::client_area_animation();
    let shrinks = h < now;
    tracing::debug!(
        from = now,
        to = h,
        animate,
        "search box: new rows drawn at once, the window's height follows"
    );
    if shrinks {
        // The new rows now, in the window as it is, and its size from the next timer beat on:
        // Windows shows a new size a frame before what is painted after it, so a window made
        // shorter first showed the old rows, cut off, for a frame.
        paint::update_now(hwnd);
    }
    // A longer list starts a frame into its glide, in this same turn: its first rows show as
    // early as they did when the window took the whole height in one step.
    let head_start = std::time::Duration::from_millis(if shrinks { 0 } else { 16 });
    let started = Instant::now();
    ACTIVE.with(|a| {
        if let Some(s) = a.borrow_mut().as_mut() {
            s.glide = Some((now, started.checked_sub(head_start).unwrap_or(started)));
        }
    });
    window::set_timer(hwnd, TIMER_GLIDE, 15);
    if !shrinks {
        step_glide_with(hwnd, animate);
    }
}

/// `TIMER_GLIDE`: the window a step closer to the height of its list.
fn step_glide(hwnd: HWND) {
    step_glide_with(hwnd, sysparams::client_area_animation());
}

/// One step of the window's height towards `list_h` (the whole way when `animate` is off).
/// A window that grew is painted before this returns: the rows it uncovers show with it.
fn step_glide_with(hwnd: HWND, animate: bool) {
    let step = ACTIVE.with(|a| {
        let mut a = a.try_borrow_mut().ok()?;
        let s = a.as_mut().filter(|s| s.window.hwnd() == hwnd)?;
        let (from, started) = s.glide?;
        let t = if animate {
            (started.elapsed().as_secs_f32() * 1000.0 / GLIDE_MS).min(1.0)
        } else {
            1.0
        };
        let eased = pecofence_render::motion::ease(pecofence_render::motion::Curve::Decelerate, t);
        let h = if t >= 1.0 {
            s.glide = None;
            s.list_h
        } else {
            from + ((s.list_h - from) as f32 * eased).round() as i32
        };
        let (_, now) = window::window_rect_size(hwnd);
        let _ = s.window.set_bounds(s.x, s.y, s.w, h);
        tracing::trace!(from, to = s.list_h, h, t, "search box: height step");
        Some((t >= 1.0, h > now))
    });
    let Some((done, grew)) = step else {
        window::kill_timer(hwnd, TIMER_GLIDE);
        return;
    };
    if done {
        window::kill_timer(hwnd, TIMER_GLIDE);
        tracing::debug!("search box: the window has its list's height");
    }
    paint::invalidate(hwnd);
    if grew {
        paint::update_now(hwnd);
    }
}

/// The first matches are in. `avoid`: the fences that hold them (screen pixels). A box that
/// covers one of them where it is moves, this once, to the nearest place that covers none
/// (see [`place`]); it stays when there is no such place. True when it moved.
pub fn keep_clear(avoid: &[RECT]) -> bool {
    ACTIVE.with(|a| {
        let mut a = a.borrow_mut();
        let Some(s) = a.as_mut() else {
            return false;
        };
        let full = s.px(FULL_H).min(s.work.bottom - s.home.1);
        if !covers(s.x, s.y, s.w, full, avoid) {
            tracing::debug!(
                fences = avoid.len(),
                "search box: first matches, none covered, it stays"
            );
            return false;
        }
        let (x, y) = place(s.home, s.w, full, s.work, avoid, s.px(AVOID_GAP));
        let moved = (x, y) != (s.x, s.y);
        tracing::debug!(
            from = ?(s.x, s.y),
            x,
            y,
            fences = avoid.len(),
            moved,
            "search box: first matches, it covered a fence that holds some"
        );
        if moved {
            (s.x, s.y) = (x, y);
            let (_, h) = window::window_rect_size(s.window.hwnd());
            let _ = s.window.set_bounds(x, y, s.w, h);
        }
        moved
    })
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
    let Some(mut s) = session else { return };
    let hwnd = s.window.hwnd();
    window::kill_timer(hwnd, TIMER_HOVER);
    window::kill_timer(hwnd, TIMER_TIP);
    if let Some(tip) = s.tip.as_mut() {
        tip.hide();
    }
    // The box fades out with the dimmer (it takes nothing more: no session is open). One that
    // never showed, and any with Windows' animations off, goes at once.
    let fades = s.revealed && sysparams::client_area_animation();
    tracing::debug!(fades, "search box closed");
    if fades {
        s.alpha_to = 0.0;
        s.alpha_tick = Instant::now();
        window::set_timer(hwnd, TIMER_FADE, 15);
        drop(LEAVING.with(|l| l.borrow_mut().replace(s)));
    } else {
        drop(s.edit);
        drop(s.window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A long folder keeps its drive and its last folders; the middle goes. One character
    /// counts as one unit of width here.
    #[test]
    fn a_long_folder_path_is_cut_in_the_middle() {
        let chars = |t: &str| t.chars().count() as i32;
        let path = r"C:\Users\pf\Documents\项目资料归档\2026 年度\第三季度\客户往来文件\最终版 A";
        assert_eq!(fit_path(path, 80, chars), path);
        assert_eq!(
            fit_path(path, 30, chars),
            r"C:\…\2026 年度\第三季度\客户往来文件\最终版 A"
        );
        assert_eq!(fit_path(path, 12, chars), r"C:\…\最终版 A");
        // Never without the last folder, also when that alone is too long.
        assert_eq!(fit_path(path, 5, chars), r"C:\…\最终版 A");
        // A network path keeps its server.
        assert_eq!(
            fit_path(r"\\nas\share\年度归档\照片\旅行", 16, chars),
            r"\\nas\…\照片\旅行"
        );
        // Nothing to leave out: as it is.
        assert_eq!(fit_path(r"C:\最终确认版本", 4, chars), r"C:\最终确认版本");
    }

    /// The bottom bar names the Enter key in letters, like the keys beside it.
    #[test]
    fn the_bar_writes_enter_by_name() {
        assert!(
            bar_keys(true).starts_with("Enter 打开 "),
            "{}",
            bar_keys(true)
        );
        assert!(!bar_keys(true).contains('↵'));
        // A row without a file (an action row, 此电脑): only what works there.
        assert_eq!(bar_keys(false), "Enter 打开");
    }

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    /// The box keeps its place in the upper middle unless a fence is under it; then it goes to
    /// the nearest clear place beside, above or below, and stays when there is none.
    #[test]
    fn the_box_moves_aside_for_a_fence() {
        // 1600 x 852 work area, the box 640 x 562 at (480, 227).
        let wa = rect(0, 0, 1600, 852);
        let home = (480, 227);
        let at = |avoid: &[RECT]| place(home, 640, 562, wa, avoid, 16);
        // No fence, and fences clear of it (the first run's right columns at 1920 wide).
        assert_eq!(at(&[]), home);
        assert_eq!(at(&[rect(1136, 16, 1376, 550)]), home);
        // The first-run layout at 1600 wide: 文件与文档 from 1088. The box ends 16 px before it.
        let first_run = [rect(1088, 16, 1328, 550), rect(1344, 16, 1584, 262)];
        assert_eq!(at(&first_run), (432, 227));
        // A fence over its left part: to the right of it.
        assert_eq!(at(&[rect(300, 300, 560, 500)]), (576, 227));
        // A wide, low fence across the top: the box goes below it.
        assert_eq!(at(&[rect(200, 40, 1400, 240)]), (480, 256));
        // Fences left and right of the middle with no room between, beside, above or below.
        let crowded = [rect(300, 240, 600, 540), rect(1040, 240, 1300, 570)];
        assert_eq!(at(&crowded), home);
        // Only the right one holds matches: clear of that one, over the other.
        assert_eq!(at(&crowded[1..]), (384, 227));
        // A fence on another screen changes nothing.
        assert_eq!(at(&[rect(1700, 100, 2000, 400)]), home);
    }

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
