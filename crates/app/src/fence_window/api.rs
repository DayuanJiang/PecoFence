//! `FenceWindow`: the HWND-owning handle and its public API (create, set_items, set_tabs, show / hide, …).

use super::*;

pub struct FenceWindow {
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
            rename_after_fade: None,
            unfold_cache: None,
            strip_kept: false,
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
            search_scroll: HashMap::new(),
            drop_wash: Tween::at(0.0, Instant::now()),
            caret_alpha: Tween::at(0.0, Instant::now()),
            caret_shown: None,
            merge_hint: false,
            merge_t: Tween::at(0.0, Instant::now()),
            item_motion: HashMap::new(),
            leaving: Vec::new(),
            fade_next_refill: false,
            drop_landing: None,
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
            note: None,
            note_edit: None,
            note_drag: false,
            note_boxes: Vec::new(),
            clock: None,
            clock_text: (0.0, 0.0),
            clock_text_logged: (0.0, 0.0),
            clock_date: "long",
            clock_time: "full",
            tool_text_h: 0.0,
            tool_line_tops: Vec::new(),
            tool_line_step: 0.0,
            text_size: 1,
            script: None,
            script_rows: Vec::new(),
            script_hover: None,
            script_fades: Fades::new(motion::FASTER, motion::FASTER, Curve::Linear),
            script_footer_form: "full",
            note_box_hover: None,
            portal_missing: false,
            empty_override: None,
            empty_action: None,
            empty_action_hover: false,
            title_count: None,
            count_tip: None,
            filter_active: false,
            filter_tag: None,
            refuses_drops: false,
            label_w: 0.0,
            label_wide_w: 0.0,
            icons_pending: false,
            hover: None,
            title_hover: false,
            selected: HashSet::new(),
            scroll_y: 0.0,
            scroll_memory: HashMap::new(),
            scroll_restore: None,
            select_after_nav: None,
            user_delete: None,
            scroll_anim: None,
            dpi,
            theme,
            glass_dark_text: Cell::new(None),
            plate_halo: Cell::new(false),
            bare_dark_text: Cell::new(None),
            content_look: Cell::new(None),
            backdrops: ctx.backdrops.borrow().clone(),
            backdrop_crop: None,
            backdrop_rect: None,
            backdrop_stale: false,
            backdrop_origin: None,
            backdrop_key: format!("backdrop:{:x}", hwnd.0 as isize),
            gpu_glass: RefCell::new(pecofence_render::gpu_glass::GpuGlass::default()),
            chrome_panel,
            content_panel,
            content_x: 0,
            content_y: (theme.title_height * dpi as f32 / 96.0).round() as i32,
            rail_dip: 0.0,
            expanded_w_px: bounds.right - bounds.left,
            column_collapsed: false,
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
            tip_move_at: None,
            cut: HashSet::new(),
            wheel_zoom_accum: 0,
            wheel_tab_accum: 0,
            tab_hover_switch: None,
            auto_height,
            rail_grow: false,
            locked: false,
            lock_press: None,
            lock_pulse: None,
            expand_upward,
            row_bottom,
            anim_upward: row_bottom,
            backdrop_override: None,
            opacity: 1.0,
            desktop: None,
            desktop_slots: Vec::new(),
            desktop_slots_moved: false,
            desktop_free: None,
            desktop_free_sent: None,
            desktop_band_base: None,
            desktop_shape_for: None,
            desktop_shape_pending: None,
            wanted_layout: ViewLayout::Icons,
            title_on_hover_override: None,
            style: FenceStyle::default(),
            spacing: 0,
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
            peek_faded: false,
            fade_trace: FadeTrace::default(),
            retired: false,
            off_desktop: false,
            desktop_tip: None,
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
            shadow_timer: false,
            // Until measured: the old fixed interval.
            shadow_cost: Duration::from_millis(SHADOW_MIN_INTERVAL_MS as u64),
            shadow_alpha: Tween::at(1.0, Instant::now()),
            shadow_fading: false,
            title_reveal: Tween::at(1.0, Instant::now()),
            plate_reveal: Tween::at(1.0, Instant::now()),
            plate_inset_px: 0,
            plate_alpha: 1.0,
            edge_top: false,
            peeking: false,
            hover_raised: false,
            peek_blocked: false,
            capped_h_px: None,
            glide: None,
            defer_place: false,
            pending_place: None,
            placing: false,
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
                path_drag: false,
                shortcut_hover: None,
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

    /// Ends a fade-in at once: the window (and its shadow) stand at full strength now. For a
    /// window the user already holds (a tab torn off by a drag).
    pub fn show_at_once(&self) {
        self.with_view(|v| {
            if v.vis_fade == Some(VisFade::Showing) {
                v.vis_fade = None;
                v.reset_root();
                tracing::debug!(title = %v.title, "fade in ended at once: the window is in the hand");
            }
        });
    }

    /// 在哪个虚拟桌面显示: `tip` names the desktops of a window that shows on some of them only
    /// (the Task View glyph before its title, the text its infotip shows); `None` for one on
    /// every desktop.
    pub fn set_desktop_mark(&self, tip: Option<String>) {
        self.with_view(|v| {
            if v.desktop_tip == tip {
                return;
            }
            v.deco.desktop_icon = tip.is_some();
            v.desktop_tip = tip;
            let _ = v.redraw_chrome_only();
        });
    }

    /// The fence belongs (or no longer belongs) to other virtual desktops than the current one:
    /// off, it fades out as for quick hide and nothing shows it until this is cleared; the
    /// caller shows it again when it is back (unless quick hide holds it). Returns whether the
    /// flag changed.
    pub fn set_off_desktop(&self, off: bool) -> bool {
        let hwnd = self.hwnd();
        let changed = match self.view.borrow_mut().as_mut() {
            Some(v) if v.off_desktop != off => {
                v.off_desktop = off;
                true
            }
            _ => false,
        };
        if let Ok(mut guard) = self.anchor.try_borrow_mut()
            && let Some(a) = guard.as_mut()
        {
            a.set_off_desktop(hwnd, off);
        }
        if changed && off {
            hide_with_fade(&self.view, hwnd, false, false);
        }
        changed
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
        // During a display change's pass: where the window is about to glide to.
        if let Ok(guard) = self.view.try_borrow()
            && let Some(r) = guard.as_ref().and_then(|v| v.pending_place)
        {
            return r;
        }
        self.window.window_rect()
    }

    /// A display change places every fence in one pass and reads the others' rects while it
    /// does. With `on`, a placement that only moves this window (same size, in sight,
    /// animations on) is kept instead of carried out, and `rect` answers with it; `on` false
    /// ends that, and the window glides from where it stands to the place it was given
    /// (250 ms), so the fences travel to their new places together instead of jumping.
    pub fn defer_moves(&self, on: bool) {
        let pending = self.view.borrow_mut().as_mut().and_then(|v| {
            v.defer_place = on;
            v.pending_place.take()
        });
        if let Some(to) = pending
            && !self.start_glide(to, motion::NORMAL, Curve::PointToPoint, false)
        {
            self.place_window(to);
        }
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
            "title={:?} title_count={:?} filtered={} opacity={:.2} scale={:.3} fade={fade} retired={} rolled={} rolling={} peeking={} items={} tabs={} selected={} focused={:?} shadow_visible={} shadow_alpha={} layout={:?} locked={} auto_height={} active={} scroll={:.1} client={:?} client_offset=({},{}) chrome={:?} glass={:?} wallpaper_uploads={} title_size={} tab_fonts={:?} tab_rects={:?} content_x={} content={:?} capture={} tab_drag={} window_drag={} detach_pending={} plate_inset={} plate_alpha={:.2} mouse_inside={} content_y={} text_size={} clock_text={}/{} note_boxes={} script_rows={} off_desktop={} desktop_tip={:?} filter_tag={:?} badge={} rolled_count={} tag_fit={:?} peek_faded={} clock_date={} clock_time={}",
            v.title,
            v.title_count,
            v.filter_active,
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
            v.content_y,
            v.text_size,
            v.clock_text.0,
            v.clock_text.1,
            dump_rects(&v.note_boxes),
            dump_rects(&v.script_rows),
            v.off_desktop,
            v.desktop_tip,
            v.filter_tag,
            // 智能栅栏's folder glyph with the magnifier badge is drawn.
            v.deco.search_glyph && v.deco.folder_icon && !v.deco.up_button,
            // The rolled title's 「N 项」 is drawn (while rolled).
            v.rolled_up && v.shows_rolled_count(),
            // What of the filter tag fits (its box in DIPs, the text, the count, the chip).
            v.filter_tag_fit(),
            // The Peek is fading this fence with its dimmer.
            v.peek_faded,
            v.clock_date,
            v.clock_time,
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
            path_drag: false,
            shortcut_hover: None,
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
            // The hover look of the item at client point (x, y) without a real cursor, kept
            // until the cursor next passes over the fence (x < 0 takes it off). A "move"
            // asks for WM_MOUSELEAVE, which comes at once while the cursor is elsewhere.
            "hover-item" => self.with_view(|v| {
                let hit = if x < 0 { None } else { v.hit_item(x, y) };
                if v.set_hover(hit, false).0 {
                    let _ = v.redraw_content();
                }
            }),
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
        let cur = self.window.window_rect();
        let hwnd = self.hwnd();
        let deferred = self.view.borrow_mut().as_mut().is_some_and(|v| {
            v.height_anim = None;
            v.glide = None;
            let moves_only = (cur.left, cur.top) != (r.left, r.top)
                && cur.right - cur.left == r.right - r.left
                && cur.bottom - cur.top == r.bottom - r.top;
            let defer = v.defer_place
                && moves_only
                && v.motion.enabled()
                && !v.retired
                && desktop::is_visible(hwnd);
            v.pending_place = defer.then_some(r);
            v.placing = !defer;
            defer
        });
        if deferred {
            return;
        }
        let _ = self
            .window
            .set_bounds(r.left, r.top, r.right - r.left, r.bottom - r.top);
        self.with_view(|v| v.placing = false);
    }

    /// A tab split/undo supplies complete geometry. Settle any hover expansion first
    /// so its old tween cannot pull the detached or restored window back on later frames.
    pub fn restore_geometry(&self, mut rect: RECT, rolled: bool, expanded_h: i32) {
        // 像桌面一样排列 places the window itself (a display change or a restore lands here).
        if self.is_desktop_arranged() {
            self.arrange_desktop(true);
            return;
        }
        // `rect` is in the pixels of the monitor it lies on: the view takes that monitor's
        // scale first, so the title row and the sizes below are measured for it (a move onto a
        // monitor with another scale then changes nothing more).
        let monitor = monitors::monitor_from_point(
            (rect.left + rect.right) / 2,
            (rect.top + rect.bottom) / 2,
        );
        let target_dpi = monitors::query(monitor).map(|m| m.dpi);
        let hwnd = self.hwnd();
        self.with_view(|v| {
            if let Some(dpi) = target_dpi {
                v.apply_dpi(dpi);
            }
            v.finish_content_swap_now();
            v.roll_anim = None;
            v.height_anim = None;
            v.peeking = false;
            // A hover peek ended here, not by its roll-up: back among the other fences too.
            if std::mem::take(&mut v.hover_raised) {
                v.queue.push(Command::EndHoverRaise(hwnd));
            }
            v.rolled_up = rolled;
            v.expanded_h_px = expanded_h;
            v.capped_h_px = None;
            v.expanded_w_px = rect.right - rect.left;
            v.column_collapsed = rolled && v.tabs_on_rail();
            v.rail_dip = v.measure_rail_w();
            v.roll_t = Tween::at(rolled as u8 as f32, Instant::now());
            let _ = v.motion.stop(&v.content_panel.visual, Prop::Opacity);
            v.content_panel.visual.set_opacity(1.0);
            v.content_panel.visual.set_visible(!rolled);
            if rolled {
                // `rect` is the expanded rect: the title row sits at its top, or at its bottom
                // for a fence that last opened upward.
                let rolled_h = v.rolled_h_px();
                if v.row_bottom {
                    rect.top = rect.bottom - rolled_h;
                } else {
                    rect.bottom = rect.top + rolled_h;
                }
                if v.column_collapsed {
                    rect.right = rect.left + v.rolled_w_px();
                }
            } else {
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

    /// Where the icons of a drop just made on this window were let go (`drop_landing`), taken
    /// out: a drop whose icons arrive later (their files are on the way to the desktop) hands
    /// each place back with `set_drop_landing` when its icon comes in.
    pub fn take_drop_landing(&self) -> HashMap<ItemId, (f32, f32)> {
        let mut places = HashMap::new();
        self.with_view(|v| {
            if let Some((_, from)) = v.drop_landing.take() {
                places = from;
            }
        });
        places
    }

    /// The icon `id`, about to be listed, was let go at `from` (centre, unscrolled content
    /// DIPs): the list refresh that brings it in starts it there (`replace_items`).
    pub fn set_drop_landing(&self, id: ItemId, from: (f32, f32)) {
        self.with_view(|v| {
            let now = Instant::now();
            match v.drop_landing.as_mut() {
                Some((at, places)) => {
                    *at = now;
                    places.insert(id, from);
                }
                None => v.drop_landing = Some((now, HashMap::from([(id, from)]))),
            }
        });
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
                // (the redraw clamps to its content; `set_grid` consumes the value).
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

    /// The filter box changed this window's filter: if the `set_items` that follows replaces
    /// most of the list (the first letter typed, the filter cleared), the old page fades out
    /// under the new one instead of being cut. A change that keeps most items glides as usual.
    pub fn fade_next_refill(&self) {
        self.with_view(|v| v.fade_next_refill = true);
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
                // The title names the folder shown: its name and glyph fade in while the
                // icons cross-fade (a single title, or the shown tab's caption: the tab keeps
                // its pill as it is).
                let now = Instant::now();
                v.chrome_fades.snap(ChromeKey::TitleSwap, 1.0, now);
                v.chrome_fades.set(true, ChromeKey::TitleSwap, 0.0, now);
                v.frames.request();
                tracing::debug!(
                    dir,
                    tabs = v.tabs.len(),
                    "folder navigation: the title fades in with the icons"
                );
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
                // Its hover and selection fills stay as they are (no snap off and fade back in).
                v.item_hover.rekey(&old, new);
                v.item_sel.rekey(&old, new);
                v.drop_fades.rekey(&old, new);
                v.search_dim.rekey(&old, new);
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

    /// Glides the scroll so `item` is in view, as the arrow keys do: the user's own rename may
    /// have moved it in a sorted fence, and the selection must not sit on an icon nobody sees.
    /// Nothing when the item is not shown here, is in view already, or the fence is rolled up.
    pub fn reveal_item(&self, item: ItemId) {
        self.with_view(|v| {
            if v.rolled_up {
                return;
            }
            let Some(i) = v.items.iter().position(|it| it.id == item) else {
                return;
            };
            let before = v.scroll_target();
            v.scroll_into_view(i, true);
            tracing::debug!(
                index = i,
                from = before,
                to = v.scroll_target(),
                "renamed item kept in view"
            );
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

    /// The user deletes these items (Delete key or the item menu): when the focused one goes
    /// with them, the item after it takes the focus and the selection, as in Explorer. The
    /// focus ring of a note the Delete key just left for them (and maybe for more: the app
    /// passes only what can be deleted) stays.
    pub fn expect_user_delete(&self, items: &[ItemId]) {
        self.with_view(|v| {
            let ids: HashSet<ItemId> = items.iter().copied().collect();
            let ring = v
                .user_delete
                .as_ref()
                .is_some_and(|d| d.ring && ids.is_subset(&d.ids));
            v.user_delete = Some(UserDelete::new(ids, ring));
        });
    }

    /// The app says how the user's Delete went. `gone`: files were deleted, their icons leave
    /// with the next refresh, and the note's wait counts from now (Windows' question may have
    /// been open for a while). Else the delete was refused, cancelled in Windows' question or
    /// failed, or nothing of it can be deleted: the note is dropped, so an icon that leaves
    /// the fence another way later does not hand the selection to its neighbour.
    pub fn settle_user_delete(&self, gone: bool) {
        self.with_view(|v| {
            if !gone {
                v.forget_user_delete("the delete was refused, cancelled or failed");
            } else if let Some(d) = v.user_delete.as_mut() {
                d.since = Instant::now();
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
            if v.search.is_none() {
                // The search is over: every tab it scrolled goes back to where it was, unless
                // the user scrolled it meanwhile (that place stays). The app has put back the
                // tab that was on show before, so a tab the search switched to is not on show
                // any more and its place lives in `scroll_memory`.
                for (tab, (from, put)) in std::mem::take(&mut v.search_scroll) {
                    let shown = tab == v.active;
                    let offset = if shown {
                        Some(v.scroll_target())
                    } else {
                        v.scroll_memory.get(&tab).copied()
                    };
                    // None: the tab left this window during the search.
                    let Some(offset) = offset else {
                        continue;
                    };
                    let rest = after_search_offset(offset, from, put);
                    let hidden = v.rolled_up && v.roll_anim.is_none();
                    tracing::debug!(
                        fence = %tab,
                        from,
                        put,
                        offset,
                        rest,
                        shown,
                        hidden,
                        "peek search ended: a place it scrolled goes back to where it was, or keeps the user's place"
                    );
                    if rest == offset {
                        continue;
                    }
                    if !shown {
                        v.scroll_memory.insert(tab, rest);
                    } else if hidden {
                        // Rolled up again (further typing left it without a match): nothing
                        // to glide, and `scroll_to` leaves a hidden offset alone. Written
                        // back, so the fence opens where it was before the search.
                        v.scroll_anim = None;
                        v.scroll_y = rest.clamp(0.0, v.scroll_max());
                    } else {
                        v.scroll_to(rest, motion::SLOW);
                    }
                }
            }
            if v.rolled_up {
                return;
            }
            let current = current.and_then(|id| v.items.iter().position(|it| it.id == id));
            if let Some(i) = current {
                let before = v.scroll_target();
                v.scroll_into_view(i, true);
                let put = v.scroll_target();
                if put != before {
                    // The offset to go back to is the one from before the search's first
                    // scroll of this tab.
                    let from = v
                        .search_scroll
                        .get(&v.active)
                        .map_or(before, |&(from, _)| from);
                    v.search_scroll.insert(v.active, (from, put));
                }
            }
            v.tip_keyboard_focus(current);
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

    /// A fence just made: its name box opens when the fade-in running now has brought it to
    /// full strength (the window then asks with `Command::BeginFenceRename`), so the box does
    /// not float fully opaque on a fence that is still nearly transparent. That is after the
    /// 167 ms of the fade (`TIMER_NAME_BOX`); the entrance's scale runs on to 250 ms and the
    /// compositor reports the whole of it later still (about 0.3 s after the click), which is
    /// when the box used to open: keys typed until then went nowhere. False when no fade-in
    /// runs (animations off, a batch the compositor refused): the caller opens the box at once.
    pub fn rename_after_fade_in(&self) -> bool {
        let mut guard = self.view.borrow_mut();
        let Some(v) = guard.as_mut() else {
            return false;
        };
        let fading = v.vis_fade == Some(VisFade::Showing);
        v.rename_after_fade = fading.then(Instant::now);
        if fading {
            window::set_timer(v.hwnd, TIMER_NAME_BOX, motion::FAST.as_millis() as u32);
        }
        fading
    }

    /// Whether the user sees the start of the title row, where the name box opens: the window
    /// is shown on this virtual desktop, no hide is under way (a fade-in may still run: the
    /// name box of a new fence opens at its end), and what lies at that spot of
    /// the screen is this window (a window that covers the fence, as after a Peek has ended,
    /// would get the topmost box on top of itself).
    pub fn title_on_screen(&self) -> bool {
        let hwnd = self.hwnd();
        let (resting, scale) = match self.view.borrow().as_ref() {
            Some(v) => (
                !v.retired && !v.off_desktop && !matches!(v.vis_fade, Some(VisFade::Hiding { .. })),
                v.scale(),
            ),
            None => return false,
        };
        if !resting || !desktop::is_visible(hwnd) {
            return false;
        }
        let r = window::window_rect(hwnd);
        // Inside the title row, where the name starts.
        let (x, y) = (
            r.left + (24.0 * scale) as i32,
            r.top + (16.0 * scale) as i32,
        );
        desktop::window_from_point(x, y) == hwnd
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
                        && a.desktop_icon == b.desktop_icon
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
            let old_widths: Vec<f32> = v.tab_rects().iter().map(|(_, w)| *w).collect();
            let had_strip = v.tabs.len() > 1;
            // A new window gets its tabs here for the first time: what it was saved with.
            let first = v.tabs.is_empty();
            let rail_fit = v.rail_fit_height_px();
            let strip_scroll = v.tab_scroll.get();
            v.tabs = tabs;
            v.active = active;
            // A left tab column comes with the second tab and goes with the last but one.
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
                    // The same tabs in the same order and a pill got wider (a rename, a folder
                    // tab that went into a folder with a longer name): the pill is drawn at
                    // its new width at once, so the neighbour that slid away from it lay over
                    // the end of the new name for the first frames. The strip then stands at
                    // its new places at once; pills that only got narrower still slide.
                    let grew = same_set
                        && old_ids.iter().eq(v.tabs.iter().map(|t| &t.id))
                        && v.tab_rects()
                            .iter()
                            .zip(&old_widths)
                            .any(|((_, w), old)| *w > *old + 0.5);
                    if grew {
                        tracing::debug!("a tab got wider: the strip stands at its new places");
                        v.tab_slide.clear();
                    }
                } else if !same_set {
                    v.tab_slide.clear();
                    v.tab_settle = None;
                }
            }
            if !same_set {
                // Per-id state of tabs that left goes with them, their hover fades too.
                let ids: Vec<FenceId> = v.tabs.iter().map(|t| t.id).collect();
                v.tab_slide.retain(|(id, _)| ids.contains(id));
                if v.tab_settle.is_some_and(|(id, _)| !ids.contains(&id)) {
                    v.tab_settle = None;
                }
                v.chrome_fades.prune(now, |k| match k {
                    ChromeKey::Tab(id) | ChromeKey::TabDrop(id) => ids.contains(id),
                    _ => true,
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

    /// 标签栏位置 changed: the tabs move between the title row and the left column.
    pub fn sync_tab_bar(&self) {
        self.with_view(|v| {
            // A tab drag or merge gap measured along the old axis ends here (only the CLI can
            // change the setting mid-drag); the dragged tab goes back to its slot and the button
            // release then finds nothing to commit.
            if let Some(td) = v.tab_drag.take()
                && td.index < v.tabs.len()
            {
                let t = v.tabs.remove(td.index);
                v.tabs.insert(td.start_index.min(v.tabs.len()), t);
            }
            v.merge_gap = None;
            v.pressed = None;
            v.tab_slide.clear();
            v.tab_settle = None;
            v.pill_anim = None;
            v.sync_rail();
            v.rail_grow = v.tabs_on_rail();
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

    #[allow(clippy::too_many_arguments)]
    pub fn set_portal_deco(
        &self,
        is_portal: bool,
        folder_icon: bool,
        search_glyph: bool,
        up_button: bool,
        navigate_folders: bool,
        folder_missing: bool,
        portal_folder: Option<PathBuf>,
    ) {
        self.with_view(|v| {
            let deco = TitleDeco {
                folder_icon,
                up_button,
                search_glyph,
                desktop_icon: v.deco.desktop_icon,
                locked: v.deco.locked,
            };
            let changed = v.deco != deco || v.navigate_folders != navigate_folders;
            let portal_changed = v.is_portal != is_portal || v.portal_missing != folder_missing;
            v.deco = deco;
            v.navigate_folders = navigate_folders;
            v.portal_folder = portal_folder;
            v.is_portal = is_portal;
            v.portal_missing = folder_missing;
            if !up_button && !v.glyph_opens_folder() {
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

    /// What the fence says while it shows nothing (`empty`: a smart fence searching, finding
    /// nothing or needing Everything, a filter matching nothing; None = the usual text) and a
    /// smart fence's count in its capsule at the right of the title row (`count`, "6"; None
    /// for any other fence). `action` is the label of a clickable row under the text (a smart
    /// fence that needs Everything: 「启动 Everything」); a click on it arrives as
    /// `Command::EmptyActionClicked`.
    pub fn set_status_view(
        &self,
        empty: Option<&'static str>,
        action: Option<&'static str>,
        count: Option<String>,
    ) {
        self.with_view(|v| {
            if v.empty_override == empty && v.empty_action == action && v.title_count == count {
                return;
            }
            v.empty_override = empty;
            v.empty_action = action;
            v.empty_action_hover = false;
            v.title_count = count;
            let _ = v.redraw_chrome_only();
            if v.items.is_empty() && !v.rolled_up {
                let _ = v.redraw_content();
            }
        });
    }

    /// The infotip of a smart fence's count capsule (None: the capsule has none). A count such
    /// as 「200+」 did not say what the plus stands for.
    pub fn set_count_tip(&self, tip: Option<String>) {
        self.with_view(|v| {
            if v.count_tip != tip {
                tracing::debug!(title = %v.title, tip = tip.is_some(), "count capsule: infotip set");
                v.count_tip = tip;
            }
        });
    }

    /// 筛选: whether the fence is filtered (Esc in it then clears the filter).
    pub fn set_filtered(&self, active: bool) {
        self.with_view(|v| v.filter_active = active);
    }

    /// 筛选: the tag in the title row (`tag`: the type, the text and the count; None: no tag).
    pub fn set_filter_tag(&self, tag: Option<(String, String, String)>) {
        self.with_view(|v| {
            if v.filter_tag != tag {
                v.filter_tag = tag;
                let _ = v.redraw_chrome_only();
            }
        });
    }

    /// 智能栅栏: the fence shown takes no drops (it shows a search).
    pub fn set_refuses_drops(&self, on: bool) {
        self.with_view(|v| v.refuses_drops = on);
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

    /// 便签与待办: the fence being shown is a note with `text`, whose lines are drawn in place of
    /// items; None = it shows items.
    pub fn set_note(&self, text: Option<String>) {
        self.with_view(|v| {
            if v.note != text {
                v.note = text;
                v.note_boxes.clear();
                let _ = v.redraw();
            }
        });
    }

    /// 便签与待办: the note shown is being edited: its text, selection and caret are drawn from
    /// `edit` in place of the stored note, and the line with the caret is scrolled into
    /// view. None = the editor closed.
    pub fn set_note_edit(&self, edit: Option<NoteEditView>) {
        self.with_view(|v| {
            if v.note_edit == edit {
                return;
            }
            let moved = match (&v.note_edit, &edit) {
                (Some(old), Some(new)) => old.text != new.text || old.active != new.active,
                (None, Some(_)) => true,
                _ => false,
            };
            v.note_edit = edit;
            if v.note_edit.is_none() {
                v.note_drag = false;
            }
            if moved {
                v.note_caret_into_view();
            }
            let _ = v.redraw_content();
        });
    }

    /// The caret of the note being edited here shows or hides (its blink).
    pub fn set_note_caret_on(&self, on: bool) {
        self.with_view(|v| {
            if let Some(e) = v.note_edit.as_mut()
                && e.caret_on != on
            {
                e.caret_on = on;
                let _ = v.redraw_content();
            }
        });
    }

    /// The caret of the note being edited here, in screen px: its x, its top and its height.
    pub fn note_caret_screen(&self) -> Option<(i32, i32, i32)> {
        let r = self.rect();
        let view = self.view.borrow();
        let v = view.as_ref()?;
        let (x, top, h) = v.note_caret_dip()?;
        let scale = v.scale();
        Some((
            r.left + v.content_x + (x * scale).round() as i32,
            r.top + v.content_y + ((top - v.scroll_y) * scale).round() as i32,
            (h * scale).round() as i32,
        ))
    }

    /// The place in the text one line on screen above (`dir` < 0) or below the caret of the
    /// note being edited here.
    pub fn note_offset_vertical(&self, dir: i8) -> Option<usize> {
        self.view.borrow().as_ref()?.note_offset_vertical(dir)
    }

    /// A script's copy action uses a short, non-activating tip beside the pointer.
    pub fn show_script_copy_result(&self, index: usize, copied: bool) {
        let text = if copied {
            pecofence_core::i18n::text("已复制")
        } else {
            pecofence_core::i18n::text("复制失败，请重试")
        };
        self.show_script_row_tip(index, text, 2000);
    }

    /// A click on a script row that would start a program, in a script whose header does not
    /// allow that: the tip beside the pointer says so and names the line to add.
    pub fn show_script_row_refused(&self, index: usize) {
        // The line to add goes on a line of its own, so the tip does not break it in two.
        let text = pecofence_core::i18n::format(
            "这个脚本不能启动程序。要允许，在脚本开头的注释里加上：{0}",
            &["\n<pecofence.allow>run</pecofence.allow>".to_string()],
        );
        // Long enough to read both sentences.
        let secs = pecofence_platform::sysparams::message_duration_secs().max(8);
        self.show_script_row_tip(index, &text, secs * 1000);
    }

    /// `text` in a non-activating tip beside the pointer for `ms`, as the result of a click
    /// on script row `index`.
    fn show_script_row_tip(&self, index: usize, text: &str, ms: u32) {
        self.with_view(|v| {
            v.hide_tip();
            v.tip_target = Some(TipTarget::ScriptRow(index));
            let (x, y, above) = pointer_tip_place(v.scale());
            v.show_tip_text_at(text, x, y, above);
            window::set_timer(v.hwnd, TIMER_TIP_HIDE, ms);
        });
    }

    /// 时钟与日期: the fence being shown is a clock (Some(true) = with seconds), drawn in place of
    /// items and redrawn each minute or second; None = it shows items.
    pub fn set_clock(&self, clock: Option<bool>) {
        self.with_view(|v| {
            if v.clock != clock {
                v.clock = clock;
                v.arm_clock();
                let _ = v.redraw();
            }
        });
    }

    /// 时钟与日期: the system time, the time zone or the region's time format changed. A clock
    /// shown here draws the new time now and ticks at the new boundaries; true when it did.
    /// These broadcasts are sent, not posted: one that arrives while the view is in use
    /// leaves the clock to its own next tick.
    pub fn clock_changed(&self) -> bool {
        let Ok(mut guard) = self.view.try_borrow_mut() else {
            return false;
        };
        let Some(v) = guard.as_mut().filter(|v| v.clock.is_some()) else {
            return false;
        };
        v.arm_clock();
        let shows = !v.rolled_up && !v.retired && pecofence_platform::desktop::is_visible(v.hwnd);
        if shows {
            let _ = v.redraw_content();
        }
        shows
    }

    /// 脚本栅栏: the fence being shown is a script fence showing `script`; None = it shows items.
    pub fn set_script(&self, script: Option<ScriptView>) {
        self.with_view(|v| {
            if v.script != script {
                if script.is_none() {
                    v.script_rows.clear();
                    v.script_hover = None;
                }
                // The first result takes the place of 「正在运行…」 with the content's
                // cross-fade (83 ms, as a change of view), not in one frame.
                let first_result = v.script.as_ref().is_some_and(|s| s.quiet)
                    && script.as_ref().is_some_and(|s| !s.quiet);
                v.script = script;
                if first_result && v.content_swap_shows() {
                    tracing::debug!(fence = %v.active, "script fence: the first result fades in");
                    v.begin_content_swap(0);
                }
                let _ = v.redraw();
            }
        });
    }

    /// 文字大小 of the note or script fence shown (0 to 3, 1 = normal).
    pub fn set_text_size(&self, size: u8) {
        self.with_view(|v| {
            if v.text_size != size {
                v.text_size = size;
                if v.note.is_some() || v.script.is_some() {
                    let _ = v.redraw_content();
                }
            }
        });
    }

    /// Screen rect of the content below the title row (the note editor opens over it).
    pub fn content_screen_rect(&self) -> RECT {
        let mut r = self.rect();
        if let Some(v) = self.view.borrow().as_ref() {
            r.left += v.content_x;
            r.top += v.content_y;
        }
        r
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

    /// Icon size, label lines, spacing, alignment and layout (the icon grid or the List /
    /// Details rows): lays the items out again at once.
    pub fn set_grid(&self, grid: GridSpec) {
        let mut changed = false;
        self.with_view(|v| changed = v.apply_grid(grid));
        // 像桌面一样排列: other cells.
        if changed {
            self.arrange_desktop(false);
        }
    }

    /// Tab switch, after `set_tabs` and before the incoming tab's items arrive: takes that
    /// tab's grid without drawing the outgoing items again. The new items are then drawn, and
    /// their icons extracted, at their own size and in their own cells from the first frame.
    /// The icon cache keeps an inactive tab's icons at that size only.
    pub fn set_incoming_tab_view(&self, grid: GridSpec) {
        self.with_view(|v| {
            if v.grid() != grid {
                v.take_grid(grid);
                v.snap_item_motion();
            }
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
            v.regroup(on);
            let _ = v.redraw_content();
        });
    }

    /// Rolled up as it rests: also while hover-peeking open (the peek closes by itself) and
    /// while a roll-up is still in flight (`rolled_up` is only set as it lands, so a toggle, a
    /// move or a save in those 167 ms read it as rolled up, not as open at a passing height).
    pub fn is_rolled(&self) -> bool {
        self.view
            .borrow()
            .as_ref()
            .is_some_and(|v| v.roll_target() || v.peeking)
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
                    out.insert(item.draw_key(icons));
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
            // Rolled up under the pointer (a click on the chevron or the title, the menu, the
            // CLI): the window shrinks to its title row there, and Windows then reports the
            // resting pointer as a move on it. That must not start a hover peek.
            if rolled {
                v.peek_blocked = v.pointer_in_rect();
                tracing::debug!(
                    fence = %v.fence_id,
                    blocked = v.peek_blocked,
                    "rolled up on request: hover peek waits until the pointer has left"
                );
            }
            v.start_roll_anim(rolled);
        });
    }

    /// The window as it rests: a rolled fence that is hover-peeking or rolling up counts as its
    /// title row, and one in a roll or height animation at the height it ends at (see
    /// `resting_heights_px`). A peek committed open counts at its full height at once.
    pub fn resting_rect(&self) -> RECT {
        let mut r = self.rect();
        let rest = self.view.try_borrow().ok().and_then(|g| {
            g.as_ref().map(|v| {
                let h = if v.roll_target() || v.peeking {
                    v.rolled_h_px()
                } else {
                    v.height_target_px().unwrap_or(r.bottom - r.top)
                };
                // The edge a height animation keeps, else the title row's.
                let upward = if v.height_animating() {
                    v.anim_upward
                } else {
                    v.row_bottom
                };
                (h, upward)
            })
        });
        if let Some((h, upward)) = rest
            && h > 0
            && h != r.bottom - r.top
        {
            r.top = top_for_height(upward, &r, h);
            r.bottom = r.top + h;
        }
        r
    }

    /// A roll-up or expand is in flight.
    pub fn rolling(&self) -> bool {
        self.view
            .try_borrow()
            .ok()
            .is_some_and(|g| g.as_ref().is_some_and(|v| v.roll_anim.is_some()))
    }

    /// The height a roll or height animation in flight ends at, and whether it keeps the
    /// bottom edge (None when none runs).
    pub fn animation_end_height_px(&self) -> Option<(i32, bool)> {
        let guard = self.view.try_borrow().ok()?;
        let v = guard.as_ref()?;
        v.height_target_px().map(|h| (h, v.anim_upward))
    }

    /// How far the fence reaches beyond `resting_rect` once it opens, to the right and below:
    /// a rolled-up fence whose title row is its top edge (no 向上展开) opens downward to its
    /// expanded height, and one collapsed to its left tab column also to its expanded width.
    /// Placement keeps that whole room inside the work area (`clearance::landing_spot`).
    pub fn open_hang_px(&self) -> (i32, i32) {
        let rest = self.resting_rect();
        self.view
            .borrow()
            .as_ref()
            .map_or((0, 0), |v| v.open_hang_px(&rest))
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

    /// Screen rect of tab `tab`'s row in the left tab column (None without a column): where the
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
        Some(RECT {
            left: win.left + at(RAIL_ROW_INSET),
            top: win.top + at(y),
            right: win.left + at(v.rail_w() - RAIL_ROW_RIGHT),
            bottom: win.top + at(y + h),
        })
    }

    /// Window rect, as wide as the fence expands to while it is collapsed to its left tab column:
    /// the room automatic placement keeps clear.
    pub fn footprint_rect(&self) -> RECT {
        let mut r = self.rect();
        if let Some(ew) = self.collapsed_expanded_width_px() {
            r.right = r.left + ew;
        }
        r
    }

    /// Width the window expands to while it is collapsed to its left tab column (None
    /// otherwise: the window has its expanded width).
    pub fn collapsed_expanded_width_px(&self) -> Option<i32> {
        let guard = self.view.try_borrow().ok()?;
        let v = guard.as_ref()?;
        v.column_collapsed.then_some(v.expanded_w_px)
    }

    /// A rolled fence whose tabs changed (added, removed, renamed, or 标签栏位置 switched)
    /// takes its rolled size again: the left tab column it is collapsed to, or its title row at
    /// the expanded width. Returns the new window rect when it changed.
    pub fn settle_rolled_size(&self) -> Option<RECT> {
        let (w, h, upward) = {
            let mut guard = self.view.borrow_mut();
            let v = guard.as_mut()?;
            if !v.rolled_up || v.peeking || v.roll_anim.is_some() || v.retired {
                return None;
            }
            let (cur_w, _) = window::window_rect_size(v.hwnd);
            let w = if v.tabs_on_rail() {
                v.rail_dip = v.measure_rail_w();
                v.rail_px()
            } else if v.column_collapsed {
                v.expanded_w_px
            } else {
                cur_w
            };
            v.column_collapsed = v.tabs_on_rail();
            (w, v.rolled_h_px(), v.row_bottom)
        };
        let r = self.rect();
        if (r.right - r.left, r.bottom - r.top) == (w, h) {
            return None;
        }
        // No borrow held: WM_SIZE lays the panels out at the new size.
        let _ = self
            .window
            .set_bounds(r.left, top_for_height(upward, &r, h), w, h);
        self.with_view(|v| v.update_shadow_now());
        Some(window::window_rect(self.hwnd()))
    }

    /// Windows a title or tear-off drag driven by this window moves right now (the dragged one
    /// and the selection moving along), and this window while it glides to the spot it was
    /// dropped for (`glide_to`: not at rest yet); empty when none runs.
    pub fn dragged_windows(&self) -> Vec<HWND> {
        self.view
            .try_borrow()
            .ok()
            .and_then(|g| {
                let v = g.as_ref()?;
                if v.glide.is_some() {
                    return Some(vec![v.hwnd]);
                }
                v.remote_drag.as_ref().map(|rd| {
                    std::iter::once(rd.hwnd)
                        .chain(rd.group.iter().map(|m| m.hwnd))
                        .collect()
                })
            })
            .unwrap_or_default()
    }

    /// Moves the window to `to` with a glide from where it is now (167 ms, decelerate): a
    /// fence let go a little over another one arrives at its free spot instead of jumping
    /// there. False when the caller should place it at once: animations off, a hidden window,
    /// another size, or 2 px or less to go.
    pub fn glide_to(&self, to: RECT) -> bool {
        self.start_glide(to, motion::FAST, Curve::Decelerate, true)
    }

    /// `glide_to` with its time and curve; `notify`: the app hears the bounds on arrival.
    fn start_glide(&self, to: RECT, time: Duration, curve: Curve, notify: bool) -> bool {
        let hwnd = self.hwnd();
        let r = self.window.window_rect();
        let same_size =
            r.right - r.left == to.right - to.left && r.bottom - r.top == to.bottom - to.top;
        let far = (to.left - r.left).abs() > 2 || (to.top - r.top).abs() > 2;
        self.view.borrow_mut().as_mut().is_some_and(|v| {
            let glide =
                same_size && far && v.motion.enabled() && !v.retired && desktop::is_visible(hwnd);
            tracing::debug!(
                fence = %v.fence_id,
                from = ?(r.left, r.top),
                to = ?(to.left, to.top),
                same_size,
                glide,
                by_user = notify,
                "fence goes to its landing place"
            );
            if glide {
                v.glide = Some(WindowGlide {
                    from: (r.left, r.top),
                    to: (to.left, to.top),
                    t: v.motion.tween(0.0, 1.0, time, curve, Instant::now()),
                    begun: false,
                    notify,
                });
                v.frames.request();
            }
            glide
        })
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

    /// 锁定位置和大小 on or off. A locked fence keeps the lock glyph's slot before the chevron
    /// free (the glyph shows while the pointer is over the fence), so the title row is laid out
    /// again.
    pub fn set_locked(&self, on: bool) {
        self.with_view(|v| {
            if v.locked == on {
                return;
            }
            v.locked = on;
            v.deco.locked = on;
            v.lock_press = None;
            v.lock_pulse = None;
            let _ = v.redraw_chrome_only();
        });
    }

    /// 向上展开 on or off. Nothing moves: the next expansion picks its side by the new setting.
    /// Returns true when the fence is rolled up at rest, so the caller records its expanded
    /// rect again around the title row (switched off, below it). An open fence or one in a roll
    /// keeps the edge it has until it is rolled up.
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
        if at_rest && !on {
            v.row_bottom = false;
        }
        at_rest
    }

    /// The title row is at the bottom: the window keeps its bottom edge as its height changes.
    pub fn row_at_bottom(&self) -> bool {
        self.view.borrow().as_ref().is_some_and(|v| v.row_bottom)
    }

    /// Which end of the saved rect a rolled title row sits at (`AppState::set_fence_bounds`
    /// chooses it).
    pub fn set_row_at_bottom(&self, on: bool) {
        self.with_view(|v| v.row_bottom = on);
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
                // The text colour is picked afresh for the new plate. The choice it keeps
                // near its threshold is for a plate that moves over the wallpaper: carried
                // over a change of 不透明度, a note in dark mode kept the white text of
                // 更厚实 on the light plate of 默认 or 全透明.
                v.forget_text_choice("opacity");
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
        self.with_view(|v| {
            if v.style != style {
                let previous_ink = v.material_foreground().label_look();
                // A new 色调 changes the plate too: the text colour is picked afresh.
                if v.style.tint != style.tint || v.style.tint_title_only != style.tint_title_only {
                    v.forget_text_choice("tint");
                }
                v.style = style;
                if previous_ink != v.material_foreground().label_look() {
                    let _ = v.redraw();
                } else {
                    let _ = v.redraw_chrome_only();
                }
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
                item.icon_stand_in = None;
            }
            let _ = v.redraw();
        });
    }

    /// Height the window should have when auto-height is on (None when off / rolled).
    pub fn auto_height_px(&self) -> Option<i32> {
        self.view
            .borrow()
            .as_ref()
            .filter(|v| !v.bare() && !v.shows_tool())
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
                // 自由摆放: cells placed from the new saved ones are sent again.
                v.desktop_free_sent = None;
                if !v.bare() {
                    v.desktop_slots.clear();
                }
                if style_changed {
                    v.snap_chrome_fades();
                }
            }
        });
        if style_changed {
            // Icons while it is on; the fence's own view comes back when it goes off.
            self.with_view(|v| {
                v.apply_grid(v.grid());
            });
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

    /// The rectangle is done; returns how many icons are selected now.
    pub fn end_desktop_band(&self) -> usize {
        let mut selected = 0;
        self.with_view(|v| {
            v.desktop_band_base = None;
            selected = v.selected.len();
        });
        selected
    }

    /// 自由摆放: the grid cells icons dropped at `at` take on this 「桌面」 (see
    /// `FenceViewState::desktop_drop_cells`); None when it does not place its icons freely.
    pub fn desktop_drop_cells(&self, at: &DesktopDrop) -> Option<Vec<(ItemId, [u16; 2])>> {
        self.view
            .borrow()
            .as_ref()
            .and_then(|v| v.desktop_drop_cells(at.cursor, &at.centres))
    }

    /// 自由摆放: see `FenceViewState::desktop_free_changes`.
    pub fn desktop_free_changes(
        &self,
    ) -> Option<(BTreeMap<ItemId, [u16; 2]>, Vec<(ItemId, [u16; 2])>)> {
        self.view
            .borrow()
            .as_ref()
            .and_then(|v| v.desktop_free_changes())
    }

    /// 像桌面一样排列 is on for this window.
    pub fn is_desktop_arranged(&self) -> bool {
        self.view
            .try_borrow()
            .is_ok_and(|g| g.as_ref().is_some_and(|v| v.bare()))
    }

    /// 自由摆放: icons of another fence were let go at `at` on the bare desktop; they are about
    /// to join this 「桌面」 and arrive from where they were let go
    /// (`FenceViewState::note_landing_at`).
    pub fn note_desktop_landing(&self, at: &DesktopDrop) {
        self.with_view(|v| v.note_landing_at(&at.centres));
    }

    /// 自由摆放: icons of this 「桌面」 were moved by hand and let go at `at`; their new cells are
    /// placed. They settle into them from where they were let go
    /// (`FenceViewState::land_moved_icons`).
    pub fn land_moved_icons(&self, at: &DesktopDrop) {
        self.with_view(|v| {
            if v.land_moved_icons(&at.centres) {
                v.sync_desktop_shape(true);
                let _ = v.redraw_content();
            }
        });
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
            let old = window::window_rect(hwnd);
            if rect != old {
                if (rect.left, rect.top) != (old.left, old.top) {
                    // The cells are measured from the window's corner: motion in flight was
                    // computed for the old one.
                    self.with_view(|v| v.snap_item_motion());
                }
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

    /// Width of the left tab column in DIPs (0 without one).
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
                    v.capped_h_px = None;
                    (animate && v.start_height_anim(height_px), v.row_bottom)
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

    /// The full height (device px) of a fence that is open only as far as the fences below
    /// leave room (`capped_h_px`); None when the window is as tall as the fence.
    pub fn capped_full_height_px(&self) -> Option<i32> {
        let guard = self.view.try_borrow().ok()?;
        let v = guard.as_ref()?;
        v.capped_h_px
            .map(|_| v.expanded_h_px.max(v.title_h_px() + 40))
    }

    /// `rollUp.pushNeighbors`: keeps an open fence of a set height inside `room_px`, the room
    /// below its top edge that the fences it pushes leave it (`app/push.rs`). One taller than
    /// that gets shorter (never under its title row plus 40 px; the content scrolls), and one
    /// held short this way gets taller again as the room grows, up to its full height. The
    /// height it is saved with stays. Rolled, animating and auto-height fences are left alone
    /// (auto height fits the room by itself), and so is a hover peek, which opens in full
    /// above the fences it cannot push. True when the window was resized without an
    /// animation. The borrow is released before SetWindowPos because WM_SIZE re-enters the
    /// handler.
    pub fn fit_open_to_room(&self, room_px: i32) -> bool {
        if self.is_desktop_arranged() {
            return false;
        }
        let (target, animated) = {
            let mut guard = self.view.borrow_mut();
            let Some(v) = guard.as_mut() else {
                return false;
            };
            if v.retired
                || v.auto_height
                || v.rolled_up
                || v.peeking
                || v.row_bottom
                || v.height_animating()
            {
                return false;
            }
            let floor = v.title_h_px() + 40;
            let full = v.expanded_h_px.max(floor);
            let target = full.min(room_px.max(floor));
            let cur = window::window_rect_size(v.hwnd).1;
            let capped = (target < full).then_some(target);
            if cur == target {
                v.capped_h_px = capped;
                return false;
            }
            // Only a fence over its room gets shorter, and only one held short gets taller.
            let shrink = cur > target && capped.is_some();
            let grow = cur < target && v.capped_h_px.is_some();
            if !shrink && !grow {
                return false;
            }
            tracing::debug!(
                fence = %v.fence_id,
                from = cur,
                to = target,
                full,
                room = room_px,
                "open fence fitted to the room the fences below leave"
            );
            v.capped_h_px = capped;
            (target, v.start_height_anim(target))
        };
        if animated {
            return false;
        }
        let mut r = self.window.window_rect();
        r.bottom = r.top + target;
        self.set_bounds(r);
        true
    }

    /// Room on each side of `item`'s name inside its cell as the name is drawn now (icons
    /// layout): the smaller one when it was fitted to the wider box (`ItemView::label_wide`).
    /// The rename box takes the same, so the name wraps in it as it does under the icon.
    pub fn item_label_inset(&self, item: ItemId) -> f32 {
        let wide = self.view.borrow().as_ref().is_some_and(|v| {
            v.items
                .iter()
                .any(|it| it.id == item && it.label.is_some() && it.label_wide)
        });
        label_inset(wide)
    }

    /// The colour this fence's names and title are drawn in now, for the name box that takes
    /// a name's place: the text picked for the plate behind the fence, or 「文字颜色」 白色 /
    /// 黑色. Icons arranged like the desktop have no plate (white names on the wallpaper, as
    /// Windows draws them): their box keeps the colour mode's text, like Explorer's.
    pub fn label_ink(&self, mode: pecofence_render::ThemeMode) -> ColorF {
        let picked = self.view.borrow().as_ref().and_then(|v| {
            (!v.bare()).then(|| {
                v.material_foreground()
                    .with_label_color(v.behavior.label_color.get())
                    .text_primary
            })
        });
        picked.unwrap_or(Theme::for_mode(mode).text_primary)
    }

    /// The title of `fence` as it is drawn, for its name box (`rename::begin_inline_rename`):
    /// where its text starts, its row's height, its size and its font. A lone title (centred
    /// or right-aligned: with the room it is aligned in, which its box takes) or a tab in the
    /// title row (its caption is centred in its pill, after its glyphs), or a row of the left
    /// tab column (`x` then counts from the row's left edge, `rail_row_screen_rect`).
    pub fn title_text_place(&self, fence: FenceId) -> Option<crate::rename::TitlePlace> {
        const SIZES: [f32; 4] = [12.0, 14.0, 16.0, 18.0];
        let guard = self.view.borrow();
        let v = guard.as_ref()?;
        if v.bare() {
            return None;
        }
        let (face, weight) = v.chrome.title_font();
        let px = |dip: f32| (dip * v.scale()).round() as i32;
        if v.tabs_on_rail() {
            // A row of the left tab column, as drawn: the caption after the colour dot and
            // the glyphs any tab has, semibold for the shown tab of an expanded fence.
            let i = v.tabs.iter().position(|t| t.id == fence)?;
            let (_, h) = *v.tab_rects().get(i)?;
            let x = pecofence_render::fence_chrome::rail_text_left(
                v.tabs.iter().any(|t| t.color.is_some()),
                v.tabs.iter().any(|t| t.folder_icon),
                v.tabs.iter().any(|t| t.desktop_icon),
            );
            let shown = fence == v.active && !v.column_collapsed;
            return Some(crate::rename::TitlePlace {
                x: px(x),
                row_h: px(h),
                font_dip: SIZES[usize::from(v.tabs[i].title_size.min(3))],
                face,
                weight: if shown { weight } else { 400 },
                aligned: None,
            });
        }
        let align = v.behavior.title_align.get();
        let (x, size, aligned) = if v.tabs.len() > 1 {
            let i = v.tabs.iter().position(|t| t.id == fence)?;
            let t = &v.tabs[i];
            let (x, w) = *v.tab_rects().get(i)?;
            let size = t.title_size.min(3);
            // As drawn: centred in the pill, after a glyph (a portal's folder, Task View).
            let dx = v.chrome.tab_caption_text_x(
                &v.theme,
                w,
                &t.title,
                size,
                (t.folder_icon, t.desktop_icon),
                v.tab_badge(t.id).as_deref(),
            );
            (x + dx, size, None)
        } else if align == pecofence_core::TitleAlign::Left {
            (v.deco.title_x(), v.style.title_size.min(3), None)
        } else {
            let (x, end, (from, to)) = v.lone_title_text_span();
            let aligned = Some((align, px(end), (px(from), px(to))));
            (x, v.style.title_size.min(3), aligned)
        };
        Some(crate::rename::TitlePlace {
            x: px(x),
            row_h: v.title_h_px(),
            font_dip: SIZES[usize::from(size)],
            face,
            weight,
            aligned,
        })
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
                item.icon_stand_in = None;
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
            let old_ink = v.material_foreground().label_look();
            v.backdrops = backdrops;
            v.glass_dark_text.set(None);
            v.bare_dark_text.set(None);
            v.plate_halo.set(false);
            v.invalidate_backdrop();
            // The content only follows the backdrop through its ink (as in `on_move`).
            let result = if old_ink != v.material_foreground().label_look()
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

    /// Tab / Shift+Tab can move the keyboard here: the window is shown, not rolled up, and has
    /// icons to put the focus on (a tool fence or an empty fence has none).
    pub fn takes_keyboard(&self) -> bool {
        desktop::is_visible(self.hwnd())
            && self.view.try_borrow().ok().is_some_and(|g| {
                g.as_ref()
                    .is_some_and(|v| !v.retired && !v.rolled_up && !v.items.is_empty())
            })
    }

    /// The keyboard comes here from another fence (Tab / Shift+Tab) or from the Peek's first
    /// key: the focused icon shows its focus ring, and with nothing selected the first icon is
    /// selected, as in Explorer's file list. Then the window takes the keyboard (during a Peek
    /// the dimmer passes the keys on to it). Call with no borrow of the view held.
    pub fn focus_by_keyboard(&self) {
        self.with_view(|v| {
            if v.items.is_empty() {
                return;
            }
            let focused = v
                .anchor_index
                .filter(|a| v.selected.contains(a))
                .or_else(|| v.selected.iter().min().copied());
            match focused {
                Some(i) => {
                    v.anchor_index = Some(i);
                    v.focus_visible = true;
                    v.scroll_into_view(i, true);
                }
                None => v.cursor_to(0, CursorMode::Select),
            }
            let _ = v.redraw_content();
        });
        handler::focus_session(&self.view, self.hwnd());
    }

    /// The window takes the keyboard as a click on it would, with the selection left as it is
    /// (its title's name box closed, or the fence beside it was deleted from its menu). Returns
    /// whether it is the foreground window now. Call with no borrow of the view held.
    pub fn take_keyboard(&self) -> bool {
        handler::focus_session(&self.view, self.hwnd());
        desktop::foreground_window() == self.hwnd()
    }

    /// The icons selected now, in the order shown.
    pub fn selected_items(&self) -> Vec<ItemId> {
        self.view
            .borrow()
            .as_ref()
            .map(|v| v.selected_ids())
            .unwrap_or_default()
    }

    /// Selects exactly the icons of `items` that are shown (what a paste just put here, as
    /// Explorer selects what it pasted): the first of them becomes the keyboard cursor and is
    /// scrolled into view. Returns the ones selected; none (and the selection as it was) when
    /// none of them is shown, or while a name box is open here (the scroll would close it).
    pub fn select_items(&self, items: &[ItemId]) -> Vec<ItemId> {
        let mut selected = Vec::new();
        self.with_view(|v| {
            if v.renaming.is_some() {
                return;
            }
            let found: Vec<usize> = v
                .items
                .iter()
                .enumerate()
                .filter(|(_, it)| items.contains(&it.id))
                .map(|(i, _)| i)
                .collect();
            let Some(&first) = found.first() else {
                return;
            };
            selected = found.iter().map(|i| v.items[*i].id).collect();
            v.selected = found.into_iter().collect();
            v.anchor_index = Some(first);
            v.range_anchor = Some(first);
            if !v.rolled_up {
                v.scroll_into_view(first, true);
                let _ = v.redraw_content();
            }
        });
        selected
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
                v.bare_dark_text.set(None);
                v.plate_halo.set(false);
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

    /// Peek began or ended (`FenceBehavior::floating` changed): redraws this fence if floating
    /// changes how it looks. Returns whether it did.
    pub fn redraw_for_floating(&self) -> bool {
        let mut changes = false;
        self.with_view(|v| {
            changes = v.floating_changes_look();
            if changes {
                let _ = v.redraw();
            }
        });
        changes
    }

    /// Peek: another program's window hides this fence, so it is made transparent before the
    /// Peek raises it (see `FenceViewState::peek_fade_prepare`). False: it shows at once.
    pub fn peek_fade_prepare(&self) -> bool {
        let mut taken = false;
        self.with_view(|v| taken = v.peek_fade_prepare());
        taken
    }

    /// Peek: fades this fence in, if `peek_fade_prepare` or `peek_fade_out` took it.
    pub fn peek_fade_in(&self) {
        self.with_view(|v| {
            v.peek_fade_to(1.0, false);
        });
    }

    /// The Peek ends and this fence goes back under a window: it fades out with the dimmer.
    /// False: it stays as it is (animations off, or a show / hide fade is running).
    pub fn peek_fade_out(&self) -> bool {
        let mut fading = false;
        self.with_view(|v| fading = v.peek_fade_to(0.0, true));
        fading
    }

    /// The Peek has this fence's opacity (`peek_fade_prepare`, `peek_fade_out`).
    pub fn peek_faded(&self) -> bool {
        self.view.borrow().as_ref().is_some_and(|v| v.peek_faded)
    }

    /// The Peek is over: a fence it faded is opaque again.
    pub fn peek_fade_rest(&self) {
        self.with_view(|v| v.peek_fade_rest());
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

/// Hit rects of a note's check boxes or a script's rows for `debug_state`, without spaces:
/// `index:left,top,right,bottom` (content DIPs) joined by `;`, `-` when there are none.
fn dump_rects(rects: &[(usize, [f32; 4])]) -> String {
    if rects.is_empty() {
        return "-".to_string();
    }
    rects
        .iter()
        .map(|(i, r)| format!("{i}:{:.1},{:.1},{:.1},{:.1}", r[0], r[1], r[2], r[3]))
        .collect::<Vec<_>>()
        .join(";")
}
