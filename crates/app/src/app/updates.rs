//! Settings → 关于 → 「检查更新」 for the installer and ZIP editions; the Store updates the app
//! itself. Checking and downloading run on a worker thread. An installed copy then starts the
//! new setup and exits; setup waits for it, installs and starts PecoFence again.
//!
//! A check asks GitHub's API first. Without a good answer within `ANSWER_WAIT` (a network that
//! cannot reach GitHub, an error, a rate limit) it asks the project's website for its copy of
//! the latest release (`update.json`); a release read there opens the site's download section
//! as its page. When both fail, the page names the site's address and offers 「打开下载页」.
//!
//! 自动检查更新 (Settings → 关于, on by default; the installer and ZIP editions, never the Store
//! edition, a named test instance only with the test feed): about a minute after start and then
//! once a day the same request runs by itself. Offline, an error or a rate limit only logs a line
//! and tries once more an hour later, then the next day; after a resume from sleep it checks
//! when the last good check is over a day old. A newer version gets a card in the corner once
//! per version (`notices.json`; a card nobody was there for is offered once more a day later),
//! never over a full-screen app or presentation, during the Peek or with another card (it waits
//! and tries again), and stays at the top of the tray menu until it is installed.

use super::*;
use pecofence_core::updates::{self, Release};

/// A named test instance's release API (`http://127.0.0.1:` only, and only with
/// `PECOFENCE_INSTANCE`): the updater's end-to-end test serves a release from a local server.
const URL_VAR: &str = "PECOFENCE_UPDATE_URL";
/// The test feed's stand-in for the site's `update.json` (`http://127.0.0.1:` only; without it
/// the file is asked for at the test feed's own origin).
const SITE_URL_VAR: &str = "PECOFENCE_UPDATE_SITE_URL";
const MAX_RELEASE_JSON: usize = 1024 * 1024;
/// How long each request of a check may take. WinHTTP's own timeouts let a network that cannot
/// reach GitHub hang for up to a minute; after this long the check goes on to the site's file.
const ANSWER_WAIT: std::time::Duration = std::time::Duration::from_secs(8);
/// The automatic check, a card that waits, the check after a resume (`TIMER_UPDATE_CHECK`,
/// `TIMER_UPDATE_CARD`, `TIMER_UPDATE_RESUME`); retried while the App is borrowed.
pub(super) const TIMER_UPDATE_CHECK: usize = 97;
pub(super) const TIMER_UPDATE_CARD: usize = 98;
pub(super) const TIMER_UPDATE_RESUME: usize = 99;
const FIRST_CHECK_MS: u32 = 60_000;
const DAILY_CHECK_MS: u32 = 24 * 60 * 60 * 1000;
/// A failed automatic check tries once more this much later.
const RETRY_CHECK_MS: u32 = 60 * 60 * 1000;
/// After a resume, the network comes back first.
const RESUME_CHECK_MS: u32 = 30_000;
/// A resume checks when the last good check is older than this (seconds).
const RESUME_CHECK_AFTER_S: i64 = 24 * 60 * 60;
/// A card nobody was there for is offered once more after this (seconds; the next daily check).
const UNSEEN_AGAIN_AFTER_S: i64 = 20 * 60 * 60;
/// A card that may not show yet tries again this much later.
const CARD_RETRY_MS: u32 = 10_000;
/// The card's line from the notes stops about here (three lines of the card).
const CARD_LINE_CHARS: usize = 110;

/// The automatic check's delays: after start, between checks, the retry after a failure, the
/// check after a resume. Debug builds take
/// `PECOFENCE_TEST_UPDATE_MS=<after start>[,<between>[,<retry>[,<after a resume>]]]`.
fn check_delays() -> [u32; 4] {
    let delays = [
        FIRST_CHECK_MS,
        DAILY_CHECK_MS,
        RETRY_CHECK_MS,
        RESUME_CHECK_MS,
    ];
    #[cfg(debug_assertions)]
    if let Ok(v) = pecofence_core::brand::var("PECOFENCE_TEST_UPDATE_MS") {
        let mut test = delays;
        for (slot, part) in test.iter_mut().zip(v.split(',')) {
            if let Ok(ms) = part.trim().parse() {
                *slot = ms;
            }
        }
        return test;
    }
    delays
}

/// After a resume, the check waits this long for the network (`TIMER_UPDATE_RESUME`).
pub(super) fn resume_check_ms() -> u32 {
    check_delays()[3]
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// The test feed is set (`PECOFENCE_UPDATE_URL`, `http://127.0.0.1:` only).
fn test_feed() -> bool {
    pecofence_core::brand::var(URL_VAR).is_ok_and(|u| u.starts_with("http://127.0.0.1:"))
}

/// The release page in the browser (the ZIP edition's download, 「查看新功能」).
fn open_release_page(page: &str) {
    match shell::shell_execute(Path::new(page), None, None) {
        Ok(_) => tracing::info!(page, "update: release page opened"),
        Err(e) => tracing::warn!(error = %e, "opening the release page failed"),
    }
}
const MAX_SETUP_BYTES: usize = 64 * 1024 * 1024;

#[derive(Default)]
enum Status {
    #[default]
    Idle,
    Checking,
    UpToDate,
    Available(Release),
    Downloading(Release),
    CheckFailed,
    DownloadFailed,
}

enum Done {
    Checked(Option<Release>),
    Downloaded(PathBuf),
}

#[derive(Default)]
pub(super) struct Updates {
    status: Status,
    /// The check running now started by itself (quiet; it shows a card for what it finds).
    auto: bool,
    /// The newer version the last check found: the tray item and the card offer it, and a
    /// failed later check keeps it.
    found: Option<Release>,
    /// The automatic check found a version whose card has not shown yet.
    card_waiting: bool,
    /// Why the card is waiting and since when (the first wait and its end are logged).
    card_wait: Option<(&'static str, Instant)>,
    notice: Option<crate::notice::Notice>,
    /// The card on screen: its version, the last input when it appeared, and whether this
    /// is its last offer.
    card_shown: Option<(String, Option<u32>, bool)>,
    /// What the card did for which version; read from `notices.json` once, kept here.
    record: Option<super::welcome::UpdateCardRecord>,
    /// The last automatic check failed and its one retry is armed.
    retrying: bool,
    /// When a check last got an answer (unix seconds).
    last_success: Option<i64>,
    /// Finished jobs; the error text goes to the log, the page only says which step failed.
    done: Arc<Mutex<Vec<std::result::Result<Done, String>>>>,
}

/// Where downloaded setups wait to run (our own local data folder, never a shared temp
/// directory); emptied on the next start.
fn download_dir() -> PathBuf {
    crate::log_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("update")
}

fn user_agent() -> String {
    format!("PecoFence/{}", env!("CARGO_PKG_VERSION"))
}

/// The release API, the prefix its files must be downloaded from, and the site's copy of the
/// latest release.
fn source(test_instance: bool) -> (String, String, String) {
    match pecofence_core::brand::var(URL_VAR) {
        Ok(url) if test_instance && url.starts_with("http://127.0.0.1:") => {
            let origin = url.split('/').take(3).collect::<Vec<_>>().join("/") + "/";
            let site = pecofence_core::brand::var(SITE_URL_VAR)
                .ok()
                .filter(|u| u.starts_with("http://127.0.0.1:"))
                .unwrap_or_else(|| format!("{origin}update.json"));
            (url, origin, site)
        }
        _ => (
            updates::latest_release_url(),
            updates::download_base(),
            updates::site_update_url(),
        ),
    }
}

/// `http::get` of one of a check's small files, waited for at most `ANSWER_WAIT`: the request
/// runs on a thread of its own, and one that got no answer in time ends by itself later.
fn get_in_time(url: &str, headers: &'static str) -> std::result::Result<Vec<u8>, String> {
    let (answer, waiting) = std::sync::mpsc::channel();
    let address = url.to_string();
    std::thread::Builder::new()
        .name("pecofence-update-get".into())
        .spawn(move || {
            let _ = answer.send(pecofence_platform::http::get(
                &address,
                &user_agent(),
                headers,
                MAX_RELEASE_JSON,
            ));
        })
        .map_err(|e| e.to_string())?;
    waiting.recv_timeout(ANSWER_WAIT).unwrap_or_else(|_| {
        Err(format!(
            "no answer from {url} within {} s",
            ANSWER_WAIT.as_secs()
        ))
    })
}

/// The site's copy of the latest release (`update.json` at `site`), with the card's line in the
/// UI language `ui_language`.
fn site_copy(site: &str, base: &str, ui_language: &str) -> std::result::Result<Release, String> {
    let body = get_in_time(site, "")?;
    updates::parse_site_update(&String::from_utf8_lossy(&body), base, ui_language)
}

/// The latest release: from GitHub's API, or from the site's copy when the API gives no good
/// answer in time. For a newer one from GitHub also the first point of its Store notes in the
/// UI language `ui_language` (one more request, only then: GitHub's raw file, or the site's
/// copy of the same version when that file cannot be read; a failure only loses that line).
fn check(test_instance: bool, ui_language: &str) -> std::result::Result<Done, String> {
    let (url, base, site) = source(test_instance);
    let started = Instant::now();
    let github = get_in_time(&url, "Accept: application/vnd.github+json")
        .and_then(|body| updates::parse_release(&String::from_utf8_lossy(&body), &base));
    let mut release = match github {
        Ok(release) => release,
        Err(first) => {
            tracing::info!(
                error = %first,
                waited_ms = started.elapsed().as_millis() as u64,
                "update: no answer from GitHub; asking the site"
            );
            let release = site_copy(&site, &base, ui_language)
                .map_err(|second| format!("GitHub: {first}; the site: {second}"))?;
            tracing::info!(
                version = %release.version,
                line = release.store_line.is_some(),
                waited_ms = started.elapsed().as_millis() as u64,
                "update: answer from the site"
            );
            let newer = updates::is_newer(&release.version, env!("CARGO_PKG_VERSION"));
            return Ok(Done::Checked(newer.then_some(release)));
        }
    };
    if !updates::is_newer(&release.version, env!("CARGO_PKG_VERSION")) {
        return Ok(Done::Checked(None));
    }
    // The test feed serves the notes too; GitHub's raw files otherwise.
    let raw_base = if base.starts_with("http://127.0.0.1:") {
        base.clone()
    } else {
        updates::RAW_BASE.to_string()
    };
    match updates::store_notes_url(&raw_base, &release.version, ui_language) {
        None => tracing::info!(ui_language, "update: no Store notes for this language"),
        Some(notes) => match get_in_time(&notes, "") {
            Ok(bytes) => {
                release.store_line = updates::store_notes_line(&String::from_utf8_lossy(&bytes));
                tracing::info!(
                    found = release.store_line.is_some(),
                    "update: Store notes read"
                );
            }
            // GitHub's raw files are out of reach on many networks that reach its API: the
            // site's copy holds the same line while it is of the same version.
            Err(e) => match site_copy(&site, &base, ui_language) {
                Ok(copy) if copy.version == release.version => {
                    release.store_line = copy.store_line;
                    tracing::info!(
                        error = %e,
                        found = release.store_line.is_some(),
                        "update: Store notes not read from GitHub; the line of the site's copy"
                    );
                }
                Ok(copy) => {
                    tracing::info!(error = %e, site_version = %copy.version, "update: Store notes not read, the site's copy is of another version; the card says something general")
                }
                Err(second) => {
                    tracing::info!(error = %e, site = %second, "update: Store notes not read; the card says something general")
                }
            },
        },
    }
    Ok(Done::Checked(Some(release)))
}

/// No checksum of our own: HTTPS covers the transfer, and setup checks its own data before
/// installing (a damaged one fails, and the old copy is started again).
fn download(release: &Release) -> std::result::Result<Done, String> {
    let setup = release.setup.as_ref().ok_or("the release has no setup")?;
    let bytes = pecofence_platform::http::get(&setup.url, &user_agent(), "", MAX_SETUP_BYTES)?;
    let dir = download_dir();
    let path = dir.join(&setup.name);
    std::fs::create_dir_all(&dir)
        .and_then(|()| std::fs::write(&path, &bytes))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Done::Downloaded(path))
}

impl App {
    /// `None` for the Store edition, which shows no update controls.
    fn update_edition(&self) -> Option<&'static str> {
        if pecofence_platform::process::is_packaged() {
            return None;
        }
        let installed = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join("unins000.exe").is_file()))
            .unwrap_or(false);
        Some(if installed { "installer" } else { "zip" })
    }

    /// At startup: deletes setups a previous update downloaded (best effort: one still running
    /// goes next time). `setup.log` stays for diagnosing a failed update.
    pub(super) fn remove_old_update_downloads(&self) {
        let Ok(entries) = std::fs::read_dir(download_dir()) else {
            return;
        };
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|e| e == "exe") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    /// The page's update card; `null` hides it (Store edition).
    pub(super) fn update_json(&self) -> serde_json::Value {
        let Some(edition) = self.update_edition() else {
            return serde_json::Value::Null;
        };
        let (state, release) = match &self.updates.status {
            Status::Idle => ("idle", None),
            Status::Checking => ("checking", None),
            Status::UpToDate => ("upToDate", None),
            Status::Available(r) => ("available", Some(r)),
            Status::Downloading(r) => ("downloading", Some(r)),
            Status::CheckFailed => ("checkFailed", None),
            Status::DownloadFailed => ("downloadFailed", None),
        };
        serde_json::json!({
            "edition": edition,
            "state": state,
            "version": release.map(|r| r.version.as_str()),
            "hasSetup": release.is_some_and(|r| r.setup.is_some()),
        })
    }

    fn post_update(&self) {
        if let Some(h) = &self.settings {
            h.post_json(
                &serde_json::json!({ "type": "update", "update": self.update_json() }).to_string(),
            );
        }
    }

    fn run_update_job(
        &mut self,
        status: Status,
        job: impl FnOnce() -> std::result::Result<Done, String> + Send + 'static,
    ) {
        let done = self.updates.done.clone();
        let control = self.control.hwnd().0 as isize;
        let spawned = std::thread::Builder::new()
            .name("pecofence-update".into())
            .spawn(move || {
                let result = job();
                if let Ok(mut d) = done.lock() {
                    d.push(result);
                }
                window::post_message(
                    HWND(control as *mut core::ffi::c_void),
                    WM_APP_COMMAND,
                    0,
                    0,
                );
            });
        self.updates.status = match spawned {
            Ok(_) => status,
            Err(e) => {
                tracing::warn!(error = %e, "update thread failed to start");
                Status::CheckFailed
            }
        };
        self.post_update();
    }

    /// `{"type": "action", "name": "…Update…"}` messages from the page.
    pub(super) fn on_update_action(&mut self, name: &str) {
        let Some(edition) = self.update_edition() else {
            return;
        };
        match (name, &self.updates.status) {
            (_, Status::Checking | Status::Downloading(_)) => {}
            ("checkUpdate", _) => {
                let test_instance = self.instance.is_some();
                let lang = pecofence_core::i18n::language().tag();
                self.run_update_job(Status::Checking, move || check(test_instance, lang));
            }
            ("installUpdate", Status::Available(release))
                if edition == "installer" && release.setup.is_some() =>
            {
                let release = release.clone();
                tracing::info!(version = %release.version, "downloading update");
                let job_release = release.clone();
                self.run_update_job(Status::Downloading(release), move || download(&job_release));
            }
            ("openReleasePage", Status::Available(release)) => open_release_page(&release.page),
            // 「打开下载页」 beside a failed check or download: the site's download section.
            ("openDownloadPage", _) => open_release_page(&updates::site_download_page(
                pecofence_core::i18n::language().tag(),
            )),
            _ => {}
        }
    }

    /// Applies finished checks and downloads (called from the command pump).
    pub(super) fn drain_updates(&mut self) {
        let done: Vec<_> = self
            .updates
            .done
            .lock()
            .map(|mut d| d.drain(..).collect())
            .unwrap_or_default();
        for result in done {
            let checking = matches!(self.updates.status, Status::Checking);
            let auto = checking && std::mem::take(&mut self.updates.auto);
            let outcome = match result {
                Ok(Done::Downloaded(setup)) => self.start_setup(&setup).map(|()| None),
                Ok(Done::Checked(release)) => Ok(Some(release)),
                Err(e) => Err(e),
            };
            self.updates.status = match outcome {
                // Setup is running and PecoFence is quitting.
                Ok(None) => return,
                Ok(Some(None)) => {
                    tracing::info!(auto, "update: up to date");
                    self.updates.found = None;
                    self.updates.retrying = false;
                    self.updates.last_success = Some(unix_now());
                    Status::UpToDate
                }
                Ok(Some(Some(release))) => {
                    tracing::info!(version = %release.version, auto, "update available");
                    self.updates.found = Some(release.clone());
                    self.updates.retrying = false;
                    self.updates.last_success = Some(unix_now());
                    if auto {
                        self.updates.card_waiting = true;
                    }
                    Status::Available(release)
                }
                // Offline, an error or a rate limit: nothing shown, a version found before stays
                // offered; one more try an hour later, then the next day.
                Err(e) if auto => {
                    if self.updates.retrying {
                        self.updates.retrying = false;
                        tracing::info!(error = %e, "update: the automatic check failed again; next try tomorrow");
                    } else {
                        self.updates.retrying = true;
                        window::set_timer(
                            self.control.hwnd(),
                            TIMER_UPDATE_CHECK,
                            check_delays()[2],
                        );
                        tracing::info!(error = %e, "update: the automatic check failed; one more try in an hour");
                    }
                    match &self.updates.found {
                        Some(r) => Status::Available(r.clone()),
                        None => Status::Idle,
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, checking, "update failed");
                    if checking {
                        Status::CheckFailed
                    } else {
                        Status::DownloadFailed
                    }
                }
            };
            self.post_update();
            // 「更新」 on the card or in the tray menu runs with Settings closed: when the
            // download failed, the About page opens, says so and offers the site's download page.
            if matches!(self.updates.status, Status::DownloadFailed) {
                tracing::info!("update: the download failed; opening the About page");
                self.open_settings_page("about", Some("updateCard"));
            }
        }
        self.show_update_card();
    }

    /// Arms the automatic check: the first about a minute after start, then once a day.
    pub(super) fn arm_update_check(&self, first: bool) {
        let [after_start, every, _, _] = check_delays();
        let ms = if first { after_start } else { every };
        window::set_timer(self.control.hwnd(), TIMER_UPDATE_CHECK, ms);
    }

    /// `TIMER_UPDATE_RESUME`: checks when the last good check is over a day old (or there was
    /// none).
    pub(super) fn update_check_after_resume(&mut self) {
        let age = self.updates.last_success.map(|t| unix_now() - t);
        if age.is_some_and(|a| a < RESUME_CHECK_AFTER_S) {
            tracing::info!(
                age_s = age.unwrap_or(0),
                "update: no check after the resume; the last one is recent"
            );
            return;
        }
        tracing::info!(age_s = age.unwrap_or(-1), "update: check after the resume");
        self.auto_check_updates();
    }

    /// `TIMER_UPDATE_CHECK`: the automatic check, when 自动检查更新 is on and this edition
    /// checks (the installer and ZIP editions; a named test instance only with the test feed).
    pub(super) fn auto_check_updates(&mut self) {
        self.arm_update_check(false);
        let why = if !self.state.config.settings.auto_check_updates {
            Some("switched off")
        } else if self.update_edition().is_none() {
            Some("Store edition")
        } else if self.instance.is_some() && !test_feed() {
            Some("test instance")
        } else if matches!(
            self.updates.status,
            Status::Checking | Status::Downloading(_)
        ) {
            Some("a check or download is running")
        } else {
            None
        };
        if let Some(why) = why {
            tracing::info!(why, "update: automatic check skipped");
            return;
        }
        tracing::info!("update: automatic check");
        self.updates.auto = true;
        let test_instance = self.instance.is_some();
        let lang = pecofence_core::i18n::language().tag();
        self.run_update_job(Status::Checking, move || check(test_instance, lang));
    }

    /// The newer version the tray menu offers at its top (「更新到 0.3.3」) until it is installed.
    pub(super) fn update_offered(&self) -> Option<String> {
        self.updates.found.as_ref().map(|r| r.version.clone())
    }

    /// The new setup is downloading (the tray item says so meanwhile).
    pub(super) fn update_downloading(&self) -> bool {
        matches!(self.updates.status, Status::Downloading(_))
    }

    /// 「更新」 (the tray item, the card): the installer edition downloads and installs, the ZIP
    /// edition opens the download page.
    pub(super) fn update_now(&mut self, by: &'static str) {
        let Some(release) = self.updates.found.clone() else {
            return;
        };
        match self.updates.status {
            Status::Checking | Status::Downloading(_) => {
                tracing::info!(
                    by,
                    "update: update now waits for the check or download running"
                );
                return;
            }
            Status::Available(_) => {}
            // A later check failed: the version found before is still the one offered.
            _ => self.updates.status = Status::Available(release.clone()),
        }
        let install = self.update_edition() == Some("installer") && release.setup.is_some();
        tracing::info!(by, installs = install, version = %release.version, "update: update now");
        // Nothing else shows until setup starts, and on a slow line that takes minutes.
        if install && let Some(tray) = &self.tray {
            tray.show_info(
                "PecoFence",
                pecofence_core::i18n::text("正在下载新版本…"),
                false,
            );
        }
        self.on_update_action(if install {
            "installUpdate"
        } else {
            "openReleasePage"
        });
    }

    /// What the card did for which version: `notices.json` read once, then the copy here.
    fn card_record(&mut self) -> super::welcome::UpdateCardRecord {
        if self.updates.record.is_none() {
            self.updates.record = Some(self.load_update_card_record());
        }
        self.updates.record.clone().unwrap_or_default()
    }

    fn set_card_record(&mut self, record: super::welcome::UpdateCardRecord) {
        self.save_update_card_record(&record);
        self.updates.record = Some(record);
    }

    /// After the automatic check found a newer version, and on `TIMER_UPDATE_CARD`: its card,
    /// once per version, when it may show. A full-screen app or presentation, the Peek or
    /// another card make it wait and try again.
    pub(super) fn show_update_card(&mut self) {
        if !self.updates.card_waiting {
            return;
        }
        let Some(release) = self.updates.found.clone() else {
            self.updates.card_waiting = false;
            tracing::info!("update card: no newer version any more");
            return;
        };
        if !self.state.config.settings.auto_check_updates {
            self.updates.card_waiting = false;
            tracing::info!("update card: the automatic check was switched off");
            return;
        }
        let record = self.card_record();
        if record.seen == release.version {
            self.updates.card_waiting = false;
            tracing::info!(version = %release.version, "update card: already shown for this version");
            return;
        }
        let offered_before = record.unseen == release.version;
        if offered_before && unix_now() - record.unseen_at < UNSEEN_AGAIN_AFTER_S {
            self.updates.card_waiting = false;
            tracing::info!(version = %release.version, "update card: shown while nobody was there; once more tomorrow");
            return;
        }
        let busy = if !pecofence_platform::store::user_accepts_notifications() {
            Some("full screen or presentation")
        } else if self.peek.is_some() {
            Some("Peek")
        } else if self.notice_showing() {
            Some("another card")
        } else {
            None
        };
        if let Some(why) = busy {
            // The first wait (or a new reason) is logged; the retries every 10 s are not.
            if self.updates.card_wait.is_none_or(|(w, _)| w != why) {
                tracing::info!(why, "update card waits");
                self.updates.card_wait = Some((why, Instant::now()));
            }
            window::set_timer(self.control.hwnd(), TIMER_UPDATE_CARD, CARD_RETRY_MS);
            return;
        }
        if let Some((why, since)) = self.updates.card_wait.take() {
            tracing::info!(
                why,
                waited_s = since.elapsed().as_secs(),
                "update card: the wait is over"
            );
        }
        self.updates.card_waiting = false;
        let install = self.update_edition() == Some("installer") && release.setup.is_some();
        let title = pecofence_core::i18n::format(
            "PecoFence {0} 已发布",
            std::slice::from_ref(&release.version),
        );
        // The Store notes' first point in the UI language; else the release notes' first
        // sentence when it is in the UI language (they are English); else a general line.
        let (body, line_from) = match &release.store_line {
            Some(line) => (updates::shorten(line, CARD_LINE_CHARS), "Store notes"),
            None => match updates::card_line(
                &release.summary,
                pecofence_core::i18n::language().tag(),
                CARD_LINE_CHARS,
            ) {
                Some(line) => (line, "release notes"),
                None => (
                    pecofence_core::i18n::text("看看这次更新了什么。").to_string(),
                    "general",
                ),
            },
        };
        tracing::info!(line_from, "update card: its line");
        let action = if install {
            pecofence_core::i18n::text("按钮\x04更新")
        } else {
            pecofence_core::i18n::text("去下载")
        };
        // The ZIP edition's download is the release page: one button for it.
        let whats_new = install.then(|| pecofence_core::i18n::text("查看新功能"));
        match crate::notice::Notice::show_with(
            &self.ctx,
            &title,
            &body,
            action,
            whats_new,
            crate::notice::locale(),
            &self.notice_avoid_rects(),
        ) {
            Ok(n) => {
                self.updates.notice = Some(n);
                self.updates.card_shown = Some((
                    release.version.clone(),
                    pecofence_platform::sysparams::last_input_tick(),
                    offered_before,
                ));
                tracing::info!(version = %release.version, installs = install, again = offered_before, "update card shown");
            }
            Err(e) => tracing::warn!(error = %e, "update card not shown"),
        }
    }

    /// The card went: its version counts as seen when the user was there (a click, or any
    /// input while it showed) or this was its last offer; else it is offered once more a day
    /// later.
    fn record_update_card(&mut self, clicked: bool) {
        let Some((version, input_then, last_offer)) = self.updates.card_shown.take() else {
            return;
        };
        let input_now = pecofence_platform::sysparams::last_input_tick();
        let there = clicked || input_now != input_then;
        let mut record = self.card_record();
        if there || last_offer {
            record.seen = version.clone();
        } else {
            record.unseen = version.clone();
            record.unseen_at = unix_now();
        }
        tracing::info!(%version, there, last_offer, "update card: recorded");
        self.set_card_record(record);
    }

    pub(super) fn update_notice_open(&self) -> bool {
        self.updates.notice.is_some()
    }

    pub(super) fn update_notice(&self) -> Option<&crate::notice::Notice> {
        self.updates.notice.as_ref()
    }

    /// The card's buttons: 「更新」 / 「去下载」, 「查看新功能」 (the release page); × or the
    /// timeout leave it for later (the tray item stays).
    pub(super) fn on_update_notice(&mut self, event: crate::notice::NoticeEvent) {
        self.updates.notice = None;
        self.record_update_card(event != crate::notice::NoticeEvent::Dismissed);
        match event {
            crate::notice::NoticeEvent::Action => self.update_now("update card"),
            crate::notice::NoticeEvent::Declined => {
                tracing::info!("update card: what's new");
                if let Some(r) = &self.updates.found {
                    open_release_page(&r.page);
                }
            }
            crate::notice::NoticeEvent::Dismissed => {
                tracing::info!("update card: later (closed or timed out)")
            }
        }
    }

    /// Hands over to setup and exits. `/UPDATE=` lists this process and its watchdogs: setup
    /// waits for them before its running-app check and starts PecoFence again when done.
    fn start_setup(&mut self, setup: &Path) -> std::result::Result<(), String> {
        let mut wait = vec![pecofence_platform::process::current_pid()];
        if let Some(a) = self.anchor.borrow().as_ref() {
            wait.extend_from_slice(a.watchdog_pids());
        }
        let wait: Vec<String> = wait.iter().map(u32::to_string).collect();
        std::process::Command::new(setup)
            .args(["/SP-", "/SILENT", "/SUPPRESSMSGBOXES", "/NORESTART"])
            .arg(format!("/UPDATE={}", wait.join(",")))
            .arg(format!(
                "/LOG={}",
                download_dir().join("setup.log").display()
            ))
            .spawn()
            .map_err(|e| format!("{}: {e}", setup.display()))?;
        tracing::info!(setup = %setup.display(), "update setup started; exiting");
        self.queue.push(Command::Quit("update"));
        Ok(())
    }
}
