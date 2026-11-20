//! A line in the log when PecoFence keeps the processor busy (GitHub #241: on one machine a
//! quarter of a core went to PecoFence all the time, and its log could not say what did it).
//! Checked with the housekeeping tick once a minute: when the last stretch took 10 % of one
//! core or more, the line says which threads used it, what the UI thread handled most and
//! how many frames were drawn. At most one line every 15 minutes.

use super::App;
use pecofence_platform::busy;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// The share of one core over the last stretch from which the line is written.
const BUSY_PERCENT: f64 = 10.0;
const QUIET_AFTER_LINE: Duration = Duration::from_secs(15 * 60);

#[derive(Default)]
pub(super) struct BusyWatch {
    since: Option<(Instant, Duration)>,
    threads: HashMap<u32, Duration>,
    logged: Option<Instant>,
    /// Frames drawn since the last check (`App::on_frame`).
    pub(super) frames: u32,
}

/// Percent of one core that `used` is of `over`.
fn percent(used: Duration, over: Duration) -> f64 {
    if over.is_zero() {
        return 0.0;
    }
    (used.as_secs_f64() / over.as_secs_f64() * 1000.0).round() / 10.0
}

impl App {
    pub(super) fn check_busy(&mut self) {
        let now = Instant::now();
        let Some(cpu) = busy::process_cpu() else {
            return;
        };
        let threads = busy::thread_cpu();
        let ui = busy::current_thread_id();
        let watch = &mut self.busy;
        let frames = std::mem::take(&mut watch.frames);
        let tally = busy::take_tally(4);
        let before = std::mem::take(&mut watch.threads);
        watch.threads = threads.iter().map(|(id, _, t)| (*id, *t)).collect();
        let Some((at, cpu_before)) = watch.since.replace((now, cpu)) else {
            return;
        };
        let over = now.duration_since(at);
        let used = percent(cpu.saturating_sub(cpu_before), over);
        let busy_line = used >= BUSY_PERCENT
            && over >= Duration::from_secs(30)
            && !watch.logged.is_some_and(|t| t.elapsed() < QUIET_AFTER_LINE);
        // The same facts at debug level every minute (the test boxes log at debug).
        if !busy_line && !tracing::enabled!(tracing::Level::DEBUG) {
            return;
        }
        if busy_line {
            watch.logged = Some(now);
        }
        // Threads of one name (the software renderer's workers, say) count together.
        let mut by_name: HashMap<String, (Duration, u32)> = HashMap::new();
        for (id, name, t) in &threads {
            let label = if *id == ui {
                "ui".to_string()
            } else if name.is_empty() {
                "unnamed".to_string()
            } else {
                name.clone()
            };
            let was = before.get(id).copied().unwrap_or_default();
            let entry = by_name.entry(label).or_default();
            entry.0 += t.saturating_sub(was);
            entry.1 += 1;
        }
        let mut by_thread: Vec<(String, u32, f64)> = by_name
            .into_iter()
            .map(|(label, (t, n))| (label, n, percent(t, over)))
            .filter(|(.., p)| *p >= 0.5)
            .collect();
        by_thread.sort_by(|a, b| b.2.total_cmp(&a.2));
        let threads_text = by_thread
            .iter()
            .take(4)
            .map(|(label, n, p)| match n {
                1 => format!("{label} {p} %"),
                _ => format!("{label} ({n} threads) {p} %"),
            })
            .collect::<Vec<_>>()
            .join(", ");
        let frame_run_s = self
            .frame_run
            .started
            .filter(|_| self.frame_prev.is_some())
            .map(|t| t.elapsed().as_secs());
        let cores = std::thread::available_parallelism().map_or(0, |n| n.get());
        let handled = tally.join(", ");
        if busy_line {
            tracing::info!(
                percent_of_a_core = used,
                cores,
                seconds = over.as_secs(),
                threads = %threads_text,
                ui_thread_handled = %handled,
                frames,
                frame_run_s,
                "busy: PecoFence used the processor"
            );
        } else {
            tracing::debug!(
                percent_of_a_core = used,
                seconds = over.as_secs(),
                threads = %threads_text,
                ui_thread_handled = %handled,
                frames,
                "processor over the last minute"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::percent;
    use std::time::Duration;

    #[test]
    fn percent_is_of_one_core_with_one_decimal() {
        let s = Duration::from_secs;
        assert_eq!(percent(Duration::from_millis(15_000), s(60)), 25.0);
        assert_eq!(percent(Duration::from_millis(61), s(60)), 0.1);
        assert_eq!(percent(s(1), Duration::ZERO), 0.0);
    }
}
