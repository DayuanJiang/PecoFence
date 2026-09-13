//! Folder-portal fences: decorations, per-portal folder watchers, creation, refresh, navigation.

use super::*;

impl App {
    /// Pushes the shown fence's portal decorations (folder glyph, up button, navigate flag) and
    /// its display title into the host window.
    pub(super) fn apply_portal_deco(&self, host: FenceId) {
        let Some(w) = self.fences.get(&host) else {
            return;
        };
        let active = self.state.active_tab_of(host);
        let Some(f) = self.state.fence(active) else {
            return;
        };
        let is_portal = f.kind == FenceKind::FolderPortal;
        w.set_portal_deco(
            is_portal,
            is_portal && !f.hide_title_icon,
            is_portal && self.state.portal_navigated(active),
            is_portal && f.portal_navigate,
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
        self.portal_watchers
            .retain(|id, (path, _)| portals.iter().any(|(pid, p)| pid == id && p == path));
        for (id, dir) in portals {
            if self.portal_watchers.contains_key(&id) {
                continue;
            }
            let pending = self.fs_pending.clone();
            let control_hwnd = self.control.hwnd().0 as isize;
            match DirWatcher::start(&dir, move |events| {
                if let Ok(mut p) = pending.lock() {
                    p.extend(events);
                }
                window::post_message(
                    HWND(control_hwnd as *mut core::ffi::c_void),
                    WM_APP_FS_CHANGED,
                    0,
                    0,
                );
            }) {
                Ok(w) => {
                    self.portal_watchers.insert(id, (dir, w));
                }
                Err(e) => {
                    tracing::warn!(dir = %dir.display(), error = %e, "portal watcher failed")
                }
            }
        }
    }

    /// Item menu "作为栅栏窗口显示": a new fence that mirrors `folder` (plan §12 文件夹门户).
    pub(super) fn create_portal(&mut self, folder: PathBuf, x: i32, y: i32, near: Option<FenceId>) {
        if !folder.is_dir() {
            tracing::warn!(folder = %folder.display(), "portal: not a directory");
            if let Some(t) = &self.tray {
                t.show_info(
                    "PecoFence",
                    &pecofence_core::i18n::format(
                        "无法创建文件夹门户：{0} 不是文件夹",
                        &[format!("{}", folder.display())],
                    ),
                    true,
                );
            }
            return;
        }
        // Item keys are case-folded; store the folder with its real casing.
        let folder = std::fs::canonicalize(&folder)
            .map(|c| {
                let s = c.to_string_lossy().to_string();
                PathBuf::from(s.strip_prefix(r"\\?\").unwrap_or(&s))
            })
            .unwrap_or(folder);
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
        let rect = self.place_new_fence(4, 260.0, x, y, near);
        if let Some(id) = self.state.new_portal_fence(&folder, rect) {
            tracing::info!(folder = %folder.display(), %id, "folder portal created");
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
        for id in changed {
            self.refresh_fence(id);
        }
    }

    /// Re-reads every portal folder (startup, overflow, layout changes) and redraws the ones
    /// that changed.
    pub(super) fn refresh_portals(&mut self) {
        let changed = self.state.refresh_all_portals();
        // refresh_portal may have reset a vanished subfolder back to the root: drop the stale
        // watcher on the old path and start one on the folder now shown.
        self.ensure_portal_watchers();
        for id in changed {
            self.refresh_fence(id);
        }
    }

    pub(super) fn portal_enter(&mut self, fence: FenceId, dir: &std::path::Path) {
        if self.state.portal_enter(fence, dir) {
            self.after_portal_navigation(fence);
        }
    }

    pub(super) fn portal_up(&mut self, fence: FenceId) {
        if self.state.portal_up(fence) {
            self.after_portal_navigation(fence);
        }
    }

    pub(super) fn after_portal_navigation(&mut self, fence: FenceId) {
        self.ensure_portal_watchers();
        self.refresh_fence(fence);
        let host = self.state.host_of(fence);
        self.apply_portal_deco(host);
        self.push_workspace_summary();
    }
}
