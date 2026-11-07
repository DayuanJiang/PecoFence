//! Smart fences (智能栅栏): each one runs its search (an Everything search while Everything
//! runs, else a reading of its folder tested against the same conditions), shows the newest
//! results, watches its folder, and runs again after a change there and once a minute.

use super::*;
use pecofence_core::everything as ev;
use pecofence_core::smart::{KnownFolders, SmartSpec, Template};
use pecofence_platform::everything;
use pecofence_platform::shell::{DesktopEntry, EntryOrigin};
use std::sync::mpsc;

/// `dwData` of Everything's replies to smart fence searches: this tag plus a sequence number
/// (the Peek search uses another tag).
const REPLY_TAG: usize = 0x5053_0000;
/// Pause after a change in a watched folder before the search runs again: Everything indexes
/// the change first (it follows the drive's change journal, usually within a second).
const SOON_MS: u32 = 1_000;
/// A second run after a change, for an Everything that took longer to see it.
const AGAIN_MS: u32 = 4_000;
/// Everything answers one search at a time (a new one cancels the one before): the next
/// fence's search waits for the answer, or this long for one that never comes.
const ANSWER_WAIT: Duration = Duration::from_secs(3);

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
    /// The folder read whose result this fence waits for (a newer run drops an older one).
    read: Option<u32>,
}

/// A folder read without Everything, done on a worker.
pub(super) struct FolderRead {
    id: FenceId,
    seq: u32,
    found: Vec<DesktopEntry>,
    total: u32,
    scanned: usize,
    ms: u64,
}

pub(super) struct SmartState {
    pub runs: HashMap<FenceId, SmartRun>,
    seq: u16,
    /// Folder reads: the last sequence number, the results coming back from the workers, and
    /// the fences being read (true: another read waits for this one to end).
    read_seq: u32,
    read_tx: mpsc::Sender<FolderRead>,
    read_rx: mpsc::Receiver<FolderRead>,
    reading: HashMap<FenceId, bool>,
    /// The search Everything is working on (fence, sequence number, when it was sent) and the
    /// fences whose search waits for its answer.
    in_flight: Option<(FenceId, u16, Instant)>,
    waiting: VecDeque<FenceId>,
    /// One watcher per smart fence with a folder (its subfolders too when it searches them).
    watchers: HashMap<FenceId, (PathBuf, DirWatcher)>,
    /// Fences to run again when `TIMER_SMART_SOON` fires, and whether a second run follows.
    soon: HashSet<FenceId>,
    again: bool,
}

impl Default for SmartState {
    fn default() -> Self {
        let (read_tx, read_rx) = mpsc::channel();
        Self {
            runs: HashMap::new(),
            seq: 0,
            read_seq: 0,
            read_tx,
            read_rx,
            reading: HashMap::new(),
            in_flight: None,
            waiting: VecDeque::new(),
            watchers: HashMap::new(),
            soon: HashSet::new(),
            again: false,
        }
    }
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
        self.refind_template_folder(id);
        let Some(spec) = self.state.smart_spec(id).cloned() else {
            return;
        };
        if let Some(instance) = everything::find() {
            match self.smart.in_flight {
                Some((busy, _, at)) if at.elapsed() < ANSWER_WAIT => {
                    if !self.smart.waiting.contains(&id) {
                        self.smart.waiting.push_back(id);
                    }
                    tracing::debug!(%id, why, %busy, queued = self.smart.waiting.len(), "smart fence: waits for Everything's answer to another search");
                    return;
                }
                Some((lost, _, _)) => {
                    tracing::info!(%lost, "smart fence: Everything did not answer; its search runs again later");
                    if lost != id && !self.smart.waiting.contains(&lost) {
                        self.smart.waiting.push_back(lost);
                    }
                    self.smart.in_flight = None;
                }
                None => {}
            }
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
                spec.everything_max(),
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
                run.read = None;
                self.smart.in_flight = Some((id, seq, Instant::now()));
                return;
            }
        }
        if !spec.works_without_everything() {
            tracing::info!(%id, why, "smart fence: needs Everything, which is not running");
            self.smart_done(id, SmartSource::NeedsEverything, Vec::new(), 0, None);
            return;
        }
        self.read_folder_later(id, spec, why);
    }

    /// Reads smart fence `id`'s folder on a worker (a big folder tree takes a while); the result
    /// comes back through `WM_APP_SMART_READ`. One read per fence at a time: a run asked for
    /// meanwhile follows the one in progress.
    fn read_folder_later(&mut self, id: FenceId, spec: SmartSpec, why: &str) {
        if let Some(again) = self.smart.reading.get_mut(&id) {
            *again = true;
            tracing::debug!(%id, why, "smart fence: a folder read runs already; another follows it");
            return;
        }
        self.smart.read_seq = self.smart.read_seq.wrapping_add(1);
        let seq = self.smart.read_seq;
        self.smart.reading.insert(id, false);
        self.smart.runs.entry(id).or_default().read = Some(seq);
        let tx = self.smart.read_tx.clone();
        let control = self.control.hwnd().0 as isize;
        tracing::info!(%id, why, seq, "smart fence: reading the folder without Everything");
        let spawned = std::thread::Builder::new()
            .name("pecofence-smart-read".into())
            .spawn(move || {
                // Shortcut targets are read through COM.
                let _com = spec
                    .needs_shortcut_targets()
                    .then(pecofence_platform::com::MtaGuard::init);
                let started = Instant::now();
                let folder = PathBuf::from(spec.folder.as_deref().unwrap_or_default());
                let (mut found, scanned) = read_folder(&spec, &folder);
                let total = found.len() as u32;
                found.sort_by_key(|e| std::cmp::Reverse(e.mtime));
                found.truncate(spec.limit as usize);
                let ms = started.elapsed().as_millis() as u64;
                let _ = tx.send(FolderRead {
                    id,
                    seq,
                    found,
                    total,
                    scanned,
                    ms,
                });
                window::post_message(
                    HWND(control as *mut core::ffi::c_void),
                    crate::commands::WM_APP_SMART_READ,
                    0,
                    0,
                );
            });
        if let Err(e) = spawned {
            tracing::warn!(%id, error = %e, "smart fence: the folder read did not start");
            self.smart.reading.remove(&id);
        }
    }

    /// `TIMER_SMART_READ`: the folder reads that finished. A result an Everything answer or a
    /// newer read replaced meanwhile is dropped.
    pub(super) fn drain_smart_reads(&mut self) {
        while let Ok(r) = self.smart.read_rx.try_recv() {
            let again = self.smart.reading.remove(&r.id).unwrap_or(false);
            let current = self.smart.runs.get(&r.id).and_then(|run| run.read) == Some(r.seq);
            if current && self.state.smart_spec(r.id).is_some() {
                tracing::info!(id = %r.id, seq = r.seq, scanned = r.scanned, total = r.total, ms = r.ms, "smart fence: folder read without Everything");
                self.smart.runs.entry(r.id).or_default().read = None;
                self.smart_done(r.id, SmartSource::Folder, r.found, r.total, None);
            } else {
                tracing::debug!(id = %r.id, seq = r.seq, "smart fence: an older folder read dropped");
            }
            if again {
                self.run_smart(r.id, "after the read in progress");
            }
        }
    }

    /// Everything's answer to a smart fence's search (`WM_COPYDATA` to the control window).
    pub(super) fn on_smart_results(&mut self, kind: usize, bytes: &[u8]) {
        if kind & !0xFFFF != REPLY_TAG {
            return;
        }
        let seq = (kind & 0xFFFF) as u16;
        // Everything is free again whatever happens to this answer: the next search may go.
        if self.smart.in_flight.is_some_and(|(_, s, _)| s == seq) {
            self.smart.in_flight = None;
        }
        let Some(id) = self
            .smart
            .runs
            .iter()
            .find(|(_, r)| r.pending == Some(seq))
            .map(|(id, _)| *id)
        else {
            tracing::debug!(seq, "smart fence: answer to an older search dropped");
            self.ask_next_smart();
            return;
        };
        let Some(spec) = self.state.smart_spec(id).cloned() else {
            self.ask_next_smart();
            return;
        };
        let Some((total, hits)) = ev::parse_list2(bytes) else {
            tracing::warn!(%id, len = bytes.len(), "smart fence: Everything's answer could not be read");
            if let Some(run) = self.smart.runs.get_mut(&id) {
                run.pending = None;
            }
            self.ask_next_smart();
            return;
        };
        // Everything matched on its index; the files themselves say what they are now (gone,
        // hidden, system) and get the conditions Everything cannot test.
        let now = pecofence_core::now_unix();
        let targets = spec.needs_shortcut_targets();
        let typed = spec.everything.is_some();
        let got = hits.len();
        let mut entries: Vec<DesktopEntry> = hits
            .iter()
            .filter_map(|h| shell::stat_entry(Path::new(&h.path), EntryOrigin::UserDesktop))
            .filter(|e| typed || entry_matches(&spec, e, targets, now))
            .collect();
        // What was left out here is left out of the total too.
        let total = total.saturating_sub((got - entries.len()) as u32);
        let kept = entries.len();
        entries.truncate(spec.limit as usize);
        tracing::info!(%id, total, got, kept, shown = entries.len(), "smart fence: Everything answered");
        self.smart_done(id, SmartSource::Everything, entries, total, Some(seq));
        // The next fence's search goes once this answer is in: sent earlier, it could mark the
        // fence's own newer search as answered.
        self.ask_next_smart();
    }

    /// Sends the next waiting fence's search once Everything has answered the one before.
    fn ask_next_smart(&mut self) {
        while self.smart.in_flight.is_none() {
            let Some(next) = self.smart.waiting.pop_front() else {
                return;
            };
            if self.state.smart_spec(next).is_some() {
                self.run_smart(next, "waited for another search");
            }
        }
    }

    /// Puts a run's results into the fence and redraws it when they changed.
    /// `answered`: the Everything search these results answer; a newer one sent for the fence
    /// meanwhile stays pending. None (a folder read, no Everything): no search is pending any more.
    fn smart_done(
        &mut self,
        id: FenceId,
        source: SmartSource,
        entries: Vec<DesktopEntry>,
        total: u32,
        answered: Option<u16>,
    ) {
        let count = entries.len();
        let changed = self.state.set_smart_results(id, entries);
        let run = self.smart.runs.entry(id).or_default();
        let source_changed = run.source != source || run.total != total;
        run.source = source;
        run.total = total;
        if answered.is_none_or(|seq| run.pending == Some(seq)) {
            run.pending = None;
        }
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

    /// One change to smart fence `fence`'s search (`fence set`, the settings page): saved, the
    /// watcher follows a new folder, and the search runs again. False when nothing changed.
    pub(super) fn change_smart(
        &mut self,
        fence: FenceId,
        change: super::fence_options::SmartChange,
    ) -> bool {
        use super::fence_options::SmartChange as C;
        let Some(spec) = self.state.fence_mut(fence).and_then(|f| f.smart.as_mut()) else {
            tracing::info!(%fence, "smart fence change refused: not a smart fence");
            return false;
        };
        let before = spec.clone();
        let what = match change {
            C::Folder(folder) => {
                spec.folder = folder;
                "folder"
            }
            C::Subfolders(on) => {
                spec.subfolders = on;
                "subfolders"
            }
            C::WithinDays(days) => {
                spec.within_days = days;
                "within days"
            }
            C::Conditions(conds) => {
                spec.all_of = conds;
                "conditions"
            }
            C::Name(name) => {
                spec.name = name;
                "name"
            }
            C::Everything(text) => {
                spec.everything = text;
                "everything search"
            }
            C::Limit(n) => {
                spec.limit = n;
                "limit"
            }
        };
        if *spec == before {
            return false;
        }
        // A folder picked by hand is the user's own: an import keeps it as it is.
        if spec.folder != before.folder {
            spec.template = None;
        }
        let (folder, days, conds, limit) = (
            spec.folder.is_some(),
            spec.within_days,
            spec.all_of.len(),
            spec.limit,
        );
        tracing::info!(%fence, what, folder, within_days = ?days, conditions = conds, limit, "smart fence search changed");
        self.state.mark_dirty();
        self.schedule_save();
        self.ensure_smart_watchers();
        self.run_smart(fence, "search changed");
        self.apply_portal_deco(self.state.host_of(fence));
        true
    }

    /// A template's folder that is not on this computer (a configuration imported from another
    /// one, a moved Downloads folder): this computer's folder of the same kind takes its place.
    fn refind_template_folder(&mut self, id: FenceId) {
        let Some(spec) = self.state.smart_spec(id) else {
            return;
        };
        let (Some(key), Some(folder)) = (spec.template.clone(), spec.folder.clone()) else {
            return;
        };
        if Path::new(&folder).is_dir() {
            return;
        }
        let found = Template::parse(&key)
            .and_then(|t| t.spec(&known_folders()))
            .and_then(|s| s.folder)
            .filter(|f| Path::new(f).is_dir());
        let Some(found) = found else {
            tracing::info!(%id, template = %key, "smart fence: the template's folder is missing here too");
            return;
        };
        tracing::info!(%id, template = %key, "smart fence: the template's folder found on this computer");
        if let Some(spec) = self.state.fence_mut(id).and_then(|f| f.smart.as_mut()) {
            spec.folder = Some(found);
        }
        self.state.mark_dirty();
        self.schedule_save();
        self.ensure_smart_watchers();
    }

    /// The settings page's 「选择文件夹…」 for smart fence `fence`.
    pub(super) fn pick_smart_folder(&mut self, fence: FenceId) {
        let start = self.state.smart_spec(fence).and_then(|s| s.folder.clone());
        tracing::info!(%fence, has_folder = start.is_some(), "smart fence: picking a folder");
        let owner = self.settings.as_ref().map(|h| h.hwnd());
        match pecofence_platform::filedialog::pick_folder(
            owner,
            pecofence_core::i18n::text("选择智能栅栏要搜索的文件夹"),
        ) {
            Ok(Some(dir)) => {
                let dir = crate::state::canonical_path(dir)
                    .to_string_lossy()
                    .into_owned();
                self.change_smart(fence, super::fence_options::SmartChange::Folder(Some(dir)));
            }
            Ok(None) => tracing::info!(%fence, "smart fence: folder picker cancelled"),
            Err(e) => tracing::warn!(error = %e, "smart fence: folder picker failed"),
        }
        self.push_settings_state();
    }

    /// Whether smart fence `id` waits for Everything (every drive, or a typed Everything
    /// search, while it does not run).
    pub(super) fn smart_needs_everything(&self, id: FenceId) -> bool {
        self.smart
            .runs
            .get(&id)
            .is_some_and(|r| r.source == SmartSource::NeedsEverything)
    }

    /// Everything was started or installed: the smart fences that waited for it search again.
    pub(super) fn run_waiting_smart(&mut self) {
        let waiting: Vec<FenceId> = self
            .smart
            .runs
            .iter()
            .filter(|(_, r)| r.source == SmartSource::NeedsEverything)
            .map(|(id, _)| *id)
            .collect();
        if !waiting.is_empty() {
            tracing::info!(
                fences = waiting.len(),
                "smart fences waiting for Everything search again"
            );
        }
        for id in waiting {
            self.run_smart(id, "everything started");
        }
    }

    /// 把筛选存为智能栅栏: a new smart fence showing what filtered fence `fence` shows now: the
    /// folder it shows (only that folder, as the folder fence does) or its own search, and the
    /// filter as its name condition.
    pub(super) fn save_filter_as_smart(&mut self, fence: FenceId, x: i32, y: i32) {
        let Some(filter) = self.fence_filters.get(&fence).map(|t| t.trim().to_string()) else {
            return;
        };
        let spec = match (self.state.smart_spec(fence), self.state.portal_path(fence)) {
            (Some(spec), _) => SmartSpec {
                template: None,
                name: Some(filter.clone()),
                ..spec.clone()
            },
            (None, Some(dir)) => SmartSpec {
                template: None,
                folder: Some(dir.to_string_lossy().into_owned()),
                subfolders: false,
                all_of: Vec::new(),
                within_days: None,
                name: Some(filter.clone()),
                everything: None,
                limit: SmartSpec::DEFAULT_LIMIT,
            },
            (None, None) => return,
        };
        let base = self
            .state
            .fence(fence)
            .map(|f| self.state.display_title(f))
            .unwrap_or_default();
        let title = pecofence_core::i18n::format("{0}：{1}", &[base, filter]);
        let folder = spec.folder.is_some();
        if let Some(id) = self.create_smart_fence(spec, Some(&title), None, x, y) {
            tracing::info!(from = %fence, %id, folder, "filter saved as a smart fence");
        }
    }

    /// The Peek search's 「把这次搜索放到桌面」: a smart fence with the same Everything search
    /// (pinyin included when Everything has it) over every drive, titled after the text.
    pub(super) fn search_to_desktop(&mut self, query: &str) {
        let Some(instance) = everything::find() else {
            tracing::info!("search to desktop: Everything is no longer running");
            return;
        };
        let spec = SmartSpec {
            template: None,
            folder: None,
            subfolders: true,
            all_of: Vec::new(),
            within_days: None,
            name: None,
            everything: Some(ev::search_text(query, instance.supports_pinyin())),
            limit: SmartSpec::DEFAULT_LIMIT,
        };
        let title = pecofence_core::i18n::format("搜索“{0}”", &[query.to_string()]);
        let (x, y) = self
            .state
            .work_areas
            .first()
            .map_or((0, 0), |w| ((w.left + w.right) / 2, (w.top + w.bottom) / 2));
        if let Some(id) = self.create_smart_fence(spec, Some(&title), None, x, y) {
            tracing::info!(%id, chars = query.chars().count(), "search put on the desktop as a smart fence");
        }
    }

    /// Smart fence `id`'s empty-state text and the count after its name: the files it shows
    /// (after a filter), with a "+" when the limit cut the results. None for other fences.
    pub(super) fn smart_view(
        &self,
        f: &pecofence_core::Fence,
    ) -> (Option<&'static str>, Option<String>) {
        let Some(spec) = f.smart.as_ref() else {
            return (None, None);
        };
        let run = self.smart.runs.get(&f.id);
        let source = run.map_or(SmartSource::Waiting, |r| r.source);
        let empty = Some(match source {
            SmartSource::Waiting => pecofence_core::i18n::text("正在搜索…"),
            SmartSource::NeedsEverything => pecofence_core::i18n::text(
                "要搜索整台电脑，需要先打开 Everything：右键这个栅栏可以启动或安装它。",
            ),
            SmartSource::Everything | SmartSource::Folder => {
                pecofence_core::i18n::text("没有符合条件的文件。")
            }
        });
        let count = matches!(source, SmartSource::Everything | SmartSource::Folder).then(|| {
            let shown = self.item_views(f).len();
            let cut =
                run.is_some_and(|r| r.total > spec.limit) && self.fence_filter(f.id).is_none();
            format!(" · {shown}{}", if cut { "+" } else { "" })
        });
        (empty, count)
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
    spec.matches(&facts, e.mtime.max(e.created), now)
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
