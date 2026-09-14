//! Item commands: launch, inline rename, shell verbs, clipboard, moving items between fences,
//! routing of newly created desktop items, item view builder.

use super::*;

impl App {
    pub(super) fn item_views(&self, fence: &pecofence_core::Fence) -> Vec<ItemView> {
        self.state
            .items_of(fence)
            .into_iter()
            .map(|it| {
                let path = PathBuf::from(it.key.as_path().unwrap_or(""));
                let (_, icon_only) = crate::icons::icon_key_for(&path, it.is_folder, it.mtime);
                ItemView::new(
                    it.id,
                    path,
                    it.display_name.clone(),
                    it.is_folder,
                    it.icon_key.clone(),
                    icon_only,
                    it.mtime,
                    it.size,
                )
            })
            .collect()
    }

    /// Fence menu "新建 ▸": create the file/folder on the real desktop and remember which fence
    /// asked for it; the watcher's next Added event routes it there and opens rename.
    pub(super) fn create_desktop_item(&mut self, fence: FenceId, folder: bool) {
        let portal_dir = self.state.portal_path(fence);
        let Some(dir) = portal_dir.clone().or_else(shell::user_desktop) else {
            return;
        };
        let (base, ext) = if folder {
            (pecofence_core::i18n::text("新建文件夹"), "")
        } else {
            (pecofence_core::i18n::text("新建 文本文档"), ".txt")
        };
        let mut path = dir.join(format!("{base}{ext}"));
        let mut n = 2;
        while path.exists() {
            path = dir.join(format!("{base} ({n}){ext}"));
            n += 1;
        }
        let created = if folder {
            std::fs::create_dir(&path)
        } else {
            std::fs::write(&path, b"")
        };
        match created {
            Ok(()) => {
                tracing::info!(path = %path.display(), %fence, "created item from fence menu");
                if let Some(dir) = portal_dir {
                    // A portal has no desktop PendingRoute. Refresh it now and start the
                    // same inline editor used for a new desktop item, after this command.
                    self.refresh_portals_in(&[dir]);
                    let key = ItemKey::from_path(&path.to_string_lossy());
                    let item = self.state.fence(fence).and_then(|f| {
                        self.state
                            .items_of(f)
                            .into_iter()
                            .find(|it| it.key == key)
                            .map(|it| it.id)
                    });
                    if let Some(item) = item {
                        self.queue.push(Command::RenameItem { fence, item });
                    }
                    return;
                }
                self.pending_routes.push(PendingRoute {
                    fence,
                    path,
                    since: Instant::now(),
                    rename: true,
                });
            }
            Err(e) => {
                tracing::warn!(error = %e, "creating desktop item failed");
                if let Some(t) = &self.tray {
                    t.show_info(
                        "PecoFence",
                        &pecofence_core::i18n::format("无法在桌面创建项目：{0}", &[e.to_string()]),
                        true,
                    );
                }
            }
        }
    }

    /// After a desktop sync: honour an unexpired PendingCreation (plan §5.8 / task 12).
    pub(super) fn route_pending_creation(&mut self) {
        self.pending_routes
            .retain(|p| p.since.elapsed().as_secs() <= PENDING_CREATION_SECS);
        let mut landed = Vec::new();
        self.pending_routes
            .retain(|p| match self.state.item_id_for_path(&p.path) {
                Some(id) => {
                    landed.push((id, p.fence, p.rename));
                    false
                }
                None => true,
            });
        for (id, fence, rename) in landed {
            self.move_items(&[id], fence);
            if rename {
                self.begin_item_rename(fence, id);
            }
        }
    }

    /// Moves portal items out to the real desktop and routes them into `to` once they appear.
    pub(super) fn move_portal_items_to_desktop(&mut self, items: &[ItemId], to: FenceId) {
        let Some(desktop) = shell::user_desktop() else {
            return;
        };
        let paths: Vec<PathBuf> = items
            .iter()
            .filter(|id| self.state.is_portal_item(**id))
            .filter_map(|id| self.state.item(*id))
            .filter_map(|it| it.key.as_path().map(PathBuf::from))
            .collect();
        if paths.is_empty() {
            return;
        }
        let routed: Vec<PathBuf> = paths
            .iter()
            .filter_map(|p| p.file_name().map(|n| desktop.join(n)))
            .collect();
        for path in &routed {
            self.pending_routes.push(PendingRoute {
                fence: to,
                path: path.clone(),
                since: Instant::now(),
                rename: false,
            });
        }
        let owner = self.window_for(to).map(|w| w.hwnd());
        // Worker thread; a cancelled / failed move withdraws only our own routes (other
        // fences' 新建 claims / drop routes stay queued) — see `on_fileop_done`.
        self.start_fileop(FileOp {
            paths,
            dest: desktop,
            copy: false,
            rename_on_collision: false,
            owner,
            then: FileOpThen::ToDesktop {
                routed,
                copy: false,
                what: "moved out of portal to the desktop",
            },
        });
        self.refresh_portals();
    }

    pub(super) fn begin_item_rename(&mut self, fence: FenceId, item: ItemId) {
        let Some(w) = self.window_for(fence) else {
            return;
        };
        if w.active_fence() != fence {
            // The item's tab is not the one on screen: switch first so the label has a rect.
            let host = self.state.host_of(fence);
            self.switch_tab(host, fence);
        }
        let Some(w) = self.window_for(fence) else {
            return;
        };
        // LVM_EDITLABEL: the item is scrolled fully into view (at its final position) before
        // the edit opens, so the box sits on its label instead of the fence edge.
        w.ensure_item_visible_now(item);
        let Some(rect) = w.item_label_rect(item) else {
            return;
        };
        let Some(it) = self.state.item(item) else {
            return;
        };
        if it.is_namespace() {
            // The Recycle Bin and friends have no file to rename.
            return;
        }
        let current = it.display_name.clone();
        let select_end = crate::rename::rename_select_len(
            &current,
            it.key.as_path().unwrap_or(""),
            it.is_folder,
        );
        let dpi = monitors::dpi_for_window(w.hwnd());
        w.set_renaming(Some(item));
        crate::rename::begin_item_rename(
            rect,
            dpi,
            item,
            current,
            select_end,
            self.queue.clone(),
            self.theme_mode,
        );
    }

    /// The inline rename edit closed (commit or cancel): labels may unfold again and a hover
    /// peek held open by the popup may start its close countdown. A newer session may already
    /// be open by the time the old one's queued command lands here (`begin_rename_at` ends the
    /// previous popup after the new flags were set): that window keeps its flag.
    pub(super) fn end_item_rename_visuals(&self) {
        let (keep_item, keep_title) = match crate::rename::active_target() {
            Some(crate::rename::RenameTarget::Item(item)) => (Some(item), None),
            Some(crate::rename::RenameTarget::Fence(fence)) => {
                (None, self.window_for(fence).map(|w| w.hwnd()))
            }
            None => (None, None),
        };
        for w in self.fences.values() {
            if keep_item.is_none() || w.renaming_item() != keep_item {
                w.set_renaming(None);
            }
            if keep_title != Some(w.hwnd()) {
                w.set_title_renaming(false);
            }
        }
    }

    /// Renames the file behind an item to `name` (+ the hidden extension Explorer would keep).
    pub(super) fn rename_item_file(&mut self, item: ItemId, name: &str) {
        if self.state.item(item).is_some_and(|it| it.is_namespace()) {
            return;
        }
        let name = name.trim();
        if name.is_empty()
            || name
                .chars()
                .any(|c| matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        {
            if let Some(t) = &self.tray {
                t.show_info(
                    "PecoFence",
                    pecofence_core::i18n::text("名称不能为空，也不能包含 \\ / : * ? \" < > |"),
                    true,
                );
            }
            return;
        }
        let Some(it) = self.state.item(item) else {
            return;
        };
        let Some(old) = it.key.as_path().map(PathBuf::from) else {
            return;
        };
        let display = it.display_name.clone();
        let file_name = old
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        // Explorer's editing name drops `.lnk` / hidden extensions: keep whatever it dropped.
        let suffix = crate::rename::rename_hidden_suffix(&display, &file_name, it.is_folder);
        let Some(parent) = old.parent() else { return };
        let new = parent.join(format!("{name}{suffix}"));
        // `old` is a lower-cased ItemKey, not the file's displayed spelling.
        if name == display {
            return;
        }
        if new.exists()
            && ItemKey::from_path(&new.to_string_lossy())
                != ItemKey::from_path(&old.to_string_lossy())
        {
            if let Some(t) = &self.tray {
                t.show_info(
                    "PecoFence",
                    pecofence_core::i18n::text("此文件夹中已有同名项目。"),
                    true,
                );
            }
            return;
        }
        match std::fs::rename(&old, &new) {
            Ok(()) => {
                tracing::info!(from = %old.display(), to = %new.display(), "item renamed");
                if self.state.is_portal_item(item) {
                    // Portal ids are path hashes. Update the old view's identity before
                    // the folder refresh so rename does not clear selection and focus.
                    let new_id = crate::state::portal_item_id(&new.to_string_lossy());
                    for w in self.fences.values() {
                        w.rekey_item(item, new_id);
                    }
                    self.refresh_portals_in(&[parent.to_path_buf()]);
                } else if self.state.rename_item(&old, &new) {
                    let fences: Vec<FenceId> = self.state.fences().iter().map(|f| f.id).collect();
                    for f in fences {
                        if self
                            .state
                            .fence(f)
                            .is_some_and(|fe| fe.items.iter().any(|r| r.item_id == item))
                        {
                            self.refresh_fence(f);
                        }
                    }
                    self.schedule_save();
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "rename failed");
                if let Some(t) = &self.tray {
                    t.show_info(
                        "PecoFence",
                        &pecofence_core::i18n::format("重命名失败：{0}", &[e.to_string()]),
                        true,
                    );
                }
            }
        }
    }

    /// The selection's paths for the shell; see [`shell::paths_for_shell`].
    pub(super) fn shell_paths_of(&self, items: &[ItemId]) -> Vec<PathBuf> {
        shell::paths_for_shell(
            items
                .iter()
                .filter_map(|id| self.state.item(*id))
                .filter_map(|it| it.key.as_path().map(PathBuf::from))
                .collect(),
        )
    }

    /// Runs one of the shell's canonical verbs (`delete`, `cut`, `copy`, `properties`) on the
    /// selection as if picked from Explorer's menu. `shift` = Shift held (permanent delete).
    pub(super) fn shell_verb_on_items(
        &mut self,
        fence: FenceId,
        items: &[ItemId],
        verb: &str,
        shift: bool,
    ) -> bool {
        let mut paths = self.shell_paths_of(items);
        if matches!(verb, "delete" | "cut" | "copy") {
            // The Recycle Bin cannot be deleted, cut or copied, and the other special items
            // only pretend to (their "delete" hides the desktop icon): leave them to Windows.
            paths.retain(|p| !shell::is_namespace_path(p));
        }
        if paths.is_empty() {
            return false;
        }
        let owner = self
            .window_for(fence)
            .map(|w| w.hwnd())
            .unwrap_or(self.control.hwnd());
        let refs: Vec<&std::path::Path> = paths.iter().map(|p| p.as_path()).collect();
        match ShellContextMenu::for_paths(&refs)
            .and_then(|mut m| m.invoke_verb(verb, owner, window::cursor_pos(), shift))
        {
            Ok(true) => {
                tracing::info!(count = paths.len(), verb, shift, "shell verb invoked");
                true
            }
            Ok(false) => {
                tracing::warn!(verb, "shell menu offers no such verb");
                false
            }
            Err(e) => {
                tracing::warn!(error = %e, verb, "shell verb failed");
                false
            }
        }
    }

    /// Delete key: the shell's own `delete` verb (Recycle Bin + its confirmation dialog);
    /// `permanent` = Shift+Delete (the shell's own 永久删除 confirmation).
    pub(super) fn delete_items(&mut self, fence: FenceId, items: &[ItemId], permanent: bool) {
        self.shell_verb_on_items(fence, items, "delete", permanent);
    }

    /// Alt+Enter / 属性: the shell's `properties` verb (multi-select gives one combined sheet).
    pub(super) fn show_properties(&mut self, fence: FenceId, items: &[ItemId]) {
        if self.shell_verb_on_items(fence, items, "properties", false) {
            return;
        }
        // Fallback: ShellExecuteEx "properties" on the first path.
        if let Some(p) = items
            .first()
            .and_then(|id| self.state.item(*id))
            .and_then(|it| it.key.as_path().map(PathBuf::from))
        {
            let owner = self
                .window_for(fence)
                .map(|w| w.hwnd())
                .unwrap_or(self.control.hwnd());
            let _ = shell::shell_execute(&p, Some("properties"), Some(owner));
        }
    }

    /// Ctrl+C / Ctrl+X: the shell's copy / cut verb puts the files on the clipboard exactly as
    /// Explorer does (`CF_HDROP` + `Preferred DropEffect`); cut items draw dimmed.
    pub(super) fn clipboard_verb(&mut self, fence: FenceId, items: &[ItemId], cut: bool) {
        if !self.shell_verb_on_items(fence, items, if cut { "cut" } else { "copy" }, false) {
            return;
        }
        self.cut_items = if cut {
            items.iter().copied().collect()
        } else {
            HashSet::new()
        };
        self.cut_clip_seq = clipboard::sequence();
        self.push_cut_items();
    }

    pub(super) fn push_cut_items(&self) {
        for w in self.fences.values() {
            w.set_cut_items(&self.cut_items);
        }
    }

    /// The clipboard changed since our cut (Explorer pasted, another copy happened): undim.
    pub(super) fn check_cut_clipboard(&mut self) {
        if !self.cut_items.is_empty() && clipboard::sequence() != self.cut_clip_seq {
            self.cut_items.clear();
            self.push_cut_items();
        }
    }

    /// Ctrl+V / 粘贴: the clipboard's files land in `fence` — desktop items change membership,
    /// foreign files move / copy to the desktop (or the portal's folder) first. Pasting copies
    /// into the folder they already live in makes Explorer's "xxx - 副本" duplicates (routed by
    /// the rules, since their names are only known to the shell).
    pub(super) fn paste_into(&mut self, fence: FenceId) {
        let Some((paths, cut)) = clipboard::file_list() else {
            return;
        };
        let dest = self.state.portal_path(fence).or_else(shell::user_desktop);
        if !cut && let Some(dest) = dest.as_ref() {
            let dest_key = ItemKey::from_path(&dest.to_string_lossy());
            let all_here = paths.iter().all(|p| {
                p.parent()
                    .is_some_and(|par| ItemKey::from_path(&par.to_string_lossy()) == dest_key)
            });
            if all_here {
                let owner = self.window_for(fence).map(|w| w.hwnd());
                self.start_fileop(FileOp {
                    paths,
                    dest: dest.clone(),
                    copy: true,
                    rename_on_collision: true,
                    owner,
                    then: FileOpThen::Duplicate,
                });
                return;
            }
        }
        self.handle(Command::ExternalDrop {
            paths,
            to: fence,
            mode: if cut {
                TransferMode::Move
            } else {
                TransferMode::Copy
            },
        });
        if cut {
            // Explorer empties the clipboard after pasting cut files.
            clipboard::clear();
            self.cut_items.clear();
            self.push_cut_items();
        }
    }

    pub(super) fn launch(&mut self, item: ItemId) {
        // A folder inside a navigating portal opens in place (Fences "Navigate").
        if let Some(fence) = self.state.portal_of_item(item)
            && self.state.fence(fence).is_some_and(|f| f.portal_navigate)
            && let Some(it) = self.state.item(item)
            && it.is_folder
            && let Some(p) = it.key.as_path().map(PathBuf::from)
        {
            self.portal_enter(fence, &p);
            return;
        }
        let Some(path) = self
            .state
            .item(item)
            .and_then(|it| it.key.as_path().map(PathBuf::from))
        else {
            return;
        };
        match shell::shell_execute(&path, None, None) {
            Ok(()) => {
                self.state.note_opened(item);
                // A fence sorted by open count reorders; persist lazily with the next save.
                let sorted: Vec<FenceId> = self
                    .state
                    .fences()
                    .iter()
                    .filter(|f| f.view.sort == SortMode::OpenCount && f.contains_item(item))
                    .map(|f| f.id)
                    .collect();
                for f in sorted {
                    self.refresh_fence(f);
                }
                self.schedule_save();
            }
            Err(e) => tracing::warn!(path = %path.display(), error = %e, "launch failed"),
        }
    }

    /// Moves items into `to`. Virtual fences only change membership (files stay put); a portal
    /// as source or target means a real file move (portal = folder view).
    pub(super) fn move_items(&mut self, items: &[ItemId], to: FenceId) {
        if self.state.portal_path(to).is_some() {
            // Namespace items cannot be moved into a folder; they stay where they are.
            let paths: Vec<PathBuf> = items
                .iter()
                .filter_map(|id| self.state.item(*id))
                .filter(|it| !it.is_namespace())
                .filter_map(|it| it.key.as_path().map(PathBuf::from))
                .collect();
            self.move_files_into_portal(paths, to);
            return;
        }
        let (portal_items, desktop_items): (Vec<ItemId>, Vec<ItemId>) = items
            .iter()
            .copied()
            .partition(|id| self.state.is_portal_item(*id));
        if !portal_items.is_empty() {
            self.move_portal_items_to_desktop(&portal_items, to);
        }
        if desktop_items.is_empty() {
            return;
        }
        let affected: Vec<FenceId> = self
            .state
            .fences()
            .iter()
            .filter(|f| f.id == to || f.items.iter().any(|r| desktop_items.contains(&r.item_id)))
            .map(|f| f.id)
            .collect();
        if self.state.move_items(&desktop_items, to) > 0 {
            for id in affected {
                self.refresh_fence(id);
            }
            self.schedule_save();
        }
    }
}
