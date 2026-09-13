//! Top-level window creation with a per-window message handler closure.
//!
//! This replaces `windows-window` because fence windows need full control over class
//! registration (double clicks, no background brush), initial placement, show behaviour
//! (never activate) and multi-window lifetime (`WM_DESTROY` must not quit the process).

use crate::bindings::*;
use crate::wide::to_wide;
use windows_core::{Error, PCWSTR, Result};

/// Per-window message handler. Return `Some(result)` to handle the message, `None` to fall
/// through to `DefWindowProcW`.
///
/// Handlers are `Fn` (not `FnMut`) and may be invoked re-entrantly: a handler that calls
/// `SetWindowPos`/`ShowWindow`/`ReleaseCapture` receives the resulting messages synchronously
/// while it is still running. Keep per-window state in `Rc<RefCell<_>>` and release borrows
/// before calling such APIs.
pub type MessageHandler = Box<dyn Fn(HWND, u32, usize, isize) -> Option<isize>>;

pub fn set_title(hwnd: HWND, title: &str) {
    let wide = to_wide(title);
    // SAFETY: the UTF-16 buffer is null-terminated and valid throughout the call.
    unsafe {
        let _ = SetWindowTextW(hwnd, PCWSTR(wide.as_ptr()));
    }
}

struct State {
    handler: std::rc::Rc<dyn Fn(HWND, u32, usize, isize) -> Option<isize>>,
}

/// Window style bits as `u32`, independent of the generated constant types.
pub mod style {
    use crate::bindings as b;
    pub const POPUP: u32 = b::WS_POPUP;
    pub const VISIBLE: u32 = b::WS_VISIBLE as u32;
    pub const THICKFRAME: u32 = b::WS_THICKFRAME as u32;
    pub const CLIPCHILDREN: u32 = b::WS_CLIPCHILDREN as u32;
    pub const CLIPSIBLINGS: u32 = b::WS_CLIPSIBLINGS as u32;
    pub const CAPTION: u32 = b::WS_CAPTION as u32;
    pub const SYSMENU: u32 = b::WS_SYSMENU as u32;
    pub const OVERLAPPEDWINDOW: u32 = b::WS_OVERLAPPEDWINDOW as u32;

    pub const EX_NOACTIVATE: u32 = b::WS_EX_NOACTIVATE as u32;
    pub const EX_TOOLWINDOW: u32 = b::WS_EX_TOOLWINDOW as u32;
    pub const EX_NOREDIRECTIONBITMAP: u32 = b::WS_EX_NOREDIRECTIONBITMAP as u32;
    pub const EX_LAYERED: u32 = b::WS_EX_LAYERED as u32;
    pub const EX_TRANSPARENT: u32 = b::WS_EX_TRANSPARENT as u32;
    pub const EX_TOPMOST: u32 = b::WS_EX_TOPMOST as u32;
}

/// `SetWindowPos` flags as `u32`.
pub mod swp {
    use crate::bindings as b;
    pub const NOACTIVATE: u32 = b::SWP_NOACTIVATE as u32;
    pub const NOMOVE: u32 = b::SWP_NOMOVE as u32;
    pub const NOSIZE: u32 = b::SWP_NOSIZE as u32;
    pub const NOZORDER: u32 = b::SWP_NOZORDER as u32;
    pub const NOOWNERZORDER: u32 = b::SWP_NOOWNERZORDER as u32;
    pub const SHOWWINDOW: u32 = b::SWP_SHOWWINDOW as u32;
    pub const HIDEWINDOW: u32 = b::SWP_HIDEWINDOW as u32;
    pub const FRAMECHANGED: u32 = b::SWP_FRAMECHANGED as u32;
    pub const NOSENDCHANGING: u32 = b::SWP_NOSENDCHANGING as u32;
    pub const NOREDRAW: u32 = b::SWP_NOREDRAW as u32;
    pub const ASYNCWINDOWPOS: u32 = b::SWP_ASYNCWINDOWPOS as u32;
}

/// Z-order insertion targets for `set_bounds_z`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZOrder {
    Keep,
    Bottom,
    Top,
    NoTopMost,
    After(HWND),
}

/// Opts in to per-monitor-v2 DPI awareness. Call once, before creating any window.
pub fn set_process_dpi_awareness_v2() -> bool {
    // SAFETY: plain FFI call.
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).as_bool() }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ClassOptions {
    /// Receive `WM_LBUTTONDBLCLK` and friends.
    pub double_clicks: bool,
    /// Solid class background (COLORREF 0x00BBGGRR); `None` = no background painting.
    pub background: Option<u32>,
}

/// A registered window class; keep it alive while windows of this class exist.
pub struct WindowClass {
    name: Vec<u16>,
}

impl WindowClass {
    pub fn register(name: &str, options: ClassOptions) -> Result<Self> {
        let name = to_wide(name);
        let mut style = 0u32;
        if options.double_clicks {
            style |= CS_DBLCLKS as u32;
        }
        // SAFETY: the class struct is fully initialized and `name` outlives the call.
        unsafe {
            let wc = WNDCLASSEXW {
                cbSize: size_of::<WNDCLASSEXW>() as u32,
                style,
                lpfnWndProc: Some(wndproc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: HINSTANCE::default(),
                hIcon: HICON::default(),
                hCursor: LoadCursorW(None, IDC_ARROW),
                hbrBackground: match options.background {
                    Some(c) => CreateSolidBrush(COLORREF(c)),
                    None => HBRUSH::default(),
                },
                lpszMenuName: PCWSTR::null(),
                lpszClassName: PCWSTR(name.as_ptr()),
                hIconSm: HICON::default(),
            };
            if RegisterClassExW(&wc).0 == 0 {
                return Err(Error::from_thread());
            }
        }
        Ok(Self { name })
    }

    fn name_ptr(&self) -> PCWSTR {
        PCWSTR(self.name.as_ptr())
    }
}

impl Drop for WindowClass {
    fn drop(&mut self) {
        // SAFETY: the class was registered by us; failure (windows still alive) is ignored.
        unsafe {
            let _ = UnregisterClassW(self.name_ptr(), None);
        }
    }
}

pub struct WindowBuilder<'a> {
    class: &'a WindowClass,
    title: Vec<u16>,
    style: u32,
    ex_style: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    parent: Option<HWND>,
}

impl<'a> WindowBuilder<'a> {
    pub fn new(class: &'a WindowClass) -> Self {
        Self {
            class,
            title: to_wide(""),
            style: style::POPUP,
            ex_style: 0,
            x: CW_USEDEFAULT,
            y: CW_USEDEFAULT,
            width: CW_USEDEFAULT,
            height: CW_USEDEFAULT,
            parent: None,
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = to_wide(title);
        self
    }

    pub fn style(mut self, style: u32) -> Self {
        self.style = style;
        self
    }

    pub fn ex_style(mut self, ex_style: u32) -> Self {
        self.ex_style = ex_style;
        self
    }

    /// Outer window rectangle in physical pixels (screen coordinates).
    pub fn bounds(mut self, x: i32, y: i32, width: i32, height: i32) -> Self {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
        self
    }

    pub fn parent(mut self, parent: HWND) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Creates the window (hidden). Call `show_no_activate` to display it.
    pub fn create(self, handler: MessageHandler) -> Result<Window> {
        // SAFETY: all string buffers outlive the call; the state box is owned by the window
        // and freed in WM_NCDESTROY.
        unsafe {
            let hwnd = CreateWindowExW(
                self.ex_style,
                self.class.name_ptr(),
                PCWSTR(self.title.as_ptr()),
                self.style & !style::VISIBLE,
                self.x,
                self.y,
                self.width,
                self.height,
                self.parent,
                None,
                None,
                None,
            );
            if hwnd.0.is_null() {
                return Err(Error::from_thread());
            }
            let state = Box::new(State {
                handler: std::rc::Rc::from(handler),
            });
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);
            Ok(Window { hwnd })
        }
    }
}

/// An owned top-level window. Destroyed on drop.
pub struct Window {
    hwnd: HWND,
}

impl Window {
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn show_no_activate(&self) {
        // SAFETY: valid HWND.
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
        }
    }

    pub fn hide(&self) {
        // SAFETY: valid HWND.
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    pub fn is_visible(&self) -> bool {
        // SAFETY: valid HWND.
        unsafe { IsWindowVisible(self.hwnd).as_bool() }
    }

    /// Moves/resizes without activating or changing z-order.
    pub fn set_bounds(&self, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
        self.set_bounds_z(x, y, width, height, ZOrder::Keep, 0)
    }

    /// Re-run nonclient layout after installing a custom frame handler. `create` installs
    /// its handler after CreateWindowEx, so the first NCCALCSIZE used DefWindowProc.
    pub fn recalculate_frame(&self) -> Result<()> {
        self.set_bounds_z(
            0,
            0,
            0,
            0,
            ZOrder::Keep,
            swp::NOMOVE | swp::NOSIZE | swp::FRAMECHANGED,
        )
    }

    pub fn set_bounds_z(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        z: ZOrder,
        extra_flags: u32,
    ) -> Result<()> {
        let mut flags = swp::NOACTIVATE | swp::NOOWNERZORDER | extra_flags;
        let insert_after = match z {
            ZOrder::Keep => {
                flags |= swp::NOZORDER;
                None
            }
            ZOrder::Bottom => Some(HWND_BOTTOM),
            ZOrder::Top => Some(HWND_TOP),
            ZOrder::NoTopMost => Some(HWND_NOTOPMOST),
            ZOrder::After(h) => Some(h),
        };
        // SAFETY: valid HWND.
        unsafe { SetWindowPos(self.hwnd, insert_after, x, y, width, height, flags).ok() }
    }

    /// Outer rectangle in screen pixels.
    pub fn window_rect(&self) -> RECT {
        let mut rect = RECT::default();
        // SAFETY: valid HWND, out-pointer to a local.
        unsafe {
            let _ = GetWindowRect(self.hwnd, &mut rect);
        }
        rect
    }

    /// Client size in pixels.
    pub fn client_size(&self) -> (i32, i32) {
        let mut rect = RECT::default();
        // SAFETY: valid HWND, out-pointer to a local.
        unsafe {
            let _ = GetClientRect(self.hwnd, &mut rect);
        }
        (rect.right - rect.left, rect.bottom - rect.top)
    }

    pub fn dpi(&self) -> u32 {
        // SAFETY: valid HWND.
        unsafe { GetDpiForWindow(self.hwnd) }
    }

    pub fn set_timer(&self, id: usize, interval_ms: u32) {
        // SAFETY: valid HWND; timer messages arrive through the window procedure.
        unsafe {
            let _ = SetTimer(Some(self.hwnd), id, interval_ms, None);
        }
    }

    /// Timer that the scheduler may delay by up to `tolerance_ms` to batch wakeups.
    pub fn set_coalescable_timer(&self, id: usize, interval_ms: u32, tolerance_ms: u32) {
        // SAFETY: valid HWND.
        unsafe {
            let _ = SetCoalescableTimer(Some(self.hwnd), id, interval_ms, None, tolerance_ms);
        }
    }

    pub fn kill_timer(&self, id: usize) {
        // SAFETY: valid HWND.
        unsafe {
            let _ = KillTimer(Some(self.hwnd), id);
        }
    }

    pub fn post_message(&self, msg: u32, wparam: usize, lparam: isize) {
        // SAFETY: valid HWND.
        unsafe {
            let _ = PostMessageW(Some(self.hwnd), msg, WPARAM(wparam), LPARAM(lparam));
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        // SAFETY: destroying a window we created; IsWindow guards double destruction.
        unsafe {
            if IsWindow(Some(self.hwnd)).as_bool() {
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}

/// Posts a message to any window (thread-safe).
/// Constant per-window alpha for a `WS_EX_LAYERED` window (0 = invisible, 255 = opaque).
pub fn set_layered_alpha(hwnd: HWND, alpha: u8) {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA as u32);
    }
}

/// Synchronous `SendMessageW`.
pub fn send_message(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize {
    // SAFETY: plain FFI call.
    unsafe { SendMessageW(hwnd, msg, WPARAM(wparam), LPARAM(lparam)).0 }
}

pub fn post_message(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = PostMessageW(Some(hwnd), msg, WPARAM(wparam), LPARAM(lparam));
    }
}

/// Captures the mouse for `hwnd`.
pub fn set_capture(hwnd: HWND) {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = SetCapture(hwnd);
    }
}

/// The window that currently owns mouse capture on this thread (null if none).
pub fn get_capture() -> HWND {
    // SAFETY: plain FFI call.
    unsafe { GetCapture() }
}

/// True while `hwnd` still identifies a live window.
pub fn is_window(hwnd: HWND) -> bool {
    // SAFETY: plain FFI call; IsWindow tolerates stale handles.
    unsafe { IsWindow(Some(hwnd)).as_bool() }
}

pub fn release_capture() {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = ReleaseCapture();
    }
}

/// Requests `WM_MOUSELEAVE` for `hwnd`.
pub fn track_mouse_leave(hwnd: HWND) {
    let mut tme = TRACKMOUSEEVENT {
        cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
        dwFlags: TME_LEAVE as u32,
        hwndTrack: hwnd,
        dwHoverTime: 0,
    };
    // SAFETY: struct outlives the call.
    unsafe {
        let _ = TrackMouseEvent(&mut tme);
    }
}

/// Is the given virtual key currently down?
pub fn key_down(vk: u32) -> bool {
    // SAFETY: plain FFI call.
    unsafe { (GetKeyState(vk as i32) as u16 & 0x8000) != 0 }
}

/// Gives `hwnd` the keyboard focus (a no-op unless this thread owns the foreground queue).
pub fn set_focus(hwnd: HWND) {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = SetFocus(Some(hwnd));
    }
}

/// System drag threshold in pixels (SM_CXDRAG / SM_CYDRAG).
pub fn drag_threshold() -> (i32, i32) {
    // SAFETY: plain FFI calls.
    unsafe {
        (
            GetSystemMetrics(SM_CXDRAG).max(4),
            GetSystemMetrics(SM_CYDRAG).max(4),
        )
    }
}

/// Sets the cursor to a standard shape.
pub fn set_standard_cursor(kind: StandardCursor) {
    let id = match kind {
        StandardCursor::Arrow => IDC_ARROW,
        StandardCursor::SizeAll => IDC_SIZEALL,
        StandardCursor::SizeWE => IDC_SIZEWE,
        StandardCursor::Hand => IDC_HAND,
        StandardCursor::IBeam => IDC_IBEAM,
    };
    // SAFETY: loading a shared system cursor.
    unsafe {
        let c = LoadCursorW(None, id);
        let _ = SetCursor(Some(c));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandardCursor {
    Arrow,
    SizeAll,
    SizeWE,
    Hand,
    IBeam,
}

/// Requests `WM_NCMOUSELEAVE` for `hwnd` (non-client hover tracking).
pub fn track_mouse_leave_nc(hwnd: HWND) {
    let mut tme = TRACKMOUSEEVENT {
        cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
        dwFlags: (TME_LEAVE | TME_NONCLIENT) as u32,
        hwndTrack: hwnd,
        dwHoverTime: 0,
    };
    // SAFETY: struct outlives the call.
    unsafe {
        let _ = TrackMouseEvent(&mut tme);
    }
}

pub fn set_timer(hwnd: HWND, id: usize, interval_ms: u32) {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = SetTimer(Some(hwnd), id, interval_ms, None);
    }
}

pub fn kill_timer(hwnd: HWND, id: usize) {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = KillTimer(Some(hwnd), id);
    }
}

/// Outer size of any window in pixels.
pub fn window_rect_size(hwnd: HWND) -> (i32, i32) {
    let r = window_rect(hwnd);
    (r.right - r.left, r.bottom - r.top)
}

/// Brings a window to the foreground (activating it).
pub fn bring_to_front(hwnd: HWND) {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = SetForegroundWindow(hwnd);
    }
}

pub fn set_coalescable_timer(hwnd: HWND, id: usize, interval_ms: u32, tolerance_ms: u32) {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = SetCoalescableTimer(Some(hwnd), id, interval_ms, None, tolerance_ms);
    }
}

/// Posts WM_QUIT to the calling (UI) thread after `ms` milliseconds (test tooling).
pub fn quit_after(ms: u32) {
    // SAFETY: plain FFI call.
    let thread = unsafe { GetCurrentThreadId() };
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
        // SAFETY: posting to a thread id obtained above.
        unsafe {
            let _ = PostThreadMessageW(thread, WM_QUIT as u32, WPARAM(0), LPARAM(0));
        }
    });
}

/// A named mutex that guarantees a single running instance per session.
pub struct SingleInstance(HANDLE);

impl SingleInstance {
    /// Returns `None` when another instance already holds the mutex.
    pub fn acquire(name: &str) -> Option<Self> {
        let w = to_wide(name);
        // SAFETY: plain FFI calls; the handle is released on drop.
        unsafe {
            let h = CreateMutexW(None, false, PCWSTR(w.as_ptr()));
            if h.0.is_null() {
                return None;
            }
            if GetLastError() == ERROR_ALREADY_EXISTS as u32 {
                let _ = CloseHandle(h);
                return None;
            }
            Some(Self(h))
        }
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        // SAFETY: we own the handle.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Explicitly shows an interactive window, restoring it only if minimized.
/// SetWindowPos is intentional: a process started hidden must still be able to open
/// settings later, and reopening an already maximized window must preserve its size.
pub fn show_normal(hwnd: HWND) {
    const SW_RESTORE: i32 = 9;
    // SAFETY: plain FFI call.
    unsafe {
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            (SWP_SHOWWINDOW | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER) as u32,
        );
        let _ = SetForegroundWindow(hwnd);
    }
}

/// Outer rectangle of any window in screen pixels.
pub fn window_rect(hwnd: HWND) -> RECT {
    let mut rect = RECT::default();
    // SAFETY: out-pointer to a local; an invalid HWND just leaves it zeroed.
    unsafe {
        let _ = GetWindowRect(hwnd, &mut rect);
    }
    rect
}

/// Moves/resizes any window without activating it or changing its z-order.
pub fn set_window_bounds(hwnd: HWND, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
    // SAFETY: plain FFI call on a caller-supplied HWND.
    unsafe {
        SetWindowPos(
            hwnd,
            None,
            x,
            y,
            width,
            height,
            swp::NOACTIVATE | swp::NOZORDER | swp::NOOWNERZORDER,
        )
        .ok()
    }
}

/// Moves `hwnd` to the top of the non-topmost band without activating it.
pub fn set_window_bounds_z_top(hwnd: HWND) -> Result<()> {
    // SAFETY: plain FFI call.
    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOP),
            0,
            0,
            0,
            0,
            swp::NOACTIVATE | swp::NOMOVE | swp::NOSIZE | swp::NOOWNERZORDER,
        )
        .ok()
    }
}

/// Inside a `WM_GETMINMAXINFO` handler: sets the minimum tracking size in pixels.
///
/// # Safety
/// `lparam` must be the `MINMAXINFO*` of the message being processed.
pub unsafe fn minmaxinfo_set_min_track(lparam: isize, min_w: i32, min_h: i32) {
    let info = lparam as *mut MINMAXINFO;
    if !info.is_null() {
        // SAFETY: caller guarantees a live MINMAXINFO.
        unsafe {
            (*info).ptMinTrackSize = POINT { x: min_w, y: min_h };
        }
    }
}

/// Inside a `WM_WINDOWPOSCHANGING` handler: forbids z-order changes not initiated by us.
///
/// # Safety
/// `lparam` must be the `WINDOWPOS*` of a `WM_WINDOWPOSCHANGING` message being processed.
pub unsafe fn windowpos_deny_zorder_change(lparam: isize) {
    let pos = lparam as *mut WINDOWPOS;
    if !pos.is_null() {
        // SAFETY: caller guarantees a live WINDOWPOS.
        unsafe { (*pos).flags |= swp::NOZORDER };
    }
}

/// A pending pure move of a top-level window, in screen pixels.
///
/// # Safety
/// `lparam` must be the live `WINDOWPOS*` of `WM_WINDOWPOSCHANGING`.
pub unsafe fn windowpos_move_rect(lparam: isize, current: RECT) -> Option<RECT> {
    let pos = unsafe { (lparam as *const WINDOWPOS).as_ref()? };
    if pos.flags & swp::NOMOVE != 0
        || (pos.flags & swp::NOSIZE == 0
            && (pos.cx != current.right - current.left || pos.cy != current.bottom - current.top))
        || (pos.x == current.left && pos.y == current.top)
    {
        return None;
    }
    Some(RECT {
        left: pos.x,
        top: pos.y,
        right: pos.x + current.right - current.left,
        bottom: pos.y + current.bottom - current.top,
    })
}

/// Screen-space cursor position.
pub fn cursor_pos() -> POINT {
    let mut pt = POINT::default();
    // SAFETY: out-pointer to a local.
    unsafe {
        let _ = GetCursorPos(&mut pt);
    }
    pt
}

/// Converts a screen point to client coordinates of `hwnd`.
pub fn screen_to_client(hwnd: HWND, mut pt: POINT) -> POINT {
    // SAFETY: valid HWND, in/out pointer to a local.
    unsafe {
        let _ = ScreenToClient(hwnd, &mut pt);
    }
    pt
}

/// Converts a captured mouse message's client point back to screen coordinates.
pub fn client_to_screen(hwnd: HWND, mut pt: POINT) -> POINT {
    // SAFETY: valid HWND, in/out pointer to a local.
    unsafe {
        let _ = ClientToScreen(hwnd, &mut pt);
    }
    pt
}

/// Default `DefWindowProcW` dispatch for use inside handlers that want to pre/post-process.
pub fn def_window_proc(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize {
    // SAFETY: forwarding a message we are already processing.
    unsafe { DefWindowProcW(hwnd, msg, WPARAM(wparam), LPARAM(lparam)).0 }
}

/// Runs the thread's message loop until `WM_QUIT`.
pub fn run_message_loop() -> i32 {
    // SAFETY: standard message pump.
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        msg.wParam.0 as i32
    }
}

/// Dispatches pending messages without blocking. Returns `false` once `WM_QUIT` was seen.
pub fn pump_messages() -> bool {
    // SAFETY: standard non-blocking pump.
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE as u32).as_bool() {
            if msg.message == WM_QUIT as u32 {
                return false;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        true
    }
}

pub fn post_quit(code: i32) {
    // SAFETY: plain FFI call.
    unsafe { PostQuitMessage(code) };
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: GWLP_USERDATA holds either null or a `Box<State>` we installed. The handler is
    // an `Rc<dyn Fn>` cloned before the call, so re-entrant messages (and even destruction of
    // the window inside the handler) cannot invalidate the closure while it runs.
    unsafe {
        let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut State;
        let mut handled = None;

        // A shell context menu being tracked on this thread gets first pick of menu messages.
        if let Some(result) = crate::shell_menu::forward_menu_message(msg, wparam.0, lparam.0) {
            return LRESULT(result);
        }

        if !state.is_null() {
            let handler = (*state).handler.clone();
            handled = handler(hwnd, msg, wparam.0, lparam.0);
        }

        if msg == WM_NCDESTROY as u32 {
            let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut State;
            if !state.is_null() {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                drop(Box::from_raw(state));
            }
        }

        match handled {
            Some(result) => LRESULT(result),
            None => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

#[cfg(test)]
mod frame_tests {
    use super::*;

    #[test]
    fn custom_frame_has_no_native_inset_before_its_first_paint() -> Result<()> {
        let class = WindowClass::register("PecoFence.Test.CustomFrame", ClassOptions::default())?;
        let window = WindowBuilder::new(&class)
            .style(style::POPUP | style::THICKFRAME)
            .ex_style(style::EX_NOREDIRECTIONBITMAP | style::EX_TOOLWINDOW)
            .bounds(100, 100, 272, 282)
            .create(Box::new(|_, message, _, _| {
                (message == WM_NCCALCSIZE as u32).then_some(0)
            }))?;
        let before = window.client_size();
        window.recalculate_frame()?;
        println!(
            "native frame: before={before:?}, after={:?}",
            window.client_size()
        );
        assert_eq!(window.client_size(), (272, 282));
        for height in [72, 74, 80, 94, 96, 112, 178, 248, 282, 120, 72] {
            window.set_bounds(100, 100, 272, height)?;
            assert_eq!(window.client_size(), (272, height), "height={height}");
            let origin = screen_to_client(window.hwnd(), POINT { x: 100, y: 100 });
            assert_eq!((origin.x, origin.y), (0, 0), "client origin moved");
        }
        Ok(())
    }
}
