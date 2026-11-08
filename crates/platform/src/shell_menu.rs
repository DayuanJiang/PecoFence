//! Explorer's own context menu (`IContextMenu`) for one or more file-system items, merged into a
//! popup menu we own (plan §10.3 task 16: 发送到 / 复制 / 删除 / 固定 … come from the shell).
//!
//! Shell extensions need a few menu messages routed back to them (`WM_INITMENUPOPUP` for lazy
//! submenus such as "Send to", owner-draw messages, `WM_MENUCHAR`). The window procedure calls
//! [`forward_menu_message`] first, which forwards to the menu that is currently being tracked.

use crate::bindings::*;
use crate::wide::to_wide;
use std::cell::RefCell;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::{Duration, Instant};
use windows_core::{Error, Interface, PCSTR, PCWSTR, PWSTR, Result};

const WM_MENUCHAR: u32 = 0x0120;
/// `GCS_VERBW`: ask `GetCommandString` for the canonical verb as UTF-16.
const GCS_VERBW: u32 = 0x0004;
/// `CMF_OPTIMIZEFORINVOKE`: the menu is not shown, a verb is invoked by name right after.
const CMF_OPTIMIZEFORINVOKE: u32 = 0x0800;

thread_local! {
    /// The shell menu currently shown by `TrackPopupMenuEx` on this thread (at most one).
    static ACTIVE: RefCell<Option<IContextMenu>> = const { RefCell::new(None) };
}

/// Where `prepare_handler_cache` is: not started, running, or done (whatever its outcome).
static CACHE_STATE: AtomicU8 = AtomicU8::new(CACHE_NOT_STARTED);
const CACHE_NOT_STARTED: u8 = 0;
const CACHE_RUNNING: u8 = 1;
const CACHE_DONE: u8 = 2;
/// A menu built while `prepare_handler_cache` still runs waits this long for it at most.
const CACHE_WAIT: Duration = Duration::from_secs(2);
/// `prepare_handler_cache` got the cache started (its library icon came back).
static CACHE_READY: AtomicBool = AtomicBool::new(false);

/// The shell's context menu object for a set of items plus the command-id range it was given.
pub struct ShellContextMenu {
    menu: IContextMenu,
    first: u32,
    end: u32,
}

impl ShellContextMenu {
    /// Binds Explorer's context-menu handler for `paths` (all must exist; unknown ones are skipped).
    pub fn for_paths(paths: &[&Path]) -> Result<Self> {
        if paths.is_empty() {
            return Err(Error::from_hresult(E_INVALIDARG));
        }
        let pidls = id_lists_for(paths);
        let bound = (|| -> Result<IContextMenu> {
            if pidls.is_empty() {
                return Err(Error::from_hresult(E_FAIL));
            }
            // SAFETY: the array copies the id lists; BHID_SFUIObject yields the UI object of the
            // common parent folder for all items.
            unsafe {
                let array: IShellItemArray = SHCreateShellItemArrayFromIDLists(&pidls)?;
                array.BindToHandler(None, &BHID_SFUIObject)
            }
        })();
        for p in pidls {
            // SAFETY: each pidl came from `id_lists_for`.
            unsafe { ILFree(Some(p)) };
        }
        Ok(Self {
            menu: bound?,
            first: 0,
            end: 0,
        })
    }

    /// Appends the shell's items to `hmenu` at position `index`, using ids in `first..=last`.
    /// Returns how many ids were used. The menu becomes the thread's active shell menu until drop.
    pub fn populate(
        &mut self,
        hmenu: HMENU,
        index: u32,
        first: u32,
        last: u32,
        extended: bool,
    ) -> Result<u32> {
        let mut flags = CMF_NORMAL as u32;
        if extended {
            flags |= CMF_EXTENDEDVERBS as u32;
        }
        wait_for_handler_cache();
        // SAFETY: hmenu is a valid menu owned by the caller.
        let hr = unsafe { self.menu.QueryContextMenu(hmenu, index, first, last, flags) };
        if hr.is_err() {
            return Err(Error::from_hresult(hr));
        }
        let count = (hr.0 as u32) & 0xFFFF;
        self.first = first;
        self.end = first + count;
        ACTIVE.with(|a| *a.borrow_mut() = Some(self.menu.clone()));
        Ok(count)
    }

    /// Removes the shell's top-level items whose canonical verb is one of `verbs` (ASCII case
    /// insensitive): commands the host menu already offers under its own label. Returns how
    /// many went; tidy the separators afterwards.
    pub fn remove_verbs(&self, hmenu: HMENU, verbs: &[&str]) -> u32 {
        let mut removed = 0;
        // SAFETY: plain FFI calls on the caller's menu; positions are walked from the end, so
        // removing one never shifts a position still to be visited.
        unsafe {
            for pos in (0..GetMenuItemCount(Some(hmenu)).max(0)).rev() {
                let id = GetMenuItemID(hmenu, pos);
                if self
                    .verb(id)
                    .is_some_and(|v| verbs.iter().any(|w| v.eq_ignore_ascii_case(w)))
                {
                    let _ = RemoveMenu(hmenu, pos as u32, MF_BYPOSITION as u32);
                    removed += 1;
                }
            }
        }
        removed
    }

    /// Canonical verbs of the shell's top-level items in `hmenu`, in menu order; items without
    /// one (submenus such as 发送到, most extensions) are left out.
    pub fn verbs(&self, hmenu: HMENU) -> Vec<String> {
        // SAFETY: plain FFI calls on the caller's menu.
        unsafe {
            (0..GetMenuItemCount(Some(hmenu)).max(0))
                .filter_map(|pos| self.verb(GetMenuItemID(hmenu, pos)))
                .collect()
        }
    }

    /// True when `cmd` (as returned by `TrackPopupMenuEx`) belongs to the shell's range.
    pub fn contains(&self, cmd: u32) -> bool {
        self.end > self.first && cmd >= self.first && cmd < self.end
    }

    /// Canonical verb of a shell command (e.g. `delete`, `properties`, `rename`), if it has one.
    pub fn verb(&self, cmd: u32) -> Option<String> {
        if !self.contains(cmd) {
            return None;
        }
        let mut buf = [0u16; 128];
        // SAFETY: buffer size passed in characters; the W variant writes UTF-16.
        let hr = unsafe {
            self.menu.GetCommandString(
                (cmd - self.first) as usize,
                GCS_VERBW,
                None,
                buf.as_mut_ptr() as *mut i8,
                buf.len() as u32,
            )
        };
        if hr.is_err() {
            return None;
        }
        let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..n]))
    }

    /// Executes a shell command chosen from the menu. `shift` sets `CMIC_MASK_SHIFT_DOWN`, which
    /// Explorer's verbs honour the way the keyboard does (e.g. `delete` skips the Recycle Bin).
    pub fn invoke(&self, cmd: u32, owner: HWND, pt: POINT, shift: bool) -> Result<()> {
        if !self.contains(cmd) {
            return Err(Error::from_hresult(E_INVALIDARG));
        }
        let offset = (cmd - self.first) as usize;
        let mut mask = (CMIC_MASK_UNICODE | CMIC_MASK_PTINVOKE) as u32;
        if shift {
            mask |= CMIC_MASK_SHIFT_DOWN as u32;
        }
        let info = CMINVOKECOMMANDINFOEX {
            cbSize: std::mem::size_of::<CMINVOKECOMMANDINFOEX>() as u32,
            fMask: mask,
            hwnd: owner,
            // MAKEINTRESOURCE(offset): the low word carries the command offset.
            lpVerb: PCSTR(offset as *const u8),
            lpVerbW: PCWSTR(offset as *const u16),
            nShow: SW_SHOWNORMAL,
            ptInvoke: pt,
            ..Default::default()
        };
        // SAFETY: the EX struct starts with the base struct; cbSize tells the shell which one.
        let hr = unsafe {
            self.menu
                .InvokeCommand(&info as *const CMINVOKECOMMANDINFOEX as *const CMINVOKECOMMANDINFO)
        };
        hr.ok()
    }
}

impl ShellContextMenu {
    /// Runs the shell command whose canonical verb is `verb` (`delete`, `cut`, `copy`,
    /// `properties`) without showing a menu, the way Explorer runs Delete or Ctrl+C on its
    /// selection: the menu is only prepared for invoking (CMF_OPTIMIZEFORINVOKE) and the verb
    /// goes by name. Building the whole menu to look the verb up loaded every menu extension
    /// and took up to a second per key press on a PC with many of them (feedback #108). Fails
    /// when the items do not offer the verb. `shift` = Shift held (Shift+Delete deletes
    /// permanently).
    pub fn invoke_verb(&self, verb: &str, owner: HWND, pt: POINT, shift: bool) -> Result<()> {
        // SAFETY: temporary menu owned and destroyed here.
        let hmenu = unsafe { CreatePopupMenu() };
        let result = (|| -> Result<()> {
            // SAFETY: hmenu is the menu created above.
            let hr = unsafe {
                self.menu
                    .QueryContextMenu(hmenu, 0, 1, 0x7FFF, CMF_OPTIMIZEFORINVOKE)
            };
            if hr.is_err() {
                return Err(Error::from_hresult(hr));
            }
            let ansi: Vec<u8> = verb.bytes().chain(std::iter::once(0)).collect();
            let wide = to_wide(verb);
            let mut mask = (CMIC_MASK_UNICODE | CMIC_MASK_PTINVOKE) as u32;
            if shift {
                mask |= CMIC_MASK_SHIFT_DOWN as u32;
            }
            let info = CMINVOKECOMMANDINFOEX {
                cbSize: std::mem::size_of::<CMINVOKECOMMANDINFOEX>() as u32,
                fMask: mask,
                hwnd: owner,
                lpVerb: PCSTR(ansi.as_ptr()),
                lpVerbW: PCWSTR(wide.as_ptr()),
                nShow: SW_SHOWNORMAL,
                ptInvoke: pt,
                ..Default::default()
            };
            // SAFETY: the verb strings outlive the call; the EX struct starts with the base
            // struct and cbSize tells the shell which one it is.
            let hr = unsafe {
                self.menu.InvokeCommand(
                    &info as *const CMINVOKECOMMANDINFOEX as *const CMINVOKECOMMANDINFO,
                )
            };
            hr.ok()
        })();
        // SAFETY: we own the menu.
        let _ = unsafe { DestroyMenu(hmenu) };
        result
    }
}

impl Drop for ShellContextMenu {
    fn drop(&mut self) {
        ACTIVE.with(|a| {
            let mut slot = a.borrow_mut();
            if slot
                .as_ref()
                .is_some_and(|m| m.as_raw() == self.menu.as_raw())
            {
                *slot = None;
            }
        });
    }
}

/// Makes the shell start its item handler cache now, on a worker thread, so that no item menu
/// has to start it. The shell starts that cache on a thread of its own the first time a
/// process needs it, and the caller waits for that thread without answering messages
/// (SHCreateThread with CTF_WAIT_NO_REENTRANCY). In a folder's first menu that happened inside
/// a menu extension's DLL initialization, with the loader lock held: the extension created a
/// window there (Baidu Netdisk's YunShellExt on the PC where it was reproduced), which ran the
/// pending update of Windows' 「包含到库中」 helper and its library icons. The new thread could
/// never start, and PecoFence froze for good at its first folder right-click (feedback #103,
/// #109). A library's icon starts the cache the same way without loading any menu extension;
/// building a menu here instead would load them all and could stop this worker the same way.
pub fn prepare_handler_cache() {
    // Windows' own libraries; a PC may lack some of them, and their file names are English
    // in every display language.
    const LIBRARIES: [&str; 4] = ["Documents", "Pictures", "Music", "Videos"];
    CACHE_STATE.store(CACHE_RUNNING, Ordering::Release);
    let spawned = std::thread::Builder::new()
        .name("shell handler cache".into())
        .spawn(|| {
            let _sta = crate::com::OleGuard::init();
            let started = Instant::now();
            let mut outcome = Err(Error::from_hresult(E_FAIL));
            for name in LIBRARIES {
                let path = format!("::{{031E4825-7B94-4DC3-B131-E946B44C8DD5}}\\{name}.library-ms");
                outcome = crate::shell::shell_image(Path::new(&path), 32, true).map(|_| name);
                if outcome.is_ok() {
                    break;
                }
            }
            let ms = started.elapsed().as_millis() as u64;
            CACHE_READY.store(outcome.is_ok(), Ordering::Release);
            CACHE_STATE.store(CACHE_DONE, Ordering::Release);
            match outcome {
                Ok(library) => tracing::info!(ms, library, "shell handler cache prepared"),
                Err(e) => tracing::warn!(ms, error = %e, "shell handler cache not prepared"),
            }
        });
    if let Err(e) = spawned {
        CACHE_STATE.store(CACHE_DONE, Ordering::Release);
        tracing::warn!(error = %e, "shell handler cache thread failed to start");
    }
}

/// Builds the shell's menu for the file `path` once on a worker thread, `after` from now, and
/// drops it unseen: the first item menu after a start loads every menu extension of the
/// installed programs, which took 0.5 to 0.9 s on a PC with many of them (the later menus 60 to
/// 180 ms), and that loading is done here instead. The modules it loaded are pinned: the
/// worker's COM apartment would unload the extensions again when it ends, while those the UI
/// thread loads stay anyway. Only once `prepare_handler_cache` got the handler cache started,
/// and only for a file: a folder's menu runs Windows' 「包含到库中」 helper, where the 0.2.4
/// freeze started (see there).
pub fn prewarm(path: std::path::PathBuf, after: Duration) {
    let spawned = std::thread::Builder::new()
        .name("shell menu prewarm".into())
        .spawn(move || {
            std::thread::sleep(after);
            wait_for_handler_cache();
            if !CACHE_READY.load(Ordering::Acquire) {
                tracing::info!("shell menu not prewarmed: the handler cache is not prepared");
                return;
            }
            let _sta = crate::com::OleGuard::init();
            let started = Instant::now();
            let before = loaded_modules();
            let built = (|| -> Result<u32> {
                let mut menu = ShellContextMenu::for_paths(&[path.as_path()])?;
                // SAFETY: a menu of our own, destroyed below; ids as the item menu hands out.
                let hmenu = unsafe { CreatePopupMenu() };
                let n = menu.populate(hmenu, 0, 0x1000, 0x6FFF, false);
                // SAFETY: as above.
                let _ = unsafe { DestroyMenu(hmenu) };
                n
            })();
            let mut pinned = 0;
            for module in loaded_modules() {
                if before.contains(&module) {
                    continue;
                }
                let mut out = HMODULE::default();
                // SAFETY: `module` is the base address of a module loaded in this process.
                let ok = unsafe {
                    GetModuleHandleExW(
                        (GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN)
                            as u32,
                        PCWSTR(module.0 as *const u16),
                        &mut out,
                    )
                };
                pinned += ok.as_bool() as u32;
            }
            let ms = started.elapsed().as_millis() as u64;
            match built {
                Ok(shell_ids) => tracing::info!(ms, shell_ids, pinned, "shell menu prewarmed"),
                Err(e) => tracing::warn!(ms, pinned, error = %e, "shell menu prewarm failed"),
            }
        });
    if let Err(e) = spawned {
        tracing::warn!(error = %e, "shell menu prewarm thread failed to start");
    }
}

/// The modules loaded in this process (their base addresses).
fn loaded_modules() -> Vec<HMODULE> {
    windows_core::link!("kernel32.dll" "system" fn K32EnumProcessModules(process: HANDLE, modules: *mut HMODULE, cb: u32, needed: *mut u32) -> windows_core::BOOL);
    let mut modules = vec![HMODULE::default(); 2048];
    let mut needed = 0u32;
    // SAFETY: the buffer's size in bytes is passed; Windows fills at most that much.
    let ok = unsafe {
        K32EnumProcessModules(
            GetCurrentProcess(),
            modules.as_mut_ptr(),
            (modules.len() * size_of::<HMODULE>()) as u32,
            &mut needed,
        )
    };
    let n = if ok.as_bool() {
        needed as usize / size_of::<HMODULE>()
    } else {
        0
    };
    modules.truncate(n);
    modules
}

/// A menu built in the first moments after start, while `prepare_handler_cache` still runs
/// (146 ms on a desktop PC), waits for it: the menu could start the cache itself otherwise.
/// Waiting is safe here, as the caller holds no loader lock; it ends after `CACHE_WAIT` anyway.
fn wait_for_handler_cache() {
    if CACHE_STATE.load(Ordering::Acquire) != CACHE_RUNNING {
        return;
    }
    let started = Instant::now();
    while CACHE_STATE.load(Ordering::Acquire) == CACHE_RUNNING && started.elapsed() < CACHE_WAIT {
        std::thread::sleep(Duration::from_millis(10));
    }
    let ms = started.elapsed().as_millis() as u64;
    let ready = CACHE_STATE.load(Ordering::Acquire) == CACHE_DONE;
    tracing::info!(ms, ready, "shell menu waited for the shell handler cache");
}

/// The id lists of `paths` for one shell handler (unknown paths are skipped); free each with
/// `ILFree`. Items of the user's Desktop and of the Public Desktop have no common parent folder,
/// so for a selection with both the shell bound nothing (delete, cut, copy and Explorer's menu
/// did nothing); such a selection comes as items of the desktop itself, which merges the two
/// folders as Explorer's desktop does.
fn id_lists_for(paths: &[&Path]) -> Vec<LPCITEMIDLIST> {
    let parent = |p: &Path| p.parent().map(|d| d.to_string_lossy().to_lowercase());
    if paths.iter().any(|p| parent(p) != parent(paths[0]))
        && let Some(children) = desktop_children(paths)
    {
        return children;
    }
    let mut pidls: Vec<LPCITEMIDLIST> = Vec::with_capacity(paths.len());
    for p in paths {
        let w = to_wide(&p.to_string_lossy());
        let mut pidl: LPITEMIDLIST = std::ptr::null_mut();
        // SAFETY: string outlives the call; the caller frees the pidl.
        let hr =
            unsafe { SHParseDisplayName(PCWSTR(w.as_ptr()), None, &mut pidl, SFGAOF(0), None) };
        if hr.is_ok() && !pidl.is_null() {
            pidls.push(pidl as LPCITEMIDLIST);
        }
    }
    pidls
}

/// `paths` as items of the desktop root, in their order; `None` unless the desktop lists all.
fn desktop_children(paths: &[&Path]) -> Option<Vec<LPCITEMIDLIST>> {
    let wanted: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().to_lowercase())
        .collect();
    let mut found: Vec<LPCITEMIDLIST> = vec![std::ptr::null(); paths.len()];
    // SAFETY: plain COM calls; every child the enumerator hands out is kept in `found` or freed.
    unsafe {
        if let Ok(desktop) = SHGetDesktopFolder()
            && let Ok(items) = desktop.EnumObjects(
                HWND::default(),
                SHCONTF_FOLDERS | SHCONTF_NONFOLDERS | SHCONTF_INCLUDEHIDDEN,
            )
        {
            loop {
                let mut child: LPITEMIDLIST = std::ptr::null_mut();
                if items.Next(1, &mut child, None) != S_OK || child.is_null() {
                    break;
                }
                let mut name = STRRET::default();
                let mut parsing = PWSTR::null();
                let path = if desktop
                    .GetDisplayNameOf(child, SHGDN_FORPARSING, &mut name)
                    .is_ok()
                    && StrRetToStrW(&mut name, Some(child), &mut parsing).is_ok()
                {
                    crate::shell::take_string(parsing).to_lowercase()
                } else {
                    String::new()
                };
                match wanted.iter().position(|w| *w == path) {
                    Some(i) if found[i].is_null() => found[i] = child,
                    _ => ILFree(Some(child)),
                }
            }
        }
    }
    if found.iter().all(|p| !p.is_null()) {
        return Some(found);
    }
    for p in found.into_iter().filter(|p| !p.is_null()) {
        // SAFETY: from the enumerator above, freed once.
        unsafe { ILFree(Some(p)) };
    }
    None
}

/// Routes menu messages to the active shell menu. Returns the result to hand back to the system
/// when the shell handled the message; `None` otherwise (including when no shell menu is active).
pub fn forward_menu_message(msg: u32, wparam: usize, lparam: isize) -> Option<isize> {
    const INITMENUPOPUP: u32 = WM_INITMENUPOPUP as u32;
    const DRAWITEM: u32 = WM_DRAWITEM as u32;
    const MEASUREITEM: u32 = WM_MEASUREITEM as u32;
    if !matches!(msg, INITMENUPOPUP | DRAWITEM | MEASUREITEM | WM_MENUCHAR) {
        return None;
    }
    let menu = ACTIVE.with(|a| a.borrow().clone())?;
    // SAFETY: plain COM calls on an interface we hold a reference to.
    unsafe {
        if let Ok(m3) = menu.cast::<IContextMenu3>() {
            let mut result = LRESULT(0);
            if m3
                .HandleMenuMsg2(msg, WPARAM(wparam), LPARAM(lparam), Some(&mut result))
                .is_ok()
            {
                return Some(result.0);
            }
        } else if let Ok(m2) = menu.cast::<IContextMenu2>()
            && m2
                .HandleMenuMsg(msg, WPARAM(wparam), LPARAM(lparam))
                .is_ok()
        {
            return Some(0);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Binds Explorer's menu for a real file and merges it into a fresh popup menu.
    #[test]
    fn binds_and_populates_for_a_file() {
        let _com = crate::com::OleGuard::init().expect("COM");
        let dir = std::env::temp_dir().join("pecofence-shell-menu-test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("probe.txt");
        std::fs::write(&file, b"x").unwrap();

        let mut menu = ShellContextMenu::for_paths(&[file.as_path()]).expect("IContextMenu");
        // SAFETY: plain FFI; the menu is destroyed below.
        let hmenu = unsafe { CreatePopupMenu() };
        let count = menu
            .populate(hmenu, 0, 0x1000, 0x6FFF, false)
            .expect("QueryContextMenu");
        assert!(count > 0, "shell menu should contribute items");
        assert!(menu.contains(0x1000));
        assert!(!menu.contains(0x1000 + count));
        // At least one of the items carries a canonical verb (open / delete / properties …).
        let verbs: Vec<String> = (0..count).filter_map(|i| menu.verb(0x1000 + i)).collect();
        assert!(verbs.iter().any(|v| !v.is_empty()), "no verbs: {verbs:?}");
        // While populated, menu messages are accepted (WM_INITMENUPOPUP on an unknown submenu is
        // simply ignored by the shell, but must not panic).
        let _ = forward_menu_message(WM_INITMENUPOPUP as u32, hmenu.0 as usize, 0);
        drop(menu);
        assert!(forward_menu_message(WM_INITMENUPOPUP as u32, 0, 0).is_none());
        // SAFETY: we own the menu.
        let _ = unsafe { DestroyMenu(hmenu) };
        let _ = std::fs::remove_dir_all(&dir);
    }
}
