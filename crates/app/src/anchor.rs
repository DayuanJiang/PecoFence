//! Desktop z-order anchoring and "Show desktop" survival (plan §5.1, §5.2).

use crate::commands::WM_APP_SET_VISIBLE;
use pecofence_platform::desktop::{self, Generation, IconHost};
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::winevent::{self, WinEventHook};
use pecofence_platform::{HWND, POINT, RECT, Result, msg, rawinput, shell_icons};
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

pub const FENCE_CLASS: &str = "PecoFence.Fence";
pub const SENTINEL_CLASS: &str = "PecoFence.Sentinel";

const TIMER_ANCHOR_CHECK: usize = 10;
const TIMER_RETRY: usize = 11;
const TIMER_QUICKHIDE: usize = 13;
/// A foreground event came while the anchor was busy (`on_foreground` after all, see the hook).
const TIMER_FOREGROUND_LATER: usize = 14;
/// Windows does not always report the desktop coming to the front: on Windows 10 a click on it
/// after closing a window, on every version a click on it while nothing or the taskbar was in
/// front (sign-in, the end of a process whose window was in front). A light poll catches it.
const TIMER_FOREGROUND_POLL: usize = 15;
const FOREGROUND_POLL_MS: u32 = 250;
/// Win+Ctrl+←/→ puts Windows' switcher window in front, and when it goes the desktop gets the
/// foreground back without an event: the double-click (quick hide) went unseen after every
/// virtual-desktop switch. A few looks at the foreground catch it.
const TIMER_FOREGROUND_AFTER_SWITCH: usize = 18;
const AFTER_SWITCH_LOOK_MS: u32 = 300;
const AFTER_SWITCH_LOOKS: u8 = 6;
const SWITCHER_CLASS: &str = "VirtualDesktopHotkeySwitcher";
/// Top-level window class of Start, search and the notification centre (an app's core window is
/// a child of its frame window).
const SHELL_CORE_WINDOW: &str = "Windows.UI.Core.CoreWindow";
/// Windows 10: the desktop's clicks followed after a switch (`follow_desktop_after_switch`)
/// stop being followed this long after it unless the desktop came to the front meanwhile: the
/// sink gets every mouse move while it is on.
const TIMER_SWITCH_FOLLOW_END: usize = 19;
const SWITCH_FOLLOW_MS: u32 = 10_000;
/// Explorer reordered windows while the anchor was busy (`on_shell_reorder` after all).
const TIMER_REORDER_LATER: usize = 16;
/// Explorer's z-order changes are checked while the desktop is in front and this long after a
/// foreground change: show desktop raises the desktop over the fences and lowers it again then.
const SHELL_REORDER_WATCH_MS: u128 = 1000;
const QUICKHIDE_DELAY_MS: u32 = 300;

const RETRY_DELAYS_MS: [u32; 6] = [50, 150, 350, 750, 1500, 1200];
/// Retries of a failed "hide desktop icons" request (logon: Explorer's desktop not ready yet),
/// one per TIMER_ANCHOR_CHECK tick (~30 s at the 2 s cadence) before giving up.
const HIDE_ICONS_MAX_ATTEMPTS: u32 = 15;
/// How long startup and the retries wait for Explorer to hide or show the icons. It needs
/// about 0.3 s for 1400 icons when idle, 20 s right after sign-in: the toggle goes on without
/// us, and the next check finds the icons hidden (`hide_desktop_icons_within`).
const ICONS_QUICK_WAIT: Duration = Duration::from_millis(100);
/// The same for a click in Settings or the tray, which waits for its answer.
const ICONS_ASKED_WAIT: Duration = Duration::from_millis(1500);
/// Showing them again (exit, the setting turned off, 「临时显示 Windows 桌面」): a toggle still out
/// when the process ends is lost, and the icons stay hidden until the watchdog shows them.
const ICONS_RESTORE_WAIT: Duration = Duration::from_secs(4);

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

/// The taskbar (it keeps the foreground after a Windows 10 virtual-desktop switch).
fn is_taskbar(class: &str) -> bool {
    class == "Shell_TrayWnd" || class == "Shell_SecondaryTrayWnd"
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

/// Whether `outer` holds all of `inner` (an empty `inner` is held by nothing).
fn rect_holds(outer: &RECT, inner: &RECT) -> bool {
    inner.right > inner.left
        && inner.bottom > inner.top
        && outer.left <= inner.left
        && outer.top <= inner.top
        && outer.right >= inner.right
        && outer.bottom >= inner.bottom
}

/// 像桌面一样排列: the fence window arranged like the desktop (0 = none). It stays under the
/// other fences, and their snapping, overlap and push rules leave it out.
static DESKTOP_FENCE: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

/// Marks or unmarks `hwnd` as the fence arranged like the desktop.
pub fn set_desktop_fence(hwnd: HWND, on: bool) {
    use std::sync::atomic::Ordering;
    let id = hwnd.0 as isize;
    if on {
        DESKTOP_FENCE.store(id, Ordering::Relaxed);
    } else {
        let _ = DESKTOP_FENCE.compare_exchange(id, 0, Ordering::Relaxed, Ordering::Relaxed);
    }
}

pub fn is_desktop_fence(hwnd: HWND) -> bool {
    let id = DESKTOP_FENCE.load(std::sync::atomic::Ordering::Relaxed);
    id != 0 && hwnd.0 as isize == id
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
    /// What raw input reported while the desktop was in front (logged when it leaves).
    seen: SinkCounts,
}

/// Raw input while the desktop is in front, counted for the log line written when it leaves
/// the front: no `inputs` at all means Windows delivered none (it keeps the input of an
/// Explorer that runs elevated from us), and presses that were not `taken` say why.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SinkCounts {
    /// `WM_INPUT` messages, moves included.
    inputs: u64,
    /// Messages whose mouse data could not be read.
    unreadable: u32,
    /// Presses of the primary button.
    presses: u32,
    /// Presses on the bare desktop (a marquee or a double-click may follow), with the one that
    /// brought the desktop to the front.
    taken: u32,
    /// Presses refused because they were on one of our windows...
    on_ours: u32,
    /// ...on a window that is not the desktop...
    not_desktop: u32,
    /// ...or on one of Windows' own desktop icons.
    on_icon: u32,
}

/// Why a press was not taken as one on the bare desktop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PressRefused {
    OnOurs,
    NotDesktop,
    OnIcon,
}

impl QuickHide {
    /// The desktop came to the front with the pointer at `pt`, the primary button `held` down
    /// or not. A press on the desktop that brought it there while the raw input sink was off
    /// reached Explorer unseen, so it counts as the first click here. Nothing else does: with
    /// the sink on, raw input reports that press itself (just before or just after this event),
    /// and a click on an item of Explorer's menu (刷新) brings the desktop back on its release.
    fn desktop_activated(&mut self, sink_was_on: bool, held: bool, now: Instant, pt: POINT) {
        if !sink_was_on && held {
            self.pending_click = Some((now, pt));
        }
    }

    /// A press on the bare desktop at `pt`. The second click of a double-click (within
    /// `limit_ms` and `slop` pixels of the first one) returns the time since the first click and
    /// where it was; any other press becomes the first click.
    fn desktop_press(
        &mut self,
        now: Instant,
        pt: POINT,
        limit_ms: u128,
        slop: (i32, i32),
    ) -> Option<(u128, POINT)> {
        if let Some((t0, p0)) = self.pending_click.take() {
            let interval_ms = now.duration_since(t0).as_millis();
            if interval_ms <= limit_ms
                && (pt.x - p0.x).abs() <= slop.0.max(4)
                && (pt.y - p0.y).abs() <= slop.1.max(4)
            {
                return Some((interval_ms, p0));
            }
        }
        self.pending_click = Some((now, pt));
        None
    }
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
    /// The double-click that hides and shows the fences came during 「临时显示 Windows 桌面」:
    /// the App, which owns that mode, ends it (the anchor showing the fences by itself left
    /// them over Windows' own icons, with the fences kept on quick hide still away).
    DoubleClickInWindowsDesktop,
}

fn marquee_rect(a: POINT, b: POINT) -> RECT {
    RECT {
        left: a.x.min(b.x),
        top: a.y.min(b.y),
        right: a.x.max(b.x),
        bottom: a.y.max(b.y),
    }
}

/// A press on one of Windows' own desktop icons (they show while 「Windows 桌面图标」 is not 隐藏)
/// selects, opens or drags that icon: it starts no marquee and counts for no quick hide.
fn on_icon(hit: HWND) -> bool {
    shell_icons::desktop_icon_hovered(hit)
}

/// Ctrl or Shift is down (physical state; the desktop, not us, has the keyboard focus).
fn additive_modifier() -> bool {
    window::key_down_async(msg::VK_CONTROL) || window::key_down_async(msg::VK_SHIFT)
}

/// Real desktop icon visibility management (plan §5.3).
struct DesktopIcons {
    we_hid_them: bool,
    marker: PathBuf,
    /// Exit left the icons hidden on purpose (`keep_icons_hidden`). Always in our own folder,
    /// where the next start and the uninstaller look, also when `marker` is the legacy one.
    kept: PathBuf,
    /// `release_desktop_icons` left the icons hidden; ensure_anchored takes them back if this
    /// process goes on after all (a cancelled sign-out).
    left_hidden: bool,
    /// A hide was requested (setting on) but has not succeeded yet; retried from ensure_anchored.
    pending: bool,
    pending_attempts: u32,
    /// The watchdog started this run (one at most); it waits for this process to exit.
    watchdogs: Vec<u32>,
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
        Self::in_dir(&base, &marker)
    }

    /// The marker named `name` in `base`, or in the legacy folder next to it if only that one
    /// has it.
    fn in_dir(base: &Path, name: &str) -> Self {
        let current = base.join(name);
        let legacy = base
            .with_file_name(pecofence_core::brand::LEGACY_DATA_DIR)
            .join(name);
        let kept = current.with_extension("kept");
        let marker = if !current.exists() && legacy.exists() {
            legacy
        } else {
            current
        };
        Self {
            we_hid_them: false,
            marker,
            kept,
            left_hidden: false,
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
    /// Fences a hover peek brought to the top (`raise_for_hover`), with the place in `fences` they
    /// go back to when the peek ends.
    hover_raised: Vec<(HWND, usize)>,
    show_desktop_active: bool,
    /// When show desktop began (how long it lasted goes into the log when it ends).
    show_desktop_since: Option<Instant>,
    /// During show desktop: the program's window that shows over the raised desktop where a
    /// fence is (`program_window_over_raised_desktop`). The fences lie under it, above the
    /// desktop, and float again when it is gone.
    show_desktop_under: Option<isize>,
    retry_step: usize,
    _hook: Option<WinEventHook>,
    quick: QuickHide,
    icons: DesktopIcons,
    /// `Settings::keep_icons_hidden`: exit leaves the icons hidden (`release_desktop_icons`).
    pub keep_icons_hidden: bool,
    pub quick_hide_enabled: bool,
    /// 「临时显示 Windows 桌面」 is on (the App sets it with its own flag): every fence is away
    /// and Windows' icons show.
    pub windows_desktop: bool,
    /// Presses, drags and releases on the empty desktop (fence selection / new fence).
    pub on_marquee: Option<Box<dyn Fn(MarqueeEvent)>>,
    /// Fence windows (as raw handles) that stay visible during quick hide.
    quick_hide_excluded: std::collections::HashSet<isize>,
    /// Fence windows of other virtual desktops than the current one: hidden, and not shown
    /// again by the anchor's own checks.
    off_desktop: std::collections::HashSet<isize>,
    /// Peek in progress: fences are topmost and anchoring is suspended.
    peeking: bool,
    /// Fences (hidden by quick hide) that Peek showed and must hide again afterwards.
    peek_shown: Vec<HWND>,
    /// Fences the Peek raises a moment later (`hold_for_peek`): another program's window hides
    /// them, and they must be transparent on screen before they come over it.
    peek_held: Vec<HWND>,
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
    /// The window (of an elevated program) the fences go above because they may not go under
    /// it; logged when it starts and when it ends.
    placed_above: Option<isize>,
    /// The last window found covering the fences (`note_covered`), logged once.
    covered_by: Option<isize>,
    /// The window the last `on_foreground` was about (the poll compares with it).
    last_foreground: isize,
    /// Looks at the foreground left after a virtual-desktop switch (`TIMER_FOREGROUND_AFTER_SWITCH`).
    switch_looks: u8,
    /// Windows 10 (the legacy desktop): the virtual desktop the foreground poll saw last (see
    /// `follow_desktop_after_switch`).
    poll_desktop: Option<[u8; 16]>,
    /// That window was the desktop (its comings and goings are logged, and the poll catches the
    /// ones Windows does not report).
    desktop_front: bool,
    /// A fence opened a shell dialog (Properties, a delete confirmation): see
    /// `dialog_opened_from`.
    dialog_from: Option<DialogFrom>,
    /// When the last `on_foreground` ran (`on_shell_reorder` watches Explorer's z-order changes
    /// for a while after it).
    last_foreground_at: Instant,
    /// The hook on Explorer's z-order changes and the process it is for (a new one after an
    /// Explorer restart).
    reorder_hook: Option<(u32, WinEventHook)>,
    /// This anchor's own cell, for the callback of a reorder hook installed later.
    cell: std::rc::Weak<RefCell<Option<DesktopAnchor>>>,
}

/// The fence a shell dialog of this process was opened from, and the dialog once the poll saw
/// it in front. When it closes, Windows activates the desktop (or the window below), since a
/// fence cannot be activated by the system: the fence takes the foreground back instead, and
/// its selection stays.
#[derive(Clone, Copy, Debug)]
struct DialogFrom {
    fence: HWND,
    dialog: Option<HWND>,
}

/// The fences' windows as a feedback report describes them.
pub struct WindowReport {
    pub fences: usize,
    pub visible: usize,
    pub cloaked: usize,
    pub topmost: usize,
    /// Below the icon host in the z-order: covered by the desktop.
    pub below_host: usize,
    /// Above a window that is not ours or the desktop's (an app, Settings, a dialog): the fence
    /// covers it.
    pub above_windows: usize,
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
            let sentinel_hwnd = sentinel.hwnd().0 as isize;
            WinEventHook::install(
                winevent::SYSTEM_FOREGROUND,
                winevent::SYSTEM_FOREGROUND,
                Box::new(move |_event, hwnd| {
                    if let Ok(mut guard) = cell.try_borrow_mut()
                        && let Some(anchor) = guard.as_mut()
                    {
                        anchor.on_foreground(hwnd);
                    } else {
                        // The hook can run inside a call the anchor makes (a SendMessage pumps
                        // it): dropped, the desktop coming to the front went unseen and its
                        // double-click did nothing (Windows 10). The timer handles the change
                        // once the anchor is free; it repeats until then.
                        tracing::debug!(?hwnd, "foreground event while the anchor is busy");
                        window::set_timer(
                            HWND(sentinel_hwnd as *mut core::ffi::c_void),
                            TIMER_FOREGROUND_LATER,
                            30,
                        );
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
            hover_raised: Vec::new(),
            show_desktop_active: false,
            show_desktop_since: None,
            show_desktop_under: None,
            retry_step: 0,
            _hook: hook,
            quick: QuickHide::default(),
            icons: DesktopIcons::new(),
            keep_icons_hidden: false,
            quick_hide_enabled,
            windows_desktop: false,
            on_marquee: None,
            quick_hide_excluded: std::collections::HashSet::new(),
            off_desktop: std::collections::HashSet::new(),
            peeking: false,
            peek_shown: Vec::new(),
            peek_held: Vec::new(),
            on_peek_interrupted: None,
            on_foreground_changed: None,
            on_desktop_front: None,
            anchor_refused: None,
            topmost_refused: false,
            placed_above: None,
            covered_by: None,
            last_foreground: 0,
            switch_looks: 0,
            poll_desktop: pecofence_platform::wallpaper::desktop_id(),
            desktop_front: false,
            dialog_from: None,
            last_foreground_at: Instant::now(),
            reorder_hook: None,
            cell: Rc::downgrade(&cell),
        };
        // The hook reports only changes of the foreground. With the desktop, the taskbar or no
        // window in front (a start at sign-in, a restart after the process whose fence was in
        // front ended) the desktop's clicks are followed from now on: a click there brings the
        // desktop to the front without any foreground event.
        let front = desktop::foreground_window();
        anchor.last_foreground = front.0 as isize;
        anchor.desktop_front = anchor.is_desktop_window(front);
        let cls = desktop::class_name(desktop::root_ancestor(front));
        if anchor.follows_desktop_clicks(front) {
            tracing::info!(
                class = %cls,
                "following the desktop's clicks from the start"
            );
            anchor.set_raw_input_sink(true);
        } else {
            tracing::info!(
                class = %cls,
                "another window in front at start; the desktop's clicks are followed once it is in front"
            );
        }
        anchor
            .sentinel
            .set_coalescable_timer(TIMER_ANCHOR_CHECK, 2000, 1000);
        anchor
            .sentinel
            .set_coalescable_timer(TIMER_FOREGROUND_POLL, FOREGROUND_POLL_MS, 50);
        anchor.sync_reorder_hook();
        *cell.borrow_mut() = Some(anchor);
        Ok(cell)
    }

    /// Show desktop (Win+D) raises the desktop over the fences without telling anyone, a few
    /// milliseconds after the foreground event (or with none at all when the desktop was in
    /// front already): the anchor checks found it 50 ms to 2 s later, and the fences were gone
    /// meanwhile. A hook on the z-order changes Explorer makes reports the raise itself. Installed
    /// for the Explorer that runs now; `ensure_anchored` installs it again after a restart.
    fn sync_reorder_hook(&mut self) {
        let pid = desktop::window_pid(desktop::shell_window());
        if pid == 0 || self.reorder_hook.as_ref().is_some_and(|(p, _)| *p == pid) {
            return;
        }
        let cell = self.cell.clone();
        let sentinel = self.sentinel.hwnd().0 as isize;
        let hook = WinEventHook::install_for(
            winevent::OBJECT_REORDER,
            winevent::OBJECT_REORDER,
            pid,
            Box::new(move |_event, _hwnd| {
                let Some(cell) = cell.upgrade() else { return };
                if let Ok(mut guard) = cell.try_borrow_mut()
                    && let Some(anchor) = guard.as_mut()
                {
                    anchor.on_shell_reorder();
                } else {
                    window::set_timer(
                        HWND(sentinel as *mut core::ffi::c_void),
                        TIMER_REORDER_LATER,
                        10,
                    );
                }
            }),
        );
        match hook {
            Ok(h) => {
                tracing::info!(pid, "following Explorer's z-order changes");
                self.reorder_hook = Some((pid, h));
            }
            Err(e) => {
                tracing::warn!(error = %e, pid, "hook on Explorer's z-order changes failed");
                self.reorder_hook = None;
            }
        }
    }

    /// Explorer changed the z-order. While the desktop is in front and right after a foreground
    /// change, that is show desktop raising the desktop over the fences or lowering it again:
    /// checked at once, the fences are back above the desktop before the next frame.
    fn on_shell_reorder(&mut self) {
        if self.peeking {
            return;
        }
        let watching = self.show_desktop_active
            || self.last_foreground_at.elapsed().as_millis() < SHELL_REORDER_WATCH_MS
            || self.is_desktop_window(desktop::foreground_window());
        if watching {
            self.ensure_anchored("desktop reordered");
        }
    }

    pub fn register_fence(&mut self, hwnd: HWND, shadow: Option<HWND>) {
        if !self.fences.iter().any(|(h, _)| *h == hwnd) {
            self.fences.push((hwnd, shadow));
        }
        self.keep_desktop_fence_lowest();
    }

    /// 像桌面一样排列: that fence lies under every other one, like the desktop's own icons.
    /// Returns whether the order changed.
    fn keep_desktop_fence_lowest(&mut self) -> bool {
        match self.fences.iter().position(|(h, _)| is_desktop_fence(*h)) {
            Some(pos) if pos > 0 => {
                let entry = self.fences.remove(pos);
                self.fences.insert(0, entry);
                true
            }
            _ => false,
        }
    }

    /// Re-anchors when the desktop-arranged fence is not the lowest yet.
    pub fn lower_desktop_fence(&mut self) {
        if self.keep_desktop_fence_lowest() {
            self.reanchor("desktop arranged fence");
        }
    }

    pub fn unregister_fence(&mut self, hwnd: HWND) {
        self.fences.retain(|(h, _)| *h != hwnd);
        self.off_desktop.remove(&(hwnd.0 as isize));
        self.hover_raised.retain(|(h, _)| *h != hwnd);
    }

    /// Windows 10, a virtual-desktop switch seen by the foreground poll: the desktop's clicks are
    /// followed now (the raw-input sink, which takes only presses on the bare desktop), so the
    /// click that brings the desktop to the front counts for a double-click, until another window
    /// comes to the front (`on_foreground`).
    fn follow_desktop_after_switch(&mut self) {
        let front = desktop::foreground_window();
        let root = desktop::root_ancestor(front);
        let cls = desktop::class_name(root);
        self.last_foreground = front.0 as isize;
        // An ordinary program in front (Windows gave it the new desktop's foreground): its
        // clicks are the program's, and a click on the desktop brings the desktop to the front
        // with an event, as without a switch.
        let program = !front.0.is_null()
            && !self.is_desktop_window(front)
            && !is_taskbar(&cls)
            && desktop::window_pid(root) != std::process::id();
        tracing::info!(
            class = %cls,
            following = self.quick.sink_registered,
            program,
            "virtual desktop switch seen by the foreground poll"
        );
        if program || self.quick.sink_registered {
            return;
        }
        self.set_raw_input_sink(true);
        self.sentinel
            .set_timer(TIMER_SWITCH_FOLLOW_END, SWITCH_FOLLOW_MS);
    }

    /// Marks a fence window as belonging to other virtual desktops (`FenceWindow` hides it).
    pub fn set_off_desktop(&mut self, hwnd: HWND, off: bool) {
        if off {
            self.off_desktop.insert(hwnd.0 as isize);
        } else {
            self.off_desktop.remove(&(hwnd.0 as isize));
        }
    }

    /// Whether quick hide (or 「临时显示 Windows 桌面」, `every`) keeps this fence hidden now.
    pub fn quick_hides(&self, hwnd: HWND, every: bool) -> bool {
        self.quick.fences_hidden
            && (every || !self.quick_hide_excluded.contains(&(hwnd.0 as isize)))
    }

    /// Puts `hwnd` above the other fences (clicked / newly created), keeping all of them just
    /// above the icon host. It stays there: a hover peek that raised it no longer puts it back.
    pub fn raise_fence(&mut self, hwnd: HWND) {
        self.hover_raised.retain(|(h, _)| *h != hwnd);
        self.move_to_top(hwnd, "raise fence");
    }

    /// A hover peek (or a file drag resting on the title) opened rolled fence `hwnd`: it goes
    /// above the other fences while it is open, so a neighbour the push could not clear (it
    /// stops at the bottom of the screen) does not cover it (#197). `end_hover_raise` puts it back.
    pub fn raise_for_hover(&mut self, hwnd: HWND) {
        let Some(pos) = self.fences.iter().position(|(h, _)| *h == hwnd) else {
            return;
        };
        if !self.hover_raised.iter().any(|(h, _)| *h == hwnd) {
            self.hover_raised.push((hwnd, pos));
        }
        tracing::info!(
            from = pos,
            of = self.fences.len(),
            "fence raised for a hover peek"
        );
        self.move_to_top(hwnd, "hover peek raise");
    }

    /// The peek that raised `hwnd` rolled it up again: back to its old place among the fences
    /// (unless a click raised it for good meanwhile).
    pub fn end_hover_raise(&mut self, hwnd: HWND) {
        let Some(i) = self.hover_raised.iter().position(|(h, _)| *h == hwnd) else {
            return;
        };
        let (_, at) = self.hover_raised.remove(i);
        let Some(pos) = self.fences.iter().position(|(h, _)| *h == hwnd) else {
            return;
        };
        let entry = self.fences.remove(pos);
        let at = at.min(self.fences.len());
        self.fences.insert(at, entry);
        tracing::info!(from = pos, to = at, "fence put back after a hover peek");
        if at != pos {
            self.keep_desktop_fence_lowest();
            self.reanchor("hover peek end");
        }
    }

    fn move_to_top(&mut self, hwnd: HWND, reason: &'static str) {
        let Some(pos) = self.fences.iter().position(|(h, _)| *h == hwnd) else {
            return;
        };
        if pos + 1 == self.fences.len() {
            return;
        }
        let entry = self.fences.remove(pos);
        self.fences.push(entry);
        self.keep_desktop_fence_lowest();
        self.reanchor(reason);
    }

    /// The fence `hwnd` is opening a shell dialog of this process (Properties, a delete
    /// confirmation): when the dialog closes and Windows activates the desktop or another
    /// window, the fence gets the foreground back and keeps its selection (`on_foreground`).
    /// Nothing happens unless the poll sees the dialog in front first.
    pub fn dialog_opened_from(&mut self, hwnd: HWND) {
        self.dialog_from = Some(DialogFrom {
            fence: hwnd,
            dialog: None,
        });
    }

    /// The foreground went to `hwnd` without a press of the button that is still held: a dialog
    /// of ours opened from a fence (and seen in front) closed, when it is gone now. Takes the
    /// pending dialog either way; returns the fence to give the foreground back to.
    fn fence_after_dialog(&mut self, held: bool) -> Option<HWND> {
        let d = self.dialog_from.take()?;
        let dialog = d.dialog?;
        let gone = !desktop::is_visible(dialog);
        let back = gone && !held && self.fences.iter().any(|(f, _)| *f == d.fence);
        tracing::info!(
            gone,
            held,
            back,
            "foreground changed after a dialog opened from a fence"
        );
        back.then_some(d.fence)
    }

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
    /// or while the raised desktop would cover the fences (no normal window can go above it),
    /// so windows created / detached / raised meanwhile end up above the dimmer or the desktop
    /// instead of behind it.
    fn place_topmost_band(&mut self) {
        let mut refused = None;
        self.with_anchoring(|s| {
            for (f, shadow) in s.fences.clone() {
                if s.peek_held.contains(&f) {
                    continue;
                }
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
        if self.show_desktop_active
            && self.behavior == ShowDesktopBehavior::KeepVisible
            && self.show_desktop_under.is_none()
        {
            tracing::debug!(
                reason,
                "reanchor during show desktop: reordering in topmost band"
            );
            self.place_topmost_band();
            return;
        }
        let resolved = desktop::resolve_icon_host(self.generation);
        if resolved != self.host {
            tracing::info!(
                reason,
                old = ?self.host,
                new = ?resolved,
                layers = %desktop::describe_desktop_windows(),
                "icon host changed"
            );
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
        let mut placed_above = None;
        self.with_anchoring(|s| {
            let mode = s.mode;
            let place = |w: HWND| -> Result<Option<HWND>> {
                match mode {
                    ZMode::InsertAboveHost => desktop::insert_above(w, host.host),
                    ZMode::HwndBottom => desktop::send_to_bottom(w).map(|()| None),
                }
            };
            let _ = place(s.sentinel.hwnd());
            // Each fence is inserted directly above the host, so the *last* one placed ends up
            // lowest. Iterate in reverse: the end of `fences` (newest / last raised) is on top.
            for (f, shadow) in s.fences.clone().into_iter().rev() {
                match place(f) {
                    Ok(above) => placed_above = placed_above.or(above),
                    Err(e) => {
                        tracing::debug!(hwnd = ?f, error = %e, "anchor SetWindowPos failed");
                        failed += 1;
                        refused.get_or_insert((f, e));
                    }
                }
                if let Some(sh) = shadow {
                    let _ = desktop::insert_after(sh, f);
                }
            }
        });
        self.note_refusal(host.host, refused, failed);
        self.note_placed_above(placed_above);
    }

    /// Logs once when the fences start going above a window they may not go under (one of an
    /// elevated program, lowest above the desktop) and once when they are directly above the
    /// desktop again: the anchor check repeats the placement every pass.
    fn note_placed_above(&mut self, above: Option<HWND>) {
        let key = above.map(|h| h.0 as isize);
        if key == self.placed_above {
            return;
        }
        self.placed_above = key;
        match above {
            Some(h) => tracing::info!(
                above = ?h,
                class = %desktop::class_name(h),
                above_elevated = ?pecofence_platform::process::is_elevated(desktop::window_pid(h)),
                "fences placed above a window they may not go under"
            ),
            None => tracing::info!("fences directly above the desktop again"),
        }
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
    pub fn is_desktop_window(&self, hwnd: HWND) -> bool {
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

    /// Is `hwnd`'s root window Explorer's taskbar (on any monitor)?
    fn is_taskbar(&self, hwnd: HWND) -> bool {
        if hwnd.0.is_null() {
            return false;
        }
        let root = desktop::root_ancestor(hwnd);
        let cls = desktop::class_name(root);
        (cls == "Shell_TrayWnd" || cls == "Shell_SecondaryTrayWnd")
            && desktop::window_pid(root) == desktop::window_pid(desktop::shell_window())
    }

    /// The desktop's clicks are followed (the raw input sink is on) while the desktop, the
    /// taskbar or no window at all is in front. From the last two a click brings the desktop to
    /// the front without a foreground event (sign-in, the end of a process whose window was in
    /// front), so waiting for that event lost the double-click. Windows 10 often reports no
    /// event for a click on the desktop from any window (an app that opened at sign-in, a
    /// virtual desktop switch): there they are followed all the time. The sink takes only
    /// presses on the bare desktop (`begin_marquee`); it costs one message per pointer move
    /// (measured: under 1 % of a core while the pointer moves without a pause).
    fn follows_desktop_clicks(&self, hwnd: HWND) -> bool {
        self.generation == Generation::Legacy
            || hwnd.0.is_null()
            || self.is_desktop_window(hwnd)
            || self.is_taskbar(hwnd)
    }

    /// One of our fences, popups (rename, notice) or menus: show desktop goes on while it is in
    /// front. Settings and the dialogs of this process (Explorer's Properties, a confirmation)
    /// end it like any app's window, or the floating fences would cover them.
    fn is_own_overlay(&self, hwnd: HWND) -> bool {
        let root = desktop::root_ancestor(hwnd);
        let cls = desktop::class_name(root);
        ((cls.starts_with("PecoFence.") && cls != crate::settings_host::SETTINGS_CLASS)
            || cls == "#32768")
            && desktop::window_pid(root) == std::process::id()
    }

    /// Have our windows ended up *below* the desktop window `host`? (Explorer raised it.)
    fn desktop_is_raised_over_us(host: Option<IconHost>) -> bool {
        let Some(h) = host else { return false };
        desktop::is_below(h.host, SENTINEL_CLASS) || desktop::is_below(h.host, FENCE_CLASS)
    }

    /// Show desktop (Win+D): Explorer raises the desktop window over every other window, the
    /// fences included, and gives it the foreground; it is over once another window comes to
    /// the front. On 24H2+ Progman itself moves up, on older builds the WorkerW the icons were
    /// moved into (`host`, as resolved now). Windows lets no normal window go above the raised
    /// desktop (SetWindowPos succeeds and changes nothing), so with 「保持栅栏可见」 the fences
    /// float in the topmost band until it ends; with 「随桌面隐藏」 they stay behind it.
    /// `reason`: which check found it (the foreground hook, its retry, the periodic check).
    fn check_show_desktop(&mut self, host: Option<IconHost>, reason: &'static str) {
        let foreground = desktop::foreground_window();
        if !self.show_desktop_active {
            if Self::desktop_is_raised_over_us(host) && self.is_desktop_window(foreground) {
                self.show_desktop_active = true;
                self.show_desktop_since = Some(Instant::now());
                if let Some(callback) = self.on_desktop_front.as_ref() {
                    callback();
                }
                let host_kind = host.map(|h| h.kind);
                match self.behavior {
                    ShowDesktopBehavior::KeepVisible => tracing::info!(
                        reason,
                        ?host_kind,
                        layers = %desktop::describe_desktop_windows(),
                        "show desktop: floating the fences above the raised desktop"
                    ),
                    ShowDesktopBehavior::HideWithDesktop => tracing::info!(
                        reason,
                        ?host_kind,
                        layers = %desktop::describe_desktop_windows(),
                        "show desktop: fences stay behind the raised desktop"
                    ),
                }
            }
        } else if !foreground.0.is_null()
            && !self.is_desktop_window(foreground)
            && !self.is_own_overlay(foreground)
            && !is_shell_popup(desktop::class_name(desktop::root_ancestor(foreground)).as_str())
        {
            self.end_show_desktop("another window came to the front");
        }
    }

    /// A window of another program that shows over the raised desktop where a fence is, while
    /// the desktop still has the foreground: one shown without being activated. Explorer leaves
    /// it above the raised desktop, and the fences, floating in the topmost band, covered it
    /// until it came to the front (measured with a stand-in window on Windows 10 19045,
    /// Windows 11 26300 and Server 2025). A black window then looks like a wallpaper gone
    /// black with only the icons and the taskbar left, which is what #187 describes after
    /// opening a file with WPS Office; WPS itself was not tested. The fences go under such a
    /// window (`show_desktop_under`). Windows that are always on top were over the desktop
    /// before and do not count, nor do overlays, tool windows and Windows' own surfaces.
    fn program_window_over_raised_desktop(&self, host: Option<IconHost>) -> Option<HWND> {
        let host = host?;
        let windows = desktop::top_level_windows();
        let host_index = windows.iter().position(|&w| w == host.host)?;
        let own = std::process::id();
        let fences: Vec<RECT> = self
            .fences
            .iter()
            .filter(|(f, _)| desktop::is_visible(*f))
            .map(|(f, _)| window::window_rect(*f))
            .collect();
        windows[..host_index].iter().copied().find(|&w| {
            // Nearly every window above the raised desktop is always on top: that test first.
            let ex = desktop::ex_style(w);
            let overlay = ex & style::EX_LAYERED != 0 && ex & style::EX_TRANSPARENT != 0;
            if desktop::is_topmost(w)
                || overlay
                || ex & (style::EX_TOOLWINDOW | style::EX_NOACTIVATE) != 0
                || !desktop::has_visible_surface(w)
                || desktop::window_pid(w) == own
                || self.is_desktop_window(w)
            {
                return false;
            }
            let class = desktop::class_name(w);
            if class == SHELL_CORE_WINDOW || is_shell_popup(&class) {
                return false;
            }
            let rect = desktop::frame_rect(w).unwrap_or_else(|| window::window_rect(w));
            fences
                .iter()
                .any(|f| crate::layout::clearance::overlaps(&rect, f, 0))
        })
    }

    /// Leaves the topmost band straight into the fences' place above the desktop: the windows
    /// Windows restores (and the one that came to the front) stay above them. Taking the topmost
    /// flag away first put the fences over every window until the next check.
    fn end_show_desktop(&mut self, reason: &'static str) {
        self.show_desktop_active = false;
        self.show_desktop_under = None;
        self.topmost_refused = false;
        let front = desktop::root_ancestor(desktop::foreground_window());
        // A wallpaper that "went black" while a program opened after show desktop (#187) may be
        // Explorer rebuilding its desktop windows or the program's own window under fences
        // that still floated (the second was reproduced with a stand-in window, the report's
        // own program was not tested): how long show desktop lasted, whether that window is
        // maximized and what Explorer's desktop windows are now tell the two apart.
        let lasted_ms = self
            .show_desktop_since
            .take()
            .map_or(0, |t| t.elapsed().as_millis() as u64);
        tracing::info!(
            reason,
            class = %desktop::class_name(front),
            front_zoomed = desktop::is_zoomed(front),
            lasted_ms,
            layers = %desktop::describe_desktop_windows(),
            "show desktop ended"
        );
        self.reanchor(reason);
    }

    /// The window that sits between the host and our windows, or the host itself when some of
    /// ours are below it (`is_anchored` is false); `None` when they are all in place.
    fn first_intruder(&self) -> Option<HWND> {
        let h = self.host?;
        let ours = self.all_windows();
        let windows = desktop::top_level_windows();
        let host_index = windows.iter().position(|&w| w == h.host)?;
        let mut remaining = ours.len();
        for &cursor in windows[..host_index].iter().rev() {
            if ours.contains(&cursor) {
                remaining -= 1;
                if remaining == 0 {
                    return None;
                }
            } else if desktop::has_visible_surface(cursor) && !is_drag_overlay(cursor) {
                return Some(cursor);
            }
        }
        Some(h.host)
    }

    /// The fences' block above the desktop was broken (a window went between, or the desktop
    /// came over some of them): logs what did it, once per window, until it happens again
    /// with another one.
    fn note_covered(&mut self, reason: &'static str) {
        let Some(by) = self.first_intruder() else {
            return;
        };
        let key = Some(by.0 as isize);
        if key == self.covered_by {
            return;
        }
        self.covered_by = key;
        let desktop = self.host.is_some_and(|h| h.host == by);
        tracing::info!(
            reason,
            window = ?by,
            class = %desktop::class_name(by),
            desktop,
            elevated = ?pecofence_platform::process::is_elevated(desktop::window_pid(by)),
            "fences covered; placing them above the desktop again"
        );
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
        self.last_foreground = hwnd.0 as isize;
        self.last_foreground_at = Instant::now();
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
        let follow = self.follows_desktop_clicks(hwnd);
        let raised = Self::desktop_is_raised_over_us(self.host);
        let held = window::primary_button_down_async();
        let back_to = self.fence_after_dialog(held);
        tracing::debug!(?hwnd, %cls, is_desktop, raised, "foreground changed");
        if cls == SWITCHER_CLASS {
            self.switch_looks = AFTER_SWITCH_LOOKS;
            self.sentinel
                .set_timer(TIMER_FOREGROUND_AFTER_SWITCH, AFTER_SWITCH_LOOK_MS);
        }
        // Only the changes to and from the desktop, once each: they say what was in front and
        // whether the desktop's clicks (quick hide, marquee) are followed. Leaving the desktop,
        // the line also counts what raw input reported since the last such line.
        if is_desktop != self.desktop_front {
            self.desktop_front = is_desktop;
            let button_down = window::primary_button_down_async();
            let pt = window::cursor_pos();
            if is_desktop {
                tracing::info!(
                    class = %cls,
                    desktop = is_desktop,
                    raised,
                    button_down,
                    ?pt,
                    "foreground moved to or from the desktop"
                );
            } else {
                let seen = std::mem::take(&mut self.quick.seen);
                tracing::info!(
                    class = %cls,
                    desktop = is_desktop,
                    follow,
                    raised,
                    button_down,
                    ?pt,
                    inputs = seen.inputs,
                    unreadable = seen.unreadable,
                    presses = seen.presses,
                    taken = seen.taken,
                    on_ours = seen.on_ours,
                    not_desktop = seen.not_desktop,
                    on_icon = seen.on_icon,
                    "foreground moved to or from the desktop"
                );
            }
        }

        if is_desktop {
            if let Some(callback) = self.on_desktop_front.as_ref() {
                callback();
            }
            let sink_was_on = self.quick.sink_registered;
            self.set_raw_input_sink(true);
            let pt = window::cursor_pos();
            self.quick
                .desktop_activated(sink_was_on, held, Instant::now(), pt);
            // The press that activated the desktop reached Explorer before the sink existed: a
            // drag it starts still draws a marquee (from where the pointer is now). Activated
            // any other way, the desktop ends the fence selection like a click on it does,
            // unless a dialog opened from a fence just closed (the fence takes it back below).
            if held {
                self.begin_marquee(pt);
            } else if back_to.is_none() {
                self.emit_marquee(MarqueeEvent::Pressed {
                    additive: additive_modifier(),
                });
            }
        } else {
            self.set_raw_input_sink(follow);
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
        if let Some(fence) = back_to {
            // As Explorer's window gets the keyboard back when its Properties close.
            let claimed = window::claim_foreground(fence);
            tracing::info!(
                claimed,
                "the fence the dialog was opened from takes the foreground back"
            );
        }
    }

    /// The fences that windows of other programs hide completely right now (or will, once a
    /// Peek ends and the fences are back above the desktop), shadow included. Such a fence
    /// fades in and out with the Peek's dimmer; one of which any part shows is left out,
    /// because fading it would make that part blink. A window counts when it paints a surface,
    /// is a program's window (no overlay, no surface of Windows itself), and its drawn frame
    /// holds the whole fence (the part of it in its own monitor's work area).
    pub fn covered_fences(&self) -> Vec<HWND> {
        // Fences kept over the raised desktop are in the topmost band: nothing covers them.
        if self.show_desktop_active
            && self.behavior == ShowDesktopBehavior::KeepVisible
            && self.show_desktop_under.is_none()
        {
            return Vec::new();
        }
        let Some(host) = self.host else {
            return Vec::new();
        };
        let windows = desktop::top_level_windows();
        let Some(host_index) = windows.iter().position(|&w| w == host.host) else {
            return Vec::new();
        };
        let own = std::process::id();
        // Overlays and Windows' own surfaces span the screen without hiding it: the task
        // switcher and the desktop switcher are tool windows that take no focus, Start, search
        // and the notification centre are top-level core windows.
        let no_cover =
            style::EX_LAYERED | style::EX_TRANSPARENT | style::EX_TOOLWINDOW | style::EX_NOACTIVATE;
        let covers: Vec<(HWND, RECT)> = windows[..host_index]
            .iter()
            .filter(|&&w| {
                desktop::has_visible_surface(w)
                    && desktop::ex_style(w) & no_cover == 0
                    && desktop::window_pid(w) != own
                    && desktop::class_name(w) != SHELL_CORE_WINDOW
            })
            .filter_map(|&w| desktop::frame_rect(w).map(|r| (w, r)))
            .collect();
        if covers.is_empty() {
            return Vec::new();
        }
        let monitors = pecofence_platform::monitors::enumerate();
        self.fences
            .iter()
            .filter(|(f, shadow)| {
                if !desktop::is_visible(*f) {
                    return false;
                }
                let body = window::window_rect(*f);
                let cast = shadow
                    .filter(|s| desktop::is_visible(*s))
                    .map_or(body, window::window_rect);
                // The shadow's spill under the taskbar or onto the next monitor does not count.
                let screen = monitors
                    .iter()
                    .find(|m| crate::layout::clearance::overlaps(&m.bounds, &body, 0))
                    .map(|m| m.work_area);
                let seen = screen.map_or(cast, |b| RECT {
                    left: cast.left.max(b.left),
                    top: cast.top.max(b.top),
                    right: cast.right.min(b.right),
                    bottom: cast.bottom.min(b.bottom),
                });
                let by = covers.iter().find(|(_, c)| rect_holds(c, &seen));
                if let Some((w, _)) = by {
                    tracing::debug!(
                        fence = ?f,
                        by = ?w,
                        class = %desktop::class_name(*w),
                        ex_style = format_args!("{:#x}", desktop::ex_style(*w)),
                        "fence hidden by a window"
                    );
                }
                by.is_some()
            })
            .map(|(f, _)| *f)
            .collect()
    }

    /// Before `set_peek(true)`: `fences` stay where they are (under the windows that hide
    /// them) until `release_peek_held`.
    pub fn hold_for_peek(&mut self, fences: &[HWND]) {
        self.peek_held = fences.to_vec();
    }

    /// Whether the Peek has not raised `hwnd` yet (`hold_for_peek`).
    pub fn is_peek_held(&self, hwnd: HWND) -> bool {
        self.peek_held.contains(&hwnd)
    }

    /// The held fences join the others in the Peek's band. False when none was held.
    pub fn release_peek_held(&mut self) -> bool {
        if self.peek_held.is_empty() {
            return false;
        }
        let count = self.peek_held.len();
        self.peek_held.clear();
        if self.peeking {
            self.place_topmost_band();
        }
        tracing::debug!(count, "covered fences raised into the peek band");
        true
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
                    if s.peek_held.contains(&f) {
                        continue;
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
            self.peek_held.clear();
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
            self.check_show_desktop(self.host, "peek off");
            self.reanchor("peek off");
            tracing::info!("peek off");
        }
    }

    /// Re-anchors only if the host changed or our block is no longer directly above it.
    fn ensure_anchored(&mut self, reason: &'static str) {
        if self.peeking {
            return;
        }
        // Windows 11 can cancel a sign-out without sending us WM_ENDSESSION(0)
        // (`shutdown_cancelled`): once the session goes on, take back the icons it left hidden.
        if self.icons.left_hidden && !window::session_ending() && !window::quit_posted() {
            self.icons.left_hidden = false;
            if !self.icons.we_hid_them {
                tracing::info!("session goes on: taking back the desktop icons left hidden");
                self.request_hide_desktop_icons();
            }
        }
        // A hide that failed at startup (desktop not ready) is retried on every tick, not only
        // when the z-order is off, so the setting converges with reality without a toggle.
        if self.icons.pending && self.icons.pending_attempts < HIDE_ICONS_MAX_ATTEMPTS {
            self.icons.pending_attempts += 1;
            if self.hide_desktop_icons_within(ICONS_QUICK_WAIT) {
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
        self.check_show_desktop(resolved, reason);
        if self.show_desktop_active {
            if resolved != self.host {
                // Windows 10: the WorkerW the icons moved into is the raised desktop.
                tracing::info!(
                    reason,
                    old = ?self.host,
                    new = ?resolved,
                    layers = %desktop::describe_desktop_windows(),
                    "icon host changed"
                );
                self.host = resolved;
            }
            if self.behavior == ShowDesktopBehavior::KeepVisible {
                let under = self.program_window_over_raised_desktop(resolved);
                let key = under.map(|w| w.0 as isize);
                if key != self.show_desktop_under {
                    self.show_desktop_under = key;
                    match under {
                        Some(by) => tracing::info!(
                            window = ?by,
                            class = %desktop::class_name(by),
                            "show desktop: a program's window shows over the raised desktop; the fences go under it"
                        ),
                        None => tracing::info!(
                            "show desktop: that window is gone; the fences float again"
                        ),
                    }
                }
                if under.is_some() {
                    // Out of the topmost band into their place above the desktop: under it.
                    if self.fences.iter().any(|(f, _)| desktop::is_topmost(*f)) {
                        self.reanchor(reason);
                    }
                } else if !self.fences.iter().all(|(f, _)| desktop::is_topmost(*f)) {
                    self.place_topmost_band();
                }
            }
            return;
        }
        self.sync_reorder_hook();
        let host_changed = resolved != self.host;
        if host_changed || !self.is_anchored() {
            if !host_changed {
                self.note_covered(reason);
            }
            self.reanchor(reason);
            for (w, _) in self.fences.clone() {
                if desktop::is_cloaked(w) {
                    tracing::info!(hwnd = ?w, "fence was cloaked; uncloaking");
                    if let Err(e) = desktop::set_cloak(w, false) {
                        tracing::warn!(hwnd = ?w, error = %e, "uncloaking a fence failed");
                    }
                }
                if (!desktop::is_visible(w) || desktop::is_iconic(w))
                    && !self.quick.fences_hidden
                    && !self.off_desktop.contains(&(w.0 as isize))
                {
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
                if on {
                    self.quick.seen = SinkCounts::default();
                }
                tracing::debug!(on, "raw input sink");
            }
            Err(e) => tracing::warn!(on, error = %e, "RegisterRawInputDevices failed"),
        }
    }

    /// `WM_INPUT` on the sentinel: left button down/up on the bare desktop drive quick hide
    /// (double-click) and the marquee (fence selection / new fence); moves in between draw it.
    fn on_raw_input(&mut self, lparam: isize) {
        self.quick.seen.inputs += 1;
        let Some(buttons) = rawinput::mouse_buttons_from_wm_input(lparam) else {
            let error = std::io::Error::last_os_error();
            self.quick.seen.unreadable += 1;
            // Once per stay of the desktop in front: a broken read would repeat on every move.
            if self.quick.seen.unreadable == 1 {
                tracing::debug!(%error, "raw input without readable mouse data");
            }
            return;
        };
        if buttons.left_down {
            self.quick.seen.presses += 1;
        }
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
            tracing::debug!(?pt, "press on the bare desktop; quick hide is off");
            return;
        }
        let limit = pecofence_platform::sysparams::double_click_time_ms() as u128;
        let slop = rawinput::double_click_slop_px();
        if let Some((interval_ms, first)) =
            self.quick.desktop_press(Instant::now(), pt, limit, slop)
        {
            if self.windows_icon_selected() {
                tracing::info!(
                    ?pt,
                    interval_ms,
                    "double-click on one of Windows' own desktop icons (it is selected); no quick hide"
                );
                return;
            }
            tracing::info!(
                ?pt,
                interval_ms,
                ?first,
                "double-click on empty desktop → quick hide on release"
            );
            self.quick.pending_toggle = Some(pt);
            self.sentinel.set_timer(TIMER_QUICKHIDE, QUICKHIDE_DELAY_MS);
        }
    }

    /// The left button is down at `pt`. On the bare desktop (not over one of our windows or one
    /// of Windows' own icons) a marquee may start there while the fences show; returns false
    /// anywhere else.
    fn begin_marquee(&mut self, pt: POINT) -> bool {
        let hit = desktop::window_from_point(pt.x, pt.y);
        let root = desktop::root_ancestor(hit);
        let refused = if self.all_windows().contains(&root) {
            Some(PressRefused::OnOurs)
        } else if !self.is_desktop_window(hit) {
            Some(PressRefused::NotDesktop)
        } else if on_icon(hit) {
            Some(PressRefused::OnIcon)
        } else {
            None
        };
        if let Some(reason) = refused {
            let seen = &mut self.quick.seen;
            match reason {
                PressRefused::OnOurs => seen.on_ours += 1,
                PressRefused::NotDesktop => seen.not_desktop += 1,
                PressRefused::OnIcon => seen.on_icon += 1,
            }
            tracing::debug!(
                ?pt,
                ?reason,
                hit = %desktop::class_name(hit),
                top = %desktop::class_name(root),
                explorer = desktop::window_pid(root) == desktop::window_pid(desktop::shell_window()),
                "press not taken as one on the bare desktop"
            );
            self.quick.marquee_origin = None;
            self.quick.marquee_live = false;
            return false;
        }
        self.quick.seen.taken += 1;
        tracing::debug!(
            ?pt,
            hit = %desktop::class_name(hit),
            top = %desktop::class_name(root),
            "press taken as one on the bare desktop"
        );
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

    /// The Windows icons show and one of them is selected: the double-click just seen was on
    /// that icon. Its first click selected it, and a click on the bare desktop selects none.
    /// `on_icon` does not catch every press on an icon: Explorer's list view marks the icon
    /// under the pointer when the pointer moves onto it, drops the mark when a window comes
    /// over the pointer, and does not set it again when that window goes away (closed, or
    /// show desktop) while the pointer rests where it was. A double-click on the icon then
    /// passed as one on the bare desktop and hid every fence (measured on Windows 10 19045
    /// and Windows 11 26300).
    fn windows_icon_selected(&self) -> bool {
        !shell_icons::desktop_icons_hidden()
            && self
                .host
                .and_then(|h| shell_icons::desktop_selected_count(h.def_view))
                .is_some_and(|n| n > 0)
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

    /// Quick hide's double-click on the empty desktop.
    pub fn toggle_fences_hidden(&mut self) {
        if self.windows_desktop {
            // Showing the fences here would leave Windows' icons under them and the mode on.
            tracing::info!(
                "double-click on the desktop during the temporary Windows desktop; the App ends it"
            );
            self.emit_marquee(MarqueeEvent::DoubleClickInWindowsDesktop);
            return;
        }
        self.toggle_fences_hidden_by("double-click on the desktop");
    }

    /// `reason` (a fixed text) goes to the log.
    pub fn toggle_fences_hidden_by(&mut self, reason: &'static str) {
        self.set_fences_hidden(!self.quick.fences_hidden, false, reason);
    }

    /// Hides or shows the fences as quick hide does; `every` includes the ones kept on the
    /// desktop by 「双击桌面隐藏时保留此栅栏」 (「临时显示 Windows 桌面」 hides those too).
    pub fn set_fences_hidden(&mut self, hide: bool, every: bool, reason: &'static str) {
        self.quick.fences_hidden = hide;
        for (w, shadow) in self.fences.clone() {
            if !every && self.quick_hide_excluded.contains(&(w.0 as isize)) {
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
            reason,
            "fences visibility toggled"
        );
    }

    /// Hides the real desktop icons (writing the crash-recovery marker first).
    pub fn hide_desktop_icons(&mut self) -> bool {
        self.hide_desktop_icons_within(ICONS_ASKED_WAIT)
    }

    /// `hide_desktop_icons`, waiting `wait` at most for Explorer. A toggle it has not answered
    /// by then stays out with the marker: the retries (`pending`) find the icons hidden and
    /// adopt them, and send no second toggle meanwhile.
    fn hide_desktop_icons_within(&mut self, wait: Duration) -> bool {
        if self.host.is_none() {
            self.host = desktop::resolve_icon_host(self.generation);
        }
        let Some(h) = self.host else { return false };
        let kept = self.icons.kept.clone();
        if shell_icons::desktop_icons_hidden() {
            if self.icons.marker.exists() || kept.exists() {
                // Our previous run died without restoring them, or left them hidden on purpose
                // (`release_desktop_icons`), or Explorer answered our toggle late. Taking them
                // back lets exit restore or keep them as the setting says, and a fresh watchdog
                // covers a crash.
                tracing::info!("desktop icons hidden under our marker; adopting");
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
        match shell_icons::set_desktop_icons_hidden_within(true, h.def_view, h.host, wait) {
            shell_icons::IconToggle::Done(true) => {
                self.icons.we_hid_them = true;
                tracing::info!("desktop icons hidden (0x7402)");
                self.spawn_watchdog();
                true
            }
            shell_icons::IconToggle::Waiting => {
                // Retried from ensure_anchored: the first check after Explorer answered adopts.
                self.icons.pending = true;
                tracing::info!(
                    wait_ms = wait.as_millis(),
                    "desktop icons hide out; Explorer has not answered it yet"
                );
                false
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
        if self.hide_desktop_icons_within(ICONS_QUICK_WAIT) {
            self.icons.pending = false;
            self.icons.pending_attempts = 0;
            return true;
        }
        self.icons.pending = true;
        self.icons.pending_attempts = 0;
        tracing::warn!("desktop icons not hidden yet; will retry when the desktop is ready");
        false
    }

    /// Runs the next checks (and so a pending hide) on the short retry ladder instead of
    /// waiting for the 2 s check.
    pub fn retry_soon(&mut self) {
        self.retry_step = 0;
        self.sentinel.set_timer(TIMER_RETRY, RETRY_DELAYS_MS[0]);
    }

    /// True after the retry budget was exhausted without success (caller should stop the
    /// retries and tell the user).
    pub fn hide_desktop_icons_gave_up(&self) -> bool {
        self.icons.pending && self.icons.pending_attempts >= HIDE_ICONS_MAX_ATTEMPTS
    }

    pub fn cancel_pending_hide(&mut self) {
        self.icons.pending = false;
        self.icons.pending_attempts = 0;
        self.icons.left_hidden = false;
    }

    /// Starts `pecofence-watchdog.exe` (next to our exe): after a crash it starts PecoFence
    /// again, and it shows the icons PecoFence hid when no PecoFence runs afterwards.
    pub fn spawn_watchdog(&mut self) {
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
        // After a crash it starts us again with these arguments.
        let ours: Vec<String> = std::env::args().skip(1).collect();
        let mut args = vec![pid.as_str(), marker.as_str()];
        args.extend(ours.iter().map(String::as_str));
        match pecofence_platform::process::spawn_detached(&wd.to_string_lossy(), &args) {
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
            match shell_icons::set_desktop_icons_hidden_within(
                false,
                h.def_view,
                h.host,
                ICONS_RESTORE_WAIT,
            ) {
                shell_icons::IconToggle::Done(false) => tracing::info!("desktop icons restored"),
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
            let _ = std::fs::write(&self.icons.kept, b"pecofence keeps desktop icons hidden\n");
            self.icons.we_hid_them = false;
            self.icons.left_hidden = true;
            let _ = std::fs::remove_file(&self.icons.marker);
            tracing::info!("desktop icons left hidden for the next sign-in");
            return;
        }
        self.restore_desktop_icons();
    }

    /// The previous run left the icons hidden (`release_desktop_icons`), but this one hides none
    /// (「Windows 桌面图标」 shown or `--no-hide-icons`): shows them again. The kept marker
    /// stays while Explorer does not answer, so the next start tries again.
    pub fn show_kept_desktop_icons(&mut self) {
        let kept = self.icons.kept.clone();
        if !kept.exists() {
            return;
        }
        if shell_icons::desktop_icons_hidden() {
            let shown = self
                .host
                .or_else(|| desktop::resolve_icon_host(self.generation))
                .map(|h| {
                    shell_icons::set_desktop_icons_hidden_within(
                        false,
                        h.def_view,
                        h.host,
                        ICONS_QUICK_WAIT,
                    )
                });
            if shown != Some(shell_icons::IconToggle::Done(false)) {
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
            TIMER_FOREGROUND_POLL => {
                // While the fences float over the raised desktop, a program's window that shows
                // without taking the foreground comes with no event of ours: found here, it is
                // under the fences for about a quarter of a second, not until the 2 s check.
                if self.show_desktop_active && self.behavior == ShowDesktopBehavior::KeepVisible {
                    self.ensure_anchored("show desktop poll");
                }
                // Windows 10 reports no foreground change when Win+Ctrl+Left/Right shows another
                // virtual desktop, and the desktop comes to the front only with the first click on
                // it: that click went unseen and the first double-click hid nothing. The current
                // desktop is one registry read, so the poll notices the switch itself.
                let current = pecofence_platform::wallpaper::desktop_id();
                if current != self.poll_desktop {
                    let before = std::mem::replace(&mut self.poll_desktop, current);
                    if before.is_some() && current.is_some() {
                        self.follow_desktop_after_switch();
                    }
                }
                let front = desktop::foreground_window();
                if front.0 as isize == self.last_foreground {
                    return;
                }
                if front.0.is_null() {
                    // No window in front (a window closed, nothing else was activated): the
                    // click that brings the desktop back may come without an event.
                    self.last_foreground = 0;
                    if !self.quick.sink_registered {
                        tracing::info!("no window in front; following the desktop's clicks");
                        self.set_raw_input_sink(true);
                    }
                    return;
                }
                // Our own windows (Peek's dimmer and search box, Settings) are not reported by the
                // hook either, on purpose.
                if desktop::window_pid(desktop::root_ancestor(front)) == std::process::id() {
                    self.last_foreground = front.0 as isize;
                    let root = desktop::root_ancestor(front);
                    if let Some(d) = self.dialog_from.as_mut()
                        && d.dialog.is_none()
                        && desktop::class_name(root) == "#32770"
                    {
                        d.dialog = Some(root);
                        tracing::info!("a dialog opened from a fence is in front");
                    }
                    return;
                }
                // Only what the hook missed and matters: the desktop in front or gone from it, or
                // the taskbar (the desktop's clicks are followed then). Other changes come as
                // events.
                if self.is_desktop_window(front) != self.desktop_front
                    || self.follows_desktop_clicks(front) != self.quick.sink_registered
                {
                    let cls = desktop::class_name(desktop::root_ancestor(front));
                    tracing::info!(
                        class = %cls,
                        "foreground change Windows did not report; caught by the poll"
                    );
                    self.on_foreground(front);
                } else {
                    self.last_foreground = front.0 as isize;
                }
            }
            TIMER_SWITCH_FOLLOW_END => {
                self.sentinel.kill_timer(TIMER_SWITCH_FOLLOW_END);
                let front = desktop::foreground_window();
                if self.quick.sink_registered && !self.follows_desktop_clicks(front) {
                    let cls = desktop::class_name(desktop::root_ancestor(front));
                    tracing::info!(
                        class = %cls,
                        "the desktop did not come to the front after the switch; its clicks are no longer followed"
                    );
                    self.set_raw_input_sink(false);
                }
            }
            TIMER_FOREGROUND_AFTER_SWITCH => {
                let front = desktop::foreground_window();
                self.switch_looks = self.switch_looks.saturating_sub(1);
                let missed = !front.0.is_null()
                    && front.0 as isize != self.last_foreground
                    && (self.is_desktop_window(front) != self.desktop_front
                        || self.follows_desktop_clicks(front) != self.quick.sink_registered);
                if missed || self.switch_looks == 0 {
                    self.sentinel.kill_timer(TIMER_FOREGROUND_AFTER_SWITCH);
                }
                if missed {
                    let cls = desktop::class_name(desktop::root_ancestor(front));
                    tracing::info!(
                        class = %cls,
                        looks_left = self.switch_looks,
                        "foreground after a virtual desktop switch caught"
                    );
                    self.on_foreground(front);
                }
            }
            TIMER_REORDER_LATER => {
                self.sentinel.kill_timer(TIMER_REORDER_LATER);
                self.on_shell_reorder();
            }
            TIMER_FOREGROUND_LATER => {
                self.sentinel.kill_timer(TIMER_FOREGROUND_LATER);
                let front = desktop::foreground_window();
                let cls = desktop::class_name(desktop::root_ancestor(front));
                tracing::info!(class = %cls, "foreground change handled after the anchor was busy");
                self.on_foreground(front);
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
            _ => {}
        }
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
        let ours = self.all_windows();
        let above_windows = |w: HWND| {
            let Some(at) = order.iter().position(|&x| x == w) else {
                return false;
            };
            // The windows between the fence and the host (none for a fence below the host).
            let between = order.get(at + 1..host_at.unwrap_or(order.len()));
            between.is_some_and(|between| {
                between.iter().any(|&x| {
                    !ours.contains(&x)
                        && !is_drag_overlay(x)
                        && !self.is_desktop_window(x)
                        && desktop::has_visible_surface(x)
                })
            })
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
            above_windows: count(&above_windows),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn pt(x: i32, y: i32) -> POINT {
        POINT { x, y }
    }

    fn ms(t: Instant, ms: u64) -> Instant {
        t + Duration::from_millis(ms)
    }

    #[test]
    fn a_window_hides_a_fence_only_when_it_holds_all_of_it() {
        let r = |left, top, right, bottom| RECT {
            left,
            top,
            right,
            bottom,
        };
        let window = r(0, 0, 1600, 852);
        // The fence with its shadow, inside the window and up to its edges.
        assert!(rect_holds(&window, &r(180, 140, 520, 400)));
        assert!(rect_holds(&window, &r(0, 0, 1600, 852)));
        // One pixel out on any side shows beside the window: not hidden.
        assert!(!rect_holds(&window, &r(-1, 140, 520, 400)));
        assert!(!rect_holds(&window, &r(1300, 600, 1601, 800)));
        assert!(!rect_holds(&window, &r(1300, 600, 1600, 853)));
        // A fence wholly off its monitor's work area (nothing left after the clip).
        assert!(!rect_holds(&window, &r(400, 300, 400, 500)));
    }

    #[test]
    fn a_click_that_brings_the_desktop_back_counts_once_when_raw_input_saw_it() {
        // The desktop was in front (sink on), then a fence or Explorer's menu; the click on the
        // desktop or on 刷新 brings the desktop back, and raw input reports that click as well.
        let t = Instant::now();
        let mut q = QuickHide::default();
        q.desktop_activated(true, true, t, pt(700, 560));
        assert_eq!(q.desktop_press(ms(t, 2), pt(700, 560), 500, (2, 2)), None);
        // A real second click still makes it a double-click.
        assert_eq!(
            q.desktop_press(ms(t, 200), pt(701, 560), 500, (2, 2)),
            Some((198, pt(700, 560)))
        );
        // A third one starts over.
        assert_eq!(q.desktop_press(ms(t, 350), pt(701, 560), 500, (2, 2)), None);
    }

    #[test]
    fn the_press_that_activates_the_desktop_unseen_is_the_first_click() {
        // Another program was in front (sink off): raw input never reports that press.
        let t = Instant::now();
        let mut q = QuickHide::default();
        q.desktop_activated(false, true, t, pt(300, 420));
        assert_eq!(
            q.desktop_press(ms(t, 150), pt(300, 421), 500, (2, 2)),
            Some((150, pt(300, 420)))
        );
    }

    #[test]
    fn a_menu_item_click_that_brings_the_desktop_back_is_no_first_click() {
        // Explorer's menu was in front (sink off); its 刷新 runs on the release, so the button is
        // up when the desktop comes back. A click there right after is a single click.
        let t = Instant::now();
        let mut q = QuickHide::default();
        q.desktop_activated(false, false, t, pt(390, 204));
        assert_eq!(q.desktop_press(ms(t, 120), pt(390, 204), 500, (2, 2)), None);
    }

    #[test]
    fn after_a_windows_10_switch_only_the_taskbar_counts_as_no_program_in_front() {
        assert!(is_taskbar("Shell_TrayWnd") && is_taskbar("Shell_SecondaryTrayWnd"));
        assert!(!is_taskbar("Notepad") && !is_taskbar("Progman") && !is_taskbar(""));
    }

    #[test]
    fn presses_too_far_apart_in_time_or_place_are_single_clicks() {
        let t = Instant::now();
        let mut q = QuickHide::default();
        assert_eq!(q.desktop_press(t, pt(100, 100), 500, (2, 2)), None);
        assert_eq!(q.desktop_press(ms(t, 600), pt(100, 100), 500, (2, 2)), None);
        assert_eq!(q.desktop_press(ms(t, 700), pt(105, 100), 500, (2, 2)), None);
        // Within 4 px (more than the 2 px of the system rectangle) is the same place.
        assert_eq!(
            q.desktop_press(ms(t, 800), pt(109, 104), 500, (2, 2)),
            Some((100, pt(105, 100)))
        );
    }

    #[test]
    fn the_kept_marker_stays_in_our_folder_when_the_crash_marker_is_legacy() {
        let root = std::env::temp_dir().join(format!("pecofence-icons-{}", std::process::id()));
        let ours = root.join("PecoFence");
        let legacy = root.join(pecofence_core::brand::LEGACY_DATA_DIR);
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("icons-hidden.marker"), b"").unwrap();
        let icons = DesktopIcons::in_dir(&ours, "icons-hidden.marker");
        assert_eq!(icons.marker, legacy.join("icons-hidden.marker"));
        assert_eq!(icons.kept, ours.join("icons-hidden.kept"));
        std::fs::remove_dir_all(&root).unwrap();
        let icons = DesktopIcons::in_dir(&ours, "icons-hidden.test.marker");
        assert_eq!(icons.marker, ours.join("icons-hidden.test.marker"));
        assert_eq!(icons.kept, ours.join("icons-hidden.test.kept"));
    }
}
