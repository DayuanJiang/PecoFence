//! Commands that work without the app (or mostly so): `paths`, `log`, `config check`. They
//! know where PecoFence keeps its files the same way the app does (`%LOCALAPPDATA%\PecoFence`
//! for the log and crash dumps, `%APPDATA%\PecoFence` for `config.json` unless a running
//! instance says otherwise, e.g. `--portable`).

use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use pecofence_core::brand;
use pecofence_core::config_store::{self, LintLevel};
use pecofence_ipc::{ErrorCode, IpcError};
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
        let mut dumps: Vec<PathBuf> = std::fs::read_dir(&self.log_dir)
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
        })
    }
}

/// Pure: the default locations from the two profile folders and the instance name (the app's
/// `log_file_path` / `ConfigStore::with_legacy` rules, without touching the disk for the config).
pub fn default_paths(
    local_appdata: Option<&Path>,
    appdata: Option<&Path>,
    instance: Option<&str>,
    legacy_config_exists: impl Fn(&Path) -> bool,
) -> (PathBuf, PathBuf, PathBuf) {
    let local = local_appdata
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let roaming = appdata
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let log_dir = local.join(brand::NAME);
    let log_name = match instance.map(str::trim).filter(|s| !s.is_empty()) {
        Some(n) => format!("pecofence.{n}.log"),
        None => "pecofence.log".to_string(),
    };
    let preferred = roaming.join(brand::NAME);
    let legacy = roaming.join(brand::LEGACY_DATA_DIR);
    let config_dir = if !legacy_config_exists(&preferred) && legacy_config_exists(&legacy) {
        legacy
    } else {
        preferred
    };
    (
        config_dir.join("config.json"),
        log_dir.join(log_name),
        log_dir,
    )
}

/// Resolves the paths, asking a running instance for its config path first (`status.get`).
pub fn resolve(instance: Option<&str>, status_config_path: Option<String>) -> Paths {
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let roaming = std::env::var_os("APPDATA").map(PathBuf::from);
    let has_data = |dir: &Path| {
        dir.join("config.json").is_file()
            || dir.join("config.bak").is_file()
            || dir.join("backups").is_dir()
    };
    let (default_config, log, log_dir) =
        default_paths(local.as_deref(), roaming.as_deref(), instance, has_data);
    match status_config_path {
        Some(p) => Paths {
            instance: instance.map(str::to_string),
            running: true,
            source: "status",
            config: PathBuf::from(p),
            log,
            log_dir,
        },
        None => Paths {
            instance: instance.map(str::to_string),
            running: false,
            source: "default",
            config: default_config,
            log,
            log_dir,
        },
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
    fn default_paths_follow_the_apps_rules() {
        let local = Path::new("C:\\Users\\me\\AppData\\Local");
        let roaming = Path::new("C:\\Users\\me\\AppData\\Roaming");
        let (config, log, log_dir) = default_paths(Some(local), Some(roaming), None, |_| false);
        assert_eq!(
            config,
            PathBuf::from("C:\\Users\\me\\AppData\\Roaming\\PecoFence\\config.json")
        );
        assert_eq!(
            log,
            PathBuf::from("C:\\Users\\me\\AppData\\Local\\PecoFence\\pecofence.log")
        );
        assert_eq!(
            log_dir,
            PathBuf::from("C:\\Users\\me\\AppData\\Local\\PecoFence")
        );

        // A named instance logs to its own file; the config folder is shared.
        let (_, log, _) = default_paths(Some(local), Some(roaming), Some(" test "), |_| false);
        assert!(log.ends_with("pecofence.test.log"), "{log:?}");

        // Legacy data folder is used only when the new one has nothing.
        let (config, _, _) = default_paths(Some(local), Some(roaming), None, |dir| {
            dir.ends_with("OpenFence")
        });
        assert!(config.ends_with("OpenFence\\config.json"), "{config:?}");
        let (config, _, _) = default_paths(Some(local), Some(roaming), None, |_| true);
        assert!(config.ends_with("PecoFence\\config.json"), "{config:?}");
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
