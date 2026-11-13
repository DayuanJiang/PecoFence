//! pecofence-watchdog: waits for the main process to exit. After a crash it starts PecoFence
//! again with the same arguments, and that run takes over the hidden desktop icons. When no
//! PecoFence runs afterwards and the icons are still hidden (marker file still present), it
//! sends the "show desktop icons" toggle itself.
//!
//! Usage: `pecofence-watchdog <parent-pid> <marker-path> [<parent arguments>...]`

#![windows_subsystem = "windows"]

use pecofence_platform::{desktop, process, shell_icons, window};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A run that crashed sooner than this is started again only when the watchdog did not already
/// restart PecoFence within `QUICK_RESTART_GAP`: a start that crashes every time must not loop.
const RESTART_MIN_RUN: Duration = Duration::from_secs(60);
const QUICK_RESTART_GAP: Duration = Duration::from_secs(600);
/// How long a PecoFence the watchdog started has to take over the hidden icons before the
/// watchdog shows them itself.
const RESTART_WAIT: Duration = Duration::from_secs(5);

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
    // When the last restart happened (Unix seconds), kept beside the marker.
    let stamp = marker.with_extension("restarted");
    if let Some((code, ran)) = exit
        && crashed(code)
        && (ran >= RESTART_MIN_RUN || !restarted_within(&stamp, QUICK_RESTART_GAP))
        && !window::SingleInstance::is_held(&mutex)
        && !window::session_ending()
    {
        let _ = std::fs::write(&stamp, now_secs().to_string());
        // PecoFence first: the new run takes over the icons the crashed one hid (the marker
        // tells it), so Windows' own icons do not show for a moment in between. They are
        // shown below only when no PecoFence runs a while later.
        if restart(&args[2..], code, ran) {
            std::thread::sleep(RESTART_WAIT);
        }
    }
    restore_icons(&marker, &mutex);
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Whether the stamp file says the watchdog restarted PecoFence less than `gap` ago.
fn restarted_within(stamp: &Path, gap: Duration) -> bool {
    std::fs::read_to_string(stamp)
        .ok()
        .and_then(|t| t.trim().parse::<u64>().ok())
        .is_some_and(|at| now_secs().saturating_sub(at) < gap.as_secs())
}

/// Tells the restarted run why it was started (its log's first line says so).
const RESTARTED_AFTER: &str = "PECOFENCE_RESTARTED_AFTER";

/// An exception ended the process: an NTSTATUS error such as an access violation (0xC0000005)
/// or a Rust panic's fail-fast (0xC0000409), or a C++ exception; or Windows ended it because
/// it did not respond (0xCFFFFFFF, the user chose "Close the program"): the fences should come
/// back either way. Not Ctrl+C, a kill (exit code 1 or -1) or a sign-out.
fn crashed(code: u32) -> bool {
    const CONTROL_C_EXIT: u32 = 0xC000_013A;
    const CPP_EXCEPTION: u32 = 0xE06D_7363;
    (code & 0xF000_0000 == 0xC000_0000 && code != CONTROL_C_EXIT) || code == CPP_EXCEPTION
}

/// Starts `pecofence.exe` beside this program with the crashed run's arguments and environment
/// (`PECOFENCE_INSTANCE` included); false when nothing was started. Not test runs: their
/// script would run again.
fn restart(args: &[String], code: u32, ran: Duration) -> bool {
    if args
        .iter()
        .any(|a| a == "--test-script" || a == "--exit-after")
    {
        return false;
    }
    // SAFETY: the watchdog runs no other threads that could read the environment meanwhile.
    unsafe {
        std::env::set_var(
            RESTARTED_AFTER,
            format!("{code:#010X} after {} s", ran.as_secs()),
        )
    };
    let Some(exe) = std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join("pecofence.exe")))
    else {
        return false;
    };
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    process::spawn_detached(&exe.to_string_lossy(), &args).is_ok()
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
                // One toggle at a time, sent after Explorer has handled what PecoFence sent
                // before it quit: a second one while Explorer is busy hid the icons again.
                shell_icons::set_desktop_icons_hidden_within(
                    false,
                    host.def_view,
                    host.host,
                    Duration::from_secs(10),
                ) == shell_icons::IconToggle::Done(false)
            });
        if restored {
            let _ = std::fs::remove_file(marker);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{crashed, now_secs, restarted_within};

    #[test]
    fn exceptions_and_hang_kills_count_as_crashes() {
        // Access violation, a Rust panic's fail-fast, heap corruption, a C++ exception, Windows
        // ending a program that does not respond.
        for code in [
            0xC000_0005,
            0xC000_0409,
            0xC000_0374,
            0xE06D_7363,
            0xCFFF_FFFF,
        ] {
            assert!(crashed(code), "{code:#x}");
        }
        // A clean exit, Task Manager (1), TerminateProcess(-1), Ctrl+C, a sign-out.
        for code in [0, 1, 0xFFFF_FFFF, 0xC000_013A, 0x4001_0004] {
            assert!(!crashed(code), "{code:#x}");
        }
    }

    #[test]
    fn a_quick_restart_happens_once_per_gap() {
        let stamp = std::env::temp_dir().join(format!("pf-wd-stamp-{}", std::process::id()));
        let _ = std::fs::remove_file(&stamp);
        let gap = std::time::Duration::from_secs(600);
        assert!(!restarted_within(&stamp, gap), "no stamp yet");
        std::fs::write(&stamp, now_secs().to_string()).unwrap();
        assert!(restarted_within(&stamp, gap));
        std::fs::write(&stamp, (now_secs() - 601).to_string()).unwrap();
        assert!(!restarted_within(&stamp, gap));
        std::fs::write(&stamp, "garbage").unwrap();
        assert!(!restarted_within(&stamp, gap));
        let _ = std::fs::remove_file(&stamp);
    }
}
