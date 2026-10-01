//! Manual updater state. Poll a hidden worker; never block the UI on network I/O.

use crate::runtime::Runtime;
use pecofence_core::distribution::{DistributionMode, MARKER_FILE};
use pecofence_core::updates::{self, CheckResult, Release, ReleaseInfo};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

pub struct Updater {
    root: PathBuf,
    directory: PathBuf,
    mode: DistributionMode,
    source: Option<ReleaseInfo>,
    phase: String,
    detail: String,
    release: Option<Release>,
    stage: Option<PathBuf>,
    recovery: Option<PathBuf>,
    job: Option<Job>,
    progress: Value,
    cleanup: Option<CleanupJob>,
    started: Instant,
    cleanup_due: bool,
    last_cleanup: Option<Instant>,
}

struct Job {
    child: Child,
    operation: &'static str,
    started: Instant,
}

struct CleanupJob {
    child: Child,
    stage: PathBuf,
    started: Instant,
}

fn read_text(path: &Path, limit: u64) -> Result<String, String> {
    pecofence_platform::updates::plain_path(path)?;
    if fs::metadata(path).map_err(|e| e.to_string())?.len() > limit {
        return Err("Update metadata exceeded its size limit".into());
    }
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_str(&read_text(path, 16384)?).map_err(|e| e.to_string())
}

impl Updater {
    pub fn new(runtime: &Runtime, instance: Option<&str>) -> Self {
        let mode = runtime.distribution.mode();
        let mut updater = Self {
            root: runtime.distribution.root().to_path_buf(),
            directory: runtime.paths.updates_dir.clone(),
            mode,
            source: None,
            phase: "disabled".into(),
            detail: String::new(),
            release: None,
            stage: None,
            recovery: None,
            job: None,
            progress: Value::Null,
            cleanup: None,
            started: Instant::now(),
            cleanup_due: true,
            last_cleanup: None,
        };
        if mode == DistributionMode::Msix {
            updater.phase = "store".into();
        } else if instance.is_none()
            && matches!(
                mode,
                DistributionMode::Portable | DistributionMode::Installed
            )
            && updater.root.join(MARKER_FILE).is_file()
        {
            match read_text(&updater.root.join(updates::METADATA_FILE), 16384)
                .and_then(|text| ReleaseInfo::parse(&text, env!("CARGO_PKG_VERSION")))
            {
                Ok(info) => {
                    updater.source = Some(info);
                    updater.phase = "idle".into();
                }
                Err(error) => updater.detail = error,
            }
            updater.restore_local_state();
        }
        updater
    }

    fn restore_local_state(&mut self) {
        if pecofence_platform::updates::plain_path(&self.directory).is_err() {
            return;
        }
        let Ok(entries) = fs::read_dir(&self.directory) else {
            return;
        };
        let mut ready = None;
        for entry in entries.flatten() {
            let stage = entry.path();
            let Some(name) = stage.file_name().and_then(|v| v.to_str()) else {
                continue;
            };
            if name.len() != 32 || !name.bytes().all(|c| c.is_ascii_hexdigit()) {
                continue;
            }
            let Ok(plan) = read_json(&stage.join("plan.json")) else {
                continue;
            };
            if plan["root"].as_str() != self.root.to_str() || plan["mode"] != self.mode.as_str() {
                continue;
            }
            if let Ok(journal) = read_json(&stage.join("transaction.json"))
                && self.mode == DistributionMode::Portable
                && journal["root"].as_str() == self.root.to_str()
                && matches!(
                    journal["state"].as_str(),
                    Some("applying" | "recoveryRequired")
                )
            {
                self.recovery = Some(stage);
                self.phase = "recovery".into();
                return;
            }
            if self.mode == DistributionMode::Installed
                && let Ok(journal) = read_json(&stage.join("installer.json"))
                && journal["root"].as_str() == self.root.to_str()
                && matches!(
                    journal["state"].as_str(),
                    Some("installing" | "interrupted")
                )
            {
                self.recovery = Some(stage);
                self.phase = "recovery".into();
                return;
            }
            if let Ok(verified) = read_json(&stage.join("verified.json"))
                && !stage.join("transaction.json").exists()
                && !stage.join("installer.json").exists()
                && !stage.join("handoff.json").exists()
                && let (Some(source), Ok(release)) = (
                    self.source.as_ref(),
                    serde_json::from_value::<Release>(plan["release"].clone()),
                )
                && plan["currentVersion"] == source.version
                && plan["repository"] == source.repository
                && release.validate(source, self.mode).is_ok()
                && verified["asset"] == release.asset
                && verified["sha256"].as_str().is_some_and(|hash| {
                    hash.len() == 64 && hash.bytes().all(|c| c.is_ascii_hexdigit())
                })
                && fs::metadata(stage.join(&release.asset))
                    .is_ok_and(|metadata| metadata.is_file() && metadata.len() == release.bytes)
            {
                // Directory names are random, so explicitly prefer the last saved plan.
                let modified = entry.metadata().ok().and_then(|m| m.modified().ok());
                if ready.as_ref().is_none_or(|(when, _, _)| modified > *when) {
                    ready = Some((modified, stage, release));
                }
            }
        }
        if let Some((_, stage, release)) = ready {
            self.stage = Some(stage);
            self.release = Some(release);
            self.phase = "ready".into();
        }
    }

    pub fn snapshot(&self) -> Value {
        json!({
            "phase": self.phase, "detail": self.detail, "mode": self.mode.as_str(),
            "repository": self.source.as_ref().map(|info| &info.repository),
            "release": self.release, "progress": self.progress,
            "directory": self.directory, "busy": self.busy(),
        })
    }

    pub fn busy(&self) -> bool {
        self.job.is_some() || self.cleanup.is_some()
    }

    pub fn open_release_url(&self) -> Option<String> {
        if self.mode == DistributionMode::Msix {
            Some("https://apps.microsoft.com/detail/9MV6WG3XNWSX".into())
        } else {
            self.source.as_ref().map(|info| {
                self.release
                    .as_ref()
                    .map_or_else(|| info.releases_url(), |r| r.page.clone())
            })
        }
    }

    fn new_stage(&mut self) -> Result<(), String> {
        self.stage = Some(self.create_stage()?);
        Ok(())
    }

    fn create_stage(&self) -> Result<PathBuf, String> {
        pecofence_platform::updates::plain_path(&self.root)?;
        pecofence_platform::updates::plain_path(&self.directory)?;
        crate::runtime::writable_directory(&self.directory)?;
        let stage = self
            .directory
            .join(uuid::Uuid::new_v4().simple().to_string());
        fs::create_dir(&stage).map_err(|e| e.to_string())?;
        Ok(stage)
    }

    fn write_plan(&self) -> Result<(), String> {
        let source = self
            .source
            .as_ref()
            .ok_or("No valid packaged update source")?;
        let path = self
            .stage
            .as_ref()
            .ok_or("Missing update stage")?
            .join("plan.json");
        pecofence_platform::updates::plain_path(&path)?;
        pecofence_platform::updates::write_plan(
            &path,
            json!({
                "schema": 1, "root": self.root, "mode": self.mode.as_str(),
                "repository": source.repository, "currentVersion": source.version,
                "release": self.release, "parentPid": std::process::id(),
            })
            .to_string()
            .as_bytes(),
        )
    }

    fn spawn(&mut self, operation: &'static str) -> Result<(), String> {
        let stage = self.stage.as_ref().ok_or("Missing update directory")?;
        // Stale result/hand-off files must never authorize a new operation.
        for name in ["result.json", "handoff.json", "progress.json"] {
            let path = stage.join(name);
            pecofence_platform::updates::plain_path(&path)?;
            match fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.to_string()),
            }
        }
        let child = pecofence_platform::updates::start(stage, operation)?;
        self.job = Some(Job {
            child,
            operation,
            started: Instant::now(),
        });
        self.cleanup_due = true;
        self.detail.clear();
        self.progress = Value::Null;
        self.phase = match operation {
            "Check" => "checking",
            "Download" => "downloading",
            _ => "applying",
        }
        .into();
        Ok(())
    }

    pub fn begin(&mut self, operation: &'static str) -> Result<(), String> {
        if self.busy() {
            return Err("An update operation is already running".into());
        }
        if operation == "Recover" {
            let stage = self.recovery.clone().ok_or("No interrupted update")?;
            self.stage = Some(stage);
            return self.spawn(operation);
        }
        if self.recovery.is_some() {
            return Err("Recover the interrupted update first".into());
        }
        let source = self
            .source
            .as_ref()
            .ok_or("Updates require a marked release package with matching metadata")?;
        // Re-read identity before every request so editing metadata does not silently
        // redirect a staged release or change the source shown in Settings.
        let current = ReleaseInfo::parse(
            &read_text(&self.root.join(updates::METADATA_FILE), 16384)?,
            env!("CARGO_PKG_VERSION"),
        )?;
        if &current != source {
            return Err("Package metadata changed; restart PecoFence".into());
        }
        match operation {
            "Check" => {
                self.release = None;
                self.new_stage()?;
            }
            "Download" => {
                self.release
                    .as_ref()
                    .ok_or("Check for updates first")?
                    .validate(source, self.mode)?;
                self.new_stage()?;
            }
            "Apply" if self.phase == "ready" => {
                self.release
                    .as_ref()
                    .ok_or("Download the update first")?
                    .validate(source, self.mode)?;
            }
            _ => return Err("This update action is not available".into()),
        }
        self.write_plan()?;
        self.spawn(operation)
    }

    pub fn fail(&mut self, error: String) {
        tracing::warn!(%error, "updater");
        self.phase = if self.recovery.is_some() {
            "recovery"
        } else {
            "error"
        }
        .into();
        self.detail = error;
    }

    fn cleanup_is_due(&self) -> bool {
        !self.busy()
            && self.source.is_some()
            && self.recovery.is_none()
            && self.started.elapsed() >= Duration::from_secs(30)
            && (self.cleanup_due
                || self
                    .last_cleanup
                    .is_none_or(|last| last.elapsed() >= Duration::from_secs(3600)))
    }

    /// Called from a delayed message-loop timer, never during startup discovery.
    /// This local maintenance does not check for, download, or apply updates.
    pub fn maybe_cleanup(&mut self) -> bool {
        if !self.cleanup_is_due() {
            return false;
        }
        self.cleanup_due = false;
        self.last_cleanup = Some(Instant::now());
        if !self.directory.is_dir() {
            return false;
        }
        let result = (|| -> Result<CleanupJob, String> {
            let source = self.source.as_ref().unwrap();
            let current = ReleaseInfo::parse(
                &read_text(&self.root.join(updates::METADATA_FILE), 16384)?,
                env!("CARGO_PKG_VERSION"),
            )?;
            if &current != source {
                return Err("Package metadata changed; cleanup deferred".into());
            }
            let stage = self.create_stage()?;
            let protected = (self.phase == "ready")
                .then(|| self.stage.as_ref().and_then(|p| p.file_name()))
                .flatten()
                .and_then(|name| name.to_str());
            pecofence_platform::updates::write_plan(
                &stage.join("plan.json"),
                json!({"schema":1,"root":self.root,"mode":self.mode.as_str(),
                    "repository":source.repository,"currentVersion":source.version,
                    "release":null,"protectedAttempt":protected})
                .to_string()
                .as_bytes(),
            )?;
            Ok(CleanupJob {
                child: pecofence_platform::updates::start(&stage, "Cleanup")?,
                stage,
                started: Instant::now(),
            })
        })();
        match result {
            Ok(job) => {
                self.cleanup = Some(job);
                true
            }
            Err(error) => {
                tracing::warn!(%error, "update cleanup deferred");
                false
            }
        }
    }

    fn poll_cleanup(&mut self) {
        let job = self.cleanup.as_mut().unwrap();
        let result = match job.child.try_wait() {
            Ok(None) if job.started.elapsed() < Duration::from_secs(120) => return,
            Ok(Some(status)) => read_json(&job.stage.join("result.json")).and_then(|value| {
                if status.success() && value["status"] == "ok" {
                    Ok(value["cleanup"].clone())
                } else {
                    Err(value["message"]
                        .as_str()
                        .unwrap_or("Cleanup worker failed")
                        .to_owned())
                }
            }),
            _ => {
                let _ = job.child.kill();
                let _ = job.child.wait();
                Err("Cleanup interrupted; remaining files will be checked later".into())
            }
        };
        self.cleanup.take();
        match result {
            Ok(report) => tracing::info!(%report, "update cleanup finished"),
            Err(error) => tracing::warn!(%error, "update cleanup deferred"),
        }
    }

    /// Returns true only after a validated apply/recovery worker has captured our
    /// process handle and is waiting for normal shutdown. Transfer its ownership.
    pub fn poll(&mut self) -> bool {
        if self.cleanup.is_some() {
            self.poll_cleanup();
            return false;
        }
        let Some(job) = self.job.as_mut() else {
            return false;
        };
        let stage = self.stage.as_ref().unwrap();
        if matches!(job.operation, "Apply" | "Recover")
            && let Ok(handoff) = read_json(&stage.join("handoff.json"))
            && handoff["status"] == "ready"
            && matches!(job.child.try_wait(), Ok(None))
        {
            return true;
        }
        if let Ok(progress) = read_json(&stage.join("progress.json")) {
            self.progress = progress;
        }
        if job.started.elapsed() > Duration::from_secs(600) {
            let _ = job.child.kill();
            let _ = job.child.wait();
            self.job.take();
            self.fail("The update worker timed out; no update was authorized to install".into());
            return false;
        }
        let status = job.child.try_wait();
        let result = match status {
            Ok(None) => return false,
            Ok(Some(status)) => {
                let operation = job.operation;
                read_json(&stage.join("result.json")).and_then(|result| {
                    if !status.success() || result["status"] == "error" {
                        return Err(result["message"]
                            .as_str()
                            .unwrap_or("Update worker failed; see the update log")
                            .to_owned());
                    }
                    if operation == "Check" {
                        if result["status"] == "noRelease" {
                            self.phase = "noRelease".into();
                        } else {
                            let response = read_text(&stage.join("release.json"), 2 * 1024 * 1024)?;
                            match updates::check_release(
                                self.source.as_ref().unwrap(),
                                self.mode,
                                &response,
                            )? {
                                CheckResult::UpToDate => self.phase = "upToDate".into(),
                                CheckResult::Available(release) => {
                                    self.release = Some(release);
                                    self.phase = "available".into();
                                }
                            }
                        }
                    } else if operation == "Download" {
                        self.phase = "ready".into();
                    } else {
                        return Err("The update worker exited before application shutdown".into());
                    }
                    Ok(())
                })
            }
            Err(error) => {
                let _ = job.child.kill();
                let _ = job.child.wait();
                Err(error.to_string())
            }
        };
        self.job.take();
        if let Err(error) = result {
            self.fail(error);
        }
        false
    }

    pub fn cancel(&mut self) {
        if let Some(mut job) = self.cleanup.take() {
            let _ = job.child.kill();
            let _ = job.child.wait();
        }
        if let Some(mut job) = self.job.take() {
            let _ = job.child.kill();
            let _ = job.child.wait();
        }
    }

    pub fn detach(&mut self) {
        self.job.take(); // Child::drop does not terminate the validated apply worker.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pecofence_core::{distribution::Distribution, runtime_paths::RuntimePaths};

    struct Fixture(Runtime);

    impl Fixture {
        fn new(mode: &str) -> Self {
            let root =
                std::env::temp_dir().join(format!("pecofence-updater-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&root).unwrap();
            let marker = json!({"schema":1,"appId":"PecoFence","mode":mode}).to_string();
            fs::write(root.join(MARKER_FILE), &marker).unwrap();
            fs::write(root.join(updates::METADATA_FILE), json!({
                "schema":1,"repository":"Contributor/PecoFence",
                "version":env!("CARGO_PKG_VERSION"),"tag":format!("v{}",env!("CARGO_PKG_VERSION"))
            }).to_string()).unwrap();
            let distribution =
                Distribution::resolve(&root.join("pecofence.exe"), Some(&marker), false, false)
                    .unwrap();
            let paths = RuntimePaths::resolve(
                &distribution,
                Some(&root.join("roaming")),
                Some(&root.join("local")),
                None,
            )
            .unwrap();
            Self(Runtime {
                distribution,
                paths,
            })
        }

        fn saved_download(&self) -> PathBuf {
            let mut updater = Updater::new(&self.0, None);
            updater.new_stage().unwrap();
            let asset = updates::asset_name("999.0.0", updater.mode).unwrap();
            let base = "https://github.com/Contributor/PecoFence/releases";
            updater.release = Some(Release {
                version: "999.0.0".into(),
                page: format!("{base}/tag/v999.0.0"),
                url: format!("{base}/download/v999.0.0/{asset}"),
                checksum_url: format!("{base}/download/v999.0.0/{asset}.sha256"),
                asset: asset.clone(),
                bytes: 1,
                digest: None,
            });
            updater.write_plan().unwrap();
            let stage = updater.stage.unwrap();
            fs::write(stage.join(&asset), b"x").unwrap();
            fs::write(
                stage.join("verified.json"),
                json!({"asset":asset,"sha256":"a".repeat(64)}).to_string(),
            )
            .unwrap();
            stage
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(self.0.distribution.root()).unwrap();
        }
    }

    #[test]
    fn startup_has_no_network_or_writes_and_preserves_fork_source() {
        let fixture = Fixture::new("portable");
        let updater = Updater::new(&fixture.0, None);
        assert_eq!(updater.phase, "idle");
        assert!(
            !updater.cleanup_is_due(),
            "cleanup must wait for the message loop"
        );
        assert!(!updater.busy());
        assert!(!updater.directory.exists());
        assert_eq!(
            updater.open_release_url().as_deref(),
            Some("https://github.com/Contributor/PecoFence/releases")
        );
        assert_eq!(Updater::new(&fixture.0, Some("test")).phase, "disabled");
        fs::remove_file(fixture.0.distribution.root().join(updates::METADATA_FILE)).unwrap();
        let mut invalid = Updater::new(&fixture.0, None);
        assert_eq!(invalid.phase, "disabled");
        assert!(invalid.open_release_url().is_none());
        assert!(invalid.begin("Check").is_err());
        assert!(!invalid.directory.exists());
    }

    #[test]
    fn completed_download_resumes_after_restart_but_partial_download_does_not() {
        let fixture = Fixture::new("portable");
        let stage = fixture.saved_download();
        let updater = Updater::new(&fixture.0, None);
        assert_eq!(updater.phase, "ready");
        assert!(!updater.busy());
        let asset = updater.release.unwrap().asset;
        fs::write(stage.join(&asset), b"").unwrap();
        assert_eq!(Updater::new(&fixture.0, None).phase, "idle");
        fs::write(stage.join(&asset), b"x").unwrap();
        fs::write(stage.join("verified.json"), "partial").unwrap();
        assert_eq!(Updater::new(&fixture.0, None).phase, "idle");
    }

    #[test]
    fn portable_restart_prioritizes_recovery_even_with_mixed_version_metadata() {
        let fixture = Fixture::new("portable");
        let stage = fixture.saved_download();
        for state in ["applying", "recoveryRequired"] {
            fs::write(
                stage.join("transaction.json"),
                json!({"state":state,"root":fixture.0.distribution.root()}).to_string(),
            )
            .unwrap();
            fs::write(
                fixture.0.distribution.root().join(updates::METADATA_FILE),
                "interrupted metadata",
            )
            .unwrap();
            let mut updater = Updater::new(&fixture.0, None);
            assert_eq!(updater.phase, "recovery");
            updater.started = Instant::now() - Duration::from_secs(60);
            assert!(!updater.cleanup_is_due(), "pending recovery blocks cleanup");
            assert_eq!(updater.recovery.as_ref(), Some(&stage));
            assert!(updater.begin("Check").is_err());
            assert!(updater.begin("Download").is_err());
            assert!(updater.begin("Apply").is_err());
            assert!(!updater.busy(), "recovery must wait for user confirmation");
        }
    }

    #[test]
    fn installer_restart_recovers_only_pending_journal_for_this_copy() {
        let fixture = Fixture::new("installed");
        let stage = fixture.saved_download();
        for (state, expected) in [
            ("installing", "recovery"),
            ("interrupted", "recovery"),
            ("complete", "idle"),
        ] {
            fs::write(
                stage.join("installer.json"),
                json!({"state":state,"root":fixture.0.distribution.root()}).to_string(),
            )
            .unwrap();
            let updater = Updater::new(&fixture.0, None);
            assert_eq!(updater.phase, expected);
            assert!(!updater.busy());
        }
        fs::write(
            stage.join("installer.json"),
            json!({"state":"installing","root":"another copy"}).to_string(),
        )
        .unwrap();
        assert_eq!(Updater::new(&fixture.0, None).phase, "idle");
    }

    #[test]
    fn changed_package_repository_cannot_redirect_an_existing_update() {
        let fixture = Fixture::new("portable");
        let mut updater = Updater::new(&fixture.0, None);
        let metadata = fixture.0.distribution.root().join(updates::METADATA_FILE);
        fs::write(
            &metadata,
            fs::read_to_string(&metadata)
                .unwrap()
                .replace("Contributor/PecoFence", "Different/PecoFence"),
        )
        .unwrap();
        assert!(
            updater
                .begin("Check")
                .unwrap_err()
                .contains("metadata changed")
        );
        assert!(!updater.directory.exists());
    }

    #[test]
    fn cleanup_waits_for_startup_and_is_throttled_without_network() {
        let fixture = Fixture::new("portable");
        let mut updater = Updater::new(&fixture.0, None);
        assert!(!updater.maybe_cleanup());
        assert!(!updater.directory.exists());
        updater.started = Instant::now() - Duration::from_secs(31);
        assert!(updater.cleanup_is_due());
        assert!(
            !updater.maybe_cleanup(),
            "no cache means no worker or writes"
        );
        assert!(!updater.directory.exists());
        assert!(!updater.cleanup_is_due());
        updater.last_cleanup = Some(Instant::now() - Duration::from_secs(3601));
        assert!(updater.cleanup_is_due());
        updater.source = None;
        assert!(!updater.cleanup_is_due());
    }

    #[test]
    fn changed_identity_defers_cleanup_without_changing_update_ui_state() {
        let fixture = Fixture::new("portable");
        let stage = fixture.saved_download();
        let mut updater = Updater::new(&fixture.0, None);
        let before = updater.snapshot();
        updater.started = Instant::now() - Duration::from_secs(31);
        fs::write(
            fixture.0.distribution.root().join(updates::METADATA_FILE),
            "changed",
        )
        .unwrap();
        assert!(!updater.maybe_cleanup());
        assert_eq!(updater.snapshot(), before);
        assert!(stage.join("verified.json").is_file());
    }
}
