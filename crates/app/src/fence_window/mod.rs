//! A fence window: composition-backed, never activated, anchored above the desktop.
//!
//! Visual tree: root → chrome panel (full window: backdrop, title, stroke) + content panel
//! (below the title bar; its surface clips the icon grid naturally).

use crate::anchor::{self, AnchorCell};
use crate::commands::{
    Command, CommandQueue, DesktopDrop, TransferMode, WM_APP_DESKTOP_SHAPE, WM_APP_FADE_DONE,
    WM_APP_SET_VISIBLE, WM_APP_TAB_RAIL, WM_APP_TAB_SWAP_DONE,
};
use crate::icons::{IconCache, Lookup};
use crate::layout::{
    CellRect, DetailColumn, DetailColumns, FIT_SLACK_PX, Grid, GridMetrics, GridSpec, GroupSpan,
    ItemLayout, RowMetrics, group_spans,
};
use crate::shadow::{ShadowStyle, ShadowWindow};
use pecofence_core::{
    CivilDate, DateBucket, FenceId, IconAlign, IconKey, ItemId, ItemKey, SortMode, ViewLayout,
    date_bucket,
};
use pecofence_platform::dragdrop::{
    self as dragdrop, DragImage, DragPoint, DropEffect, DropHandler, DropImage,
    DropTargetRegistration, IDataObject,
};
use pecofence_platform::fileinfo;
use pecofence_platform::frameclock::FrameClock;
use pecofence_platform::tooltip::Tooltip;
use pecofence_platform::tray::PopupMenu;
use pecofence_platform::window::{
    self, MessageHandler, StandardCursor, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, RECT, desktop, dwm, hotkey, monitors, msg};
use pecofence_render::fence_chrome::{
    Backdrop, BackdropCrop, ContentDraw, EmptyAction, FenceChrome, FenceStyle, GroupHeaderDraw,
    HeaderColumn, ItemCell, RAIL_BOTTOM, RAIL_COLLAPSED_BOTTOM, RAIL_ROW_H, RAIL_ROW_INSET,
    RAIL_ROW_RIGHT, RAIL_TOP, RowCell, RowColumns, RowsDraw, ScrollbarDraw, StripClip, TabDraw,
    TitleDeco, TitleState,
};
use pecofence_render::motion::{self, Curve, Fades, Motion, Prop, Tween};
use pecofence_render::{
    BitmapCache, ColorF, ContainerVisual, DesktopWindowTarget, Image, Matrix3x2, MonitorBackdrop,
    Panel, Rect, RenderStack, Theme,
};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};
use windows_core::Result;

/// File operations of PecoFence's own that are running (a paste, a delete, a move into a
/// folder). The shell takes the activation from the fence that owns the operation and gives
/// it back about 0.3 s later: the selection turned grey for those frames after every Delete
/// and Ctrl+V (`on_activate` keeps its colour while this is above zero).
static OWN_FILE_OPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub fn own_file_op_started() {
    OWN_FILE_OPS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

pub fn own_file_op_ended() {
    use std::sync::atomic::Ordering::Relaxed;
    let _ = OWN_FILE_OPS.try_update(Relaxed, Relaxed, |n| n.checked_sub(1));
}

fn own_file_op_running() -> bool {
    OWN_FILE_OPS.load(std::sync::atomic::Ordering::Relaxed) > 0
}

mod api;
mod consts;
mod desktop_layout;
mod dnd;
mod frame;
mod group;
mod handler;
mod hit;
mod items;
mod note_text;
mod render;
mod roll;
mod scroll;
mod selection;
mod snap;
mod state;
mod tabs;
#[cfg(test)]
mod tests;
mod tooltip;
mod window_drag;

pub use api::FenceWindow;
pub use dnd::filter_folder_paths;
pub use frame::FrameHandle;
pub use state::{DesktopArea, WindowsIconPlaces};

use self::consts::*;
use self::dnd::*;
use self::group::*;
use self::hit::*;
use self::items::*;
use self::note_text::TodoDrag;
use self::render::*;
use self::roll::*;
use self::scroll::*;
use self::selection::*;
use self::snap::*;
use self::state::*;
use self::tabs::*;
use self::tooltip::*;
use self::window_drag::*;

/// Shared per-process context for all fence windows.
pub struct FenceContext {
    pub stack: Rc<RenderStack>,
    pub class: WindowClass,
    pub chrome: Rc<FenceChrome>,
    pub theme: RefCell<Theme>,
    pub backdrops: RefCell<Rc<BackdropSets>>,
    pub anchor: AnchorCell,
    pub taskbar_created: u32,
    pub icons: Rc<RefCell<IconCache>>,
    pub bitmaps: Rc<RefCell<BitmapCache>>,
    pub queue: CommandQueue,
    pub shadow_class: WindowClass,
    pub shadow_style: std::cell::Cell<ShadowStyle>,
    pub behavior: Rc<Behavior>,
    /// Compositor animations + Fluent constants (see `pecofence_render::motion`).
    pub motion: Rc<Motion>,
    /// Frame ticks for client-side tweens; call `request()` whenever a tween is running.
    pub frames: Rc<FrameClock>,
}

/// Live-tunable behaviour flags shared by all fence windows (mirrors `Settings`).
pub struct Behavior {
    /// Temporary readability backing while fences float over other applications.
    pub floating: std::cell::Cell<bool>,
    pub hover_peek: std::cell::Cell<bool>,
    pub snapping: std::cell::Cell<bool>,
    /// Gap kept to other fences and the work-area edges while snapping, in DIPs
    /// (`snapping.gapPx`).
    pub snap_gap_dip: std::cell::Cell<i32>,
    /// Resizing keeps whole icon columns and rows (`snapping.sizeToCells`).
    pub size_to_cells: std::cell::Cell<bool>,
    /// Alignment guides while dragging / resizing with snapping on (`snapping.guideLines`).
    pub guide_lines: std::cell::Cell<bool>,
    pub backdrop: std::cell::Cell<BackdropMode>,
    /// Rolled fences expand on a single title click (hover peek off while set).
    pub click_to_expand: std::cell::Cell<bool>,
    /// How long the pointer rests on a rolled fence before it expands, and on a tab before
    /// the hover switch (`rollUp.hoverSpeed`).
    pub hover_speed: std::cell::Cell<pecofence_core::HoverSpeed>,
    /// Title row drawn only while hovered; at rest the plate starts at the content.
    pub title_on_hover: std::cell::Cell<bool>,
    /// Title (or tab strip) position in the title row (`titleAlign`).
    pub title_align: std::cell::Cell<pecofence_core::TitleAlign>,
    /// Tabs in the title row or in a column on the left (`tabBar`).
    pub tab_bar: std::cell::Cell<pecofence_core::TabBar>,
    /// Item names in the backdrop's ink or always white / black (`labelColor`).
    pub label_color: std::cell::Cell<pecofence_core::LabelColor>,
    /// Scrollbar drawn only while hovered / shortly after scrolling.
    pub hide_inactive_scrollbar: std::cell::Cell<bool>,
    /// Resting the pointer on another tab switches to it (`rollUp.switchTabOnHover`).
    pub switch_tab_on_hover: std::cell::Cell<bool>,
    /// No rolled item count and no roll chevron (`rollUp.hideCountAndChevron`).
    pub hide_count_and_chevron: std::cell::Cell<bool>,
    /// `SPI_GETWHEELSCROLLLINES`: rows per wheel notch; `u32::MAX` (`WHEEL_PAGESCROLL`) = one
    /// viewport, 0 = the wheel does not scroll. Refreshed on WM_SETTINGCHANGE.
    pub wheel_lines: std::cell::Cell<u32>,
    /// Explorer's "Single-click to open an item (point to select)": resting the pointer on an
    /// item selects it and a click opens it, as on the desktop. Refreshed on WM_SETTINGCHANGE.
    pub single_click: std::cell::Cell<bool>,
    /// Explorer's "Show pop-up description for folder and desktop items": a resting pointer
    /// shows the item's infotip. Refreshed on WM_SETTINGCHANGE.
    pub info_tips: std::cell::Cell<bool>,
    /// Outline of the spot a fence dragged over another one will move to on release, or of
    /// the fence it would join as a tab.
    pub drop_preview: std::cell::RefCell<Option<crate::drop_preview::DropPreview>>,
    pub drag_guides: std::cell::RefCell<Option<crate::drag_guides::DragGuides>>,
    /// Fence windows (and their host fence ids) selected with a marquee on the desktop; a
    /// title drag of one of them moves them all. Owned by the App (`App::set_fence_selection`).
    pub selection: std::cell::RefCell<Vec<(HWND, FenceId)>>,
    /// Fence windows moved down (or up) out of an expanded neighbour's way
    /// (`rollUp.pushNeighbors`). Owned by the App's push pass (`app/push.rs`).
    pub pushed: std::cell::RefCell<Vec<Pushed>>,
    /// For each fence window with 向上展开: the room above its bottom edge and below its top
    /// edge it may open into, the fences there sliding out of its way or not (`app/push.rs`).
    /// With `rollUp.pushNeighbors` also for every other fence that pushes: the room below its
    /// top edge (the room above is the work area's).
    pub expand_room: std::cell::RefCell<Vec<(HWND, i32, i32)>>,
    /// The system move / size loop in progress (one at a time: it is modal).
    pub size_move: std::cell::RefCell<Option<SizeMove>>,
    /// Set while a title drag moves windows on a mouse move (`window_drag.rs`): they move at
    /// once and redraw their backdrop for the new place on the next frame instead.
    pub defer_backdrop: std::cell::Cell<bool>,
    /// Open while the app runs a frame (`App::on_frame`): the smaller sizes the frame's
    /// height animations ask for and the moves of its push pass. They are applied in one
    /// batch when the frame has them all, so a fence that rolls up and the fences that
    /// follow it move in the same composed frame. `None` outside a frame: a position applies
    /// at once.
    pub frame_positions: std::cell::RefCell<Option<Vec<window::WindowPos>>>,
    /// Height animations whose last step is in `frame_positions`, with the rolled state they
    /// end in: they land (`FrameHandle::land`) once the batch is applied.
    pub frame_landings: std::cell::RefCell<Vec<(HWND, Option<bool>)>>,
    /// A rolled fence expanded under the pointer: the next rolled fence the pointer comes to
    /// waits `TAB_HOVER_NEXT_MS` only (as the tabs of a strip do after a hover switch).
    pub roll_quick: std::cell::Cell<bool>,
    /// When the pointer left the title of a rolled or hover-expanded fence while `roll_quick`
    /// (`TAB_HOVER_NEXT_KEEP` later the full wait is back).
    pub roll_title_left: std::cell::Cell<Option<Instant>>,
}

/// A fence window the push pass moved down (or up, out of the way of a fence that expands
/// upward). The shift stands only while the window is still at the top the pass gave it:
/// anything else that moves it makes its new place the resting one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pushed {
    pub hwnd: HWND,
    /// Window top the pass gave it.
    pub top: i32,
    /// How far below its resting top that is (negative: above it).
    pub dy: i32,
    /// The fences that pushed it, directly or through the ones in between.
    pub by: Vec<HWND>,
    /// The work-area edge stopped it short of where the push wanted it (it still overlaps the
    /// fence that pushed it), and the largest shift (px) of this push so far: for the log lines.
    pub blocked: bool,
    pub max_dy: i32,
}

/// A window in the system move / size loop and, for a title drag, the selection moving along.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SizeMove {
    pub hwnd: HWND,
    pub moving: bool,
    pub group: Vec<HWND>,
}

impl Behavior {
    /// One frame of the drag feedback's fades (the alignment guides and the landing outline
    /// are layered windows: their alpha is a client tween, ticked by `App::on_frame`). Returns
    /// true while one is still on its way.
    pub fn tick_drag_feedback(&self, now: Instant) -> bool {
        let mut more = false;
        if let Ok(mut preview) = self.drop_preview.try_borrow_mut()
            && let Some(preview) = preview.as_mut()
        {
            more |= preview.tick(now);
        }
        if let Ok(mut guides) = self.drag_guides.try_borrow_mut()
            && let Some(guides) = guides.as_mut()
        {
            more |= guides.tick(now);
        }
        more
    }

    /// The wait before a rolled fence expands under the pointer or a dragged file, and before
    /// the tab under a resting pointer is switched to.
    pub fn hover_open_ms(&self) -> u32 {
        self.hover_speed.get().delay_ms(Tooltip::hover_delay_ms())
    }

    /// Keeps the short wait of rolled fences while the pointer is on the title of a rolled or
    /// hover-expanded fence (`on_title`) and for `TAB_HOVER_NEXT_KEEP` after it left one.
    pub fn note_rolled_title_pointer(&self, on_title: bool) {
        if !self.roll_quick.get() {
            return;
        }
        match (on_title, self.roll_title_left.get()) {
            (true, Some(left)) => {
                self.roll_title_left.set(None);
                if left.elapsed() > TAB_HOVER_NEXT_KEEP {
                    tracing::debug!(
                        "hover expand: off the rolled titles too long, the full wait again"
                    );
                    self.roll_quick.set(false);
                }
            }
            (false, None) => self.roll_title_left.set(Some(Instant::now())),
            _ => {}
        }
    }

    /// The wait before the rolled fence the pointer came to expands: the hover delay of
    /// 「展开快慢」, or the short wait once a hover has expanded a rolled fence.
    pub fn peek_open_wait_ms(&self) -> u32 {
        self.note_rolled_title_pointer(true);
        let quick = self.roll_quick.get();
        let ms = tab_hover_wait_ms(self.hover_open_ms(), quick);
        tracing::debug!(ms, quick, "hover expand armed");
        ms
    }

    /// A rolled fence expanded under the pointer: the next ones wait the short time.
    pub fn rolled_fence_hover_expanded(&self) {
        self.roll_quick.set(true);
        self.roll_title_left.set(None);
    }

    /// Inside the app's frame: notes `pos` for the frame's batch and returns true. A second
    /// note for the same window moves the first (a fence that is resized by its animation
    /// and pushed in the same frame). Outside a frame: false, the caller applies it.
    pub fn defer_position(&self, pos: window::WindowPos) -> bool {
        let mut guard = self.frame_positions.borrow_mut();
        let Some(batch) = guard.as_mut() else {
            return false;
        };
        match batch.iter_mut().find(|p| p.hwnd == pos.hwnd) {
            Some(p) => {
                (p.x, p.y) = (pos.x, pos.y);
                p.size = pos.size.or(p.size);
            }
            None => batch.push(pos),
        }
        true
    }

    /// The rectangle `hwnd` has once the frame's batch is applied: what the rest of the frame
    /// lays out for. Outside a frame, and for a window the frame does not move, its rect now.
    pub fn frame_rect(&self, hwnd: HWND) -> RECT {
        let r = window::window_rect(hwnd);
        let guard = self.frame_positions.borrow();
        let Some(p) = guard
            .as_ref()
            .and_then(|batch| batch.iter().find(|p| p.hwnd == hwnd))
        else {
            return r;
        };
        let (w, h) = p.size.unwrap_or((r.right - r.left, r.bottom - r.top));
        RECT {
            left: p.x,
            top: p.y,
            right: p.x + w,
            bottom: p.y + h,
        }
    }

    /// `rect`, `hwnd`'s window rectangle, without the push shift: what the layout records.
    pub fn resting_rect(&self, hwnd: HWND, mut rect: RECT) -> RECT {
        if let Some(p) = self
            .pushed
            .borrow()
            .iter()
            .find(|p| p.hwnd == hwnd && p.top == rect.top)
        {
            rect.top -= p.dy;
            rect.bottom -= p.dy;
        }
        rect
    }

    /// The user moved `hwnd`: where it is now is where it rests.
    pub fn release_push(&self, hwnd: HWND) {
        self.pushed.borrow_mut().retain(|p| p.hwnd != hwnd);
    }

    /// The pointer is over a fence that `hwnd` pushed out of its way: moving onto it must not
    /// close `hwnd`'s hover peek (the fence would slide away from under the pointer).
    pub fn holds_open(&self, hwnd: HWND, pt: pecofence_platform::POINT) -> bool {
        self.pushed.borrow().iter().any(|p| {
            let r = window::window_rect(p.hwnd);
            p.by.contains(&hwnd)
                && pt.x >= r.left
                && pt.x < r.right
                && pt.y >= r.top
                && pt.y < r.bottom
        })
    }
}

/// The snapping gap (`snapping.gapPx`, DIPs) in device px at `scale`, rounded. Every place
/// that lines fences up takes it from here: cut off in some and rounded in others, a fence
/// moved next to another sat 1 px closer than one sized to it (a gap of 3 at 150 %).
pub fn snap_gap_px(gap_dip: i32, scale: f32) -> i32 {
    (gap_dip as f32 * scale).round() as i32
}

/// A rectangle the user drew for a new fence, moved as a fence dragged there snaps: the
/// snapping gap from a fence or screen edge it nearly touches, in line with an edge it nearly
/// lines up with. Unchanged with snapping off.
pub fn snap_drawn_rect(behavior: &Behavior, rect: RECT) -> RECT {
    let mut snapped = rect;
    if behavior.snapping.get() {
        let centre = ((rect.left + rect.right) / 2, (rect.top + rect.bottom) / 2);
        let scale = monitors::query(monitors::monitor_from_point(centre.0, centre.1))
            .map_or(1.0, |m| m.scale());
        snap_rect(
            &mut snapped,
            HWND::default(),
            snap_gap_px(behavior.snap_gap_dip.get(), scale),
            (SNAP_DIST_DIP as f32 * scale) as i32,
        );
    }
    tracing::debug!(drawn = ?rect, ?snapped, "drawn rectangle snapped for a new fence");
    snapped
}

/// Queues `FenceBoundsChanged` with `hwnd`'s window rectangle as it is now, the push shift taken
/// out in the same breath (a push pass running before the command is handled cannot skew it).
pub(super) fn queue_bounds_changed(
    queue: &CommandQueue,
    behavior: &Behavior,
    fence: FenceId,
    hwnd: HWND,
) {
    queue.push(Command::FenceBoundsChanged {
        fence,
        rect: behavior.resting_rect(hwnd, window::window_rect(hwnd)),
    });
}

/// How the shown items fit the window (see [`FenceWindow::fit_report`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FitReport {
    pub columns: usize,
    pub rows: usize,
    /// Window height (device px) showing every row, capped at the work-area bottom.
    pub fitting_height_px: i32,
    /// The expanded height is shorter than the content needs.
    pub overflow: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackdropMode {
    /// Wallpaper-derived frosted glass, with a solid fallback when no sample is available.
    Acrylic,
}

/// The shared frosted-glass wallpaper images, one per monitor.
#[derive(Default)]
pub struct BackdropSets {
    pub acrylic: Rc<Vec<MonitorBackdrop>>,
    /// The monitors (virtual-screen pixels) whose live wallpaper cannot be read: their
    /// backdrop is a plain pane colour, and Liquid Glass lets the wallpaper itself show
    /// through it (`FenceViewState::material_opacity`).
    pub unreadable: Vec<RECT>,
}

impl BackdropSets {
    pub fn for_mode(&self, mode: BackdropMode) -> &Rc<Vec<MonitorBackdrop>> {
        match mode {
            BackdropMode::Acrylic => &self.acrylic,
        }
    }
}

/// One item as shown in a fence.
#[derive(Clone, Debug)]
pub struct ItemView {
    pub id: ItemId,
    pub path: PathBuf,
    pub name: String,
    pub is_folder: bool,
    pub icon_key: IconKey,
    pub icon_only: bool,
    /// Last write time (Unix seconds) and size in bytes, for the details columns.
    pub mtime: i64,
    pub size: u64,
    /// Local calendar day of `mtime` ("按时间分组" sections); namespace items have none.
    local_date: Option<CivilDate>,
    /// Label fitted to the cell width (computed lazily).
    label: Option<String>,
    /// `label` was fitted to the wider box, which showed more of the name (icons view;
    /// `text::fit_lines_wider`). Set with `label`.
    label_wide: bool,
    /// Details columns, formatted lazily (locale date, shell type name, KB size).
    date_label: Option<String>,
    type_label: Option<String>,
    size_label: Option<String>,
    icon: Option<Rc<Image>>,
    /// Pixel size `icon` was extracted at. After an icon-size, layout or DPI change the old
    /// bitmap stays up, scaled into the new box, until the new size arrives; its GPU upload
    /// is keyed by this size, never by the size being shown.
    icon_px: u32,
    icon_failed: bool,
    /// The icon lookup came back pending at least once: when it finally lands the bitmap
    /// cross-fades in over the loading tile (a synchronous cache hit shows at once).
    icon_waited: bool,
    /// Bitmap opacity 0 → 1 (167 ms linear) after an asynchronous icon arrived.
    icon_fade: Option<Tween>,
    /// A new item whose fade-in waits for its picture (`ICON_WAIT_MAX`): the moment the fade
    /// would have started without the wait. Taken when the picture is there.
    fade_waits: Option<Instant>,
    /// `icon` is the picture the item showed under its previous icon key (a rename changes a
    /// per-file key, a save a by-content one) and stays up until the new one arrives: the
    /// draw key it is uploaded under, so it is never cached under the new key's.
    icon_stand_in: Option<String>,
}

/// Peek search marks for one fence: matching items are highlighted, every other item dims, and
/// `current` (what Enter opens) is drawn selected with a focus ring. A rolled-up fence with
/// matches expands until they are gone. `tab_hits` counts the matches in each tab: a tab with
/// some shows the number in a badge, a tab without any dims.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchMarks {
    pub matches: HashSet<ItemId>,
    pub current: Option<ItemId>,
    pub tab_hits: HashMap<FenceId, usize>,
}

impl ItemView {
    /// Details "修改日期" text; namespace items (Recycle Bin, ...) have none.
    fn date_text(&self) -> String {
        if pecofence_platform::shell::is_namespace_path(&self.path) {
            String::new()
        } else {
            fileinfo::format_local_datetime(self.mtime)
        }
    }

    /// The icon for `px` is still to come: none yet, or only one of another size.
    fn wants_icon(&self, px: u32) -> bool {
        (self.icon.is_none() || self.icon_px != px) && !self.icon_failed
    }

    /// Bitmap-cache key of `icon` as it is drawn: the stand-in's own key while one is up.
    fn draw_key(&self, icons: &IconCache) -> String {
        match &self.icon_stand_in {
            Some(key) => key.clone(),
            None => icons.draw_key(&self.icon_key, self.icon_px),
        }
    }

    /// Details "大小" text; namespace items have none.
    fn size_text(&self) -> String {
        if pecofence_platform::shell::is_namespace_path(&self.path) {
            String::new()
        } else {
            fileinfo::format_size_kb(self.size, self.is_folder)
        }
    }

    pub fn new(
        id: ItemId,
        path: PathBuf,
        name: String,
        is_folder: bool,
        icon_key: IconKey,
        icon_only: bool,
        mtime: i64,
        size: u64,
    ) -> Self {
        let local_date = if pecofence_platform::shell::is_namespace_path(&path) {
            None
        } else {
            fileinfo::local_civil_date(mtime).map(|(y, m, d)| CivilDate::new(y, m, d))
        };
        Self {
            id,
            path,
            name,
            is_folder,
            icon_key,
            icon_only,
            mtime,
            size,
            local_date,
            label: None,
            label_wide: false,
            date_label: None,
            type_label: None,
            size_label: None,
            icon: None,
            icon_px: 0,
            icon_failed: false,
            icon_waited: false,
            icon_fade: None,
            fade_waits: None,
            icon_stand_in: None,
        }
    }
}

/// 脚本栅栏: what a script fence shows, the rows of its last output and a footer line (script
/// name, update time or what went wrong).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScriptView {
    pub rows: Vec<ScriptRowView>,
    pub footer: String,
    /// The footer without the script's name (the status alone), shown when the whole line
    /// does not fit the fence's width; empty when the footer is the name alone.
    pub footer_short: String,
    /// The first run has not ended: the empty body says 「正在运行…」 in place of the "printed
    /// no text" hint.
    pub quiet: bool,
}

/// A note while its editor is open (`crate::note_edit`): the text as it stands in the
/// editor (line breaks are `\n`), the two ends of the selection in UTF-16 units (the caret is
/// at `active`), and whether the caret shows right now (it blinks).
#[derive(Clone, Debug, PartialEq)]
pub struct NoteEditView {
    pub text: String,
    pub anchor: usize,
    pub active: usize,
    pub caret_on: bool,
}

/// One row of a script fence as drawn (see `pecofence_core::script_output`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScriptRowView {
    pub text: String,
    pub separator: bool,
    pub todo: Option<bool>,
    /// `color=` as the script wrote it, resolved for the theme when drawn.
    pub color: Option<String>,
    pub icon: Option<String>,
    /// The script's tooltip, else what a click opens, runs or copies.
    pub tip: Option<String>,
    pub clickable: bool,
    /// A click opens a URL or a file (an arrow shows on hover).
    pub opens: bool,
}

impl ScriptRowView {
    /// `text` as plain rows (an error, a hint).
    pub fn plain(text: &str) -> Vec<Self> {
        text.lines()
            .map(|t| Self {
                text: t.to_string(),
                ..Self::default()
            })
            .collect()
    }

    /// A failed run: `error` (the line that says what went wrong) in the colour of a
    /// `color=red` row, then what the script printed until it failed, as plain rows.
    pub fn failed(error: &str, output: &str) -> Vec<Self> {
        let mut rows = Vec::new();
        if !error.is_empty() {
            rows.push(Self {
                text: error.to_string(),
                color: Some("red".to_string()),
                ..Self::default()
            });
        }
        rows.extend(Self::plain(output));
        rows
    }

    /// A script's output read as rows with their parameters.
    pub fn from_output(text: &str) -> Vec<Self> {
        use pecofence_core::script_output::{RowAction, ScriptLine, parse};
        parse(text)
            .into_iter()
            .map(|line| match line {
                ScriptLine::Separator => Self {
                    separator: true,
                    ..Self::default()
                },
                ScriptLine::Row(r) => Self {
                    tip: r.tooltip.clone().or_else(|| r.target().map(str::to_string)),
                    clickable: r.clickable(),
                    opens: matches!(r.action, Some(RowAction::Href(_) | RowAction::Open(_))),
                    text: r.text,
                    separator: false,
                    todo: r.todo,
                    color: r.color,
                    icon: r.icon,
                },
            })
            .collect()
    }
}

/// One tab of a tabbed fence window (Fences 6): another fence shown inside this window.
#[derive(Clone, Debug)]
pub struct TabView {
    pub id: FenceId,
    /// A portal tab's current folder (where files dropped on its pill land); None = desktop.
    pub folder: Option<PathBuf>,
    pub title: String,
    pub color: Option<[u8; 3]>,
    pub title_size: u8,
    pub title_color: Option<ColorF>,
    /// A folder portal whose title icon is shown: the tab pill carries the folder glyph.
    pub folder_icon: bool,
    /// A folder portal shown inside a subfolder: while it is the shown tab, its folder glyph is
    /// the up button.
    pub up_button: bool,
    /// The window shows on some virtual desktops only: the pill carries the Task View glyph.
    pub desktop_icon: bool,
    /// Items and files can be dropped on its pill (not on a smart fence's).
    pub takes_drops: bool,
}

type ViewCell = Rc<RefCell<Option<FenceViewState>>>;
