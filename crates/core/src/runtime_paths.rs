//! Pure path planning shared by the app and CLI. No environment lookup, directory
//! creation or fallback I/O occurs here. Callers must report write failures at the
//! selected locations rather than retrying in AppData, Temp or the working directory.

use crate::brand;
use crate::distribution::{Distribution, DistributionMode};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePaths {
    pub config_dir: PathBuf,
    pub log_file: PathBuf,
    pub crash_dir: PathBuf,
    pub webview_data_dir: PathBuf,
    pub recovery_marker: PathBuf,
    /// Optional candidate for `ConfigStore::with_legacy`; never present in portable mode.
    pub legacy_config_dir: Option<PathBuf>,
    /// Only non-portable callers may adopt an outstanding pre-rename recovery marker.
    pub legacy_recovery_marker: Option<PathBuf>,
}

impl RuntimePaths {
    /// Plan locations from explicitly supplied profile directories. Portable copies
    /// do not require or consult either profile directory. Other modes require valid
    /// absolute directories and preserve the existing PecoFence/OpenFence layout.
    /// Named instances share config as before; logs, markers and WebView2 profiles differ.
    pub fn resolve(
        distribution: &Distribution,
        appdata: Option<&Path>,
        local_appdata: Option<&Path>,
        instance: Option<&str>,
    ) -> Result<Self, String> {
        let instance = instance.map(str::trim).filter(|name| !name.is_empty());
        if instance.is_some_and(|name| {
            name.chars().any(|c| {
                c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
            })
        }) {
            return Err(
                "Instance name contains characters that cannot be used in a file name".into(),
            );
        }
        let log_name = match instance {
            Some(name) => format!("pecofence.{name}.log"),
            None => "pecofence.log".into(),
        };
        let marker_name = match instance {
            Some(name) => format!("icons-hidden.{name}.marker"),
            None => "icons-hidden.marker".into(),
        };
        let profile = webview_profile(instance);
        if distribution.mode() == DistributionMode::Portable {
            let root = distribution.root();
            let data = root.join("data");
            return Ok(Self {
                config_dir: root.join("config"),
                log_file: data.join("logs").join(log_name),
                crash_dir: data.join("crashes"),
                webview_data_dir: data.join("WebView2Profiles").join(profile),
                recovery_marker: data.join("recovery").join(marker_name),
                legacy_config_dir: None,
                legacy_recovery_marker: None,
            });
        }

        let roaming = profile_directory(appdata, "APPDATA")?;
        let local = profile_directory(local_appdata, "LOCALAPPDATA")?;
        let data = local.join(brand::NAME);
        Ok(Self {
            config_dir: roaming.join(brand::NAME),
            log_file: data.join(log_name),
            crash_dir: data.clone(),
            webview_data_dir: data.join("WebView2Profiles").join(profile),
            recovery_marker: data.join(&marker_name),
            legacy_config_dir: Some(roaming.join(brand::LEGACY_DATA_DIR)),
            legacy_recovery_marker: Some(local.join(brand::LEGACY_DATA_DIR).join(marker_name)),
        })
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.json")
    }

    pub fn backups_dir(&self) -> PathBuf {
        self.config_dir.join("backups")
    }
}

fn profile_directory<'a>(directory: Option<&'a Path>, name: &str) -> Result<&'a Path, String> {
    directory
        .filter(|path| path.is_absolute() && !path.components().any(|c| c == Component::ParentDir))
        .ok_or_else(|| format!("{name} must be an absolute directory without '..'"))
}

fn webview_profile(instance: Option<&str>) -> String {
    let Some(instance) = instance else {
        return "default".into();
    };
    // Preserve settings_host's existing profile names when the shared resolver is adopted.
    let mut hash = DefaultHasher::new();
    instance.hash(&mut hash);
    format!("instance-{:016x}", hash.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PORTABLE: &str = r#"{"schema":1,"appId":"PecoFence","mode":"portable"}"#;
    const INSTALLED: &str = r#"{"schema":1,"appId":"PecoFence","mode":"installed"}"#;

    fn root() -> PathBuf {
        std::env::temp_dir().join(format!("pecofence-paths-{}", uuid::Uuid::new_v4()))
    }

    fn portable(root: &Path) -> Distribution {
        Distribution::resolve(&root.join("pecofence.exe"), Some(PORTABLE), false, false).unwrap()
    }

    fn managed_paths(paths: &RuntimePaths) -> Vec<PathBuf> {
        vec![
            paths.config_dir.clone(),
            paths.config_file(),
            paths.backups_dir(),
            paths.log_file.clone(),
            paths.crash_dir.clone(),
            paths.webview_data_dir.clone(),
            paths.recovery_marker.clone(),
        ]
    }

    #[test]
    fn portable_paths_are_local_without_creating_files_or_using_profile_directories() {
        let root = root();
        let distribution = portable(&root);
        let paths = RuntimePaths::resolve(&distribution, None, None, None).unwrap();
        assert_eq!(paths.config_file(), root.join("config/config.json"));
        assert_eq!(paths.backups_dir(), root.join("config/backups"));
        assert_eq!(paths.log_file, root.join("data/logs/pecofence.log"));
        assert_eq!(paths.crash_dir, root.join("data/crashes"));
        assert_eq!(
            paths.webview_data_dir,
            root.join("data/WebView2Profiles/default")
        );
        assert_eq!(
            paths.recovery_marker,
            root.join("data/recovery/icons-hidden.marker")
        );
        assert_eq!(paths.legacy_config_dir, None);
        assert_eq!(paths.legacy_recovery_marker, None);
        for path in managed_paths(&paths) {
            assert!(path.is_absolute() && path.starts_with(&root));
        }
        // Even invalid profile-directory inputs are irrelevant to a portable copy.
        assert_eq!(
            paths,
            RuntimePaths::resolve(
                &distribution,
                Some(Path::new("../old")),
                Some(Path::new("relative")),
                None
            )
            .unwrap()
        );
        assert!(!root.exists(), "path planning must not write to disk");
    }

    #[test]
    fn moving_a_portable_copy_rebases_every_managed_path() {
        let original = root().join("original");
        let moved = root().join("moved folder");
        let before =
            RuntimePaths::resolve(&portable(&original), None, None, Some("smoke")).unwrap();
        let after = RuntimePaths::resolve(&portable(&moved), None, None, Some("smoke")).unwrap();
        for (before, after) in managed_paths(&before).iter().zip(managed_paths(&after)) {
            assert_eq!(after, moved.join(before.strip_prefix(&original).unwrap()));
        }
    }

    #[test]
    fn non_portable_modes_preserve_current_and_legacy_locations() {
        let root = root();
        let roaming = root.join("Roaming");
        let local = root.join("Local");
        for (marker, packaged) in [(Some(INSTALLED), false), (None, false), (None, true)] {
            let distribution =
                Distribution::resolve(&root.join("app/pecofence.exe"), marker, false, packaged)
                    .unwrap();
            let paths =
                RuntimePaths::resolve(&distribution, Some(&roaming), Some(&local), None).unwrap();
            assert_eq!(paths.config_file(), roaming.join("PecoFence/config.json"));
            assert_eq!(paths.backups_dir(), roaming.join("PecoFence/backups"));
            assert_eq!(paths.log_file, local.join("PecoFence/pecofence.log"));
            assert_eq!(paths.crash_dir, local.join("PecoFence"));
            assert_eq!(
                paths.webview_data_dir,
                local.join("PecoFence/WebView2Profiles/default")
            );
            assert_eq!(
                paths.recovery_marker,
                local.join("PecoFence/icons-hidden.marker")
            );
            assert_eq!(paths.legacy_config_dir, Some(roaming.join("OpenFence")));
            assert_eq!(
                paths.legacy_recovery_marker,
                Some(local.join("OpenFence/icons-hidden.marker"))
            );
        }
    }

    #[test]
    fn legacy_portable_flag_uses_the_same_paths_as_a_portable_marker() {
        let root = root();
        let legacy = Distribution::resolve(&root.join("pecofence.exe"), None, true, false).unwrap();
        assert_eq!(
            RuntimePaths::resolve(&legacy, None, None, None),
            RuntimePaths::resolve(&portable(&root), None, None, None)
        );
    }

    #[test]
    fn named_instances_keep_their_own_logs_markers_and_webview_profiles() {
        let root = root();
        let distribution = portable(&root);
        let main = RuntimePaths::resolve(&distribution, None, None, None).unwrap();
        let named = RuntimePaths::resolve(&distribution, None, None, Some("smoke")).unwrap();
        assert_eq!(named.config_dir, main.config_dir);
        assert_eq!(named.log_file, root.join("data/logs/pecofence.smoke.log"));
        assert_eq!(
            named.recovery_marker,
            root.join("data/recovery/icons-hidden.smoke.marker")
        );
        assert_ne!(named.webview_data_dir, main.webview_data_dir);
        assert!(
            named
                .webview_data_dir
                .starts_with(root.join("data/WebView2Profiles"))
        );
        assert_eq!(
            named,
            RuntimePaths::resolve(&distribution, None, None, Some(" smoke ")).unwrap()
        );
        assert_eq!(
            main,
            RuntimePaths::resolve(&distribution, None, None, Some("  ")).unwrap()
        );
    }

    #[test]
    fn instance_names_cannot_escape_the_selected_directories() {
        let distribution = portable(&root());
        for name in [
            "../other",
            r"..\other",
            r"C:\other",
            "name:stream",
            "a/b",
            "a\nb",
            "a\0b",
            "a?b",
            "a*b",
            "a<b",
            "a>b",
            "a|b",
            "a\"b",
        ] {
            assert!(
                RuntimePaths::resolve(&distribution, None, None, Some(name)).is_err(),
                "{name:?}"
            );
        }
        assert!(RuntimePaths::resolve(&distribution, None, None, Some("測試 instance")).is_ok());
    }

    #[test]
    fn missing_or_relative_profile_directories_do_not_fall_back_to_temp_or_cwd() {
        let root = root();
        let distribution =
            Distribution::resolve(&root.join("pecofence.exe"), Some(INSTALLED), false, false)
                .unwrap();
        for bad in [
            None,
            Some(Path::new("relative")),
            Some(Path::new("")),
            Some(Path::new("../other")),
        ] {
            assert!(
                RuntimePaths::resolve(&distribution, bad, Some(&root), None)
                    .unwrap_err()
                    .contains("APPDATA")
            );
            assert!(
                RuntimePaths::resolve(&distribution, Some(&root), bad, None)
                    .unwrap_err()
                    .contains("LOCALAPPDATA")
            );
        }
    }
}
