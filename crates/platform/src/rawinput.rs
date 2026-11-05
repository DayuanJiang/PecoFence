//! Raw mouse input sink, registered only while the desktop is in the foreground (plan §5.4).

use crate::bindings::*;
use windows_core::Result;

/// Presses and releases of the primary button (the right one when the buttons are swapped).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseButtons {
    pub left_down: bool,
    pub left_up: bool,
}

/// Registers `hwnd` to receive `WM_INPUT` for all mouse devices, even when not in the foreground.
pub fn register_mouse_sink(hwnd: HWND) -> Result<()> {
    let device = RAWINPUTDEVICE {
        usUsagePage: HID_USAGE_PAGE_GENERIC.0,
        usUsage: HID_USAGE_GENERIC_MOUSE.0,
        dwFlags: RIDEV_INPUTSINK as u32,
        hwndTarget: hwnd,
    };
    // SAFETY: the slice outlives the call.
    unsafe { RegisterRawInputDevices(&[device], size_of::<RAWINPUTDEVICE>() as u32).ok() }
}

/// Stops raw mouse delivery for this process.
pub fn unregister_mouse_sink() -> Result<()> {
    let device = RAWINPUTDEVICE {
        usUsagePage: HID_USAGE_PAGE_GENERIC.0,
        usUsage: HID_USAGE_GENERIC_MOUSE.0,
        dwFlags: RIDEV_REMOVE as u32,
        hwndTarget: HWND::default(),
    };
    // SAFETY: the slice outlives the call.
    unsafe { RegisterRawInputDevices(&[device], size_of::<RAWINPUTDEVICE>() as u32).ok() }
}

/// Decodes a `WM_INPUT` message's button flags. Returns `None` for non-mouse input or errors.
pub fn mouse_buttons_from_wm_input(lparam: isize) -> Option<MouseButtons> {
    let handle = HRAWINPUT(lparam as *mut core::ffi::c_void);
    // SAFETY: RAWINPUT is plain old data; all-zero is a valid (empty) packet.
    let mut raw: RAWINPUT = unsafe { core::mem::zeroed() };
    let mut size = size_of::<RAWINPUT>() as u32;
    // SAFETY: `raw` is large enough for a mouse packet; `size` is in/out.
    let copied = unsafe {
        GetRawInputData(
            handle,
            RID_INPUT as u32,
            Some((&mut raw as *mut RAWINPUT).cast()),
            &mut size,
            size_of::<RAWINPUTHEADER>() as u32,
        )
    };
    if copied == u32::MAX || raw.header.dwType != RIM_TYPEMOUSE as u32 {
        return None;
    }
    // SAFETY: dwType says this is a mouse packet, so the `mouse` union member is valid; the
    // button flags live in the low word of `ulButtons`.
    let flags = unsafe { raw.data.mouse.Anonymous.ulButtons } & 0xffff;
    // Raw input reports physical buttons.
    let (down, up) = if buttons_swapped() {
        (RI_MOUSE_RIGHT_BUTTON_DOWN, RI_MOUSE_RIGHT_BUTTON_UP)
    } else {
        (RI_MOUSE_LEFT_BUTTON_DOWN, RI_MOUSE_LEFT_BUTTON_UP)
    };
    Some(MouseButtons {
        left_down: flags & down as u32 != 0,
        left_up: flags & up as u32 != 0,
    })
}

/// Registers `hwnd` to receive `WM_INPUT` for keyboards even when not in the foreground (双击
/// Ctrl 打开搜索). Only listens: the keys still reach the window they were typed into.
pub fn register_keyboard_sink(hwnd: HWND) -> Result<()> {
    let device = RAWINPUTDEVICE {
        usUsagePage: HID_USAGE_PAGE_GENERIC.0,
        usUsage: HID_USAGE_GENERIC_KEYBOARD.0,
        dwFlags: RIDEV_INPUTSINK as u32,
        hwndTarget: hwnd,
    };
    // SAFETY: the slice outlives the call.
    unsafe { RegisterRawInputDevices(&[device], size_of::<RAWINPUTDEVICE>() as u32).ok() }
}

/// Stops raw keyboard delivery for this process.
pub fn unregister_keyboard_sink() -> Result<()> {
    let device = RAWINPUTDEVICE {
        usUsagePage: HID_USAGE_PAGE_GENERIC.0,
        usUsage: HID_USAGE_GENERIC_KEYBOARD.0,
        dwFlags: RIDEV_REMOVE as u32,
        hwndTarget: HWND::default(),
    };
    // SAFETY: the slice outlives the call.
    unsafe { RegisterRawInputDevices(&[device], size_of::<RAWINPUTDEVICE>() as u32).ok() }
}

/// A key of a `WM_INPUT` keyboard packet: its virtual key and whether it went up.
pub fn key_from_wm_input(lparam: isize) -> Option<(u16, bool)> {
    let handle = HRAWINPUT(lparam as *mut core::ffi::c_void);
    // SAFETY: RAWINPUT is plain old data; all-zero is a valid (empty) packet.
    let mut raw: RAWINPUT = unsafe { core::mem::zeroed() };
    let mut size = size_of::<RAWINPUT>() as u32;
    // SAFETY: `raw` is large enough for a keyboard packet; `size` is in/out.
    let copied = unsafe {
        GetRawInputData(
            handle,
            RID_INPUT as u32,
            Some((&mut raw as *mut RAWINPUT).cast()),
            &mut size,
            size_of::<RAWINPUTHEADER>() as u32,
        )
    };
    if copied == u32::MAX || raw.header.dwType != RIM_TYPEKEYBOARD as u32 {
        return None;
    }
    // SAFETY: dwType says this is a keyboard packet, so the `keyboard` member is valid.
    let kb = unsafe { raw.data.keyboard };
    Some((kb.VKey, kb.Flags as u32 & RI_KEY_BREAK as u32 != 0))
}

thread_local! {
    static MOUSE_USED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Notes mouse button presses and wheel turns while it lives (a low-level mouse hook on this
/// thread, which must pump messages). Used only while Ctrl is held, so Ctrl+click or Ctrl+wheel
/// is not taken for a tap of Ctrl.
pub struct MouseWatch(HHOOK);

impl MouseWatch {
    pub fn start() -> Option<Self> {
        MOUSE_USED.with(|m| m.set(false));
        // SAFETY: a low-level hook needs no module handle; the procedure only sets a flag.
        let hook = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_watch_proc), None, 0) };
        (!hook.0.is_null()).then_some(Self(hook))
    }

    /// A button was pressed or the wheel turned since `start`.
    pub fn used(&self) -> bool {
        MOUSE_USED.with(|m| m.get())
    }
}

impl Drop for MouseWatch {
    fn drop(&mut self) {
        // SAFETY: balances SetWindowsHookExW.
        unsafe {
            let _ = UnhookWindowsHookEx(self.0);
        }
    }
}

unsafe extern "system" fn mouse_watch_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let m = wparam.0 as u32;
    if code >= 0
        && [
            WM_LBUTTONDOWN as u32,
            WM_RBUTTONDOWN as u32,
            WM_MBUTTONDOWN as u32,
            WM_XBUTTONDOWN as u32,
            WM_MOUSEWHEEL as u32,
            WM_MOUSEHWHEEL as u32,
        ]
        .contains(&m)
    {
        MOUSE_USED.with(|u| u.set(true));
    }
    // SAFETY: passes the event on unchanged.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Is the primary mouse button the right one (Settings › Mouse › Primary button)?
pub fn buttons_swapped() -> bool {
    // SAFETY: plain FFI call.
    unsafe { GetSystemMetrics(SM_SWAPBUTTON) != 0 }
}

/// System double-click interval in milliseconds.
pub fn double_click_time_ms() -> u32 {
    // SAFETY: plain FFI call.
    unsafe { GetDoubleClickTime() }
}

/// Half the system double-click rectangle (pixels) — the max distance between the two clicks.
pub fn double_click_slop_px() -> (i32, i32) {
    // SAFETY: plain FFI calls.
    unsafe {
        (
            GetSystemMetrics(SM_CXDOUBLECLK) / 2,
            GetSystemMetrics(SM_CYDOUBLECLK) / 2,
        )
    }
}
