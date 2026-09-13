//! Monitor enumeration and DPI queries.

use crate::bindings::*;
use crate::wide::from_wide;
use windows_core::BOOL;

pub use crate::bindings::HMONITOR;

#[derive(Clone, Debug)]
pub struct MonitorInfo {
    pub handle: HMONITOR,
    /// GDI device name such as `\\.\DISPLAY1`.
    pub device_name: String,
    /// Full monitor rectangle in virtual-screen pixels.
    pub bounds: RECT,
    /// Work area (excluding taskbar) in virtual-screen pixels.
    pub work_area: RECT,
    /// Effective DPI (96 = 100%).
    pub dpi: u32,
    pub primary: bool,
}

impl MonitorInfo {
    pub fn scale(&self) -> f32 {
        self.dpi as f32 / USER_DEFAULT_SCREEN_DPI as f32
    }
}

pub fn enumerate() -> Vec<MonitorInfo> {
    let mut monitors: Vec<MonitorInfo> = Vec::new();
    // SAFETY: the callback only runs during this call and receives a pointer to `monitors`.
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(enum_proc),
            LPARAM(&mut monitors as *mut Vec<MonitorInfo> as isize),
        );
    }
    monitors
}

unsafe extern "system" fn enum_proc(
    monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    data: LPARAM,
) -> BOOL {
    // SAFETY: `data` is the `Vec` pointer passed by `enumerate` on this same thread.
    let monitors = unsafe { &mut *(data.0 as *mut Vec<MonitorInfo>) };
    if let Some(info) = query(monitor) {
        monitors.push(info);
    }
    BOOL(1)
}

/// Queries a single monitor handle.
pub fn query(monitor: HMONITOR) -> Option<MonitorInfo> {
    let mut info = MONITORINFOEXW {
        Base: MONITORINFO {
            cbSize: size_of::<MONITORINFOEXW>() as u32,
            ..Default::default()
        },
        szDevice: [0; 32],
    };
    // SAFETY: `info.cbSize` announces the EX layout; out-pointers reference locals.
    unsafe {
        if !GetMonitorInfoW(monitor, (&mut info as *mut MONITORINFOEXW).cast()).as_bool() {
            return None;
        }
        let mut dpi_x = USER_DEFAULT_SCREEN_DPI as u32;
        let mut dpi_y = dpi_x;
        let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
        Some(MonitorInfo {
            handle: monitor,
            device_name: from_wide(&info.szDevice),
            bounds: info.Base.rcMonitor,
            work_area: info.Base.rcWork,
            dpi: dpi_x,
            primary: info.Base.dwFlags & MONITORINFOF_PRIMARY as u32 != 0,
        })
    }
}

pub fn monitor_from_window(hwnd: HWND) -> HMONITOR {
    // SAFETY: plain FFI call.
    unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST as u32) }
}

pub fn monitor_from_point(x: i32, y: i32) -> HMONITOR {
    // SAFETY: plain FFI call.
    unsafe { MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST as u32) }
}

pub fn dpi_for_window(hwnd: HWND) -> u32 {
    // SAFETY: plain FFI call.
    unsafe { GetDpiForWindow(hwnd) }
}

pub fn system_dpi() -> u32 {
    // SAFETY: plain FFI call.
    unsafe { GetDpiForSystem() }
}
