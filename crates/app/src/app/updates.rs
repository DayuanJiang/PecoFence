//! Settings → 关于 → 「检查更新」 for the installer and ZIP editions; the Store updates the app
//! itself. Checking and downloading run on a worker thread. An installed copy then starts the
//! new setup and exits; setup waits for it, installs and starts PecoFence again.
//!
//! 自动检查更新 (on by default): about a minute after start and then once a day the same check
//! runs by itself (installer and ZIP editions; never the Store edition or a named test
//! instance), quietly: a failure only logs a line and the next day tries again. A newer version
//! is announced once (never over a full-screen app, during Focus or the Peek) and stays at the
//! top of the tray menu. How it is announced is a sample for now (`PECOFENCE_SAMPLE_UPDATE`):
//! A a card in the corner, B a Windows notification, C a dot on the tray icon.

use super::*;
use pecofence_core::updates::{self, Release};

/// A named test instance's release API (`http://127.0.0.1:` only, and only with
/// `PECOFENCE_INSTANCE`): the updater's end-to-end test serves a release from a local server.
const URL_VAR: &str = "PECOFENCE_UPDATE_URL";
const MAX_RELEASE_JSON: usize = 1024 * 1024;
/// The automatic check (`TIMER_UPDATE_CHECK`); retried while the App is borrowed.
pub(super) const TIMER_UPDATE_CHECK: usize = 97;
const FIRST_CHECK_MS: u32 = 60_000;
const DAILY_CHECK_MS: u32 = 24 * 60 * 60 * 1000;
/// Samples: the first check comes sooner.
const SAMPLE_FIRST_CHECK_MS: u32 = 5_000;

/// `PECOFENCE_SAMPLE_UPDATE=A|B|C`: which announcement to show (temporary, for the samples);
/// it also lets a named test instance check by itself.
fn sample() -> Option<char> {
    pecofence_core::brand::var("PECOFENCE_SAMPLE_UPDATE")
        .ok()
        .and_then(|v| v.trim().chars().next())
        .map(|c| c.to_ascii_uppercase())
        .filter(|c| matches!(c, 'A' | 'B' | 'C'))
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
    /// The check running now started by itself (quiet, announces what it finds).
    auto: bool,
    /// The version announced in this run (once per version).
    announced: Option<String>,
    /// A newer version waits for a moment it may be announced (a full-screen app, the Peek).
    pending: bool,
    /// Sample A's card.
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
            ("openReleasePage", Status::Available(release)) => {
                let page = release.page.clone();
                if let Err(e) = shell::shell_execute(Path::new(&page), None, None) {
                    tracing::warn!(error = %e, "opening the release page failed");
                }
            }
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
                Ok(Some(None)) => Status::UpToDate,
                Ok(Some(Some(release))) => {
                    tracing::info!(version = %release.version, auto, "update available");
                    if auto {
                        self.updates.pending = true;
                    }
                    Status::Available(release)
                }
                // Offline, a metered connection Windows holds back, GitHub down: quiet, the next
                // day tries again.
                Err(e) if auto => {
                    tracing::info!(error = %e, "update: the automatic check failed; next try tomorrow");
                    Status::Idle
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
        self.announce_update();
    }

    /// Arms the automatic check: the first about a minute after start, then once a day.
    pub(super) fn arm_update_check(&self, first: bool) {
        let ms = match (first, sample()) {
            (false, _) => DAILY_CHECK_MS,
            (true, Some(_)) => SAMPLE_FIRST_CHECK_MS,
            (true, None) => FIRST_CHECK_MS,
        };
        window::set_timer(self.control.hwnd(), TIMER_UPDATE_CHECK, ms);
    }

    /// `TIMER_UPDATE_CHECK`: the automatic check, when it is on and this edition checks.
    pub(super) fn auto_check_updates(&mut self) {
        self.arm_update_check(false);
        let on = self.state.config.settings.auto_check_updates;
        let edition = self.update_edition();
        let test_instance = self.instance.is_some();
        if !on || edition.is_none() || (test_instance && sample().is_none()) {
            tracing::info!(
                on,
                store = edition.is_none(),
                test_instance,
                "update: no automatic check"
            );
            return;
        }
        if matches!(
            self.updates.status,
            Status::Checking | Status::Downloading(_)
        ) {
            return;
        }
        tracing::info!("update: automatic check");
        self.updates.auto = true;
        self.run_update_job(Status::Checking, move || check(test_instance));
    }

    /// The newer version the tray menu offers (「更新到 0.3.3」).
    pub(super) fn update_offered(&self) -> Option<String> {
        match &self.updates.status {
            Status::Available(r) => Some(r.version.clone()),
            _ => None,
        }
    }

    /// 「更新」 (tray item, the card): the installer edition downloads and installs, the ZIP
    /// edition opens the download page.
    pub(super) fn update_now(&mut self, by: &'static str) {
        let installer = self.update_edition() == Some("installer")
            && matches!(&self.updates.status, Status::Available(r) if r.setup.is_some());
        tracing::info!(by, installer, "update: update now");
        self.on_update_action(if installer {
            "installUpdate"
        } else {
            "openReleasePage"
        });
    }

    /// Housekeeping and after a check: a newer version found by the automatic check is
    /// announced once, when it may be (no full-screen app, Focus or presentation, no Peek, no
    /// other notice).
    pub(super) fn announce_update(&mut self) {
        if !self.updates.pending {
            return;
        }
        let Status::Available(release) = &self.updates.status else {
            self.updates.pending = false;
            return;
        };
        if self.updates.announced.as_deref() == Some(release.version.as_str()) {
            self.updates.pending = false;
            return;
        }
        if !pecofence_platform::store::user_accepts_notifications()
            || self.peek.is_some()
            || self.notice_showing()
        {
            tracing::info!("update: announcement waits (full screen, Focus, Peek or a notice)");
            return;
        }
        let release = release.clone();
        self.updates.pending = false;
        self.updates.announced = Some(release.version.clone());
        let zip = self.update_edition() == Some("zip") || release.setup.is_none();
        let title = pecofence_core::i18n::format(
            "PecoFence {0} 已发布",
            std::slice::from_ref(&release.version),
        );
        let body = if release.summary.is_empty() {
            pecofence_core::i18n::text("看看这次更新了什么。").to_string()
        } else {
            release.summary.clone()
        };
        let how = sample().unwrap_or('C');
        tracing::info!(version = %release.version, how = %how, zip, "update: announced");
        match how {
            'A' => {
                let action = if zip {
                    pecofence_core::i18n::text("去下载")
                } else {
                    pecofence_core::i18n::text("更新")
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
                    Ok(n) => self.updates.notice = Some(n),
                    Err(e) => tracing::warn!(error = %e, "update notice not shown"),
                }
            }
            'B' => {
                if let Some(t) = &self.tray {
                    t.show_info(&title, &body, false);
                }
            }
            _ => self.set_tray_dot(true),
        }
    }

    /// Sample C: the tray icon with (or without) its dot.
    fn set_tray_dot(&mut self, on: bool) {
        let px = (16.0 * (monitors::system_dpi().max(96) as f32 / 96.0)).round() as i32;
        let image = if on {
            super::visuals::tray_icon_with_dot(px, [0, 0, 0], false)
        } else {
            super::visuals::tray_icon_image(px, [0, 0, 0], false)
        };
        match tray::icon_from_bgra(px, px, &image) {
            Ok(icon) => {
                if let Some(t) = self.tray.as_mut() {
                    t.set_icon(icon);
                }
            }
            Err(e) => tracing::warn!(error = %e, "tray icon with a dot failed"),
        }
    }

    pub(super) fn update_notice_open(&self) -> bool {
        self.updates.notice.is_some()
    }

    /// Sample A's card: 「更新」, 「查看新功能」 (the release page), × (later: the tray item
    /// stays).
    pub(super) fn on_update_notice(&mut self, event: crate::notice::NoticeEvent) {
        self.updates.notice = None;
        match event {
            crate::notice::NoticeEvent::Action => self.update_now("update notice"),
            crate::notice::NoticeEvent::Declined => {
                if let Status::Available(r) = &self.updates.status {
                    let page = r.page.clone();
                    if let Err(e) = shell::shell_execute(Path::new(&page), None, None) {
                        tracing::warn!(error = %e, "opening the release page failed");
                    }
                }
            }
            crate::notice::NoticeEvent::Dismissed => tracing::info!("update notice: later"),
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
