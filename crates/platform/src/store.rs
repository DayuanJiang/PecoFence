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

/// Whether Windows would show a notification now: no full-screen app or presentation, nobody
/// away from a locked screen. A fair moment to ask for something. Windows 11's Do not disturb
/// and Focus sessions are not reported here (measured: the state stays at accepts).
pub fn user_accepts_notifications() -> bool {
    // SAFETY: plain FFI call with an out parameter.
    unsafe { SHQueryUserNotificationState() }.is_ok_and(|state| state == QUNS_ACCEPTS_NOTIFICATIONS)
}

/// Windows' startup switch for the package: the manifest's `PecoFenceStartup` task, shown in
/// Settings > Apps > Startup and on Task Manager's Startup tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum StartupState {
    Disabled = 1,
    /// Turned off in Windows' Startup settings or Task Manager: only the user can turn it on
    /// again, there.
    DisabledByUser = 2,
    Enabled = 3,
    /// Turned off by an administrator or group policy.
    DisabledByPolicy = 4,
    /// Turned on by an administrator or group policy.
    EnabledByPolicy = 5,
}

impl StartupState {
    const ALL: [Self; 5] = [
        Self::Disabled,
        Self::DisabledByUser,
        Self::Enabled,
        Self::DisabledByPolicy,
        Self::EnabledByPolicy,
    ];

    pub fn is_on(self) -> bool {
        matches!(self, Self::Enabled | Self::EnabledByPolicy)
    }

    /// The state stored as `state as u8` (in an atomic between threads); 0 is none.
    pub fn from_code(code: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|s| *s as u8 == code)
    }

    fn from_task(state: StartupTaskState) -> Self {
        match state {
            StartupTaskState::DisabledByUser => Self::DisabledByUser,
            StartupTaskState::Enabled => Self::Enabled,
            StartupTaskState::DisabledByPolicy => Self::DisabledByPolicy,
            StartupTaskState::EnabledByPolicy => Self::EnabledByPolicy,
            _ => Self::Disabled,
        }
    }
}

fn startup_task() -> Result<StartupTask> {
    // The activation factory is cached for the process. Keep the MTA it lives in alive, or the
    // worker leaving it tears it down and the next call crashes in combase.
    static MTA: std::sync::Once = std::sync::Once::new();
    MTA.call_once(|| {
        windows_core::link!("ole32.dll" "system" fn CoIncrementMTAUsage(cookie: *mut *mut core::ffi::c_void) -> HRESULT);
        let mut cookie = core::ptr::null_mut();
        // SAFETY: out-pointer to a local; the usage is never released.
        let _ = unsafe { CoIncrementMTAUsage(&mut cookie) };
    });
    StartupTask::GetAsync(&windows_core::HSTRING::from("PecoFenceStartup"))?.join()
}

/// The state of Windows' startup switch for the package. Waits for WinRT, so call it on a
/// worker.
pub fn startup_task_state() -> Result<StartupState> {
    Ok(StartupState::from_task(startup_task()?.State()?))
}

/// Turns Windows' startup switch for the package on or off where the app may: on from
/// `Disabled` (a packaged desktop app gets no consent prompt) and off from `Enabled`. The other
/// states are the user's or an administrator's and stay as they are. Returns the states before
/// and after. Waits for WinRT, so call it on a worker.
pub fn set_startup_task(on: bool) -> Result<(StartupState, StartupState)> {
    let task = startup_task()?;
    let before = StartupState::from_task(task.State()?);
    let after = match (on, before) {
        (true, StartupState::Disabled) => {
            StartupState::from_task(task.RequestEnableAsync()?.join()?)
        }
        (false, StartupState::Enabled) => {
            task.Disable()?;
            StartupState::from_task(task.State()?)
        }
        _ => before,
    };
    Ok((before, after))
}

#[cfg(test)]
mod tests {
    use super::StartupState;

    #[test]
    fn startup_state_codes_round_trip() {
        for state in StartupState::ALL {
            assert_eq!(StartupState::from_code(state as u8), Some(state));
        }
        assert_eq!(StartupState::from_code(0), None);
        assert_eq!(StartupState::from_code(6), None);
    }
}
