//! Product identity and narrowly scoped compatibility with pre-rename installs.

use std::env::VarError;
use std::ffi::OsString;

pub const NAME: &str = "PecoFence";
/// Microsoft Store product id (Partner Center, packaging/msix/identity.json).
pub const STORE_PRODUCT_ID: &str = "9MV6WG3XNWSX";
pub const REPOSITORY_URL: &str = "https://github.com/DayuanJiang/PecoFence";
/// The CLI guide, at its part for AI agents (Settings › 整理规则 › 命令行文档).
pub const CLI_DOCS_URL: &str =
    "https://github.com/DayuanJiang/PecoFence/blob/main/docs/CLI.md#start-with-your-ai-agent";
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
    let suffix = instance_suffix(instance);
    [
        format!(r"Local\PecoFence.SingleInstance{suffix}"),
        format!(r"Local\openFence.SingleInstance{suffix}"),
    ]
}

/// Named pipe the CLI (`pecofence-cli`) talks to: `PecoFence.<user SID>[.<instance>]`. Pipe
/// names are machine-wide, so the SID keeps two signed-in accounts apart, while the same user
/// in another session (ssh) still reaches their own instance.
pub fn ipc_pipe_name(user_sid: &str, instance: Option<&str>) -> String {
    format!(
        r"\\.\pipe\PecoFence.{user_sid}{}",
        instance_suffix(instance)
    )
}

/// [`log_file_name`] suffixes: this run's log, the previous run's, and the shareable log (only
/// what may be sent with feedback) of this and the previous run.
pub const LOG: &str = "";
pub const PREVIOUS_LOG: &str = ".prev";
pub const SHARE_LOG: &str = ".share";
pub const PREVIOUS_SHARE_LOG: &str = ".share.prev";

/// Log file in `%LOCALAPPDATA%\PecoFence`: `pecofence[.name]<suffix>.log`, one set per instance.
pub fn log_file_name(instance: Option<&str>, suffix: &str) -> String {
    format!("pecofence{}{suffix}.log", instance_suffix(instance))
}

/// `.name` for a named (test) instance, empty for the main one; whitespace-only = main.
fn instance_suffix(instance: Option<&str>) -> String {
    instance
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!(".{value}"))
        .unwrap_or_default()
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

    #[test]
    fn pipe_name_carries_the_user_sid_and_the_instance_suffix() {
        let sid = "S-1-5-21-1-2-3-1001";
        assert_eq!(
            ipc_pipe_name(sid, None),
            r"\\.\pipe\PecoFence.S-1-5-21-1-2-3-1001"
        );
        assert_eq!(ipc_pipe_name(sid, Some(" ")), ipc_pipe_name(sid, None));
        assert_eq!(
            ipc_pipe_name(sid, Some(" test ")),
            r"\\.\pipe\PecoFence.S-1-5-21-1-2-3-1001.test"
        );
        assert_ne!(
            ipc_pipe_name(sid, None),
            ipc_pipe_name("S-1-5-21-1-2-3-1002", None)
        );
    }

    #[test]
    fn log_names_follow_the_instance_suffix() {
        assert_eq!(log_file_name(None, LOG), "pecofence.log");
        assert_eq!(log_file_name(Some(" "), PREVIOUS_LOG), "pecofence.prev.log");
        assert_eq!(
            log_file_name(Some("test"), SHARE_LOG),
            "pecofence.test.share.log"
        );
        assert_eq!(
            log_file_name(Some("test"), PREVIOUS_SHARE_LOG),
            "pecofence.test.share.prev.log"
        );
    }
}
