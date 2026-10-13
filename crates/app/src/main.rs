//! PecoFence — open-source Fences-style desktop organizer for Windows 11.
// GUI subsystem: no console window when launched from Explorer. Logs go to a file (see
// `init_logging`); stderr is still used when a console is attached (e.g. `cargo run`).
#![windows_subsystem = "windows"]

mod anchor;
mod app;
mod commands;
mod drag_guides;
mod drop_preview;
mod fence_window;
mod icons;
mod ipc_server;
mod layout;
mod marquee_band;
mod notice;
mod peek;
mod peek_search;
mod rename;
mod settings_host;
mod shadow;
mod share_log;
mod state;

use pecofence_platform::com::OleGuard;
use pecofence_platform::window;
use windows_core::Result;

fn parse_args() -> app::Args {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |name: &str| args.iter().any(|a| a == name);
    let value = |name: &str| -> Option<String> {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    app::Args {
        light: has("--light"),
        dark: has("--dark"),
        wallpaper_override: value("--wallpaper"),
        portable: has("--portable"),
        no_hide_icons: has("--no-hide-icons"),
        exit_after_ms: value("--exit-after").and_then(|v| v.parse().ok()),
        dump_stats: has("--dump-stats"),
        open_settings: has("--open-settings"),
        portal: value("--portal"),
        test_script: value("--test-script"),
        instance: None,
    }
}

/// `%LOCALAPPDATA%\PecoFence`: the logs (see `pecofence_core::brand::log_file_name`) and crash
/// dumps.
pub(crate) fn log_dir() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from)?;
    let dir = base.join("PecoFence");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// Starts this run's `current` log file, keeping the last run's as `previous` (a crash report
/// needs the log of the run that crashed). A second instance (PECOFENCE_INSTANCE) keeps its own
/// files.
fn start_log(
    dir: &std::path::Path,
    instance: Option<&str>,
    current: &str,
    previous: &str,
) -> Option<std::fs::File> {
    use pecofence_core::brand::log_file_name;
    let path = dir.join(log_file_name(instance, current));
    let prev = dir.join(log_file_name(instance, previous));
    // The Store edition installed on another drive cannot rename files in its copy of
    // %LOCALAPPDATA% (os error 17): keep a copy instead.
    if let Err(e) = std::fs::rename(&path, &prev)
        && e.kind() == std::io::ErrorKind::CrossesDevices
    {
        let _ = std::fs::copy(&path, &prev);
    }
    std::fs::File::create(path).ok()
}

/// `RUST_LOG` takes `level` and `target=level` directives, e.g. `pecofence=debug` (default
/// `info`). `Targets` instead of `EnvFilter` keeps the regex engine out of the binary; span and
/// field filters are not supported. Output goes to the log file and to stderr; the latter only
/// shows up when a console is attached. The shareable log (`share_log`) runs beside them.
fn init_logging(instance: Option<&str>) {
    use pecofence_core::brand;
    use tracing_subscriber::filter::Targets;
    use tracing_subscriber::fmt::writer::MakeWriterExt;
    use tracing_subscriber::prelude::*;
    let filter = std::env::var("RUST_LOG")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .and_then(|s| s.parse::<Targets>().ok())
        .unwrap_or_else(|| Targets::new().with_default(tracing::Level::INFO));
    let dir = log_dir();
    let file = dir
        .as_deref()
        .and_then(|d| start_log(d, instance, brand::LOG, brand::PREVIOUS_LOG));
    let share = dir
        .as_deref()
        .and_then(|d| start_log(d, instance, brand::SHARE_LOG, brand::PREVIOUS_SHARE_LOG))
        .map(share_log::ShareLayer::new);
    match file {
        Some(file) => {
            let file = std::sync::Mutex::new(file);
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_ansi(false)
                        .with_writer(file.and(std::io::stderr)),
                )
                .with(share)
                .with(filter)
                .init();
        }
        None => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
                .with(share)
                .with(filter)
                .init();
        }
    }
}

/// Panics inside a window procedure or COM callback abort the process (GUI subsystem: no
/// console to print to), so the message and a backtrace go to the log first.
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        // A literal panic message is code text; a formatted one may hold data, so it stays in
        // `detail`, which the shareable log omits.
        let payload = info
            .payload()
            .downcast_ref::<&'static str>()
            .copied()
            .unwrap_or("(formatted message)");
        let bt = std::backtrace::Backtrace::force_capture();
        tracing::error!(%location, payload, detail = %info, backtrace = %bt, "PANIC");
    }));
}

/// Asks the running instance to open its settings window. Fire and forget: it answers late
/// while one of its menus is open, and this process has nothing else to do. Not when that
/// instance has only just started: two autostart entries (say the Store and the ZIP edition)
/// race at logon, and the loser must not open Settings.
fn show_running_settings(instance: Option<&str>) {
    use std::io::Write;
    use std::os::windows::fs::OpenOptionsExt;
    const SECURITY_IDENTIFICATION: u32 = 0x0001_0000;
    const STARTUP: std::time::Duration = std::time::Duration::from_secs(30);
    pecofence_platform::process::allow_foreground_handoff();
    let request = pecofence_ipc::Request::new(pecofence_ipc::Method::SettingsOpenUi);
    let pipe = std::fs::OpenOptions::new()
        .write(true)
        .security_qos_flags(SECURITY_IDENTIFICATION)
        .open(pecofence_core::brand::ipc_pipe_name(instance));
    let Ok(mut pipe) = pipe else {
        return;
    };
    if pecofence_platform::process::pipe_server_age(&pipe).is_some_and(|age| age < STARTUP) {
        return;
    }
    if let Ok(line) = serde_json::to_string(&request) {
        let _ = writeln!(pipe, "{line}");
    }
}

fn main() -> Result<()> {
    // `PECOFENCE_INSTANCE=<name>` runs a second, independent instance (developer testing with
    // `--portable`); the default name keeps one PecoFence per session. Checked before logging
    // starts: a second launch must leave the running instance's log files alone (it used to
    // truncate the log on its way out).
    let instance_name = pecofence_core::brand::var("PECOFENCE_INSTANCE").ok();
    let instance = instance_name
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(str::to_string);
    let [current_name, legacy_name] =
        pecofence_core::brand::instance_mutex_names(instance_name.as_deref());
    let Some(_instance) = window::SingleInstance::acquire(&current_name) else {
        // Started again from the Start menu or a shortcut: Windows 11 hides new tray icons
        // in the overflow, so show the running copy's settings rather than exit silently.
        if std::env::args().len() == 1 {
            show_running_settings(instance.as_deref());
        }
        return Ok(());
    };
    // A pre-rename instance is running: exit it before starting PecoFence.
    let Some(_legacy_instance) = window::SingleInstance::acquire(&legacy_name) else {
        return Ok(());
    };

    init_logging(instance.as_deref());
    install_panic_hook();
    if let Some(dir) = log_dir() {
        pecofence_platform::crashlog::install(dir, instance.as_deref().unwrap_or("main"));
    }

    let mut args = parse_args();
    args.instance = instance;
    let exit_after = args.exit_after_ms;
    // Back after an update closed it. Not named test instances: their environment would be
    // lost and the restart would take the main instance's place.
    if args.instance.is_none()
        && exit_after.is_none()
        && let Err(e) = pecofence_platform::process::register_restart_after_update()
    {
        tracing::warn!(error = %e, "restart after updates not registered");
    }

    window::set_process_dpi_awareness_v2();
    let _ole = OleGuard::init()?;

    let cell = app::App::create(args)?;
    if let Some(ms) = exit_after {
        window::quit_after(ms);
    }

    let code = window::run_message_loop();
    if let Some(app) = cell.borrow_mut().as_mut() {
        app.shutdown();
    }
    drop(cell);
    std::process::exit(code);
}
