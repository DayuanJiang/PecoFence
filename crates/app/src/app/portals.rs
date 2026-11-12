//! Folder-portal fences, and regular fences showing a folder: decorations, per-fence folder
//! watchers, creation, refresh, navigation.

use super::*;

/// Opens `dir` in Explorer when it is a folder given by its full path. A folder fence's folder
/// and a smart fence's folders come from the configuration, which may be a file from someone
/// else (an import): a program, a document or a web address there is not handed to Windows.
/// False when refused or when Explorer did not open it.
pub(super) fn open_folder_in_explorer(dir: &std::path::Path) -> bool {
    if !dir.is_absolute() || !dir.is_dir() {
        tracing::info!(
            place = shell::path_place(dir),
            absolute = dir.is_absolute(),
            "open folder refused: not a folder"
        );
        return false;
    }
    shell::shell_execute(dir, None, None).is_ok()
}

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
        // 智能栅栏: the folder glyph with a magnifier badge.
        let smart = f.smart.is_some();
        w.set_portal_deco(
            is_portal,
            (f.kind == FenceKind::FolderPortal || smart) && !f.hide_title_icon,
            smart,
            self.state.can_go_up(active),
            is_portal && f.navigates_folders(),
            is_portal && self.state.portal_missing(active),
            folder,
        );
        let (empty, action, count) = self.smart_view(f);
        let filtered = self.fence_filter(active).is_some();
        let empty = if filtered {
            Some(pecofence_core::i18n::text("没有符合筛选的文件。"))
        } else {
            empty
        };
        let action = action.filter(|_| !filtered);
        // The count belongs to a single title; in a tab strip the tab shows the name only.
        let count = count.filter(|_| active == host && self.state.tabs_of(host).len() <= 1);
        w.set_count_tip(count.as_ref().and_then(|_| self.smart_count_tip(f)));
        w.set_status_view(empty, action, count);
        w.set_filtered(filtered);
        // The tag's count follows what the fence shows (files come and go).
        self.sync_filter_view(active);
        // A fence that says 找不到此文件夹 has nowhere to put a file: it takes no drop (the
        // shell's move would fail with a system error).
        w.set_refuses_drops(smart || (is_portal && self.state.portal_missing(active)));
        if let Some(h) = self.state.fence(host) {
            let title = if active == host {
                self.filter_title(host, self.state.display_title(h))
            } else {
                h.title.clone()
            };
            w.set_title(&title);
        }
    }

    /// Starts/stops folder watchers so every portal fence follows its folder.
    pub(super) fn ensure_portal_watchers(&mut self) {
        self.ensure_smart_watchers();
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
        self.create_portal_at(folder, x, y, near, None);
    }

    /// The desktop marquee's 「在此显示文件夹…」: the folder fence fills the drawn rectangle.
    pub(super) fn create_portal_in(&mut self, folder: PathBuf, rect: RECT) {
        let (x, y) = (rect.left, rect.top);
        self.create_portal_at(folder, x, y, None, Some(rect));
    }

    /// A folder fence for `folder` at `rect`, or placed near (`x`, `y`) / fence `near` like a
    /// new fence when `rect` is None.
    fn create_portal_at(
        &mut self,
        folder: PathBuf,
        x: i32,
        y: i32,
        near: Option<FenceId>,
        rect: Option<RECT>,
    ) {
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
            tracing::info!(fence = %existing, place = shell::path_place(&folder), "folder portal exists: shown instead of a new one");
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
        let drawn = rect.is_some();
        let rect = rect.unwrap_or_else(|| self.place_new_fence(4, 260.0, x, y, near));
        if let Some(id) = self.state.new_portal_fence(&folder, rect) {
            tracing::info!(folder = %folder.display(), place = shell::path_place(&folder), %id, drawn, ?rect, "folder portal created");
            self.resync_windows();
            if let Some(w) = self.fences.get(&id) {
                w.show(true);
            }
            self.schedule_save();
            self.offer_roll_tip();
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
        // Whatever re-read found a folder gone (a watcher batch, a file operation, a click on
        // the title's folder): it is looked for from now on, so it is shown again as soon as
        // it is back.
        self.arm_portal_back_check();
    }

    pub(super) fn portal_enter(&mut self, fence: FenceId, dir: &std::path::Path) {
        if self.state.portal_enter(fence, dir) {
            self.portal_forward.remove(&fence);
            self.after_portal_navigation(fence, 1, None);
        }
    }

    /// Opens folder `dir` inside regular fence `fence`: a subfolder of the folder it shows opens
    /// as a subfolder (up goes to its parent); any other folder is shown in place of the fence's
    /// icons until up brings them back. False when refused.
    pub(super) fn open_folder_in_fence(&mut self, fence: FenceId, dir: &std::path::Path) -> bool {
        let entered = self.state.portal_enter(fence, dir) || self.state.open_folder(fence, dir);
        if entered {
            self.portal_forward.remove(&fence);
            self.after_portal_navigation(fence, 1, None);
        }
        entered
    }

    /// One level up selects the folder just left, as in Explorer.
    pub(super) fn portal_up(&mut self, fence: FenceId) {
        let left = self.state.up_selection(fence);
        let folder = self.state.portal_path(fence);
        if self.state.portal_up(fence) {
            if let Some(folder) = folder {
                self.portal_forward.entry(fence).or_default().push(folder);
            }
            self.after_portal_navigation(fence, -1, left);
        }
    }

    /// Opens the folder `fence` shows in Explorer (the title glyph at the root, the fence
    /// menu's 在资源管理器中打开文件夹). `by` names the control for the log.
    pub(super) fn open_portal_folder(&mut self, fence: FenceId, by: &'static str) {
        let Some(dir) = self.state.portal_path(fence) else {
            tracing::info!(fence = %fence, by, "open folder: not a folder fence");
            return;
        };
        // A folder that is gone (renamed, deleted, its drive unplugged) while the fence still
        // shows its icons: Explorer would put up its own "Windows cannot find" box, inside
        // this call, and every fence would wait until it is closed. The fence is read again
        // instead and says 找不到此文件夹。
        if !dir.is_dir() {
            tracing::info!(fence = %fence, by, place = shell::path_place(&dir), "open folder: the folder is gone; fence re-read");
            self.portal_watchers.remove(&fence);
            self.refresh_portals_in(std::slice::from_ref(&dir));
            return;
        }
        let ok = open_folder_in_explorer(&dir);
        tracing::info!(fence = %fence, by, place = shell::path_place(&dir), ok, "folder opened in Explorer");
    }

    /// The forward button: into the folder the last up left (a subfolder, or the folder that
    /// was opened in a regular fence). Nothing when no up came before or the folder is gone.
    pub(super) fn portal_forward(&mut self, fence: FenceId) {
        let Some(folder) = self.portal_forward.get_mut(&fence).and_then(Vec::pop) else {
            return;
        };
        if self.state.portal_enter(fence, &folder) || self.state.open_folder(fence, &folder) {
            tracing::info!(fence = %fence, "folder forward");
            self.after_portal_navigation(fence, 1, None);
        } else {
            tracing::info!(fence = %fence, "folder forward: the folder is gone");
            self.portal_forward.remove(&fence);
        }
    }

    /// `dir` and `select` as in [`FenceWindow::begin_portal_navigation`].
    pub(super) fn after_portal_navigation(
        &mut self,
        fence: FenceId,
        dir: i8,
        select: Option<ItemId>,
    ) {
        self.clear_fence_filter(fence, "another folder shown");
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
