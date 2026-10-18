//! Item commands: launch, inline rename, shell verbs, clipboard, moving items between fences,
//! routing of newly created desktop items, item view builder.

use super::*;

/// Why [`App::rename_item_to`] declined.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum RenameError {
    /// Namespace item, or no path / parent folder to rename within.
    Unsupported,
    /// Empty, `.`/`..`, or contains a character Windows forbids in file names.
    InvalidName,
    /// Another entry of that name is already in the folder.
    Exists,
    /// Access denied (message): the folder needs administrator rights (the Public Desktop),
    /// or a file inside the folder being renamed is open.
    Denied(String),
    /// The filesystem refused (message).
    Io(String),
}

fn rename_error(e: std::io::Error) -> RenameError {
    tracing::warn!(error = %e, "rename failed");
    match e.kind() {
        std::io::ErrorKind::PermissionDenied => RenameError::Denied(e.to_string()),
        std::io::ErrorKind::AlreadyExists => RenameError::Exists,
        _ => RenameError::Io(e.to_string()),
    }
}

impl App {
    pub(super) fn item_views(&self, fence: &pecofence_core::Fence) -> Vec<ItemView> {
        self.state
            .items_of(fence)
            .into_iter()
            .map(|it| {
                let path = PathBuf::from(it.path().unwrap_or(""));
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
            // `create_new`: a file another program put there since the check is not truncated.
            std::fs::File::create_new(&path).map(drop)
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
        let running = self.fileops_running > 0;
        self.pending_routes
            .retain(|p| running || p.since.elapsed().as_secs() <= PENDING_CREATION_SECS);
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
                // The rules may have filed the item here already: then nothing moved, the
                // window does not list it yet, and the label the edit box needs has no rect.
                self.refresh_fence(fence);
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
            .filter_map(|it| it.path().map(PathBuf::from))
            .collect();
        if paths.is_empty() {
            return;
        }
        // A desktop file of the same name is not the one being moved: no route for it (see the
        // external drop in `handle`).
        let routed: Vec<PathBuf> = paths
            .iter()
            .filter_map(|p| p.file_name().map(|n| desktop.join(n)))
            .filter(|p| self.state.live_item_by_path(p).is_none())
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
    /// The inline-edit path: problems are toasted, nothing is returned.
    pub(super) fn rename_item_file(&mut self, item: ItemId, name: &str) {
        let Some(it) = self.state.item(item) else {
            return;
        };
        if it.is_namespace() {
            return;
        }
        let name = name.trim();
        // `old` is the file name (hidden extension included), not the displayed spelling. An
        // emptied box keeps the name, as in Explorer (else the hidden suffix alone, ".txt" or
        // ".lnk", would become the whole name).
        if name.is_empty() || name == it.display_name {
            return;
        }
        let file_name = it
            .path()
            .and_then(|p| p.rsplit('\\').next())
            .unwrap_or_default();
        // Explorer's editing name drops `.lnk` / hidden extensions: keep whatever it dropped.
        let suffix = crate::rename::rename_hidden_suffix(&it.display_name, file_name, it.is_folder);
        let old = it.path().map(PathBuf::from);
        let new_name = format!("{name}{suffix}");
        let toast = match self.rename_item_to(item, &new_name) {
            Ok(_) | Err(RenameError::Unsupported) => None,
            Err(RenameError::Denied(e)) => match old {
                // Explorer asks for administrator permission here; the shell's own rename
                // does the same.
                Some(old) => {
                    let owner = self
                        .state
                        .portal_of_item(item)
                        .or_else(|| {
                            let f = self.state.fences().iter().find(|f| f.contains_item(item));
                            f.map(|f| f.id)
                        })
                        .and_then(|f| self.window_for(f))
                        .map_or_else(|| self.control.hwnd(), |w| w.hwnd());
                    // The watcher reports the prompt's rename; it must not be refused for a
                    // deleted file's record at the new name (as in `rename_item_to`).
                    self.state.forget_orphan_at(&old.with_file_name(&new_name));
                    self.start_shell_rename(old, new_name, owner);
                    None
                }
                None => Some(pecofence_core::i18n::format("重命名失败：{0}", &[e])),
            },
            Err(RenameError::InvalidName) => {
                self.invalid_name_toast();
                None
            }
            Err(RenameError::Exists) => {
                Some(pecofence_core::i18n::text("此文件夹中已有同名项目。").to_string())
            }
            Err(RenameError::Io(e)) => Some(pecofence_core::i18n::format("重命名失败：{0}", &[e])),
        };
        if let (Some(text), Some(t)) = (toast, &self.tray) {
            t.show_info("PecoFence", &text, true);
        }
    }

    /// The toast for a name no file can have (the inline editor's Enter and commit paths).
    pub(super) fn invalid_name_toast(&self) {
        if let Some(t) = &self.tray {
            t.show_info(
                "PecoFence",
                pecofence_core::i18n::text("名称不能为空，也不能包含 \\ / : * ? \" < > |"),
                true,
            );
        }
    }

    /// Renames the file behind `item` to `file_name` (the complete new file name, extension
    /// included) inside its folder and updates the item table, views and portal ids. Shared by
    /// inline editing and the CLI; returns the new path.
    pub(super) fn rename_item_to(
        &mut self,
        item: ItemId,
        file_name: &str,
    ) -> std::result::Result<PathBuf, RenameError> {
        let Some(it) = self.state.item(item) else {
            return Err(RenameError::Unsupported);
        };
        if it.is_namespace() {
            return Err(RenameError::Unsupported);
        }
        // Win32 drops trailing dots and spaces from a name: strip them here too, or the
        // record would be keyed "note." for a file that is "note" on disk.
        let file_name = file_name.trim().trim_end_matches(['.', ' ']);
        if crate::rename::invalid_file_name(file_name) {
            return Err(RenameError::InvalidName);
        }
        let Some(old) = it.path().map(PathBuf::from) else {
            return Err(RenameError::Unsupported);
        };
        let Some(parent) = old.parent().map(Path::to_path_buf) else {
            return Err(RenameError::Unsupported);
        };
        let new = parent.join(file_name);
        if new == old {
            // Only trailing dots or spaces differed ("note" to "note."): nothing to rename.
            return Ok(new);
        }
        if ItemKey::from_path(&new.to_string_lossy()) == ItemKey::from_path(&old.to_string_lossy())
        {
            // Same name, another case: the item keeps its key, its path and name take the new
            // spelling. (Callers already skipped unchanged names.)
            shell::rename_no_replace(&old, &new).map_err(rename_error)?;
            if self.state.rename_item(&old, &new) {
                self.schedule_save();
            }
            self.refresh_fences_with(item);
            return Ok(new);
        }
        if new.exists() {
            return Err(RenameError::Exists);
        }
        // Not `std::fs::rename`: that would replace a file created at `new` since the check.
        shell::rename_no_replace(&old, &new).map_err(rename_error)?;
        tracing::info!(from = %old.display(), to = %new.display(), "item renamed");
        if self.state.is_portal_item(item) {
            // Portal ids are path hashes. Update the old view's identity before the folder
            // refresh so rename does not clear selection and focus.
            let new_id = crate::state::portal_item_id(&new.to_string_lossy());
            for w in self.fences.values() {
                w.rekey_item(item, new_id);
            }
            self.refresh_portals_in(&[parent]);
        } else {
            self.state.forget_orphan_at(&new);
            if self.state.rename_item(&old, &new) {
                self.refresh_fences_with(item);
                self.schedule_save();
            }
        }
        Ok(new)
    }

    /// Re-renders every fence that lists `item`.
    fn refresh_fences_with(&mut self, item: ItemId) {
        let fences: Vec<FenceId> = self
            .state
            .fences()
            .iter()
            .filter(|f| f.items.iter().any(|r| r.item_id == item))
            .map(|f| f.id)
            .collect();
        for f in fences {
            self.refresh_fence(f);
        }
    }

    /// The selection's paths for the shell; see [`shell::paths_for_shell`].
    pub(super) fn shell_paths_of(&self, items: &[ItemId]) -> Vec<PathBuf> {
        shell::paths_for_shell(
            items
                .iter()
                .filter_map(|id| self.state.item(*id))
                .filter_map(|it| it.path().map(PathBuf::from))
                .collect(),
        )
    }

    /// Runs one of the shell's canonical verbs (`delete`, `cut`, `copy`, `properties`) on the
    /// selection as if picked from Explorer's menu. `shift` = Shift held (permanent delete).
    /// `delete` runs on a worker thread (see [`App::start_shell_delete`]).
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
        if verb == "delete" {
            self.start_shell_delete(paths, owner, shift);
            return true;
        }
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
            .and_then(|it| it.path().map(PathBuf::from))
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
        let running = self.fileops_running;
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
            if self.fileops_running > running {
                // Foreign files are still being moved by a worker: emptied when it succeeds,
                // so a cancelled conflict dialog leaves the files on the clipboard for another
                // try (`on_fileop_done`). The serial names that operation (another transfer
                // finishing first must not settle this paste); the sequence number tells
                // whether the clipboard still holds this paste by then.
                self.cut_paste = Some((self.fileop_serial, clipboard::sequence()));
            } else {
                self.finish_cut_paste();
            }
        }
    }

    /// Explorer empties the clipboard after pasting cut files.
    pub(super) fn finish_cut_paste(&mut self) {
        self.cut_paste = None;
        clipboard::clear();
        self.cut_items.clear();
        self.push_cut_items();
    }

    /// Opens `item`, which the user picked in fence `from`.
    pub(super) fn launch(&mut self, item: ItemId, from: FenceId) {
        let Some((path, is_folder)) = self
            .state
            .item(item)
            .and_then(|it| Some((PathBuf::from(it.path()?), it.is_folder)))
        else {
            return;
        };
        // A folder, or a folder shortcut's target, opens inside the fence it was picked in, as
        // Explorer opens it in its own window (Fences "Navigate"). Two portals can show the same
        // folder, so only the one the item was opened from navigates.
        if self.state.fence(from).is_some_and(|f| f.portal_navigate)
            && let Some(dir) = folder_to_open(&path, is_folder)
            && self.open_folder_in_fence(from, &dir)
        {
            self.note_opened(item);
            return;
        }
        match shell::shell_execute(&path, None, None) {
            Ok(()) => self.note_opened(item),
            Err(e) => tracing::warn!(path = %path.display(), error = %e, "launch failed"),
        }
    }

    /// Counts an open of `item`; a fence sorted by open count reorders.
    fn note_opened(&mut self, item: ItemId) {
        self.state.note_opened(item);
        // Persist lazily with the next save.
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

    /// Moves items into `to`. Virtual fences only change membership (files stay put); a portal
    /// as source or target means a real file move (portal = folder view).
    pub(super) fn move_items(&mut self, items: &[ItemId], to: FenceId) {
        if self.state.portal_path(to).is_some() {
            // Namespace items cannot be moved into a folder; they stay where they are.
            let paths: Vec<PathBuf> = items
                .iter()
                .filter_map(|id| self.state.item(*id))
                .filter(|it| !it.is_namespace())
                .filter_map(|it| it.path().map(PathBuf::from))
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

/// The folder that double-clicking the item at `path` opens: the item itself, or the folder a
/// `.lnk` shortcut points to.
fn folder_to_open(path: &Path, is_folder: bool) -> Option<PathBuf> {
    if is_folder {
        return Some(path.to_path_buf());
    }
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("lnk"))
    {
        return None;
    }
    shell::shortcut_target_expanded(path).filter(|target| target.is_dir())
}
