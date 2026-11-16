//! 网页层: a web plugin's own page inside its fence. All web plugins share the app's WebView2
//! environment (the one the settings window uses), each plugin with a profile of its own. The page
//! is drawn into the fence's visual tree (the composition controller), so the fence's glass,
//! rounded clip, roll-up and fades apply to it; the fence forwards the mouse and the cursor, and a
//! press in the page hands it the keyboard (`focus`).
//!
//! The plugin's files come from PecoFence at `pecofence-plugin://<id>/` (`SCHEME`: never the
//! network or a proxy, a secure context with an origin per plugin), every response
//! with a fixed Content-Security-Policy whose `connect-src` lists only the plugin's `network`
//! hosts (and the plugin's own files), and a Connection-Allowlist of the same hosts, which the
//! browser applies to every connection (`Plugin::allowlist`). Permission requests, other sites,
//! new windows, external protocols (`mailto:`, `ms-settings:`: they start as navigations) and
//! downloads are refused. The page calls `pf.*` through `postMessage`; only its own main frame is
//! heard, and the app answers it with the code the light layer uses (`App::web_call`). Events
//! (`show`, `hide`, `resize`, `theme`, `settings`, ...) reach the page as messages too.

use crate::commands::{Command, CommandQueue};
use pecofence_core::FenceId;
use pecofence_platform::{HWND, msg, window};
use pecofence_render::ContainerVisual;
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};
use windows_webview::{
    Color, CompositionController, Controller, ControllerOptions, Environment, EventRegistration,
    MemoryUsageTargetLevel, MoveFocusReason, PermissionState, PreferredColorScheme,
    ProcessFailedKind, WebResourceResponse, WebView,
};

/// The custom scheme of the plugins' pages (registered with the environment,
/// `WebEnvironment::create`).
pub const SCHEME: &str = "pecofence-plugin";
/// `WM_TIMER` id on the fence window: its hidden page is suspended.
pub const TIMER_WEB_SUSPEND: usize = 0x5E0;
/// `WM_TIMER` id on the fence window: the browser is made, the page is set up (`setup`). Not in
/// WebView2's own completion handler: WebView2 delivers no other callback while one runs, and
/// the setup waits for some.
pub const TIMER_WEB_READY: usize = 0x5E1;
/// `WM_TIMER` id on the fence window: looks for the browser's window over the page
/// (`pass_mouse_through`) until it is there.
pub const TIMER_WEB_INPUT: usize = 0x5E2;
/// How long a page stays hidden before it is suspended.
const SUSPEND_AFTER_MS: u32 = 30_000;
const WM_MOUSELEAVE: u32 = 0x02A3;
const WM_MOUSEWHEEL: u32 = 0x020A;
const WM_MOUSEHWHEEL: u32 = 0x020E;
/// `MK_LBUTTON | MK_RBUTTON | MK_MBUTTON | MK_XBUTTON1 | MK_XBUTTON2`.
const MK_BUTTONS: usize = 0x0001 | 0x0002 | 0x0010 | 0x0020 | 0x0040;
/// `pf.open` and the fence's menu need a press in the page this recently.
const PRESS_GRACE: Duration = Duration::from_secs(1);
/// The `pf.*` calls a page has, all answered by the app (`App::web_call`) with a Promise.
const CALLS: &str = "time.now time.format http.get http.post data.read data.write data.list \
    data.remove clipboard.text clipboard.files clipboard.image clipboard.setText \
    clipboard.setFiles clipboard.setImage sys.cpu sys.memory sys.drives sys.ip sys.network \
    sys.battery open openSystem run folder.pick folder.list folder.read notify launch.list \
    launch.open launch.pick launch.remove media.current media.control apps.running apps.front \
    drop.read drop.text drop.save";

/// A step of a drag over the page (`WebFence::page_drag`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragStep {
    Enter = 0,
    Over = 1,
    Leave = 2,
    Drop = 3,
}

/// An installed web plugin a fence shows.
pub struct Plugin {
    pub id: String,
    pub dir: PathBuf,
    pub network: Vec<String>,
    /// `links`: hosts it may open (and a link dragged out of its page may lead to).
    pub links: Vec<String>,
    /// `background`: its page keeps running while the fence is hidden (no low memory, no
    /// suspend).
    pub background: bool,
    /// It may read the clipboard: its page gets the `clipboard` event.
    pub clipboard_read: bool,
    /// It may see the media session (`media` event) and know of programs (`apps` event).
    pub media: bool,
    pub apps: bool,
}

/// Why a page stopped (`Command::WebPageFailed`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    /// The browser process ended: every page and the settings window went with it.
    BrowserExited,
    /// The page's render process ended (a crash, or ended from outside).
    PageExited,
    /// The page stopped answering.
    PageHung,
}

impl Plugin {
    fn origin(&self) -> String {
        format!("{SCHEME}://{}/", self.id)
    }

    /// The header every response gets; what the plugin writes itself does not count.
    fn csp(&self) -> String {
        let hosts: Vec<String> = self
            .network
            .iter()
            .map(|h| format!("https://{h} wss://{h}"))
            .collect();
        let connect = if hosts.is_empty() {
            String::new()
        } else {
            format!(" {}", hosts.join(" "))
        };
        // 'self': the plugin may fetch its own files (a JSON of its own).
        format!(
            "default-src 'self'; connect-src 'self'{connect}; frame-src 'none'; form-action 'none'"
        )
    }

    /// The Connection-Allowlist header every response gets too (WebView2 152 and later): the
    /// browser refuses each connection of the page, its blank frames and its workers to anything
    /// but the plugin's own files and its `network` hosts, also those the CSP does not cover
    /// (`dns-prefetch`, `preconnect`), and all of WebRTC. Redirects are left to the CSP, which
    /// checks each one.
    fn allowlist(&self) -> String {
        let hosts: String = self
            .network
            .iter()
            .map(|h| format!(" \"https://{h}/*\""))
            .collect();
        format!("(response-origin{hosts}); redirects=allow")
    }
}

/// The page's browser once WebView2 has made it.
struct Live {
    comp: CompositionController,
    controller: Controller,
    webview: WebView,
    _events: Vec<EventRegistration>,
}

/// One fence's page.
pub struct WebFence {
    pub plugin: Plugin,
    pub fence: FenceId,
    /// The visual the page is drawn into; the fence puts it into its content panel.
    pub visual: ContainerVisual,
    hwnd: HWND,
    me: Weak<Self>,
    queue: CommandQueue,
    /// What `pf` says about the fence when the page starts (`lang`, `theme`, `settings`,
    /// `fence`); events bring the changes.
    init: serde_json::Value,
    /// Where the page starts (its own address before it loaded anew), else index.html.
    at: Option<String>,
    dark: Cell<bool>,
    /// The WebView2 runtime's version, for the log.
    version: String,
    /// The last press of a mouse button in the page.
    pressed: Cell<Option<Instant>>,
    /// The size last told to the page (`resize`, DIPs).
    told: Cell<(i32, i32)>,
    live: RefCell<Option<Live>>,
    /// The browser WebView2 made, until `setup` takes it.
    made: RefCell<Option<CompositionController>>,
    pub started: std::time::Instant,
    /// The page's place in the window's client area (px) and the fence's scale.
    bounds: Cell<(i32, i32, i32, i32, f64)>,
    shown: Cell<bool>,
    /// Looks left for the browser's window over the page (`TIMER_WEB_INPUT`).
    input_tries: Cell<u32>,
    /// The pointer is over the page (it was sent a move and no leave since).
    inside: Cell<bool>,
    /// A button went down in the page: the moves and the release go to it wherever they are.
    captured: Cell<bool>,
    /// The page's own drag is over the page (it was sent DragEnter and no leave since), and
    /// the data object of the drag it let start last, until that drag ends over the fence.
    drag_inside: Cell<bool>,
    drag_data: RefCell<Option<windows_core::IUnknown>>,
}

impl WebFence {
    /// Starts the page of `plugin` for the fence window `hwnd`; WebView2 makes its browser in the
    /// background and the page loads when it is ready (`ready`). `init` is what `pf` says about
    /// the fence at first; the page's calls and failures go to the app through `queue`.
    pub fn start(
        env: &Environment,
        stack: &pecofence_render::RenderStack,
        hwnd: HWND,
        fence: FenceId,
        plugin: Plugin,
        queue: CommandQueue,
        init: serde_json::Value,
        at: Option<String>,
    ) -> Rc<Self> {
        let web = Rc::new_cyclic(|me| Self {
            visual: stack.compositor.create_container_visual(),
            plugin,
            fence,
            hwnd,
            me: me.clone(),
            queue,
            dark: Cell::new(init["theme"] == "dark"),
            init,
            at,
            version: env.browser_version(),
            pressed: Cell::new(None),
            told: Cell::new((0, 0)),
            live: RefCell::new(None),
            made: RefCell::new(None),
            started: std::time::Instant::now(),
            bounds: Cell::new((0, 0, 1, 1, 1.0)),
            shown: Cell::new(true),
            input_tries: Cell::new(0),
            inside: Cell::new(false),
            captured: Cell::new(false),
            drag_inside: Cell::new(false),
            drag_data: RefCell::new(None),
        });
        let options = ControllerOptions::new()
            .profile_name(format!("plugin-{}", web.plugin.id))
            .default_background_color(Color::TRANSPARENT);
        let weak = Rc::downgrade(&web);
        // SAFETY: the fence window outlives its page: the page is closed when the fence's view
        // state (which owns it) goes, before the window is destroyed.
        let begun = unsafe {
            env.create_composition_controller(
                pecofence_platform::hwnd_ptr(hwnd),
                &options,
                move |result| match (weak.upgrade(), result) {
                    (Some(web), Ok(comp)) => {
                        *web.made.borrow_mut() = Some(comp);
                        window::set_timer(web.hwnd, TIMER_WEB_READY, 0);
                    }
                    (Some(web), Err(e)) => {
                        tracing::warn!(plugin = %web.plugin.id, error = %e, "web plugin: browser not made")
                    }
                    (None, Ok(comp)) => {
                        // The fence went while the browser was being made.
                        if let Ok(c) = comp.controller() {
                            let _ = c.close();
                        }
                    }
                    (None, Err(_)) => {}
                },
            )
        };
        if let Err(e) = begun {
            tracing::warn!(plugin = %web.plugin.id, error = %e, "web plugin: browser not started");
        }
        web
    }

    /// `TIMER_WEB_READY`: hangs the browser WebView2 made into the fence and loads the page.
    pub fn setup(&self) {
        window::kill_timer(self.hwnd, TIMER_WEB_READY);
        let Some(comp) = self.made.borrow_mut().take() else {
            return;
        };
        let ms = self.started.elapsed().as_millis() as u64;
        match self.ready(comp) {
            Ok(()) => {
                tracing::info!(plugin = %self.plugin.id, fence = %self.fence, ms, version = self.version, background = self.plugin.background, "web plugin: page loading");
            }
            Err(e) => {
                tracing::warn!(plugin = %self.plugin.id, error = %e, "web plugin: browser setup failed")
            }
        }
    }

    fn ready(&self, comp: CompositionController) -> windows_core::Result<()> {
        let controller = comp.controller()?;
        let webview = controller.webview()?;
        comp.set_root_visual_target((&self.visual.as_raw()).into())?;
        // The fence gives the scale with the bounds (`set_bounds`).
        controller.set_should_detect_monitor_scale_changes(false)?;
        if let Ok(s) = webview.settings() {
            let _ = s.set_default_context_menus_enabled(false);
            let _ = s.set_status_bar_enabled(false);
            let _ = s.set_zoom_control_enabled(false);
            let _ = s.set_browser_accelerator_keys_enabled(false);
            let _ = s.set_default_script_dialogs_enabled(false);
            let _ = s.set_host_objects_allowed(false);
            let debug = cfg!(debug_assertions)
                && pecofence_core::brand::var("PECOFENCE_WEBVIEW_ARGS").is_ok();
            let _ = s.set_dev_tools_enabled(debug);
        }
        // prefers-color-scheme follows what the fence draws (the profile is the plugin's own).
        if let Ok(profile) = webview.profile() {
            let _ = profile.set_preferred_color_scheme(scheme(self.dark.get()));
        }
        let id = self.plugin.id.clone();
        let origin = self.plugin.origin();
        let mut events = Vec::new();
        // The plugin's files, each with the fixed CSP.
        {
            let (dir, csp, allow, origin) = (
                self.plugin.dir.clone(),
                self.plugin.csp(),
                self.plugin.allowlist(),
                origin.clone(),
            );
            events.push(
                webview.on_web_resource_requested(&format!("{origin}*"), move |req| {
                    let uri = req.uri();
                    let path = uri
                        .strip_prefix(&origin)?
                        .split(['?', '#'])
                        .next()
                        .unwrap_or("");
                    let path = if path.is_empty() { "index.html" } else { path };
                    let plain = path.split('/').all(|p| {
                        !p.is_empty() && p != "." && p != ".." && !p.contains(['\\', ':', '%'])
                    });
                    let body = plain.then(|| std::fs::read(dir.join(path)).ok()).flatten();
                    tracing::debug!(path, found = body.is_some(), "web plugin: file asked for");
                    let response = match body {
                        Some(b) => WebResourceResponse::new(b).content_type(mime(path)),
                        None => WebResourceResponse::new(Vec::new()).status(404, "Not Found"),
                    };
                    // nosniff: a file runs as a script only when it is one (a picture that is
                    // also valid JavaScript does not).
                    Some(
                        response
                            .header("Content-Security-Policy", &csp)
                            .header("Connection-Allowlist", &allow)
                            .header("X-Content-Type-Options", "nosniff"),
                    )
                })?,
            );
        }
        {
            let (id, origin, page, queue) = (
                id.clone(),
                origin.clone(),
                self.me.clone(),
                self.queue.clone(),
            );
            let first = Cell::new(true);
            events.push(webview.on_navigation_starting(move |args| {
                let uri = args.uri();
                if !uri.starts_with(&origin) {
                    let _ = args.set_cancel(true);
                    tracing::info!(plugin = %id, uri = %short(&uri), "web plugin: navigation to another site refused");
                } else if !first.replace(false) {
                    // `location.reload()` or another page of its own: a new page there, so it
                    // gets what `pf` says now and its calls count alone.
                    let _ = args.set_cancel(true);
                    tracing::info!(plugin = %id, uri = %short(&uri), "web plugin: the page loads anew, made again");
                    let page = page.clone();
                    queue.push(Command::WebPageReload { page, uri });
                }
            })?);
        }
        {
            let id = id.clone();
            events.push(webview.on_new_window_requested(move |args| {
                let _ = args.set_handled(true);
                tracing::info!(plugin = %id, uri = %short(&args.uri()), "web plugin: new window refused");
            })?);
        }
        {
            let id = id.clone();
            events.push(webview.on_permission_requested(move |args| {
                let _ = args.set_state(PermissionState::Deny);
                tracing::info!(plugin = %id, kind = ?args.kind(), "web plugin: permission refused");
            })?);
        }
        {
            let id = id.clone();
            events.push(webview.on_download_starting(move |args| {
                let _ = args.set_cancel(true);
                tracing::info!(plugin = %id, "web plugin: download refused");
            })?);
        }
        // `pf.*`: only the plugin's own main frame is heard; the app answers.
        {
            let (id, origin, page, queue) = (
                id.clone(),
                origin.clone(),
                self.me.clone(),
                self.queue.clone(),
            );
            events.push(webview.on_web_message_received(move |args| {
                let source = args.source();
                if !source.starts_with(&origin) {
                    tracing::info!(plugin = %id, uri = %short(&source), "web plugin: message from another page ignored");
                    return;
                }
                let json = args.web_message_as_json();
                if json.len() > pecofence_plugin_host::MAX_POST {
                    tracing::info!(plugin = %id, bytes = json.len(), "web plugin: a message over 1 MB ignored");
                    return;
                }
                let m: serde_json::Value = serde_json::from_str(&json).unwrap_or_default();
                let call = m["call"].as_str().unwrap_or("").to_string();
                let args = m.get("args").cloned().unwrap_or_else(|| serde_json::json!([]));
                let id = m.get("id").cloned().unwrap_or_default();
                let page = page.clone();
                queue.push(Command::WebCall { page, id, call, args });
            })?);
        }
        // A crashed or hung page: the app makes it anew, or shows why (`App::web_page_failed`).
        {
            let (id, fence, page, queue) =
                (id.clone(), self.fence, self.me.clone(), self.queue.clone());
            events.push(webview.on_process_failed(move |args| {
                let kind = args.kind();
                let failure = match kind {
                    ProcessFailedKind::BrowserProcessExited => Failure::BrowserExited,
                    ProcessFailedKind::RenderProcessExited => Failure::PageExited,
                    ProcessFailedKind::RenderProcessUnresponsive => Failure::PageHung,
                    _ => {
                        // WebView2 starts a GPU or utility process again by itself.
                        return tracing::info!(plugin = %id, %fence, ?kind, "web plugin: a helper process failed, the page goes on");
                    }
                };
                tracing::warn!(plugin = %id, %fence, ?kind, "web plugin: the page failed");
                let page = page.clone();
                queue.push(Command::WebPageFailed { page, failure });
            })?);
        }
        // What the page drags out passes the allow-list of a row's drag, or does not start
        // (runtime 144 and later; on an older one the bridge stops every drag in the page).
        // Debug builds: `PECOFENCE_TEST_OLD_WEBVIEW` stands in for an older runtime.
        let old = cfg!(debug_assertions)
            && pecofence_core::brand::var("PECOFENCE_TEST_OLD_WEBVIEW").is_ok();
        let drags = if old {
            Err(windows_core::Error::empty())
        } else {
            let (id, origin, page) = (id.clone(), origin.clone(), self.me.clone());
            let hosts = [&self.plugin.network[..], &self.plugin.links[..]].concat();
            comp.on_drag_starting(move |args| {
                if let Some(why) = page_drag_refusal(args, &hosts, &origin) {
                    let _ = args.set_handled(true);
                    return tracing::info!(plugin = %id, why, "web plugin: the drag out of the page is refused");
                }
                if let Some(page) = page.upgrade() {
                    *page.drag_data.borrow_mut() = args.data().ok();
                }
            })
        };
        let drags = match drags {
            Ok(r) => {
                events.push(r);
                true
            }
            Err(e) => {
                tracing::info!(plugin = %id, version = self.version, error = %e, "web plugin: this WebView2 cannot check a drag out of the page, the page drags nothing out");
                false
            }
        };
        // The browser takes no drop itself: the fence hands the page what it declared (`drop`).
        let _ = controller.set_allow_external_drop(false);
        {
            let hwnd = self.hwnd;
            let comp2 = comp.clone();
            events.push(comp.on_cursor_changed(move || {
                if pointer_in_client(hwnd) {
                    window::set_cursor_handle(comp2.cursor());
                }
            })?);
        }
        webview.add_script_to_execute_on_document_created(&bridge_script(
            &origin, &self.init, drags,
        ))?;
        let (x, y, w, h, scale) = self.bounds.get();
        controller.set_rasterization_scale(scale)?;
        controller.set_bounds(x, y, x + w, y + h)?;
        controller.set_visible(true)?;
        webview.navigate(self.at.as_deref().unwrap_or(&format!("{origin}index.html")))?;
        self.look_for_browser_window();
        *self.live.borrow_mut() = Some(Live {
            comp,
            controller,
            webview,
            _events: events,
        });
        Ok(())
    }

    /// The page's place in the fence window's client area (px) at the fence's `scale`.
    pub fn set_bounds(&self, x: i32, y: i32, w: i32, h: i32, scale: f64) {
        let old = self.bounds.replace((x, y, w.max(1), h.max(1), scale));
        if old == self.bounds.get() {
            return;
        }
        if let Some(live) = self.live.borrow().as_ref() {
            if old.4 != scale {
                tracing::debug!(plugin = %self.plugin.id, scale, "web plugin: page scale follows the fence");
                let _ = live.controller.set_rasterization_scale(scale);
            }
            let _ = live.controller.set_bounds(x, y, x + w.max(1), y + h.max(1));
        }
        let dips = ((f64::from(w) / scale) as i32, (f64::from(h) / scale) as i32);
        if self.told.replace(dips) != dips {
            let size = serde_json::json!({ "width": dips.0, "height": dips.1 });
            self.event("resize", &size);
        }
    }

    /// Event `name` for the page (`pf.on(name)`), when it is there.
    pub fn event(&self, name: &str, data: &serde_json::Value) {
        if let Some(live) = self.live.borrow().as_ref() {
            let m = serde_json::json!({ "event": name, "data": data });
            let ok = live
                .webview
                .post_web_message_as_json(&m.to_string())
                .is_ok();
            tracing::debug!(plugin = %self.plugin.id, fence = %self.fence, name, ok, "web plugin: event to the page");
        }
    }

    /// The answer to the page's call `id`: its Promise resolves or rejects.
    pub fn reply(&self, id: &serde_json::Value, result: Result<serde_json::Value, String>) {
        let ok = result.is_ok();
        let m = match result {
            Ok(v) => serde_json::json!({ "id": id, "result": v }),
            Err(e) => serde_json::json!({ "id": id, "error": e }),
        };
        if let Some(live) = self.live.borrow().as_ref() {
            let _ = live.webview.post_web_message_as_json(&m.to_string());
        }
        tracing::debug!(plugin = %self.plugin.id, fence = %self.fence, ok, "web plugin: pf call answered");
    }

    /// A mouse button went down in the page within the last second (`pf.open`, the menu): true
    /// once, the press is used up.
    pub fn take_press(&self) -> bool {
        (self.pressed.take()).is_some_and(|t| t.elapsed() < PRESS_GRACE)
    }

    /// The fence's text changed between light and dark (`redraw_content`): `prefers-color-scheme`
    /// and `pf.theme` follow.
    pub fn set_theme(&self, dark: bool) {
        if self.dark.replace(dark) == dark {
            return;
        }
        if let Some(live) = self.live.borrow().as_ref()
            && let Ok(profile) = live.webview.profile()
        {
            let _ = profile.set_preferred_color_scheme(scheme(dark));
        }
        tracing::info!(plugin = %self.plugin.id, fence = %self.fence, dark, "web plugin: the page follows the theme");
        self.event(
            "theme",
            &serde_json::json!(pecofence_core::plugins::theme(dark)),
        );
    }

    /// A mouse message of the fence window: sent on to the page when it is over the page (or a
    /// button went down there). True when the page took it.
    pub fn mouse(&self, message: u32, wparam: usize, lparam: isize) -> bool {
        let live = self.live.borrow();
        let Some(live) = live.as_ref() else {
            return false;
        };
        let (mut px, mut py) = (msg::lo_i16(lparam), msg::hi_i16(lparam));
        if message == WM_MOUSEWHEEL || message == WM_MOUSEHWHEEL {
            let p = window::screen_to_client(self.hwnd, pecofence_platform::POINT { x: px, y: py });
            (px, py) = (p.x, p.y);
        }
        let (x, y, ..) = self.bounds.get();
        let over = self.shown.get() && self.over_page(px, py);
        if message == WM_MOUSELEAVE || (!over && !self.captured.get()) {
            if self.inside.replace(false) {
                let _ = live.comp.send_mouse_input(WM_MOUSELEAVE, 0, 0, 0, 0);
            }
            return false;
        }
        self.inside.set(true);
        let data = match message {
            WM_MOUSEWHEEL | WM_MOUSEHWHEEL => (wparam >> 16) as u16 as i16 as i32 as u32,
            0x020B..=0x020D => ((wparam >> 16) & 0xFFFF) as u32,
            _ => 0,
        };
        // A button went down (WM_LBUTTONDOWN, ...): the page keeps the mouse until all are up.
        let down = matches!(message, 0x0201 | 0x0204 | 0x0207 | 0x020B);
        let up = matches!(message, 0x0202 | 0x0205 | 0x0208 | 0x020C);
        if down {
            self.pressed.set(Some(Instant::now()));
        }
        if down && !self.captured.replace(true) {
            window::set_capture(self.hwnd);
        }
        if up && wparam & MK_BUTTONS == 0 && self.captured.replace(false) {
            window::release_capture();
        }
        let keys = (wparam & 0xFFFF) as u32;
        let sent = live
            .comp
            .send_mouse_input(message, keys, data, px - x, py - y);
        if message != 0x0200 {
            tracing::debug!(
                message,
                x = px - x,
                y = py - y,
                ok = sent.is_ok(),
                "web plugin: mouse to the page"
            );
        }
        true
    }

    /// Whether the drag of `data` is one this page started (and PecoFence let start).
    pub fn started_drag(&self, data: &windows_core::IUnknown) -> bool {
        let identity = |u: &windows_core::IUnknown| {
            windows_core::Interface::cast::<windows_core::IUnknown>(u)
                .map(|u| windows_core::Interface::as_raw(&u))
                .ok()
        };
        let own = self.drag_data.borrow();
        own.as_ref()
            .and_then(identity)
            .is_some_and(|o| Some(o) == identity(data))
    }

    /// The page's own drag at screen point (`sx`, `sy`): over the page the page gets it (its
    /// drag and drop inside it works as in a browser), elsewhere on the fence it is told the
    /// drag left. `keys`: the key state and the effects the source allows. The effect the page
    /// takes (`DROPEFFECT_*`).
    pub fn page_drag(
        &self,
        step: DragStep,
        data: &windows_core::IUnknown,
        keys: (u32, u32),
        sx: i32,
        sy: i32,
    ) -> u32 {
        let live = self.live.borrow();
        let Some(live) = live.as_ref() else {
            return 0;
        };
        let p = window::screen_to_client(self.hwnd, pecofence_platform::POINT { x: sx, y: sy });
        let (x, y, ..) = self.bounds.get();
        let over = self.shown.get() && self.over_page(p.x, p.y) && step != DragStep::Leave;
        let (px, py) = (p.x - x, p.y - y);
        let inside = self.drag_inside.get();
        // The drag ends (left or dropped): its data object is held no longer.
        if matches!(step, DragStep::Leave | DragStep::Drop) {
            self.drag_data.borrow_mut().take();
        }
        // WebView2 takes a forwarded drag only while it allows external drops; the rest of the
        // time it allows none (the fence hands the page what it declared).
        let comp = &live.comp;
        let mut sent = Ok(0);
        if !over {
            self.drag_inside.set(false);
            if inside {
                sent = comp.drag(DragStep::Leave as u8, None, keys, px, py);
            }
        } else if !inside {
            let _ = live.controller.set_allow_external_drop(true);
            self.drag_inside.set(true);
            sent = comp.drag(DragStep::Enter as u8, Some(data), keys, px, py);
        } else if step == DragStep::Over {
            sent = comp.drag(DragStep::Over as u8, None, keys, px, py);
        }
        if over && step == DragStep::Drop {
            self.drag_inside.set(false);
            sent = comp.drag(DragStep::Drop as u8, Some(data), keys, px, py);
        }
        if !self.drag_inside.get() {
            let _ = live.controller.set_allow_external_drop(false);
        }
        if step != DragStep::Over {
            tracing::debug!(plugin = %self.plugin.id, ?step, over, effect = ?sent, "web plugin: the page's own drag over its page");
        }
        sent.unwrap_or(0)
    }

    /// Point (`px`, `py`) of the fence window's client area is over the page.
    fn over_page(&self, px: i32, py: i32) -> bool {
        let (x, y, w, h, _) = self.bounds.get();
        px >= x && py >= y && px < x + w && py < y + h
    }

    /// Starts `TIMER_WEB_INPUT`: at most 40 looks, 100 ms apart (`pass_mouse_through`).
    fn look_for_browser_window(&self) {
        self.input_tries.set(40);
        window::set_timer(self.hwnd, TIMER_WEB_INPUT, 100);
    }

    /// `TIMER_WEB_INPUT`. The browser lays a window of its own over the page (a top-level
    /// layered window, no activation): it would take the mouse that visual hosting has the
    /// fence send on (`mouse`), so the fence's resize edges, the keyboard hand-over and the wheel
    /// did not work. It gets WS_EX_TRANSPARENT: the mouse goes through it to the fence. Looked
    /// for every 100 ms until it is there (at most 4 s after the page loads or shows again). The
    /// log names the WebView2 version each time, so a runtime that works another way shows.
    pub fn pass_mouse_through(&self) {
        const WS_EX_TRANSPARENT: u32 = 0x20;
        const WS_EX_LAYERED: u32 = 0x8_0000;
        let pid = self
            .live
            .borrow()
            .as_ref()
            .map(|l| l.webview.browser_process_id());
        let (x, y, w, h, _) = self.bounds.get();
        let origin = window::client_to_screen(self.hwnd, pecofence_platform::POINT { x, y });
        let over = pecofence_platform::desktop::windows_of_class("Chrome_WidgetWin_1")
            .into_iter()
            .find(|&win| {
                let r = window::window_rect(win);
                Some(pecofence_platform::desktop::window_pid(win)) == pid
                    && (r.left, r.top, r.right - r.left, r.bottom - r.top)
                        == (origin.x, origin.y, w, h)
            });
        let tries = self.input_tries.get().saturating_sub(1);
        self.input_tries.set(tries);
        if let Some(win) = over {
            let ex = pecofence_platform::desktop::ex_style(win);
            if ex & WS_EX_LAYERED != 0 && ex & WS_EX_TRANSPARENT == 0 {
                pecofence_platform::desktop::set_ex_style(win, ex | WS_EX_TRANSPARENT);
                tracing::info!(plugin = %self.plugin.id, ex, version = self.version, "web plugin: the mouse goes through the browser's window to the fence");
            }
        } else if tries == 0 && self.shown.get() {
            tracing::info!(plugin = %self.plugin.id, version = self.version, "web plugin: no browser window over the page found, the fence gets the mouse as it is");
        }
        if over.is_some() || tries == 0 {
            window::kill_timer(self.hwnd, TIMER_WEB_INPUT);
        }
    }

    /// The fence window moved: the browser places its windows (popups, the input method's) anew.
    pub fn parent_moved(&self) {
        if let Some(live) = self.live.borrow().as_ref() {
            let _ = live.controller.notify_parent_window_position_changed();
        }
    }

    /// The capture went elsewhere (a menu, another window).
    pub fn capture_lost(&self) {
        self.captured.set(false);
    }

    /// WM_SETCURSOR: the page's cursor when the pointer is over it.
    pub fn set_cursor(&self) -> bool {
        let p = window::screen_to_client(self.hwnd, window::cursor_pos());
        let over = self.over_page(p.x, p.y);
        let live = self.live.borrow();
        match live.as_ref() {
            Some(live) if over && self.shown.get() => {
                window::set_cursor_handle(live.comp.cursor());
                true
            }
            _ => false,
        }
    }

    /// A press in the page: the keyboard and the input method go to it. The fence already has the
    /// foreground (`focus_session`); the page gives the keyboard back when the user clicks
    /// anywhere else, as any window does.
    pub fn focus(&self) {
        if let Some(live) = self.live.borrow().as_ref() {
            let ok = live
                .controller
                .move_focus(MoveFocusReason::Programmatic)
                .is_ok();
            tracing::debug!(plugin = %self.plugin.id, ok, "web plugin: the page has the keyboard");
        }
    }

    /// The page shows, or runs on hidden (`background`): the media session and the programs are
    /// read for it, and it hears of them.
    pub fn runs(&self) -> bool {
        self.shown.get() || self.plugin.background
    }

    /// The fence shows its page (false: rolled up, hidden with the desktop). A hidden page goes to
    /// low memory at once; 30 s later it stops drawing and is suspended (`suspend`). It keeps
    /// drawing until then: a fence expanded again within 30 s shows its page in the same frame as
    /// its glass (an invisible browser needs a frame to draw again).
    pub fn set_shown(&self, shown: bool) {
        if self.shown.replace(shown) == shown {
            return;
        }
        self.event(
            if shown { "show" } else { "hide" },
            &serde_json::Value::Null,
        );
        // The media session and the programs are read for it while it runs (`App::sync_media`).
        self.queue.push(Command::ToolShownChanged);
        if !shown && self.plugin.background {
            // A background plugin's page goes on as it is (music keeps playing).
            return tracing::info!(plugin = %self.plugin.id, fence = %self.fence, "web plugin: page hidden, runs on in the background");
        }
        if let Some(live) = self.live.borrow().as_ref() {
            let level = if shown {
                let _ = live.controller.set_visible(true);
                let _ = live.webview.resume();
                MemoryUsageTargetLevel::Normal
            } else {
                MemoryUsageTargetLevel::Low
            };
            let _ = live.webview.set_memory_usage_target_level(level);
        }
        if shown {
            window::kill_timer(self.hwnd, TIMER_WEB_SUSPEND);
            self.look_for_browser_window();
        } else {
            window::set_timer(self.hwnd, TIMER_WEB_SUSPEND, SUSPEND_AFTER_MS);
        }
        tracing::info!(plugin = %self.plugin.id, fence = %self.fence, shown, "web plugin: page shown or hidden");
    }

    /// `TIMER_WEB_SUSPEND`: the page has been hidden for 30 s.
    pub fn suspend(&self) {
        window::kill_timer(self.hwnd, TIMER_WEB_SUSPEND);
        if self.shown.get() {
            return;
        }
        if let Some(live) = self.live.borrow().as_ref() {
            // WebView2 suspends only an invisible browser.
            let _ = live.controller.set_visible(false);
            let id = self.plugin.id.clone();
            let asked = live.webview.try_suspend(move |r| {
                tracing::info!(plugin = %id, suspended = ?r, "web plugin: hidden page suspended");
            });
            if let Err(e) = asked {
                tracing::warn!(plugin = %self.plugin.id, error = %e, "web plugin: suspend refused");
            }
        }
    }
}

impl Drop for WebFence {
    fn drop(&mut self) {
        if let Some(live) = self.live.get_mut().take() {
            let _ = live.controller.close();
            tracing::info!(plugin = %self.plugin.id, fence = %self.fence, "web plugin: page closed");
        }
    }
}

fn scheme(dark: bool) -> PreferredColorScheme {
    if dark {
        PreferredColorScheme::Dark
    } else {
        PreferredColorScheme::Light
    }
}

/// `pf` for the page, made before its own scripts run (scripts the host adds are not subject
/// to the page's CSP). Every call goes to the app as `{id, call, args}` and comes back as
/// `{id, result}` or `{id, error}`; events come as `{event, data}`. A right-click the page does
/// not take (`preventDefault` on `contextmenu`) asks for the fence's menu; a navigation to another
/// site is cancelled in the page and told to the app (`navigate`, for the log).
fn bridge_script(origin: &str, init: &serde_json::Value, drags: bool) -> String {
    let origin = origin.trim_end_matches('/');
    // A WebView2 that cannot check a drag out of the page (`on_drag_starting`): none starts.
    let no_drags = if drags {
        ""
    } else {
        "addEventListener(\"dragstart\", e => e.preventDefault(), true);"
    };
    format!(
        r#"(() => {{
  // No WebRTC: it reaches hosts the CSP does not list. Connection-Allowlist blocks it in the
  // browser (WebView2 152 and later); removing it here covers older runtimes. The script runs in
  // every frame, blank ones too.
  for (const k of ["RTCPeerConnection", "webkitRTCPeerConnection", "RTCDataChannel", "RTCRtpSender", "RTCRtpReceiver", "RTCIceTransport"]) delete window[k];
  // No File System Access (the browser has it off too, `--disable-blink-features`): a file picked
  // or dropped there would be read or written without `folders`.
  for (const k of ["showOpenFilePicker", "showSaveFilePicker", "showDirectoryPicker"]) delete window[k];
  if (window.DataTransferItem) delete DataTransferItem.prototype.getAsFileSystemHandle;
  {no_drags}
  if (location.origin !== "{origin}" || window.top !== window || !window.chrome || !chrome.webview) return;
  const wv = chrome.webview, waiting = new Map(), handlers = {{}}, state = {init};
  // Ids start anywhere: an answer meant for the page before a reload finds nothing waiting.
  let n = Math.floor(Math.random() * 1e9), errors = 0;
  // An error nobody caught: the first three count as main.js's do after an update.
  const failed = () => {{ if (++errors <= 3) wv.postMessage({{ call: "errored", args: [] }}); }};
  addEventListener("error", failed);
  addEventListener("unhandledrejection", failed);
  wv.addEventListener("message", e => {{
    const m = e.data;
    if (m && m.event) {{
      if (m.event === "settings" || m.event === "theme" || m.event === "lang") state[m.event] = m.data;
      if (m.event === "resize") Object.assign(state.fence, m.data);
      for (const fn of handlers[m.event] || []) try {{ fn(m.data); }} catch (err) {{ console.error(err); failed(); }}
      return;
    }}
    const w = m && waiting.get(m.id);
    if (!w) return;
    waiting.delete(m.id);
    if ("error" in m) w[1](new Error(m.error)); else w[0](m.result);
  }});
  // A message over 1 MB is refused here: PecoFence drops it unanswered (as JSON in UTF-8, as it
  // measures).
  const call = (name, args) => new Promise((ok, fail) => {{
    const id = ++n, m = {{ id, call: name, args }};
    if (new TextEncoder().encode(JSON.stringify(m)).length > {max}) throw new RangeError(`pf.${{name}}: a message to PecoFence is at most 1 MB`);
    waiting.set(id, [ok, fail]);
    wv.postMessage(m);
  }});
  const pf = {{
    on(name, fn) {{ (handlers[name] = handlers[name] || []).push(fn); }},
    post: data => call("post", [data === undefined ? null : data]),
  }};
  for (const name of "{calls}".split(" ")) {{
    const [a, b] = name.split("."), f = (...args) => call(name, args);
    if (b) (pf[a] = pf[a] || {{}})[b] = f; else pf[a] = f;
  }}
  for (const k of Object.keys(pf)) if (typeof pf[k] === "object") Object.freeze(pf[k]);
  for (const k of ["lang", "theme", "settings", "fence"]) Object.defineProperty(pf, k, {{ get: () => state[k], enumerable: true }});
  Object.defineProperty(window, "pf", {{ value: Object.freeze(pf) }});
  addEventListener("contextmenu", e => setTimeout(() => {{ if (!e.defaultPrevented) wv.postMessage({{ call: "menu", args: [] }}); }}));
  // A link or a script that leads to another site or protocol stops here and the page stays:
  // the browser would block it (Connection-Allowlist) and show its error page in place of this one.
  navigation.addEventListener("navigate", e => {{
    if (!e.cancelable || new URL(e.destination.url).origin === location.origin) return;
    e.preventDefault();
    wv.postMessage({{ call: "navigate", args: [e.destination.url] }});
  }});
}})();"#,
        calls = CALLS,
        max = pecofence_plugin_host::MAX_POST,
        no_drags = no_drags,
    )
}

/// [`drag_refusal`] of what the drag `args` carries (logged).
fn page_drag_refusal(
    args: &windows_webview::DragStarting,
    hosts: &[String],
    origin: &str,
) -> Option<String> {
    use pecofence_platform::dragdrop as dd;
    let data = args
        .data()
        .and_then(|d| windows_core::Interface::cast::<dd::IDataObject>(&d));
    let data = match data {
        Ok(data) => data,
        Err(e) => return Some(format!("no data ({e})")),
    };
    let formats = dd::format_names(&data);
    let files = dd::file_group_descriptor_names(&data);
    let link = (formats
        .iter()
        .any(|f| f.starts_with("UniformResourceLocator")))
    .then(|| dd::inet_url_text(&data).unwrap_or_default());
    let why = drag_refusal(&formats, link.as_deref(), &files, hosts, origin);
    let shown = link.as_deref().map(short);
    tracing::info!(?formats, ?files, link = shown, refused = why, at = ?args.position(), "web plugin: a drag out of the page");
    why
}

/// Why a drag the page started may not leave it, if it may not: the allow-list of a plugin row's
/// drag (plan v6 change 2). No file of the computer (`CF_HDROP`, a file name, shell items: a
/// page has none of its own, and a download link would write any kind of file); a link (`link`,
/// read from `UniformResourceLocatorW`)
/// only to an http or https host of `hosts`, or to the page's own `origin` with a picture of it;
/// the files the page makes (`files`, a picture dragged) only of the data types, and the
/// link's own Internet Shortcut. Text and HTML go.
fn drag_refusal(
    formats: &[String],
    link: Option<&str>,
    files: &[String],
    hosts: &[String],
    origin: &str,
) -> Option<String> {
    const COMPUTER: [&str; 5] = [
        "#15",
        "FileName",
        "FileNameW",
        "Shell IDList Array",
        "Shell Object Offsets",
    ];
    if let Some(f) = formats.iter().find(|f| COMPUTER.contains(&f.as_str())) {
        return Some(format!("it carries files of the computer ({f})"));
    }
    let data = |n: &String| pecofence_core::folder_write::is_data_file(n);
    let pictures = !files.is_empty() && files.iter().all(data);
    let own = link.is_some_and(|l| l.starts_with(origin)) && pictures;
    let address = link.and_then(pecofence_core::web_address::WebAddress::parse);
    let listed = |h: &str| hosts.iter().any(|x| x.eq_ignore_ascii_case(h));
    match (link, &address) {
        (Some(_), _) if own => {}
        (Some(l), None) => return Some(format!("a link of another kind ({})", short(l))),
        (Some(_), Some(a)) if !listed(&a.host) => {
            return Some(format!("a link to {}, not in links or network", a.host));
        }
        _ => {}
    }
    let shortcut = |n: &String| address.is_some() && n.to_ascii_lowercase().ends_with(".url");
    let other = files.iter().find(|n| !data(n) && !shortcut(n));
    other.map(|n| {
        format!(
            "a file {n} of another type than .txt .md .json .csv .log .ics .png .jpg .gif .webp"
        )
    })
}

fn mime(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "gif" => "image/gif",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

/// An address for the log: no query, at most 80 characters.
pub(crate) fn short(uri: &str) -> String {
    uri.split('?')
        .next()
        .unwrap_or("")
        .chars()
        .take(80)
        .collect()
}

fn pointer_in_client(hwnd: HWND) -> bool {
    let p = window::screen_to_client(hwnd, window::cursor_pos());
    let (w, h) = window::client_size_of(hwnd);
    p.x >= 0 && p.y >= 0 && p.x < w && p.y < h
}

/// The profiles of uninstalled plugins that were in use then, deleted before WebView2 starts.
const LEFT_PROFILES: &str = "profiles-to-delete.txt";

/// Uninstall (`App::uninstall_plugin`): the plugin's WebView2 profile, its cookies and storage,
/// goes with it. A browser that still runs may hold its files: what is left is deleted before the
/// next browser starts (`delete_left_profiles`).
pub fn delete_profile(id: &str) {
    let data = crate::settings_host::user_data_folder();
    match std::fs::remove_dir_all(profile_dir(&data, id)) {
        Ok(()) => tracing::info!(plugin = id, "web plugin: profile deleted"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::info!(plugin = id, "web plugin: no profile to delete")
        }
        Err(e) => {
            use std::io::Write;
            let list = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(data.join(LEFT_PROFILES));
            let kept = list.and_then(|mut f| writeln!(f, "{id}")).is_ok();
            tracing::info!(plugin = id, error = %e, kept, "web plugin: profile in use, deleted before the browser starts next");
        }
    }
}

/// A plugin installed again (`App::switch_plugin`): the deletion its uninstall left for the next
/// start is tried now, and is no longer left, so the next start keeps the new install's data.
pub fn keep_profile(id: &str) {
    let data = crate::settings_host::user_data_folder();
    let list = data.join(LEFT_PROFILES);
    let Ok(text) = std::fs::read_to_string(&list) else {
        return;
    };
    if !text.lines().any(|l| l == id) {
        return;
    }
    let gone = std::fs::remove_dir_all(profile_dir(&data, id)).is_ok();
    let rest: String = text
        .lines()
        .filter(|l| *l != id)
        .map(|l| format!("{l}\n"))
        .collect();
    let kept = if rest.is_empty() {
        std::fs::remove_file(&list)
    } else {
        std::fs::write(&list, rest)
    };
    tracing::info!(
        plugin = id,
        gone,
        list_ok = kept.is_ok(),
        "web plugin: installed again, its left profile is no longer deleted at the next start"
    );
}

/// Before the browser starts (`WebEnvironment::create`): the profiles `delete_profile` left.
pub fn delete_left_profiles(data: &Path) {
    let list = data.join(LEFT_PROFILES);
    let Ok(text) = std::fs::read_to_string(&list) else {
        return;
    };
    for id in text
        .lines()
        .filter(|id| pecofence_core::plugins::valid_id(id))
    {
        let gone = std::fs::remove_dir_all(profile_dir(data, id));
        tracing::info!(
            plugin = id,
            ok = gone.is_ok(),
            "web plugin: profile left at the uninstall deleted"
        );
    }
    let _ = std::fs::remove_file(&list);
}

/// Where WebView2 keeps the profile `plugin-<id>` in the user data folder `data`.
fn profile_dir(data: &Path, id: &str) -> PathBuf {
    data.join("EBWebView")
        .join(format!("WV2Profile_plugin-{id}"))
}

#[cfg(test)]
mod tests {
    use super::drag_refusal;

    #[test]
    fn a_drag_out_of_a_page_carries_only_what_a_row_may() {
        let f = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let hosts = f(&["example.com"]);
        let origin = "pecofence-plugin://demo/";
        let refused = |formats: &[&str], link: Option<&str>, files: &[&str]| {
            drag_refusal(&f(formats), link, &f(files), &hosts, origin)
        };
        // Text, and a link to a listed host with its Internet Shortcut.
        assert_eq!(refused(&["#13", "HTML Format"], None, &[]), None);
        let link = Some("https://example.com/a");
        assert_eq!(
            refused(&["UniformResourceLocatorW", "#13"], link, &["A.url"]),
            None
        );
        // A picture of the page with its own address goes; that address alone does not.
        let own = Some("pecofence-plugin://demo/a.png");
        assert_eq!(refused(&["UniformResourceLocatorW"], own, &["a.png"]), None);
        assert!(refused(&["UniformResourceLocatorW"], own, &[]).is_some());
        assert!(
            refused(
                &["UniformResourceLocatorW"],
                Some("file:///C:/a.png"),
                &["a.png"]
            )
            .is_some()
        );
        assert!(
            refused(
                &["UniformResourceLocatorW"],
                Some("https://other.org/"),
                &[]
            )
            .is_some()
        );
        // Files of the computer, a file of another type, a shortcut without a link.
        assert!(refused(&["#15"], None, &[]).is_some());
        assert!(refused(&["FileNameW"], None, &[]).is_some());
        assert!(refused(&[], None, &["run.hta"]).is_some());
        assert!(refused(&[], None, &["a.url"]).is_some());
        assert!(refused(&[], link, &["a.png", "b.lnk"]).is_some());
    }
}
