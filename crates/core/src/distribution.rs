//! Distribution discovery shared by the app and CLI. Windows package identity and the
//! executable path are supplied by the caller; this module does not use Windows APIs.

use crate::brand;
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

pub const MARKER_FILE: &str = "deployment.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DistributionMode {
    Portable,
    Installed,
    Msix,
    /// An existing ZIP or source build with no marker and no `--portable` flag.
    Unmarked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Distribution {
    root: PathBuf,
    mode: DistributionMode,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum MarkerMode {
    Portable,
    Installed,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Marker {
    schema: u32,
    app_id: String,
    mode: MarkerMode,
}

impl Distribution {
    /// Pure resolution, suitable for tests and callers that already read the marker.
    ///
    /// A version-1 marker contains `schema`, `appId: "PecoFence"` and a `mode` of
    /// `portable` or `installed`. Invalid markers never fall back to another mode.
    /// `--portable` remains supported for unmarked copies and is redundant for a
    /// portable package. An installed marker or MSIX identity conflicts with that flag.
    pub fn resolve(
        executable: &Path,
        marker: Option<&str>,
        portable_flag: bool,
        is_packaged: bool,
    ) -> Result<Self, String> {
        let root = executable_root(executable)?;
        let marked_mode = marker.map(parse_marker).transpose()?;
        let mode = if is_packaged {
            if portable_flag || marked_mode == Some(DistributionMode::Portable) {
                return Err("An MSIX package cannot use portable mode".into());
            }
            DistributionMode::Msix
        } else {
            match (marked_mode, portable_flag) {
                (Some(DistributionMode::Installed), true) => {
                    return Err("--portable conflicts with the installed deployment marker".into());
                }
                (Some(mode), _) => mode,
                (None, true) => DistributionMode::Portable,
                (None, false) => DistributionMode::Unmarked,
            }
        };
        Ok(Self {
            root: root.to_path_buf(),
            mode,
        })
    }

    /// Read only the marker beside this executable, never in the working directory.
    /// Only a missing file means an unmarked copy; unreadable or malformed files fail.
    pub fn detect(
        executable: &Path,
        portable_flag: bool,
        is_packaged: bool,
    ) -> Result<Self, String> {
        let marker_path = executable_root(executable)?.join(MARKER_FILE);
        let marker = match fs::read_to_string(&marker_path) {
            Ok(text) => Some(text),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(format!("Cannot read {}: {error}", marker_path.display())),
        };
        Self::resolve(executable, marker.as_deref(), portable_flag, is_packaged)
            .map_err(|error| format!("{}: {error}", marker_path.display()))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn mode(&self) -> DistributionMode {
        self.mode
    }
}

fn executable_root(executable: &Path) -> Result<&Path, String> {
    if !executable.is_absolute()
        || executable.file_name().is_none()
        || executable.components().any(|c| c == Component::ParentDir)
    {
        return Err("Executable path must be absolute and must not contain '..'".into());
    }
    executable
        .parent()
        .ok_or_else(|| "Executable directory is missing".into())
}

fn parse_marker(text: &str) -> Result<DistributionMode, String> {
    // Accept UTF-8 BOMs written by Windows PowerShell, but otherwise parse strict JSON.
    let marker: Marker = serde_json::from_str(text.strip_prefix('\u{feff}').unwrap_or(text))
        .map_err(|error| format!("Invalid {MARKER_FILE}: {error}"))?;
    if marker.schema != 1 {
        return Err(format!("Unsupported deployment schema: {}", marker.schema));
    }
    if marker.app_id != brand::NAME {
        return Err(format!("Deployment marker belongs to {}", marker.app_id));
    }
    Ok(match marker.mode {
        MarkerMode::Portable => DistributionMode::Portable,
        MarkerMode::Installed => DistributionMode::Installed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PORTABLE: &str = r#"{"schema":1,"appId":"PecoFence","mode":"portable"}"#;
    const INSTALLED: &str = r#"{"schema":1,"appId":"PecoFence","mode":"installed"}"#;

    fn executable() -> PathBuf {
        std::env::temp_dir()
            .join("pecofence-distribution")
            .join("pecofence.exe")
    }

    #[test]
    fn markers_select_the_mode_without_a_flag_and_keep_unmarked_compatibility() {
        for (marker, flag, expected) in [
            (Some(PORTABLE), false, DistributionMode::Portable),
            (Some(PORTABLE), true, DistributionMode::Portable),
            (Some(INSTALLED), false, DistributionMode::Installed),
            (None, true, DistributionMode::Portable),
            (None, false, DistributionMode::Unmarked),
        ] {
            let found = Distribution::resolve(&executable(), marker, flag, false).unwrap();
            assert_eq!(found.mode(), expected);
            assert_eq!(found.root(), executable().parent().unwrap());
        }
        assert!(Distribution::resolve(&executable(), Some(INSTALLED), true, false).is_err());
    }

    #[test]
    fn package_identity_preserves_msix_and_rejects_portable_conflicts() {
        for marker in [None, Some(INSTALLED)] {
            assert_eq!(
                Distribution::resolve(&executable(), marker, false, true)
                    .unwrap()
                    .mode(),
                DistributionMode::Msix
            );
            assert!(Distribution::resolve(&executable(), marker, true, true).is_err());
        }
        assert!(Distribution::resolve(&executable(), Some(PORTABLE), false, true).is_err());
    }

    #[test]
    fn invalid_markers_never_silently_choose_a_mode() {
        for text in [
            "",
            "null",
            "{}",
            "not JSON",
            r#"{"schema":2,"appId":"PecoFence","mode":"portable"}"#,
            r#"{"schema":1,"appId":"OtherApp","mode":"portable"}"#,
            r#"{"schema":1,"appId":"PecoFence","mode":"msix"}"#,
            r#"{"schema":1,"appId":"PecoFence","mode":"Portable"}"#,
            r#"{"schema":1,"appId":"PecoFence","mode":"portable","mode":"installed"}"#,
            r#"{"schema":1,"appId":"PecoFence","mode":"portable","dataDir":"elsewhere"}"#,
            r#"{"schema":1,"appId":"PecoFence","mode":"portable"} trailing"#,
        ] {
            for (flag, packaged) in [(false, false), (true, false), (false, true)] {
                assert!(
                    Distribution::resolve(&executable(), Some(text), flag, packaged).is_err(),
                    "{text}"
                );
            }
        }
        assert_eq!(
            parse_marker(&format!("\u{feff}{PORTABLE}")).unwrap(),
            DistributionMode::Portable
        );
    }

    #[test]
    fn executable_paths_cannot_depend_on_the_working_directory() {
        for path in [
            PathBuf::new(),
            PathBuf::from("pecofence.exe"),
            executable().parent().unwrap().join("../pecofence.exe"),
        ] {
            assert!(Distribution::resolve(&path, Some(PORTABLE), false, false).is_err());
        }
    }

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let root = std::env::temp_dir()
                .join(format!("pecofence-distribution-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn discovery_uses_the_executable_directory_for_both_app_and_cli() {
        let folder = TestDirectory::new();
        let exe = folder.0.join("pecofence.exe");
        assert_eq!(
            Distribution::detect(&exe, false, false).unwrap().mode(),
            DistributionMode::Unmarked
        );
        fs::write(folder.0.join(MARKER_FILE), PORTABLE).unwrap();
        for file in ["pecofence.exe", "pecofence-cli.exe"] {
            let found = Distribution::detect(&folder.0.join(file), false, false).unwrap();
            assert_eq!(found.mode(), DistributionMode::Portable);
            assert_eq!(found.root(), folder.0);
        }
        let other = TestDirectory::new();
        assert_eq!(
            Distribution::detect(&other.0.join("pecofence.exe"), false, false)
                .unwrap()
                .mode(),
            DistributionMode::Unmarked
        );
    }

    #[test]
    fn unreadable_or_malformed_marker_is_not_treated_as_missing() {
        let folder = TestDirectory::new();
        let exe = folder.0.join("pecofence.exe");
        let marker = folder.0.join(MARKER_FILE);
        fs::create_dir(&marker).unwrap();
        assert!(
            Distribution::detect(&exe, true, false)
                .unwrap_err()
                .contains(MARKER_FILE)
        );
        fs::remove_dir(&marker).unwrap();
        for contents in [b"broken".as_slice(), &[0xff, 0xfe]] {
            fs::write(&marker, contents).unwrap();
            assert!(Distribution::detect(&exe, true, false).is_err());
        }
    }
}
