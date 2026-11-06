//! Open / Save dialogs of other programs and Explorer's folders, worked from outside their
//! processes (对话框跳转). Nothing is loaded into the other program: the dialog's own address
//! bar takes the path, and Explorer reports its folder through the documented ShellWindows
//! object.

use crate::bindings::*;
use crate::desktop::{class_name, is_visible};
use crate::wide::to_wide;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use windows_core::{Interface, PCWSTR};

/// How a jump ended, for the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpOutcome {
    /// The address bar shows the new folder.
    Done,
    /// The keys were sent but the address bar did not change in time (a slow network folder,
    /// or the dialog refused the path).
    Unconfirmed,
    /// No address bar in this dialog (an old-style dialog).
    NoAddressBar,
    /// The address bar did not turn into a text box after the click.
    NoAddressEdit,
    /// The dialog already shows the folder: nothing was sent.
    AlreadyThere,
    /// The folder's path holds `%NAME%` of an environment variable, which the address bar
    /// would expand (`%OS%` becomes `Windows_NT`) and then fail to find: nothing was sent.
    VariableInPath,
}

/// True when the address text (`Address: C:\Users\me\Documents`) shows `dir`.
fn shows_folder(address: &str, dir: &Path) -> bool {
    let dir = dir.to_string_lossy().trim_end_matches('\\').to_lowercase();
    let address = address.trim_end_matches('\\').to_lowercase();
    if dir.is_empty() || !address.ends_with(&dir) {
        return false;
    }
    let head = &address[..address.len() - dir.len()];
    head.is_empty() || head.ends_with([' ', ':', '\u{FF1A}'])
}

/// True when the dialog's address bar shows `dir`.
pub fn dialog_shows(dlg: HWND, dir: &Path) -> bool {
    dialog_address(dlg).is_some_and(|a| shows_folder(&a, dir))
}

/// True when part of `path` between two `%` signs names an environment variable.
fn expands_variables(path: &str) -> bool {
    let parts: Vec<&str> = path.split('%').collect();
    parts.len() > 2
        && parts[1..parts.len() - 1]
            .iter()
            .any(|name| !name.is_empty() && std::env::var_os(name).is_some())
}

/// All descendants of `parent` (EnumChildWindows walks the whole tree).
fn descendants(parent: HWND) -> Vec<HWND> {
    unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> windows_core::BOOL {
        // SAFETY: lparam is the Vec passed below, alive for the whole enumeration.
        let list = unsafe { &mut *(lparam.0 as *mut Vec<HWND>) };
        list.push(hwnd);
        windows_core::BOOL(1)
    }
    let mut list: Vec<HWND> = Vec::new();
    // SAFETY: the callback only pushes into `list`, which outlives the call.
    unsafe {
        let _ = EnumChildWindows(
            Some(parent),
            Some(collect),
            LPARAM(&mut list as *mut Vec<HWND> as isize),
        );
    }
    list
}

fn child_of_class(parent: HWND, class: &str) -> Option<HWND> {
    descendants(parent)
        .into_iter()
        .find(|&h| class_name(h) == class)
}

fn window_text(hwnd: HWND) -> String {
    let mut buf = vec![0u16; 1024];
    let mut copied = 0usize;
    // SAFETY: WM_GETTEXT with a buffer of the given length; the system marshals it across
    // processes. A hung window times out.
    unsafe {
        let _ = SendMessageTimeoutW(
            hwnd,
            WM_GETTEXT as u32,
            WPARAM(buf.len()),
            LPARAM(buf.as_mut_ptr() as isize),
            SMTO_ABORTIFHUNG as u32,
            500,
            Some(&mut copied),
        );
    }
    String::from_utf16_lossy(&buf[..copied.min(buf.len())])
}

/// The breadcrumb toolbar of a dialog's address bar ("Address: C:\Users").
fn breadcrumbs(band: HWND) -> Option<HWND> {
    let parent = child_of_class(band, "Breadcrumb Parent")?;
    child_of_class(parent, "ToolbarWindow32")
}

/// A common file dialog (Open, Save As, Select Folder): a `#32770` window with an Explorer
/// address bar. Old-style dialogs (no address bar) are left alone.
pub fn is_file_dialog(hwnd: HWND) -> bool {
    !hwnd.0.is_null()
        && class_name(hwnd) == "#32770"
        && child_of_class(hwnd, "Address Band Root").is_some()
        && child_of_class(hwnd, "SHELLDLL_DefView").is_some()
}

/// An Explorer folder window.
pub fn is_explorer(hwnd: HWND) -> bool {
    !hwnd.0.is_null() && matches!(class_name(hwnd).as_str(), "CabinetWClass" | "ExploreWClass")
}

/// The topmost Explorer folder window, if any is open.
pub fn topmost_explorer() -> Option<HWND> {
    let class = to_wide("CabinetWClass");
    // SAFETY: plain FFI call with a NUL-terminated class name.
    let h = unsafe { FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) };
    (!h.0.is_null() && is_visible(h)).then_some(h)
}

/// What the dialog's address bar shows ("Address: Documents"), for the log.
pub fn dialog_address(dlg: HWND) -> Option<String> {
    let band = child_of_class(dlg, "Address Band Root")?;
    breadcrumbs(band).map(window_text)
}

/// Waits (up to `max`) until no mouse button is held: a click on the dialog's title bar that
/// brought it to the front runs the window's move loop until the button comes up, and a click
/// posted to the address bar meanwhile is lost.
fn wait_buttons_up(max: Duration) {
    let start = Instant::now();
    // SAFETY: plain FFI calls reading the global button state.
    let held = || unsafe {
        [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON]
            .iter()
            .any(|&b| GetAsyncKeyState(b) < 0)
    };
    while held() && start.elapsed() < max {
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The control of `dlg`'s thread that has the keyboard focus.
fn focused_control(dlg: HWND) -> Option<HWND> {
    // SAFETY: plain FFI calls with a sized out-structure.
    unsafe {
        let thread = GetWindowThreadProcessId(dlg, None);
        let mut info = GUITHREADINFO {
            cbSize: size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        (GetGUIThreadInfo(thread, &mut info).as_bool() && !info.hwndFocus.0.is_null())
            .then_some(info.hwndFocus)
    }
}

/// Gives the keyboard focus back to `control` (in `dlg`'s thread): after Enter the address bar
/// keeps it, and the user expects the file name box (or the list) they were in.
/// The selection of an edit control (EM_GETSEL with no pointers answers in its result).
fn edit_selection(edit: HWND) -> Option<(usize, usize)> {
    if class_name(edit) != "Edit" {
        return None;
    }
    let mut packed = 0usize;
    // SAFETY: a plain message whose answer is the packed selection; times out when hung.
    unsafe {
        let _ = SendMessageTimeoutW(
            edit,
            EM_GETSEL as u32,
            WPARAM(0),
            LPARAM(0),
            SMTO_ABORTIFHUNG as u32,
            500,
            Some(&mut packed),
        );
    }
    Some((packed & 0xFFFF, (packed >> 16) & 0xFFFF))
}

fn restore_focus(dlg: HWND, control: HWND) {
    // SAFETY: the input states are joined only for the SetFocus call and detached right after.
    unsafe {
        let theirs = GetWindowThreadProcessId(dlg, None);
        let ours = GetCurrentThreadId();
        if theirs == 0 || !AttachThreadInput(ours, theirs, true).as_bool() {
            return;
        }
        let _ = SetFocus(Some(control));
        let _ = AttachThreadInput(ours, theirs, false);
    }
}

fn click_address_bar(crumbs: HWND, band: HWND) -> Option<HWND> {
    let mut rect = RECT::default();
    // SAFETY: out-pointer to a local.
    unsafe {
        let _ = GetClientRect(crumbs, &mut rect);
    }
    // GetClientRect answers in the dialog's own pixels, the ones its mouse messages use (also
    // for a program that is not DPI aware on a scaled screen: tested at 150 %).
    let x = (rect.right - 4).max(0);
    let y = rect.bottom / 2;
    let xy = LPARAM(((y as isize) << 16) | (x as isize & 0xFFFF));
    // SAFETY: plain FFI call.
    let dpi = unsafe { GetDpiForWindow(crumbs) };
    tracing::info!(dpi, x, y, "dialog: click on the address bar");
    // SAFETY: posting mouse messages to another process's window; no pointers involved.
    unsafe {
        let _ = PostMessageW(
            Some(crumbs),
            WM_LBUTTONDOWN as u32,
            WPARAM(MK_LBUTTON as usize),
            xy,
        );
        let _ = PostMessageW(Some(crumbs), WM_LBUTTONUP as u32, WPARAM(0), xy);
    }
    // The text box is created (or shown) once the dialog has handled the click.
    let start = Instant::now();
    loop {
        if let Some(e) = descendants(band)
            .into_iter()
            .find(|&h| class_name(h) == "Edit" && is_visible(h))
        {
            return Some(e);
        }
        if start.elapsed() > Duration::from_millis(800) {
            return None;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// Sends dialog `dlg` to `dir`: clicks the empty end of its breadcrumb bar (posted, the real
/// mouse does not move) so the address text box appears, puts the path in and presses Enter
/// there. The typed file name of a Save dialog stays, and a Select Folder dialog stays open
/// (Enter in its folder box would choose the folder). Blocks up to about two seconds: call it
/// on a worker thread.
pub fn jump(dlg: HWND, dir: &Path) -> JumpOutcome {
    let Some(band) = child_of_class(dlg, "Address Band Root") else {
        return JumpOutcome::NoAddressBar;
    };
    let Some(crumbs) = breadcrumbs(band) else {
        return JumpOutcome::NoAddressBar;
    };
    let before = window_text(crumbs);
    // The address text would not change to confirm a jump to where the dialog already is.
    if shows_folder(&before, dir) {
        return JumpOutcome::AlreadyThere;
    }
    if expands_variables(&dir.to_string_lossy()) {
        return JumpOutcome::VariableInPath;
    }
    wait_buttons_up(Duration::from_secs(3));
    let focus = focused_control(dlg).filter(|&f| !descendants(band).contains(&f));
    // Focusing an edit selects all of it: the caret goes back where it was afterwards, so
    // typing does not replace a Save dialog's file name.
    let selection = focus.and_then(edit_selection);
    // A second click when the first came while the dialog was still busy (activating).
    let Some(edit) = click_address_bar(crumbs, band).or_else(|| click_address_bar(crumbs, band))
    else {
        return JumpOutcome::NoAddressEdit;
    };
    let text = to_wide(&dir.to_string_lossy());
    // SAFETY: WM_SETTEXT with a NUL-terminated buffer that outlives the call; the system
    // copies it into the other process. Enter is posted as plain key messages.
    unsafe {
        let _ = SendMessageTimeoutW(
            edit,
            WM_SETTEXT as u32,
            WPARAM(0),
            LPARAM(text.as_ptr() as isize),
            SMTO_ABORTIFHUNG as u32,
            1000,
            None,
        );
        let _ = PostMessageW(
            Some(edit),
            WM_KEYDOWN as u32,
            WPARAM(VK_RETURN as usize),
            LPARAM(0x001C_0001),
        );
        let _ = PostMessageW(
            Some(edit),
            WM_KEYUP as u32,
            WPARAM(VK_RETURN as usize),
            LPARAM(0xC01C_0001u32 as i32 as isize),
        );
    }
    let start = Instant::now();
    let mut outcome = JumpOutcome::Unconfirmed;
    while start.elapsed() < Duration::from_millis(1500) {
        std::thread::sleep(Duration::from_millis(50));
        let now = window_text(crumbs);
        if !now.is_empty() && now != before {
            outcome = JumpOutcome::Done;
            break;
        }
    }
    if let Some(f) = focus.filter(|&f| crate::desktop::is_window(f)) {
        restore_focus(dlg, f);
        if let Some((start, end)) = selection {
            // SAFETY: a plain message with numbers only.
            unsafe {
                let _ = SendMessageTimeoutW(
                    f,
                    EM_SETSEL as u32,
                    WPARAM(start),
                    LPARAM(end as isize),
                    SMTO_ABORTIFHUNG as u32,
                    500,
                    None,
                );
            }
        }
    }
    // Unconfirmed: the folder may have been the one already shown.
    outcome
}

/// The file-system folder an Explorer window shows: the active tab's on Windows 11 (the first
/// tab window in z-order). `None` for places with no folder behind them (This PC, Home, a
/// library) or when Explorer does not answer; the reason goes to the log. COM must be
/// initialized on the calling thread.
pub fn explorer_folder(top: HWND) -> Option<PathBuf> {
    match explorer_folder_inner(top) {
        Ok(p) => Some(p),
        Err(why) => {
            tracing::info!(top = top.0 as isize, why, "Explorer folder not read");
            None
        }
    }
}

fn explorer_folder_inner(top: HWND) -> Result<PathBuf, String> {
    let tab_class = to_wide("ShellTabWindowClass");
    // SAFETY: plain FFI call; a missing tab window is a null handle.
    let active_tab =
        unsafe { FindWindowExW(Some(top), None, PCWSTR(tab_class.as_ptr()), PCWSTR::null()) };
    let mut seen = Vec::new();
    // SAFETY: COM calls on interfaces this function owns; every result is checked.
    unsafe {
        let windows: IShellWindows = CoCreateInstance(&ShellWindows, None, CLSCTX_ALL as u32)
            .map_err(|e| format!("ShellWindows: {e}"))?;
        let count = windows.Count().map_err(|e| format!("Count: {e}"))?;
        for i in 0..count {
            let mut index = VARIANT::default();
            (*index.Anonymous.Anonymous).vt = VARTYPE(VT_I4 as u16);
            (*index.Anonymous.Anonymous).Anonymous.lVal = i;
            let item = match windows.Item(&index) {
                Ok(item) => item,
                Err(e) => {
                    seen.push(format!("{i}: Item {e}"));
                    continue;
                }
            };
            let browser = match item
                .cast::<IServiceProvider>()
                .and_then(|sp| sp.QueryService::<IShellBrowser>(&SID_STopLevelBrowser))
            {
                Ok(b) => b,
                Err(e) => {
                    seen.push(format!("{i}: IShellBrowser {e}"));
                    continue;
                }
            };
            let hwnd = match browser.cast::<IOleWindow>().and_then(|w| w.GetWindow()) {
                Ok(h) => h,
                Err(e) => {
                    seen.push(format!("{i}: GetWindow {e}"));
                    continue;
                }
            };
            // Each tab is its own item; the window is the tab (Windows 11) or the frame.
            if hwnd != top && (active_tab.0.is_null() || hwnd != active_tab) {
                seen.push(format!("{i}: window {}", hwnd.0 as isize));
                continue;
            }
            let view = browser
                .QueryActiveShellView()
                .map_err(|e| format!("QueryActiveShellView: {e}"))?;
            let folder: IShellItem = view
                .cast::<IFolderView>()
                .and_then(|v| v.GetFolder())
                .map_err(|e| format!("IFolderView::GetFolder: {e}"))?;
            let name = folder
                .GetDisplayName(SIGDN_FILESYSPATH)
                .map_err(|e| format!("no file-system path: {e}"))?;
            let path = crate::shell::take_string(name);
            if path.is_empty() {
                return Err("empty path".into());
            }
            return Ok(PathBuf::from(path));
        }
    }
    Err(format!(
        "no window matched (tab {}): {}",
        active_tab.0 as isize,
        seen.join("; ")
    ))
}

/// What a common file dialog is for, told apart by its file name box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    /// 打开: the file name box is the combo box 1148.
    Open,
    /// 另存为: the file name box is edit 1001 inside a combo box.
    Save,
    /// 选择文件夹: neither (its "Folder:" box would choose the folder at once).
    Folder,
}

fn file_name_box(dlg: HWND) -> (DialogKind, Option<HWND>) {
    // SAFETY: plain FFI calls on another process's windows (read-only).
    unsafe {
        let combo = GetDlgItem(Some(dlg), 1148);
        if !combo.0.is_null() {
            let edit = if class_name(combo) == "Edit" {
                Some(combo)
            } else {
                child_of_class(combo, "Edit")
            };
            return (DialogKind::Open, edit);
        }
        let save = descendants(dlg).into_iter().find(|&h| {
            class_name(h) == "Edit"
                && GetDlgCtrlID(h) == 1001
                && class_name(GetParent(h)) == "ComboBox"
        });
        match save {
            Some(e) => (DialogKind::Save, Some(e)),
            None => (DialogKind::Folder, None),
        }
    }
}

pub fn dialog_kind(dlg: HWND) -> DialogKind {
    file_name_box(dlg).0
}

fn set_text(edit: HWND, text: &str) {
    let wide = to_wide(text);
    // SAFETY: WM_SETTEXT with a NUL-terminated buffer that outlives the call.
    unsafe {
        let _ = SendMessageTimeoutW(
            edit,
            WM_SETTEXT as u32,
            WPARAM(0),
            LPARAM(wide.as_ptr() as isize),
            SMTO_ABORTIFHUNG as u32,
            1000,
            None,
        );
    }
}

/// Hands `path` to dialog `dlg` as Enter in Listary's search does: a folder opens in the
/// dialog; a file is opened by an Open dialog, and in a Save dialog its folder opens with its
/// name in the file name box (the user still confirms). A Select Folder dialog goes to the
/// file's folder. Blocks like [`jump`]: call it on a worker thread.
pub fn choose(dlg: HWND, path: &Path, is_folder: bool) -> JumpOutcome {
    // The file name box expands `%NAME%` as the address bar does.
    if expands_variables(&path.to_string_lossy()) {
        return JumpOutcome::VariableInPath;
    }
    if is_folder {
        return jump(dlg, path);
    }
    let (kind, edit) = file_name_box(dlg);
    match (kind, edit) {
        (DialogKind::Open, Some(edit)) => {
            set_text(edit, &path.to_string_lossy());
            // SAFETY: a plain posted command: the Open button (IDOK).
            unsafe {
                let _ = PostMessageW(Some(dlg), WM_COMMAND as u32, WPARAM(1), LPARAM(0));
            }
            JumpOutcome::Done
        }
        (DialogKind::Save, Some(edit)) => {
            let Some(dir) = path.parent() else {
                return JumpOutcome::NoAddressBar;
            };
            let outcome = jump(dlg, dir);
            if let Some(name) = path.file_name() {
                set_text(edit, &name.to_string_lossy());
            }
            outcome
        }
        _ => match path.parent() {
            Some(dir) => jump(dlg, dir),
            None => JumpOutcome::NoAddressBar,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_text_matches_its_folder() {
        let dir = Path::new(r"C:\Users\me\Documents");
        assert!(shows_folder(r"Address: C:\Users\me\Documents", dir));
        assert!(shows_folder(r"地址: c:\users\me\documents\", dir));
        assert!(shows_folder("地址\u{FF1A}C:\\Users\\me\\Documents", dir));
        assert!(!shows_folder(r"Address: C:\Users\me\My Documents", dir));
        assert!(!shows_folder(r"Address: C:\Users\me", dir));
        assert!(shows_folder(r"Address: C:\", Path::new(r"C:\")));
    }

    #[test]
    fn variables_between_percent_signs() {
        // SAFETY: the test sets a variable of its own; no other test reads it.
        unsafe { std::env::set_var("PF_TEST_VAR", "x") };
        assert!(expands_variables(r"C:\pf\pct %PF_TEST_VAR% and #hash"));
        assert!(expands_variables(r"C:\a%b\%PF_TEST_VAR%\c"));
        assert!(!expands_variables(r"C:\pf\50% off"));
        assert!(!expands_variables(r"C:\pf\%PF_NO_SUCH_VAR_HERE%"));
        assert!(!expands_variables(r"C:\pf\%%"));
    }
}
