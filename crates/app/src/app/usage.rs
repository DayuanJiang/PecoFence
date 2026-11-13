//! Usage statistics, the app's side: whether this run records, the file `usage.json` and the
//! uploads. The recorder and the form of a report are `pecofence_core::usage`; what is recorded
//! where, and every name, is in `docs/USAGE_STATS.md`.
//!
//! No log line carries the install number or a report.

use super::*;
use pecofence_core::usage as recorder;
use std::sync::PoisonError;

/// Where the reports go: `POST <USAGE_URL>/usage`.
const USAGE_URL: &str = "https://dipg04t9bhh01.cloudfront.net";
/// A test's own listener in place of `USAGE_URL`, taken only when it starts with
/// `http://127.0.0.1:`. With it a test run records and sends.
const URL_VAR: &str = "PECOFENCE_USAGE_URL";
/// In `%LOCALAPPDATA%\PecoFence`; beside the configuration for the portable edition.
const FILE_NAME: &str = "usage.json";
/// The file is written this often at most, and only when something changed.
const WRITE_EVERY: Duration = Duration::from_secs(5 * 60);
/// A new installation sends its first day once, this long after its first start.
const FIRST_DAY_UPLOAD: Duration = Duration::from_secs(30 * 60);
/// A start within this many seconds of Explorer's counts as a start at sign-in.
const SIGN_IN_WINDOW_S: f64 = 120.0;

/// Held while `usage.json` is written or deleted: a write under way on a worker cannot put
/// back the file of statistics that were just switched off.
static FILE_LOCK: Mutex<()> = Mutex::new(());

/// What a run that may record keeps.
pub(super) struct Usage {
    file: PathBuf,
    url: String,
    /// The local day at the last look (`usage_minute`), and when the days before it go out.
    day: String,
    due: Option<Instant>,
    /// When this run made the install number: today's report goes out once,
    /// `FIRST_DAY_UPLOAD` later.
    made_id: Option<Instant>,
    written: Instant,
    /// From the start of the process until the fences were up.
    startup_ms: u64,
}

/// The local date (`2026-11-14`) and the second of the day, for the recorder.
fn local_now() -> (String, u32) {
    let stamp = pecofence_platform::fileinfo::format_local_timestamp(updates::unix_now());
    let (day, time) = stamp.split_once(' ').unwrap_or_default();
    let second = time
        .split(':')
        .fold(0, |total, part| total * 60 + part.parse().unwrap_or(0));
    (day.to_string(), second)
}

/// Writes `usage.json` when something changed since it was last written.
fn write(file: &Path) {
    let _held = FILE_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(text) = recorder::snapshot()
        && let Err(error) = std::fs::write(file, text)
    {
        tracing::warn!(%error, "usage statistics: usage.json not written");
    }
}

/// Sends every earlier day's report, and today's too when `today` is set. A day the service
/// took (2xx) is forgotten; any other is tried again at the next start or day change.
fn upload(url: &str, today: bool) {
    for (day, body) in recorder::unsent(today) {
        let headers = format!(
            "Content-Type: application/json\r\nx-amz-content-sha256: {}",
            pecofence_platform::http::sha256_hex(body.as_bytes())
        );
        let answer = pecofence_platform::http::post(
            &format!("{url}/usage"),
            &updates::user_agent(),
            &headers,
            body.as_bytes(),
        );
        tracing::info!(day, bytes = body.len(), ?answer, "usage report upload");
        if matches!(answer, Ok(200..=299)) {
            recorder::sent(&day);
        }
    }
}

impl App {
    /// At the end of a start: decides whether this run may record, switches the recorder on
    /// when 「发送使用统计」 is on, and records how the run started.
    pub(super) fn begin_usage(&mut self, args: &Args, after_explorer_s: Option<f64>) {
        let test_url = pecofence_core::brand::var(URL_VAR)
            .ok()
            .filter(|url| url.starts_with("http://127.0.0.1:"));
        let test_run =
            args.test_script.is_some() || args.exit_after_ms.is_some() || args.instance.is_some();
        let dir = if self.portable {
            self.state.config_path().parent().map(Path::to_path_buf)
        } else {
            crate::log_dir()
        };
        let Some(dir) = dir.filter(|_| !test_run || test_url.is_some()) else {
            tracing::info!(test_run, "usage statistics: not recorded in this run");
            return;
        };
        self.usage = Some(Usage {
            file: dir.join(FILE_NAME),
            url: test_url.unwrap_or_else(|| USAGE_URL.to_string()),
            day: String::new(),
            due: None,
            made_id: None,
            written: Instant::now(),
            startup_ms: self.started.elapsed().as_millis() as u64,
        });
        let Some(last_app) = self.sync_usage() else {
            return;
        };
        recorder::event_with(
            "start",
            if args.restarted_after_crash.is_some() {
                "after_crash"
            } else if !last_app.is_empty() && last_app != env!("CARGO_PKG_VERSION") {
                "after_update"
            } else if after_explorer_s.is_none_or(|s| s < SIGN_IN_WINDOW_S) {
                // A guess: at sign-in PecoFence starts right after Explorer, or before it.
                "at_sign_in"
            } else {
                "by_hand"
            },
        );
    }

    /// Follows the 「发送使用统计」 switch. On: the recorder takes up `usage.json`, with a new
    /// install number when there is none; returns the PecoFence version of the last report
    /// (empty for a new number). Off: recording stops at once and the file is deleted.
    pub(super) fn sync_usage(&mut self) -> Option<String> {
        let run = self.usage.as_mut()?;
        // Defaults standing in for a configuration another program holds open do not say
        // what the user chose.
        let on = self.state.config.settings.usage_stats && self.state.locked_config.is_none();
        if on == recorder::today().is_some() {
            return None;
        }
        tracing::info!(on, "usage statistics: recording");
        if on {
            let saved = std::fs::read_to_string(&run.file).unwrap_or_default();
            let last_app = recorder::start(&saved, local_now);
            run.made_id = last_app.is_empty().then(Instant::now);
            return Some(last_app);
        }
        recorder::stop();
        let _held = FILE_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        if let Err(error) = std::fs::remove_file(&run.file)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(%error, "usage statistics: usage.json not deleted");
        }
        None
    }

    /// Once a minute. A worker writes `usage.json` when something changed, every
    /// `WRITE_EVERY` at most, and first sends what is due: the earlier days at a start and
    /// after the local date changed, a new installation's first day `FIRST_DAY_UPLOAD` after
    /// its start.
    pub(super) fn usage_minute(&mut self) {
        let Some((day, unsaved)) = recorder::today() else {
            return;
        };
        let Some(run) = self.usage.as_mut() else {
            return;
        };
        if day != run.day {
            // At a start the earlier days go out now. When the date changes while the program
            // runs, a random 1 to 60 minutes later: the service takes two reports at a time,
            // and midnight comes to every PC of a time zone at once.
            let wait = match run.day.is_empty() {
                true => 0,
                false => 60 + (uuid::Uuid::new_v4().as_u128() % 3541) as u64,
            };
            run.due = Some(Instant::now() + Duration::from_secs(wait));
            run.day = day;
        }
        let today = run
            .made_id
            .is_some_and(|at| at.elapsed() >= FIRST_DAY_UPLOAD);
        let send = today || run.due.is_some_and(|due| Instant::now() >= due);
        if !send && !(unsaved && run.written.elapsed() >= WRITE_EVERY) {
            return;
        }
        if send {
            run.due = None;
        }
        if today {
            run.made_id = None;
        }
        run.written = Instant::now();
        let (file, url) = (run.file.clone(), run.url.clone());
        self.set_usage_facts();
        std::thread::spawn(move || {
            if send {
                upload(&url, today);
            }
            write(&file);
        });
    }

    /// The end of the run or of the session: `usage.json` is written here, on this thread.
    pub(super) fn save_usage(&self) {
        if let Some(run) = &self.usage {
            self.set_usage_facts();
            write(&run.file);
        }
    }

    /// Sets what today's report says beside its events.
    fn set_usage_facts(&self) {
        let Some(run) = &self.usage else {
            return;
        };
        let config = &self.state.config;
        let fences = self.state.fences();
        let count = |test: &dyn Fn(&pecofence_core::Fence) -> bool| {
            fences.iter().filter(|f| test(f)).count()
        };
        let settings = recorder::settings_report(&config.settings, &config.rules);
        let state = serde_json::json!({
            "fences": fences.len(),
            "tabs": count(&|f| f.tab_host.is_some()),
            "portals": count(&|f| f.kind == FenceKind::FolderPortal),
            "smart": count(&|f| f.smart.is_some()),
            "tools": count(&|f| f.tool.is_some()),
            "rules": config.rules.list.len(),
            "items": config.items.len(),
        });
        // The private commit: the working set is handed back whenever the program is idle.
        let memory_mb = pecofence_platform::memstats::MemoryStats::current()
            .map_or(0, |m| m.private_commit as u64 >> 20);
        let (startup_ms, crashed) = (run.startup_ms, !self.previous_run_ended_normally());
        let edition = feedback::install_kind(self.portable);
        let windows = format!("10.0.{}", desktop::os_build());
        recorder::update(move |report| {
            report.app = env!("CARGO_PKG_VERSION").to_string();
            report.edition = edition;
            report.windows = windows;
            report.language = pecofence_core::i18n::language().tag().to_string();
            report.settings = settings.into();
            report.state = state;
            // Whether a start of the day found that the run before it crashed or was cut off.
            let crashed = crashed || report.health["crashed"] == true;
            // The most the program held at one of the day's writes.
            let memory_mb = memory_mb.max(report.health["memoryMB"].as_u64().unwrap_or(0));
            report.health = serde_json::json!({
                "startupMs": startup_ms, "memoryMB": memory_mb, "crashed": crashed,
            });
        });
    }
}
