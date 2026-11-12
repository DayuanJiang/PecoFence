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
    /// The label was fitted to the wider box (`ItemView::label_wide`).
    pub(super) label_wide: bool,
    pub(super) date: String,
    pub(super) type_: String,
    pub(super) size: String,
    pub(super) failed: bool,
    pub(super) alpha: Tween,
}

/// Layout motion runs only for a *rearrangement*: when at most half of the ids changed
/// (`changed` = removed + added) relative to the larger of the two lists, or when an empty list
/// fills up (its items fade in). A tab switch, portal navigation or a refill replaces most ids
/// and snaps instead.
pub(super) fn layout_motion_applies(changed: usize, old_len: usize, new_len: usize) -> bool {
    old_len == 0 || changed * 2 <= old_len.max(new_len)
}

/// A change that only takes icons away, or only brings icons in, is no refill either, however
/// many they are: 「立即应用」, its undo and a snapshot restore move dozens of icons between
/// fences at once, and the ones that go fade out as the ones that come fade in. (A refill
/// swaps ids: it removes and adds.)
pub(super) fn one_way_change(removed: usize, added: usize) -> bool {
    (removed == 0) != (added == 0)
}

/// How long after a drop its icons still count as "just let go here" (`drop_landing`): the
/// refresh the drop causes follows within a frame or two; anything later is another change.
pub(super) const DROP_LANDING_MAX: Duration = Duration::from_millis(500);

/// A cell origin as a map key (cells sit on whole DIPs).
pub(super) fn cell_key(x: f32, y: f32) -> (i32, i32) {
    (x.round() as i32, y.round() as i32)
}

/// Cell origin of an icon whose centre is at (`cx`, `cy`): where a dropped icon starts, under
/// its drag picture.
pub(super) fn landing_origin(cx: f32, cy: f32, cell: &CellRect) -> (f32, f32) {
    (cx - cell.w / 2.0, cy - cell.h / 2.0)
}

/// When a new icon's fade-in starts: at once, or 83 ms later when its cell is still being left
/// (`vacated`).
pub(super) fn fade_in_start(vacated: bool, now: Instant) -> Instant {
    if vacated { now + motion::FASTER } else { now }
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

/// The glides of an icon grid whose column count changed under a resize (`old` to `new`,
/// `count` items): for every icon in or near the viewport, before or after, its index, where
/// it is drawn now and the origin of its new cell, in unscrolled content DIPs. `drawn(i)` is
/// where a glide in flight has icon `i`; an icon at rest is in its old cell. `scroll` is the
/// offset before and after (fewer rows may clamp it): the start moves with it, so every icon
/// starts from its place on screen. One viewport of slack above and below, as the icon
/// loading takes; icons further away snap.
pub(super) fn reflow_glides(
    old: &Grid,
    new: &Grid,
    count: usize,
    drawn: impl Fn(usize) -> Option<(f32, f32)>,
    scroll: (f32, f32),
    view_h: f32,
) -> Vec<(usize, (f32, f32), (f32, f32))> {
    let near = |y: f32, h: f32| y + h > scroll.1 - view_h && y < scroll.1 + 2.0 * view_h;
    (0..count)
        .filter_map(|i| {
            let (o, c) = (old.cell(i), new.cell(i));
            let (fx, fy) = drawn(i).unwrap_or((o.x, o.y));
            let fy = fy - scroll.0 + scroll.1;
            (near(fy, c.h) || near(c.y, c.h)).then_some((i, (fx, fy), (c.x, c.y)))
        })
        .collect()
}

/// The user's own Delete (the key or the item menu) of these items is under way: when the
/// focused one goes with them, its neighbour takes the focus (`focus_after_delete`). The note
/// must not outlive that Delete, or an icon that leaves the fence another way later (dragged to
/// another fence, moved by a rule, removed by another program) would hand the selection to its
/// neighbour, and one more Delete would remove that one. So it is dropped when the app says the
/// delete was refused, cancelled or failed (`FenceWindow::settle_user_delete`), by the next
/// press or key in the fence, and by itself `USER_DELETE_WAIT` after it was made.
pub(super) struct UserDelete {
    pub(super) ids: HashSet<ItemId>,
    /// The Delete came from the keyboard: the neighbour that takes the focus shows the ring.
    pub(super) ring: bool,
    /// When the Delete was pressed, or when the app said its files are gone.
    pub(super) since: Instant,
}

impl UserDelete {
    pub(super) fn new(ids: HashSet<ItemId>, ring: bool) -> Self {
        Self {
            ids,
            ring,
            since: Instant::now(),
        }
    }

    pub(super) fn expired(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.since) > USER_DELETE_WAIT
    }
}

/// Where the keyboard focus goes when the user's own Delete took the focused item
/// (`old[anchor]`) away: the first item after it that stays, else the last one before it
/// (Explorer moves to the neighbour, so Delete, Delete, Delete walks down a list). `deleting`
/// are the items of that Delete, which may leave over several refreshes.
pub(super) fn focus_after_delete(
    old: &[ItemId],
    anchor: usize,
    new: &HashMap<ItemId, usize>,
    deleting: &HashSet<ItemId>,
) -> Option<usize> {
    let stays = |id: &ItemId| new.get(id).copied().filter(|_| !deleting.contains(id));
    let after = old.get(anchor + 1..).unwrap_or(&[]);
    let before = old.get(..anchor).unwrap_or(&[]);
    after
        .iter()
        .find_map(stays)
        .or_else(|| before.iter().rev().find_map(stays))
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
    /// where they were; the first items of an empty fence fade in. Icons the user just let go on
    /// this fence (`drop_landing`) glide in from where their drag picture was instead (167 ms
    /// decelerate), however many they are; a new item whose cell another one is still sliding
    /// out of starts its fade-in 83 ms later. Everything snaps when
    /// animations are off, the fence is rolled or rolling, hidden, a drag / marquee is running,
    /// a content swap brings in a whole page (tab switch, portal navigation), or more than half
    /// of the ids changed (a refill — not a rearrangement).
    pub(super) fn replace_items(&mut self, items: Vec<ItemView>) {
        let now = Instant::now();
        // Asked for by the filter box, for this replacement only.
        let fade_refill = std::mem::take(&mut self.fade_next_refill);
        let scale = self.scale();
        let (cw, _) = self.content_size_px();
        let width_dip = cw as f32 / scale;
        let animate = self.motion.enabled()
            && !self.rolled_up
            && self.roll_anim.is_none()
            && !self.ole_drag
            && self.marquee.is_none()
            && self.outgoing.is_empty()
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
        // 自由摆放: every icon stands in a cell of its own, but `desktop_slots` pairs with the
        // items by their place in the list, and holds the old list's cells until
        // `arrange_desktop` places the new one (after this, in `set_items`). With an icon gone
        // from the middle of the list, every icon after it took the cell of the one before it
        // as its new place: it glided there, and another file's icon stood in its cell, until
        // the cells were placed again. Each icon's own cell, by id, for the new list below.
        let own_slots: HashMap<ItemId, CellRect> =
            if self.places_freely() && !self.desktop_slots.is_empty() {
                self.items
                    .iter()
                    .zip(&self.desktop_slots)
                    .map(|(it, slot)| (it.id, *slot))
                    .collect()
            } else {
                HashMap::new()
            };
        let old_len = self.items.len();
        let old_ids: Vec<ItemId> = self.items.iter().map(|i| i.id).collect();
        // Keep icons / labels for unchanged items.
        let mut old: HashMap<ItemId, ItemView> = self.items.drain(..).map(|i| (i.id, i)).collect();
        let icons = self.icons.borrow();
        self.items = items
            .into_iter()
            .map(|mut n| {
                let Some(o) = old.remove(&n.id) else {
                    return n;
                };
                if o.icon_key == n.icon_key && o.name == n.name {
                    n.icon = o.icon;
                    n.icon_px = o.icon_px;
                    n.icon_fade = o.icon_fade;
                    n.icon_waited = o.icon_waited;
                    n.icon_stand_in = o.icon_stand_in;
                    n.label = o.label;
                    n.label_wide = o.label_wide;
                    n.icon_failed = o.icon_failed;
                    n.type_label = o.type_label;
                    if o.mtime == n.mtime {
                        n.date_label = o.date_label;
                    }
                    if o.size == n.size {
                        n.size_label = o.size_label;
                    }
                } else if o.icon.is_some() {
                    // The same item under a new name or icon key (a rename; a per-file key
                    // holds the path): its picture stays up, as in Explorer, until the lookup
                    // for the new key answers (`ensure_icons_and_labels`, no fade). It keeps
                    // the draw key it was uploaded under, and size 0 asks for the new one.
                    tracing::debug!(
                        item = ?n.id,
                        key_changed = o.icon_key != n.icon_key,
                        "item changed: its icon stays up until the new one arrives"
                    );
                    n.icon_stand_in = Some(o.draw_key(&icons));
                    n.icon = o.icon;
                    n.icon_px = 0;
                }
                n
            })
            .collect();
        drop(icons);
        if !own_slots.is_empty() && !self.items.iter().map(|n| n.id).eq(old_ids.iter().copied()) {
            // An icon that is new here has no cell yet: the spare one (where a single new icon
            // goes) until the cells are placed. One slot per item and none to spare, so the
            // placing that follows always differs from these and fits the window to it.
            let spare = self.desktop_slots.last().copied();
            let kept = self
                .items
                .iter()
                .filter(|n| own_slots.contains_key(&n.id))
                .count();
            self.desktop_slots = self
                .items
                .iter()
                .filter_map(|n| own_slots.get(&n.id).copied().or(spare))
                .collect();
            tracing::debug!(
                kept,
                new = self.items.len() - kept,
                gone = old_len - kept,
                "free placement: icons keep their own cells while the list changes"
            );
        }
        // `old` now holds only the removed items: their fades leave with them.
        if !old.is_empty() {
            self.item_hover.prune(now, |k| !old.contains_key(k));
            self.item_sel.prune(now, |k| !old.contains_key(k));
            self.drop_fades.prune(now, |k| !old.contains_key(k));
            self.search_dim.prune(now, |k| !old.contains_key(k));
        }
        self.remap_item_state(&old_ids);
        // Icons of our own let go on this fence a moment ago (`note_drop_landing`): the user
        // put them here, however many, so they never make the change a refill.
        let landing: HashMap<ItemId, (f32, f32)> = self
            .drop_landing
            .take()
            .filter(|(at, _)| animate && at.elapsed() < DROP_LANDING_MAX)
            .map(|(_, from)| from)
            .unwrap_or_default();
        let added = self
            .items
            .iter()
            .filter(|n| !old_pos.contains_key(&n.id) && !landing.contains_key(&n.id))
            .count();
        let changed = old.len() + added;
        // The filter box asks for its own cross-fade of a refill (`fade_refill`): its lists
        // keep the half rule alone. (`added` counts every item when nothing can animate.)
        let rearranged = layout_motion_applies(changed, old_len, self.items.len())
            || (animate && !fade_refill && one_way_change(old.len(), added));
        let animate = animate && rearranged;
        if !animate && changed > 0 && rearranged {
            // A genuine rearrangement that snapped instead of gliding: say why.
            tracing::info!(
                ole_drag = self.ole_drag,
                rolled = self.rolled_up,
                rolling = self.roll_anim.is_some(),
                marquee = self.marquee.is_some(),
                swapping = !self.outgoing.is_empty(),
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
            // A whole new list from the filter box: the old page fades out under the new one
            // (the tab switch's cross-fade without its slide) instead of a cut.
            if fade_refill
                && !layout_motion_applies(changed, old_len, self.items.len())
                && !self.ole_drag
                && self.outgoing.is_empty()
                && self.content_swap_shows()
            {
                tracing::debug!(
                    old_len,
                    new_len = self.items.len(),
                    changed,
                    "filter refill cross-fades"
                );
                self.begin_content_swap(0);
            }
            return;
        }
        let cells = self.cell_origins(width_dip);
        // Cells an icon at rest slides out of in this pass: a new icon there waits for it.
        let vacated: HashSet<(i32, i32)> = self
            .items
            .iter()
            .zip(&cells)
            .filter_map(|(n, c)| {
                let o = old_pos.get(&n.id)?;
                ((o.x - c.x).abs() > 0.5 || (o.y - c.y).abs() > 0.5).then(|| cell_key(o.x, o.y))
            })
            .collect();
        let mut motion: HashMap<ItemId, ItemMotion> = HashMap::new();
        let (mut landed, mut waited) = (0usize, 0usize);
        for (n, c) in self.items.iter().zip(cells) {
            // A motion still in flight continues from where it is, not from the old slot.
            let prev = self.item_motion.remove(&n.id);
            if let Some(&(lx, ly)) = landing.get(&n.id) {
                // Let go here by the user: it arrives from where its drag picture was, at
                // full strength (167 ms decelerate, WinUI's drop-to-rest).
                let (fx, fy) = landing_origin(lx, ly, &c);
                // The first few say where from and where to (a drop of hundreds stays short).
                if landed < DRAG_IMAGE_MAX_ITEMS {
                    tracing::debug!(
                        name = %n.name,
                        from_x = lx.round(),
                        from_y = ly.round(),
                        to_x = (c.x + c.w / 2.0).round(),
                        to_y = (c.y + c.h / 2.0).round(),
                        "dropped icon glides from where it was let go to its cell"
                    );
                }
                landed += 1;
                motion.insert(
                    n.id,
                    ItemMotion {
                        x: self
                            .motion
                            .tween(fx, c.x, motion::FAST, Curve::Decelerate, now),
                        y: self
                            .motion
                            .tween(fy, c.y, motion::FAST, Curve::Decelerate, now),
                        alpha: Tween::at(1.0, now),
                    },
                );
                continue;
            }
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
                    // Its cell is still being left by an icon sliding away: the fade-in starts
                    // 83 ms later, so the two names do not lie on top of each other (WinUI
                    // staggers the add after the reposition the same way).
                    let waits = vacated.contains(&cell_key(c.x, c.y));
                    let start = fade_in_start(waits, now);
                    waited += waits as usize;
                    motion.insert(
                        n.id,
                        ItemMotion {
                            x: Tween::at(c.x, now),
                            y: Tween::at(c.y, now),
                            alpha: self
                                .motion
                                .tween(0.0, 1.0, motion::FAST, Curve::Linear, start),
                        },
                    );
                }
            }
        }
        if waited > 0 || landed > 0 {
            tracing::debug!(
                landed,
                waited,
                "layout motion: icons arriving from a drop, new icons fading in 83 ms later"
            );
        }
        // Survivors took their motions out of the old map: what is left belongs to the
        // removed items (a glide or fade-in still in flight when the item vanished).
        let stale = std::mem::replace(&mut self.item_motion, motion);
        // Removed items fade out from where they are drawn — the mid-glide position and the
        // current alpha, not the layout cell (WinUI AddDeleteThemeTransition fades in place);
        // a removal mid fade-out just keeps fading.
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
                icon_key: o.draw_key(&icons),
                icon: o.icon,
                label_wide: o.label.is_some() && o.label_wide,
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
        if !self.leaving.is_empty() {
            tracing::debug!(
                leaving = self.leaving.len(),
                old_len,
                new_len = self.items.len(),
                "icons leaving fade out"
            );
        }
        if !self.item_motion.is_empty() || !self.leaving.is_empty() {
            self.frames.request();
        }
    }

    /// The content panel was laid out again (WM_SIZE outside a height animation); it was
    /// `old_cw_px` wide before. While the user drags an edge (`interactive`) and the icon grid
    /// gains or loses a column, every icon near the viewport glides from where it is drawn to
    /// its new cell (167 ms decelerate; one already gliding continues from its place), as a
    /// sort or a delete glides them. A width that keeps the columns moves the cells with the
    /// edge as before, and glides in flight ride along with their target cells. Everything
    /// else snaps: rows (one column at any width), 像桌面一样排列, animations off, a resize
    /// nobody drags.
    pub(super) fn reflow_items(&mut self, old_cw_px: i32, interactive: bool) {
        let (cw, _) = self.content_size_px();
        if cw == old_cw_px {
            // Only the height changed: every cell is where it was.
            return;
        }
        let animate = interactive
            && self.motion.enabled()
            && self.row_metrics().is_none()
            && !self.rolled_up
            && self.roll_anim.is_none()
            && !self.ole_drag
            && self.marquee.is_none()
            && desktop::is_visible(self.hwnd);
        if !animate {
            self.snap_item_motion();
            return;
        }
        let scale = self.scale();
        let (old, new) = (
            self.layout(old_cw_px as f32 / scale),
            self.layout(cw as f32 / scale),
        );
        let (ItemLayout::Grid(og), ItemLayout::Grid(ng)) = (&old, &new) else {
            self.snap_item_motion();
            return;
        };
        if og.has_slots() || ng.has_slots() {
            self.snap_item_motion();
            return;
        }
        self.leaving.clear();
        if og.columns == ng.columns {
            // Same cells, a little wider or narrower: a glide in flight ends where its cell
            // is now, with its timing kept.
            if !self.item_motion.is_empty() {
                for (i, item) in self.items.iter().enumerate() {
                    if let Some(m) = self.item_motion.get_mut(&item.id) {
                        let c = ng.cell(i);
                        m.x.shift(c.x - m.x.target());
                        m.y.shift(c.y - m.y.target());
                    }
                }
            }
            return;
        }
        let now = Instant::now();
        // Fewer rows may clamp the scroll offset (the redraw does it): the icons start from
        // the same place on screen.
        let view_h = self.scroll_view_h(&new);
        let old_scroll = self.scroll_y;
        let new_scroll = old_scroll.clamp(0.0, ng.max_scroll(view_h));
        let glides = reflow_glides(
            og,
            ng,
            self.items.len(),
            |i| {
                self.item_motion
                    .get(&self.items[i].id)
                    .map(|m| (m.x.value_at(now), m.y.value_at(now)))
            },
            (old_scroll, new_scroll),
            view_h,
        );
        let mut motion: HashMap<ItemId, ItemMotion> = HashMap::new();
        for (i, (fx, fy), (tx, ty)) in glides {
            let id = self.items[i].id;
            let alpha = self
                .item_motion
                .get(&id)
                .map_or(Tween::at(1.0, now), |m| m.alpha);
            if (fx - tx).abs() > 0.5 || (fy - ty).abs() > 0.5 || !alpha.is_done(now) {
                let glide = |from: f32, to: f32| {
                    self.motion
                        .tween(from, to, motion::FAST, Curve::Decelerate, now)
                };
                motion.insert(
                    id,
                    ItemMotion {
                        x: glide(fx, tx),
                        y: glide(fy, ty),
                        alpha,
                    },
                );
            }
        }
        tracing::debug!(
            from = og.columns,
            to = ng.columns,
            items = self.items.len(),
            gliding = motion.len(),
            in_flight = self.item_motion.len(),
            "resize changed the columns: icons glide"
        );
        self.item_motion = motion;
        if !self.item_motion.is_empty() {
            self.frames.request();
        }
    }

    /// Drops the note of the user's own Delete (see `UserDelete`); `why` goes to the log.
    pub(super) fn forget_user_delete(&mut self, why: &'static str) {
        if let Some(d) = self.user_delete.take() {
            tracing::debug!(items = d.ids.len(), why, "user delete: the note is dropped");
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
        let old_anchor = self.anchor_index;
        self.selected = remap_indices(old_ids, &new, &self.selected);
        self.anchor_index = self.anchor_index.and_then(map);
        self.range_anchor = self.range_anchor.and_then(map);
        if self.anchor_index.is_none() {
            self.focus_visible = false;
        }
        // The user's own Delete took the focused item away: its neighbour takes the focus and
        // the selection. The note stays while the focused item is one of them and still here
        // (the shell deletes them one by one, or still asks).
        if let Some(d) = self.user_delete.take() {
            let was_focused = old_anchor
                .and_then(|a| old_ids.get(a))
                .is_some_and(|id| d.ids.contains(id));
            if d.expired(Instant::now()) {
                tracing::debug!(
                    items = d.ids.len(),
                    was_focused,
                    "user delete: the note is too old, no focus moves"
                );
            } else if self.anchor_index.is_some() {
                if was_focused {
                    self.user_delete = Some(d);
                }
            } else if was_focused {
                let next = old_anchor.and_then(|a| focus_after_delete(old_ids, a, &new, &d.ids));
                tracing::debug!(
                    deleted = d.ids.len(),
                    ?old_anchor,
                    ?next,
                    ring = d.ring,
                    "the focused item was deleted by the user: the focus moves to its neighbour"
                );
                if let Some(i) = next {
                    self.select_only(i);
                    self.focus_visible = d.ring;
                }
            }
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
        // A Ctrl / Shift rectangle on the desktop beside 像桌面一样排列 keeps the icons selected
        // when it started: by identity too, or a file coming or going meanwhile selects
        // another icon.
        if let Some(base) = self.desktop_band_base.as_mut() {
            *base = remap_indices(old_ids, &new, base);
        }
    }
}
