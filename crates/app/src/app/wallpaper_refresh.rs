//! Short, non-postponing wallpaper checks and a bounded cache for desktop round trips.

use crate::fence_window::BackdropSets;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub(super) const FIRST_CHECK_MS: u32 = 50;
const RETRY_MS: u32 = 100;
const SETTLE_MS: u64 = 2_000;
/// Starts a monitor's part of a backdrop signature whose backdrop shows a live wallpaper's
/// capture (no path holds `|`).
pub(super) const LIVE_MARK: &str = "|live@";

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

/// A signature without the per-file `#size#mtime` stamps (see `WallpaperSnapshot::signature`):
/// equal for two reads of the same files in the same layout.
fn file_identity(signature: &str) -> String {
    signature
        .split('|')
        .map(|segment| {
            let mut parts = segment.rsplitn(3, '#');
            match (parts.next(), parts.next(), parts.next()) {
                (Some(time), Some(size), Some(rest))
                    if !time.is_empty()
                        && !size.is_empty()
                        && time.bytes().all(|b| b.is_ascii_digit())
                        && size.bytes().all(|b| b.is_ascii_digit()) =>
                {
                    rest
                }
                _ => segment,
            }
        })
        .collect::<Vec<_>>()
        .join("|")
}

/// `new` shows the same wallpaper files as `old` with different contents (a slideshow or an
/// app rewriting the picture in place): `old`'s pixels cannot come back.
pub(super) fn supersedes(old: &str, new: &str) -> bool {
    old != new && file_identity(old) == file_identity(new)
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
    /// Bumped by `clear` (theme / layout change): work started for an older generation
    /// (a wallpaper prepared in advance) must not be inserted.
    generation: u64,
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
            generation: 0,
        }
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
        self.generation += 1;
    }

    pub(super) fn generation(&self) -> u64 {
        self.generation
    }

    /// Windows paths and monitor ids are case-insensitive, and Explorer does not keep one
    /// spelling (per-desktop registry values differ in case for the same picture).
    fn position(&self, signature: &str) -> Option<usize> {
        self.entries
            .iter()
            .position(|e| e.signature.eq_ignore_ascii_case(signature))
    }

    pub(super) fn contains(&self, signature: &str) -> bool {
        self.position(signature).is_some()
    }

    /// Room for `bytes` more without evicting anything.
    pub(super) fn has_room(&self, bytes: usize) -> bool {
        self.entries.len() < self.max_entries && self.bytes + bytes <= self.max_bytes
    }

    pub(super) fn get(&mut self, signature: &str) -> Option<Rc<BackdropSets>> {
        let index = self.position(signature)?;
        let entry = self.entries.remove(index)?;
        let backdrops = entry.backdrops.clone();
        self.entries.push_front(entry);
        Some(backdrops)
    }

    pub(super) fn remove(&mut self, signature: &str) {
        if let Some(index) = self.position(signature) {
            self.bytes -= self.entries.remove(index).unwrap().bytes;
        }
    }

    /// A GPU-backed set keeps its full-resolution image as a texture (or, before the upload,
    /// in memory): count that, not just the small sampling copy.
    fn set_bytes(backdrops: &BackdropSets) -> usize {
        backdrops
            .acrylic
            .iter()
            .map(|b| b.image.bgra.len() + b.gpu.as_ref().map_or(0, |g| g.full_bytes()))
            .sum()
    }

    /// Inserts a set prepared ahead of use only if nothing has to be evicted for it: the
    /// sets of desktops already visited stay. Returns whether it was kept.
    pub(super) fn insert_if_room(
        &mut self,
        signature: String,
        backdrops: Rc<BackdropSets>,
    ) -> bool {
        if self.contains(&signature) || !self.has_room(Self::set_bytes(&backdrops)) {
            return false;
        }
        self.insert(signature, backdrops);
        true
    }

    pub(super) fn insert(&mut self, signature: String, backdrops: Rc<BackdropSets>) {
        self.remove(&signature);
        let bytes = Self::set_bytes(&backdrops);
        // Very large monitor sets can still be displayed without retaining another copy. A
        // live wallpaper's capture is never shown again once replaced.
        if self.max_entries == 0 || bytes > self.max_bytes || signature.contains(LIVE_MARK) {
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
                gpu: None,
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
        // A set showing a live wallpaper's capture is never kept.
        cache.insert(format!("b{LIVE_MARK}0,0,1,1"), backdrop(1));
        assert_eq!(cache.entries.len(), 1);
        cache.clear();
        assert!(cache.get("b").is_none());
        assert_eq!(cache.bytes, 0);
    }

    #[test]
    fn sets_prepared_in_advance_never_evict_and_match_any_case() {
        let mut cache = BackdropCache::with_limits(3, 20);
        cache.insert("current".into(), backdrop(2));
        assert!(cache.insert_if_room(r"C:\Web\img28.jpg".into(), backdrop(2)));
        // Same picture spelled differently by another desktop's registry value.
        assert!(cache.contains(r"c:\web\IMG28.jpg"));
        assert!(!cache.insert_if_room(r"c:\web\img28.jpg".into(), backdrop(1)));
        // Over the byte cap: refused, nothing evicted.
        assert!(!cache.insert_if_room("big".into(), backdrop(4)));
        assert!(cache.contains("current"));
        assert!(cache.insert_if_room("small".into(), backdrop(1)));
        // Over the entry cap.
        assert!(!cache.insert_if_room("fourth".into(), backdrop(0)));
        let generation = cache.generation();
        cache.clear();
        assert_ne!(cache.generation(), generation);
    }

    #[test]
    fn gpu_backed_sets_count_their_full_resolution_image() {
        let full = MonitorBackdrop {
            left: 0,
            top: 0,
            width: 16,
            height: 2,
            downscale: 1,
            image: Image::solid(16, 2, [1, 2, 3]),
            gpu: None,
        }
        .into_gpu_backed(8, || None);
        let set = Rc::new(BackdropSets {
            acrylic: Rc::new(vec![full]),
        });
        let mut cache = BackdropCache::with_limits(4, 1024);
        cache.insert("a".into(), set.clone());
        // 2x1 sampling copy + the 16x2 image the GPU upload owns.
        assert_eq!(cache.bytes, 2 * 4 + 16 * 2 * 4);
        // Still counted once the upload took the pixels: they now live in the texture.
        set.acrylic[0].gpu.as_ref().unwrap().take();
        cache.insert("a".into(), set);
        assert_eq!(cache.bytes, 2 * 4 + 16 * 2 * 4);
        cache.remove("a");
        assert!(cache.get("a").is_none());
        assert_eq!(cache.bytes, 0);
        cache.remove("missing");
        assert_eq!(cache.bytes, 0);
    }

    #[test]
    fn only_the_same_files_with_new_contents_supersede_a_set() {
        let sig = |path: &str, stamp: &str| {
            format!("Fill|[0, 0, 0]|DISPLAY#GSM5B09#5&1#{{e6f07b5f}}@0,0,3840,2160={path}{stamp}")
        };
        let old = sig("C:/wall#1.jpg", "#100#5");
        // Slideshow rewriting the same file: stale.
        assert!(supersedes(&old, &sig("C:/wall#1.jpg", "#120#6")));
        // Another picture (another desktop, or the user picked a new one): kept.
        assert!(!supersedes(&old, &sig("C:/other.jpg", "#100#5")));
        // Unchanged, or a file whose metadata could not be read.
        assert!(!supersedes(&old, &old));
        assert!(supersedes(&old, &sig("C:/wall#1.jpg", "")));
        // A new layout of the same files is a different set, not a stale one.
        assert!(!supersedes(&old, &old.replace("Fill", "Fit")));
        // Solid colour monitors (no path) and multi-monitor signatures.
        assert!(!supersedes(
            "Fill|[1, 2, 3]|m@0,0,1,1",
            "Fill|[4, 5, 6]|m@0,0,1,1"
        ));
        let two =
            |a: &str, b: &str| format!("Fill|[0, 0, 0]|m1@0,0,1,1=x.jpg{a}|m2@1,0,2,1=y.jpg{b}");
        assert!(supersedes(&two("#1#1", "#2#2"), &two("#1#1", "#3#3")));
    }
}
