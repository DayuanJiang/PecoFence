//! Smart fences (智能栅栏): each one runs its search (an Everything search while Everything
//! runs, else a reading of its folder tested against the same conditions), shows the newest
//! results, watches its folder, and runs again after a change there and once a minute.

use super::*;
use pecofence_core::everything as ev;
use pecofence_core::smart::{KnownFolders, SmartSpec, Template};
use pecofence_platform::everything;
use pecofence_platform::shell::{DesktopEntry, EntryOrigin};

/// `dwData` of Everything's replies to smart fence searches: this tag plus a sequence number
/// (the Peek search uses another tag).
const REPLY_TAG: usize = 0x5053_0000;
/// Pause after a change in a watched folder before the search runs again: Everything indexes
/// the change first (it follows the drive's change journal, usually within a second).
const SOON_MS: u32 = 1_000;
/// A second run after a change, for an Everything that took longer to see it.
const AGAIN_MS: u32 = 4_000;

/// Where a smart fence's results came from at its last run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum SmartSource {
    /// Not run yet, or waiting for Everything's answer.
    #[default]
    Waiting,
    Everything,
    /// The folder read without Everything.
    Folder,
    /// Every drive (or a typed Everything search) while Everything is not running.
    NeedsEverything,
}

impl SmartSource {
    fn as_str(self) -> &'static str {
        match self {
            SmartSource::Waiting => "waiting",
            SmartSource::Everything => "everything",
            SmartSource::Folder => "folder",
            SmartSource::NeedsEverything => "needsEverything",
        }
    }
}

#[derive(Default)]
pub(super) struct SmartRun {
    pub source: SmartSource,
    /// Matches in all (Everything counts past the limit); the fence shows at most the limit.
    pub total: u32,
    /// The Everything search sent and the sequence number its answer carries.
    pub query: String,
    pending: Option<u16>,
}

#[derive(Default)]
pub(super) struct SmartState {
    pub runs: HashMap<FenceId, SmartRun>,
    seq: u16,
    /// One watcher per smart fence with a folder (its subfolders too when it searches them).
    watchers: HashMap<FenceId, (PathBuf, DirWatcher)>,
    /// Fences to run again when `TIMER_SMART_SOON` fires, and whether a second run follows.
    soon: HashSet<FenceId>,
    again: bool,
}

/// Which mark tells a smart fence from a normal one (a temporary switch for the screenshots of
/// the decision: `PECOFENCE_SAMPLE_SMART_MARK` = A magnifier before the title, B the count
/// after the title, C the count at the right of the title row).
pub(super) fn smart_mark() -> char {
    std::env::var("PECOFENCE_SAMPLE_SMART_MARK")
        .ok()
        .and_then(|v| v.trim().chars().next())
        .map(|c| c.to_ascii_uppercase())
        .filter(|c| matches!(c, 'A' | 'B' | 'C'))
        .unwrap_or('A')
}

/// The folders the templates search on this computer.
pub(super) fn known_folders() -> KnownFolders {
    let text = |p: Option<PathBuf>| p.map(|p| p.to_string_lossy().into_owned());
    KnownFolders {
        downloads: text(shell::user_downloads()),
        screenshots: text(shell::user_screenshots()),
        recent: text(shell::user_recent()),
    }
}

impl App {
    /// 新建智能栅栏 (tray and fence menu), command ids `base..base + Template::ALL.len()`.
    pub(super) fn new_smart_submenu(base: u32) -> PopupMenu {
        let menu = PopupMenu::new();
        for (i, t) in Template::ALL.iter().enumerate() {
            menu.item(base + i as u32, t.title(), false, false);
        }
        menu
    }

    /// A smart fence from template `template` near the screen point.
    pub(super) fn new_smart_from_template(&mut self, template: Template, x: i32, y: i32) {
        let Some(spec) = template.spec(&known_folders()) else {
            tracing::warn!(
                template = template.key(),
                "smart fence: the template's folder is unknown"
            );
            return;
        };
        self.create_smart_fence(spec, Some(template.title()), None, x, y);
    }

    /// Creates a smart fence (menu, CLI) and runs its search at once.
    pub(super) fn create_smart_fence(
        &mut self,
        spec: SmartSpec,
        title: Option<&str>,
        rect: Option<RECT>,
        x: i32,
        y: i32,
    ) -> Option<FenceId> {
        if !self.fence_room_or_notice() {
            return None;
        }
        let rect = rect.unwrap_or_else(|| self.place_new_fence(4, 260.0, x, y, None));
        let title = title
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| pecofence_core::i18n::text("智能栅栏").to_string());
        let id = self.state.new_smart_fence(&title, rect, spec)?;
        self.resync_windows();
        if let Some(w) = self.fences.get(&id) {
            w.show(true);
        }
        self.ensure_smart_watchers();
        self.run_smart(id, "created");
        self.schedule_save();
        Some(id)
    }

    /// Runs every smart fence's search (startup, once a minute, Everything started).
    pub(super) fn run_all_smart(&mut self, why: &str) {
        let ids: Vec<FenceId> = self
            .state
            .fences()
            .iter()
            .filter(|f| f.smart.is_some())
            .map(|f| f.id)
            .collect();
        // Fences that are gone (deleted, another layout shown) stop being searched.
        self.smart.runs.retain(|id, _| ids.contains(id));
        self.ensure_smart_watchers();
        for id in ids {
            self.run_smart(id, why);
        }
    }

    /// Runs smart fence `id`'s search: asks Everything when it runs (the answer comes to the
    /// control window), else reads the folder.
    pub(super) fn run_smart(&mut self, id: FenceId, why: &str) {
        let Some(spec) = self.state.smart_spec(id).cloned() else {
            return;
        };
        if let Some(instance) = everything::find() {
            let since = spec.within_days.and_then(|days| {
                let start = pecofence_core::now_unix() - i64::from(days) * 86_400;
                pecofence_platform::fileinfo::local_civil_date(start).map(|(y, m, d)| {
                    pecofence_core::smart::everything_date(pecofence_core::CivilDate::new(y, m, d))
                })
            });
            let query = spec.everything_query(since.as_deref());
            self.smart.seq = self.smart.seq.wrapping_add(1);
            let seq = self.smart.seq;
            let payload = ev::query2w(
                self.control.hwnd().0 as usize as u32,
                (REPLY_TAG | usize::from(seq)) as u32,
                &query,
                spec.limit,
                ev::SORT_DATE_MODIFIED_DESCENDING,
            );
            let sent = everything::send_query(
                instance.hwnd,
                self.control.hwnd(),
                ev::COPYDATA_QUERY2W,
                &payload,
            );
            tracing::info!(%id, why, sent, version = %format!("{}.{}", instance.major, instance.minor), search = %query, "smart fence: Everything asked");
            if sent {
                let run = self.smart.runs.entry(id).or_default();
                run.query = query;
                run.pending = Some(seq);
                return;
            }
        }
        if !spec.works_without_everything() {
            tracing::info!(%id, why, "smart fence: needs Everything, which is not running");
            self.smart_done(id, SmartSource::NeedsEverything, Vec::new(), 0);
            return;
        }
        let started = Instant::now();
        let folder = PathBuf::from(spec.folder.as_deref().unwrap_or_default());
        let (found, scanned) = read_folder(&spec, &folder);
        let total = found.len() as u32;
        let mut found = found;
        found.sort_by_key(|e| std::cmp::Reverse(e.mtime));
        found.truncate(spec.limit as usize);
        tracing::info!(%id, why, folder = %folder.display(), scanned, total, ms = crate::share_log::millis(started.elapsed()), "smart fence: folder read without Everything");
        self.smart_done(id, SmartSource::Folder, found, total);
    }

    /// Everything's answer to a smart fence's search (`WM_COPYDATA` to the control window).
    pub(super) fn on_smart_results(&mut self, kind: usize, bytes: &[u8]) {
        if kind & !0xFFFF != REPLY_TAG {
            return;
        }
        let seq = (kind & 0xFFFF) as u16;
        let Some(id) = self
            .smart
            .runs
            .iter()
            .find(|(_, r)| r.pending == Some(seq))
            .map(|(id, _)| *id)
        else {
            tracing::debug!(seq, "smart fence: answer to an older search dropped");
            return;
        };
        let Some(spec) = self.state.smart_spec(id).cloned() else {
            return;
        };
        let Some((total, hits)) = ev::parse_list2(bytes) else {
            tracing::warn!(%id, len = bytes.len(), "smart fence: Everything's answer could not be read");
            return;
        };
        // Everything matched on its index; the files themselves say what they are now (gone,
        // hidden, system) and get the conditions Everything cannot test.
        let now = pecofence_core::now_unix();
        let targets = spec.needs_shortcut_targets();
        let typed = spec.everything.is_some();
        let got = hits.len();
        let entries: Vec<DesktopEntry> = hits
            .iter()
            .filter_map(|h| shell::stat_entry(Path::new(&h.path), EntryOrigin::UserDesktop))
            .filter(|e| typed || entry_matches(&spec, e, targets, now))
            .collect();
        // What was left out here is left out of the total too.
        let total = total.saturating_sub((got - entries.len()) as u32);
        tracing::info!(%id, total, got, kept = entries.len(), "smart fence: Everything answered");
        self.smart_done(id, SmartSource::Everything, entries, total);
    }

    /// Puts a run's results into the fence and redraws it when they changed.
    fn smart_done(
        &mut self,
        id: FenceId,
        source: SmartSource,
        entries: Vec<DesktopEntry>,
        total: u32,
    ) {
        let count = entries.len();
        let changed = self.state.set_smart_results(id, entries);
        let run = self.smart.runs.entry(id).or_default();
        let source_changed = run.source != source || run.total != total;
        run.source = source;
        run.total = total;
        run.pending = None;
        if changed || source_changed {
            tracing::info!(%id, source = source.as_str(), count, total, "smart fence results changed");
        }
        if changed {
            self.refresh_fence(id);
        } else if source_changed {
            self.apply_portal_deco(self.state.host_of(id));
        }
    }

    /// Starts and stops the folder watchers of the smart fences.
    pub(super) fn ensure_smart_watchers(&mut self) {
        let wanted: Vec<(FenceId, PathBuf, bool)> = self
            .state
            .fences()
            .iter()
            .filter_map(|f| {
                let spec = f.smart.as_ref()?;
                Some((f.id, PathBuf::from(spec.folder.as_ref()?), spec.subfolders))
            })
            .collect();
        self.smart.watchers.retain(|id, (dir, w)| {
            w.is_running() && wanted.iter().any(|(wid, wdir, _)| wid == id && wdir == dir)
        });
        for (id, dir, subfolders) in wanted {
            if self.smart.watchers.contains_key(&id) {
                continue;
            }
            let pending = self.fs_pending.clone();
            let control = self.control.hwnd().0 as isize;
            let on_events = move |events: Vec<FsEvent>| {
                if let Ok(mut p) = pending.lock() {
                    p.extend(events);
                }
                window::post_message(
                    HWND(control as *mut core::ffi::c_void),
                    WM_APP_FS_CHANGED,
                    0,
                    0,
                );
            };
            let started = if subfolders {
                DirWatcher::start_tree(&dir, on_events)
            } else {
                DirWatcher::start(&dir, on_events)
            };
            match started {
                Ok(w) => {
                    tracing::info!(%id, dir = %dir.display(), subfolders, "smart fence watcher started");
                    self.smart.watchers.insert(id, (dir, w));
                }
                Err(e) => {
                    tracing::warn!(%id, dir = %dir.display(), error = %e, "smart fence watcher failed")
                }
            }
        }
    }

    /// A watcher batch named `dirs` (`all`: it overflowed): the smart fences whose folder holds
    /// one of them run again shortly.
    pub(super) fn smart_folders_changed(&mut self, dirs: &[PathBuf], all: bool) {
        let lower = |p: &Path| p.to_string_lossy().to_lowercase();
        let dirs: Vec<String> = dirs.iter().map(|d| lower(d)).collect();
        let hit: Vec<FenceId> = self
            .state
            .fences()
            .iter()
            .filter_map(|f| {
                let spec = f.smart.as_ref()?;
                let folder = spec
                    .folder
                    .as_deref()?
                    .trim_end_matches('\\')
                    .to_lowercase();
                let inside = dirs.iter().any(|d| {
                    *d == folder || (spec.subfolders && d.starts_with(&format!("{folder}\\")))
                });
                (all || inside).then_some(f.id)
            })
            .collect();
        if hit.is_empty() {
            return;
        }
        tracing::debug!(fences = hit.len(), "smart fence folders changed");
        self.smart.soon.extend(hit);
        self.smart.again = true;
        window::set_timer(self.control.hwnd(), TIMER_SMART_SOON, SOON_MS);
    }

    /// `TIMER_SMART_SOON`: runs the fences a folder change named, once more a few seconds later.
    pub(super) fn run_soon_smart(&mut self) {
        let ids: Vec<FenceId> = self.smart.soon.iter().copied().collect();
        for id in &ids {
            self.run_smart(*id, "folder changed");
        }
        if std::mem::take(&mut self.smart.again) {
            window::set_timer(self.control.hwnd(), TIMER_SMART_SOON, AGAIN_MS);
        } else {
            self.smart.soon.clear();
        }
    }

    /// The fence's empty-state text and its count caption (mark C).
    pub(super) fn smart_view(&self, id: FenceId) -> (Option<&'static str>, Option<String>) {
        let Some(spec) = self.state.smart_spec(id) else {
            return (None, None);
        };
        let run = self.smart.runs.get(&id);
        let source = run.map_or(SmartSource::Waiting, |r| r.source);
        let empty = Some(match source {
            SmartSource::Waiting => pecofence_core::i18n::text("正在搜索…"),
            SmartSource::NeedsEverything => {
                pecofence_core::i18n::text("要搜索整台电脑，需要先打开 Everything。")
            }
            SmartSource::Everything | SmartSource::Folder => {
                pecofence_core::i18n::text("没有符合条件的文件。")
            }
        });
        let caption = (smart_mark() == 'C' && source != SmartSource::NeedsEverything).then(|| {
            let shown = run.map_or(0, |r| r.total).min(spec.limit);
            pecofence_core::i18n::format("{0} 个，自动更新", &[shown.to_string()])
        });
        (empty, caption)
    }

    /// The title of smart fence `id` with mark B (its count after it), else as it is.
    pub(super) fn smart_title(&self, id: FenceId, title: String) -> String {
        if self.state.smart_spec(id).is_none() || smart_mark() != 'B' {
            return title;
        }
        match self.smart.runs.get(&id) {
            Some(r) if r.source != SmartSource::Waiting => {
                format!(
                    "{title} · {}",
                    r.total
                        .min(self.state.smart_spec(id).map_or(0, |s| s.limit))
                )
            }
            _ => title,
        }
    }

    /// 智能栅栏 for `fence get` / `fence list`: the search and its last run.
    pub(super) fn smart_dto(&self, id: FenceId) -> Option<serde_json::Value> {
        let spec = self.state.smart_spec(id)?;
        let run = self.smart.runs.get(&id);
        Some(serde_json::json!({
            "spec": spec,
            "source": run.map_or("waiting", |r| r.source.as_str()),
            "total": run.map_or(0, |r| r.total),
            "shown": self.state.fence(id).map_or(0, |f| self.state.items_of(f).len()),
            "query": run.map(|r| r.query.clone()).filter(|q| !q.is_empty()),
        }))
    }
}

/// Whether a found file belongs in the fence (every condition, the time window).
fn entry_matches(spec: &SmartSpec, e: &DesktopEntry, targets: bool, now: i64) -> bool {
    let facts = if targets {
        let mut f = crate::state::AppState::plain_facts(e, None, now);
        f.shortcut_target = shell::shortcut_target(&e.path);
        f.shortcut_target_ext = f
            .shortcut_target
            .as_deref()
            .map(pecofence_core::rules::ext_of);
        f
    } else {
        crate::state::AppState::plain_facts(e, None, now)
    };
    spec.matches(&facts, e.mtime, now)
}

/// Reads `folder` (and its subfolders when the search takes them) without Everything, at most
/// `SmartSpec::MAX_SCANNED` entries, and keeps what matches. Hidden and system entries, and
/// folder links (which may loop), are left out. Returns the matches and how many were read.
fn read_folder(spec: &SmartSpec, folder: &Path) -> (Vec<DesktopEntry>, usize) {
    let now = pecofence_core::now_unix();
    let targets = spec.needs_shortcut_targets();
    let mut found = Vec::new();
    let mut scanned = 0usize;
    let mut stack = vec![folder.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for e in shell::enumerate_folder(&dir) {
            scanned += 1;
            if scanned > SmartSpec::MAX_SCANNED {
                return (found, scanned);
            }
            const REPARSE_POINT: u32 = 0x400;
            if e.is_folder && spec.subfolders && e.attributes & REPARSE_POINT == 0 {
                stack.push(e.path.clone());
            }
            if entry_matches(spec, &e, targets, now) {
                found.push(e);
            }
        }
    }
    (found, scanned)
}
