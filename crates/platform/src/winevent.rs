//! `SetWinEventHook` wrapper (out-of-context, callbacks arrive on this thread's message loop).

use crate::bindings::*;
use std::cell::RefCell;
use std::collections::HashMap;
use windows_core::{Error, Result};

pub type EventCallback = Box<dyn FnMut(u32, HWND)>;

thread_local! {
    static CALLBACKS: RefCell<HashMap<isize, EventCallback>> = RefCell::new(HashMap::new());
}

pub const SYSTEM_FOREGROUND: u32 = EVENT_SYSTEM_FOREGROUND as u32;
pub const SYSTEM_MINIMIZESTART: u32 = EVENT_SYSTEM_MINIMIZESTART as u32;
pub const SYSTEM_MINIMIZEEND: u32 = EVENT_SYSTEM_MINIMIZEEND as u32;

/// An installed hook; unhooked on drop.
pub struct WinEventHook {
    hook: HWINEVENTHOOK,
}

impl WinEventHook {
    /// Installs an out-of-context hook for `[event_min, event_max]` that skips our own
    /// process. The thread must pump messages for callbacks to be delivered.
    pub fn install(event_min: u32, event_max: u32, callback: EventCallback) -> Result<Self> {
        // SAFETY: the callback is a `extern "system"` fn with the documented signature.
        let hook = unsafe {
            SetWinEventHook(
                event_min,
                event_max,
                None,
                Some(event_proc),
                0,
                0,
                (WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS) as u32,
            )
        };
        if hook.0.is_null() {
            return Err(Error::from_thread());
        }
        CALLBACKS.with(|c| c.borrow_mut().insert(hook.0 as isize, callback));
        Ok(Self { hook })
    }
}

impl Drop for WinEventHook {
    fn drop(&mut self) {
        // SAFETY: balances SetWinEventHook.
        unsafe {
            let _ = UnhookWinEvent(self.hook);
        }
        CALLBACKS.with(|c| c.borrow_mut().remove(&(self.hook.0 as isize)));
    }
}

unsafe extern "system" fn event_proc(
    hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    // OBJID_WINDOW == 0: only whole-window events.
    if id_object != 0 {
        return;
    }
    // Take the callback out while running it so a re-entrant event cannot alias it.
    let taken = CALLBACKS.with(|c| c.borrow_mut().remove(&(hook.0 as isize)));
    if let Some(mut cb) = taken {
        cb(event, hwnd);
        CALLBACKS.with(|c| {
            // Only restore if the hook was not dropped inside the callback.
            let mut map = c.borrow_mut();
            map.entry(hook.0 as isize).or_insert(cb);
        });
    }
}
