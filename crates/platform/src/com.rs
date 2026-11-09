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

/// A single-threaded apartment for a long-lived worker thread (no OLE: no drag and drop or
/// clipboard there). Its wait must pump messages: [`sta_channel`].
pub struct StaGuard(());

impl StaGuard {
    pub fn init() -> Result<Self> {
        // SAFETY: plain FFI call; S_FALSE (already initialized) is a success code.
        unsafe {
            CoInitializeEx(
                None,
                COINIT_APARTMENTTHREADED as u32 | COINIT_DISABLE_OLE1DDE as u32,
            )
            .ok()?
        };
        Ok(Self(()))
    }
}

impl Drop for StaGuard {
    fn drop(&mut self) {
        // SAFETY: balances CoInitializeEx.
        unsafe { CoUninitialize() };
    }
}

struct StaShared<T> {
    items: std::sync::Mutex<std::collections::VecDeque<T>>,
    /// Auto-reset event: set when an item is queued (a HANDLE kept as an integer).
    event: isize,
}

impl<T> Drop for StaShared<T> {
    fn drop(&mut self) {
        // SAFETY: the event came from CreateEventW and is closed once, with the last owner.
        unsafe {
            let _ = CloseHandle(HANDLE(self.event as *mut core::ffi::c_void));
        }
    }
}

/// Sends work to an STA worker ([`sta_channel`]).
pub struct StaSender<T>(std::sync::Arc<StaShared<T>>);

/// The STA worker's end of [`sta_channel`].
pub struct StaReceiver<T>(std::sync::Arc<StaShared<T>>);

/// A queue for a long-lived STA worker: [`StaReceiver::recv`] waits for the next item with
/// MsgWaitForMultipleObjects and dispatches the thread's messages meanwhile, so COM's hidden
/// window there answers (another program's broadcast SendMessage would stall on it otherwise).
pub fn sta_channel<T>() -> (StaSender<T>, StaReceiver<T>) {
    // SAFETY: plain FFI call; an unnamed auto-reset event.
    let event = unsafe { CreateEventW(None, false, false, windows_core::PCWSTR::null()) };
    let shared = std::sync::Arc::new(StaShared {
        items: std::sync::Mutex::new(std::collections::VecDeque::new()),
        event: event.0 as isize,
    });
    (StaSender(shared.clone()), StaReceiver(shared))
}

impl<T> StaSender<T> {
    pub fn send(&self, item: T) {
        self.0
            .items
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push_back(item);
        // SAFETY: the event lives as long as `self.0`.
        unsafe {
            let _ = SetEvent(HANDLE(self.0.event as *mut core::ffi::c_void));
        }
    }
}

impl<T> StaReceiver<T> {
    /// The next item, pumping the thread's messages while there is none.
    pub fn recv(&self) -> T {
        windows_core::link!("user32.dll" "system" fn MsgWaitForMultipleObjects(count: u32, handles: *const HANDLE, all: windows_core::BOOL, ms: u32, mask: u32) -> u32);
        const QS_ALLINPUT: u32 = 0x04FF;
        loop {
            if let Some(item) = self
                .0
                .items
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .pop_front()
            {
                return item;
            }
            let event = HANDLE(self.0.event as *mut core::ffi::c_void);
            // SAFETY: one live event handle; `msg` is a live out-struct for PeekMessageW.
            unsafe {
                MsgWaitForMultipleObjects(1, &event, false.into(), u32::MAX, QS_ALLINPUT);
                let mut msg = MSG::default();
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE as u32).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
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
