//! Resolve the distribution once, before logging or touching desktop state.

use pecofence_core::distribution::{Distribution, DistributionMode};
use pecofence_core::runtime_paths::RuntimePaths;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Runtime {
    pub distribution: Distribution,
    pub paths: RuntimePaths,
}

impl Runtime {
    pub fn detect(portable: bool, instance: Option<&str>) -> Result<Self, String> {
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        let distribution = Distribution::detect(
            &executable,
            portable,
            pecofence_platform::process::is_packaged(),
        )?;
        let roaming = std::env::var_os("APPDATA").map(PathBuf::from);
        let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
        let mut paths = RuntimePaths::resolve(
            &distribution,
            roaming.as_deref(),
            local.as_deref(),
            instance,
        )?;
        // Select existing pre-rename data before probing or creating directories.
        paths.config_dir = pecofence_core::ConfigStore::from_runtime_paths(&paths)
            .dir()
            .to_path_buf();
        Ok(Self {
            distribution,
            paths,
        })
    }

    pub fn portable(&self) -> bool {
        self.distribution.mode() == DistributionMode::Portable
    }

    /// Fail before loading config or hiding icons. Never retry outside these paths.
    pub fn prepare(&self) -> Result<File, String> {
        for directory in [
            self.paths.config_dir.as_path(),
            &self.paths.backups_dir(),
            self.paths.log_file.parent().unwrap(),
            &self.paths.crash_dir,
            &self.paths.webview_data_dir,
            self.paths.recovery_marker.parent().unwrap(),
        ] {
            writable_directory(directory)?;
        }
        // A writable parent does not imply that existing files can be updated.
        for name in ["config.json", "config.bak", "config.json.tmp"] {
            writable_existing_file(&self.paths.config_dir.join(name))?;
        }
        writable_existing_file(&self.paths.recovery_marker)?;
        File::create(&self.paths.log_file).map_err(|error| path_error(&self.paths.log_file, error))
    }
}

fn path_error(path: &Path, error: std::io::Error) -> String {
    format!("{}: {error}", path.display())
}

fn writable_existing_file(path: &Path) -> Result<(), String> {
    match OpenOptions::new().write(true).open(path) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(path_error(path, error)),
    }
}

pub fn writable_directory(directory: &Path) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| path_error(directory, error))?;
    let probe = directory.join(format!(".pecofence-write-{}", uuid::Uuid::new_v4()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .map_err(|error| path_error(directory, error))?;
    let result = file.write_all(b"probe").and_then(|_| file.sync_all());
    drop(file);
    let cleanup = fs::remove_file(&probe);
    result
        .and(cleanup)
        .map_err(|error| path_error(directory, error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_preparation_stays_local_and_reports_blocked_paths() {
        let root = std::env::temp_dir().join(format!("pecofence-runtime-{}", uuid::Uuid::new_v4()));
        let distribution =
            Distribution::resolve(&root.join("pecofence.exe"), None, true, false).unwrap();
        let paths = RuntimePaths::resolve(&distribution, None, None, None).unwrap();
        let runtime = Runtime {
            distribution,
            paths,
        };
        drop(runtime.prepare().unwrap());
        assert!(runtime.paths.log_file.is_file());
        assert!(runtime.paths.backups_dir().is_dir());
        assert!(runtime.paths.webview_data_dir.is_dir());
        assert!(
            !runtime.paths.config_file().exists(),
            "preflight must not overwrite config"
        );
        fs::remove_dir(&runtime.paths.webview_data_dir).unwrap();
        fs::write(&runtime.paths.webview_data_dir, b"blocked").unwrap();
        let error = runtime.prepare().unwrap_err();
        assert!(error.contains(&runtime.paths.webview_data_dir.display().to_string()));
        fs::remove_file(&runtime.paths.webview_data_dir).unwrap();
        fs::write(runtime.paths.config_file(), b"preserve").unwrap();
        let mut permissions = fs::metadata(runtime.paths.config_file())
            .unwrap()
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(runtime.paths.config_file(), permissions.clone()).unwrap();
        assert!(runtime.prepare().unwrap_err().contains("config.json"));
        assert_eq!(fs::read(runtime.paths.config_file()).unwrap(), b"preserve");
        // Windows test: restore the readonly bit before removing this owned fixture.
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        fs::set_permissions(runtime.paths.config_file(), permissions).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
