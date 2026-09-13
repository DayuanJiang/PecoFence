//! Fallback creation of a Windows.System.DispatcherQueue on the current thread.
//!
//! `windows-composition` creates its queue with `DQTAT_COM_ASTA`, which is not meaningful for
//! `DQTYPE_THREAD_CURRENT` and may conflict with a thread already initialized by
//! `OleInitialize`. This path uses `DQTAT_COM_NONE`, as Microsoft's documentation prescribes.

use crate::bindings::*;
use windows_core::Result;

/// Owns the dispatcher queue controller; keep it alive while the compositor is used.
pub struct DispatcherQueue(#[allow(dead_code)] IDispatcherQueueController);

impl DispatcherQueue {
    pub fn create_on_current_thread() -> Result<Self> {
        let options = DispatcherQueueOptions {
            dwSize: size_of::<DispatcherQueueOptions>() as u32,
            threadType: DQTYPE_THREAD_CURRENT,
            apartmentType: DQTAT_COM_NONE,
        };
        // SAFETY: options is fully initialized; the call returns an owned COM reference.
        let controller = unsafe { CreateDispatcherQueueController(options)? };
        Ok(Self(controller))
    }
}
