//! Read-only diagnostics using the same distribution paths as the app.

use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use pecofence_core::config_store::{self, LintLevel};
use pecofence_core::distribution::Distribution;
use pecofence_core::runtime_paths::RuntimePaths;
use pecofence_ipc::{ErrorCode, IpcError, StatusDto};
use serde_json::{Value, json};

/// Where the files are (or would be). `config` comes from a running instance when one answers
/// (`source: "status"`), else from the default location (`source: "default"`).
#[derive(Debug, Clone, PartialEq)]
pub struct Paths {
    pub instance: Option<String>,
    pub running: bool,
    pub source: &'static str,
    pub config: PathBuf,
    pub log: PathBuf,
    pub log_dir: PathBuf,
    pub crash_dir: PathBuf,
    pub webview_data_dir: PathBuf,
    pub recovery_marker: PathBuf,
    pub distribution: String,
}

impl Paths {
    pub fn config_dir(&self) -> PathBuf {
        self.config
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.config.clone())
    }

    pub fn backups_dir(&self) -> PathBuf {
        self.config_dir().join("backups")
    }

    pub fn crash_dumps(&self) -> Vec<PathBuf> {
        let mut dumps: Vec<PathBuf> = std::fs::read_dir(&self.crash_dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| {
                        p.extension().is_some_and(|e| e.eq_ignore_ascii_case("dmp"))
                            && p.file_name()
                                .and_then(|n| n.to_str())
                                .is_some_and(|n| n.starts_with("crash-"))
                    })
                    .collect()
            })
            .unwrap_or_default();
        dumps.sort();
        dumps
    }

    pub fn to_json(&self) -> Value {
        json!({
            "instance": self.instance,
            "running": self.running,
            "source": self.source,
            "config": self.config,
            "configDir": self.config_dir(),
            "configExists": self.config.is_file(),
            "backupsDir": self.backups_dir(),
            "log": self.log,
            "logExists": self.log.is_file(),
            "logDir": self.log_dir,
            "crashDumps": self.crash_dumps(),
            "crashDir": self.crash_dir,
            "webviewDataDir": self.webview_data_dir,
            "recoveryMarker": self.recovery_marker,
            "distribution": self.distribution,
        })
    }
}

/// Shared discovery also rejects malformed deployment markers for offline commands.
pub fn distribution() -> Result<Distribution, IpcError> {
    let executable = std::env::current_exe().map_err(|e| IpcError::internal(e.to_string()))?;
    Distribution::detect(
        &executable,
        false,
        pecofence_platform::process::is_packaged(),
    )
    .map_err(|e| IpcError::new(ErrorCode::InvalidValue, e))
}

/// Read-only discovery; an offline CLI never creates data directories.
pub fn resolve(instance: Option<&str>, status: Option<StatusDto>) -> Result<Paths, IpcError> {
    let distribution = distribution()?;
    if let Some(status) = &status
        && let Some(paths) = &status.runtime_paths
    {
        return Ok(Paths {
            instance: instance.map(str::to_string),
            running: true,
            source: "status",
            config: status.config_path.clone().into(),
            log: paths.log_file.clone().into(),
            log_dir: Path::new(&paths.log_file)
                .parent()
                .unwrap_or(Path::new(""))
                .into(),
            crash_dir: paths.crash_dir.clone().into(),
            webview_data_dir: paths.webview_data_dir.clone().into(),
            recovery_marker: paths.recovery_marker.clone().into(),
            distribution: paths.distribution.clone(),
        });
    }
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let roaming = std::env::var_os("APPDATA").map(PathBuf::from);
    let paths = RuntimePaths::resolve(
        &distribution,
        roaming.as_deref(),
        local.as_deref(),
        instance,
    )
    .map_err(|e| IpcError::new(ErrorCode::InvalidValue, e))?;
    let mut result = planned_paths(instance, &paths, distribution.mode().as_str());
    if let Some(status) = status {
        if distribution.mode() == pecofence_core::distribution::DistributionMode::Portable {
            return Err(IpcError::new(
                ErrorCode::VersionMismatch,
                "The running app does not report portable runtime paths",
            )
            .hint("Update the app and CLI together"));
        }
        result.config = status.config_path.into();
        result.running = true;
        result.source = "status";
    }
    Ok(result)
}

fn planned_paths(instance: Option<&str>, paths: &RuntimePaths, mode: &str) -> Paths {
    Paths {
        instance: instance.map(str::to_string),
        running: false,
        source: "default",
        config: pecofence_core::ConfigStore::from_runtime_paths(paths).primary_path(),
        log: paths.log_file.clone(),
        log_dir: paths.log_file.parent().unwrap().into(),
        crash_dir: paths.crash_dir.clone(),
        webview_data_dir: paths.webview_data_dir.clone(),
        recovery_marker: paths.recovery_marker.clone(),
        distribution: mode.into(),
    }
}

// ---- config check --------------------------------------------------------------------------

/// `config check`: parse + validate like the app would, then cross-check. `Ok` carries the
/// report and whether it has errors (exit 1); a file the app would refuse is `Err`.
pub fn check_config(path: &Path) -> Result<(Value, bool), IpcError> {
    if !path.is_file() {
        return Err(IpcError::new(
            ErrorCode::InvalidValue,
            format!("{} is not a file", path.display()),
        )
        .hint("Pass a config.json or a `config export` file; `pecofence-cli paths` shows where the app keeps its own"));
    }
    let cfg = config_store::ConfigStore::parse_file(path).map_err(|e| {
        IpcError::new(
            ErrorCode::ValidationFailed,
            format!("{} would be rejected by PecoFence: {e}", path.display()),
        )
        .details(json!({ "expected": "a config.json as written by PecoFence (describe --schema Config)" }))
    })?;
    let problems = config_store::lint(&cfg);
    let errors = problems
        .iter()
        .filter(|p| p.level == LintLevel::Error)
        .count();
    let warnings = problems.len() - errors;
    let fences: usize = cfg.layouts.iter().map(|l| l.fences.len()).sum();
    let portals: usize = cfg
        .layouts
        .iter()
        .flat_map(|l| l.fences.iter())
        .filter(|f| f.kind == pecofence_core::FenceKind::FolderPortal)
        .count();
    let report = json!({
        "ok": errors == 0,
        "path": path,
        "schema": cfg.schema,
        "schemaVersion": cfg.schema_version,
        "summary": {
            "layouts": cfg.layouts.len(),
            "fences": fences,
            "portals": portals,
            "items": cfg.items.len(),
            "rules": cfg.rules.list.len(),
            "snapshots": cfg.snapshots.len(),
            "errors": errors,
            "warnings": warnings,
        },
        "problems": problems,
    });
    Ok((report, errors > 0))
}

// ---- log -------------------------------------------------------------------------------------

/// Last `lines` lines of `text` (all of it when `lines == 0`).
pub fn tail(text: &str, lines: usize) -> &str {
    if lines == 0 {
        return text;
    }
    let trimmed = text.strip_suffix('\n').unwrap_or(text);
    let mut start = trimmed.len();
    for _ in 0..lines {
        match trimmed[..start].rfind('\n') {
            Some(i) => start = i,
            None => return text,
        }
    }
    &text[start + 1..]
}

/// `log`: prints the tail of the log file and, with `follow`, new bytes as they arrive (a
/// truncated file, i.e. a restarted app, starts over from the top). Never returns with
/// `follow`; the user stops it with Ctrl+C.
pub fn print_log(path: &Path, lines: usize, follow: bool) -> Result<(), IpcError> {
    let mut file = std::fs::File::open(path).map_err(|e| {
        IpcError::new(
            ErrorCode::InvalidValue,
            format!("cannot open {}: {e}", path.display()),
        )
        .hint("PecoFence writes the log on start; `pecofence-cli paths` shows where it is expected")
    })?;
    let mut text = String::new();
    file.read_to_string(&mut text)
        .map_err(|e| IpcError::internal(format!("cannot read {}: {e}", path.display())))?;
    let mut out = std::io::stdout().lock();
    let _ = out.write_all(tail(&text, lines).as_bytes());
    let _ = out.flush();
    if !follow {
        return Ok(());
    }
    let mut offset = text.len() as u64;
    drop(text);
    loop {
        std::thread::sleep(Duration::from_millis(250));
        let Ok(meta) = std::fs::metadata(path) else {
            continue;
        };
        let len = meta.len();
        if len < offset {
            // Truncated: the app restarted and started a fresh log.
            offset = 0;
        }
        if len == offset {
            continue;
        }
        let Ok(mut f) = std::fs::File::open(path) else {
            continue;
        };
        if f.seek(SeekFrom::Start(offset)).is_err() {
            continue;
        }
        let mut chunk = Vec::new();
        if f.read_to_end(&mut chunk).is_err() {
            continue;
        }
        offset += chunk.len() as u64;
        if out.write_all(&chunk).and_then(|_| out.flush()).is_err() {
            // The reader went away (`| head`).
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_portable_paths_do_not_adopt_installed_config_or_scan_its_dumps() {
        let root =
            std::env::temp_dir().join(format!("pecofence-cli-paths-{}", uuid::Uuid::new_v4()));
        let distro = Distribution::resolve(&root.join("pecofence.exe"), None, true, false).unwrap();
        let runtime = RuntimePaths::resolve(&distro, None, None, Some("test")).unwrap();
        let paths = planned_paths(Some("test"), &runtime, "portable");
        assert!(
            !root.exists(),
            "offline discovery must not create directories"
        );
        assert_eq!(paths.config, root.join("config/config.json"));
        assert_eq!(paths.log, root.join("data/logs/pecofence.test.log"));
        std::fs::create_dir_all(&paths.crash_dir).unwrap();
        std::fs::create_dir_all(&paths.log_dir).unwrap();
        let dump = paths.crash_dir.join("crash-test-1.dmp");
        std::fs::write(&dump, b"dump").unwrap();
        std::fs::write(
            paths.log_dir.join("crash-test-2.dmp"),
            b"not in dump directory",
        )
        .unwrap();
        assert_eq!(paths.crash_dumps(), vec![dump]);
        assert_eq!(paths.to_json()["distribution"], "portable");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn tail_returns_the_last_lines() {
        let text = "a\nb\nc\nd\n";
        assert_eq!(tail(text, 2), "c\nd\n");
        assert_eq!(tail(text, 1), "d\n");
        assert_eq!(tail(text, 10), text);
        assert_eq!(tail(text, 0), text);
        assert_eq!(tail("single", 1), "single");
        assert_eq!(tail("", 3), "");
    }

    #[test]
    fn check_reports_lints_and_rejects_garbage() {
        let dir = std::env::temp_dir().join(format!("pecofence-cli-check-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let good = dir.join("good.json");
        let cfg = pecofence_core::Config::default();
        pecofence_core::ConfigStore::export_to(&cfg, &good).unwrap();
        let (report, failed) = check_config(&good).unwrap();
        assert!(!failed, "{report}");
        assert_eq!(report["ok"], json!(true));
        assert_eq!(
            report["schema"],
            json!(pecofence_core::CONFIG_SCHEMA_URL),
            "export writes $schema"
        );
        // No layouts yet: a warning, not an error.
        assert!(
            report["summary"]["warnings"].as_u64().unwrap() >= 1,
            "{report}"
        );

        let bad = dir.join("bad.json");
        std::fs::write(&bad, "{\"schemaVersion\": 99}").unwrap();
        let err = check_config(&bad).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationFailed);
        assert_eq!(
            check_config(&dir.join("missing.json")).unwrap_err().code,
            ErrorCode::InvalidValue
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
