//! Run at logon through a Task Scheduler logon task, which starts at sign-in. Explorer launches
//! Run-key entries one at a time after its own startup delay, 20 to 30 seconds later on a PC
//! with a few of them; the Run key remains the fallback when the task cannot be registered.
//! The Store edition keeps its manifest startup task and adds the same logon task while
//! Windows' startup switch for it is on.

use crate::bindings::*;
use crate::wide::{from_wide, to_wide};
use pecofence_core::brand::{LEGACY_AUTOSTART, NAME};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use windows_core::{Error, HRESULT, PCWSTR, PWSTR, Result};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// Uninstalling a Store package runs none of our code, so its task expires unless a launch
/// renews it, and Task Scheduler then deletes it.
const STORE_TASK_DAYS: i64 = 30;
/// Marks a Store launch by the logon task (see [`logon_task_unwanted`]).
pub const LOGON_TASK_ARG: &str = "--logon-task";

/// At launch the running release owns login startup: a task pointing at another copy (an older
/// version left in Downloads, a moved folder) is re-pointed here, and the Run-key entries of
/// earlier versions are removed. Runs on a worker: logon is when the first frame matters most.
pub fn reconcile_product(wanted: bool, development: bool) {
    if development {
        return;
    }
    if crate::process::is_packaged() {
        in_background(|| {
            let _com = crate::com::MtaGuard::init()?;
            reconcile_store()
        });
    } else {
        in_background(move || apply(wanted, false));
    }
}

/// The Settings switch. A portable copy starts with `--portable`, so it reads the `config`
/// folder beside it again. The Store edition's switch belongs to Windows.
pub fn set_product_enabled(enabled: bool, portable: bool) {
    if !crate::process::is_packaged() {
        in_background(move || apply(enabled, portable));
    }
}

/// Store edition launched by the logon task after the user turned its startup off in Windows:
/// removes the task and returns true, so the caller exits before showing anything.
pub fn logon_task_unwanted() -> bool {
    if !crate::process::is_packaged() {
        return false;
    }
    // A worker for WinRT: this thread has no COM apartment yet.
    let enabled = std::thread::spawn(|| {
        let _com = crate::com::MtaGuard::init()?;
        crate::store::startup_task_enabled()
    })
    .join();
    if !matches!(enabled, Ok(Ok(false))) {
        return false;
    }
    if let Err(error) = delete_task(NAME) {
        tracing::warn!(%error, "logon task not removed");
    }
    true
}

static LATEST: AtomicU64 = AtomicU64::new(0);
static APPLYING: Mutex<()> = Mutex::new(());

/// Runs `work` on a worker. Changes apply one at a time, and one superseded by a newer request
/// before its turn is skipped, so the last switch position wins.
fn in_background(work: impl FnOnce() -> Result<()> + Send + 'static) {
    let generation = LATEST.fetch_add(1, Ordering::SeqCst) + 1;
    let spawned = std::thread::Builder::new()
        .name("autostart".into())
        .spawn(move || {
            let _turn = APPLYING.lock().unwrap_or_else(|e| e.into_inner());
            if LATEST.load(Ordering::SeqCst) == generation
                && let Err(error) = work()
            {
                tracing::warn!(%error, "login startup not updated");
            }
        });
    if let Err(error) = spawned {
        tracing::warn!(%error, "login startup worker not started");
    }
}

fn apply(enabled: bool, portable: bool) -> Result<()> {
    let (exe, args) = (exe_path(), portable.then_some("--portable"));
    let mut run_value = None;
    if !enabled {
        delete_task(NAME)?;
    } else if let Err(error) = register_task(NAME, &exe, args, None) {
        tracing::warn!(%error, "logon task not registered; using the Run key");
        // An older task would otherwise start another copy first.
        let _ = delete_task(NAME);
        run_value = Some(command_line(&exe, args));
    } else {
        tracing::info!(portable, "logon task registered");
    }
    set_run_value(NAME, run_value.as_deref())?;
    set_run_value(LEGACY_AUTOSTART, None)
}

/// Store edition: the task starts the package through its `pecofence.exe` execution alias (the
/// install folder changes with every update) and exists only while Windows' switch is on.
fn reconcile_store() -> Result<()> {
    let alias = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(r"Microsoft\WindowsApps\pecofence.exe");
    if !crate::store::startup_task_enabled()? || !alias.exists() {
        return delete_task(NAME);
    }
    let expires = format!(
        "{}T00:00:00",
        pecofence_core::config_store::date_in_days(STORE_TASK_DAYS)
    );
    register_task(
        NAME,
        &alias.to_string_lossy(),
        Some(LOGON_TASK_ARG),
        Some(&expires),
    )?;
    tracing::info!(%expires, "logon task registered");
    Ok(())
}

/// The current user's logon task. Without these settings a task does not start on battery, is
/// stopped after 72 hours and runs below normal priority; 4 is the normal priority a Run-key
/// launch gets. `expires` (local time) lets Task Scheduler delete the task once no launch has
/// renewed it.
fn task_xml(command: &str, args: Option<&str>, user: &str, expires: Option<&str>) -> String {
    let end = expires
        .map(|at| format!("\n      <EndBoundary>{at}</EndBoundary>"))
        .unwrap_or_default();
    let delete = if expires.is_some() {
        "\n    <DeleteExpiredTaskAfter>PT0S</DeleteExpiredTaskAfter>"
    } else {
        ""
    };
    let args = args
        .map(|a| format!("\n      <Arguments>{}</Arguments>", xml_escape(a)))
        .unwrap_or_default();
    let (command, user) = (xml_escape(command), xml_escape(user));
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <Triggers>
    <LogonTrigger>{end}
      <Enabled>true</Enabled>
      <UserId>{user}</UserId>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{user}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>{delete}
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>4</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{command}</Command>{args}
    </Exec>
  </Actions>
</Task>
"#
    )
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Registers the logon task `name`, replacing an existing one.
fn register_task(
    name: &str,
    command: &str,
    args: Option<&str>,
    expires: Option<&str>,
) -> Result<()> {
    // The SID: an ssh session's USERDOMAIN is WORKGROUP, which names no account.
    let xml = task_xml(command, args, &crate::pipe::current_user_sid()?, expires);
    // UTF-16 with a byte order mark, as the XML declaration says.
    let bytes: Vec<u8> = std::iter::once(0xFEFF)
        .chain(xml.encode_utf16())
        .flat_map(u16::to_le_bytes)
        .collect();
    let path = std::env::temp_dir().join(format!("pecofence-task-{}.xml", std::process::id()));
    std::fs::write(&path, bytes).map_err(io_error)?;
    let result = schtasks(&[
        "/Create",
        "/TN",
        name,
        "/XML",
        &path.to_string_lossy(),
        "/F",
    ]);
    let _ = std::fs::remove_file(&path);
    result
}

/// Removes the logon task `name`. A missing task is fine; one that remains is an error.
fn delete_task(name: &str) -> Result<()> {
    match schtasks(&["/Delete", "/TN", name, "/F"]) {
        Err(error) if schtasks(&["/Query", "/TN", name]).is_ok() => Err(error),
        _ => Ok(()),
    }
}

fn schtasks(args: &[&str]) -> Result<()> {
    let exe = std::env::var_os("SystemRoot")
        .map(|root| PathBuf::from(root).join(r"System32\schtasks.exe"))
        .unwrap_or_else(|| PathBuf::from("schtasks.exe"));
    let output = Command::new(exe)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW as u32)
        .stdin(Stdio::null())
        .output()
        .map_err(io_error)?;
    if output.status.success() {
        return Ok(());
    }
    // The message is in the console code page; good enough for the log.
    let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(Error::new(
        E_FAIL,
        format!("schtasks {}: {message}", args[0]),
    ))
}

fn io_error(error: std::io::Error) -> Error {
    Error::new(E_FAIL, error.to_string())
}

/// Full path of the running executable.
pub fn exe_path() -> String {
    let mut buf = [0u16; 1024];
    // SAFETY: buffer length passed.
    let n = unsafe { GetModuleFileNameW(None, PWSTR(buf.as_mut_ptr()), buf.len() as u32) };
    from_wide(&buf[..n as usize])
}

fn command_line(exe: &str, args: Option<&str>) -> String {
    match args {
        Some(args) => format!("\"{exe}\" {args}"),
        None => format!("\"{exe}\""),
    }
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

/// Writes the Run value `value_name`, or deletes it for `None`.
fn set_run_value(value_name: &str, command: Option<&str>) -> Result<()> {
    let hkey = open_run_key()?;
    let name = to_wide(value_name);
    // SAFETY: buffers outlive the calls; key closed afterwards.
    let status = unsafe {
        let s = if let Some(command) = command {
            let cmd = to_wide(command);
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

fn win32_error(code: u32) -> Error {
    // HRESULT_FROM_WIN32
    let hr = if code == 0 {
        0
    } else {
        (code & 0xffff) | 0x8007_0000
    };
    Error::from_hresult(HRESULT(hr as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_desktop_task_never_expires_and_keeps_its_arguments() {
        let xml = task_xml(
            r"C:\Peco & Co\pecofence.exe",
            Some("--portable"),
            r"PC\me",
            None,
        );
        assert!(xml.contains(r"<Command>C:\Peco &amp; Co\pecofence.exe</Command>"));
        assert!(xml.contains("<Arguments>--portable</Arguments>"));
        assert_eq!(xml.matches(r"<UserId>PC\me</UserId>").count(), 2);
        assert!(!xml.contains("EndBoundary") && !xml.contains("DeleteExpiredTaskAfter"));
        assert!(!task_xml("a.exe", None, "me", None).contains("<Arguments>"));
    }

    #[test]
    fn a_store_task_expires_and_is_then_deleted() {
        let xml = task_xml(
            "a.exe",
            Some(LOGON_TASK_ARG),
            "me",
            Some("2026-11-10T00:00:00"),
        );
        assert!(xml.contains("<EndBoundary>2026-11-10T00:00:00</EndBoundary>"));
        assert!(xml.contains("<DeleteExpiredTaskAfter>PT0S</DeleteExpiredTaskAfter>"));
    }

    /// `cargo test -p pecofence-platform -- --ignored` registers a uniquely named real task.
    #[test]
    #[ignore = "writes to Task Scheduler"]
    fn task_scheduler_accepts_replaces_and_deletes_the_task() {
        let name = format!("{NAME}.Test.{}", std::process::id());
        let exe = exe_path();
        let expires = Some("2099-01-01T00:00:00");
        register_task(&name, &exe, Some(LOGON_TASK_ARG), expires).unwrap();
        register_task(&name, &exe, Some("--portable"), None).unwrap();
        assert!(schtasks(&["/Query", "/TN", &name]).is_ok());
        delete_task(&name).unwrap();
        delete_task(&name).unwrap();
        assert!(schtasks(&["/Query", "/TN", &name]).is_err());
    }

    #[test]
    fn the_run_key_fallback_quotes_the_executable() {
        assert_eq!(command_line(r"C:\A B\p.exe", None), r#""C:\A B\p.exe""#);
        assert_eq!(
            command_line(r"C:\A B\p.exe", Some("--portable")),
            r#""C:\A B\p.exe" --portable"#
        );
    }
}
