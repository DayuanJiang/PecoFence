//! Ctrl+Z in a fence, as on the desktop: undoes the last fence move or resize, icons moved
//! between fences, a fence deleted from its menu, or a 「立即应用」 of the sorting rules.
//! File operations (a deleted file, a
//! file moved into a folder) stay with Windows' own undo. A short stack in memory, gone at
//! restart and when the whole layout is replaced (import, backup, snapshot restore).

use super::*;
use crate::state::DeletedFence;

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
    /// Icons 「立即应用」 sorted: where each was, grouped by the fence it went to.
    Refiled {
        id: u64,
        places: Vec<(FenceId, Vec<(FenceId, pecofence_core::ItemRef)>)>,
    },
}

/// What the notice's 撤销 undoes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UndoNotice {
    /// The fence deleted from its menu.
    Deleted(FenceId),
    /// The 「立即应用」 step of this id.
    Refiled(u64),
}

/// Ids of the 「立即应用」 steps, so the notice undoes its own one.
static NEXT_REFILED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl UndoStep {
    fn kind(&self) -> &'static str {
        match self {
            Self::Bounds { .. } => "bounds",
            Self::Items { .. } => "items",
            Self::Deleted(_) => "deleted fence",
            Self::Refiled { .. } => "rules applied",
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

    /// Ctrl+Z: undoes the newest step.
    pub(super) fn undo_last(&mut self, from: &'static str) {
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
                {
                    self.undo_notice = None;
                }
                usize::from(back)
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
                {
                    self.undo_notice = None;
                }
                n
            }
        };
        tracing::info!(kind, from, done, left = self.undo.len(), "undone");
        if done > 0 {
            self.schedule_save();
        }
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

    /// After a delete from the fence menu: 「已删除栅栏“X”」 with 撤销, unless another notice
    /// is up (Ctrl+Z still undoes it then).
    pub(super) fn offer_undo_delete(&mut self, fence: FenceId, title: &str) {
        if self.notice_showing() {
            tracing::info!("undo notice not shown: another notice is up");
            return;
        }
        self.undo_notice = None;
        let title = pecofence_core::i18n::format("已删除栅栏“{0}”", &[title.to_string()]);
        match crate::notice::Notice::show(
            &self.ctx,
            &title,
            pecofence_core::i18n::text("点「撤销」，或在栅栏上按 Ctrl+Z，就能把它找回来。"),
            pecofence_core::i18n::text("撤销"),
            crate::notice::locale(),
            &self.notice_avoid_rects(),
        ) {
            Ok(notice) => {
                tracing::info!(%fence, "undo notice shown");
                self.undo_notice = Some((notice, UndoNotice::Deleted(fence)));
            }
            Err(error) => tracing::warn!(%error, "undo notice not shown"),
        }
    }

    /// After 「立即应用」 moved `moved` icons (step `id`): 「已按规则整理 N 个项目」 with 撤销, unless
    /// another notice is up (Ctrl+Z still undoes it then). Returns whether it is shown.
    pub(super) fn offer_undo_refiled(&mut self, id: u64, moved: usize) -> bool {
        if self.notice_showing() {
            tracing::info!("undo notice not shown: another notice is up");
            return false;
        }
        self.undo_notice = None;
        let title = pecofence_core::i18n::format("已按规则整理 {0} 个项目", &[moved.to_string()]);
        match crate::notice::Notice::show(
            &self.ctx,
            &title,
            pecofence_core::i18n::text("点「撤销」，或在栅栏上按 Ctrl+Z，就能把它们放回原处。"),
            pecofence_core::i18n::text("撤销"),
            crate::notice::locale(),
            &self.notice_avoid_rects(),
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

    /// The undo notice was clicked, closed or timed out.
    pub(super) fn on_undo_notice(&mut self, target: UndoNotice, event: crate::notice::NoticeEvent) {
        tracing::info!(?target, ?event, "undo notice answered");
        if event == crate::notice::NoticeEvent::Action {
            self.undo_from_notice(target);
        }
    }
}
