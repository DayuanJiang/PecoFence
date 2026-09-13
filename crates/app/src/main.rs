//! PecoFence — open-source Fences-style desktop organizer for Windows 11.
// GUI subsystem: no console window when launched from Explorer. Logs go to a file (see
// `init_logging`); stderr is still used when a console is attached (e.g. `cargo run`).
#![windows_subsystem = "windows"]

mod anchor;
mod app;
mod commands;
mod fence_window;
mod icons;
mod layout;
mod peek;
mod rename;
mod settings_host;
mod shadow;
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
    }
}

/// Log file next to the config: `%LOCALAPPDATA%\PecoFence\pecofence.log` (truncated per run).
fn log_file_path() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from)?;
    let dir = base.join("PecoFence");
    std::fs::create_dir_all(&dir).ok()?;
    // A second instance (PECOFENCE_INSTANCE) logs to its own file instead of truncating the
    // main one.
    let name = match pecofence_core::brand::var("PECOFENCE_INSTANCE") {
        Ok(n) if !n.trim().is_empty() => format!("pecofence.{}.log", n.trim()),
        _ => "pecofence.log".to_string(),
    };
    Some(dir.join(name))
}

/// `RUST_LOG` filters as usual (default `info`). Output goes to the log file and to stderr; the
/// latter only shows up when a console is attached.
fn init_logging() {
    use tracing_subscriber::fmt::writer::MakeWriterExt;
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let file = log_file_path().and_then(|p| std::fs::File::create(p).ok());
    match file {
        Some(file) => {
            let file = std::sync::Mutex::new(file);
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_ansi(false)
                .with_writer(file.and(std::io::stderr))
                .init();
        }
        None => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(std::io::stderr)
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
        let bt = std::backtrace::Backtrace::force_capture();
        tracing::error!(%location, "PANIC: {info}
{bt}");
    }));
}

fn main() -> Result<()> {
    init_logging();
    install_panic_hook();
    if let Some(dir) = log_file_path().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        let instance =
            pecofence_core::brand::var("PECOFENCE_INSTANCE").unwrap_or_else(|_| "main".into());
        pecofence_platform::crashlog::install(dir, instance.trim());
    }

    let args = parse_args();
    let exit_after = args.exit_after_ms;

    window::set_process_dpi_awareness_v2();
    let _ole = OleGuard::init()?;

    // `PECOFENCE_INSTANCE=<name>` runs a second, independent instance (developer testing with
    // `--portable`); the default name keeps one PecoFence per session.
    let instance_name = pecofence_core::brand::var("PECOFENCE_INSTANCE").ok();
    let [current_name, legacy_name] =
        pecofence_core::brand::instance_mutex_names(instance_name.as_deref());
    let Some(_instance) = window::SingleInstance::acquire(&current_name) else {
        tracing::warn!("another PecoFence instance is running; exiting");
        return Ok(());
    };
    let Some(_legacy_instance) = window::SingleInstance::acquire(&legacy_name) else {
        tracing::warn!("a pre-rename instance is running; exit it before starting PecoFence");
        return Ok(());
    };

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
