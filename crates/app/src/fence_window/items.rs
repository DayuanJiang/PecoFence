//! Item-list replacement with layout motion (reorder / add-delete transitions) and id-keyed state remapping.

use super::*;

/// Layout motion of one item (WinUI ReorderThemeTransition / AddDeleteThemeTransition): its
/// cell origin gliding from the old slot to the new one (250 ms point-to-point) and its alpha
/// (0 → 1 over 167 ms for an item that just appeared). Values are unscrolled content DIPs.
#[derive(Clone, Copy, Debug)]
pub(super) struct ItemMotion {
    pub(super) x: Tween,
    pub(super) y: Tween,
    pub(super) alpha: Tween,
}

impl ItemMotion {
    pub(super) fn done(&self, now: Instant) -> bool {
        self.x.is_done(now) && self.y.is_done(now) && self.alpha.is_done(now)
    }
}

/// An item that was removed, still drawn where it was while it fades out (83 ms linear).
pub(super) struct LeavingItem {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) w: f32,
    pub(super) h: f32,
    pub(super) icon_key: String,
    pub(super) icon: Option<Rc<Image>>,
    pub(super) label: String,
    pub(super) date: String,
    pub(super) type_: String,
    pub(super) size: String,
    pub(super) failed: bool,
    pub(super) alpha: Tween,
}

/// Layout motion runs only for a *rearrangement*: when at most half of the ids changed
/// (`changed` = removed + added) relative to the larger of the two lists. A tab switch, portal
/// navigation or a refill replaces most ids and snaps instead.
pub(super) fn layout_motion_applies(changed: usize, old_len: usize, new_len: usize) -> bool {
    old_len > 0 && changed * 2 <= old_len.max(new_len)
}

/// Index-keyed per-item state carried across a list refresh by item identity: `old` / `new`
/// are the ids before and after, `index` an index into `old`. Items that vanished map to None
/// (Explorer / ListView keep selection and focus by item, not by slot, across sorts, drops,
/// renames and folder-change refreshes).
pub(super) fn remap_index(
    old: &[ItemId],
    new: &HashMap<ItemId, usize>,
    index: usize,
) -> Option<usize> {
    old.get(index).and_then(|id| new.get(id).copied())
}

pub(super) fn remap_indices(
    old: &[ItemId],
    new: &HashMap<ItemId, usize>,
    set: &HashSet<usize>,
) -> HashSet<usize> {
    set.iter()
        .filter_map(|&i| remap_index(old, new, i))
        .collect()
}

impl FenceViewState {
    /// Cell origins of the current items in unscrolled content DIPs (the basis of the layout
    /// motion): `layout(width).cell(i)` for every index.
    pub(super) fn cell_origins(&self, width_dip: f32) -> Vec<CellRect> {
        let layout = self.layout(width_dip);
        (0..self.items.len()).map(|i| layout.cell(i)).collect()
    }

    /// Replaces the item list with layout motion (WinUI Reorder / AddDelete theme transitions):
    /// items present before and after glide from their old cell to the new one (250 ms
    /// point-to-point), new items fade in (167 ms) at their cell, removed items fade out (83 ms)
    /// where they were. Everything snaps when animations are off, the fence is rolled or
    /// rolling, hidden, a drag / marquee is running, the list was empty, or more than half of
    /// the ids changed (a tab switch, portal navigation or a refill — not a rearrangement).
    pub(super) fn replace_items(&mut self, items: Vec<ItemView>) {
        let now = Instant::now();
        let scale = self.scale();
        let (cw, _) = self.content_size_px();
        let width_dip = cw as f32 / scale;
        let animate = self.motion.enabled()
            && !self.rolled_up
            && self.roll_anim.is_none()
            && !self.ole_drag
            && self.marquee.is_none()
            && !self.items.is_empty()
            && desktop::is_visible(self.hwnd);
        let old_pos: HashMap<ItemId, CellRect> = if animate {
            let cells = self.cell_origins(width_dip);
            self.items
                .iter()
                .zip(cells)
                .map(|(it, c)| (it.id, c))
                .collect()
        } else {
            HashMap::new()
        };
        let old_len = self.items.len();
        let old_ids: Vec<ItemId> = self.items.iter().map(|i| i.id).collect();
        // Keep icons / labels for unchanged items.
        let mut old: HashMap<ItemId, ItemView> = self.items.drain(..).map(|i| (i.id, i)).collect();
        self.items = items
            .into_iter()
            .map(|mut n| {
                if let Some(o) = old.remove(&n.id)
                    && o.icon_key == n.icon_key
                    && o.name == n.name
                {
                    n.icon = o.icon;
                    n.icon_fade = o.icon_fade;
                    n.icon_waited = o.icon_waited;
                    n.label = o.label;
                    n.icon_failed = o.icon_failed;
                    n.type_label = o.type_label;
                    if o.mtime == n.mtime {
                        n.date_label = o.date_label;
                    }
                    if o.size == n.size {
                        n.size_label = o.size_label;
                    }
                }
                n
            })
            .collect();
        // `old` now holds only the removed items: their fades leave with them.
        if !old.is_empty() {
            self.item_hover.prune(now, |k| !old.contains_key(k));
            self.item_sel.prune(now, |k| !old.contains_key(k));
            self.drop_fades.prune(now, |k| !old.contains_key(k));
        }
        self.remap_item_state(&old_ids);
        let added = self
            .items
            .iter()
            .filter(|n| !old_pos.contains_key(&n.id))
            .count();
        let changed = old.len() + added;
        let animate = animate && layout_motion_applies(changed, old_len, self.items.len());
        if !animate && changed > 0 && layout_motion_applies(changed, old_len, self.items.len()) {
            // A genuine rearrangement that snapped instead of gliding: say why.
            tracing::info!(
                ole_drag = self.ole_drag,
                rolled = self.rolled_up,
                rolling = self.roll_anim.is_some(),
                marquee = self.marquee.is_some(),
                old_len,
                new_len = self.items.len(),
                changed,
                "layout motion skipped"
            );
        } else {
            tracing::debug!(
                animate,
                ole_drag = self.ole_drag,
                old_len,
                new_len = self.items.len(),
                changed,
                "replace_items"
            );
        }
        if !animate {
            self.snap_item_motion();
            return;
        }
        let cells = self.cell_origins(width_dip);
        let mut motion: HashMap<ItemId, ItemMotion> = HashMap::new();
        for (n, c) in self.items.iter().zip(cells) {
            // A motion still in flight continues from where it is, not from the old slot.
            let prev = self.item_motion.remove(&n.id);
            match old_pos.get(&n.id) {
                Some(o) => {
                    let (fx, fy) =
                        prev.map_or((o.x, o.y), |m| (m.x.value_at(now), m.y.value_at(now)));
                    let alpha = prev.map_or(Tween::at(1.0, now), |m| m.alpha);
                    if (fx - c.x).abs() > 0.5 || (fy - c.y).abs() > 0.5 || !alpha.is_done(now) {
                        motion.insert(
                            n.id,
                            ItemMotion {
                                x: self.motion.tween(
                                    fx,
                                    c.x,
                                    motion::NORMAL,
                                    Curve::PointToPoint,
                                    now,
                                ),
                                y: self.motion.tween(
                                    fy,
                                    c.y,
                                    motion::NORMAL,
                                    Curve::PointToPoint,
                                    now,
                                ),
                                alpha,
                            },
                        );
                    }
                }
                None => {
                    motion.insert(
                        n.id,
                        ItemMotion {
                            x: Tween::at(c.x, now),
                            y: Tween::at(c.y, now),
                            alpha: self
                                .motion
                                .tween(0.0, 1.0, motion::FAST, Curve::Linear, now),
                        },
                    );
                }
            }
        }
        // Survivors took their motions out of the old map: what is left belongs to the
        // removed items (a glide or fade-in still in flight when the item vanished).
        let stale = std::mem::replace(&mut self.item_motion, motion);
        // Removed items fade out from where they are drawn — the mid-glide position and the
        // current alpha, not the layout cell (WinUI AddDeleteThemeTransition fades in place);
        // a removal mid fade-out just keeps fading.
        let icon_px = (self.icon_dip() * scale).round() as u32;
        let icons = self.icons.borrow();
        for (id, o) in old {
            let Some(c) = old_pos.get(&id) else {
                continue;
            };
            let m = stale.get(&id);
            let (x, y) = m.map_or((c.x, c.y), |m| (m.x.value_at(now), m.y.value_at(now)));
            let a0 = m.map_or(1.0, |m| m.alpha.value_at(now)).clamp(0.0, 1.0);
            if a0 <= 0.01 {
                continue;
            }
            self.leaving.push(LeavingItem {
                x,
                y,
                w: c.w,
                h: c.h,
                icon_key: icons.draw_key(&o.icon_key, icon_px),
                icon: o.icon,
                label: o.label.unwrap_or(o.name),
                date: o.date_label.unwrap_or_default(),
                type_: o.type_label.unwrap_or_default(),
                size: o.size_label.unwrap_or_default(),
                failed: o.icon_failed,
                alpha: self
                    .motion
                    .tween(a0, 0.0, motion::FASTER, Curve::Linear, now),
            });
        }
        drop(icons);
        if !self.item_motion.is_empty() || !self.leaving.is_empty() {
            self.frames.request();
        }
    }

    /// Carries every index-keyed item state (selection, focus / range anchors, hover, drop
    /// target, press, drag source, marquee base) from the list `old_ids` describes to the
    /// current `self.items` by identity. Vanished items simply drop out; the focus ring hides
    /// when its item is gone. The fades are keyed by id and need nothing here.
    pub(super) fn remap_item_state(&mut self, old_ids: &[ItemId]) {
        let new: HashMap<ItemId, usize> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, it)| (it.id, i))
            .collect();
        let map = |i: usize| remap_index(old_ids, &new, i);
        self.selected = remap_indices(old_ids, &new, &self.selected);
        self.anchor_index = self.anchor_index.and_then(map);
        self.range_anchor = self.range_anchor.and_then(map);
        if self.anchor_index.is_none() {
            self.focus_visible = false;
        }
        self.hover = self.hover.and_then(map);
        self.drop_item = self.drop_item.and_then(map);
        if let Some(PressTarget::Item(i)) = self.pressed {
            self.pressed = map(i).map(PressTarget::Item);
            if self.pressed.is_none() {
                self.press_inside = false;
            }
        }
        if let Some(d) = self.drag.as_mut() {
            match map(d.item) {
                Some(i) => d.item = i,
                None => self.drag = None,
            }
        }
        if let Some(m) = self.marquee.as_mut() {
            m.base = remap_indices(old_ids, &new, &m.base);
        }
    }
}
