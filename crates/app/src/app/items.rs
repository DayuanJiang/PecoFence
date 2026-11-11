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
        // 筛选: only what matches the fence's filter.
        let filter = self.fence_filter(fence.id);
        self.state
            .items_of(fence)
            .into_iter()
            .filter(|it| {
                filter
                    .as_deref()
                    .is_none_or(|q| filter::filter_keeps(it, q))
            })
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
        // Ctrl+Shift+N in a smart or tool fence: it has no folder to make it in and would not
        // show a desktop item routed to it.
        if portal_dir.is_none()
            && let Some(f) = self.state.fence(fence).filter(|f| !f.holds_items())
        {
            let kind = if f.smart.is_some() { "smart" } else { "tool" };
            tracing::info!(%fence, kind, folder, "new item refused: the fence holds no items");
            return;
        }
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
            self.move_items_unrecorded(&[id], fence);
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
        // In the icon view the name is centred under the icon, and so is the box's text.
        let centered = self
            .state
            .fence(fence)
            .is_some_and(|f| f.view.layout == ViewLayout::Icons);
        tracing::info!(?rect, centered, "item rename box opened");
        w.set_renaming(Some(item));
        crate::rename::begin_item_rename(
            rect,
            dpi,
            crate::layout::row_text_scale(self.ctx.chrome.label_size()),
            item,
            current,
            select_end,
            centered,
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
        if !matches!(verb, "cut" | "copy")
            && let Some(a) = self.anchor.borrow_mut().as_mut()
        {
            // Properties and the delete confirmation: closing them gives the fence the
            // keyboard back with its selection.
            a.dialog_opened_from(owner);
        }
        if verb == "delete" {
            self.start_shell_delete(paths, owner, shift);
            return true;
        }
        let refs: Vec<&std::path::Path> = paths.iter().map(|p| p.as_path()).collect();
        let started = Instant::now();
        let result = ShellContextMenu::for_paths(&refs)
            .and_then(|m| m.invoke_verb(verb, owner, window::cursor_pos(), shift));
        let ms = crate::share_log::millis(started.elapsed());
        match result {
            Ok(()) => {
                tracing::info!(count = paths.len(), verb, shift, ms, "shell verb invoked");
                true
            }
            Err(e) => {
                tracing::warn!(error = %e, verb, ms, "shell verb failed");
                false
            }
        }
    }

    /// Delete key: the shell's own `delete` verb (Recycle Bin + its confirmation dialog);
    /// `permanent` = Shift+Delete (the shell's own 永久删除 confirmation).
    pub(super) fn delete_items(&mut self, fence: FenceId, items: &[ItemId], permanent: bool) {
        if let Some(w) = self.window_for(fence) {
            w.expect_user_delete(items);
        }
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
    /// into the folder they already live in makes Explorer's "xxx - 副本" duplicates, which go
    /// into `fence` too (`claim_pasted_duplicates`). What was pasted is selected there
    /// (`select_pasted`).
    pub(super) fn paste_into(&mut self, fence: FenceId) {
        // A note (便签) holds no files: Ctrl+V on it does nothing.
        if self
            .state
            .fence(fence)
            .is_some_and(|f| f.tool.is_some() || f.smart.is_some())
        {
            tracing::info!(%fence, "paste into a tool or smart fence ignored");
            return;
        }
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
                let known = dir_entries(dest);
                let running = self.fileops_running;
                self.start_fileop(FileOp {
                    paths: paths.clone(),
                    dest: dest.clone(),
                    copy: true,
                    rename_on_collision: true,
                    owner,
                    then: FileOpThen::Duplicate,
                });
                if self.fileops_running > running {
                    // The copies go into the fence pasted into, whatever the rules say (as a
                    // drop into it does), and are selected there.
                    tracing::debug!(%fence, files = paths.len(), dir = %dest.display(), "paste copies files within their folder");
                    self.paste_dups.push(PasteDup {
                        serial: self.fileop_serial,
                        fence,
                        dir: dest.clone(),
                        known,
                        sources: paths,
                    });
                    self.pasted = Some(Pasted {
                        fence,
                        paths: Vec::new(),
                        selected: 0,
                        since: Instant::now(),
                    });
                }
                return;
            }
        }
        // Where the pasted files will be: in a folder fence's folder, else on the desktop
        // (files already there stay where they are and only change fence).
        let portal = self.state.portal_path(fence).is_some();
        let desktops = [shell::user_desktop(), shell::public_desktop()];
        let expected: Vec<PathBuf> = paths
            .iter()
            .filter_map(|p| {
                let on_desktop = !portal
                    && p.parent()
                        .is_some_and(|par| desktops.iter().flatten().any(|d| d == par));
                if on_desktop {
                    return Some(p.clone());
                }
                Some(dest.as_ref()?.join(shell::transferred_name(p)?))
            })
            .collect();
        self.pasted = Some(Pasted {
            fence,
            paths: expected,
            selected: 0,
            since: Instant::now(),
        });
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
        self.select_pasted();
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

    /// Copies that pastes still running have made so far go into the fence pasted into: a
    /// route for each (it bypasses the rules) and a place in the selection to come. Called
    /// before a desktop sync, so a copy never shows in the fence the rules would pick, and
    /// when the paste `done` has finished, which forgets that paste.
    pub(super) fn claim_pasted_duplicates(&mut self, done: Option<u64>) {
        let mut dups = std::mem::take(&mut self.paste_dups);
        for d in &mut dups {
            let landed = pasted_duplicates(&d.known, &dir_entries(&d.dir), &d.sources);
            if landed.is_empty() {
                continue;
            }
            let fence = d.fence;
            let virtual_fence = self.state.portal_path(fence).is_none();
            tracing::info!(%fence, copies = landed.len(), virtual_fence, "pasted copies found");
            for path in landed {
                d.known.insert(path.clone());
                if virtual_fence && !self.pending_routes.iter().any(|r| r.path == path) {
                    self.pending_routes.push(PendingRoute {
                        fence,
                        path: path.clone(),
                        since: Instant::now(),
                        rename: false,
                    });
                }
                if let Some(p) = self.pasted.as_mut().filter(|p| p.fence == fence) {
                    p.paths.push(path);
                }
            }
        }
        if let Some(serial) = done {
            dups.retain(|d| d.serial != serial);
        }
        self.paste_dups = dups;
    }

    /// Selects what the last paste put into its fence, as far as it shows there by now, with
    /// the first of them as the keyboard cursor (Explorer selects what it pasted). Runs again
    /// when more of it has arrived; forgotten once everything is selected or it took too long.
    pub(super) fn select_pasted(&mut self) {
        let Some(p) = self.pasted.as_ref() else {
            return;
        };
        let fence = p.fence;
        let copying = self.paste_dups.iter().any(|d| d.fence == fence);
        let keys: Vec<ItemKey> = p
            .paths
            .iter()
            .map(|path| ItemKey::from_path(&path.to_string_lossy()))
            .collect();
        let ids: Vec<ItemId> = self
            .state
            .fence(fence)
            .map(|f| {
                self.state
                    .items_of(f)
                    .into_iter()
                    .filter(|it| keys.contains(&it.key))
                    .map(|it| it.id)
                    .collect()
            })
            .unwrap_or_default();
        let mut selected = p.selected;
        if ids.len() > selected
            && let Some(w) = self.window_for(fence)
            && w.active_fence() == fence
        {
            selected = w.select_items(&ids);
            tracing::debug!(%fence, pasted = keys.len(), arrived = ids.len(), selected, "pasted files selected");
        }
        let expired =
            self.fileops_running == 0 && p.since.elapsed().as_secs() > PENDING_CREATION_SECS;
        if (!copying && selected >= keys.len()) || expired {
            self.pasted = None;
        } else if let Some(p) = self.pasted.as_mut() {
            p.selected = selected;
        }
    }

    /// Explorer empties the clipboard after pasting cut files.
    pub(super) fn finish_cut_paste(&mut self) {
        self.cut_paste = None;
        clipboard::clear();
        self.cut_items.clear();
        self.push_cut_items();
    }

    /// Opens `item`, which the user picked in fence `from`. `inside`: a folder may open inside
    /// the fence; false opens it in Explorer (one of several items opened at once, a Peek search
    /// match).
    pub(super) fn launch(&mut self, item: ItemId, from: FenceId, inside: bool) {
        let Some((path, is_folder)) = self
            .state
            .item(item)
            .and_then(|it| Some((PathBuf::from(it.path()?), it.is_folder)))
        else {
            return;
        };
        let (portal, navigate) = self.state.fence(from).map_or((false, false), |f| {
            (
                f.kind == FenceKind::FolderPortal,
                inside && f.navigates_folders(),
            )
        });
        // A folder inside a navigating portal opens in place (Fences "Navigate"). Two portals
        // can show the same folder, so only the one the item was opened from navigates.
        if portal && navigate && is_folder && self.state.is_portal_item(item) {
            self.portal_enter(from, &path);
            return;
        }
        // A regular fence with 「双击文件夹或文件夹快捷方式时在此栅栏内打开」 (off by default) shows a
        // folder, or a folder shortcut's target, inside itself, as Explorer opens it in its own
        // window.
        if !portal
            && navigate
            && let Some(dir) = folder_to_open(&path, is_folder)
            && self.open_folder_in_fence(from, &dir)
        {
            if !self.state.is_portal_item(item) {
                self.state.set_opened_by(from, item);
            }
            self.note_opened(item);
            return;
        }
        match shell::shell_execute(&path, None, None) {
            Ok(()) => {
                self.note_opened(item);
                self.folder_opened(item, from, &path, is_folder);
            }
            Err(e) => {
                tracing::warn!(path = %path.display(), place = shell::path_place(&path), error = %e, "launch failed")
            }
        }
    }

    /// 功能提示: a folder (or a shortcut to one) opened in Explorer may earn the folder tip.
    /// The cheap checks come first: a shortcut's target is read only when the tip could show.
    fn folder_opened(&mut self, item: ItemId, from: FenceId, path: &Path, is_folder: bool) {
        let often = self
            .state
            .item(item)
            .is_some_and(|it| it.open_count >= super::welcome::FOLDER_TIP_OPENS);
        if !often || !self.notices || !self.state.config.settings.feature_tips {
            return;
        }
        if let Some(dir) = folder_to_open(path, is_folder) {
            self.offer_folder_tip(item, from, &dir);
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

    /// Moves items into `to`. Virtual fences only change membership (files stay put), also while
    /// they show a folder opened in them; a portal as source or target means a real file move
    /// (portal = folder view).
    /// The user moved `items` into `to`: one step of Ctrl+Z in the fences, unless files move
    /// into a folder (Windows undoes those).
    pub(super) fn move_items(&mut self, items: &[ItemId], to: FenceId) {
        self.move_items_at(items, to, None);
    }

    /// [`Self::move_items`] for icons dropped between the icons of a 手动 fence: they go before
    /// display index `at` of `to` (counted before they arrived) instead of after the others.
    pub(super) fn move_items_at(&mut self, items: &[ItemId], to: FenceId, at: Option<usize>) {
        let places = if self.state.move_target_folder(to).is_none() {
            self.state.item_places(items)
        } else {
            Vec::new()
        };
        if self.move_items_placed(items, to, at) > 0 {
            self.note_items_for_undo(to, places);
        }
    }

    /// [`Self::move_items`] without the undo step; returns how many fence entries moved.
    pub(super) fn move_items_unrecorded(&mut self, items: &[ItemId], to: FenceId) -> usize {
        self.move_items_placed(items, to, None)
    }

    /// [`Self::move_items_at`] without the undo step; returns how many fence entries moved.
    fn move_items_placed(&mut self, items: &[ItemId], to: FenceId, at: Option<usize>) -> usize {
        // A smart or tool fence takes nothing: refused before a folder fence's file is moved
        // out to the desktop on its way there.
        if self.state.move_target_folder(to).is_none()
            && let Some(f) = self.state.fence(to).filter(|f| !f.holds_items())
        {
            let kind = if f.smart.is_some() { "smart" } else { "tool" };
            tracing::info!(fence = %to, kind, items = items.len(), "move refused: the fence holds no items");
            return 0;
        }
        if self.state.move_target_folder(to).is_some() {
            // Namespace items cannot be moved into a folder; they stay where they are.
            let paths: Vec<PathBuf> = items
                .iter()
                .filter_map(|id| self.state.item(*id))
                .filter(|it| !it.is_namespace())
                .filter_map(|it| it.path().map(PathBuf::from))
                .collect();
            self.move_files_into_portal(paths, to);
            return 0;
        }
        let (portal_items, desktop_items): (Vec<ItemId>, Vec<ItemId>) = items
            .iter()
            .copied()
            .partition(|id| self.state.is_portal_item(*id));
        if !portal_items.is_empty() {
            self.move_portal_items_to_desktop(&portal_items, to);
        }
        if desktop_items.is_empty() {
            return 0;
        }
        let affected: Vec<FenceId> = self
            .state
            .fences()
            .iter()
            .filter(|f| f.id == to || f.items.iter().any(|r| desktop_items.contains(&r.item_id)))
            .map(|f| f.id)
            .collect();
        let moved = self.state.move_items(&desktop_items, to);
        if moved > 0 {
            if let Some(index) = at {
                // Before the windows are refreshed, so the icons glide straight to that place.
                let placed = self.state.reorder_items(to, &desktop_items, index);
                tracing::debug!(
                    fence = %to,
                    index,
                    moved,
                    placed,
                    "icons dropped between the icons of a fence: placed there (false: not a 手动 fence)"
                );
            }
            for id in affected {
                self.refresh_fence(id);
            }
            self.schedule_save();
        }
        moved
    }
}

/// The entries directly in `dir` (before and after a paste that copies within it).
fn dir_entries(dir: &Path) -> HashSet<PathBuf> {
    std::fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).collect())
        .unwrap_or_default()
}

/// The copies a paste of `sources` has made in their own folder so far: the entries of `now`
/// that `known` does not hold and whose name is a source's name with a space and more added
/// to its stem ("报告 - 副本.txt", "报告 - Copy (2).txt" for 报告.txt) or, as for a folder, to
/// the whole name ("v1.2 - Copy"), so a file another program puts into the folder meanwhile
/// is not taken for one. Sorted by path.
pub(super) fn pasted_duplicates(
    known: &HashSet<PathBuf>,
    now: &HashSet<PathBuf>,
    sources: &[PathBuf],
) -> Vec<PathBuf> {
    let lower = |s: Option<&std::ffi::OsStr>| s.map(|s| s.to_string_lossy().to_lowercase());
    // Per source: its name and its stem, each with the space a copy's name goes on with, and
    // its extension.
    let names: Vec<(String, String, Option<String>)> = sources
        .iter()
        .filter_map(|p| {
            Some((
                lower(p.file_name())? + " ",
                lower(p.file_stem())? + " ",
                lower(p.extension()),
            ))
        })
        .collect();
    let mut copies: Vec<PathBuf> = now
        .difference(known)
        .filter(|p| {
            let (Some(name), Some(stem)) = (lower(p.file_name()), lower(p.file_stem())) else {
                return false;
            };
            let ext = lower(p.extension());
            names.iter().any(|(full, s, e)| {
                name.starts_with(full.as_str()) || (*e == ext && stem.starts_with(s.as_str()))
            })
        })
        .cloned()
        .collect();
    copies.sort();
    copies
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
