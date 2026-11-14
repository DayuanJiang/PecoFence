//! Config persistence: atomic writes, `.bak` rotation, daily backups, load fallbacks (plan §7.5).

use crate::geometry::MAX_DIP;
use crate::model::{Config, ItemSourceSpec, NormGeometry, SCHEMA_VERSION};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// Fences per layout. Enforced where fences are created (`AppState::new_fence`, the IPC create
/// paths) as well as here, so a save can never be refused for it.
pub const MAX_FENCES: usize = 64;
pub const DAILY_BACKUPS_KEPT: usize = 7;
/// Unreadable primaries moved aside by [`ConfigStore::load_reporting`]; the oldest go first.
pub const UNREADABLE_KEPT: usize = 5;
const UNREADABLE_PREFIX: &str = "config.unreadable-";
/// How long a start waits for config files another program holds open (antivirus, backup and
/// sync tools) before it goes on without them.
pub const LOCK_WAIT: std::time::Duration = std::time::Duration::from_secs(10);
/// Pause between two tries to open a file another program holds open.
const LOCK_RETRY: std::time::Duration = std::time::Duration::from_millis(250);

/// ERROR_SHARING_VIOLATION or ERROR_LOCK_VIOLATION: another program holds the file open
/// without sharing it. It is not damaged, only unavailable for now.
fn held_open(e: &io::Error) -> bool {
    matches!(e.raw_os_error(), Some(32 | 33))
}

#[derive(Debug)]
pub enum LoadOutcome {
    /// Loaded from the primary file.
    Primary(Config),
    /// Primary missing/corrupt; loaded from `.bak` or a daily backup (path given).
    Recovered(Config, PathBuf),
    /// Nothing usable: fresh default (first run or total loss).
    Fresh(Config),
}

impl LoadOutcome {
    pub fn into_config(self) -> Config {
        match self {
            LoadOutcome::Primary(c) | LoadOutcome::Recovered(c, _) | LoadOutcome::Fresh(c) => c,
        }
    }
}

/// ERROR_APPX_FILE_NOT_ENCRYPTED: Windows refuses the Store edition a file of its own data
/// that is not encrypted. With the package on a drive other than C:, its data lives on that
/// drive and Windows expects every file there to be encrypted.
pub const APPX_FILE_NOT_ENCRYPTED: i32 = 409;

/// Why a config file could not be used.
#[derive(Debug)]
pub enum ReadError {
    /// The file could not be read.
    Io(io::Error),
    /// It was read but is no configuration this version accepts.
    Invalid(String),
}

impl ReadError {
    /// Windows refused to read the file because it is not encrypted ([`APPX_FILE_NOT_ENCRYPTED`]).
    pub fn not_encrypted(&self) -> bool {
        matches!(self, ReadError::Io(e) if e.raw_os_error() == Some(APPX_FILE_NOT_ENCRYPTED))
    }

    /// Another program holds the file open without sharing it (see [`held_open`]).
    pub fn locked(&self) -> bool {
        matches!(self, ReadError::Io(e) if held_open(e))
    }
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::Io(e) => e.fmt(f),
            ReadError::Invalid(reason) => f.write_str(reason),
        }
    }
}

/// What [`ConfigStore::load_reporting`] met on its way to the outcome.
#[derive(Debug, Default)]
pub struct LoadReport {
    /// Where an unreadable primary was moved.
    pub moved_to: Option<PathBuf>,
    /// The files that exist but could not be used, in the order tried: which one
    /// (`config.json`, `config.bak` or `backup`), its path and why.
    pub failures: Vec<(&'static str, PathBuf, ReadError)>,
    /// The first file in line that another program still held open after the wait: the
    /// outcome comes from a file further down the line (or is the defaults), so nothing may be
    /// saved over the files until this one can be read again ([`ConfigStore::is_locked`]).
    pub locked: Option<PathBuf>,
    /// How long the load waited for files held open by another program.
    pub waited: std::time::Duration,
}

/// A failed [`ConfigStore::save`]: the step that failed (a fixed name, safe to log) and why.
#[derive(Debug)]
pub struct SaveError {
    pub step: &'static str,
    pub error: io::Error,
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.step, self.error)
    }
}

/// [`SaveError::step`] of the day's backup copy, written after config.json.
const DAILY_BACKUP_STEP: &str = "daily backup";

impl SaveError {
    /// config.json itself is written and holds every change: only the day's backup copy
    /// could not be written. To log, not to tell the user that changes will be lost.
    pub fn config_written(&self) -> bool {
        self.step == DAILY_BACKUP_STEP
    }
}

impl std::error::Error for SaveError {}

#[derive(Clone)]
pub struct ConfigStore {
    dir: PathBuf,
}

impl ConfigStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// Reuse an existing installation's data in place. No copying, rewriting or
    /// renaming of user files is needed just because the product name changed.
    pub fn with_legacy(preferred: impl Into<PathBuf>, legacy: impl Into<PathBuf>) -> Self {
        let preferred = Self::new(preferred);
        let legacy = Self::new(legacy);
        if !preferred.has_saved_data() && legacy.has_saved_data() {
            legacy
        } else {
            preferred
        }
    }

    fn has_saved_data(&self) -> bool {
        self.primary_path().exists() || self.bak_path().exists() || !self.list_backups().is_empty()
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
    pub fn primary_path(&self) -> PathBuf {
        self.dir.join("config.json")
    }
    fn bak_path(&self) -> PathBuf {
        self.dir.join("config.bak")
    }
    pub fn tmp_path(&self) -> PathBuf {
        self.dir.join("config.json.tmp")
    }
    fn backups_dir(&self) -> PathBuf {
        self.dir.join("backups")
    }

    /// Reads and validates any config file (import / backup restore), with the reason on error.
    pub fn parse_file(path: &Path) -> Result<Config, ReadError> {
        let text = fs::read_to_string(path).map_err(ReadError::Io)?;
        // Windows PowerShell 5.1 `Set-Content -Encoding utf8` and Notepad can add a BOM.
        let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
        let cfg: Config =
            serde_json::from_str(text).map_err(|e| ReadError::Invalid(e.to_string()))?;
        validate(&cfg).map_err(ReadError::Invalid)?;
        Ok(migrate(cfg))
    }

    /// Writes the config as pretty JSON to an arbitrary path (export).
    pub fn export_to(cfg: &Config, path: &Path) -> io::Result<()> {
        // The search history lists the user's file paths: it stays on this computer.
        if !cfg.search_history.is_empty() {
            let mut shared = cfg.clone();
            shared.search_history.clear();
            return fs::write(path, Self::to_json(&shared)?);
        }
        fs::write(path, Self::to_json(cfg)?)
    }

    /// The file text: pretty JSON with the `$schema` field pointing at the published schema
    /// (serde_json sorts keys, and `$` sorts first, so it is the first line after the brace).
    fn to_json(cfg: &Config) -> io::Result<String> {
        if cfg.schema.as_deref() == Some(crate::model::CONFIG_SCHEMA_URL) {
            // A loaded config already carries it: no copy of the whole config per save.
            return serde_json::to_string_pretty(cfg).map_err(io::Error::other);
        }
        let mut cfg = cfg.clone();
        cfg.schema = Some(crate::model::CONFIG_SCHEMA_URL.to_string());
        serde_json::to_string_pretty(&cfg).map_err(io::Error::other)
    }

    /// Loads with fallbacks: primary → .bak → newest daily backup → default.
    pub fn load(&self) -> LoadOutcome {
        self.load_reporting().0
    }

    /// [`Self::load`], plus where an unreadable primary was moved and which files could not be
    /// used. A `config.json` that exists but does not parse or validate (written by a newer
    /// version, or hand-edited) is renamed to `config.unreadable-<date>-<secs>.json` first: the
    /// `.bak` rotation in [`Self::save`] would otherwise turn it into `config.bak` and delete it
    /// on the next save. Files another program holds open are waited for up to [`LOCK_WAIT`].
    pub fn load_reporting(&self) -> (LoadOutcome, LoadReport) {
        self.load_waiting(LOCK_WAIT)
    }

    /// [`Self::load_reporting`] with its own wait for files another program holds open. A file
    /// still held open after it is skipped like a missing one, but never moved aside, and
    /// `LoadReport::locked` names it: it is not damaged, the caller must not save over it.
    pub fn load_waiting(&self, wait: std::time::Duration) -> (LoadOutcome, LoadReport) {
        let deadline = std::time::Instant::now() + wait;
        let mut report = LoadReport::default();
        let primary = self.primary_path();
        let mut candidates = vec![("config.bak", self.bak_path())];
        let mut backups = self.list_backups();
        backups.sort();
        candidates.extend(backups.into_iter().rev().map(|p| ("backup", p)));
        match self.read_when_free(&primary, deadline, &mut report) {
            Ok(c) => return (LoadOutcome::Primary(c), report),
            Err(ReadError::Io(e)) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => {
                if !e.locked() {
                    report.moved_to = self.quarantine(&primary);
                }
                report.failures.push(("config.json", primary, e));
            }
        }
        let mut outcome = LoadOutcome::Fresh(Config::default());
        for (name, path) in candidates {
            match self.read_when_free(&path, deadline, &mut report) {
                Ok(c) => {
                    outcome = LoadOutcome::Recovered(c, path);
                    break;
                }
                Err(ReadError::Io(e)) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => report.failures.push((name, path, e)),
            }
        }
        report.locked = report
            .failures
            .iter()
            .find(|(_, _, e)| e.locked())
            .map(|(_, path, _)| path.clone());
        (outcome, report)
    }

    /// [`Self::parse_file`], trying again until `deadline` while another program holds the
    /// file open; the time spent waiting adds to `report.waited`.
    fn read_when_free(
        &self,
        path: &Path,
        deadline: std::time::Instant,
        report: &mut LoadReport,
    ) -> Result<Config, ReadError> {
        loop {
            match Self::parse_file(path) {
                Err(e) if e.locked() && std::time::Instant::now() < deadline => {
                    std::thread::sleep(LOCK_RETRY);
                    report.waited += LOCK_RETRY;
                }
                other => return other,
            }
        }
    }

    /// Whether another program still holds `path` open without sharing it.
    pub fn is_locked(path: &Path) -> bool {
        fs::File::open(path).is_err_and(|e| held_open(&e))
    }

    fn quarantine(&self, primary: &Path) -> Option<PathBuf> {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let to = self.dir.join(format!(
            "{UNREADABLE_PREFIX}{}-{secs}.json",
            today_yyyy_mm_dd()
        ));
        // A copy still protects it when the rename is refused (the save then rotates the
        // original into .bak as before, but this copy is never touched).
        if fs::rename(primary, &to).is_err() && fs::copy(primary, &to).is_err() {
            return None;
        }
        let mut kept: Vec<PathBuf> = fs::read_dir(&self.dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .is_some_and(|n| n.starts_with(UNREADABLE_PREFIX))
                    })
                    .collect()
            })
            .unwrap_or_default();
        kept.sort();
        while kept.len() > UNREADABLE_KEPT {
            let _ = fs::remove_file(kept.remove(0));
        }
        Some(to)
    }

    /// Copies `config.json` into the backups as `backups/<today>-<tag>.json` (once a day per
    /// tag): a file a newer version wrote, before this version's first save replaces what that
    /// version keeps and this one does not know (see [`newer_version`]).
    pub fn keep_copy(&self, tag: &str) -> io::Result<PathBuf> {
        let dir = self.backups_dir();
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}-{tag}.json", today_yyyy_mm_dd()));
        if !path.exists() {
            fs::copy(self.primary_path(), &path)?;
        }
        Ok(path)
    }

    /// Daily backups (`backups/YYYY-MM-DD.json`), unsorted.
    pub fn list_backups(&self) -> Vec<PathBuf> {
        fs::read_dir(self.backups_dir())
            .map(|rd| {
                rd.filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| p.extension().is_some_and(|e| e == "json"))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Atomically writes the config: tmp → fsync → rotate old to .bak → rename tmp over primary.
    pub fn save(&self, cfg: &Config) -> Result<(), SaveError> {
        let at = |step: &'static str| move |error: io::Error| SaveError { step, error };
        validate(cfg)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            .map_err(at("validate"))?;
        fs::create_dir_all(&self.dir).map_err(at("create folder"))?;
        let json = Self::to_json(cfg).map_err(at("serialize"))?;
        {
            let mut f = fs::File::create(self.tmp_path()).map_err(at("create temp file"))?;
            io::Write::write_all(&mut f, json.as_bytes()).map_err(at("write temp file"))?;
            f.sync_all().map_err(at("flush temp file"))?;
        }
        let primary = self.primary_path();
        let mut rotated = false;
        if primary.exists() {
            // Best effort: keep the previous good file as .bak.
            let _ = fs::remove_file(self.bak_path());
            rotated = fs::rename(&primary, self.bak_path()).is_ok();
        }
        match fs::rename(self.tmp_path(), &primary) {
            Ok(()) => {}
            // The Store edition sees %APPDATA% merged with its package's copy, and Windows
            // refuses renames in it when %APPDATA% was moved to another drive (the .bak
            // rotation above fails too). Write the file in place, keeping the old one as .bak.
            // Without that copy a torn write would leave nothing readable, so it must succeed;
            // the new file is flushed like the temp file before the temp file goes.
            Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
                if primary.exists() {
                    fs::copy(&primary, self.bak_path()).map_err(at("keep config.bak"))?;
                }
                let mut f = fs::File::create(&primary).map_err(at("write config.json"))?;
                io::Write::write_all(&mut f, json.as_bytes()).map_err(at("write config.json"))?;
                f.sync_all().map_err(at("flush config.json"))?;
                let _ = fs::remove_file(self.tmp_path());
            }
            Err(e) => {
                // Put the previous file back: a load never looks at the temp file.
                if rotated {
                    let _ = fs::rename(self.bak_path(), &primary);
                }
                return Err(at("replace config.json")(e));
            }
        }
        self.daily_backup(&json).map_err(at(DAILY_BACKUP_STEP))?;
        Ok(())
    }

    /// Writes at most one backup per day and prunes to `DAILY_BACKUPS_KEPT`.
    fn daily_backup(&self, json: &str) -> io::Result<()> {
        let dir = self.backups_dir();
        fs::create_dir_all(&dir)?;
        let today = today_yyyy_mm_dd();
        let path = dir.join(format!("{today}.json"));
        if !path.exists() {
            fs::write(&path, json)?;
        }
        let mut backups = self.list_backups();
        backups.sort();
        while backups.len() > DAILY_BACKUPS_KEPT {
            let oldest = backups.remove(0);
            let _ = fs::remove_file(oldest);
        }
        Ok(())
    }
}

/// Saves configurations on a thread of its own. [`ConfigStore::save`] waits until the disk
/// has the file, and a disk busy with a large file copy answers only when the copy is over (40
/// seconds for 6 GB on a test PC): the thread that owns the windows hands the configuration
/// over here and goes on. Once a saver exists for a folder, only its thread writes there.
pub struct ConfigSaver {
    shared: Arc<SaverShared>,
}

/// What a [`ConfigSaver`] calls on its thread after every write: how long the write took and
/// how it ended. [`ConfigSaver::take_failed`] already knows the outcome; [`ConfigSaver::wait`]
/// returns once this has run.
pub type SaveDone = dyn Fn(Duration, &Result<(), SaveError>) + Send + Sync;

struct SaverShared {
    store: ConfigStore,
    done: Box<SaveDone>,
    state: Mutex<SaverState>,
    /// A configuration was handed over, a write ended, or the saver was dropped.
    changed: Condvar,
}

#[derive(Default)]
struct SaverState {
    /// The newest configuration not written yet. One handed over while it waits takes its
    /// place: the older one is never written.
    next: Option<Config>,
    writing: bool,
    /// The newest write failed, and [`ConfigSaver::take_failed`] has not been asked since.
    failed: bool,
    thread_started: bool,
    dropped: bool,
}

impl ConfigSaver {
    /// The thread starts with the first save.
    pub fn new(store: ConfigStore, done: Box<SaveDone>) -> Self {
        let shared = SaverShared {
            store,
            done,
            state: Mutex::default(),
            changed: Condvar::new(),
        };
        Self {
            shared: Arc::new(shared),
        }
    }

    /// Hands `cfg` over to be written and returns at once.
    pub fn save(&self, cfg: Config) {
        let mut state = self.shared.lock();
        state.next = Some(cfg);
        if !state.thread_started {
            let shared = self.shared.clone();
            let spawned = std::thread::Builder::new()
                .name("pecofence-save".into())
                .spawn(move || shared.run());
            if spawned.is_err() {
                // No thread to be had: written here, as before there was a saver.
                let cfg = state.next.take();
                state.writing = true;
                drop(state);
                self.shared.write(&cfg.expect("handed over above"));
                return;
            }
            state.thread_started = true;
        }
        drop(state);
        self.shared.changed.notify_all();
    }

    /// Waits until every configuration handed over is written, or its write has failed.
    pub fn wait(&self) {
        self.shared.wait_idle();
    }

    /// A handle that waits like [`Self::wait`], for a caller that cannot reach the saver
    /// when it has to wait (the saver's owner is borrowed elsewhere).
    pub fn waiter(&self) -> SaveWaiter {
        SaveWaiter {
            shared: self.shared.clone(),
        }
    }

    /// Whether the newest finished write failed. Asked once per failure: the next call says
    /// false until another write fails.
    pub fn take_failed(&self) -> bool {
        std::mem::take(&mut self.shared.lock().failed)
    }
}

/// See [`ConfigSaver::waiter`].
pub struct SaveWaiter {
    shared: Arc<SaverShared>,
}

impl SaveWaiter {
    /// [`ConfigSaver::wait`].
    pub fn wait(&self) {
        self.shared.wait_idle();
    }
}

impl Drop for ConfigSaver {
    /// The thread writes what still waits, then ends.
    fn drop(&mut self) {
        self.shared.lock().dropped = true;
        self.shared.changed.notify_all();
    }
}

impl SaverShared {
    // A panic never happens with the lock held (it guards a few assignments), so a poisoned
    // lock still holds a consistent state.
    fn lock(&self) -> MutexGuard<'_, SaverState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn wait<'a>(&self, state: MutexGuard<'a, SaverState>) -> MutexGuard<'a, SaverState> {
        self.changed
            .wait(state)
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn wait_idle(&self) {
        let mut state = self.lock();
        while state.next.is_some() || state.writing {
            state = self.wait(state);
        }
    }

    fn run(&self) {
        loop {
            let mut state = self.lock();
            let cfg = loop {
                if let Some(cfg) = state.next.take() {
                    state.writing = true;
                    break cfg;
                }
                if state.dropped {
                    return;
                }
                state = self.wait(state);
            };
            drop(state);
            self.write(&cfg);
        }
    }

    /// Writes `cfg` (the caller has set `writing`) and records how it ended.
    fn write(&self, cfg: &Config) {
        let started = Instant::now();
        let result = self.store.save(cfg);
        let took = started.elapsed();
        // Before `done`: it may tell the owner's thread, which then asks `take_failed`. A
        // write that only missed the day's backup copy saved every change: not failed.
        self.lock().failed = result.as_ref().is_err_and(|e| !e.config_written());
        (self.done)(took, &result);
        self.lock().writing = false;
        self.changed.notify_all();
    }
}

/// `saved_by` names a version newer than `ours` (`0.4.0` against `0.3.2`; a suffix such as
/// `-beta` is ignored). False for anything that does not read as a version.
pub fn newer_version(saved_by: &str, ours: &str) -> bool {
    fn parts(v: &str) -> Option<(u32, u32, u32)> {
        let core = v.trim().split(['-', '+']).next()?;
        let mut it = core.split('.').map(|p| p.parse::<u32>().ok());
        Some((it.next()??, it.next()??, it.next().unwrap_or(Some(0))?))
    }
    matches!((parts(saved_by), parts(ours)), (Some(a), Some(b)) if a > b)
}

/// Range checks from plan §7.5.
pub fn validate(cfg: &Config) -> Result<(), String> {
    if cfg.schema_version == 0 || cfg.schema_version > SCHEMA_VERSION {
        return Err(format!("unsupported schemaVersion {}", cfg.schema_version));
    }
    for layout in &cfg.layouts {
        if layout.fences.len() > MAX_FENCES {
            return Err(format!("too many fences: {}", layout.fences.len()));
        }
        let inboxes = layout
            .fences
            .iter()
            .filter(|f| f.kind == crate::model::FenceKind::Inbox)
            .count();
        if inboxes > 1 {
            return Err(format!("layout has {inboxes} inbox fences"));
        }
        for f in &layout.fences {
            let g = &f.geometry;
            if !(g.w.is_finite() && g.h.is_finite() && g.x.is_finite() && g.y.is_finite()) {
                return Err(format!("fence {} has non-finite geometry", f.title));
            }
            // The presets are 0.55 ("更透明"), 1.6 ("更厚实" = a veil) and 0 ("全透明").
            if let Some(a) = &f.appearance
                && let Some(o) = a.opacity
                && o != 0.0
                && !(0.2..=2.0).contains(&o)
            {
                return Err(format!("fence {} opacity {o} out of range", f.title));
            }
        }
    }
    Ok(())
}

/// `g`'s size clamped to 1 to 8192 DIP each way.
fn size_in_range(g: &NormGeometry) -> (f32, f32) {
    (g.w.clamp(1.0, MAX_DIP), g.h.clamp(1.0, MAX_DIP))
}

/// Brings fence sizes outside 1 to 8192 DIP (a hand-edited file) into that range, for the app
/// to load or import the file with just those sizes changed. True when one changed.
pub fn clamp_sizes(cfg: &mut Config) -> bool {
    let mut changed = false;
    for f in cfg.layouts.iter_mut().flat_map(|l| &mut l.fences) {
        let (w, h) = size_in_range(&f.geometry);
        if (w, h) != (f.geometry.w, f.geometry.h) {
            (f.geometry.w, f.geometry.h) = (w, h);
            changed = true;
        }
    }
    changed
}

/// 0.2.1 saved a portal on a network share or a mapped drive as the relative path
/// `UNC\server\share\…` (the canonical path with only its long-path prefix stripped), which
/// can neither be listed nor watched: such paths, in the layouts and the snapshots, become
/// `\\server\share\…`. True when one changed.
pub fn repair_unc_portals(cfg: &mut Config) -> bool {
    let mut changed = false;
    let layouts = cfg
        .layouts
        .iter_mut()
        .chain(cfg.snapshots.iter_mut().flat_map(|s| &mut s.layouts));
    for f in layouts.flat_map(|l| &mut l.fences) {
        if let ItemSourceSpec::Folder { path, .. } = &mut f.source
            && let Some(rest) = path.strip_prefix(r"UNC\")
        {
            *path = format!(r"\\{rest}");
            changed = true;
        }
    }
    changed
}

/// Severity of a [`lint`] finding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LintLevel {
    /// The app would refuse or silently drop this.
    Error,
    /// Loads, but something will not behave as the file suggests.
    Warning,
}

/// One finding of [`lint`].
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lint {
    pub level: LintLevel,
    pub message: String,
    /// What the finding is about (`rule:<name>`, `fence:<title>`, `layout:<n>`, `file`).
    pub subject: String,
}

fn lint_at(out: &mut Vec<Lint>, level: LintLevel, subject: impl Into<String>, message: String) {
    out.push(Lint {
        level,
        message,
        subject: subject.into(),
    });
}

/// Cross-checks a configuration that already passed [`validate`] for things that load but will
/// not work as written: rules whose target fence exists in no layout, portals whose folder is
/// gone, tabs hosted by a missing fence, duplicate fence ids, fence sizes the app clamps,
/// memberships of unknown items, more snapshots than the app keeps, a missing or foreign
/// `$schema`. Pure apart from the folder existence checks; safe to run on an exported file
/// without the app.
pub fn lint(cfg: &Config) -> Vec<Lint> {
    use crate::model::FenceKind;
    use crate::rules::Target;
    let mut out = Vec::new();
    match cfg.schema.as_deref() {
        None => lint_at(
            &mut out,
            LintLevel::Warning,
            "file",
            format!(
                "no \"$schema\" field; add {:?} for editor validation",
                crate::model::CONFIG_SCHEMA_URL
            ),
        ),
        Some(url) if url != crate::model::CONFIG_SCHEMA_URL => lint_at(
            &mut out,
            LintLevel::Warning,
            "file",
            format!(
                "\"$schema\" is {url:?}; PecoFence publishes {:?}",
                crate::model::CONFIG_SCHEMA_URL
            ),
        ),
        Some(_) => {}
    }
    if cfg.layouts.is_empty() {
        lint_at(
            &mut out,
            LintLevel::Warning,
            "file",
            "no layouts: the app will create a default one on load".into(),
        );
    }
    let mut all_fences: Vec<(&crate::model::Fence, usize)> = Vec::new();
    for (li, layout) in cfg.layouts.iter().enumerate() {
        let subject = format!("layout:{li}");
        if layout.fences.iter().all(|f| f.kind != FenceKind::Inbox) {
            lint_at(
                &mut out,
                LintLevel::Warning,
                &subject,
                "no inbox fence; the app adds one on load".into(),
            );
        }
        let mut seen = std::collections::HashSet::new();
        for f in &layout.fences {
            if !seen.insert(f.id) {
                lint_at(
                    &mut out,
                    LintLevel::Error,
                    format!("fence:{}", f.title),
                    format!("fence id {} appears twice in layout {li}", f.id),
                );
            }
            all_fences.push((f, li));
        }
        for f in &layout.fences {
            let subject = format!("fence:{}", f.title);
            if let Some(host) = f.tab_host {
                match layout.fences.iter().find(|h| h.id == host) {
                    None => lint_at(
                        &mut out,
                        LintLevel::Warning,
                        &subject,
                        format!(
                            "hosted by tab host {host}, which is not in layout {li}; shown as its own window"
                        ),
                    ),
                    Some(h) if h.tab_host.is_some() => lint_at(
                        &mut out,
                        LintLevel::Warning,
                        &subject,
                        format!("tab host {:?} is itself a tab", h.title),
                    ),
                    Some(_) => {}
                }
            }
            if let crate::model::ItemSourceSpec::Folder { path, .. } = &f.source
                && !Path::new(path).is_dir()
            {
                lint_at(
                    &mut out,
                    LintLevel::Warning,
                    &subject,
                    format!(
                        "portal folder {path:?} does not exist (or is not reachable from here)"
                    ),
                );
            }
            let g = &f.geometry;
            let (w, h) = size_in_range(g);
            if (w, h) != (g.w, g.h) {
                lint_at(
                    &mut out,
                    LintLevel::Warning,
                    &subject,
                    format!(
                        "size {}x{} is out of range; the app changes it to {w}x{h}",
                        g.w, g.h
                    ),
                );
            }
            if f.kind == FenceKind::FolderPortal
                && !matches!(f.source, crate::model::ItemSourceSpec::Folder { .. })
            {
                lint_at(
                    &mut out,
                    LintLevel::Error,
                    &subject,
                    "portal fence without a folder source".into(),
                );
            }
            let unknown = f
                .items
                .iter()
                .filter(|r| !cfg.items.contains_key(&r.item_id))
                .count();
            if unknown > 0 {
                lint_at(
                    &mut out,
                    LintLevel::Warning,
                    &subject,
                    format!(
                        "{unknown} membership entr{} refer to items missing from \"items\"; they are ignored",
                        if unknown == 1 { "y" } else { "ies" }
                    ),
                );
            }
        }
    }
    for rule in &cfg.rules.list {
        if let Target::Fence(id) = rule.target {
            match all_fences.iter().find(|(f, _)| f.id == id) {
                None => lint_at(
                    &mut out,
                    LintLevel::Warning,
                    format!("rule:{}", rule.name),
                    format!(
                        "target fence {id} exists in no layout; matches fall back to the inbox"
                    ),
                ),
                Some((f, _)) if f.kind == FenceKind::FolderPortal => lint_at(
                    &mut out,
                    LintLevel::Error,
                    format!("rule:{}", rule.name),
                    format!(
                        "target {:?} is a folder portal; rules can only target virtual fences or the inbox",
                        f.title
                    ),
                ),
                Some(_) => {}
            }
        }
        if rule.all_of.is_empty() {
            lint_at(
                &mut out,
                LintLevel::Warning,
                format!("rule:{}", rule.name),
                "no conditions: the rule never matches".into(),
            );
        }
    }
    if let Target::Fence(id) = cfg.rules.default_target
        && !all_fences.iter().any(|(f, _)| f.id == id)
    {
        lint_at(
            &mut out,
            LintLevel::Warning,
            "rules",
            format!(
                "defaultTarget fence {id} exists in no layout; unmatched items go to the inbox"
            ),
        );
    }
    if let Some(text) = &cfg.settings.peek.custom_hotkey
        && let Err(problem) = crate::hotkey::Hotkey::parse(text).and_then(|h| h.check())
    {
        lint_at(
            &mut out,
            LintLevel::Warning,
            "settings",
            format!("peek.customHotkey {text:?} is not used ({problem}); Peek uses peek.hotkey"),
        );
    }
    if let Some(text) = &cfg.settings.windows_desktop_hotkey
        && let Err(problem) = crate::hotkey::Hotkey::parse(text).and_then(|h| h.check())
    {
        lint_at(
            &mut out,
            LintLevel::Warning,
            "settings",
            format!("windowsDesktopHotkey {text:?} is not used ({problem}); it has no shortcut"),
        );
    }
    if cfg.snapshots.len() > crate::model::MAX_SNAPSHOTS {
        lint_at(
            &mut out,
            LintLevel::Warning,
            "snapshots",
            format!(
                "{} snapshots; the app keeps at most {}",
                cfg.snapshots.len(),
                crate::model::MAX_SNAPSHOTS
            ),
        );
    }
    out
}

/// Schema migration chain, and choices this version no longer offers.
pub fn migrate(mut cfg: Config) -> Config {
    if cfg.schema_version < SCHEMA_VERSION {
        cfg.schema_version = SCHEMA_VERSION;
    }
    // 0.2.0 offered Win+Space and Win+Shift+Space, which Windows keeps for the input switcher:
    // Peek never got them, and a file still choosing one showed the fallback notice at every
    // start. It takes the default (a usable `customHotkey` stays the choice).
    let peek = &mut cfg.settings.peek;
    if peek.chosen_hotkey() == peek.hotkey.combo() {
        peek.hotkey = crate::model::PeekHotkey::CtrlAltSpace;
    }
    cfg
}

/// Local-date string without pulling in a date crate (civil-from-days algorithm, UTC-based;
/// backups are named for humans so the timezone offset is irrelevant).
pub fn today_yyyy_mm_dd() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    #[test]
    fn export_leaves_the_search_history_out() {
        let dir = tmpdir("export-history");
        fs::create_dir_all(&dir).unwrap();
        let mut c = sample();
        c.note_search_open("C:\\Users\\me\\secret.txt");
        let path = dir.join("out.json");
        ConfigStore::export_to(&c, &path).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(!text.contains("secret"), "{text}");
        assert!(!text.contains("searchHistory"));
        let _ = fs::remove_dir_all(&dir);
    }

    fn tmpdir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("pecofence-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    fn sample() -> Config {
        let mut c = Config::default();
        c.layouts.push(Layout {
            shown: 0,
            fingerprint: vec![],
            fences: vec![Fence::new(
                "A",
                FenceKind::Inbox,
                NormGeometry {
                    monitor: "m".into(),
                    x: 1.0,
                    y: 2.0,
                    w: 300.0,
                    h: 200.0,
                    work_w: 1920.0,
                    work_h: 1000.0,
                    anchor: Anchor::LeftTop,
                },
            )],
        });
        c
    }

    #[test]
    fn save_then_load_primary() {
        let store = ConfigStore::new(tmpdir("primary"));
        store.save(&sample()).unwrap();
        assert!(matches!(store.load(), LoadOutcome::Primary(_)));
        assert!(store.primary_path().exists());
        assert!(!store.tmp_path().exists());
        assert_eq!(store.list_backups().len(), 1);
    }

    /// A saver whose `done` reports every write on the returned channel and then waits for
    /// one message on the returned sender: the test decides when a write is over.
    fn held_saver(
        store: &ConfigStore,
    ) -> (
        ConfigSaver,
        std::sync::mpsc::Receiver<bool>,
        std::sync::mpsc::Sender<()>,
    ) {
        let (wrote_tx, wrote) = std::sync::mpsc::channel();
        let (release, held) = std::sync::mpsc::channel::<()>();
        let held = Mutex::new(held);
        let done = move |_: Duration, result: &Result<(), SaveError>| {
            wrote_tx.send(result.is_ok()).unwrap();
            held.lock().unwrap().recv().unwrap();
        };
        (
            ConfigSaver::new(store.clone(), Box::new(done)),
            wrote,
            release,
        )
    }

    fn titled(title: &str) -> Config {
        let mut c = sample();
        c.layouts[0].fences[0].title = title.into();
        c
    }

    fn saved_title(store: &ConfigStore) -> String {
        store.load().into_config().layouts[0].fences[0]
            .title
            .clone()
    }

    #[test]
    fn saver_writes_on_its_own_thread_and_wait_returns_after_the_write() {
        let store = ConfigStore::new(tmpdir("saver-wait"));
        let caller = std::thread::current().id();
        let on_caller = Arc::new(Mutex::new(None));
        let seen = on_caller.clone();
        let saver = ConfigSaver::new(
            store.clone(),
            Box::new(move |_, result| {
                assert!(result.is_ok());
                *seen.lock().unwrap() = Some(std::thread::current().id() == caller);
            }),
        );
        saver.wait(); // nothing handed over: returns at once
        saver.save(titled("first"));
        saver.wait();
        assert_eq!(*on_caller.lock().unwrap(), Some(false));
        assert_eq!(saved_title(&store), "first");
        assert!(!store.tmp_path().exists());
        assert!(!saver.take_failed());
    }

    #[test]
    fn saver_writes_only_the_newest_of_those_handed_over_during_a_write() {
        let store = ConfigStore::new(tmpdir("saver-newest"));
        let (saver, wrote, release) = held_saver(&store);
        saver.save(titled("a"));
        assert!(wrote.recv().unwrap()); // "a" is on disk, its write not yet over
        saver.save(titled("b"));
        saver.save(titled("c"));
        saver.save(titled("d"));
        assert_eq!(saved_title(&store), "a");
        release.send(()).unwrap();
        assert!(wrote.recv().unwrap());
        assert_eq!(saved_title(&store), "d");
        release.send(()).unwrap();
        saver.wait();
        // Two writes in all: "b" and "c" were never written.
        assert!(wrote.try_recv().is_err());
        // "a" was rotated to config.bak by the write of "d".
        let bak = ConfigStore::parse_file(&store.bak_path()).unwrap();
        assert_eq!(bak.layouts[0].fences[0].title, "a");
    }

    #[test]
    fn saver_wait_holds_the_caller_until_the_write_is_over() {
        let store = ConfigStore::new(tmpdir("saver-holds"));
        let (saver, wrote, release) = held_saver(&store);
        saver.save(titled("a"));
        assert!(wrote.recv().unwrap());
        saver.save(titled("b"));
        let waited = Arc::new(Mutex::new(false));
        std::thread::scope(|scope| {
            let flag = waited.clone();
            let saver = &saver;
            scope.spawn(move || {
                saver.wait();
                *flag.lock().unwrap() = true;
            });
            std::thread::sleep(Duration::from_millis(200));
            assert!(!*waited.lock().unwrap(), "wait returned during a write");
            release.send(()).unwrap(); // "a" over, "b" starts
            assert!(wrote.recv().unwrap());
            std::thread::sleep(Duration::from_millis(200));
            assert!(!*waited.lock().unwrap(), "wait returned before the newest");
            release.send(()).unwrap();
        });
        assert!(*waited.lock().unwrap());
        assert_eq!(saved_title(&store), "b");
    }

    #[test]
    fn a_waiter_waits_like_the_saver_itself() {
        let store = ConfigStore::new(tmpdir("saver-waiter"));
        let (saver, wrote, release) = held_saver(&store);
        let waiter = saver.waiter();
        waiter.wait(); // nothing handed over: returns at once
        saver.save(titled("a"));
        assert!(wrote.recv().unwrap());
        let waited = Arc::new(Mutex::new(false));
        std::thread::scope(|scope| {
            let flag = waited.clone();
            let waiter = &waiter;
            scope.spawn(move || {
                waiter.wait();
                *flag.lock().unwrap() = true;
            });
            std::thread::sleep(Duration::from_millis(200));
            assert!(!*waited.lock().unwrap(), "wait returned during a write");
            release.send(()).unwrap();
        });
        assert!(*waited.lock().unwrap());
        assert_eq!(saved_title(&store), "a");
    }

    #[test]
    fn saver_reports_a_failed_write_once_and_a_later_good_write_clears_it() {
        let dir = tmpdir("saver-failed");
        // A file where the config folder should be: the save fails at "create folder".
        let blocker = dir.join("in-the-way");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&blocker, "x").unwrap();
        let store = ConfigStore::new(blocker.join("config"));
        let steps = Arc::new(Mutex::new(Vec::new()));
        let seen = steps.clone();
        let saver = ConfigSaver::new(
            store.clone(),
            Box::new(move |_, result| {
                seen.lock()
                    .unwrap()
                    .push(result.as_ref().err().map(|e| e.step));
            }),
        );
        saver.save(titled("a"));
        saver.wait();
        assert!(saver.take_failed());
        assert!(!saver.take_failed());
        // An unreported failure followed by a good write: the disk is current, nothing to say.
        saver.save(titled("b"));
        saver.wait();
        fs::remove_file(&blocker).unwrap();
        saver.save(titled("c"));
        saver.wait();
        assert!(!saver.take_failed());
        assert_eq!(saved_title(&store), "c");
        assert_eq!(
            *steps.lock().unwrap(),
            [Some("create folder"), Some("create folder"), None]
        );
    }

    /// config.json is written but the day's backup copy cannot be: the changes are saved, so
    /// the write does not count as failed (the owner logs it and shows no "cannot save").
    #[test]
    fn a_save_that_only_misses_the_daily_backup_is_no_failed_save() {
        let store = ConfigStore::new(tmpdir("saver-backup-only"));
        fs::create_dir_all(store.dir()).unwrap();
        // A file where the backups folder should be.
        fs::write(store.backups_dir(), "x").unwrap();
        let error = store.save(&titled("a")).unwrap_err();
        assert!(error.config_written(), "{error}");
        assert_eq!(saved_title(&store), "a");
        let saver = ConfigSaver::new(store.clone(), Box::new(|_, _| {}));
        saver.save(titled("b"));
        saver.wait();
        assert!(!saver.take_failed());
        assert_eq!(saved_title(&store), "b");
    }

    #[test]
    fn dropped_saver_still_writes_what_waits() {
        let store = ConfigStore::new(tmpdir("saver-dropped"));
        let (saver, wrote, release) = held_saver(&store);
        saver.save(titled("a"));
        assert!(wrote.recv().unwrap());
        saver.save(titled("b"));
        drop(saver);
        release.send(()).unwrap();
        assert!(wrote.recv().unwrap());
        assert_eq!(saved_title(&store), "b");
        release.send(()).unwrap();
    }

    #[test]
    fn corrupt_primary_falls_back_to_bak() {
        let store = ConfigStore::new(tmpdir("bak"));
        store.save(&sample()).unwrap();
        store.save(&sample()).unwrap(); // creates .bak
        fs::write(store.primary_path(), "{ not json").unwrap();
        match store.load() {
            LoadOutcome::Recovered(c, from) => {
                assert_eq!(c.layouts[0].fences[0].title, "A");
                assert_eq!(from, store.bak_path());
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn opacity_presets_validate_and_stray_values_do_not() {
        for (opacity, ok) in [
            (0.0, true),
            (0.55, true),
            (1.6, true),
            (0.1, false),
            (2.5, false),
        ] {
            let mut c = sample();
            c.layouts[0].fences[0].appearance = Some(AppearanceOverride {
                opacity: Some(opacity),
                ..Default::default()
            });
            assert_eq!(validate(&c).is_ok(), ok, "opacity {opacity}");
        }
    }

    #[test]
    fn unreadable_primary_survives_later_saves() {
        // A config written by a newer version: this build cannot read it, falls back to the
        // defaults and saves twice. The original text must still be on disk afterwards.
        let store = ConfigStore::new(tmpdir("unreadable"));
        fs::create_dir_all(store.dir()).unwrap();
        let text = r#"{ "schemaVersion": 99, "marker": "users-layout" }"#;
        fs::write(store.primary_path(), text).unwrap();
        let (outcome, report) = store.load_reporting();
        assert!(matches!(outcome, LoadOutcome::Fresh(_)));
        let moved = report.moved_to.expect("unreadable primary moved aside");
        store.save(&sample()).unwrap();
        store.save(&sample()).unwrap();
        assert_eq!(fs::read_to_string(&moved).unwrap(), text);
        assert!(matches!(store.load(), LoadOutcome::Primary(_)));
    }

    #[test]
    fn primary_with_a_utf8_bom_loads() {
        let store = ConfigStore::new(tmpdir("bom"));
        store.save(&sample()).unwrap();
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend(fs::read(store.primary_path()).unwrap());
        fs::write(store.primary_path(), bytes).unwrap();
        assert!(ConfigStore::parse_file(&store.primary_path()).is_ok());
        assert!(matches!(store.load(), LoadOutcome::Primary(_)));
    }

    #[cfg(windows)]
    #[test]
    fn primary_locked_for_a_moment_still_loads() {
        use std::os::windows::fs::OpenOptionsExt;
        let store = ConfigStore::new(tmpdir("locked"));
        store.save(&sample()).unwrap();
        // Held without sharing, as a scanner does: opens fail with ERROR_SHARING_VIOLATION.
        let lock = fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(store.primary_path())
            .unwrap();
        let unlock = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(400));
            drop(lock);
        });
        let (outcome, report) = store.load_reporting();
        unlock.join().unwrap();
        assert!(matches!(outcome, LoadOutcome::Primary(_)), "{outcome:?}");
        assert_eq!(report.moved_to, None);
        assert!(report.failures.is_empty());
    }

    /// Opens `path` without sharing, as a scanner or a sync tool does.
    #[cfg(windows)]
    fn hold(path: &Path) -> fs::File {
        use std::os::windows::fs::OpenOptionsExt;
        fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(path)
            .unwrap()
    }

    #[cfg(windows)]
    #[test]
    fn a_primary_locked_past_the_wait_is_named_and_left_alone() {
        let store = ConfigStore::new(tmpdir("locked-long"));
        store.save(&sample()).unwrap();
        store.save(&sample()).unwrap(); // config.bak
        let original = fs::read(store.primary_path()).unwrap();
        let lock = hold(&store.primary_path());
        assert!(ConfigStore::is_locked(&store.primary_path()));
        let (outcome, report) = store.load_waiting(std::time::Duration::from_millis(600));
        assert!(
            matches!(&outcome, LoadOutcome::Recovered(_, from) if *from == store.bak_path()),
            "{outcome:?}"
        );
        assert_eq!(report.locked, Some(store.primary_path()));
        assert_eq!(report.moved_to, None, "a locked file is not damaged");
        assert!(report.waited >= std::time::Duration::from_millis(500));
        drop(lock);
        assert!(!ConfigStore::is_locked(&store.primary_path()));
        assert_eq!(fs::read(store.primary_path()).unwrap(), original);
        let (outcome, report) = store.load_waiting(std::time::Duration::ZERO);
        assert!(matches!(outcome, LoadOutcome::Primary(_)), "{outcome:?}");
        assert_eq!(report.locked, None);
    }

    #[cfg(windows)]
    #[test]
    fn every_file_locked_gives_the_defaults_and_names_the_primary() {
        let store = ConfigStore::new(tmpdir("locked-all"));
        store.save(&sample()).unwrap();
        store.save(&sample()).unwrap();
        let mut held = vec![hold(&store.primary_path()), hold(&store.bak_path())];
        held.extend(store.list_backups().iter().map(|p| hold(p)));
        let started = std::time::Instant::now();
        let (outcome, report) = store.load_waiting(std::time::Duration::from_millis(400));
        // One shared wait, not one per file.
        assert!(started.elapsed() < std::time::Duration::from_millis(1500));
        assert!(matches!(outcome, LoadOutcome::Fresh(_)), "{outcome:?}");
        assert_eq!(report.locked, Some(store.primary_path()));
        assert_eq!(report.failures.len(), 3, "{report:?}");
        assert!(report.failures.iter().all(|(_, _, e)| e.locked()));
        drop(held);
        assert!(matches!(store.load(), LoadOutcome::Primary(_)));
    }

    #[test]
    fn a_free_or_missing_file_is_not_locked() {
        let store = ConfigStore::new(tmpdir("not-locked"));
        assert!(!ConfigStore::is_locked(&store.primary_path()));
        store.save(&sample()).unwrap();
        assert!(!ConfigStore::is_locked(&store.primary_path()));
        let (_, report) = store.load_reporting();
        assert_eq!(report.locked, None);
        assert_eq!(report.waited, std::time::Duration::ZERO);
    }

    #[cfg(unix)]
    #[test]
    fn primary_without_read_access_is_not_waited_for() {
        use std::os::unix::fs::PermissionsExt;
        let store = ConfigStore::new(tmpdir("denied"));
        store.save(&sample()).unwrap();
        fs::set_permissions(store.primary_path(), fs::Permissions::from_mode(0o000)).unwrap();
        let started = std::time::Instant::now();
        let _ = store.load_reporting();
        assert!(started.elapsed() < std::time::Duration::from_millis(1000));
    }

    #[test]
    fn nothing_usable_gives_fresh() {
        let store = ConfigStore::new(tmpdir("fresh"));
        let (outcome, report) = store.load_reporting();
        assert!(matches!(outcome, LoadOutcome::Fresh(_)));
        // Missing files are no failures: a first start.
        assert!(report.failures.is_empty(), "{report:?}");
    }

    #[test]
    fn files_that_cannot_be_used_are_reported_in_order() {
        let store = ConfigStore::new(tmpdir("failures"));
        store.save(&sample()).unwrap();
        fs::write(store.primary_path(), "{ not json").unwrap();
        // A folder in place of config.bak cannot be read at all.
        fs::create_dir_all(store.bak_path()).unwrap();
        let (outcome, report) = store.load_reporting();
        assert!(matches!(outcome, LoadOutcome::Recovered(..)), "{outcome:?}");
        let found: Vec<_> = report
            .failures
            .iter()
            .map(|(name, path, e)| (*name, path.clone(), matches!(e, ReadError::Io(_))))
            .collect();
        assert_eq!(
            found,
            [
                ("config.json", store.primary_path(), false),
                ("config.bak", store.bak_path(), true),
            ]
        );
        assert!(report.failures.iter().all(|(_, _, e)| !e.not_encrypted()));
    }

    #[test]
    fn error_409_means_not_encrypted() {
        let e = ReadError::Io(io::Error::from_raw_os_error(APPX_FILE_NOT_ENCRYPTED));
        assert!(e.not_encrypted());
        assert!(e.to_string().contains("409"), "{e}");
        assert!(!ReadError::Io(io::Error::from_raw_os_error(5)).not_encrypted());
        assert!(!ReadError::Invalid("unsupported schemaVersion 99".into()).not_encrypted());
    }

    #[test]
    fn renamed_product_reuses_old_configuration_without_changing_it() {
        let parent = tmpdir("rebrand");
        let old = ConfigStore::new(parent.join(crate::brand::LEGACY_DATA_DIR));
        let mut config = sample();
        config.layouts[0].fences[0].title = "My OpenFence files {1}".into();
        config.settings.language = crate::i18n::Language::Japanese;
        old.save(&config).unwrap();
        let original = fs::read(old.primary_path()).unwrap();
        let preferred = parent.join(crate::brand::NAME);
        let selected = ConfigStore::with_legacy(&preferred, old.dir());
        assert_eq!(selected.dir(), old.dir());
        let mut loaded = selected.load().into_config();
        // Saving stamps the published schema URL; everything else is untouched.
        assert_eq!(
            loaded.schema.as_deref(),
            Some(crate::model::CONFIG_SCHEMA_URL)
        );
        loaded.schema = None;
        assert_eq!(
            serde_json::to_value(loaded).unwrap(),
            serde_json::to_value(config).unwrap()
        );
        assert_eq!(fs::read(old.primary_path()).unwrap(), original);
        assert!(!preferred.exists());
    }

    #[test]
    fn current_product_data_wins_and_old_backup_only_installs_still_load() {
        let parent = tmpdir("rebrand-precedence");
        let old = ConfigStore::new(parent.join(crate::brand::LEGACY_DATA_DIR));
        old.save(&sample()).unwrap();
        old.save(&sample()).unwrap();
        fs::remove_file(old.primary_path()).unwrap();
        let preferred = ConfigStore::new(parent.join(crate::brand::NAME));
        let selected = ConfigStore::with_legacy(preferred.dir(), old.dir());
        assert!(matches!(selected.load(), LoadOutcome::Recovered(..)));
        preferred.save(&sample()).unwrap();
        // Even a damaged new config belongs to the new installation. Its normal
        // backup recovery must not silently switch to a stale legacy layout.
        fs::write(preferred.primary_path(), "{ bad json").unwrap();
        assert_eq!(
            ConfigStore::with_legacy(preferred.dir(), old.dir()).dir(),
            preferred.dir()
        );
        assert_eq!(
            ConfigStore::with_legacy(parent.join("fresh"), parent.join("missing")).dir(),
            parent.join("fresh")
        );
    }

    #[test]
    fn validation_rejects_two_inboxes() {
        let mut c = sample();
        let f = c.layouts[0].fences[0].clone();
        c.layouts[0].fences.push(Fence {
            id: uuid::Uuid::new_v4(),
            ..f
        });
        assert!(validate(&c).is_err());
    }

    #[test]
    fn a_win_space_peek_shortcut_from_0_2_0_loads_as_the_default() {
        let mut c = sample();
        c.settings.peek.hotkey = PeekHotkey::WinSpace;
        assert_eq!(migrate(c.clone()).settings.peek, PeekSettings::default());
        c.settings.peek.hotkey = PeekHotkey::WinShiftSpace;
        assert_eq!(migrate(c.clone()).settings.peek, PeekSettings::default());
        // A usable custom shortcut is the choice; `hotkey` is only for older versions then.
        c.settings.peek.custom_hotkey = Some("Ctrl+Shift+F".into());
        let peek = migrate(c).settings.peek;
        assert_eq!(peek.hotkey, PeekHotkey::WinShiftSpace);
        assert_eq!(peek.chosen_hotkey().to_string(), "Ctrl+Shift+F");
    }

    #[test]
    fn an_out_of_range_size_costs_only_that_size() {
        // A hand edit: one fence 1e9 wide and -500 tall, another 0 tall. The file still loads
        // as it is, `config check` warns, and the sizes come into range.
        let store = ConfigStore::new(tmpdir("sizes"));
        let mut c = sample();
        let mut b = Fence::new(
            "B",
            FenceKind::Virtual,
            c.layouts[0].fences[0].geometry.clone(),
        );
        b.geometry.h = 0.0;
        c.layouts[0].fences[0].geometry.w = 1e9;
        c.layouts[0].fences[0].geometry.h = -500.0;
        c.layouts[0].fences.push(b);
        fs::create_dir_all(store.dir()).unwrap();
        fs::write(store.primary_path(), serde_json::to_string(&c).unwrap()).unwrap();
        let (outcome, report) = store.load_reporting();
        let LoadOutcome::Primary(mut loaded) = outcome else {
            panic!("{outcome:?}");
        };
        assert_eq!(report.moved_to, None);
        let lints = lint(&loaded);
        let sizes: Vec<&Lint> = lints
            .iter()
            .filter(|l| l.message.starts_with("size "))
            .collect();
        assert_eq!(sizes.len(), 2, "{lints:?}");
        assert_eq!(sizes[0].subject, "fence:A");
        assert_eq!(sizes[1].subject, "fence:B");
        assert!(sizes.iter().all(|l| l.level == LintLevel::Warning));
        assert!(clamp_sizes(&mut loaded));
        let g = &loaded.layouts[0].fences[0].geometry;
        assert_eq!((g.w, g.h), (8192.0, 1.0));
        assert_eq!(loaded.layouts[0].fences[1].geometry.h, 1.0);
        assert!(!clamp_sizes(&mut loaded));
        assert!(
            lint(&loaded)
                .iter()
                .all(|l| !l.message.starts_with("size "))
        );
    }

    /// A network portal saved by 0.2.1 (`UNC\server\share\…`) is repaired in the layouts and
    /// in the snapshots; drive paths are left alone.
    #[test]
    fn a_network_portal_saved_by_0_2_1_gets_its_path_back() {
        let folder = |path: &str| ItemSourceSpec::Folder {
            path: path.to_string(),
            recursive: false,
            filter: None,
        };
        let mut c = sample();
        let mut portal = c.layouts[0].fences[0].clone();
        portal.id = uuid::Uuid::new_v4();
        portal.kind = FenceKind::FolderPortal;
        portal.source = folder(r"UNC\nas\share\docs");
        let mut local = portal.clone();
        local.id = uuid::Uuid::new_v4();
        local.source = folder(r"C:\Users\Me\Documents");
        c.layouts[0].fences.push(portal.clone());
        c.layouts[0].fences.push(local);
        c.snapshots.push(Snapshot {
            id: uuid::Uuid::new_v4(),
            name: "before".into(),
            ts: 1,
            layouts: vec![Layout {
                shown: 0,
                fingerprint: vec![],
                fences: vec![portal],
            }],
            notes: Default::default(),
            scripts: Default::default(),
        });
        assert!(repair_unc_portals(&mut c));
        let path_of = |f: &Fence| match &f.source {
            ItemSourceSpec::Folder { path, .. } => path.clone(),
            ItemSourceSpec::Desktop => String::new(),
        };
        assert_eq!(path_of(&c.layouts[0].fences[1]), r"\\nas\share\docs");
        assert_eq!(path_of(&c.layouts[0].fences[2]), r"C:\Users\Me\Documents");
        assert_eq!(
            path_of(&c.snapshots[0].layouts[0].fences[0]),
            r"\\nas\share\docs"
        );
        assert!(!repair_unc_portals(&mut c));
    }

    #[test]
    fn saved_file_starts_with_the_schema_field_and_loads_back() {
        let store = ConfigStore::new(tmpdir("schema"));
        store.save(&sample()).unwrap();
        let text = fs::read_to_string(store.primary_path()).unwrap();
        let second_line = text.lines().nth(1).unwrap_or_default();
        assert!(second_line.contains("\"$schema\""), "{second_line}");
        assert!(text.contains(crate::model::CONFIG_SCHEMA_URL));
        let cfg = ConfigStore::parse_file(&store.primary_path()).unwrap();
        assert_eq!(cfg.schema.as_deref(), Some(crate::model::CONFIG_SCHEMA_URL));
        // A file without the field (older versions) still loads.
        let stripped = text.replacen(
            &format!("\"$schema\": \"{}\",", crate::model::CONFIG_SCHEMA_URL),
            "",
            1,
        );
        assert!(!stripped.contains("$schema"));
        let cfg: Config = serde_json::from_str(&stripped).unwrap();
        assert_eq!(cfg.schema, None);
    }

    #[test]
    fn lint_finds_dangling_targets_and_folders() {
        use crate::rules::{Cond, Rule, Target, TypeCategory};
        let mut c = sample();
        let inbox = c.layouts[0].fences[0].id;
        // Valid rule to the inbox, a rule to a fence that exists nowhere, a rule without
        // conditions, a portal whose folder is gone, a tab hosted by a missing fence.
        c.rules.list.push(Rule::new(
            "ok",
            Target::Fence(inbox),
            vec![Cond::Type(vec![TypeCategory::Images])],
        ));
        let ghost = uuid::Uuid::new_v4();
        c.rules.list.push(Rule::new(
            "ghost",
            Target::Fence(ghost),
            vec![Cond::Type(vec![TypeCategory::Video])],
        ));
        c.rules.list.push(Rule::new("empty", Target::Inbox, vec![]));
        let geometry = c.layouts[0].fences[0].geometry.clone();
        let mut portal = Fence::new("Gone", FenceKind::FolderPortal, geometry.clone());
        portal.source = ItemSourceSpec::Folder {
            path: "Z:\\definitely\\missing\\folder".into(),
            recursive: false,
            filter: None,
        };
        let portal_id = portal.id;
        c.layouts[0].fences.push(portal);
        let mut tab = Fence::new("Tab", FenceKind::Virtual, geometry);
        tab.tab_host = Some(ghost);
        c.layouts[0].fences.push(tab);
        c.rules.list.push(Rule::new(
            "to-portal",
            Target::Fence(portal_id),
            vec![Cond::FoldersOnly],
        ));
        assert!(validate(&c).is_ok(), "lint covers what validate does not");

        let lints = lint(&c);
        let subjects: Vec<&str> = lints.iter().map(|l| l.subject.as_str()).collect();
        assert!(subjects.contains(&"file"), "$schema missing: {lints:?}");
        assert!(subjects.contains(&"rule:ghost"), "{lints:?}");
        assert!(subjects.contains(&"rule:empty"), "{lints:?}");
        assert!(subjects.contains(&"fence:Gone"), "{lints:?}");
        assert!(subjects.contains(&"fence:Tab"), "{lints:?}");
        assert!(!subjects.contains(&"rule:ok"), "{lints:?}");
        let errors: Vec<&Lint> = lints
            .iter()
            .filter(|l| l.level == LintLevel::Error)
            .collect();
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].subject, "rule:to-portal");

        // A clean, saved-and-reloaded config lints without findings.
        let store = ConfigStore::new(tmpdir("lint-clean"));
        let mut clean = sample();
        let clean_inbox = clean.layouts[0].fences[0].id;
        clean.rules.list.push(Rule::new(
            "ok",
            Target::Fence(clean_inbox),
            vec![Cond::Type(vec![TypeCategory::Images])],
        ));
        store.save(&clean).unwrap();
        let reloaded = ConfigStore::parse_file(&store.primary_path()).unwrap();
        assert_eq!(lint(&reloaded), Vec::<Lint>::new());
    }

    #[test]
    fn versions_compare_by_number() {
        assert!(newer_version("0.4.0", "0.3.2"));
        assert!(newer_version("0.3.10", "0.3.9"));
        assert!(newer_version("1.0", "0.9.9"));
        assert!(newer_version("0.4.0-beta.1", "0.3.2"));
        assert!(!newer_version("0.3.2", "0.3.2"));
        assert!(!newer_version("0.2.5", "0.3.2"));
        assert!(!newer_version("garbage", "0.3.2"));
        assert!(!newer_version("", "0.3.2"));
    }

    #[test]
    fn a_copy_of_the_primary_goes_to_the_backups_once_a_day() {
        let store = ConfigStore::new(tmpdir("keep-copy"));
        let mut c = sample();
        c.saved_by = Some("9.9.9".into());
        store.save(&c).unwrap();
        let text = fs::read_to_string(store.primary_path()).unwrap();
        assert!(text.contains("\"savedBy\": \"9.9.9\""), "{text}");
        let copy = store.keep_copy("9.9.9").unwrap();
        assert_eq!(fs::read_to_string(&copy).unwrap(), text);
        assert!(store.list_backups().contains(&copy));
        // A later save keeps it; the same day's second copy does not replace it.
        c.saved_by = Some("0.0.1".into());
        store.save(&c).unwrap();
        assert_eq!(store.keep_copy("9.9.9").unwrap(), copy);
        assert_eq!(fs::read_to_string(&copy).unwrap(), text);
        assert!(ConfigStore::parse_file(&copy).is_ok());
    }

    #[test]
    fn date_formatting_is_sane() {
        let s = today_yyyy_mm_dd();
        assert_eq!(s.len(), 10);
        assert!(s.starts_with("20"));
    }
}
