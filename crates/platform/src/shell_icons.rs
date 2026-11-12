//! Reading and toggling the "Show desktop icons" shell setting (plan §5.3).
//!
//! Reading uses `SHGetSetSettings(SSF_HIDEICONS)`. Toggling sends the same `WM_COMMAND 0x7402`
//! that Explorer's "View → Show desktop icons" menu item sends to `SHELLDLL_DefView`, so Explorer
//! persists the state itself (registry `HideIcons`) and the menu stays in sync.

use crate::bindings::*;
use crate::wide::to_wide;
use windows_core::PCWSTR;

const HIDE_ICONS_BIT: u32 = 1 << 12; // SHELLSTATE::fHideIcons
const DOUBLE_CLICK_BIT: u32 = 1 << 5; // SHELLSTATE::fDoubleClickInWebView
const SSF_DOUBLECLICKINWEBVIEW: u32 = 0x80;
const SHOW_INFO_TIP_BIT: u32 = 1 << 11; // SHELLSTATE::fShowInfoTip
const SSF_SHOWINFOTIP: u32 = 0x2000;
const CMD_TOGGLE_DESKTOP_ICONS: usize = 0x7402;
/// The desktop view's Edit › Undo (`SFVIDM_EDIT_UNDO`).
const CMD_UNDO: usize = 0x701B;
const LVM_GETSELECTEDCOUNT: u32 = 0x1000 + 50; // LVM_FIRST + 50
const LVM_GETHOTITEM: u32 = 0x1000 + 61; // LVM_FIRST + 61
const LVM_GETITEMCOUNT: u32 = 0x1000 + 4; // LVM_FIRST + 4
const LVM_GETITEMSTATE: u32 = 0x1000 + 44; // LVM_FIRST + 44
const LVIS_DROPHILITED: isize = 0x0008;

/// The first bit field of the shell's `SHELLSTATE`, with the flags in `mask` filled in.
fn shell_flags(mask: u32) -> u32 {
    let mut state = SHELLSTATEW::default();
    // SAFETY: SHELLSTATEA and SHELLSTATEW share the same layout; `state` outlives the call.
    unsafe {
        SHGetSetSettings(Some((&mut state as *mut SHELLSTATEW).cast()), mask, false);
    }
    state._bitfield1.0 as u32
}

/// True when the shell currently hides desktop icons.
pub fn desktop_icons_hidden() -> bool {
    shell_flags(SSF_HIDEICONS as u32) & HIDE_ICONS_BIT != 0
}

/// True when Explorer opens items with a single click (Folder Options › "Single-click to open
/// an item (point to select)"), as the desktop then does.
pub fn single_click_open() -> bool {
    shell_flags(SSF_DOUBLECLICKINWEBVIEW) & DOUBLE_CLICK_BIT == 0
}

/// True when Explorer shows a tip for the item under a resting pointer (Folder Options › View ›
/// "Show pop-up description for folder and desktop items", on by default).
pub fn show_info_tips() -> bool {
    shell_flags(SSF_SHOWINFOTIP) & SHOW_INFO_TIP_BIT != 0
}

/// Sends the toggle command to the desktop view (falls back to `Progman` when `def_view` is
/// null). Returns false if Explorer did not answer within the timeout.
pub fn toggle_desktop_icons(def_view: HWND, fallback: HWND) -> bool {
    let target = if def_view.0.is_null() {
        fallback
    } else {
        def_view
    };
    if target.0.is_null() {
        return false;
    }
    let mut result: usize = 0;
    // SAFETY: plain FFI call with a timeout; Explorer owns the target window.
    let ok = unsafe {
        SendMessageTimeoutW(
            target,
            WM_COMMAND as u32,
            WPARAM(CMD_TOGGLE_DESKTOP_ICONS),
            LPARAM(0),
            SMTO_ABORTIFHUNG as u32,
            1000,
            Some(&mut result),
        )
    };
    ok.0 != 0
}

/// Asks the desktop view to undo the newest file operation in Windows' own undo list: the
/// command its Edit › Undo and Ctrl+Z on the desktop run. The list is Explorer's and holds the
/// operations of every program that recorded them (`FOFX_ADDUNDORECORD`): a delete to the
/// Recycle Bin, a move, a copy. Posted, so a busy Explorer does not hold the caller, and the
/// view takes it with its icons hidden too. False when there is no view to ask.
pub fn undo_last_file_operation(def_view: HWND) -> bool {
    if def_view.0.is_null() {
        return false;
    }
    // SAFETY: plain FFI call; Explorer owns the target window.
    unsafe {
        PostMessageW(
            Some(def_view),
            WM_COMMAND as u32,
            WPARAM(CMD_UNDO),
            LPARAM(0),
        )
        .as_bool()
    }
}

/// How many desktop icons are selected in Explorer's list view under `def_view`; None when
/// there is no view or Explorer did not answer.
pub fn desktop_selected_count(def_view: HWND) -> Option<usize> {
    if def_view.0.is_null() {
        return None;
    }
    let class = to_wide("SysListView32");
    // SAFETY: the string outlives the call.
    let list =
        unsafe { FindWindowExW(Some(def_view), None, PCWSTR(class.as_ptr()), PCWSTR::null()) };
    if list.0.is_null() {
        return None;
    }
    let mut count: usize = 0;
    // SAFETY: plain FFI call with a timeout; Explorer owns the list view.
    let ok = unsafe {
        SendMessageTimeoutW(
            list,
            LVM_GETSELECTEDCOUNT,
            WPARAM(0),
            LPARAM(0),
            SMTO_ABORTIFHUNG as u32,
            200,
            Some(&mut count),
        )
    };
    (ok.0 != 0).then_some(count)
}

/// True when the pointer is on one of Windows' own desktop icons: `window` (what
/// `WindowFromPoint` found under it) is Explorer's desktop list view, and the icon it tracks for
/// its hover highlight is set. False for its empty space, any other window, or no answer.
pub fn desktop_icon_hovered(window: HWND) -> bool {
    if crate::desktop::class_name(window) != "SysListView32" {
        return false;
    }
    let mut item = usize::MAX;
    // SAFETY: plain FFI call with a timeout; Explorer owns the list view.
    unsafe {
        SendMessageTimeoutW(
            window,
            LVM_GETHOTITEM,
            WPARAM(0),
            LPARAM(0),
            SMTO_ABORTIFHUNG as u32,
            200,
            Some(&mut item),
        );
    }
    (item as isize) >= 0
}

/// How long [`desktop_icon_drop_target`] may ask Explorer in all. 1500 icons take about
/// 30 ms when Explorer is idle.
const DROP_TARGET_BUDGET: std::time::Duration = std::time::Duration::from_millis(300);

/// True while a drag is over one of Windows' own desktop icons that takes the drop (a folder,
/// a program, the Recycle Bin): `window` is Explorer's desktop list view, which marks that icon
/// drop-highlighted. False for its empty space, any other window, or no answer. Each icon is
/// asked on its own: on build 26300 `LVM_GETNEXTITEM` finds no drop-highlighted icon while
/// `LVM_GETITEMSTATE` shows it. The asking ends at the first question Explorer does not
/// answer within 200 ms and after `DROP_TARGET_BUDGET` in all (a busy Explorer answered each
/// one only after up to 200 ms, a freeze of icons x 200 ms): the drop then goes back to the
/// fence, as on empty desktop space.
pub fn desktop_icon_drop_target(window: HWND) -> bool {
    if crate::desktop::class_name(window) != "SysListView32" {
        return false;
    }
    let started = std::time::Instant::now();
    let ask = |msg: u32, wparam: usize, lparam: isize| {
        let mut out = 0usize;
        // SAFETY: plain FFI call with a timeout; Explorer owns the list view.
        let answered = unsafe {
            SendMessageTimeoutW(
                window,
                msg,
                WPARAM(wparam),
                LPARAM(lparam),
                SMTO_ABORTIFHUNG as u32,
                200,
                Some(&mut out),
            )
        };
        (answered.0 != 0).then_some(out)
    };
    let Some(count) = ask(LVM_GETITEMCOUNT, 0, 0).map(|n| n.min(4096)) else {
        tracing::info!("desktop icon under a drop: Explorer did not answer; not a drop target");
        return false;
    };
    for i in 0..count {
        match ask(LVM_GETITEMSTATE, i, LVIS_DROPHILITED) {
            Some(state) if state & LVIS_DROPHILITED as usize != 0 => return true,
            Some(_) if started.elapsed() < DROP_TARGET_BUDGET => {}
            answer => {
                tracing::info!(
                    asked = i + 1,
                    count,
                    answered = answer.is_some(),
                    elapsed_ms = started.elapsed().as_millis(),
                    "desktop icon under a drop: Explorer too slow to ask every icon; not a drop target"
                );
                return false;
            }
        }
    }
    false
}

/// Sets the hidden state, toggling only when it differs. Returns the state read back afterwards.
pub fn set_desktop_icons_hidden(hidden: bool, def_view: HWND, fallback: HWND) -> Option<bool> {
    if desktop_icons_hidden() != hidden && !toggle_desktop_icons(def_view, fallback) {
        return None;
    }
    Some(desktop_icons_hidden())
}

/// What [`set_desktop_icons_hidden_within`] found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconToggle {
    /// The state read back (after a toggle, or at once when it already matched).
    Done(bool),
    /// Explorer has not answered within the wait: the toggle is still out, and no other is
    /// sent until it is back.
    Waiting,
    /// No answer from Explorer (it hangs), or no window to send to.
    Failed,
}

/// A toggle sent from [`set_desktop_icons_hidden_within`]'s worker that Explorer has not
/// answered yet: 0 none, `TOGGLE_HIDE` or `TOGGLE_SHOW`. A second one sent meanwhile would undo
/// the first once both are processed.
static TOGGLE_OUT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
const TOGGLE_HIDE: u8 = 1;
const TOGGLE_SHOW: u8 = 2;

/// How long the worker waits for Explorer to process a toggle. Explorer hides 1400 icons in
/// about 0.3 s when idle; at sign-in, or with a large copy into the desktop, it took 20 s.
const TOGGLE_ANSWER_MS: u32 = 60_000;

/// [`set_desktop_icons_hidden`] on a worker thread, waiting `wait` at most for it. The UI
/// thread waited on Explorer for 20.6 s here (the 1000 ms timeout of the call did not hold
/// there), with every fence frozen. A toggle the other way still out is waited for first
/// (within `wait`); one the same way is not sent again. A toggle still out when the process
/// ends is lost: exit waits longer.
pub fn set_desktop_icons_hidden_within(
    hidden: bool,
    def_view: HWND,
    fallback: HWND,
    wait: std::time::Duration,
) -> IconToggle {
    use std::sync::atomic::Ordering;
    let started = std::time::Instant::now();
    let want = if hidden { TOGGLE_HIDE } else { TOGGLE_SHOW };
    loop {
        match TOGGLE_OUT.load(Ordering::Acquire) {
            0 => break,
            out if out == want || started.elapsed() >= wait => return IconToggle::Waiting,
            _ => std::thread::sleep(std::time::Duration::from_millis(10)),
        }
    }
    let target = if def_view.0.is_null() {
        fallback
    } else {
        def_view
    };
    if target.0.is_null() {
        return IconToggle::Failed;
    }
    let (answer, answered) = std::sync::mpsc::channel();
    let target = target.0 as isize;
    TOGGLE_OUT.store(want, Ordering::Release);
    let spawned = std::thread::Builder::new()
        .name("pecofence-icons-toggle".into())
        .stack_size(64 * 1024)
        .spawn(move || {
            let started = std::time::Instant::now();
            let target = HWND(target as *mut core::ffi::c_void);
            // SAFETY: plain FFI call with a timeout; Explorer owns the target window. No
            // SMTO_ABORTIFHUNG: a busy Explorer that pumps no messages for 5 s still processes
            // the message later, and the next toggle must wait for it.
            let send = |msg: u32, wparam: usize| unsafe {
                let mut result: usize = 0;
                SendMessageTimeoutW(
                    target,
                    msg,
                    WPARAM(wparam),
                    LPARAM(0),
                    SMTO_NORMAL as u32,
                    TOGGLE_ANSWER_MS,
                    Some(&mut result),
                )
                .0 != 0
            };
            // Explorer answers sent messages in the order they came: once it has answered this
            // one, a toggle sent before (by an attempt given up on, or by a PecoFence that has
            // quit since: its message still arrives) is done, and the state read next is the
            // one to act on. Without it the watchdog toggled the icons back to hidden.
            let state = if !send(WM_NULL as u32, 0) {
                None
            } else if desktop_icons_hidden() == hidden {
                Some(hidden)
            } else {
                send(WM_COMMAND as u32, CMD_TOGGLE_DESKTOP_ICONS).then(desktop_icons_hidden)
            };
            TOGGLE_OUT.store(0, Ordering::Release);
            tracing::info!(
                hidden,
                ?state,
                elapsed_ms = started.elapsed().as_millis(),
                "desktop icons toggle answered"
            );
            let _ = answer.send(state);
        });
    if spawned.is_err() {
        TOGGLE_OUT.store(0, Ordering::Release);
        return set_desktop_icons_hidden(hidden, def_view, fallback)
            .map_or(IconToggle::Failed, IconToggle::Done);
    }
    let outcome = match answered.recv_timeout(wait.saturating_sub(started.elapsed())) {
        Ok(Some(state)) => IconToggle::Done(state),
        Ok(None) => IconToggle::Failed,
        Err(_) => IconToggle::Waiting,
    };
    if outcome != IconToggle::Done(hidden) {
        tracing::info!(
            hidden,
            ?outcome,
            waited_ms = started.elapsed().as_millis(),
            "desktop icons toggle not done within the wait"
        );
    }
    outcome
}

/// Windows' own desktop icons where its desktop view has them (Explorer keeps them while
/// PecoFence hides them): 自由摆放 starts from these places when it has no cells of its own.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopIconPlaces {
    /// The spacing of Windows' icon grid as Explorer reports it: one column, one row, in px of
    /// the scale Explorer started at ([`Self::explorer_dpi`]).
    pub spacing: (i32, i32),
    /// The system DPI of Explorer's process (the scale it started at). After a scale change
    /// Explorer keeps reporting the spacing in that scale until it restarts, while the icon
    /// places follow the new scale (measured: 76 x 104 reported at 150 %, icons 114 x 156 apart).
    pub explorer_dpi: Option<u32>,
    /// Windows' 「自动排列图标」 is on.
    pub auto_arrange: bool,
    /// Each icon's parsing name (a file path, or `::{CLSID}` for This PC, the Recycle Bin, ...)
    /// and the top-left of its place (screen px).
    pub icons: Vec<(String, (i32, i32))>,
}

/// `IShellWindows::FindWindowSW` class of the desktop, and its "give me a dispatch" option.
const SWC_DESKTOP: i32 = 8;
const SWFO_NEEDDISPATCH: i32 = 1;
/// `IFolderView::ItemCount` / `Item`: every item of the view, not only the selected ones.
const SVGIO_ALLVIEW: u32 = 2;
windows_core::link!("user32.dll" "system" fn GetSystemDpiForProcess(hprocess: HANDLE) -> u32);

/// The system DPI of the process that owns the desktop window (Explorer).
fn explorer_system_dpi() -> Option<u32> {
    let pid = crate::desktop::window_pid(crate::desktop::shell_window());
    if pid == 0 {
        return None;
    }
    // SAFETY: plain FFI calls; the handle is closed before returning.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION as u32, false, pid);
        if process.0.is_null() {
            return None;
        }
        let dpi = GetSystemDpiForProcess(process);
        let _ = CloseHandle(process);
        (dpi > 0).then_some(dpi)
    }
}

/// `GetSystemMetrics`: the top-left of all monitors together.
const SM_XVIRTUALSCREEN: i32 = 76;
const SM_YVIRTUALSCREEN: i32 = 77;

/// Reads [`DesktopIconPlaces`] from Explorer's desktop view (the desktop's top-level browser,
/// its active view, `IFolderView`). Explorer answers these calls itself, so call it on a worker
/// thread with COM initialized. The error is the step that failed (no desktop yet at sign-in,
/// for one) and Windows' error.
pub fn desktop_icon_places() -> Result<DesktopIconPlaces, (&'static str, String)> {
    use windows_core::Interface;
    // SAFETY: COM calls on interfaces this function owns; every result is checked, and every
    // item ID list the view hands out is freed once.
    unsafe {
        let windows: IShellWindows = CoCreateInstance(&ShellWindows, None, CLSCTX_ALL as u32)
            .map_err(|e| ("ShellWindows", e.to_string()))?;
        // CSIDL_DESKTOP (0) as the location, as the shell's own samples ask for the desktop.
        let mut loc = VARIANT::default();
        (*loc.Anonymous.Anonymous).vt = VARTYPE(VT_I4 as u16);
        (*loc.Anonymous.Anonymous).Anonymous.lVal = 0;
        let root = VARIANT::default();
        let mut hwnd = 0i32;
        let desktop = windows
            .FindWindowSW(&loc, &root, SWC_DESKTOP, &mut hwnd, SWFO_NEEDDISPATCH)
            .map_err(|e| ("FindWindowSW", e.to_string()))?;
        let view = desktop
            .cast::<IServiceProvider>()
            .and_then(|sp| sp.QueryService::<IShellBrowser>(&SID_STopLevelBrowser))
            .and_then(|b| b.QueryActiveShellView())
            .map_err(|e| ("desktop view", e.to_string()))?;
        let folder_view = view
            .cast::<IFolderView>()
            .map_err(|e| ("IFolderView", e.to_string()))?;
        let mut spacing = POINT::default();
        folder_view
            .GetSpacing(&mut spacing)
            .ok()
            .map_err(|e| ("GetSpacing", e.to_string()))?;
        let auto_arrange = folder_view.GetAutoArrange() == S_OK;
        // The positions are the list view's client coordinates; its client area starts at the
        // top-left of all monitors together. (The view's own GetWindow is an input-synchronous
        // call, which a worker thread may not make: RPC_E_CANTCALLOUT_ININPUTSYNCCALL.)
        let mut origin = POINT {
            x: GetSystemMetrics(SM_XVIRTUALSCREEN),
            y: GetSystemMetrics(SM_YVIRTUALSCREEN),
        };
        if let Some(host) = crate::desktop::resolve_icon_host(crate::desktop::detect_generation())
            && !host.def_view.0.is_null()
        {
            let class = to_wide("SysListView32");
            let list = FindWindowExW(
                Some(host.def_view),
                None,
                PCWSTR(class.as_ptr()),
                PCWSTR::null(),
            );
            let mut at = POINT::default();
            if !list.0.is_null() && ClientToScreen(list, &mut at).as_bool() {
                origin = at;
            }
        }
        let names = SHGetDesktopFolder().map_err(|e| ("desktop folder", e.to_string()))?;
        let count = folder_view
            .ItemCount(SVGIO_ALLVIEW)
            .map_err(|e| ("ItemCount", e.to_string()))?;
        let mut icons = Vec::with_capacity(count.max(0) as usize);
        for i in 0..count {
            let Ok(pidl) = folder_view.Item(i) else {
                continue;
            };
            let at = folder_view.GetItemPosition(pidl);
            let mut name = STRRET::default();
            let mut parsing = windows_core::PWSTR::null();
            let path = if names
                .GetDisplayNameOf(pidl, SHGDN_FORPARSING, &mut name)
                .is_ok()
                && StrRetToStrW(&mut name, Some(pidl), &mut parsing).is_ok()
            {
                crate::shell::take_string(parsing)
            } else {
                String::new()
            };
            ILFree(Some(pidl));
            if let (Ok(p), false) = (at, path.is_empty()) {
                icons.push((path, (p.x + origin.x, p.y + origin.y)));
            }
        }
        Ok(DesktopIconPlaces {
            spacing: (spacing.x, spacing.y),
            explorer_dpi: explorer_system_dpi(),
            auto_arrange,
            icons,
        })
    }
}
