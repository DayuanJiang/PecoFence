//! On-demand WebView2 settings window (plan §9): created when opened, destroyed when closed so
//! the browser processes go away and nothing stays resident. The smart fence editor
//! (智能栅栏编辑器, `HostKind::SmartEditor`) is the same page in a small window of its own that
//! shows only the editor.
//!
//! IPC: the page posts JSON (`{"type": "ready" | "patchSettings" | "setRules" | "action"}`);
//! the host pushes `{"type": "state", ...}` / `{"type": "toast", ...}` with `post_json`.

use crate::commands::{Command, CommandQueue};
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, RECT, desktop, dwm, monitors, msg};
use pecofence_render::ThemeMode;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use windows_core::Result;
use windows_webview::{Controller, Environment, EnvironmentOptions, WebView};

pub const SETTINGS_CLASS: &str = "PecoFence.Settings";
pub const SMART_EDITOR_CLASS: &str = "PecoFence.SmartEditor";

/// Which window the host is: the settings, or the smart fence editor. The editor's page
/// messages come as `Command::EditorMessage`, the settings page's as `SettingsMessage`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostKind {
    Settings,
    SmartEditor,
}
const SETTINGS_HTML: &str = include_str!("../../../ui/settings.html");
const I18N_JS: &str = include_str!("../../../ui/i18n.js");

fn profile_name(instance: Option<&str>) -> String {
    use std::hash::{Hash, Hasher};
    let Some(instance) = instance.map(str::trim).filter(|name| !name.is_empty()) else {
        return "default".into();
    };
    // Instance names are not filesystem paths. Hash them to avoid separators,
    // invalid Windows filename characters and collisions from lossy sanitizing.
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    instance.hash(&mut hash);
    format!("instance-{:016x}", hash.finish())
}

/// The WebView2 environment. Kept alive across opens (warm start ≈ 200 ms); the browser
/// processes exit by themselves when no controller is alive.
pub struct WebEnvironment {
    env: Environment,
}

impl WebEnvironment {
    pub fn create() -> Result<Self> {
        let user_data = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("PecoFence")
            // Keep independent instances from sharing incompatible controller options.
            .join("WebView2Profiles")
            .join(profile_name(
                pecofence_core::brand::var("PECOFENCE_INSTANCE")
                    .ok()
                    .as_deref(),
            ));
        let _ = std::fs::create_dir_all(&user_data);
        let mut options =
            EnvironmentOptions::new().user_data_folder(user_data.to_string_lossy().to_string());
        // Debug builds: `PECOFENCE_WEBVIEW_ARGS=--remote-debugging-port=9352` lets a test drive
        // the page (the options always pass browser arguments, so WebView2's own
        // WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS is not honoured).
        if cfg!(debug_assertions)
            && let Ok(args) = pecofence_core::brand::var("PECOFENCE_WEBVIEW_ARGS")
        {
            options = options.additional_browser_arguments(args);
        }
        tracing::debug!("settings: creating WebView2 environment");
        let env = Environment::with_options(&options)?;
        tracing::debug!("settings: WebView2 environment ready");
        Ok(Self { env })
    }
}

struct HostState {
    controller: Option<Controller>,
    webview: Option<WebView>,
    /// Where the window stood when it closed and its scale there (`SettingsHost::closed_at`).
    closed_at: Option<(RECT, f32)>,
}

/// `WM_TIMER` id of the host window: shows a window whose page never said `drawn`.
const SHOW_TIMER: usize = 1;
/// How long after `ready` a window shows anyway (a script error must not keep it hidden).
const SHOW_FALLBACK_MS: u32 = 500;

pub struct SettingsHost {
    window: Window,
    state: Rc<RefCell<HostState>>,
    /// DWM Mica is active behind a transparent page.
    mica: bool,
    /// The page reported `ready`: messages posted from now on reach its script.
    pub ready: bool,
    /// When `open` started: the page's load time goes to the log.
    pub opened: std::time::Instant,
    /// Caption icons (small/big); destroyed with the host.
    icons: Vec<pecofence_platform::tray::OwnedIcon>,
    _registrations: Vec<windows_webview::EventRegistration>,
}

impl SettingsHost {
    pub fn register_class(kind: HostKind) -> Result<WindowClass> {
        let name = match kind {
            HostKind::Settings => SETTINGS_CLASS,
            HostKind::SmartEditor => SMART_EDITOR_CLASS,
        };
        WindowClass::register(name, ClassOptions::default())
    }

    /// Creates the window and the WebView2 controller and loads the settings page. Page
    /// messages are forwarded to the command queue as `Command::SettingsMessage` (the editor's
    /// as `Command::EditorMessage`). The settings open on the primary monitor, the editor on
    /// the monitor of `near` (the point its menu opened at), centred on its work area. `last`:
    /// where this run's last settings window was closed (its rectangle and scale); the window
    /// opens there again.
    #[allow(clippy::too_many_arguments)]
    pub fn open(
        class: &WindowClass,
        env: &WebEnvironment,
        mode: ThemeMode,
        _liquid_glass: bool,
        queue: CommandQueue,
        kind: HostKind,
        title: &str,
        near: Option<(i32, i32)>,
        last: Option<(RECT, f32)>,
    ) -> Result<Self> {
        let opened = std::time::Instant::now();
        let state = Rc::new(RefCell::new(HostState {
            controller: None,
            webview: None,
            closed_at: None,
        }));

        let mons = monitors::enumerate();
        let primary = mons.iter().find(|m| m.primary).or(mons.first());
        let last_centre = last.map(|(r, _)| ((r.left + r.right) / 2, (r.top + r.bottom) / 2));
        let monitor = near
            .or(last_centre)
            .and_then(|(x, y)| {
                mons.iter().find(|m| {
                    let r = m.work_area;
                    x >= r.left && x < r.right && y >= r.top && y < r.bottom
                })
            })
            .or(primary);
        let scale = monitor.map(|m| m.scale()).unwrap_or(1.0);
        let place_on = move |work: RECT, scale: f32| match kind {
            HostKind::Settings => placement(work, scale),
            HostKind::SmartEditor => editor_placement(work, scale),
        };
        let place = monitor.map(|m| {
            let p = place_on(m.work_area, scale);
            match last {
                Some((rect, last_scale)) => {
                    let to = p.reopened(rect, last_scale, m.work_area, scale);
                    tracing::debug!(
                        ?rect,
                        last_scale,
                        work = ?m.work_area,
                        scale,
                        ?to,
                        "settings: window reopens where it was closed"
                    );
                    to
                }
                None => p,
            }
        });
        let (min_w, min_h) = place
            .map(|p| (p.min_w, p.min_h))
            .unwrap_or(((560.0 * scale) as i32, (420.0 * scale) as i32));

        let handler: MessageHandler = {
            let state = state.clone();
            let queue = queue.clone();
            Box::new(
                move |hwnd: HWND, message: u32, wparam: usize, lparam: isize| -> Option<isize> {
                    match message {
                        msg::WM_SIZE => {
                            let w = msg::lo_i16(lparam);
                            let h = msg::hi_i16(lparam);
                            if let Some(c) = state.borrow().controller.as_ref() {
                                let _ = c.set_bounds(0, 0, w, h);
                                let _ = c.set_visible(w > 0 && h > 0);
                            }
                            Some(0)
                        }
                        msg::WM_MOVE => {
                            if let Some(c) = state.borrow().controller.as_ref() {
                                let _ = c.notify_parent_window_position_changed();
                            }
                            Some(0)
                        }
                        // Never paint the client area: DWM's Mica shows through (and no grey flash).
                        msg::WM_ERASEBKGND => Some(1),
                        msg::WM_GETMINMAXINFO => {
                            // For the monitor the window is on now, which may have another
                            // scale than the one it opened on.
                            let (min_w, min_h) =
                                monitors::query(monitors::monitor_from_window(hwnd))
                                    .map(|m| place_on(m.work_area, m.scale()))
                                    .map_or((min_w, min_h), |p| (p.min_w, p.min_h));
                            // SAFETY: lParam is the MINMAXINFO for this message.
                            unsafe { window::minmaxinfo_set_min_track(lparam, min_w, min_h) };
                            Some(0)
                        }
                        msg::WM_DPICHANGED => {
                            // A per-monitor aware window resizes itself: take the size Windows
                            // suggests for the new scale.
                            // SAFETY: lParam points to the suggested RECT for the duration of
                            // the message.
                            let r = unsafe { *(lparam as *const RECT) };
                            let _ = window::set_window_bounds(
                                hwnd,
                                r.left,
                                r.top,
                                r.right - r.left,
                                r.bottom - r.top,
                            );
                            Some(0)
                        }
                        msg::WM_SETFOCUS => {
                            // Keyboard focus belongs to the page, not the host window.
                            if let Some(c) = state.borrow().controller.as_ref() {
                                let _ =
                                    c.move_focus(windows_webview::MoveFocusReason::Programmatic);
                            }
                            Some(0)
                        }
                        msg::WM_TIMER if wparam == SHOW_TIMER => {
                            window::kill_timer(hwnd, SHOW_TIMER);
                            if !desktop::is_visible(hwnd) {
                                tracing::warn!(
                                    ?kind,
                                    "settings: the page did not say it has drawn, window shown anyway"
                                );
                                window::show_normal(hwnd);
                                window::bring_to_front(hwnd);
                            }
                            Some(0)
                        }
                        msg::WM_ACTIVATE if wparam & 0xffff != 0 && kind == HostKind::Settings => {
                            queue.push(Command::SettingsActivated);
                            None
                        }
                        msg::WM_DESTROY => {
                            // Where the window stood, for the next open. A maximized or
                            // minimized window has no place of its own to keep.
                            const WS_MAXIMIZE: u32 = 0x0100_0000;
                            const WS_MINIMIZE: u32 = 0x2000_0000;
                            let normal = desktop::style(hwnd) & (WS_MAXIMIZE | WS_MINIMIZE) == 0;
                            let closed_at = normal.then(|| {
                                let scale = monitors::dpi_for_window(hwnd).max(96) as f32 / 96.0;
                                (window::window_rect(hwnd), scale)
                            });
                            // Taken out of the state first: closing the browser may let a
                            // message sent by another thread in, whose handling reads the
                            // state (GitHub #166).
                            let (webview, controller) = {
                                let mut s = state.borrow_mut();
                                s.closed_at = closed_at;
                                (s.webview.take(), s.controller.take())
                            };
                            drop(webview);
                            if let Some(c) = controller {
                                let _ = c.close();
                            }
                            queue.push(Command::WindowGone(hwnd));
                            Some(0)
                        }
                        _ => None,
                    }
                },
            )
        };

        let (x, y, w, h) = place.map(|p| (p.x, p.y, p.w, p.h)).unwrap_or((
            100,
            100,
            (960.0 * scale) as i32,
            (660.0 * scale) as i32,
        ));

        let window = WindowBuilder::new(class)
            .title(title)
            .style(style::OVERLAPPEDWINDOW | style::CLIPCHILDREN)
            .bounds(x, y, w, h)
            .create(handler)?;
        let hwnd = window.hwnd();
        tracing::debug!(?hwnd, "settings: host window created");
        let _ = dwm::set_immersive_dark_mode(hwnd, matches!(mode, ThemeMode::Dark));
        apply_caption(hwnd, mode, false);
        // A transparent page needs the WebView2 *composition* controller (HWND children cannot be
        // per-pixel transparent); windows-webview 0.100 only wraps the HWND controller, so the
        // Mica sheet stays behind an experiment flag and the page paints the theme's base colour.
        let mica = pecofence_core::brand::var_os("PECOFENCE_SETTINGS_MICA").is_some()
            && dwm::set_system_backdrop(hwnd, dwm::SystemBackdrop::MainWindow).is_ok();
        apply_caption(hwnd, mode, mica);

        // SAFETY: `hwnd` is a live window owned by this thread and outlives the controller
        // (the controller is closed in WM_DESTROY).
        let controller = unsafe {
            env.env
                .create_controller_for_hwnd(pecofence_platform::hwnd_ptr(hwnd))?
        };
        tracing::debug!("settings: WebView2 controller ready");
        let webview = controller.webview()?;
        let (cw, ch) = window.client_size();
        controller.set_bounds(0, 0, cw, ch)?;
        // Transparent page over DWM Mica; opaque theme colour when Mica is unavailable.
        controller.set_default_background_color(page_background(mode, mica))?;
        if let Ok(settings) = webview.settings() {
            let _ = settings.set_default_context_menus_enabled(false);
            let _ = settings.set_status_bar_enabled(false);
            let _ = settings.set_zoom_control_enabled(false);
            // The browser's own keys are not the window's: F5 reloaded the page (back on 常规,
            // a half-typed rule gone), F12 opened DevTools, F3, F7, Ctrl+J and Ctrl+U opened
            // browser UI. The page still gets every key (Ctrl+F for 查找设置).
            let _ = settings.set_browser_accelerator_keys_enabled(false);
            // DevTools stay for a debug build that a test drives over the debugging port.
            let dev_tools = cfg!(debug_assertions)
                && pecofence_core::brand::var("PECOFENCE_WEBVIEW_ARGS").is_ok();
            let _ = settings.set_dev_tools_enabled(dev_tools);
            tracing::debug!(dev_tools, "settings: browser keys off");
        }

        let mut registrations = Vec::new();
        {
            let queue = queue.clone();
            registrations.push(webview.on_web_message_received(move |args| {
                let json = args.web_message_as_json();
                queue.push(match kind {
                    HostKind::Settings => Command::SettingsMessage(json),
                    HostKind::SmartEditor => Command::EditorMessage(json),
                });
            })?);
        }
        // Tell the page whether it sits on Mica before it renders (opaque fallback class).
        let payload = pecofence_core::i18n::ui_payload()
            .to_string()
            .replace('<', "\\u003c");
        // The editor's page says what it is: the page shows only the editor, and a test finds
        // it among the browser's pages by its title.
        let editor = match kind {
            HostKind::Settings => "",
            HostKind::SmartEditor => "window.PECOFENCE_MODE='smartEditor';",
        };
        let document = SETTINGS_HTML
            .replace(
                "<!-- PECOFENCE_LOCALE -->",
                &format!("<script>window.PECOFENCE_LOCALE={payload};{editor}</script>"),
            )
            .replace(
                "<script src=\"i18n.js\"></script>",
                &format!("<script>{I18N_JS}</script>"),
            );
        let document = match kind {
            HostKind::Settings => document,
            HostKind::SmartEditor => document.replacen(
                "<title>PecoFence 设置</title>",
                "<title>PecoFence smart fence editor</title>",
                1,
            ),
        };
        // The colour mode too, so that the page's first style pass is already in it (the state
        // brings it again and follows a later switch).
        let mut classes = Vec::new();
        if !mica {
            classes.push("no-mica");
        }
        if mode == ThemeMode::Light {
            classes.push("light");
        }
        let html = document.replacen(
            "<html lang=\"zh-CN\">",
            &format!("<html lang=\"zh-CN\" class=\"{}\">", classes.join(" ")),
            1,
        );
        webview.navigate_to_string(&html)?;
        tracing::debug!("settings: page navigation submitted");

        {
            let mut s = state.borrow_mut();
            s.controller = Some(controller);
            s.webview = Some(webview);
        }
        // The controller is created while the host is still hidden and stays invisible until
        // told. The host itself is shown once the page reports `drawn` (`App::on_settings_message`):
        // shown at once, it was white until WebView2 painted, even in dark mode; shown on
        // `ready`, it was black for a frame and had the page's built-in values and dark colours
        // for another.
        if let Some(c) = state.borrow().controller.as_ref() {
            let _ = c.set_visible(true);
        }
        Ok(Self {
            window,
            state,
            mica,
            ready: false,
            opened,
            icons: Vec::new(),
            _registrations: registrations,
        })
    }

    pub fn hwnd(&self) -> HWND {
        self.window.hwnd()
    }

    /// After the window was closed: where it stood (screen px) and the scale of its monitor.
    /// `None` when it was closed maximized or minimized.
    pub fn closed_at(&self) -> Option<(RECT, f32)> {
        self.state.borrow().closed_at
    }

    /// The window's caption (the settings' after a language change, the editor's for the fence
    /// it shows).
    pub fn set_title(&self, title: &str) {
        window::set_title(self.window.hwnd(), title);
    }

    /// The page is loaded and has been sent what it shows: the window appears when the page
    /// says it has drawn that (`show`), or after `SHOW_FALLBACK_MS` when it never says so.
    pub fn show_soon(&self) {
        window::set_timer(self.window.hwnd(), SHOW_TIMER, SHOW_FALLBACK_MS);
    }

    /// Shows the window for the first time. False when it was showing already.
    pub fn show(&self) -> bool {
        let hwnd = self.window.hwnd();
        window::kill_timer(hwnd, SHOW_TIMER);
        if desktop::is_visible(hwnd) {
            return false;
        }
        window::show_normal(hwnd);
        window::bring_to_front(hwnd);
        true
    }

    /// 栅栏选项…: moves the window beside `fence` (screen px) on the fence's monitor, on the
    /// side with room for it, so the fence stays in view while its options change. A
    /// maximized window stays as it is.
    pub fn place_beside(&self, fence: RECT) {
        const WS_MAXIMIZE: u32 = 0x0100_0000;
        let hwnd = self.window.hwnd();
        if desktop::style(hwnd) & WS_MAXIMIZE != 0 {
            tracing::info!("settings window maximized: not moved beside the fence");
            return;
        }
        let (cx, cy) = (
            (fence.left + fence.right) / 2,
            (fence.top + fence.bottom) / 2,
        );
        let Some(m) = monitors::query(monitors::monitor_from_point(cx, cy)) else {
            return;
        };
        let scale = m.scale();
        let p = placement(m.work_area, scale);
        let cur = window::window_rect(hwnd);
        let size = (cur.right - cur.left, cur.bottom - cur.top);
        let r = beside(fence, m.work_area, size, p.min_w, (12.0 * scale) as i32);
        tracing::info!(?fence, ?r, work = ?m.work_area, "settings window placed beside the fence");
        let _ = window::set_window_bounds(hwnd, r.left, r.top, r.right - r.left, r.bottom - r.top);
    }

    /// Sets the window's caption icons (`WM_SETICON` small + big) from premultiplied BGRA.
    pub fn set_icons(&mut self, small: (i32, Vec<u8>), big: (i32, Vec<u8>)) {
        const WM_SETICON: u32 = 0x0080;
        let mut icons = Vec::new();
        for (which, (px, bgra)) in [(0usize, small), (1usize, big)] {
            if let Ok(icon) = pecofence_platform::tray::OwnedIcon::from_bgra(px, &bgra) {
                window::send_message(self.window.hwnd(), WM_SETICON, which, icon.raw());
                icons.push(icon);
            }
        }
        self.icons = icons;
    }

    /// Re-themes the caption and the page background after a light/dark switch. The sheet is
    /// the same for both materials: an opaque window has nothing for Liquid Glass to refract.
    pub fn set_theme(&self, mode: ThemeMode, _liquid_glass: bool) {
        let _ = dwm::set_immersive_dark_mode(self.window.hwnd(), matches!(mode, ThemeMode::Dark));
        dwm::repaint_title_bar(self.window.hwnd());
        apply_caption(self.window.hwnd(), mode, self.mica);
        if let Some(c) = self.state.borrow().controller.as_ref() {
            let _ = c.set_default_background_color(page_background(mode, self.mica));
        }
    }

    pub fn post_json(&self, json: &str) {
        if let Some(w) = self.state.borrow().webview.as_ref() {
            let _ = w.post_web_message_as_json(json);
        }
    }

    /// Closes the browser and destroys the window. The browser is taken out of the state
    /// before it closes: closing it sends the window messages (focus, size) whose handling
    /// reads the state.
    pub fn close(self) {
        let (webview, controller) = {
            let mut s = self.state.borrow_mut();
            (s.webview.take(), s.controller.take())
        };
        drop(webview);
        if let Some(c) = controller {
            let _ = c.close();
        }
        drop(self.window);
    }
}

/// Page background: fully transparent over Mica, else the theme's solid base colour.
fn page_background(mode: ThemeMode, mica: bool) -> windows_webview::Color {
    if mica {
        return windows_webview::Color {
            a: 0,
            r: 0,
            g: 0,
            b: 0,
        };
    }
    // Keep these RGB values in sync with --sheet in the embedded settings document.
    let (r, g, b) = match mode {
        ThemeMode::Dark => (0x20, 0x20, 0x20),
        ThemeMode::Light => (0xF3, 0xF3, 0xF3),
    };
    windows_webview::Color { a: 255, r, g, b }
}

/// Match the page sheet so the native caption belongs to the same surface.
fn apply_caption(hwnd: HWND, mode: ThemeMode, mica: bool) {
    if mica {
        return;
    }
    let bg = page_background(mode, false);
    // DWM takes COLORREF (0x00BBGGRR).
    let c = u32::from(bg.r) | (u32::from(bg.g) << 8) | (u32::from(bg.b) << 16);
    let _ = dwm::set_caption_color(hwnd, c);
}

/// Where the settings window opens on a work area (physical px) at `scale`, and its minimum
/// tracking size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Placement {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    min_w: i32,
    min_h: i32,
}

impl Placement {
    /// The window as it was closed: `last` is its rectangle then, on a monitor at `last_scale`.
    /// The size follows this monitor's scale and stays between the minimum and the work area.
    /// The place is kept while the rectangle's centre is on this work area (moved inside it);
    /// when the monitor it stood on is gone, the window is centred as on a first open.
    fn reopened(self, last: RECT, last_scale: f32, work: RECT, scale: f32) -> Placement {
        let (area_w, area_h) = (work.right - work.left, work.bottom - work.top);
        let rescaled = |px: i32| (px as f32 * scale / last_scale).round() as i32;
        let w = rescaled(last.right - last.left).clamp(self.min_w, area_w);
        let h = rescaled(last.bottom - last.top).clamp(self.min_h, area_h);
        let (cx, cy) = ((last.left + last.right) / 2, (last.top + last.bottom) / 2);
        let on_work = cx >= work.left && cx < work.right && cy >= work.top && cy < work.bottom;
        let (x, y) = if on_work {
            (
                last.left.clamp(work.left, work.right - w),
                last.top.clamp(work.top, work.bottom - h),
            )
        } else {
            (work.left + (area_w - w) / 2, work.top + (area_h - h) / 2)
        };
        Placement { x, y, w, h, ..self }
    }
}

/// The smart fence editor: 880 x 660 DIP (minimum 560 x 420), centred on the work area, and
/// smaller on a work area that has no room for it. That size holds the template with the most
/// rows (最近一周的截图: five, two of them with the type buttons) and every kind of row on one
/// line in each interface language (`tests/owner-smart-editor.ps1`); 680 x 560 scrolled its list
/// in all of them and pushed 选择… or 取消 and 创建 to a second line in five.
fn editor_placement(work: RECT, scale: f32) -> Placement {
    let (area_w, area_h) = (work.right - work.left, work.bottom - work.top);
    let w = ((880.0 * scale) as i32).min(area_w);
    let h = ((660.0 * scale) as i32).min(area_h);
    Placement {
        x: work.left + (area_w - w) / 2,
        y: work.top + (area_h - h) / 2,
        w,
        h,
        min_w: ((560.0 * scale) as i32).min(w),
        min_h: ((420.0 * scale) as i32).min(h),
    }
}

/// The designed 960 x 660 DIP window (minimum 720 x 480), centred on the work area. Both
/// shrink to fit it: 1366 x 768 at 125 % would otherwise open 825 px tall on a 708 px work
/// area, caption above the screen where it cannot be dragged back.
fn placement(work: RECT, scale: f32) -> Placement {
    let (area_w, area_h) = (work.right - work.left, work.bottom - work.top);
    let w = ((960.0 * scale) as i32).min(area_w);
    let h = ((660.0 * scale) as i32).min(area_h);
    Placement {
        x: work.left + (area_w - w) / 2,
        y: work.top + (area_h - h) / 2,
        w,
        h,
        min_w: ((720.0 * scale) as i32).min(w),
        min_h: ((480.0 * scale) as i32).min(h),
    }
}

/// Where a window of `size` goes beside `fence` in `work` (screen px), `gap` apart: to the
/// right when it fits there, else to the left, else on the side with more room, narrowed to
/// that room but not below `min_w` (then it may cover part of the fence). The top lines up
/// with the fence's; the window stays inside `work`.
fn beside(fence: RECT, work: RECT, size: (i32, i32), min_w: i32, gap: i32) -> RECT {
    let (w, h) = (
        size.0.min(work.right - work.left),
        size.1.min(work.bottom - work.top),
    );
    let right_room = work.right - fence.right - gap;
    let left_room = fence.left - work.left - gap;
    let (on_right, room) = if right_room >= w {
        (true, right_room)
    } else if left_room >= w {
        (false, left_room)
    } else if right_room >= left_room {
        (true, right_room)
    } else {
        (false, left_room)
    };
    let w = w.min(room.max(min_w));
    let x = if on_right {
        fence.right + gap
    } else {
        fence.left - gap - w
    };
    let x = x.clamp(work.left, (work.right - w).max(work.left));
    let y = fence.top.clamp(work.top, (work.bottom - h).max(work.top));
    RECT {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    /// 栅栏选项… keeps the fence in view: the window goes to the free side of it.
    #[test]
    fn fence_options_window_goes_beside_the_fence() {
        let work = rect(0, 0, 1600, 852);
        // The first start's right-hand column: the window opens to its left.
        let r = beside(rect(1270, 16, 1584, 262), work, (960, 660), 720, 12);
        assert_eq!((r.left, r.right, r.top), (298, 1258, 16));
        // A fence at the left edge: to its right; a low fence lifts the window into the area.
        let r = beside(rect(20, 600, 320, 840), work, (960, 660), 720, 12);
        assert_eq!((r.left, r.top, r.bottom), (332, 192, 852));
        // In the middle with no room for the full width: the larger side, narrowed.
        let r = beside(rect(600, 100, 900, 400), work, (960, 660), 720, 12);
        assert_eq!((r.left, r.right), (880, 1600));
        let r = beside(rect(700, 100, 1000, 400), work, (960, 660), 600, 12);
        assert_eq!((r.left, r.right), (0, 688));
        // A second monitor to the right keeps the window on the fence's own monitor.
        let work2 = rect(1600, 0, 3520, 1040);
        let r = beside(rect(3200, 40, 3500, 300), work2, (960, 660), 720, 12);
        assert!(r.left >= work2.left && r.right <= 3200 - 12, "{r:?}");
    }

    fn work(w: i32, h: i32) -> RECT {
        RECT {
            left: 0,
            top: 0,
            right: w,
            bottom: h,
        }
    }

    /// 1366 x 768 at 125 % (a 48 DIP taskbar = 60 px): the window must fit, caption on screen.
    #[test]
    fn settings_window_fits_small_high_dpi_screens() {
        for (area, scale) in [
            (work(1366, 708), 1.25),
            (work(1920, 996), 1.75),
            (work(1280, 672), 1.5),
            (work(2560, 1392), 1.0),
        ] {
            let p = placement(area, scale);
            assert!(
                p.y >= area.top && p.x >= area.left,
                "{p:?} off-screen on {area:?}"
            );
            assert!(
                p.y + p.h <= area.bottom && p.x + p.w <= area.right,
                "{p:?} too big"
            );
            assert!(
                p.min_w <= p.w && p.min_h <= p.h,
                "{p:?} minimum above the window"
            );
        }
        // Room to spare: the designed size, centred.
        let p = placement(work(2560, 1392), 1.0);
        assert_eq!((p.w, p.h), (960, 660));
        assert_eq!((p.x, p.y), (800, 366));
    }

    /// Reopening in the same run: the window comes back where it was closed, on the monitors
    /// that exist now.
    #[test]
    fn settings_window_reopens_where_it_was_closed() {
        let area = work(1600, 852);
        let first = placement(area, 1.0);
        // Moved and resized: the same rectangle again.
        let p = first.reopened(rect(60, 40, 1260, 820), 1.0, area, 1.0);
        assert_eq!((p.x, p.y, p.w, p.h), (60, 40, 1200, 780));
        assert_eq!((p.min_w, p.min_h), (first.min_w, first.min_h));
        // Dragged half off the right edge, or closed on a larger screen: back inside the work
        // area, no larger than it.
        let p = first.reopened(rect(1100, 300, 2060, 960), 1.0, area, 1.0);
        assert_eq!((p.x, p.y, p.w, p.h), (640, 192, 960, 660));
        let p = first.reopened(rect(100, 0, 2500, 1300), 1.0, area, 1.0);
        assert_eq!((p.x, p.y, p.w, p.h), (0, 0, 1600, 852));
        // The monitor it stood on is gone: its size, centred like a first open.
        let p = first.reopened(rect(1700, 100, 2700, 800), 1.0, area, 1.0);
        assert_eq!((p.x, p.y, p.w, p.h), (300, 76, 1000, 700));
        // The monitor's scale changed from 100 % to 150 %: the same size in DIP, and never
        // below the minimum of the new scale.
        let big = work(2560, 1392);
        let p = placement(big, 1.5).reopened(rect(200, 100, 1200, 800), 1.0, big, 1.5);
        assert_eq!((p.x, p.y, p.w, p.h), (200, 100, 1500, 1050));
        let p = placement(big, 1.5).reopened(rect(200, 100, 920, 580), 1.0, big, 1.5);
        assert_eq!((p.w, p.h), (1080, 720));
        // A second monitor left of the primary one (negative coordinates) keeps the window.
        let left = rect(-1920, 0, 0, 1040);
        let p = placement(left, 1.0).reopened(rect(-1500, 200, -400, 900), 1.0, left, 1.0);
        assert_eq!((p.x, p.y, p.w, p.h), (-1500, 200, 1100, 700));
    }

    #[test]
    fn the_smart_fence_editor_is_a_small_window_that_fits() {
        let p = editor_placement(work(1600, 852), 1.0);
        assert_eq!((p.w, p.h, p.x, p.y), (880, 660, 360, 96));
        // 1366 x 768 at 100 % has room for the whole window.
        let p = editor_placement(work(1366, 720), 1.0);
        assert_eq!((p.w, p.h, p.x, p.y), (880, 660, 243, 30));
        // Smaller work areas (1024 x 768 at 150 % among them) get a window no larger than they
        // are; its list scrolls there.
        for (area, scale) in [
            (work(1024, 696), 1.5),
            (work(1366, 708), 1.5),
            (work(1280, 672), 1.75),
        ] {
            let p = editor_placement(area, scale);
            assert!(
                p.x >= 0 && p.y >= 0 && p.x + p.w <= area.right && p.y + p.h <= area.bottom,
                "{p:?} on {area:?}"
            );
            assert!(p.min_w <= p.w && p.min_h <= p.h, "{p:?}");
        }
    }
}
