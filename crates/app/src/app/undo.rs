//! Ctrl+Z in a fence, as on the desktop: undoes the newest thing the user did in the fences.
//! Put back here: a fence move or resize, icons moved between fences, a fence deleted from
//! its menu, a 「立即应用」 of the sorting rules, a rename made in a fence, a folder or text
//! file made from a fence. File operations the shell recorded in Windows' own undo list (a
//! delete to the Recycle Bin, files moved or copied into a folder or onto the desktop, a
//! paste) are handed to Windows: Ctrl+Z runs the desktop view's undo command. Layout steps and
//! file steps share one stack, newest first. A short stack in memory, gone at restart and
//! when the whole layout is replaced (import, backup, snapshot restore).

use super::*;
use crate::state::DeletedFence;
use items::RenameError;

/// Steps kept; the oldest go first.
const UNDO_KEPT: usize = 20;
/// Bounds changes this close together are one step: a group drag ends with one per fence.
const SAME_GESTURE: Duration = Duration::from_millis(400);

pub(super) enum UndoStep {
    /// Fences moved or resized: where each one was (its expanded rect, device px).
    Bounds {
        at: Instant,
        rects: Vec<(FenceId, RECT)>,
    },
    /// Desktop icons moved into `to`: where each one was.
    Items {
        to: FenceId,
        places: Vec<(FenceId, pecofence_core::ItemRef)>,
    },
    /// A fence deleted from its menu.
    Deleted(Box<DeletedFence>),
    /// A fence sent away from the current virtual desktop (its menu, Settings): the desktops
    /// its window showed on before (empty: every desktop).
    Desktops { fence: FenceId, before: Vec<String> },
    /// A fence made by hand (the rectangle's button, a fence's or the tray's menu): Ctrl+Z
    /// takes it away again while nothing would go with it (no icons, no rule, no note text).
    NewFence(FenceId),
    /// Icons 「立即应用」 sorted: where each was, grouped by the fence it went to.
    Refiled {
        id: u64,
        places: Vec<(FenceId, Vec<(FenceId, pecofence_core::ItemRef)>)>,
    },
    /// A file renamed in a fence (renamed directly, so Windows' undo list does not hold it).
    Renamed { from: PathBuf, to: PathBuf },
    /// Folders or files made from a fence (made directly too): Ctrl+Z moves them to the
    /// Recycle Bin, as Explorer's own undo of 新建 does, without a record in Windows' undo list
    /// (a record would be the newest one there, and the next step handed to Windows would
    /// bring the item back instead of undoing its own operation). Each path comes with the
    /// item's creation time: whatever else has that name by the time of the undo (the item
    /// was renamed, another one was made under its first name) is not it and stays.
    Created {
        items: Vec<(PathBuf, Option<std::time::SystemTime>)>,
    },
    /// A file operation the shell recorded in Windows' undo list: Ctrl+Z passes the undo on.
    /// Windows undoes the newest operation of its list, which is this one unless another
    /// program's came after it, exactly as Ctrl+Z on the desktop would.
    Shell { what: &'static str },
    /// A file operation nothing can undo (a permanent delete): Ctrl+Z says so and stops there,
    /// rather than put back an older step the user did not mean.
    Permanent { what: &'static str },
    /// A fence dragged by its title onto another fence's title: it became a tab there. `tabs`
    /// are the tabs its own window had (itself among them, in strip order), `before` where that
    /// window stood before the drag (its expanded rect, device px).
    Merged {
        fence: FenceId,
        tabs: Vec<FenceId>,
        before: RECT,
    },
    /// A tab dragged out of its window, or split out from the tab menu. `settled` once the
    /// drag that tore it off has ended: until then the new window's move is part of this step.
    Split {
        change: Box<pecofence_core::TabDetach>,
        settled: bool,
    },
}

/// What the notice's 撤销 undoes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UndoNotice {
    /// The fence deleted from its menu.
    Deleted(FenceId),
    /// The 「立即应用」 step of this id.
    Refiled(u64),
    /// The fence sent to another virtual desktop.
    Desktops(FenceId),
}

/// Ids of the 「立即应用」 steps, so the notice undoes its own one.
static NEXT_REFILED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl UndoStep {
    fn kind(&self) -> &'static str {
        match self {
            Self::Bounds { .. } => "bounds",
            Self::Items { .. } => "items",
            Self::Deleted(_) => "deleted fence",
            Self::Desktops { .. } => "virtual desktops",
            Self::NewFence(_) => "new fence",
            Self::Refiled { .. } => "rules applied",
            Self::Renamed { .. } => "rename",
            Self::Created { .. } => "new item",
            Self::Shell { what } | Self::Permanent { what } => what,
            Self::Merged { .. } => "merge into tabs",
            Self::Split { .. } => "tab split out",
        }
    }
}

impl App {
    fn push_undo(&mut self, step: UndoStep) {
        tracing::info!(
            kind = step.kind(),
            depth = self.undo.len() + 1,
            "undo step noted"
        );
        self.undo.push(step);
        if self.undo.len() > UNDO_KEPT {
            self.undo.remove(0);
        }
    }

    /// The whole layout was replaced: no step applies to it any more.
    pub(super) fn clear_undo(&mut self, why: &'static str) {
        if !self.undo.is_empty() {
            tracing::info!(steps = self.undo.len(), why, "undo steps dropped");
            self.undo.clear();
        }
        self.undo_notice = None;
    }

    /// A user move or resize of `fence` that left `before` (its expanded rect until then).
    pub(super) fn note_bounds_for_undo(&mut self, fence: FenceId, before: RECT) {
        // The drag that tore this tab off ends here: where the new window was let go belongs
        // to the split, which Ctrl+Z takes back as a whole.
        if let Some(UndoStep::Split { change, settled }) = self.undo.last_mut()
            && change.tab == fence
            && !*settled
        {
            *settled = true;
            tracing::debug!(%fence, "undo: the torn-off tab's move stays part of its split");
            return;
        }
        let now = Instant::now();
        if let Some(UndoStep::Bounds { at, rects }) = self.undo.last_mut()
            && now.duration_since(*at) < SAME_GESTURE
            && !rects.iter().any(|(f, _)| *f == fence)
        {
            rects.push((fence, before));
            *at = now;
            return;
        }
        self.push_undo(UndoStep::Bounds {
            at: now,
            rects: vec![(fence, before)],
        });
    }

    /// `fence` was dropped on another fence's title and is a tab there now. The drag noted
    /// where its window stood before (a Bounds step): that step becomes the merge's, so Ctrl+Z
    /// splits the fence out again at that place. A merge that no move of the window preceded
    /// (a tab torn off and dropped on another strip in one drag) is covered by its split.
    pub(super) fn note_merge_for_undo(&mut self, fence: FenceId, tabs: &[FenceId]) {
        let Some(UndoStep::Bounds { rects, .. }) = self.undo.last_mut() else {
            tracing::debug!(%fence, "undo: merge not noted, no move of the window before it");
            return;
        };
        let Some(i) = rects.iter().position(|(f, _)| *f == fence) else {
            tracing::debug!(%fence, "undo: merge not noted, the last move was another fence's");
            return;
        };
        let (_, before) = rects.remove(i);
        if rects.is_empty() {
            self.undo.pop();
        }
        self.push_undo(UndoStep::Merged {
            fence,
            tabs: tabs.to_vec(),
            before,
        });
    }

    /// `fence`, a window of its own until now, was merged into another from the tab menu's
    /// 合并到…: Ctrl+Z splits it out again where it stood (`before`, its expanded rect).
    pub(super) fn note_menu_merge_for_undo(
        &mut self,
        fence: FenceId,
        tabs: &[FenceId],
        before: RECT,
    ) {
        self.push_undo(UndoStep::Merged {
            fence,
            tabs: tabs.to_vec(),
            before,
        });
    }

    /// A tab left its window (`change`), by a drag (`from_drag`) or from the tab menu.
    pub(super) fn note_split_for_undo(
        &mut self,
        change: pecofence_core::TabDetach,
        from_drag: bool,
    ) {
        self.push_undo(UndoStep::Split {
            change: Box::new(change),
            settled: !from_drag,
        });
    }

    /// The tear-off was cancelled (Esc, the right button): its step goes with it.
    pub(super) fn drop_split_step(&mut self, tab: FenceId) {
        if matches!(self.undo.last(), Some(UndoStep::Split { change, .. }) if change.tab == tab) {
            self.undo.pop();
            tracing::debug!(%tab, "undo: the cancelled tear-off is no step");
        }
    }

    /// Desktop icons the user moved into `to` from `places` (see `AppState::item_places`).
    pub(super) fn note_items_for_undo(
        &mut self,
        to: FenceId,
        places: Vec<(FenceId, pecofence_core::ItemRef)>,
    ) {
        let places: Vec<_> = places.into_iter().filter(|(f, _)| *f != to).collect();
        if !places.is_empty() {
            self.push_undo(UndoStep::Items { to, places });
        }
    }

    pub(super) fn note_deleted_for_undo(&mut self, deleted: DeletedFence) {
        self.push_undo(UndoStep::Deleted(Box::new(deleted)));
    }

    /// The user made `fence` by hand. Without a step of its own, Ctrl+Z right after it undid
    /// whatever came before (an older move of another fence).
    pub(super) fn note_new_fence_for_undo(&mut self, fence: FenceId) {
        self.push_undo(UndoStep::NewFence(fence));
    }

    /// The user renamed a file in a fence, `from` to `to`.
    pub(super) fn note_rename_for_undo(&mut self, from: PathBuf, to: PathBuf) {
        self.push_undo(UndoStep::Renamed { from, to });
    }

    /// The user made the folders or files `paths` from a fence.
    pub(super) fn note_created_for_undo(&mut self, paths: Vec<PathBuf>) {
        if !paths.is_empty() {
            let items = paths
                .into_iter()
                .map(|p| {
                    let born = std::fs::metadata(&p).and_then(|m| m.created()).ok();
                    (p, born)
                })
                .collect();
            self.push_undo(UndoStep::Created { items });
        }
    }

    /// A file operation the user started in a fence has finished (`what`). `recorded`: the
    /// shell put it in Windows' undo list; false for one nothing can undo.
    pub(super) fn note_fileop_for_undo(&mut self, what: &'static str, recorded: bool) {
        self.push_undo(if recorded {
            UndoStep::Shell { what }
        } else {
            UndoStep::Permanent { what }
        });
    }

    /// The item whose file is at `path` now: on the desktop or in a folder a fence shows.
    fn item_at(&self, path: &Path) -> Option<ItemId> {
        self.state.live_item_by_path(path).or_else(|| {
            let id = crate::state::portal_item_id(&path.to_string_lossy());
            self.state.is_portal_item(id).then_some(id)
        })
    }

    /// Passes Ctrl+Z on to Windows (see [`UndoStep::Shell`]). The fences follow through the
    /// folder watchers once Windows has put the files back.
    fn ask_windows_to_undo(&self) -> bool {
        let view = self
            .anchor
            .borrow()
            .as_ref()
            .and_then(|a| desktop::resolve_icon_host(a.generation))
            .map(|h| h.def_view);
        let asked = view.is_some_and(pecofence_platform::shell_icons::undo_last_file_operation);
        if !asked {
            tracing::warn!(?view, "Windows' undo not asked: no desktop view");
        }
        asked
    }

    /// 「立即应用」 (settings page, tray menu, 快速添加): sorts the desktop by the rules and notes
    /// the moves for Ctrl+Z. Returns how many icons moved and the step's id (None: nothing
    /// moved, no step).
    pub(super) fn apply_rules_now(&mut self) -> (usize, Option<u64>) {
        let entries = shell::enumerate_desktop();
        let refiled = self.state.apply_rules_all(&entries);
        self.refresh_all();
        self.schedule_save();
        if refiled.moved == 0 {
            return (0, None);
        }
        let id = NEXT_REFILED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.push_undo(UndoStep::Refiled {
            id,
            places: refiled.places,
        });
        (refiled.moved, Some(id))
    }

    /// Ctrl+Z: undoes the newest step. A file operation started here that is still running
    /// has no step yet (it is noted when the shell reports back): the undo waits for it and
    /// `drain_fileops` runs it then, so a Ctrl+Z right after a Delete undoes that delete and
    /// not the step before it.
    pub(super) fn undo_last(&mut self, from: &'static str) {
        if self.fileops_running > 0 {
            tracing::info!(
                from,
                running = self.fileops_running,
                "undo waits for the file operation in flight"
            );
            self.undo_waiting = true;
            return;
        }
        let Some(step) = self.undo.pop() else {
            tracing::info!(from, "nothing to undo");
            return;
        };
        self.apply_undo(step, from);
    }

    /// The 撤销 of a notice: puts that fence back, or those icons, whatever came after it.
    fn undo_from_notice(&mut self, target: UndoNotice) {
        let Some(i) = self.undo.iter().rposition(|s| match (s, target) {
            (UndoStep::Deleted(d), UndoNotice::Deleted(fence)) => d.fence.id == fence,
            (UndoStep::Refiled { id, .. }, UndoNotice::Refiled(want)) => *id == want,
            (UndoStep::Desktops { fence, .. }, UndoNotice::Desktops(want)) => *fence == want,
            _ => false,
        }) else {
            tracing::info!(?target, "no longer undoable");
            return;
        };
        let step = self.undo.remove(i);
        self.apply_undo(step, "notice");
    }

    fn apply_undo(&mut self, step: UndoStep, from: &'static str) {
        let kind = step.kind();
        let done = match step {
            UndoStep::Bounds { rects, .. } => {
                let mut n = 0;
                for (fence, rect) in rects {
                    n += usize::from(self.put_fence_at(fence, rect));
                }
                n
            }
            UndoStep::Items { to, places } => {
                let moved = self.state.put_items_back(&places, to);
                if moved > 0 {
                    let mut fences: Vec<FenceId> = places.iter().map(|(f, _)| *f).collect();
                    fences.push(to);
                    fences.dedup();
                    for f in fences {
                        self.refresh_fence(f);
                    }
                }
                moved
            }
            UndoStep::Deleted(d) => {
                let id = d.fence.id;
                let portal = d.fence.kind == FenceKind::FolderPortal;
                let back = self.state.undelete_fence(*d);
                if back {
                    self.resync_windows();
                    if portal {
                        self.refresh_portals();
                    }
                    self.refresh_all();
                    self.push_settings_state();
                }
                if self
                    .undo_notice
                    .as_ref()
                    .is_some_and(|(_, t)| *t == UndoNotice::Deleted(id))
                    && let Some((notice, _)) = self.undo_notice.take()
                {
                    notice.leave();
                }
                usize::from(back)
            }
            UndoStep::Desktops { fence, before } => {
                let back = self.state.fence(fence).is_some()
                    && self.set_fence_desktops(fence, before, "undo");
                if self
                    .undo_notice
                    .as_ref()
                    .is_some_and(|(_, t)| *t == UndoNotice::Desktops(fence))
                {
                    self.undo_notice = None;
                }
                if back {
                    self.push_settings_state();
                }
                usize::from(back)
            }
            UndoStep::NewFence(fence) => {
                // Still there and still empty: it goes as it came. With icons, a rule or a
                // note's text in it by now the step is spent; 删除栅栏 in its menu asks.
                let empty =
                    self.state.fence(fence).is_some() && self.delete_fence_prompt(fence).is_none();
                tracing::info!(%fence, empty, "new fence undone");
                if empty {
                    let gone = self.window_for(fence).map(|w| w.rect());
                    self.delete_fence(fence);
                    if let Some(gone) = gone {
                        self.keyboard_to_nearest_fence(gone);
                    }
                }
                usize::from(empty)
            }
            UndoStep::Refiled { id, places } => {
                let mut n = 0;
                let mut touched: Vec<FenceId> = Vec::new();
                for (to, back) in &places {
                    let moved = self.state.put_items_back(back, *to);
                    if moved > 0 {
                        n += moved;
                        for f in std::iter::once(*to).chain(back.iter().map(|(f, _)| *f)) {
                            if !touched.contains(&f) {
                                touched.push(f);
                            }
                        }
                    }
                }
                for f in touched {
                    self.refresh_fence(f);
                }
                if self
                    .undo_notice
                    .as_ref()
                    .is_some_and(|(_, t)| *t == UndoNotice::Refiled(id))
                    && let Some((notice, _)) = self.undo_notice.take()
                {
                    notice.leave();
                }
                n
            }
            UndoStep::Renamed { from, to } => {
                let name = from.file_name().map(|n| n.to_string_lossy().into_owned());
                match (self.item_at(&to), name) {
                    (Some(item), Some(name)) => match self.rename_item_to(item, &name) {
                        Ok(_) => 1,
                        Err(error) => {
                            tracing::warn!(?error, to = %to.display(), "rename not undone");
                            if error == RenameError::Exists {
                                self.toast(
                                    pecofence_core::i18n::text("此文件夹中已有同名项目。")
                                        .to_string(),
                                );
                            }
                            0
                        }
                    },
                    _ => {
                        tracing::info!(to = %to.display(), "rename not undone: no such item any more");
                        0
                    }
                }
            }
            UndoStep::Created { items } => {
                let count = items.len();
                tracing::info!(count, first = ?items.first(), "new items undone: to the Recycle Bin");
                self.start_undo_created(items);
                count
            }
            UndoStep::Shell { .. } => usize::from(self.ask_windows_to_undo()),
            UndoStep::Permanent { .. } => {
                self.toast(pecofence_core::i18n::text("永久删除的文件无法撤销。").to_string());
                0
            }
            UndoStep::Merged {
                fence,
                tabs,
                before,
            } => usize::from(self.undo_merge(fence, &tabs, before)),
            UndoStep::Split { change, .. } => usize::from(self.undo_split(*change)),
        };
        tracing::info!(kind, from, done, left = self.undo.len(), "undone");
        if done > 0 {
            self.schedule_save();
        }
    }

    /// Takes a merge by drag back: `fence` leaves the window it was dropped on, gets the tabs
    /// it brought along again and stands where it stood before the drag. A fence that is no
    /// tab any more (split out by hand since) only moves back there.
    fn undo_merge(&mut self, fence: FenceId, tabs: &[FenceId], before: RECT) -> bool {
        if self.state.fence(fence).is_none() {
            tracing::info!(%fence, "merge not undone: the fence is gone");
            return false;
        }
        let host = self.state.host_of(fence);
        if host != fence {
            let (x, y) = ((before.left + before.right) / 2, before.top);
            if self.detach_tab(fence, x, y, false).is_none() {
                tracing::info!(%fence, %host, "merge not undone: the tab could not leave");
                return false;
            }
            // The tabs its window had, in their order (those still in the window it left).
            let mut brought = 0;
            for (k, tab) in tabs.iter().enumerate() {
                if *tab != fence
                    && self.state.host_of(*tab) == host
                    && self.state.attach_tab(*tab, fence)
                {
                    self.state.reorder_tab(fence, *tab, k);
                    brought += 1;
                }
            }
            if brought > 0 {
                self.state.set_active_tab(fence, fence);
                self.resync_windows();
                self.apply_fence_view(host);
                self.apply_fence_view(fence);
            }
            tracing::debug!(%fence, %host, brought, "undo: merged fence split out again");
        }
        self.put_fence_at(fence, before)
    }

    /// Takes a split back: the tab returns to its window at the place it had in the strip. A
    /// tab that the same drag dropped on another window leaves that one first.
    fn undo_split(&mut self, change: pecofence_core::TabDetach) -> bool {
        let tab = change.tab;
        if self.state.fence(tab).is_some() && self.state.host_of(tab) != tab {
            let _ = self.state.detach_tab(tab);
        }
        let back = self.cancel_tab_detach(change);
        tracing::debug!(%tab, back, "undo: split-out tab back in its window");
        if back {
            self.give_keyboard(self.state.host_of(tab), "undo of a split");
        }
        back
    }

    /// Moves `fence` back to `rect` (its expanded rect) the way `fence move` does. False when
    /// it is gone, a tab, or placed by 像桌面一样排列.
    fn put_fence_at(&mut self, fence: FenceId, rect: RECT) -> bool {
        if self.state.fence(fence).is_none_or(|f| f.tab_host.is_some()) {
            return false;
        }
        let Some(w) = self.fences.get(&fence) else {
            return false;
        };
        if w.is_desktop_arranged() {
            return false;
        }
        let rolled = self
            .state
            .fence(fence)
            .map_or_else(|| w.is_rolled(), |f| f.rolled_up || w.is_rolled());
        self.ctx.behavior.release_push(w.hwnd());
        if rolled && self.state.fence(fence).is_some_and(|f| f.expand_upward) {
            w.set_row_at_bottom(true);
        }
        let h = rect.bottom - rect.top;
        w.restore_geometry(rect, rolled, h);
        self.record_bounds(fence, rect, rolled, h, "undo");
        self.apply_auto_height(fence);
        true
    }

    /// An undo card is up and something newer can be undone: the card takes the newer texts
    /// where it stands and its 撤销 undoes `target` (Ctrl+Z still reaches the older step after
    /// it). False when no undo card is up or it cannot take the texts.
    fn retarget_undo_notice(&mut self, title: &str, body: &str, target: UndoNotice) -> bool {
        let Some((notice, shown)) = self.undo_notice.as_mut() else {
            return false;
        };
        let action = pecofence_core::i18n::text("撤销");
        match notice.set_content(title, body, action, crate::notice::locale()) {
            Ok(()) => {
                tracing::info!(from = ?*shown, to = ?target, "undo notice names the newer step");
                *shown = target;
                true
            }
            Err(error) => {
                tracing::warn!(%error, "undo notice did not take the newer step");
                false
            }
        }
    }

    /// What the undo card keeps clear of: the fences, and a card of another kind that is up
    /// (a tip, the update card, the rating offer). The delete used to get no card then.
    fn undo_notice_avoid_rects(&self) -> Vec<RECT> {
        let cards = self.notice_rects();
        if !cards.is_empty() {
            tracing::info!(cards = cards.len(), "undo notice shows beside another card");
        }
        let mut avoid = self.notice_avoid_rects();
        avoid.extend(cards);
        avoid
    }

    /// After a delete from the fence menu: 「已删除栅栏“X”」 with 撤销. The card of an earlier
    /// undo offer takes the new texts; beside another kind of card (a tip, the update card) it
    /// shows at the nearest free place.
    pub(super) fn offer_undo_delete(&mut self, fence: FenceId, title: &str) {
        let title = pecofence_core::i18n::format("已删除栅栏“{0}”", &[title.to_string()]);
        let body = pecofence_core::i18n::text("点“撤销”，或在栅栏上按 Ctrl+Z，就能把它找回来。");
        if self.retarget_undo_notice(&title, body, UndoNotice::Deleted(fence)) {
            return;
        }
        self.undo_notice = None;
        match crate::notice::Notice::show(
            &self.ctx,
            &title,
            body,
            pecofence_core::i18n::text("撤销"),
            crate::notice::locale(),
            &self.undo_notice_avoid_rects(),
        ) {
            Ok(notice) => {
                tracing::info!(%fence, "undo notice shown");
                self.undo_notice = Some((notice, UndoNotice::Deleted(fence)));
            }
            Err(error) => tracing::warn!(%error, "undo notice not shown"),
        }
    }

    /// After 「立即应用」 moved `moved` icons (step `id`): 「已按规则整理 N 个项目」 with 撤销. The
    /// card of an earlier undo offer takes the new texts; beside another kind of card it shows
    /// at the nearest free place. Returns whether it is shown.
    pub(super) fn offer_undo_refiled(&mut self, id: u64, moved: usize) -> bool {
        let title = pecofence_core::i18n::format("已按规则整理 {0} 个项目", &[moved.to_string()]);
        let body =
            pecofence_core::i18n::text("点“撤销”，或在栅栏上按 Ctrl+Z，就能把它们放回原处。");
        if self.retarget_undo_notice(&title, body, UndoNotice::Refiled(id)) {
            return true;
        }
        self.undo_notice = None;
        match crate::notice::Notice::show(
            &self.ctx,
            &title,
            body,
            pecofence_core::i18n::text("撤销"),
            crate::notice::locale(),
            &self.undo_notice_avoid_rects(),
        ) {
            Ok(notice) => {
                tracing::info!(moved, "undo notice shown for the rules applied");
                self.undo_notice = Some((notice, UndoNotice::Refiled(id)));
                true
            }
            Err(error) => {
                tracing::warn!(%error, "undo notice not shown");
                false
            }
        }
    }

    /// Fence `fence` was sent away from the current virtual desktop; its window showed on
    /// `before` until then.
    pub(super) fn note_desktops_for_undo(&mut self, fence: FenceId, before: Vec<String>) {
        self.push_undo(UndoStep::Desktops { fence, before });
    }

    /// After a fence was sent to another virtual desktop from its menu or Settings: 「“X”现在
    /// 只在“Y”上显示」 with 撤销, unless another notice is up (Ctrl+Z still undoes it then).
    /// Returns whether it is shown.
    pub(super) fn offer_undo_desktops(&mut self, fence: FenceId, title: &str, on: &str) -> bool {
        if self.notice_showing() {
            tracing::info!("undo notice not shown: another notice is up");
            return false;
        }
        self.undo_notice = None;
        let head = pecofence_core::i18n::format(
            "“{0}”现在只在“{1}”上显示",
            &[title.to_string(), on.to_string()],
        );
        match crate::notice::Notice::show(
            &self.ctx,
            &head,
            pecofence_core::i18n::text("切换到那个虚拟桌面就能看到它；点“撤销”让它回到这个桌面。"),
            pecofence_core::i18n::text("撤销"),
            crate::notice::locale(),
            &self.notice_avoid_rects(),
        ) {
            Ok(notice) => {
                tracing::info!(%fence, "undo notice shown for the fence sent to another desktop");
                self.undo_notice = Some((notice, UndoNotice::Desktops(fence)));
                true
            }
            Err(error) => {
                tracing::warn!(%error, "undo notice not shown");
                false
            }
        }
    }

    /// The undo notice was clicked, closed or timed out.
    pub(super) fn on_undo_notice(&mut self, target: UndoNotice, event: crate::notice::NoticeEvent) {
        tracing::info!(?target, ?event, "undo notice answered");
        if event == crate::notice::NoticeEvent::Action {
            self.undo_from_notice(target);
        }
    }
}
