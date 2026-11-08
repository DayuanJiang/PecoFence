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
use pecofence_platform::{HWND, RECT, dwm, monitors, msg};
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
}

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
    /// the monitor of `near` (the point its menu opened at), centred on its work area.
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
    ) -> Result<Self> {
        let opened = std::time::Instant::now();
        let state = Rc::new(RefCell::new(HostState {
            controller: None,
            webview: None,
        }));

        let mons = monitors::enumerate();
        let primary = mons.iter().find(|m| m.primary).or(mons.first());
        let monitor = near
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
        let place = monitor.map(|m| place_on(m.work_area, scale));
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
                        msg::WM_ACTIVATE if wparam & 0xffff != 0 && kind == HostKind::Settings => {
                            queue.push(Command::SettingsActivated);
                            None
                        }
                        msg::WM_DESTROY => {
                            // Taken out of the state first: closing the browser may let a
                            // message sent by another thread in, whose handling reads the
                            // state (GitHub #166).
                            let (webview, controller) = {
                                let mut s = state.borrow_mut();
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
        let html = if mica {
            document
        } else {
            document.replacen(
                "<html lang=\"zh-CN\">",
                "<html lang=\"zh-CN\" class=\"no-mica\">",
                1,
            )
        };
        webview.navigate_to_string(&html)?;
        tracing::debug!("settings: page navigation submitted");

        {
            let mut s = state.borrow_mut();
            s.controller = Some(controller);
            s.webview = Some(webview);
        }
        // The controller is created while the host is still hidden and stays invisible until
        // told. The host itself is shown once the page reports `ready` (`App::on_settings_message`):
        // shown at once, it was white until WebView2 painted, even in dark mode.
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

    /// The window's caption (the settings' after a language change, the editor's for the fence
    /// it shows).
    pub fn set_title(&self, title: &str) {
        window::set_title(self.window.hwnd(), title);
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

/// The smart fence editor: 680 x 560 DIP (minimum 560 x 420), centred on the work area, and
/// smaller on a work area that has no room for it.
fn editor_placement(work: RECT, scale: f32) -> Placement {
    let (area_w, area_h) = (work.right - work.left, work.bottom - work.top);
    let w = ((680.0 * scale) as i32).min(area_w);
    let h = ((560.0 * scale) as i32).min(area_h);
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn the_smart_fence_editor_is_a_small_window_that_fits() {
        let p = editor_placement(work(1600, 852), 1.0);
        assert_eq!((p.w, p.h, p.x, p.y), (680, 560, 460, 146));
        for (area, scale) in [(work(1366, 708), 1.5), (work(1280, 672), 1.75)] {
            let p = editor_placement(area, scale);
            assert!(
                p.y + p.h <= area.bottom && p.x >= 0 && p.min_h <= p.h,
                "{p:?}"
            );
        }
    }
}
