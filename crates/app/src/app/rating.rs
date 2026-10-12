//! Asking for a Microsoft Store rating, in the Store edition only: once, a week after the first
//! start, a notice in the corner (`crate::notice`, 30 s) offers it, and clicking it opens the
//! Store's own rating dialog. Windows keeps a window that a background app opens behind the
//! active one, so the dialog only ever follows a click (the notice, or the button on the
//! 「关于」 page).

use super::*;
use pecofence_platform::store::{self, RatingOutcome};

/// How long PecoFence is in use before it asks.
const ASK_AFTER: Duration = Duration::from_secs(7 * 24 * 60 * 60);
/// Not right after a start, which is usually the logon rush.
const QUIET_AFTER_START: Duration = Duration::from_secs(10 * 60);
/// Kept beside config.json, so each test instance has its own.
const FILE: &str = "rating.json";

#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Saved {
    /// Unix seconds of the first start that knew about rating.
    first_seen: u64,
    asked: bool,
}

pub(super) struct RatingPrompt {
    path: PathBuf,
    saved: Saved,
    started: Instant,
    /// The rating notice while it is on screen.
    notice: Option<crate::notice::Notice>,
    /// Showing the notice failed this run; do not retry every tick.
    failed: bool,
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Whether to ask now (apart from the edition and the user's notification state).
fn due(now: u64, saved: &Saved, uptime: Duration) -> bool {
    !saved.asked
        && now.saturating_sub(saved.first_seen) >= ASK_AFTER.as_secs()
        && uptime >= QUIET_AFTER_START
}

impl RatingPrompt {
    /// `persist`: record the first start on disk (the Store edition; other editions never ask
    /// and leave no file behind).
    pub(super) fn load(config_dir: &Path, persist: bool) -> Self {
        let path = config_dir.join(FILE);
        let saved = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Saved>(&bytes).ok())
            .filter(|s| s.first_seen > 0);
        let mut prompt = Self {
            path,
            saved: saved.unwrap_or_default(),
            started: Instant::now(),
            notice: None,
            failed: false,
        };
        if prompt.saved.first_seen == 0 {
            prompt.saved.first_seen = unix_now();
            if persist {
                prompt.save();
            }
        }
        prompt
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_vec_pretty(&self.saved)
            && let Err(error) = std::fs::write(&self.path, json)
        {
            tracing::warn!(%error, "rating state not saved");
        }
    }
}

impl App {
    /// Housekeeping: offer the rating once it is due. `PECOFENCE_RATING_PROMPT=now` offers it
    /// on the next tick in any edition (tests; without a package the Store page opens instead).
    pub(super) fn offer_rating_if_due(&mut self) {
        let forced =
            pecofence_core::brand::var("PECOFENCE_RATING_PROMPT").is_ok_and(|v| v == "now");
        let ready = if forced {
            !self.rating.saved.asked
        } else {
            pecofence_platform::process::is_packaged()
                && due(
                    unix_now(),
                    &self.rating.saved,
                    self.rating.started.elapsed(),
                )
                && store::user_accepts_notifications()
        };
        if !ready || self.rating.failed || self.rating.notice.is_some() || self.welcome.is_some() {
            return;
        }
        match crate::notice::Notice::show(
            &self.ctx,
            pecofence_core::i18n::text("喜欢 PecoFence 吗？"),
            pecofence_core::i18n::text("在 Microsoft Store 给它打个分，几秒钟就好。"),
            pecofence_core::i18n::text("去评分"),
            crate::notice::locale(),
        ) {
            Ok(notice) => {
                self.rating.notice = Some(notice);
                // Once, whether or not it is clicked.
                self.rating.saved.asked = true;
                self.rating.save();
                tracing::info!("offered a Store rating");
            }
            Err(error) => {
                self.rating.failed = true;
                tracing::warn!(%error, "rating notice not shown");
            }
        }
    }

    /// A notice was clicked or went away. The welcome and the rating offer never show at
    /// the same time.
    pub(super) fn on_notice(&mut self, event: crate::notice::NoticeEvent) {
        if self.welcome.take().is_some() {
            if event == crate::notice::NoticeEvent::Action {
                self.open_manual();
            }
            return;
        }
        self.rating.notice = None;
        if event == crate::notice::NoticeEvent::Action {
            self.open_store_rating();
        }
    }

    /// The Store's rating dialog, centred on the settings window when it is open, otherwise
    /// on the screen with the pointer. Falls back to the product's review page in the Store.
    pub(super) fn open_store_rating(&mut self) {
        let owner = match &self.settings {
            Some(host) => host.hwnd(),
            None => {
                // The control window is a hidden 0×0 popup; the dialog centres on it.
                let cursor = window::cursor_pos();
                if let Some(info) =
                    monitors::query(monitors::monitor_from_point(cursor.x, cursor.y))
                {
                    let work = info.work_area;
                    let _ = self.control.set_bounds(
                        (work.left + work.right) / 2,
                        (work.top + work.bottom) / 2,
                        0,
                        0,
                    );
                }
                self.control.hwnd()
            }
        };
        let request = store::request_rating(owner, |outcome| {
            let (what, code) = match outcome {
                RatingOutcome::Rated { updated: false } => ("rated", 0),
                RatingOutcome::Rated { updated: true } => ("updated", 0),
                RatingOutcome::Cancelled => ("cancelled", 0),
                RatingOutcome::NetworkError => ("network error", 0),
                RatingOutcome::Failed(hr) => ("failed", hr.0),
            };
            tracing::info!(what, code, "store rating dialog closed");
        });
        if let Err(error) = request {
            tracing::warn!(%error, "store rating dialog unavailable; opening the Store page");
            let page = format!(
                "ms-windows-store://review/?ProductId={}",
                pecofence_core::brand::STORE_PRODUCT_ID
            );
            let _ = shell::shell_execute(Path::new(&page), None, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: u64 = 24 * 60 * 60;

    fn saved(first_seen: u64, asked: bool) -> Saved {
        Saved { first_seen, asked }
    }

    #[test]
    fn asks_once_a_week_after_the_first_start_and_not_right_after_a_start() {
        let settled = QUIET_AFTER_START;
        assert!(!due(6 * DAY, &saved(0, false), settled));
        assert!(due(7 * DAY, &saved(0, false), settled));
        assert!(!due(7 * DAY, &saved(0, false), Duration::from_secs(60)));
        assert!(!due(30 * DAY, &saved(0, true), settled));
        // A clock set back does not underflow.
        assert!(!due(0, &saved(7 * DAY, false), settled));
    }

    #[test]
    fn the_state_file_survives_and_a_missing_one_starts_the_week() {
        let dir = std::env::temp_dir().join(format!("pecofence-rating-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        RatingPrompt::load(&dir, false);
        assert!(!dir.join(FILE).exists());
        let first = RatingPrompt::load(&dir, true);
        assert!(first.saved.first_seen > 0 && !first.saved.asked);
        let mut again = RatingPrompt::load(&dir, true);
        assert_eq!(again.saved.first_seen, first.saved.first_seen);
        again.saved.asked = true;
        again.save();
        assert!(RatingPrompt::load(&dir, true).saved.asked);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
