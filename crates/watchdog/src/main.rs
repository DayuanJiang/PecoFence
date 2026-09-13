//! pecofence-watchdog: waits for the main process to exit and, if it died without restoring the
//! desktop icons (marker file still present), sends the "show desktop icons" toggle itself.
//!
//! Usage: `pecofence-watchdog <parent-pid> <marker-path>`

#![windows_subsystem = "windows"]

use pecofence_platform::{desktop, process, shell_icons};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(pid) = args.first().and_then(|a| a.parse::<u32>().ok()) else {
        return;
    };
    let Some(marker) = args.get(1).map(PathBuf::from) else {
        return;
    };

    // Block until the parent exits (or fails to open → it is already gone).
    process::wait_for_process_exit(pid);

    if !marker.exists() {
        return; // clean shutdown restored the icons already
    }
    // Give Explorer a moment, then restore if still hidden.
    std::thread::sleep(std::time::Duration::from_millis(300));
    let generation = desktop::detect_generation();
    if let Some(host) = desktop::resolve_icon_host(generation)
        && shell_icons::desktop_icons_hidden()
    {
        let _ = shell_icons::set_desktop_icons_hidden(false, host.def_view, host.host);
    }
    let _ = std::fs::remove_file(&marker);
}
