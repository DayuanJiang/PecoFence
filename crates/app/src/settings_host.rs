//! On-demand WebView2 settings window (plan §9): created when opened, destroyed when closed so
//! the browser processes go away and nothing stays resident.
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
use std::path::Path;
use std::rc::Rc;
use windows_core::Result;
use windows_webview::{Controller, Environment, EnvironmentOptions, WebView};

pub const SETTINGS_CLASS: &str = "PecoFence.Settings";
const SETTINGS_HTML: &str = include_str!("../../../ui/settings.html");
const I18N_JS: &str = include_str!("../../../ui/i18n.js");

/// The WebView2 environment. Kept alive across opens (warm start ≈ 200 ms); the browser
/// processes exit by themselves when no controller is alive.
pub struct WebEnvironment {
    env: Environment,
}

impl WebEnvironment {
    pub fn create(user_data: &Path) -> Result<Self> {
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
        _liquid_glass: bool,
        queue: CommandQueue,
    ) -> Result<Self> {
        let state = Rc::new(RefCell::new(HostState {
            controller: None,
            webview: None,
        }));

        let mons = monitors::enumerate();
        let primary = mons.iter().find(|m| m.primary).or(mons.first());
        let scale = primary.map(|m| m.scale()).unwrap_or(1.0);
        let place = primary.map(|m| placement(m.work_area, scale));
        let (min_w, min_h) = place
            .map(|p| (p.min_w, p.min_h))
            .unwrap_or(((720.0 * scale) as i32, (480.0 * scale) as i32));

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

        let (x, y, w, h) = place.map(|p| (p.x, p.y, p.w, p.h)).unwrap_or((
            100,
            100,
            (960.0 * scale) as i32,
            (660.0 * scale) as i32,
        ));

        let window = WindowBuilder::new(class)
            .title(pecofence_core::i18n::text("PecoFence 设置"))
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
}
