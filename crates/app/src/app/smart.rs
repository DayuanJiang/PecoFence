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
/// The same pause while Everything does not run: PecoFence reads the folder itself, so there is
/// no index to wait for, only the burst of events one saved file makes.
const SOON_NO_INDEX_MS: u32 = 250;
/// A second run after a change, for an Everything that took longer to see it.
const AGAIN_MS: u32 = 4_000;
/// Everything answers one search at a time (a new one cancels the one before): the next
/// fence's search waits for the answer, or this long for one that never comes.
pub(super) const ANSWER_WAIT: Duration = Duration::from_secs(3);
/// While a fence waits for Everything, its window is looked for this often: an Everything
/// started by hand, or after PecoFence at logon, is used within seconds (the once-a-minute run
/// left the fence asking for it up to a minute longer).
const WAIT_POLL_MS: u32 = 2_000;
/// The same look right after 「启动 Everything」 was picked.
const STARTING_POLL_MS: u32 = 250;
/// How long the waiting fences say 「正在搜索…」 after 「启动 Everything」; an Everything that is
/// not there by then did not start, and the row comes back.
const STARTING_FOR: Duration = Duration::from_secs(10);
/// How long an Everything that is there but says its index is not read yet is waited for (a
/// search sent before that finds nothing); after it the search is sent anyway, and this
/// Everything is no longer asked to wait for (`SmartState::index_unknown`).
const INDEX_WAIT: Duration = Duration::from_secs(10);
/// The smart fence editor's live count runs as this make-believe fence: it takes its turn with
/// Everything (after the Peek search, between the fences' own runs) and its workers like a
/// fence's, and its results go to the editor instead of a fence.
pub(super) const PREVIEW: FenceId =
    uuid::Uuid::from_u128(0x5053_4d41_5254_0000_0000_0000_0000_0001);

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
    /// The worker (a folder read, or the check of Everything's answer) whose result this fence
    /// waits for (a newer run drops an older one).
    read: Option<u32>,
    /// The run in progress is the once-a-minute one (see `smart_line`).
    routine: bool,
    /// The last run found nothing and none of the fence's folders is there (deleted, renamed,
    /// a drive that is unplugged): the fence says so in place of "no matching files".
    pub missing: bool,
}

/// A log line of a smart fence's run: info for a run something asked for (startup, F5, the
/// CLI, a changed folder), debug for the once-a-minute run. Its two lines per fence and minute
/// were all an idle PecoFence wrote at the default level, about 26 KB an hour for each smart
/// fence; what such a run changes is still logged at info (`smart fence results changed`).
macro_rules! smart_line {
    ($routine:expr, $($arg:tt)*) => {
        if $routine {
            tracing::debug!($($arg)*)
        } else {
            tracing::info!($($arg)*)
        }
    };
}

/// What a smart fence's worker found: the results kept (newest first, at most the limit), the
/// matches in all, the entries looked at and how many matched before the limit cut them.
struct Found {
    entries: Vec<DesktopEntry>,
    total: u32,
    scanned: usize,
    kept: usize,
    /// Nothing found and none of the folders is there (see `SmartRun::missing`).
    missing: bool,
}

/// A smart fence's results worked out on a worker: a folder read without Everything, or
/// Everything's answer checked file by file. The results come named.
pub(super) struct FolderRead {
    id: FenceId,
    seq: u32,
    source: SmartSource,
    found: Vec<(DesktopEntry, String)>,
    total: u32,
    scanned: usize,
    kept: usize,
    missing: bool,
    /// Time spent finding (reading or checking) and naming the results.
    ms: u64,
    names_ms: u64,
}

pub(super) struct SmartState {
    pub runs: HashMap<FenceId, SmartRun>,
    seq: u16,
    /// Workers (folder reads, checks of Everything's answers): the last sequence number, the
    /// results coming back, and the fences whose results a worker is working out (true: the
    /// fence runs again once they are in). One worker per fence at a time.
    read_seq: u32,
    read_tx: mpsc::Sender<FolderRead>,
    read_rx: mpsc::Receiver<FolderRead>,
    reading: HashMap<FenceId, bool>,
    /// The search Everything is working on (fence, sequence number, when it was sent) and the
    /// fences whose search waits for its answer.
    in_flight: Option<(FenceId, u16, Instant)>,
    waiting: VecDeque<FenceId>,
    /// One watcher per folder of a smart fence (its subfolders too when the search takes
    /// them), and the fences whose watcher saw a change since the last look. Kept apart from
    /// the desktop's batch: a change (or an overflow) there never syncs the desktop.
    watchers: HashMap<(FenceId, PathBuf), (bool, DirWatcher)>,
    changed: Arc<Mutex<HashSet<FenceId>>>,
    /// Fences to run again when `TIMER_SMART_SOON` fires, and whether a second run follows.
    soon: HashSet<FenceId>,
    again: bool,
    /// The editor's search to count (`PREVIEW`) and the number of the page's request, and the
    /// number of the request the run in progress counts.
    preview: Option<(SmartSpec, u64)>,
    preview_sent: u64,
    /// Everything's total in its last answer to the editor's count.
    preview_total: u32,
    /// `TIMER_SMART_WAIT` is set: a fence waits for Everything, whose window is looked for.
    wait_poll: bool,
    /// When 「启动 Everything」 was picked: until Everything is there (or `STARTING_FOR`) the
    /// fences that wait for it say 「正在搜索…」, without the row.
    starting: Option<Instant>,
    /// When the look for Everything first saw its window (see `INDEX_WAIT`).
    seen: Option<Instant>,
    /// Everything was there for `INDEX_WAIT` without saying that its index is read: it is not
    /// waited for again (until it says so once).
    index_unknown: bool,
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
            changed: Arc::new(Mutex::new(HashSet::new())),
            soon: HashSet::new(),
            again: false,
            preview: None,
            preview_sent: 0,
            preview_total: 0,
            wait_poll: false,
            starting: None,
            seen: None,
            index_unknown: false,
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
    /// Creates a smart fence (the editor, the CLI, Peek) and runs its search at once.
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
        self.smart
            .runs
            .retain(|id, _| ids.contains(id) || *id == PREVIEW);
        self.ensure_smart_watchers();
        for id in ids {
            self.run_smart(id, why);
        }
    }

    /// Runs smart fence `id`'s search: asks Everything when it runs (the answer comes to the
    /// control window), else reads the folder.
    pub(super) fn run_smart(&mut self, id: FenceId, why: &str) {
        self.refind_template_folder(id);
        let Some(spec) = self.smart_spec_of(id) else {
            return;
        };
        let routine = why == "minute";
        // One worker per fence: a run asked for while one works out the fence's results (a
        // slow drive, a folder that keeps changing) follows it, so those results still show
        // and no threads pile up.
        if let Some(again) = self.smart.reading.get_mut(&id) {
            *again = true;
            tracing::debug!(%id, why, "smart fence: its results are being worked out; it runs again after them");
            return;
        }
        let mut instance = everything::find();
        // At sign-in Everything may be there before it has read its index, and a search sent
        // then finds nothing: the fence said 「没有符合条件的文件。」 until the next once-a-minute
        // run. Such an Everything counts as not running for the first search (a folder is read
        // by PecoFence, a whole-computer fence waits), and the look for Everything sends the
        // search once the index is read (`look_for_everything`).
        if why == "startup" && !self.smart.index_unknown && instance.is_some_and(|i| !i.db_loaded())
        {
            tracing::info!(%id, "smart fence: Everything has not read its index yet; it is waited for");
            self.smart.seen.get_or_insert_with(Instant::now);
            instance = None;
        }
        if let Some(instance) = instance {
            // The Peek search waits for Everything's answer: it goes first.
            if self.peek_waits_for_everything() {
                if !self.smart.waiting.contains(&id) {
                    self.smart.waiting.push_back(id);
                }
                tracing::debug!(%id, why, queued = self.smart.waiting.len(), "smart fence: waits for the Peek search's answer");
                return;
            }
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
            let now = pecofence_core::now_unix();
            let date_ago = |days: u32| {
                pecofence_platform::fileinfo::local_civil_date(now - i64::from(days) * 86_400).map(
                    |(y, m, d)| {
                        pecofence_core::smart::everything_date(pecofence_core::CivilDate::new(
                            y, m, d,
                        ))
                    },
                )
            };
            let query = spec.everything_query(&date_ago);
            self.smart.seq = self.smart.seq.wrapping_add(1);
            let seq = self.smart.seq;
            // The editor's count needs only Everything's total, unless the app checks a
            // condition on the files.
            let max = if id == PREVIEW && !spec.app_checks() {
                1
            } else {
                spec.everything_max()
            };
            if id == PREVIEW {
                self.smart.preview_sent = self.smart.preview.as_ref().map_or(0, |p| p.1);
            }
            // The editor's count checks the same files the fence does: the newest first.
            let payload = ev::query2w(
                self.control.hwnd().0 as usize as u32,
                (REPLY_TAG | usize::from(seq)) as u32,
                &query,
                max,
                ev::SORT_DATE_MODIFIED_DESCENDING,
            );
            let sent = everything::send_query(
                instance.hwnd,
                self.control.hwnd(),
                ev::COPYDATA_QUERY2W,
                &payload,
            );
            smart_line!(routine, %id, why, sent, version = %format!("{}.{}", instance.major, instance.minor), search = %query, "smart fence: Everything asked");
            if sent {
                let run = self.smart.runs.entry(id).or_default();
                run.query = query;
                run.pending = Some(seq);
                run.read = None;
                run.routine = routine;
                self.smart.in_flight = Some((id, seq, Instant::now()));
                return;
            }
        }
        if id == PREVIEW {
            self.smart.preview_sent = self.smart.preview.as_ref().map_or(0, |p| p.1);
        }
        if !spec.works_without_everything() {
            smart_line!(routine, %id, why, "smart fence: needs Everything, which is not running");
            self.smart_done(id, SmartSource::NeedsEverything, Vec::new(), 0);
            return;
        }
        self.smart.runs.entry(id).or_default().routine = routine;
        self.read_folder_later(id, spec, why);
    }

    /// The search of fence `id`, or the editor's search to count for `PREVIEW`.
    fn smart_spec_of(&self, id: FenceId) -> Option<SmartSpec> {
        if id == PREVIEW {
            return self.smart.preview.as_ref().map(|p| p.0.clone());
        }
        self.state.smart_spec(id).cloned()
    }

    /// Reads smart fence `id`'s folders on a worker (a big folder tree takes a while).
    fn read_folder_later(&mut self, id: FenceId, spec: SmartSpec, why: &str) {
        smart_line!(why == "minute", %id, why, folders = spec.folders.len(), "smart fence: reading the folders without Everything");
        // The editor's count names nothing: it shows only how many.
        let limit = if id == PREVIEW {
            0
        } else {
            spec.limit as usize
        };
        self.smart_work_later(id, SmartSource::Folder, move || {
            let (mut entries, scanned) = read_folders(&spec);
            let kept = entries.len();
            entries.sort_by_key(|e| std::cmp::Reverse(e.mtime));
            entries.truncate(limit);
            Found {
                entries,
                total: kept as u32,
                scanned,
                kept,
                missing: kept == 0 && folders_missing(&spec),
            }
        });
    }

    /// Works out smart fence `id`'s results on a worker: `find` (a folder read, or the check of
    /// Everything's answer: a file system call per file, a COM call per shortcut when a
    /// condition looks at targets), then the shell's display name of each result not shown
    /// already. Both cost too much for the UI thread with thousands of files or a OneDrive
    /// folder. The result comes back through `WM_APP_SMART_READ`; until then the fence's
    /// runs wait (`SmartState::reading`).
    fn smart_work_later(
        &mut self,
        id: FenceId,
        source: SmartSource,
        find: impl FnOnce() -> Found + Send + 'static,
    ) {
        self.smart.read_seq = self.smart.read_seq.wrapping_add(1);
        let seq = self.smart.read_seq;
        self.smart.runs.entry(id).or_default().read = Some(seq);
        self.smart.reading.insert(id, false);
        let known = self.state.smart_known_names(id);
        let tx = self.smart.read_tx.clone();
        let control = self.control.hwnd().0 as isize;
        let spawned = std::thread::Builder::new()
            .name("pecofence-smart-read".into())
            .spawn(move || {
                // Shortcut targets and display names come through COM.
                let _com = pecofence_platform::com::MtaGuard::init();
                let started = Instant::now();
                let Found {
                    entries,
                    total,
                    scanned,
                    kept,
                    missing,
                } = find();
                let ms = started.elapsed().as_millis() as u64;
                let named_at = Instant::now();
                let found: Vec<(DesktopEntry, String)> = entries
                    .into_iter()
                    .map(|e| {
                        let name = known
                            .get(&e)
                            .unwrap_or_else(|| crate::state::entry_display_name(&e));
                        (e, name)
                    })
                    .collect();
                let names_ms = named_at.elapsed().as_millis() as u64;
                let _ = tx.send(FolderRead {
                    id,
                    seq,
                    source,
                    found,
                    total,
                    scanned,
                    kept,
                    missing,
                    ms,
                    names_ms,
                });
                window::post_message(
                    HWND(control as *mut core::ffi::c_void),
                    crate::commands::WM_APP_SMART_READ,
                    0,
                    0,
                );
            });
        if let Err(e) = &spawned {
            tracing::warn!(%id, source = source.as_str(), error = %e, "smart fence: the worker did not start");
            if let Some(run) = self.smart.runs.get_mut(&id) {
                run.read = None;
            }
            self.smart.reading.remove(&id);
        }
    }

    /// `TIMER_SMART_READ`: the workers that finished (folder reads, checked Everything
    /// answers). A result a newer run replaced meanwhile is dropped.
    pub(super) fn drain_smart_reads(&mut self) {
        while let Ok(r) = self.smart.read_rx.try_recv() {
            let again = self.smart.reading.remove(&r.id).unwrap_or(false);
            let current = self.smart.runs.get(&r.id).and_then(|run| run.read) == Some(r.seq);
            if r.id == PREVIEW {
                if current && self.smart.preview.is_some() {
                    if let Some(run) = self.smart.runs.get_mut(&PREVIEW) {
                        run.read = None;
                    }
                    tracing::info!(
                        source = r.source.as_str(),
                        total = r.total,
                        got = r.scanned,
                        kept = r.kept,
                        ms = r.ms,
                        "smart fence editor: counted"
                    );
                    // Everything's own total, kept as its answer came in.
                    let total = match r.source {
                        SmartSource::Everything => self.smart.preview_total,
                        _ => r.total,
                    };
                    self.preview_done(r.source, total, r.scanned, r.kept);
                }
                if again {
                    self.run_smart(PREVIEW, "after the count in progress");
                }
                continue;
            }
            if current && self.state.smart_spec(r.id).is_some() {
                let run = self.smart.runs.entry(r.id).or_default();
                run.read = None;
                let routine = run.routine;
                let shown = r.found.len();
                let applied_at = Instant::now();
                let missing_changed = std::mem::replace(&mut run.missing, r.missing) != r.missing;
                self.smart_done(r.id, r.source, r.found, r.total);
                if missing_changed {
                    tracing::info!(id = %r.id, missing = r.missing, "smart fence: whether its folders are there changed");
                    self.apply_portal_deco(self.state.host_of(r.id));
                }
                let ui_ms = applied_at.elapsed().as_millis() as u64;
                if r.source == SmartSource::Folder {
                    smart_line!(routine, id = %r.id, seq = r.seq, scanned = r.scanned, total = r.total, shown, ms = r.ms, names_ms = r.names_ms, ui_ms, "smart fence: folder read without Everything");
                } else {
                    smart_line!(routine, id = %r.id, seq = r.seq, total = r.total, got = r.scanned, kept = r.kept, shown, ms = r.ms, names_ms = r.names_ms, ui_ms, "smart fence: Everything answered");
                }
            } else {
                tracing::debug!(id = %r.id, seq = r.seq, source = r.source.as_str(), "smart fence: an older result dropped");
            }
            if again {
                self.run_smart(r.id, "after the results in progress");
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
        let Some(spec) = self.smart_spec_of(id) else {
            self.ask_next_smart();
            return;
        };
        if let Some(run) = self.smart.runs.get_mut(&id) {
            run.pending = None;
        }
        // A late answer to a search sent before the fence's worker started (a folder read
        // while Everything was gone): the fence runs again after that worker instead.
        if let Some(again) = self.smart.reading.get_mut(&id) {
            *again = true;
            tracing::debug!(%id, seq, "smart fence: answer dropped, a worker is busy with the fence");
            self.ask_next_smart();
            return;
        }
        let Some((total, hits)) = ev::parse_list2(bytes) else {
            tracing::warn!(%id, len = bytes.len(), "smart fence: Everything's answer could not be read");
            self.ask_next_smart();
            return;
        };
        // The editor's count of a search Everything tests whole: Everything's total.
        if id == PREVIEW && !spec.app_checks() {
            tracing::info!(total, "smart fence editor: counted by Everything");
            self.preview_done(SmartSource::Everything, total, 0, 0);
            self.ask_next_smart();
            return;
        }
        if id == PREVIEW {
            self.smart.preview_total = total;
        }
        tracing::debug!(%id, seq, total, got = hits.len(), "smart fence: Everything's answer is checked on a worker");
        // Everything matched on its index; the files themselves say what they are now (gone,
        // hidden, system) and get the conditions Everything cannot test.
        let limit = if id == PREVIEW {
            0
        } else {
            spec.limit as usize
        };
        let app_checks = spec.app_checks();
        self.smart_work_later(id, SmartSource::Everything, move || {
            let now = pecofence_core::now_unix();
            let targets = spec.needs_shortcut_targets();
            let got = hits.len();
            let mut entries: Vec<DesktopEntry> = hits
                .iter()
                .filter_map(|h| shell::stat_entry(Path::new(&h.path), EntryOrigin::UserDesktop))
                .filter(|e| entry_matches(&spec, e, targets, now))
                .collect();
            let kept = entries.len();
            entries.truncate(limit);
            Found {
                entries,
                // What was left out here is left out of the total too. When the app tests a
                // condition itself, Everything's total says nothing about the files not checked:
                // the total is what matched among the newest checked.
                total: if app_checks {
                    kept as u32
                } else {
                    total.saturating_sub((got - kept) as u32)
                },
                scanned: got,
                kept,
                missing: kept == 0 && folders_missing(&spec),
            }
        });
        // Everything is free: the next waiting search goes.
        self.ask_next_smart();
    }

    /// Sends the next waiting fence's search once Everything has answered the one before.
    /// The Peek search's query to Everything got no answer in time (a smart search cancelled
    /// it, or Everything was busy) is sent again first.
    pub(super) fn ask_next_smart(&mut self) {
        if self
            .search
            .asked_at
            .is_some_and(|t| t.elapsed() >= ANSWER_WAIT)
        {
            tracing::info!("search: Everything did not answer the Peek search; asked again");
            self.search.asked_at = None;
            self.search.sent.clear();
            self.send_everything_query();
            if self.search.asked_at.is_some() {
                return;
            }
        }
        while self.smart.in_flight.is_none() && !self.peek_waits_for_everything() {
            let Some(next) = self.smart.waiting.pop_front() else {
                return;
            };
            if self.smart_spec_of(next).is_some() {
                self.run_smart(next, "waited for another search");
            }
        }
    }

    /// Whether the Peek search's query is out and its answer not in yet.
    fn peek_waits_for_everything(&self) -> bool {
        self.search
            .asked_at
            .is_some_and(|t| t.elapsed() < ANSWER_WAIT)
    }

    /// The Peek search is about to ask Everything: a smart fence's search out now would be
    /// cancelled, so that fence asks again first after the Peek's answer.
    pub(super) fn smart_yield_to_peek(&mut self) {
        if let Some((id, _, _)) = self.smart.in_flight.take() {
            self.smart.waiting.retain(|w| *w != id);
            self.smart.waiting.push_front(id);
            tracing::info!(%id, "smart fence: its search gives way to the Peek search");
        }
    }

    /// Puts a run's results into the fence and redraws it when they changed. The search a
    /// fence waits for (`SmartRun::pending`) is left alone: an answer clears it as it comes in.
    fn smart_done(
        &mut self,
        id: FenceId,
        source: SmartSource,
        entries: Vec<(DesktopEntry, String)>,
        total: u32,
    ) {
        if id == PREVIEW {
            self.smart.runs.entry(PREVIEW).or_default().source = source;
            self.preview_done(source, total, 0, 0);
            return;
        }
        let count = entries.len();
        let changed = self.state.set_smart_results(id, entries);
        let run = self.smart.runs.entry(id).or_default();
        let source_changed = run.source != source || run.total != total;
        run.source = source;
        run.total = total;
        if changed || source_changed {
            tracing::info!(%id, source = source.as_str(), count, total, "smart fence results changed");
        }
        if changed {
            // Results that replace most of what the fence shows (none any more, another
            // search) fade over the old page, as a filter does, instead of cutting to it.
            let host = self.state.host_of(id);
            if self.state.active_tab_of(host) == id
                && let Some(w) = self.fences.get(&host)
            {
                w.fade_next_refill();
            }
            self.refresh_fence(id);
        } else if source_changed {
            self.apply_portal_deco(self.state.host_of(id));
        }
        if source == SmartSource::NeedsEverything {
            // Soon when Everything is there already and only reads its index.
            self.poll_for_everything(self.smart.seen.is_some());
        }
    }

    /// Sets `TIMER_SMART_WAIT`: Everything's window is looked for again soon (`fast`: right
    /// after 「启动 Everything」), while a fence waits for it.
    fn poll_for_everything(&mut self, fast: bool) {
        if self.smart.wait_poll && !fast {
            return;
        }
        self.smart.wait_poll = true;
        let ms = if fast { STARTING_POLL_MS } else { WAIT_POLL_MS };
        window::set_timer(self.control.hwnd(), TIMER_SMART_WAIT, ms);
    }

    /// The fences that wait for Everything.
    fn smart_waiting_for_everything(&self) -> Vec<FenceId> {
        self.smart
            .runs
            .iter()
            .filter(|(_, r)| r.source == SmartSource::NeedsEverything)
            .map(|(id, _)| *id)
            .collect()
    }

    /// `TIMER_SMART_WAIT`: while a fence waits for Everything, an Everything that runs by now
    /// (and has read its index) gets the fences' searches; else the look is repeated.
    pub(super) fn look_for_everything(&mut self) {
        self.smart.wait_poll = false;
        let waiting = self.smart_waiting_for_everything();
        if waiting.is_empty() {
            self.smart.starting = None;
            self.smart.seen = None;
            return;
        }
        // Only the window is looked for while Everything is not there: asking it waits for
        // an Everything that is busy.
        let found = everything::running().then(everything::find).flatten();
        let loaded = found.is_some_and(|i| i.db_loaded());
        if loaded {
            self.smart.index_unknown = false;
        }
        let was_seen = self.smart.seen.is_some();
        let seen_for = match found {
            Some(_) => Some(self.smart.seen.get_or_insert_with(Instant::now).elapsed()),
            None => {
                self.smart.seen = None;
                None
            }
        };
        // Everything came or went: the waiting fences say 「正在搜索…」 while it is there.
        if was_seen != self.smart.seen.is_some() {
            for id in &waiting {
                self.apply_portal_deco(self.state.host_of(*id));
            }
        }
        // An Everything that never says its index is read (a version that does not answer the
        // question) is waited for once, `INDEX_WAIT` long, and searched at once from then on.
        let waited = seen_for.is_some_and(|d| d >= INDEX_WAIT || self.smart.index_unknown);
        if waited && !loaded {
            self.smart.index_unknown = true;
        }
        if loaded || waited {
            tracing::info!(
                fences = waiting.len(),
                started_here = self.smart.starting.is_some(),
                loaded,
                seen_ms = seen_for.map(|d| d.as_millis() as u64),
                "smart fence: Everything is running now"
            );
            self.smart.starting = None;
            self.smart.seen = None;
            self.run_waiting_smart();
            self.push_settings_state();
            return;
        }
        let starting = self
            .smart
            .starting
            .is_some_and(|t| t.elapsed() < STARTING_FOR);
        if self.smart.starting.is_some() && !starting {
            tracing::info!(
                found = found.is_some(),
                "smart fence: Everything did not come up; the fences offer to start it again"
            );
            self.smart.starting = None;
            for id in waiting {
                self.apply_portal_deco(self.state.host_of(id));
            }
        }
        // Soon again while it is on its way (just started, or there and reading its index).
        self.poll_for_everything(starting || found.is_some());
    }

    /// 「启动 Everything」 was picked: the fences that wait for it say 「正在搜索…」 at once (the
    /// row stayed as it was for the 2.5 s until the search ran, as if the click had done
    /// nothing), and search as soon as Everything is there.
    pub(super) fn smart_everything_starting(&mut self) {
        let waiting = self.smart_waiting_for_everything();
        if waiting.is_empty() {
            return;
        }
        tracing::debug!(
            fences = waiting.len(),
            "smart fence: Everything is being started; the waiting fences say so"
        );
        self.smart.starting = Some(Instant::now());
        for id in waiting {
            self.apply_portal_deco(self.state.host_of(id));
        }
        self.poll_for_everything(true);
    }

    /// Starts and stops the folder watchers of the smart fences: one per folder searched.
    pub(super) fn ensure_smart_watchers(&mut self) {
        let wanted: Vec<(FenceId, PathBuf, bool)> = self
            .state
            .fences()
            .iter()
            .filter_map(|f| f.smart.as_ref().map(|spec| (f.id, spec)))
            .flat_map(|(id, spec)| {
                spec.folders
                    .iter()
                    .map(move |dir| (id, PathBuf::from(dir), spec.subfolders))
            })
            .collect();
        self.smart.watchers.retain(|(id, dir), (tree, w)| {
            w.is_running()
                && wanted
                    .iter()
                    .any(|(wid, wdir, sub)| wid == id && wdir == dir && sub == tree)
        });
        for (id, dir, subfolders) in wanted {
            if self.smart.watchers.contains_key(&(id, dir.clone())) {
                continue;
            }
            let changed = self.smart.changed.clone();
            let control = self.control.hwnd().0 as isize;
            // Every event of this watcher lies in a folder of the fence: the fence is all it names.
            let on_events = move |events: Vec<FsEvent>| {
                if events.iter().any(|e| matches!(e, FsEvent::Overflow)) {
                    tracing::info!(%id, "smart fence watcher overflowed; its search runs again");
                }
                if let Ok(mut c) = changed.lock() {
                    c.insert(id);
                }
                window::post_message(
                    HWND(control as *mut core::ffi::c_void),
                    crate::commands::WM_APP_SMART_FS,
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
                    self.smart.watchers.insert((id, dir), (subfolders, w));
                }
                Err(e) => {
                    tracing::warn!(%id, dir = %dir.display(), error = %e, "smart fence watcher failed")
                }
            }
        }
    }

    /// The pointer came onto `fence`. A searched folder that was renamed or moved away keeps
    /// its watcher running (the handle follows the folder) and the watcher reports nothing of
    /// it, so a smart fence showed the files of a folder no longer at its path until the
    /// once-a-minute search (a double-click on one failed). Looked at now, as a folder fence
    /// is (`portal_pointer_in`): the search runs again at once. One file-system call per
    /// searched folder on a fixed drive.
    pub(super) fn smart_pointer_in(&mut self, fence: FenceId) {
        let moved = self.smart.watchers.iter().any(|((id, dir), (_, w))| {
            *id == fence && shell::is_on_fixed_drive(dir) && !w.still_at(dir)
        });
        if !moved {
            return;
        }
        tracing::info!(%fence, "smart fence: a searched folder is no longer at its path; noticed as the pointer came onto it");
        self.smart
            .watchers
            .retain(|(id, dir), (_, w)| *id != fence || w.still_at(dir));
        self.run_smart(fence, "a searched folder is no longer at its path");
    }

    /// A watcher batch named `dirs` (`all`: it overflowed): the smart fences with a folder that
    /// holds one of them run again shortly.
    pub(super) fn smart_folders_changed(&mut self, dirs: &[PathBuf], all: bool) {
        let lower = |p: &Path| p.to_string_lossy().to_lowercase();
        let dirs: Vec<String> = dirs.iter().map(|d| lower(d)).collect();
        let hit: Vec<FenceId> = self
            .state
            .fences()
            .iter()
            .filter_map(|f| {
                let spec = f.smart.as_ref()?;
                if spec.folders.is_empty() {
                    return None;
                }
                let inside = spec.folders.iter().any(|folder| {
                    let folder = folder.trim_end_matches('\\').to_lowercase();
                    dirs.iter().any(|d| {
                        *d == folder || (spec.subfolders && d.starts_with(&format!("{folder}\\")))
                    })
                });
                (all || inside).then_some(f.id)
            })
            .collect();
        self.run_smart_soon(hit, "a desktop or portal watcher");
    }

    /// `WM_APP_SMART_FS`: the fences whose own watcher saw a change run again shortly.
    pub(super) fn smart_watchers_fired(&mut self) {
        let ids: Vec<FenceId> = self
            .smart
            .changed
            .lock()
            .map(|mut c| c.drain().collect())
            .unwrap_or_default();
        let ids: Vec<FenceId> = ids
            .into_iter()
            .filter(|id| self.state.smart_spec(*id).is_some())
            .collect();
        self.run_smart_soon(ids, "its own watcher");
    }

    /// Fences `hit` run again after `SOON_MS` (`SOON_NO_INDEX_MS` without Everything), and once
    /// more `AGAIN_MS` later. A timer that is
    /// already set is left as it is, so a folder that keeps changing (a long download) still
    /// gets its runs.
    fn run_smart_soon(&mut self, hit: Vec<FenceId>, by: &str) {
        if hit.is_empty() {
            return;
        }
        let armed = !self.smart.soon.is_empty();
        self.smart.soon.extend(hit);
        self.smart.again = true;
        if armed {
            tracing::debug!(by, "smart fence folders changed; their run is already due");
            return;
        }
        let pause = if everything::find().is_some() {
            SOON_MS
        } else {
            SOON_NO_INDEX_MS
        };
        tracing::debug!(
            fences = self.smart.soon.len(),
            by,
            pause,
            "smart fence folders changed"
        );
        window::set_timer(self.control.hwnd(), TIMER_SMART_SOON, pause);
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
        let Some(spec) = self.state.fence_mut(fence).and_then(|f| f.smart.as_mut()) else {
            tracing::info!(%fence, "smart fence change refused: not a smart fence");
            return false;
        };
        let before = spec.clone();
        let what = change.apply(spec);
        if let Some(problem) = spec.problem() {
            tracing::info!(%fence, what, problem, "smart fence change refused");
            *spec = before;
            return false;
        }
        if *spec == before {
            return false;
        }
        // A folder picked by hand is the user's own: an import keeps it as it is.
        if spec.folders != before.folders {
            spec.template = None;
        }
        self.smart_search_changed(fence, what);
        self.smart_editor_fence_changed(fence);
        true
    }

    /// The smart fence editor's 好: the whole search at once. The template it came from stays
    /// only while the folders are the template's. False when nothing changed.
    pub(super) fn replace_smart_spec(&mut self, fence: FenceId, mut spec: SmartSpec) -> bool {
        let Some(cur) = self.state.fence_mut(fence).and_then(|f| f.smart.as_mut()) else {
            tracing::info!(%fence, "smart fence change refused: not a smart fence");
            return false;
        };
        spec.template = cur.template.clone().filter(|_| spec.folders == cur.folders);
        if *cur == spec {
            tracing::info!(%fence, "smart fence editor: the search is unchanged");
            return false;
        }
        *cur = spec;
        self.smart_search_changed(fence, "the whole search");
        true
    }

    /// After a change to smart fence `fence`'s search: logged, saved, the watchers follow its
    /// folders, the search runs again and the title shows its new state.
    fn smart_search_changed(&mut self, fence: FenceId, what: &str) {
        if let Some(spec) = self.state.smart_spec(fence) {
            tracing::info!(%fence, what, folders = spec.folders.len(), matching = spec.matching.as_str(), subfolders = spec.subfolders, within_days = ?spec.within_days, conditions = spec.all_of.len(), name = spec.name.is_some(), everything = spec.everything.is_some(), limit = spec.limit, "smart fence search changed");
        }
        self.state.mark_dirty();
        self.schedule_save();
        self.ensure_smart_watchers();
        self.run_smart(fence, "search changed");
        self.apply_portal_deco(self.state.host_of(fence));
    }

    /// A template's folder that is not on this computer (a configuration imported from another
    /// one, a moved Downloads folder): this computer's folder of the same kind takes its place.
    /// A template's search has one folder; a second one makes it the user's own.
    fn refind_template_folder(&mut self, id: FenceId) {
        let Some(spec) = self.state.smart_spec(id) else {
            return;
        };
        let (Some(key), [folder]) = (spec.template.clone(), spec.folders.as_slice()) else {
            return;
        };
        if Path::new(&folder).is_dir() {
            return;
        }
        let found = Template::parse(&key)
            .and_then(|t| t.spec(&known_folders()))
            .and_then(|s| s.folders.into_iter().next())
            .filter(|f| Path::new(f).is_dir());
        let Some(found) = found else {
            tracing::info!(%id, template = %key, "smart fence: the template's folder is missing here too");
            return;
        };
        tracing::info!(%id, template = %key, "smart fence: the template's folder found on this computer");
        if let Some(spec) = self.state.fence_mut(id).and_then(|f| f.smart.as_mut()) {
            spec.folders = vec![found];
        }
        self.state.mark_dirty();
        self.schedule_save();
        self.ensure_smart_watchers();
        self.smart_editor_fence_changed(id);
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
            // Until the answer is in the fence says 「正在搜索…」, no longer that it needs
            // Everything.
            if let Some(run) = self.smart.runs.get_mut(&id) {
                run.source = SmartSource::Waiting;
            }
            self.apply_portal_deco(self.state.host_of(id));
            self.run_smart(id, "everything started");
        }
    }

    /// 把筛选存为智能栅栏: a new smart fence showing what filtered fence `fence` shows now: the
    /// folder it shows (only that folder, as the folder fence does) or its own search (folder,
    /// conditions, time, Everything text and name), and the filter added to its name condition.
    pub(super) fn save_filter_as_smart(&mut self, fence: FenceId, x: i32, y: i32) {
        let Some(whole) = self.fence_filters.get(&fence).map(|t| t.trim().to_string()) else {
            return;
        };
        // A 类型 chip becomes the smart fence's 类型 (or extension) condition, the text its name.
        let (types, filter) = pecofence_core::type_filter::split(&whole);
        let type_conds: Vec<_> = types
            .iter()
            .map(pecofence_core::type_filter::cond)
            .collect();
        let name = |own: Option<&str>| -> Option<String> {
            match (
                own.map(str::trim).filter(|o| !o.is_empty()),
                filter.is_empty(),
            ) {
                (Some(own), true) => Some(own.to_string()),
                (Some(own), false) => Some(format!("{own} {filter}")),
                (None, true) => None,
                (None, false) => Some(filter.clone()),
            }
        };
        let spec = match (self.state.smart_spec(fence), self.state.portal_path(fence)) {
            // 任一 over two rows or more and the filter are not one search (the menu item is
            // off for such a fence).
            (Some(spec), _)
                if spec.matching == pecofence_core::smart::SmartMatch::Any && spec.rows() >= 2 =>
            {
                tracing::info!(%fence, rows = spec.rows(), "filter not saved as a smart fence: an any search of several rows");
                return;
            }
            // The fence's own name condition still holds: every word must match. With one row
            // or none, 任一 is 全部: the filter joins it as one more condition that must hold.
            (Some(spec), _) => {
                if spec.matching == pecofence_core::smart::SmartMatch::Any {
                    tracing::info!(%fence, rows = spec.rows(), "filter saved as a smart fence: any of one row or none saved as all");
                }
                SmartSpec {
                    template: None,
                    matching: pecofence_core::smart::SmartMatch::All,
                    name: name(spec.name.as_deref()),
                    all_of: spec.all_of.iter().cloned().chain(type_conds).collect(),
                    ..spec.clone()
                }
            }
            (None, Some(dir)) => SmartSpec {
                folders: vec![dir.to_string_lossy().into_owned()],
                subfolders: false,
                name: name(None),
                all_of: type_conds,
                ..SmartSpec::default()
            },
            (None, None) => return,
        };
        let base = self
            .state
            .fence(fence)
            .map(|f| self.state.display_title(f))
            .unwrap_or_default();
        let shown = types
            .iter()
            .map(pecofence_core::type_filter::label)
            .chain((!filter.is_empty()).then(|| filter.clone()))
            .collect::<Vec<_>>()
            .join(" ");
        let title = pecofence_core::i18n::format("{0}：{1}", &[base, shown]);
        let folders = spec.folders.len();
        let typed = spec.everything.is_some();
        let conditions = spec.all_of.len();
        if let Some(id) = self.create_smart_fence(spec, Some(&title), None, x, y) {
            tracing::info!(from = %fence, %id, folders, typed, conditions, "filter saved as a smart fence");
        }
    }

    /// The Peek search's 「把这次搜索放到桌面」: a smart fence with the same Everything search
    /// (pinyin included when Everything has it) over every drive, titled after the text, of
    /// the type tab's type when one is picked.
    pub(super) fn search_to_desktop(
        &mut self,
        query: &str,
        tab: Option<pecofence_core::rules::TypeCategory>,
    ) {
        let Some(instance) = everything::find() else {
            tracing::info!("search to desktop: Everything is no longer running");
            return;
        };
        // The type tab comes along as the smart fence's 类型 condition.
        let spec = SmartSpec {
            everything: Some(ev::search_text(query, instance.supports_pinyin())),
            all_of: tab
                .map(|c| {
                    pecofence_core::type_filter::cond(&pecofence_core::type_filter::TypeToken::Cat(
                        c,
                    ))
                })
                .into_iter()
                .collect(),
            ..SmartSpec::default()
        };
        tracing::info!(tab = ?tab, "search to desktop: the type tab as a condition");
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

    /// What the infotip of smart fence `f`'s count says when the count ends with a "+": how
    /// many files the fence shows (the newest, up to its limit) and how many the search found.
    /// None when the fence shows all it found, is filtered, or is no smart fence.
    pub(super) fn smart_count_tip(&self, f: &pecofence_core::Fence) -> Option<String> {
        let spec = f.smart.as_ref()?;
        let run = self.smart.runs.get(&f.id)?;
        let found =
            matches!(run.source, SmartSource::Everything | SmartSource::Folder) && !run.missing;
        (found && run.total > spec.limit && self.fence_filter(f.id).is_none()).then(|| {
            pecofence_core::i18n::format(
                "显示最新的 {0} 个，共找到 {1} 个",
                &[self.item_views(f).len().to_string(), run.total.to_string()],
            )
        })
    }

    /// Smart fence `id`'s empty-state text, the label of the row under it that starts or
    /// installs Everything (only while the fence waits for Everything) and the count after
    /// its name: the files it shows (after a filter), with a "+" when the limit cut the
    /// results. None for other fences.
    pub(super) fn smart_view(
        &self,
        f: &pecofence_core::Fence,
    ) -> (Option<&'static str>, Option<&'static str>, Option<String>) {
        let Some(spec) = f.smart.as_ref() else {
            return (None, None, None);
        };
        let run = self.smart.runs.get(&f.id);
        let source = match run.map_or(SmartSource::Waiting, |r| r.source) {
            // 「启动 Everything」 was just picked, or Everything is there and reads its index: it
            // is on its way.
            SmartSource::NeedsEverything
                if self.smart.starting.is_some() || self.smart.seen.is_some() =>
            {
                SmartSource::Waiting
            }
            source => source,
        };
        let empty = Some(match source {
            SmartSource::Waiting => pecofence_core::i18n::text("正在搜索…"),
            // A folder search waits for Everything only for its Everything syntax.
            SmartSource::NeedsEverything if !spec.folders.is_empty() => pecofence_core::i18n::text(
                "这个栅栏用了 Everything 语法，需要先打开 Everything：右键这个栅栏可以启动或安装它。",
            ),
            SmartSource::NeedsEverything => pecofence_core::i18n::text(
                "要搜索整台电脑，需要先打开 Everything：右键这个栅栏可以启动或安装它。",
            ),
            // What a folder fence says when its folder is gone.
            SmartSource::Everything | SmartSource::Folder if run.is_some_and(|r| r.missing) => {
                pecofence_core::i18n::text("找不到此文件夹。")
            }
            SmartSource::Everything | SmartSource::Folder => {
                pecofence_core::i18n::text("没有符合条件的文件。")
            }
        });
        let found = matches!(source, SmartSource::Everything | SmartSource::Folder)
            && !run.is_some_and(|r| r.missing);
        let count = found.then(|| {
            let shown = self.item_views(f).len();
            let cut =
                run.is_some_and(|r| r.total > spec.limit) && self.fence_filter(f.id).is_none();
            format!("{shown}{}", if cut { "+" } else { "" })
        });
        // While winget installs Everything the row is away: a second click starts nothing.
        let action = (source == SmartSource::NeedsEverything && !self.everything_installing)
            .then(everything_action_label);
        (empty, action, count)
    }

    /// 「启动 Everything」 / 「安装 Everything」 from a smart fence that waits for it (`by`: its
    /// menu or the row in its empty state): what Settings' Everything card does.
    pub(super) fn start_or_install_everything(&mut self, by: &'static str) {
        let installed = pecofence_platform::everything::installed_exe().is_some();
        tracing::info!(by, installed, "Everything asked for from a smart fence");
        if installed {
            self.start_everything();
        } else {
            self.install_everything();
        }
    }

    /// The row under a smart fence's empty-state text was clicked.
    pub(super) fn smart_everything_clicked(&mut self, fence: FenceId) {
        if !self.smart_needs_everything(fence) {
            tracing::debug!(%fence, "empty-state row clicked: the fence no longer waits for Everything");
            return;
        }
        self.start_or_install_everything("fence row");
    }

    /// 智能栅栏 for `fence get` / `fence list`: the search and its last run.
    pub(super) fn smart_dto(&self, id: FenceId) -> Option<serde_json::Value> {
        let spec = self.state.smart_spec(id)?;
        let run = self.smart.runs.get(&id);
        Some(serde_json::json!({
            "spec": spec,
            // Said in full here: the saved search leaves out `match` when it is `all`.
            "match": spec.matching.as_str(),
            "folders": spec.folders,
            "source": run.map_or("waiting", |r| r.source.as_str()),
            "total": run.map_or(0, |r| r.total),
            "shown": self.state.fence(id).map_or(0, |f| self.state.items_of(f).len()),
            "query": run.map(|r| r.query.clone()).filter(|q| !q.is_empty()),
        }))
    }
}

/// What a smart fence that waits for Everything offers, in its menu and in its empty state.
pub(super) fn everything_action_label() -> &'static str {
    if pecofence_platform::everything::installed_exe().is_some() {
        pecofence_core::i18n::text("启动 Everything")
    } else {
        pecofence_core::i18n::text("安装 Everything")
    }
}

impl App {
    /// `editorCount {seq, spec}`: counts the editor's search (the page asks again after each
    /// change, a moment after the last one). A newer request replaces one that waits.
    pub(super) fn count_editor_search(&mut self, spec: SmartSpec, seq: u64) {
        tracing::debug!(
            seq,
            folders = spec.folders.len(),
            matching = spec.matching.as_str(),
            conditions = spec.all_of.len(),
            "smart fence editor: count asked"
        );
        self.smart.preview = Some((spec, seq));
        self.run_smart(PREVIEW, "editor");
    }

    /// The editor closed: its count stops (an answer still on its way is dropped).
    pub(super) fn stop_editor_count(&mut self) {
        self.smart.preview = None;
        self.smart.runs.remove(&PREVIEW);
        self.smart.waiting.retain(|id| *id != PREVIEW);
    }

    /// A count for the editor, of the same files the fence shows: Everything's total, or when
    /// the app tests a condition itself what matched among the newest files Everything gave
    /// (`got`; the fence checks the same ones), or what the folder read found (it stops after
    /// `SmartSpec::MAX_SCANNED` entries). The page says when not every file was checked.
    fn preview_done(&mut self, source: SmartSource, total: u32, got: usize, kept: usize) {
        let Some(spec) = self.smart.preview.as_ref().map(|p| &p.0) else {
            return;
        };
        let (n, newest, read) = match source {
            SmartSource::Everything if spec.app_checks() => {
                (kept as u64, ((got as u32) < total).then_some(got), None)
            }
            SmartSource::Folder => (
                u64::from(total),
                None,
                (got > SmartSpec::MAX_SCANNED).then_some(SmartSpec::MAX_SCANNED),
            ),
            _ => (u64::from(total), None, None),
        };
        let seq = self.smart.preview_sent;
        tracing::info!(
            seq,
            source = source.as_str(),
            n,
            newest,
            read,
            "smart fence editor: count shown"
        );
        let json = serde_json::json!({
            "type": "editorCount",
            "seq": seq,
            "source": source.as_str(),
            "n": n,
            // Only the newest this many files were checked, or this many entries read.
            "checkedNewest": newest,
            "readOnly": read,
            "everything": self.everything_json(),
        })
        .to_string();
        self.post_to_smart_editor(&json);
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
            .map(pecofence_core::rules::target_ext);
        f
    } else {
        crate::state::AppState::plain_facts(e, None, now)
    };
    spec.matches(&facts, e.mtime.max(e.created), now)
}

/// Whether the search names folders and none of them is there (on a worker: a folder on a
/// network drive that does not answer takes its time).
fn folders_missing(spec: &SmartSpec) -> bool {
    !spec.folders.is_empty() && !spec.folders.iter().any(|f| Path::new(f).is_dir())
}

/// Reads the search's folders (and their subfolders when the search takes them) without
/// Everything, at most `SmartSpec::MAX_SCANNED` entries in all, and keeps what matches.
/// Hidden and system entries, and folder links (which may loop), are left out; a file in two
/// of the folders (one inside the other) counts once. Returns the matches and how many entries
/// were read.
fn read_folders(spec: &SmartSpec) -> (Vec<DesktopEntry>, usize) {
    let now = pecofence_core::now_unix();
    let targets = spec.needs_shortcut_targets();
    let mut found = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut scanned = 0usize;
    let mut stack: Vec<PathBuf> = spec.folders.iter().rev().map(PathBuf::from).collect();
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
            if entry_matches(spec, &e, targets, now)
                && seen.insert(e.path.to_string_lossy().to_lowercase())
            {
                found.push(e);
            }
        }
    }
    (found, scanned)
}
