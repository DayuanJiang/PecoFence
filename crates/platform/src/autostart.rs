//! Run-at-logon via Task Scheduler, with the Run key as a compatibility fallback.

use crate::bindings::*;
use crate::wide::{from_wide, to_wide};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use windows_core::{Error, PCWSTR, PWSTR, Result};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RRF_RT_REG_SZ: u32 = 0x0000_0002;
const TASK_NAME: &str = "PecoFence";
const LEGACY_TASK_NAME: &str = "openFence";

static STARTUP_RECONCILER: StartupReconciler = StartupReconciler::new();

/// Serialize registration changes while allowing a newer settings change to supersede a
/// startup worker that has not run yet. Submitting work never waits for Task Scheduler.
struct StartupReconciler {
    generation: AtomicU64,
    apply_lock: Mutex<()>,
}

impl StartupReconciler {
    const fn new() -> Self {
        Self {
            generation: AtomicU64::new(0),
            apply_lock: Mutex::new(()),
        }
    }

    fn submit(&self) -> u64 {
        self.generation
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1)
    }

    fn apply(&self, generation: u64, work: impl FnOnce() -> Result<()>) -> Result<()> {
        let _guard = self.apply_lock.lock().unwrap_or_else(|poisoned| {
            tracing::warn!("recovering poisoned startup reconciliation lock");
            poisoned.into_inner()
        });
        if generation != self.generation.load(Ordering::SeqCst) {
            tracing::debug!(generation, "skipping superseded startup reconciliation");
            return Ok(());
        }
        // A newer request may arrive during this call. It will hold a newer generation and
        // acquire this same lock after us, so the newest requested state is always applied last.
        work()
    }
}

/// The startup behavior that belongs to the process currently changing startup settings.
///
/// `portable` is part of the registration rather than inferred from the executable path:
/// portable mode must come back with `--portable`, otherwise the next logon would read the
/// normal `%APPDATA%` configuration instead of the directory beside the executable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StartupSpec {
    pub enabled: bool,
    pub portable: bool,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct StartupActions {
    remove_current_task: bool,
    remove_legacy_task: bool,
    register_run_fallback: bool,
    remove_current_run: bool,
    remove_legacy_run: bool,
}

/// Decide the cleanup/fallback work after the preferred task registration attempt.
/// Keeping this decision pure makes the compatibility behavior testable without modifying the
/// current user's task database or Run key.
fn startup_actions(spec: StartupSpec, development: bool, task_registered: bool) -> StartupActions {
    if development {
        return StartupActions::default();
    }
    if !spec.enabled {
        return StartupActions {
            remove_current_task: true,
            remove_legacy_task: true,
            remove_current_run: true,
            remove_legacy_run: true,
            ..Default::default()
        };
    }
    if task_registered {
        StartupActions {
            remove_legacy_task: true,
            remove_current_run: true,
            remove_legacy_run: true,
            ..Default::default()
        }
    } else {
        StartupActions {
            remove_current_task: true,
            remove_legacy_task: true,
            register_run_fallback: true,
            remove_legacy_run: true,
            ..Default::default()
        }
    }
}

/// Do not retire a previous working registration until its replacement exists. In
/// particular, a failed task registration followed by a denied Run write must leave
/// both the old tasks and legacy Run value intact.
fn prepare_startup_cleanup(
    spec: StartupSpec,
    task_registered: bool,
    register_fallback: impl FnOnce() -> Result<()>,
) -> Result<StartupActions> {
    let actions = startup_actions(spec, false, task_registered);
    if actions.register_run_fallback {
        register_fallback()?;
    }
    Ok(actions)
}

/// `same_executable` is retained for compatibility and ownership tests around legacy Run values.
#[allow(dead_code)]
fn same_executable(a: &str, b: &str) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => a.eq_ignore_ascii_case(b),
    }
}

/// Reconcile startup after a normal release has launched. Debug builds and packaged Store
/// builds leave their existing startup mechanisms alone.
pub fn reconcile_product(spec: StartupSpec, development: bool) -> Result<()> {
    if development || cfg!(debug_assertions) || crate::process::is_packaged() {
        return Ok(());
    }
    let generation = STARTUP_RECONCILER.submit();
    STARTUP_RECONCILER.apply(generation, || reconcile_enabled(spec))
}

/// Queue startup reconciliation without blocking the first visible frame on Task Scheduler
/// subprocesses. An explicit settings change supersedes an older queued startup request.
pub fn reconcile_product_in_background(spec: StartupSpec, development: bool) -> Result<()> {
    if development || cfg!(debug_assertions) || crate::process::is_packaged() {
        return Ok(());
    }
    let generation = STARTUP_RECONCILER.submit();
    std::thread::Builder::new()
        .name("startup-reconcile".into())
        .spawn(move || {
            if let Err(error) = STARTUP_RECONCILER.apply(generation, || reconcile_enabled(spec)) {
                tracing::warn!(%error, "background startup reconciliation failed");
            }
        })
        .map_err(io_error)?;
    Ok(())
}

/// Apply an explicit settings change. Store packages use the manifest startup task, while
/// desktop releases use Task Scheduler and fall back to the Run key if registration fails.
pub fn set_product_enabled(spec: StartupSpec) -> Result<()> {
    if cfg!(debug_assertions) || crate::process::is_packaged() {
        return Ok(());
    }
    let generation = STARTUP_RECONCILER.submit();
    STARTUP_RECONCILER.apply(generation, || reconcile_enabled(spec))
}

fn reconcile_enabled(spec: StartupSpec) -> Result<()> {
    let task_registered = if spec.enabled {
        match register_task(spec) {
            Ok(()) => {
                tracing::info!(
                    portable = spec.portable,
                    task = TASK_NAME,
                    "startup task registered"
                );
                true
            }
            Err(error) => {
                tracing::warn!(%error, portable = spec.portable, task = TASK_NAME, "startup task registration failed; falling back to Run key");
                false
            }
        }
    } else {
        false
    };
    let actions = prepare_startup_cleanup(spec, task_registered, || {
        match set_enabled_for_spec(pecofence_core::brand::NAME, true, spec.portable) {
            Ok(()) => {
                tracing::info!(
                    portable = spec.portable,
                    "startup Run key fallback registered"
                );
                Ok(())
            }
            Err(error) => {
                tracing::warn!(%error, "startup Run key fallback could not be registered; preserving existing registrations");
                Err(error)
            }
        }
    })?;

    let mut first_error = None;
    if actions.remove_current_task {
        if let Err(error) = delete_task(TASK_NAME) {
            tracing::warn!(%error, task = TASK_NAME, "startup task cleanup failed");
            first_error.get_or_insert(error);
        }
    }
    if actions.remove_legacy_task {
        if let Err(error) = delete_task(LEGACY_TASK_NAME) {
            tracing::warn!(%error, task = LEGACY_TASK_NAME, "legacy startup task cleanup failed");
            first_error.get_or_insert(error);
        }
    }
    if actions.remove_current_run {
        if let Err(error) = set_enabled_for_spec(pecofence_core::brand::NAME, false, false) {
            tracing::warn!(%error, "startup Run key cleanup failed");
            first_error.get_or_insert(error);
        }
    }
    if actions.remove_legacy_run {
        if let Err(error) =
            set_enabled_for_spec(pecofence_core::brand::LEGACY_AUTOSTART, false, false)
        {
            tracing::warn!(%error, "legacy startup Run key cleanup failed");
            first_error.get_or_insert(error);
        }
    }
    first_error.map_or(Ok(()), Err)
}

fn task_xml(command: &str, arguments: Option<&str>, user_id: &str) -> String {
    let escaped_command = xml_escape(command);
    let escaped_user = xml_escape(user_id);
    let arguments = arguments
        .filter(|value| !value.is_empty())
        .map(|value| format!("<Arguments>{}</Arguments>", xml_escape(value)))
        .unwrap_or_else(|| "<Arguments></Arguments>".to_string());
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Author>{escaped_user}</Author>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
      <UserId>{escaped_user}</UserId>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{escaped_user}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <StartWhenAvailable>false</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>3</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{escaped_command}</Command>
      {arguments}
    </Exec>
  </Actions>
</Task>
"#
    )
}

fn xml_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn encode_utf16le(value: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(2 + value.len() * 2);
    bytes.extend_from_slice(&[0xff, 0xfe]);
    for character in value.encode_utf16() {
        bytes.extend_from_slice(&character.to_le_bytes());
    }
    bytes
}

fn register_task(spec: StartupSpec) -> Result<()> {
    let command = exe_path();
    let arguments = spec.portable.then_some("--portable");
    let xml = task_xml(&command, arguments, &current_user_id());
    register_task_xml(TASK_NAME, &xml)
}

fn register_task_xml(name: &str, xml: &str) -> Result<()> {
    let path = task_xml_path();
    std::fs::write(&path, encode_utf16le(xml)).map_err(io_error)?;
    let path_arg = path.to_string_lossy().into_owned();
    let result = run_schtasks(&[
        "/Create".to_string(),
        "/TN".to_string(),
        name.to_string(),
        "/XML".to_string(),
        path_arg,
        "/F".to_string(),
    ])
    .map(|_| ());
    let _ = std::fs::remove_file(path);
    result
}

fn task_xml_path() -> PathBuf {
    let ticks = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    std::env::temp_dir().join(format!(
        "PecoFence-startup-{}-{ticks}.xml",
        std::process::id()
    ))
}

fn current_user_id() -> String {
    const NAME_SAM_COMPATIBLE: u32 = 2;
    windows_core::link!("secur32.dll" "system" fn GetUserNameExW(
        name_format: u32,
        name_buffer: PWSTR,
        size: *mut u32,
    ) -> windows_core::BOOL);
    let mut buffer = [0u16; 512];
    let mut size = buffer.len() as u32;
    // SAFETY: the buffer and length are valid for the duration of the call.
    if unsafe { GetUserNameExW(NAME_SAM_COMPATIBLE, PWSTR(buffer.as_mut_ptr()), &mut size) }
        .as_bool()
        && size > 0
    {
        return String::from_utf16_lossy(&buffer[..size as usize]);
    }
    let output = Command::new("whoami.exe")
        .creation_flags(CREATE_NO_WINDOW as u32)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok();
    if let Some(value) = output
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return value;
    }
    let name = std::env::var("USERNAME").unwrap_or_default();
    let domain = std::env::var("USERDOMAIN").unwrap_or_default();
    if !domain.is_empty() && !name.is_empty() {
        format!("{domain}\\{name}")
    } else if !name.is_empty() {
        name
    } else {
        String::new()
    }
}

fn schtasks_path() -> PathBuf {
    std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .map(|root| root.join("System32").join("schtasks.exe"))
        .filter(|path| path.exists())
        .unwrap_or_else(|| PathBuf::from("schtasks.exe"))
}

fn run_schtasks(arguments: &[String]) -> Result<Output> {
    let output = Command::new(schtasks_path())
        .args(arguments)
        .creation_flags(CREATE_NO_WINDOW as u32)
        .stdin(Stdio::null())
        .output()
        .map_err(io_error)?;
    if output.status.success() {
        return Ok(output);
    }
    tracing::debug!(
        status = ?output.status.code(),
        stdout = %String::from_utf8_lossy(&output.stdout),
        stderr = %String::from_utf8_lossy(&output.stderr),
        "schtasks.exe failed"
    );
    Err(win32_error(output.status.code().unwrap_or(1) as u32))
}

fn delete_task(name: &str) -> Result<()> {
    let query = run_schtasks(&["/Query".into(), "/TN".into(), name.into()]);
    if query.is_err() {
        // A missing task is the normal state for first launch and for disabling an already
        // disabled setting. Query failures are treated as absent; /Delete would otherwise turn
        // harmless cleanup into a user-visible startup error on localized Windows versions.
        return Ok(());
    }
    run_schtasks(&["/Delete".into(), "/TN".into(), name.into(), "/F".into()]).map(|_| ())
}

/// Full path of the running executable.
pub fn exe_path() -> String {
    let mut buf = [0u16; 1024];
    // SAFETY: buffer length passed.
    let n = unsafe { GetModuleFileNameW(None, PWSTR(buf.as_mut_ptr()), buf.len() as u32) };
    from_wide(&buf[..n as usize])
}

fn open_run_key() -> Result<HKEY> {
    let key = to_wide(RUN_KEY);
    let mut hkey = HKEY::default();
    // SAFETY: out-pointer to a local; the key is closed by the caller.
    let status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key.as_ptr()),
            None,
            PCWSTR::null(),
            0,
            ACCESS_MASK((KEY_SET_VALUE | KEY_QUERY_VALUE) as u32),
            None,
            &mut hkey,
            None,
        )
    };
    if status.0 != 0 {
        return Err(win32_error(status.0 as u32));
    }
    Ok(hkey)
}

/// Set a Run value for compatibility callers. New startup registrations use `StartupSpec`.
pub fn set_enabled(value_name: &str, enabled: bool) -> Result<()> {
    if cfg!(debug_assertions) {
        return Ok(());
    }
    set_enabled_for_spec(value_name, enabled, false)
}

fn set_enabled_for_spec(value_name: &str, enabled: bool, portable: bool) -> Result<()> {
    let hkey = open_run_key()?;
    let name = to_wide(value_name);
    // SAFETY: buffers outlive the calls; key closed afterwards.
    let status = unsafe {
        let s = if enabled {
            let cmd = to_wide(&run_command(&exe_path(), portable));
            let bytes: Vec<u8> = cmd.iter().flat_map(|c| c.to_le_bytes()).collect();
            RegSetValueExW(hkey, PCWSTR(name.as_ptr()), None, REG_SZ, Some(&bytes))
        } else {
            let s = RegDeleteValueW(hkey, PCWSTR(name.as_ptr()));
            // ERROR_FILE_NOT_FOUND is fine when already disabled.
            if s.0 == 2 { LSTATUS(0) } else { s }
        };
        let _ = RegCloseKey(hkey);
        s
    };
    if status.0 != 0 {
        return Err(win32_error(status.0 as u32));
    }
    Ok(())
}

fn run_command(exe: &str, portable: bool) -> String {
    if portable {
        format!("\"{exe}\" --portable")
    } else {
        format!("\"{exe}\"")
    }
}

/// The executable path currently registered for `value_name` (quotes and arguments stripped),
/// if any.
pub fn registered_path(value_name: &str) -> Option<String> {
    let key = to_wide(RUN_KEY);
    let name = to_wide(value_name);
    let mut buf = [0u16; 1024];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: out-pointers reference locals with the declared sizes.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(key.as_ptr()),
            PCWSTR(name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    if status.0 != 0 {
        return None;
    }
    let chars = (size as usize / 2).min(buf.len());
    let raw = from_wide(&buf[..chars]);
    let raw = raw.trim_end_matches('\0').trim();
    Some(command_executable(raw))
}

fn command_executable(command: &str) -> String {
    let command = command.trim();
    if let Some(quoted) = command.strip_prefix('"') {
        return quoted
            .find('"')
            .map(|end| quoted[..end].to_string())
            .unwrap_or_else(|| quoted.to_string());
    }
    command
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string()
}

pub fn is_enabled(value_name: &str) -> bool {
    let key = to_wide(RUN_KEY);
    let name = to_wide(value_name);
    let mut buf = [0u16; 1024];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: out-pointers reference locals with the declared sizes.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(key.as_ptr()),
            PCWSTR(name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    status.0 == 0 && !from_wide(&buf).trim().is_empty()
}

fn io_error(error: std::io::Error) -> Error {
    win32_error(error.raw_os_error().unwrap_or(1) as u32)
}

fn win32_error(code: u32) -> Error {
    // HRESULT_FROM_WIN32
    let hr = if code == 0 {
        0
    } else {
        (code & 0xffff) | 0x8007_0000
    };
    Error::from_hresult(windows_core::HRESULT(hr as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queued_startup_enable_cannot_override_a_later_disable() {
        let reconciler = StartupReconciler::new();
        let startup = reconciler.submit();
        let settings = reconciler.submit();
        let mut enabled = true;
        reconciler
            .apply(settings, || {
                enabled = false;
                Ok(())
            })
            .unwrap();
        reconciler
            .apply(startup, || {
                enabled = true;
                Ok(())
            })
            .unwrap();
        assert!(!enabled);
    }

    #[test]
    fn in_flight_startup_enable_finishes_before_a_later_disable() {
        let reconciler = StartupReconciler::new();
        let applied = Mutex::new(Vec::new());
        let startup = reconciler.submit();
        std::thread::scope(|scope| {
            let (started_tx, started_rx) = std::sync::mpsc::channel();
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            let reconciler = &reconciler;
            let applied = &applied;
            let worker = scope.spawn(move || {
                reconciler.apply(startup, || {
                    started_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                    applied.lock().unwrap().push(true);
                    Ok(())
                })
            });
            started_rx.recv().unwrap();
            let settings = reconciler.submit();
            let change = scope.spawn(move || {
                reconciler.apply(settings, || {
                    applied.lock().unwrap().push(false);
                    Ok(())
                })
            });
            release_tx.send(()).unwrap();
            worker.join().unwrap().unwrap();
            change.join().unwrap().unwrap();
        });
        assert_eq!(*applied.lock().unwrap(), [true, false]);
    }

    #[test]
    fn task_xml_contains_the_requested_scheduler_policy() {
        let xml = task_xml(
            r"C:\Program Files\Peco&Fence\pecofence.exe",
            None,
            r"DESKTOP\user",
        );
        assert!(xml.contains("<Priority>3</Priority>"));
        assert!(xml.contains("<LogonTrigger>"));
        assert!(!xml.contains("<Delay>"));
        assert!(xml.contains("<MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>"));
        assert!(xml.contains("<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>"));
        assert!(xml.contains("<StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>"));
        assert!(xml.contains("<LogonType>InteractiveToken</LogonType>"));
        assert!(xml.contains("<RunLevel>LeastPrivilege</RunLevel>"));
        assert!(xml.contains("<StartWhenAvailable>false</StartWhenAvailable>"));
        assert!(xml.contains("<RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>"));
        assert!(xml.contains("<RunOnlyIfIdle>false</RunOnlyIfIdle>"));
        assert!(xml.contains("<ExecutionTimeLimit>PT0S</ExecutionTimeLimit>"));
        assert!(xml.contains("C:\\Program Files\\Peco&amp;Fence\\pecofence.exe"));
        assert!(xml.contains("<Arguments></Arguments>"));
        assert!(!xml.contains("--portable"));
    }

    #[test]
    fn portable_task_has_an_explicit_argument() {
        let xml = task_xml(r"C:\PecoFence\pecofence.exe", Some("--portable"), r"user<&");
        assert!(xml.contains("<Arguments>--portable</Arguments>"));
        assert!(xml.contains("<UserId>user&lt;&amp;</UserId>"));
    }

    #[test]
    fn xml_escape_handles_all_xml_text_delimiters() {
        assert_eq!(xml_escape("<&>\"'"), "&lt;&amp;&gt;&quot;&apos;");
    }

    #[test]
    fn run_key_fallback_keeps_the_portable_mode_argument() {
        assert_eq!(
            run_command(r"C:\Peco Fence\pecofence.exe", false),
            r#""C:\Peco Fence\pecofence.exe""#
        );
        assert_eq!(
            run_command(r"C:\Peco Fence\pecofence.exe", true),
            r#""C:\Peco Fence\pecofence.exe" --portable"#
        );
    }

    #[test]
    fn utf16_encoding_has_bom_and_round_trips() {
        let text = "PecoFence <startup> 桌面 🖥";
        let bytes = encode_utf16le(text);
        assert_eq!(&bytes[..2], &[0xff, 0xfe]);
        let mut code_units = Vec::new();
        for index in (2..bytes.len()).step_by(2) {
            code_units.push(u16::from_le_bytes([bytes[index], bytes[index + 1]]));
        }
        let decoded = String::from_utf16(&code_units).unwrap();
        assert_eq!(decoded, text);
    }

    #[test]
    fn startup_actions_cover_enable_disable_and_fallback_cleanup() {
        assert_eq!(
            startup_actions(
                StartupSpec {
                    enabled: true,
                    portable: false
                },
                false,
                true,
            ),
            StartupActions {
                remove_legacy_task: true,
                remove_current_run: true,
                remove_legacy_run: true,
                ..Default::default()
            }
        );
        let fallback = startup_actions(
            StartupSpec {
                enabled: true,
                portable: true,
            },
            false,
            false,
        );
        assert!(fallback.register_run_fallback);
        assert!(fallback.remove_current_task && fallback.remove_legacy_task);
        assert!(fallback.remove_legacy_run);
        assert_eq!(
            startup_actions(
                StartupSpec {
                    enabled: false,
                    portable: false
                },
                false,
                false,
            ),
            StartupActions {
                remove_current_task: true,
                remove_legacy_task: true,
                remove_current_run: true,
                remove_legacy_run: true,
                ..Default::default()
            }
        );
        assert_eq!(
            startup_actions(
                StartupSpec {
                    enabled: true,
                    portable: false
                },
                true,
                false,
            ),
            StartupActions::default()
        );
    }

    #[test]
    fn failed_fallback_does_not_authorize_any_cleanup() {
        let spec = StartupSpec {
            enabled: true,
            portable: true,
        };
        let error = prepare_startup_cleanup(spec, false, || Err(win32_error(5))).unwrap_err();
        assert_eq!(error.code(), win32_error(5).code());
        let cleanup = prepare_startup_cleanup(spec, false, || Ok(())).unwrap();
        assert!(cleanup.remove_current_task && cleanup.remove_legacy_task);
        assert!(cleanup.remove_legacy_run);
    }

    #[test]
    fn successful_task_or_disabled_startup_never_writes_a_fallback() {
        for (enabled, task_registered) in [(true, true), (false, false)] {
            prepare_startup_cleanup(
                StartupSpec {
                    enabled,
                    portable: false,
                },
                task_registered,
                || panic!("fallback must not run"),
            )
            .unwrap();
        }
    }

    #[test]
    #[ignore = "registers an isolated Windows user task; run explicitly"]
    fn isolated_task_registration_round_trip() {
        struct Cleanup(String);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = delete_task(&self.0);
            }
        }
        let unique = task_xml_path();
        let name = format!(
            "PecoFence.StartupTest.{}",
            unique.file_stem().unwrap().to_string_lossy()
        );
        let cleanup = Cleanup(name.clone());
        // Even if interrupted before cleanup, this points to the test runner, never
        // a production application capable of rewriting the real startup task.
        let command = std::env::current_exe()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        for arguments in [None, Some("--portable")] {
            let xml = task_xml(&command, arguments, &current_user_id());
            register_task_xml(&name, &xml).unwrap();
            let output =
                run_schtasks(&["/Query".into(), "/TN".into(), name.clone(), "/XML".into()])
                    .unwrap();
            // schtasks redirects text as a console encoding on Windows. These policy
            // fields and the argument are ASCII in either UTF-8 or the active code page.
            let queried = String::from_utf8_lossy(&output.stdout);
            assert!(queried.contains("<Priority>3</Priority>"));
            // Scheduler omits the default LeastPrivilege element on XML export.
            assert!(
                !queried.contains("<RunLevel>")
                    || queried.contains("<RunLevel>LeastPrivilege</RunLevel>")
            );
            assert!(queried.contains("<LogonType>InteractiveToken</LogonType>"));
            assert!(!queried.contains("<Delay>"));
            assert_eq!(queried.contains("--portable"), arguments.is_some());
        }
        delete_task(&name).unwrap();
        assert!(run_schtasks(&["/Query".into(), "/TN".into(), name, "/XML".into()]).is_err());
        drop(cleanup);
    }

    #[test]
    fn command_executable_strips_quotes_and_portable_arguments() {
        assert_eq!(
            command_executable(r#""C:\Peco Fence\pecofence.exe" --portable"#),
            r"C:\Peco Fence\pecofence.exe"
        );
        assert_eq!(
            command_executable(r"C:\PecoFence\pecofence.exe --portable"),
            r"C:\PecoFence\pecofence.exe"
        );
    }

    #[test]
    fn same_executable_ignores_case_and_tolerates_missing_files() {
        let exe = std::env::current_exe().unwrap();
        let text = exe.to_string_lossy().to_string();
        assert!(same_executable(&text, &text.to_uppercase()));
        assert!(!same_executable(&text, r"C:\nowhere\pecofence.exe"));
        assert!(same_executable(
            r"C:\nowhere\PecoFence.exe",
            r"c:\NOWHERE\pecofence.exe"
        ));
    }
}
