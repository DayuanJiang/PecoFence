//! The Microsoft Store's own rating dialog (`StoreContext.RequestRateAndReviewAppAsync`) and the
//! package's startup switch. Only the Store edition can use them: Windows finds the product
//! through the package identity, so an unpackaged process gets an error back.

use crate::HWND;
use crate::store_bindings::{
    self, IInitializeWithWindow, QUNS_ACCEPTS_NOTIFICATIONS, SHQueryUserNotificationState,
    StartupTask, StartupTaskState, StoreContext, StoreRateAndReviewStatus,
};
use windows_core::{HRESULT, Interface, Result};

const E_FAIL: HRESULT = HRESULT(0x8000_4005_u32 as i32);

/// How the rating dialog ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RatingOutcome {
    /// A rating or review was submitted; `updated` when it replaced the user's earlier one.
    Rated {
        updated: bool,
    },
    Cancelled,
    NetworkError,
    Failed(HRESULT),
}

/// Opens the rating dialog centred on `owner`, a window of the calling thread, which must pump
/// messages (Desktop Bridge apps fail with ERROR_INVALID_WINDOW_HANDLE otherwise). The dialog
/// only comes to the front while this process may set the foreground window, so call it in
/// response to the user (a click on our window or tray balloon). Returns once the dialog is on
/// its way; `done` runs on a system worker thread when it closes.
pub fn request_rating(
    owner: HWND,
    done: impl FnOnce(RatingOutcome) + Send + 'static,
) -> Result<()> {
    // The dialog belongs to Explorer / StoreExperienceHost; hand them our foreground right.
    // SAFETY: plain FFI call; fails harmlessly when we hold no such right.
    let _ = unsafe { store_bindings::AllowSetForegroundWindow(store_bindings::ASFW_ANY) };
    let context = StoreContext::GetDefault()?;
    let init: IInitializeWithWindow = context.cast()?;
    // SAFETY: plain COM call; the handle belongs to this thread and outlives the dialog.
    unsafe { init.Initialize(store_bindings::HWND(owner.0)) }.ok()?;
    let operation = context.RequestRateAndReviewAppAsync()?;
    operation.when(move |result| {
        let outcome = match result {
            Ok(r) => match r.Status() {
                Ok(StoreRateAndReviewStatus::Succeeded) => RatingOutcome::Rated {
                    updated: r.WasUpdated().unwrap_or(false),
                },
                Ok(StoreRateAndReviewStatus::CanceledByUser) => RatingOutcome::Cancelled,
                Ok(StoreRateAndReviewStatus::NetworkError) => RatingOutcome::NetworkError,
                _ => RatingOutcome::Failed(r.ExtendedError().unwrap_or(E_FAIL)),
            },
            Err(error) => RatingOutcome::Failed(error.code()),
        };
        done(outcome);
    })
}

/// Whether Windows would show a notification now: no full-screen app or presentation, no
/// Focus session, nobody away from a locked screen. A fair moment to ask for something.
pub fn user_accepts_notifications() -> bool {
    // SAFETY: plain FFI call with an out parameter.
    unsafe { SHQueryUserNotificationState() }.is_ok_and(|state| state == QUNS_ACCEPTS_NOTIFICATIONS)
}

/// Whether Windows' startup switch for the package (Settings > Apps > Startup, the manifest's
/// `PecoFenceStartup` task) is on. Waits for WinRT, so call it on a worker.
pub fn startup_task_enabled() -> Result<bool> {
    // The activation factory is cached for the process. Keep the MTA it lives in alive, or the
    // worker leaving it tears it down and the next call crashes in combase.
    static MTA: std::sync::Once = std::sync::Once::new();
    MTA.call_once(|| {
        windows_core::link!("ole32.dll" "system" fn CoIncrementMTAUsage(cookie: *mut *mut core::ffi::c_void) -> HRESULT);
        let mut cookie = core::ptr::null_mut();
        // SAFETY: out-pointer to a local; the usage is never released.
        let _ = unsafe { CoIncrementMTAUsage(&mut cookie) };
    });
    let task = StartupTask::GetAsync(&windows_core::HSTRING::from("PecoFenceStartup"))?.join()?;
    Ok(matches!(
        task.State()?,
        StartupTaskState::Enabled | StartupTaskState::EnabledByPolicy
    ))
}
