//! The application: owns state, fence windows, tray, watchers; drains the command queue.

use crate::anchor::{self, AnchorCell, DesktopAnchor, ShowDesktopBehavior, ZMode};
use crate::commands::{
    Command, CommandQueue, TransferMode, WM_APP_COMMAND, WM_APP_FRAME, WM_APP_FS_CHANGED,
    WM_APP_LIVE_WALLPAPER, WM_APP_PEEK_FOCUSED, WM_APP_PREWARM_DONE, WM_APP_SHELL_CHANGED,
    WM_APP_STARTUP_BACKDROPS, WM_APP_TRAY, WM_APP_WALLPAPER,
};
use crate::fence_window::{
    BackdropMode, BackdropSets, Behavior, FenceContext, FenceWindow, ItemView, SearchMarks, TabView,
};
use crate::icons::{IconCache, IconVariant, WM_APP_ICON_READY};
use crate::ipc_server::{IpcServer, PendingQueue};
use crate::peek::PeekOverlay;
use crate::settings_host::{SettingsHost, WebEnvironment};
use crate::shadow::{ShadowStyle, ShadowWindow};
use crate::state::AppState;
use pecofence_core::geometry::WorkArea;
use pecofence_core::hotkey::Hotkey;
use pecofence_core::{
    FenceId, FenceKind, ItemId, ItemKey, ShowDesktopSetting, SortMode, Spacing, Target,
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
use pecofence_platform::{
    HWND, POINT, RECT, desktop, monitors, msg, process, theme as systheme, wallpaper,
};
use pecofence_render::fence_chrome::FenceChrome;
use pecofence_render::fence_chrome::FenceStyle;
use pecofence_render::motion::Motion;
use pecofence_render::{
    BitmapCache, Image, MonitorBackdrop, RenderStack, Theme, ThemeMode, WallpaperPosition,
};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use windows_core::Result;

mod arrange;
mod dnd;
mod feedback;
mod fence_options;
mod fences;
mod fileops;
mod group;
mod ipc;
mod items;
mod live_wallpaper;
mod menus;
mod motion;
mod peek;
mod portals;
mod push;
mod rating;
mod settings;
mod sync;
mod tabs;
#[cfg(test)]
mod tests;
mod testscript;
mod updates;
mod visuals;
mod wallpaper_refresh;
mod welcome;

use fences::work_areas;
use fileops::{FileOp, FileOpThen};
use peek::peek_hotkey_label;
use visuals::{
    StartupBackdrops, backdrop_mode_for, fence_style_for, icon_variant_for, pick_theme_mode,
    shadow_style_for, theme_for, tray_icon_image, verify_wallpaper_later,
};
use wallpaper_refresh::{BackdropCache, FIRST_CHECK_MS, RefreshSchedule, supersedes};

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
/// After a display change the fences wait this long for the monitors to settle (plugging,
/// unplugging and a monitor waking pass through in-between sets) before they are laid out.
const DISPLAY_SETTLE_MS: u32 = 1000;
const TIMER_CMD_RETRY: usize = 46;
/// The frame message arrived while the App was borrowed (modal loop); retry shortly.
const TIMER_FRAME_RETRY: usize = 47;
/// `--test-script` stepper.
const TIMER_TEST: usize = 48;
/// Retry a completed Peek focus handoff if the App is temporarily borrowed.
const TIMER_PEEK_FOCUS_RETRY: usize = 49;
/// Low-frequency safety net for changes that did not produce a wallpaper notification.
const TIMER_WALLPAPER_POLL: usize = 50;
/// `events.subscribe` diff tick while at least one CLI stream is open (see `ipc.rs`).
const TIMER_IPC_EVENTS: usize = 53;
/// Read only the 16-byte desktop identity, never images/COM, while otherwise idle.
const TIMER_DESKTOP_ID: usize = 51;
/// Re-armed by every panel draw: fires once drawing has been quiet this long and releases
/// GPU scratch memory (`RenderStack::trim`).
const TIMER_GPU_TRIM: usize = 54;
const GPU_TRIM_IDLE_MS: u32 = 3_000;
/// Prepares the other virtual desktops' wallpapers once wallpaper signals have been quiet
/// this long (every signal re-arms it), and finishes a prepared one.
const TIMER_WALLPAPER_PREWARM: usize = 55;
const TIMER_PREWARM_FINISH: usize = 56;
/// The tray icon could not be added (taskbar not ready, or the App was busy on
/// `TaskbarCreated`): try again.
const TIMER_TRAY_RETRY: usize = 57;
const TRAY_RETRY_MS: u32 = 2_000;
/// WM_ENDSESSION(FALSE) arrived while the App was borrowed: hide the icons again shortly.
const TIMER_SHUTDOWN_CANCELLED: usize = 58;
/// A theme, accent or system settings change arrived while the App was borrowed: apply it
/// shortly.
const TIMER_SYSTEM_SETTINGS_RETRY: usize = 52;
/// Rebuilding the GPU device failed (driver reset in progress): try again, backing off.
const TIMER_DEVICE_RETRY: usize = 59;
const DEVICE_RETRY_FIRST_MS: u32 = 1_000;
/// Re-armed by every panel draw, like `TIMER_GPU_TRIM`: once drawing has been quiet this long,
/// hands the memory the last burst of work touched back to Windows
/// (`memstats::trim_working_set`). While memory is plentiful Windows leaves a normal-priority
/// process's pages resident, so without it Task Manager keeps counting everything any earlier
/// interaction touched (~28 MB after hours of use on a 4K PC), although an idle app touches
/// ~3 MB.
const TIMER_IDLE_TRIM: usize = 60;
const IDLE_TRIM_MS: u32 = 30_000;
/// The live wallpaper capture worker finished (`WM_APP_LIVE_WALLPAPER`); retried while the App
/// is borrowed.
const TIMER_LIVE_CAPTURED: usize = 61;
/// `TaskbarCreated` arrived while the App was borrowed: renew the shell change registration
/// shortly.
const TIMER_SHELL_WATCH_RETRY: usize = 62;
/// The startup wallpaper worker finished late (see `StartupBackdrops`).
const TIMER_STARTUP_BACKDROPS: usize = 63;
const DEVICE_RETRY_MAX_MS: u32 = 30_000;
/// Growth of private commit plus GPU memory that logs a memory line before the hour is up.
const MEMORY_LOG_GROWTH: u64 = 64 << 20;
const WALLPAPER_PREWARM_DELAY_MS: u32 = 3_000;
const SPI_SETDESKWALLPAPER: usize = 0x0014;
const SPI_SETWORKAREA: usize = 0x002F;
const TRAY_ID: u32 = 1;
const HOTKEY_PEEK: i32 = 1;
/// Registered for a moment to learn whether a shortcut the user picks is free.
const HOTKEY_PROBE: i32 = 2;

/// How long a fence-menu "新建" keeps its claim on the next desktop item (plan §5.8).
const PENDING_CREATION_SECS: u64 = 15;
/// How long a shell drop onto a virtual fence claims the shortcuts that appear on the desktop
/// afterwards.
const SHELL_DROP_CLAIM_SECS: u64 = 5;

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
    /// `PECOFENCE_INSTANCE` (trimmed, non-empty) of this process; names the IPC pipe.
    pub instance: Option<String>,
}

pub struct App {
    state: AppState,
    ctx: Rc<FenceContext>,
    fences: HashMap<FenceId, FenceWindow>,
    /// Windows of deleted / merged fences, kept alive while they fade out; dropped (destroyed)
    /// on `Command::FadeOutDone`.
    dying: Vec<FenceWindow>,
    /// GPU recovery retry: one is armed (`TIMER_DEVICE_RETRY`), and the delay of the next one.
    device_retry_pending: bool,
    device_retry_ms: u32,
    /// GPU memory at the first idle point after startup or the last rebuild that handed it
    /// back, and when that rebuild ran (`motion::hand_back_gpu_memory`).
    gpu_baseline: Option<u64>,
    gpu_rebuilt: Option<Instant>,
    /// `--test-script` state while a script runs.
    test: Option<testscript::TestScript>,
    /// Selection kept by a Ctrl / Shift desktop marquee in progress (empty for a plain one);
    /// `None` between marquees.
    marquee_base: Option<Vec<(HWND, FenceId)>>,
    /// Our rubber band for desktop marquees (created on first use), and whether the marquee in
    /// progress shows it (only while Explorer's icons are hidden: Explorer draws its own).
    marquee_band: Option<crate::marquee_band::MarqueeBand>,
    marquee_draws_band: bool,
    anchor: AnchorCell,
    control: Window,
    queue: CommandQueue,
    tray: Option<TrayIcon>,
    /// The user's and the public desktop folder, each with its watcher (restarted per folder
    /// by `revive_watchers`).
    watchers: Vec<(PathBuf, DirWatcher)>,
    /// Shell change notifications for the Recycle Bin and the desktop namespace, which no
    /// folder watcher sees (`None` when the shell refused the registration).
    _shell_watch: Option<ShellChangeWatch>,
    /// One watcher per folder-portal fence on the folder it currently shows (kept in sync by
    /// `ensure_portal_watchers`; navigating swaps the watcher).
    portal_watchers: HashMap<FenceId, (PathBuf, DirWatcher)>,
    /// Portal folders whose watcher could not be started after a long wait (an unreachable
    /// network share): how many times, and when to try again (see `ensure_portal_watchers`).
    watcher_retry: HashMap<PathBuf, (u32, Instant)>,
    fs_pending: Arc<Mutex<Vec<FsEvent>>>,
    /// Finished shell file operations from the worker threads (see `fileops.rs`).
    fileops_done: fileops::FileOpResults,
    /// File operations still running: their PendingRoutes must not expire mid-copy.
    fileops_running: usize,
    /// Serial of the last file operation started (`run_fileop`).
    fileop_serial: u64,
    /// CLI requests waiting for the UI thread (see `ipc_server.rs` / `ipc.rs`).
    ipc_pending: PendingQueue,
    /// The named-pipe listener; `None` when the pipe name was unavailable.
    _ipc: Option<IpcServer>,
    /// Warnings the CLI handler being run has queued for its reply (see `ipc.rs`).
    ipc_warnings: Vec<String>,
    /// Open `events.subscribe` streams and the last tick's state they are diffed against.
    ipc_subscribers: Vec<ipc::IpcSubscriber>,
    ipc_event_state: Option<ipc::EventState>,
    /// `PECOFENCE_INSTANCE` of this process (reported by `status.get`).
    instance: Option<String>,
    settings: Option<SettingsHost>,
    /// Fence to select on the settings page once it reports `ready` (opened via 栅栏选项…).
    settings_focus_fence: Option<FenceId>,
    /// The 「反馈」 form on 「关于」: pending focus and the crash offer (see `feedback.rs`).
    feedback: feedback::FeedbackState,
    updates: updates::Updates,
    /// The one-time Store rating offer (see `rating.rs`).
    rating: rating::RatingPrompt,
    /// The first start's welcome notice while it is on screen (see `welcome.rs`).
    welcome: Option<crate::notice::Notice>,
    /// The fence Settings › 「栅栏」 is editing, which wears the selection ring (`group.rs`).
    settings_ring: Option<FenceId>,
    web_env: Option<WebEnvironment>,
    settings_class: WindowClass,
    theme_mode: ThemeMode,
    /// The user's accent palette at the last visual refresh (None = built-in default look).
    accent: Option<systheme::AccentPalette>,
    wallpaper_override: Option<String>,
    pending_routes: Vec<PendingRoute>,
    /// The virtual fence a shell drop just went to, and when: the first desktop change batch
    /// within `SHELL_DROP_CLAIM_SECS` that adds shortcuts puts them into it (the shell writes
    /// some after `Drop` returned), see `on_fs_changed`.
    shell_drop_claim: Option<(FenceId, Instant)>,
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
    /// Peek search matches in reading order (host window, tab, item) and the highlighted one.
    peek_hits: Vec<(FenceId, FenceId, ItemId)>,
    peek_hit: usize,
    /// Fences the search moved to another tab, with the tab each showed before.
    peek_tabs: HashMap<FenceId, FenceId>,
    /// The desktop folder could not be read at the last sync (removable / network drive).
    desktop_unavailable: bool,
    /// The tray told the user that the configuration cannot be saved (once per run).
    save_failure_shown: bool,
    /// Last run of the rules that depend on the clock ("闲置天数"); see `housekeeping`.
    idle_rules_checked: Option<Instant>,
    /// When the last memory line was logged and the total it showed; see `log_memory`.
    memory_logged: Option<(Instant, u64)>,
    /// Fingerprint of the snapshot the current backdrops were built from.
    wallpaper_sig: Option<String>,
    wallpaper_cache: BackdropCache,
    /// Another virtual desktop's wallpaper being decoded ahead of a switch there.
    prewarm: Option<visuals::PrewarmJob>,
    /// The startup wallpaper worker, while Explorer has not answered it (see `StartupBackdrops`).
    startup_backdrops: Option<StartupBackdrops>,
    /// Monitors a live wallpaper covers, and the capture of them in progress.
    live: Vec<live_wallpaper::LiveWallpaper>,
    live_capture: Option<live_wallpaper::LiveCapture>,
    /// The worker of a given-up capture that has not returned yet (one at most).
    live_abandoned: Option<std::thread::JoinHandle<()>>,
    /// The hotkey currently registered (None = registration failed or Peek disabled).
    peek_hotkey: Option<Hotkey>,
    /// The combination the last `sync_peek_hotkey` tried to register (the user's choice), so a
    /// persisting conflict does not re-toast on every settings apply.
    peek_hotkey_wanted: Option<Hotkey>,
    /// The settings page is recording a new shortcut: `peek_hotkey` is unregistered meanwhile,
    /// so pressing it reaches the page instead of starting a Peek.
    peek_hotkey_paused: bool,
    /// Items put on the clipboard with 剪切 (drawn dimmed) and the clipboard sequence number at
    /// that moment: any later clipboard change (Explorer pasted, something else was copied)
    /// clears the dimming.
    cut_items: HashSet<ItemId>,
    cut_clip_seq: u32,
    /// A paste of cut files is still being moved by a worker thread: the serial of that file
    /// operation and the clipboard sequence number at the paste. The clipboard is emptied when
    /// the move succeeds, unless something else was put on it meanwhile (see `paste_into`).
    cut_paste: Option<(u64, u32)>,
}

pub type AppCell = Rc<RefCell<Option<App>>>;

/// Seconds from Explorer's start to this process's, to a tenth (negative: Explorer restarted
/// since).
fn seconds_after_explorer() -> Option<f64> {
    let explorer = desktop::window_pid(desktop::shell_window());
    process::seconds_after_start_of(explorer).map(|s| (s * 10.0).round() / 10.0)
}

/// Re-reads whether virtual desktops keep different pictures. The 100 ms desktop poll (and the
/// foreground signal) only run while they do: a switch cannot change the wallpaper otherwise.
fn sync_desktops_differ(control: HWND, differ: &Cell<bool>) {
    let now = wallpaper::desktops_differ();
    // Starts false with no poll running.
    if differ.replace(now) != now {
        tracing::debug!(desktops_differ = now, "virtual desktop wallpapers");
        if now {
            window::set_coalescable_timer(control, TIMER_DESKTOP_ID, 100, 25);
        } else {
            window::kill_timer(control, TIMER_DESKTOP_ID);
        }
    }
}

/// The variant name of a command (its `Debug` form up to the first `{`, `(` or space).
fn command_name(cmd: &Command) -> String {
    let s = format!("{cmd:?}");
    s.split(['{', '(', ' ']).next().unwrap_or("").to_string()
}

impl App {
    /// Creates everything and returns the shared cell the control window drives.
    pub fn create(args: Args) -> Result<AppCell> {
        let cell: AppCell = Rc::new(RefCell::new(None));

        // Elevation decides which windows the fences may be placed under; the delay after
        // Explorer shows how long a launch at sign-in waited.
        let after_explorer_s = seconds_after_explorer();
        tracing::info!(
            packaged = process::is_packaged(),
            elevated = ?process::is_elevated(process::current_pid()),
            after_explorer_s = ?after_explorer_s,
            "process started"
        );
        let stack = Rc::new(RenderStack::new()?);
        tracing::info!(
            queue = stack.queue_path,
            warp = stack.is_warp,
            "composition stack ready"
        );

        let areas = work_areas();
        fences::log_monitors(&areas);
        let state = AppState::load(areas, args.portable);
        tracing::info!(
            path = %state.config_path().display(),
            storage = state.storage_kind(),
            first_run = state.first_run,
            fences = state.fences().len(),
            "config loaded"
        );
        if let Some(p) = &state.recovered_from {
            tracing::warn!(from = %p.display(), "config recovered from backup");
        }
        if let Some(p) = &state.unreadable_moved_to {
            tracing::warn!(to = %p.display(), "config.json unreadable; kept aside");
        }

        let theme_mode = pick_theme_mode(state.config.settings.theme, &args);
        tray::set_dark_menus(theme_mode == ThemeMode::Dark);
        let accent = systheme::accent_palette();
        let theme = theme_for(
            theme_mode,
            state.config.settings.theme_style,
            accent.as_ref(),
        );
        // Decoded on a worker while the rest of startup runs; in place before the first fence.
        let mut startup_backdrops =
            StartupBackdrops::start(&theme, args.wallpaper_override.as_deref());

        let queue = CommandQueue::new();
        // Whether a virtual-desktop switch can change the picture (see `sync_desktops_differ`).
        let desktops_differ = Rc::new(Cell::new(false));
        // The desktop anchor once it exists: WM_QUERYENDSESSION shows the real icons again
        // through it while a menu or dialog holds the App.
        let session_anchor: Rc<std::cell::OnceCell<AnchorCell>> = Rc::default();

        // Control window: timers, tray callbacks, command pump.
        let control_class = WindowClass::register("PecoFence.Control", ClassOptions::default())?;
        let control_handler: MessageHandler = {
            let cell = cell.clone();
            let taskbar_created = desktop::taskbar_created_message();
            // Folder-watcher debounce state (see WM_APP_FS_CHANGED / TIMER_FS).
            let fs_armed = Cell::new(false);
            let fs_flushed: Cell<Option<Instant>> = Cell::new(None);
            // The pending batch must sync the desktop even if its events all lie in portal
            // folders: a shell notice or a menu asked for it.
            let fs_desktop = Cell::new(false);
            // TIMER_ICONS is armed for a batch of results (see WM_APP_ICON_READY).
            let icons_armed = Cell::new(false);
            let peek_focus_pending: Cell<Option<usize>> = Cell::new(None);
            // A display change is waiting out DISPLAY_SETTLE_MS on TIMER_WORKAREA.
            let display_settling = Cell::new(false);
            // This lives outside App so nested/modal loops cannot lose a wallpaper signal.
            let wallpaper_schedule = Rc::new(Cell::new(RefreshSchedule::default()));
            let last_desktop_id = Cell::new(wallpaper::desktop_id());
            let desktops_differ = desktops_differ.clone();
            let session_anchor = session_anchor.clone();
            let queue = queue.clone();
            // Virtual desktop of the last foreground window (see WM_APP_WALLPAPER).
            let last_foreground_desktop: Cell<Option<u128>> = Cell::new(None);
            let notify_wallpaper = {
                let wallpaper_schedule = wallpaper_schedule.clone();
                move |hwnd| {
                    let mut schedule = wallpaper_schedule.get();
                    if let Some(delay) = schedule.notify(Instant::now()) {
                        window::set_timer(hwnd, TIMER_WALLPAPER, delay);
                    }
                    wallpaper_schedule.set(schedule);
                    window::set_timer(hwnd, TIMER_WALLPAPER_PREWARM, WALLPAPER_PREWARM_DELAY_MS);
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
                                // The Recycle Bin may have changed: ask it at the next sync.
                                // Plain folder events leave the cached answer alone (asking
                                // touches every fixed drive); without this channel the sync
                                // itself asks every time.
                                shell::recycle_bin_changed();
                            }
                            if message == WM_APP_SHELL_CHANGED || wparam != 0 {
                                fs_desktop.set(true);
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
                        WM_APP_STARTUP_BACKDROPS => {
                            window::set_timer(hwnd, TIMER_STARTUP_BACKDROPS, 1);
                            Some(0)
                        }
                        WM_APP_PREWARM_DONE => {
                            window::set_timer(hwnd, TIMER_PREWARM_FINISH, 1);
                            Some(0)
                        }
                        WM_APP_LIVE_WALLPAPER => {
                            if wparam == 0 {
                                window::set_timer(hwnd, TIMER_LIVE_CAPTURED, 1);
                            } else if let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.capture_live_wallpapers(live_wallpaper::LIVE_FRONT_REFRESH);
                            }
                            Some(0)
                        }
                        WM_APP_ICON_READY => {
                            // One timer per batch: SetTimer restarts a running timer, so arming
                            // it for every result would hold the drain back for as long as the
                            // workers keep finishing icons less than 40 ms apart.
                            if !icons_armed.replace(true) {
                                window::set_timer(hwnd, TIMER_ICONS, 40);
                            }
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
                            // Right click (or Shift+F10) = menu; left click (or Enter) =
                            // settings, at once: no double-click action to wait for.
                            if ev.event == tray::TRAY_EVENT_CONTEXTMENU {
                                if let Ok(mut guard) = cell.try_borrow_mut()
                                    && let Some(app) = guard.as_mut()
                                {
                                    app.show_tray_menu(ev.x, ev.y);
                                }
                            } else if (ev.event == tray::TRAY_EVENT_SELECT
                                || ev.event == tray::TRAY_EVENT_KEYSELECT)
                                && let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.open_settings();
                            } else if ev.event == tray::TRAY_EVENT_BALLOONUSERCLICK
                                && let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.on_balloon_click();
                            } else if (ev.event == tray::TRAY_EVENT_BALLOONTIMEOUT
                                || ev.event == tray::TRAY_EVENT_BALLOONHIDE)
                                && let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.on_balloon_gone();
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
                                        app.notice_save_failure();
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
                                        app.on_fs_changed(fs_desktop.take());
                                        fs_flushed.set(Some(Instant::now()));
                                    } else {
                                        // Busy (modal loop): retry shortly; stays armed.
                                        window::set_timer(hwnd, TIMER_FS, FS_STORM_MS as u32);
                                    }
                                }
                                TIMER_GPU_TRIM => {
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        window::kill_timer(hwnd, TIMER_GPU_TRIM);
                                        app.trim_gpu_if_idle();
                                    }
                                }
                                TIMER_IDLE_TRIM => {
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        window::kill_timer(hwnd, TIMER_IDLE_TRIM);
                                        app.trim_if_idle();
                                    }
                                }
                                TIMER_WALLPAPER_PREWARM
                                | TIMER_PREWARM_FINISH
                                | TIMER_LIVE_CAPTURED
                                | TIMER_SHELL_WATCH_RETRY
                                | TIMER_STARTUP_BACKDROPS => {
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        window::kill_timer(hwnd, wparam);
                                        match wparam {
                                            TIMER_PREWARM_FINISH => app.finish_prewarm(),
                                            TIMER_LIVE_CAPTURED => app.finish_live_capture(),
                                            TIMER_SHELL_WATCH_RETRY => app.renew_shell_watch(),
                                            TIMER_STARTUP_BACKDROPS => {
                                                app.finish_startup_backdrops()
                                            }
                                            _ => app.prewarm_wallpapers(),
                                        }
                                    } else {
                                        window::set_timer(hwnd, wparam, 200);
                                    }
                                }
                                TIMER_HOUSEKEEPING => {
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        app.housekeeping();
                                    }
                                }
                                TIMER_IPC_EVENTS => {
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        app.ipc_events_tick();
                                    }
                                }
                                TIMER_ICONS => {
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        icons_armed.set(false);
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
                                    sync_desktops_differ(hwnd, &desktops_differ);
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
                                TIMER_DEVICE_RETRY => {
                                    window::kill_timer(hwnd, TIMER_DEVICE_RETRY);
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        app.retry_device_recovery();
                                    } else {
                                        window::set_timer(hwnd, TIMER_DEVICE_RETRY, 250);
                                    }
                                }
                                TIMER_SHUTDOWN_CANCELLED => {
                                    window::kill_timer(hwnd, TIMER_SHUTDOWN_CANCELLED);
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        app.shutdown_cancelled();
                                    } else {
                                        window::set_timer(hwnd, TIMER_SHUTDOWN_CANCELLED, 250);
                                    }
                                }
                                TIMER_SYSTEM_SETTINGS_RETRY => {
                                    window::kill_timer(hwnd, TIMER_SYSTEM_SETTINGS_RETRY);
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        app.on_system_settings_changed();
                                    } else {
                                        window::set_timer(hwnd, TIMER_SYSTEM_SETTINGS_RETRY, 250);
                                    }
                                }
                                TIMER_TRAY_RETRY => {
                                    window::kill_timer(hwnd, TIMER_TRAY_RETRY);
                                    let shown = cell
                                        .try_borrow()
                                        .ok()
                                        .and_then(|g| g.as_ref().map(App::show_tray));
                                    if shown != Some(true) {
                                        window::set_timer(hwnd, TIMER_TRAY_RETRY, TRAY_RETRY_MS);
                                    }
                                }
                                TIMER_WORKAREA => {
                                    window::kill_timer(hwnd, TIMER_WORKAREA);
                                    if let Ok(mut guard) = cell.try_borrow_mut()
                                        && let Some(app) = guard.as_mut()
                                    {
                                        display_settling.set(false);
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
                            sync_desktops_differ(hwnd, &desktops_differ);
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
                            if wparam == 1 {
                                // A foreground change: a desktop switch announcing itself. It
                                // cannot change the picture while every desktop shows the same,
                                // and a window on the previous one's desktop is no switch.
                                if !desktops_differ.get() {
                                    last_foreground_desktop.set(None);
                                    return Some(0);
                                }
                                let desktop = wallpaper::window_desktop(HWND(
                                    lparam as *mut core::ffi::c_void,
                                ));
                                if desktop.is_some()
                                    && last_foreground_desktop.replace(desktop) == desktop
                                {
                                    return Some(0);
                                }
                                // Unknown (the desktop itself, say): the next one counts again.
                                last_foreground_desktop.set(desktop);
                            } else {
                                sync_desktops_differ(hwnd, &desktops_differ);
                            }
                            // Try promptly, then keep checking while Explorer publishes the
                            // new desktop's image. Repeated notifications never delay a check.
                            tracing::debug!(source = wparam, "wallpaper refresh signal");
                            notify_wallpaper(hwnd);
                            Some(0)
                        }
                        msg::WM_SETTINGCHANGE if wparam == SPI_SETWORKAREA => {
                            // Taskbar moved/resized/auto-hide toggled. The shell may broadcast
                            // several times while the taskbar animates; coalesce (and never cut
                            // a display change's wait short).
                            let wait = if display_settling.get() {
                                DISPLAY_SETTLE_MS
                            } else {
                                250
                            };
                            window::set_timer(hwnd, TIMER_WORKAREA, wait);
                            None
                        }
                        msg::WM_SETTINGCHANGE
                        | msg::WM_THEMECHANGED
                        | msg::WM_DWMCOLORIZATIONCOLORCHANGED => {
                            if let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.on_system_settings_changed();
                            } else {
                                // A menu or dialog runs; Windows does not repeat the broadcast.
                                window::set_timer(hwnd, TIMER_SYSTEM_SETTINGS_RETRY, 250);
                            }
                            None
                        }
                        msg::WM_DISPLAYCHANGE => {
                            // Each change restarts the wait: lay out once the monitors settled.
                            display_settling.set(true);
                            window::set_timer(hwnd, TIMER_WORKAREA, DISPLAY_SETTLE_MS);
                            Some(0)
                        }
                        m if m == taskbar_created && m != 0 => {
                            // Explorer (re)started: its new taskbar has no notification icons.
                            tracing::info!("TaskbarCreated: adding the tray icon again");
                            let shown = cell
                                .try_borrow()
                                .ok()
                                .and_then(|g| g.as_ref().map(App::show_tray));
                            if shown != Some(true) {
                                window::set_timer(hwnd, TIMER_TRAY_RETRY, 250);
                            }
                            if let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.renew_shell_watch();
                            } else {
                                window::set_timer(hwnd, TIMER_SHELL_WATCH_RETRY, 250);
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
                            } else if let Some(anchor) = session_anchor.get()
                                && let Ok(mut guard) = anchor.try_borrow_mut()
                                && let Some(a) = guard.as_mut()
                            {
                                // The session may end before the menu or dialog returns.
                                a.restore_desktop_icons();
                            }
                            Some(1)
                        }
                        msg::WM_ENDSESSION if wparam == 0 => {
                            // The shutdown was cancelled (another program refused, or the user
                            // chose Cancel): WM_QUERYENDSESSION showed the real icons again.
                            if let Ok(mut guard) = cell.try_borrow_mut()
                                && let Some(app) = guard.as_mut()
                            {
                                app.shutdown_cancelled();
                            } else {
                                window::set_timer(hwnd, TIMER_SHUTDOWN_CANCELLED, 250);
                            }
                            Some(0)
                        }
                        msg::WM_ENDSESSION => {
                            // The session ends, or a package update or Restart Manager asks
                            // this program to close (ENDSESSION_CLOSEAPP): leave as 退出 does
                            // (WM_QUERYENDSESSION saved and showed the icons). Staying makes
                            // Windows wait, then kill it and report a hang.
                            tracing::info!(close_app = lparam & 1 != 0, "session ending: quitting");
                            if cell.try_borrow_mut().is_err() {
                                // A menu holds the App, and its loop keeps running past the
                                // quit below: close it, then quit as 退出 does (saving first)
                                // once the App is free again.
                                tray::end_menu();
                                queue.push(Command::Quit);
                            }
                            window::post_quit(0);
                            Some(0)
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
        let _ = session_anchor.set(anchor_cell.clone());
        if let Some(a) = anchor_cell.borrow_mut().as_mut() {
            let q = queue.clone();
            a.on_marquee = Some(Box::new(move |event| {
                q.push(Command::DesktopMarquee(event))
            }));
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
            backdrops: RefCell::new(Rc::new(BackdropSets::default())),
            anchor: anchor_cell.clone(),
            taskbar_created: desktop::taskbar_created_message(),
            icons: Rc::new(RefCell::new({
                let mut c = IconCache::new(control.hwnd());
                c.set_variant(icon_variant_for(&state.config.settings.icons));
                // Started at sign-in (or before Explorer): the shell's first answers may be
                // placeholders.
                if after_explorer_s.is_none_or(|s| s < 300.0) {
                    c.recheck_after_start();
                }
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
                snap_gap_dip: std::cell::Cell::new(settings::snap_gap_dip(
                    &state.config.settings.snapping,
                )),
                size_to_cells: std::cell::Cell::new(state.config.settings.snapping.size_to_cells),
                guide_lines: std::cell::Cell::new(state.config.settings.snapping.guide_lines),
                backdrop: std::cell::Cell::new(backdrop_mode_for(state.config.settings.backdrop)),
                click_to_expand: std::cell::Cell::new(
                    state.config.settings.roll_up.click_to_expand,
                ),
                title_on_hover: std::cell::Cell::new(state.config.settings.roll_up.title_on_hover),
                title_align: std::cell::Cell::new(state.config.settings.title_align),
                tab_bar: std::cell::Cell::new(state.config.settings.tab_bar),
                hide_inactive_scrollbar: std::cell::Cell::new(
                    state.config.settings.roll_up.hide_inactive_scrollbar,
                ),
                switch_tab_on_hover: std::cell::Cell::new(
                    state.config.settings.roll_up.switch_tab_on_hover,
                ),
                wheel_lines: std::cell::Cell::new(sysparams::wheel_scroll_lines()),
                drop_preview: std::cell::RefCell::new(
                    crate::drop_preview::DropPreview::create()
                        .map_err(|e| tracing::warn!(error = %e, "drop preview unavailable"))
                        .ok(),
                ),
                drag_guides: std::cell::RefCell::new(
                    crate::drag_guides::DragGuides::create()
                        .map_err(|e| tracing::warn!(error = %e, "drag guides unavailable"))
                        .ok(),
                ),
                selection: std::cell::RefCell::new(Vec::new()),
                pushed: std::cell::RefCell::new(Vec::new()),
                size_move: std::cell::RefCell::new(None),
            }),
        });

        // Tray (the glyph uses the same accent token as the selection).
        let tray_px = (16.0 * (monitors::system_dpi().max(96) as f32 / 96.0)).round() as i32;
        let tray = tray::icon_from_bgra(
            tray_px,
            tray_px,
            &tray_icon_image(tray_px, theme.accent_rgb8(), theme_mode == ThemeMode::Dark),
        )
        .map(|icon| TrayIcon::new(control.hwnd(), TRAY_ID, WM_APP_TRAY, icon, "PecoFence"))
        .map_err(|e| tracing::warn!(error = %e, "tray icon image failed"))
        .ok();
        if let Some(t) = &tray
            && let Err(e) = t.show()
        {
            // At logon the taskbar may not exist yet; TaskbarCreated or the retry adds it.
            tracing::warn!(error = %e, "tray icon not added yet; retrying");
            window::set_timer(control.hwnd(), TIMER_TRAY_RETRY, TRAY_RETRY_MS);
        }

        // Watchers.
        let fs_pending: Arc<Mutex<Vec<FsEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let watchers =
            sync::start_desktop_watchers(sync::desktop_folders(), control.hwnd(), &fs_pending);

        let shell_watch = ShellChangeWatch::register_desktop(control.hwnd(), WM_APP_SHELL_CHANGED)
            .map_err(|e| tracing::warn!(error = %e, "shell change notifications unavailable"))
            .ok();

        let settings_class = SettingsHost::register_class()?;
        let peek_class = PeekOverlay::register_class()?;

        // CLI named pipe: requests land in `ipc_pending`, the control window is poked.
        let ipc_pending: PendingQueue = Arc::new(Mutex::new(VecDeque::new()));
        let ipc = match pecofence_platform::pipe::ipc_pipe_name(args.instance.as_deref()) {
            Ok(name) => IpcServer::start(&name, control.hwnd(), ipc_pending.clone()),
            Err(error) => {
                tracing::warn!(target: "pecofence::ipc", %error, "user SID unavailable; CLI disabled");
                None
            }
        };
        let rating = rating::RatingPrompt::load(
            state.config_path().parent().unwrap_or(Path::new(".")),
            pecofence_platform::process::is_packaged(),
        );

        let mut app = App {
            state,
            ctx,
            fences: HashMap::new(),
            anchor: anchor_cell,
            control,
            queue,
            tray,
            watchers,
            _shell_watch: shell_watch,
            portal_watchers: HashMap::new(),
            watcher_retry: HashMap::new(),
            fs_pending,
            fileops_done: Arc::new(Mutex::new(Vec::new())),
            fileops_running: 0,
            fileop_serial: 0,
            ipc_pending,
            _ipc: ipc,
            ipc_warnings: Vec::new(),
            ipc_subscribers: Vec::new(),
            ipc_event_state: None,
            instance: args.instance.clone(),
            settings: None,
            settings_focus_fence: None,
            feedback: Default::default(),
            updates: Default::default(),
            rating,
            welcome: None,
            settings_ring: None,
            web_env: None,
            settings_class,
            theme_mode,
            accent,
            wallpaper_override: args.wallpaper_override.clone(),
            pending_routes: Vec::new(),
            shell_drop_claim: None,
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
            peek_hits: Vec::new(),
            peek_hit: 0,
            peek_tabs: HashMap::new(),
            dying: Vec::new(),
            device_retry_pending: false,
            device_retry_ms: DEVICE_RETRY_FIRST_MS,
            gpu_baseline: None,
            gpu_rebuilt: None,
            test: None,
            marquee_base: None,
            marquee_band: None,
            marquee_draws_band: false,
            peek_hotkey: None,
            peek_hotkey_wanted: None,
            peek_hotkey_paused: false,
            desktop_unavailable: false,
            save_failure_shown: false,
            idle_rules_checked: None,
            memory_logged: None,
            cut_items: HashSet::new(),
            cut_clip_seq: 0,
            cut_paste: None,
            wallpaper_sig: None,
            wallpaper_cache: BackdropCache::default(),
            prewarm: None,
            startup_backdrops: None,
            live: Vec::new(),
            live_capture: None,
            live_abandoned: None,
        };
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
            a.on_peek_interrupted = Some(Box::new(move || {
                q.push(Command::EndPeek("another window came to the front"))
            }));
            let control_hwnd = app.control.hwnd().0 as isize;
            a.on_foreground_changed = Some(Box::new(move |foreground| {
                window::post_message(
                    HWND(control_hwnd as *mut core::ffi::c_void),
                    WM_APP_WALLPAPER,
                    1,
                    foreground.0 as isize,
                );
            }));
            // A live wallpaper's backdrop catches up with what the desktop shows now.
            a.on_desktop_front = Some(Box::new(move || {
                window::post_message(
                    HWND(control_hwnd as *mut core::ffi::c_void),
                    WM_APP_LIVE_WALLPAPER,
                    1,
                    0,
                );
            }));
        }

        if let Some(moved) = &app.state.unreadable_moved_to
            && let Some(t) = &app.tray
        {
            t.show_info(
                "PecoFence",
                &pecofence_core::i18n::format(
                    "无法读取配置文件（可能是较新版本的 PecoFence 写入的），原文件已改名为 {0}。",
                    &[moved
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()],
                ),
                true,
            );
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
        // Installed before the first fence draws, and armed once: an idle startup with no
        // later draw must still release its scratch memory and hand its pages back.
        let control = app.control.hwnd();
        pecofence_render::stack::set_draw_hook(move || {
            window::set_timer(control, TIMER_GPU_TRIM, GPU_TRIM_IDLE_MS);
            window::set_timer(control, TIMER_IDLE_TRIM, IDLE_TRIM_MS);
        });
        window::set_timer(control, TIMER_GPU_TRIM, GPU_TRIM_IDLE_MS);
        window::set_timer(control, TIMER_IDLE_TRIM, IDLE_TRIM_MS);
        // Initial desktop sync + windows.
        app.sync_desktop_if_available("startup");
        app.state.refresh_all_portals();
        let mut startup_snapshot = None;
        match startup_backdrops.finish(&app.ctx.theme.borrow(), app.control.hwnd()) {
            Some(Ok((backdrops, snapshot, signature))) => {
                *app.ctx.backdrops.borrow_mut() = backdrops.clone();
                app.wallpaper_cache.insert(signature.clone(), backdrops);
                app.wallpaper_sig = Some(signature);
                startup_snapshot = Some(snapshot);
            }
            Some(Err(error)) => {
                tracing::warn!(%error, "wallpaper unavailable at startup; solid backdrop");
            }
            // Explorer is slow to answer: the fences go up now, the glass follows.
            None => {
                startup_backdrops.generation = app.wallpaper_cache.generation();
                app.startup_backdrops = Some(startup_backdrops);
            }
        }
        app.sync_fence_windows();
        if app.state.first_run {
            app.fit_first_run_column();
        }
        // Fences never stay on top of each other: pull apart any overlap the layout brings.
        app.settle_unplaced(true);
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
                            "上次 PecoFence 没有正常退出，桌面图标没能恢复显示。现在已重新接管。",
                        ),
                        true,
                    );
                }
            }
            if app.state.config.settings.hide_real_icons && !app.no_hide_icons {
                // At logon Explorer's desktop may not be ready yet: retried from the anchor's
                // periodic check; housekeeping tells the user if it never succeeds.
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
        // What startup changed is derived again on the next start: write it once the fences
        // are up instead of before the message loop runs.
        if app.state.is_dirty() {
            app.schedule_save();
        }
        window::set_coalescable_timer(app.control.hwnd(), TIMER_HOUSEKEEPING, 60_000, 5_000);
        window::set_coalescable_timer(app.control.hwnd(), TIMER_WALLPAPER_POLL, 5_000, 500);
        sync_desktops_differ(app.control.hwnd(), &desktops_differ);
        // Close the gap between the startup snapshot and installing the registry watchers.
        if let Some(signature) = app.wallpaper_sig.clone() {
            // One check (no signal arrived, so no follow-ups), then prepare the other desktops.
            // The check asks Explorer from a worker: it is busy hiding the desktop icons right
            // now (0.8 s for 1000 icons), and the UI thread would wait on it that long. The
            // rest of the check (live wallpapers) runs on the startup read right away.
            verify_wallpaper_later(app.control.hwnd(), signature);
            if let Some(snapshot) = &startup_snapshot {
                app.follow_snapshot(snapshot, "startup");
            }
            window::set_timer(
                app.control.hwnd(),
                TIMER_WALLPAPER_PREWARM,
                WALLPAPER_PREWARM_DELAY_MS,
            );
        } else if app.startup_backdrops.is_none() {
            // Not readable yet (Explorer still starting): keep trying as for a signal.
            window::post_message(app.control.hwnd(), WM_APP_WALLPAPER, 0, 0);
        }

        app.remove_old_update_downloads();
        // Last of the startup balloons, so it is the one left showing.
        app.offer_crash_feedback();
        if app.state.first_run && args.exit_after_ms.is_none() && args.test_script.is_none() {
            app.offer_welcome();
        }
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
                Ok(m) => tracing::info!(memory = %m.summary(), "startup memory"),
                Err(e) => tracing::warn!(error = %e, "memstats failed"),
            }
        }
        *cell.borrow_mut() = Some(app);
        Ok(cell)
    }

    fn schedule_save(&self) {
        window::set_timer(self.control.hwnd(), TIMER_SAVE, 800);
    }

    /// After a save: one that failed leaves the state dirty. The first time in a run the tray
    /// says so, since the changes are lost at exit and only the log had the reason.
    fn notice_save_failure(&mut self) {
        if !self.state.is_dirty() || self.save_failure_shown {
            return;
        }
        let Some(t) = &self.tray else { return };
        self.save_failure_shown = true;
        let path = self.state.config_path_on_disk();
        t.show_info(
            "PecoFence",
            &pecofence_core::i18n::format(
                "无法保存配置，退出后更改会丢失。请检查这个文件夹能否写入：{0}",
                &[path.parent().unwrap_or(&path).display().to_string()],
            ),
            true,
        );
    }

    fn housekeeping(&mut self) {
        self.check_cut_clipboard();
        if self.state.is_dirty() {
            self.state.save_if_dirty();
            self.notice_save_failure();
        }
        // Icons no window needs any more, if a change left some behind without a redraw; the
        // GPU trim then frees the released uploads.
        if self.prune_icons() {
            window::set_timer(self.control.hwnd(), TIMER_GPU_TRIM, GPU_TRIM_IDLE_MS);
        }
        self.revive_watchers();
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
            let moved = self.state.apply_idle_rules(&entries);
            if moved > 0 {
                tracing::info!(moved, "idle-days rules re-applied");
                self.refresh_all();
                self.schedule_save();
            }
        }
        // A DPI that settled late re-lays the window out; its rectangle stays (it still shows
        // whole icons: the columns share the rest, the rows leave it empty).
        for window in self.fences.values() {
            window.check_dpi();
        }
        // Startup hide that kept failing: stop retrying for this run and tell the user. The
        // setting stays on, so the next start (or turning it off and on) tries again.
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
                    "hiding desktop icons kept failing after startup; trying again at the next start"
                );
                if let Some(t) = &self.tray {
                    t.show_info(
                        "PecoFence",
                        pecofence_core::i18n::text("无法隐藏桌面图标，请稍后再试。"),
                        true,
                    );
                }
            }
        }
        self.offer_rating_if_due();
        self.log_memory();
    }

    /// One line with the process's memory: on the first housekeeping tick, then hourly, and
    /// whenever the total grew by `MEMORY_LOG_GROWTH` since the last line, so a log sent with
    /// a high Task Manager number shows when it grew and in which kind of memory.
    fn log_memory(&mut self) {
        let Ok(m) = pecofence_platform::memstats::MemoryStats::current() else {
            return;
        };
        let gpu = self.ctx.stack.gpu_memory();
        let (gpu_local, gpu_shared) = gpu.unwrap_or_default();
        let total = m.private_commit as u64 + gpu_local + gpu_shared;
        let due = match self.memory_logged {
            None => true,
            Some((at, logged)) => {
                at.elapsed() >= Duration::from_secs(3600)
                    || total >= logged.saturating_add(MEMORY_LOG_GROWTH)
            }
        };
        if !due {
            return;
        }
        self.memory_logged = Some((Instant::now(), total));
        let mb = |b: u64| (b as f64 / (1u64 << 20) as f64 * 10.0).round() / 10.0;
        // The GPU fields are left out when the query failed: no zero that looks measured.
        tracing::info!(
            private_ws_mb = mb(m.private_working_set as u64),
            commit_mb = mb(m.private_commit as u64),
            gpu_local_mb = gpu.map(|(local, _)| mb(local)),
            gpu_shared_mb = gpu.map(|(_, shared)| mb(shared)),
            "memory"
        );
    }

    /// Drains and executes queued commands. Loops because handlers may enqueue more.
    pub fn process_commands(&mut self) {
        self.check_cut_clipboard();
        self.drain_fileops();
        self.drain_ipc();
        self.drain_updates();
        for _ in 0..8 {
            let cmds = self.queue.drain();
            if cmds.is_empty() {
                break;
            }
            for cmd in cmds {
                // A command that blocks the UI thread for more than a compositor frame delays
                // the first frame of any animation it started (time-based tweens then appear
                // to jump ahead): name it.
                let name = command_name(&cmd);
                let started = Instant::now();
                let in_menus = tray::time_in_menus();
                self.handle(cmd);
                // A menu it opened waited for the user, which is no delay of ours.
                let spent = started
                    .elapsed()
                    .saturating_sub(tray::time_in_menus() - in_menus);
                if spent > Duration::from_millis(8) {
                    tracing::info!(
                        ms = spent.as_secs_f32() * 1000.0,
                        command = name,
                        "slow command"
                    );
                }
            }
        }
        // After the batch, however it ended: heights it changed push the fences below now.
        self.reflow_pushed();
    }

    fn handle(&mut self, cmd: Command) {
        match cmd {
            Command::FenceBoundsChanged { fence, rect } => {
                let (rolled, expanded) = self
                    .fences
                    .get(&fence)
                    .map(|w| (w.is_rolled(), w.expanded_height_px()))
                    .unwrap_or((false, rect.bottom - rect.top));
                self.record_bounds(fence, rect, rolled, expanded);
                self.schedule_save();
                self.apply_auto_height(fence);
            }
            Command::FenceDropped(fence) => self.move_out_of_overlap(fence),
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
            Command::LaunchItem { item, from } => self.launch(item, from),
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
            Command::MergeHint { target, at } => {
                for w in self.fences.values() {
                    w.set_merge_hint(!target.0.is_null() && w.hwnd() == target, at);
                }
            }
            Command::MergeFence { fence, into, at } => {
                // The slot the gap sat at (read before the hints — and the gap — are cleared).
                let host = self.fence_of_hwnd(into);
                let index = host
                    .and_then(|h| self.fences.get(&h))
                    .map(|w| w.merge_slot_at(at));
                for w in self.fences.values() {
                    w.set_merge_hint(false, (0, 0));
                }
                // The dragged window's tabs in their order (attach_tab adds them in storage
                // order, which may interleave them with the target's tabs).
                let moved = self.state.tabs_of(fence);
                if let Some(host) = host
                    && host != fence
                    && self.state.attach_tab(fence, host)
                {
                    if let Some(index) = index {
                        // Land where the gap was, so the neighbours do not move again, and
                        // keep the dragged window's tabs together in their order. First to
                        // the end in order (a host without a hand-made order lists them in
                        // creation order, possibly ahead of the gap), then into the gap.
                        for tab in &moved {
                            self.state.reorder_tab(host, *tab, usize::MAX);
                        }
                        for (k, tab) in moved.iter().enumerate() {
                            self.state.reorder_tab(host, *tab, index.saturating_add(k));
                        }
                    }
                    self.resync_windows();
                    self.apply_fence_view(host);
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
                        // A desktop file of the same name is not the one being dropped: a
                        // route would move it into `to` as soon as any watcher batch arrives,
                        // before the conflict dialog is even answered.
                        let routed: Vec<PathBuf> = foreign
                            .iter()
                            .filter_map(|p| p.file_name().map(|n| desktop.join(n)))
                            .filter(|p| self.state.live_item_by_path(p).is_none())
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
            Command::ExternalUrlDrop {
                url,
                name,
                to,
                folder,
            } => self.create_url_shortcut(url, name, to, folder),
            Command::ShellDrop {
                to,
                landed,
                on_desktop,
            } => self.shell_drop_landed(to, landed, on_desktop),
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
            Command::DesktopMarquee(event) => self.on_desktop_marquee(event),
            Command::ClearFenceSelection => self.set_fence_selection(Vec::new()),
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
            Command::RenameItemInvalid => self.invalid_name_toast(),
            Command::EndItemRename { commit } => crate::rename::end_active(commit),
            Command::DeleteItems {
                fence,
                items,
                permanent,
            } => self.delete_items(fence, &items, permanent),
            Command::DeviceLost => self.recover_device(),
            Command::ToggleAllFences => self.toggle_all_fences(),
            Command::TogglePeek => self.toggle_peek(),
            Command::EndPeek(reason) => self.end_peek(reason),
            Command::PeekSearchType(text) => self.peek_search_type(&text),
            Command::PeekSearchChanged => self.update_peek_search(),
            Command::PeekSearchStep(step) => self.step_peek_search(step),
            Command::PeekSearchOpen => self.open_peek_hit(),
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
                    self.set_settings_ring(None);
                    self.resume_peek_hotkey();
                }
            }
            Command::FadeOutDone(hwnd) => {
                // Dropping the window destroys it (RevokeDragDrop + DestroyWindow).
                self.dying.retain(|w| w.hwnd() != hwnd);
            }
            Command::SettingsMessage(json) => self.on_settings_message(&json),
            Command::Notice(event) => self.on_notice(event),
        }
    }

    /// The session did not end after all: hide the real desktop icons again if the setting
    /// wants them hidden (WM_QUERYENDSESSION restored them, as for a real logoff).
    fn shutdown_cancelled(&mut self) {
        tracing::info!("shutdown cancelled");
        if self.state.config.settings.hide_real_icons
            && !self.no_hide_icons
            && let Some(a) = self.anchor.borrow_mut().as_mut()
        {
            a.request_hide_desktop_icons();
        }
    }

    /// Explorer (re)started: the new Explorer knows nothing of the old shell change
    /// registration. Its id is not deregistered: the new Explorer may have handed the same
    /// number to another program, or to one of its own views. Should the old registration
    /// still be alive, the duplicate notices merge in the same 60 ms batch.
    fn renew_shell_watch(&mut self) {
        std::mem::forget(self._shell_watch.take());
        self._shell_watch =
            ShellChangeWatch::register_desktop(self.control.hwnd(), WM_APP_SHELL_CHANGED)
                .map_err(|e| tracing::warn!(error = %e, "shell change notifications unavailable"))
                .ok();
    }

    /// (Re)adds the notification icon; false while the taskbar does not take it.
    fn show_tray(&self) -> bool {
        let Some(t) = &self.tray else {
            return true;
        };
        match t.show() {
            Ok(()) => true,
            Err(e) => {
                tracing::debug!(error = %e, "tray icon not added");
                false
            }
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
