//! Idle display/sleep prevention for the lifetime of the app's UI thread.

use crate::bindings::*;

/// Set or clear this thread's continuous display and system power requirements.
/// Call on the UI thread at startup, on a settings change and at shutdown. Windows
/// also releases the requirements when the thread exits, including after a crash.
/// Explicit sleep and screen-saver settings remain under the user's control.
pub fn keep_screen_on(enabled: bool) -> bool {
    let flags = if enabled {
        ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED
    } else {
        ES_CONTINUOUS
    };
    // SAFETY: valid execution-state flags; only affects the calling thread.
    let success = unsafe { SetThreadExecutionState(EXECUTION_STATE(flags)).0 != 0 };
    if success {
        tracing::info!(enabled, "keep screen on applied");
    } else {
        tracing::warn!(enabled, "keep screen on failed");
    }
    success
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_requirements_are_set_and_cleared_on_the_same_thread() {
        // A dedicated thread keeps the test runner's own execution state untouched.
        std::thread::spawn(|| {
            assert!(keep_screen_on(true));
            // SAFETY: clear this test thread's requirements and inspect the prior state.
            let previous = unsafe { SetThreadExecutionState(EXECUTION_STATE(ES_CONTINUOUS)) };
            assert_eq!(previous.0 & ES_DISPLAY_REQUIRED, ES_DISPLAY_REQUIRED);
            assert_eq!(previous.0 & ES_SYSTEM_REQUIRED, ES_SYSTEM_REQUIRED);
            assert!(keep_screen_on(true));
            assert!(keep_screen_on(false));
            // SAFETY: same thread, valid flags; the requirements should already be clear.
            let previous = unsafe { SetThreadExecutionState(EXECUTION_STATE(ES_CONTINUOUS)) };
            assert_eq!(previous.0 & (ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED), 0);
        })
        .join()
        .unwrap();
    }
}
