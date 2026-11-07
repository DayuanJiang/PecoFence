//! 速览搜索也找整台电脑 (Everything): Everything's state for the settings page, and its
//! 安装 / 启动 buttons.
//! Installing runs winget in a console window the user can watch; Windows asks once for
//! administrator rights. Version 1.5 is installed because it matches Chinese names by pinyin.

use super::*;
use pecofence_platform::everything;
use std::os::windows::process::CommandExt;

/// winget package installed by 「安装 Everything」 (1.5: pinyin search).
const PACKAGE: &str = "voidtools.Everything.Alpha";
const DOWNLOAD_PAGE: &str = "https://www.voidtools.com/downloads/";
/// `WM_APP_EVERYTHING_SETUP` wParam: how the install ended.
const SETUP_OK: usize = 1;
const SETUP_FAILED: usize = 0;
const SETUP_NO_WINGET: usize = 2;
/// 改用 1.5 ended (1.5 runs and starts at sign-in, 1.4 closed).
const SETUP_SWITCHED: usize = 3;
const CREATE_NEW_CONSOLE: u32 = 0x10;
/// How the last install ended, for `WM_APP_EVERYTHING_SETUP` (handled from a timer, which is
/// retried while the App is borrowed: a menu open at that moment must not lose it).
pub(super) static SETUP_RESULT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(SETUP_FAILED);
/// After 安装 / 启动, the settings page asks again this long later (Everything takes a moment
/// to open its window).
const STATUS_RECHECK_MS: u32 = 2500;

impl App {
    /// The 「速览搜索也找整台电脑」 card: running (with version and pinyin), installed, missing, or
    /// being installed.
    pub(super) fn everything_json(&self) -> serde_json::Value {
        if self.everything_installing {
            return serde_json::json!({ "state": "installing" });
        }
        if let Some(i) = everything::find() {
            return serde_json::json!({
                "state": "running",
                "version": format!("{}.{}", i.major, i.minor),
                "pinyin": i.supports_pinyin(),
                // 1.4 runs although 1.5 is installed (only 1.4 starts at sign-in): 「改用 1.5」.
                "has15": !i.supports_pinyin() && everything::installed_15_exe().is_some(),
            });
        }
        let state = if everything::installed_exe().is_some() {
            "installed"
        } else {
            "missing"
        };
        serde_json::json!({ "state": state })
    }

    /// 「安装 Everything」 (or 「升级到 1.5」): winget in a console window, on a worker that waits
    /// for it; the end comes back as `WM_APP_EVERYTHING_SETUP`.
    pub(super) fn install_everything(&mut self) {
        if self.everything_installing {
            return;
        }
        self.everything_installing = true;
        self.push_settings_state();
        let control = self.control.hwnd().0 as isize;
        let spawned = std::thread::Builder::new()
            .name("everything-install".into())
            .spawn(move || {
                let status = std::process::Command::new("winget")
                    .args([
                        "install",
                        "--id",
                        PACKAGE,
                        "--exact",
                        // The community source only: the Microsoft Store source fails on some
                        // networks (certificate pinning, 0x8A15005E) and stops the whole install.
                        "--source",
                        "winget",
                        "--accept-package-agreements",
                        "--accept-source-agreements",
                    ])
                    .creation_flags(CREATE_NEW_CONSOLE)
                    .status();
                let result = match status {
                    Ok(s) if s.success() => SETUP_OK,
                    Ok(s) => {
                        tracing::warn!(code = ?s.code(), "Everything install: winget failed");
                        SETUP_FAILED
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Everything install: winget not found");
                        SETUP_NO_WINGET
                    }
                };
                SETUP_RESULT.store(result, std::sync::atomic::Ordering::Relaxed);
                window::post_message(
                    HWND(control as *mut core::ffi::c_void),
                    WM_APP_EVERYTHING_SETUP,
                    result,
                    0,
                );
            });
        if let Err(e) = spawned {
            tracing::error!(error = %e, "Everything install: worker failed to start");
            self.everything_installing = false;
            self.push_settings_state();
        }
    }

    /// `WM_APP_EVERYTHING_SETUP`: start the fresh install, or send the user to the download
    /// page when there is no winget.
    pub(super) fn on_everything_setup(&mut self, result: usize) {
        self.everything_installing = false;
        tracing::info!(result, "Everything install finished");
        match result {
            // winget also reports success when the administrator prompt was answered No.
            // A fresh 1.5 has no startup entry of its own: make it the one that starts.
            SETUP_OK if everything::installed_15_exe().is_some() => self.use_everything_15(),
            SETUP_OK if everything::installed_exe().is_some() => self.start_everything(),
            SETUP_SWITCHED => {
                self.settings_toast(pecofence_core::i18n::text(
                    "已改用 Everything 1.5，以后登录 Windows 时也会启动它",
                ));
                window::set_timer(
                    self.control.hwnd(),
                    TIMER_EVERYTHING_STATUS,
                    STATUS_RECHECK_MS,
                );
            }
            SETUP_NO_WINGET => {
                let _ = shell::shell_execute(Path::new(DOWNLOAD_PAGE), None, None);
                self.settings_toast(pecofence_core::i18n::text(
                    "这台电脑没有 winget，已打开 Everything 的下载页面",
                ));
            }
            _ => self.settings_toast(pecofence_core::i18n::text(
                "Everything 没有装好，可以再试一次或到 voidtools.com 下载",
            )),
        }
        self.push_settings_state();
    }

    /// 「改用 1.5」, and the end of 「安装 Everything」 / 「升级到 1.5」: Everything 1.5 runs and starts
    /// at sign-in; a running 1.4 is closed and no longer starts at sign-in (its machine-wide
    /// entry makes Windows ask for administrator rights once). Everything's own switches do it,
    /// on a worker: the prompt can wait. The end comes back as `WM_APP_EVERYTHING_SETUP`.
    pub(super) fn use_everything_15(&mut self) {
        let Some(new) = everything::installed_15_exe() else {
            self.start_everything();
            return;
        };
        let old = everything::find()
            .filter(|i| !i.supports_pinyin())
            .and_then(|i| i.exe())
            .filter(|p| !p.eq(&new));
        let control = self.control.hwnd().0 as isize;
        let spawned = std::thread::Builder::new()
            .name("everything-switch".into())
            .spawn(move || {
                // ShellExecuteEx (runas) wants a single-threaded apartment on its thread.
                let _com = pecofence_platform::com::OleGuard::init();
                if let Some(old) = &old {
                    let dir = old.parent();
                    let exited = everything::run_program(old, "-exit", dir, false, 5000);
                    tracing::info!(exe = %old.display(), exited, "Everything switch: 1.4 asked to exit");
                    // This user's entry (written by Everything's own startup option), if any.
                    let user_entry = pecofence_platform::autostart::registered_path("Everything")
                        .is_some_and(|p| everything::command_names(&p, old));
                    let user_removed = user_entry
                        && pecofence_platform::autostart::set_command("Everything", None).is_ok();
                    // The installer's entry for all users: an elevated reg.exe, one prompt.
                    let keys = everything::all_users_run_keys(old);
                    let reg = std::env::var_os("SystemRoot")
                        .map(|r| std::path::PathBuf::from(r).join(r"System32\reg.exe"))
                        .unwrap_or_else(|| std::path::PathBuf::from("reg.exe"));
                    let mut removed = 0;
                    for key in &keys {
                        let params = format!("delete \"{key}\" /v Everything /f");
                        if everything::run_program(&reg, &params, None, true, 120_000) {
                            removed += 1;
                        }
                    }
                    let machine_left = everything::all_users_run_keys(old).len();
                    tracing::info!(
                        exited, user_entry, user_removed, machine_keys = keys.len(), removed, machine_left,
                        "Everything 1.4 closed and taken out of startup"
                    );
                }
                let mut cmd = std::process::Command::new(&new);
                cmd.arg("-startup");
                if let Some(dir) = new.parent() {
                    cmd.current_dir(dir);
                }
                let started = cmd.spawn().is_ok();
                // The value Everything 1.5's own 「登录时启动」 writes for this user.
                let command = format!("\"{}\" -startup", new.display());
                let startup =
                    pecofence_platform::autostart::set_command("Everything 1.5a", Some(&command)).is_ok();
                tracing::info!(exe = %new.display(), started, startup, "Everything 1.5 started and set to start at sign-in");
                SETUP_RESULT.store(SETUP_SWITCHED, std::sync::atomic::Ordering::Relaxed);
                window::post_message(
                    HWND(control as *mut core::ffi::c_void),
                    WM_APP_EVERYTHING_SETUP,
                    SETUP_SWITCHED,
                    0,
                );
            });
        if let Err(e) = spawned {
            tracing::error!(error = %e, "Everything switch: worker failed to start");
        }
    }

    /// 「启动 Everything」: in the background (its tray icon), then the card asks again.
    pub(super) fn start_everything(&mut self) {
        match everything::installed_exe() {
            Some(exe) => {
                // Unquoted switches: Everything reads a quoted "-startup" as search text. It
                // runs in its own folder: it keeps running after PecoFence, and PecoFence's
                // folder held as its working directory could not be removed by the uninstaller.
                let mut cmd = std::process::Command::new(&exe);
                cmd.arg("-startup");
                if let Some(dir) = exe.parent() {
                    cmd.current_dir(dir);
                }
                let started = cmd.spawn();
                tracing::info!(exe = %exe.display(), ok = started.is_ok(), "Everything started");
            }
            None => tracing::warn!("Everything start: no Everything.exe found"),
        }
        window::set_timer(
            self.control.hwnd(),
            TIMER_EVERYTHING_STATUS,
            STATUS_RECHECK_MS,
        );
    }
}
