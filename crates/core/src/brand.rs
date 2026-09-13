//! Product identity and narrowly scoped compatibility with pre-rename installs.

use std::env::VarError;
use std::ffi::OsString;

pub const NAME: &str = "PecoFence";
pub const LEGACY_DATA_DIR: &str = "OpenFence";
pub const LEGACY_AUTOSTART: &str = "openFence";

fn lookup_env(name: &str, lookup: impl Fn(&str) -> Option<OsString>) -> Option<OsString> {
    lookup(name).or_else(|| {
        name.strip_prefix("PECOFENCE_")
            .and_then(|suffix| lookup(&format!("OPENFENCE_{suffix}")))
    })
}

/// Canonical variables take priority; old environment overrides keep working.
pub fn var_os(name: &str) -> Option<OsString> {
    lookup_env(name, |key| std::env::var_os(key))
}

pub fn var(name: &str) -> Result<String, VarError> {
    var_os(name)
        .ok_or(VarError::NotPresent)?
        .into_string()
        .map_err(VarError::NotUnicode)
}

/// Hold both names so a renamed build and an older executable cannot concurrently
/// manage the same desktop. Named test instances remain independent of the main app.
pub fn instance_mutex_names(instance: Option<&str>) -> [String; 2] {
    let suffix = instance
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!(".{value}"))
        .unwrap_or_default();
    [
        format!(r"Local\PecoFence.SingleInstance{suffix}"),
        format!(r"Local\openFence.SingleInstance{suffix}"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_environment_overrides_legacy_without_mutating_process_environment() {
        let both = |name: &str| match name {
            "PECOFENCE_INSTANCE" => Some(OsString::from("new")),
            "OPENFENCE_INSTANCE" => Some(OsString::from("old")),
            _ => None,
        };
        assert_eq!(lookup_env("PECOFENCE_INSTANCE", both), Some("new".into()));
        assert_eq!(
            lookup_env("PECOFENCE_INSTANCE", |name| {
                (name == "OPENFENCE_INSTANCE").then(|| "legacy".into())
            }),
            Some("legacy".into())
        );
        assert_eq!(
            lookup_env("PECOFENCE_INSTANCE", |name| {
                (name == "PECOFENCE_INSTANCE").then(OsString::new)
            }),
            Some(OsString::new())
        );
        assert_eq!(lookup_env("UNRELATED", both), None);
    }

    #[test]
    fn legacy_and_current_instance_names_use_the_same_normalized_suffix() {
        let main = instance_mutex_names(None);
        assert_eq!(main[1], r"Local\openFence.SingleInstance");
        assert_eq!(main, instance_mutex_names(Some("  ")));
        let named = instance_mutex_names(Some(" test "));
        assert_eq!(named[0], r"Local\PecoFence.SingleInstance.test");
        assert_eq!(named[1], r"Local\openFence.SingleInstance.test");
        assert_ne!(main, named);
    }
}
