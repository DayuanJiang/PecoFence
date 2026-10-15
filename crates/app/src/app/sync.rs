//! Desktop-folder synchronisation: fs-watcher batches, availability-guarded sync, manual refresh.

use super::*;

/// Starts a watcher on `dir` whose batches land in `pending` and poke the control window.
pub(super) fn start_watcher(
    dir: &Path,
    control: HWND,
    pending: Arc<Mutex<Vec<FsEvent>>>,
) -> Result<DirWatcher> {
    let control_hwnd = control.0 as isize;
    DirWatcher::start(dir, move |events| {
        if let Ok(mut p) = pending.lock() {
            p.extend(events);
        }
        window::post_message(
            HWND(control_hwnd as *mut core::ffi::c_void),
            WM_APP_FS_CHANGED,
            0,
            0,
        );
    })
}

/// The user's and the public desktop folder.
pub(super) fn desktop_folders() -> Vec<PathBuf> {
    [shell::user_desktop(), shell::public_desktop()]
        .into_iter()
        .flatten()
        .collect()
}

/// Watchers on the desktop folders `dirs`, each with its folder.
pub(super) fn start_desktop_watchers(
    dirs: Vec<PathBuf>,
    control: HWND,
    pending: &Arc<Mutex<Vec<FsEvent>>>,
) -> Vec<(PathBuf, DirWatcher)> {
    let mut watchers = Vec::new();
    for dir in dirs {
        match start_watcher(&dir, control, pending.clone()) {
            Ok(w) => watchers.push((dir, w)),
            Err(e) => tracing::warn!(dir = %dir.display(), error = %e, "watcher failed"),
        }
    }
    watchers
}

impl App {
    /// A watcher exits when its folder is deleted, renamed or unmounted. Once a minute, watch
    /// the folders that are back again and show what they hold now.
    pub(super) fn revive_watchers(&mut self) {
        // A desktop folder without a running watcher: unreachable at startup (a network
        // location) or its watcher exited. Per folder, since the public desktop's watcher may
        // well run while the user's never started; only a folder readable now is tried.
        self.watchers.retain(|(_, w)| w.is_running());
        let back: Vec<PathBuf> = desktop_folders()
            .into_iter()
            .filter(|d| !self.watchers.iter().any(|(p, _)| p == d))
            .filter(|d| std::fs::read_dir(d).is_ok())
            .collect();
        if !back.is_empty() {
            let started = start_desktop_watchers(back, self.control.hwnd(), &self.fs_pending);
            if !started.is_empty() {
                self.watchers.extend(started);
                window::post_message(self.control.hwnd(), WM_APP_FS_CHANGED, 0, 0);
            }
        }
        self.portal_watchers.retain(|_, (_, w)| w.is_running());
        // Portals without a watcher, plus those last read as missing: a folder renamed away
        // and back keeps its watcher alive, and any batch meanwhile may have started one on a
        // folder that came back without re-reading it.
        let stale: Vec<FenceId> = self
            .state
            .fences()
            .iter()
            .filter(|f| {
                self.state.portal_path(f.id).is_some()
                    && (!self.portal_watchers.contains_key(&f.id)
                        || self.state.portal_missing(f.id))
            })
            .map(|f| f.id)
            .collect();
        if stale.is_empty() {
            return;
        }
        self.ensure_portal_watchers();
        let changed: Vec<FenceId> = stale
            .into_iter()
            .filter(|id| self.portal_watchers.contains_key(id) && self.state.refresh_portal(*id))
            .collect();
        self.refresh_portal_fences(&changed);
    }

    /// `desktop`: the batch carries a shell notice or a menu's request, so the desktop is
    /// synced whatever folders its events name.
    pub(super) fn on_fs_changed(&mut self, desktop: bool) {
        let events: Vec<FsEvent> = self
            .fs_pending
            .lock()
            .map(|mut p| p.drain(..).collect())
            .unwrap_or_default();
        tracing::debug!(count = events.len(), "desktop change batch");
        // What a shell drop onto a virtual fence writes after the fact (a shortcut per item)
        // goes into that fence: the new shortcuts of the first batch that has any. Other files
        // that land meanwhile, and paths already routed elsewhere, are left alone.
        if let Some((fence, since)) = self.shell_drop_claim {
            if since.elapsed() > Duration::from_secs(SHELL_DROP_CLAIM_SECS) {
                self.shell_drop_claim = None;
            } else if let Some(desktop) = shell::user_desktop() {
                for ev in &events {
                    if let FsEvent::Added(p) = ev
                        && p.parent() == Some(desktop.as_path())
                        && p.extension().is_some_and(|e| e.eq_ignore_ascii_case("lnk"))
                        && !self.pending_routes.iter().any(|r| r.path == *p)
                    {
                        self.pending_routes.push(PendingRoute {
                            fence,
                            path: p.clone(),
                            since: Instant::now(),
                            rename: false,
                        });
                        self.shell_drop_claim = None;
                    }
                }
            }
        }
        // Pair RENAMED_OLD/RENAMED_NEW so the item keeps its fence membership.
        let mut pending_old: Option<PathBuf> = None;
        let mut renamed = 0usize;
        for ev in &events {
            match ev {
                FsEvent::RenamedOld(p) => pending_old = Some(p.clone()),
                FsEvent::RenamedNew(new) => {
                    if let Some(old) = pending_old.take()
                        && self.state.rename_item(&old, new)
                    {
                        renamed += 1;
                        tracing::info!(from = %old.display(), to = %new.display(), "item renamed in place");
                    }
                }
                _ => pending_old = None,
            }
        }
        // Only the portals showing a folder this batch touched are re-read (each re-read asks
        // the shell for the names of new entries; a buffer overflow re-reads everything).
        let mut dirs: Vec<PathBuf> = Vec::new();
        let mut all = false;
        for ev in &events {
            match ev {
                FsEvent::Overflow => all = true,
                FsEvent::Added(p)
                | FsEvent::Removed(p)
                | FsEvent::Modified(p)
                | FsEvent::RenamedOld(p)
                | FsEvent::RenamedNew(p) => {
                    if let Some(d) = p.parent()
                        && !dirs.iter().any(|x| x == d)
                    {
                        dirs.push(d.to_path_buf());
                    }
                }
            }
        }
        // A batch that touched only portal folders (a download landing in one) leaves the
        // desktop alone, unless a shell notice (the Recycle Bin after a delete in that portal)
        // or a menu landed in the same batch; an empty batch is one of those on its own.
        let desktops = [shell::user_desktop(), shell::public_desktop()];
        let touches_desktop = desktop
            || all
            || dirs.is_empty()
            || dirs
                .iter()
                .any(|d| desktops.iter().flatten().any(|x| x == d));
        let t0 = Instant::now();
        let report = if touches_desktop {
            self.sync_desktop_if_available("fs change")
        } else {
            crate::state::SyncReport::default()
        };
        let t_sync = t0.elapsed();
        self.route_pending_creation();
        if all {
            self.refresh_portals();
        } else {
            self.refresh_portals_in(&dirs);
        }
        let t_portals = t0.elapsed() - t_sync;
        tracing::debug!(
            sync_ms = t_sync.as_secs_f32() * 1000.0,
            portals_ms = t_portals.as_secs_f32() * 1000.0,
            "fs change batch timing"
        );
        if report.changed() || renamed > 0 {
            tracing::info!(?report, "desktop resynced");
            // Renamed / modified files may have new icons; a file that just arrived (moved in
            // by the shell) may have been extracted while it was still landing. Once per icon
            // key: a file being written reports several events, documents of one type share
            // a key, and each invalidation scans the whole icon and bitmap caches.
            let mut invalidated: HashSet<pecofence_core::IconKey> = HashSet::new();
            for ev in &events {
                if let FsEvent::Modified(p) | FsEvent::RenamedNew(p) | FsEvent::Added(p) = ev
                    && let Some(id) = self.state.item_by_path(p)
                    && let Some(item) = self.state.item(id)
                    && invalidated.insert(item.icon_key.clone())
                {
                    self.ctx.icons.borrow_mut().invalidate(&item.icon_key);
                    // Draw keys are `<cache_key><variant>@<px>`: drop every size / variant.
                    self.ctx
                        .bitmaps
                        .borrow_mut()
                        .remove_prefix(&IconCache::cache_key(&item.icon_key));
                }
            }
            let t1 = Instant::now();
            self.refresh_all();
            tracing::debug!(
                refresh_all_ms = t1.elapsed().as_secs_f32() * 1000.0,
                "fs change refresh timing"
            );
            self.schedule_save();
        }
        self.push_workspace_summary();
    }

    /// Reconciles the item table with the desktop — unless the desktop folder is unreachable
    /// (removable / network drive), in which case nothing is orphaned and we retry later.
    pub(super) fn sync_desktop_if_available(
        &mut self,
        reason: &'static str,
    ) -> crate::state::SyncReport {
        if !shell::desktop_available() {
            if !self.desktop_unavailable {
                tracing::warn!(reason, "desktop folder unavailable; keeping item records");
                if let Some(t) = &self.tray {
                    t.show_info(
                        "PecoFence",
                        pecofence_core::i18n::text(
                            "桌面文件夹暂时无法访问（位于移动磁盘或网络位置），各项目所在的栅栏和位置都会保留。",
                        ),
                        true,
                    );
                }
            }
            self.desktop_unavailable = true;
            return crate::state::SyncReport::default();
        }
        if self.desktop_unavailable {
            tracing::info!(reason, "desktop folder is back");
            self.desktop_unavailable = false;
        }
        let (mut entries, unread) = shell::enumerate_desktop_reporting();
        // With the real icons hidden, the Recycle Bin and the other special desktop items would
        // be unreachable: surface the ones Windows shows inside the inbox fence. When the
        // setting is off they drop out of the listing and are orphaned (hidden) at once.
        if self.state.config.settings.hide_real_icons && !self.no_hide_icons {
            if self._shell_watch.is_none() {
                // No change notices for the Recycle Bin: ask it every time.
                shell::recycle_bin_changed();
            }
            entries.extend(shell::enumerate_special_desktop_items());
        }
        let report = self.state.sync_desktop_partly_read(&entries, &unread);
        tracing::debug!(
            reason,
            items = entries.len(),
            ?unread,
            ?report,
            "desktop synced"
        );
        report
    }

    /// F5: re-read what this fence shows (portal folder or desktop) and its icons, like
    /// Explorer's refresh.
    pub(super) fn manual_refresh(&mut self, fence: FenceId) {
        let host = self.state.host_of(fence);
        let active = self.state.active_tab_of(host);
        let portal = self.state.portal_path(active).is_some();
        if portal {
            self.state.refresh_portal(active);
            // A watcher that exited with its folder is started again now that the user asks.
            self.portal_watchers.retain(|_, (_, w)| w.is_running());
            self.ensure_portal_watchers();
        } else {
            shell::recycle_bin_changed();
            let report = self.sync_desktop_if_available("manual refresh");
            self.route_pending_creation();
            if report.changed() {
                self.schedule_save();
            }
        }
        if let Some(f) = self.state.fence(active) {
            let keys: Vec<_> = self
                .state
                .items_of(f)
                .iter()
                .map(|it| it.icon_key.clone())
                .collect();
            let mut icons = self.ctx.icons.borrow_mut();
            let mut bitmaps = self.ctx.bitmaps.borrow_mut();
            for k in keys {
                icons.invalidate(&k);
                bitmaps.remove_prefix(&IconCache::cache_key(&k));
            }
        }
        if portal {
            // Another portal on the same folder shares the items just re-read.
            self.refresh_portal_fences(&[active]);
        } else {
            self.refresh_fence(active);
        }
        if let Some(w) = self.fences.get(&host) {
            w.drop_icons();
        }
        self.push_workspace_summary();
    }
}
