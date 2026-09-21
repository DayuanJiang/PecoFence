//! Run-at-logon via `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.

use crate::bindings::*;
use crate::wide::{from_wide, to_wide};
use windows_core::{Error, PCWSTR, PWSTR, Result};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RRF_RT_REG_SZ: u32 = 0x0000_0002;

#[derive(Debug, Default, PartialEq)]
struct StartupActions {
    register_current: bool,
    remove_current: bool,
    remove_legacy: bool,
}

/// `current_is_this_exe`: `None` when nothing is registered, `Some(true)` when the entry
/// already launches this executable, `Some(false)` when it launches another copy (an older
/// release in a different folder, or a deleted one).
fn startup_actions(
    wanted: bool,
    development: bool,
    current_is_this_exe: Option<bool>,
    legacy: bool,
) -> StartupActions {
    if development {
        return StartupActions::default();
    }
    StartupActions {
        register_current: wanted && current_is_this_exe != Some(true),
        remove_current: !wanted && current_is_this_exe.is_some(),
        remove_legacy: legacy,
    }
}

/// The release that is running owns login startup: an entry pointing at another copy (an
/// older version left in Downloads, a moved folder) is re-pointed here, so upgrading by
/// unzipping a new build and launching it is enough. Also adopts the previous product
/// name's entry and removes it only after the new one is written.
pub fn reconcile_product(wanted: bool, development: bool) -> Result<()> {
    use pecofence_core::brand::{LEGACY_AUTOSTART, NAME};
    if crate::process::is_packaged() {
        return Ok(());
    }
    let current = registered_path(NAME);
    let actions = startup_actions(
        wanted,
        development,
        current
            .as_deref()
            .map(|path| same_executable(path, &exe_path())),
        registered_path(LEGACY_AUTOSTART).is_some(),
    );
    if actions.register_current {
        set_enabled(NAME, true)?;
    }
    if actions.remove_current {
        set_enabled(NAME, false)?;
    }
    if actions.remove_legacy {
        set_enabled(LEGACY_AUTOSTART, false)?;
    }
    Ok(())
}

/// Explicit settings changes address both names, avoiding duplicate login launches.
/// Store (MSIX) installs leave the registry alone: the package startup task owns autostart.
pub fn set_product_enabled(enabled: bool) -> Result<()> {
    use pecofence_core::brand::{LEGACY_AUTOSTART, NAME};
    if crate::process::is_packaged() {
        return Ok(());
    }
    set_enabled(NAME, enabled)?;
    if registered_path(LEGACY_AUTOSTART).is_some() {
        set_enabled(LEGACY_AUTOSTART, false)?;
    }
    Ok(())
}

/// Whether two paths name the same file. Canonical paths first (case, 8.3 names, symlinks),
/// then a case-insensitive comparison when one of them no longer exists.
fn same_executable(a: &str, b: &str) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => a.eq_ignore_ascii_case(b),
    }
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

pub fn set_enabled(value_name: &str, enabled: bool) -> Result<()> {
    let hkey = open_run_key()?;
    let name = to_wide(value_name);
    // SAFETY: buffers outlive the calls; key closed afterwards.
    let status = unsafe {
        let s = if enabled {
            let cmd = to_wide(&format!("\"{}\"", exe_path()));
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

/// The executable path currently registered for `value_name` (quotes stripped), if any.
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
    Some(raw.trim_matches('"').to_string())
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
    status.0 == 0 && !from_wide(&buf).is_empty()
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
    fn startup_follows_the_running_release_and_removes_duplicates() {
        assert_eq!(
            startup_actions(true, false, None, true),
            StartupActions {
                register_current: true,
                remove_legacy: true,
                ..Default::default()
            }
        );
        assert_eq!(
            startup_actions(true, false, Some(true), true),
            StartupActions {
                remove_legacy: true,
                ..Default::default()
            }
        );
        // Registered to another copy (older version elsewhere, or deleted): take over.
        assert!(startup_actions(true, false, Some(false), false).register_current);
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

    #[test]
    fn disabled_and_development_startup_do_not_enable_the_application() {
        assert_eq!(
            startup_actions(false, false, Some(true), true),
            StartupActions {
                remove_current: true,
                remove_legacy: true,
                ..Default::default()
            }
        );
        for wanted in [true, false] {
            assert_eq!(
                startup_actions(wanted, true, Some(false), true),
                StartupActions::default()
            );
        }
    }
}
