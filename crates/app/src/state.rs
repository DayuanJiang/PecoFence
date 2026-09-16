//! Application state: config + layout selection + desktop item catalog + rule routing.

use pecofence_core::geometry::{self, PxRect, WorkArea};
use pecofence_core::rules::{Decision, RuleSet, Target, Template};
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
    /// Portal fence → subfolder it has navigated into (absent = its root folder).
    portal_cwd: HashMap<FenceId, PathBuf>,
    pub work_areas: Vec<WorkArea>,
    pub first_run: bool,
    pub recovered_from: Option<PathBuf>,
}

/// Summary of a desktop sync pass.
#[derive(Debug, Default)]
pub struct SyncReport {
    pub added: usize,
    pub removed: usize,
    pub updated: usize,
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
        let (config, first_run, recovered_from) = match store.load() {
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
            work_areas,
            first_run,
            recovered_from,
        };
        state.rebuild_catalog();
        state.ensure_layout();
        state.normalize_tabs();
        state
    }

    pub fn config_path(&self) -> PathBuf {
        self.store.primary_path()
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
        // never let an import / backup file wipe them. Merge the file's by id, oldest first.
        let mut snaps = std::mem::take(&mut self.config.snapshots);
        for s in std::mem::take(&mut config.snapshots) {
            if !snaps.iter().any(|x| x.id == s.id) {
                snaps.push(s);
            }
        }
        snaps.sort_by_key(|s| s.ts);
        while snaps.len() > pecofence_core::MAX_SNAPSHOTS {
            snaps.remove(0);
        }
        config.snapshots = snaps;
        self.config = config;
        self.rebuild_catalog();
        self.layout = 0;
        self.ensure_layout();
        self.normalize_tabs();
        self.portal_cwd.clear();
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
    /// backup's eviction at `MAX_SNAPSHOTS` can never remove it. Returns false (and saves
    /// nothing) when `id` is unknown.
    pub fn restore_snapshot_with_backup(&mut self, id: uuid::Uuid, backup_name: &str) -> bool {
        let Some(snap) = self.config.snapshots.iter().find(|s| s.id == id).cloned() else {
            return false;
        };
        self.save_snapshot(backup_name);
        self.apply_snapshot_layouts(snap.layouts);
        true
    }

    fn apply_snapshot_layouts(&mut self, layouts: Vec<pecofence_core::Layout>) {
        self.config.layouts = layouts;
        self.layout = 0;
        self.ensure_layout();
        self.normalize_tabs();
        self.portal_cwd.clear();
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

    fn device_paths(&self) -> Vec<String> {
        self.work_areas
            .iter()
            .map(|w| w.device_path.clone())
            .collect()
    }

    /// Selects the layout for the current monitors, creating the default one on first use.
    pub fn ensure_layout(&mut self) {
        let paths = self.device_paths();
        if let Some(i) = self.config.layout_for(&paths) {
            self.layout = i;
            return;
        }
        // Reuse the first existing layout's fences if monitors changed (best effort), else
        // build the wizard default.
        let fences = if let Some(existing) = self.config.layouts.first() {
            existing.fences.clone()
        } else {
            self.default_fences()
        };
        let fingerprint = self
            .work_areas
            .iter()
            .map(|w| MonitorIdentity {
                device_path: w.device_path.clone(),
                work_dip: [w.width_dip(), w.height_dip()],
                dpi: w.dpi,
            })
            .collect();
        self.config.layouts.push(Layout {
            fingerprint,
            fences,
        });
        self.layout = self.config.layouts.len() - 1;
        self.dirty = true;
    }

    /// Stardock-style first-run layout: right column 程序 / 文件夹 / 文件与文档 / 桌面(inbox).
    fn default_fences(&mut self) -> Vec<Fence> {
        let work = self
            .work_areas
            .iter()
            .find(|w| w.device_path.ends_with("DISPLAY1") || true)
            .cloned()
            .unwrap_or(WorkArea {
                device_path: "primary".into(),
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

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
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
                tracing::error!(error = %e, "config save failed");
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
            self.config.rules.evaluate(&self.facts_for(entry))
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
        }
    }

    /// Reconciles the item table with the current desktop contents. New items are routed by the
    /// rule set; missing items are marked orphaned (membership kept) and garbage-collected
    /// after seven days.
    pub fn sync_desktop(&mut self, entries: &[DesktopEntry]) -> SyncReport {
        let mut report = SyncReport::default();
        let now = pecofence_core::now_unix();
        let mut seen: HashSet<ItemId> = HashSet::new();

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
                    if changed {
                        report.updated += 1;
                        self.dirty = true;
                    }
                }
                // An item that exists but is in no fence (e.g. its fence was deleted) → route.
                if self.config.fence_of_item(self.layout, id).is_none() {
                    let decision = self.route_decision(entry);
                    if let Some((fence, by)) = self.target_fence(decision) {
                        self.config.assign(self.layout, id, fence, by);
                        self.dirty = true;
                    }
                }
                continue;
            }
            // New item.
            let display_name = shell::display_name(&entry.path).unwrap_or_else(|| {
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
        let mut to_remove = Vec::new();
        for (id, item) in self.config.items.iter_mut() {
            if seen.contains(id) {
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
            if let Some(item) = self.config.items.remove(&id) {
                self.catalog.remove(&item.key);
            }
            for f in &mut self.config.layouts[self.layout].fences {
                f.items.retain(|r| r.item_id != id);
            }
            self.dirty = true;
        }
        report
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

    /// Items of `fence` in display order (orphans hidden).
    pub fn items_of(&self, fence: &Fence) -> Vec<&Item> {
        let mut items: Vec<(Option<&ItemRef>, &Item)> = if fence.kind == FenceKind::FolderPortal {
            self.portal_members
                .get(&fence.id)
                .map(|ids| {
                    ids.iter()
                        .filter_map(|i| self.portal_items.get(i).map(|it| (None, it)))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            fence
                .items
                .iter()
                .filter_map(|r| self.config.items.get(&r.item_id).map(|it| (Some(r), it)))
                .filter(|(_, it)| it.orphaned_since.is_none())
                .collect()
        };
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
    pub fn portal_path(&self, id: FenceId) -> Option<PathBuf> {
        let root = self.portal_root(id)?;
        Some(self.portal_cwd.get(&id).cloned().unwrap_or(root))
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

    /// Navigates a portal into `dir` (must be inside its root). Returns false if refused.
    pub fn portal_enter(&mut self, id: FenceId, dir: &Path) -> bool {
        let Some(root) = self.portal_root(id) else {
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

    /// One folder up (never above the root). Returns false when already at the root.
    pub fn portal_up(&mut self, id: FenceId) -> bool {
        let Some(cur) = self.portal_cwd.get(&id).cloned() else {
            return false;
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

    /// Back to the root folder.
    pub fn portal_home(&mut self, id: FenceId) -> bool {
        if self.portal_cwd.remove(&id).is_some() {
            self.refresh_portal(id);
            true
        } else {
            false
        }
    }

    /// The portal (if any) whose current folder contains this runtime item.
    pub fn portal_of_item(&self, item: ItemId) -> Option<FenceId> {
        self.portal_members
            .iter()
            .find(|(_, ids)| ids.contains(&item))
            .map(|(f, _)| *f)
    }

    pub fn set_portal_navigate(&mut self, id: FenceId, on: bool) {
        if let Some(f) = self.fence_mut(id)
            && f.portal_navigate != on
        {
            f.portal_navigate = on;
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

    /// Title shown for a fence: a navigated portal shows the current folder's name.
    pub fn display_title(&self, f: &Fence) -> String {
        match self.portal_cwd.get(&f.id) {
            Some(cwd) => cwd
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| f.title.clone()),
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
        let mut old = Vec::with_capacity(old_ids.len());
        for i in &old_ids {
            let shared = self.portal_members.values().any(|v| v.contains(i));
            let item = if shared {
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
        let old_items = self.evict_portal_members(id);
        // Display names survive a re-read: asking the shell costs several ms per item on a
        // OneDrive folder, and an unchanged file (same path, same mtime) has the same name.
        let known: HashMap<(ItemKey, i64), String> = old_items
            .iter()
            .map(|it| ((it.key.clone(), it.mtime), it.display_name.clone()))
            .collect();
        let mut ids = Vec::new();
        let mut new_items = Vec::new();
        for entry in shell::enumerate_folder(&dir) {
            let path_str = entry.path.to_string_lossy().to_string();
            let item_id = portal_item_id(&path_str);
            let (icon_key, _) =
                crate::icons::icon_key_for(&entry.path, entry.is_folder, entry.mtime);
            let key = ItemKey::from_path(&path_str);
            let display_name = known
                .get(&(key.clone(), entry.mtime))
                // ItemKey ignores case. A case-only rename leaves that key and mtime
                // unchanged, but the old display spelling must not be reused.
                .filter(|name| entry.file_name.starts_with(name.as_str()))
                .cloned()
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
            };
            new_items.push(item.clone());
            self.portal_items.insert(item_id, item);
            ids.push(item_id);
        }
        let changed = old_items != new_items;
        let ms = started.elapsed().as_secs_f32() * 1000.0;
        if changed {
            tracing::info!(%id, folder = %dir.display(), count = ids.len(), ms, "portal refreshed");
        } else {
            tracing::debug!(%id, count = ids.len(), ms, "portal re-read, unchanged");
        }
        self.portal_members.insert(id, ids);
        changed
    }

    /// Re-reads every portal folder (startup, watcher batches). Returns the fences that changed.
    pub fn refresh_all_portals(&mut self) -> Vec<FenceId> {
        let portals: Vec<FenceId> = self
            .fences()
            .iter()
            .filter(|f| f.kind == FenceKind::FolderPortal)
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

    /// Re-keys an item after a rename so it stays in its fence. Returns false if unknown.
    pub fn rename_item(&mut self, old: &Path, new: &Path) -> bool {
        let old_key = ItemKey::from_path(&old.to_string_lossy());
        let Some(id) = self.catalog.remove(&old_key) else {
            return false;
        };
        let new_key = ItemKey::from_path(&new.to_string_lossy());
        let Some(item) = self.config.items.get_mut(&id) else {
            return false;
        };
        let is_folder = std::fs::metadata(new)
            .map(|m| m.is_dir())
            .unwrap_or(item.is_folder);
        item.key = new_key.clone();
        item.is_folder = is_folder;
        item.display_name = shell::display_name(new).unwrap_or_else(|| {
            new.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        });
        let (icon_key, _) = crate::icons::icon_key_for(new, is_folder, item.mtime);
        item.icon_key = icon_key;
        item.orphaned_since = None;
        // A stale item already keyed by the new path (e.g. deleted then re-created under the old
        // name) would otherwise leave two items with one key; drop it first.
        if let Some(&other) = self.catalog.get(&new_key)
            && other != id
        {
            self.config.items.remove(&other);
            for layout in &mut self.config.layouts {
                for f in &mut layout.fences {
                    f.items.retain(|r| r.item_id != other);
                }
            }
        }
        self.catalog.insert(new_key, id);
        self.dirty = true;
        true
    }

    pub fn move_items(&mut self, items: &[ItemId], to: FenceId) -> usize {
        // Portal fences show a folder, not desktop membership: nothing can be moved into them,
        // and their (runtime) items cannot be moved out.
        if self.portal_path(to).is_some() {
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
        let mut moved = 0;
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
            if let Some((fence, by)) = self.target_fence(decision)
                && self.config.assign(self.layout, id, fence, by).is_some()
            {
                moved += 1;
            }
        }
        if moved > 0 {
            self.dirty = true;
        }
        moved
    }

    /// Re-files `ids` by the rules as if they had just arrived on the desktop (the user dragged
    /// them out of a fence onto the desktop: an explicit "let go", so an earlier manual
    /// placement no longer pins them). Unmatched items stay where they are (the inbox).
    /// Returns the fences whose membership changed.
    pub fn apply_rules_to(&mut self, ids: &[ItemId], entries: &[DesktopEntry]) -> Vec<FenceId> {
        let mut touched = Vec::new();
        for entry in entries {
            if entry.origin == EntryOrigin::Namespace {
                continue;
            }
            let key = ItemKey::from_path(&entry.path.to_string_lossy());
            let Some(&id) = self.catalog.get(&key) else {
                continue;
            };
            if !ids.contains(&id) {
                continue;
            }
            let from = self
                .fences()
                .iter()
                .find(|f| f.contains_item(id))
                .map(|f| f.id);
            let decision = self.config.rules.evaluate(&self.facts_for(entry));
            if let Some((fence, by)) = self.target_fence(decision)
                && Some(fence) != from
                && self.config.assign(self.layout, id, fence, by).is_some()
            {
                touched.extend(from);
                touched.push(fence);
            }
        }
        if !touched.is_empty() {
            self.dirty = true;
            touched.sort_unstable();
            touched.dedup();
        }
        touched
    }

    // ---- fences -----------------------------------------------------------------------------

    pub fn set_fence_bounds(&mut self, id: FenceId, rect: RECT, rolled: bool, expanded_h_px: i32) {
        let px = PxRect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: if rolled {
                rect.top + expanded_h_px.max(1)
            } else {
                rect.bottom
            },
        };
        let (cx, cy) = px.center();
        let work = self
            .work_areas
            .iter()
            .find(|w| cx >= w.left && cx < w.right && cy >= w.top && cy < w.bottom)
            .or(self.work_areas.first())
            .cloned();
        let Some(work) = work else { return };
        let geo = geometry::normalize(px, &work);
        if let Some(f) = self.fence_mut(id) {
            if f.geometry != geo || f.rolled_up != rolled {
                f.expanded_h = geo.h;
                f.geometry = geo;
                f.rolled_up = rolled;
                self.dirty = true;
            }
        }
    }

    /// Physical rectangle for a fence on the current monitors (expanded height).
    pub fn fence_px_rect(&self, fence: &Fence) -> PxRect {
        let work = geometry::resolve_work_area(&fence.geometry, &self.work_areas)
            .cloned()
            .unwrap_or(WorkArea {
                device_path: String::new(),
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

    /// Per-fence appearance override (None = follow the global settings).
    pub fn set_appearance(
        &mut self,
        id: FenceId,
        backdrop: Option<pecofence_core::Backdrop>,
        opacity: Option<f32>,
    ) {
        if let Some(f) = self.fence_mut(id) {
            let mut next = f.appearance.clone().unwrap_or_default();
            next.backdrop = backdrop;
            next.opacity = opacity;
            let next = (next != pecofence_core::AppearanceOverride::default()).then_some(next);
            if f.appearance != next {
                f.appearance = next;
                self.dirty = true;
            }
        }
    }

    /// Per-fence colour wash / title colour / title size (None fields = theme defaults).
    pub fn set_style(
        &mut self,
        id: FenceId,
        tint_rgb: Option<[u8; 3]>,
        title_rgb: Option<[u8; 3]>,
        title_size: Option<pecofence_core::TitleSize>,
    ) {
        if let Some(f) = self.fence_mut(id) {
            let mut next = f.appearance.clone().unwrap_or_default();
            next.tint_rgb = tint_rgb;
            next.title_rgb = title_rgb;
            next.title_size = title_size;
            let next = (next != pecofence_core::AppearanceOverride::default()).then_some(next);
            if f.appearance != next {
                f.appearance = next;
                self.dirty = true;
            }
        }
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

    pub fn set_sort(&mut self, id: FenceId, sort: SortMode) {
        if let Some(f) = self.fence_mut(id)
            && f.view.sort != sort
        {
            f.view.sort = sort;
            self.dirty = true;
        }
    }

    pub fn new_fence(&mut self, title: &str, rect: RECT) -> Option<FenceId> {
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
        self.config.rules.list.insert(0, template.rule(id));
        self.dirty = true;
        Ok(id)
    }

    /// Deletes a fence; its items go back to the inbox. The inbox itself cannot be deleted.
    pub fn delete_fence(&mut self, id: FenceId) -> bool {
        let Some(inbox) = self.inbox_id() else {
            return false;
        };
        if inbox == id {
            return false;
        }
        let Some(pos) = self.fences().iter().position(|f| f.id == id) else {
            return false;
        };
        let removed = self.config.layouts[self.layout].fences.remove(pos);
        self.portal_cwd.remove(&id);
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

    fn test_state() -> AppState {
        let work = WorkArea {
            device_path: "test".into(),
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1040,
            dpi: 96,
            mon_left: 0,
            mon_top: 0,
            mon_right: 1920,
            mon_bottom: 1040,
        };
        let dir = std::env::temp_dir().join(format!(
            "pecofence-state-test-{}-{}",
            std::process::id(),
            pecofence_core::now_unix()
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
            work_areas: vec![work],
            first_run: true,
            recovered_from: None,
        };
        state.ensure_layout();
        state
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

    #[test]
    fn dir_prefix_tolerates_drive_roots() {
        assert_eq!(dir_prefix("d:\\"), "d:\\");
        assert_eq!(dir_prefix("d:\\users"), "d:\\users\\");
        assert!("d:\\users".starts_with(&dir_prefix("d:\\")));
        assert!(!"d:\\usersx".starts_with(&dir_prefix("d:\\users")));
    }
}
