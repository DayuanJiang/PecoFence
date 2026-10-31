//! pecofence-watchdog: waits for the main process to exit and, if it died without restoring the
//! desktop icons (marker file still present), sends the "show desktop icons" toggle itself.
//! After a crash it starts PecoFence again with the same arguments.
//!
//! Usage: `pecofence-watchdog <parent-pid> <marker-path> [<parent arguments>...]`

#![windows_subsystem = "windows"]

use pecofence_platform::{desktop, process, shell_icons, window};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A run that crashed sooner than this is not started again: it would likely crash again.
const RESTART_MIN_RUN: Duration = Duration::from_secs(60);

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(pid) = args.first().and_then(|a| a.parse::<u32>().ok()) else {
        return;
    };
    let Some(marker) = args.get(1).map(PathBuf::from) else {
        return;
    };
    let instance = pecofence_core::brand::var("PECOFENCE_INSTANCE").ok();
    let [mutex, _] = pecofence_core::brand::instance_mutex_names(instance.as_deref());

    // Block until the parent exits (or fails to open → it is already gone).
    let exit = process::wait_for_process_exit(pid);
    restore_icons(&marker, &mutex);
    if let Some((code, ran)) = exit
        && crashed(code)
        && ran >= RESTART_MIN_RUN
        && !window::SingleInstance::is_held(&mutex)
    {
        restart(&args[2..]);
    }
}

/// An exception ended the process: an NTSTATUS error such as an access violation (0xC0000005)
/// or a Rust panic's fail-fast (0xC0000409), or a C++ exception. Not Windows ending a program
/// that did not respond (0xCFFFFFFF), Ctrl+C, a kill (exit code 1 or -1) or a sign-out.
fn crashed(code: u32) -> bool {
    const APPLICATION_HUNG: u32 = 0xCFFF_FFFF;
    const CONTROL_C_EXIT: u32 = 0xC000_013A;
    const CPP_EXCEPTION: u32 = 0xE06D_7363;
    (code & 0xF000_0000 == 0xC000_0000 && code != APPLICATION_HUNG && code != CONTROL_C_EXIT)
        || code == CPP_EXCEPTION
}

/// Starts `pecofence.exe` beside this program with the crashed run's arguments and environment
/// (`PECOFENCE_INSTANCE` included). Not test runs: their script would run again.
fn restart(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--test-script" || a == "--exit-after")
    {
        return;
    }
    let Some(exe) = std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join("pecofence.exe")))
    else {
        return;
    };
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let _ = process::spawn_detached(&exe.to_string_lossy(), &args);
}

fn restore_icons(marker: &Path, mutex: &str) {
    if !marker.exists() {
        return; // clean shutdown restored the icons already
    }
    // Give Explorer a moment, then restore if still hidden. Explorer may be restarting or hung,
    // so keep trying for a while; if it never works the marker stays, and the next start adopts
    // the hidden icons and restores them at exit. A PecoFence started meanwhile takes them
    // over: the icons and the marker are its own then.
    std::thread::sleep(Duration::from_millis(300));
    for attempt in 0..10 {
        if attempt > 0 {
            std::thread::sleep(Duration::from_secs(1));
        }
        if window::SingleInstance::is_held(mutex) {
            return;
        }
        let restored = !shell_icons::desktop_icons_hidden()
            || desktop::resolve_icon_host(desktop::detect_generation()).is_some_and(|host| {
                shell_icons::set_desktop_icons_hidden(false, host.def_view, host.host)
                    == Some(false)
            });
        if restored {
            let _ = std::fs::remove_file(marker);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::crashed;

    #[test]
    fn only_exceptions_count_as_crashes() {
        // Access violation, a Rust panic's fail-fast, heap corruption, a C++ exception.
        for code in [0xC000_0005, 0xC000_0409, 0xC000_0374, 0xE06D_7363] {
            assert!(crashed(code), "{code:#x}");
        }
        // A clean exit, Task Manager (1), TerminateProcess(-1), a hang kill, Ctrl+C, a sign-out.
        for code in [0, 1, 0xFFFF_FFFF, 0xCFFF_FFFF, 0xC000_013A, 0x4001_0004] {
            assert!(!crashed(code), "{code:#x}");
        }
    }
}
