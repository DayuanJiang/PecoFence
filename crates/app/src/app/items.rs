//! Item commands: launch, inline rename, shell verbs, clipboard, moving items between fences,
//! routing of newly created desktop items, item view builder.

use super::*;
use pecofence_platform::dragdrop;

const VK_V: u32 = 0x56;

thread_local! {
    /// Ctrl+V is registered on the control window for the desktop (`App::follow_desktop_paste`).
    static DESKTOP_PASTE_KEY: Cell<bool> = const { Cell::new(false) };
}

/// The desktop's Ctrl+V was pressed with something else in front, or while the App was busy:
/// the shortcut is dropped and the key goes on to the window in front (the user still holds
/// Ctrl, so it gets its Ctrl+V).
pub(super) fn pass_on_desktop_paste(control: HWND, why: &'static str) {
    if DESKTOP_PASTE_KEY.replace(false) {
        hotkey::unregister(control, HOTKEY_DESKTOP_PASTE);
    }
    tracing::info!(why, "Ctrl+V passed on to the window in front");
    window::send_key(VK_V);
}

/// A paste through the shell, ready to run (`App::paste_shell_items`).
pub(super) struct ShellPaste {
    fence: FenceId,
    dest: PathBuf,
    /// The folder's entries before the paste.
    before: HashSet<PathBuf>,
    /// The names the source describes (`FileGroupDescriptorW`).
    names: Vec<String>,
    /// The routes made for them before the paste.
    routed: Vec<PathBuf>,
    on_desktop: bool,
}

/// Runs `job`'s paste: the shell's modal progress and name-conflict dialogs run here, so the
/// caller holds no App borrow.
pub(super) fn run_shell_paste(job: &ShellPaste) -> dragdrop::DropEffect {
    clipboard::paste_shell_items(&job.dest).unwrap_or_else(|error| {
        tracing::warn!(%error, folder = %job.dest.display(), "paste through the shell failed");
        dragdrop::DropEffect::None
    })
}

/// Outcome of an inline item rename (`App::rename_item_file`): either finished synchronously
/// (carrying the item id to focus, which changes for a renamed portal item) or handed off to
/// the shell's own prompt (elevation / access-denied fallback), which keeps keyboard focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ItemRenameOutcome {
    Finished(ItemId),
    ShellPrompt,
}

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
    /// a file inside the folder being renamed is open, or another program has the file open
    /// (feedback #207). The shell's rename takes it over with Explorer's own dialogs.
    Denied(String),
    /// The filesystem refused (message).
    Io(String),
}

fn rename_error(e: std::io::Error) -> RenameError {
    tracing::warn!(error = %e, "rename failed");
    // ERROR_SHARING_VIOLATION, ERROR_LOCK_VIOLATION: another program has the file open. A notice
    // said "(os error 32)"; Explorer names the program and offers to try again.
    if matches!(e.raw_os_error(), Some(32 | 33)) {
        tracing::info!("rename: the file is in use; the shell's rename shows Explorer's dialog");
        return RenameError::Denied(e.to_string());
    }
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
        let started = Instant::now();
        let items = self.state.items_of(fence);
        let (listed, total) = (started.elapsed(), items.len());
        let kept: Vec<_> = items
            .into_iter()
            .filter(|it| {
                filter
                    .as_deref()
                    .is_none_or(|q| filter::filter_keeps(it, q))
            })
            .collect();
        let matched = started.elapsed();
        let views: Vec<ItemView> = kept
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
            .collect();
        // What a list of thousands costs before the window gets it, part by part.
        if total >= 500 {
            let ms = crate::share_log::millis;
            tracing::debug!(
                fence = %fence.id,
                items = total,
                shown = views.len(),
                sort_ms = ms(listed),
                filter_ms = ms(matched - listed),
                views_ms = ms(started.elapsed() - matched),
                "item list built for the window"
            );
        }
        views
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
                self.note_created_for_undo(vec![path.clone()]);
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
        // The places kept for routes that just landed (`RoutePlace`), and none for a route
        // that is gone (expired; a failed or cancelled move takes its own away).
        let mut places: HashMap<ItemId, RoutePlace> = HashMap::new();
        let arrived: Vec<PathBuf> = self
            .route_places
            .keys()
            .filter(|p| self.state.item_id_for_path(p).is_some())
            .cloned()
            .collect();
        for path in arrived {
            if let (Some(id), Some(place)) = (
                self.state.item_id_for_path(&path),
                self.route_places.remove(&path),
            ) {
                places.insert(id, place);
            }
        }
        self.route_places
            .retain(|p, _| self.pending_routes.iter().any(|r| r.path == *p));
        for (id, fence, rename) in landed {
            // Let go between the icons of the fence: in front of the icon the line stood
            // before, wherever that icon is by now (after the others when it is gone), and
            // gliding in from where it was let go.
            let place = places.remove(&id);
            let at = place.as_ref().and_then(|p| p.before).and_then(|anchor| {
                self.state
                    .fence(fence)
                    .and_then(|f| self.state.items_of(f).iter().position(|it| it.id == anchor))
            });
            if let Some(place) = &place {
                tracing::debug!(
                    %fence,
                    anchor = place.before.is_some(),
                    at = ?at,
                    glide = place.from.is_some(),
                    "routed icon lands where it was let go"
                );
                if let (Some(from), Some(w)) = (place.from, self.window_for(fence)) {
                    w.set_drop_landing(id, from);
                }
            }
            let moved = self.move_items_placed(&[id], fence, at);
            // Already in this fence when it came back (the rules filed it here, or the fence
            // still knew it from before): nothing moved, only its place changes.
            if moved == 0
                && let Some(index) = at
                && self.state.reorder_items(fence, &[id], index)
            {
                self.refresh_fence(fence);
                self.schedule_save();
            }
            if rename {
                // The rules may have filed the item here already: then nothing moved, the
                // window does not list it yet, and the label the edit box needs has no rect.
                self.refresh_fence(fence);
                self.begin_item_rename(fence, id);
            }
        }
    }

    /// Feedback #207: what the desktop's menu (新建, 粘贴) makes after a right-click on the empty
    /// desktop of a monitor that 「桌面」 does not stand on stays on that monitor, as on a
    /// Windows desktop: it goes into the plain fence nearest the click there, after its other
    /// icons. Only files no rule took (the rules come first), within [`super::fences::MENU_PLACE_FOR`]
    /// of the click; with no fence there that can take them they stay in 「桌面」. One new empty
    /// file or folder (what 新建 makes) gets its name box, as Windows starts renaming it.
    pub(super) fn route_to_menu_monitor(&mut self, added: &[PathBuf]) {
        let Some((at, pt)) = self.desktop_menu_at else {
            return;
        };
        let Some(inbox) = self.state.inbox_id() else {
            return;
        };
        let new: Vec<ItemId> = added
            .iter()
            .filter_map(|p| self.state.item_id_for_path(p))
            .filter(|id| {
                self.state.fence(inbox).is_some_and(|f| {
                    f.items.iter().any(|r| {
                        r.item_id == *id && r.assigned_by == pecofence_core::AssignedBy::Migration
                    })
                })
            })
            .collect();
        let Some(wa) = self.work_area_at(pt.x, pt.y) else {
            return;
        };
        let on_wa = |r: RECT| {
            let (x, y) = ((r.left + r.right) / 2, (r.top + r.bottom) / 2);
            x >= wa.left && x < wa.right && y >= wa.top && y < wa.bottom
        };
        // Where 「桌面」 stands: its window, or the place it is saved at while it has none (hidden
        // while empty), so the files of a one-monitor desktop still go to it.
        let inbox_host = self.state.host_of(inbox);
        let inbox_rect = match self.fences.get(&inbox_host) {
            Some(w) => Some(w.rect()),
            None => self.state.fence(inbox_host).map(|f| {
                let px = self.state.fence_px_rect(f);
                RECT {
                    left: px.left,
                    top: px.top,
                    right: px.right,
                    bottom: px.bottom,
                }
            }),
        };
        if new.is_empty()
            || inbox_rect.is_some_and(on_wa)
            || at.elapsed() >= super::fences::MENU_PLACE_FOR
        {
            return;
        }
        // Plain fences that show their items on that monitor now: not a folder, smart or tool
        // fence, not one with a folder opened inside, not 「桌面」; not locked (a lock is its
        // window's, the host's), hidden or on another virtual desktop. A tab host takes them
        // into the tab on screen.
        let gap = |r: RECT| {
            let dx = (r.left - pt.x).max(pt.x - r.right).max(0) as f64;
            let dy = (r.top - pt.y).max(pt.y - r.bottom).max(0) as f64;
            dx.hypot(dy)
        };
        let pick = self
            .fences
            .iter()
            .filter(|(host, w)| {
                self.on_current_desktop(**host)
                    && pecofence_platform::desktop::is_visible(w.hwnd())
                    && on_wa(w.rect())
                    && self.state.fence(**host).is_some_and(|f| !f.locked)
            })
            .map(|(host, w)| (*host, self.state.active_tab_of(*host), w.rect()))
            .filter(|(_, id, _)| {
                self.state.portal_path(*id).is_none()
                    && self
                        .state
                        .fence(*id)
                        .is_some_and(|f| f.kind == FenceKind::Virtual && f.holds_items())
            })
            .map(|(host, id, r)| (gap(r), host, id))
            .min_by(|a, b| a.0.total_cmp(&b.0));
        // The click is used up by these icons, as 「桌面」's own (#238).
        self.desktop_menu_at = None;
        let Some((distance, host, fence)) = pick else {
            tracing::info!(click = ?pt, monitor = ?wa, items = new.len(), "desktop menu on a monitor without 「桌面」: no fence there takes new items, 「桌面」 keeps them");
            return;
        };
        let rolled = self.fences.get(&host).is_some_and(|w| w.is_rolled());
        tracing::info!(click = ?pt, monitor = ?wa, %fence, distance = distance.round(), rolled, items = new.len(), "desktop menu on a monitor without 「桌面」: new items go to the fence nearest the click");
        // A rolled fence opens, as a click on it would, so the new icon and its name box show.
        if rolled {
            self.set_roll(host, false);
        }
        let moved = self.move_items_placed(&new, fence, None);
        // What 新建 makes is empty; an empty file pasted with the menu gets the box too (accepted).
        let fresh = |id: &ItemId| {
            self.state
                .item(*id)
                .and_then(|it| it.path().map(PathBuf::from))
                .is_some_and(|p| {
                    std::fs::read_dir(&p).map_or_else(
                        |_| std::fs::metadata(&p).is_ok_and(|m| m.len() == 0),
                        |mut d| d.next().is_none(),
                    )
                })
        };
        if moved > 0 && new.len() == 1 && fresh(&new[0]) {
            self.refresh_fence(fence);
            self.begin_item_rename(fence, new[0]);
        }
        self.schedule_save();
    }

    /// Moves portal items out to the real desktop and routes them into `to` once they appear.
    pub(super) fn move_portal_items_to_desktop(
        &mut self,
        items: &[ItemId],
        to: FenceId,
        at: Option<usize>,
    ) {
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
        // Dragged onto the fence: once its file is on the desktop each icon lands in front of
        // the icon the insertion line stood before (`at`, a display index of `to` now) and
        // glides in from where it was let go, as an icon of a virtual fence does at once.
        let before = at.and_then(|i| {
            self.state
                .fence(to)
                .and_then(|f| self.state.items_of(f).get(i).map(|it| it.id))
        });
        let mut let_go = self
            .window_for(to)
            .map(|w| w.take_drop_landing())
            .unwrap_or_default();
        for id in items {
            let from = let_go.remove(id);
            let path = self
                .state
                .item(*id)
                .and_then(|it| it.path().map(PathBuf::from))
                .and_then(|p| p.file_name().map(|n| desktop.join(n)));
            if let Some(path) = path
                && routed.contains(&path)
                && (before.is_some() || from.is_some())
            {
                self.route_places.insert(path, RoutePlace { before, from });
            }
        }
        tracing::debug!(
            fence = %to,
            routed = routed.len(),
            at = ?at,
            anchor = before.is_some(),
            places = self.route_places.len(),
            "files leave a folder fence for the desktop: where their icons will land is kept with the routes"
        );
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
        // In the icon view the name is centred under the icon and wraps: so does the box.
        let centered = self
            .state
            .fence(fence)
            .is_some_and(|f| f.view.layout == ViewLayout::Icons);
        // As wide as the name is drawn: a name fitted to the wider box keeps it in the box.
        let inset = w.item_label_inset(item);
        tracing::info!(?rect, centered, inset, "item rename box opened");
        let ink = w.label_ink(self.theme_mode);
        let label_size = self.ctx.chrome.label_size();
        crate::rename::begin_item_rename(
            rect,
            dpi,
            &self.ctx.chrome.label_family(),
            label_size,
            crate::layout::row_text_scale(label_size),
            fence,
            item,
            current,
            select_end,
            centered.then_some(inset),
            self.queue.clone(),
            ink,
        );
        // After the box is up and painted: the fence then draws the icon without its name.
        // The other way round the name was gone for a frame before the box showed.
        w.set_renaming(Some(item));
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
            Some(crate::rename::RenameTarget::PluginLine(..)) | None => (None, None),
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

    /// Reconciles every fence's cached `win_active` flag with the real foreground window once
    /// a rename popup closes without handing focus back to the fence. While `PecoFence.Rename`
    /// was open, `on_activate` kept `win_active` true so the selection stayed accented; when
    /// focus then leaves for another program or the desktop, the fence receives no second
    /// `WM_ACTIVATE(WA_INACTIVE)` and would otherwise stay stuck active.
    pub(super) fn reconcile_fence_activation(&self) {
        if self.fileops_running > 0 {
            return;
        }
        let front = desktop::root_ancestor(desktop::foreground_window());
        let cls = desktop::class_name(front);
        if cls == "#32768" {
            return;
        }
        let rename_owner = (cls == "PecoFence.Rename")
            .then(|| {
                crate::rename::active_fence().and_then(|f| self.window_for(f).map(|w| w.hwnd()))
            })
            .flatten();
        for w in self.fences.values() {
            let active = front == w.hwnd() || rename_owner == Some(w.hwnd());
            w.sync_active(active);
        }
    }

    /// Renames the file behind an item to `name` (+ the hidden extension Explorer would keep).
    /// The inline-edit path: problems are toasted; returns the item id to focus afterwards
    /// (which may have been rekeyed in a folder portal) or `ShellPrompt` when an elevation /
    /// shell prompt owns the interaction.
    pub(super) fn rename_item_file(&mut self, item: ItemId, name: &str) -> ItemRenameOutcome {
        let Some(it) = self.state.item(item) else {
            return ItemRenameOutcome::Finished(item);
        };
        if it.is_namespace() {
            return ItemRenameOutcome::Finished(item);
        }
        let name = name.trim();
        // `old` is the file name (hidden extension included), not the displayed spelling. An
        // emptied box keeps the name, as in Explorer (else the hidden suffix alone, ".txt" or
        // ".lnk", would become the whole name).
        if name.is_empty() || name == it.display_name {
            return ItemRenameOutcome::Finished(item);
        }
        let file_name = it
            .path()
            .and_then(|p| p.rsplit('\\').next())
            .unwrap_or_default();
        // Explorer's editing name drops `.lnk` / hidden extensions: keep whatever it dropped.
        let suffix = crate::rename::rename_hidden_suffix(&it.display_name, file_name, it.is_folder);
        let old = it.path().map(PathBuf::from);
        let new_name = format!("{name}{suffix}");
        let portal = self.state.is_portal_item(item);
        let mut outcome = ItemRenameOutcome::Finished(item);
        let toast = match self.rename_item_to(item, &new_name) {
            Ok(new) => {
                // In a sorted fence the new name may send the icon out of view: the fence
                // follows it (a folder fence's item has a new id, made from its path).
                let id = if portal {
                    crate::state::portal_item_id(&new.to_string_lossy())
                } else {
                    item
                };
                outcome = ItemRenameOutcome::Finished(id);
                for w in self.fences.values() {
                    w.reveal_item(id);
                }
                // One step of Ctrl+Z in the fences, unless the name on disk stayed the same
                // ("note" to "note.").
                if let Some(old) = old.filter(|old| *old != new) {
                    self.note_rename_for_undo(old, new);
                }
                None
            }
            Err(RenameError::Unsupported) => None,
            Err(RenameError::Denied(e)) => match old {
                // Explorer asks for administrator permission here, or says which program has
                // the file open with 重试; the shell's own rename does the same.
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
                    if self.start_shell_rename(old, new_name, owner) {
                        outcome = ItemRenameOutcome::ShellPrompt;
                    }
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
        outcome
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

    /// Runs the shell's `properties` verb on the selection as if picked from Explorer's menu, or
    /// deletes it (`delete`: the shell's file operation on a worker thread, see
    /// [`App::start_shell_delete`]). `shift` = Shift held (permanent delete).
    pub(super) fn shell_verb_on_items(
        &mut self,
        fence: FenceId,
        items: &[ItemId],
        verb: &str,
        shift: bool,
    ) -> bool {
        let mut paths = self.shell_paths_of(items);
        if verb == "delete" {
            // The Recycle Bin cannot be deleted, and the other special items only pretend to
            // (their "delete" hides the desktop icon): leave them to Windows.
            paths.retain(|p| !shell::is_namespace_path(p));
        }
        if paths.is_empty() {
            return false;
        }
        let owner = self
            .window_for(fence)
            .map(|w| w.hwnd())
            .unwrap_or(self.control.hwnd());
        if let Some(a) = self.anchor.borrow_mut().as_mut() {
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
        let _loads = pecofence_platform::shell_menu::LoadWatch::new("Properties");
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

    /// Delete key: the shell's own delete (Recycle Bin + its confirmation dialog);
    /// `permanent` = Shift+Delete (the shell's own 永久删除 confirmation).
    pub(super) fn delete_items(&mut self, fence: FenceId, items: &[ItemId], permanent: bool) {
        // The fence waits only for what the shell is asked to delete (the Recycle Bin and the
        // other special icons stay), and not at all when nothing is.
        let real: Vec<ItemId> = items
            .iter()
            .copied()
            .filter(|id| self.state.item(*id).is_none_or(|it| !it.is_namespace()))
            .collect();
        if let Some(w) = self.window_for(fence) {
            w.expect_user_delete(&real);
        }
        if !self.shell_verb_on_items(fence, items, "delete", permanent)
            && let Some(w) = self.window_for(fence)
        {
            w.settle_user_delete(false);
        }
    }

    /// Alt+Enter / 属性: the shell's `properties` verb (multi-select gives one combined sheet).
    pub(super) fn show_properties(&mut self, fence: FenceId, items: &[ItemId]) {
        // The Windows part of the item menus is off after a crash in a menu extension: the
        // verb would load every menu extension again (`ShellContextMenu::invoke_verb`). The
        // property sheet of the first file is opened without the menu.
        if self.windows_menu_off() {
            let path = self.shell_paths_of(items).into_iter().next();
            let owner = self
                .window_for(fence)
                .map(|w| w.hwnd())
                .unwrap_or(self.control.hwnd());
            let shown = path.as_deref().is_some_and(|p| {
                !shell::is_namespace_path(p) && shell::show_properties_sheet(p, owner)
            });
            tracing::info!(
                count = items.len(),
                shown,
                "properties without the Windows menu: its part is off"
            );
            return;
        }
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

    /// Ctrl+C / Ctrl+X: the files go on the clipboard exactly as Explorer puts them there (the
    /// shell's data object of the files with `Preferred DropEffect`, `clipboard::set_files`);
    /// cut items draw dimmed. The Recycle Bin and the other special items cannot be cut or
    /// copied.
    pub(super) fn clipboard_verb(&mut self, _fence: FenceId, items: &[ItemId], cut: bool) {
        let mut paths = self.shell_paths_of(items);
        paths.retain(|p| !shell::is_namespace_path(p));
        if paths.is_empty() {
            return;
        }
        let refs: Vec<&std::path::Path> = paths.iter().map(|p| p.as_path()).collect();
        let started = Instant::now();
        let _loads =
            pecofence_platform::shell_menu::LoadWatch::new(if cut { "Cut" } else { "Copy" });
        let result = clipboard::set_files(&refs, cut);
        let ms = crate::share_log::millis(started.elapsed());
        if let Err(e) = result {
            tracing::warn!(error = %e, cut, ms, "files not put on the clipboard");
            return;
        }
        tracing::info!(count = paths.len(), cut, ms, "files put on the clipboard");
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

    /// Esc after Ctrl+X: the cut is taken back. Explorer's desktop does the same (measured on
    /// Windows Server 2025: the icon is whole again and the clipboard is empty; a click
    /// elsewhere leaves both as they are). The clipboard is emptied only while it still
    /// holds our cut, and never under a paste of it that is still running.
    pub(super) fn cancel_cut(&mut self) {
        if self.cut_items.is_empty() || self.cut_paste.is_some() {
            return;
        }
        let ours = clipboard::sequence() == self.cut_clip_seq;
        tracing::info!(
            items = self.cut_items.len(),
            clipboard_emptied = ours,
            "Esc: cut taken back"
        );
        if ours {
            clipboard::clear();
        }
        self.cut_items.clear();
        self.push_cut_items();
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
            self.paste_shell_items(fence);
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
                    self.pasted = Some(self.paste_started(fence, Vec::new()));
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
        self.pasted = Some(self.paste_started(fence, expected));
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

    /// Ctrl+V on Explorer's desktop itself (after a click on an empty spot of it, or Win+D):
    /// Windows' desktop pastes nothing while its icons are hidden, and the keys go to it, not
    /// to a fence. So while it is in front and PecoFence hides its icons, Ctrl+V is registered
    /// for the desktop (`on_desktop_paste_key` decides at each press by the window in front
    /// then: our own windows bring no foreground event). Runs when another program's window
    /// comes to the front, at the start, and when the icons are hidden or shown.
    pub(super) fn follow_desktop_paste(&mut self, foreground: HWND) {
        let want = self.desktop_in_front(foreground)
            && self.state.inbox_id().is_some()
            && self.state.config.settings.hide_real_icons
            && !self.no_hide_icons
            && !self.windows_desktop;
        if want == DESKTOP_PASTE_KEY.get() {
            return;
        }
        if want {
            let ok = hotkey::register(
                self.control.hwnd(),
                HOTKEY_DESKTOP_PASTE,
                hotkey::mods::CONTROL | hotkey::mods::NOREPEAT,
                VK_V,
            );
            if !ok {
                tracing::info!("Ctrl+V on the desktop: taken by another program, left to it");
                return;
            }
        } else {
            hotkey::unregister(self.control.hwnd(), HOTKEY_DESKTOP_PASTE);
        }
        DESKTOP_PASTE_KEY.set(want);
        tracing::debug!(on = want, "Ctrl+V on the desktop pastes into 「桌面」");
    }

    /// Is `front` Explorer's desktop window (Progman, a WorkerW, the view in them)?
    fn desktop_in_front(&self, front: HWND) -> bool {
        self.anchor
            .try_borrow()
            .is_ok_and(|a| a.as_ref().is_some_and(|a| a.is_desktop_window(front)))
    }

    /// `HOTKEY_DESKTOP_PASTE` was pressed: decided by the window in front now. Explorer's
    /// desktop with its icons hidden: a paste into 「桌面」 is queued (it runs once this
    /// message is done). Anything else in front (a fence, its name box, the settings, another
    /// program): false, and the caller hands the key back (`pass_on_desktop_paste`).
    pub(super) fn on_desktop_paste_key(&mut self) -> bool {
        let front = desktop::foreground_window();
        let on_desktop = self.desktop_in_front(front);
        let hidden = pecofence_platform::shell_icons::desktop_icons_hidden();
        let inbox = self.state.inbox_id();
        tracing::info!(
            on_desktop,
            icons_hidden = hidden,
            inbox = inbox.is_some(),
            front = %desktop::class_name(desktop::root_ancestor(front)),
            "Ctrl+V while the desktop's shortcut is registered"
        );
        match inbox.filter(|_| on_desktop && hidden) {
            Some(inbox) => {
                self.queue.push(Command::Paste { fence: inbox });
                true
            }
            None => {
                self.follow_desktop_paste(front);
                false
            }
        }
    }

    /// Pastes what only the shell can write out: files copied inside a .zip, an Outlook
    /// attachment, a phone's photos (the clipboard has their contents or their shell items, no
    /// file paths, `clipboard::shell_items`). The clipboard's data object goes to the
    /// destination folder's own drop target, as Explorer's 粘贴 does: the shell writes the files
    /// and asks about names that are taken. That runs a modal loop, so the paste only gets ready
    /// here and the control window runs it once the App is free (`take_shell_paste`,
    /// `finish_shell_paste`). What lands on the desktop for a virtual fence goes into that fence
    /// and is selected, as any pasted file.
    fn paste_shell_items(&mut self, fence: FenceId) {
        let Some((data, cut)) = clipboard::shell_items() else {
            tracing::info!(%fence, clipboard = %clipboard::describe(), "nothing to paste: no files on the clipboard");
            return;
        };
        let names = dragdrop::file_group_descriptor_names(&data);
        drop(data);
        let portal = self.state.portal_path(fence);
        let on_desktop = portal.is_none();
        let Some(dest) = portal.or_else(shell::user_desktop) else {
            return;
        };
        let before = dir_entries(&dest);
        // The files the source describes are routed into the fence before they appear, so the
        // rules never file them elsewhere first (a phone's files describe no names: those are
        // routed when the paste is over).
        let expected = crate::fence_window::shell_drop_landed(&dest, &before, &before, &names);
        if on_desktop {
            for path in &expected {
                self.pending_routes.push(PendingRoute {
                    fence,
                    path: path.clone(),
                    since: Instant::now(),
                    rename: false,
                });
            }
        }
        self.pasted = Some(self.paste_started(fence, expected.clone()));
        tracing::info!(%fence, cut, described = names.len(), folder = %dest.display(), "paste through the shell queued: items without file paths (a .zip's files, an attachment)");
        self.shell_paste = Some(ShellPaste {
            fence,
            dest,
            before,
            names,
            routed: if on_desktop { expected } else { Vec::new() },
            on_desktop,
        });
        window::post_message(self.control.hwnd(), WM_APP_COMMAND, 0, 0);
    }

    /// The paste `paste_shell_items` got ready, for the control window to run with no App
    /// borrow (`run_shell_paste`).
    pub(super) fn take_shell_paste(&mut self) -> Option<ShellPaste> {
        self.shell_paste.take()
    }

    /// The shell's part of a paste is over: what landed goes into the fence and is selected.
    /// Whether Windows' undo list holds such a paste is not known, so Ctrl+Z stops at it and
    /// says so, rather than undo an older step the user did not mean (an older paste).
    pub(super) fn finish_shell_paste(&mut self, job: ShellPaste, effect: dragdrop::DropEffect) {
        let ShellPaste {
            fence,
            dest,
            before,
            names,
            routed,
            on_desktop,
        } = job;
        let landed =
            crate::fence_window::shell_drop_landed(&dest, &before, &dir_entries(&dest), &names);
        tracing::info!(
            %fence,
            ?effect,
            described = names.len(),
            landed = landed.len(),
            folder = %dest.display(),
            "pasted through the shell"
        );
        // A move the source finishes later (a file cut inside a .zip) reports no effect: what
        // arrived decides. Nothing yet: the routes stay for files still to come, and expire.
        let arrived = landed.iter().any(|p| p.exists());
        if effect == dragdrop::DropEffect::None && !arrived {
            return;
        }
        self.pending_routes.retain(|r| !routed.contains(&r.path));
        self.note_fileop_for_undo(SHELL_PASTE_STEP, false);
        if let Some(p) = self.pasted.as_mut().filter(|p| p.fence == fence) {
            // The shell's dialogs may have waited for the user: the files count as just pasted.
            p.since = Instant::now();
            for path in &landed {
                if !p.paths.contains(path) {
                    p.paths.push(path.clone());
                }
            }
        }
        self.shell_drop_landed(
            fence,
            if on_desktop { landed } else { Vec::new() },
            on_desktop,
        );
        self.select_pasted();
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

    /// The record of a paste into `fence` that starts now and will bring `paths`: it notes
    /// what the fence has selected, so `select_pasted` can tell whether the user selected
    /// something else since.
    fn paste_started(&self, fence: FenceId, paths: Vec<PathBuf>) -> Pasted {
        let left = self
            .window_for(fence)
            .filter(|w| w.active_fence() == fence)
            .map(|w| w.selected_items().into_iter().collect())
            .unwrap_or_default();
        Pasted {
            fence,
            paths,
            chosen: Vec::new(),
            left,
            since: Instant::now(),
        }
    }

    /// Selects what the last paste put into its fence, as far as it shows there by now, with
    /// the first of them as the keyboard cursor (Explorer selects what it pasted). Runs again
    /// when more of it has arrived, and adds that to the selection only while the selection is
    /// still what the paste left: once the user has selected something else, or the paste is
    /// too long ago, the rest arrives unselected. Forgotten once everything is selected.
    pub(super) fn select_pasted(&mut self) {
        let Some(p) = self.pasted.as_ref() else {
            return;
        };
        let fence = p.fence;
        // Checked before anything is selected: a file that shows only now (it was hidden by
        // the fence's filter, or it never came) is not "what I just pasted" any more.
        if self.fileops_running == 0 && p.since.elapsed().as_secs() > PENDING_CREATION_SECS {
            tracing::debug!(%fence, "pasted files are no longer selected: the paste is too long ago");
            self.pasted = None;
            return;
        }
        let Some(w) = self.window_for(fence).filter(|w| w.active_fence() == fence) else {
            return;
        };
        let now: HashSet<ItemId> = w.selected_items().into_iter().collect();
        if now != p.left {
            tracing::debug!(
                %fence,
                selected = now.len(),
                left = p.left.len(),
                "pasted files are no longer selected: the selection changed since the paste"
            );
            self.pasted = None;
            return;
        }
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
        let mut chosen = p.chosen.clone();
        let renaming = w.renaming_item().is_some();
        if renaming {
            tracing::debug!(%fence, arrived = ids.len(), "pasted files wait: a name box is open in the fence");
        }
        if let Some(want) = pasted_to_select(&chosen, &ids, renaming) {
            let got = w.select_items(&want);
            tracing::debug!(
                %fence,
                pasted = keys.len(),
                arrived = ids.len(),
                selected = got.len(),
                "pasted files selected"
            );
            // None of them shows (the fence's filter hides them): the selection is untouched.
            if !got.is_empty() {
                chosen = got;
            }
        }
        let done = !copying && ids.len() >= keys.len() && ids.iter().all(|id| chosen.contains(id));
        if done {
            self.pasted = None;
        } else if let Some(p) = self.pasted.as_mut()
            && p.chosen != chosen
        {
            p.left = chosen.iter().copied().collect();
            p.chosen = chosen;
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
        // An icon of a folder shown in a fence whose file is gone: the folder was renamed or
        // deleted and the fence has not noticed yet (its watcher follows a renamed folder; the
        // check for that runs once a minute). Explorer would put up its "Windows cannot find"
        // box inside the call below, and every fence would wait until it is closed. The fence
        // is read again instead and shows what is there now.
        if self.state.is_portal_item(item)
            && let Some(dir) = self.state.portal_path(from)
            && !path.exists()
        {
            tracing::info!(fence = %from, place = shell::path_place(&path), "launch: the file is gone; its fence is re-read");
            self.portal_watchers.remove(&from);
            self.refresh_portals_in(std::slice::from_ref(&dir));
            return;
        }
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
        // GitHub #244: the Store edition installed on another drive does not see that drive's
        // `\Users\<name>\AppData`, so a shortcut to a program there would only bring up Windows'
        // "the item has been changed or moved" box. A process outside the package opens it.
        if let Some(target) = hidden_from_package(&path) {
            let outcome = shell::shell_execute_outside(&path);
            tracing::info!(
                place = shell::path_place(&target),
                started = outcome.is_ok(),
                error = outcome.as_ref().err().map(|e| e.to_string()),
                "launch: the shortcut's program is hidden from the Store edition here; opened through rundll32"
            );
            if outcome.is_ok() {
                self.note_opened(item);
                return;
            }
        }
        let _loads = pecofence_platform::shell_menu::LoadWatch::new("Launch");
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
            self.move_portal_items_to_desktop(&portal_items, to, at);
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

/// The icons a paste selects now: the pasted ones it selected before (`chosen`) plus those
/// that have `arrived` since. None when nothing new has arrived, and while a name box is open
/// in the fence (F2 on a file that arrived earlier): selecting scrolls, and a scroll closes
/// the box with whatever is typed so far, so the paste waits for the next change of the files.
pub(super) fn pasted_to_select(
    chosen: &[ItemId],
    arrived: &[ItemId],
    renaming: bool,
) -> Option<Vec<ItemId>> {
    if renaming || arrived.iter().all(|id| chosen.contains(id)) {
        return None;
    }
    let mut want = chosen.to_vec();
    want.extend(arrived.iter().copied().filter(|id| !chosen.contains(id)));
    Some(want)
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

/// For the Store edition: the program of the `.lnk` at `path` when it lies where the package
/// sees its own data instead of the real folders and this process therefore cannot see it
/// (`shell::under_package_view_of_appdata`, GitHub #244). Cheap for other files: the shortcut
/// is read only in the Store edition, the disk only for such a target.
fn hidden_from_package(path: &Path) -> Option<PathBuf> {
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("lnk"))
        || !pecofence_platform::process::is_packaged()
    {
        return None;
    }
    shell::shortcut_target_expanded(path)
        .filter(|t| shell::under_package_view_of_appdata(t) && !t.exists())
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
