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

/// Our own click-through drag feedback (the drop outline sits directly below the dragged fence,
/// the desktop marquee band directly above the icon host, both inside the block): not a
/// foreign window that broke the anchoring.
fn is_drag_overlay(hwnd: HWND) -> bool {
    let cls = desktop::class_name(hwnd);
    (cls == crate::drop_preview::DROP_PREVIEW_CLASS
        || cls == crate::drag_guides::DRAG_GUIDE_CLASS
        || cls.starts_with(crate::marquee_band::MARQUEE_BAND_CLASS))
        && desktop::window_pid(hwnd) == std::process::id()
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
    /// The pointer left the drag threshold since that press: the marquee is being drawn.
    marquee_live: bool,
    /// Cursor position last reported with `MarqueeEvent::Moved`.
    marquee_last: Option<POINT>,
}

/// Smallest marquee (device pixels, either axis) that counts as "draw a fence here".
pub const MARQUEE_MIN_PX: i32 = 48;

/// A drag on the bare desktop as it happens (Explorer's own marquee selects nothing while the
/// icons are hidden): the fences it touches are selected, and one that touches no fence offers
/// a new fence there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarqueeEvent {
    /// The left button went down on the empty desktop (or the desktop was just activated);
    /// `additive` = Ctrl or Shift held, which keeps the current selection.
    Pressed { additive: bool },
    /// The marquee now spans this screen rectangle (past the drag threshold).
    Moved(RECT),
    /// Released: the final marquee.
    Released(RECT),
    /// Another window took the foreground mid-drag: the marquee ends where it was.
    Cancelled,
}

fn marquee_rect(a: POINT, b: POINT) -> RECT {
    RECT {
        left: a.x.min(b.x),
        top: a.y.min(b.y),
        right: a.x.max(b.x),
        bottom: a.y.max(b.y),
    }
}

/// A press on one of Windows' own desktop icons (they show while 「隐藏 Windows 桌面图标」 is off)
/// selects, opens or drags that icon: it starts no marquee and counts for no quick hide.
fn on_icon(hit: HWND, pt: POINT) -> bool {
    let on = shell_icons::desktop_icon_hovered(hit);
    if on {
        tracing::debug!(?pt, "press on a Windows desktop icon");
    }
    on
}

/// Ctrl or Shift is down (physical state; the desktop, not us, has the keyboard focus).
fn additive_modifier() -> bool {
    window::key_down_async(msg::VK_CONTROL) || window::key_down_async(msg::VK_SHIFT)
}

/// Real desktop icon visibility management (plan §5.3).
struct DesktopIcons {
    we_hid_them: bool,
    marker: PathBuf,
    /// A hide was requested (setting on) but has not succeeded yet; retried from ensure_anchored.
    pending: bool,
    pending_attempts: u32,
    /// The watchdog started this run (one at most); it waits for this process to exit.
    watchdogs: Vec<u32>,
}

impl DesktopIcons {
    /// Next to the marker: exit left the icons hidden on purpose (`keep_icons_hidden`).
    fn kept_marker(&self) -> PathBuf {
        self.marker.with_extension("kept")
    }

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
            watchdogs: Vec::new(),
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
    /// `Settings::keep_icons_hidden`: exit leaves the icons hidden (`release_desktop_icons`).
    pub keep_icons_hidden: bool,
    pub quick_hide_enabled: bool,
    /// Optional test script driven by TIMER_TEST on the sentinel window.
    pub test_hook: Option<TestHook>,
    /// Set by the test hook / owner to request process exit.
    #[allow(dead_code)]
    pub quit_requested: bool,
    /// Presses, drags and releases on the empty desktop (fence selection / new fence).
    pub on_marquee: Option<Box<dyn Fn(MarqueeEvent)>>,
    /// Fence windows (as raw handles) that stay visible during quick hide.
    quick_hide_excluded: std::collections::HashSet<isize>,
    /// Peek in progress: fences are topmost and anchoring is suspended.
    peeking: bool,
    /// Fences (hidden by quick hide) that Peek showed and must hide again afterwards.
    peek_shown: Vec<HWND>,
    /// Called when another application takes the foreground during a peek.
    pub on_peek_interrupted: Option<Box<dyn Fn()>>,
    /// Foreground transitions also signal virtual-desktop switches before registry writes
    /// (called with the new foreground window).
    pub on_foreground_changed: Option<Box<dyn Fn(HWND)>>,
    /// The desktop came to the front: clicked, or shown with Win+D (which Windows 10 does not
    /// always report as a foreground change).
    pub on_desktop_front: Option<Box<dyn Fn()>>,
    /// The last refused placement above the host (window above it, HRESULT), logged once until
    /// the cause changes or the placement works again.
    anchor_refused: Option<(isize, i32)>,
    /// A topmost-band placement failed during this peek / show desktop (logged once).
    topmost_refused: bool,
}

/// The fences' windows as a feedback report describes them.
pub struct WindowReport {
    pub fences: usize,
    pub visible: usize,
    pub cloaked: usize,
    pub topmost: usize,
    /// Below the icon host in the z-order: covered by the desktop.
    pub below_host: usize,
    /// On no monitor at all.
    pub off_screen: usize,
    pub quick_hidden: bool,
    pub peeking: bool,
    pub show_desktop: bool,
    pub host: Option<desktop::HostKind>,
}

/// Windows 11's shell popups (snap layouts after Win+Up, the task switcher) take the foreground
/// for a moment and hand it back without another foreground event. Taken for an app, they ended
/// a Peek, turned the desktop's double-click off, or ended show desktop.
fn is_shell_popup(class: &str) -> bool {
    class == "XamlExplorerHostIslandWindow"
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
            ?mode,
            ?behavior,
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

        let mut anchor = DesktopAnchor {
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
            keep_icons_hidden: false,
            quick_hide_enabled,
            test_hook: None,
            quit_requested: false,
            on_marquee: None,
            quick_hide_excluded: std::collections::HashSet::new(),
            peeking: false,
            peek_shown: Vec::new(),
            on_peek_interrupted: None,
            on_foreground_changed: None,
            on_desktop_front: None,
            anchor_refused: None,
            topmost_refused: false,
        };
        // The hook reports only changes of the foreground. With the desktop already in front
        // (a start at sign-in before any window opens) its clicks are followed from now on.
        if anchor.is_desktop_window(desktop::foreground_window()) {
            tracing::info!("desktop in front at start; following its clicks");
            anchor.set_raw_input_sink(true);
        }
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

    /// Peek or show desktop in progress: keep every fence (and shadow) in the topmost band,
    /// ordered like `fences` (last = on top). Used instead of host insertion while `peeking`
    /// or while the raised desktop would cover the fences, so windows created / detached /
    /// raised meanwhile end up above the dimmer or the desktop instead of behind it.
    fn place_topmost_band(&mut self) {
        let mut refused = None;
        self.with_anchoring(|s| {
            for (f, shadow) in s.fences.clone() {
                for h in std::iter::once(f).chain(shadow) {
                    // Fences created during the peek (resync_windows) are not topmost yet.
                    if let Err(e) = desktop::set_topmost(h, true) {
                        refused.get_or_insert(e);
                    }
                }
                if let Err(e) = desktop::raise_in_band(f) {
                    refused.get_or_insert(e);
                }
                if let Some(sh) = shadow {
                    let _ = desktop::insert_after(sh, f);
                }
            }
        });
        if let Some(e) = refused
            && !std::mem::replace(&mut self.topmost_refused, true)
        {
            tracing::warn!(error = %e, "topmost placement failed");
        }
    }

    /// Re-resolves the icon host and re-inserts every fence (and the sentinel) above it.
    pub fn reanchor(&mut self, reason: &'static str) {
        if self.peeking {
            // Fences live in the topmost band during Peek; insert_above / send_to_bottom would
            // strip WS_EX_TOPMOST and drop them behind the dimmer. Re-apply the peek band
            // instead. (`set_peek(false)` clears `peeking` before calling us, so the restore
            // path still does the normal desktop-band placement.)
            tracing::debug!(reason, "reanchor during peek: reordering in topmost band");
            self.place_topmost_band();
            return;
        }
        if self.show_desktop_active && self.behavior == ShowDesktopBehavior::KeepVisible {
            tracing::debug!(
                reason,
                "reanchor during show desktop: reordering in topmost band"
            );
            self.place_topmost_band();
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
        let mut refused = None;
        let mut failed = 0;
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
                    tracing::debug!(hwnd = ?f, error = %e, "anchor SetWindowPos failed");
                    failed += 1;
                    refused.get_or_insert((f, e));
                }
                if let Some(sh) = shadow {
                    let _ = desktop::insert_after(sh, f);
                }
            }
        });
        self.note_refusal(host.host, refused, failed);
    }

    /// Logs a refused placement once per cause (the window it had to go under, the error):
    /// the anchor check repeats it every pass. Access denied there means that window belongs
    /// to a process running as administrator.
    fn note_refusal(
        &mut self,
        host: HWND,
        refused: Option<(HWND, pecofence_platform::Error)>,
        failed: u32,
    ) {
        let Some((fence, e)) = refused else {
            if self.anchor_refused.take().is_some() {
                tracing::info!("anchoring works again");
            }
            return;
        };
        let above = desktop::surface_window_above(host, fence);
        let cause = (above.0 as isize, e.code().0);
        if self.anchor_refused == Some(cause) {
            return;
        }
        self.anchor_refused = Some(cause);
        let class = desktop::class_name(above);
        let above_elevated = (!above.0.is_null())
            .then(|| pecofence_platform::process::is_elevated(desktop::window_pid(above)))
            .flatten();
        tracing::warn!(
            error = %e,
            failed,
            above = ?above,
            class = %class,
            above_elevated = ?above_elevated,
            "anchor SetWindowPos failed"
        );
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

    /// A window of this process (fence, shadow, settings, menu, rename popup).
    fn is_own_window(&self, hwnd: HWND) -> bool {
        desktop::window_pid(desktop::root_ancestor(hwnd)) == std::process::id()
    }

    /// Have our windows ended up *below* the host? (Explorer raised the desktop.)
    fn desktop_is_raised_over_us(&self) -> bool {
        let Some(h) = self.host else { return false };
        desktop::is_below(h.host, SENTINEL_CLASS) || desktop::is_below(h.host, FENCE_CLASS)
    }

    /// Show desktop (Win+D): Explorer raises the desktop window over every other window, the
    /// fences included, and gives it the foreground; it is over once another window comes to
    /// the front. On 24H2+ Progman itself moves up (the fences keep their place in the z-order,
    /// so inserting them "just above" it again is not reliable), on older builds the WorkerW
    /// the icons were moved into, which `reanchor` follows as a host change.
    fn check_show_desktop(&mut self) {
        let foreground = desktop::foreground_window();
        if !self.show_desktop_active {
            if self.desktop_is_raised_over_us() && self.is_desktop_window(foreground) {
                self.show_desktop_active = true;
                if let Some(callback) = self.on_desktop_front.as_ref() {
                    callback();
                }
                match self.behavior {
                    ShowDesktopBehavior::KeepVisible => {
                        tracing::info!("show desktop: floating the fences above the raised desktop")
                    }
                    ShowDesktopBehavior::HideWithDesktop => {
                        tracing::info!("show desktop: fences stay behind the raised desktop")
                    }
                }
            }
        } else if !foreground.0.is_null()
            && !self.is_desktop_window(foreground)
            && !self.is_own_window(foreground)
            && !is_shell_popup(desktop::class_name(desktop::root_ancestor(foreground)).as_str())
        {
            self.end_show_desktop("another window came to the front");
        }
    }

    /// Leaves the topmost band; the caller re-anchors above the host.
    fn end_show_desktop(&mut self, reason: &'static str) {
        self.show_desktop_active = false;
        self.topmost_refused = false;
        tracing::info!(reason, "show desktop ended");
        if self.behavior == ShowDesktopBehavior::KeepVisible {
            self.with_anchoring(|s| {
                for (f, shadow) in s.fences.clone() {
                    for h in std::iter::once(f).chain(shadow) {
                        let _ = desktop::set_topmost(h, false);
                    }
                }
            });
        }
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
            } else if desktop::has_visible_surface(cursor) && !is_drag_overlay(cursor) {
                return false;
            }
        }
        false
    }

    pub fn on_foreground(&mut self, hwnd: HWND) {
        if let Some(callback) = self.on_foreground_changed.as_ref() {
            callback(hwnd);
        }
        let root = desktop::root_ancestor(hwnd);
        let cls = desktop::class_name(root);
        // The window the user goes to from a shell popup is the real change.
        if is_shell_popup(&cls) {
            tracing::debug!(%cls, "shell popup in front; foreground change ignored");
            return;
        }
        if self.peeking {
            // Peek is a glance: the first foreign window that comes to the front ends it. Our
            // own windows (fences, dimmer, rename popup, menus) do not.
            let ours = cls == FENCE_CLASS
                || cls == SENTINEL_CLASS
                || cls.starts_with("PecoFence.")
                || cls == "#32768";
            if !ours && !root.0.is_null() {
                tracing::info!(class = %cls, "foreground changed during peek; ending it");
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
            if let Some(callback) = self.on_desktop_front.as_ref() {
                callback();
            }
            self.set_raw_input_sink(true);
            let pt = window::cursor_pos();
            self.quick.pending_click = Some((Instant::now(), pt));
            // The press that activated the desktop reached Explorer before the sink existed: a
            // drag it starts still draws a marquee (from where the pointer is now). Activated
            // any other way, the desktop ends the fence selection like a click on it does.
            if window::primary_button_down_async() {
                self.begin_marquee(pt);
            } else {
                self.emit_marquee(MarqueeEvent::Pressed {
                    additive: additive_modifier(),
                });
            }
        } else {
            self.set_raw_input_sink(false);
            self.quick.pending_click = None;
            if self.quick.marquee_origin.take().is_some()
                && std::mem::take(&mut self.quick.marquee_live)
            {
                self.emit_marquee(MarqueeEvent::Cancelled);
            }
        }

        // Explorer may reorder the host slightly after the foreground event (Win+D activates
        // the desktop before raising it), so verify now and again on a short retry ladder.
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
                    // One that quick hide is still fading out is visible until the fade ends.
                    let quick_hidden =
                        s.quick.fences_hidden && !s.quick_hide_excluded.contains(&(f.0 as isize));
                    if !desktop::is_visible(f) || quick_hidden {
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
            self.topmost_refused = false;
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
            // Show desktop may have ended during the peek, which suspends its checks; left
            // active, the fences would float above the window that came to the front.
            self.check_show_desktop();
            self.reanchor("peek off");
            tracing::info!("peek off");
        }
    }

    /// Re-anchors only if the host changed or our block is no longer directly above it.
    fn ensure_anchored(&mut self, reason: &'static str) {
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
        if resolved != self.host && self.show_desktop_active {
            // Explorer restarted: the raised desktop went with it.
            self.end_show_desktop("icon host changed");
        }
        self.check_show_desktop();
        if self.show_desktop_active {
            if self.behavior == ShowDesktopBehavior::KeepVisible
                && !self.fences.iter().all(|(f, _)| desktop::is_topmost(*f))
            {
                self.place_topmost_band();
            }
            return;
        }
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
    /// (double-click) and the marquee (fence selection / new fence); moves in between draw it.
    fn on_raw_input(&mut self, lparam: isize) {
        let Some(buttons) = rawinput::mouse_buttons_from_wm_input(lparam) else {
            return;
        };
        if !buttons.left_down && !buttons.left_up {
            self.track_marquee();
            return;
        }
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
        if !self.begin_marquee(pt) {
            self.quick.pending_click = None;
            return;
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

    /// The left button is down at `pt`. On the bare desktop (not over one of our windows or one
    /// of Windows' own icons) a marquee may start there while the fences show; returns false
    /// anywhere else.
    fn begin_marquee(&mut self, pt: POINT) -> bool {
        let hit = desktop::window_from_point(pt.x, pt.y);
        let root = desktop::root_ancestor(hit);
        if self.all_windows().contains(&root) || !self.is_desktop_window(hit) || on_icon(hit, pt) {
            self.quick.marquee_origin = None;
            self.quick.marquee_live = false;
            return false;
        }
        if !self.quick.fences_hidden {
            self.quick.marquee_origin = Some(pt);
            self.quick.marquee_live = false;
            self.quick.marquee_last = None;
            self.emit_marquee(MarqueeEvent::Pressed {
                additive: additive_modifier(),
            });
        }
        true
    }

    /// The pointer moved with the button down since a press on the bare desktop: past the drag
    /// threshold the marquee is live and reported on every move.
    fn track_marquee(&mut self) {
        let Some(origin) = self.quick.marquee_origin else {
            return;
        };
        if !window::primary_button_down_async() {
            // The release went unseen: finish the marquee here.
            self.on_desktop_button_up();
            return;
        }
        let pt = window::cursor_pos();
        if !self.quick.marquee_live {
            let (sx, sy) = window::drag_threshold();
            if (pt.x - origin.x).abs() <= sx && (pt.y - origin.y).abs() <= sy {
                return;
            }
            self.quick.marquee_live = true;
        }
        if self.quick.marquee_last == Some(pt) {
            return;
        }
        self.quick.marquee_last = Some(pt);
        self.emit_marquee(MarqueeEvent::Moved(marquee_rect(origin, pt)));
    }

    /// Left button released after a drag that started on the bare desktop: the fences the
    /// marquee touches are selected, or a large enough one that touches none offers a new
    /// fence there (the App decides).
    fn on_desktop_button_up(&mut self) {
        let Some(origin) = self.quick.marquee_origin.take() else {
            return;
        };
        if !std::mem::take(&mut self.quick.marquee_live) {
            return;
        }
        let rect = marquee_rect(origin, window::cursor_pos());
        tracing::info!(?rect, "desktop marquee");
        if rect.right - rect.left >= MARQUEE_MIN_PX && rect.bottom - rect.top >= MARQUEE_MIN_PX {
            // A marquee that began with the second click of a double-click is not a quick hide.
            self.quick.pending_toggle = None;
            self.sentinel.kill_timer(TIMER_QUICKHIDE);
        }
        self.emit_marquee(MarqueeEvent::Released(rect));
    }

    fn emit_marquee(&self, event: MarqueeEvent) {
        if let Some(cb) = self.on_marquee.as_ref() {
            cb(event);
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
            // Shown during a peek: ending it must not hide them again.
            self.peek_shown.clear();
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
        let kept = self.icons.kept_marker();
        if shell_icons::desktop_icons_hidden() {
            if self.icons.marker.exists() || kept.exists() {
                // Our previous run died without restoring them, or left them hidden on purpose
                // (`release_desktop_icons`). Taking them back lets exit restore or keep them as
                // the setting says, and a fresh watchdog covers a crash.
                tracing::info!("desktop icons still hidden by our previous run; adopting");
                if let Some(dir) = self.icons.marker.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                let _ = std::fs::write(&self.icons.marker, b"pecofence hid desktop icons\n");
                let _ = std::fs::remove_file(&kept);
                self.icons.we_hid_them = true;
                self.spawn_watchdog();
            } else {
                tracing::info!("desktop icons already hidden by user/other tool; leaving as is");
            }
            return true;
        }
        // Shown by the user since: nothing of ours to keep.
        let _ = std::fs::remove_file(&kept);
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

    /// True after the retry budget was exhausted without success (caller should stop the
    /// retries and tell the user).
    pub fn hide_desktop_icons_gave_up(&self) -> bool {
        self.icons.pending && self.icons.pending_attempts >= HIDE_ICONS_MAX_ATTEMPTS
    }

    pub fn cancel_pending_hide(&mut self) {
        self.icons.pending = false;
        self.icons.pending_attempts = 0;
    }

    /// Starts `pecofence-watchdog.exe` (next to our exe) so a crash still restores the icons.
    fn spawn_watchdog(&mut self) {
        // One per run: it checks the marker when we exit, whatever happened before.
        if !self.icons.watchdogs.is_empty() {
            return;
        }
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
            Ok(wpid) => {
                tracing::info!(watchdog_pid = wpid, "watchdog started");
                self.icons.watchdogs.push(wpid);
            }
            Err(e) => tracing::warn!(error = %e, "watchdog spawn failed"),
        }
    }

    /// The watchdog processes setup has to wait for before it replaces their executable.
    pub fn watchdog_pids(&self) -> &[u32] {
        &self.icons.watchdogs
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

    /// At exit and at the end of the session: restores the icons, or with `keep_icons_hidden`
    /// leaves them hidden for the next sign-in. The kept marker says they are still ours, so
    /// the next start adopts them and the uninstaller shows them; the watchdog leaves them.
    pub fn release_desktop_icons(&mut self) {
        if self.keep_icons_hidden && self.icons.we_hid_them {
            self.cancel_pending_hide();
            let _ = std::fs::write(
                self.icons.kept_marker(),
                b"pecofence keeps desktop icons hidden\n",
            );
            self.icons.we_hid_them = false;
            let _ = std::fs::remove_file(&self.icons.marker);
            tracing::info!("desktop icons left hidden for the next sign-in");
            return;
        }
        self.restore_desktop_icons();
    }

    /// The previous run left the icons hidden (`release_desktop_icons`), but this one hides none
    /// (「隐藏 Windows 桌面图标」 off or `--no-hide-icons`): shows them again. The kept marker
    /// stays while Explorer does not answer, so the next start tries again.
    pub fn show_kept_desktop_icons(&mut self) {
        let kept = self.icons.kept_marker();
        if !kept.exists() {
            return;
        }
        if shell_icons::desktop_icons_hidden() {
            let shown = self
                .host
                .or_else(|| desktop::resolve_icon_host(self.generation))
                .and_then(|h| shell_icons::set_desktop_icons_hidden(false, h.def_view, h.host));
            if shown != Some(false) {
                tracing::warn!(?shown, "showing the kept desktop icons failed");
                return;
            }
            tracing::info!("desktop icons kept hidden by our previous run shown again");
        }
        let _ = std::fs::remove_file(&kept);
    }

    /// True when a previous run left the marker behind (crashed without restoring).
    pub fn stale_marker_present(&self) -> bool {
        self.icons.marker.exists()
    }

    /// The previous run's marker with the icons showing anyway: it restored them but was ended
    /// before it removed the marker. Kept, it would take a later hide by the user for ours.
    pub fn forget_stale_marker(&mut self) {
        if !self.icons.we_hid_them {
            let _ = std::fs::remove_file(&self.icons.marker);
        }
    }

    fn on_timer(&mut self, id: usize) {
        match id {
            TIMER_ANCHOR_CHECK => self.ensure_anchored("periodic check"),
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

    /// The fences' windows right now, for a feedback report.
    pub fn window_report(&self) -> WindowReport {
        let order = desktop::top_level_windows();
        let host_at = self
            .host
            .and_then(|h| order.iter().position(|&w| w == h.host));
        let below_host = |w: HWND| {
            host_at.is_some_and(|i| order.iter().position(|&x| x == w).is_some_and(|p| p > i))
        };
        let screens: Vec<RECT> = pecofence_platform::monitors::enumerate()
            .iter()
            .map(|m| m.bounds)
            .collect();
        let off_screen = |w: HWND| {
            let r = window::window_rect(w);
            !screens.iter().any(|s| {
                r.left < s.right && r.right > s.left && r.top < s.bottom && r.bottom > s.top
            })
        };
        let count =
            |test: &dyn Fn(HWND) -> bool| self.fences.iter().filter(|(f, _)| test(*f)).count();
        WindowReport {
            fences: self.fences.len(),
            visible: count(&desktop::is_visible),
            cloaked: count(&desktop::is_cloaked),
            topmost: count(&desktop::is_topmost),
            below_host: count(&below_host),
            off_screen: count(&off_screen),
            quick_hidden: self.quick.fences_hidden,
            peeking: self.peeking,
            show_desktop: self.show_desktop_active,
            host: self.host.map(|h| h.kind),
        }
    }
}

/// Registers the sentinel window class.
pub fn register_sentinel_class() -> Result<WindowClass> {
    WindowClass::register(SENTINEL_CLASS, ClassOptions::default())
}

impl Drop for DesktopAnchor {
    fn drop(&mut self) {
        // A clean exit released the icons already; a panic unwinding to here shows them.
        self.restore_desktop_icons();
        self.set_raw_input_sink(false);
    }
}
