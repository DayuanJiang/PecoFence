//! `FenceViewState`: everything one fence window knows about itself, plus its smallest accessors.

use super::*;

pub struct FenceViewState {
    #[allow(dead_code)]
    pub fence_id: FenceId,
    pub title: String,
    pub rolled_up: bool,
    pub is_inbox: bool,
    /// Expanded window height in physical pixels (restored when un-rolling).
    pub expanded_h_px: i32,
    pub icon_size: u32,
    pub label_lines: u8,
    pub items: Vec<ItemView>,
    /// Tabs shown in the title row (the host itself first). One entry = plain title.
    pub(super) tabs: Vec<TabView>,
    /// The fence whose items are displayed (a tab id, or the host's own id).
    pub(super) active: FenceId,
    pub(super) tab_hover: Option<usize>,
    pub(super) tab_drag: Option<TabDrag>,
    /// Tab pills sliding to their slots after a reorder (drag past a neighbour, 左移 / 右移,
    /// cancel): 167 ms point-to-point, keyed by tab id so a strip refresh cannot desync them.
    pub(super) tab_slide: Vec<(FenceId, Tween)>,
    /// The just-released dragged pill settling from the pointer into its slot (167 ms).
    pub(super) tab_settle: Option<(FenceId, Tween)>,
    /// Slot a fence being dragged over this title would be inserted at (drag-to-merge onto a
    /// tabbed strip): the strip opens a `TAB_MERGE_GAP_W` gap there and the neighbours slide
    /// aside. None = no hint, or a plain (single-tab) title.
    pub(super) merge_gap: Option<usize>,
    /// Natural pill widths by caption, see `tab_natural_widths`.
    pub(super) tab_natural_w: RefCell<Vec<(String, u8, bool, f32)>>,
    /// Tab head under an OLE drag (highlighted like a folder drop target; not the hover).
    pub(super) drop_tab: Option<usize>,
    /// Drag dwell timer: the rolled-fence unroll is armed.
    pub(super) drag_peek_armed: bool,
    /// A right button press cancelled a drag: swallow the WM_RBUTTONUP that follows.
    pub(super) rbutton_swallow: bool,
    /// Item whose inline rename edit is open: its label stays folded under the edit box.
    pub(super) renaming: Option<ItemId>,
    /// The fence-title rename popup is open (a hover peek stays expanded under it).
    pub(super) title_renaming: bool,
    /// The anchor item's unfolded label, cached by (item, cell width, name) so a content redraw
    /// (hover fade, scroll glide, marquee) does not lay the text out again.
    pub(super) unfold_cache: Option<UnfoldCache>,
    /// A tab was just torn off; the App will hand us the new window to drag.
    pub(super) detach_pending: bool,
    pub(super) detach_cancelled: bool,
    pub(super) remote_drag: Option<RemoteDrag>,
    /// Portal title decorations (folder glyph, up button) for the fence being shown.
    pub(super) deco: TitleDeco,
    pub(super) up_hovered: bool,
    pub(super) chevron_hovered: bool,
    /// Control the mouse button is held on, and whether the pointer is still over it (Win32
    /// button semantics: the pressed fill hides when you drag off and returns when you come
    /// back; release outside cancels).
    pub(super) pressed: Option<PressTarget>,
    pub(super) press_inside: bool,
    /// Client point of the press (a chevron press dragged past the threshold becomes a title
    /// drag, so the right end of the title row still moves the fence).
    pub(super) press_origin: (i32, i32),
    /// Keyboard focus ring shown on the cursor item (after keyboard navigation, until a mouse
    /// click or focus loss).
    pub(super) focus_visible: bool,
    /// This fence (or its rename popup) is the active window: selection draws in the accent;
    /// otherwise in neutral grey (Explorer LISS_SELECTEDNOTFOCUS), cross-faded over 167 ms.
    pub(super) win_active: bool,
    pub(super) selection_inactive: Tween,
    /// 1 while a drag from elsewhere hovers the fence (own selection greys out so only the
    /// drop target reads accent), 83 ms both ways like the other drop feedback.
    pub(super) drop_inactive: Tween,
    /// Draw-side state fades (WinUI 83 ms brush transitions), synced from the logical
    /// hover / selection / header fields at draw time and sampled while painting. Keyed by
    /// item id so a refresh that moves an item (sort, reorder drop, watcher reconcile) carries
    /// its fade with it instead of leaving it on the index.
    pub(super) item_hover: Fades<ItemId>,
    pub(super) item_sel: Fades<ItemId>,
    pub(super) header_fades: Fades<HeaderColumn>,
    pub(super) chrome_fades: Fades<ChromeKey>,
    /// Drop-target fade per item (a folder under a drag), 83 ms both ways.
    pub(super) drop_fades: Fades<ItemId>,
    /// Fence-wide drop wash + ring alpha (83 ms in, 167 ms out).
    pub(super) drop_wash: Tween,
    /// Insertion caret alpha (83 ms) and the slot it is (still) drawn at while fading out.
    pub(super) caret_alpha: Tween,
    pub(super) caret_shown: Option<usize>,
    /// Drag-to-merge title highlight: logical state and its alpha (83 ms in, 167 ms out).
    pub(super) merge_hint: bool,
    pub(super) merge_t: Tween,
    /// Layout motion: items gliding to new cells / fading in, and removed items fading out.
    pub(super) item_motion: HashMap<ItemId, ItemMotion>,
    pub(super) leaving: Vec<LeavingItem>,
    /// A fade was still running when the surface was last drawn: `tick` redraws it.
    pub(super) content_busy: bool,
    pub(super) chrome_busy: bool,
    /// Mouse is somewhere over the window (title or content) — drives title-on-hover and the
    /// inactive-scrollbar rule.
    pub(super) mouse_inside: bool,
    /// Last scroll / scrollbar activity, for the inactive-scrollbar linger.
    pub(super) last_scroll: Option<Instant>,
    /// A rolled title was pressed with click-to-expand on; released without dragging = expand.
    pub(super) click_pending: Option<(i32, i32)>,
    /// The last title click expanded the fence (click-to-expand); a WM_NCLBUTTONDBLCLK that
    /// completes it must not roll the fence back up.
    pub(super) suppress_dblclk: bool,
    /// Double-click on a folder item navigates inside the portal instead of launching it.
    pub(super) navigate_folders: bool,
    /// Icons grid, compact list or details rows (FenceView.layout).
    pub(super) layout: ViewLayout,
    /// Current sort, for the details header indicator.
    pub(super) sort: SortMode,
    pub(super) sort_reverse: bool,
    pub(super) header_hover: Option<DetailColumn>,
    /// Details column widths (修改日期, 类型, 大小) in DIPs.
    pub(super) column_widths: [f32; 3],
    /// Details columns shown (header context menu).
    pub(super) columns_visible: [bool; 3],
    pub(super) col_drag: Option<ColDrag>,
    /// The fence shown is a folder portal (sort-only: no manual arrangement).
    pub(super) is_portal: bool,
    /// Width the cached labels were fitted for; a change invalidates them.
    pub(super) label_w: f32,
    pub(super) hover: Option<usize>,
    pub(super) title_hover: bool,
    pub(super) selected: HashSet<usize>,
    /// Scroll offset in DIPs as drawn and hit-tested; while `scroll_anim` runs it is the
    /// animated value (WinUI ScrollPresenter: wheel, keyboard and paging glide, thumb drags and
    /// auto-scroll write it directly).
    pub(super) scroll_y: f32,
    /// Last scroll offset per tab of this window (session only, like Explorer): switching
    /// away and back restores where that tab was.
    pub(super) scroll_memory: HashMap<FenceId, f32>,
    /// Offset to re-apply once the new tab's items have arrived (`set_items`); `set_layout`
    /// consumes it, so a tab switch whose tabs differ in layout keeps the restored offset.
    pub(super) scroll_restore: Option<f32>,
    /// Smooth-scroll client tween (the offset is rasterised into the D2D content surface, so
    /// the surface is redrawn per frame from `tick`). Its target is where pending input
    /// accumulates: a second wheel notch retargets from the anticipated end, not from the
    /// current frame.
    pub(super) scroll_anim: Option<Tween>,
    pub(super) dpi: u32,
    pub(super) theme: Theme,
    /// Last contrast choice for a clear plate; interior mutability keeps the draw API read-only.
    pub(super) glass_dark_text: Cell<Option<bool>>,
    pub(super) backdrops: Rc<BackdropSets>,
    pub(super) backdrop_crop: Option<Image>,
    /// Screen rect (left, top, width, height) `backdrop_crop` was cut for. While the height
    /// animates the crop covers the taller end, so the per-frame chrome draws reuse it.
    pub(super) backdrop_rect: Option<(i32, i32, i32, i32)>,
    /// `BitmapCache` key of the uploaded crop (per window; dropped whenever the crop changes).
    pub(super) backdrop_key: String,
    pub(super) gpu_glass: RefCell<pecofence_render::gpu_glass::GpuGlass>,
    pub(super) chrome_panel: Panel,
    pub(super) content_panel: Panel,
    pub(super) _target: DesktopWindowTarget,
    /// Root of the window's visual tree: whole-window fades / scales animate this.
    pub(super) root: ContainerVisual,
    pub(super) chrome: Rc<FenceChrome>,
    pub(super) icons: Rc<RefCell<IconCache>>,
    pub(super) bitmaps: Rc<RefCell<BitmapCache>>,
    #[allow(dead_code)]
    pub(super) queue: CommandQueue,
    pub(super) drag: Option<DragState>,
    /// An OLE drag started from this window is running: hover is suppressed, the selection keeps
    /// its normal look (Explorer leaves the source drawn as is; the shell drag image is the only
    /// feedback).
    pub(super) ole_drag: bool,
    /// Client-pixel point of the press that became the OLE drag: the drag image lifts off
    /// from there (the pointer has already moved past the drag threshold when the shell asks
    /// for the image, see `render_drag_image`).
    pub(super) ole_drag_press: (i32, i32),
    pub(super) marquee: Option<MarqueeState>,
    /// Folder item highlighted as the drop target of a running drag.
    pub(super) drop_item: Option<usize>,
    /// Insertion caret index while an internal drag reorders this fence.
    pub(super) drop_insert: Option<usize>,
    pub(super) scroll_drag: Option<ScrollDrag>,
    /// Track segment pressed and held: pages again at the keyboard-repeat cadence until the
    /// thumb reaches the pointer (TIMER_SCROLL_REPEAT).
    pub(super) page_repeat: Option<ScrollHit>,
    /// Pointer inside the scrollbar hot zone (the 12 DIP strip); the thumb widens 0.4 s later.
    pub(super) scrollbar_hot: bool,
    /// Thumb width 2..=6 DIP (167 ms decelerate after the expand / contract delays).
    pub(super) sb_width: Tween,
    /// Thumb opacity 0..=1 (83 ms linear; always 1 unless the inactive-scrollbar rule hides it).
    pub(super) sb_alpha: Tween,
    /// Track fill + arrow buttons reveal 0..=1 (83 ms linear with the expand / contract).
    pub(super) sb_parts: Tween,
    /// PointerOver fades of the two arrow buttons (83 ms), keyed by `ArrowUp` / `ArrowDown`.
    pub(super) sb_arrow_fades: Fades<ScrollHit>,
    /// Scrollbar part under the pointer as of the last mouse message (None = off the bar).
    pub(super) sb_pointer: Option<ScrollHit>,
    /// A scrollbar tween was started: `tick` redraws the content until both have settled.
    pub(super) sb_busy: bool,
    /// OLE drag edge auto-scroll (see `DragScroll`).
    pub(super) drag_scroll: Option<DragScroll>,
    /// Type-to-select prefix and when it was last extended.
    pub(super) type_ahead: String,
    pub(super) type_ahead_at: Option<Instant>,
    /// Slow-double-click rename armed for this item (TIMER_RENAME).
    pub(super) pending_rename: Option<ItemId>,
    pub(super) tip: Option<Tooltip>,
    pub(super) tip_target: Option<TipTarget>,
    pub(super) tip_shown: bool,
    /// When a visible infotip was last hidden (fast re-show while sweeping between items).
    pub(super) tip_hidden_at: Option<Instant>,
    /// Client px where TIMER_TIP was armed and whether it is armed: like TME_HOVER the tip
    /// shows only once the pointer has rested inside the SM_C[XY]MOUSEHOVER box for the delay.
    pub(super) tip_anchor: (i32, i32),
    pub(super) tip_armed: bool,
    /// Items on the clipboard via 剪切 (drawn at half alpha like Explorer).
    pub(super) cut: HashSet<ItemId>,
    /// Ctrl+wheel accumulator (touchpads deliver fractional notches).
    pub(super) wheel_zoom_accum: i32,
    /// Fence height follows the item grid (FenceView.auto_height).
    pub(super) auto_height: bool,
    /// Mouse move/resize disabled (menu "锁定位置和大小").
    pub(super) locked: bool,
    /// Per-fence material override; None follows the global setting.
    pub(super) backdrop_override: Option<BackdropMode>,
    /// Per-fence opacity multiplier for the glass layer (1.0 = default).
    pub(super) opacity: f32,
    /// Per-fence colour wash / title colour / title size.
    pub(super) style: FenceStyle,
    /// Icon grid spacing (FenceView.spacing).
    pub(super) spacing: Spacing,
    /// Keyboard/selection anchor: the last item clicked or moved to with the arrow keys.
    pub(super) anchor_index: Option<usize>,
    /// Start of a Shift range selection.
    pub(super) range_anchor: Option<usize>,
    /// A draw hit `DXGI_ERROR_DEVICE_REMOVED`; surfaces must be recreated (task 6).
    pub(super) device_lost: bool,
    pub(super) drop_hover: bool,
    pub(super) roll_anim: Option<RollAnim>,
    /// Roll progress 0 (expanded) ..= 1 (rolled), phase-locked with the roll's height tween
    /// (same duration and curve): the chevron rotates with it instead of flipping at the end.
    pub(super) roll_t: Tween,
    /// Auto-height settle (250 ms point-to-point): the bottom edge glides to the fitting
    /// height. Separate from `roll_anim` so it never blocks hover-peek or the chevron.
    pub(super) height_anim: Option<Tween>,
    /// Whole-window fade in progress and its generation (see `VisFade`).
    pub(super) vis_fade: Option<VisFade>,
    pub(super) vis_gen: usize,
    pub(super) vis_deadline: Option<Instant>,
    /// `FenceWindow::retire` was called: the window is fading out for good and lives only in
    /// `App::dying`. Nothing may show it again, and every hide must end in `FadeOutDone`.
    pub(super) retired: bool,
    /// Outgoing content panels of tab switches, still fading out under the new panel (a
    /// second switch mid-swap hands off from the current opacity instead of snapping, so more
    /// than one can be in flight); all parked in `spares` when `WM_APP_TAB_SWAP_DONE` arrives
    /// with the newest generation. At most two spares are kept.
    pub(super) outgoing: Vec<Panel>,
    pub(super) spares: Vec<Panel>,
    pub(super) swap_gen: usize,
    pub(super) swap_deadline: Option<Instant>,
    /// Active-tab pill gliding to the newly selected tab: (x, w) in DIPs, 167 ms decelerate.
    pub(super) pill_anim: Option<(Tween, Tween)>,
    pub(super) stack: Rc<RenderStack>,
    pub(super) motion: Rc<Motion>,
    pub(super) frames: Rc<FrameClock>,
    pub(super) hwnd: HWND,
    pub(super) shadow: ShadowWindow,
    /// When the shadow bitmap was last uploaded (throttle for interactive resizing).
    pub(super) shadow_uploaded: Option<Instant>,
    /// Constant alpha of the shadow window (0..=1), a client tween mirroring the root's
    /// whole-window fade: a layered HWND has no compositor property, so `tick` re-issues the
    /// cached shadow bitmap with the sampled alpha each frame while `shadow_fading`.
    pub(super) shadow_alpha: Tween,
    pub(super) shadow_fading: bool,
    /// Temporarily expanded by hovering a rolled fence (state stays "rolled" for persistence).
    pub(super) peeking: bool,
    pub(super) behavior: Rc<Behavior>,
}

impl FenceViewState {
    /// Posts `msg` with `wparam` to this window from the compositor's callback thread.
    pub(super) fn poster(&self, msg: u32, wparam: usize) -> impl Fn() + Send + 'static {
        let hwnd = self.hwnd.0 as isize;
        move || window::post_message(HWND(hwnd as *mut core::ffi::c_void), msg, wparam, 0)
    }

    pub(super) fn scale(&self) -> f32 {
        self.dpi as f32 / 96.0
    }

    /// Adopts a new DPI: rescales the remembered expanded height and drops DPI-dependent
    /// caches (icons, fitted labels). Returns true when something changed.
    pub(super) fn apply_dpi(&mut self, new_dpi: u32) -> bool {
        let new_dpi = new_dpi.max(96);
        if new_dpi == self.dpi {
            return false;
        }
        let old = self.dpi.max(96) as f32;
        tracing::info!(fence = %self.fence_id, from = self.dpi, to = new_dpi, "fence DPI changed");
        self.dpi = new_dpi;
        // The optical bezel and upload mask are measured in DIPs. A DPI change can keep
        // the same physical rectangle, so geometry alone cannot validate the old crop.
        self.backdrop_rect = None;
        self.expanded_h_px = (self.expanded_h_px as f32 * new_dpi as f32 / old).round() as i32;
        self.drop_icons_for_reload();
        self.snap_item_motion();
        // The infotip window scales its max width from the owner DPI at creation: rebuild it
        // on the next show.
        self.hide_tip();
        self.tip = None;
        true
    }

    /// Drops every cached icon and fitted label (they reload lazily, without a cross-fade).
    pub(super) fn drop_icons_for_reload(&mut self) {
        for item in &mut self.items {
            item.icon = None;
            item.icon_failed = false;
            item.icon_waited = false;
            item.icon_fade = None;
            item.label = None;
        }
        self.unfold_cache = None;
    }

    /// Ends the layout motion at once: the geometry it was computed for (cell size, layout,
    /// width, DPI) is gone, so old origins would glide to the wrong places.
    pub(super) fn snap_item_motion(&mut self) {
        self.item_motion.clear();
        self.leaving.clear();
    }

    pub(super) fn backdrop_mode(&self) -> BackdropMode {
        self.backdrop_override
            .unwrap_or_else(|| self.behavior.backdrop.get())
    }

    /// Zero when animations are off (client tweens then snap; the begin delays still apply,
    /// they are behaviour rather than motion).
    pub(super) fn anim_dur(&self, dur: Duration) -> Duration {
        if self.motion.enabled() {
            dur
        } else {
            Duration::ZERO
        }
    }

    pub(super) fn title_h_px(&self) -> i32 {
        (self.theme.title_height * self.scale()).round() as i32
    }

    /// Size of the content surface. Equal to the window minus the title bar after every
    /// `layout_panels`; during a height animation it stays at the expanded size (hit-testing,
    /// scrolling and redraws keep addressing the frozen surface).
    pub(super) fn content_size_px(&self) -> (i32, i32) {
        self.content_panel.size_px()
    }
}
