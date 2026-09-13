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
use windows_core::{Error, Interface, PCSTR, PCWSTR, Result};

const WM_MENUCHAR: u32 = 0x0120;
/// `GCS_VERBW`: ask `GetCommandString` for the canonical verb as UTF-16.
const GCS_VERBW: u32 = 0x0004;

thread_local! {
    /// The shell menu currently shown by `TrackPopupMenuEx` on this thread (at most one).
    static ACTIVE: RefCell<Option<IContextMenu>> = const { RefCell::new(None) };
}

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
        let mut pidls: Vec<LPCITEMIDLIST> = Vec::with_capacity(paths.len());
        for p in paths {
            let w = to_wide(&p.to_string_lossy());
            let mut pidl: LPITEMIDLIST = std::ptr::null_mut();
            // SAFETY: string outlives the call; pidl is freed below.
            let hr =
                unsafe { SHParseDisplayName(PCWSTR(w.as_ptr()), None, &mut pidl, SFGAOF(0), None) };
            if hr.is_ok() && !pidl.is_null() {
                pidls.push(pidl as LPCITEMIDLIST);
            }
        }
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
            // SAFETY: each pidl came from SHParseDisplayName.
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
    /// Runs the shell command whose canonical verb is `verb` (e.g. `delete`) as if picked from
    /// the menu, without showing it. Returns `Ok(false)` if the verb is not offered. `shift` =
    /// Shift held (Shift+Delete deletes permanently).
    pub fn invoke_verb(&mut self, verb: &str, owner: HWND, pt: POINT, shift: bool) -> Result<bool> {
        // SAFETY: temporary menu owned and destroyed here.
        let hmenu = unsafe { CreatePopupMenu() };
        let result = (|| -> Result<bool> {
            let count = self.populate(hmenu, 0, 0x1000, 0x6FFF, false)?;
            for cmd in 0x1000..0x1000 + count {
                if self.verb(cmd).is_some_and(|v| v.eq_ignore_ascii_case(verb)) {
                    self.invoke(cmd, owner, pt, shift)?;
                    return Ok(true);
                }
            }
            Ok(false)
        })();
        ACTIVE.with(|a| *a.borrow_mut() = None);
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
