//! Icon/thumbnail cache. Extraction runs on MTA worker threads; the UI thread gets a
//! `WM_APP_ICON_READY` poke and drains finished results.

use pecofence_core::IconKey;
use pecofence_platform::HWND;
use pecofence_platform::com::OleGuard;
use pecofence_platform::shell;
use pecofence_platform::window;
use pecofence_render::Image;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const WM_APP_ICON_READY: u32 = pecofence_platform::msg::WM_APP + 4;
const WORKERS: usize = 2;

/// Global icon post-processing (Fences Icon Tint / Chameleon); part of the cache key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IconVariant {
    pub tint: Option<[u8; 3]>,
    /// 0..=100 percent.
    pub tint_strength: u8,
    pub chameleon: bool,
}

impl IconVariant {
    /// Only the parts that change pixels: strength is irrelevant without a tint colour, so
    /// moving the strength slider with tinting off must not flush every icon.
    fn normalized(&self) -> Self {
        Self {
            tint: self.tint,
            tint_strength: if self.tint.is_some() {
                self.tint_strength
            } else {
                0
            },
            chameleon: self.chameleon,
        }
    }

    fn suffix(&self) -> String {
        match (self.tint, self.chameleon) {
            (None, false) => String::new(),
            (Some(t), c) => format!(
                "#t{:02x}{:02x}{:02x}{}{}",
                t[0],
                t[1],
                t[2],
                self.tint_strength,
                if c { "c" } else { "" }
            ),
            (None, true) => "#c".to_string(),
        }
    }

    fn apply(&self, img: &mut Image) {
        if let Some(t) = self.tint {
            img.colorize(t, self.tint_strength as f32 / 100.0);
        }
        if self.chameleon {
            img.fade(0.55, 0.72);
        }
    }
}

#[derive(Clone)]
struct Request {
    key: String,
    path: PathBuf,
    size_px: u32,
    icon_only: bool,
    variant: IconVariant,
    /// Tells a result apart from one of an earlier request for the same key that
    /// `set_variant` or `invalidate` dropped while it was in flight.
    id: u64,
}

/// An extraction in flight (or waiting for a retry), with how often it has failed so far.
struct InFlight {
    req: Request,
    attempts: u32,
}

/// Extraction failures on a file that still exists are retried: the shell hands back nothing
/// while a file is being moved in (OneDrive materialises it a moment later), while a copy is
/// still writing, or when the icon handler was busy. Explorer's views show the generic icon
/// first and repaint when the extraction eventually succeeds; here the loading tile stays up.
const RETRY_ATTEMPTS: u32 = 3;
const RETRY_DELAYS_MS: [u64; 3] = [400, 1500, 4000];

struct Ready {
    key: String,
    size_px: u32,
    id: u64,
    image: Option<Image>,
    /// HRESULT of a failed extraction.
    error: Option<i32>,
}

/// Extraction outcomes since the last summary line (see `IconCache::drain_results`).
#[derive(Default)]
struct IconStats {
    ok: u32,
    failed: u32,
    retried: u32,
    /// Images the shell handed back smaller than asked for (drawn scaled up).
    smaller: u32,
    first_error: Option<i32>,
}

pub enum Lookup {
    /// Extraction finished; `None` means it failed (draw the placeholder, do not retry).
    Ready(Option<Rc<Image>>),
    /// Queued or in flight.
    Pending,
}

pub struct IconCache {
    map: HashMap<(String, u32), Option<Rc<Image>>>,
    pending: HashMap<(String, u32), InFlight>,
    /// Failed extractions waiting to be re-queued (`due`, key); `flush_retries` sends them.
    retry: Vec<(Instant, (String, u32))>,
    tx: Sender<Request>,
    rx: Receiver<Ready>,
    variant: IconVariant,
    stats: IconStats,
    next_id: u64,
}

impl IconCache {
    /// Spawns the worker threads; results wake `notify` with `WM_APP_ICON_READY`.
    pub fn new(notify: HWND) -> Self {
        let (tx, work_rx) = channel::<Request>();
        let (done_tx, rx) = channel::<Ready>();
        let work_rx = Arc::new(Mutex::new(work_rx));
        let notify_raw = notify.0 as isize;
        for i in 0..WORKERS {
            let work_rx = work_rx.clone();
            let done_tx = done_tx.clone();
            let _ = std::thread::Builder::new()
                .name(format!("pecofence-icons-{i}"))
                .stack_size(256 * 1024)
                .spawn(move || {
                    // Single-threaded apartment on purpose: the shell's icon handlers for
                    // Internet shortcuts (.url → IconFile) hand back the generic document icon
                    // from an MTA thread; Explorer extracts icons on STA threads too.
                    let _sta = OleGuard::init();
                    loop {
                        let req = {
                            let Ok(guard) = work_rx.lock() else { break };
                            guard.recv()
                        };
                        let Ok(req) = req else { break };
                        let (image, error) =
                            match shell::shell_image(&req.path, req.size_px, req.icon_only) {
                                Ok(img) => {
                                    let mut image = Image {
                                        width: img.width,
                                        height: img.height,
                                        bgra: img.bgra,
                                    };
                                    req.variant.apply(&mut image);
                                    (Some(image), None)
                                }
                                Err(e) => (None, Some(e.code().0)),
                            };
                        if done_tx
                            .send(Ready {
                                key: req.key,
                                size_px: req.size_px,
                                id: req.id,
                                image,
                                error,
                            })
                            .is_err()
                        {
                            break;
                        }
                        window::post_message(
                            HWND(notify_raw as *mut core::ffi::c_void),
                            WM_APP_ICON_READY,
                            0,
                            0,
                        );
                    }
                });
        }
        Self {
            map: HashMap::new(),
            pending: HashMap::new(),
            retry: Vec::new(),
            tx,
            rx,
            variant: IconVariant::default(),
            stats: IconStats::default(),
            next_id: 0,
        }
    }

    /// Changes the global tint / chameleon processing; drops everything cached (callers must
    /// also drop the per-window icon references).
    pub fn set_variant(&mut self, variant: IconVariant) -> bool {
        let variant = variant.normalized();
        if self.variant == variant {
            return false;
        }
        self.variant = variant;
        self.map.clear();
        self.pending.clear();
        self.retry.clear();
        true
    }

    pub fn cache_key(key: &IconKey) -> String {
        match key {
            IconKey::ByExt(e) => format!("ext:{e}"),
            IconKey::ByContent { path, mtime } => format!("file:{path}:{mtime}"),
        }
    }

    /// Cached image, or queues extraction and reports `Pending`.
    pub fn get(&mut self, key: &IconKey, path: &Path, size_px: u32, icon_only: bool) -> Lookup {
        let k = (
            format!("{}{}", Self::cache_key(key), self.variant.suffix()),
            size_px,
        );
        if let Some(entry) = self.map.get(&k) {
            return Lookup::Ready(entry.clone());
        }
        if !self.pending.contains_key(&k) {
            self.next_id += 1;
            let req = Request {
                key: k.0.clone(),
                path: path.to_path_buf(),
                size_px,
                icon_only,
                variant: self.variant,
                id: self.next_id,
            };
            let _ = self.tx.send(req.clone());
            self.pending.insert(k, InFlight { req, attempts: 0 });
        }
        Lookup::Pending
    }

    /// Bitmap-cache key for an icon as currently processed (variant suffix included).
    pub fn draw_key(&self, key: &IconKey, size_px: u32) -> String {
        format!(
            "{}{}@{}",
            Self::cache_key(key),
            self.variant.suffix(),
            size_px
        )
    }

    /// Moves finished extractions into the cache. Returns how many arrived.
    pub fn drain_results(&mut self) -> usize {
        let mut n = 0;
        let now = Instant::now();
        while let Ok(r) = self.rx.try_recv() {
            let k = (r.key, r.size_px);
            let Some(f) = self.pending.get_mut(&k).filter(|f| f.req.id == r.id) else {
                // Dropped while in flight: an old variant, or a file whose icon changed.
                continue;
            };
            if let Some(e) = r.error {
                self.stats.first_error.get_or_insert(e);
            }
            if r.image.is_none() && f.attempts < RETRY_ATTEMPTS && f.req.path.exists() {
                // Still there but not extractable right now: try again later, and keep the
                // entry pending so the windows keep showing the loading tile instead of the
                // failure placeholder.
                let delay = RETRY_DELAYS_MS[f.attempts as usize % RETRY_DELAYS_MS.len()];
                f.attempts += 1;
                tracing::debug!(key = %k.0, attempt = f.attempts, delay_ms = delay, "icon extraction failed; retrying");
                self.stats.retried += 1;
                self.retry.push((now + Duration::from_millis(delay), k));
                continue;
            }
            match &r.image {
                Some(img) => {
                    self.stats.ok += 1;
                    // Bigger images come back scaled down to the box, smaller ones get drawn
                    // scaled up.
                    if img.width.max(img.height) < k.1 {
                        self.stats.smaller += 1;
                    }
                }
                None => self.stats.failed += 1,
            }
            self.pending.remove(&k);
            self.map.insert(k, r.image.map(Rc::new));
            n += 1;
        }
        if self.pending.is_empty() {
            self.log_stats();
        }
        n
    }

    /// One line per burst of extractions (startup, an icon size change, new files), once
    /// nothing is pending any more; at info only when something went wrong.
    fn log_stats(&mut self) {
        let s = std::mem::take(&mut self.stats);
        if s.ok + s.failed == 0 {
            return;
        }
        let error = s
            .first_error
            .map(|e| format!("{:#010x}", e as u32))
            .unwrap_or_default();
        if s.failed + s.retried + s.smaller > 0 {
            tracing::info!(
                ok = s.ok,
                failed = s.failed,
                retried = s.retried,
                smaller = s.smaller,
                error = %error,
                "icons extracted"
            );
        } else {
            tracing::debug!(ok = s.ok, "icons extracted");
        }
    }

    /// Re-queues the retries that are due; returns how long until the next one (None = no
    /// retries outstanding), so the caller can arm a timer.
    pub fn flush_retries(&mut self) -> Option<Duration> {
        let now = Instant::now();
        let mut i = 0;
        while i < self.retry.len() {
            if self.retry[i].0 <= now {
                let (_, k) = self.retry.swap_remove(i);
                if let Some(f) = self.pending.get(&k) {
                    let _ = self.tx.send(f.req.clone());
                }
            } else {
                i += 1;
            }
        }
        self.retry
            .iter()
            .map(|(due, _)| due.saturating_duration_since(now))
            .min()
    }

    pub fn invalidate(&mut self, key: &IconKey) {
        let prefix = Self::cache_key(key);
        let hit = |k: &str| {
            (k.starts_with(&prefix) && k[prefix.len()..].starts_with(['#'])) || k == prefix
        };
        self.map.retain(|(k, _), _| !hit(k));
        self.pending.retain(|(k, _), _| !hit(k));
        self.retry.retain(|(_, (k, _))| !hit(k));
    }

    /// Drops the per-file icons nothing can show again: older versions of a file that changed
    /// since (`live` holds every item's current `cache_key`) and files that are gone. A file
    /// still on disk keeps its icon (a portal may go back to its folder), and so do namespace
    /// items (the Recycle Bin flips between two) and per-extension icons. Returns the keys
    /// dropped, for `is_dropped_draw_key`.
    pub fn sweep(&mut self, live: &HashSet<String>) -> HashSet<String> {
        let live_paths: HashSet<&str> = live.iter().filter_map(|k| file_key_path(k)).collect();
        let suffix = self.variant.suffix();
        let mut dropped = HashSet::new();
        for (k, _) in self.map.keys() {
            let Some(base) = k.strip_suffix(&suffix) else {
                continue;
            };
            let Some(path) = file_key_path(base) else {
                continue;
            };
            if live.contains(base) || path.starts_with("::") {
                continue;
            }
            if live_paths.contains(path) || !Path::new(path).exists() {
                dropped.insert(base.to_string());
            }
        }
        if !dropped.is_empty() {
            self.map
                .retain(|(k, _), _| !k.strip_suffix(&suffix).is_some_and(|b| dropped.contains(b)));
        }
        dropped
    }

    /// A `draw_key` of an icon `sweep` dropped.
    pub fn is_dropped_draw_key(&self, key: &str, dropped: &HashSet<String>) -> bool {
        key.rsplit_once('@')
            .and_then(|(k, _)| k.strip_suffix(&self.variant.suffix()))
            .is_some_and(|b| dropped.contains(b))
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.map.len()
    }
}

/// The path in a per-file `cache_key` (`file:{path}:{mtime}`).
fn file_key_path(key: &str) -> Option<&str> {
    key.strip_prefix("file:")?
        .rsplit_once(':')
        .map(|(path, _)| path)
}

/// Decides how an item's icon is cached: per-extension for ordinary documents, per-file for
/// things whose icon depends on content (executables, shortcuts, images, folders with icons).
pub fn icon_key_for(path: &Path, is_folder: bool, mtime: i64) -> (IconKey, bool) {
    // Namespace items have no extension; each one has its own icon, and the Recycle Bin's
    // changes with its contents (encoded in `mtime`).
    if pecofence_platform::shell::is_namespace_path(path) {
        return (
            IconKey::ByContent {
                path: path.to_string_lossy().to_lowercase(),
                mtime,
            },
            true,
        );
    }
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
        .unwrap_or_default();
    let by_content = is_folder
        || matches!(
            ext.as_str(),
            ".exe"
                | ".lnk"
                | ".url"
                | ".ico"
                | ".cur"
                | ".appref-ms"
                | ".scr"
                | ".png"
                | ".jpg"
                | ".jpeg"
                | ".gif"
                | ".bmp"
                | ".webp"
                | ".heic"
                | ".tif"
                | ".tiff"
                | ".mp4"
                | ".mkv"
                | ".mov"
                | ".avi"
        );
    let thumbnail_capable = matches!(
        ext.as_str(),
        ".png"
            | ".jpg"
            | ".jpeg"
            | ".gif"
            | ".bmp"
            | ".webp"
            | ".heic"
            | ".tif"
            | ".tiff"
            | ".mp4"
            | ".mkv"
            | ".mov"
            | ".avi"
    );
    let key = if by_content {
        IconKey::ByContent {
            path: path.to_string_lossy().to_lowercase(),
            mtime,
        }
    } else {
        IconKey::ByExt(ext)
    };
    (key, !thumbnail_capable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_drops_old_versions_and_gone_files_only() {
        let dir = std::env::temp_dir().join(format!("pf-icon-sweep-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let kept = dir.join("kept.lnk");
        std::fs::write(&kept, b"x").unwrap();
        let kept = kept.to_string_lossy().to_lowercase();
        let gone = dir.join("gone.lnk").to_string_lossy().to_lowercase();
        let changed = dir.join("changed.exe").to_string_lossy().to_lowercase();
        let mut cache = IconCache::new(HWND::default());
        let keys = [
            format!("file:{changed}:1"), // older version of a live item
            format!("file:{changed}:2"), // live
            format!("file:{gone}:5"),    // no item, no file
            format!("file:{kept}:7"),    // no item, file still there (portal folder left)
            "file:::{645ff040-5081-101b-9f08-00aa002f954e}:0".to_string(), // Recycle Bin, empty
            "ext:.txt".to_string(),
        ];
        for k in &keys {
            cache.map.insert((k.clone(), 96), None);
        }
        let live: HashSet<String> = [format!("file:{changed}:2")].into_iter().collect();
        let dropped = cache.sweep(&live);
        let mut left: Vec<&str> = cache.map.keys().map(|(k, _)| k.as_str()).collect();
        left.sort();
        let mut want = vec![
            keys[1].as_str(),
            keys[3].as_str(),
            keys[4].as_str(),
            keys[5].as_str(),
        ];
        want.sort();
        assert_eq!(left, want);
        assert!(cache.is_dropped_draw_key(&format!("file:{gone}:5@96"), &dropped));
        assert!(!cache.is_dropped_draw_key(&format!("file:{kept}:7@96"), &dropped));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn results_of_dropped_requests_are_not_cached() {
        let mut cache = IconCache::new(HWND::default());
        let (done, rx) = channel();
        cache.rx = rx;
        let key = IconKey::ByExt(".pf-test".into());
        let path = Path::new("C:\\pf-test.pf-test");
        let ready = |id, k: &str| Ready {
            key: k.to_string(),
            size_px: 96,
            id,
            image: Some(Image::solid(1, 1, [0; 3])),
            error: None,
        };
        // The first request is dropped in flight and asked again; only the second one counts.
        assert!(matches!(cache.get(&key, path, 96, false), Lookup::Pending));
        cache.invalidate(&key);
        assert!(matches!(cache.get(&key, path, 96, false), Lookup::Pending));
        done.send(ready(1, "ext:.pf-test")).unwrap();
        assert_eq!(cache.drain_results(), 0);
        assert!(matches!(cache.get(&key, path, 96, false), Lookup::Pending));
        done.send(ready(2, "ext:.pf-test")).unwrap();
        assert_eq!(cache.drain_results(), 1);
        assert!(matches!(
            cache.get(&key, path, 96, false),
            Lookup::Ready(Some(_))
        ));
        // A result for a variant that was switched away from stays out of the cache.
        assert!(matches!(
            cache.get(&IconKey::ByExt(".pf-other".into()), path, 96, false),
            Lookup::Pending
        ));
        cache.set_variant(IconVariant {
            chameleon: true,
            ..IconVariant::default()
        });
        done.send(ready(3, "ext:.pf-other")).unwrap();
        assert_eq!(cache.drain_results(), 0);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn variant_suffix_ignores_strength_without_tint() {
        let off = IconVariant::default();
        let strong = IconVariant {
            tint_strength: 90,
            ..off
        };
        assert_eq!(off.normalized(), strong.normalized());
        assert_eq!(off.suffix(), "");
        let tinted = IconVariant {
            tint: Some([1, 2, 3]),
            tint_strength: 60,
            chameleon: true,
        };
        assert_eq!(tinted.suffix(), "#t01020360c");
        assert_eq!(
            IconVariant {
                chameleon: true,
                ..off
            }
            .suffix(),
            "#c"
        );
    }
}

#[cfg(test)]
mod namespace_icon_tests {
    use super::*;

    /// Namespace items get a per-item content key (no extension to share) whose `mtime` carries
    /// the Recycle Bin's empty/full state, and they are icons rather than thumbnails.
    #[test]
    fn namespace_items_use_content_keys() {
        let bin = Path::new("::{645FF040-5081-101B-9F08-00AA002F954E}");
        let (empty, icon_only) = icon_key_for(bin, false, 0);
        let (full, _) = icon_key_for(bin, false, 1);
        assert!(icon_only);
        assert!(matches!(&empty, IconKey::ByContent { path, mtime: 0 } if path.starts_with("::{")));
        assert_ne!(empty, full);
    }
}
