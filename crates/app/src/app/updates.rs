//! Settings → 关于 → 「检查更新」 for the installer and ZIP editions; the Store updates the app
//! itself. Checking and downloading run on a worker thread. An installed copy then starts the
//! new setup and exits; setup waits for it, installs and starts PecoFence again.

use super::*;
use pecofence_core::updates::{self, Release};

/// A named test instance's release API (`http://127.0.0.1:` only, and only with
/// `PECOFENCE_INSTANCE`): the updater's end-to-end test serves a release from a local server.
const URL_VAR: &str = "PECOFENCE_UPDATE_URL";
const MAX_RELEASE_JSON: usize = 1024 * 1024;
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
                    tracing::info!(version = %release.version, "update available");
                    Status::Available(release)
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
