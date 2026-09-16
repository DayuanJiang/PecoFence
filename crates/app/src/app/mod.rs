//! The application: owns state, fence windows, tray, watchers; drains the command queue.

use crate::anchor::{self, AnchorCell, DesktopAnchor, ShowDesktopBehavior, ZMode};
use crate::commands::{
    Command, CommandQueue, TransferMode, WM_APP_COMMAND, WM_APP_FRAME, WM_APP_FS_CHANGED,
    WM_APP_PEEK_FOCUSED, WM_APP_SHELL_CHANGED, WM_APP_TRAY, WM_APP_WALLPAPER,
};
use crate::fence_window::{
    BackdropMode, BackdropSets, Behavior, FenceContext, FenceWindow, ItemView, TabView,
};
use crate::icons::{IconCache, IconVariant, WM_APP_ICON_READY};
use crate::peek::PeekOverlay;
use crate::settings_host::{SettingsHost, WebEnvironment};
use crate::shadow::{ShadowStyle, ShadowWindow};
use crate::state::AppState;
use pecofence_core::geometry::WorkArea;
use pecofence_core::{
    FenceId, FenceKind, ItemId, ItemKey, PeekHotkey, ShowDesktopSetting, SortMode, Spacing, Target,
    ThemeSetting, TitleSize, ViewLayout, ZOrderSetting,
};
use pecofence_platform::clipboard;
use pecofence_platform::frameclock::FrameClock;
use pecofence_platform::hotkey;
use pecofence_platform::shell;
use pecofence_platform::shell_menu::ShellContextMenu;
use pecofence_platform::shell_notify::ShellChangeWatch;
use pecofence_platform::sysparams;
use pecofence_platform::tray::{self, PopupMenu, TrayIcon};
use pecofence_platform::watcher::{DirWatcher, FsEvent};
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, POINT, RECT, desktop, monitors, msg, theme as systheme, wallpaper};
use pecofence_render::fence_chrome::FenceChrome;
use pecofence_render::fence_chrome::FenceStyle;
use pecofence_render::motion::Motion;
use pecofence_render::{
    BitmapCache, Image, MonitorBackdrop, RenderStack, Theme, ThemeMode, WallpaperPosition,
};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use windows_core::Result;

mod dnd;
mod fence_options;
mod fences;
mod fileops;
mod items;
mod menus;
mod motion;
mod peek;
mod portals;
mod settings;
mod sync;
mod tabs;
#[cfg(test)]
mod tests;
mod testscript;
mod visuals;
mod wallpaper_refresh;

use fences::work_areas;
use fileops::{FileOp, FileOpThen};
use peek::peek_hotkey_label;
use visuals::{
    backdrop_mode_for, build_backdrop_sets, fence_style_for, icon_variant_for, pick_theme_mode,
    shadow_style_for, theme_for, tray_icon_image,
};
use wallpaper_refresh::{BackdropCache, FIRST_CHECK_MS, RefreshSchedule};

const TIMER_SAVE: usize = 40;
const TIMER_FS: usize = 41;
/// Folder-change coalescing window: the first event of a batch is handled this soon.
const FS_DEBOUNCE_MS: u64 = 60;
/// Minimum spacing between batches while changes keep streaming in (large copies).
const FS_STORM_MS: u64 = 400;
const TIMER_HOUSEKEEPING: usize = 42;
const TIMER_ICONS: usize = 43;
const TIMER_WALLPAPER: usize = 44;
const TIMER_WORKAREA: usize = 45;
const TIMER_CMD_RETRY: usize = 46;
/// The frame message arrived while the App was borrowed (modal loop); retry shortly.
const TIMER_FRAME_RETRY: usize = 47;
/// `--test-script` stepper.
const TIMER_TEST: usize = 48;
/// Retry a completed Peek focus handoff if the App is temporarily borrowed.
const TIMER_PEEK_FOCUS_RETRY: usize = 49;
/// Low-frequency safety net for changes that did not produce a wallpaper notification.
const TIMER_WALLPAPER_POLL: usize = 50;
/// Read only the 16-byte desktop identity, never images/COM, while otherwise idle.
const TIMER_DESKTOP_ID: usize = 51;
const SPI_SETDESKWALLPAPER: usize = 0x0014;
const SPI_SETWORKAREA: usize = 0x002F;
const TRAY_ID: u32 = 1;
const HOTKEY_PEEK: i32 = 1;

/// How long a fence-menu "新建" keeps its claim on the next desktop item (plan §5.8).
const PENDING_CREATION_SECS: u64 = 15;

/// Frame cadence of one animation run (from the first requested frame to the frame nothing
/// moves any more), for the "animation run stuttered" diagnostics.
#[derive(Default, Clone, Copy)]
struct FrameRun {
    started: Option<Instant>,
    frames: u32,
    max_gap: Duration,
    max_frame: Duration,
}

/// Runtime-only: a desktop path we expect to appear shortly (created from a fence's "新建" menu,
/// or moved out of a folder portal) goes straight into `fence`, bypassing the rules.
struct PendingRoute {
    fence: FenceId,
    path: PathBuf,
    since: Instant,
    /// Open the inline rename popup once it lands ("新建").
    rename: bool,
}

pub struct Args {
    pub light: bool,
    pub dark: bool,
    pub wallpaper_override: Option<String>,
    pub portable: bool,
    pub no_hide_icons: bool,
    pub exit_after_ms: Option<u32>,
    pub dump_stats: bool,
    pub open_settings: bool,
    /// `--portal <folder>`: create (or reveal) a folder-portal fence for this folder at start.
    pub portal: Option<String>,
    /// `--test-script <file>`: drive the app from a script (see `testscript.rs`).
    pub test_script: Option<String>,
}

pub struct App {
    state: AppState,
    ctx: Rc<FenceContext>,
    fences: HashMap<FenceId, FenceWindow>,
    /// Windows of deleted / merged fences, kept alive while they fade out; dropped (destroyed)
    /// on `Command::FadeOutDone`.
    dying: Vec<FenceWindow>,
    /// `--test-script` state while a script runs.
    test: Option<testscript::TestScript>,
    anchor: AnchorCell,
    control: Window,
    queue: CommandQueue,
    tray: Option<TrayIcon>,
    _watchers: Vec<DirWatcher>,
    /// Shell change notifications for the Recycle Bin and the desktop namespace, which no
    /// folder watcher sees (`None` when the shell refused the registration).
    _shell_watch: Option<ShellChangeWatch>,
    /// One watcher per folder-portal fence on the folder it currently shows (kept in sync by
    /// `ensure_portal_watchers`; navigating swaps the watcher).
    portal_watchers: HashMap<FenceId, (PathBuf, DirWatcher)>,
    fs_pending: Arc<Mutex<Vec<FsEvent>>>,
    /// Finished shell file operations from the worker threads (see `fileops.rs`).
    fileops_done: fileops::FileOpResults,
    settings: Option<SettingsHost>,
    /// Fence to select on the settings page once it reports `ready` (opened via 栅栏选项…).
    settings_focus_fence: Option<FenceId>,
    web_env: Option<WebEnvironment>,
    settings_class: WindowClass,
    theme_mode: ThemeMode,
    /// The user's accent palette at the last visual refresh (None = built-in default look).
    accent: Option<systheme::AccentPalette>,
    wallpaper_override: Option<String>,
    pending_routes: Vec<PendingRoute>,
    /// Previous frame's timestamp while an animation runs (frame-gap diagnostics).
    frame_prev: Option<Instant>,
    /// Cadence statistics of the animation run in progress.
    frame_run: FrameRun,
    /// `--no-hide-icons`: keep Explorer's icons visible this run without persisting the choice.
    no_hide_icons: bool,
    /// `--light` / `--dark`: pin the theme this run.
    theme_override: Option<ThemeMode>,
    peek_class: WindowClass,
    /// Dimmer windows while a Peek is active.
    peek: Option<PeekOverlay>,
    /// The desktop folder could not be read at the last sync (removable / network drive).
    desktop_unavailable: bool,
    /// Last run of the rules that depend on the clock ("闲置天数"); see `housekeeping`.
    idle_rules_checked: Option<Instant>,
    /// Fingerprint of the snapshot the current backdrops were built from.
    wallpaper_sig: Option<String>,
    wallpaper_cache: BackdropCache,
    /// The hotkey currently registered (None = registration failed or Peek disabled).
    peek_hotkey: Option<PeekHotkey>,
    /// The combination the last `sync_peek_hotkey` tried to register (the user's choice), so a
    /// persisting conflict does not re-toast on every settings apply.
    peek_hotkey_wanted: Option<PeekHotkey>,
    /// Items put on the clipboard with 剪切 (drawn dimmed) and the clipboard sequence number at
    /// that moment: any later clipboard change (Explorer pasted, something else was copied)
    /// clears the dimming.
    cut_items: HashSet<ItemId>,
    cut_clip_seq: u32,
}

pub type AppCell = Rc<RefCell<Option<App>>>;

/// The variant name of a command (its `Debug` form up to the first `{`, `(` or space).
fn command_name(cmd: &Command) -> String {
    let s = format!("{cmd:?}");
    s.split(['{', '(', ' ']).next().unwrap_or("").to_string()
}

impl App {
    /// Creates everything and returns the shared cell the control window drives.
    pub fn create(args: Args) -> Result<AppCell> {
        let cell: AppCell = Rc::new(RefCell::new(None));

        let stack = Rc::new(RenderStack::new()?);
        tracing::info!(
            queue = stack.queue_path,
            warp = stack.is_warp,
            "composition stack ready"
        );

        let areas = work_areas();
        for w in &areas {
            tracing::info!(name = %w.device_path, dpi = w.dpi, work = ?(w.left, w.top, w.right, w.bottom), "monitor");
        }
        let state = AppState::load(areas, args.portable);
        tracing::info!(path = %state.config_path().display(), first_run = state.first_run, fences = state.fences().len(), "config loaded");
        if let Some(p) = &state.recovered_from {
            tracing::warn!(from = %p.display(), "config recovered from backup");
        }

        let theme_mode = pick_theme_mode(state.config.settings.theme, &args);
        let accent = systheme::accent_palette();
        let theme = theme_for(
            theme_mode,
            state.config.settings.theme_style,
            accent.as_ref(),
        );
        let (backdrops, wallpaper_sig) =
            match build_backdrop_sets(&theme, args.wallpaper_override.as_deref()) {
                Ok((backdrops, signature)) => (backdrops, Some(signature)),
                Err(error) => {
                    tracing::warn!(%error, "wallpaper unavailable at startup; solid backdrop");
                    (Rc::new(BackdropSets::default()), None)
                }
            };

        let queue = CommandQueue::new();

        // Control window: timers, tray callbacks, command pump.
        let control_class = WindowClass::register("PecoFence.Control", ClassOptions::default())?;
        let control_handler: MessageHandler = {
            let cell = cell.clone();
            // Folder-watcher debounce state (see WM_APP_FS_CHANGED / TIMER_FS).
            let fs_armed = Cell::new(false);
            let fs_flushed: Cell<Option<Instant>> = Cell::new(None);
            let peek_focus_pending: Cell<Option<usize>> = Cell::new(None);
            // This lives outside App so nested/modal loops cannot lose a wallpaper signal.
            let wallpaper_schedule = Rc::new(Cell::new(RefreshSchedule::default()));
            let last_desktop_id = Cell::new(wallpaper::desktop_id());
            let notify_wallpaper = {
                let wallpaper_schedule = wallpaper_schedule.clone();
                move |hwnd| {
                    let mut schedule = wallpaper_schedule.get();
                    if let Some(delay) = schedule.notify(Instant::now()) {
                        window::set_timer(hwnd, TIMER_WALLPAPER, delay);
                    }
                    wallpaper_schedule.set(schedule);
                }
            };
            Box::new(
                move |hwnd: HWND, message: u32, wparam: usize, lparam: isize| -> Option<isize> {
                    match message {
                        WM_APP_COMMAND => {
                            if let Ok(mut guard) = cell.try_borrow_mut() {
                                if let Some(app) = guard.as_mut() {
                                    app.process_commands();
                                }
                            } else {
                                // The App is borrowed (modal loop inside a handler: menu, file
                                // dialog): retry shortly. A re-post would spin the CPU for as
                                // long as the modal loop runs.
                                window::set_timer(hwnd, TIMER_CMD_RETRY, 50);
                            }
                            Some(0)
                        }
                        WM_APP_PEEK_FOCUSED => {
                            peek_focus_pending.set(Some(wparam));
                            if let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.on_peek_focused(HWND(wparam as *mut core::ffi::c_void));
                                peek_focus_pending.set(None);
                            } else {
                                window::set_timer(hwnd, TIMER_PEEK_FOCUS_RETRY, 50);
                            }
                            Some(0)
                        }
                        WM_APP_FS_CHANGED | WM_APP_SHELL_CHANGED => {
                            if message == WM_APP_SHELL_CHANGED {
                                // The shell's own notice (Recycle Bin filled or emptied, a
                                // desktop namespace item toggled): release its payload and
                                // treat it like a folder change.
                                ShellChangeWatch::release(wparam, lparam);
                            }
                            // Explorer reacts to a folder change within a frame or two; a file
                            // the shell just moved onto the desktop (drag-out from a portal)
                            // must be filed as fast. Coalesce for one short window, never
                            // postpone while events keep coming, and back off only when a
                            // batch has just run (a big copy in progress).
                            if !fs_armed.replace(true) {
                                let storm = fs_flushed.get().is_some_and(|t| {
                                    t.elapsed() < Duration::from_millis(FS_STORM_MS)
                                });
                                let delay = if storm { FS_STORM_MS } else { FS_DEBOUNCE_MS };
                                window::set_timer(hwnd, TIMER_FS, delay as u32);
                            }
                            Some(0)
                        }
                        WM_APP_ICON_READY => {
                            window::set_timer(hwnd, TIMER_ICONS, 40);
                            Some(0)
                        }
                        msg::WM_HOTKEY if wparam as i32 == HOTKEY_PEEK => {
                            if let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.toggle_peek();
                            }
                            Some(0)
                        }
                        WM_APP_TRAY => {
                            let ev = tray::decode_tray_message(wparam, lparam);
                            if ev.event == tray::TRAY_EVENT_CONTEXTMENU
                                || ev.event == tray::TRAY_EVENT_SELECT
                                || ev.event == tray::TRAY_EVENT_KEYSELECT
                            {
                                if let Ok(mut guard) = cell.try_borrow_mut()
                                    && let Some(app) = guard.as_mut()
                                {
                                    app.show_tray_menu(ev.x, ev.y);
                                }
                            } else if ev.event == tray::TRAY_EVENT_LBUTTONDBLCLK
                                && let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.toggle_all_fences();
                            }
                            Some(0)
                        }
                        msg::WM_TIMER => {
                            match wparam {
                                TIMER_SAVE => {
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        window::kill_timer(hwnd, TIMER_SAVE);
                                        app.state.save_if_dirty();
                                    } else {
                                        // Busy (modal loop): retry shortly instead of dropping.
                                        window::set_timer(hwnd, TIMER_SAVE, 500);
                                    }
                                }
                                TIMER_FS => {
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        window::kill_timer(hwnd, TIMER_FS);
                                        fs_armed.set(false);
                                        app.on_fs_changed();
                                        fs_flushed.set(Some(Instant::now()));
                                    } else {
                                        // Busy (modal loop): retry shortly; stays armed.
                                        window::set_timer(hwnd, TIMER_FS, FS_STORM_MS as u32);
                                    }
                                }
                                TIMER_HOUSEKEEPING => {
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        app.housekeeping();
                                    }
                                }
                                TIMER_ICONS => {
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        window::kill_timer(hwnd, TIMER_ICONS);
                                        app.on_icons_ready();
                                    } else {
                                        // WebView2 creation and native menus pump messages while
                                        // the App is borrowed. Keep the completed icon batch
                                        // pending; workers may have sent their final notification.
                                        window::set_timer(hwnd, TIMER_ICONS, 100);
                                    }
                                }
                                TIMER_WALLPAPER => {
                                    window::kill_timer(hwnd, TIMER_WALLPAPER);
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        app.check_wallpaper("notification");
                                        let mut schedule = wallpaper_schedule.get();
                                        if let Some(delay) = schedule.checked(Instant::now()) {
                                            window::set_timer(hwnd, TIMER_WALLPAPER, delay);
                                        }
                                        wallpaper_schedule.set(schedule);
                                    } else {
                                        window::set_timer(hwnd, TIMER_WALLPAPER, FIRST_CHECK_MS);
                                    }
                                }
                                TIMER_WALLPAPER_POLL => {
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        app.check_wallpaper("poll");
                                    }
                                }
                                TIMER_DESKTOP_ID => {
                                    if let Some(id) = wallpaper::desktop_id()
                                        && last_desktop_id.replace(Some(id)) != Some(id)
                                    {
                                        tracing::debug!(
                                            "desktop identity changed; checking wallpaper"
                                        );
                                        notify_wallpaper(hwnd);
                                    }
                                }
                                TIMER_CMD_RETRY => {
                                    window::kill_timer(hwnd, TIMER_CMD_RETRY);
                                    window::post_message(hwnd, WM_APP_COMMAND, 0, 0);
                                }
                                TIMER_FRAME_RETRY => {
                                    window::kill_timer(hwnd, TIMER_FRAME_RETRY);
                                    window::post_message(hwnd, WM_APP_FRAME, 0, 0);
                                }
                                TIMER_PEEK_FOCUS_RETRY => {
                                    window::kill_timer(hwnd, TIMER_PEEK_FOCUS_RETRY);
                                    if let Some(dimmer) = peek_focus_pending.get() {
                                        if let Ok(mut guard) = cell.try_borrow_mut()
                                            && let Some(app) = guard.as_mut()
                                        {
                                            app.on_peek_focused(HWND(
                                                dimmer as *mut core::ffi::c_void,
                                            ));
                                            peek_focus_pending.set(None);
                                        } else {
                                            window::set_timer(hwnd, TIMER_PEEK_FOCUS_RETRY, 50);
                                        }
                                    }
                                }
                                TIMER_TEST => {
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        app.test_step();
                                        if app.test.is_none() {
                                            window::kill_timer(hwnd, TIMER_TEST);
                                        }
                                    }
                                }
                                TIMER_WORKAREA => {
                                    window::kill_timer(hwnd, TIMER_WORKAREA);
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        tracing::info!("work area changed; re-laying out fences");
                                        app.on_display_changed();
                                    } else {
                                        window::set_timer(hwnd, TIMER_WORKAREA, 500);
                                    }
                                }
                                _ => {}
                            }
                            Some(0)
                        }
                        msg::WM_SETTINGCHANGE if wparam == SPI_SETDESKWALLPAPER => {
                            notify_wallpaper(hwnd);
                            None
                        }
                        WM_APP_FRAME => {
                            if let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.on_frame();
                            } else {
                                window::set_timer(hwnd, TIMER_FRAME_RETRY, 16);
                            }
                            Some(0)
                        }
                        WM_APP_WALLPAPER => {
                            // Try promptly, then keep checking while Explorer publishes the
                            // new desktop's image. Repeated notifications never delay a check.
                            tracing::debug!(source = wparam, "wallpaper refresh signal");
                            notify_wallpaper(hwnd);
                            Some(0)
                        }
                        msg::WM_SETTINGCHANGE if wparam == SPI_SETWORKAREA => {
                            // Taskbar moved/resized/auto-hide toggled. The shell may broadcast
                            // several times while the taskbar animates; coalesce.
                            window::set_timer(hwnd, TIMER_WORKAREA, 250);
                            None
                        }
                        msg::WM_SETTINGCHANGE
                        | msg::WM_THEMECHANGED
                        | msg::WM_DWMCOLORIZATIONCOLORCHANGED => {
                            if let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.on_system_settings_changed();
                            }
                            None
                        }
                        msg::WM_DISPLAYCHANGE => {
                            if let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.on_display_changed();
                            }
                            Some(0)
                        }
                        msg::WM_QUERYENDSESSION => {
                            if let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.state.save_if_dirty();
                                if let Some(a) = app.anchor.borrow_mut().as_mut() {
                                    a.restore_desktop_icons();
                                }
                            }
                            Some(1)
                        }
                        msg::WM_DESTROY => Some(0),
                        _ => None,
                    }
                },
            )
        };
        let control = WindowBuilder::new(&control_class)
            .title("PecoFence")
            .style(style::POPUP)
            .ex_style(style::EX_TOOLWINDOW | style::EX_NOACTIVATE)
            .bounds(0, 0, 0, 0)
            .create(control_handler)?;
        queue.attach(&control);
        std::mem::forget(control_class); // class lives for the process

        // Desktop anchor (z-order, show desktop, quick hide, icon hiding).
        let sentinel_class = anchor::register_sentinel_class()?;
        let zmode = match state.config.settings.zorder {
            ZOrderSetting::InsertAboveHost => ZMode::InsertAboveHost,
            ZOrderSetting::HwndBottom => ZMode::HwndBottom,
        };
        let behavior = match state.config.settings.show_desktop {
            ShowDesktopSetting::KeepVisible => ShowDesktopBehavior::KeepVisible,
            ShowDesktopSetting::HideWithDesktop => ShowDesktopBehavior::HideWithDesktop,
        };
        let anchor_cell = DesktopAnchor::create(
            zmode,
            behavior,
            state.config.settings.quick_hide.enabled,
            &sentinel_class,
        )?;
        std::mem::forget(sentinel_class);
        if let Some(a) = anchor_cell.borrow_mut().as_mut() {
            let q = queue.clone();
            a.on_marquee = Some(Box::new(move |rect| q.push(Command::NewFenceRect(rect))));
        }

        let motion = Rc::new(Motion::new(&stack)?);
        motion.set_enabled(sysparams::client_area_animation());
        let frames = {
            // HWND is not Send; the clock thread only ever posts to it.
            let control_hwnd = control.hwnd().0 as isize;
            Rc::new(FrameClock::start(move || {
                window::post_message(
                    HWND(control_hwnd as *mut core::ffi::c_void),
                    WM_APP_FRAME,
                    0,
                    0,
                );
            }))
        };
        let ctx = Rc::new(FenceContext {
            stack: stack.clone(),
            motion,
            frames,
            class: WindowClass::register(
                anchor::FENCE_CLASS,
                ClassOptions {
                    double_clicks: true,
                    background: None,
                },
            )?,
            chrome: Rc::new(FenceChrome::new()?),
            theme: RefCell::new(theme),
            backdrops: RefCell::new(backdrops),
            anchor: anchor_cell.clone(),
            taskbar_created: desktop::taskbar_created_message(),
            icons: Rc::new(RefCell::new({
                let mut c = IconCache::new(control.hwnd());
                c.set_variant(icon_variant_for(&state.config.settings.icons));
                c
            })),
            bitmaps: Rc::new(RefCell::new(BitmapCache::new())),
            queue: queue.clone(),
            shadow_class: ShadowWindow::register_class()?,
            shadow_style: std::cell::Cell::new(shadow_style_for(&theme)),
            behavior: Rc::new(Behavior {
                floating: std::cell::Cell::new(false),
                hover_peek: std::cell::Cell::new(state.config.settings.roll_up.hover_peek),
                snapping: std::cell::Cell::new(state.config.settings.snapping.enabled),
                backdrop: std::cell::Cell::new(backdrop_mode_for(state.config.settings.backdrop)),
                click_to_expand: std::cell::Cell::new(
                    state.config.settings.roll_up.click_to_expand,
                ),
                title_on_hover: std::cell::Cell::new(state.config.settings.roll_up.title_on_hover),
                hide_inactive_scrollbar: std::cell::Cell::new(
                    state.config.settings.roll_up.hide_inactive_scrollbar,
                ),
                wheel_lines: std::cell::Cell::new(sysparams::wheel_scroll_lines()),
            }),
        });

        // Tray (the glyph uses the same accent token as the selection).
        let tray_px = (16.0 * (monitors::system_dpi().max(96) as f32 / 96.0)).round() as i32;
        let tray = tray::icon_from_bgra(
            tray_px,
            tray_px,
            &tray_icon_image(tray_px, theme.accent_rgb8(), theme_mode == ThemeMode::Dark),
        )
        .and_then(|icon| TrayIcon::add(control.hwnd(), TRAY_ID, WM_APP_TRAY, icon, "PecoFence"))
        .map_err(|e| tracing::warn!(error = %e, "tray icon failed"))
        .ok();

        // Watchers.
        let fs_pending: Arc<Mutex<Vec<FsEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let mut watchers = Vec::new();
        for dir in [shell::user_desktop(), shell::public_desktop()]
            .into_iter()
            .flatten()
        {
            let pending = fs_pending.clone();
            let control_hwnd = control.hwnd().0 as isize;
            match DirWatcher::start(&dir, move |events| {
                if let Ok(mut p) = pending.lock() {
                    p.extend(events);
                }
                window::post_message(
                    HWND(control_hwnd as *mut core::ffi::c_void),
                    WM_APP_FS_CHANGED,
                    0,
                    0,
                );
            }) {
                Ok(w) => watchers.push(w),
                Err(e) => tracing::warn!(dir = %dir.display(), error = %e, "watcher failed"),
            }
        }

        let shell_watch = ShellChangeWatch::register_desktop(control.hwnd(), WM_APP_SHELL_CHANGED)
            .map_err(|e| tracing::warn!(error = %e, "shell change notifications unavailable"))
            .ok();

        let settings_class = SettingsHost::register_class()?;
        let peek_class = PeekOverlay::register_class()?;

        let mut app = App {
            state,
            ctx,
            fences: HashMap::new(),
            anchor: anchor_cell,
            control,
            queue,
            tray,
            _watchers: watchers,
            _shell_watch: shell_watch,
            portal_watchers: HashMap::new(),
            fs_pending,
            fileops_done: Arc::new(Mutex::new(Vec::new())),
            settings: None,
            settings_focus_fence: None,
            web_env: None,
            settings_class,
            theme_mode,
            accent,
            wallpaper_override: args.wallpaper_override.clone(),
            pending_routes: Vec::new(),
            frame_prev: None,
            frame_run: FrameRun::default(),
            no_hide_icons: args.no_hide_icons,
            theme_override: if args.light {
                Some(ThemeMode::Light)
            } else if args.dark {
                Some(ThemeMode::Dark)
            } else {
                None
            },
            peek_class,
            peek: None,
            dying: Vec::new(),
            test: None,
            peek_hotkey: None,
            peek_hotkey_wanted: None,
            desktop_unavailable: false,
            idle_rules_checked: None,
            cut_items: HashSet::new(),
            cut_clip_seq: 0,
            wallpaper_sig,
            wallpaper_cache: BackdropCache::default(),
        };
        if let Some(signature) = app.wallpaper_sig.clone() {
            app.wallpaper_cache
                .insert(signature, app.ctx.backdrops.borrow().clone());
        }
        // Watch both wallpaper settings and virtual-desktop state; either signal starts a
        // bounded burst of fast reads, because the image may arrive after the notification.
        {
            let control_hwnd = app.control.hwnd().0 as isize;
            if let Err(e) = wallpaper::watch_changes(move || {
                window::post_message(
                    HWND(control_hwnd as *mut core::ffi::c_void),
                    WM_APP_WALLPAPER,
                    0,
                    0,
                );
            }) {
                tracing::warn!(error = %e, "wallpaper registry watcher failed; polling only");
            }
        }
        app.sync_peek_hotkey();
        if let Some(a) = app.anchor.borrow_mut().as_mut() {
            let q = app.queue.clone();
            a.on_peek_interrupted = Some(Box::new(move || q.push(Command::EndPeek)));
            let control_hwnd = app.control.hwnd().0 as isize;
            a.on_foreground_changed = Some(Box::new(move || {
                window::post_message(
                    HWND(control_hwnd as *mut core::ffi::c_void),
                    WM_APP_WALLPAPER,
                    1,
                    0,
                );
            }));
        }

        // Desktop folder moved since last run? Re-point the item records before syncing, or
        // every item would be orphaned and re-routed by the rules.
        if let Some(desk) = shell::user_desktop() {
            let n = app.state.migrate_desktop_path(&desk);
            if n > 0 {
                tracing::info!(count = n, to = %desk.display(), "desktop folder moved; records migrated");
                if let Some(t) = &app.tray {
                    t.show_info(
                        "PecoFence",
                        &pecofence_core::i18n::format(
                            "检测到桌面文件夹已移动，已迁移 {0} 个项目的记录。",
                            &[n.to_string()],
                        ),
                        false,
                    );
                }
            }
        }
        // Initial desktop sync + windows.
        app.sync_desktop_if_available("startup");
        app.state.refresh_all_portals();
        app.sync_fence_windows();
        if let Some(folder) = args.portal.as_deref() {
            let (cx, cy) = app
                .state
                .work_areas
                .first()
                .map(|w| ((w.left + w.right) / 2, (w.top + w.bottom) / 2))
                .unwrap_or((400, 300));
            app.create_portal(PathBuf::from(folder), cx, cy, None);
        }
        if let Some(a) = app.anchor.borrow_mut().as_mut() {
            a.reanchor("startup");
        }
        for f in app.fences.values() {
            f.show(false);
        }
        if let Some(a) = app.anchor.borrow_mut().as_mut() {
            a.reanchor("after show");
            tracing::debug!(
                "z-order after show:
{}",
                a.zorder_report(false)
            );
            if a.stale_marker_present() {
                tracing::warn!(
                    "stale icons-hidden marker found: previous run did not restore icons"
                );
                if let Some(t) = &app.tray {
                    t.show_info(
                        "PecoFence",
                        pecofence_core::i18n::text(
                            "检测到上次未正常退出，已接管桌面图标的隐藏状态。",
                        ),
                        true,
                    );
                }
            }
            if app.state.config.settings.hide_real_icons && !app.no_hide_icons {
                // At logon Explorer's desktop may not be ready yet: retried from the anchor's
                // periodic check; housekeeping resets the setting if it never succeeds.
                a.request_hide_desktop_icons();
            }
        }
        // Portable/development runs do not change login startup. Normal releases
        // adopt the renamed entry while keeping an existing working PecoFence copy.
        if !args.portable {
            if let Err(error) = pecofence_platform::autostart::reconcile_product(
                app.state.config.settings.autostart,
                cfg!(debug_assertions),
            ) {
                tracing::warn!(%error, "autostart reconciliation failed");
            }
        }
        app.state.save_if_dirty();
        window::set_coalescable_timer(app.control.hwnd(), TIMER_HOUSEKEEPING, 60_000, 5_000);
        window::set_coalescable_timer(app.control.hwnd(), TIMER_WALLPAPER_POLL, 5_000, 500);
        window::set_coalescable_timer(app.control.hwnd(), TIMER_DESKTOP_ID, 100, 25);
        // Close the gap between the startup snapshot and installing the registry watchers.
        window::post_message(app.control.hwnd(), WM_APP_WALLPAPER, 0, 0);

        if args.open_settings {
            app.queue.push(Command::OpenSettings);
        }
        if let Some(path) = args.test_script.as_deref() {
            app.test = testscript::TestScript::load(path);
            if app.test.is_some() {
                tracing::info!(target: "pecofence::test", path, "test script loaded");
                window::set_timer(app.control.hwnd(), TIMER_TEST, 50);
            }
        }
        if args.dump_stats {
            match pecofence_platform::memstats::MemoryStats::current() {
                Ok(m) => tracing::info!("startup memory: {}", m.summary()),
                Err(e) => tracing::warn!(error = %e, "memstats failed"),
            }
        }
        *cell.borrow_mut() = Some(app);
        Ok(cell)
    }

    fn schedule_save(&self) {
        window::set_timer(self.control.hwnd(), TIMER_SAVE, 800);
    }

    fn housekeeping(&mut self) {
        self.check_cut_clipboard();
        if self.state.is_dirty() {
            self.state.save_if_dirty();
        }
        // While the special desktop items are shown, re-read them once a minute as well: a
        // change in Windows' own "Desktop icon settings" leaves no folder event behind, and the
        // shell notification channel may be unavailable.
        let special_items_shown = self.state.config.settings.hide_real_icons && !self.no_hide_icons;
        if (self.desktop_unavailable && shell::desktop_available()) || special_items_shown {
            let report = self.sync_desktop_if_available("housekeeping");
            if report.changed() {
                self.refresh_all();
                self.schedule_save();
                self.push_workspace_summary();
            }
        }
        // Rules with an idle-days condition depend on the clock, not on desktop events: while
        // automatic filing is on, re-run the rules on the first tick and then hourly.
        let idle_rules_due = {
            let rules = &self.state.config.rules;
            rules.keep_updated
                && rules.has_idle_rules()
                && !self.desktop_unavailable
                && self
                    .idle_rules_checked
                    .is_none_or(|t| t.elapsed() >= Duration::from_secs(3600))
        };
        if idle_rules_due {
            self.idle_rules_checked = Some(Instant::now());
            let entries = shell::enumerate_desktop();
            let moved = self.state.apply_rules_all(&entries);
            if moved > 0 {
                tracing::info!(moved, "idle-days rules re-applied");
                self.refresh_all();
                self.schedule_save();
            }
        }
        let changed_dpi: Vec<FenceId> = self
            .fences
            .iter()
            .filter_map(|(&id, window)| window.check_dpi().then_some(id))
            .collect();
        // Ordinary housekeeping must preserve a tabbed window's shared geometry.
        for id in changed_dpi {
            self.apply_column_snap(id);
        }
        // Startup hide that kept failing: converge to the same end state as apply_settings
        // (setting reflects reality, user is told) instead of silently showing the toggle on.
        if self.state.config.settings.hide_real_icons {
            let gave_up = self
                .anchor
                .borrow()
                .as_ref()
                .is_some_and(|a| a.hide_desktop_icons_gave_up());
            if gave_up {
                if let Some(a) = self.anchor.borrow_mut().as_mut() {
                    a.cancel_pending_hide();
                }
                tracing::warn!(
                    "hiding desktop icons kept failing after startup; turning the setting off"
                );
                self.state.config.settings.hide_real_icons = false;
                self.settings_mutated();
                self.desktop_icons_setting_changed();
                if let Some(t) = &self.tray {
                    t.show_info(
                        "PecoFence",
                        pecofence_core::i18n::text("无法隐藏桌面图标，请稍后再试。"),
                        true,
                    );
                }
            }
        }
    }

    /// Drains and executes queued commands. Loops because handlers may enqueue more.
    pub fn process_commands(&mut self) {
        self.check_cut_clipboard();
        self.drain_fileops();
        for _ in 0..8 {
            let cmds = self.queue.drain();
            if cmds.is_empty() {
                return;
            }
            for cmd in cmds {
                // A command that blocks the UI thread for more than a compositor frame delays
                // the first frame of any animation it started (time-based tweens then appear
                // to jump ahead): name it.
                let name = command_name(&cmd);
                let started = Instant::now();
                self.handle(cmd);
                let spent = started.elapsed();
                if spent > Duration::from_millis(8) {
                    tracing::info!(
                        ms = spent.as_secs_f32() * 1000.0,
                        command = name,
                        "slow command"
                    );
                }
            }
        }
    }

    fn handle(&mut self, cmd: Command) {
        match cmd {
            Command::FenceBoundsChanged { fence, rect } => {
                let (rolled, expanded) = self
                    .fences
                    .get(&fence)
                    .map(|w| (w.is_rolled(), w.expanded_height_px()))
                    .unwrap_or((false, rect.bottom - rect.top));
                self.state.set_fence_bounds(fence, rect, rolled, expanded);
                self.schedule_save();
                self.apply_auto_height(fence);
            }
            Command::ToggleRollUp(fence) => self.toggle_roll(fence),
            Command::CommitExpanded(fence) => {
                if let Some(w) = self.fences.get(&fence) {
                    w.commit_expanded();
                }
                if let Some(f) = self.state.fence_mut(fence) {
                    f.rolled_up = false;
                }
                self.state.mark_dirty();
                self.schedule_save();
                // A committed peek keeps whatever height the peek had; apply the auto-height
                // rule like every other path that expands a fence.
                self.apply_auto_height(fence);
            }
            Command::LaunchItem(item) => self.launch(item),
            Command::MoveItems { items, to } => self.move_items(&items, to),
            Command::ReorderItems {
                fence,
                items,
                index,
            } => {
                if self.state.reorder_items(fence, &items, index) {
                    self.refresh_fence(fence);
                    self.schedule_save();
                }
            }
            Command::DropIntoFolder {
                paths,
                folder,
                fence,
                mode,
            } => self.transfer_files_into_folder(paths, folder, fence, mode),
            Command::DragOutFinished => {
                // The shell performs the file operation itself, often asynchronously: what is
                // not on the desktop yet arrives through the folder watcher a moment later.
                let report = self.sync_desktop_if_available("drag out");
                self.route_pending_creation();
                self.refresh_portals();
                if report.changed() {
                    tracing::info!(?report, "desktop resynced right after the drag-out");
                    self.refresh_all();
                    self.schedule_save();
                }
            }
            Command::PortalEnter { fence, path } => self.portal_enter(fence, &path),
            Command::PortalUp(fence) => self.portal_up(fence),
            Command::SwitchTab { host, tab } => self.switch_tab(host, tab),
            Command::SetColumnWidths { fence, widths } => {
                self.state.set_column_widths(fence, widths);
                self.schedule_save();
            }
            Command::MergeHint { target, x } => {
                for w in self.fences.values() {
                    w.set_merge_hint(!target.0.is_null() && w.hwnd() == target, x);
                }
            }
            Command::MergeFence { fence, into, x } => {
                // The slot the gap sat at (read before the hints — and the gap — are cleared).
                let host = self.fence_of_hwnd(into);
                let index = host
                    .and_then(|h| self.fences.get(&h))
                    .map(|w| w.merge_slot_at(x));
                for w in self.fences.values() {
                    w.set_merge_hint(false, 0);
                }
                if let Some(host) = host
                    && host != fence
                    && self.state.attach_tab(fence, host)
                {
                    if let Some(index) = index {
                        // Land where the gap was, so the neighbours do not move again.
                        self.state.reorder_tab(host, fence, index);
                    }
                    self.resync_windows();
                    self.apply_fence_view_with_snap(host, false);
                    if let Some(w) = self.fences.get(&host) {
                        self.queue.push(Command::RaiseFence(w.hwnd()));
                    }
                    self.schedule_save();
                }
            }
            Command::DetachTab {
                tab,
                x,
                y,
                from_drag,
            } => self.detach_tab(tab, x, y, from_drag),
            Command::CancelDetach { change } => self.cancel_tab_detach(*change),
            Command::ReorderTab { host, tab, to } => self.reorder_tab(host, tab, to),
            Command::MoveItemsToInbox { items } => {
                // Dropped on the bare desktop: back to the inbox, then straight through the
                // rules — the same routing a new desktop item gets, now instead of whenever the
                // watcher next reconciles (which used to leave the icon in the rolled-up inbox
                // for a second or two).
                if let Some(inbox) = self.state.inbox_id() {
                    self.move_items(&items, inbox);
                    let entries = shell::enumerate_desktop();
                    let touched = self.state.apply_rules_to(&items, &entries);
                    tracing::debug!(
                        items = items.len(),
                        refiled = ?touched,
                        "dropped on the desktop: inbox, then rules"
                    );
                    for id in touched {
                        self.refresh_fence(id);
                    }
                    self.schedule_save();
                }
            }
            Command::ExternalDrop { paths, to, mode } if self.state.portal_path(to).is_some() => {
                self.transfer_files_into_portal(paths, to, mode);
            }
            Command::ExternalDrop {
                paths,
                to,
                mode: TransferMode::Link,
            } => {
                // Create shortcut: the .lnk lands on the desktop and goes straight into `to`
                // (a PendingRoute bypasses the rules), the originals stay where they are.
                if let Some(desktop) = shell::user_desktop() {
                    let made = self.create_shortcuts_in(&paths, &desktop);
                    for path in made {
                        self.pending_routes.push(PendingRoute {
                            fence: to,
                            path,
                            since: Instant::now(),
                            rename: false,
                        });
                    }
                    let report = self.sync_desktop_if_available("shortcut drop");
                    self.route_pending_creation();
                    if report.changed() {
                        self.refresh_all();
                        self.schedule_save();
                    }
                }
            }
            Command::ExternalDrop { paths, to, mode } => {
                let copy = mode == TransferMode::Copy;
                let desktop_dirs: Vec<PathBuf> = [shell::user_desktop(), shell::public_desktop()]
                    .into_iter()
                    .flatten()
                    .collect();
                let mut ids = Vec::new();
                let mut foreign = Vec::new();
                for p in paths {
                    if let Some(id) = self.state.item_by_path(&p) {
                        ids.push(id);
                    } else if p
                        .parent()
                        .is_some_and(|parent| desktop_dirs.iter().any(|d| d == parent))
                    {
                        // On the desktop but not yet synced: sync then retry.
                        let entries = shell::enumerate_desktop();
                        self.state.sync_desktop(&entries);
                        if let Some(id) = self.state.item_by_path(&p) {
                            ids.push(id);
                        }
                    } else {
                        foreign.push(p);
                    }
                }
                if !foreign.is_empty() {
                    // Like Fences: dropping a file from elsewhere onto a fence moves it to the
                    // desktop and puts it in that fence (a PendingRoute bypasses the rules).
                    if let Some(desktop) = shell::user_desktop() {
                        let routed: Vec<PathBuf> = foreign
                            .iter()
                            .filter_map(|p| p.file_name().map(|n| desktop.join(n)))
                            .collect();
                        for path in &routed {
                            self.pending_routes.push(PendingRoute {
                                fence: to,
                                path: path.clone(),
                                since: Instant::now(),
                                rename: false,
                            });
                        }
                        let owner = self.window_for(to).map(|w| w.hwnd());
                        self.start_fileop(FileOp {
                            paths: foreign,
                            dest: desktop,
                            copy,
                            rename_on_collision: false,
                            owner,
                            then: FileOpThen::ToDesktop {
                                routed,
                                copy,
                                what: "dropped files placed on the desktop",
                            },
                        });
                    }
                }
                if !ids.is_empty() {
                    self.move_items(&ids, to);
                }
            }
            Command::ExternalUrlDrop { url, name, to } => self.create_url_shortcut(url, name, to),
            Command::FenceMenu { fence, x, y } => self.show_fence_menu(fence, x, y),
            Command::HeaderMenu { fence, x, y } => self.show_header_menu(fence, x, y),
            Command::ItemMenu { fence, items, x, y } => self.show_item_menu(fence, items, x, y),
            Command::ClipboardVerb { fence, items, cut } => self.clipboard_verb(fence, &items, cut),
            Command::Paste { fence } => self.paste_into(fence),
            Command::NewFolder { fence } => self.create_desktop_item(fence, true),
            Command::StepIconSize { fence, larger } => self.step_icon_size(fence, larger),
            Command::ItemProperties { fence, items } => self.show_properties(fence, &items),
            Command::RefreshFence(fence) => self.manual_refresh(fence),
            Command::RenameFence { fence, title } => {
                self.end_item_rename_visuals();
                let title = title.trim();
                if !title.is_empty() {
                    self.state.rename_fence(fence, title);
                    self.refresh_fence(fence);
                    self.schedule_save();
                    self.push_settings_state();
                }
                if let Some(w) = self.window_for(fence) {
                    w.redraw();
                }
            }
            Command::DeleteFence(fence) => self.delete_fence(fence),
            Command::NewFence { x, y } => self.new_fence_near(x, y),
            Command::NewFenceRect(rect) => self.offer_new_fence(rect),
            Command::RaiseFence(hwnd) => {
                if let Some(a) = self.anchor.borrow_mut().as_mut() {
                    a.raise_fence(hwnd);
                }
            }
            Command::SortColumn { fence, sort } => {
                // Explorer: clicking the active column flips the direction.
                if let Some(f) = self.state.fence(fence) {
                    let reverse = f.view.sort == sort && !f.view.reverse;
                    self.state.set_sort(fence, sort);
                    self.state.set_reverse(fence, reverse);
                    self.refresh_fence(fence);
                    self.schedule_save();
                }
            }
            Command::RenameItem { fence, item } => self.begin_item_rename(fence, item),
            Command::RenameItemCommit { item, name } => {
                self.end_item_rename_visuals();
                self.rename_item_file(item, &name);
            }
            Command::EndItemRename { commit } => crate::rename::end_active(commit),
            Command::DeleteItems {
                fence,
                items,
                permanent,
            } => self.delete_items(fence, &items, permanent),
            Command::DeviceLost => self.recover_device(),
            Command::ToggleAllFences => self.toggle_all_fences(),
            Command::TogglePeek => self.toggle_peek(),
            Command::EndPeek => self.end_peek(),
            Command::ApplyRulesNow => {
                let entries = shell::enumerate_desktop();
                let moved = self.state.apply_rules_all(&entries);
                self.refresh_all();
                self.schedule_save();
                if let Some(t) = &self.tray {
                    t.show_info(
                        "PecoFence",
                        &pecofence_core::i18n::format(
                            "已按规则整理 {0} 个项目。",
                            &[moved.to_string()],
                        ),
                        false,
                    );
                }
            }
            Command::OpenSettings => self.open_settings(),
            Command::OpenOptionsForFence(fence) => self.open_fence_options(fence),
            Command::RedrawAll => {
                // Also the rename edit's cancel path: the edited label may unfold again.
                self.end_item_rename_visuals();
                for w in self.fences.values() {
                    w.redraw();
                }
            }
            Command::Quit => {
                self.state.save_if_dirty();
                if let Some(a) = self.anchor.borrow_mut().as_mut() {
                    a.restore_desktop_icons();
                }
                window::post_quit(0);
            }
            Command::WindowGone(hwnd) => {
                if self.settings.as_ref().is_some_and(|h| h.hwnd() == hwnd) {
                    self.settings = None;
                }
            }
            Command::FadeOutDone(hwnd) => {
                // Dropping the window destroys it (RevokeDragDrop + DestroyWindow).
                self.dying.retain(|w| w.hwnd() != hwnd);
            }
            Command::SettingsMessage(json) => self.on_settings_message(&json),
        }
    }

    pub fn shutdown(&mut self) {
        self.end_peek_now();
        self.state.save_if_dirty();
        self.fences.clear();
        self.dying.clear();
        if let Some(a) = self.anchor.borrow_mut().as_mut() {
            a.restore_desktop_icons();
        }
        self.tray = None;
    }
}
