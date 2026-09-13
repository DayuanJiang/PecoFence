//! Process memory statistics, using the two counters that matter:
//! private working set (what Task Manager shows) and private commit.

use crate::bindings::*;
use windows_core::Result;

#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryStats {
    /// Private working set in bytes (Task Manager "Memory (active private working set)").
    pub private_working_set: usize,
    /// Total working set in bytes (private + shared pages resident).
    pub working_set: usize,
    /// Peak working set in bytes.
    pub peak_working_set: usize,
    /// Private commit charge in bytes (`PrivateUsage`).
    pub private_commit: usize,
    /// Shared commit in bytes (section objects, incl. GPU shared surfaces).
    pub shared_commit: u64,
    /// Page fault count since process start.
    pub page_faults: u32,
}

impl MemoryStats {
    pub fn current() -> Result<Self> {
        let mut counters = PROCESS_MEMORY_COUNTERS_EX2 {
            cb: size_of::<PROCESS_MEMORY_COUNTERS_EX2>() as u32,
            ..Default::default()
        };
        // SAFETY: the EX2 structure is a prefix-compatible superset of PROCESS_MEMORY_COUNTERS
        // and `cb` tells the API how much we can receive.
        unsafe {
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                (&mut counters as *mut PROCESS_MEMORY_COUNTERS_EX2).cast(),
                counters.cb,
            )
            .ok()?;
        }
        Ok(Self {
            private_working_set: counters.PrivateWorkingSetSize,
            working_set: counters.WorkingSetSize,
            peak_working_set: counters.PeakWorkingSetSize,
            private_commit: counters.PrivateUsage,
            shared_commit: counters.SharedCommitUsage,
            page_faults: counters.PageFaultCount,
        })
    }

    pub fn summary(&self) -> String {
        const MB: f64 = 1024.0 * 1024.0;
        format!(
            "private WS {:.1} MB | WS {:.1} MB (peak {:.1}) | private commit {:.1} MB | shared commit {:.1} MB",
            self.private_working_set as f64 / MB,
            self.working_set as f64 / MB,
            self.peak_working_set as f64 / MB,
            self.private_commit as f64 / MB,
            self.shared_commit as f64 / MB,
        )
    }
}
