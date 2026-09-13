//! COM / OLE apartment initialization.

use crate::bindings::*;
use windows_core::Result;

/// Keeps OLE initialized (single-threaded apartment) on the current thread.
///
/// Must be created on the UI thread **before** the composition dispatcher queue and
/// before any WebView2 environment, so that `RegisterDragDrop` works.
pub struct OleGuard(());

impl OleGuard {
    pub fn init() -> Result<Self> {
        // SAFETY: plain FFI call; S_FALSE (already initialized) is a success code.
        unsafe { OleInitialize(core::ptr::null()).ok()? };
        Ok(Self(()))
    }
}

impl Drop for OleGuard {
    fn drop(&mut self) {
        // SAFETY: balances the successful OleInitialize in `init`.
        unsafe { OleUninitialize() };
    }
}

/// Initializes a multithreaded apartment on a worker thread (icon extraction etc.).
pub struct MtaGuard(());

impl MtaGuard {
    pub fn init() -> Result<Self> {
        // SAFETY: plain FFI call.
        unsafe {
            CoInitializeEx(
                None,
                COINIT_MULTITHREADED as u32 | COINIT_DISABLE_OLE1DDE as u32,
            )
            .ok()?
        };
        Ok(Self(()))
    }
}

impl Drop for MtaGuard {
    fn drop(&mut self) {
        // SAFETY: balances CoInitializeEx.
        unsafe { CoUninitialize() };
    }
}
