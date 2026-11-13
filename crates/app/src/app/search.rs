//! The Peek search list (Listary-style): fence matches ranked by how well they match (pinyin
//! included) and how often they were opened from the search, Everything's whole-computer
//! results under them, and what Enter / Ctrl+Enter / the actions menu do with a row. When an
//! Open / Save dialog was in front as the Peek began, Enter hands the row to that dialog.

use super::*;
use crate::peek_search::Row;
use pecofence_core::everything as ev;
use pecofence_core::matcher;
use pecofence_core::type_filter;
use pecofence_platform::{clipboard, dialogs, everything};
use std::ops::Range;
use std::os::windows::process::CommandExt;

/// Result rows the list shows at once (Ctrl+1 .. Ctrl+9 reach them all); with more results
/// it scrolls.
const MAX_ROWS: usize = 9;
/// Everything results asked for; they are ranked again here.
const EVERYTHING_MAX: u32 = 60;
/// Typing pause before Everything is asked.
const EVERYTHING_DELAY_MS: u32 = 120;
/// `dwData` of Everything's replies: this tag plus the query's sequence number.
const REPLY_TAG: usize = 0x5046_0000;

const CMD_OPEN: u32 = 1;
const CMD_OPEN_FOLDER: u32 = 2;
const CMD_COPY_PATH: u32 = 3;
const CMD_SHELL_FIRST: u32 = 0x100;
const CMD_SHELL_LAST: u32 = 0x7FFF;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum RowKind {
    /// `peek_hits[i]`.
    Fence(usize),
    File {
        path: PathBuf,
        is_folder: bool,
    },
    /// 「在 Everything 里查看全部结果」.
    ShowAll,
    /// 「把这次搜索放到桌面」: a smart fence with this search.
    ToDesktop,
}

#[derive(Default)]
pub(super) struct SearchState {
    /// Every row of the list: the fences' matches, Everything's results, the action rows.
    pub rows: Vec<RowKind>,
    pub sel: usize,
    /// The first result row the box shows (the list scrolls when there are more than fit),
    /// and how many it shows at once.
    pub top: usize,
    pub window: usize,
    /// `rows` indices of the rows in the box, top down: the results shown, then the action
    /// rows, which stay under them.
    pub shown: Vec<usize>,
    /// Highlight ranges of each `peek_hits` entry in its shown name.
    pub fence_marks: Vec<Vec<Range<usize>>>,
    /// Everything's last answer for the current text, and how many it found in all.
    pub everything: Vec<ev::Hit>,
    pub everything_total: u32,
    /// How many results the search has in all (the bottom bar): the fences' matches and
    /// Everything's, less those of its answer already in a fence or not of the type tab.
    pub total: usize,
    /// Sequence number of the last query sent, and its text.
    pub seq: u32,
    pub sent: String,
    /// When that query went to Everything, until its answer is in: the smart fences' searches
    /// wait meanwhile (Everything answers one search at a time, a new one cancels the last).
    pub asked_at: Option<Instant>,
    /// The box had its chance to move away from the fences that hold the first matches.
    pub placed: bool,
    /// The Open / Save dialog in front when the Peek began.
    pub dialog: Option<isize>,
    /// The window in front when the Peek began (double Ctrl gives the foreground back to it).
    pub before: Option<isize>,
}

fn hwnd_of(raw: isize) -> HWND {
    HWND(raw as *mut core::ffi::c_void)
}

impl App {
    /// Notes an Open / Save dialog in front as the Peek begins (Enter then hands it the row).
    pub(super) fn note_search_dialog(&mut self) {
        let front = desktop::foreground_window();
        self.search.dialog = dialogs::is_file_dialog(front).then_some(front.0 as isize);
        self.search.before = (!front.0.is_null()
            && desktop::window_pid(front) != pecofence_platform::process::current_pid())
        .then_some(front.0 as isize);
        if self.search.dialog.is_some() {
            tracing::info!(kind = ?dialogs::dialog_kind(front), "peek began over a file dialog");
        }
    }

    /// Ctrl pressed twice: the search box (with the Peek), or the Peek ends when the box is
    /// already open. Ignored while a full-screen program (a game, a video) is in front.
    pub(super) fn search_by_double_ctrl(&mut self) {
        if desktop::fullscreen_app_in_front() {
            tracing::debug!("double Ctrl ignored: a full-screen window is in front");
            return;
        }
        let peeking = self.peek.as_ref().is_some_and(|p| !p.is_closing());
        if peeking && crate::peek_search::is_open() {
            let before = self.search.before;
            self.end_peek("Ctrl pressed twice");
            if let Some(b) = before.map(hwnd_of).filter(|&b| desktop::is_window(b)) {
                window::bring_to_front(b);
            }
            return;
        }
        if !peeking {
            self.toggle_peek();
        }
        if self.peek.is_none() {
            return;
        }
        tracing::info!("search box opened by double Ctrl");
        self.open_peek_search("");
        self.update_peek_search();
    }

    /// The empty box says what can be typed: the whole computer too when that search is on and
    /// Everything is running. Asked without a message to Everything (`everything::running`):
    /// `find` waits for its answer, up to a second while Everything is busy, and the box
    /// opened that much later.
    pub(super) fn show_search_cue(&self) {
        let whole_pc = self.state.config.settings.tools.everything && everything::running();
        tracing::debug!(whole_pc, "search: the empty box shows its hint");
        crate::peek_search::set_cue(whole_pc);
    }

    /// Ranks the fence matches for `query` (lower case): best match first, then the ones opened
    /// most from the search, then reading order. Fills `peek_hits` and their highlights.
    pub(super) fn rank_fence_matches(&mut self, query: &str) {
        let started = Instant::now();
        self.rank_fence_matches_now(query);
        crate::peek_search::slow_stage("rank_fence_matches", started);
    }

    fn rank_fence_matches_now(&mut self, query: &str) {
        let mut found: Vec<(
            u32,
            u32,
            usize,
            (FenceId, FenceId, ItemId),
            Vec<Range<usize>>,
        )> = Vec::new();
        // The type tab keeps the matches to its type (as the rule conditions test it).
        let types = peek_types();
        if !query.is_empty() {
            let mut windows: Vec<(RECT, FenceId)> =
                self.fences.iter().map(|(id, w)| (w.rect(), *id)).collect();
            windows.sort_by_key(|(r, id)| (r.left, r.top, *id));
            for (_, host) in windows {
                for tab in self.state.tabs_of(host) {
                    let Some(fence) = self.state.fence(tab) else {
                        continue;
                    };
                    for it in self.state.items_of(fence) {
                        let Some((score, marks)) =
                            super::peek::search_match(&it.display_name, it.key.as_path(), query)
                        else {
                            continue;
                        };
                        // After the name: 程序 reads a shortcut's target.
                        if !types.is_empty() && !item_is_of(&types, it) {
                            continue;
                        }
                        let uses = it
                            .path()
                            .map(|p| self.state.config.search_count(p))
                            .unwrap_or(0);
                        let order = found.len();
                        found.push((score, uses, order, (host, tab, it.id), marks));
                    }
                }
            }
        }
        found.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
        self.peek_hits = found.iter().map(|f| f.3).collect();
        self.search.fence_marks = found.into_iter().map(|f| f.4).collect();
    }

    /// Asks Everything after a short typing pause (switch 搜索整台电脑 on, text in the box).
    pub(super) fn schedule_everything(&mut self, query: &str) {
        if !self.state.config.settings.tools.everything || query.is_empty() {
            self.search.everything.clear();
            self.search.everything_total = 0;
            self.search.sent.clear();
            self.search.asked_at = None;
            // An answer still on its way is for text that is gone.
            self.search.seq = self.search.seq.wrapping_add(1);
            self.ask_next_smart();
            return;
        }
        window::set_timer(self.control.hwnd(), TIMER_EVERYTHING, EVERYTHING_DELAY_MS);
    }

    /// `TIMER_EVERYTHING`: sends the box's text to Everything (the answer comes to the box).
    pub(super) fn send_everything_query(&mut self) {
        let Some(query) = crate::peek_search::query().map(|q| q.trim().to_string()) else {
            return;
        };
        let Some(reply_to) = crate::peek_search::hwnd() else {
            return;
        };
        // The type tab is part of what was asked.
        let asked = with_type_words(query.clone());
        if query.is_empty() || asked == self.search.sent {
            return;
        }
        let Some(instance) = everything::find() else {
            tracing::debug!("search: Everything is not running");
            self.search.everything.clear();
            self.search.everything_total = 0;
            self.refresh_search_rows();
            return;
        };
        let text = with_type_words(ev::search_text(&query, instance.supports_pinyin()));
        // The Peek search goes first: a smart fence's search out now would be cancelled by
        // this one, so that fence asks again after the Peek's answer.
        self.smart_yield_to_peek();
        self.search.seq = self.search.seq.wrapping_add(1);
        let kind = REPLY_TAG | (self.search.seq as usize & 0xFFFF);
        let payload = ev::query2w(
            reply_to.0 as usize as u32,
            kind as u32,
            &text,
            EVERYTHING_MAX,
            ev::SORT_DATE_MODIFIED_DESCENDING,
        );
        let sent = everything::send_query(instance.hwnd, reply_to, ev::COPYDATA_QUERY2W, &payload);
        tracing::debug!(
            sent,
            version = %format!("{}.{}", instance.major, instance.minor),
            search = %text,
            "search: Everything asked"
        );
        if sent {
            self.search.sent = asked;
            self.search.asked_at = Some(Instant::now());
        }
    }

    /// Everything's answer (`WM_COPYDATA` to the search box). Answers to older text are dropped.
    pub(super) fn on_everything_results(&mut self, kind: usize, bytes: &[u8]) {
        if kind != REPLY_TAG | (self.search.seq as usize & 0xFFFF) {
            return;
        }
        // The box changed after the query went out: the next query answers for it.
        let now = with_type_words(
            crate::peek_search::query()
                .map(|q| q.trim().to_string())
                .unwrap_or_default(),
        );
        if now != self.search.sent {
            return;
        }
        let Some((total, hits)) = ev::parse_list2(bytes) else {
            tracing::warn!(
                len = bytes.len(),
                "search: Everything's answer could not be read"
            );
            return;
        };
        tracing::debug!(total, got = hits.len(), "search: Everything answered");
        self.search.asked_at = None;
        // 程序: Everything's hits come without their times, so each shortcut whose target is
        // known gets it checked against its file once per answer (on its drive's thread).
        let types = peek_types();
        for h in &hits {
            if !h.is_folder
                && type_filter::needs_target(&types, &h.path, false)
                && super::lnk_targets::known(&h.path, None).is_some()
            {
                super::lnk_targets::ask(&h.path, None);
            }
        }
        self.search.everything = hits;
        self.search.everything_total = total;
        self.refresh_search_rows();
        // Smart fences that waited for the Peek search ask now.
        self.ask_next_smart();
    }

    /// Rebuilds the list from `peek_hits` and Everything's results, keeps the highlighted row
    /// when it is still there, and shows it.
    pub(super) fn refresh_search_rows(&mut self) {
        let started = Instant::now();
        self.refresh_search_rows_now();
        crate::peek_search::slow_stage("refresh_search_rows", started);
    }

    fn refresh_search_rows_now(&mut self) {
        let query = crate::peek_search::query()
            .map(|q| q.trim().to_lowercase())
            .unwrap_or_default();
        let types = peek_types();
        let before = self.search.rows.get(self.search.sel).cloned();
        let before_hit = match before {
            Some(RowKind::Fence(i)) => self.peek_hits.get(i).copied(),
            _ => None,
        };
        let fence_paths: Vec<String> = self
            .peek_hits
            .iter()
            .filter_map(|(_, _, item)| self.state.item(*item)?.path().map(|p| p.to_lowercase()))
            .collect();
        // Everything's results, ranked here: Everything sorts by date only.
        let mut files: Vec<(u32, u32, usize, PathBuf, bool)> = Vec::new();
        let mut skipped = 0;
        for (order, hit) in self.search.everything.iter().enumerate() {
            if fence_paths.contains(&hit.path.to_lowercase()) {
                skipped += 1;
                continue;
            }
            let path = PathBuf::from(&hit.path);
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| hit.path.clone());
            // Asked with the type's words already; the same test as the fences' items.
            // A shortcut whose target is being read waits (held back, as not of the type).
            if super::filter::type_fit(&types, &name, hit.is_folder, Some(&hit.path), None)
                != Some(true)
            {
                skipped += 1;
                continue;
            }
            let score = matcher::match_name(&name, &query).map_or(0, |m| m.score);
            let uses = self.state.config.search_count(&hit.path);
            files.push((score, uses, order, path, hit.is_folder));
        }
        files.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)).then(a.2.cmp(&b.2)));
        // As many rows as fit under the box: Everything's rows bring two headings and the
        // "show all" row.
        let show_all = self.search.everything_total > 0;
        // Headings: 栅栏里, 整台电脑 when Everything found some, 操作 before the action rows.
        let headings = 1 + usize::from(!files.is_empty()) + usize::from(show_all);
        let max_rows = crate::peek_search::row_room(headings)
            .saturating_sub(2 * usize::from(show_all))
            .clamp(1, MAX_ROWS);
        // One row per file. A file that several fences show (a folder fence and a smart fence
        // over one folder) keeps its best-ranked hit: among equal matches the one in the
        // leftmost fence, then the topmost, then the first tab (the order of `peek_hits`).
        // The fences still mark every copy.
        let hits: Vec<(usize, Option<String>)> = (0..self.peek_hits.len())
            .filter_map(|i| {
                let it = self.state.item(self.peek_hits[i].2)?;
                Some((i, it.path().map(|p| p.to_lowercase())))
            })
            .collect();
        let copies = hits.len();
        let in_fences = one_per_path(hits);
        if in_fences.len() < copies {
            tracing::debug!(
                hits = copies,
                listed = in_fences.len(),
                "search: files that several fences show are listed once"
            );
        }
        self.search.total =
            in_fences.len() + (self.search.everything_total as usize).saturating_sub(skipped);
        // Every result is a row: the box shows `max_rows` of them and scrolls.
        let mut rows: Vec<RowKind> = in_fences.into_iter().map(RowKind::Fence).collect();
        rows.extend(files.into_iter().map(|f| RowKind::File {
            path: f.3,
            is_folder: f.4,
        }));
        self.search.window = max_rows;
        if show_all {
            rows.push(RowKind::ShowAll);
            rows.push(RowKind::ToDesktop);
        }
        self.search.sel = rows
            .iter()
            .position(|r| match (r, &before, before_hit) {
                (RowKind::Fence(i), _, Some(h)) => self.peek_hits.get(*i) == Some(&h),
                (r, Some(b), None) => r == b,
                _ => false,
            })
            .unwrap_or(0);
        self.search.rows = rows;
        self.reveal_search_row();
        self.show_search_rows();
    }

    /// How many of the rows are results (the action rows come after them).
    fn search_results(&self) -> usize {
        self.search
            .rows
            .iter()
            .position(|r| matches!(r, RowKind::ShowAll | RowKind::ToDesktop))
            .unwrap_or(self.search.rows.len())
    }

    /// Scrolls the list just far enough for the highlighted row to show.
    fn reveal_search_row(&mut self) {
        let (results, window) = (self.search_results(), self.search.window);
        let top = scroll_top(self.search.top, self.search.sel, results, window);
        if top != self.search.top {
            tracing::debug!(
                from = self.search.top,
                top,
                sel = self.search.sel,
                results,
                window,
                "search list scrolled to the highlighted row"
            );
            self.search.top = top;
        }
    }

    /// Puts the list in the box and the fence highlight on the selected fence row.
    pub(super) fn show_search_rows(&mut self) {
        self.peek_hit = match self.search.rows.get(self.search.sel) {
            Some(RowKind::Fence(i)) => *i,
            _ => usize::MAX,
        };
        let query = crate::peek_search::query().unwrap_or_default();
        let searching = !query.trim().is_empty();
        self.show_peek_hits(searching);
        // The results from `top` on, as many as fit, then the action rows.
        let results = self.search_results();
        let window = self.search.window.max(1);
        let top = self.search.top.min(results.saturating_sub(window));
        self.search.shown = (top..results.min(top + window))
            .chain(results..self.search.rows.len())
            .collect();
        let mut rows = Vec::with_capacity(self.search.shown.len());
        let mut last_group = None;
        for (i, kind) in self
            .search
            .shown
            .iter()
            .map(|&r| &self.search.rows[r])
            .enumerate()
        {
            // Every group under its heading, the two action rows too.
            let group = match kind {
                RowKind::Fence(_) => 0,
                RowKind::File { .. } => 1,
                RowKind::ShowAll | RowKind::ToDesktop => 2,
            };
            let heading = (last_group != Some(group)).then(|| {
                match group {
                    0 => pecofence_core::i18n::text("栅栏里"),
                    1 => pecofence_core::i18n::text("整台电脑（Everything）"),
                    _ => pecofence_core::i18n::text("操作"),
                }
                .to_string()
            });
            last_group = Some(group);
            let shortcut = (i < MAX_ROWS).then_some(i as u8 + 1);
            let row = match kind {
                RowKind::Fence(h) => {
                    let (_, tab, item) = self.peek_hits[*h];
                    let Some(it) = self.state.item(item) else {
                        // Gone since the rows were made: an empty row keeps the others in place.
                        rows.push(Row {
                            name: String::new(),
                            marks: Vec::new(),
                            detail: String::new(),
                            icon_path: None,
                            is_folder: false,
                            exact_icon: false,
                            heading,
                            shortcut,
                            glyph: None,
                        });
                        continue;
                    };
                    let path = it.key.as_path().map(PathBuf::from);
                    // The fence this row opens the file from; a file more fences show says so.
                    let title = self
                        .state
                        .fence(tab)
                        .map(|f| f.title.clone())
                        .unwrap_or_default();
                    let shown_in = it.path().map(|p| p.to_lowercase()).map_or(1, |p| {
                        self.peek_hits
                            .iter()
                            .filter_map(|(_, _, other)| self.state.item(*other)?.path())
                            .filter(|other| other.to_lowercase() == p)
                            .count()
                    });
                    Row {
                        name: it.display_name.clone(),
                        marks: self.search.fence_marks.get(*h).cloned().unwrap_or_default(),
                        detail: if shown_in > 1 {
                            pecofence_core::i18n::format(
                                "{0} 等 {1} 个栅栏",
                                &[title, shown_in.to_string()],
                            )
                        } else {
                            title
                        },
                        exact_icon: path.as_deref().is_some_and(shell::is_on_fixed_drive),
                        icon_path: path,
                        is_folder: it.is_folder,
                        heading,
                        shortcut,
                        glyph: None,
                    }
                }
                RowKind::File { path, is_folder } => {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.to_string_lossy().to_string());
                    let marks = matcher::match_name(&name, &query.trim().to_lowercase())
                        .map(|m| m.ranges)
                        .unwrap_or_default();
                    Row {
                        name,
                        marks,
                        detail: path
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        exact_icon: shell::is_on_fixed_drive(path),
                        icon_path: Some(path.clone()),
                        is_folder: *is_folder,
                        heading,
                        shortcut,
                        glyph: None,
                    }
                }
                RowKind::ShowAll => Row {
                    name: pecofence_core::i18n::text("在 Everything 里查看全部结果").to_string(),
                    marks: Vec::new(),
                    detail: pecofence_core::i18n::format(
                        "共 {0} 个",
                        &[self.search.everything_total.to_string()],
                    ),
                    icon_path: None,
                    is_folder: false,
                    exact_icon: false,
                    heading,
                    shortcut,
                    // Search (E721): more of the same search, in Everything.
                    glyph: Some("\u{E721}"),
                },
                RowKind::ToDesktop => Row {
                    name: pecofence_core::i18n::text("把这次搜索放到桌面").to_string(),
                    marks: Vec::new(),
                    detail: pecofence_core::i18n::text("新建一个智能栅栏，结果跟着文件自动更新")
                        .to_string(),
                    icon_path: None,
                    is_folder: false,
                    exact_icon: false,
                    heading,
                    shortcut,
                    // Pin (E718): the search stays on the desktop.
                    glyph: Some("\u{E718}"),
                },
            };
            rows.push(row);
        }
        // None while the highlighted row is scrolled out of sight (the wheel).
        let selected = self.search.shown.iter().position(|&r| r == self.search.sel);
        // More results than fit: where the rows shown are in them, and how many headings the
        // results have in all (the list keeps their room, so its height stays while it scrolls).
        let scroll = (results > window).then(|| crate::peek_search::ListScroll {
            first: top,
            shown: window,
            of: results,
            headings: usize::from(matches!(self.search.rows.first(), Some(RowKind::Fence(_))))
                + usize::from(
                    self.search.rows[..results]
                        .iter()
                        .any(|r| matches!(r, RowKind::File { .. })),
                ),
        });
        crate::peek_search::set_rows(rows, selected, self.search.total, scroll);
    }

    /// Up / Down (Ctrl+P / Ctrl+N) in the box: the previous / next row; the list scrolls with
    /// it, and from the last row Down goes back to the first (Up from the first to the last).
    pub(super) fn step_search_row(&mut self, step: i32) {
        let n = self.search.rows.len() as i32;
        if n == 0 {
            return;
        }
        self.search.sel = (self.search.sel as i32 + step).rem_euclid(n) as usize;
        self.reveal_search_row();
        self.show_search_rows();
    }

    /// PageUp / PageDown: the row a page (the rows shown) up or down, stopping at the first
    /// and the last row; Home / End (`i32::MIN` / `i32::MAX`) go to those.
    pub(super) fn page_search_row(&mut self, pages: i32) {
        let Some(last) = self.search.rows.len().checked_sub(1) else {
            return;
        };
        let by = pages as i64 * self.search.window.max(1) as i64;
        self.search.sel = (self.search.sel as i64 + by).clamp(0, last as i64) as usize;
        tracing::debug!(pages, sel = self.search.sel, "search list: page key");
        self.reveal_search_row();
        self.show_search_rows();
    }

    /// The wheel over the list: scrolls it by `rows` rows (down is positive). The highlighted
    /// row stays the same one, also when it scrolls out of sight.
    pub(super) fn scroll_search_rows(&mut self, rows: i32) {
        let max_top = self
            .search_results()
            .saturating_sub(self.search.window.max(1));
        let top = (self.search.top as i64 + rows as i64).clamp(0, max_top as i64) as usize;
        tracing::debug!(
            from = self.search.top,
            top,
            max_top,
            "search list scrolled by the wheel"
        );
        if top != self.search.top {
            self.search.top = top;
            self.show_search_rows();
        }
    }

    /// The path behind a row (`None` for a namespace item or the show-all row).
    fn row_path(&self, row: &RowKind) -> Option<(PathBuf, bool)> {
        match row {
            RowKind::Fence(i) => {
                let (_, _, item) = *self.peek_hits.get(*i)?;
                let it = self.state.item(item)?;
                let p = it.path()?;
                (!shell::is_namespace_path(Path::new(p))).then(|| (PathBuf::from(p), it.is_folder))
            }
            RowKind::File { path, is_folder } => Some((path.clone(), *is_folder)),
            RowKind::ShowAll | RowKind::ToDesktop => None,
        }
    }

    /// Enter, a click or Ctrl+digit on row `index`.
    pub(super) fn open_search_row(&mut self, index: usize) {
        let Some(row) = self.search.rows.get(index).cloned() else {
            return;
        };
        let query = crate::peek_search::query()
            .unwrap_or_default()
            .trim()
            .to_string();
        let path = self.row_path(&row);
        // Deleted since the list was made: Windows would show its "cannot find" box, which
        // holds PecoFence until it is closed. The row goes instead.
        if let Some((p, _)) = &path
            && !p.exists()
        {
            tracing::info!(
                index,
                "search row opened: its file is gone, the row is dropped"
            );
            self.search.rows.remove(index);
            self.search.sel = self
                .search
                .sel
                .min(self.search.rows.len().saturating_sub(1));
            self.reveal_search_row();
            self.show_search_rows();
            return;
        }
        if let Some((p, _)) = &path {
            self.state.config.note_search_open(&p.to_string_lossy());
            self.state.mark_dirty();
            self.schedule_save();
        }
        let dialog = self
            .search
            .dialog
            .filter(|d| desktop::is_window(hwnd_of(*d)) && dialogs::is_file_dialog(hwnd_of(*d)));
        tracing::info!(row = ?row, dialog = dialog.is_some(), "search row opened");
        match (&row, path, dialog) {
            (RowKind::ToDesktop, _, _) => {
                // Read before the box closes with the Peek.
                let tab = crate::peek_search::type_tab();
                self.end_peek("search: put on the desktop");
                self.search_to_desktop(&query, tab);
            }
            (RowKind::ShowAll, _, _) => {
                // Read before the box closes with the Peek.
                let types = peek_types();
                self.end_peek("search: show all in Everything");
                // The same search as the list (pinyin included, the type tab's words too), in
                // Everything's window.
                let found = everything::find().and_then(|i| Some((i.exe()?, i.supports_pinyin())));
                match found {
                    Some((exe, pinyin)) => {
                        let mut text = ev::search_text(&query, pinyin);
                        if !types.is_empty() {
                            text = format!("{text} {}", type_filter::everything_words(&types));
                        }
                        tracing::info!(search = %text, types = types.len(), "search: shown in Everything");
                        let started = std::process::Command::new(exe)
                            .arg("-search")
                            .arg(text)
                            .spawn();
                        if let Err(e) = started {
                            tracing::warn!(error = %e, "search: Everything did not start");
                        }
                    }
                    None => tracing::info!("search: Everything is no longer running"),
                }
            }
            (_, Some((p, is_folder)), Some(d)) => {
                self.end_peek("search result sent to a dialog");
                crate::dialog_jump::send_to_dialog(hwnd_of(d), p, is_folder);
            }
            (RowKind::Fence(i), _, _) => {
                let Some(&(_, tab, item)) = self.peek_hits.get(*i) else {
                    return;
                };
                self.end_peek("search match opened");
                self.launch(item, tab, false);
            }
            (RowKind::File { .. }, Some((p, _)), None) => {
                self.end_peek("search: whole-computer result opened");
                if let Err(e) = shell::shell_execute(&p, None, Some(self.control.hwnd())) {
                    tracing::warn!(error = %e, path = %p.display(), "search: open failed");
                }
            }
            _ => {}
        }
    }

    /// Ctrl+Enter: Explorer with the highlighted row's file selected.
    pub(super) fn open_search_row_folder(&mut self) {
        let Some(row) = self.search.rows.get(self.search.sel).cloned() else {
            return;
        };
        let Some((path, _)) = self.row_path(&row) else {
            return;
        };
        self.end_peek("search: containing folder opened");
        open_containing_folder(&path);
    }

    /// What a row is, in fixed words for the log.
    fn row_kind(&self, row: &RowKind) -> &'static str {
        match row {
            RowKind::Fence(i) => {
                let Some(&(_, tab, item)) = self.peek_hits.get(*i) else {
                    return "fence row, gone";
                };
                let Some(it) = self.state.item(item) else {
                    return "fence row, gone";
                };
                let folder_fence = self
                    .state
                    .fence(tab)
                    .is_some_and(|f| f.kind == FenceKind::FolderPortal);
                let system = it.is_namespace()
                    || it
                        .path()
                        .is_none_or(|p| shell::is_namespace_path(Path::new(p)));
                match (system, folder_fence, it.is_folder) {
                    (true, _, _) => "fence system item",
                    (false, true, true) => "folder fence, folder",
                    (false, true, false) => "folder fence, file",
                    (false, false, true) => "fence, folder",
                    (false, false, false) => "fence, file",
                }
            }
            RowKind::File {
                is_folder: true, ..
            } => "Everything, folder",
            RowKind::File { .. } => "Everything, file",
            RowKind::ShowAll => "show all in Everything",
            RowKind::ToDesktop => "put the search on the desktop",
        }
    }

    /// A right-click on the box's row `shown` (`x`, `y`: screen pixels): the row is highlighted
    /// and its actions open at the pointer, as in Explorer.
    pub(super) fn search_row_menu(&mut self, shown: usize, x: i32, y: i32) {
        let Some(&index) = self.search.shown.get(shown) else {
            tracing::info!(
                row = shown,
                rows = self.search.shown.len(),
                "search: right-click on a row that is gone, no menu"
            );
            return;
        };
        self.search.sel = index;
        self.show_search_rows();
        self.show_search_actions(Some((x, y)));
    }

    /// The highlighted row's actions: 打开 / 打开所在文件夹 / 复制路径, then its Windows
    /// context menu. `at`: the right-click's screen point; `None` for the keys (Right at the
    /// end, Ctrl+O, the menu key, Shift+F10), which show it under the row. The Peek and the
    /// box stay while it is open, after Esc and after 复制路径; 打开, 打开所在文件夹 and the
    /// Windows entries end the Peek. A click on the dimmer closes the menu and, as any click
    /// there, the Peek.
    pub(super) fn show_search_actions(&mut self, at: Option<(i32, i32)>) {
        let by = if at.is_some() { "right-click" } else { "keys" };
        let index = self.search.sel;
        let Some(row) = self.search.rows.get(index).cloned() else {
            tracing::info!(by, "search actions: no row, no menu");
            return;
        };
        let row_kind = self.row_kind(&row);
        let Some((path, _)) = self.row_path(&row) else {
            tracing::info!(
                row = index,
                by,
                what = row_kind,
                "search actions: the row has no file path, no menu"
            );
            return;
        };
        let Some(owner) = crate::peek_search::hwnd() else {
            return;
        };
        let (x, y) = at
            .or_else(crate::peek_search::selected_row_point)
            .unwrap_or_else(|| {
                let p = window::cursor_pos();
                (p.x, p.y)
            });
        // The same glyphs as the icon menu in a fence.
        let menu = PopupMenu::new();
        menu.item(CMD_OPEN, pecofence_core::i18n::text("打开"), false, false)
            .icon(super::menus::glyph::OPEN)
            .default_item(CMD_OPEN)
            .item(
                CMD_OPEN_FOLDER,
                pecofence_core::i18n::text("打开所在文件夹\tCtrl+Enter"),
                false,
                false,
            )
            .icon(super::menus::glyph::FOLDER_OPEN)
            .item(
                CMD_COPY_PATH,
                pecofence_core::i18n::text("复制路径"),
                false,
                false,
            )
            .icon(super::menus::glyph::COPY);
        let _loads = pecofence_platform::shell_menu::LoadWatch::new("SearchMenu");
        // Not after a crash in a menu extension (see `App::windows_menu_off`).
        let bound =
            (!self.windows_menu_off()).then(|| ShellContextMenu::for_paths(&[path.as_path()]));
        let mut shell_menu = match bound {
            None => None,
            Some(Ok(sm)) => Some(sm),
            Some(Err(e)) => {
                tracing::info!(error = %e, "search actions: no Windows menu for the row");
                None
            }
        };
        if let Some(sm) = shell_menu.as_mut() {
            menu.separator();
            match sm.populate(
                menu.handle(),
                menu.len(),
                CMD_SHELL_FIRST,
                CMD_SHELL_LAST,
                false,
            ) {
                Ok(_) => {
                    sm.remove_verbs(menu.handle(), &["open"]);
                    menu.tidy_separators();
                }
                Err(e) => {
                    tracing::info!(error = %e, "search actions: the Windows menu could not be added");
                    shell_menu = None;
                }
            }
        }
        tracing::info!(
            row = index,
            by,
            what = row_kind,
            windows_menu = shell_menu.is_some(),
            x,
            y,
            "search actions menu shown"
        );
        let cmd = self.track_menu("search_row", || menu.show_context(owner, x, y));
        if let Some(sm) = shell_menu.as_ref()
            && sm.contains(cmd)
        {
            let verb = sm.verb(cmd).unwrap_or_default();
            tracing::info!(row = index, by, %verb, "search: shell verb invoked");
            // The window an entry opens (Properties, Open with, the program of Edit, a delete
            // confirmation) would sit under the Peek, which stays topmost. The Peek ends first,
            // as for 打开, and at once: a confirmation runs inside `invoke`, where no frame of
            // the fade would run.
            self.end_peek("search: Windows menu entry chosen");
            self.end_peek_now();
            let owner = self.control.hwnd();
            if let Err(e) = sm.invoke(cmd, owner, POINT { x, y }, false) {
                tracing::warn!(error = %e, %verb, "search: shell verb failed");
            }
            return;
        }
        drop(shell_menu);
        let outcome = match cmd {
            CMD_OPEN => "open",
            CMD_OPEN_FOLDER => "open containing folder",
            CMD_COPY_PATH => "copy path",
            0 => "closed without a choice",
            _ => "unknown command",
        };
        tracing::info!(
            row = index,
            by,
            outcome,
            peek = self.peek.is_some(),
            box_open = crate::peek_search::is_open(),
            "search actions menu closed"
        );
        match cmd {
            CMD_OPEN => self.open_search_row(index),
            CMD_OPEN_FOLDER => self.open_search_row_folder(),
            CMD_COPY_PATH => {
                let ok = clipboard::set_text(owner, &path.to_string_lossy());
                tracing::info!(ok, "search: path copied");
            }
            _ => {}
        }
    }
}

/// Of the fence hits (index and lower-case path, best-ranked first) the ones the list shows:
/// the first of those that share a path; hits without a path (system items) all stay.
fn one_per_path(hits: Vec<(usize, Option<String>)>) -> Vec<usize> {
    let mut seen = std::collections::HashSet::new();
    hits.into_iter()
        .filter(|(_, path)| path.as_ref().is_none_or(|p| seen.insert(p.clone())))
        .map(|(i, _)| i)
        .collect()
}

/// The first result row to show so that row `sel` is in sight: `top` when it already is (or
/// when `sel` is an action row, which always shows), else moved just far enough. Never past
/// the place where the last `window` results fill the list.
fn scroll_top(top: usize, sel: usize, results: usize, window: usize) -> usize {
    let window = window.max(1);
    let top = top.min(results.saturating_sub(window));
    if sel >= results {
        top
    } else if sel < top {
        sel
    } else if sel >= top + window {
        sel + 1 - window
    } else {
        top
    }
}

/// Explorer with `path` selected in its folder.
fn open_containing_folder(path: &Path) {
    // `/select,"<path>"`: quoted as a whole.
    let spawned = std::process::Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{}\"", path.display()))
        .spawn();
    if let Err(e) = spawned {
        tracing::warn!(error = %e, "search: Explorer did not start");
    }
}

/// The Peek search's type tab as tokens (none on 全部).
fn peek_types() -> Vec<type_filter::TypeToken> {
    crate::peek_search::type_tab()
        .map(type_filter::TypeToken::Cat)
        .into_iter()
        .collect()
}

/// Whether a fence's item is of `types` (its file name with the extension).
fn item_is_of(types: &[type_filter::TypeToken], it: &pecofence_core::Item) -> bool {
    let file = it
        .key
        .as_path()
        .and_then(|p| Path::new(p).file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| it.display_name.clone());
    super::filter::type_fit(types, &file, it.is_folder, it.key.as_path(), Some(it.mtime))
        == Some(true)
}

/// `text` with Everything's words for the type tab after it.
fn with_type_words(text: String) -> String {
    let types = peek_types();
    if types.is_empty() {
        return text;
    }
    format!("{text} {}", type_filter::everything_words(&types))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file two fences show is listed once, from its best-ranked hit; other files and
    /// system items (no path) keep their rows and their order.
    #[test]
    fn a_file_in_two_fences_is_listed_once() {
        let hit = |i: usize, path: Option<&str>| (i, path.map(str::to_string));
        let hits = vec![
            hit(0, Some(r"c:\users\pf\desktop\报告 草稿.docx")),
            hit(1, None),
            hit(2, Some(r"c:\users\pf\desktop\季度报告 q3.xlsx")),
            hit(3, Some(r"c:\users\pf\desktop\报告 草稿.docx")),
            hit(4, None),
            hit(5, Some(r"c:\users\pf\desktop\季度报告 q3.xlsx")),
        ];
        assert_eq!(one_per_path(hits), vec![0, 1, 2, 4]);
    }

    /// The list scrolls just far enough for the highlighted row, and an action row (after the
    /// results) leaves it where it is.
    #[test]
    fn the_list_scrolls_to_the_highlighted_row() {
        // 18 results, nine shown.
        assert_eq!(scroll_top(0, 0, 18, 9), 0);
        assert_eq!(scroll_top(0, 8, 18, 9), 0);
        assert_eq!(scroll_top(0, 9, 18, 9), 1);
        assert_eq!(scroll_top(1, 17, 18, 9), 9);
        assert_eq!(scroll_top(9, 12, 18, 9), 9);
        assert_eq!(scroll_top(9, 3, 18, 9), 3);
        // Down from the last row wraps to the first.
        assert_eq!(scroll_top(9, 0, 18, 9), 0);
        // An action row (index 18 or 19) shows whatever the scroll is.
        assert_eq!(scroll_top(4, 19, 18, 9), 4);
        // Fewer results than the list holds, or a list that got shorter: back to the top.
        assert_eq!(scroll_top(0, 3, 5, 9), 0);
        assert_eq!(scroll_top(9, 2, 5, 9), 0);
        assert_eq!(scroll_top(9, 11, 12, 9), 3);
        assert_eq!(scroll_top(0, 0, 0, 0), 0);
    }
}
