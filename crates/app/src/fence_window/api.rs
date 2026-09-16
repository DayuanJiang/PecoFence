//! `FenceWindow`: the HWND-owning handle and its public API (create, set_items, set_tabs, show / hide, …).

use super::*;

pub struct FenceWindow {
    #[allow(dead_code)]
    pub id: FenceId,
    pub(super) window: Window,
    pub(super) view: Rc<RefCell<Option<FenceViewState>>>,
    pub(super) anchor: AnchorCell,
    pub(super) _drop_target: Option<DropTargetRegistration>,
}

impl Drop for FenceWindow {
    fn drop(&mut self) {
        // RevokeDragDrop needs a live HWND; otherwise OLE keeps its AddRef'd IDropTarget (and
        // with it the view state and the shadow window) forever.
        self._drop_target.take();
        // The uploaded wallpaper crop is keyed by HWND, which Windows may hand to the next
        // window: drop it so a new fence never starts with this one's backdrop.
        if let Ok(guard) = self.view.try_borrow()
            && let Some(v) = guard.as_ref()
            && let Ok(mut bitmaps) = v.bitmaps.try_borrow_mut()
        {
            bitmaps.remove(&v.backdrop_key);
        }
    }
}

impl FenceWindow {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        ctx: &FenceContext,
        fence_id: FenceId,
        title: &str,
        is_inbox: bool,
        rolled_up: bool,
        icon_size: u32,
        label_lines: u8,
        auto_height: bool,
        bounds: RECT,
        expanded_h_px: i32,
        items: Vec<ItemView>,
    ) -> Result<Self> {
        let view: Rc<RefCell<Option<FenceViewState>>> = Rc::new(RefCell::new(None));
        let handler = Self::make_handler(ctx, fence_id, view.clone());
        let shadow = ShadowWindow::create(&ctx.shadow_class, ctx.shadow_style.get())?;

        let height = if rolled_up {
            (ctx.theme.borrow().title_height * (monitors::system_dpi().max(96) as f32 / 96.0))
                .round() as i32
        } else {
            bounds.bottom - bounds.top
        };
        // Desktop automation needs an activatable app window. Keep the production
        // transparency path: a GDI redirection bitmap would fill the clipped corners and
        // clear glass interior, hiding composition/desktop integration bugs in the audit.
        let extended_style = if cfg!(debug_assertions)
            && pecofence_core::brand::var_os("PECOFENCE_UI_TEST_WINDOWS").is_some()
        {
            style::EX_NOREDIRECTIONBITMAP
        } else {
            style::EX_NOACTIVATE | style::EX_TOOLWINDOW | style::EX_NOREDIRECTIONBITMAP
        };
        let window = WindowBuilder::new(&ctx.class)
            .title(title)
            .style(style::POPUP | style::THICKFRAME | style::CLIPCHILDREN | style::CLIPSIBLINGS)
            .ex_style(extended_style)
            .bounds(
                bounds.left,
                bounds.top,
                bounds.right - bounds.left,
                height.max(1),
            )
            .create(handler)?;
        let hwnd = window.hwnd();
        apply_window_shape(hwnd, ctx.theme.borrow().liquid_glass);
        let _ = dwm::set_border_color_none(hwnd);
        let _ = dwm::set_excluded_from_peek(hwnd, true);
        let _ = dwm::set_transitions_disabled(hwnd, true);
        apply_system_backdrop(
            hwnd,
            ctx.behavior.backdrop.get(),
            matches!(ctx.theme.borrow().mode, pecofence_render::ThemeMode::Dark),
        );
        // WM_NCCALCSIZE was not handled during CreateWindowEx. Remove its cached native
        // resize border BEFORE sizing the composition surfaces or showing the first frame.
        let initial_client = window.client_size();
        window.recalculate_frame()?;
        tracing::debug!(
            target: "pecofence::frame_shape",
            title, ?initial_client, client = ?window.client_size(),
            "initial nonclient layout corrected"
        );

        let target = ctx.stack.create_target(hwnd)?;
        let root = ctx.stack.compositor.create_container_visual();
        target.set_root(&root);
        let chrome_panel = Panel::new(&ctx.stack)?;
        let content_panel = Panel::new(&ctx.stack)?;
        root.children().insert_at_top(&chrome_panel.visual);
        root.children().insert_at_top(&content_panel.visual);

        let dpi = window.dpi().max(96);
        // Re-derive the rolled height with the window's real DPI.
        let theme = *ctx.theme.borrow();
        let mut state = FenceViewState {
            fence_id,
            title: title.to_string(),
            rolled_up,
            is_inbox,
            expanded_h_px: expanded_h_px.max(1),
            icon_size,
            label_lines,
            items,
            tabs: Vec::new(),
            active: fence_id,
            tab_hover: None,
            tab_drag: None,
            tab_slide: Vec::new(),
            merge_gap: None,
            tab_natural_w: RefCell::new(Vec::new()),
            tab_settle: None,
            drop_tab: None,
            drag_peek_armed: false,
            rbutton_swallow: false,
            renaming: None,
            title_renaming: false,
            unfold_cache: None,
            detach_pending: false,
            detach_cancelled: false,
            remote_drag: None,
            deco: TitleDeco::default(),
            up_hovered: false,
            chevron_hovered: false,
            pressed: None,
            press_inside: false,
            press_origin: (0, 0),
            focus_visible: false,
            win_active: false,
            selection_inactive: Tween::at(1.0, Instant::now()),
            drop_inactive: Tween::at(0.0, Instant::now()),
            item_hover: Fades::new(motion::FASTER, motion::FASTER, Curve::Linear),
            item_sel: Fades::new(motion::FASTER, motion::FASTER, Curve::Linear),
            header_fades: Fades::new(motion::FASTER, motion::FASTER, Curve::Linear),
            chrome_fades: Fades::new(motion::FASTER, motion::FASTER, Curve::Linear),
            drop_fades: Fades::new(motion::FASTER, motion::FASTER, Curve::Linear),
            drop_wash: Tween::at(0.0, Instant::now()),
            caret_alpha: Tween::at(0.0, Instant::now()),
            caret_shown: None,
            merge_hint: false,
            merge_t: Tween::at(0.0, Instant::now()),
            item_motion: HashMap::new(),
            leaving: Vec::new(),
            content_busy: false,
            chrome_busy: false,
            mouse_inside: false,
            last_scroll: None,
            click_pending: None,
            suppress_dblclk: false,
            navigate_folders: false,
            layout: ViewLayout::Icons,
            sort: SortMode::Manual,
            sort_reverse: false,
            group_by_date: false,
            header_hover: None,
            column_widths: DetailColumns::DEFAULT_WIDTHS,
            columns_visible: [true; 3],
            col_drag: None,
            is_portal: false,
            label_w: 0.0,
            hover: None,
            title_hover: false,
            selected: HashSet::new(),
            scroll_y: 0.0,
            scroll_memory: HashMap::new(),
            scroll_restore: None,
            scroll_anim: None,
            dpi,
            theme,
            glass_dark_text: Cell::new(None),
            backdrops: ctx.backdrops.borrow().clone(),
            backdrop_crop: None,
            backdrop_rect: None,
            backdrop_key: format!("backdrop:{:x}", hwnd.0 as isize),
            gpu_glass: RefCell::new(pecofence_render::gpu_glass::GpuGlass::default()),
            chrome_panel,
            content_panel,
            _target: target,
            root,
            chrome: ctx.chrome.clone(),
            icons: ctx.icons.clone(),
            bitmaps: ctx.bitmaps.clone(),
            queue: ctx.queue.clone(),
            drag: None,
            ole_drag: false,
            ole_drag_press: (0, 0),
            marquee: None,
            drop_item: None,
            drop_insert: None,
            scroll_drag: None,
            page_repeat: None,
            scrollbar_hot: false,
            sb_width: Tween::at(ScrollbarDraw::REST_WIDTH, Instant::now()),
            sb_alpha: Tween::at(
                if ctx.behavior.hide_inactive_scrollbar.get() {
                    0.0
                } else {
                    1.0
                },
                Instant::now(),
            ),
            sb_parts: Tween::at(0.0, Instant::now()),
            sb_arrow_fades: Fades::new(motion::FASTER, motion::FASTER, Curve::Linear),
            sb_pointer: None,
            sb_busy: false,
            drag_scroll: None,
            type_ahead: String::new(),
            type_ahead_at: None,
            pending_rename: None,
            tip: None,
            tip_target: None,
            tip_shown: false,
            tip_hidden_at: None,
            tip_anchor: (0, 0),
            tip_armed: false,
            cut: HashSet::new(),
            wheel_zoom_accum: 0,
            auto_height,
            locked: false,
            backdrop_override: None,
            opacity: 1.0,
            style: FenceStyle::default(),
            spacing: Spacing::Normal,
            anchor_index: None,
            range_anchor: None,
            device_lost: false,
            drop_hover: false,
            roll_anim: None,
            roll_t: Tween::at(rolled_up as u8 as f32, Instant::now()),
            height_anim: None,
            vis_fade: None,
            vis_gen: 0,
            vis_deadline: None,
            retired: false,
            outgoing: Vec::new(),
            spares: Vec::new(),
            swap_gen: 0,
            swap_deadline: None,
            pill_anim: None,
            stack: ctx.stack.clone(),
            motion: ctx.motion.clone(),
            frames: ctx.frames.clone(),
            hwnd,
            shadow,
            shadow_uploaded: None,
            shadow_alpha: Tween::at(1.0, Instant::now()),
            shadow_fading: false,
            peeking: false,
            behavior: ctx.behavior.clone(),
        };
        if rolled_up {
            let th = state.title_h_px();
            let _ = window.set_bounds(bounds.left, bounds.top, bounds.right - bounds.left, th);
        }
        state.update_backdrop();
        state.snap_chrome_fades();
        let (w, h) = window.client_size();
        state.layout_panels(w, h)?;
        state.redraw()?;
        state.update_shadow();
        let shadow_hwnd = state.shadow.hwnd();
        *view.borrow_mut() = Some(state);

        let drop_target = DropTargetRegistration::register(
            hwnd,
            Box::new(FenceDropHandler {
                queue: ctx.queue.clone(),
                view: view.clone(),
                kind: DropKind::None,
                behavior: ctx.behavior.clone(),
                helper: dragdrop::DropTargetHelper::new(),
                data: None,
                last_description: None,
                right_button: false,
                over_stats: Default::default(),
            }),
        )
        .map_err(|e| tracing::warn!(error = %e, "RegisterDragDrop failed"))
        .ok();

        if let Some(a) = ctx.anchor.borrow_mut().as_mut() {
            a.register_fence(hwnd, Some(shadow_hwnd));
        }
        Ok(Self {
            id: fence_id,
            window,
            view,
            anchor: ctx.anchor.clone(),
            _drop_target: drop_target,
        })
    }

    pub fn hwnd(&self) -> HWND {
        self.window.hwnd()
    }

    /// Shows the window with a 167 ms fade; `entrance` (a fence that did not exist a moment
    /// ago: new, torn off, new portal) adds the Fluent 0.97 → 1 scale. No-op when visible.
    pub fn show(&self, entrance: bool) {
        show_with_fade(&self.view, self.hwnd(), entrance);
    }

    /// Fades the window out (167 ms, shrinking to 0.97) ahead of its destruction. Returns
    /// true when the fade runs — the owner must keep the `FenceWindow` alive until
    /// `Command::FadeOutDone` arrives for its hwnd; false = already hidden, drop it now.
    /// The window leaves the desktop anchor at once so quick hide / show, Peek and re-anchoring
    /// no longer touch it, and nothing may show it again (`retired`).
    pub fn retire(&self) -> bool {
        let hwnd = self.hwnd();
        if let Some(v) = self.view.borrow_mut().as_mut() {
            v.retired = true;
            // A dying window is not ticked any more: freeze motion in flight and stop the
            // hover-peek timers instead of leaving them to fire into a fading window.
            v.roll_anim = None;
            v.height_anim = None;
            window::kill_timer(hwnd, TIMER_PEEK_OPEN);
            window::kill_timer(hwnd, TIMER_PEEK_CLOSE);
        }
        if let Ok(mut guard) = self.anchor.try_borrow_mut()
            && let Some(a) = guard.as_mut()
        {
            a.unregister_fence(hwnd);
        }
        hide_with_fade(&self.view, hwnd, true, true)
    }

    pub fn rect(&self) -> RECT {
        self.window.window_rect()
    }

    /// One-line state summary for `--test-script dump`: what the compositor shows (root
    /// opacity / scale), the whole-window fade phase, roll state and the shadow's alpha.
    pub fn debug_state(&self) -> String {
        let guard = self.view.borrow();
        let Some(v) = guard.as_ref() else {
            return "view=None".to_string();
        };
        let fade = match v.vis_fade {
            None => "none".to_string(),
            Some(VisFade::Showing) => "showing".to_string(),
            Some(VisFade::Hiding { destroy }) => format!("hiding(destroy={destroy})"),
        };
        let r = self.window.window_rect();
        let origin = window::screen_to_client(
            self.hwnd(),
            pecofence_platform::POINT {
                x: r.left,
                y: r.top,
            },
        );
        format!(
            "title={:?} opacity={:.2} scale={:.3} fade={fade} retired={} rolled={} rolling={} peeking={} items={} tabs={} selected={} focused={:?} shadow_visible={} shadow_alpha={} layout={:?} locked={} auto_height={} active={} scroll={:.1} client={:?} client_offset=({},{}) chrome={:?} glass={:?} wallpaper_uploads={} title_size={} tab_fonts={:?} tab_rects={:?} capture={} tab_drag={} window_drag={} detach_pending={}",
            v.title,
            v.root.opacity(),
            v.root.scale().x,
            v.retired,
            v.rolled_up,
            v.roll_anim.is_some(),
            v.peeking,
            v.items.len(),
            v.tabs.len(),
            v.selected.len(),
            v.anchor_index
                .and_then(|i| v.items.get(i))
                .map(|it| &it.name),
            desktop::is_visible(v.shadow.hwnd()),
            v.shadow.alpha(),
            v.layout,
            v.locked,
            v.auto_height,
            v.active,
            v.scroll_y,
            self.window.client_size(),
            -origin.x,
            -origin.y,
            v.chrome_panel.size_px(),
            v.gpu_glass.borrow().stats(),
            v.bitmaps.borrow().glass_wallpaper_uploads(),
            v.style.title_size,
            v.tabs
                .iter()
                .map(|t| (&t.title, t.title_size))
                .collect::<Vec<_>>(),
            v.tab_rects(),
            window::get_capture() == self.hwnd(),
            v.tab_drag.is_some(),
            v.remote_drag.is_some(),
            v.detach_pending,
        )
    }

    /// Opt-in native regression input; never compiled into a release build.
    /// Pointer samples exercise the production coalescer without moving the user's cursor.
    #[cfg(debug_assertions)]
    pub fn test_input(&self, action: &str, x: i32, y: i32) {
        if pecofence_core::brand::var_os("PECOFENCE_UI_TEST_WINDOWS").is_none() {
            return;
        }
        let hwnd = self.hwnd();
        match action {
            "pointer" => self.with_view(|v| {
                if let Some(drag) = v.remote_drag.as_mut() {
                    drag.track_pointer((x, y), window::drag_threshold());
                    if drag.pending_pointer.is_some() {
                        v.frames.request();
                    }
                }
            }),
            "capture-lost" => window::release_capture(),
            _ => {
                let (message, key, point) = match action {
                    "down" => (msg::WM_LBUTTONDOWN, 1, (x, y)),
                    "move" => (msg::WM_MOUSEMOVE, 1, (x, y)),
                    "up" => (msg::WM_LBUTTONUP, 0, (x, y)),
                    "caption-down" => {
                        let p = window::client_to_screen(hwnd, pecofence_platform::POINT { x, y });
                        (msg::WM_NCLBUTTONDOWN, msg::HTCAPTION as usize, (p.x, p.y))
                    }
                    "pointer-up" => {
                        let p = window::screen_to_client(hwnd, pecofence_platform::POINT { x, y });
                        (msg::WM_LBUTTONUP, 0, (p.x, p.y))
                    }
                    "escape" => (msg::WM_KEYDOWN, msg::VK_ESCAPE as usize, (0, 0)),
                    "right" => (msg::WM_RBUTTONDOWN, 0, (x, y)),
                    _ => return,
                };
                window::send_message(hwnd, message, key, msg::make_lparam(point.0, point.1));
            }
        }
    }

    /// Explicit geometry from the app (layout, snapping, monitor changes) takes precedence over
    /// a running auto-height settle.
    pub fn set_bounds(&self, r: RECT) {
        self.with_view(|v| v.height_anim = None);
        let _ = self
            .window
            .set_bounds(r.left, r.top, r.right - r.left, r.bottom - r.top);
    }

    /// A tab split/undo supplies complete geometry. Settle any hover expansion first
    /// so its old tween cannot pull the detached or restored window back on later frames.
    pub fn restore_geometry(&self, mut rect: RECT, rolled: bool, expanded_h: i32) {
        self.with_view(|v| {
            v.finish_content_swap_now();
            v.roll_anim = None;
            v.height_anim = None;
            v.peeking = false;
            v.rolled_up = rolled;
            v.expanded_h_px = expanded_h;
            v.roll_t = Tween::at(rolled as u8 as f32, Instant::now());
            let _ = v.motion.stop(&v.content_panel.visual, Prop::Opacity);
            v.content_panel.visual.set_opacity(1.0);
            v.content_panel.visual.set_visible(!rolled);
            rect.bottom = rect.top + if rolled { v.title_h_px() } else { expanded_h };
        });
        self.set_bounds(rect);
        self.with_view(|v| {
            let _ = v
                .layout_panels(rect.right - rect.left, rect.bottom - rect.top)
                .and_then(|_| v.redraw());
            v.update_shadow_now();
        });
    }

    pub(super) fn with_view(&self, f: impl FnOnce(&mut FenceViewState)) {
        if let Some(v) = self.view.borrow_mut().as_mut() {
            f(v);
        }
    }

    /// Replaces the item list with layout motion (see `FenceViewState::replace_items`).
    pub fn set_items(&self, items: Vec<ItemView>) {
        self.with_view(|v| {
            // Selection / focus / hover ride along by item id (`remap_item_state`), so a sort,
            // a reorder drop, a rename or a watcher refresh keeps them on the same items while
            // the layout glide moves those items to their new cells.
            v.replace_items(items);
            v.drop_insert = None;
            // The caret's slot index means nothing in the new list.
            v.caret_shown = None;
            v.caret_alpha = Tween::at(0.0, Instant::now());
            if let Some(PressTarget::Item(_)) = v.pressed {
                v.pressed = None;
                v.press_inside = false;
            }
            v.type_ahead.clear();
            v.cancel_pending_rename();
            v.hide_tip();
            if let Some(s) = v.scroll_restore {
                // Tab switch: the new tab's items are here, back to where that tab was
                // (the redraw clamps to its content; `set_layout` consumes the value).
                v.scroll_y = s;
            }
            let _ = v.redraw();
            // The item under a stationary pointer may have changed: re-hit-test from the loop.
            v.request_hover_refresh();
        });
    }

    /// A portal rename changes its path-derived id. Re-key the old view before replacing
    /// its items so selection, keyboard focus and layout motion follow the renamed file.
    pub fn rekey_item(&self, old: ItemId, new: ItemId) {
        if old == new {
            return;
        }
        self.with_view(|v| {
            if let Some(item) = v.items.iter_mut().find(|it| it.id == old) {
                item.id = new;
                if let Some(motion) = v.item_motion.remove(&old) {
                    v.item_motion.insert(new, motion);
                }
                let now = Instant::now();
                v.item_hover.prune(now, |id| *id != old);
                v.item_sel.prune(now, |id| *id != old);
                v.drop_fades.prune(now, |id| *id != old);
                if v.renaming == Some(old) {
                    v.renaming = Some(new);
                }
            }
        });
    }

    /// Scrolls item `item` fully into view at once (no glide) and redraws, so the caller can
    /// place a popup on its final label rect (LVM_EDITLABEL ensures visibility before the
    /// edit opens).
    pub fn ensure_item_visible_now(&self, item: ItemId) {
        self.with_view(|v| {
            let Some(i) = v.items.iter().position(|it| it.id == item) else {
                return;
            };
            if let Some(t) = v.scroll_anim.take() {
                v.scroll_y = t.target().clamp(0.0, v.scroll_max());
            }
            let before = v.scroll_y;
            v.scroll_into_view(i, false);
            if v.scroll_y != before && !v.rolled_up {
                let _ = v.redraw_content();
            }
        });
    }

    /// Items on the clipboard via 剪切: drawn at half alpha until the clipboard changes.
    pub fn set_cut_items(&self, cut: &HashSet<ItemId>) {
        self.with_view(|v| {
            if &v.cut != cut {
                v.cut = cut.clone();
                let _ = v.redraw_content();
            }
        });
    }

    /// Details columns shown (修改日期, 类型, 大小) for the fence being shown.
    pub fn set_columns_visible(&self, visible: [bool; 3]) {
        self.with_view(|v| {
            if v.columns_visible != visible {
                v.columns_visible = visible;
                if v.layout == ViewLayout::Details {
                    let _ = v.redraw();
                }
            }
        });
    }

    /// Item whose inline rename edit is open (None = closed): its label stays folded under
    /// the edit box instead of unfolding to the full name.
    pub fn set_renaming(&self, item: Option<ItemId>) {
        self.with_view(|v| {
            if v.renaming != item {
                v.renaming = item;
                if let Some(item) = item
                    && let Some(index) = v.items.iter().position(|it| it.id == item)
                {
                    // New items enter rename without a preceding click. Keep keyboard
                    // actions on the edited item after the popup closes.
                    v.select_only(index);
                }
                if !v.rolled_up {
                    let _ = v.redraw_content();
                }
                if item.is_none() {
                    v.resume_peek_close();
                }
            }
        });
    }

    /// The fence-title rename popup opened / closed. While it is open a hover peek stays
    /// expanded (the popup is part of the fence); closing it restarts the close countdown.
    pub fn set_title_renaming(&self, on: bool) {
        self.with_view(|v| {
            if v.title_renaming != on {
                v.title_renaming = on;
                if !on {
                    v.resume_peek_close();
                }
            }
        });
    }

    pub fn set_title(&self, title: &str) {
        self.with_view(|v| {
            v.title = title.to_string();
            let _ = v.redraw();
        });
    }

    pub fn set_icon_size(&self, size: u32) {
        self.with_view(|v| {
            v.icon_size = size;
            v.drop_icons_for_reload();
            v.snap_item_motion();
            let _ = v.redraw();
        });
    }

    /// The system icon-title font changed (font or Accessibility text size): fitted labels
    /// are stale and the cell grid has a new height, so refit and relayout at once (a DPI-like
    /// snap, no transition). Icons stay cached.
    pub fn on_icon_font_changed(&self) {
        self.with_view(|v| {
            for item in &mut v.items {
                item.label = None;
            }
            v.unfold_cache = None;
            v.snap_item_motion();
            let _ = v.redraw();
        });
    }

    /// Replaces the tab strip. `active` is the fence whose items the window shows.
    pub fn set_tabs(&self, mut tabs: Vec<TabView>, active: FenceId) {
        self.with_view(|v| {
            // A tab drag in progress shows its own provisional order: a refresh with the same set
            // of tabs (the SwitchTab pushed on button-down) keeps that order, updating titles only.
            if v.tab_drag.is_some()
                && v.tabs.len() == tabs.len()
                && v.tabs.iter().all(|t| tabs.iter().any(|n| n.id == t.id))
            {
                let order: Vec<FenceId> = v.tabs.iter().map(|t| t.id).collect();
                tabs.sort_by_key(|t| order.iter().position(|id| *id == t.id).unwrap_or(0));
            }
            let same_titles = v.tabs.len() == tabs.len()
                && v.tabs.iter().zip(tabs.iter()).all(|(a, b)| {
                    a.id == b.id
                        && a.title == b.title
                        && a.color == b.color
                        && a.title_size == b.title_size
                        && a.title_color == b.title_color
                });
            let same_set = v.tabs.len() == tabs.len()
                && v.tabs.iter().all(|t| tabs.iter().any(|n| n.id == t.id));
            let active_changed = v.active != active;
            let old_active = v.active;
            let old_idx = v.tabs.iter().position(|t| t.id == v.active);
            let new_idx = tabs.iter().position(|t| t.id == active);
            let old_pill = old_idx.and_then(|i| v.tab_rects().get(i).copied());
            let now = Instant::now();
            // Painted x per tab before the change: a reorder (左移 / 右移, or a width change)
            // slides the pills from there to their new slots (167 ms point-to-point).
            let prev: Vec<(FenceId, f32)> = v
                .tabs
                .iter()
                .map(|t| t.id)
                .zip(v.tab_draw_xs(now))
                .collect();
            let old_ids: Vec<FenceId> = v.tabs.iter().map(|t| t.id).collect();
            let had_strip = v.tabs.len() > 1;
            v.tabs = tabs;
            v.active = active;
            if !same_titles {
                // Reorder, rename, or a tab added / removed (merge, tear-off, close): the
                // survivors slide from where they were painted into their new slots (WinUI
                // TabView neighbours moving into the vacated slot); a drag in progress keeps
                // its own provisional positions.
                if v.tab_drag.is_none() {
                    v.sync_tab_slides(&prev, now);
                } else if !same_set {
                    v.tab_slide.clear();
                    v.tab_settle = None;
                }
            }
            if !same_set {
                // Per-id state of tabs that left goes with them; index-keyed hover fades
                // would otherwise replay on whichever neighbour inherited the index.
                let ids: Vec<FenceId> = v.tabs.iter().map(|t| t.id).collect();
                v.tab_slide.retain(|(id, _)| ids.contains(id));
                if v.tab_settle.is_some_and(|(id, _)| !ids.contains(&id)) {
                    v.tab_settle = None;
                }
                v.chrome_fades.prune(now, |k| {
                    !matches!(k, ChromeKey::Tab(_) | ChromeKey::TabDrop(_))
                });
                // Closed / merged / detached tabs take their remembered offsets with them.
                v.scroll_memory.retain(|id, _| ids.contains(id));
                // A pill that just joined a strip fades in (83 ms linear).
                if had_strip || v.tabs.len() > 1 {
                    let animate = v.motion.enabled();
                    for id in ids.iter().filter(|id| !old_ids.contains(id)) {
                        v.chrome_fades.snap(ChromeKey::TabNew(*id), 1.0, now);
                        v.chrome_fades
                            .set(animate, ChromeKey::TabNew(*id), 0.0, now);
                    }
                    v.frames.request();
                }
            }
            if active_changed {
                v.selected.clear();
                v.anchor_index = None;
                v.range_anchor = None;
                v.focus_visible = false;
                v.hover = None;
                v.reset_item_fades();
                // Explorer tabs keep their own scroll offset: remember the outgoing tab's and
                // come back to the incoming tab's (0 when it was never shown). The incoming
                // items arrive in `set_items`, which re-applies `scroll_restore`.
                if v.tabs.iter().any(|t| t.id == old_active) {
                    v.scroll_memory.insert(old_active, v.scroll_target());
                }
                let restored = v.scroll_memory.get(&active).copied().unwrap_or(0.0);
                v.scroll_anim = None;
                v.scroll_y = restored;
                v.scroll_restore = Some(restored);
                // A real switch between two existing tabs (click, Ctrl+Tab) — not creation,
                // merge, detach or closing the active tab — gets the content swap and the pill
                // glide. Rolled / animating fences and animations-off snap as before.
                if let (Some(o), Some(n)) = (old_idx, new_idx)
                    && o != n
                    && !v.rolled_up
                    && !v.height_animating()
                    && v.motion.enabled()
                {
                    v.begin_content_swap(if n > o { 1 } else { -1 });
                    // While a drag has already reordered the strip the pressed tab carries the
                    // fill itself; otherwise the pill glides (167 ms decelerate, the Fluent
                    // selection-indicator curve).
                    let dragging = v
                        .tab_drag
                        .as_ref()
                        .is_some_and(|d| d.index != d.start_index);
                    if !dragging
                        && let (Some((ox, ow)), Some(&(nx, nw))) = (old_pill, v.tab_rects().get(n))
                    {
                        let now = Instant::now();
                        let tween = |from: f32, to: f32| {
                            v.motion
                                .tween(from, to, motion::FAST, Curve::Decelerate, now)
                        };
                        v.pill_anim = Some((tween(ox, nx), tween(ow, nw)));
                        v.frames.request();
                    }
                }
            }
            if !active_changed {
                // A strip refresh with the same active tab must not leave a stale pending
                // offset for a later `set_items`.
                v.scroll_restore = None;
            }
            if !same_titles || active_changed {
                v.tab_hover = None;
                v.drop_tab = None;
                let _ = v.redraw_chrome_only();
            }
        });
    }

    /// Portal title decorations and folder double-click behaviour for the fence being shown.
    /// After a tab was torn off by dragging: this (host) window still holds the mouse capture
    /// and from now on moves `hwnd` (the new fence window) with the cursor until release.
    /// `change` restores the original host and order on Escape or right button.
    pub fn take_cancelled_detach(&self) -> bool {
        self.view
            .borrow_mut()
            .as_mut()
            .is_some_and(|v| std::mem::take(&mut v.detach_cancelled))
    }

    pub fn begin_remote_drag(&self, hwnd: HWND, change: pecofence_core::TabDetach) {
        let r = window::window_rect(hwnd);
        let pt = window::cursor_pos();
        let mut started = false;
        self.with_view(|v| {
            tracing::debug!(
                pending = v.detach_pending,
                offset = ?(pt.x - r.left, pt.y - r.top),
                "begin_remote_drag"
            );
            if !v.detach_pending {
                return;
            }
            v.detach_pending = false;
            v.remote_drag = Some(RemoteDrag {
                hwnd,
                fence: change.tab,
                offset: (pt.x - r.left, pt.y - r.top),
                merge_target: 0,
                merge_x: i32::MIN,
                moved_once: false,
                origin: WindowDragOrigin::Tab(Box::new(change)),
                press: (pt.x, pt.y),
                last_pointer: (pt.x, pt.y),
                pending_pointer: Some((pt.x, pt.y)),
                started: true,
                requests: 1,
                applied: 0,
            });
            v.frames.request();
            started = true;
        });
        if started {
            // Esc with a stationary pointer must reach this window's WM_KEYDOWN: the pointer
            // now rests on the torn-off window, so keep the keyboard focus with the capture
            // (no borrow held: SetFocus re-enters the handler). The GetKeyState poll in
            // WM_MOUSEMOVE stays as the fallback when another thread owns the foreground.
            window::set_focus(self.hwnd());
        }
    }

    /// Highlights (or clears) this window's title as a drag-to-merge target: the wash + ring
    /// fade in over 83 ms and out over 167 ms, and a tabbed strip opens an insertion gap at
    /// the slot under pointer screen `x` (neighbours slide 167 ms). Cheap when nothing changed:
    /// the hint is re-sent for every pointer move along the target.
    pub fn set_merge_hint(&self, on: bool, x: i32) {
        self.with_view(|v| {
            let mut changed = v.set_merge_gap(on.then_some(x));
            if v.merge_hint != on {
                v.merge_hint = on;
                let (to, dur) = if on {
                    (1.0, motion::FASTER)
                } else {
                    (0.0, motion::FAST)
                };
                v.merge_t
                    .retarget(to, v.anim_dur(dur), Curve::Linear, Instant::now());
                changed = true;
            }
            if changed {
                let _ = v.redraw_chrome_only();
            }
        });
    }

    /// Where a fence dropped on this title at pointer screen `x` lands in the strip (the slot
    /// the merge gap sits at); `tabs.len()` = append.
    pub fn merge_slot_at(&self, x: i32) -> usize {
        self.view
            .borrow()
            .as_ref()
            .map_or(usize::MAX, |v| v.merge_slot_at(x))
    }

    /// Details column widths (修改日期, 类型, 大小) for the fence being shown.
    pub fn set_column_widths(&self, widths: [f32; 3]) {
        self.with_view(|v| {
            if v.column_widths != widths && v.col_drag.is_none() {
                v.column_widths = widths;
                if v.layout == ViewLayout::Details {
                    let _ = v.redraw_content();
                }
            }
        });
    }

    pub fn set_portal_deco(
        &self,
        is_portal: bool,
        folder_icon: bool,
        up_button: bool,
        navigate_folders: bool,
    ) {
        self.with_view(|v| {
            let deco = TitleDeco {
                folder_icon,
                up_button,
            };
            let changed = v.deco != deco || v.navigate_folders != navigate_folders;
            let portal_changed = v.is_portal != is_portal;
            v.deco = deco;
            v.navigate_folders = navigate_folders;
            v.is_portal = is_portal;
            if !up_button {
                v.up_hovered = false;
                if v.pressed == Some(PressTarget::Up) {
                    v.pressed = None;
                    v.press_inside = false;
                }
            }
            if changed {
                let _ = v.redraw_chrome_only();
            }
            if portal_changed && v.items.is_empty() && !v.rolled_up {
                // The empty-state wording differs for a folder portal.
                let _ = v.redraw_content();
            }
        });
    }

    /// The fence whose items this window currently shows (host or active tab).
    pub fn active_fence(&self) -> FenceId {
        self.view
            .borrow()
            .as_ref()
            .map(|v| v.active)
            .unwrap_or(self.id)
    }

    /// Item whose inline rename edit this window currently shows (see `set_renaming`).
    pub fn renaming_item(&self) -> Option<ItemId> {
        self.view.borrow().as_ref().and_then(|v| v.renaming)
    }

    /// Whether the fence being shown is the desktop Inbox (chooses the empty-state text).
    pub fn set_is_inbox(&self, on: bool) {
        self.with_view(|v| {
            if v.is_inbox != on {
                v.is_inbox = on;
                if v.items.is_empty() {
                    let _ = v.redraw();
                }
            }
        });
    }

    /// Switches between the icon grid and the List / Details rows.
    pub fn set_layout(&self, layout: ViewLayout) {
        self.with_view(|v| {
            // Consumes the tab switch's pending offset either way (see `set_tabs`).
            let restore = v.scroll_restore.take();
            if v.layout == layout {
                return;
            }
            v.layout = layout;
            v.scroll_anim = None;
            // A genuine layout change starts at the top; a tab switch between tabs of
            // different layouts keeps the restored offset (the redraw clamps it).
            v.scroll_y = restore.unwrap_or(0.0);
            v.header_hover = None;
            v.header_fades.clear();
            v.reset_item_fades();
            v.drop_icons_for_reload();
            v.snap_item_motion();
            let _ = v.redraw();
        });
    }

    /// Current sort, shown by the details header (chevron on the active column).
    pub fn set_sort_indicator(&self, sort: SortMode, reverse: bool) {
        self.with_view(|v| {
            if v.sort != sort || v.sort_reverse != reverse {
                v.sort = sort;
                v.sort_reverse = reverse;
                if v.layout == ViewLayout::Details {
                    let _ = v.redraw_content();
                }
            }
        });
    }

    /// "按时间分组": the items sit under 今天 / 昨天 / 本周 / 本月 / 更早 section headers, in
    /// every layout. Push it before `set_items` so the layout glide targets the new sections.
    pub fn set_group_by_date(&self, on: bool) {
        self.with_view(|v| {
            if v.group_by_date == on {
                return;
            }
            v.group_by_date = on;
            // The cells a running glide was computed for no longer exist.
            v.snap_item_motion();
            let _ = v.redraw_content();
        });
    }

    pub fn is_rolled(&self) -> bool {
        self.view
            .borrow()
            .as_ref()
            .is_some_and(|v| v.rolled_up || v.peeking)
    }

    pub fn icon_size(&self) -> u32 {
        self.view
            .borrow()
            .as_ref()
            .map(|v| v.icon_size)
            .unwrap_or(48)
    }

    /// Ends a peek without rolling back (the fence stays expanded).
    pub fn commit_expanded(&self) {
        self.with_view(|v| {
            v.peeking = false;
            v.rolled_up = false;
        });
    }

    /// Animates to the rolled/expanded state.
    pub fn set_rolled(&self, rolled: bool) {
        self.with_view(|v| {
            v.peeking = false;
            if v.rolled_up == rolled && v.roll_anim.is_none() {
                return;
            }
            if !rolled && v.expanded_h_px <= v.title_h_px() {
                v.expanded_h_px = v.title_h_px() + (200.0 * v.scale()) as i32;
            }
            v.start_roll_anim(rolled);
        });
    }

    pub fn expanded_height_px(&self) -> i32 {
        self.view
            .borrow()
            .as_ref()
            .map(|v| v.expanded_h_px)
            .unwrap_or(0)
    }

    pub fn set_auto_height(&self, on: bool) {
        self.with_view(|v| v.auto_height = on);
    }

    pub fn set_locked(&self, on: bool) {
        self.with_view(|v| v.locked = on);
    }

    /// Re-reads the window's DPI and relayouts if it drifted (logon-time 96-DPI windows).
    pub fn check_dpi(&self) -> bool {
        let hwnd = self.hwnd();
        let dpi = monitors::dpi_for_window(hwnd);
        let mut changed = false;
        self.with_view(|v| {
            if v.apply_dpi(dpi) {
                changed = true;
                v.update_backdrop();
                let (w, h) = window::window_rect_size(hwnd);
                if let Err(e) = v.layout_panels(w, h).and_then(|_| v.redraw()) {
                    tracing::error!(error = %e, "relayout after DPI drift failed");
                }
                v.update_shadow();
            }
        });
        changed
    }

    /// Per-fence material/opacity override (None = follow the global setting).
    pub fn set_appearance(&self, backdrop: Option<BackdropMode>, opacity: f32) {
        self.with_view(|v| {
            if v.backdrop_override != backdrop || (v.opacity - opacity).abs() > f32::EPSILON {
                v.backdrop_override = backdrop;
                v.opacity = opacity;
                apply_system_backdrop(
                    v.hwnd,
                    v.backdrop_mode(),
                    matches!(v.theme.mode, pecofence_render::ThemeMode::Dark),
                );
                v.invalidate_backdrop();
                let _ = v.redraw();
            }
        });
    }

    /// Per-fence colour wash, title colour and title size.
    pub fn set_style(&self, style: FenceStyle) {
        self.with_view(|v| {
            if v.style != style {
                let previous_ink = v.material_foreground().text_primary;
                v.style = style;
                if previous_ink != v.material_foreground().text_primary {
                    let _ = v.redraw();
                } else {
                    let _ = v.redraw_chrome_only();
                }
            }
        });
    }

    pub fn set_spacing(&self, spacing: Spacing) {
        self.with_view(|v| {
            if v.spacing != spacing {
                v.spacing = spacing;
                for item in &mut v.items {
                    item.label = None;
                }
                v.snap_item_motion();
                let _ = v.redraw();
            }
        });
    }

    /// Drops every cached icon (global icon processing changed); they reload lazily.
    pub fn drop_icons(&self) {
        self.with_view(|v| {
            for item in &mut v.items {
                item.icon = None;
                item.icon_failed = false;
                item.icon_waited = false;
                item.icon_fade = None;
            }
            let _ = v.redraw();
        });
    }

    /// Height the window should have when auto-height is on (None when off / rolled).
    pub fn auto_height_px(&self) -> Option<i32> {
        self.view.borrow().as_ref().and_then(|v| v.auto_height_px())
    }

    /// Applies a new expanded height. `animate` (auto-height following an item change) glides
    /// the bottom edge there over 250 ms; otherwise, or when the animation declines (hidden
    /// window, animations off, roll in progress), the window snaps. The returned rectangle is
    /// the final geometry either way, so the state persists it immediately. The borrow is
    /// released before SetWindowPos because WM_SIZE re-enters the handler.
    pub fn apply_height(&self, height_px: i32, animate: bool) -> RECT {
        let animated = {
            let mut guard = self.view.borrow_mut();
            match guard.as_mut() {
                Some(v) => {
                    v.expanded_h_px = height_px;
                    animate && v.start_height_anim(height_px)
                }
                None => false,
            }
        };
        let mut r = self.window.window_rect();
        r.bottom = r.top + height_px;
        if !animated {
            self.set_bounds(r);
        }
        r
    }

    /// Screen rectangle of an item's label, for the rename popup.
    pub fn item_label_rect(&self, item: ItemId) -> Option<RECT> {
        self.view
            .borrow()
            .as_ref()
            .and_then(|v| v.item_label_rect(item))
    }

    pub fn device_lost(&self) -> bool {
        self.view.borrow().as_ref().is_some_and(|v| v.device_lost)
    }

    /// After the render stack rebuilt its device: new surfaces on the same visuals + redraw.
    pub fn recreate_surfaces(&self, stack: &RenderStack) {
        self.with_view(|v| {
            v.gpu_glass.borrow_mut().reset();
            // The spare panels belong to the lost device too; they are recreated on demand.
            v.finish_content_swap_now();
            v.spares.clear();
            if let Err(e) = v
                .chrome_panel
                .recreate(stack)
                .and_then(|_| v.content_panel.recreate(stack))
            {
                tracing::error!(error = %e, "surface recreation failed");
                return;
            }
            for item in &mut v.items {
                item.icon = None;
                item.icon_failed = false;
                item.icon_waited = false;
                item.icon_fade = None;
            }
            v.device_lost = false;
            let (w, h) = window::window_rect_size(v.hwnd);
            if let Err(e) = v.layout_panels(w, h).and_then(|_| v.redraw()) {
                tracing::error!(error = %e, "redraw after device recovery failed");
            }
        });
    }

    /// A desktop switch changes the sampled pixels without changing window shape, layout,
    /// animations or icon resources.
    pub fn set_backdrops(&self, backdrops: Rc<BackdropSets>) {
        self.with_view(|v| {
            v.backdrops = backdrops;
            v.glass_dark_text.set(None);
            v.invalidate_backdrop();
            if let Err(error) = v.redraw() {
                tracing::warn!(%error, "redraw after wallpaper change failed");
            }
        });
    }

    pub fn set_theme(&self, theme: Theme, backdrops: Rc<BackdropSets>, shadow: ShadowStyle) {
        let hwnd = self.hwnd();
        apply_window_shape(hwnd, theme.liquid_glass);
        let mode = self
            .view
            .borrow()
            .as_ref()
            .map(|v| v.behavior.backdrop.get());
        if let Some(mode) = mode {
            apply_system_backdrop(
                hwnd,
                mode,
                matches!(theme.mode, pecofence_render::ThemeMode::Dark),
            );
        }
        let _ = self.window.recalculate_frame();
        self.with_view(|v| {
            if v.theme.mode != theme.mode {
                // comctl32 tips take their theme class at creation (DarkMode_Explorer):
                // rebuild on the next show so a tip never stays dark after a light switch.
                v.hide_tip();
                v.tip = None;
            }
            if v.theme.liquid_glass != theme.liquid_glass || v.theme.mode != theme.mode {
                v.glass_dark_text.set(None);
            }
            v.theme = theme;
            if !theme.liquid_glass {
                v.gpu_glass.borrow_mut().reset();
            }
            v.backdrops = backdrops;
            v.shadow.set_style(shadow);
            v.update_shadow();
            v.invalidate_backdrop();
            let (w, h) = window::window_rect_size(v.hwnd);
            let _ = v.layout_panels(w, h);
            let _ = v.redraw();
        });
    }

    pub fn redraw(&self) {
        self.with_view(|v| {
            let _ = v.redraw();
        });
    }

    /// Redraws only if icons are still missing (used after the icon worker delivers results).
    pub fn redraw_if_pending_icons(&self) {
        self.with_view(|v| {
            if v.has_pending_icons() {
                let _ = v.redraw();
            }
        });
    }
}
