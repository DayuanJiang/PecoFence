//! Raw mouse input sink, registered only while the desktop is in the foreground (plan §5.4).

use crate::bindings::*;
use windows_core::Result;

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
    Some(MouseButtons {
        left_down: flags & RI_MOUSE_LEFT_BUTTON_DOWN as u32 != 0,
        left_up: flags & RI_MOUSE_LEFT_BUTTON_UP as u32 != 0,
    })
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
