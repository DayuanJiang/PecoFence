//! Short, non-postponing wallpaper checks, the worker that reads the wallpaper for them, and a
//! bounded cache for desktop round trips.

use crate::commands::{WM_APP_WALLPAPER, WM_APP_WALLPAPER_READ};
use crate::fence_window::BackdropSets;
use pecofence_platform::wallpaper::{self, WallpaperSnapshot};
use pecofence_platform::{HWND, window};
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
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

/// A read that waited this long for Explorer is logged (once while it waits, once answered).
const LATE_READ: Duration = Duration::from_secs(1);

/// What a check is for (`App::follow_snapshot` reads it): the 5 s poll, a change signal, a
/// rebuild after a theme or display change, a live wallpaper capture that needs new backdrops.
pub(super) const POLL: &str = "poll";
pub(super) const NOTIFICATION: &str = "notification";
pub(super) const REFRESH: &str = "refresh";

/// Reads the wallpaper (`IDesktopWallpaper`, COM calls into Explorer) on a worker thread of its
/// own. Explorer answers within a few ms, but while it is busy (1500 new desktop files, its
/// desktop thread stuck) it answered after 20 to 99 s, and every fence froze that long when the
/// UI thread asked. One read is out at a time: a poll while one is out is skipped, any other
/// check runs once the answer is back.
pub(super) struct WallpaperReader {
    requests: Option<Sender<()>>,
    answers: Receiver<std::result::Result<WallpaperSnapshot, String>>,
    /// The read out now: when it started and the check it is for.
    pending: Option<(Instant, &'static str)>,
    /// A check asked for while `pending` was out.
    again: Option<&'static str>,
    /// `pending` waits longer than `LATE_READ` and was logged.
    reported: bool,
}

impl WallpaperReader {
    /// Starts the worker; it posts `WM_APP_WALLPAPER_READ` to `control` after every read.
    pub(super) fn start(control: HWND, wallpaper_override: Option<String>) -> Self {
        let (requests, jobs) = mpsc::channel::<()>();
        let (answer, answers) = mpsc::channel();
        let control = control.0 as isize;
        let spawned = std::thread::Builder::new()
            .name("pecofence-wallpaper-read".into())
            .spawn(move || {
                let _com = pecofence_platform::com::MtaGuard::init();
                while jobs.recv().is_ok() {
                    let read = super::visuals::wallpaper_snapshot(wallpaper_override.as_deref())
                        .map_err(|error| error.to_string());
                    if answer.send(read).is_err() {
                        break;
                    }
                    window::post_message(
                        HWND(control as *mut core::ffi::c_void),
                        WM_APP_WALLPAPER_READ,
                        0,
                        0,
                    );
                }
            });
        if let Err(error) = &spawned {
            tracing::warn!(%error, "wallpaper reader thread failed to start; wallpaper not followed");
        }
        Self {
            requests: spawned.is_ok().then_some(requests),
            answers,
            pending: None,
            again: None,
            reported: false,
        }
    }

    pub(super) fn is_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Asks for a read for the check `reason`. While one is out, a poll is skipped and any
    /// other check is kept for `take_again` (a signal before a poll).
    pub(super) fn request(&mut self, reason: &'static str) {
        if let Some((started, _)) = self.pending {
            if reason != POLL {
                self.queue(reason);
            }
            if started.elapsed() >= LATE_READ && !self.reported {
                self.reported = true;
                // `reason`: the check that waits (the read's own comes with its answer).
                tracing::info!(
                    waited_ms = started.elapsed().as_millis(),
                    reason,
                    "wallpaper read still waits for Explorer; fences go on, checks wait for it"
                );
            }
            return;
        }
        let Some(requests) = &self.requests else {
            return;
        };
        if requests.send(()).is_ok() {
            self.pending = Some((Instant::now(), reason));
            self.reported = false;
        }
    }

    /// Keeps `reason` for after the read out now; a signal's check wins over the others.
    pub(super) fn queue(&mut self, reason: &'static str) {
        if self.again != Some(NOTIFICATION) {
            self.again = Some(reason);
        }
    }

    /// The check kept while a read was out, once none is.
    pub(super) fn take_again(&mut self) -> Option<&'static str> {
        if self.pending.is_some() {
            return None;
        }
        self.again.take()
    }

    /// The answer of the read out now with the check it was for: at once if it has come,
    /// else after waiting `wait` at most. None when nothing is out or it has not come.
    pub(super) fn answer(
        &mut self,
        wait: Duration,
    ) -> Option<(&'static str, std::result::Result<WallpaperSnapshot, String>)> {
        let (started, reason) = self.pending?;
        let read = if wait.is_zero() {
            self.answers
                .try_recv()
                .map_err(|e| e == mpsc::TryRecvError::Disconnected)
        } else {
            self.answers
                .recv_timeout(wait)
                .map_err(|e| e == mpsc::RecvTimeoutError::Disconnected)
        };
        let read = match read {
            Ok(read) => read,
            Err(gone) => {
                if gone {
                    tracing::warn!("wallpaper reader thread ended; wallpaper not followed");
                    self.pending = None;
                    self.requests = None;
                }
                return None;
            }
        };
        self.pending = None;
        let waited = started.elapsed();
        if waited >= LATE_READ {
            tracing::info!(
                reason,
                waited_ms = waited.as_millis(),
                "wallpaper read answered late"
            );
        }
        Some((reason, read))
    }
}

/// Asks which virtual desktop the foreground window is on from a worker thread
/// (`IVirtualDesktopManager`, a COM call into Explorer) and posts `WM_APP_WALLPAPER` (wparam
/// 2) to `control` when it is not the desktop of the last window asked about. `Some(hwnd)`
/// asks about a window, `None` forgets the last desktop.
pub(super) fn start_foreground_desktops(control: HWND) -> Option<Sender<Option<isize>>> {
    let (send, jobs) = mpsc::channel::<Option<isize>>();
    let control = control.0 as isize;
    let spawned = std::thread::Builder::new()
        .name("pecofence-window-desktop".into())
        .spawn(move || {
            let _com = pecofence_platform::com::MtaGuard::init();
            let mut last: Option<u128> = None;
            while let Ok(first) = jobs.recv() {
                // Only the newest window counts; a `None` on the way forgets the last desktop.
                let mut window = None;
                for job in std::iter::once(first).chain(jobs.try_iter()) {
                    window = job;
                    if job.is_none() {
                        last = None;
                    }
                }
                let Some(hwnd) = window else {
                    continue;
                };
                let desktop = wallpaper::window_desktop(HWND(hwnd as *mut core::ffi::c_void));
                // Unknown (the desktop itself, say): the next one counts again.
                let same = desktop.is_some() && last == desktop;
                last = desktop;
                if !same {
                    window::post_message(
                        HWND(control as *mut core::ffi::c_void),
                        WM_APP_WALLPAPER,
                        2,
                        0,
                    );
                }
            }
        });
    match spawned {
        Ok(_) => Some(send),
        Err(error) => {
            tracing::warn!(%error, "window desktop thread failed to start");
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

    /// A reader without its worker: the test answers through `answer_tx`.
    fn reader() -> (
        WallpaperReader,
        Receiver<()>,
        Sender<std::result::Result<WallpaperSnapshot, String>>,
    ) {
        let (requests, jobs) = mpsc::channel();
        let (answer_tx, answers) = mpsc::channel();
        let reader = WallpaperReader {
            requests: Some(requests),
            answers,
            pending: None,
            again: None,
            reported: false,
        };
        (reader, jobs, answer_tx)
    }

    #[test]
    fn one_read_at_a_time_and_a_poll_is_skipped_while_one_is_out() {
        let (mut reader, jobs, answer) = reader();
        reader.request(POLL);
        reader.request(POLL);
        reader.request(POLL);
        assert_eq!(jobs.try_iter().count(), 1);
        assert!(reader.answer(Duration::ZERO).is_none());
        assert!(reader.is_pending());
        answer.send(Err("busy".into())).unwrap();
        let (reason, read) = reader.answer(Duration::ZERO).unwrap();
        assert_eq!(reason, POLL);
        assert!(read.is_err());
        // Polls skipped meanwhile do not run afterwards.
        assert_eq!(reader.take_again(), None);
    }

    #[test]
    fn a_signal_while_a_read_is_out_runs_once_it_is_back() {
        let (mut reader, jobs, answer) = reader();
        reader.request(POLL);
        reader.request(NOTIFICATION);
        reader.request(REFRESH);
        reader.request(POLL);
        assert_eq!(jobs.try_iter().count(), 1);
        // Still out: nothing to run yet.
        assert_eq!(reader.take_again(), None);
        answer.send(Err("busy".into())).unwrap();
        assert_eq!(reader.answer(Duration::ZERO).unwrap().0, POLL);
        // The signal's check wins over the rebuild's and runs once.
        assert_eq!(reader.take_again(), Some(NOTIFICATION));
        assert_eq!(reader.take_again(), None);
        reader.request(NOTIFICATION);
        assert_eq!(jobs.try_iter().count(), 1);
    }

    #[test]
    fn a_reader_whose_worker_ended_stops_asking() {
        let (mut reader, jobs, answer) = reader();
        reader.request(POLL);
        drop(answer);
        assert!(reader.answer(Duration::from_millis(10)).is_none());
        assert!(!reader.is_pending());
        reader.request(NOTIFICATION);
        assert_eq!(jobs.try_iter().count(), 1);
        assert!(!reader.is_pending());
    }

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
            ..Default::default()
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
            ..Default::default()
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
