//! Short, non-postponing wallpaper checks and a bounded cache for desktop round trips.

use crate::fence_window::BackdropSets;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub(super) const FIRST_CHECK_MS: u32 = 50;
const RETRY_MS: u32 = 100;
const SETTLE_MS: u64 = 2_000;

#[derive(Clone, Copy, Default)]
pub(super) struct RefreshSchedule {
    next: Option<Instant>,
    settle_until: Option<Instant>,
}

impl RefreshSchedule {
    /// Additional signals may bring a check forward, but must never postpone it.
    pub(super) fn notify(&mut self, now: Instant) -> Option<u32> {
        self.settle_until = Some(now + Duration::from_millis(SETTLE_MS));
        let next = now + Duration::from_millis(FIRST_CHECK_MS as u64);
        if self.next.is_some_and(|pending| pending <= next) {
            return None;
        }
        self.next = Some(next);
        Some(FIRST_CHECK_MS)
    }

    /// Even an unchanged/successful read needs follow-ups: Explorer can publish the
    /// desktop identity, picture path and transcoded file at different times.
    pub(super) fn checked(&mut self, now: Instant) -> Option<u32> {
        if self.settle_until.is_some_and(|until| now < until) {
            self.next = Some(now + Duration::from_millis(RETRY_MS as u64));
            Some(RETRY_MS)
        } else {
            self.next = None;
            self.settle_until = None;
            None
        }
    }
}

struct Entry {
    signature: String,
    backdrops: Rc<BackdropSets>,
    bytes: usize,
}

pub(super) struct BackdropCache {
    entries: VecDeque<Entry>,
    bytes: usize,
    max_entries: usize,
    max_bytes: usize,
}

impl Default for BackdropCache {
    fn default() -> Self {
        Self::with_limits(4, 128 * 1024 * 1024)
    }
}

impl BackdropCache {
    fn with_limits(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            bytes: 0,
            max_entries,
            max_bytes,
        }
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
    }

    pub(super) fn get(&mut self, signature: &str) -> Option<Rc<BackdropSets>> {
        let index = self.entries.iter().position(|e| e.signature == signature)?;
        let entry = self.entries.remove(index)?;
        let backdrops = entry.backdrops.clone();
        self.entries.push_front(entry);
        Some(backdrops)
    }

    pub(super) fn insert(&mut self, signature: String, backdrops: Rc<BackdropSets>) {
        if let Some(index) = self.entries.iter().position(|e| e.signature == signature) {
            self.bytes -= self.entries.remove(index).unwrap().bytes;
        }
        let bytes = backdrops.acrylic.iter().map(|b| b.image.bgra.len()).sum();
        // Very large monitor sets can still be displayed without retaining another copy.
        if self.max_entries == 0 || bytes > self.max_bytes {
            return;
        }
        while self.entries.len() >= self.max_entries || self.bytes + bytes > self.max_bytes {
            let Some(oldest) = self.entries.pop_back() else {
                break;
            };
            self.bytes -= oldest.bytes;
        }
        self.bytes += bytes;
        self.entries.push_front(Entry {
            signature,
            backdrops,
            bytes,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pecofence_render::{Image, MonitorBackdrop};

    #[test]
    fn notification_storm_does_not_postpone_first_check() {
        let start = Instant::now();
        let mut schedule = RefreshSchedule::default();
        assert_eq!(schedule.notify(start), Some(FIRST_CHECK_MS));
        for ms in 1..50 {
            assert_eq!(schedule.notify(start + Duration::from_millis(ms)), None);
        }
        assert_eq!(schedule.next, Some(start + Duration::from_millis(50)));
    }

    #[test]
    fn early_reads_keep_retrying_then_stop_when_explorer_settles() {
        let start = Instant::now();
        let mut schedule = RefreshSchedule::default();
        schedule.notify(start);
        // These reads may all see the old wallpaper; a later read must still run.
        for ms in (50..2_000).step_by(100) {
            assert_eq!(
                schedule.checked(start + Duration::from_millis(ms)),
                Some(RETRY_MS)
            );
        }
        assert_eq!(schedule.checked(start + Duration::from_millis(2_050)), None);
        assert_eq!(schedule.notify(start + Duration::from_secs(3)), Some(50));
    }

    #[test]
    fn another_desktop_switch_accelerates_a_retry_and_extends_settling() {
        let start = Instant::now();
        let mut schedule = RefreshSchedule::default();
        schedule.notify(start);
        schedule.checked(start + Duration::from_millis(50));
        assert_eq!(schedule.notify(start + Duration::from_millis(60)), Some(50));
        assert_eq!(schedule.next, Some(start + Duration::from_millis(110)));
        assert_eq!(
            schedule.checked(start + Duration::from_millis(2_010)),
            Some(RETRY_MS)
        );
        assert_eq!(schedule.checked(start + Duration::from_millis(2_110)), None);
    }

    fn backdrop(pixels: u32) -> Rc<BackdropSets> {
        Rc::new(BackdropSets {
            acrylic: Rc::new(vec![MonitorBackdrop {
                left: 0,
                top: 0,
                width: pixels as i32,
                height: 1,
                downscale: 1,
                image: Image::solid(pixels, 1, [1, 2, 3]),
            }]),
        })
    }

    #[test]
    fn desktop_round_trip_reuses_pixels_and_evicts_the_least_recently_used() {
        let mut cache = BackdropCache::with_limits(2, 64);
        let first = backdrop(2);
        cache.insert("desktop-a".into(), first.clone());
        cache.insert("desktop-b".into(), backdrop(2));
        assert!(Rc::ptr_eq(&cache.get("desktop-a").unwrap(), &first));
        cache.insert("desktop-c".into(), backdrop(2));
        assert!(cache.get("desktop-b").is_none());
        assert!(cache.get("desktop-a").is_some());
    }

    #[test]
    fn memory_budget_limits_retained_monitor_images() {
        let mut cache = BackdropCache::with_limits(4, 16);
        cache.insert("a".into(), backdrop(2));
        cache.insert("b".into(), backdrop(3));
        assert!(cache.get("a").is_none());
        assert!(cache.get("b").is_some());
        cache.insert("too-large".into(), backdrop(5));
        assert!(cache.get("too-large").is_none());
        assert!(cache.get("b").is_some());
        cache.insert("b".into(), backdrop(1));
        assert_eq!(cache.bytes, 4);
        cache.clear();
        assert!(cache.get("b").is_none());
        assert_eq!(cache.bytes, 0);
    }
}
