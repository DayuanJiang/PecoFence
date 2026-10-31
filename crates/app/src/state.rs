//! Application state: config + layout selection + desktop item catalog + rule routing.

use pecofence_core::geometry::{self, PxRect, WorkArea};
use pecofence_core::rules::{Cond, Decision, RuleSet, Target, Template, Unmatched};
use pecofence_core::{
    AssignedBy, Config, ConfigStore, Fence, FenceId, FenceKind, IconKey, Item, ItemId, ItemKey,
    ItemRef, ItemSourceSpec, Layout, LoadOutcome, MonitorIdentity, Origin, SortMode,
};
use pecofence_platform::RECT;
use pecofence_platform::shell::{self, DesktopEntry, EntryOrigin};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const ORPHAN_GC_SECS: i64 = 7 * 24 * 3600;

pub struct AppState {
    pub config: Config,
    store: ConfigStore,
    pub layout: usize,
    dirty: bool,
    catalog: HashMap<ItemKey, ItemId>,
    /// Runtime-only items of folder portals (never persisted; rebuilt from the folder).
    portal_items: HashMap<ItemId, Item>,
    /// Portal fence → its current item ids.
    portal_members: HashMap<FenceId, Vec<ItemId>>,
    /// Fence showing a folder → subfolder it has navigated into (absent = the folder itself).
    portal_cwd: HashMap<FenceId, PathBuf>,
    /// Regular fence → a folder double-clicked in it (a desktop folder or a folder shortcut's
    /// target), shown in place of its icons until the fence goes up from it. Runtime only.
    opened_folders: HashMap<FenceId, OpenedFolder>,
    /// Portals whose folder could not be found at the last read.
    portal_missing: HashSet<FenceId>,
    /// When the youngest `.tmp` the rules skipped as too new has lasted `TMP_SETTLE_SECS`
    /// (Unix seconds): the App syncs the desktop again then (`take_tmp_settle_at`).
    tmp_settle_at: std::cell::Cell<Option<i64>>,
    pub work_areas: Vec<WorkArea>,
    /// Fences with no spot saved on the current monitors (see [`Self::ensure_layout`]): they
    /// sit where their spot elsewhere puts them until the App moves them out of the others' way.
    pub unplaced: Vec<FenceId>,
    pub first_run: bool,
    pub recovered_from: Option<PathBuf>,
    /// Where an unreadable `config.json` was moved at load (see `ConfigStore::load_reporting`).
    pub unreadable_moved_to: Option<PathBuf>,
    /// The saved files that could not be used at load, and why.
    pub load_failures: Vec<(
        &'static str,
        PathBuf,
        pecofence_core::config_store::ReadError,
    )>,
}

/// Summary of a desktop sync pass.
#[derive(Debug, Default)]
pub struct SyncReport {
    pub added: usize,
    pub removed: usize,
    pub updated: usize,
}

/// One move the rule engine would make (see [`AppState::plan_rules`]).
#[derive(Clone, Debug, PartialEq)]
pub struct PlannedRuleMove {
    pub item: ItemId,
    /// The fence the item is in now (`None`: not a member of any fence yet).
    pub from: Option<FenceId>,
    pub to: FenceId,
    pub by: AssignedBy,
}

impl SyncReport {
    pub fn changed(&self) -> bool {
        self.added + self.removed + self.updated > 0
    }
}

fn config_dir(portable: bool) -> PathBuf {
    if portable {
        let exe = std::env::current_exe().unwrap_or_default();
        return exe
            .parent()
            .map(|p| p.join("config"))
            .unwrap_or_else(|| PathBuf::from("config"));
    }
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("PecoFence")
}

/// `path` as the file system spells it (real casing, symlinks and junctions resolved), without
/// the `\\?\` prefix canonicalize adds. A network share or a mapped drive comes back as
/// `\\?\UNC\server\share\…`, which must turn into `\\server\share\…`, not a relative path.
/// `path` itself when it cannot be resolved.
pub fn canonical_path(path: PathBuf) -> PathBuf {
    match std::fs::canonicalize(&path) {
        Ok(real) => {
            let s = real.to_string_lossy();
            match s.strip_prefix(r"\\?\UNC\") {
                Some(unc) => PathBuf::from(format!(r"\\{unc}")),
                None => PathBuf::from(s.strip_prefix(r"\\?\").unwrap_or(&s)),
            }
        }
        Err(_) => path,
    }
}

/// Whether `path` really lives in the Store package's private copy of %APPDATA% (see
/// `AppState::config_path_on_disk`).
fn in_package_folder(path: &Path) -> bool {
    std::fs::canonicalize(path).is_ok_and(|real| real.to_string_lossy().contains(r"\Packages\"))
}

impl AppState {
    pub fn load(work_areas: Vec<WorkArea>, portable: bool) -> Self {
        let directory = config_dir(portable);
        let store = if portable {
            ConfigStore::new(&directory)
        } else {
            ConfigStore::with_legacy(
                &directory,
                directory.with_file_name(pecofence_core::brand::LEGACY_DATA_DIR),
            )
        };
        if store.dir() != directory {
            tracing::info!(path = %store.dir().display(), "reusing pre-rename configuration");
        }
        #[cfg_attr(not(debug_assertions), allow(unused_mut))]
        let (outcome, mut report) = store.load_reporting();
        // A test instance stands in for a PC where Windows refuses the saved files (os error
        // 409 needs the Store edition on a second drive).
        #[cfg(debug_assertions)]
        if let Ok(code) = pecofence_core::brand::var("PECOFENCE_TEST_READ_ERROR")
            && let Ok(code) = code.parse()
        {
            let error = std::io::Error::from_raw_os_error(code);
            let error = pecofence_core::config_store::ReadError::Io(error);
            report
                .failures
                .push(("config.json", store.primary_path(), error));
        }
        let (config, first_run, recovered_from) = match outcome {
            LoadOutcome::Primary(c) => (c, false, None),
            LoadOutcome::Recovered(c, from) => (c, false, Some(from)),
            LoadOutcome::Fresh(c) => (c, true, None),
        };
        pecofence_core::i18n::set_language(
            config
                .settings
                .language
                .resolve(pecofence_platform::locale::ui_language()),
        );
        let mut state = Self {
            config,
            store,
            layout: 0,
            dirty: false,
            catalog: HashMap::new(),
            portal_items: HashMap::new(),
            portal_members: HashMap::new(),
            portal_cwd: HashMap::new(),
            opened_folders: HashMap::new(),
            portal_missing: HashSet::new(),
            tmp_settle_at: std::cell::Cell::new(None),
            work_areas,
            unplaced: Vec::new(),
            first_run,
            recovered_from,
            unreadable_moved_to: report.moved_to,
            load_failures: report.failures,
        };
        state.rebuild_catalog();
        if state.config.drop_duplicates() {
            tracing::warn!("duplicate fences or memberships in the configuration dropped");
            state.dirty = true;
        }
        if pecofence_core::config_store::clamp_sizes(&mut state.config) {
            tracing::warn!("fence sizes out of range in the configuration clamped");
            state.dirty = true;
        }
        if pecofence_core::config_store::repair_unc_portals(&mut state.config) {
            tracing::info!("network portal paths saved by 0.2.1 repaired");
            state.dirty = true;
        }
        state.ensure_layout();
        state.normalize_tabs();
        state
    }

    pub fn config_path(&self) -> PathBuf {
        self.store.primary_path()
    }

    /// The folder custom themes are read from (see `pecofence_render::CustomTheme`).
    pub fn themes_dir(&self) -> PathBuf {
        self.store.dir().join("themes")
    }

    /// `config_path` where Explorer can find it. The Store package's writes under %APPDATA%
    /// land in its `LocalCache\Roaming`, so only this process sees the file at the %APPDATA%
    /// path; the final path of the opened file names the real one.
    pub fn config_path_on_disk(&self) -> PathBuf {
        canonical_path(self.config_path())
    }

    /// Where the configuration lives, as a fixed name for the log and feedback: the portable
    /// folder next to the program, %APPDATA% (under the old OpenFence name when that is
    /// reused), or the Store package's private copy of %APPDATA%.
    pub fn storage_kind(&self) -> &'static str {
        let dir = self.store.dir();
        if dir == config_dir(true) {
            return "portable";
        }
        let in_package = in_package_folder(&self.store.primary_path()) || in_package_folder(dir);
        let old_name = dir
            .file_name()
            .is_some_and(|n| n == std::ffi::OsStr::new(pecofence_core::brand::LEGACY_DATA_DIR));
        match (in_package, old_name) {
            (false, false) => "appdata",
            (false, true) => "appdata, old folder name",
            (true, false) => "store package",
            (true, true) => "store package, old folder name",
        }
    }

    pub fn backup_files(&self) -> Vec<PathBuf> {
        let mut v = self.store.list_backups();
        v.sort();
        v.reverse();
        v
    }

    /// Replaces the whole configuration (import / backup restore) and re-derives the runtime
    /// state. Fence windows must be resynced by the caller. (Snapshot restore goes through
    /// `restore_snapshot`, which only swaps layouts.)
    pub fn replace_config(&mut self, mut config: Config) {
        // Keep the current item table when the file has none for this desktop (a snapshot
        // only carries layouts); memberships reference item ids either way.
        if config.items.is_empty() {
            config.items = std::mem::take(&mut self.config.items);
        }
        // Snapshots are local history, including the "…前" undo snapshot the caller just took:
        // never let an import / backup file wipe them. The file's own (by id) fill the free
        // slots, newest first; the list stays oldest first.
        let mut snaps = std::mem::take(&mut self.config.snapshots);
        let mut incoming: Vec<_> = std::mem::take(&mut config.snapshots)
            .into_iter()
            .filter(|s| !snaps.iter().any(|x| x.id == s.id))
            .collect();
        incoming.sort_by_key(|s| std::cmp::Reverse(s.ts));
        incoming.truncate(pecofence_core::MAX_SNAPSHOTS.saturating_sub(snaps.len()));
        snaps.extend(incoming);
        snaps.sort_by_key(|s| s.ts);
        while snaps.len() > pecofence_core::MAX_SNAPSHOTS {
            snaps.remove(0);
        }
        config.snapshots = snaps;
        config.drop_duplicates();
        pecofence_core::config_store::clamp_sizes(&mut config);
        pecofence_core::config_store::repair_unc_portals(&mut config);
        self.config = config;
        self.rebuild_catalog();
        self.layout = 0;
        self.ensure_layout();
        self.normalize_tabs();
        self.portal_cwd.clear();
        self.opened_folders.clear();
        self.dirty = true;
    }

    // ---- snapshots ---------------------------------------------------------------------------

    pub fn save_snapshot(&mut self, name: &str) -> uuid::Uuid {
        let name = name.trim();
        let name = if name.is_empty() {
            pecofence_core::i18n::format(
                "快照 {0}",
                &[format!("{}", self.config.snapshots.len() + 1)],
            )
        } else {
            name.to_string()
        };
        let snap = pecofence_core::Snapshot {
            id: uuid::Uuid::new_v4(),
            name,
            ts: pecofence_core::now_unix(),
            layouts: self.config.layouts.clone(),
        };
        let id = snap.id;
        self.config.snapshots.push(snap);
        while self.config.snapshots.len() > pecofence_core::MAX_SNAPSHOTS {
            self.config.snapshots.remove(0);
        }
        self.dirty = true;
        id
    }

    /// Restores a snapshot's layouts (settings, rules and items stay as they are) after saving
    /// a `backup_name` snapshot of the current ones. The target is looked up first, so the
    /// backup's eviction at `MAX_SNAPSHOTS` can never remove it. Returns the backup's id, or
    /// None (and saves nothing) when `id` is unknown.
    pub fn restore_snapshot_with_backup(
        &mut self,
        id: uuid::Uuid,
        backup_name: &str,
    ) -> Option<uuid::Uuid> {
        let snap = self.config.snapshots.iter().find(|s| s.id == id).cloned()?;
        let backup = self.save_snapshot(backup_name);
        self.apply_snapshot_layouts(snap.layouts);
        Some(backup)
    }

    /// Restores a snapshot's layouts without taking a backup first. False when `id` is unknown.
    pub fn restore_snapshot(&mut self, id: uuid::Uuid) -> bool {
        let Some(snap) = self.config.snapshots.iter().find(|s| s.id == id).cloned() else {
            return false;
        };
        self.apply_snapshot_layouts(snap.layouts);
        true
    }

    fn apply_snapshot_layouts(&mut self, layouts: Vec<pecofence_core::Layout>) {
        self.config.layouts = layouts;
        self.layout = 0;
        self.ensure_layout();
        self.normalize_tabs();
        self.portal_cwd.clear();
        self.opened_folders.clear();
        self.dirty = true;
    }

    pub fn delete_snapshot(&mut self, id: uuid::Uuid) -> bool {
        let before = self.config.snapshots.len();
        self.config.snapshots.retain(|s| s.id != id);
        let removed = self.config.snapshots.len() != before;
        if removed {
            self.dirty = true;
        }
        removed
    }

    /// Fences "swap screen contents": every fence on monitor `a` moves to `b` and vice versa,
    /// keeping its anchored edge gaps (a right-anchored fence stays 16 DIP from the right edge
    /// of the other monitor, whatever its width).
    pub fn swap_monitors(&mut self, a: &str, b: &str) -> usize {
        if a == b {
            return 0;
        }
        let layout = self.layout;
        let areas = self.work_areas.clone();
        let mut n = 0;
        for f in &mut self.config.layouts[layout].fences {
            let to = if f.geometry.monitor == a {
                b
            } else if f.geometry.monitor == b {
                a
            } else {
                continue;
            };
            // Map onto the other work area first (denormalize keeps the anchored gaps against
            // the *stored* work_w/work_h), then store the geometry re-normalised there so x/y
            // and work_w/work_h stay consistent — a fence hosted as a tab has no window to do
            // this through FenceBoundsChanged.
            match areas.iter().find(|w| w.device_path == to) {
                Some(w) => {
                    let px = geometry::denormalize(&f.geometry, w);
                    f.geometry = geometry::normalize(px, w);
                }
                None => f.geometry.monitor = to.to_string(),
            }
            n += 1;
        }
        if n > 0 {
            self.dirty = true;
        }
        n
    }

    /// Desktop folder moved (OneDrive redirect, another drive): re-point every item record
    /// under the old path to the new one so fence memberships survive. Returns how many.
    pub fn migrate_desktop_path(&mut self, new_desktop: &Path) -> usize {
        let new_s = new_desktop.to_string_lossy().to_string();
        let old = match &self.config.settings.desktop_path {
            Some(o) => o.clone(),
            None => {
                self.config.settings.desktop_path = Some(new_s);
                self.dirty = true;
                return 0;
            }
        };
        let old_key = ItemKey::from_path(&old);
        let new_key = ItemKey::from_path(&new_s);
        if old_key == new_key {
            return 0;
        }
        let Some(old_prefix) = old_key.as_path().map(dir_prefix) else {
            return 0;
        };
        let Some(new_prefix) = new_key.as_path().map(dir_prefix) else {
            return 0;
        };
        let mut n = 0;
        for item in self.config.items.values_mut() {
            if let Some(path) = item.key.as_path()
                && let Some(rest) = path.strip_prefix(old_prefix.as_str())
            {
                let rest = rest.to_string();
                let moved = format!("{new_prefix}{rest}");
                item.key = ItemKey::Path(moved.clone());
                // The next desktop sync brings the new spelling.
                item.path = None;
                if let IconKey::ByContent { path, .. } = &mut item.icon_key {
                    *path = moved;
                }
                n += 1;
            }
        }
        self.config.settings.desktop_path = Some(new_s);
        self.rebuild_catalog();
        self.dirty = true;
        n
    }

    fn rebuild_catalog(&mut self) {
        self.catalog = self
            .config
            .items
            .iter()
            .map(|(id, it)| (it.key.clone(), *id))
            .collect();
    }

    /// Selects the layout for the current monitors, creating it on first use (the wizard
    /// default on the very first), and shows the fences there as the layout shown last has
    /// them: every monitor set shows the same fences, a layout keeps where they sit. Fences
    /// with no spot on these monitors yet are left in [`Self::unplaced`].
    pub fn ensure_layout(&mut self) {
        let found = self.config.layout_for(&self.work_areas);
        let target = match &found {
            Some(m) => {
                self.update_monitors(m);
                m.index
            }
            None => {
                let fences = if self.config.layouts.is_empty() {
                    self.default_fences()
                } else {
                    Vec::new()
                };
                let fingerprint = self.work_areas.iter().map(MonitorIdentity::of).collect();
                self.config.layouts.push(Layout {
                    fingerprint,
                    fences,
                    shown: 0,
                });
                self.dirty = true;
                self.config.layouts.len() - 1
            }
        };
        let source = self.config.last_shown().unwrap_or_else(|| {
            // Files from 0.1.3 and earlier kept separate fences per monitor set: the layout of
            // these monitors (or the first) takes in the others'.
            let base = if found.is_some() { target } else { 0 };
            if self.config.layouts.len() > 1 {
                self.config.merge_layout_fences(base);
            }
            base
        });
        let mut unplaced = if source == target {
            // Import / snapshot restore run this twice before the App settles them: keep the
            // fences the first pass found no spot for.
            std::mem::take(&mut self.unplaced)
        } else {
            self.config.carry_fences(source, target)
        };
        // A spot on a monitor that is not connected is no spot (older versions cloned a whole
        // layout onto a new monitor set).
        for f in &self.config.layouts[target].fences {
            let connected = self
                .work_areas
                .iter()
                .any(|w| w.device_path == f.geometry.monitor);
            if !connected && f.tab_host.is_none() && !unplaced.contains(&f.id) {
                unplaced.push(f.id);
            }
        }
        self.unplaced = unplaced;
        if self.config.mark_shown(target) || source != target {
            self.dirty = true;
        }
        self.layout = target;
    }

    /// The matched layout takes the connected monitors as they are: one found under another id
    /// (another port, or a file that saved GDI names) takes the new id there and in its fences'
    /// geometry, and a layout saved without resolutions takes the current ones.
    fn update_monitors(&mut self, found: &pecofence_core::LayoutMatch) {
        let layout = &mut self.config.layouts[found.index];
        for (saved, &j) in layout.fingerprint.iter_mut().zip(&found.pairs) {
            let now = MonitorIdentity::of(&self.work_areas[j]);
            let differs = saved.device_path != now.device_path
                || saved.gdi_name != now.gdi_name
                || saved.screen_px != now.screen_px
                || saved.dpi != now.dpi;
            if differs {
                *saved = now;
                self.dirty = true;
            }
        }
        let renamed = &found.renamed;
        if renamed.is_empty() {
            return;
        }
        for f in &mut layout.fences {
            if let Some((_, new)) = renamed.iter().find(|(old, _)| *old == f.geometry.monitor) {
                f.geometry.monitor = new.clone();
            }
        }
        tracing::info!(?renamed, "monitors found under another id");
    }

    /// Stardock-style first-run layout: right column 程序 / 文件夹 / 文件与文档 / 桌面(inbox).
    fn default_fences(&mut self) -> Vec<Fence> {
        let work = self.work_areas.first().cloned().unwrap_or(WorkArea {
            device_path: "primary".into(),
            gdi_name: String::new(),
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1040,
            dpi: 96,
            mon_left: 0,
            mon_top: 0,
            mon_right: 1920,
            mon_bottom: 1040,
        });
        let ww = work.width_dip();
        let wh = work.height_dip();
        // Exactly three icon columns at the default icon size, so the column snap applied by
        // sync_fence_windows is a no-op and the right edge stays `gap` inside the work area.
        let icon = self.config.settings.icon_size;
        let col_w = crate::layout::GridMetrics::for_icon_size(
            icon,
            pecofence_core::FenceView::default().label_lines,
        )
        .width_for_columns(3);
        let gap = 16.0f32;
        let x = ww - gap - col_w;
        let heights = [240.0f32, 200.0, 240.0];
        let titles = [
            pecofence_core::i18n::text("程序"),
            pecofence_core::i18n::text("文件夹"),
            pecofence_core::i18n::text("文件与文档"),
        ];
        let mut y = gap;
        let mut fences = Vec::new();
        let mut ids = Vec::new();
        for (title, h) in titles.iter().zip(heights) {
            let h = h.min((wh - gap * 2.0) / 4.0).max(120.0);
            let geo = geometry::normalize(
                PxRect {
                    left: work.left + (x * work.scale()) as i32,
                    top: work.top + (y * work.scale()) as i32,
                    right: work.left + ((x + col_w) * work.scale()) as i32,
                    bottom: work.top + ((y + h) * work.scale()) as i32,
                },
                &work,
            );
            let mut f = Fence::new(title, FenceKind::Virtual, geo);
            f.view.icon_size = icon;
            ids.push(f.id);
            fences.push(f);
            y += h + gap;
        }
        let inbox_h = (wh - gap - y).max(160.0);
        let inbox_geo = geometry::normalize(
            PxRect {
                left: work.left + (x * work.scale()) as i32,
                top: work.top + (y * work.scale()) as i32,
                right: work.left + ((x + col_w) * work.scale()) as i32,
                bottom: work.top + ((y + inbox_h) * work.scale()) as i32,
            },
            &work,
        );
        let mut inbox = Fence::new(
            pecofence_core::i18n::text("桌面"),
            FenceKind::Inbox,
            inbox_geo,
        );
        inbox.view.icon_size = icon;
        fences.push(inbox);
        self.config.rules = RuleSet::default_presets(ids[0], ids[1], ids[2]);
        fences
    }

    pub fn fences(&self) -> &[Fence] {
        &self.config.layouts[self.layout].fences
    }

    fn layout_ref(&self) -> &Layout {
        &self.config.layouts[self.layout]
    }

    // ---- tabbed fences ----------------------------------------------------------------------

    /// The window (host fence) a fence is shown in.
    pub fn host_of(&self, id: FenceId) -> FenceId {
        self.layout_ref().host_of(id)
    }

    /// Fences that own a window (not hosted as tabs), in layout order.
    pub fn host_fences(&self) -> Vec<Fence> {
        self.fences()
            .iter()
            .filter(|f| f.tab_host.is_none())
            .cloned()
            .collect()
    }

    pub fn tabs_of(&self, host: FenceId) -> Vec<FenceId> {
        self.layout_ref().tabs_of(host)
    }

    pub fn active_tab_of(&self, host: FenceId) -> FenceId {
        self.layout_ref().active_tab_of(host)
    }

    /// Repairs tab links (call after load / layout switch / delete). Returns true if changed.
    pub fn normalize_tabs(&mut self) -> bool {
        let layout = self.layout;
        let changed = self.config.layouts[layout].normalize_tabs();
        if changed {
            self.dirty = true;
        }
        changed
    }

    /// Makes `fence` a tab of `host`'s window (and moves any tabs `fence` hosted along). No-op
    /// when the two are the same window already.
    pub fn attach_tab(&mut self, fence: FenceId, host: FenceId) -> bool {
        let host = self.host_of(host);
        if fence == host || self.fence(fence).is_none() || self.fence(host).is_none() {
            return false;
        }
        let layout = self.layout;
        for f in &mut self.config.layouts[layout].fences {
            if f.id == fence {
                f.tab_host = Some(host);
                f.active_tab = None;
            } else if f.tab_host == Some(fence) {
                f.tab_host = Some(host);
            }
        }
        if let Some(h) = self.fence_mut(host) {
            h.active_tab = Some(fence);
        }
        self.normalize_tabs();
        self.dirty = true;
        true
    }

    pub fn detach_tab(&mut self, fence: FenceId) -> Option<pecofence_core::TabDetach> {
        let change = self.config.layouts[self.layout].detach_tab(fence)?;
        self.dirty = true;
        Some(change)
    }

    pub fn cancel_tab_detach(&mut self, change: &pecofence_core::TabDetach) -> bool {
        let restored = self.config.layouts[self.layout].cancel_tab_detach(change);
        self.dirty |= restored;
        restored
    }

    /// Places `tab` at strip index `to` in `host`'s window (drag along the strip / 左移 右移).
    pub fn reorder_tab(&mut self, host: FenceId, tab: FenceId, to: usize) -> bool {
        let host = self.host_of(host);
        let layout = self.layout;
        let changed = self.config.layouts[layout].reorder_tab(host, tab, to);
        if changed {
            self.dirty = true;
        }
        changed
    }

    pub fn set_active_tab(&mut self, host: FenceId, tab: FenceId) {
        let host = self.host_of(host);
        let value = (tab != host).then_some(tab);
        if let Some(h) = self.fence_mut(host)
            && h.active_tab != value
        {
            h.active_tab = value;
            self.dirty = true;
        }
    }

    pub fn fence(&self, id: FenceId) -> Option<&Fence> {
        self.fences().iter().find(|f| f.id == id)
    }

    pub fn fence_mut(&mut self, id: FenceId) -> Option<&mut Fence> {
        let layout = self.layout;
        self.config.fence_mut(layout, id)
    }

    pub fn inbox_id(&self) -> Option<FenceId> {
        self.fences()
            .iter()
            .find(|f| f.kind == FenceKind::Inbox)
            .map(|f| f.id)
    }

    /// The fences kept without a window: 为空时自动隐藏 is on and they show no items (a
    /// portal only at its root folder and a regular fence not while it shows a folder opened in
    /// it, so opening an empty folder keeps it), and the inbox while 显示 Windows 桌面图标时隐藏
    /// “桌面”栅栏 is on and the Windows icons show. Only a window of its own hides (a tab, or a
    /// host of tabs, stays).
    pub fn hidden_fences(&self) -> Vec<FenceId> {
        let settings = &self.config.settings;
        let for_windows_icons = settings.hide_inbox_when_icons_shown && !settings.hide_real_icons;
        self.fences()
            .iter()
            .filter(|f| {
                let empty = f.hide_when_empty
                    && !self.can_go_up(f.id)
                    && self.unsorted_held_items_of(f).next().is_none();
                (empty || (for_windows_icons && f.kind == FenceKind::Inbox))
                    && f.tab_host.is_none()
                    && self.tabs_of(f.id).len() <= 1
            })
            .map(|f| f.id)
            .collect()
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Icons `host`'s window can switch to without waiting: the items of its tabs other than
    /// `shown` (the window reports those itself), each with that tab's icon edge in pixels on
    /// a monitor of `dpi`. Orphaned records show nowhere, so their icons are not wanted.
    pub fn tab_icons(&self, host: FenceId, shown: FenceId, dpi: u32) -> Vec<(&IconKey, u32)> {
        self.tabs_of(host)
            .into_iter()
            .filter(|&tab| tab != shown)
            .filter_map(|tab| self.fence(tab))
            .flat_map(|f| {
                let px = crate::layout::icon_px(f.view.layout, f.view.icon_size, dpi);
                self.items_of(f)
                    .into_iter()
                    .map(move |item| (&item.icon_key, px))
            })
            .collect()
    }

    pub fn save_if_dirty(&mut self) -> bool {
        if !self.dirty {
            return false;
        }
        match self.store.save(&self.config) {
            Ok(()) => {
                self.dirty = false;
                tracing::debug!(path = %self.store.primary_path().display(), "config saved");
                true
            }
            Err(e) => {
                // Where config.json and its temp file really are: one inside the Store
                // package's copy of %APPDATA% and one outside would explain a refused rename.
                tracing::error!(
                    what = e.step,
                    error = %e.error,
                    config_in_package = in_package_folder(&self.store.primary_path()),
                    temp_in_package = in_package_folder(&self.store.tmp_path()),
                    "config save failed"
                );
                false
            }
        }
    }

    // ---- desktop items ---------------------------------------------------------------------

    /// Rules only run automatically while "新项目出现在桌面时自动归类" is on; otherwise new
    /// items go to the inbox until the user applies the rules by hand.
    fn route_decision(&self, entry: &DesktopEntry) -> Decision {
        // The Recycle Bin and friends are not files: they always live in the inbox ("桌面").
        if entry.origin == EntryOrigin::Namespace {
            return Decision::Default(Target::Inbox);
        }
        if self.config.rules.keep_updated {
            let facts = self.facts_for(entry);
            let decision = self.config.rules.evaluate(&facts);
            if decision == Decision::Skip
                && facts.file_name.to_lowercase().ends_with(".tmp")
                && let Some(age) = facts.age_secs
                && age < pecofence_core::rules::TMP_SETTLE_SECS
            {
                let at = pecofence_core::now_unix()
                    + (pecofence_core::rules::TMP_SETTLE_SECS - age) as i64;
                let soonest = self.tmp_settle_at.get().map_or(at, |t| t.min(at));
                self.tmp_settle_at.set(Some(soonest));
            }
            decision
        } else {
            Decision::Default(Target::Inbox)
        }
    }

    /// `id` if it is a fence that can hold routed desktop items. A folder portal never shows
    /// `fence.items` (its content comes from the folder), so routing into one would make the
    /// item vanish from every fence.
    pub fn routable_fence(&self, id: FenceId) -> Option<FenceId> {
        self.fence(id)
            .filter(|f| f.kind != FenceKind::FolderPortal)
            .map(|f| f.id)
    }

    fn target_fence(&self, decision: Decision) -> Option<(FenceId, AssignedBy)> {
        let inbox = self.inbox_id();
        match decision {
            Decision::Skip => None,
            Decision::Route { target, rule } => {
                let fence = match target {
                    Target::Inbox => inbox?,
                    Target::Fence(id) if self.routable_fence(id).is_some() => id,
                    Target::Fence(_) => inbox?,
                };
                Some((fence, AssignedBy::Rule(rule)))
            }
            Decision::Default(target) => {
                let fence = match target {
                    Target::Inbox => inbox?,
                    Target::Fence(id) if self.routable_fence(id).is_some() => id,
                    Target::Fence(_) => inbox?,
                };
                Some((fence, AssignedBy::Migration))
            }
        }
    }

    fn facts_for(&self, entry: &DesktopEntry) -> pecofence_core::ItemFacts {
        let last_opened = self
            .catalog
            .get(&ItemKey::from_path(&entry.path.to_string_lossy()))
            .and_then(|id| self.config.items.get(id))
            .and_then(|it| it.last_opened);
        Self::facts_at(entry, last_opened, pecofence_core::now_unix())
    }

    /// Whole days between `now` and the later of last write and last open; `None` when the
    /// file reports no time at all.
    fn idle_days(mtime: i64, last_opened: Option<i64>, now: i64) -> Option<u32> {
        let last = mtime.max(last_opened.unwrap_or(0));
        (last > 0).then(|| ((now - last).max(0) / 86_400) as u32)
    }

    fn facts_at(
        entry: &DesktopEntry,
        last_opened: Option<i64>,
        now: i64,
    ) -> pecofence_core::ItemFacts {
        let lower = entry.file_name.to_lowercase();
        let (shortcut_target, shortcut_target_ext) = if lower.ends_with(".lnk") {
            let t = shell::shortcut_target(&entry.path);
            let ext = t.as_deref().map(pecofence_core::rules::ext_of);
            (t, ext)
        } else if lower.ends_with(".url") {
            (shell::url_shortcut_target(&entry.path), Some(".url".into()))
        } else {
            (None, None)
        };
        pecofence_core::ItemFacts {
            file_name: entry.file_name.clone(),
            is_folder: entry.is_folder,
            size_bytes: entry.size,
            shortcut_target,
            shortcut_target_ext,
            created_minutes_local: pecofence_platform::fileinfo::local_time_parts(entry.created)
                .map(|(m, _)| m),
            created_weekday: pecofence_platform::fileinfo::local_time_parts(entry.created)
                .map(|(_, d)| d),
            origin: match entry.origin {
                EntryOrigin::UserDesktop => Origin::UserDesktop,
                EntryOrigin::PublicDesktop => Origin::PublicDesktop,
                EntryOrigin::Namespace => Origin::Namespace,
            },
            is_hidden: false,
            is_system: false,
            idle_days: Self::idle_days(entry.mtime, last_opened, now),
            age_secs: Self::age_secs(entry.created, entry.mtime, now),
        }
    }

    /// Seconds since the later of creation and last write; `None` when the file reports no
    /// time at all.
    fn age_secs(created: i64, mtime: i64, now: i64) -> Option<u64> {
        let last = created.max(mtime);
        (last > 0).then(|| (now - last).max(0) as u64)
    }

    /// When a `.tmp` skipped as too new will have lasted long enough to be routed (Unix
    /// seconds), once; see `tmp_settle_at`.
    pub fn take_tmp_settle_at(&self) -> Option<i64> {
        self.tmp_settle_at.take()
    }

    /// Reconciles the item table with the current desktop contents. New items are routed by the
    /// rule set; missing items are marked orphaned (membership kept) and garbage-collected
    /// after seven days.
    pub fn sync_desktop(&mut self, entries: &[DesktopEntry]) -> SyncReport {
        self.sync_desktop_partly_read(entries, &[])
    }

    /// [`Self::sync_desktop`] with a listing that could not read the `unread` desktop folders
    /// in full: their items missing from it are not orphaned.
    pub fn sync_desktop_partly_read(
        &mut self,
        entries: &[DesktopEntry],
        unread: &[EntryOrigin],
    ) -> SyncReport {
        let mut report = SyncReport::default();
        let now = pecofence_core::now_unix();
        let mut seen: HashSet<ItemId> = HashSet::new();
        // One pass over the fences instead of a search per entry (quadratic on a crowded
        // desktop).
        let in_fence: HashSet<ItemId> = self
            .fences()
            .iter()
            .flat_map(|f| f.items.iter().map(|r| r.item_id))
            .collect();
        // Naming new entries one at a time costs several ms each on a OneDrive desktop: a batch
        // of them (first run, an unpacked archive) is named per folder in one go, as
        // `refresh_portal` does.
        let mut new_dirs: Vec<&Path> = Vec::new();
        let mut new_count = 0usize;
        for entry in entries {
            if entry.origin == EntryOrigin::Namespace
                || self
                    .catalog
                    .contains_key(&ItemKey::from_path(&entry.path.to_string_lossy()))
            {
                continue;
            }
            new_count += 1;
            if let Some(d) = entry.path.parent()
                && !new_dirs.contains(&d)
            {
                new_dirs.push(d);
            }
        }
        let listed: HashMap<&Path, HashMap<String, String>> = if new_count > 2 {
            new_dirs
                .iter()
                .map(|d| (*d, shell::display_names_in(d)))
                .collect()
        } else {
            HashMap::new()
        };

        for entry in entries {
            let key = ItemKey::from_path(&entry.path.to_string_lossy());
            let (icon_key, _) =
                crate::icons::icon_key_for(&entry.path, entry.is_folder, entry.mtime);
            if let Some(&id) = self.catalog.get(&key) {
                seen.insert(id);
                if let Some(item) = self.config.items.get_mut(&id) {
                    let mut changed = false;
                    if item.orphaned_since.is_some() {
                        item.orphaned_since = None;
                        changed = true;
                    }
                    if item.mtime != entry.mtime || item.icon_key != icon_key {
                        item.mtime = entry.mtime;
                        item.icon_key = icon_key;
                        changed = true;
                    }
                    if item.size != entry.size {
                        item.size = entry.size;
                        changed = true;
                    }
                    // A deleted file's record is reused by a folder of the same name (or the
                    // other way round) while it is still catalogued.
                    if item.is_folder != entry.is_folder {
                        item.is_folder = entry.is_folder;
                        changed = true;
                    }
                    // The key ignores case: a case-only rename (or a file from an older
                    // version) brings the on-disk spelling here.
                    let path = entry.path.to_string_lossy();
                    if item.path.as_deref() != Some(&*path) {
                        item.path = Some(path.into_owned());
                        changed = true;
                    }
                    if changed {
                        report.updated += 1;
                        self.dirty = true;
                    }
                }
                // An item that exists but is in no fence (e.g. its fence was deleted, or a .tmp
                // skipped while fresh) → route. Counted, so the caller refreshes the windows.
                if !in_fence.contains(&id) {
                    let decision = self.route_decision(entry);
                    if let Some((fence, by)) = self.target_fence(decision)
                        && self.config.assign(self.layout, id, fence, by).is_some()
                    {
                        report.updated += 1;
                        self.dirty = true;
                    }
                }
                continue;
            }
            // New item.
            let display_name = entry
                .path
                .parent()
                .and_then(|d| listed.get(d))
                .and_then(|names| names.get(&entry.file_name.to_lowercase()))
                .cloned()
                .or_else(|| shell::display_name(&entry.path))
                .unwrap_or_else(|| {
                    let stem = Path::new(&entry.file_name)
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| entry.file_name.clone());
                    if entry.is_folder {
                        entry.file_name.clone()
                    } else {
                        stem
                    }
                });
            let item = Item {
                id: uuid::Uuid::new_v4(),
                key: key.clone(),
                origin: match entry.origin {
                    EntryOrigin::UserDesktop => Origin::UserDesktop,
                    EntryOrigin::PublicDesktop => Origin::PublicDesktop,
                    EntryOrigin::Namespace => Origin::Namespace,
                },
                display_name,
                file_id: None,
                mtime: entry.mtime,
                is_folder: entry.is_folder,
                attrs: entry.attributes,
                icon_key,
                orphaned_since: None,
                size: entry.size,
                open_count: 0,
                last_opened: None,
                path: Some(entry.path.to_string_lossy().into_owned()),
            };
            let id = item.id;
            self.config.items.insert(id, item);
            self.catalog.insert(key, id);
            seen.insert(id);
            let decision = self.route_decision(entry);
            if let Some((fence, by)) = self.target_fence(decision) {
                self.config.assign(self.layout, id, fence, by);
            }
            report.added += 1;
            self.dirty = true;
        }

        // Orphans.
        let unread: Vec<Origin> = unread
            .iter()
            .map(|o| match o {
                EntryOrigin::UserDesktop => Origin::UserDesktop,
                EntryOrigin::PublicDesktop => Origin::PublicDesktop,
                EntryOrigin::Namespace => Origin::Namespace,
            })
            .collect();
        let mut to_remove = Vec::new();
        for (id, item) in self.config.items.iter_mut() {
            if seen.contains(id) || unread.contains(&item.origin) {
                continue;
            }
            match item.orphaned_since {
                None => {
                    item.orphaned_since = Some(now);
                    report.removed += 1;
                    self.dirty = true;
                }
                Some(since) if now - since > ORPHAN_GC_SECS => to_remove.push(*id),
                Some(_) => {}
            }
        }
        for id in to_remove {
            self.remove_item_record(id);
        }
        report
    }

    /// Removes a desktop item's record from the table, the catalog and the fences.
    fn remove_item_record(&mut self, id: ItemId) {
        if let Some(item) = self.config.items.remove(&id) {
            self.catalog.remove(&item.key);
        }
        for f in &mut self.config.layouts[self.layout].fences {
            f.items.retain(|r| r.item_id != id);
        }
        self.dirty = true;
    }

    /// Before the user renames an item to `path`: a file of that name deleted less than seven
    /// days ago still has its (orphaned) record, which would take the renamed file for the old
    /// one at the next sync and move it into the old file's fence. Drop that record.
    pub fn forget_orphan_at(&mut self, path: &Path) {
        let key = ItemKey::from_path(&path.to_string_lossy());
        if let Some(&id) = self.catalog.get(&key)
            && self
                .config
                .items
                .get(&id)
                .is_some_and(|it| it.orphaned_since.is_some())
        {
            self.remove_item_record(id);
        }
    }

    /// Live workspace entries, including the current contents of folder portals.
    /// The same file mirrored by several fences is counted once.
    pub fn workspace_item_count(&self) -> usize {
        self.config
            .items
            .values()
            .chain(self.portal_items.values())
            .filter(|item| item.orphaned_since.is_none())
            .map(|item| &item.key)
            .collect::<HashSet<_>>()
            .len()
    }

    /// Items of `fence` with their refs, as stored (orphans hidden), before any sort: what its
    /// window shows, or with `held` what it holds (a regular fence showing a folder opened in it
    /// still holds its own icons).
    fn fence_members<'s, 'f>(
        &'s self,
        fence: &'f Fence,
        held: bool,
    ) -> Vec<(Option<&'f ItemRef>, &'s Item)> {
        let folder = if held {
            self.portal_root(fence.id)
        } else {
            self.folder_root(fence.id)
        };
        if folder.is_some() {
            self.portal_members
                .get(&fence.id)
                .map(|ids| {
                    ids.iter()
                        .filter_map(|i| self.portal_items.get(i).map(|it| (None, it)))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            self.own_members(fence).collect()
        }
    }

    /// The desktop items assigned to `fence` (orphans hidden), whatever it shows right now.
    fn own_members<'s, 'f>(
        &'s self,
        fence: &'f Fence,
    ) -> impl Iterator<Item = (Option<&'f ItemRef>, &'s Item)> {
        fence
            .items
            .iter()
            .filter_map(|r| self.config.items.get(&r.item_id).map(|it| (Some(r), it)))
            .filter(|(_, it)| it.orphaned_since.is_none())
    }

    /// How many desktop items are assigned to `fence`: what deleting it sends back to the inbox
    /// (a fence showing a folder keeps its own icons meanwhile).
    pub fn own_item_count(&self, fence: &Fence) -> usize {
        self.own_members(fence).count()
    }

    /// What `fence` holds, in no particular order (orphans hidden): a portal's folder, any other
    /// fence's own icons, also while it shows a folder opened in it. The CLI's view, for callers
    /// that only need the set, without the sort `held_items_of` pays for.
    pub fn unsorted_held_items_of<'s>(&'s self, fence: &Fence) -> impl Iterator<Item = &'s Item> {
        self.fence_members(fence, true)
            .into_iter()
            .map(|(_, it)| it)
    }

    /// Items of `fence` in display order (orphans hidden), as its window shows them.
    pub fn items_of(&self, fence: &Fence) -> Vec<&Item> {
        self.sorted_members(fence, false)
    }

    /// [`Self::unsorted_held_items_of`] in display order.
    pub fn held_items_of(&self, fence: &Fence) -> Vec<&Item> {
        self.sorted_members(fence, true)
    }

    fn sorted_members(&self, fence: &Fence, held: bool) -> Vec<&Item> {
        let mut items = self.fence_members(fence, held);
        match fence.view.sort {
            SortMode::Manual => items.sort_by_key(|(r, it)| {
                (
                    r.and_then(|r| r.manual_index).unwrap_or(u32::MAX),
                    it.display_name.to_lowercase(),
                )
            }),
            SortMode::Name => {
                items.sort_by(|a, b| natural_cmp(&a.1.display_name, &b.1.display_name))
            }
            SortMode::Type => items.sort_by(|a, b| {
                let ka = (
                    !a.1.is_folder,
                    ext_of_key(&a.1.key),
                    a.1.display_name.to_lowercase(),
                );
                let kb = (
                    !b.1.is_folder,
                    ext_of_key(&b.1.key),
                    b.1.display_name.to_lowercase(),
                );
                ka.cmp(&kb)
            }),
            SortMode::Date => items.sort_by_key(|(_, it)| std::cmp::Reverse(it.mtime)),
            SortMode::Size => items.sort_by(|a, b| {
                // Folders first (Explorer), then largest first, ties by name.
                (!a.1.is_folder, std::cmp::Reverse(a.1.size))
                    .cmp(&(!b.1.is_folder, std::cmp::Reverse(b.1.size)))
                    .then_with(|| natural_cmp(&a.1.display_name, &b.1.display_name))
            }),
            SortMode::OpenCount => items.sort_by(|a, b| {
                std::cmp::Reverse(a.1.open_count)
                    .cmp(&std::cmp::Reverse(b.1.open_count))
                    .then_with(|| natural_cmp(&a.1.display_name, &b.1.display_name))
            }),
        }
        if fence.view.sort != SortMode::Manual {
            // Explorer keeps the Recycle Bin and friends ahead of files whatever the order.
            items.sort_by_key(|(_, it)| !it.is_namespace());
        }
        if fence.view.reverse {
            items.reverse();
        }
        items.into_iter().map(|(_, it)| it).collect()
    }

    /// Counts a launch (desktop items only; portal items are rebuilt from the folder).
    pub fn note_opened(&mut self, id: ItemId) {
        if let Some(it) = self.config.items.get_mut(&id) {
            it.open_count = it.open_count.saturating_add(1);
            it.last_opened = Some(pecofence_core::now_unix());
            self.dirty = true;
        }
    }

    pub fn item(&self, id: ItemId) -> Option<&Item> {
        self.config
            .items
            .get(&id)
            .or_else(|| self.portal_items.get(&id))
    }

    /// True for items that live in a folder portal rather than on the desktop.
    pub fn is_portal_item(&self, id: ItemId) -> bool {
        self.portal_items.contains_key(&id)
    }

    /// The folder a portal fence currently shows (its root, or the subfolder navigated into).
    /// Also the folder a fence shows after a folder was opened in it ([`Self::open_folder`]).
    pub fn portal_path(&self, id: FenceId) -> Option<PathBuf> {
        let root = self.folder_root(id)?;
        Some(self.portal_cwd.get(&id).cloned().unwrap_or(root))
    }

    /// The folder a fence navigates from: a folder opened in it, else a portal's root.
    /// Navigation never goes above it.
    fn folder_root(&self, id: FenceId) -> Option<PathBuf> {
        self.opened_folders
            .get(&id)
            .map(|s| s.root.clone())
            .or_else(|| self.portal_root(id))
    }

    /// The folder a portal was created for.
    pub fn portal_root(&self, id: FenceId) -> Option<PathBuf> {
        let f = self.fence(id)?;
        match (&f.kind, &f.source) {
            (FenceKind::FolderPortal, ItemSourceSpec::Folder { path, .. }) => {
                Some(PathBuf::from(path))
            }
            _ => None,
        }
    }

    /// True when the portal shows a subfolder of its root (an "up" is possible).
    pub fn portal_navigated(&self, id: FenceId) -> bool {
        self.portal_cwd.contains_key(&id)
    }

    /// True when the fence can go up: a portal inside a subfolder, or a regular fence showing a
    /// folder opened in it (up from there brings its own icons back).
    pub fn can_go_up(&self, id: FenceId) -> bool {
        self.portal_navigated(id) || self.opened_folders.contains_key(&id)
    }

    /// True when the portal's folder could not be found at the last read.
    pub fn portal_missing(&self, id: FenceId) -> bool {
        self.portal_missing.contains(&id)
    }

    /// Navigates a portal into `dir` (must be inside its root). Returns false if refused.
    pub fn portal_enter(&mut self, id: FenceId, dir: &Path) -> bool {
        let Some(root) = self.folder_root(id) else {
            return false;
        };
        if !dir.is_dir() {
            return false;
        }
        let root_key = ItemKey::from_path(&root.to_string_lossy());
        let dir_key = ItemKey::from_path(&dir.to_string_lossy());
        let (Some(rk), Some(dk)) = (root_key.as_path(), dir_key.as_path()) else {
            return false;
        };
        if dk == rk {
            self.portal_cwd.remove(&id);
        } else if dk.starts_with(&dir_prefix(rk)) {
            self.portal_cwd.insert(id, folder_path_with_real_case(dir));
        } else {
            return false;
        }
        self.refresh_portal(id);
        true
    }

    /// Makes regular fence `id` show `dir` (a desktop folder or a folder shortcut's target) in
    /// place of its icons; a folder opened from another opened folder replaces it. Returns false
    /// if refused (an unknown fence, a folder portal, or a folder that is gone).
    pub fn open_folder(&mut self, id: FenceId, dir: &Path) -> bool {
        if self
            .fence(id)
            .is_none_or(|f| f.kind == FenceKind::FolderPortal)
            || !dir.is_dir()
        {
            return false;
        }
        self.portal_cwd.remove(&id);
        let root = folder_path_with_real_case(dir);
        // A drive's root has no name of its own: Explorer titles it "Local Disk (C:)".
        let title = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .or_else(|| shell::normal_display_name(&root))
            .unwrap_or_else(|| root.to_string_lossy().to_string());
        // A folder opened from another opened folder keeps the icon that opened the first one.
        let opened_by = self.opened_folders.get(&id).and_then(|o| o.opened_by);
        self.opened_folders.insert(
            id,
            OpenedFolder {
                root,
                title,
                opened_by,
            },
        );
        self.refresh_portal(id);
        true
    }

    /// Notes the fence's own icon that opened the folder regular fence `id` shows (a desktop
    /// folder or a folder shortcut), so up selects it again.
    pub fn set_opened_by(&mut self, id: FenceId, item: ItemId) {
        if let Some(o) = self.opened_folders.get_mut(&id) {
            o.opened_by = Some(item);
        }
    }

    /// What one level up selects, as in Explorer: the subfolder just left, or the fence's own
    /// icon that opened the folder when up brings its icons back.
    pub fn up_selection(&self, id: FenceId) -> Option<ItemId> {
        match self.portal_cwd.get(&id) {
            Some(cur) => Some(portal_item_id(&cur.to_string_lossy())),
            None => self.opened_folders.get(&id).and_then(|o| o.opened_by),
        }
    }

    /// Leaves the folder opened in regular fence `id`: it shows its own icons again. False when
    /// it shows none.
    fn leave_opened_folder(&mut self, id: FenceId) -> bool {
        if self.opened_folders.remove(&id).is_none() {
            return false;
        }
        self.portal_missing.remove(&id);
        self.evict_portal_members(id);
        true
    }

    /// One folder up (never above the root). From a folder opened in a regular fence, back to
    /// its own icons. Returns false when there is nowhere to go.
    pub fn portal_up(&mut self, id: FenceId) -> bool {
        let Some(cur) = self.portal_cwd.get(&id).cloned() else {
            return self.leave_opened_folder(id);
        };
        match cur.parent() {
            Some(parent) => self.portal_enter(id, parent),
            None => {
                self.portal_cwd.remove(&id);
                self.refresh_portal(id);
                true
            }
        }
    }

    /// Back to the root folder: a portal's own, or the folder opened in a regular fence.
    pub fn portal_home(&mut self, id: FenceId) -> bool {
        if self.portal_cwd.remove(&id).is_some() {
            self.refresh_portal(id);
            true
        } else {
            false
        }
    }

    /// The portal (if any) whose current folder contains this runtime item: the first in layout
    /// order when two portals show the same folder.
    pub fn portal_of_item(&self, item: ItemId) -> Option<FenceId> {
        self.fences().iter().map(|f| f.id).find(|f| {
            self.portal_members
                .get(f)
                .is_some_and(|ids| ids.contains(&item))
        })
    }

    pub fn set_portal_navigate(&mut self, id: FenceId, on: bool) {
        if let Some(f) = self.fence_mut(id)
            && f.portal_navigate != on
        {
            f.portal_navigate = on;
            self.dirty = true;
        }
    }

    pub fn set_open_folders_inside(&mut self, id: FenceId, on: bool) {
        if let Some(f) = self.fence_mut(id)
            && f.open_folders_inside != on
        {
            f.open_folders_inside = on;
            self.dirty = true;
        }
    }

    pub fn set_hide_title_icon(&mut self, id: FenceId, on: bool) {
        if let Some(f) = self.fence_mut(id)
            && f.hide_title_icon != on
        {
            f.hide_title_icon = on;
            self.dirty = true;
        }
    }

    /// Title shown for a fence: one showing a subfolder, or a folder opened in it, shows that
    /// folder's name.
    pub fn display_title(&self, f: &Fence) -> String {
        if let Some(name) = self.portal_cwd.get(&f.id).and_then(|cwd| cwd.file_name()) {
            return name.to_string_lossy().to_string();
        }
        match self.opened_folders.get(&f.id) {
            Some(opened) => opened.title.clone(),
            None => f.title.clone(),
        }
    }

    /// Creates a folder-portal fence (plan §12 "文件夹门户", pulled into the MVP on request): a
    /// fence whose items are the folder's entries. Files are never moved.
    pub fn new_portal_fence(&mut self, folder: &Path, rect: RECT) -> Option<FenceId> {
        let title = shell::display_name(folder).unwrap_or_else(|| {
            folder
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| folder.to_string_lossy().to_string())
        });
        let id = self.new_fence(&title, rect)?;
        if let Some(f) = self.fence_mut(id) {
            f.kind = FenceKind::FolderPortal;
            f.source = ItemSourceSpec::Folder {
                path: folder.to_string_lossy().to_string(),
                recursive: false,
                filter: None,
            };
            f.view.sort = SortMode::Name;
        }
        self.refresh_portal(id);
        Some(id)
    }

    /// Forgets portal `id`'s runtime items. Ids are path hashes, so another portal showing the
    /// same folder (navigated into it, or rooted inside the first) shares them: only items no
    /// other portal references leave `portal_items`. Returns the old items for diffing.
    fn evict_portal_members(&mut self, id: FenceId) -> Vec<Item> {
        let old_ids = self.portal_members.remove(&id).unwrap_or_default();
        let shared: HashSet<ItemId> = self.portal_members.values().flatten().copied().collect();
        let mut old = Vec::with_capacity(old_ids.len());
        for i in &old_ids {
            let item = if shared.contains(i) {
                self.portal_items.get(i).cloned()
            } else {
                self.portal_items.remove(i)
            };
            old.extend(item);
        }
        old
    }

    /// Re-reads a portal's folder. Returns true when the items changed (set, order or content:
    /// a case-only rename or a size/mtime change must redraw a Details view too).
    pub fn refresh_portal(&mut self, id: FenceId) -> bool {
        if let Some(cwd) = self.portal_cwd.get(&id)
            && !cwd.is_dir()
        {
            tracing::info!(%id, folder = %cwd.display(), "portal subfolder vanished; back to root");
            self.portal_cwd.remove(&id);
        }
        let Some(dir) = self.portal_path(id) else {
            return false;
        };
        let started = std::time::Instant::now();
        let was_missing = self.portal_missing.contains(&id);
        let old_items = self.evict_portal_members(id);
        // Display names survive a re-read: asking the shell costs several ms per item on a
        // OneDrive folder, and an unchanged file (same path, same mtime) has the same name.
        let known: HashMap<(ItemKey, i64), String> = old_items
            .iter()
            .map(|it| ((it.key.clone(), it.mtime), it.display_name.clone()))
            .collect();
        let known_name = |key: &ItemKey, entry: &DesktopEntry| {
            known
                .get(&(key.clone(), entry.mtime))
                // ItemKey ignores case. A case-only rename leaves that key and mtime
                // unchanged, but the old display spelling must not be reused.
                .filter(|name| entry.file_name.starts_with(name.as_str()))
                .cloned()
        };
        let entries = shell::enumerate_folder(&dir);
        // A folder that is gone lists as empty; the fence must say which of the two it is. Only
        // an empty listing asks (one more round trip on an unreachable share otherwise).
        let missing = entries.is_empty() && !dir.is_dir();
        if missing {
            self.portal_missing.insert(id);
        } else {
            self.portal_missing.remove(&id);
        }
        let unnamed = entries
            .iter()
            .filter(|e| known_name(&ItemKey::from_path(&e.path.to_string_lossy()), e).is_none())
            .count();
        // A first read (startup, a subfolder entered) names the whole folder in one go.
        let listed = if unnamed > 2 {
            shell::display_names_in(&dir)
        } else {
            HashMap::new()
        };
        let mut ids = Vec::new();
        let mut new_items = Vec::new();
        for entry in entries {
            let path_str = entry.path.to_string_lossy().to_string();
            let item_id = portal_item_id(&path_str);
            let (icon_key, _) =
                crate::icons::icon_key_for(&entry.path, entry.is_folder, entry.mtime);
            let key = ItemKey::from_path(&path_str);
            let display_name = known_name(&key, &entry)
                .or_else(|| listed.get(&entry.file_name.to_lowercase()).cloned())
                .unwrap_or_else(|| {
                    shell::display_name(&entry.path).unwrap_or_else(|| {
                        if entry.is_folder {
                            entry.file_name.clone()
                        } else {
                            Path::new(&entry.file_name)
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| entry.file_name.clone())
                        }
                    })
                });
            let item = Item {
                id: item_id,
                key,
                origin: Origin::Namespace,
                display_name,
                file_id: None,
                mtime: entry.mtime,
                is_folder: entry.is_folder,
                attrs: entry.attributes,
                icon_key,
                orphaned_since: None,
                size: entry.size,
                open_count: 0,
                last_opened: None,
                path: Some(path_str),
            };
            new_items.push(item.clone());
            self.portal_items.insert(item_id, item);
            ids.push(item_id);
        }
        let changed = old_items != new_items || missing != was_missing;
        let ms = started.elapsed().as_secs_f32() * 1000.0;
        if changed {
            tracing::info!(%id, folder = %dir.display(), count = ids.len(), ms, "portal refreshed");
        } else {
            tracing::debug!(%id, count = ids.len(), ms, "portal re-read, unchanged");
        }
        self.portal_members.insert(id, ids);
        changed
    }

    /// Re-reads every folder a fence shows (startup, watcher batches). Returns the fences that
    /// changed.
    pub fn refresh_all_portals(&mut self) -> Vec<FenceId> {
        let portals: Vec<FenceId> = self
            .fences()
            .iter()
            .filter(|f| self.folder_root(f.id).is_some())
            .map(|f| f.id)
            .collect();
        // Drop runtime state for portals that no longer exist in the current layout (import /
        // snapshot restore / layout switch): their item ids are path hashes and would otherwise
        // collide with a new portal on the same folder in portal_of_item.
        let stale: Vec<FenceId> = self
            .portal_members
            .keys()
            .filter(|id| !portals.contains(id))
            .copied()
            .collect();
        for id in stale {
            self.evict_portal_members(id);
            self.portal_cwd.remove(&id);
            self.portal_missing.remove(&id);
        }
        portals
            .into_iter()
            .filter(|id| self.refresh_portal(*id))
            .collect()
    }

    pub fn item_by_path(&self, path: &Path) -> Option<ItemId> {
        self.catalog
            .get(&ItemKey::from_path(&path.to_string_lossy()))
            .copied()
    }

    /// The item at `path` if its file is on the desktop right now (a deleted file's record,
    /// kept for a while, does not count).
    pub fn live_item_by_path(&self, path: &Path) -> Option<ItemId> {
        let id = self.item_by_path(path)?;
        self.config
            .items
            .get(&id)
            .is_some_and(|it| it.orphaned_since.is_none())
            .then_some(id)
    }

    /// Re-keys an item after a rename so it stays in its fence. Returns false if unknown.
    pub fn rename_item(&mut self, old: &Path, new: &Path) -> bool {
        let old_key = ItemKey::from_path(&old.to_string_lossy());
        let new_key = ItemKey::from_path(&new.to_string_lossy());
        // Safe-save (Word, Excel): the document is renamed to a temporary name and a fresh copy
        // is renamed over its name. The item already on the new key (the document's) is kept;
        // the sync finds the file again. AutoCAD does the same with `.bak` as the temporary
        // name (abcd.dwg → abcd.bak, then sav*.tmp → abcd.dwg).
        let to_temp = new.file_name().is_some_and(|n| {
            let n = n.to_string_lossy();
            pecofence_core::rules::is_temporary(&n, None)
                || pecofence_core::rules::ext_of(&n) == ".bak"
        });
        if to_temp || (new_key != old_key && self.catalog.contains_key(&new_key)) {
            return false;
        }
        let Some(id) = self.catalog.remove(&old_key) else {
            return false;
        };
        let Some(item) = self.config.items.get_mut(&id) else {
            return false;
        };
        let is_folder = std::fs::metadata(new)
            .map(|m| m.is_dir())
            .unwrap_or(item.is_folder);
        item.key = new_key.clone();
        item.path = Some(new.to_string_lossy().into_owned());
        item.is_folder = is_folder;
        item.display_name = shell::display_name(new).unwrap_or_else(|| {
            new.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        });
        let (icon_key, _) = crate::icons::icon_key_for(new, is_folder, item.mtime);
        item.icon_key = icon_key;
        item.orphaned_since = None;
        self.catalog.insert(new_key, id);
        self.dirty = true;
        true
    }

    /// The folder that moving items into fence `to` puts their files in: what a folder portal
    /// shows. `None` for any other fence, also while it shows a folder opened in it: moving
    /// icons there (「移出栅栏」, a drop on the bare desktop, a 新建 or drop that lands later,
    /// `item move`) only changes membership, so no desktop file goes into that folder unseen.
    pub fn move_target_folder(&self, to: FenceId) -> Option<PathBuf> {
        self.portal_root(to)?;
        self.portal_path(to)
    }

    pub fn move_items(&mut self, items: &[ItemId], to: FenceId) -> usize {
        // Portal fences show a folder, not desktop membership: nothing can be moved into them,
        // and their (runtime) items cannot be moved out.
        if self.move_target_folder(to).is_some() {
            return 0;
        }
        let mut n = 0;
        for &id in items {
            if !self.config.items.contains_key(&id) {
                continue;
            }
            if self
                .config
                .assign(self.layout, id, to, AssignedBy::User)
                .is_some()
            {
                n += 1;
            }
        }
        if n > 0 {
            self.dirty = true;
        }
        n
    }

    /// Manual arrangement: places `items` (keeping their relative order) before display index
    /// `index` of `fence`. Only for 手动-sorted virtual fences (portals are sort-only; a sorted
    /// fence snaps back, like Explorer with auto-arrange on). Returns true when the order changed.
    pub fn reorder_items(&mut self, fence: FenceId, items: &[ItemId], index: usize) -> bool {
        if self.portal_path(fence).is_some() {
            return false;
        }
        let Some(f) = self.fence(fence) else {
            return false;
        };
        if f.view.sort != SortMode::Manual {
            return false;
        }
        let reverse = f.view.reverse;
        let mut order: Vec<ItemId> = self.items_of(f).iter().map(|it| it.id).collect();
        let before_order = order.clone();
        let moved: Vec<ItemId> = order
            .iter()
            .copied()
            .filter(|id| items.contains(id))
            .collect();
        if moved.is_empty() {
            return false;
        }
        let before = order[..index.min(order.len())]
            .iter()
            .filter(|id| items.contains(id))
            .count();
        order.retain(|id| !items.contains(id));
        let at = index.saturating_sub(before).min(order.len());
        order.splice(at..at, moved);
        if order == before_order {
            return false;
        }
        if reverse {
            // manual_index is stored in forward order; the view reverses it on display.
            order.reverse();
        }
        let Some(fm) = self.fence_mut(fence) else {
            return false;
        };
        for (i, id) in order.iter().enumerate() {
            if let Some(r) = fm.items.iter_mut().find(|r| r.item_id == *id) {
                r.manual_index = Some(i as u32);
            }
        }
        self.dirty = true;
        true
    }

    /// Re-runs the rules over every item (user assignments are respected).
    pub fn apply_rules_all(&mut self, entries: &[DesktopEntry]) -> usize {
        self.apply_rules_filtered(entries, false)
    }

    /// The hourly sweep for clock-dependent rules: only an item whose winning rule has an
    /// idle-days condition moves, so a later-added type rule still waits for "立即应用".
    pub fn apply_idle_rules(&mut self, entries: &[DesktopEntry]) -> usize {
        self.apply_rules_filtered(entries, true)
    }

    fn apply_rules_filtered(&mut self, entries: &[DesktopEntry], idle_only: bool) -> usize {
        let plan = self.plan_rules(entries, idle_only);
        let mut moved = 0;
        for m in plan {
            if self
                .config
                .assign(self.layout, m.item, m.to, m.by)
                .is_some()
            {
                moved += 1;
            }
        }
        if moved > 0 {
            self.dirty = true;
        }
        moved
    }

    /// What [`apply_rules_all`](Self::apply_rules_all) would do, without doing it: one entry per
    /// item that is not pinned by a manual placement and whose rule target differs from the
    /// fence it is in now (`idle_only`: see [`apply_idle_rules`](Self::apply_idle_rules)). With
    /// [`Unmatched::Stay`] an item in a fence that no rule matches stays there.
    pub fn plan_rules(&self, entries: &[DesktopEntry], idle_only: bool) -> Vec<PlannedRuleMove> {
        let mut plan = Vec::new();
        for entry in entries {
            if entry.origin == EntryOrigin::Namespace {
                continue;
            }
            let key = ItemKey::from_path(&entry.path.to_string_lossy());
            let Some(&id) = self.catalog.get(&key) else {
                continue;
            };
            let current = self.fences().iter().find(|f| f.contains_item(id));
            if let Some(f) = current
                && f.items
                    .iter()
                    .any(|r| r.item_id == id && r.assigned_by == AssignedBy::User)
            {
                continue;
            }
            let decision = self.config.rules.evaluate(&self.facts_for(entry));
            if idle_only {
                let by_idle_rule = match decision {
                    Decision::Route { rule, .. } => self
                        .config
                        .rules
                        .list
                        .iter()
                        .any(|r| r.id == rule && Self::has_idle_cond(r)),
                    _ => false,
                };
                if !by_idle_rule {
                    continue;
                }
            }
            if current.is_some()
                && matches!(decision, Decision::Default(_))
                && self.config.rules.unmatched == Unmatched::Stay
            {
                continue;
            }
            let Some((to, by)) = self.target_fence(decision) else {
                continue;
            };
            let from = current.map(|f| f.id);
            if from == Some(to) {
                continue;
            }
            plan.push(PlannedRuleMove {
                item: id,
                from,
                to,
                by,
            });
        }
        plan
    }

    // ---- fences -----------------------------------------------------------------------------

    /// Records fence `id` at window rect `rect`. Rolled up, `rect` is its title row and the
    /// saved rect is `expanded_h_px` tall: hanging below the row, or standing on it when
    /// `row_bottom`. `choose` (a 向上展开 fence rolled up at rest) picks that end the way an
    /// expansion picks its side, on the work area alone (`clearance::expand_side`): the saved rect
    /// stays on the screen around the row, so a restart puts the row back where it was. Where
    /// the fence fits on neither side, the saved rect is cut to the room there and `expanded_h`
    /// keeps the full height. Returns the end it saved (`Fence::bottom_anchored`).
    pub fn set_fence_bounds(
        &mut self,
        id: FenceId,
        rect: RECT,
        rolled: bool,
        expanded_h_px: i32,
        row_bottom: bool,
        choose: bool,
    ) -> bool {
        let full = expanded_h_px.max(1);
        let area_at = |x: i32, y: i32| {
            self.work_areas
                .iter()
                .find(|w| x >= w.left && x < w.right && y >= w.top && y < w.bottom)
                .or(self.work_areas.first())
                .cloned()
        };
        let (mut bottom_anchored, mut h) = (row_bottom, full);
        if rolled
            && choose
            && let Some(w) = area_at((rect.left + rect.right) / 2, (rect.top + rect.bottom) / 2)
        {
            let (up, shown) = crate::layout::clearance::expand_side(
                full,
                rect.bottom - w.top,
                w.bottom - rect.top,
            );
            bottom_anchored = up;
            h = shown.max(1);
        }
        let (top, bottom) = match (rolled, bottom_anchored) {
            (true, false) => (rect.top, rect.top + h),
            (true, true) => (rect.bottom - h, rect.bottom),
            (false, _) => (rect.top, rect.bottom),
        };
        let px = PxRect {
            left: rect.left,
            top,
            right: rect.right,
            bottom,
        };
        let (cx, cy) = px.center();
        let Some(work) = area_at(cx, cy) else {
            return bottom_anchored;
        };
        let geo = geometry::normalize(px, &work);
        let expanded_h = if rolled {
            full as f32 / work.scale()
        } else {
            geo.h
        };
        if let Some(f) = self.fence_mut(id)
            && (f.geometry != geo
                || f.rolled_up != rolled
                || f.bottom_anchored != bottom_anchored
                || (f.expanded_h - expanded_h).abs() > 0.5)
        {
            f.expanded_h = expanded_h;
            f.geometry = geo;
            f.rolled_up = rolled;
            f.bottom_anchored = bottom_anchored;
            self.dirty = true;
        }
        bottom_anchored
    }

    /// Height (device px) fence `fence` opens to: its saved rect's, or rolled up the full
    /// expanded height, which a saved rect cut to the room around its title row keeps.
    pub fn fence_expanded_px(&self, fence: &Fence) -> i32 {
        let h = self.fence_px_rect(fence).height();
        if !fence.rolled_up {
            return h;
        }
        let scale = geometry::resolve_work_area(&fence.geometry, &self.work_areas)
            .map_or(1.0, |w| w.scale());
        ((fence.expanded_h * scale).round() as i32).max(h)
    }

    /// Physical rectangle for a fence on the current monitors (expanded height).
    pub fn fence_px_rect(&self, fence: &Fence) -> PxRect {
        let work = geometry::resolve_work_area(&fence.geometry, &self.work_areas)
            .cloned()
            .unwrap_or(WorkArea {
                device_path: String::new(),
                gdi_name: String::new(),
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1040,
                dpi: 96,
                mon_left: 0,
                mon_top: 0,
                mon_right: 1920,
                mon_bottom: 1040,
            });
        geometry::denormalize(&fence.geometry, &work)
    }

    #[allow(dead_code)]
    pub fn toggle_rolled(&mut self, id: FenceId) -> Option<bool> {
        let rolled = {
            let f = self.fence_mut(id)?;
            f.rolled_up = !f.rolled_up;
            f.rolled_up
        };
        self.dirty = true;
        Some(rolled)
    }

    pub fn rename_fence(&mut self, id: FenceId, title: &str) {
        if let Some(f) = self.fence_mut(id)
            && f.title != title
        {
            f.title = title.to_string();
            self.dirty = true;
        }
    }

    pub fn set_icon_size(&mut self, id: FenceId, size: u32) {
        if let Some(f) = self.fence_mut(id)
            && f.view.icon_size != size
        {
            f.view.icon_size = size;
            self.dirty = true;
        }
    }

    pub fn set_auto_height(&mut self, id: FenceId, on: bool) {
        if let Some(f) = self.fence_mut(id)
            && f.view.auto_height != on
        {
            f.view.auto_height = on;
            self.dirty = true;
        }
    }

    /// The catalogued item for a desktop path, if known.
    pub fn item_id_for_path(&self, path: &Path) -> Option<ItemId> {
        self.catalog
            .get(&ItemKey::from_path(&path.to_string_lossy()))
            .copied()
    }

    pub fn set_reverse(&mut self, id: FenceId, on: bool) {
        if let Some(f) = self.fence_mut(id)
            && f.view.reverse != on
        {
            f.view.reverse = on;
            self.dirty = true;
        }
    }

    pub fn set_locked(&mut self, id: FenceId, on: bool) {
        if let Some(f) = self.fence_mut(id)
            && f.locked != on
        {
            f.locked = on;
            self.dirty = true;
        }
    }

    pub fn set_exclude_from_quick_hide(&mut self, id: FenceId, on: bool) {
        if let Some(f) = self.fence_mut(id)
            && f.exclude_from_quick_hide != on
        {
            f.exclude_from_quick_hide = on;
            self.dirty = true;
        }
    }

    pub fn set_hide_when_empty(&mut self, id: FenceId, on: bool) {
        if let Some(f) = self.fence_mut(id)
            && f.hide_when_empty != on
        {
            f.hide_when_empty = on;
            self.dirty = true;
        }
    }

    pub fn set_expand_upward(&mut self, id: FenceId, on: bool) {
        if let Some(f) = self.fence_mut(id)
            && f.expand_upward != on
        {
            f.expand_upward = on;
            self.dirty = true;
        }
    }

    /// Changes a fence's appearance override; one left at every default is dropped (None =
    /// follow the global settings).
    fn edit_appearance(
        &mut self,
        id: FenceId,
        edit: impl FnOnce(&mut pecofence_core::AppearanceOverride),
    ) {
        if let Some(f) = self.fence_mut(id) {
            let mut next = f.appearance.clone().unwrap_or_default();
            edit(&mut next);
            let next = (next != pecofence_core::AppearanceOverride::default()).then_some(next);
            if f.appearance != next {
                f.appearance = next;
                self.dirty = true;
            }
        }
    }

    /// Per-fence appearance override (None = follow the global settings).
    pub fn set_appearance(
        &mut self,
        id: FenceId,
        backdrop: Option<pecofence_core::Backdrop>,
        opacity: Option<f32>,
    ) {
        self.edit_appearance(id, |a| {
            a.backdrop = backdrop;
            a.opacity = opacity;
        });
    }

    /// Per-fence colour wash / title colour / title size (None fields = theme defaults).
    pub fn set_style(
        &mut self,
        id: FenceId,
        tint_rgb: Option<[u8; 3]>,
        title_rgb: Option<[u8; 3]>,
        title_size: Option<pecofence_core::TitleSize>,
    ) {
        self.edit_appearance(id, |a| {
            a.tint_rgb = tint_rgb;
            a.title_rgb = title_rgb;
            a.title_size = title_size;
        });
    }

    /// Per-fence title on hover (None = follow `rollUp.titleOnHover`).
    pub fn set_title_on_hover(&mut self, id: FenceId, on: Option<bool>) {
        self.edit_appearance(id, |a| a.title_on_hover = on);
    }

    /// The tint washes only the title row.
    pub fn set_tint_title_only(&mut self, id: FenceId, on: bool) {
        self.edit_appearance(id, |a| a.tint_title_only = on);
    }

    /// The fence's bar in a tab strip (None = its tint).
    pub fn set_tab_color(&mut self, id: FenceId, rgb: Option<[u8; 3]>) {
        self.edit_appearance(id, |a| a.tab_rgb = rgb);
    }

    pub fn set_spacing(&mut self, id: FenceId, spacing: pecofence_core::Spacing) {
        if let Some(f) = self.fence_mut(id)
            && f.view.spacing != spacing
        {
            f.view.spacing = spacing;
            self.dirty = true;
        }
    }

    pub fn set_columns_visible(&mut self, id: FenceId, visible: [bool; 3]) {
        let next = (visible != [true; 3]).then_some(visible);
        if let Some(f) = self.fence_mut(id)
            && f.view.columns_visible != next
        {
            f.view.columns_visible = next;
            self.dirty = true;
        }
    }

    pub fn set_column_widths(&mut self, id: FenceId, widths: [f32; 3]) {
        let next = (widths != crate::layout::DetailColumns::DEFAULT_WIDTHS).then_some(widths);
        if let Some(f) = self.fence_mut(id)
            && f.view.column_widths != next
        {
            f.view.column_widths = next;
            self.dirty = true;
        }
    }

    pub fn set_layout(&mut self, id: FenceId, layout: pecofence_core::ViewLayout) {
        if let Some(f) = self.fence_mut(id)
            && f.view.layout != layout
        {
            f.view.layout = layout;
            self.dirty = true;
        }
    }

    /// Any order but by date ends "按时间分组" (sections only make sense in date order).
    pub fn set_sort(&mut self, id: FenceId, sort: SortMode) {
        let Some(f) = self.fence_mut(id) else {
            return;
        };
        let mut changed = false;
        if f.view.sort != sort {
            f.view.sort = sort;
            changed = true;
        }
        if sort != SortMode::Date && f.view.group_by_date {
            f.view.group_by_date = false;
            changed = true;
        }
        if changed {
            self.dirty = true;
        }
    }

    /// "按时间分组"; turning it on also sorts by date.
    pub fn set_group_by_date(&mut self, id: FenceId, on: bool) {
        let Some(f) = self.fence_mut(id) else {
            return;
        };
        let mut changed = false;
        if f.view.group_by_date != on {
            f.view.group_by_date = on;
            changed = true;
        }
        if on && f.view.sort != SortMode::Date {
            f.view.sort = SortMode::Date;
            changed = true;
        }
        if changed {
            self.dirty = true;
        }
    }

    /// The layout holds `MAX_FENCES` already: `new_fence` refuses, since a save would fail.
    pub fn fence_limit_reached(&self) -> bool {
        self.fences().len() >= pecofence_core::config_store::MAX_FENCES
    }

    /// None when there is no monitor to place it on or [`Self::fence_limit_reached`].
    pub fn new_fence(&mut self, title: &str, rect: RECT) -> Option<FenceId> {
        if self.fence_limit_reached() {
            return None;
        }
        let px = PxRect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        };
        let (cx, cy) = px.center();
        let work = self
            .work_areas
            .iter()
            .find(|w| cx >= w.left && cx < w.right && cy >= w.top && cy < w.bottom)
            .or(self.work_areas.first())
            .cloned()?;
        let geo = geometry::normalize(px, &work);
        let mut fence = Fence::new(title, FenceKind::Virtual, geo);
        fence.view.icon_size = self.config.settings.icon_size;
        let id = fence.id;
        self.config.layouts[self.layout].fences.push(fence);
        self.dirty = true;
        Some(id)
    }

    /// "快速添加": a fence for `template` at `rect` plus its rule, inserted ahead of the
    /// existing rules so it wins over the broader first-run presets (an installer is also a
    /// program). `Err(existing)` when the template was added before; `existing` is the fence
    /// its rule still points to, if any.
    pub fn add_template(
        &mut self,
        template: Template,
        rect: RECT,
    ) -> Result<FenceId, Option<FenceId>> {
        let key = template.key();
        if let Some(rule) = self
            .config
            .rules
            .list
            .iter()
            .find(|r| r.template.as_deref() == Some(key))
        {
            let existing = match rule.target {
                Target::Fence(id) => self.routable_fence(id),
                Target::Inbox => self.inbox_id(),
            };
            return Err(existing);
        }
        let id = self.new_fence(&template.title(), rect).ok_or(None)?;
        let rule = template.rule(id);
        // Idle-days rules must stay ahead of the type-only ones ("待清理" before "安装包"),
        // otherwise a plain type rule claims every installer first and the idle rule never fires.
        let at = if Self::has_idle_cond(&rule) {
            0
        } else {
            self.config
                .rules
                .list
                .iter()
                .take_while(|r| Self::has_idle_cond(r))
                .count()
        };
        self.config.rules.list.insert(at, rule);
        self.dirty = true;
        Ok(id)
    }

    fn has_idle_cond(rule: &pecofence_core::rules::Rule) -> bool {
        rule.all_of
            .iter()
            .any(|c| matches!(c, Cond::IdleDays { .. }))
    }

    /// Deletes a fence; its items go back to the inbox. The inbox itself cannot be deleted.
    pub fn delete_fence(&mut self, id: FenceId) -> bool {
        let Some(inbox) = self.inbox_id() else {
            return false;
        };
        if inbox == id {
            return false;
        }
        // Out of its tab group first: a deleted host hands its place to the next tab, which
        // keeps the others (they would otherwise all open on the host's rect).
        self.detach_tab(id);
        let Some(pos) = self.fences().iter().position(|f| f.id == id) else {
            return false;
        };
        let removed = self.config.layouts[self.layout].fences.remove(pos);
        self.portal_cwd.remove(&id);
        self.opened_folders.remove(&id);
        self.portal_missing.remove(&id);
        self.evict_portal_members(id);
        let ids: Vec<ItemId> = removed.items.iter().map(|r| r.item_id).collect();
        for item in ids {
            self.config
                .assign(self.layout, item, inbox, AssignedBy::Migration);
        }
        self.config
            .rules
            .list
            .retain(|r| r.target != Target::Fence(id));
        // Its tabs (if it hosted any) become windows again; a deleted tab leaves its host.
        self.normalize_tabs();
        self.dirty = true;
        true
    }
}

fn ext_of_key(key: &ItemKey) -> String {
    key.as_path()
        .and_then(|p| p.rsplit_once('.').map(|(_, e)| e.to_string()))
        .unwrap_or_default()
}

/// Case-insensitive natural ordering (digits compared numerically), like Explorer.
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let (a, b) = (a.to_lowercase(), b.to_lowercase());
    let mut ai = a.chars().peekable();
    let mut bi = b.chars().peekable();
    loop {
        match (ai.peek().copied(), bi.peek().copied()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, _) => return std::cmp::Ordering::Less,
            (_, None) => return std::cmp::Ordering::Greater,
            (Some(ca), Some(cb)) if ca.is_ascii_digit() && cb.is_ascii_digit() => {
                let mut na = 0u64;
                while let Some(c) = ai.peek().copied().filter(|c| c.is_ascii_digit()) {
                    na = na.saturating_mul(10).saturating_add(c as u64 - '0' as u64);
                    ai.next();
                }
                let mut nb = 0u64;
                while let Some(c) = bi.peek().copied().filter(|c| c.is_ascii_digit()) {
                    nb = nb.saturating_mul(10).saturating_add(c as u64 - '0' as u64);
                    bi.next();
                }
                if na != nb {
                    return na.cmp(&nb);
                }
            }
            (Some(ca), Some(cb)) => {
                if ca != cb {
                    return ca.cmp(&cb);
                }
                ai.next();
                bi.next();
            }
        }
    }
}

/// `key` with exactly one trailing separator, for "is inside" prefix tests. Drive roots from
/// `ItemKey::from_path` already end in a backslash (`d:\`); deeper paths do not.
fn dir_prefix(key: &str) -> String {
    if key.ends_with('\\') {
        key.to_string()
    } else {
        format!("{key}\\")
    }
}

/// Restore the displayed component from its directory entry. Do not canonicalize:
/// resolving a junction could replace the portal's navigable path with an outside target.
fn folder_path_with_real_case(path: &Path) -> PathBuf {
    let actual = (|| {
        let name = path.file_name()?.to_string_lossy().to_lowercase();
        std::fs::read_dir(path.parent()?)
            .ok()?
            .filter_map(Result::ok)
            .find(|entry| entry.file_name().to_string_lossy().to_lowercase() == name)
            .map(|entry| entry.path())
    })();
    actual.unwrap_or_else(|| path.to_path_buf())
}

/// A folder opened in a regular fence ([`AppState::open_folder`]).
struct OpenedFolder {
    /// The folder opened: navigation never goes above it.
    root: PathBuf,
    /// The fence's title meanwhile: the folder's name, or a drive's display name.
    title: String,
    /// The fence's own icon that opened it, selected again when up brings the icons back.
    opened_by: Option<ItemId>,
}

/// Stable id for a portal item derived from its (case-folded) path, so selections and pending
/// icon lookups survive a folder refresh.
pub(crate) fn portal_item_id(path: &str) -> ItemId {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let key = path.to_lowercase().replace('/', "\\");
    let mut h1 = DefaultHasher::new();
    key.hash(&mut h1);
    let mut h2 = DefaultHasher::new();
    (&key, 0x5eedu16).hash(&mut h2);
    uuid::Uuid::from_u128(((h1.finish() as u128) << 64) | h2.finish() as u128)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pecofence_core::geometry::WorkArea;

    fn monitor(id: &str, left: i32, w: i32, h: i32, dpi: u32) -> WorkArea {
        WorkArea {
            device_path: id.into(),
            gdi_name: String::new(),
            left,
            top: 0,
            right: left + w,
            bottom: h,
            dpi,
            mon_left: left,
            mon_top: 0,
            mon_right: left + w,
            mon_bottom: h + 60,
        }
    }

    fn test_state() -> AppState {
        state_on(vec![monitor("test", 0, 1920, 1040, 96)])
    }

    fn state_on(work_areas: Vec<WorkArea>) -> AppState {
        // One folder per test: tests saving at the same time shared config.json.tmp before.
        static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "pecofence-state-test-{}-{}-{}",
            std::process::id(),
            pecofence_core::now_unix(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let mut state = AppState {
            config: Config::default(),
            store: ConfigStore::new(dir),
            layout: 0,
            dirty: false,
            catalog: HashMap::new(),
            portal_items: HashMap::new(),
            portal_members: HashMap::new(),
            portal_cwd: HashMap::new(),
            opened_folders: HashMap::new(),
            portal_missing: HashSet::new(),
            tmp_settle_at: std::cell::Cell::new(None),
            work_areas,
            unplaced: Vec::new(),
            first_run: true,
            recovered_from: None,
            unreadable_moved_to: None,
            load_failures: Vec::new(),
        };
        state.ensure_layout();
        state
    }

    fn rect(left: i32, top: i32, w: i32, h: i32) -> RECT {
        RECT {
            left,
            top,
            right: left + w,
            bottom: top + h,
        }
    }

    #[test]
    fn an_imported_file_never_pushes_local_snapshots_out() {
        let snap = |name: String, ts: i64| pecofence_core::Snapshot {
            id: uuid::Uuid::new_v4(),
            name,
            ts,
            layouts: Vec::new(),
        };
        let mut state = test_state();
        state.config.snapshots = vec![snap("home".into(), 1), snap("work".into(), 2)];
        let local: Vec<uuid::Uuid> = state.config.snapshots.iter().map(|s| s.id).collect();
        let mut file = state.config.clone();
        file.snapshots = (0..pecofence_core::MAX_SNAPSHOTS)
            .map(|i| snap(format!("file {i}"), 100 + i as i64))
            .collect();
        let newest = file.snapshots.last().unwrap().id;
        state.replace_config(file);
        let kept = &state.config.snapshots;
        assert_eq!(kept.len(), pecofence_core::MAX_SNAPSHOTS);
        assert!(local.iter().all(|id| kept.iter().any(|s| s.id == *id)));
        assert!(kept.iter().any(|s| s.id == newest));
        assert!(kept.windows(2).all(|w| w[0].ts <= w[1].ts));
    }

    /// An import saved at another resolution: relayout_from_state runs ensure_layout again,
    /// and the fences with no spot here must still be moved apart after that.
    #[test]
    fn an_imported_layout_keeps_its_fences_to_place() {
        let id = "GSM7787#5&2C948443&0&UID24832";
        let mut state = state_on(vec![monitor(id, 0, 3840, 2064, 192)]);
        let mut file = state.config.clone();
        file.layouts[0].fingerprint[0].screen_px = [1920, 1080];
        file.layouts[0].fingerprint[0].dpi = 96;
        state.replace_config(file);
        let n = state.fences().len();
        assert!(n > 0);
        assert_eq!(state.unplaced.len(), n);
        state.ensure_layout();
        assert_eq!(state.unplaced.len(), n);
    }

    #[test]
    fn an_imported_file_shows_a_fence_listed_twice_once() {
        let mut state = test_state();
        let mut file = state.config.clone();
        let mut copy = file.layouts[0].fences[0].clone();
        copy.title = "Dup".into();
        file.layouts[0].fences.push(copy);
        let fences = file.layouts[0].fences.len();
        state.replace_config(file);
        assert_eq!(state.fences().len(), fences - 1);
        assert!(state.fences().iter().all(|f| f.title != "Dup"));
    }

    /// Issue #24: unplugging the big monitor, working on the laptop, plugging it back in.
    #[test]
    fn every_monitor_set_shows_the_same_fences_at_its_own_spots() {
        let big = monitor("GSM7787#5&2C948443&0&UID24832", 0, 3840, 2088, 144);
        let laptop = monitor("BOE0A1B#4&1A&0&UID1", -1920, 1920, 1020, 120);
        let mut state = state_on(vec![big.clone(), laptop.clone()]);
        let docked: Vec<Fence> = state.fences().to_vec();
        let first = docked[0].id;
        assert!(state.unplaced.is_empty());
        let ids = |s: &AppState| s.fences().iter().map(|f| f.id).collect::<Vec<_>>();

        state.work_areas = vec![laptop.clone()];
        state.ensure_layout();
        assert_eq!(state.layout, 1);
        // The same fences; all were on the big monitor, so none has a spot here yet.
        assert_eq!(ids(&state), docked.iter().map(|f| f.id).collect::<Vec<_>>());
        assert_eq!(state.unplaced.len(), docked.len());
        state.set_fence_bounds(first, rect(-1900, 20, 300, 200), false, 200, false, false);
        state.rename_fence(first, "Laptop title");
        let made = state
            .new_fence("Made on the laptop", rect(-1500, 600, 300, 200))
            .unwrap();

        state.work_areas = vec![laptop.clone(), big.clone()];
        state.ensure_layout();
        assert_eq!(state.layout, 0);
        // Back at the docked spots, plus the new fence, which has no spot here yet.
        let f = state.fence(first).unwrap();
        assert_eq!(f.title, "Laptop title");
        assert_eq!(f.geometry, docked[0].geometry);
        assert_eq!(state.unplaced, vec![made]);
        assert_eq!(
            state.fence(made).unwrap().geometry.monitor,
            laptop.device_path
        );
        assert!(state.delete_fence(made));

        state.work_areas = vec![laptop.clone()];
        state.ensure_layout();
        assert!(state.fence(made).is_none());
        assert_eq!(
            state.fence(first).unwrap().geometry.monitor,
            laptop.device_path
        );
        assert!(!state.unplaced.contains(&first));
    }

    /// Files from 0.1.3 kept separate fences per monitor set: the first start merges them.
    #[test]
    fn old_per_set_fences_are_merged_once() {
        let mut state = state_on(vec![monitor(r"\\.\DISPLAY1", 0, 2560, 1392, 96)]);
        let mut other = state.config.layouts[0].clone();
        let own = Fence::new(
            "Only on the laptop",
            FenceKind::Virtual,
            other.fences[0].geometry.clone(),
        );
        other.fences.push(own.clone());
        other.fingerprint[0].device_path = r"\\.\DISPLAY2".into();
        state.config.layouts.push(other);
        // As 0.1.3 saved them: no "shown last" stamp, no resolutions.
        for l in &mut state.config.layouts {
            l.shown = 0;
            l.fingerprint[0].screen_px = [0, 0];
        }
        state.ensure_layout();
        assert_eq!(state.layout, 0);
        assert!(state.fence(own.id).is_some());
        // Merged once: a fence deleted now stays deleted on the other set.
        assert!(state.delete_fence(own.id));
        state.work_areas = vec![monitor(r"\\.\DISPLAY2", 0, 1920, 1020, 96)];
        state.ensure_layout();
        assert_eq!(state.layout, 1);
        assert!(state.fence(own.id).is_none());
    }

    /// Rolled up, a fence saves its expanded rect below the title row, or above it when the
    /// title row is at its bottom. A 向上展开 fence at rest picks the end that keeps the rect on
    /// the screen around the row (above first), cut to the room where it fits neither way.
    #[test]
    fn rolled_bounds_stay_on_the_screen_around_the_title_row() {
        let mut state = test_state(); // work area 0..1040
        let id = state.fences()[0].id;
        let saved = |s: &AppState| {
            let f = s.fence(id).unwrap();
            let px = s.fence_px_rect(f);
            (px.top, px.bottom, f.bottom_anchored, s.fence_expanded_px(f))
        };
        // Title row at 600..632, 300 px expanded.
        assert!(!state.set_fence_bounds(id, rect(20, 600, 300, 32), true, 300, false, false));
        assert_eq!(saved(&state), (600, 900, false, 300));
        state.set_expand_upward(id, true);
        assert!(state.set_fence_bounds(id, rect(20, 600, 300, 32), true, 300, false, true));
        assert_eq!(saved(&state), (332, 632, true, 300));
        // A row near the top: the fence fits below it only.
        assert!(!state.set_fence_bounds(id, rect(20, 30, 300, 32), true, 300, true, true));
        assert_eq!(saved(&state), (30, 330, false, 300));
        // 800 px fits on neither side of a row at 500: the larger room below, the full height
        // kept for when it opens.
        assert!(!state.set_fence_bounds(id, rect(20, 500, 300, 32), true, 800, false, true));
        assert_eq!(saved(&state), (500, 1040, false, 800));
        // Expanded, the window is the rect either way, with the end it opened from.
        assert!(state.set_fence_bounds(id, rect(20, 100, 300, 300), false, 300, true, false));
        assert_eq!(saved(&state), (100, 400, true, 300));
    }

    /// The same monitor at 4K / 200 % and at 1080p / 100 %: each keeps its own spots.
    #[test]
    fn each_resolution_keeps_its_own_spots() {
        let id = "GSM7787#5&2C948443&0&UID24832";
        let mut state = state_on(vec![monitor(id, 0, 3840, 2064, 192)]);
        let at_4k = state.fences().to_vec();
        let first = at_4k[0].id;
        state.work_areas = vec![monitor(id, 0, 1920, 1032, 96)];
        state.ensure_layout();
        assert_eq!((state.layout, state.config.layouts.len()), (1, 2));
        assert_eq!(state.unplaced.len(), at_4k.len());
        state.set_fence_bounds(first, rect(20, 20, 300, 200), false, 200, false, false);

        state.work_areas = vec![monitor(id, 0, 3840, 2064, 192)];
        state.ensure_layout();
        assert_eq!(state.layout, 0);
        assert!(state.unplaced.is_empty());
        assert_eq!(state.fence(first).unwrap().geometry, at_4k[0].geometry);

        state.work_areas = vec![monitor(id, 0, 1920, 1032, 96)];
        state.ensure_layout();
        assert_eq!(state.layout, 1);
        let g = &state.fence(first).unwrap().geometry;
        assert_eq!((g.x, g.y), (20.0, 20.0));
    }

    #[test]
    fn a_monitor_on_another_port_keeps_its_layout() {
        let mut state = state_on(vec![monitor(
            "GSM7787#5&2C948443&0&UID24832",
            0,
            2560,
            1392,
            96,
        )]);
        let before = state.fences()[0].geometry.clone();
        let moved = "GSM7787#7&257C0199&0&UID16640";
        state.work_areas = vec![monitor(moved, 0, 2560, 1392, 96)];
        state.ensure_layout();
        assert_eq!((state.layout, state.config.layouts.len()), (0, 1));
        assert!(state.unplaced.is_empty());
        let after = &state.fences()[0].geometry;
        assert_eq!(after.monitor, moved);
        assert_eq!((after.x, after.y), (before.x, before.y));
        assert_eq!(state.config.layouts[0].fingerprint[0].device_path, moved);
    }

    #[test]
    fn gui_fence_creation_never_makes_the_config_unsavable() {
        // The GUI paths (marquee, menu, templates, tabs, portals) all go through new_fence.
        let mut state = test_state();
        let rect = RECT {
            left: 100,
            top: 100,
            right: 340,
            bottom: 300,
        };
        let mut created = 0;
        for i in 0..80 {
            if state.new_fence(&format!("F{i}"), rect).is_some() {
                created += 1;
            }
        }
        assert!(created > 0);
        assert!(state.fences().len() <= pecofence_core::config_store::MAX_FENCES);
        state
            .store
            .save(&state.config)
            .expect("config must stay savable");
    }

    #[test]
    fn many_desktop_items_never_make_the_config_unsavable() {
        let mut state = test_state();
        let inbox = state.inbox_id().unwrap();
        for i in 0..5200 {
            add_item(&mut state, inbox, &format!("file{i}"));
        }
        state
            .store
            .save(&state.config)
            .expect("config must stay savable");
        assert!(matches!(
            state.store.load(),
            pecofence_core::LoadOutcome::Primary(_)
        ));
    }

    fn add_item(state: &mut AppState, fence: FenceId, name: &str) -> ItemId {
        let item = Item {
            id: uuid::Uuid::new_v4(),
            key: ItemKey::from_path(&format!("C:/Users/Me/Desktop/{name}.txt")),
            origin: Origin::UserDesktop,
            display_name: name.to_string(),
            file_id: None,
            mtime: 1,
            is_folder: false,
            attrs: 0,
            icon_key: IconKey::ByExt(".txt".into()),
            orphaned_since: None,
            size: 0,
            open_count: 0,
            last_opened: None,
            path: None,
        };
        let id = item.id;
        state.catalog.insert(item.key.clone(), id);
        state.config.items.insert(id, item);
        state.config.assign(0, id, fence, AssignedBy::User);
        id
    }

    fn names(state: &AppState, fence: FenceId) -> Vec<String> {
        let f = state.fence(fence).unwrap();
        state
            .items_of(f)
            .iter()
            .map(|i| i.display_name.clone())
            .collect()
    }

    #[test]
    fn host_detach_through_app_state_keeps_content_and_can_restore() {
        let mut state = test_state();
        let (a, b, c) = (
            state.fences()[0].id,
            state.fences()[1].id,
            state.fences()[2].id,
        );
        add_item(&mut state, a, "first");
        add_item(&mut state, b, "second");
        add_item(&mut state, c, "third");
        state.attach_tab(b, a);
        state.attach_tab(c, a);
        let before = state.fences().to_vec();
        assert!(state.fence(a).unwrap().tab_host.is_none());
        state.dirty = false;
        let change = state
            .detach_tab(a)
            .expect("the host must be detachable too");
        assert!(state.dirty);
        assert_eq!(change.remaining_host, b);
        assert_eq!(state.host_of(a), a);
        assert_eq!(state.host_of(c), b);
        assert_eq!(state.active_tab_of(b), c);
        assert_eq!(names(&state, a), vec!["first"]);
        assert_eq!(names(&state, b), vec!["second"]);
        assert_eq!(names(&state, c), vec!["third"]);
        state.dirty = false;
        assert!(state.cancel_tab_detach(&change));
        assert!(state.dirty);
        assert_eq!(state.fences(), before.as_slice());
    }

    /// A window keeps the icons of its other tabs at each tab's own size on its DPI; the tab
    /// it shows reports its items itself, and orphaned records are not shown anywhere.
    #[test]
    fn tab_icons_cover_the_other_tabs_at_their_own_size() {
        let mut state = test_state();
        let (a, b, c) = (
            state.fences()[0].id,
            state.fences()[1].id,
            state.fences()[2].id,
        );
        let d = state
            .new_fence("Elsewhere", rect(600, 600, 300, 200))
            .unwrap();
        let keyed = |state: &mut AppState, fence, name: &str| {
            let id = add_item(state, fence, name);
            let key = IconKey::ByContent {
                path: format!("c:/pics/{name}.jpg"),
                mtime: 1,
            };
            state.config.items.get_mut(&id).unwrap().icon_key = key.clone();
            (id, key)
        };
        keyed(&mut state, a, "shown");
        let (_, big) = keyed(&mut state, b, "big");
        let (_, row) = keyed(&mut state, c, "row");
        let (gone, _) = keyed(&mut state, c, "gone");
        keyed(&mut state, d, "elsewhere");
        state.config.items.get_mut(&gone).unwrap().orphaned_since = Some(1);
        state.fence_mut(b).unwrap().view.icon_size = 96;
        state.fence_mut(c).unwrap().view.layout = pecofence_core::ViewLayout::Details;
        state.attach_tab(b, a);
        state.attach_tab(c, a);
        let mut got = state.tab_icons(a, a, 144);
        got.sort_by_key(|(_, px)| *px);
        assert_eq!(got, vec![(&row, 30), (&big, 144)]);
        // Showing another tab: the host's items join, at the host's own size.
        let got = state.tab_icons(a, b, 96);
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|(k, _)| *k != &big));
    }

    #[test]
    fn workspace_count_includes_portals_deduplicates_paths_and_excludes_orphans() {
        let mut state = test_state();
        let fence = state.fences()[0].id;
        let desktop = add_item(&mut state, fence, "desktop");
        let orphan = add_item(&mut state, fence, "gone");
        state.config.items.get_mut(&orphan).unwrap().orphaned_since = Some(1);
        let mut mirrored = state.config.items[&desktop].clone();
        mirrored.id = uuid::Uuid::new_v4();
        state.portal_items.insert(mirrored.id, mirrored.clone());
        let mut portal_only = mirrored;
        portal_only.id = uuid::Uuid::new_v4();
        portal_only.key = ItemKey::from_path("C:/Projects/portal-only.txt");
        let portal_id = portal_only.id;
        state.portal_items.insert(portal_id, portal_only);
        assert_eq!(state.workspace_item_count(), 2);
        state.portal_items.remove(&portal_id);
        assert_eq!(state.workspace_item_count(), 1);
    }

    #[test]
    fn portal_navigation_preserves_folder_case_from_folded_item_keys() {
        let root = std::env::temp_dir().join(format!("pecofence-portal-{}", uuid::Uuid::new_v4()));
        let nested = root.join("Nested Target").join("日本語 Folder");
        std::fs::create_dir_all(&nested).unwrap();
        let mut state = test_state();
        let fence = state.new_portal_fence(&root, RECT::default()).unwrap();
        let root_title = state.fence(fence).unwrap().title.clone();
        let folded = PathBuf::from(nested.to_string_lossy().to_lowercase());
        assert!(state.portal_enter(fence, &folded));
        assert_eq!(
            state.display_title(state.fence(fence).unwrap()),
            "日本語 Folder"
        );
        assert!(state.portal_up(fence));
        assert_eq!(
            state.display_title(state.fence(fence).unwrap()),
            "Nested Target"
        );
        assert!(state.portal_up(fence));
        assert_eq!(state.display_title(state.fence(fence).unwrap()), root_title);
        assert!(!state.portal_up(fence));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn regular_fence_shows_an_opened_folder_and_gets_its_icons_back() {
        let root = std::env::temp_dir().join(format!("pecofence-browse-{}", uuid::Uuid::new_v4()));
        let nested = root.join("Nested");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(root.join("a.txt"), b"").unwrap();
        let mut state = test_state();
        let fence = state.fences()[0].id;
        let own = add_item(&mut state, fence, "own");
        let shown = |state: &AppState| -> Vec<ItemId> {
            let f = state.fence(fence).unwrap();
            state.items_of(f).iter().map(|it| it.id).collect()
        };
        assert!(!state.can_go_up(fence));
        assert_eq!(shown(&state), [own]);

        assert!(state.open_folder(fence, &root));
        assert_eq!(state.portal_path(fence).as_deref(), Some(root.as_path()));
        assert!(state.can_go_up(fence));
        assert!(!state.portal_navigated(fence));
        assert_eq!(shown(&state).len(), 2);
        assert!(!shown(&state).contains(&own));
        assert_eq!(state.own_item_count(state.fence(fence).unwrap()), 1);
        // The CLI still sees the fence's own icons.
        let held: Vec<ItemId> = state
            .held_items_of(state.fence(fence).unwrap())
            .iter()
            .map(|it| it.id)
            .collect();
        assert_eq!(held, [own]);

        state.set_opened_by(fence, own);
        assert!(state.portal_enter(fence, &nested));
        assert!(state.portal_navigated(fence));
        // Up selects the subfolder just left, then the icon that opened the folder.
        assert_eq!(
            state.up_selection(fence),
            Some(portal_item_id(&nested.to_string_lossy()))
        );
        assert!(state.portal_up(fence));
        assert_eq!(state.portal_path(fence).as_deref(), Some(root.as_path()));
        assert_eq!(state.up_selection(fence), Some(own));
        // At the opened folder, up leaves it: the fence's own icons are back.
        assert!(state.portal_up(fence));
        assert!(!state.can_go_up(fence));
        assert!(state.portal_path(fence).is_none());
        assert_eq!(shown(&state), [own]);
        assert!(!state.portal_up(fence));
        std::fs::remove_dir_all(root).unwrap();
    }

    /// While the 桌面 fence shows a folder opened in it, 「移出栅栏（放回“桌面”）」, a drop on the
    /// bare desktop, a pending route and `item move` only change membership: the desktop file
    /// is never moved into that folder, and the icon is back in view after going up.
    #[test]
    fn moving_icons_to_a_fence_showing_a_folder_only_changes_membership() {
        let root = std::env::temp_dir().join(format!("pecofence-browse-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let mut state = test_state();
        let inbox = state.inbox_id().unwrap();
        let other = state.fences().iter().find(|f| f.id != inbox).unwrap().id;
        let docs = add_item(&mut state, other, "Docs");
        assert!(state.open_folder(inbox, &root));
        assert!(state.portal_path(inbox).is_some());
        assert!(state.move_target_folder(inbox).is_none());

        assert_eq!(state.move_items(&[docs], inbox), 1);
        assert!(state.fence(inbox).unwrap().contains_item(docs));
        assert!(!state.fence(other).unwrap().contains_item(docs));
        assert!(state.portal_up(inbox));
        let f = state.fence(inbox).unwrap();
        assert!(state.items_of(f).iter().any(|it| it.id == docs));

        // A folder portal is a folder: moving into it is a file move, not a membership change.
        let portal = state.new_portal_fence(&root, RECT::default()).unwrap();
        assert_eq!(state.move_target_folder(portal), state.portal_path(portal));
        assert_eq!(state.move_items(&[docs], portal), 0);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn an_opened_folder_replaces_the_one_before_and_portals_keep_to_their_own() {
        let base = std::env::temp_dir().join(format!("pecofence-browse-{}", uuid::Uuid::new_v4()));
        let root = base.join("Portal");
        let sub = root.join("Sub");
        let outside = base.join("Outside");
        let other = base.join("Other");
        for dir in [&sub, &outside, &other] {
            std::fs::create_dir_all(dir).unwrap();
        }
        let mut state = test_state();
        let fence = state.fences()[0].id;
        assert!(state.open_folder(fence, &outside));
        // A folder opened from there replaces it; up goes straight back to the fence's icons.
        assert!(state.open_folder(fence, &other));
        assert_eq!(state.portal_path(fence).as_deref(), Some(other.as_path()));
        assert_eq!(state.display_title(state.fence(fence).unwrap()), "Other");
        assert!(!state.portal_home(fence));
        assert!(state.portal_up(fence));
        assert!(state.portal_path(fence).is_none());
        assert!(!state.open_folder(fence, &base.join("missing")));
        assert!(!state.can_go_up(fence));

        // A folder portal shows its own folder and its subfolders only.
        let portal = state.new_portal_fence(&root, RECT::default()).unwrap();
        assert!(state.portal_enter(portal, &sub));
        assert!(!state.portal_enter(portal, &outside));
        assert!(!state.open_folder(portal, &outside));
        assert_eq!(state.portal_path(portal).as_deref(), Some(sub.as_path()));
        assert!(state.portal_home(portal));
        assert_eq!(state.portal_path(portal).as_deref(), Some(root.as_path()));
        assert!(!state.can_go_up(portal));
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn portal_reports_a_missing_folder_and_recovers() {
        let root = std::env::temp_dir().join(format!("pecofence-portal-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let mut state = test_state();
        let fence = state.new_portal_fence(&root, RECT::default()).unwrap();
        assert!(!state.portal_missing(fence));
        // Nothing to list before or after: only the missing flag can make this a change.
        std::fs::remove_dir(&root).unwrap();
        assert!(state.refresh_portal(fence));
        assert!(state.portal_missing(fence));
        std::fs::create_dir_all(&root).unwrap();
        assert!(state.refresh_portal(fence));
        assert!(!state.portal_missing(fence));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reorder_items_places_before_display_index() {
        let mut state = test_state();
        let fence = state.fences()[0].id;
        assert_eq!(state.fence(fence).unwrap().view.sort, SortMode::Manual);
        let a = add_item(&mut state, fence, "a");
        let b = add_item(&mut state, fence, "b");
        let c = add_item(&mut state, fence, "c");
        assert_eq!(names(&state, fence), ["a", "b", "c"]);
        // c before a.
        assert!(state.reorder_items(fence, &[c], 0));
        assert_eq!(names(&state, fence), ["c", "a", "b"]);
        // a,b to the end: already there -> no change.
        assert!(!state.reorder_items(fence, &[a, b], 3));
        assert_eq!(names(&state, fence), ["c", "a", "b"]);
        // Dropping onto its own slot (index right after itself) is a no-op.
        assert!(!state.reorder_items(fence, &[c], 1));
        // b before c.
        assert!(state.reorder_items(fence, &[b], 0));
        assert_eq!(names(&state, fence), ["b", "c", "a"]);
        // Reversed view: display order is reversed, drop index is a display index.
        state.set_reverse(fence, true);
        assert_eq!(names(&state, fence), ["a", "c", "b"]);
        assert!(state.reorder_items(fence, &[b], 0));
        assert_eq!(names(&state, fence), ["b", "a", "c"]);
        // A sorted fence refuses.
        state.set_sort(fence, SortMode::Name);
        assert!(!state.reorder_items(fence, &[a], 0));
    }

    fn desktop_entry(name: &str) -> DesktopEntry {
        DesktopEntry {
            path: PathBuf::from(format!("C:/Users/Me/Desktop/{name}")),
            file_name: name.to_string(),
            origin: EntryOrigin::UserDesktop,
            is_folder: false,
            attributes: 0,
            mtime: 1,
            size: 0,
            created: 1,
        }
    }

    /// Feedback #25: `ABC.zip` was opened and listed as `abc.zip`.
    #[test]
    fn items_keep_the_on_disk_spelling_and_match_any_case() {
        let mut state = test_state();
        let fence = state.fences()[0].id;
        // An item saved by an older version has only the folded key, until the sync.
        let old = add_item(&mut state, fence, "readme");
        assert_eq!(
            state.item(old).unwrap().path(),
            Some(r"c:\users\me\desktop\readme.txt")
        );
        state.sync_desktop(&[desktop_entry("README.txt"), desktop_entry("ABC.zip")]);
        assert_eq!(
            state.item(old).unwrap().path(),
            Some("C:/Users/Me/Desktop/README.txt")
        );
        let id = state
            .item_by_path(Path::new(r"c:\users\me\desktop\abc.ZIP"))
            .unwrap();
        assert_eq!(
            state.item(id).unwrap().path(),
            Some("C:/Users/Me/Desktop/ABC.zip")
        );
        // A case-only rename keeps the item and hands out the new spelling.
        let report = state.sync_desktop(&[desktop_entry("README.txt"), desktop_entry("Abc.zip")]);
        assert_eq!((report.added, report.removed, report.updated), (0, 0, 1));
        assert_eq!(
            state.item(id).unwrap().path(),
            Some("C:/Users/Me/Desktop/Abc.zip")
        );
        assert!(state.rename_item(
            Path::new("C:/Users/Me/Desktop/Abc.zip"),
            Path::new("C:/Users/Me/Desktop/aBC.zip")
        ));
        assert_eq!(
            state.item(id).unwrap().path(),
            Some("C:/Users/Me/Desktop/aBC.zip")
        );
    }

    #[test]
    fn safe_save_keeps_the_document_in_its_fence() {
        let desk = |name: &str| PathBuf::from(format!("C:/Users/Me/Desktop/{name}"));
        let mut state = test_state();
        let (inbox, work) = (state.inbox_id().unwrap(), state.fences()[1].id);
        let doc = add_item(&mut state, work, "report");
        // Word: the document goes to ~WRL0001.tmp, the new copy ~WRD0000.tmp takes its name.
        state.rename_item(&desk("report.txt"), &desk("~WRL0001.tmp"));
        state.rename_item(&desk("~WRD0000.tmp"), &desk("report.txt"));
        state.sync_desktop(&[desktop_entry("report.txt")]);
        assert_eq!(state.item_id_for_path(&desk("report.txt")), Some(doc));
        assert_eq!(state.config.fence_of_item(0, doc), Some(work));
        // Excel: a catalogued temporary copy without an extension is renamed over the document.
        let copy = add_item(&mut state, inbox, "6A1B2C3D");
        state.rename_item(&desk("report.txt"), &desk("A1B2C3D4.tmp"));
        state.rename_item(&desk("6A1B2C3D.txt"), &desk("report.txt"));
        state.sync_desktop(&[desktop_entry("report.txt")]);
        assert_eq!(state.item_id_for_path(&desk("report.txt")), Some(doc));
        assert_eq!(state.config.fence_of_item(0, doc), Some(work));
        assert!(state.config.items[&copy].orphaned_since.is_some());
        // The renames in two batches, with a sync between them and the copy catalogued.
        let copy = add_item(&mut state, inbox, "~WRD0001");
        state.rename_item(&desk("report.txt"), &desk("~WRL0002.tmp"));
        state.sync_desktop(&[desktop_entry("~WRD0001.txt")]);
        state.rename_item(&desk("~WRD0001.txt"), &desk("report.txt"));
        state.sync_desktop(&[desktop_entry("report.txt")]);
        assert_eq!(state.item_id_for_path(&desk("report.txt")), Some(doc));
        assert_eq!(state.config.fence_of_item(0, doc), Some(work));
        assert!(state.config.items[&copy].orphaned_since.is_some());
        // A sync that catches the fresh copy mid-save puts it in no fence. Once the file has
        // lasted a minute, the next sync (nothing else changed) puts it in a fence and reports
        // that, so housekeeping refreshes the windows.
        let mut fresh = desktop_entry("~WRD0002.tmp");
        fresh.created = pecofence_core::now_unix();
        state.sync_desktop(&[desktop_entry("report.txt"), fresh.clone()]);
        let tmp = state.item_id_for_path(&desk("~WRD0002.tmp")).unwrap();
        assert!(state.fences().iter().all(|f| !f.contains_item(tmp)));
        fresh.created -= pecofence_core::rules::TMP_SETTLE_SECS as i64;
        let report = state.sync_desktop(&[desktop_entry("report.txt"), fresh]);
        assert!(report.changed(), "a routed .tmp must show up in the report");
        assert_eq!(state.config.fence_of_item(0, tmp), Some(inbox));
        // A .tmp that has been around for a while, or a .bak, shows like any other file.
        state.sync_desktop(&[
            desktop_entry("report.txt"),
            desktop_entry("leftover.tmp"),
            desktop_entry("notes.bak"),
        ]);
        for name in ["leftover.tmp", "notes.bak"] {
            let id = state.item_id_for_path(&desk(name)).unwrap();
            assert!(
                state.config.fence_of_item(0, id).is_some(),
                "{name} shows nowhere"
            );
        }
        // AutoCAD: the drawing is renamed to .bak and the new copy (catalogued by a sync while
        // it was written) is renamed over its name. The drawing keeps its record and fence; the
        // .bak is a file like any other.
        let mut sav = desktop_entry("sav1.tmp");
        sav.created = pecofence_core::now_unix();
        state.sync_desktop(&[desktop_entry("report.txt"), sav]);
        assert!(!state.rename_item(&desk("report.txt"), &desk("report.bak")));
        assert!(!state.rename_item(&desk("sav1.tmp"), &desk("report.txt")));
        state.sync_desktop(&[desktop_entry("report.txt"), desktop_entry("report.bak")]);
        assert_eq!(state.item_id_for_path(&desk("report.txt")), Some(doc));
        assert_eq!(state.config.fence_of_item(0, doc), Some(work));
        let bak = state.item_id_for_path(&desk("report.bak")).unwrap();
        assert_eq!(state.config.fence_of_item(0, bak), Some(inbox));
    }

    /// #69 / #71: the shortcut rule is disabled and type rules plus a final 「普通文件」 rule are
    /// applied. With 「留在当前栅栏」 the programs and shortcuts stay where that rule put them;
    /// with the default they fall back to 「桌面」 as before. A new item still goes to 「桌面」.
    #[test]
    fn unmatched_items_can_stay_in_their_fence_when_the_rules_are_applied() {
        use pecofence_core::rules::{Rule, TypeCategory};
        let mut state = test_state();
        let (programs, folders, docs) = (
            state.fences()[0].id,
            state.fences()[1].id,
            state.fences()[2].id,
        );
        let inbox = state.inbox_id().unwrap();
        let [archives, images, other] =
            ["压缩", "图片", "其它"].map(|t| state.new_fence(t, rect(0, 0, 300, 200)).unwrap());
        let mut folder = desktop_entry("Projects");
        folder.is_folder = true;
        let mut entries: Vec<DesktopEntry> = [
            "Notepad.lnk",
            "tool.exe",
            "site.url",
            "backup.zip",
            "report.docx",
            "table.tsv",
            "photo.png",
            "query.sql",
            "page.html",
            "data.qwe",
        ]
        .map(desktop_entry)
        .into();
        entries.push(folder);
        // The first-run rules put the programs and shortcuts into 程序.
        state.sync_desktop(&entries);
        let id = |state: &AppState, name: &str| {
            state
                .item_id_for_path(Path::new(&format!("C:/Users/Me/Desktop/{name}")))
                .unwrap()
        };
        let fence_of =
            |state: &AppState, name: &str| state.config.fence_of_item(0, id(state, name));
        assert_eq!(fence_of(&state, "Notepad.lnk"), Some(programs));
        assert_eq!(fence_of(&state, "query.sql"), Some(inbox));
        let type_rule = |cat, to| Rule::new("t", Target::Fence(to), vec![Cond::Type(vec![cat])]);
        let rules = &mut state.config.rules;
        rules.list[0].enabled = false;
        rules.list.truncate(1);
        rules.list.extend([
            type_rule(TypeCategory::Archives, archives),
            type_rule(TypeCategory::Folders, folders),
            type_rule(TypeCategory::Documents, docs),
            type_rule(TypeCategory::Images, images),
            Rule::new("其它", Target::Fence(other), vec![Cond::PlainFiles]),
        ]);
        let shortcuts = ["Notepad.lnk", "tool.exe", "site.url"];
        let plan = state.plan_rules(&entries, false);
        for name in shortcuts {
            let m = plan.iter().find(|m| m.item == id(&state, name)).unwrap();
            assert_eq!((m.from, m.to), (Some(programs), inbox), "{name}");
        }
        state.config.rules.unmatched = Unmatched::Stay;
        let plan = state.plan_rules(&entries, false);
        assert!(
            plan.iter()
                .all(|m| shortcuts.iter().all(|n| m.item != id(&state, n)))
        );
        assert_eq!(state.apply_rules_all(&entries), plan.len());
        for (name, fence) in [
            ("Notepad.lnk", programs),
            ("tool.exe", programs),
            ("site.url", programs),
            ("backup.zip", archives),
            ("report.docx", docs),
            ("table.tsv", docs),
            ("photo.png", images),
            ("query.sql", other),
            ("page.html", other),
            ("data.qwe", other),
            ("Projects", folders),
        ] {
            assert_eq!(fence_of(&state, name), Some(fence), "{name}");
        }
        entries.push(desktop_entry("New.lnk"));
        state.sync_desktop(&entries);
        assert_eq!(fence_of(&state, "New.lnk"), Some(inbox));
        assert!(state.plan_rules(&entries, false).is_empty());
    }

    /// b.txt deleted, then a.txt renamed to b.txt within the week: the deleted file's record
    /// must not claim the renamed file and pull it into the old file's fence.
    #[test]
    fn a_rename_over_a_recently_deleted_name_keeps_the_item_in_its_fence() {
        let desk = |name: &str| PathBuf::from(format!("C:/Users/Me/Desktop/{name}"));
        let mut state = test_state();
        let (inbox, work) = (state.inbox_id().unwrap(), state.fences()[1].id);
        let gone = add_item(&mut state, inbox, "b");
        state.config.items.get_mut(&gone).unwrap().orphaned_since = Some(1);
        let kept = add_item(&mut state, work, "a");
        state.forget_orphan_at(&desk("b.txt"));
        assert!(state.rename_item(&desk("a.txt"), &desk("b.txt")));
        state.sync_desktop(&[desktop_entry("b.txt")]);
        assert_eq!(state.item_id_for_path(&desk("b.txt")), Some(kept));
        assert_eq!(state.config.fence_of_item(0, kept), Some(work));
        assert!(!state.config.items.contains_key(&gone));
        assert!(state.fences().iter().all(|f| !f.contains_item(gone)));
        // A live file of that name keeps its record; the rename is refused as before.
        let live = add_item(&mut state, inbox, "c");
        state.forget_orphan_at(&desk("c.txt"));
        assert!(!state.rename_item(&desk("b.txt"), &desk("c.txt")));
        assert_eq!(state.item_id_for_path(&desk("c.txt")), Some(live));
    }

    /// The public desktop could not be read: its files stay, the user's deleted one goes.
    #[test]
    fn a_desktop_folder_that_could_not_be_read_orphans_nothing() {
        let mut state = test_state();
        let mut public = desktop_entry("shared.txt");
        public.path = PathBuf::from("C:/Users/Public/Desktop/shared.txt");
        public.origin = EntryOrigin::PublicDesktop;
        state.sync_desktop(&[desktop_entry("mine.txt"), public.clone()]);
        let shared = state.item_id_for_path(&public.path).unwrap();
        let mine = state
            .item_id_for_path(Path::new("C:/Users/Me/Desktop/mine.txt"))
            .unwrap();
        let report = state.sync_desktop_partly_read(&[], &[EntryOrigin::PublicDesktop]);
        assert_eq!(report.removed, 1);
        assert!(state.config.items[&shared].orphaned_since.is_none());
        assert!(state.config.items[&mine].orphaned_since.is_some());
    }

    #[test]
    fn a_record_reused_by_a_folder_of_the_same_name_becomes_a_folder() {
        let path = PathBuf::from("C:/Users/Me/Desktop/proj");
        let mut state = test_state();
        state.sync_desktop(&[desktop_entry("proj")]);
        let id = state.item_id_for_path(&path).unwrap();
        assert!(!state.config.items[&id].is_folder);
        // Deleted, then a folder of that name within the week: the record is reused.
        state.sync_desktop(&[]);
        let mut folder = desktop_entry("proj");
        folder.is_folder = true;
        state.sync_desktop(&[folder]);
        assert_eq!(state.item_id_for_path(&path), Some(id));
        assert!(state.config.items[&id].is_folder);
    }

    /// Issue #39: the inbox cannot be deleted, but 为空时自动隐藏 keeps an empty one out of sight.
    /// Feedback #88: every other fence can hide the same way.
    #[test]
    fn an_empty_fence_hides_only_when_asked_and_on_its_own() {
        let mut state = test_state();
        let inbox = state.inbox_id().unwrap();
        let other = state.fences().iter().find(|f| f.id != inbox).unwrap().id;
        let item = add_item(&mut state, inbox, "new");
        state.set_hide_when_empty(inbox, true);
        assert!(state.hidden_fences().is_empty(), "it still holds an item");
        state.move_items(&[item], other);
        assert_eq!(state.hidden_fences(), vec![inbox]);
        // A deleted file stays catalogued for a while (orphaned) but is not shown.
        let gone = add_item(&mut state, inbox, "gone");
        assert!(state.hidden_fences().is_empty());
        state.config.items.get_mut(&gone).unwrap().orphaned_since = Some(1);
        assert_eq!(state.hidden_fences(), vec![inbox]);
        // As a tab, or with tabs of its own, it is part of a window that stays.
        state.attach_tab(other, inbox);
        assert!(state.hidden_fences().is_empty());
        state.detach_tab(other);
        assert_eq!(state.hidden_fences(), vec![inbox]);
        state.set_hide_when_empty(inbox, false);
        assert!(state.hidden_fences().is_empty());
        // Any other fence: gone with its last item, back with the next one.
        state.set_hide_when_empty(other, true);
        assert!(state.hidden_fences().is_empty());
        state.move_items(&[item], inbox);
        assert_eq!(state.hidden_fences(), vec![other]);
        state.move_items(&[item], other);
        assert!(state.hidden_fences().is_empty());
    }

    /// Feedback #4 and #21: with the Windows icons shown, the inbox can stay out of sight even
    /// while it holds items; hiding the Windows icons again brings it back.
    #[test]
    fn the_inbox_hides_while_the_windows_icons_show_when_asked() {
        let mut state = test_state();
        let inbox = state.inbox_id().unwrap();
        let other = state.fences().iter().find(|f| f.id != inbox).unwrap().id;
        add_item(&mut state, inbox, "new");
        state.config.settings.hide_inbox_when_icons_shown = true;
        assert!(state.config.settings.hide_real_icons);
        assert!(
            state.hidden_fences().is_empty(),
            "the Windows icons are hidden"
        );
        state.config.settings.hide_real_icons = false;
        assert_eq!(state.hidden_fences(), vec![inbox]);
        // As a tab, or with tabs of its own, it is part of a window that stays.
        state.attach_tab(other, inbox);
        assert!(state.hidden_fences().is_empty());
        state.detach_tab(other);
        assert_eq!(state.hidden_fences(), vec![inbox]);
        state.config.settings.hide_inbox_when_icons_shown = false;
        assert!(state.hidden_fences().is_empty());
    }

    #[test]
    fn dir_prefix_tolerates_drive_roots() {
        assert_eq!(dir_prefix("d:\\"), "d:\\");
        assert_eq!(dir_prefix("d:\\users"), "d:\\users\\");
        assert!("d:\\users".starts_with(&dir_prefix("d:\\")));
        assert!(!"d:\\usersx".starts_with(&dir_prefix("d:\\users")));
    }
}
