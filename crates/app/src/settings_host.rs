//! On-demand WebView2 settings window (plan §9): created when opened, destroyed when closed so
//! the browser processes go away and nothing stays resident.
//!
//! IPC: the page posts JSON (`{"type": "ready" | "patchSettings" | "setRules" | "action"}`);
//! the host pushes `{"type": "state", ...}` / `{"type": "toast", ...}` with `post_json`.

use crate::commands::{Command, CommandQueue};
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, dwm, monitors, msg};
use pecofence_render::ThemeMode;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use windows_core::Result;
use windows_webview::{Controller, Environment, EnvironmentOptions, WebView};

pub const SETTINGS_CLASS: &str = "PecoFence.Settings";
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
        let options =
            EnvironmentOptions::new().user_data_folder(user_data.to_string_lossy().to_string());
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
    /// Caption icons (small/big); destroyed with the host.
    icons: Vec<pecofence_platform::tray::OwnedIcon>,
    _registrations: Vec<windows_webview::EventRegistration>,
}

impl SettingsHost {
    pub fn register_class() -> Result<WindowClass> {
        WindowClass::register(SETTINGS_CLASS, ClassOptions::default())
    }

    /// Creates the window and the WebView2 controller and loads the settings page. Page
    /// messages are forwarded to the command queue as `Command::SettingsMessage`.
    pub fn open(
        class: &WindowClass,
        env: &WebEnvironment,
        mode: ThemeMode,
        liquid_glass: bool,
        queue: CommandQueue,
    ) -> Result<Self> {
        let state = Rc::new(RefCell::new(HostState {
            controller: None,
            webview: None,
        }));

        let mons = monitors::enumerate();
        let primary = mons.iter().find(|m| m.primary).or(mons.first());
        let scale = primary.map(|m| m.scale()).unwrap_or(1.0);
        let (min_w, min_h) = ((720.0 * scale) as i32, (480.0 * scale) as i32);

        let handler: MessageHandler = {
            let state = state.clone();
            let queue = queue.clone();
            Box::new(
                move |hwnd: HWND, message: u32, _wparam: usize, lparam: isize| -> Option<isize> {
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
                            // SAFETY: lParam is the MINMAXINFO for this message.
                            unsafe { window::minmaxinfo_set_min_track(lparam, min_w, min_h) };
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
                        msg::WM_DESTROY => {
                            let mut s = state.borrow_mut();
                            s.webview = None;
                            if let Some(c) = s.controller.take() {
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

        let (w, h) = ((960.0 * scale) as i32, (660.0 * scale) as i32);
        let (x, y) = primary
            .map(|m| {
                (
                    (m.work_area.left + m.work_area.right - w) / 2,
                    (m.work_area.top + m.work_area.bottom - h) / 2,
                )
            })
            .unwrap_or((100, 100));

        let window = WindowBuilder::new(class)
            .title(pecofence_core::i18n::text("PecoFence 设置"))
            .style(style::OVERLAPPEDWINDOW | style::CLIPCHILDREN)
            .bounds(x, y, w, h)
            .create(handler)?;
        let hwnd = window.hwnd();
        tracing::debug!(?hwnd, "settings: host window created");
        let _ = dwm::set_immersive_dark_mode(hwnd, matches!(mode, ThemeMode::Dark));
        apply_caption(hwnd, mode, false, liquid_glass);
        // A transparent page needs the WebView2 *composition* controller (HWND children cannot be
        // per-pixel transparent); windows-webview 0.100 only wraps the HWND controller, so the
        // Mica sheet stays behind an experiment flag and the page paints the theme's base colour.
        let mica = pecofence_core::brand::var_os("PECOFENCE_SETTINGS_MICA").is_some()
            && dwm::set_system_backdrop(hwnd, dwm::SystemBackdrop::MainWindow).is_ok();
        apply_caption(hwnd, mode, mica, liquid_glass);

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
        controller.set_default_background_color(page_background(mode, mica, liquid_glass))?;
        if let Ok(settings) = webview.settings() {
            let _ = settings.set_default_context_menus_enabled(false);
            let _ = settings.set_status_bar_enabled(false);
            let _ = settings.set_zoom_control_enabled(false);
        }

        let mut registrations = Vec::new();
        {
            let queue = queue.clone();
            registrations.push(webview.on_web_message_received(move |args| {
                queue.push(Command::SettingsMessage(args.web_message_as_json()));
            })?);
        }
        // Tell the page whether it sits on Mica before it renders (opaque fallback class).
        let payload = pecofence_core::i18n::ui_payload()
            .to_string()
            .replace('<', "\\u003c");
        let document = SETTINGS_HTML
            .replace(
                "<!-- PECOFENCE_LOCALE -->",
                &format!("<script>window.PECOFENCE_LOCALE={payload};</script>"),
            )
            .replace(
                "<script src=\"i18n.js\"></script>",
                &format!("<script>{I18N_JS}</script>"),
            );
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
        window::show_normal(hwnd);
        tracing::debug!(visible = window.is_visible(), "settings: host window shown");
        // The controller is created while the host is still hidden and stays invisible until told.
        if let Some(c) = state.borrow().controller.as_ref() {
            let _ = c.set_visible(true);
        }
        Ok(Self {
            window,
            state,
            mica,
            icons: Vec::new(),
            _registrations: registrations,
        })
    }

    pub fn hwnd(&self) -> HWND {
        self.window.hwnd()
    }

    pub fn update_language(&self) {
        window::set_title(
            self.window.hwnd(),
            pecofence_core::i18n::text("PecoFence 设置"),
        );
    }

    /// Re-themes the caption and the page background after a light/dark switch.
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

    pub fn set_theme(&self, mode: ThemeMode, liquid_glass: bool) {
        let _ = dwm::set_immersive_dark_mode(self.window.hwnd(), matches!(mode, ThemeMode::Dark));
        apply_caption(self.window.hwnd(), mode, self.mica, liquid_glass);
        if let Some(c) = self.state.borrow().controller.as_ref() {
            let _ = c.set_default_background_color(page_background(mode, self.mica, liquid_glass));
        }
    }

    pub fn post_json(&self, json: &str) {
        if let Some(w) = self.state.borrow().webview.as_ref() {
            let _ = w.post_web_message_as_json(json);
        }
    }

    /// Closes the browser and destroys the window.
    #[allow(dead_code)]
    pub fn close(self) {
        {
            let mut s = self.state.borrow_mut();
            s.webview = None;
            if let Some(c) = s.controller.take() {
                let _ = c.close();
            }
        }
        drop(self.window);
    }
}

/// Page background: fully transparent over Mica, else the theme's solid base colour.
fn page_background(mode: ThemeMode, mica: bool, liquid_glass: bool) -> windows_webview::Color {
    if mica {
        return windows_webview::Color {
            a: 0,
            r: 0,
            g: 0,
            b: 0,
        };
    }
    // Keep these RGB values in sync with --sheet in the embedded settings document.
    let (r, g, b) = match (mode, liquid_glass) {
        (ThemeMode::Dark, true) => (0x24, 0x25, 0x28),
        (ThemeMode::Light, true) => (0xE2, 0xE3, 0xE7),
        (ThemeMode::Dark, false) => (0x15, 0x1D, 0x23),
        (ThemeMode::Light, false) => (0xF4, 0xF7, 0xF8),
    };
    windows_webview::Color { a: 255, r, g, b }
}

/// Match the page sheet so the native caption belongs to the same surface.
fn apply_caption(hwnd: HWND, mode: ThemeMode, mica: bool, liquid_glass: bool) {
    if mica {
        return;
    }
    let bg = page_background(mode, false, liquid_glass);
    // DWM takes COLORREF (0x00BBGGRR).
    let c = u32::from(bg.r) | (u32::from(bg.g) << 8) | (u32::from(bg.b) << 16);
    let _ = dwm::set_caption_color(hwnd, c);
}
