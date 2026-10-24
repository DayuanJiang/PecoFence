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
        expand_upward: bool,
        row_bottom: bool,
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
        apply_window_shape(hwnd);
        let _ = dwm::set_border_color_none(hwnd);
        // Not DWMWA_EXCLUDED_FROM_PEEK: DWM hides such windows for the whole virtual-desktop
        // switch animation while their shadows slide, leaving empty panes. Plain windows slide
        // out and in with the desktop.
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
            tab_scroll: Cell::new(0.0),
            tab_edge_last: None,
            merge_pointer: None,
            tab_settle: None,
            drop_tab: None,
            drag_peek_armed: false,
            rbutton_press: None,
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
            search: None,
            search_dim: Fades::new(motion::FASTER, motion::FASTER, Curve::Linear),
            search_expanded: false,
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
            portal_folder: None,
            layout: ViewLayout::Icons,
            sort: SortMode::Manual,
            sort_reverse: false,
            group_by_date: false,
            header_hover: None,
            column_widths: DetailColumns::DEFAULT_WIDTHS,
            columns_visible: [true; 3],
            col_drag: None,
            is_portal: false,
            portal_missing: false,
            label_w: 0.0,
            icons_pending: false,
            hover: None,
            title_hover: false,
            selected: HashSet::new(),
            scroll_y: 0.0,
            scroll_memory: HashMap::new(),
            scroll_restore: None,
            select_after_nav: None,
            scroll_anim: None,
            dpi,
            theme,
            glass_dark_text: Cell::new(None),
            backdrops: ctx.backdrops.borrow().clone(),
            backdrop_crop: None,
            backdrop_rect: None,
            backdrop_origin: None,
            backdrop_key: format!("backdrop:{:x}", hwnd.0 as isize),
            gpu_glass: RefCell::new(pecofence_render::gpu_glass::GpuGlass::default()),
            chrome_panel,
            content_panel,
            content_x: 0,
            content_y: (theme.title_height * dpi as f32 / 96.0).round() as i32,
            rail_dip: 0.0,
            expanded_w_px: bounds.right - bounds.left,
            width_collapsed: false,
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
            wheel_tab_accum: 0,
            tab_hover_switch: None,
            auto_height,
            title_align: None,
            rail_grow: false,
            locked: false,
            expand_upward,
            kept_end: row_bottom,
            anim_end: row_bottom,
            cut_w: None,
            width_pivot: None,
            backdrop_override: None,
            opacity: 1.0,
            desktop: None,
            desktop_slots: Vec::new(),
            desktop_band_base: None,
            desktop_shape_for: None,
            desktop_shape_pending: None,
            wanted_layout: ViewLayout::Icons,
            title_on_hover_override: None,
            style: FenceStyle::default(),
            spacing: Spacing::Normal,
            icon_align: IconAlign::Left,
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
            title_reveal: Tween::at(1.0, Instant::now()),
            plate_reveal: Tween::at(1.0, Instant::now()),
            plate_inset_px: 0,
            plate_alpha: 1.0,
            peeking: false,
            behavior: ctx.behavior.clone(),
        };
        if rolled_up {
            // `bounds` is the expanded rect: the title row sits at its top, or at its bottom for
            // a fence that last opened upward.
            let th = state.title_h_px();
            let top = top_for_height(row_bottom, &bounds, th);
            let _ = window.set_bounds(bounds.left, top, bounds.right - bounds.left, th);
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
                source_volume: None,
                desktop: None,
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
            "title={:?} opacity={:.2} scale={:.3} fade={fade} retired={} rolled={} rolling={} peeking={} items={} tabs={} selected={} focused={:?} shadow_visible={} shadow_alpha={} layout={:?} locked={} auto_height={} active={} scroll={:.1} client={:?} client_offset=({},{}) chrome={:?} glass={:?} wallpaper_uploads={} title_size={} tab_fonts={:?} tab_rects={:?} content_x={} content={:?} capture={} tab_drag={} window_drag={} detach_pending={} plate_inset={} plate_alpha={:.2} mouse_inside={} title_edge={:?} kept_end={} column_right={} rail_dip={:.1} width_collapsed={} cut_w={:?}",
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
            v.content_x,
            v.content_panel.size_px(),
            window::get_capture() == self.hwnd(),
            v.tab_drag.is_some(),
            v.remote_drag.is_some(),
            v.detach_pending,
            v.plate_inset_px,
            v.plate_alpha,
            v.mouse_inside,
            v.style.title_edge,
            v.kept_end,
            v.column_right(),
            v.rail_dip,
            v.width_collapsed,
            v.cut_w,
        )
    }

    /// Debug-only: this window's next draw found its GPU device gone (what a failed surface
    /// draw reports), which queues the app's device recovery.
    #[cfg(debug_assertions)]
    pub fn test_lose_device(&self) {
        self.with_view(|v| v.note_draw_result(false));
    }

    /// Debug-only drop regression hook: runs this window's OLE drop handler for `data` at the
    /// window centre (or over the item whose name contains `at`) with `key_state`, as a drag
    /// from another program offering `allowed` (None = every effect) would, without the drag
    /// image. Returns what DragEnter reported and, when `perform`, what Drop returned.
    #[cfg(debug_assertions)]
    #[allow(clippy::too_many_arguments)]
    pub fn test_drop(
        &self,
        ctx: &FenceContext,
        data: &dragdrop::IDataObject,
        key_state: u32,
        perform: bool,
        at: Option<&str>,
        allowed: Option<u32>,
    ) -> (DropEffect, Option<DropEffect>) {
        use pecofence_platform::dragdrop::DropHandler;
        let r = self.rect();
        let (x, y) = at
            .and_then(|name| {
                let view = self.view.borrow();
                view.as_ref().and_then(|v| v.test_item_point(name))
            })
            .unwrap_or(((r.left + r.right) / 2, (r.top + r.bottom) / 2));
        let pt = dragdrop::DragPoint { x, y };
        let allowed = allowed.unwrap_or(
            DropEffect::Copy.to_raw() | DropEffect::Move.to_raw() | DropEffect::Link.to_raw(),
        );
        let mut handler = FenceDropHandler {
            queue: ctx.queue.clone(),
            view: self.view.clone(),
            kind: DropKind::None,
            behavior: ctx.behavior.clone(),
            helper: None,
            data: None,
            last_description: None,
            right_button: false,
            over_stats: Default::default(),
            source_volume: None,
            desktop: None,
        };
        let entered = handler.drag_enter(data, key_state, pt, allowed);
        if !perform {
            handler.drag_leave();
            return (entered, None);
        }
        let dropped = handler.drop(data, key_state, pt, allowed);
        (entered, Some(dropped))
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
            // The pointer entering (x = 1) / leaving (x = 0) the fence without a real cursor:
            // title on hover and 全透明 react as they do to WM_MOUSEMOVE / WM_MOUSELEAVE.
            "hover" => self.with_view(|v| v.set_mouse_inside(x != 0)),
            _ => {
                let (message, key, point) = match action {
                    "down" => (msg::WM_LBUTTONDOWN, 1, (x, y)),
                    "move" => (msg::WM_MOUSEMOVE, 1, (x, y)),
                    "up" => (msg::WM_LBUTTONUP, 0, (x, y)),
                    "dblclk" => (msg::WM_LBUTTONDBLCLK, 1, (x, y)),
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

    /// Test script: one pointer step of an edge drag as the system size loop runs it. The
    /// dragged edge(s) of `wmsz` (WMSZ_*) go to `x` / `y`, WM_SIZING places them and the window
    /// takes the result; [`Self::test_size_end`] then releases the drag.
    pub fn test_size(&self, wmsz: usize, x: i32, y: i32) -> RECT {
        let hwnd = self.hwnd();
        let edges = SizingEdges::from_wmsz(wmsz);
        let mut r = self.window.window_rect();
        if edges.left {
            r.left = x;
        } else if edges.right {
            r.right = x;
        }
        if edges.top {
            r.top = y;
        } else if edges.bottom {
            r.bottom = y;
        }
        window::send_message(hwnd, msg::WM_ENTERSIZEMOVE, 0, 0);
        window::send_message(hwnd, msg::WM_SIZING, wmsz, &mut r as *mut RECT as isize);
        let _ = self
            .window
            .set_bounds(r.left, r.top, r.right - r.left, r.bottom - r.top);
        r
    }

    /// Test script: releases the edge drag [`Self::test_size`] started (records the rect).
    pub fn test_size_end(&self) {
        window::send_message(self.hwnd(), msg::WM_EXITSIZEMOVE, 0, 0);
    }

    /// Explicit geometry from the app (layout, snapping, monitor changes) takes precedence over
    /// a running auto-height settle.
    pub fn set_bounds(&self, r: RECT) {
        // 像桌面一样排列 places the window itself.
        if self.is_desktop_arranged() {
            self.arrange_desktop(false);
            return;
        }
        self.place_window(r);
    }

    fn place_window(&self, r: RECT) {
        self.with_view(|v| v.height_anim = None);
        let _ = self
            .window
            .set_bounds(r.left, r.top, r.right - r.left, r.bottom - r.top);
    }

    /// A tab split/undo supplies complete geometry. Settle any hover expansion first
    /// so its old tween cannot pull the detached or restored window back on later frames.
    pub fn restore_geometry(&self, mut rect: RECT, rolled: bool, expanded_h: i32) {
        // 像桌面一样排列 places the window itself (a display change or a restore lands here).
        if self.is_desktop_arranged() {
            self.arrange_desktop(true);
            return;
        }
        self.with_view(|v| {
            v.finish_content_swap_now();
            v.roll_anim = None;
            v.height_anim = None;
            v.peeking = false;
            v.rolled_up = rolled;
            v.expanded_h_px = expanded_h;
            v.expanded_w_px = rect.right - rect.left;
            v.cut_w = None;
            v.width_collapsed = rolled && v.collapses_width();
            v.rail_dip = v.measure_rail_w();
            v.roll_t = Tween::at(rolled as u8 as f32, Instant::now());
            let _ = v.motion.stop(&v.content_panel.visual, Prop::Opacity);
            v.content_panel.visual.set_opacity(1.0);
            v.content_panel.visual.set_visible(!rolled);
            if rolled {
                v.width_pivot = None;
                if v.width_collapsed {
                    let w = v.rolled_w_px();
                    let (from, anchor) = (rect.left, v.width_anchor());
                    rect.left = super::anchored_start(rect.left, rect.right - rect.left, w, anchor);
                    // A title row shrunk around its centre pivots there, measured on the
                    // expanded rect (`pivot_left`).
                    if anchor > 0.0 && anchor < 1.0 {
                        let pivot = from as f32 + (rect.right - from) as f32 * anchor;
                        v.width_pivot = Some((pivot, rect.left, w));
                    }
                    rect.right = rect.left + w;
                }
                // Measured where the window goes (a column at its own left and right edges),
                // not where it is now.
                let mut h = v.rolled_h_px_at(&rect);
                // A title column is no taller rolled up than the rect it opens to.
                if v.side_column() {
                    h = h.min(rect.bottom - rect.top);
                }
                rect.top =
                    super::anchored_start(rect.top, rect.bottom - rect.top, h, v.height_anchor());
                rect.bottom = rect.top + h;
            } else {
                v.width_pivot = None;
                rect.bottom = rect.top + expanded_h;
            }
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

    /// The window already shows exactly `items` (same order, names, icons and details).
    pub fn shows_items(&self, items: &[ItemView]) -> bool {
        self.view.borrow().as_ref().is_some_and(|v| {
            v.items.len() == items.len()
                && v.items.iter().zip(items).all(|(a, b)| {
                    a.id == b.id
                        && a.name == b.name
                        && a.path == b.path
                        && a.is_folder == b.is_folder
                        && a.icon_key == b.icon_key
                        && a.icon_only == b.icon_only
                        && a.mtime == b.mtime
                        && a.size == b.size
                })
        })
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
            if let Some((id, ring)) = v.select_after_nav.take()
                && let Some(i) = v.items.iter().position(|it| it.id == id)
            {
                v.select_only(i);
                v.focus_visible = ring;
                if !v.rolled_up {
                    v.scroll_into_view(i, false);
                }
            }
            let _ = v.redraw();
            // The item under a stationary pointer may have changed: re-hit-test from the loop.
            v.request_hover_refresh();
        });
        // 像桌面一样排列: more or fewer icons take more or fewer cells.
        self.arrange_desktop(false);
    }

    /// Portal navigation, before the new folder's items arrive: the page swaps like a tab
    /// switch (`dir` +1 = into a subfolder, sliding in from the right; -1 = up / home, from the
    /// left) and the new folder starts at the top. `select` (the folder just left by 上一级)
    /// is selected by the next `set_items`.
    pub fn begin_portal_navigation(&self, dir: i8, select: Option<ItemId>) {
        self.with_view(|v| {
            v.scroll_anim = None;
            v.scroll_y = 0.0;
            v.select_after_nav = select.map(|id| (id, v.focus_visible));
            if !v.rolled_up
                && !v.height_animating()
                && v.motion.enabled()
                && desktop::is_visible(v.hwnd)
            {
                v.begin_content_swap(dir);
            }
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
                v.search_dim.prune(now, |id| *id != old);
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

    /// Peek search marks (None = drawn as usual). The current match glides into view; a
    /// rolled-up fence with matches expands like a hover peek and rolls back up once it has
    /// none (or the search ends).
    pub fn set_search_marks(&self, marks: Option<SearchMarks>) {
        self.with_view(|v| {
            if v.search == marks {
                return;
            }
            let expand = marks.as_ref().is_some_and(|m| !m.matches.is_empty());
            let current = marks.as_ref().and_then(|m| m.current);
            // The tab badges change the pill widths: the pills slide to their new places.
            let tabs_changed = v.tabs.len() > 1
                && v.search.as_ref().map(|m| &m.tab_hits) != marks.as_ref().map(|m| &m.tab_hits);
            let now = Instant::now();
            let prev: Vec<(FenceId, f32)> = if tabs_changed {
                v.tabs
                    .iter()
                    .map(|t| t.id)
                    .zip(v.tab_draw_xs(now))
                    .collect()
            } else {
                Vec::new()
            };
            v.search = marks;
            if tabs_changed {
                if v.tab_drag.is_none() {
                    v.sync_tab_slides(&prev, now);
                }
                let _ = v.redraw_chrome_only();
            }
            if expand && !v.search_expanded {
                if v.peeking {
                    v.search_expanded = true;
                } else if v.rolled_up && v.roll_anim.is_none() && !v.retired {
                    v.peeking = true;
                    v.search_expanded = true;
                    if v.expanded_h_px <= v.title_h_px() {
                        v.expanded_h_px = v.title_h_px() + (200.0 * v.scale()) as i32;
                    }
                    v.start_roll_anim(false);
                }
            } else if !expand && v.search_expanded {
                // The hover-peek close countdown rolls it back up (unless the pointer is on it).
                v.search_expanded = false;
                v.resume_peek_close();
            }
            if v.rolled_up {
                return;
            }
            if let Some(i) = current.and_then(|id| v.items.iter().position(|it| it.id == id)) {
                v.scroll_into_view(i, true);
            }
            let _ = v.redraw_content();
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
                // Title on hover / 全透明: the title row and plate stay up under the popup.
                let _ = v.redraw_chrome_only();
            }
        });
    }

    pub fn set_title(&self, title: &str) {
        let mut changed = false;
        self.with_view(|v| {
            if v.title == title {
                return;
            }
            v.title = title.to_string();
            changed = true;
            let _ = v.redraw();
        });
        if changed {
            // UI Automation / screen readers read the window text. SetWindowText sends
            // WM_SETTEXT synchronously: called with no borrow of the view held.
            window::set_title(self.hwnd(), title);
        }
    }

    pub fn set_icon_size(&self, size: u32) {
        self.with_view(|v| {
            v.icon_size = size;
            v.reload_icons_and_labels();
            v.snap_item_motion();
            let _ = v.redraw();
        });
        // 像桌面一样排列: other cells.
        self.arrange_desktop(false);
    }

    /// Lines of label text under each icon (1-3): refit the labels and relayout at once.
    pub fn set_label_lines(&self, lines: u8) {
        self.with_view(|v| {
            if v.label_lines == lines {
                return;
            }
            v.label_lines = lines;
            for item in &mut v.items {
                item.label = None;
            }
            v.unfold_cache = None;
            v.snap_item_motion();
            let _ = v.redraw();
        });
        self.arrange_desktop(false);
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
        self.arrange_desktop(false);
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
                        && a.folder_icon == b.folder_icon
                        && a.up_button == b.up_button
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
            // A new window gets its tabs here for the first time: what it was saved with.
            let first = v.tabs.is_empty();
            let rail_fit = v.rail_fit_height_px();
            let strip_scroll = v.tab_scroll.get();
            v.tabs = tabs;
            v.active = active;
            // A title column comes with the second tab and goes with the last but one.
            let rail_changed = v.sync_rail();
            // Only a column that needs more room than before (a new tab, a longer caption)
            // asks the height pass to grow the fence; a reorder, a colour or a start does not.
            if !first && v.rail_fit_height_px() > rail_fit {
                v.rail_grow = true;
            }
            // A strip too wide for its room scrolled to show the new active tab.
            let _ = v.tab_rects();
            let scrolled = (v.tab_scroll.get() - strip_scroll).abs() > 0.5;
            if !same_titles || scrolled {
                // Reorder, rename, or a tab added / removed (merge, tear-off, close): the
                // survivors slide from where they were painted into their new slots (WinUI
                // TabView neighbours moving into the vacated slot); a drag in progress keeps
                // its own provisional positions. A scrolled strip slides the same way.
                let pressed = v.tab_drag.as_ref().is_some_and(|d| !d.moved);
                if v.tab_drag.is_none() || (pressed && same_set) {
                    // A press that switched to a cut-off tab scrolled the strip: the grab
                    // moves with the pressed tab, so a drag from there starts at its new slot.
                    if let Some(d) = v.tab_drag.as_mut() {
                        d.grab_dx += v.tab_scroll.get() - strip_scroll;
                    }
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
            if rail_changed {
                let _ = v.redraw();
            }
        });
    }

    /// 标签栏位置 changed: the tabs of a fence with its title bar at the top move between the
    /// title row and the left column.
    pub fn sync_tab_bar(&self) {
        self.with_view(|v| {
            // Only the CLI can change the setting mid-drag.
            v.reset_tab_axis();
            let _ = v.redraw();
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
                offset_dpi: monitors::dpi_for_window(hwnd),
                merge_target: 0,
                merge_at: (i32::MIN, i32::MIN),
                moved_once: false,
                origin: WindowDragOrigin::Tab(Box::new(change)),
                group: Vec::new(),
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
    /// the slot under pointer screen point `at` (neighbours slide 167 ms). Cheap when nothing
    /// changed: the hint is re-sent for every pointer move along the target.
    pub fn set_merge_hint(&self, on: bool, at: (i32, i32)) {
        self.with_view(|v| {
            let mut changed = v.set_merge_gap(on.then_some(at));
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

    /// Where a fence dropped on this title at pointer screen point `at` lands in the strip (the
    /// slot the merge gap sits at); `tabs.len()` = append.
    pub fn merge_slot_at(&self, at: (i32, i32)) -> usize {
        self.view
            .borrow()
            .as_ref()
            .map_or(usize::MAX, |v| v.merge_slot_at(at))
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
        folder_missing: bool,
        portal_folder: Option<PathBuf>,
    ) {
        self.with_view(|v| {
            let deco = TitleDeco {
                folder_icon,
                up_button,
            };
            let changed = v.deco != deco || v.navigate_folders != navigate_folders;
            let portal_changed = v.is_portal != is_portal || v.portal_missing != folder_missing;
            v.deco = deco;
            v.navigate_folders = navigate_folders;
            v.portal_folder = portal_folder;
            v.is_portal = is_portal;
            v.portal_missing = folder_missing;
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
                // The empty-state wording differs for a folder portal, and for a folder that
                // cannot be found.
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
            // 像桌面一样排列 shows icons whatever the fence's view; the fence's own comes back
            // when it goes off (`set_desktop_area`).
            v.wanted_layout = layout;
            let layout = if v.bare() { ViewLayout::Icons } else { layout };
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
            v.reload_icons_and_labels();
            v.snap_item_motion();
            let _ = v.redraw();
        });
    }

    /// Tab switch, after `set_tabs` and before the incoming tab's items arrive: takes that
    /// tab's icon size and layout without drawing the outgoing items again, so the new items
    /// are drawn, and their icons extracted, at their own size from the first frame; the icon
    /// cache keeps an inactive tab's icons at that size only.
    pub fn set_incoming_tab_view(&self, icon_size: u32, layout: ViewLayout) {
        self.with_view(|v| {
            if v.icon_size == icon_size && v.layout == layout {
                return;
            }
            if v.layout != layout {
                v.header_hover = None;
                v.header_fades.clear();
            }
            v.icon_size = icon_size;
            v.layout = layout;
            v.reload_icons_and_labels();
            v.snap_item_motion();
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

    pub fn dpi(&self) -> u32 {
        self.view.borrow().as_ref().map(|v| v.dpi).unwrap_or(96)
    }

    /// Adds the `IconCache::draw_key`s this window draws or is about to draw to `out`: every
    /// item at the current icon size, and at the size it still shows until that one arrives.
    pub fn needed_icons(&self, icons: &IconCache, out: &mut HashSet<String>) {
        if let Some(v) = self.view.borrow().as_ref() {
            let px = crate::layout::icon_px(v.layout, v.icon_size, v.dpi);
            for item in &v.items {
                out.insert(icons.draw_key(&item.icon_key, px));
                if item.icon.is_some() && item.icon_px != px {
                    out.insert(icons.draw_key(&item.icon_key, item.icon_px));
                }
            }
        }
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

    /// The window as it rests: a rolled fence that is hover-peeking or rolling counts as its
    /// title bar, a row or a column (see `resting_heights_px`).
    pub fn resting_rect(&self) -> RECT {
        let r = self.rect();
        // Rolled (and maybe hover-peeking open) by its state, not by a roll in flight: a peek
        // committed open counts at its full size at once.
        self.view
            .try_borrow()
            .ok()
            .and_then(|g| {
                g.as_ref()
                    .filter(|v| v.rolled_up || v.peeking)
                    .map(|v| rolled_bar_in(v, r))
            })
            .unwrap_or(r)
    }

    /// Title-row height and the height the window settles at: the title row while it is rolled
    /// up, rolling up or hover-peeking, else the expanded height. The push pass lays the fences
    /// out from these (`app/push.rs`).
    pub fn resting_heights_px(&self) -> Option<(i32, i32)> {
        let guard = self.view.try_borrow().ok()?;
        let v = guard.as_ref()?;
        let title = v.title_h_px();
        let rolled = v.roll_target() || v.peeking;
        Some((
            title,
            if rolled {
                v.rolled_h_px()
            } else {
                v.expanded_h_px
            },
        ))
    }

    /// Screen rect of tab `tab`'s row in the title column (None without a column): where the
    /// rename box goes.
    pub fn rail_row_screen_rect(&self, tab: FenceId) -> Option<RECT> {
        let guard = self.view.try_borrow().ok()?;
        let v = guard.as_ref()?;
        if !v.tabs_on_rail() {
            return None;
        }
        let i = v.tabs.iter().position(|t| t.id == tab)?;
        let (y, h) = *v.tab_rects().get(i)?;
        let s = v.scale();
        let win = self.rect();
        let at = |dip: f32| (dip * s).round() as i32;
        if v.vertical_tabs() {
            // The box does not turn: it lies across the tab's start and reaches into the fence,
            // as for a single vertical title (`title_rename_rect`).
            let w = (win.right - win.left - at(8.0)).max(at(160.0));
            let (left, right) = if v.column_right() {
                (win.right - at(4.0) - w, win.right - at(4.0))
            } else {
                (win.left + at(4.0), win.left + at(4.0) + w)
            };
            let top = win.top + at(y);
            return Some(RECT {
                left,
                top,
                right,
                bottom: top + at(h.min(28.0)),
            });
        }
        let left = win.left + v.column_x_px();
        Some(RECT {
            left: left + at(RAIL_ROW_INSET),
            top: win.top + at(y),
            right: left + at(v.rail_w() - RAIL_ROW_RIGHT),
            bottom: win.top + at(y + h),
        })
    }

    /// Screen rect for the title's rename box when it is not in a top title row with the chevron
    /// at its end, where the box has its own spot: the top of a single fence's title column, a
    /// bottom title row, or a row whose chevron leads, which keeps the start of the row free for
    /// the chevron instead of the end.
    pub fn title_rename_rect(&self) -> Option<RECT> {
        let guard = self.view.try_borrow().ok()?;
        let v = guard.as_ref()?;
        let s = v.scale();
        let win = self.rect();
        if v.title_column() && !v.tabs_on_rail() {
            // The box does not turn: it lies across the column's top and reaches into the
            // fence, past the window while the fence is collapsed to its column.
            let at = |dip: f32| (dip * s).round() as i32;
            let w = (win.right - win.left - at(8.0)).max(at(160.0));
            let (left, right) = if v.column_right() {
                (win.right - at(4.0) - w, win.right - at(4.0))
            } else {
                (win.left + at(4.0), win.left + at(4.0) + w)
            };
            let top = win.top + at(RAIL_TOP);
            return Some(RECT {
                left,
                top,
                right,
                bottom: top + at(28.0),
            });
        }
        let leads = v.chevron_leads();
        if !leads && !v.title_bottom() {
            return None;
        }
        let (lead, trail) = if leads {
            (v.deco.lead_title_start(), 8.0)
        } else {
            (8.0, 44.0)
        };
        let left = win.left + (lead * s) as i32;
        let top = win.top + v.title_top_px();
        Some(RECT {
            left,
            top,
            right: (win.right - (trail * s) as i32).max(left + (80.0 * s) as i32),
            bottom: top + v.title_h_px(),
        })
    }

    /// Window rect, as wide as the fence expands to while it is collapsed to its title column:
    /// the room automatic placement keeps clear.
    pub fn footprint_rect(&self) -> RECT {
        let mut r = self.rect();
        if let Some(ew) = self.collapsed_expanded_width_px() {
            r.left = super::anchored_start(r.left, r.right - r.left, ew, self.width_anchor());
            r.right = r.left + ew;
        }
        r
    }

    /// Screen rect of the rolled title bar: the window rolled up at rest, else the bar at the
    /// edges a hover peek or a roll in flight keeps, where the window rolls back to.
    pub fn rolled_bar_rect(&self) -> RECT {
        let r = self.window.window_rect();
        self.view
            .borrow()
            .as_ref()
            .map_or(r, |v| rolled_bar_in(v, r))
    }

    /// Width (device px) the window opens to: its own, or the width a title column keeps while
    /// it is collapsed or open narrower for want of room.
    pub fn expanded_width_px(&self) -> i32 {
        let r = self.window.window_rect();
        self.view
            .borrow()
            .as_ref()
            .map_or(r.right - r.left, |v| v.expanded_w_px)
    }

    /// A title column's full width when its saved rect was cut to the room beside it
    /// (`AppState::fence_expanded_w_px`): the next expansion targets it, and an open window
    /// narrower than that keeps it.
    pub fn set_expanded_width_px(&self, full: i32) {
        let r = self.window.window_rect();
        self.with_view(|v| {
            if full > v.expanded_w_px {
                if !v.width_collapsed {
                    v.cut_w = Some(r.right - r.left);
                }
                v.expanded_w_px = full;
            }
        });
    }

    /// Width the window expands to while it is collapsed to its title column (None
    /// otherwise: the window has its expanded width).
    pub fn collapsed_expanded_width_px(&self) -> Option<i32> {
        let guard = self.view.try_borrow().ok()?;
        let v = guard.as_ref()?;
        v.width_collapsed.then_some(v.expanded_w_px)
    }

    /// A rolled fence whose tabs changed (added, removed, renamed, or the title edge switched)
    /// takes its rolled size again: the title column it is collapsed to, or its title row at
    /// the expanded width. Returns the new window rect when it changed.
    pub fn settle_rolled_size(&self) -> Option<RECT> {
        let (w, h) = {
            let mut guard = self.view.borrow_mut();
            let v = guard.as_mut()?;
            if !v.rolled_up || v.peeking || v.roll_anim.is_some() || v.retired {
                return None;
            }
            let (cur_w, _) = window::window_rect_size(v.hwnd);
            let w = if v.collapses_width() {
                v.rail_dip = v.measure_rail_w();
                v.rolled_w_px()
            } else if v.width_collapsed {
                v.expanded_w_px
            } else {
                cur_w
            };
            v.width_collapsed = v.collapses_width();
            (w, v.rolled_h_px())
        };
        let r = self.rect();
        if (r.right - r.left, r.bottom - r.top) == (w, h) {
            return None;
        }
        let top = super::anchored_start(r.top, r.bottom - r.top, h, self.height_anchor());
        let left = self.pivoted_left(&r, w, self.width_anchor());
        // No borrow held: WM_SIZE lays the panels out at the new size.
        let _ = self.window.set_bounds(left, top, w, h);
        self.with_view(|v| v.update_shadow_now());
        Some(window::window_rect(self.hwnd()))
    }

    /// Windows a title or tear-off drag driven by this window moves right now (the dragged one
    /// and the selection moving along); empty when none runs.
    pub fn dragged_windows(&self) -> Vec<HWND> {
        self.view
            .try_borrow()
            .ok()
            .and_then(|g| {
                g.as_ref()?.remote_drag.as_ref().map(|rd| {
                    std::iter::once(rd.hwnd)
                        .chain(rd.group.iter().map(|m| m.hwnd))
                        .collect()
                })
            })
            .unwrap_or_default()
    }

    pub fn expanded_height_px(&self) -> i32 {
        self.view
            .borrow()
            .as_ref()
            .map(|v| v.expanded_h_px)
            .unwrap_or(0)
    }

    /// Height the next expand targets, for a rolled window (leaves the window as it is).
    pub fn set_expanded_height_px(&self, height_px: i32) {
        self.with_view(|v| v.expanded_h_px = height_px);
    }

    pub fn set_auto_height(&self, on: bool) {
        self.with_view(|v| v.auto_height = on);
    }

    /// Left edge of the window at `r` going `w` wide around `anchor` (`anchored_start`). Around a
    /// centre each change's rounding would add up, so the pivot is kept (`pivot_left`).
    pub(super) fn pivoted_left(&self, r: &RECT, w: i32, anchor: f32) -> i32 {
        if anchor <= 0.0 || anchor >= 1.0 {
            return super::anchored_start(r.left, r.right - r.left, w, anchor);
        }
        let mut left = r.left;
        self.with_view(|v| {
            let (l, kept) = pivot_left(v.width_pivot, r.left, r.right - r.left, w, anchor);
            v.width_pivot = Some(kept);
            left = l;
        });
        left
    }

    /// Where height changes of a collapse pivot (`anchored_start`).
    pub fn height_anchor(&self) -> f32 {
        self.view
            .borrow()
            .as_ref()
            .map_or(0.0, |v| v.height_anchor())
    }

    /// Where width changes pivot (`anchored_start`).
    pub fn width_anchor(&self) -> f32 {
        self.view
            .borrow()
            .as_ref()
            .map_or(0.0, |v| v.width_anchor())
    }

    pub fn set_locked(&self, on: bool) {
        self.with_view(|v| v.locked = on);
    }

    /// 向上展开 on or off. Nothing moves: the next expansion picks its side by the new setting.
    /// Returns true when the fence is rolled up at rest, so the caller records its expanded
    /// rect again around the title row (switched off, below a top one). An open fence or one in
    /// a roll keeps the edge it has until it is rolled up.
    pub fn set_expand_upward(&self, on: bool) -> bool {
        let mut guard = self.view.borrow_mut();
        let Some(v) = guard.as_mut() else {
            return false;
        };
        if v.expand_upward == on {
            return false;
        }
        v.expand_upward = on;
        let at_rest = v.rolled_at_rest();
        if at_rest && !on && !v.side_column() && !v.title_bottom() {
            v.kept_end = false;
        }
        at_rest
    }

    /// The title row is at the bottom: the window keeps its bottom edge as its height changes.
    pub fn row_at_bottom(&self) -> bool {
        self.view
            .borrow()
            .as_ref()
            .is_some_and(|v| v.row_at_bottom())
    }

    /// The title column keeps the right edge: rolled up, it sits at the right of the saved rect.
    pub fn column_at_right(&self) -> bool {
        self.view
            .borrow()
            .as_ref()
            .is_some_and(|v| v.column_at_right())
    }

    /// The edge the fence collapses to: the one that stays put as it rolls up and expands.
    pub fn collapse_to(&self) -> pecofence_core::TitleEdge {
        self.view
            .borrow()
            .as_ref()
            .map_or(pecofence_core::TitleEdge::Top, |v| v.collapse_to())
    }

    /// Which end of the saved rect a rolled title bar sits at: the bottom for a title row, the
    /// right for a title column (`AppState::set_fence_bounds`, `column_rest_side` choose it).
    pub fn set_kept_end(&self, on: bool) {
        self.with_view(|v| {
            v.kept_end = on;
            // A height animation already running keeps the edge it started with.
            if !v.height_animating() {
                v.anim_end = on;
            }
        });
    }

    /// A title column is collapsing or opening (its width is moving).
    pub fn column_rolling(&self) -> bool {
        self.view
            .borrow()
            .as_ref()
            .is_some_and(|v| v.column_rolling())
    }

    /// Rolled up and still: no hover peek, no roll or height animation in flight.
    pub fn rolled_at_rest(&self) -> bool {
        self.view
            .borrow()
            .as_ref()
            .is_some_and(|v| v.rolled_at_rest())
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
        if changed {
            self.arrange_desktop(false);
        }
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

    /// Per-fence title on hover (None = follow the global setting).
    pub fn set_title_on_hover(&self, on: Option<bool>) {
        self.with_view(|v| {
            if v.title_on_hover_override != on {
                v.title_on_hover_override = on;
                let _ = v.redraw();
            }
        });
    }

    /// Per-fence colour wash, title colour and title size.
    pub fn set_style(&self, style: FenceStyle) {
        let hwnd = self.hwnd();
        self.with_view(|v| {
            if v.style != style {
                let previous_ink = v.material_foreground().text_primary;
                let moved = v.style.title_edge != style.title_edge
                    || v.style.side_tab_text != style.side_tab_text;
                v.style = style;
                if moved {
                    // The title bar moves to another edge: the tabs may change axis, and the
                    // content and the plate's fold move.
                    v.reset_tab_axis();
                    let _ = v.motion.clear_clip(&v.content_panel.visual);
                    let (w, h) = window::window_rect_size(hwnd);
                    if let Err(e) = v.layout_panels(w, h).and_then(|_| v.redraw()) {
                        tracing::error!(error = %e, "relayout for the title position failed");
                    }
                    v.update_shadow_now();
                } else if previous_ink != v.material_foreground().text_primary {
                    let _ = v.redraw();
                } else {
                    let _ = v.redraw_chrome_only();
                }
            }
        });
    }

    /// This fence's title alignment (None = the global setting).
    pub fn set_title_align(&self, align: Option<pecofence_core::TitleAlign>) {
        self.with_view(|v| {
            if v.title_align != align {
                v.title_align = align;
                let _ = v.redraw_chrome_only();
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
        self.arrange_desktop(false);
    }

    /// 图标对齐 (#154): a short row of icons stays at the left or moves to the middle.
    pub fn set_icon_align(&self, align: IconAlign) {
        self.with_view(|v| {
            if v.icon_align != align {
                v.icon_align = align;
                v.snap_item_motion();
                let _ = v.redraw_content();
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
        self.view
            .borrow()
            .as_ref()
            .filter(|v| !v.bare())
            .and_then(|v| v.auto_height_px())
    }

    /// 像桌面一样排列 (`Fence::desktop_layout`): the icons stand down the columns of `area`'s
    /// work area around the other fences, and the window covers just their cells. None = a
    /// normal fence again (it keeps its rect).
    pub fn set_desktop_area(&self, area: Option<DesktopArea>) {
        let mut style_changed = false;
        self.with_view(|v| {
            if v.desktop != area {
                style_changed = v.desktop.is_some() != area.is_some();
                v.desktop = area;
                if !v.bare() {
                    v.desktop_slots.clear();
                }
                if style_changed {
                    v.snap_chrome_fades();
                }
            }
        });
        if style_changed {
            let wanted = self.view.borrow().as_ref().map(|v| v.wanted_layout);
            if let Some(wanted) = wanted {
                self.set_layout(wanted);
            }
        }
        let on = self.is_desktop_arranged();
        anchor::set_desktop_fence(self.hwnd(), on);
        if style_changed && !on {
            self.with_view(|v| {
                v.desktop_shape_for = None;
                v.desktop_shape_pending = None;
            });
            window::set_window_region(self.hwnd(), None);
        }
        self.arrange_desktop(style_changed);
    }

    /// A rectangle starts on the empty desktop: with `additive` (Ctrl / Shift) the icons
    /// selected now stay selected while it grows.
    pub fn begin_desktop_band(&self, additive: bool) {
        self.with_view(|v| {
            v.desktop_band_base = Some(if additive {
                v.selected.clone()
            } else {
                HashSet::new()
            });
        });
    }

    /// The rectangle drawn on the desktop now covers `band` (screen px): the icons it touches
    /// are selected. Returns whether it touches any.
    pub fn band_desktop_items(&self, band: RECT) -> bool {
        let mut any = false;
        self.with_view(|v| any = v.band_desktop_items(band));
        any
    }

    pub fn end_desktop_band(&self) {
        self.with_view(|v| v.desktop_band_base = None);
    }

    /// 像桌面一样排列 is on for this window.
    pub fn is_desktop_arranged(&self) -> bool {
        self.view
            .try_borrow()
            .is_ok_and(|g| g.as_ref().is_some_and(|v| v.bare()))
    }

    /// Re-places the desktop cells (items added or removed, area changed) and fits the window
    /// to them; `force` relayouts even when nothing moved (the look changed).
    fn arrange_desktop(&self, force: bool) {
        let hwnd = self.hwnd();
        let mut placed = None;
        self.with_view(|v| placed = v.arrange_desktop());
        let mut relayout = force;
        if let Some((rect, changed)) = placed {
            relayout |= changed;
            if rect != window::window_rect(hwnd) {
                self.place_window(rect);
                relayout = true;
            }
            if relayout {
                self.with_view(|v| v.sync_desktop_shape(true));
            }
        }
        if relayout {
            self.with_view(|v| {
                let (w, h) = window::window_rect_size(hwnd);
                if let Err(e) = v.layout_panels(w, h).and_then(|_| v.redraw()) {
                    tracing::warn!(error = %e, "relayout for 像桌面一样排列 failed");
                }
                v.update_shadow();
            });
        }
    }

    /// Width of the title column in DIPs (0 without one).
    pub fn tab_rail_dip(&self) -> f32 {
        self.view.borrow().as_ref().map_or(0.0, |v| v.rail_w())
    }

    /// See `FenceViewState::take_rail_growth_px`: the height a fence grows to once so its left
    /// tab column shows every tab.
    pub fn take_rail_growth_px(&self) -> Option<i32> {
        self.view
            .borrow_mut()
            .as_mut()
            .and_then(|v| v.take_rail_growth_px())
    }

    /// Columns, rows and fitting height of the fence currently shown ([`Self::active_fence`]).
    pub fn fit_report(&self) -> Option<FitReport> {
        self.view.borrow().as_ref().map(|v| v.fit_report())
    }

    /// Applies a new expanded height. `animate` (auto-height following an item change) glides
    /// the bottom edge there over 250 ms; otherwise, or when the animation declines (hidden
    /// window, animations off, roll in progress), the window snaps. The returned rectangle is
    /// the final geometry either way, so the state persists it immediately. The borrow is
    /// released before SetWindowPos because WM_SIZE re-enters the handler.
    pub fn apply_height(&self, height_px: i32, animate: bool) -> RECT {
        if self.is_desktop_arranged() {
            self.arrange_desktop(false);
            return self.window.window_rect();
        }
        let (animated, upward) = {
            let mut guard = self.view.borrow_mut();
            match guard.as_mut() {
                Some(v) => {
                    v.expanded_h_px = height_px;
                    (animate && v.start_height_anim(height_px), v.row_at_bottom())
                }
                None => (false, false),
            }
        };
        let mut r = self.window.window_rect();
        r.top = top_for_height(upward, &r, height_px);
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
            let old_ink = v.material_foreground().text_primary;
            v.backdrops = backdrops;
            v.glass_dark_text.set(None);
            v.invalidate_backdrop();
            // The content only follows the backdrop through its ink (as in `on_move`).
            let result = if old_ink != v.material_foreground().text_primary
                || (v.theme.liquid_glass && v.row_metrics().is_some())
            {
                v.redraw()
            } else {
                v.redraw_chrome_only()
            };
            if let Err(error) = result {
                tracing::warn!(%error, "redraw after wallpaper change failed");
            }
        });
    }

    /// Renders and presents the shadow again even at an unchanged rect (its layered window
    /// holds the only copy of the pixels, which a display change may drop).
    pub fn repaint_shadow(&self) {
        self.with_view(|v| {
            v.shadow.invalidate();
            v.update_shadow();
        });
    }

    /// A press on the desktop or in another fence: the items lose their selection, as icons
    /// on the Windows desktop do.
    pub fn end_item_selection(&self) {
        self.with_view(|v| {
            if v.end_selection() {
                let _ = v.redraw_content();
            }
        });
    }

    /// Marks the window as part of the desktop selection: a white ring just outside it (like
    /// the drop outline), drawn with its shadow.
    pub fn set_group_selected(&self, on: bool) {
        self.with_view(|v| {
            v.shadow.set_ring(on.then(|| selection_ring(&v.theme)));
            v.update_shadow();
            if v.is_clear() {
                // The ring lives in the shadow, which a fully transparent plate hides at rest:
                // a selected fence shows its plate.
                let _ = v.redraw_chrome_only();
            }
        });
    }

    /// The selection ring is on (test dumps).
    pub fn group_selected(&self) -> bool {
        self.view
            .try_borrow()
            .ok()
            .and_then(|g| g.as_ref().map(|v| v.shadow.ring().is_some()))
            .unwrap_or(false)
    }

    pub fn set_theme(&self, theme: Theme, backdrops: Rc<BackdropSets>, shadow: ShadowStyle) {
        let hwnd = self.hwnd();
        apply_window_shape(hwnd);
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
            // A theme may bring its own title font: the tabs are measured again.
            v.tab_natural_w.borrow_mut().clear();
            if !theme.liquid_glass {
                v.gpu_glass.borrow_mut().reset();
            }
            v.backdrops = backdrops;
            v.shadow.set_style(shadow);
            if v.shadow.ring().is_some() {
                v.shadow.set_ring(Some(selection_ring(&theme)));
            }
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

    /// The icon cache has new pictures for these draw keys (`IconCache::take_replaced`):
    /// items showing one of them, or its failure placeholder, take the new one.
    pub fn reload_icons(&self, replaced: &HashSet<String>) {
        self.with_view(|v| {
            let px = (v.icon_dip() * v.scale()).round() as u32;
            let mut any = false;
            {
                let icons = v.icons.borrow();
                for item in &mut v.items {
                    if replaced.contains(&icons.draw_key(&item.icon_key, px)) {
                        item.icon_px = 0;
                        item.icon_failed = false;
                        any = true;
                    }
                }
            }
            // A removed item still fading out holds the old picture and would put it back into
            // the GPU cache under the same key: its fade ends now.
            let leaving = v.leaving.len();
            v.leaving.retain(|l| !replaced.contains(&l.icon_key));
            any |= v.leaving.len() != leaving;
            if any && !(v.rolled_up && v.roll_anim.is_none()) {
                let _ = v.redraw_content();
            }
        });
    }

    /// Redraws only if icons are still missing (used after the icon worker delivers results).
    pub fn redraw_if_pending_icons(&self) {
        self.with_view(|v| {
            // Icons live on the content surface only, hidden while rolled up (as in `redraw`).
            if v.has_pending_icons() && !(v.rolled_up && v.roll_anim.is_none()) {
                let _ = v.redraw_content();
            }
        });
    }
}

/// The desktop selection ring, concentric with the fence's corners.
fn selection_ring(theme: &Theme) -> crate::shadow::SelectionRing {
    crate::shadow::SelectionRing {
        radius_dip: theme.corner_radius,
    }
}

/// The rolled title bar of a window at `r`: the window itself rolled up at rest, else the bar
/// at the edges a hover peek or a roll in flight keeps, where the window rolls back to.
fn rolled_bar_in(v: &FenceViewState, r: RECT) -> RECT {
    if v.rolled_at_rest() {
        return r;
    }
    let (cur_w, cur_h) = (r.right - r.left, r.bottom - r.top);
    let w = if v.collapses_width() {
        v.rolled_w_px().min(cur_w)
    } else {
        cur_w
    };
    let left = super::anchored_start(r.left, cur_w, w, v.width_anchor());
    let bar = RECT {
        left,
        right: left + w,
        ..r
    };
    let h = v.rolled_h_px_at(&bar).min(cur_h);
    let top = super::anchored_start(r.top, cur_h, h, v.height_anchor());
    RECT {
        top,
        bottom: top + h,
        ..bar
    }
}
