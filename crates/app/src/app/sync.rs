//! Desktop-folder synchronisation: fs-watcher batches, availability-guarded sync, manual refresh.

use super::*;

impl App {
    pub(super) fn on_fs_changed(&mut self) {
        let events: Vec<FsEvent> = self
            .fs_pending
            .lock()
            .map(|mut p| p.drain(..).collect())
            .unwrap_or_default();
        tracing::debug!(count = events.len(), "desktop change batch");
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
        let t0 = Instant::now();
        let report = self.sync_desktop_if_available("fs change");
        let t_sync = t0.elapsed();
        self.route_pending_creation();
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
            // by the shell) may have been extracted while it was still landing.
            for ev in &events {
                if let FsEvent::Modified(p) | FsEvent::RenamedNew(p) | FsEvent::Added(p) = ev
                    && let Some(id) = self.state.item_by_path(p)
                    && let Some(item) = self.state.item(id)
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
    pub(super) fn sync_desktop_if_available(&mut self, reason: &str) -> crate::state::SyncReport {
        if !shell::desktop_available() {
            if !self.desktop_unavailable {
                tracing::warn!(reason, "desktop folder unavailable; keeping item records");
                if let Some(t) = &self.tray {
                    t.show_info(
                        "PecoFence",
                        pecofence_core::i18n::text(
                            "桌面文件夹当前不可用（可移动磁盘或网络位置），项目记录已保留。",
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
        let mut entries = shell::enumerate_desktop();
        // With the real icons hidden, the Recycle Bin and the other special desktop items would
        // be unreachable: surface the ones Windows shows inside the inbox fence. When the
        // setting is off they drop out of the listing and are orphaned (hidden) at once.
        if self.state.config.settings.hide_real_icons && !self.no_hide_icons {
            entries.extend(shell::enumerate_special_desktop_items());
        }
        let report = self.state.sync_desktop(&entries);
        tracing::debug!(reason, items = entries.len(), ?report, "desktop synced");
        report
    }

    /// F5: re-read what this fence shows (portal folder or desktop) and its icons, like
    /// Explorer's refresh.
    pub(super) fn manual_refresh(&mut self, fence: FenceId) {
        let host = self.state.host_of(fence);
        let active = self.state.active_tab_of(host);
        if self.state.portal_path(active).is_some() {
            self.state.refresh_portal(active);
            self.ensure_portal_watchers();
        } else {
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
        self.refresh_fence(active);
        if let Some(w) = self.fences.get(&host) {
            w.drop_icons();
        }
        self.push_workspace_summary();
    }
}
