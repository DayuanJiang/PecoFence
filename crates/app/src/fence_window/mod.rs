//! A fence window: composition-backed, never activated, anchored above the desktop.
//!
//! Visual tree: root → chrome panel (full window: backdrop, title, stroke) + content panel
//! (below the title bar; its surface clips the icon grid naturally).

use crate::anchor::{self, AnchorCell};
use crate::commands::{
    Command, CommandQueue, TransferMode, WM_APP_FADE_DONE, WM_APP_SET_VISIBLE, WM_APP_TAB_SWAP_DONE,
};
use crate::icons::{IconCache, Lookup};
use crate::layout::{
    CellRect, DetailColumn, DetailColumns, Grid, GridMetrics, ItemLayout, RowMetrics,
};
use crate::shadow::{ShadowStyle, ShadowWindow};
use pecofence_core::{FenceId, IconKey, ItemId, ItemKey, SortMode, Spacing, ViewLayout};
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
use pecofence_platform::{HWND, RECT, desktop, dwm, monitors, msg};
use pecofence_render::fence_chrome::{
    Backdrop, BackdropCrop, ContentDraw, FenceChrome, FenceStyle, HeaderColumn, ItemCell, RowCell,
    RowColumns, RowsDraw, ScrollbarDraw, TabDraw, TitleDeco, TitleState,
};
use pecofence_render::motion::{self, Curve, Fades, Motion, Prop, Tween};
use pecofence_render::{
    BitmapCache, ColorF, ContainerVisual, DesktopWindowTarget, Image, Matrix3x2, MonitorBackdrop,
    Panel, Rect, RenderStack, Theme,
};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};
use windows_core::Result;

mod api;
mod consts;
mod dnd;
mod frame;
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

use self::consts::*;
use self::dnd::*;
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
    pub backdrop: std::cell::Cell<BackdropMode>,
    /// Rolled fences expand on a single title click (hover peek off while set).
    pub click_to_expand: std::cell::Cell<bool>,
    /// Title row drawn only while hovered.
    pub title_on_hover: std::cell::Cell<bool>,
    /// Scrollbar drawn only while hovered / shortly after scrolling.
    pub hide_inactive_scrollbar: std::cell::Cell<bool>,
    /// `SPI_GETWHEELSCROLLLINES`: rows per wheel notch; `u32::MAX` (`WHEEL_PAGESCROLL`) = one
    /// viewport, 0 = the wheel does not scroll. Refreshed on WM_SETTINGCHANGE.
    pub wheel_lines: std::cell::Cell<u32>,
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
    /// Label fitted to the cell width (computed lazily).
    label: Option<String>,
    /// Details columns, formatted lazily (locale date, shell type name, KB size).
    date_label: Option<String>,
    type_label: Option<String>,
    size_label: Option<String>,
    icon: Option<Rc<Image>>,
    icon_failed: bool,
    /// The icon lookup came back pending at least once: when it finally lands the bitmap
    /// cross-fades in over the loading tile (a synchronous cache hit shows at once).
    icon_waited: bool,
    /// Bitmap opacity 0 → 1 (167 ms linear) after an asynchronous icon arrived.
    icon_fade: Option<Tween>,
}

impl ItemView {
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
        Self {
            id,
            path,
            name,
            is_folder,
            icon_key,
            icon_only,
            mtime,
            size,
            label: None,
            date_label: None,
            type_label: None,
            size_label: None,
            icon: None,
            icon_failed: false,
            icon_waited: false,
            icon_fade: None,
        }
    }
}

/// One tab of a tabbed fence window (Fences 6): another fence shown inside this window.
#[derive(Clone, Debug)]
pub struct TabView {
    pub id: FenceId,
    pub title: String,
    pub color: Option<[u8; 3]>,
    pub title_size: u8,
    pub title_color: Option<ColorF>,
}

type ViewCell = Rc<RefCell<Option<FenceViewState>>>;
