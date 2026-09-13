//! Everything that rasterises a surface: chrome panel, content grid, rows, backdrop crop, icon / label fitting and per-item fade sampling.

use super::*;

pub(super) fn header_column(c: DetailColumn) -> HeaderColumn {
    match c {
        DetailColumn::Name => HeaderColumn::Name,
        DetailColumn::Date => HeaderColumn::Date,
        DetailColumn::Type => HeaderColumn::Type,
        DetailColumn::Size => HeaderColumn::Size,
    }
}

pub(super) fn sort_column(sort: SortMode) -> Option<HeaderColumn> {
    match sort {
        SortMode::Name => Some(HeaderColumn::Name),
        SortMode::Date => Some(HeaderColumn::Date),
        SortMode::Type => Some(HeaderColumn::Type),
        SortMode::Size => Some(HeaderColumn::Size),
        _ => None,
    }
}

pub(super) fn column_sort(c: DetailColumn) -> SortMode {
    match c {
        DetailColumn::Name => SortMode::Name,
        DetailColumn::Date => SortMode::Date,
        DetailColumn::Type => SortMode::Type,
        DetailColumn::Size => SortMode::Size,
    }
}

/// Title-row things whose state alpha fades (keys of the chrome [`Fades`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum ChromeKey {
    /// Title-row reveal (title-on-hover).
    Title,
    /// Title row hovered: row pill + chevron reveal.
    Hover,
    Up,
    Chevron,
    /// Rolled item count.
    Count,
    Tab(usize),
    /// A tab head with files dragged over it (drop-target look).
    TabDrop(usize),
    /// A tab that just joined the strip (merge, re-attach): 1 → 0 is its fade-in remainder,
    /// so the pill draws at alpha `1 - value` (Fluent Fade In, 83 ms linear).
    TabNew(FenceId),
}

/// The selected anchor's unfolded label: `text` is the full name fitted to `MAX_UNFOLD_LINES`
/// at `cell_w`, `lines` how many lines it takes (measured once, when the cache fills).
pub(super) struct UnfoldCache {
    pub(super) id: ItemId,
    pub(super) cell_w: f32,
    pub(super) name: String,
    pub(super) text: String,
    pub(super) lines: u32,
}

/// Applies (or removes) the DWM system backdrop for the chosen material.
pub(super) fn apply_system_backdrop(hwnd: HWND, mode: BackdropMode, dark: bool) {
    // DWM materials fall back to a solid colour on never-activated windows (verified on 26200),
    // so Acrylic is drawn by our own wallpaper pipeline; DWM is only told the colour mode.
    let _ = mode;
    let _ = dwm::set_system_backdrop(hwnd, dwm::SystemBackdrop::None);
    let _ = dwm::set_immersive_dark_mode(hwnd, dark);
}

pub(super) fn apply_window_shape(hwnd: HWND, liquid_glass: bool) {
    let _ = dwm::set_nonclient_rendering(hwnd, !liquid_glass);
    let _ = dwm::set_corner_preference(
        hwnd,
        if liquid_glass {
            dwm::CornerPreference::DoNotRound
        } else {
            dwm::CornerPreference::RoundSmall
        },
    );
}

impl FenceViewState {
    pub(super) fn material_opacity(&self) -> f32 {
        if self.behavior.floating.get() && self.theme.liquid_glass {
            self.opacity.max(1.0)
        } else {
            self.opacity
        }
    }

    /// Adapt content to the clear plate instead of darkening the whole wallpaper.
    pub(super) fn material_foreground(&self) -> Theme {
        if self.theme.liquid_glass && self.gpu_glass.borrow().failed() {
            return self.theme;
        }
        match &self.backdrop_crop {
            Some(image) if self.theme.liquid_glass => {
                // The GPU renders a complete wallpaper-backed material, so floating
                // fences do not expose application text through a transparent centre.
                let theme = self.theme;
                let foreground = theme.over_glass_with_previous_ink(
                    image,
                    self.material_opacity(),
                    self.style.tint,
                    self.glass_dark_text.get(),
                );
                self.glass_dark_text
                    .set(Some(foreground.text_primary.r < 0.5));
                foreground
            }
            _ => self.theme,
        }
    }

    /// Drops the per-item fades: the view now shows a different list (tab switch, layout
    /// change), so nothing should cross-fade from the old one.
    pub(super) fn reset_item_fades(&mut self) {
        self.item_hover.clear();
        self.item_sel.clear();
        self.drop_fades.clear();
    }

    /// Syncs the per-item hover / selection / drop-target fades from the logical state and
    /// samples them: one `(hover, selection, drop_target)` alpha triple per item. Called at the
    /// top of every content draw.
    pub(super) fn item_alphas(&mut self, now: Instant) -> Vec<(f32, f32, f32)> {
        let animate = self.motion.enabled();
        let dragging = self.ole_drag;
        for (i, it) in self.items.iter().enumerate() {
            let hover = (self.hover == Some(i) && !dragging) as u8 as f32;
            self.item_hover.set(animate, it.id, hover, now);
            let sel = self.selected.contains(&i) as u8 as f32;
            self.item_sel.set(animate, it.id, sel, now);
            let drop = (self.drop_item == Some(i)) as u8 as f32;
            self.drop_fades.set(animate, it.id, drop, now);
        }
        let out = self
            .items
            .iter()
            .map(|it| {
                (
                    self.item_hover.value(&it.id, now),
                    self.item_sel.value(&it.id, now),
                    self.drop_fades.value(&it.id, now),
                )
            })
            .collect();
        // Fades that settled at 0 go; ids that left the list were dropped in `replace_items`
        // (no per-draw id set: this runs every frame of a glide).
        self.item_hover.prune(now, |_| true);
        self.item_sel.prune(now, |_| true);
        self.drop_fades.prune(now, |_| true);
        out
    }

    /// Drops finished layout / icon motion before a content draw (a finished tween sits at its
    /// end value, so nothing changes visually) and folds the caret away once it has faded out.
    pub(super) fn prune_content_motion(&mut self, now: Instant) {
        self.item_motion.retain(|_, m| !m.done(now));
        self.leaving.retain(|l| !l.alpha.is_done(now));
        for item in &mut self.items {
            if item.icon_fade.is_some_and(|t| t.is_done(now)) {
                item.icon_fade = None;
            }
        }
        if self.drop_insert.is_none() && self.caret_alpha.is_done(now) {
            self.caret_shown = None;
        }
    }

    /// Notes whether a content fade / motion is still running after a draw and keeps the frame
    /// clock ticking while it is.
    pub(super) fn note_content_fades(&mut self, now: Instant) {
        self.content_busy = self.item_hover.busy(now)
            || self.item_sel.busy(now)
            || self.header_fades.busy(now)
            || self.drop_fades.busy(now)
            || !self.selection_inactive.is_done(now)
            || !self.drop_inactive.is_done(now)
            || !self.drop_wash.is_done(now)
            || !self.caret_alpha.is_done(now)
            || self.item_motion.values().any(|m| !m.done(now))
            || self.leaving.iter().any(|l| !l.alpha.is_done(now))
            || self
                .items
                .iter()
                .any(|i| i.icon_fade.is_some_and(|t| !t.is_done(now)));
        if self.content_busy {
            self.frames.request();
        }
    }

    /// Selection colour blend for this draw: the activation cross-fade, pushed towards neutral
    /// grey (1) while a drag from elsewhere hovers the fence (`drop_inactive`, 83 ms) —
    /// Explorer's view is inactive then and only the drop target reads accent.
    pub(super) fn selection_inactive_now(&self, now: Instant) -> f32 {
        let base = self.selection_inactive.value_at(now);
        let foreign = self.drop_inactive.value_at(now).clamp(0.0, 1.0);
        base + (1.0 - base) * foreign
    }

    /// Empty-state caption for the fence being shown.
    pub(super) fn empty_text(&self) -> &'static str {
        if self.is_inbox {
            pecofence_core::i18n::text("没有未归类的项目")
        } else if self.drop_hover {
            pecofence_core::i18n::text("松开以放入")
        } else if self.is_portal {
            pecofence_core::i18n::text("此文件夹为空。")
        } else {
            pecofence_core::i18n::text("将项目拖到此处")
        }
    }

    /// Syncs the title-row fades from the logical state and samples them for this draw.
    pub(super) fn title_state(&mut self, now: Instant) -> TitleState {
        let animate = self.motion.enabled();
        let title_visible = self.title_visible_target();
        let rolled = self.roll_target();
        let f = &mut self.chrome_fades;
        f.set(animate, ChromeKey::Title, title_visible as u8 as f32, now);
        f.set(
            animate,
            ChromeKey::Hover,
            self.title_hover as u8 as f32,
            now,
        );
        f.set(animate, ChromeKey::Up, self.up_hovered as u8 as f32, now);
        f.set(
            animate,
            ChromeKey::Chevron,
            self.chevron_hovered as u8 as f32,
            now,
        );
        f.set(animate, ChromeKey::Count, rolled as u8 as f32, now);
        let ntabs = if self.tabs.len() > 1 {
            self.tabs.len()
        } else {
            0
        };
        for i in 0..ntabs {
            f.set(
                animate,
                ChromeKey::Tab(i),
                (self.tab_hover == Some(i)) as u8 as f32,
                now,
            );
            f.set(
                animate,
                ChromeKey::TabDrop(i),
                (self.drop_tab == Some(i)) as u8 as f32,
                now,
            );
        }
        let inside = self.press_inside;
        TitleState {
            title: f.value(&ChromeKey::Title, now),
            hover: f.value(&ChromeKey::Hover, now),
            up_hover: f.value(&ChromeKey::Up, now),
            up_pressed: self.pressed == Some(PressTarget::Up) && inside,
            chevron_hover: f.value(&ChromeKey::Chevron, now),
            chevron_pressed: self.pressed == Some(PressTarget::Chevron) && inside,
            count: f.value(&ChromeKey::Count, now),
            merge_hint: self.merge_t.value_at(now),
        }
    }

    /// Places the title-row fades at their rest values (first draw: nothing fades in).
    pub(super) fn snap_chrome_fades(&mut self) {
        let now = Instant::now();
        let title = self.title_visible_target() as u8 as f32;
        self.chrome_fades.snap(ChromeKey::Title, title, now);
        self.chrome_fades
            .snap(ChromeKey::Count, self.roll_target() as u8 as f32, now);
    }

    /// Draws the chrome (backdrop, tabs or title, rim, chevron).
    pub(super) fn draw_chrome_panel(&mut self) -> Result<()> {
        let now = Instant::now();
        let state = self.title_state(now);
        // DIP height the crop covers (taller than the surface mid height animation).
        let crop_h = self
            .backdrop_rect
            .map_or(0.0, |(_, _, _, h)| h as f32 / self.scale());
        let crop = self.backdrop_crop.as_ref().map(|img| BackdropCrop {
            image: img,
            key: &self.backdrop_key,
            height: crop_h,
        });
        let wallpaper = self.backdrops.for_mode(self.backdrop_mode()).clone();
        let backdrop = match (self.backdrop_mode(), crop) {
            (BackdropMode::Acrylic, _) if self.theme.liquid_glass && !wallpaper.is_empty() => {
                let (x, y, w, h) = self.backdrop_rect.unwrap_or_else(|| {
                    let r = window::window_rect(self.hwnd);
                    (r.left, r.top, r.right - r.left, r.bottom - r.top)
                });
                Backdrop::GpuGlass {
                    material: &self.gpu_glass,
                    wallpaper: &wallpaper,
                    rect: [x, y, w, h],
                    scale: self.scale(),
                }
            }
            (BackdropMode::Acrylic, Some(crop)) => Backdrop::Glass(crop, self.scale()),
            _ => Backdrop::Solid,
        };
        let bitmaps = self.bitmaps.clone();
        let title = self.title.clone();
        let rolled = self.roll_target();
        // A roll set / cancelled without the animation (reduced motion, retire, external
        // geometry) leaves the progress tween behind: rest it at the settled state.
        if self.roll_anim.is_none()
            && (self.roll_t.target() - self.rolled_up as u8 as f32).abs() > f32::EPSILON
        {
            self.roll_t = Tween::at(self.rolled_up as u8 as f32, now);
        }
        let roll_t = self.roll_t.value_at(now);
        // The count is drawn while it fades out after expanding, not only while rolled.
        let count = (rolled || state.count > 0.0)
            .then(|| pecofence_core::i18n::format("{0} 项", &[format!("{}", self.items.len())]));
        let opacity = self.material_opacity();
        let theme = self.material_foreground();
        let scale = self.scale();
        let chrome = self.chrome.clone();
        let rects = self.tab_rects();
        let xs = self.tab_draw_xs(now);
        let active = self.active_index();
        // A pressed tab counts as dragged only past the drag threshold; a plain click keeps the
        // pressed caption and the SwitchTab pill glide.
        let dragging = self.tab_drag.as_ref().filter(|d| d.moved).map(|d| d.index);
        // A dragged tab carries the active fill itself: no pill glide underneath it.
        if dragging.is_some() {
            self.pill_anim = None;
        }
        // Active pill mid-glide: drawn at the tweened place instead of on the active tab.
        let pill = self
            .pill_anim
            .map(|(x, w)| (x.value_at(now), w.value_at(now)));
        if self
            .pill_anim
            .is_some_and(|(x, w)| x.is_done(now) && w.is_done(now))
        {
            self.pill_anim = None;
        }
        let deco = self.deco;
        let style = self.style;
        let pressed_tab = match self.pressed {
            Some(PressTarget::Tab(t)) if self.press_inside => Some(t),
            _ => None,
        };
        let tabs: Vec<TabDraw<'_>> = self
            .tabs
            .iter()
            .zip(rects.iter())
            .enumerate()
            .map(|(i, (t, (x, w)))| TabDraw {
                key: t.id.as_u128(),
                x: xs.get(i).copied().unwrap_or(*x),
                w: *w,
                title: &t.title,
                title_size: t.title_size,
                title_color: t.title_color,
                active: i == active,
                hover: self.chrome_fades.value(&ChromeKey::Tab(i), now),
                pressed: pressed_tab == Some(i),
                color: t.color.map(|c| ColorF::from_rgba8(c[0], c[1], c[2], 0xFF)),
                drop_target: self.chrome_fades.value(&ChromeKey::TabDrop(i), now),
                dragging: dragging == Some(i),
                alpha: 1.0 - self.chrome_fades.value(&ChromeKey::TabNew(t.id), now),
            })
            .collect();
        let ok = self.chrome_panel.draw(self.dpi, |session, wd, hd| {
            chrome.draw(
                session,
                &mut bitmaps.borrow_mut(),
                &theme,
                scale,
                wd,
                hd,
                backdrop,
                &title,
                count.as_deref(),
                rolled,
                roll_t,
                opacity,
                &tabs,
                pill,
                deco,
                state,
                style,
            )
        })?;
        self.note_draw_result(ok);
        let ntabs = self.tabs.len();
        self.chrome_fades.prune(now, |k| match k {
            ChromeKey::Tab(i) | ChromeKey::TabDrop(i) => *i < ntabs,
            ChromeKey::TabNew(id) => self.tabs.iter().any(|t| t.id == *id),
            _ => true,
        });
        self.tab_slide.retain(|(_, t)| !t.is_done(now));
        if self.tab_settle.is_some_and(|(_, t)| t.is_done(now)) {
            self.tab_settle = None;
        }
        self.chrome_busy = self.chrome_fades.busy(now)
            || self.pill_anim.is_some()
            || !self.merge_t.is_done(now)
            || !self.tab_slide.is_empty()
            || self.tab_settle.is_some();
        if self.chrome_busy {
            self.frames.request();
        }
        Ok(())
    }

    /// Re-crops the wallpaper under the window when its rect changed. While the height
    /// animates the crop is cut once for the taller end of the tween in either direction —
    /// the taller of the current height, the tween's target and the crop already in hand —
    /// so the moving bottom edge only reveals or conceals finished pixels. Liquid Glass
    /// instead follows the curved bottom bezel each frame. Returns whether the crop changed.
    pub(super) fn update_backdrop(&mut self) -> bool {
        let r = window::window_rect(self.hwnd);
        self.update_backdrop_at(r)
    }

    /// Also accepts a pending pure move, so the GPU surface is prepared before Windows
    /// changes the HWND position. Only a tiny contrast sample is computed on the CPU.
    pub(super) fn update_backdrop_at(&mut self, r: RECT) -> bool {
        let w = r.right - r.left;
        let mut h = r.bottom - r.top;
        // Liquid Glass's displacement field is rebuilt by the GPU renderer only on
        // size/DPI changes; position changes retain every uploaded image.
        if !self.theme.liquid_glass
            && let Some(target) = self.height_target_px()
        {
            h = h.max(target);
        }
        if self.backdrop_crop.is_some()
            && let Some((l, t, cw, ch)) = self.backdrop_rect
            && (l, t, cw) == (r.left, r.top, w)
            && (ch == h || (!self.theme.liquid_glass && self.height_animating() && ch >= h))
        {
            // Same rect, or a collapse / shrink whose crop (cut for the taller start) still
            // covers the visible band: the surface clips it.
            return false;
        }
        let want = (r.left, r.top, w, h);
        let cx = r.left + w / 2;
        let cy = r.top + h.min(r.bottom - r.top) / 2;
        // The set for this fence's *effective* material, so an override crops the right tint.
        let set = self.backdrops.for_mode(self.backdrop_mode());
        let hit = set
            .iter()
            .find(|b| cx >= b.left && cx < b.left + b.width && cy >= b.top && cy < b.top + b.height)
            .or(set.first());
        self.backdrop_crop = if self.theme.liquid_glass && !set.is_empty() {
            Some(pecofence_render::liquid_glass::foreground_sample(
                set,
                [r.left, r.top, w, h],
            ))
        } else {
            hit.map(|b| b.crop_screen_rect(r.left, r.top, w, h))
        };
        self.backdrop_rect = Some(want);
        // The uploaded bitmap belongs to the old crop.
        if !self.theme.liquid_glass {
            self.bitmaps.borrow_mut().remove(&self.backdrop_key);
        }
        true
    }

    /// Forces a fresh crop (new wallpaper set, material or theme) even at the same rect.
    pub(super) fn invalidate_backdrop(&mut self) {
        self.backdrop_rect = None;
        self.update_backdrop();
    }

    /// Lays out the content panel below the title bar and resizes both surfaces.
    pub(super) fn layout_panels(&mut self, w: i32, h: i32) -> Result<()> {
        // A content swap still moving would be laid out at the wrong size: settle it first.
        self.finish_content_swap_now();
        // Entrance / exit scales pivot on the window centre (visual units are physical px).
        self.motion
            .set_center(&self.root, w as f32 / 2.0, h as f32 / 2.0);
        self.update_shape_clip(w, h)?;
        self.chrome_panel.resize(w, h)?;
        let title_h = self.title_h_px();
        let content_h = (h - title_h).max(1);
        self.content_panel.resize(w, content_h)?;
        self.content_panel
            .visual
            .set_offset(0.0, title_h as f32, 0.0);
        self.content_panel
            .visual
            .set_visible(!self.rolled_up || self.roll_anim.is_some());
        Ok(())
    }

    /// The clip must follow every height-animation frame even while content stays frozen.
    pub(super) fn update_shape_clip(&self, w: i32, h: i32) -> Result<()> {
        if self.theme.liquid_glass {
            self.motion.clip_rounded(
                &self.root,
                w as f32,
                h as f32,
                self.theme.corner_radius * self.scale(),
            )?;
        } else {
            self.motion.clear_clip(&self.root)?;
        }
        Ok(())
    }

    /// `label_w` is the width the label must fit: the cell width for icons, the name column's
    /// text width for rows. Changing it drops every fitted label.
    pub(super) fn ensure_icons_and_labels(&mut self, label_w: f32) {
        let icon_px = (self.icon_dip() * self.scale()).round() as u32;
        if (self.label_w - label_w).abs() > 0.5 {
            self.label_w = label_w;
            for item in &mut self.items {
                item.label = None;
            }
            self.unfold_cache = None;
        }
        let details = self.layout == ViewLayout::Details;
        let now = Instant::now();
        let mut started = false;
        let mut icons = self.icons.borrow_mut();
        for item in &mut self.items {
            if item.icon.is_none() && !item.icon_failed {
                match icons.get(&item.icon_key, &item.path, icon_px, item.icon_only) {
                    Lookup::Ready(Some(img)) => {
                        item.icon = Some(img);
                        if item.icon_waited {
                            // Delivered by the worker: cross-fade over the loading tile (167 ms
                            // linear); a synchronous cache hit shows at once.
                            item.icon_waited = false;
                            item.icon_fade =
                                Some(
                                    self.motion
                                        .tween(0.0, 1.0, motion::FAST, Curve::Linear, now),
                                );
                            started = true;
                        }
                    }
                    Lookup::Ready(None) => item.icon_failed = true,
                    Lookup::Pending => item.icon_waited = true,
                }
            }
            if item.label.is_none() {
                item.label = Some(match self.layout {
                    ViewLayout::Icons => pecofence_render::text::fit_lines(
                        &item.name,
                        &self.chrome.label_format(),
                        (label_w - pecofence_render::fence_chrome::ICON_LABEL_SIDE_INSET * 2.0)
                            .max(1.0),
                        self.label_lines as u32,
                    ),
                    _ => pecofence_render::text::fit_width(
                        &item.name,
                        self.chrome.row_format(),
                        label_w,
                    ),
                });
            }
            if details {
                if item.date_label.is_none() {
                    item.date_label = Some(fileinfo::format_local_datetime(item.mtime));
                }
                if item.type_label.is_none() {
                    item.type_label =
                        Some(fileinfo::type_name(&item.path, item.is_folder).unwrap_or_default());
                }
                if item.size_label.is_none() {
                    item.size_label = Some(fileinfo::format_size_kb(item.size, item.is_folder));
                }
            }
        }
        drop(icons);
        if started {
            self.frames.request();
        }
    }

    /// True when some item is still waiting for its icon.
    pub fn has_pending_icons(&self) -> bool {
        self.items
            .iter()
            .any(|i| i.icon.is_none() && !i.icon_failed)
    }

    /// Redraws the chrome surface only (backdrop/title); the content surface is untouched.
    pub fn redraw_chrome_only(&mut self) -> Result<()> {
        self.draw_chrome_panel()
    }

    /// `Ok(false)` from a surface draw means the GPU device is gone: ask the app to rebuild the
    /// stack once, then recreate our surfaces (`recreate_surfaces`).
    pub(super) fn note_draw_result(&mut self, ok: bool) {
        if !ok && !self.device_lost {
            self.device_lost = true;
            tracing::warn!(fence = %self.fence_id, "composition surface lost its device");
            self.queue.push(Command::DeviceLost);
        }
    }

    pub fn redraw(&mut self) -> Result<()> {
        self.draw_chrome_panel()?;
        if self.rolled_up && self.roll_anim.is_none() {
            return Ok(());
        }
        self.redraw_content()
    }

    pub(super) fn columns_for(&self, width_dip: f32) -> DetailColumns {
        match self.layout {
            ViewLayout::Details => {
                DetailColumns::for_width_with(width_dip, self.column_widths, self.columns_visible)
            }
            _ => DetailColumns {
                name_x: 8.0,
                name_w: (width_dip - 16.0).max(40.0),
                date: None,
                type_: None,
                size: None,
            },
        }
    }

    /// Fills `unfold_cache` for item `i` at `cell_w` unless it already holds that entry: one
    /// DirectWrite layout pass per anchor / width / name change instead of one per redraw.
    pub(super) fn refresh_unfold_cache(&mut self, i: usize, cell_w: f32) {
        let it = &self.items[i];
        let fresh = self
            .unfold_cache
            .as_ref()
            .is_some_and(|c| c.id == it.id && c.cell_w == cell_w && c.name == it.name);
        if fresh {
            return;
        }
        let (text, lines) = pecofence_render::text::fit_lines_counted(
            &it.name,
            &self.chrome.label_format(),
            (cell_w - pecofence_render::fence_chrome::ICON_LABEL_SIDE_INSET * 2.0).max(1.0),
            MAX_UNFOLD_LINES,
        );
        self.unfold_cache = Some(UnfoldCache {
            id: it.id,
            cell_w,
            name: it.name.clone(),
            text,
            lines,
        });
    }

    pub(super) fn redraw_content(&mut self) -> Result<()> {
        if self.row_metrics().is_some() {
            return self.redraw_rows();
        }
        let (cw, ch) = self.content_size_px();
        let scale = self.scale();
        let width_dip = cw as f32 / scale;
        let height_dip = ch as f32 / scale;
        let grid = Grid::new(self.grid_metrics(), width_dip, self.items.len());
        let max_scroll = grid.max_scroll(height_dip);
        self.scroll_y = self.scroll_y.clamp(0.0, max_scroll);
        self.ensure_icons_and_labels(grid.metrics.cell_w);

        let now = Instant::now();
        self.prune_content_motion(now);
        self.sync_scrollbar_alpha();
        let alphas = self.item_alphas(now);
        let scroll = self.scroll_y;
        let dragging = self.ole_drag;
        let cut = self.cut.clone();
        let pressed = match self.pressed {
            Some(PressTarget::Item(i)) if self.press_inside => Some(i),
            _ => None,
        };
        let focused = self.focus_visible.then_some(self.anchor_index).flatten();
        // The selected anchor item shows its full name (Windows desktop): only when the fitted
        // label had to be shortened, and never under an open rename edit.
        let unfold = self.anchor_index.filter(|&a| {
            a < self.items.len()
                && self.selected.contains(&a)
                && !dragging
                && self.renaming != Some(self.items[a].id)
        });
        let shortened = unfold.filter(|&i| {
            let it = &self.items[i];
            it.label.as_deref().unwrap_or(&it.name) != it.name
        });
        if let Some(i) = shortened {
            self.refresh_unfold_cache(i, grid.metrics.cell_w);
        }
        let full: Option<(&str, u32)> = shortened.and_then(|i| {
            let id = self.items[i].id;
            self.unfold_cache
                .as_ref()
                .filter(|c| c.id == id)
                .map(|c| (c.text.as_str(), c.lines))
        });
        let keys: Vec<String> = {
            let icons = self.icons.borrow();
            let px = (self.icon_size as f32 * scale).round() as u32;
            self.items
                .iter()
                .map(|i| icons.draw_key(&i.icon_key, px))
                .collect()
        };
        // Removed items fade out beneath everything else (painted first).
        let mut cells: Vec<ItemCell<'_>> = self
            .leaving
            .iter()
            .map(|l| ItemCell {
                x: l.x,
                y: l.y - scroll,
                w: l.w,
                h: l.h,
                icon_key: &l.icon_key,
                icon: l.icon.as_deref(),
                label: &l.label,
                hover: 0.0,
                selection: 0.0,
                pressed: false,
                focused: false,
                dimmed: false,
                failed: l.failed,
                drop_target: 0.0,
                alpha: l.alpha.value_at(now),
                icon_alpha: 1.0,
                full_label: None,
            })
            .collect();
        cells.extend(self.items.iter().enumerate().map(|(i, item)| {
            let c = grid.cell(i);
            let (x, y, alpha) = match self.item_motion.get(&item.id) {
                Some(m) => (m.x.value_at(now), m.y.value_at(now), m.alpha.value_at(now)),
                None => (c.x, c.y, 1.0),
            };
            ItemCell {
                x,
                y: y - scroll,
                w: c.w,
                h: c.h,
                icon_key: &keys[i],
                icon: item.icon.as_deref(),
                label: item.label.as_deref().unwrap_or(&item.name),
                hover: alphas[i].0,
                selection: alphas[i].1,
                pressed: pressed == Some(i) && !dragging,
                focused: focused == Some(i),
                dimmed: cut.contains(&item.id),
                failed: item.icon_failed,
                drop_target: alphas[i].2,
                alpha,
                icon_alpha: item.icon_fade.map_or(1.0, |t| t.value_at(now)),
                full_label: if unfold == Some(i) { full } else { None },
            }
        }));
        let scrollbar = self.scrollbar_draw();
        let content = ContentDraw {
            items: &cells,
            icon_size: self.icon_size as f32,
            label_lines: self.label_lines,
            line_h: grid.metrics.line_h,
            icon_top: grid.metrics.icon_top,
            label_gap: grid.metrics.label_gap,
            scrollbar,
            empty_text: Some(self.empty_text()),
            drop_highlight: self.drop_wash.value_at(now),
            marquee: self.marquee_rect_dip(),
            insert_caret: self.insert_caret_dip(0.0),
            insert_caret_alpha: self.caret_alpha.value_at(now),
            selection_inactive: self.selection_inactive_now(now),
        };
        let theme = self.material_foreground();
        let chrome = self.chrome.clone();
        let bitmaps = self.bitmaps.clone();
        let ok = self.content_panel.draw(self.dpi, |session, wd, hd| {
            chrome.draw_content(
                session,
                &theme,
                scale,
                &mut bitmaps.borrow_mut(),
                wd,
                hd,
                &content,
            )
        })?;
        self.note_draw_result(ok);
        self.note_content_fades(now);
        Ok(())
    }

    /// List / Details: rows below an optional fixed header.
    pub(super) fn redraw_rows(&mut self) -> Result<()> {
        let Some(metrics) = self.row_metrics() else {
            return Ok(());
        };
        let (cw, ch) = self.content_size_px();
        let scale = self.scale();
        let width_dip = cw as f32 / scale;
        let height_dip = ch as f32 / scale;
        let layout = ItemLayout::rows(metrics, width_dip, self.items.len());
        let view_h = (height_dip - metrics.header_h).max(1.0);
        let max_scroll = layout.max_scroll(view_h);
        self.scroll_y = self.scroll_y.clamp(0.0, max_scroll);
        let cols = self.columns_for(width_dip);
        let text_w = (cols.name_w - metrics.icon - 8.0 - 4.0).max(20.0);
        self.ensure_icons_and_labels(text_w);

        let now = Instant::now();
        self.prune_content_motion(now);
        self.sync_scrollbar_alpha();
        let alphas = self.item_alphas(now);
        let scroll = self.scroll_y;
        let dragging = self.ole_drag;
        let cut = self.cut.clone();
        let pressed_item = match self.pressed {
            Some(PressTarget::Item(i)) if self.press_inside => Some(i),
            _ => None,
        };
        let header_pressed = match self.pressed {
            Some(PressTarget::Header(c)) if self.press_inside => Some(header_column(c)),
            _ => None,
        };
        let focused = self.focus_visible.then_some(self.anchor_index).flatten();
        // Header cell hover fades (Details only).
        let header_alphas: Vec<(HeaderColumn, f32)> = if metrics.header_h > 0.0 {
            let animate = self.motion.enabled();
            let hovered = self.header_hover.map(header_column);
            let shown = [
                Some(HeaderColumn::Name),
                cols.date.map(|_| HeaderColumn::Date),
                cols.type_.map(|_| HeaderColumn::Type),
                cols.size.map(|_| HeaderColumn::Size),
            ];
            for c in shown.into_iter().flatten() {
                self.header_fades
                    .set(animate, c, (hovered == Some(c)) as u8 as f32, now);
            }
            let out = shown
                .into_iter()
                .flatten()
                .map(|c| (c, self.header_fades.value(&c, now)))
                .collect();
            self.header_fades.prune(now, |_| true);
            out
        } else {
            self.header_fades.clear();
            Vec::new()
        };
        let icon_px = (metrics.icon * scale).round() as u32;
        let keys: Vec<String> = {
            let icons = self.icons.borrow();
            self.items
                .iter()
                .map(|i| icons.draw_key(&i.icon_key, icon_px))
                .collect()
        };
        // Removed rows fade out beneath everything else (painted first).
        let mut rows: Vec<RowCell<'_>> = self
            .leaving
            .iter()
            .map(|l| RowCell {
                y: l.y - scroll + metrics.header_h,
                h: l.h,
                icon_key: &l.icon_key,
                icon: l.icon.as_deref(),
                name: &l.label,
                date: &l.date,
                type_name: &l.type_,
                size: &l.size,
                hover: 0.0,
                selection: 0.0,
                pressed: false,
                focused: false,
                dimmed: false,
                failed: l.failed,
                drop_target: 0.0,
                alpha: l.alpha.value_at(now),
                icon_alpha: 1.0,
            })
            .collect();
        rows.extend(self.items.iter().enumerate().map(|(i, item)| {
            let c = layout.cell(i);
            let (y, alpha) = match self.item_motion.get(&item.id) {
                Some(m) => (m.y.value_at(now), m.alpha.value_at(now)),
                None => (c.y, 1.0),
            };
            RowCell {
                y: y - scroll + metrics.header_h,
                h: c.h,
                icon_key: &keys[i],
                icon: item.icon.as_deref(),
                name: item.label.as_deref().unwrap_or(&item.name),
                date: item.date_label.as_deref().unwrap_or(""),
                type_name: item.type_label.as_deref().unwrap_or(""),
                size: item.size_label.as_deref().unwrap_or(""),
                hover: alphas[i].0,
                selection: alphas[i].1,
                pressed: pressed_item == Some(i) && !dragging,
                focused: focused == Some(i),
                dimmed: cut.contains(&item.id),
                failed: item.icon_failed,
                drop_target: alphas[i].2,
                alpha,
                icon_alpha: item.icon_fade.map_or(1.0, |t| t.value_at(now)),
            }
        }));
        let scrollbar = self.scrollbar_draw();
        let empty_text = Some(self.empty_text());
        let marquee = self.marquee_rect_dip().map(|m| Rect {
            left: m.left,
            top: m.top + metrics.header_h,
            right: m.right,
            bottom: m.bottom + metrics.header_h,
        });
        let draw = RowsDraw {
            rows: &rows,
            columns: RowColumns {
                name_x: cols.name_x,
                name_w: cols.name_w,
                date: cols.date,
                type_: cols.type_,
                size: cols.size,
            },
            icon_size: metrics.icon,
            header_h: metrics.header_h,
            sort_column: sort_column(self.sort),
            sort_descending: self.sort_reverse,
            header_hover: &header_alphas,
            header_pressed,
            scrollbar,
            empty_text,
            drop_highlight: self.drop_wash.value_at(now),
            marquee,
            insert_caret: self.insert_caret_dip(metrics.header_h),
            insert_caret_alpha: self.caret_alpha.value_at(now),
            selection_inactive: self.selection_inactive_now(now),
        };
        let theme = self.material_foreground();
        let chrome = self.chrome.clone();
        let bitmaps = self.bitmaps.clone();
        let wallpaper = self.backdrops.for_mode(self.backdrop_mode()).clone();
        let backdrop = if self.theme.liquid_glass && !wallpaper.is_empty() {
            let r = self.backdrop_rect.unwrap_or_else(|| {
                let r = window::window_rect(self.hwnd);
                (r.left, r.top, r.right - r.left, r.bottom - r.top)
            });
            Some(Backdrop::GpuGlass {
                material: &self.gpu_glass,
                wallpaper: &wallpaper,
                rect: [r.0, r.1, r.2, r.3],
                scale,
            })
        } else {
            None
        };
        let opacity = self.material_opacity();
        let ok = self
            .content_panel
            .draw_ex(self.dpi, |session, clip, wd, hd| {
                chrome.draw_rows(
                    session,
                    Some(clip),
                    &theme,
                    scale,
                    &mut bitmaps.borrow_mut(),
                    wd,
                    hd,
                    &draw,
                    backdrop.as_ref().map(|b| (b, opacity, self.style.tint)),
                )
            })?;
        self.note_draw_result(ok);
        self.note_content_fades(now);
        Ok(())
    }
}
