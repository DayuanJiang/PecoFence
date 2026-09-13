//! Desktop z-order anchoring and "Show desktop" survival (plan §5.1, §5.2).

use crate::commands::WM_APP_SET_VISIBLE;
use pecofence_platform::desktop::{self, Generation, IconHost};
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::winevent::{self, WinEventHook};
use pecofence_platform::{HWND, POINT, RECT, Result, msg, rawinput, shell_icons};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

pub const FENCE_CLASS: &str = "PecoFence.Fence";
pub const SENTINEL_CLASS: &str = "PecoFence.Sentinel";

const TIMER_ANCHOR_CHECK: usize = 10;
const TIMER_RETRY: usize = 11;
const TIMER_QUICKHIDE: usize = 13;
const QUICKHIDE_DELAY_MS: u32 = 300;
pub const TIMER_TEST: usize = 20;

const RETRY_DELAYS_MS: [u32; 6] = [50, 150, 350, 750, 1500, 1200];
/// Retries of a failed "hide desktop icons" request (logon: Explorer's desktop not ready yet),
/// one per TIMER_ANCHOR_CHECK tick (~30 s at the 2 s cadence) before giving up.
const HIDE_ICONS_MAX_ATTEMPTS: u32 = 15;

thread_local! {
    static ANCHORING: Cell<bool> = const { Cell::new(false) };
}

/// Shows / hides a fence through its own window (`WM_APP_SET_VISIBLE`): the fence fades and
/// shows / hides its shadow itself. A window that does not answer (not ours, or gone) gets the
/// plain `ShowWindow` for itself and its shadow.
fn set_fence_visible(fence: HWND, shadow: Option<HWND>, show: bool) {
    if window::send_message(fence, WM_APP_SET_VISIBLE, show as usize, 0) == 1 {
        return;
    }
    for h in std::iter::once(fence).chain(shadow) {
        if show {
            desktop::show_no_activate(h);
        } else {
            desktop::hide_window(h);
        }
    }
}

/// True while this module is repositioning windows; fence handlers let z-order changes through
/// only in that case.
pub fn is_anchoring() -> bool {
    ANCHORING.with(|a| a.get())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZMode {
    /// Insert every fence directly above the icon host (default).
    InsertAboveHost,
    /// Push every fence to HWND_BOTTOM.
    HwndBottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShowDesktopBehavior {
    /// Keep fences visible on top of the raised desktop (Stardock default).
    KeepVisible,
    /// Let Win+D hide fences together with everything else.
    HideWithDesktop,
}

pub type AnchorCell = Rc<RefCell<Option<DesktopAnchor>>>;
pub type TestHook = Box<dyn FnMut(&mut DesktopAnchor) -> Option<u32>>;

/// Double-click-on-empty-desktop quick hide (plan §5.4).
#[derive(Default)]
struct QuickHide {
    fences_hidden: bool,
    sink_registered: bool,
    /// First click candidate: when and where.
    pending_click: Option<(Instant, POINT)>,
    /// Double-click confirmed; the toggle fires after a short delay unless a drag starts.
    pending_toggle: Option<POINT>,
    /// Left button went down on the bare desktop here (marquee candidate).
    marquee_origin: Option<POINT>,
}

/// Smallest marquee (device pixels, either axis) that counts as "draw a fence here".
const MARQUEE_MIN_PX: i32 = 48;

/// Real desktop icon visibility management (plan §5.3).
struct DesktopIcons {
    we_hid_them: bool,
    marker: PathBuf,
    /// A hide was requested (setting on) but has not succeeded yet; retried from ensure_anchored.
    pending: bool,
    pending_attempts: u32,
}

impl DesktopIcons {
    fn new() -> Self {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("PecoFence");
        // A second (test) instance keeps its own marker so it never adopts the main one's.
        let marker = match pecofence_core::brand::var("PECOFENCE_INSTANCE") {
            Ok(n) if !n.trim().is_empty() => format!("icons-hidden.{}.marker", n.trim()),
            _ => "icons-hidden.marker".to_string(),
        };
        let current = base.join(&marker);
        let legacy = base
            .with_file_name(pecofence_core::brand::LEGACY_DATA_DIR)
            .join(&marker);
        let marker = if !current.exists() && legacy.exists() {
            legacy
        } else {
            current
        };
        Self {
            we_hid_them: false,
            marker,
            pending: false,
            pending_attempts: 0,
        }
    }
}

pub struct DesktopAnchor {
    pub generation: Generation,
    pub mode: ZMode,
    pub behavior: ShowDesktopBehavior,
    host: Option<IconHost>,
    sentinel: Window,
    fences: Vec<(HWND, Option<HWND>)>,
    show_desktop_active: bool,
    retry_step: usize,
    _hook: Option<WinEventHook>,
    quick: QuickHide,
    icons: DesktopIcons,
    pub quick_hide_enabled: bool,
    /// Optional test script driven by TIMER_TEST on the sentinel window.
    pub test_hook: Option<TestHook>,
    /// Set by the test hook / owner to request process exit.
    #[allow(dead_code)]
    pub quit_requested: bool,
    /// Called with the screen rectangle when the user draws a marquee on the empty desktop.
    pub on_marquee: Option<Box<dyn Fn(RECT)>>,
    /// Fence windows (as raw handles) that stay visible during quick hide.
    quick_hide_excluded: std::collections::HashSet<isize>,
    /// Peek in progress: fences are topmost and anchoring is suspended.
    peeking: bool,
    /// Fences (hidden by quick hide) that Peek showed and must hide again afterwards.
    peek_shown: Vec<HWND>,
    /// Called when another application takes the foreground during a peek.
    pub on_peek_interrupted: Option<Box<dyn Fn()>>,
}

impl DesktopAnchor {
    /// Creates the anchor state, its sentinel window and the foreground hook. Returns the shared
    /// cell so window handlers can reach it.
    pub fn create(
        mode: ZMode,
        behavior: ShowDesktopBehavior,
        quick_hide_enabled: bool,
        sentinel_class: &WindowClass,
    ) -> Result<AnchorCell> {
        let cell: AnchorCell = Rc::new(RefCell::new(None));
        let generation = desktop::detect_generation();
        tracing::info!(
            ?generation,
            build = desktop::os_build(),
            "desktop generation"
        );

        let handler: MessageHandler = {
            let cell = cell.clone();
            Box::new(move |_hwnd, message, wparam, _lparam| match message {
                msg::WM_MOUSEACTIVATE => Some(msg::MA_NOACTIVATE),
                msg::WM_TIMER => {
                    if let Ok(mut guard) = cell.try_borrow_mut()
                        && let Some(anchor) = guard.as_mut()
                    {
                        anchor.on_timer(wparam);
                    }
                    Some(0)
                }
                msg::WM_INPUT => {
                    if let Ok(mut guard) = cell.try_borrow_mut()
                        && let Some(anchor) = guard.as_mut()
                    {
                        anchor.on_raw_input(_lparam);
                    }
                    // Per docs, WM_INPUT must still reach DefWindowProc for cleanup.
                    None
                }
                msg::WM_DESTROY => Some(0),
                _ => None,
            })
        };
        let sentinel = WindowBuilder::new(sentinel_class)
            .title("PecoFence sentinel")
            .style(style::POPUP)
            .ex_style(style::EX_NOACTIVATE | style::EX_TOOLWINDOW)
            .bounds(0, 0, 0, 0)
            .create(handler)?;
        sentinel.show_no_activate();

        let hook = {
            let cell = cell.clone();
            WinEventHook::install(
                winevent::SYSTEM_FOREGROUND,
                winevent::SYSTEM_FOREGROUND,
                Box::new(move |_event, hwnd| {
                    if let Ok(mut guard) = cell.try_borrow_mut()
                        && let Some(anchor) = guard.as_mut()
                    {
                        anchor.on_foreground(hwnd);
                    }
                }),
            )
        };
        let hook = match hook {
            Ok(h) => Some(h),
            Err(e) => {
                tracing::error!(error = %e, "SetWinEventHook failed; show-desktop detection disabled");
                None
            }
        };

        let anchor = DesktopAnchor {
            generation,
            mode,
            behavior,
            host: None,
            sentinel,
            fences: Vec::new(),
            show_desktop_active: false,
            retry_step: 0,
            _hook: hook,
            quick: QuickHide::default(),
            icons: DesktopIcons::new(),
            quick_hide_enabled,
            test_hook: None,
            quit_requested: false,
            on_marquee: None,
            quick_hide_excluded: std::collections::HashSet::new(),
            peeking: false,
            peek_shown: Vec::new(),
            on_peek_interrupted: None,
        };
        anchor
            .sentinel
            .set_coalescable_timer(TIMER_ANCHOR_CHECK, 2000, 1000);
        *cell.borrow_mut() = Some(anchor);
        Ok(cell)
    }

    pub fn register_fence(&mut self, hwnd: HWND, shadow: Option<HWND>) {
        if !self.fences.iter().any(|(h, _)| *h == hwnd) {
            self.fences.push((hwnd, shadow));
        }
    }

    pub fn unregister_fence(&mut self, hwnd: HWND) {
        self.fences.retain(|(h, _)| *h != hwnd);
    }

    /// Puts `hwnd` above the other fences (clicked / newly created), keeping all of them just
    /// above the icon host.
    pub fn raise_fence(&mut self, hwnd: HWND) {
        let Some(pos) = self.fences.iter().position(|(h, _)| *h == hwnd) else {
            return;
        };
        if pos + 1 == self.fences.len() {
            return;
        }
        let entry = self.fences.remove(pos);
        self.fences.push(entry);
        self.reanchor("raise fence");
    }

    #[allow(dead_code)]
    pub fn sentinel_window(&self) -> &Window {
        &self.sentinel
    }

    #[allow(dead_code)]
    pub fn host(&self) -> Option<IconHost> {
        self.host
    }

    fn all_windows(&self) -> Vec<HWND> {
        let mut v: Vec<HWND> = Vec::with_capacity(self.fences.len() * 2 + 1);
        for (f, s) in &self.fences {
            v.push(*f);
            if let Some(s) = s {
                v.push(*s);
            }
        }
        v.push(self.sentinel.hwnd());
        v
    }

    /// Peek in progress: keep every fence (and shadow) in the topmost band, ordered like
    /// `fences` (last = on top). Used instead of host insertion while `peeking`, so windows
    /// created / detached / raised mid-peek end up above the dimmer instead of behind it.
    fn place_peek_band(&mut self) {
        self.with_anchoring(|s| {
            for (f, shadow) in s.fences.clone() {
                for h in std::iter::once(f).chain(shadow) {
                    // Fences created during the peek (resync_windows) are not topmost yet.
                    let _ = desktop::set_topmost(h, true);
                }
                let _ = desktop::raise_in_band(f);
                if let Some(sh) = shadow {
                    let _ = desktop::insert_after(sh, f);
                }
            }
        });
    }

    /// Re-resolves the icon host and re-inserts every fence (and the sentinel) above it.
    pub fn reanchor(&mut self, reason: &str) {
        if self.peeking {
            // Fences live in the topmost band during Peek; insert_above / send_to_bottom would
            // strip WS_EX_TOPMOST and drop them behind the dimmer. Re-apply the peek band
            // instead. (`set_peek(false)` clears `peeking` before calling us, so the restore
            // path still does the normal desktop-band placement.)
            tracing::debug!(reason, "reanchor during peek: reordering in topmost band");
            self.place_peek_band();
            return;
        }
        let resolved = desktop::resolve_icon_host(self.generation);
        if resolved != self.host {
            tracing::info!(reason, old = ?self.host, new = ?resolved, "icon host changed");
            self.host = resolved;
        } else {
            tracing::debug!(reason, "reanchor");
        }
        let Some(host) = self.host else {
            tracing::warn!("no icon host found; falling back to HWND_BOTTOM");
            self.with_anchoring(|s| {
                for w in s.all_windows() {
                    let _ = desktop::send_to_bottom(w);
                }
            });
            return;
        };
        self.with_anchoring(|s| {
            let mode = s.mode;
            let place = |w: HWND| -> Result<()> {
                match mode {
                    ZMode::InsertAboveHost => desktop::insert_above(w, host.host),
                    ZMode::HwndBottom => desktop::send_to_bottom(w),
                }
            };
            let _ = place(s.sentinel.hwnd());
            // Each fence is inserted directly above the host, so the *last* one placed ends up
            // lowest. Iterate in reverse: the end of `fences` (newest / last raised) is on top.
            for (f, shadow) in s.fences.clone().into_iter().rev() {
                if let Err(e) = place(f) {
                    tracing::warn!(hwnd = ?f, error = %e, "anchor SetWindowPos failed");
                }
                if let Some(sh) = shadow {
                    let _ = desktop::insert_after(sh, f);
                }
            }
        });
    }

    fn with_anchoring(&mut self, f: impl FnOnce(&mut Self)) {
        ANCHORING.with(|a| a.set(true));
        f(self);
        ANCHORING.with(|a| a.set(false));
    }

    /// Is `hwnd`'s root window the desktop icon host (or another Explorer desktop window)?
    fn is_desktop_window(&self, hwnd: HWND) -> bool {
        if hwnd.0.is_null() {
            return false;
        }
        let root = desktop::root_ancestor(hwnd);
        if let Some(h) = self.host
            && root == h.host
        {
            return true;
        }
        let cls = desktop::class_name(root);
        (cls == "Progman" || cls == "WorkerW")
            && desktop::window_pid(root) == desktop::window_pid(desktop::shell_window())
    }

    /// Have our windows ended up *below* the host? (Explorer raised the desktop.)
    fn desktop_is_raised_over_us(&self) -> bool {
        let Some(h) = self.host else { return false };
        desktop::is_below(h.host, SENTINEL_CLASS) || desktop::is_below(h.host, FENCE_CLASS)
    }

    /// Are all our windows directly above the host (contiguous block ending at host)?
    fn is_anchored(&self) -> bool {
        let Some(h) = self.host else { return false };
        let ours = self.all_windows();
        let windows = desktop::top_level_windows();
        let Some(host_index) = windows.iter().position(|&w| w == h.host) else {
            return false;
        };
        let mut remaining = ours.len();
        for &cursor in windows[..host_index].iter().rev() {
            if ours.contains(&cursor) {
                remaining -= 1;
                if remaining == 0 {
                    return true;
                }
            } else if desktop::has_visible_surface(cursor) {
                return false;
            }
        }
        false
    }

    pub fn on_foreground(&mut self, hwnd: HWND) {
        let root = desktop::root_ancestor(hwnd);
        let cls = desktop::class_name(root);
        if self.peeking {
            // Peek is a glance: the first foreign window that comes to the front ends it. Our
            // own windows (fences, dimmer, rename popup, menus) do not.
            let ours = cls == FENCE_CLASS
                || cls == SENTINEL_CLASS
                || cls.starts_with("PecoFence.")
                || cls == "#32768";
            if !ours && !root.0.is_null() {
                tracing::debug!(%cls, "foreground changed during peek; ending it");
                if let Some(cb) = self.on_peek_interrupted.as_ref() {
                    cb();
                }
            }
            return;
        }
        let is_desktop = self.is_desktop_window(hwnd);
        let raised = self.desktop_is_raised_over_us();
        tracing::debug!(?hwnd, %cls, is_desktop, raised, "foreground changed");

        if is_desktop {
            self.set_raw_input_sink(true);
            self.quick.pending_click = Some((Instant::now(), window::cursor_pos()));
        } else {
            self.set_raw_input_sink(false);
            self.quick.pending_click = None;
            self.quick.marquee_origin = None;
        }

        if is_desktop && raised {
            if !self.show_desktop_active {
                tracing::info!("show-desktop detected (desktop raised above fences)");
            }
            self.show_desktop_active = true;
            if self.behavior == ShowDesktopBehavior::HideWithDesktop {
                return;
            }
        } else if !is_desktop && self.show_desktop_active {
            tracing::info!("show-desktop ended");
            self.show_desktop_active = false;
        }

        // Explorer may reorder the host slightly after the foreground event, so verify now and
        // again on a short retry ladder.
        self.ensure_anchored("foreground");
        self.retry_step = 0;
        self.sentinel.set_timer(TIMER_RETRY, RETRY_DELAYS_MS[0]);
    }

    /// Peek (Fences Win+Space): float every fence into the topmost band above `over` (the
    /// dimmer windows, if any) and pause anchoring; `false` restores the desktop z-band.
    pub fn set_peek(&mut self, on: bool, over: &[HWND]) {
        if on == self.peeking {
            return;
        }
        self.peeking = on;
        if on {
            self.with_anchoring(|s| {
                for w in over {
                    let _ = desktop::set_topmost(*w, true);
                }
                for (f, shadow) in s.fences.clone() {
                    if !desktop::is_visible(f) {
                        // Quick-hidden: fade it in for the peek (its shadow follows on its own).
                        set_fence_visible(f, shadow, true);
                        s.peek_shown.push(f);
                    }
                    for h in std::iter::once(f).chain(shadow) {
                        let _ = desktop::set_topmost(h, true);
                    }
                    let _ = desktop::raise_in_band(f);
                    if let Some(sh) = shadow {
                        let _ = desktop::insert_after(sh, f);
                    }
                }
            });
            tracing::info!(count = self.fences.len(), "peek on");
        } else {
            self.with_anchoring(|s| {
                for (f, shadow) in s.fences.clone() {
                    for h in std::iter::once(f).chain(shadow) {
                        let _ = desktop::set_topmost(h, false);
                    }
                }
                for h in std::mem::take(&mut s.peek_shown) {
                    let shadow = s.fences.iter().find(|(f, _)| *f == h).and_then(|(_, s)| *s);
                    set_fence_visible(h, shadow, false);
                }
            });
            self.reanchor("peek off");
            tracing::info!("peek off");
        }
    }

    /// Re-anchors only if the host changed or our block is no longer directly above it.
    fn ensure_anchored(&mut self, reason: &str) {
        if self.peeking {
            return;
        }
        // A hide that failed at startup (desktop not ready) is retried on every tick, not only
        // when the z-order is off, so the setting converges with reality without a toggle.
        if self.icons.pending && self.icons.pending_attempts < HIDE_ICONS_MAX_ATTEMPTS {
            self.icons.pending_attempts += 1;
            if self.hide_desktop_icons() {
                tracing::info!(
                    attempts = self.icons.pending_attempts,
                    "desktop icons hidden on retry"
                );
                self.icons.pending = false;
                self.icons.pending_attempts = 0;
            }
        }
        let resolved = desktop::resolve_icon_host(self.generation);
        if resolved != self.host || !self.is_anchored() {
            self.reanchor(reason);
            for (w, _) in self.fences.clone() {
                if desktop::is_cloaked(w) {
                    tracing::info!(hwnd = ?w, "fence was cloaked; uncloaking");
                    let _ = desktop::set_cloak(w, false);
                }
                if (!desktop::is_visible(w) || desktop::is_iconic(w)) && !self.quick.fences_hidden {
                    tracing::info!(hwnd = ?w, "fence hidden/minimized; showing");
                    desktop::show_no_activate(w);
                }
            }
        }
    }

    fn set_raw_input_sink(&mut self, on: bool) {
        if on == self.quick.sink_registered {
            return;
        }
        let r = if on {
            rawinput::register_mouse_sink(self.sentinel.hwnd())
        } else {
            rawinput::unregister_mouse_sink()
        };
        match r {
            Ok(()) => {
                self.quick.sink_registered = on;
                tracing::debug!(on, "raw input sink");
            }
            Err(e) => tracing::warn!(on, error = %e, "RegisterRawInputDevices failed"),
        }
    }

    /// `WM_INPUT` on the sentinel: left button down/up on the bare desktop drive quick hide
    /// (double-click) and the marquee → new fence gesture.
    fn on_raw_input(&mut self, lparam: isize) {
        let Some(buttons) = rawinput::mouse_buttons_from_wm_input(lparam) else {
            return;
        };
        if buttons.left_up {
            self.on_desktop_button_up();
            // Second click of the double-click released without dragging: toggle now rather
            // than 300 ms later (TIMER_QUICKHIDE stays armed as the fallback for a lost up).
            if let Some(origin) = self.quick.pending_toggle.take() {
                self.sentinel.kill_timer(TIMER_QUICKHIDE);
                if self.quick_hide_drag_started(origin) {
                    tracing::debug!("quick hide cancelled (drag/marquee started)");
                } else {
                    self.toggle_fences_hidden();
                }
            }
        }
        if !buttons.left_down {
            return;
        }
        let pt = window::cursor_pos();
        let hit = desktop::window_from_point(pt.x, pt.y);
        let root = desktop::root_ancestor(hit);
        if self.all_windows().contains(&root) || !self.is_desktop_window(hit) {
            self.quick.pending_click = None;
            self.quick.marquee_origin = None;
            return;
        }
        if !self.quick.fences_hidden {
            self.quick.marquee_origin = Some(pt);
        }
        if !self.quick_hide_enabled {
            return;
        }
        let now = Instant::now();
        let (sx, sy) = rawinput::double_click_slop_px();
        let limit = rawinput::double_click_time_ms() as u128;
        if let Some((t0, p0)) = self.quick.pending_click.take()
            && now.duration_since(t0).as_millis() <= limit
            && (pt.x - p0.x).abs() <= sx.max(4)
            && (pt.y - p0.y).abs() <= sy.max(4)
        {
            tracing::info!(?pt, "double-click on empty desktop → quick hide on release");
            self.quick.pending_toggle = Some(pt);
            self.sentinel.set_timer(TIMER_QUICKHIDE, QUICKHIDE_DELAY_MS);
        } else {
            self.quick.pending_click = Some((now, pt));
        }
    }

    /// Left button released: a large enough drag that started on the bare desktop becomes a
    /// "new fence here" offer. Explorer's own marquee selects nothing while icons are hidden.
    fn on_desktop_button_up(&mut self) {
        let Some(origin) = self.quick.marquee_origin.take() else {
            return;
        };
        let pt = window::cursor_pos();
        let w = (pt.x - origin.x).abs();
        let h = (pt.y - origin.y).abs();
        if w < MARQUEE_MIN_PX || h < MARQUEE_MIN_PX {
            return;
        }
        let rect = RECT {
            left: origin.x.min(pt.x),
            top: origin.y.min(pt.y),
            right: origin.x.max(pt.x),
            bottom: origin.y.max(pt.y),
        };
        tracing::info!(?rect, "desktop marquee → offer new fence");
        // A marquee that began with the second click of a double-click is not a quick hide.
        self.quick.pending_toggle = None;
        self.sentinel.kill_timer(TIMER_QUICKHIDE);
        if let Some(cb) = self.on_marquee.as_ref() {
            cb(rect);
        }
    }

    /// The pointer left the double-click spot (twice the drag threshold) since the second
    /// click: the user is dragging a marquee, not asking for a quick hide.
    fn quick_hide_drag_started(&self, origin: POINT) -> bool {
        let now = window::cursor_pos();
        let (sx, sy) = window::drag_threshold();
        (now.x - origin.x).abs() > sx * 2 || (now.y - origin.y).abs() > sy * 2
    }

    pub fn fences_hidden(&self) -> bool {
        self.quick.fences_hidden
    }

    /// Marks a fence as exempt from quick hide (Fences "排除出快速隐藏").
    pub fn set_quick_hide_excluded(&mut self, hwnd: HWND, excluded: bool) {
        if excluded {
            self.quick_hide_excluded.insert(hwnd.0 as isize);
        } else {
            self.quick_hide_excluded.remove(&(hwnd.0 as isize));
        }
    }

    pub fn toggle_fences_hidden(&mut self) {
        let hide = !self.quick.fences_hidden;
        self.quick.fences_hidden = hide;
        for (w, shadow) in self.fences.clone() {
            if self.quick_hide_excluded.contains(&(w.0 as isize)) {
                continue;
            }
            set_fence_visible(w, shadow, !hide);
        }
        if !hide {
            self.reanchor("fences shown again");
        }
        tracing::info!(
            hidden = hide,
            count = self.fences.len(),
            "fences visibility toggled"
        );
    }

    /// Hides the real desktop icons (writing the crash-recovery marker first).
    pub fn hide_desktop_icons(&mut self) -> bool {
        if self.host.is_none() {
            self.host = desktop::resolve_icon_host(self.generation);
        }
        let Some(h) = self.host else { return false };
        if shell_icons::desktop_icons_hidden() {
            if self.icons.marker.exists() {
                // Our previous run (and its watchdog) died without restoring: take ownership
                // again so exit restores the icons and a fresh watchdog covers a crash.
                tracing::info!("desktop icons still hidden by our previous run; adopting");
                self.icons.we_hid_them = true;
                self.spawn_watchdog();
            } else {
                tracing::info!("desktop icons already hidden by user/other tool; leaving as is");
            }
            return true;
        }
        if let Some(dir) = self.icons.marker.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&self.icons.marker, b"pecofence hid desktop icons\n");
        match shell_icons::set_desktop_icons_hidden(true, h.def_view, h.host) {
            Some(true) => {
                self.icons.we_hid_them = true;
                tracing::info!("desktop icons hidden (0x7402)");
                self.spawn_watchdog();
                true
            }
            other => {
                tracing::warn!(?other, "hiding desktop icons failed");
                let _ = std::fs::remove_file(&self.icons.marker);
                false
            }
        }
    }

    /// Requests hiding; if it cannot be done now (Explorer not ready) keeps retrying from
    /// ensure_anchored until it works or the retry budget is spent.
    pub fn request_hide_desktop_icons(&mut self) -> bool {
        if self.hide_desktop_icons() {
            self.icons.pending = false;
            self.icons.pending_attempts = 0;
            return true;
        }
        self.icons.pending = true;
        self.icons.pending_attempts = 0;
        tracing::warn!("hiding desktop icons failed; will retry when the desktop is ready");
        false
    }

    /// True after the retry budget was exhausted without success (caller should reset the
    /// setting and tell the user).
    pub fn hide_desktop_icons_gave_up(&self) -> bool {
        self.icons.pending && self.icons.pending_attempts >= HIDE_ICONS_MAX_ATTEMPTS
    }

    pub fn cancel_pending_hide(&mut self) {
        self.icons.pending = false;
        self.icons.pending_attempts = 0;
    }

    /// Starts `pecofence-watchdog.exe` (next to our exe) so a crash still restores the icons.
    fn spawn_watchdog(&self) {
        let Ok(exe) = std::env::current_exe() else {
            return;
        };
        let Some(dir) = exe.parent() else { return };
        let wd = dir.join("pecofence-watchdog.exe");
        if !wd.exists() {
            tracing::warn!(path = %wd.display(), "watchdog executable missing; no crash protection");
            return;
        }
        let pid = pecofence_platform::process::current_pid().to_string();
        let marker = self.icons.marker.to_string_lossy().to_string();
        match pecofence_platform::process::spawn_detached(&wd.to_string_lossy(), &[&pid, &marker]) {
            Ok(wpid) => tracing::info!(watchdog_pid = wpid, "watchdog started"),
            Err(e) => tracing::warn!(error = %e, "watchdog spawn failed"),
        }
    }

    /// Restores the icons if (and only if) we hid them. Also cancels a pending retry, so turning
    /// the setting off never hides the icons later.
    pub fn restore_desktop_icons(&mut self) {
        self.cancel_pending_hide();
        if !self.icons.we_hid_them {
            return;
        }
        let host = self
            .host
            .or_else(|| desktop::resolve_icon_host(self.generation));
        if let Some(h) = host {
            match shell_icons::set_desktop_icons_hidden(false, h.def_view, h.host) {
                Some(false) => tracing::info!("desktop icons restored"),
                other => {
                    tracing::warn!(?other, "restoring desktop icons failed");
                    // Keep ownership and the crash-recovery marker until restoration
                    // succeeds. Explorer can be restarting or temporarily unresponsive.
                    return;
                }
            }
        } else {
            return;
        }
        self.icons.we_hid_them = false;
        let _ = std::fs::remove_file(&self.icons.marker);
    }

    /// True when a previous run left the marker behind (crashed without restoring).
    pub fn stale_marker_present(&self) -> bool {
        self.icons.marker.exists()
    }

    fn on_timer(&mut self, id: usize) {
        match id {
            TIMER_ANCHOR_CHECK => {
                if self.behavior == ShowDesktopBehavior::HideWithDesktop && self.show_desktop_active
                {
                    return;
                }
                self.ensure_anchored("periodic check");
            }
            TIMER_RETRY => {
                self.sentinel.kill_timer(TIMER_RETRY);
                self.ensure_anchored("retry");
                self.retry_step += 1;
                if self.retry_step < RETRY_DELAYS_MS.len() {
                    self.sentinel
                        .set_timer(TIMER_RETRY, RETRY_DELAYS_MS[self.retry_step]);
                }
            }
            TIMER_QUICKHIDE => {
                self.sentinel.kill_timer(TIMER_QUICKHIDE);
                if let Some(origin) = self.quick.pending_toggle.take() {
                    let moved = self.quick_hide_drag_started(origin);
                    let still_down = window::key_down(msg::VK_LBUTTON);
                    if moved || still_down {
                        tracing::debug!("quick hide cancelled (drag/marquee started)");
                    } else {
                        self.toggle_fences_hidden();
                    }
                }
            }
            TIMER_TEST => {
                self.sentinel.kill_timer(TIMER_TEST);
                if let Some(mut hook) = self.test_hook.take() {
                    let next = hook(self);
                    self.test_hook = Some(hook);
                    if let Some(ms) = next {
                        self.sentinel.set_timer(TIMER_TEST, ms);
                    }
                }
            }
            _ => {}
        }
    }

    /// Starts the test script after `delay_ms`.
    #[allow(dead_code)]
    pub fn start_test(&self, delay_ms: u32) {
        self.sentinel.set_timer(TIMER_TEST, delay_ms);
    }

    pub fn zorder_report(&self, all_visible: bool) -> String {
        desktop::describe_zorder(&self.all_windows(), all_visible)
    }
}

/// Registers the sentinel window class.
pub fn register_sentinel_class() -> Result<WindowClass> {
    WindowClass::register(SENTINEL_CLASS, ClassOptions::default())
}

impl Drop for DesktopAnchor {
    fn drop(&mut self) {
        self.restore_desktop_icons();
        self.set_raw_input_sink(false);
    }
}
