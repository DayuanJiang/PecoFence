//! Settings → 关于 → 「检查更新」 for the installer and ZIP editions; the Store updates the app
//! itself. Checking and downloading run on a worker thread. An installed copy then starts the
//! new setup and exits; setup waits for it, installs and starts PecoFence again.
//!
//! 自动检查更新 (Settings → 关于, on by default; the installer and ZIP editions, never the Store
//! edition, a named test instance only with the test feed): about a minute after start and then
//! once a day the same request runs by itself. Offline, an error or a rate limit only logs a line
//! and the next day tries again. A newer version gets a card in the corner once per version
//! (`notices.json`), never over a full-screen app, during Focus, the Peek or another card (it
//! waits and tries again), and stays at the top of the tray menu until it is installed.

use super::*;
use pecofence_core::updates::{self, Release};

/// A named test instance's release API (`http://127.0.0.1:` only, and only with
/// `PECOFENCE_INSTANCE`): the updater's end-to-end test serves a release from a local server.
const URL_VAR: &str = "PECOFENCE_UPDATE_URL";
const MAX_RELEASE_JSON: usize = 1024 * 1024;
/// The automatic check, and a card that waits (`TIMER_UPDATE_CHECK`, `TIMER_UPDATE_CARD`);
/// retried while the App is borrowed.
pub(super) const TIMER_UPDATE_CHECK: usize = 97;
pub(super) const TIMER_UPDATE_CARD: usize = 98;
const FIRST_CHECK_MS: u32 = 60_000;
const DAILY_CHECK_MS: u32 = 24 * 60 * 60 * 1000;
/// A card that may not show yet tries again this much later.
const CARD_RETRY_MS: u32 = 10_000;
/// The card's line from the notes stops about here (three lines of the card).
const CARD_LINE_CHARS: usize = 110;

/// The automatic check's delays (after start, between checks). Debug builds take
/// `PECOFENCE_TEST_UPDATE_MS=<after start>[,<between checks>]` (the kit's update-check).
fn check_delays() -> (u32, u32) {
    #[cfg(debug_assertions)]
    if let Ok(v) = pecofence_core::brand::var("PECOFENCE_TEST_UPDATE_MS") {
        let mut parts = v.split(',').map(|p| p.trim().parse::<u32>().ok());
        if let Some(Some(first)) = parts.next() {
            return (first, parts.next().flatten().unwrap_or(DAILY_CHECK_MS));
        }
    }
    (FIRST_CHECK_MS, DAILY_CHECK_MS)
}

/// The test feed is set (`PECOFENCE_UPDATE_URL`, `http://127.0.0.1:` only).
fn test_feed() -> bool {
    pecofence_core::brand::var(URL_VAR).is_ok_and(|u| u.starts_with("http://127.0.0.1:"))
}

/// The release page in the browser (the ZIP edition's download, 「查看新功能」).
fn open_release_page(page: &str) {
    match shell::shell_execute(Path::new(page), None, None) {
        Ok(_) => tracing::info!("update: release page opened"),
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
    notice: Option<crate::notice::Notice>,
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

/// The release API and the prefix its files must be downloaded from.
fn source(test_instance: bool) -> (String, String) {
    match pecofence_core::brand::var(URL_VAR) {
        Ok(url) if test_instance && url.starts_with("http://127.0.0.1:") => {
            let origin = url.split('/').take(3).collect::<Vec<_>>().join("/") + "/";
            (url, origin)
        }
        _ => (updates::latest_release_url(), updates::download_base()),
    }
}

fn check(test_instance: bool) -> std::result::Result<Done, String> {
    let (url, base) = source(test_instance);
    let body = pecofence_platform::http::get(
        &url,
        &user_agent(),
        "Accept: application/vnd.github+json",
        MAX_RELEASE_JSON,
    )?;
    let release = updates::parse_release(&String::from_utf8_lossy(&body), &base)?;
    Ok(Done::Checked(
        updates::is_newer(&release.version, env!("CARGO_PKG_VERSION")).then_some(release),
    ))
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
                self.run_update_job(Status::Checking, move || check(test_instance));
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
                    Status::UpToDate
                }
                Ok(Some(Some(release))) => {
                    tracing::info!(version = %release.version, auto, "update available");
                    self.updates.found = Some(release.clone());
                    if auto {
                        self.updates.card_waiting = true;
                    }
                    Status::Available(release)
                }
                // Offline, an error or a rate limit: nothing shown, a version found before stays
                // offered, the next day tries again.
                Err(e) if auto => {
                    tracing::info!(error = %e, "update: the automatic check failed; next try tomorrow");
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
        }
        self.show_update_card();
    }

    /// Arms the automatic check: the first about a minute after start, then once a day.
    pub(super) fn arm_update_check(&self, first: bool) {
        let (after_start, every) = check_delays();
        let ms = if first { after_start } else { every };
        window::set_timer(self.control.hwnd(), TIMER_UPDATE_CHECK, ms);
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
        self.run_update_job(Status::Checking, move || check(test_instance));
    }

    /// The newer version the tray menu offers at its top (「更新到 0.3.3」) until it is installed.
    pub(super) fn update_offered(&self) -> Option<String> {
        self.updates.found.as_ref().map(|r| r.version.clone())
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
        self.on_update_action(if install {
            "installUpdate"
        } else {
            "openReleasePage"
        });
    }

    /// After the automatic check found a newer version, and on `TIMER_UPDATE_CARD`: its card,
    /// once per version, when it may show. A full-screen app or presentation, Focus, the Peek
    /// or another card make it wait and try again.
    pub(super) fn show_update_card(&mut self) {
        if !self.updates.card_waiting {
            return;
        }
        let Some(release) = self.updates.found.clone() else {
            self.updates.card_waiting = false;
            tracing::info!("update card: no newer version any more");
            return;
        };
        if self.update_card_shown_for().as_deref() == Some(release.version.as_str()) {
            self.updates.card_waiting = false;
            tracing::info!(version = %release.version, "update card: already shown for this version");
            return;
        }
        let busy = if !pecofence_platform::store::user_accepts_notifications() {
            Some("full screen, presentation or Focus")
        } else if self.peek.is_some() {
            Some("Peek")
        } else if self.notice_showing() {
            Some("another card")
        } else {
            None
        };
        if let Some(why) = busy {
            tracing::info!(why, "update card waits");
            window::set_timer(self.control.hwnd(), TIMER_UPDATE_CARD, CARD_RETRY_MS);
            return;
        }
        self.updates.card_waiting = false;
        let install = self.update_edition() == Some("installer") && release.setup.is_some();
        let title = pecofence_core::i18n::format(
            "PecoFence {0} 已发布",
            std::slice::from_ref(&release.version),
        );
        // The notes are written in English or Chinese: other UI languages get the general line.
        let body = updates::card_line(
            &release.summary,
            pecofence_core::i18n::language().tag(),
            CARD_LINE_CHARS,
        )
        .unwrap_or_else(|| pecofence_core::i18n::text("看看这次更新了什么。").to_string());
        let action = if install {
            pecofence_core::i18n::text("按钮\x04更新")
        } else {
            pecofence_core::i18n::text("去下载")
        };
        match crate::notice::Notice::show_with(
            &self.ctx,
            &title,
            &body,
            action,
            Some(pecofence_core::i18n::text("查看新功能")),
            crate::notice::locale(),
            &self.notice_avoid_rects(),
        ) {
            Ok(n) => {
                self.updates.notice = Some(n);
                self.remember_update_card(&release.version);
                tracing::info!(version = %release.version, installs = install, "update card shown");
            }
            Err(e) => tracing::warn!(error = %e, "update card not shown"),
        }
    }

    pub(super) fn update_notice_open(&self) -> bool {
        self.updates.notice.is_some()
    }

    /// The card's buttons: 「更新」 / 「去下载」, 「查看新功能」 (the release page); × or the
    /// timeout leave it for later (the tray item stays).
    pub(super) fn on_update_notice(&mut self, event: crate::notice::NoticeEvent) {
        self.updates.notice = None;
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
