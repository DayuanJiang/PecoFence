//! Desktop shell window discovery and z-order anchoring (plan §5.1 / §5.2).
//!
//! Two separate decisions, per the review:
//! * **OS generation** — detected once by capability, never by window shape.
//! * **Icon host** — the top-level window that owns `SHELLDLL_DefView`; re-resolved on every
//!   anchor / show-desktop check because Explorer restructures it at runtime on ≤23H2.

use crate::bindings::*;
use crate::wide::{from_wide, to_wide};
use windows_core::{PCSTR, PCWSTR, PWSTR, Result};

pub use crate::bindings::HWND as RawHwnd;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Generation {
    /// Windows 11 24H2+ : `SHELLDLL_DefView` lives permanently under `Progman`.
    Modern,
    /// ≤ 23H2 : Explorer moves DefView into a top-level `WorkerW` when the desktop is raised.
    Legacy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostKind {
    /// 24H2+: host is Progman (always).
    Progman,
    /// Legacy, right after logon: DefView still under Progman; expect a WorkerW takeover later.
    ProgmanNotYetRaised,
    /// Legacy, raised: a top-level WorkerW contains DefView.
    WorkerW,
    /// Fallback: root ancestor of the FolderView list view.
    ListViewRoot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconHost {
    pub host: HWND,
    pub def_view: HWND,
    pub kind: HostKind,
}

/// Windows build number via `RtlGetVersion` (not subject to manifest lies).
pub fn os_build() -> u32 {
    let mut info = OSVERSIONINFOW {
        dwOSVersionInfoSize: size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };
    // SAFETY: out-pointer to a properly sized local.
    unsafe {
        let _ = RtlGetVersion(&mut info);
    }
    info.dwBuildNumber
}

pub fn shell_window() -> HWND {
    // SAFETY: plain FFI call.
    unsafe { GetShellWindow() }
}

pub fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 128];
    // SAFETY: buffer length is passed; result is NUL-terminated within it.
    let len = unsafe { GetClassNameW(hwnd, PWSTR(buf.as_mut_ptr()), buf.len() as i32) };
    from_wide(&buf[..len.max(0) as usize])
}

pub fn window_pid(hwnd: HWND) -> u32 {
    let mut pid = 0u32;
    // SAFETY: out-pointer to a local.
    unsafe {
        let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    pid
}

pub fn ex_style(hwnd: HWND) -> u32 {
    // SAFETY: plain FFI call.
    unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 }
}

pub fn is_visible(hwnd: HWND) -> bool {
    // SAFETY: plain FFI call.
    unsafe { IsWindowVisible(hwnd).as_bool() }
}

pub fn is_iconic(hwnd: HWND) -> bool {
    // SAFETY: plain FFI call.
    unsafe { IsIconic(hwnd).as_bool() }
}

pub fn is_window(hwnd: HWND) -> bool {
    // SAFETY: plain FFI call.
    unsafe { IsWindow(Some(hwnd)).as_bool() }
}

pub fn root_ancestor(hwnd: HWND) -> HWND {
    // SAFETY: plain FFI call.
    unsafe { GetAncestor(hwnd, GA_ROOT as u32) }
}

pub fn foreground_window() -> HWND {
    // SAFETY: plain FFI call.
    unsafe { GetForegroundWindow() }
}

pub fn window_from_point(x: i32, y: i32) -> HWND {
    // SAFETY: plain FFI call.
    unsafe { WindowFromPoint(POINT { x, y }) }
}

/// Detects the desktop window generation by capability (Lively + Rainmeter probes).
pub fn detect_generation() -> Generation {
    let shell = shell_window();
    let noredir = !shell.0.is_null() && (ex_style(shell) & WS_EX_NOREDIRECTIONBITMAP as u32) != 0;
    // SAFETY: querying an export by name; both handles are process-lifetime.
    let topology_export = unsafe {
        let user32 = GetModuleHandleW(PCWSTR(to_wide("user32.dll").as_ptr()));
        !user32.0.is_null()
            && GetProcAddress(
                user32,
                PCSTR(c"GetCurrentMonitorTopologyId".as_ptr().cast()),
            )
            .is_some()
    };
    if noredir != topology_export {
        tracing::warn!(
            noredir,
            topology_export,
            "generation probes disagree; taking OR"
        );
    }
    if noredir || topology_export {
        Generation::Modern
    } else {
        Generation::Legacy
    }
}

fn find_child(parent: HWND, class: &str) -> HWND {
    let class = to_wide(class);
    // SAFETY: strings outlive the call.
    unsafe { FindWindowExW(Some(parent), None, PCWSTR(class.as_ptr()), PCWSTR::null()) }
}

/// Direct child windows of `parent` (class, rect, visible), for diagnostics.
pub fn child_windows(parent: HWND) -> Vec<(HWND, String, RECT, bool)> {
    let mut out = Vec::new();
    let mut prev: Option<HWND> = None;
    loop {
        // SAFETY: plain FFI enumeration.
        let child = unsafe { FindWindowExW(Some(parent), prev, PCWSTR::null(), PCWSTR::null()) };
        if child.0.is_null() {
            break;
        }
        let mut rect = RECT::default();
        // SAFETY: out-pointer to a local.
        unsafe {
            let _ = GetWindowRect(child, &mut rect);
        }
        out.push((child, class_name(child), rect, is_visible(child)));
        prev = Some(child);
        if out.len() > 64 {
            break;
        }
    }
    out
}

/// Enumerates top-level windows in z-order (top to bottom).
pub fn top_level_windows() -> Vec<HWND> {
    let mut out: Vec<HWND> = Vec::new();
    // SAFETY: the callback only runs during this call with a pointer to `out`.
    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(&mut out as *mut Vec<HWND> as isize));
    }
    out
}

unsafe extern "system" fn enum_proc(hwnd: HWND, data: LPARAM) -> windows_core::BOOL {
    // SAFETY: `data` is the Vec pointer from `top_level_windows`.
    unsafe { (*(data.0 as *mut Vec<HWND>)).push(hwnd) };
    windows_core::BOOL(1)
}

/// Resolves the current desktop icon host. Never cache the result across events.
pub fn resolve_icon_host(generation: Generation) -> Option<IconHost> {
    let progman = shell_window();
    if progman.0.is_null() || class_name(progman) != "Progman" {
        return None;
    }
    let def_view_under_progman = find_child(progman, "SHELLDLL_DefView");

    if generation == Generation::Modern {
        // 24H2+: DefView may be absent for a moment during Explorer restart; still anchor to
        // Progman.
        return Some(IconHost {
            host: progman,
            def_view: def_view_under_progman,
            kind: HostKind::Progman,
        });
    }

    if !def_view_under_progman.0.is_null() {
        return Some(IconHost {
            host: progman,
            def_view: def_view_under_progman,
            kind: HostKind::ProgmanNotYetRaised,
        });
    }

    let explorer_pid = window_pid(progman);
    for hwnd in top_level_windows() {
        if class_name(hwnd) != "WorkerW" || window_pid(hwnd) != explorer_pid {
            continue;
        }
        let def_view = find_child(hwnd, "SHELLDLL_DefView");
        if !def_view.0.is_null() {
            return Some(IconHost {
                host: hwnd,
                def_view,
                kind: HostKind::WorkerW,
            });
        }
    }

    // Last resort: locate the FolderView list view anywhere and take its root.
    let list_view = {
        let class = to_wide("SysListView32");
        let title = to_wide("FolderView");
        // SAFETY: strings outlive the call.
        unsafe { FindWindowExW(None, None, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr())) }
    };
    if !list_view.0.is_null() {
        let root = root_ancestor(list_view);
        return Some(IconHost {
            host: root,
            def_view: find_child(root, "SHELLDLL_DefView"),
            kind: HostKind::ListViewRoot,
        });
    }
    None
}

const ANCHOR_FLAGS: u32 = (SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER) as u32;

/// A foreign window that can actually occlude the desktop. Minimized terminal
/// windows and zero-sized pseudo-console hosts may keep WS_VISIBLE set.
pub fn has_visible_surface(hwnd: HWND) -> bool {
    if !is_visible(hwnd) || is_iconic(hwnd) || is_cloaked(hwnd) {
        return false;
    }
    let mut rect = RECT::default();
    // SAFETY: caller-supplied HWND and a local, correctly sized output rectangle.
    (unsafe { GetWindowRect(hwnd, &mut rect).as_bool() })
        && rect.right > rect.left
        && rect.bottom > rect.top
}

/// Inserts `hwnd` directly above `host` in the z-order (or at the very bottom if `host` is
/// already the bottom-most window).
pub fn insert_above(hwnd: HWND, host: HWND) -> Result<()> {
    // SAFETY: plain FFI calls on caller-supplied handles.
    unsafe {
        let mut above = GetWindow(host, GW_HWNDPREV as u32);
        // Nonvisual windows can sit directly above Progman. A minimized elevated
        // terminal may reject relative placement even though it paints no surface.
        while !above.0.is_null() && above != hwnd && !has_visible_surface(above) {
            above = GetWindow(above, GW_HWNDPREV as u32);
        }
        if above == hwnd {
            return Ok(());
        }
        let insert_after = if above.0.is_null() {
            HWND_BOTTOM
        } else {
            above
        };
        SetWindowPos(hwnd, Some(insert_after), 0, 0, 0, 0, ANCHOR_FLAGS).ok()
    }
}

/// The window directly above `hwnd` in the z-order (null HWND if none).
pub fn window_above(hwnd: HWND) -> HWND {
    // SAFETY: plain FFI call.
    unsafe { GetWindow(hwnd, GW_HWNDPREV as u32) }
}

/// Inserts `hwnd` directly below `reference` in the z-order.
pub fn insert_after(hwnd: HWND, reference: HWND) -> Result<()> {
    // SAFETY: plain FFI call.
    unsafe { SetWindowPos(hwnd, Some(reference), 0, 0, 0, 0, ANCHOR_FLAGS).ok() }
}

/// Moves `hwnd` into (or out of) the topmost band without activating it.
pub fn set_topmost(hwnd: HWND, on: bool) -> Result<()> {
    let after = if on { HWND_TOPMOST } else { HWND_NOTOPMOST };
    // SAFETY: plain FFI call.
    unsafe { SetWindowPos(hwnd, Some(after), 0, 0, 0, 0, ANCHOR_FLAGS).ok() }
}

/// Raises `hwnd` to the top of whichever band (topmost / normal) it is in.
pub fn raise_in_band(hwnd: HWND) -> Result<()> {
    // SAFETY: plain FFI call.
    unsafe { SetWindowPos(hwnd, Some(HWND_TOP), 0, 0, 0, 0, ANCHOR_FLAGS).ok() }
}

/// Pushes `hwnd` to the bottom of the non-topmost band.
pub fn send_to_bottom(hwnd: HWND) -> Result<()> {
    // SAFETY: plain FFI call.
    unsafe { SetWindowPos(hwnd, Some(HWND_BOTTOM), 0, 0, 0, 0, ANCHOR_FLAGS).ok() }
}

/// True when a window of class `class` exists *below* `host` in the z-order — i.e. Explorer
/// has raised the desktop above our (normally just-above-host) windows.
pub fn is_below(host: HWND, class: &str) -> bool {
    let class = to_wide(class);
    // SAFETY: strings outlive the call.
    let found = unsafe { FindWindowExW(None, Some(host), PCWSTR(class.as_ptr()), PCWSTR::null()) };
    !found.0.is_null()
}

/// The lowest visible `WS_EX_TOPMOST` window, i.e. the boundary between the topmost band and
/// the normal band. Inserting after it puts a window on top of all normal windows.
pub fn lowest_topmost_window() -> Option<HWND> {
    top_level_windows()
        .into_iter()
        .filter(|&h| is_visible(h) && (ex_style(h) & WS_EX_TOPMOST as u32) != 0)
        .last()
}

pub fn is_cloaked(hwnd: HWND) -> bool {
    let mut cloaked: u32 = 0;
    // SAFETY: out-pointer sized for a DWORD.
    unsafe {
        let _ = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED as u32,
            (&mut cloaked as *mut u32).cast(),
            size_of::<u32>() as u32,
        );
    }
    cloaked != 0
}

pub fn set_cloak(hwnd: HWND, cloak: bool) -> Result<()> {
    let value: i32 = cloak as i32;
    // SAFETY: value outlives the call.
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_CLOAK as u32,
            (&value as *const i32).cast(),
            size_of::<i32>() as u32,
        )
        .ok()
    }
}

pub fn hide_window(hwnd: HWND) {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

/// Injects a left double-click at screen coordinates (test tooling only).
pub fn send_double_click(x: i32, y: i32) {
    // SAFETY: synthesizes input for the current session.
    unsafe {
        let _ = SetCursorPos(x, y);
        for _ in 0..2 {
            mouse_event(MOUSEEVENTF_LEFTDOWN as u32, 0, 0, 0, 0);
            mouse_event(MOUSEEVENTF_LEFTUP as u32, 0, 0, 0, 0);
            std::thread::sleep(std::time::Duration::from_millis(60));
        }
    }
}

pub fn show_no_activate(hwnd: HWND) {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    }
}

/// Registers (or looks up) the `TaskbarCreated` broadcast message id.
pub fn taskbar_created_message() -> u32 {
    let name = to_wide("TaskbarCreated");
    // SAFETY: string outlives the call.
    unsafe { RegisterWindowMessageW(PCWSTR(name.as_ptr())) }
}

/// Injects a Win+D keystroke (test tooling only).
pub fn send_win_d() {
    const VK_D: u8 = 0x44;
    // SAFETY: plain FFI calls; synthesizes keyboard input for the current session.
    unsafe {
        keybd_event(VK_LWIN as u8, 0, 0, 0);
        keybd_event(VK_D, 0, 0, 0);
        keybd_event(VK_D, 0, KEYEVENTF_KEYUP as u32, 0);
        keybd_event(VK_LWIN as u8, 0, KEYEVENTF_KEYUP as u32, 0);
    }
}

/// Human-readable z-order dump around the fences, for spike logs.
pub fn describe_zorder(interesting: &[HWND], all_visible: bool) -> String {
    let mut lines = Vec::new();
    for (i, h) in top_level_windows().into_iter().enumerate() {
        let cls = class_name(h);
        let tagged = interesting.contains(&h);
        let is_shell = cls == "Progman" || (cls == "WorkerW" && is_visible(h));
        if tagged || is_shell || (all_visible && is_visible(h) && !is_cloaked(h)) {
            lines.push(format!(
                "{i:4} {:>18} {:#x} vis={} cloak={} iconic={}{}",
                cls,
                h.0 as usize,
                is_visible(h) as u8,
                is_cloaked(h) as u8,
                is_iconic(h) as u8,
                if tagged { "  <== ours" } else { "" }
            ));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod anchor_tests {
    use super::*;
    use crate::window::{ClassOptions, WindowBuilder, WindowClass, ZOrder, style, swp};

    #[test]
    fn desktop_placement_skips_hidden_popups() -> Result<()> {
        let class = WindowClass::register("PecoFence.Test.VisibleAnchor", ClassOptions::default())?;
        let make = || {
            WindowBuilder::new(&class)
                .style(style::POPUP)
                .ex_style(style::EX_TOOLWINDOW | style::EX_NOACTIVATE)
                .bounds(-32000, -32000, 1, 1)
                .create(Box::new(|_, _, _, _| None))
        };
        let host = make()?;
        let reference = make()?;
        let hidden_popup = make()?;
        let target = make()?;
        host.show_no_activate();
        reference.show_no_activate();
        target.show_no_activate();
        hidden_popup.set_bounds_z(
            0,
            0,
            0,
            0,
            ZOrder::After(reference.hwnd()),
            swp::NOMOVE | swp::NOSIZE,
        )?;
        host.set_bounds_z(
            0,
            0,
            0,
            0,
            ZOrder::After(hidden_popup.hwnd()),
            swp::NOMOVE | swp::NOSIZE,
        )?;
        assert_eq!(window_above(host.hwnd()), hidden_popup.hwnd());
        insert_above(target.hwnd(), host.hwnd())?;
        assert_eq!(window_above(target.hwnd()), reference.hwnd());
        // Already in the correct visible position: do not try inserting after itself.
        insert_above(target.hwnd(), host.hwnd())?;
        assert_eq!(window_above(target.hwnd()), reference.hwnd());
        Ok(())
    }

    #[test]
    fn desktop_placement_skips_empty_minimized_and_cloaked_windows() -> Result<()> {
        let class =
            WindowClass::register("PecoFence.Test.NonvisualAnchor", ClassOptions::default())?;
        let make = || {
            WindowBuilder::new(&class)
                .style(style::POPUP)
                .ex_style(style::EX_TOOLWINDOW | style::EX_NOACTIVATE)
                .bounds(-32000, -32000, 1, 1)
                .create(Box::new(|_, _, _, _| None))
        };
        for mode in ["empty", "minimized", "cloaked"] {
            let host = make()?;
            let reference = make()?;
            let phantom = make()?;
            let target = make()?;
            for w in [&host, &reference, &phantom, &target] {
                w.show_no_activate();
            }
            match mode {
                "empty" => phantom.set_bounds(-32000, -32000, 0, 0)?,
                "minimized" => {
                    // SAFETY: our own off-screen fixture, without activation.
                    unsafe {
                        windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                            phantom.hwnd().0,
                            windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWMINNOACTIVE,
                        );
                    }
                }
                "cloaked" => set_cloak(phantom.hwnd(), true)?,
                _ => unreachable!(),
            }
            assert!(!has_visible_surface(phantom.hwnd()), "{mode}");
            phantom.set_bounds_z(
                0,
                0,
                0,
                0,
                ZOrder::After(reference.hwnd()),
                swp::NOMOVE | swp::NOSIZE,
            )?;
            host.set_bounds_z(
                0,
                0,
                0,
                0,
                ZOrder::After(phantom.hwnd()),
                swp::NOMOVE | swp::NOSIZE,
            )?;
            insert_above(target.hwnd(), host.hwnd())?;
            assert_eq!(window_above(target.hwnd()), reference.hwnd(), "{mode}");
        }
        Ok(())
    }
}
