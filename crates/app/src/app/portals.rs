//! Folder-portal fences, and regular fences showing a folder: decorations, per-fence folder
//! watchers, creation, refresh, navigation.

use super::*;

impl App {
    /// Pushes the shown fence's portal decorations (folder glyph, up button, navigate flag) and
    /// its display title into the host window. A regular fence showing a folder gets them too,
    /// except the glyph: it marks folder portals, whose option hides it.
    pub(super) fn apply_portal_deco(&self, host: FenceId) {
        let Some(w) = self.fences.get(&host) else {
            return;
        };
        let active = self.state.active_tab_of(host);
        let Some(f) = self.state.fence(active) else {
            return;
        };
        let folder = self.state.portal_path(active);
        let is_portal = folder.is_some();
        w.set_portal_deco(
            is_portal,
            f.kind == FenceKind::FolderPortal && !f.hide_title_icon,
            self.state.can_go_up(active),
            is_portal && f.navigates_folders(),
            is_portal && self.state.portal_missing(active),
            folder,
        );
        if let Some(h) = self.state.fence(host) {
            let title = if active == host {
                self.state.display_title(h)
            } else {
                h.title.clone()
            };
            w.set_title(&title);
        }
    }

    /// Starts/stops folder watchers so every portal fence follows its folder.
    pub(super) fn ensure_portal_watchers(&mut self) {
        let portals: Vec<(FenceId, PathBuf)> = self
            .state
            .fences()
            .iter()
            .filter_map(|f| self.state.portal_path(f.id).map(|p| (f.id, p)))
            .collect();
        // An exited watcher (its folder was deleted, renamed or unmounted) is kept here: every
        // batch ends up in this function, and a share that cannot deliver change notices fails
        // and posts an Overflow at once, so dropping it here would restart it without end.
        // F5 and `revive_watchers` (once a minute) drop exited watchers before calling this.
        self.portal_watchers
            .retain(|id, (path, _)| portals.iter().any(|(pid, p)| pid == id && p == path));
        self.watcher_retry
            .retain(|dir, _| portals.iter().any(|(_, p)| p == dir));
        for (id, dir) in portals {
            if self.portal_watchers.contains_key(&id)
                || self
                    .watcher_retry
                    .get(&dir)
                    .is_some_and(|(_, at)| *at > Instant::now())
            {
                continue;
            }
            let started = Instant::now();
            match sync::start_watcher(&dir, self.control.hwnd(), self.fs_pending.clone()) {
                Ok(w) => {
                    self.watcher_retry.remove(&dir);
                    self.portal_watchers.insert(id, (dir, w));
                }
                Err(e) => {
                    // A folder that is gone (the fence says so) is tried on every batch: no
                    // warning each time.
                    const NOT_FOUND: [i32; 2] = [0x8007_0002_u32 as i32, 0x8007_0003_u32 as i32];
                    if NOT_FOUND.contains(&e.code().0) {
                        tracing::debug!(dir = %dir.display(), "portal folder not found");
                    } else {
                        tracing::warn!(dir = %dir.display(), place = shell::path_place(&dir), error = %e, "portal watcher failed");
                    }
                    // Opening a folder on an unreachable share blocks the UI thread until the
                    // network gives up, and this runs on every batch and once a minute: such
                    // a folder waits twice as long before each further try, up to an hour. A
                    // local folder that is gone fails at once and is tried every time.
                    if started.elapsed() > Duration::from_secs(1) {
                        let failures = self.watcher_retry.get(&dir).map_or(1, |(n, _)| n + 1);
                        let wait = Duration::from_secs(60 << failures.min(6));
                        self.watcher_retry
                            .insert(dir, (failures, Instant::now() + wait));
                    }
                }
            }
        }
    }

    /// 「在桌面显示文件夹」 submenu shared by the tray and fence menus: Documents, Downloads,
    /// Pictures (named as Explorer names them, so they follow the Windows language), then a
    /// folder picker, on ids `base..base + 4`. User-facing text never says "portal"; the menu
    /// names what happens instead.
    pub(super) fn show_folder_submenu(base: u32) -> PopupMenu {
        use pecofence_platform::shell;
        let menu = PopupMenu::new();
        let known = [
            shell::user_documents(),
            shell::user_downloads(),
            shell::user_pictures(),
        ];
        for (i, dir) in known.into_iter().enumerate() {
            let Some(dir) = dir.filter(|d| d.is_dir()) else {
                continue;
            };
            let name = shell::display_name(&dir).unwrap_or_else(|| {
                dir.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| dir.display().to_string())
            });
            menu.item(base + i as u32, &name, false, false);
        }
        menu.separator().item(
            base + 3,
            pecofence_core::i18n::text("选择其他文件夹…"),
            false,
            false,
        );
        menu
    }

    /// Runs entry `offset` of [`Self::show_folder_submenu`].
    pub(super) fn show_folder_command(
        &mut self,
        offset: u32,
        x: i32,
        y: i32,
        near: Option<FenceId>,
    ) {
        use pecofence_platform::shell;
        let folder = match offset {
            0 => shell::user_documents(),
            1 => shell::user_downloads(),
            2 => shell::user_pictures(),
            _ => match pecofence_platform::filedialog::pick_folder(
                None,
                pecofence_core::i18n::text("选择要在桌面显示的文件夹"),
            ) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(error = %e, "folder picker failed");
                    None
                }
            },
        };
        if let Some(folder) = folder {
            self.create_portal(folder, x, y, near);
        }
    }

    /// Item menu 「在桌面显示此文件夹」: a new fence that mirrors `folder` (plan §12 文件夹门户).
    pub(super) fn create_portal(&mut self, folder: PathBuf, x: i32, y: i32, near: Option<FenceId>) {
        if !folder.is_dir() {
            tracing::warn!(folder = %folder.display(), "portal: not a directory");
            if let Some(t) = &self.tray {
                t.show_info(
                    "PecoFence",
                    &pecofence_core::i18n::format(
                        "无法在桌面显示：{0} 不是文件夹",
                        &[format!("{}", folder.display())],
                    ),
                    true,
                );
            }
            return;
        }
        // Item keys are case-folded; store the folder with its real casing.
        let folder = crate::state::canonical_path(folder);
        if let Some(existing) = self
            .state
            .fences()
            .iter()
            .find(|f| {
                self.state.portal_root(f.id).is_some_and(|p| {
                    p.to_string_lossy().to_lowercase() == folder.to_string_lossy().to_lowercase()
                })
            })
            .map(|f| f.id)
        {
            // Already shown: just bring attention to it (switch to its tab if hosted).
            let host = self.state.host_of(existing);
            if host != existing {
                self.switch_tab(host, existing);
            }
            if let Some(w) = self.fences.get(&host) {
                w.show(false);
            }
            return;
        }
        if !self.fence_room_or_notice() {
            return;
        }
        let rect = self.place_new_fence(4, 260.0, x, y, near);
        if let Some(id) = self.state.new_portal_fence(&folder, rect) {
            tracing::info!(folder = %folder.display(), place = shell::path_place(&folder), %id, "folder portal created");
            self.resync_windows();
            if let Some(w) = self.fences.get(&id) {
                w.show(true);
            }
            self.schedule_save();
        }
    }

    /// Re-reads only the portals currently showing one of `dirs` (a watcher batch names the
    /// folders it saw; a file operation names its source and destination folders). Folder
    /// comparison is case-insensitive, like NTFS names.
    pub(super) fn refresh_portals_in(&mut self, dirs: &[PathBuf]) {
        let wanted: Vec<String> = dirs
            .iter()
            .map(|d| d.to_string_lossy().to_lowercase())
            .collect();
        let ids: Vec<FenceId> = self
            .state
            .fences()
            .iter()
            .filter_map(|f| self.state.portal_path(f.id).map(|p| (f.id, p)))
            .filter(|(_, p)| wanted.contains(&p.to_string_lossy().to_lowercase()))
            .map(|(id, _)| id)
            .collect();
        let mut changed = Vec::new();
        for id in ids {
            if self.state.refresh_portal(id) {
                changed.push(id);
            }
        }
        self.ensure_portal_watchers();
        self.refresh_portal_fences(&changed);
    }

    /// Re-reads every portal folder (startup, overflow, layout changes) and redraws the ones
    /// that changed.
    pub(super) fn refresh_portals(&mut self) {
        let changed = self.state.refresh_all_portals();
        // refresh_portal may have reset a vanished subfolder back to the root: drop the stale
        // watcher on the old path and start one on the folder now shown.
        self.ensure_portal_watchers();
        self.refresh_portal_fences(&changed);
    }

    /// Redraws the portals whose re-read found a change, and every other portal showing one of
    /// the same folders: two portals on one folder share its items, so the second re-read finds
    /// them already updated and reports no change of its own.
    pub(super) fn refresh_portal_fences(&mut self, changed: &[FenceId]) {
        let dirs: Vec<String> = changed
            .iter()
            .filter_map(|id| self.state.portal_path(*id))
            .map(|p| p.to_string_lossy().to_lowercase())
            .collect();
        let ids: Vec<FenceId> = self
            .state
            .fences()
            .iter()
            .filter(|f| {
                self.state
                    .portal_path(f.id)
                    .is_some_and(|p| dirs.contains(&p.to_string_lossy().to_lowercase()))
            })
            .map(|f| f.id)
            .collect();
        for id in ids {
            self.refresh_fence(id);
        }
    }

    pub(super) fn portal_enter(&mut self, fence: FenceId, dir: &std::path::Path) {
        if self.state.portal_enter(fence, dir) {
            self.after_portal_navigation(fence, 1, None);
        }
    }

    /// Opens folder `dir` inside regular fence `fence`: a subfolder of the folder it shows opens
    /// as a subfolder (up goes to its parent); any other folder is shown in place of the fence's
    /// icons until up brings them back. False when refused.
    pub(super) fn open_folder_in_fence(&mut self, fence: FenceId, dir: &std::path::Path) -> bool {
        let entered = self.state.portal_enter(fence, dir) || self.state.open_folder(fence, dir);
        if entered {
            self.after_portal_navigation(fence, 1, None);
        }
        entered
    }

    /// One level up selects the folder just left, as in Explorer.
    pub(super) fn portal_up(&mut self, fence: FenceId) {
        let left = self.state.up_selection(fence);
        if self.state.portal_up(fence) {
            self.after_portal_navigation(fence, -1, left);
        }
    }

    /// `dir` and `select` as in [`FenceWindow::begin_portal_navigation`].
    pub(super) fn after_portal_navigation(
        &mut self,
        fence: FenceId,
        dir: i8,
        select: Option<ItemId>,
    ) {
        self.ensure_portal_watchers();
        if let Some(w) = self.window_for(fence)
            && w.active_fence() == fence
        {
            w.begin_portal_navigation(dir, select);
        }
        self.refresh_fence(fence);
        // The folder just left may still have icons queued; they would delay this one's.
        self.drop_unneeded_icon_requests();
        let host = self.state.host_of(fence);
        self.apply_portal_deco(host);
        self.push_workspace_summary();
    }
}
