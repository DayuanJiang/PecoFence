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

/// Opacity of the lock glyph: its hover fade `shown` (0..=1), or the pulse `since` its start
/// when the lock has just refused a drag: full strength dipping to `LOCK_PULSE_LOW` halfway
/// and back, whatever the hover fade is doing.
pub(super) fn lock_alpha(shown: f32, pulse: Option<Duration>) -> f32 {
    match pulse {
        Some(since) if since < LOCK_PULSE => {
            let t = since.as_secs_f32() / LOCK_PULSE.as_secs_f32();
            1.0 - (1.0 - LOCK_PULSE_LOW) * (std::f32::consts::PI * t).sin()
        }
        _ => shown,
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
    /// The lock glyph of a locked fence: shown while the pointer is over the fence.
    Lock,
    /// A tab pill under the pointer, by the tab's id: a reorder moves the fade with its pill.
    Tab(FenceId),
    /// A tab head with files dragged over it (drop-target look).
    TabDrop(FenceId),
    /// A tab that just joined the strip (merge, re-attach): 1 → 0 is its fade-in remainder,
    /// so the pill draws at alpha `1 - value` (Fluent Fade In, 83 ms linear).
    TabNew(FenceId),
}

/// The selected anchor's unfolded label: `text` is the full name fitted to `MAX_UNFOLD_LINES`
/// at `label_w`, `lines` how many lines it takes (measured once, when the cache fills).
pub(super) struct UnfoldCache {
    pub(super) id: ItemId,
    pub(super) label_w: f32,
    pub(super) name: String,
    pub(super) text: String,
    pub(super) lines: u32,
}

/// Where the hover and selection plates of a content `width_dip` wide end: before the
/// scrollbar's strip in a fence that scrolls, nowhere otherwise.
pub(super) fn plates_right(width_dip: f32, scrolls: bool) -> f32 {
    if scrolls {
        width_dip - pecofence_render::fence_chrome::SCROLL_STRIP
    } else {
        f32::INFINITY
    }
}

/// Room on each side of an icon's name inside its cell: the usual one, or the smaller one of
/// a name that was fitted to the wider box.
pub(super) fn label_inset(wide: bool) -> f32 {
    if wide {
        pecofence_render::fence_chrome::ICON_LABEL_SIDE_INSET_WIDE
    } else {
        pecofence_render::fence_chrome::ICON_LABEL_SIDE_INSET
    }
}

/// How much earlier the columns of the list and details views end in a fence that scrolls: a
/// row plate ends 4 DIPs before the edge, the strip is 8.
pub(super) fn rows_strip(scrolls: bool) -> f32 {
    if scrolls {
        pecofence_render::fence_chrome::SCROLL_STRIP - 4.0
    } else {
        0.0
    }
}

/// Applies (or removes) the DWM system backdrop for the chosen material.
pub(super) fn apply_system_backdrop(hwnd: HWND, mode: BackdropMode, dark: bool) {
    // DWM materials fall back to a solid colour on never-activated windows (verified on 26200),
    // so Acrylic is drawn by our own wallpaper pipeline; DWM is only told the colour mode.
    let _ = mode;
    let _ = dwm::set_system_backdrop(hwnd, dwm::SystemBackdrop::None);
    let _ = dwm::set_immersive_dark_mode(hwnd, dark);
}

/// The fence clips its window itself (`update_shape_clip`), so DWM must neither round it nor
/// draw its frame. DWM's small Fluent corner (4 DIP) left the 8 DIP rim a sliver of plate
/// outside its arc.
pub(super) fn apply_window_shape(hwnd: HWND) {
    let _ = dwm::set_nonclient_rendering(hwnd, false);
    let _ = dwm::set_corner_preference(hwnd, dwm::CornerPreference::DoNotRound);
}

/// Indices that hold every item whose cell crosses the band `top..bottom` (content DIPs, ends
/// excluded). Cells run down the content in list order, so the band's rows are found by
/// bisection, without a look at every item of a fence with thousands. Where the cells stand
/// anywhere (像桌面一样排列) every index is returned; callers test the cells they get.
pub(super) fn band_range(
    layout: &ItemLayout,
    count: usize,
    top: f32,
    bottom: f32,
) -> std::ops::Range<usize> {
    if matches!(layout, ItemLayout::Grid(g) if g.has_slots()) {
        return 0..count;
    }
    // The first index `above` no longer holds for (it holds for a run of leading indices).
    let first = |above: &dyn Fn(CellRect) -> bool| {
        let (mut lo, mut hi) = (0, count);
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if above(layout.cell(mid)) {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    };
    let start = first(&|c| c.y + c.h <= top);
    let end = first(&|c| c.y < bottom);
    start..end.max(start)
}

impl FenceViewState {
    /// Opacity multiplier the plate is drawn with. A fully transparent fence draws the
    /// default plate while it shows it (its alpha comes from `plate_reveal`).
    pub(super) fn material_opacity(&self) -> f32 {
        let opacity = if self.is_clear() { 1.0 } else { self.opacity };
        if self.behavior.floating.get() && self.theme.liquid_glass {
            opacity.max(1.0)
        } else {
            opacity
        }
    }

    /// Adapt content to the clear plate instead of darkening the whole wallpaper.
    pub(super) fn material_foreground(&self) -> Theme {
        if self.theme.liquid_glass && self.gpu_glass.borrow().failed() {
            return self.theme;
        }
        if self.bare() {
            // 像桌面一样排列: white captions with a halo, as Windows draws desktop icons.
            return self.theme.desktop_ink();
        }
        match &self.backdrop_crop {
            Some(image) if self.is_clear() => {
                // Captions sit on bare wallpaper at rest, on the default plate while hovered.
                let foreground =
                    self.theme
                        .over_clear_plate(image, self.style.tint, self.glass_dark_text.get());
                self.glass_dark_text
                    .set(Some(foreground.text_primary.r < 0.5));
                foreground
            }
            None if self.is_clear() => Theme {
                text_halo: true,
                ..self.theme
            },
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
            Some(image) => {
                // Fluent: the see-through plate is as bright as the wallpaper behind it,
                // whatever the colour mode, so the ink is picked from the plate as drawn.
                let opacity = self.material_opacity();
                let tint = self.plate_tint();
                let previous = self.glass_dark_text.get();
                let foreground = self.theme.over_acrylic(image, opacity, tint, previous);
                let dark_text = foreground.text_primary.r < 0.5;
                if previous != Some(dark_text) {
                    let (brightness, bright_share, dark_share) =
                        self.theme.acrylic_plate_stats(image, opacity, tint);
                    tracing::debug!(
                        fence = %self.title,
                        brightness,
                        bright_share,
                        dark_share,
                        opacity,
                        ?previous,
                        dark_text,
                        halo = foreground.text_halo,
                        "text colour picked from the plate"
                    );
                }
                self.glass_dark_text.set(Some(dark_text));
                foreground
            }
            None => self.theme,
        }
    }

    /// The tint washed over the whole plate (a tint for the title row alone leaves the plate
    /// under the item names as it is).
    fn plate_tint(&self) -> Option<ColorF> {
        self.style.tint.filter(|_| !self.style.tint_title_only)
    }

    /// The colour of the band the title and tabs sit on: the plate with the tint wash. None
    /// for a fully transparent fence, whose title sits on the wallpaper.
    fn title_band_color(&self) -> Option<ColorF> {
        if self.is_clear() {
            return None;
        }
        let tint = self.style.tint;
        Some(match &self.backdrop_crop {
            Some(image) if !image.bgra.is_empty() => {
                self.theme.plate_color(image, self.material_opacity(), tint)
            }
            _ => {
                let solid = self.theme.solid_fill;
                tint.map_or(solid, |t| pecofence_render::theme::lerp(solid, t, 0.22))
            }
        })
    }

    /// Drops the per-item fades: the view now shows a different list (tab switch, layout
    /// change), so nothing should cross-fade from the old one.
    pub(super) fn reset_item_fades(&mut self) {
        self.item_hover.clear();
        self.item_sel.clear();
        self.drop_fades.clear();
        self.search_dim.clear();
    }

    /// Syncs the per-item hover / selection / drop-target / search-dim fades from the logical
    /// state and samples them for the items drawn: one `(hover, selection, drop_target, item
    /// alpha)` per index of `drawn`, in its order. During a Peek search the matches show a
    /// lighter selection, the current match the full one, and the rest fade to
    /// `SEARCH_DIM_ALPHA`. Called at the top of every content draw, so outside a search it
    /// touches only the items that are on (hovered, selected, the drop target) and the fades
    /// still in the maps, never every item of a fence with thousands.
    pub(super) fn item_alphas(
        &mut self,
        now: Instant,
        drawn: impl Iterator<Item = usize>,
    ) -> Vec<(f32, f32, f32, f32)> {
        let animate = self.motion.enabled();
        let dragging = self.ole_drag;
        if let Some(m) = &self.search {
            for (i, it) in self.items.iter().enumerate() {
                let hit = m.matches.contains(&it.id);
                let hover = (self.hover == Some(i) && !dragging && !hit) as u8 as f32;
                let sel = if m.current == Some(it.id) {
                    1.0
                } else if hit {
                    SEARCH_MATCH_SELECTION
                } else {
                    0.0
                };
                self.item_hover.set(animate, it.id, hover, now);
                self.item_sel.set(animate, it.id, sel, now);
                let drop = (self.drop_item == Some(i)) as u8 as f32;
                self.drop_fades.set(animate, it.id, drop, now);
                self.search_dim
                    .set(animate, it.id, (!hit) as u8 as f32, now);
            }
        } else {
            let id_at = |i: usize| self.items.get(i).map(|it| it.id);
            let hover = self.hover.filter(|_| !dragging).and_then(id_at);
            if let Some(id) = hover {
                self.item_hover.set(animate, id, 1.0, now);
            }
            self.item_hover
                .release_unless(animate, now, |id| Some(*id) == hover);
            let selected: HashSet<ItemId> =
                self.selected.iter().filter_map(|&i| id_at(i)).collect();
            for id in &selected {
                self.item_sel.set(animate, *id, 1.0, now);
            }
            self.item_sel
                .release_unless(animate, now, |id| selected.contains(id));
            let drop = self.drop_item.and_then(id_at);
            if let Some(id) = drop {
                self.drop_fades.set(animate, id, 1.0, now);
            }
            self.drop_fades
                .release_unless(animate, now, |id| Some(*id) == drop);
            self.search_dim.release_unless(animate, now, |_| false);
        }
        let out = drawn
            .map(|i| {
                let id = &self.items[i].id;
                let dim = self.search_dim.value(id, now);
                (
                    self.item_hover.value(id, now),
                    self.item_sel.value(id, now),
                    self.drop_fades.value(id, now),
                    1.0 - dim * (1.0 - SEARCH_DIM_ALPHA),
                )
            })
            .collect();
        // Fades that settled at 0 go; ids that left the list were dropped in `replace_items`
        // (no per-draw id set: this runs every frame of a glide).
        self.item_hover.prune(now, |_| true);
        self.item_sel.prune(now, |_| true);
        self.drop_fades.prune(now, |_| true);
        self.search_dim.prune(now, |_| true);
        out
    }

    /// Indices of the items a content draw has to look at: the rows that cross the viewport,
    /// or every item while some glide to new cells (one may be drawn far from its cell).
    pub(super) fn rows_drawn(
        &self,
        layout: &ItemLayout,
        scroll: f32,
        view_h: f32,
    ) -> std::ops::Range<usize> {
        let count = self.items.len();
        if self.item_motion.is_empty() {
            band_range(layout, count, scroll - 1.0, scroll + view_h + 1.0)
        } else {
            0..count
        }
    }

    /// Drops finished layout / icon motion before a content draw (a finished tween sits at its
    /// end value, so nothing changes visually) and folds the caret away once it has faded out.
    pub(super) fn prune_content_motion(&mut self, now: Instant) {
        let sliding = !self.item_motion.is_empty();
        self.item_motion.retain(|_, m| !m.done(now));
        if sliding && self.item_motion.is_empty() {
            // The icons have come to rest: the one under a resting pointer may be another now
            // (the hover was last tested while they were still drawn at their old places).
            tracing::debug!("layout motion ended: hover refresh requested");
            self.request_hover_refresh();
        }
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
            || self.search_dim.busy(now)
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

    /// The item drawn with the keyboard focus ring: the Peek search's current match while a
    /// search runs, else the cursor item after keyboard navigation.
    pub(super) fn focus_index(&self) -> Option<usize> {
        match &self.search {
            Some(m) => m
                .current
                .and_then(|id| self.items.iter().position(|it| it.id == id)),
            None => self.focus_visible.then_some(self.anchor_index).flatten(),
        }
    }

    /// Selection colour blend for this draw: the activation cross-fade, pushed towards neutral
    /// grey (1) while a drag from elsewhere hovers the fence (`drop_inactive`, 83 ms) —
    /// Explorer's view is inactive then and only the drop target reads accent.
    pub(super) fn selection_inactive_now(&self, now: Instant) -> f32 {
        // The Peek search's current match reads accent although the search box has the focus.
        if self.search.is_some() {
            return 0.0;
        }
        let base = self.selection_inactive.value_at(now);
        let foreign = self.drop_inactive.value_at(now).clamp(0.0, 1.0);
        base + (1.0 - base) * foreign
    }

    /// Empty-state caption for the fence being shown.
    pub(super) fn empty_text(&self) -> &'static str {
        if self.bare() {
            // An empty desktop shows nothing.
            ""
        } else if self.is_inbox {
            pecofence_core::i18n::text("没有未归类的项目")
        } else if self.drop_hover {
            pecofence_core::i18n::text("松开即可放入")
        } else if let Some(text) = self.empty_override {
            text
        } else if self.is_portal && self.portal_missing {
            pecofence_core::i18n::text("找不到此文件夹。")
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
        // Title on hover folds the title row away (the plate starts at the content) and the
        // plate's top edge glides back up with the reveal; 全透明 fades the plate itself.
        let reveal = title_visible as u8 as f32;
        if (self.title_reveal.target() - reveal).abs() > f32::EPSILON {
            let dur = self.anim_dur(motion::FAST);
            self.title_reveal
                .retarget(reveal, dur, Curve::Decelerate, now);
        }
        let plate = self.plate_visible_target() as u8 as f32;
        if (self.plate_reveal.target() - plate).abs() > f32::EPSILON {
            let dur = self.anim_dur(motion::FAST);
            self.plate_reveal.retarget(plate, dur, Curve::Linear, now);
        }
        // 像桌面一样排列 has no title row: nothing above the plate, no title.
        let plate_top = if self.bare() {
            0.0
        } else {
            self.theme.title_height * (1.0 - self.title_reveal.value_at(now).clamp(0.0, 1.0))
        };
        let plate_clear = 1.0 - self.plate_reveal.value_at(now).clamp(0.0, 1.0);
        let shown = (!self.bare()) as u8 as f32;
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
        f.set(
            animate,
            ChromeKey::Lock,
            (self.locked && self.mouse_inside) as u8 as f32,
            now,
        );
        let pulse = self.lock_pulse.map(|t| now.saturating_duration_since(t));
        let lock = lock_alpha(f.value(&ChromeKey::Lock, now), pulse);
        if pulse.is_some_and(|since| since >= LOCK_PULSE) {
            self.lock_pulse = None;
        }
        let ntabs = if self.tabs.len() > 1 {
            self.tabs.len()
        } else {
            0
        };
        for i in 0..ntabs {
            let id = self.tabs[i].id;
            f.set(
                animate,
                ChromeKey::Tab(id),
                (self.tab_hover == Some(i)) as u8 as f32,
                now,
            );
            f.set(
                animate,
                ChromeKey::TabDrop(id),
                (self.drop_tab == Some(i)) as u8 as f32,
                now,
            );
        }
        let inside = self.press_inside;
        TitleState {
            title: f.value(&ChromeKey::Title, now) * shown,
            hover: f.value(&ChromeKey::Hover, now) * shown,
            up_hover: f.value(&ChromeKey::Up, now),
            up_pressed: self.pressed == Some(PressTarget::Up) && inside,
            chevron_hover: f.value(&ChromeKey::Chevron, now),
            chevron_pressed: self.pressed == Some(PressTarget::Chevron) && inside,
            count: f.value(&ChromeKey::Count, now),
            merge_hint: self.merge_t.value_at(now),
            plate_top,
            plate_clear,
            strip_clip: self.tab_strip_clip(),
            lock,
        }
    }

    /// The lock refused a drag (a title or edge drag of a locked fence): the lock glyph
    /// pulses once so the fence reads as locked, not as stuck. With animations off there is
    /// no pulse; the glyph itself still shows under the pointer.
    pub(super) fn pulse_lock(&mut self, why: &'static str) {
        let pulsing = self.lock_pulse.is_some();
        let animate = self.motion.enabled();
        tracing::debug!(
            fence = %self.fence_id,
            why,
            pulsing,
            animate,
            "the lock refused a drag: lock glyph pulse"
        );
        if pulsing || !animate {
            return;
        }
        self.lock_pulse = Some(Instant::now());
        let _ = self.redraw_chrome_only();
    }

    /// A press on a locked fence was remembered (`lock_press`) and the pointer is now at the
    /// screen point `pt`: past the drag threshold with the button still held, that is a drag
    /// the lock refuses.
    pub(super) fn note_lock_drag(&mut self, pt: (i32, i32)) {
        let Some((px, py)) = self.lock_press else {
            return;
        };
        if !window::key_down(msg::VK_LBUTTON) {
            self.lock_press = None;
            return;
        }
        let (tx, ty) = window::drag_threshold();
        if (pt.0 - px).abs() > tx || (pt.1 - py).abs() > ty {
            self.lock_press = None;
            self.pulse_lock("drag on a locked fence");
        }
    }

    /// Places the title-row fades at their rest values (first draw: nothing fades in).
    pub(super) fn snap_chrome_fades(&mut self) {
        let now = Instant::now();
        let title = self.title_visible_target() as u8 as f32;
        self.chrome_fades.snap(ChromeKey::Title, title, now);
        self.title_reveal = Tween::at(title, now);
        self.plate_reveal = Tween::at(self.plate_visible_target() as u8 as f32, now);
        self.chrome_fades
            .snap(ChromeKey::Count, self.roll_target() as u8 as f32, now);
    }

    /// Draws the chrome (backdrop, tabs or title, rim, chevron).
    pub(super) fn draw_chrome_panel(&mut self) -> Result<()> {
        let now = Instant::now();
        // The tab rects first: they bring the strip's scroll up to this size (the active tab in
        // view), and the strip's clip in `title_state` reads that scroll. In the other order a
        // narrowed fence kept the last frame's scroll and a fade at its right end.
        let rects = self.tab_rects();
        let state = self.title_state(now);
        // DIP size the crop covers (larger than the surface mid size animation).
        let (crop_w, crop_h) = self.backdrop_rect.map_or((0.0, 0.0), |(_, _, w, h)| {
            (w as f32 / self.scale(), h as f32 / self.scale())
        });
        let crop = self.backdrop_crop.as_ref().map(|img| BackdropCrop {
            image: img,
            key: &self.backdrop_key,
            width: crop_w,
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
        // A note (便签) has no items to count.
        // A smart fence shows its count after the name, rolled or not: no second one. A
        // filter tag carries the count too.
        let count = ((rolled || state.count > 0.0) && self.shows_rolled_count())
            .then(|| pecofence_core::i18n::format("{0} 项", &[format!("{}", self.items.len())]));
        let title_count = self.title_count.clone();
        let filter_tag = self.filter_tag.clone();
        let opacity = self.material_opacity();
        // Titles and tabs without a 「标题颜色」 of their own follow 「文字颜色」 (#141).
        let theme = self
            .material_foreground()
            .with_label_color(self.behavior.label_color.get());
        let scale = self.scale();
        let chrome = self.chrome.clone();
        let xs = self.tab_draw_xs(now);
        let active = self.active_index();
        // A pressed tab counts as dragged only past the drag threshold; a plain click keeps the
        // pressed caption and the SwitchTab pill glide.
        let dragging = self.tab_drag.as_ref().filter(|d| d.moved).map(|d| d.index);
        // A dragged tab carries the active fill itself: no pill glide underneath it.
        if dragging.is_some() {
            self.pill_anim = None;
        }
        // Active pill mid-glide: drawn at the tweened place instead of on the active tab. The
        // glide's end was the tab's rect when it started; once it is done the active tab draws
        // its own fill, so a strip that changed width meanwhile (Peek badges) is not left with
        // the pill at the old place.
        if self
            .pill_anim
            .is_some_and(|(x, w)| x.is_done(now) && w.is_done(now))
        {
            self.pill_anim = None;
        }
        let pill = self
            .pill_anim
            .map(|(x, w)| (x.value_at(now), w.value_at(now)));
        let deco = self.deco;
        // A title in the fence's own tint (标题颜色 → 跟随染色) sits on a band washed with
        // that tint: its lightness moves until it reads against the band.
        let (tint, band) = (self.style.tint, self.title_band_color());
        let light_text = theme.text_primary.r > 0.5;
        let on_band = move |color: Option<ColorF>| match (color, tint, band) {
            (Some(c), Some(tint), Some(band)) if c == tint => {
                Some(pecofence_render::theme::legible_on(c, band, light_text))
            }
            _ => color,
        };
        let style = FenceStyle {
            title_align: self.behavior.title_align.get(),
            tab_rail: self.rail_w(),
            hide_count_and_chevron: self.behavior.hide_count_and_chevron.get(),
            title_color: on_band(self.style.title_color),
            ..self.style
        };
        let pressed_tab = match self.pressed {
            Some(PressTarget::Tab(t)) if self.press_inside => Some(t),
            _ => None,
        };
        let badges: Vec<Option<String>> = self.tabs.iter().map(|t| self.tab_badge(t.id)).collect();
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
                title_color: on_band(t.title_color),
                folder_icon: t.folder_icon,
                up_button: i == active && t.up_button,
                desktop_icon: t.desktop_icon,
                active: i == active,
                hover: self.chrome_fades.value(&ChromeKey::Tab(t.id), now),
                pressed: pressed_tab == Some(i),
                color: t.color.map(|c| ColorF::from_rgba8(c[0], c[1], c[2], 0xFF)),
                drop_target: self.chrome_fades.value(&ChromeKey::TabDrop(t.id), now),
                dragging: dragging == Some(i),
                alpha: (1.0 - self.chrome_fades.value(&ChromeKey::TabNew(t.id), now))
                    * self.tab_search_alpha(t.id),
                badge: badges[i].as_deref(),
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
                title_count.as_deref(),
                filter_tag
                    .as_ref()
                    .map(|(k, t, c)| (k.as_str(), t.as_str(), c.as_str())),
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
        let inset = (state.plate_top * scale).round() as i32;
        self.sync_plate_shape(inset, 1.0 - state.plate_clear);
        self.chrome_fades.prune(now, |k| match k {
            ChromeKey::Tab(id) | ChromeKey::TabDrop(id) | ChromeKey::TabNew(id) => {
                self.tabs.iter().any(|t| t.id == *id)
            }
            _ => true,
        });
        self.tab_slide.retain(|(_, t)| !t.is_done(now));
        if self.tab_settle.is_some_and(|(_, t)| t.is_done(now)) {
            self.tab_settle = None;
        }
        self.chrome_busy = self.chrome_fades.busy(now)
            || !self.title_reveal.is_done(now)
            || !self.plate_reveal.is_done(now)
            || self.pill_anim.is_some()
            || !self.merge_t.is_done(now)
            || !self.tab_slide.is_empty()
            || self.tab_settle.is_some()
            || self.lock_pulse.is_some();
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
        let mut w = r.right - r.left;
        let mut h = r.bottom - r.top;
        // Liquid Glass's displacement field is rebuilt by the GPU renderer only on
        // size/DPI changes; position changes retain every uploaded image.
        if !self.theme.liquid_glass {
            if let Some(target) = self.height_target_px() {
                h = h.max(target);
            }
            // Collapsing to a left tab column (or expanding from it) moves the width too.
            if let Some(target) = self.width_target_px() {
                w = w.max(target);
            }
        }
        let shrinking = !self.theme.liquid_glass && self.height_animating();
        if self.backdrop_crop.is_some()
            && let Some((l, t, cw, ch)) = self.backdrop_rect
            && (l, t) == (r.left, r.top)
            && (cw == w || (shrinking && cw >= w))
            && (ch == h || (shrinking && ch >= h))
        {
            // Same rect, or a collapse / shrink whose crop (cut for the larger start) still
            // covers the visible part: the surface clips it.
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
        let origin = hit.map(|b| b.crop_origin(r.left, r.top));
        if !self.theme.liquid_glass
            && self.backdrop_crop.is_some()
            && origin.is_some()
            && origin == self.backdrop_origin
            && self
                .backdrop_rect
                .is_some_and(|(_, _, cw, ch)| (cw, ch) == (w, h))
        {
            // A move by less than one texel: the crop and its upload stay as they are.
            self.backdrop_rect = Some(want);
            return false;
        }
        self.backdrop_origin = origin;
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

    /// The window moved: re-crops the backdrop for where it is now and redraws what shows it.
    pub(super) fn refresh_backdrop(&mut self) {
        self.backdrop_stale = false;
        let old_ink = self.material_foreground().label_look();
        if self.update_backdrop() {
            if old_ink != self.material_foreground().label_look()
                || (self.theme.liquid_glass && self.row_metrics().is_some())
            {
                // Moving clear glass across a light/dark boundary also changes the icon labels.
                let _ = self.redraw();
            } else {
                let _ = self.redraw_chrome_only();
            }
        }
    }

    /// Forces a fresh crop (new wallpaper set, material or theme) even at the same rect.
    pub(super) fn invalidate_backdrop(&mut self) {
        self.backdrop_rect = None;
        self.update_backdrop();
    }

    /// Lays out the content panel below the title bar (and beside a left tab column) and
    /// resizes both surfaces.
    pub(super) fn layout_panels(&mut self, w: i32, h: i32) -> Result<()> {
        // A content swap still moving would be laid out at the wrong size: settle it first.
        self.finish_content_swap_now();
        // Entrance / exit scales pivot on the window centre (visual units are physical px).
        self.motion
            .set_center(&self.root, w as f32 / 2.0, h as f32 / 2.0);
        self.update_shape_clip(w, h)?;
        self.chrome_panel.resize(w, h)?;
        // A fence collapsed (or collapsing) to its left tab column keeps the width it returns
        // to; the column's width follows the expanded fence.
        if !self.column_collapsed && self.roll_anim.as_ref().is_none_or(|a| a.width.is_none()) {
            self.expanded_w_px = w;
        }
        self.rail_dip = self.measure_rail_w();
        self.content_x = self.rail_px();
        self.content_y = self.head_px();
        let content_h = (h - self.content_y).max(1);
        self.content_panel
            .resize((w - self.content_x).max(1), content_h)?;
        self.content_panel
            .visual
            .set_offset(self.content_x as f32, self.content_y as f32, 0.0);
        self.content_panel
            .visual
            .set_visible(!self.rolled_up || self.roll_anim.is_some());
        Ok(())
    }

    /// The clip must follow every height-animation frame even while content stays frozen. It
    /// starts at the plate top: a title row folded away by title on hover is cut off too.
    pub(super) fn update_shape_clip(&self, w: i32, h: i32) -> Result<()> {
        let inset = self.plate_inset_px.clamp(0, (h - 1).max(0));
        self.motion.clip_rounded_at(
            &self.root,
            inset as f32,
            w as f32,
            (h - inset) as f32,
            self.theme.corner_radius * self.scale(),
        )?;
        Ok(())
    }

    /// `label_w` is the width the label must fit: the cell width less the room beside the name
    /// for icons, the name column's text width for rows. `wide_w` is the wider box an icon's
    /// name may take when that shows more of it (`text::fit_lines_wider`), `label_w` again
    /// where there is none. Changing either drops every fitted label. Icons are asked for the
    /// items `near` the viewport only; the others get theirs when they scroll in (a crowded
    /// fence would otherwise extract every thumbnail at startup).
    pub(super) fn ensure_icons_and_labels(
        &mut self,
        label_w: f32,
        wide_w: f32,
        near: impl Fn(usize) -> bool,
    ) {
        let icon_px = (self.icon_dip() * self.scale()).round() as u32;
        if (self.label_w - label_w).abs() > 0.5 || (self.label_wide_w - wide_w).abs() > 0.5 {
            self.label_w = label_w;
            self.label_wide_w = wide_w;
            for item in &mut self.items {
                item.label = None;
            }
            self.unfold_cache = None;
        }
        let details = self.layout == ViewLayout::Details;
        // Explorer's list and details rows show folders without the preview of their contents.
        let rows = self.layout != ViewLayout::Icons;
        let now = Instant::now();
        let mut started = false;
        let mut pending = false;
        let mut icons = self.icons.borrow_mut();
        for (i, item) in self.items.iter_mut().enumerate() {
            if item.wants_icon(icon_px) && near(i) {
                let icon_only = item.icon_only || (rows && item.is_folder);
                match icons.get(&item.icon_key, &item.path, icon_px, self.dpi, icon_only) {
                    Lookup::Ready(Some(img)) => {
                        item.icon = Some(img);
                        item.icon_px = icon_px;
                        item.icon_stand_in = None;
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
                    Lookup::Ready(None) => {
                        item.icon_failed = true;
                        // No picture for the new key: the stand-in goes too (the glyph shows).
                        if item.icon_stand_in.take().is_some() {
                            item.icon = None;
                        }
                    }
                    // Another size's bitmap is the same picture: it stays up, and the sharper
                    // one replaces it without the fade.
                    Lookup::Pending => {
                        item.icon_waited = item.icon.is_none();
                        pending = true;
                    }
                }
            }
            if item.label.is_none() {
                let (label, wide) = match self.layout {
                    // No lines to fit into. The full name counts as not shortened, so the
                    // selected item never unfolds.
                    ViewLayout::Icons if self.label_lines == 0 => (item.name.clone(), false),
                    ViewLayout::Icons => pecofence_render::text::fit_lines_wider(
                        &item.name,
                        &self.chrome.label_format(),
                        label_w.max(1.0),
                        wide_w.max(1.0),
                        self.label_lines as u32,
                    ),
                    _ => (
                        pecofence_render::text::fit_width(
                            &item.name,
                            &self.chrome.row_format(),
                            label_w,
                        ),
                        false,
                    ),
                };
                item.label = Some(label);
                item.label_wide = wide;
            }
            if details {
                if item.date_label.is_none() {
                    item.date_label = Some(item.date_text());
                }
                if item.type_label.is_none() {
                    item.type_label =
                        Some(fileinfo::type_name(&item.path, item.is_folder).unwrap_or_default());
                }
                if item.size_label.is_none() {
                    item.size_label = Some(item.size_text());
                }
            }
        }
        drop(icons);
        self.icons_pending = pending;
        if started {
            self.frames.request();
        }
    }

    /// True when an item near the viewport is still waiting for its icon (as of the last
    /// content draw).
    pub fn has_pending_icons(&self) -> bool {
        self.icons_pending
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

    /// The columns of the rows at `width_dip`. A fence that scrolls keeps the strip along its
    /// right edge for the thumb (`SCROLL_STRIP`): the columns end that much earlier.
    pub(super) fn columns_for(&self, width_dip: f32) -> DetailColumns {
        self.columns_at(width_dip, self.scroll_max() > 0.0)
    }

    /// `columns_for` for a caller that already knows whether the rows scroll.
    pub(super) fn columns_at(&self, width_dip: f32, scrolls: bool) -> DetailColumns {
        // The columns end 8 DIPs before the edge anyway (4 inside the row plate): the plate
        // gives way to the strip, and they keep their 4 inside it.
        let width_dip = width_dip - rows_strip(scrolls);
        match self.layout {
            ViewLayout::Details => {
                DetailColumns::for_width_with(width_dip, self.column_widths, self.columns_visible)
            }
            _ => DetailColumns {
                name_x: DetailColumns::NAME_X,
                name_w: (width_dip - DetailColumns::NAME_X - DetailColumns::GUTTER).max(40.0),
                date: None,
                type_: None,
                size: None,
            },
        }
    }

    /// Fills `unfold_cache` for item `i` at `label_w` unless it already holds that entry: one
    /// DirectWrite layout pass per anchor / width / name change instead of one per redraw.
    pub(super) fn refresh_unfold_cache(&mut self, i: usize, label_w: f32) {
        let it = &self.items[i];
        let fresh = self
            .unfold_cache
            .as_ref()
            .is_some_and(|c| c.id == it.id && c.label_w == label_w && c.name == it.name);
        if fresh {
            return;
        }
        let (text, lines) = pecofence_render::text::fit_lines_counted(
            &it.name,
            &self.chrome.label_format(),
            label_w.max(1.0),
            MAX_UNFOLD_LINES,
        );
        self.unfold_cache = Some(UnfoldCache {
            id: it.id,
            label_w,
            name: it.name.clone(),
            text,
            lines,
        });
    }

    /// 脚本栅栏: the script's output lines and the footer in place of items.
    pub(super) fn redraw_script(&mut self) -> Result<()> {
        let Some(script) = self.script.clone() else {
            return Ok(());
        };
        use pecofence_render::fence_chrome::ScriptRowDraw;
        let scale = self.scale();
        let picked = self.material_foreground();
        // Colour names take the shade for the plate they are on: light text means a dark one.
        let dark = picked.text_primary.r > 0.5;
        let theme = picked.with_label_color(self.behavior.label_color.get());
        let rows: Vec<ScriptRowDraw<'_>> = script
            .rows
            .iter()
            .map(|r| ScriptRowDraw {
                text: &r.text,
                separator: r.separator,
                todo: r.todo,
                color: r
                    .color
                    .as_deref()
                    .and_then(|c| pecofence_core::script_output::color_rgb(c, dark))
                    .map(|[cr, cg, cb]| ColorF::from_rgba8(cr, cg, cb, 0xFF)),
                icon: r.icon.as_deref(),
                clickable: r.clickable,
                opens: r.opens,
            })
            .collect();
        let chrome = self.chrome.clone();
        let hint = if script.quiet {
            ""
        } else {
            pecofence_core::i18n::text("脚本没有输出文字")
        };
        let hover = self.script_hover;
        let text_size = self.text_size;
        let mut hits = Vec::new();
        let ok = self
            .content_panel
            .draw_ex(self.dpi, |session, _clip, wd, hd| {
                hits = chrome.draw_script(
                    session,
                    &theme,
                    scale,
                    wd,
                    hd,
                    text_size,
                    &rows,
                    hover,
                    hint,
                    &script.footer,
                )?;
                Ok(())
            })?;
        self.note_draw_result(ok);
        self.script_rows = hits
            .into_iter()
            .map(|(row, r)| (row, [r.left, r.top, r.right, r.bottom]))
            .collect();
        Ok(())
    }

    /// 便签与待办: the note's lines in place of items.
    pub(super) fn redraw_note(&mut self) -> Result<()> {
        let Some(text) = self.note.clone() else {
            return Ok(());
        };
        use pecofence_core::note::NoteLine;
        use pecofence_render::fence_chrome::NoteLineDraw;
        let lines: Vec<NoteLineDraw<'_>> = pecofence_core::note::lines(&text)
            .into_iter()
            .map(|l| match l {
                NoteLine::Text(text) => NoteLineDraw { text, todo: None },
                NoteLine::Todo { done, text } => NoteLineDraw {
                    text,
                    todo: Some(done),
                },
            })
            .collect();
        let scale = self.scale();
        let theme = self
            .material_foreground()
            .with_label_color(self.behavior.label_color.get());
        let chrome = self.chrome.clone();
        let hint =
            pecofence_core::i18n::text("单击这里写字。行首写 [ ] 的一行是待办，单击方框打勾。");
        let text_size = self.text_size;
        let mut boxes = Vec::new();
        let ok = self
            .content_panel
            .draw_ex(self.dpi, |session, _clip, wd, hd| {
                boxes = chrome.draw_note(
                    session, &theme, scale, wd, hd, text_size, &lines, hint, None,
                )?;
                Ok(())
            })?;
        self.note_draw_result(ok);
        self.note_boxes = boxes
            .into_iter()
            .map(|(line, r)| (line, [r.left, r.top, r.right, r.bottom]))
            .collect();
        Ok(())
    }

    /// 时钟与日期: the time and date in place of items.
    pub(super) fn redraw_clock(&mut self) -> Result<()> {
        let Some(seconds) = self.clock else {
            return Ok(());
        };
        let (time, date) = pecofence_platform::fileinfo::clock_texts(seconds);
        let theme = self
            .material_foreground()
            .with_label_color(self.behavior.label_color.get());
        let chrome = self.chrome.clone();
        let mut sizes = self.clock_text;
        let mut area = (0.0, 0.0);
        let ok = self
            .content_panel
            .draw_ex(self.dpi, |session, _clip, wd, hd| {
                area = (wd, hd);
                sizes = chrome.draw_clock(session, &theme, wd, hd, &time, &date)?;
                Ok(())
            })?;
        self.note_draw_result(ok);
        self.clock_text = sizes;
        // Once per size the user ends up with: not on every step of a resize drag (the end of
        // the drag draws the clock again).
        if sizes != self.clock_text_logged && self.behavior.size_move.borrow().is_none() {
            tracing::info!(
                fence = %self.active,
                area_w = area.0.round(),
                area_h = area.1.round(),
                time_dip = sizes.0,
                date_dip = sizes.1,
                from_time_dip = self.clock_text_logged.0,
                from_date_dip = self.clock_text_logged.1,
                "clock text size"
            );
            self.clock_text_logged = sizes;
        }
        Ok(())
    }

    pub(super) fn redraw_content(&mut self) -> Result<()> {
        if self.note.is_some() {
            return self.redraw_note();
        }
        if self.clock.is_some() {
            return self.redraw_clock();
        }
        if self.script.is_some() {
            return self.redraw_script();
        }
        if self.row_metrics().is_some() {
            return self.redraw_rows();
        }
        let (cw, _) = self.content_size_px();
        let scale = self.scale();
        let width_dip = cw as f32 / scale;
        let layout = self.layout(width_dip);
        let ItemLayout::Grid(grid) = &layout else {
            return Ok(());
        };
        let view_h = self.scroll_view_h(&layout);
        let max_scroll = grid.max_scroll(view_h);
        self.scroll_y = self.scroll_y.clamp(0.0, max_scroll);
        let scroll = self.scroll_y;
        // A fence that scrolls keeps the strip along its right edge for the thumb: no name
        // takes the wider box there and the plates end before the strip.
        let scrolls = max_scroll > 0.0;
        let label_w = grid.metrics.cell_w - 2.0 * label_inset(false);
        let wide_w = if scrolls {
            label_w
        } else {
            grid.metrics.cell_w - 2.0 * label_inset(true)
        };
        if scrolls != self.strip_kept {
            self.strip_kept = scrolls;
            tracing::debug!(
                fence = %self.fence_id,
                scrolls,
                max_scroll,
                label_w,
                wide_w,
                "scrollbar strip: names and plates {}",
                if scrolls { "end before it" } else { "use the whole cell" }
            );
        }
        // The rows in view plus one viewport above and below, and the same around where a
        // running glide ends (End, PageDown, a thumb drag, a Peek reveal).
        let target = self.scroll_target();
        let near = |y: f32, h: f32, at: f32| y + h > at - view_h && y < at + 2.0 * view_h;
        let count = self.items.len();
        let rows_near = |at: f32| band_range(&layout, count, at - view_h, at + 2.0 * view_h);
        let (here, there) = (rows_near(scroll), rows_near(target));
        self.ensure_icons_and_labels(label_w, wide_w, |i| {
            (here.contains(&i) || there.contains(&i)) && {
                let c = grid.cell(i);
                near(c.y, c.h, scroll) || near(c.y, c.h, target)
            }
        });

        let now = Instant::now();
        self.prune_content_motion(now);
        self.sync_scrollbar_alpha();
        // Only the cells in the viewport are prepared (`draw_content` skips the others anyway);
        // an item gliding in from outside counts where it is drawn.
        let placed: Vec<(usize, f32, f32, f32)> = self
            .rows_drawn(&layout, scroll, view_h)
            .filter_map(|i| {
                let item = &self.items[i];
                let c = grid.cell(i);
                let (x, y, alpha) = match self.item_motion.get(&item.id) {
                    Some(m) => (m.x.value_at(now), m.y.value_at(now), m.alpha.value_at(now)),
                    None => (c.x, c.y, 1.0),
                };
                let y = y - scroll;
                (y + c.h >= 0.0 && y <= view_h).then_some((i, x, y, alpha))
            })
            .collect();
        let alphas = self.item_alphas(now, placed.iter().map(|&(i, ..)| i));
        let dragging = self.ole_drag;
        let cut = self.cut.clone();
        let pressed = match self.pressed {
            Some(PressTarget::Item(i)) if self.press_inside => Some(i),
            _ => None,
        };
        let focused = self.focus_index();
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
            self.refresh_unfold_cache(i, label_w);
        }
        // 像桌面一样排列: a selected shortened name needs room below its cell in the shape.
        self.sync_desktop_shape(false);
        let full: Option<(&str, u32)> = shortened.and_then(|i| {
            let id = self.items[i].id;
            self.unfold_cache
                .as_ref()
                .filter(|c| c.id == id)
                .map(|c| (c.text.as_str(), c.lines))
        });
        let keys: Vec<String> = {
            let icons = self.icons.borrow();
            placed
                .iter()
                .map(|&(i, ..)| self.items[i].draw_key(&icons))
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
                label_inset: label_inset(l.label_wide),
            })
            .collect();
        let drawn = placed.iter().zip(&keys).zip(&alphas);
        cells.extend(drawn.map(|((&(i, x, y, alpha), key), fade)| {
            let item = &self.items[i];
            let c = grid.cell(i);
            ItemCell {
                x,
                y,
                w: c.w,
                h: c.h,
                icon_key: key,
                icon: item.icon.as_deref(),
                // The name under an open rename box is not drawn: the box holds it.
                label: if self.renaming == Some(item.id) {
                    ""
                } else {
                    item.label.as_deref().unwrap_or(&item.name)
                },
                hover: fade.0,
                selection: fade.1,
                pressed: pressed == Some(i) && !dragging,
                focused: focused == Some(i),
                dimmed: cut.contains(&item.id),
                failed: item.icon_failed,
                drop_target: fade.2,
                alpha: alpha * fade.3,
                icon_alpha: item.icon_fade.map_or(1.0, |t| t.value_at(now)),
                full_label: if unfold == Some(i) { full } else { None },
                label_inset: label_inset(item.label_wide),
            }
        }));
        let group_headers = group_header_draws(&layout, scroll, 0.0);
        let scrollbar = self.scrollbar_draw_in(&layout, view_h);
        let content = ContentDraw {
            items: &cells,
            group_headers: &group_headers,
            icon_size: self.icon_size as f32,
            label_lines: self.label_lines,
            line_h: grid.metrics.line_h,
            icon_top: grid.metrics.icon_top,
            label_gap: grid.metrics.label_gap,
            scrollbar,
            // `cells` holds the viewport only; the hint is for a fence without items.
            empty_text: self.items.is_empty().then(|| self.empty_text()),
            drop_highlight: self.drop_wash.value_at(now),
            marquee: self.marquee_rect_dip(),
            insert_caret: self.insert_caret_dip(0.0),
            insert_caret_alpha: self.caret_alpha.value_at(now),
            selection_inactive: self.selection_inactive_now(now),
            items_bottom: view_h,
            plates_right: plates_right(width_dip, scrolls),
        };
        let theme = self
            .material_foreground()
            .with_label_color(self.behavior.label_color.get());
        let chrome = self.chrome.clone();
        let bitmaps = self.bitmaps.clone();
        let ok = self
            .content_panel
            .draw_ex(self.dpi, |session, clip, wd, hd| {
                chrome.draw_content(
                    session,
                    Some(clip),
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
        let (cw, _) = self.content_size_px();
        let scale = self.scale();
        let width_dip = cw as f32 / scale;
        let layout = self.layout(width_dip);
        let view_h = self.scroll_view_h(&layout);
        let max_scroll = layout.max_scroll(view_h);
        self.scroll_y = self.scroll_y.clamp(0.0, max_scroll);
        let scrolls = max_scroll > 0.0;
        if scrolls != self.strip_kept {
            self.strip_kept = scrolls;
            tracing::debug!(
                fence = %self.fence_id,
                scrolls,
                max_scroll,
                "scrollbar strip: columns and plates {}",
                if scrolls { "end before it" } else { "use the whole width" }
            );
        }
        let cols = self.columns_at(width_dip, scrolls);
        let text_w = (cols.name_w - metrics.icon - 8.0 - 4.0).max(20.0);
        let scroll = self.scroll_y;
        // The rows in view plus one viewport above and below, and the same around where a
        // running glide ends (as in `redraw_content`).
        let target = self.scroll_target();
        let near = |y: f32, h: f32, at: f32| y + h > at - view_h && y < at + 2.0 * view_h;
        let count = self.items.len();
        let rows_near = |at: f32| band_range(&layout, count, at - view_h, at + 2.0 * view_h);
        let (here, there) = (rows_near(scroll), rows_near(target));
        self.ensure_icons_and_labels(text_w, text_w, |i| {
            (here.contains(&i) || there.contains(&i)) && {
                let c = layout.cell(i);
                near(c.y, c.h, scroll) || near(c.y, c.h, target)
            }
        });

        let now = Instant::now();
        self.prune_content_motion(now);
        self.sync_scrollbar_alpha();
        // Only the rows in the viewport are prepared (`draw_rows` skips the others anyway); a
        // row gliding in from outside counts where it is drawn.
        let placed: Vec<(usize, f32, f32)> = self
            .rows_drawn(&layout, scroll, view_h)
            .filter_map(|i| {
                let item = &self.items[i];
                let c = layout.cell(i);
                let (y, alpha) = match self.item_motion.get(&item.id) {
                    Some(m) => (m.y.value_at(now), m.alpha.value_at(now)),
                    None => (c.y, 1.0),
                };
                let y = y - scroll + metrics.header_h;
                (y + c.h > metrics.header_h && y <= metrics.header_h + view_h)
                    .then_some((i, y, alpha))
            })
            .collect();
        let alphas = self.item_alphas(now, placed.iter().map(|&(i, ..)| i));
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
        let focused = self.focus_index();
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
        let keys: Vec<String> = {
            let icons = self.icons.borrow();
            placed
                .iter()
                .map(|&(i, ..)| self.items[i].draw_key(&icons))
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
        let drawn = placed.iter().zip(&keys).zip(&alphas);
        rows.extend(drawn.map(|((&(i, y, alpha), key), fade)| {
            let item = &self.items[i];
            let c = layout.cell(i);
            RowCell {
                y,
                h: c.h,
                icon_key: key,
                icon: item.icon.as_deref(),
                name: item.label.as_deref().unwrap_or(&item.name),
                date: item.date_label.as_deref().unwrap_or(""),
                type_name: item.type_label.as_deref().unwrap_or(""),
                size: item.size_label.as_deref().unwrap_or(""),
                hover: fade.0,
                selection: fade.1,
                pressed: pressed_item == Some(i) && !dragging,
                focused: focused == Some(i),
                dimmed: cut.contains(&item.id),
                failed: item.icon_failed,
                drop_target: fade.2,
                alpha: alpha * fade.3,
                icon_alpha: item.icon_fade.map_or(1.0, |t| t.value_at(now)),
            }
        }));
        let group_headers = group_header_draws(&layout, scroll, metrics.header_h);
        let scrollbar = self.scrollbar_draw_in(&layout, view_h);
        // `rows` holds the viewport only; the hint is for a fence without items.
        let empty_text = self.items.is_empty().then(|| self.empty_text());
        let marquee = self.marquee_rect_dip().map(|m| Rect {
            left: m.left,
            top: m.top + metrics.header_h,
            right: m.right,
            bottom: m.bottom + metrics.header_h,
        });
        let draw = RowsDraw {
            rows: &rows,
            group_headers: &group_headers,
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
            origin_x: self.content_x as f32 / self.scale(),
            origin_y: self.content_y as f32 / self.scale(),
            rows_bottom: metrics.header_h + view_h,
            plates_right: plates_right(width_dip, scrolls),
        };
        let label_color = self.behavior.label_color.get();
        let theme = self.material_foreground().with_label_color(label_color);
        let chrome = self.chrome.clone();
        let bitmaps = self.bitmaps.clone();
        let wallpaper = self.backdrops.for_mode(self.backdrop_mode()).clone();
        // The glass is sampled under each text only to pick its ink: a fixed name colour skips it.
        let backdrop = if self.theme.liquid_glass
            && !wallpaper.is_empty()
            && label_color == pecofence_core::LabelColor::Auto
        {
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

/// Localised caption of a "按时间分组" section (literal keys keep `check-locales.py` honest).
pub(super) fn bucket_caption(bucket: DateBucket) -> &'static str {
    match bucket {
        DateBucket::Today => pecofence_core::i18n::text("今天"),
        DateBucket::Yesterday => pecofence_core::i18n::text("昨天"),
        DateBucket::ThisWeek => pecofence_core::i18n::text("本周"),
        DateBucket::ThisMonth => pecofence_core::i18n::text("本月"),
        DateBucket::Earlier => pecofence_core::i18n::text("更早"),
    }
}

/// Section headers of `layout` in surface DIPs: scrolled by `scroll` and, for rows, pushed
/// below the fixed column header by `header_h` (the same offset the row cells get). Headers
/// are static: item motion glides the items, the bands stay where the new layout puts them.
pub(super) fn group_header_draws(
    layout: &ItemLayout,
    scroll: f32,
    header_h: f32,
) -> Vec<GroupHeaderDraw<'static>> {
    layout
        .headers()
        .into_iter()
        .map(|h| GroupHeaderDraw {
            y: h.rect.y - scroll + header_h,
            h: h.rect.h,
            text: bucket_caption(h.bucket),
        })
        .collect()
}
