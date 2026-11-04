//! A Windows job object that ends every process in it when it is closed: a script fence's run
//! takes the programs its script started with it (they would otherwise keep running and hold
//! the output pipes open).

use crate::bindings::*;

#[link(name = "kernel32")]
unsafe extern "system" {
    // Not in the generated bindings.
    fn TerminateJobObject(job: HANDLE, exit_code: u32) -> windows_core::BOOL;
}

/// Every process assigned to it ends when it is dropped (or at [`Self::terminate`]).
pub struct KillOnCloseJob(HANDLE);

impl KillOnCloseJob {
    /// None when Windows refuses a job object.
    pub fn new() -> Option<Self> {
        // SAFETY: plain FFI; the info struct outlives the call and its size is passed.
        unsafe {
            let job = CreateJobObjectW(None, windows_core::PCWSTR::null());
            if job.0.is_null() {
                return None;
            }
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE as _;
            let ok = SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            if !ok.as_bool() {
                let _ = CloseHandle(job);
                return None;
            }
            Some(Self(job))
        }
    }

    /// Puts `child` (and what it starts from now on) in the job.
    pub fn assign(&self, child: &std::process::Child) -> bool {
        use std::os::windows::io::AsRawHandle;
        // SAFETY: the child's process handle stays open while `child` lives.
        unsafe { AssignProcessToJobObject(self.0, HANDLE(child.as_raw_handle())).as_bool() }
    }

    /// Ends every process in the job now.
    pub fn terminate(&self) {
        // SAFETY: our own job handle.
        unsafe {
            let _ = TerminateJobObject(self.0, 1);
        }
    }
}

impl Drop for KillOnCloseJob {
    fn drop(&mut self) {
        // SAFETY: we own the handle; closing it ends the processes still in the job.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

// SAFETY: a job handle may be used and closed from any thread.
unsafe impl Send for KillOnCloseJob {}
