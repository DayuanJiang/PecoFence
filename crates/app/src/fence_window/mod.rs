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
    Backdrop, BackdropCrop, ContentDraw, FenceChrome, FenceStyle, GroupHeaderDraw, HeaderColumn,
    ItemCell, RAIL_BOTTOM, RAIL_COLLAPSED_BOTTOM, RAIL_ROW_H, RAIL_ROW_INSET, RAIL_ROW_RIGHT,
    RAIL_TOP, RowCell, RowColumns, RowsDraw, ScrollbarDraw, StripClip, TabDraw, TitleDeco,
    TitleState,
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

mod api;
mod consts;
mod desktop_layout;
mod dnd;
mod frame;
mod group;
mod handler;
mod hit;
mod items;
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
    /// How long the pointer rests on a rolled fence before it expands (`rollUp.hoverSpeed`).
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
    pub expand_room: std::cell::RefCell<Vec<(HWND, i32, i32)>>,
    /// The system move / size loop in progress (one at a time: it is modal).
    pub size_move: std::cell::RefCell<Option<SizeMove>>,
    /// Set while a title drag moves windows on a mouse move (`window_drag.rs`): they move at
    /// once and redraw their backdrop for the new place on the next frame instead.
    pub defer_backdrop: std::cell::Cell<bool>,
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
    /// The wait before a rolled fence expands under the pointer or a dragged file.
    pub fn hover_open_ms(&self) -> u32 {
        self.hover_speed.get().delay_ms(Tooltip::hover_delay_ms())
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
            date_label: None,
            type_label: None,
            size_label: None,
            icon: None,
            icon_px: 0,
            icon_failed: false,
            icon_waited: false,
            icon_fade: None,
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
    /// No "printed no text" hint while the body is empty (the first run has not ended).
    pub quiet: bool,
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
