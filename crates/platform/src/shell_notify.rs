//! Shell change notifications (`SHChangeNotifyRegister`): the channel Explorer's own views use
//! to learn that the Recycle Bin filled up or emptied, or that a desktop namespace item was
//! added or removed. Folder watchers see neither.

use crate::bindings::*;
use crate::wide::to_wide;
use windows_core::{Error, HRESULT, PCWSTR, Result};

const E_FAIL: HRESULT = HRESULT(0x8000_4005_u32 as i32);

/// A live registration; dropping it deregisters.
pub struct ShellChangeWatch {
    id: u32,
}

impl ShellChangeWatch {
    /// Makes the shell post `message` to `hwnd` for changes under the Recycle Bin (files
    /// recycled from anywhere, the bin emptied) and for the desktop namespace's own entries.
    /// `wparam`/`lparam` of each message must be handed to [`ShellChangeWatch::release`].
    pub fn register_desktop(hwnd: HWND, message: u32) -> Result<Self> {
        let bin = to_wide(&format!("::{}", crate::shell::RECYCLE_BIN_CLSID));
        let mut bin_pidl: LPITEMIDLIST = std::ptr::null_mut();
        // SAFETY: the string outlives the call; the id list is freed below.
        let hr = unsafe {
            SHParseDisplayName(PCWSTR(bin.as_ptr()), None, &mut bin_pidl, SFGAOF(0), None)
        };
        if hr.is_err() {
            return Err(Error::from_hresult(hr));
        }
        if bin_pidl.is_null() {
            return Err(Error::from_hresult(E_FAIL));
        }
        // The desktop root is the empty id list (a single zero `cb`).
        let root: [u16; 1] = [0];
        let entries = [
            SHChangeNotifyEntry {
                pidl: bin_pidl as LPCITEMIDLIST,
                fRecursive: true.into(),
            },
            SHChangeNotifyEntry {
                pidl: root.as_ptr().cast(),
                fRecursive: false.into(),
            },
        ];
        let events = SHCNE_UPDATEDIR
            | SHCNE_UPDATEITEM
            | SHCNE_UPDATEIMAGE
            | SHCNE_CREATE
            | SHCNE_DELETE
            | SHCNE_MKDIR
            | SHCNE_RMDIR
            | SHCNE_RENAMEITEM
            | SHCNE_RENAMEFOLDER
            | SHCNE_ATTRIBUTES
            | SHCNE_ASSOCCHANGED;
        // SAFETY: `entries` and the id lists they point to outlive the call (the shell copies
        // them); `hwnd` is a live window on this thread.
        let id = unsafe {
            SHChangeNotifyRegister(
                hwnd,
                SHCNRF_ShellLevel | SHCNRF_NewDelivery,
                events,
                message,
                entries.len() as i32,
                entries.as_ptr(),
            )
        };
        // SAFETY: the id list came from SHParseDisplayName.
        unsafe { ILFree(Some(bin_pidl)) };
        if id == 0 {
            return Err(Error::from_hresult(E_FAIL));
        }
        Ok(Self { id })
    }

    /// Frees the shared memory behind one delivered message (`SHCNRF_NewDelivery`). The
    /// payload itself is not needed: any event means "look again".
    pub fn release(wparam: usize, lparam: isize) {
        let handle = HANDLE(wparam as *mut core::ffi::c_void);
        let mut pidls: *mut LPITEMIDLIST = std::ptr::null_mut();
        // SAFETY: `wparam`/`lparam` are the handle and process id the shell posted; the lock
        // is released right away.
        unsafe {
            let lock = SHChangeNotification_Lock(handle, lparam as u32, &mut pidls, None);
            if !lock.0.is_null() {
                let _ = SHChangeNotification_Unlock(lock);
            }
        }
    }
}

impl Drop for ShellChangeWatch {
    fn drop(&mut self) {
        // SAFETY: `id` came from SHChangeNotifyRegister and is deregistered once.
        unsafe {
            let _ = SHChangeNotifyDeregister(self.id);
        }
    }
}
