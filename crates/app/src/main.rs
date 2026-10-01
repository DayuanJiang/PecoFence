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
mod peek;
mod rename;
mod runtime;
mod settings_host;
mod shadow;
mod state;
mod updates;

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

/// `RUST_LOG` takes `level` and `target=level` directives, e.g. `pecofence=debug` (default
/// `info`). `Targets` instead of `EnvFilter` keeps the regex engine out of the binary; span and
/// field filters are not supported. Output goes to the log file and to stderr; the latter only
/// shows up when a console is attached.
fn init_logging(file: std::fs::File) {
    use tracing_subscriber::filter::Targets;
    use tracing_subscriber::fmt::writer::MakeWriterExt;
    use tracing_subscriber::prelude::*;
    let filter = std::env::var("RUST_LOG")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .and_then(|s| s.parse::<Targets>().ok())
        .unwrap_or_else(|| Targets::new().with_default(tracing::Level::INFO));
    let file = std::sync::Mutex::new(file);
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(file.and(std::io::stderr)),
        )
        .with(filter)
        .init();
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
    let mut args = parse_args();
    let exit_after = args.exit_after_ms;
    window::set_process_dpi_awareness_v2();
    pecofence_core::i18n::set_language(pecofence_core::i18n::Language::from_locale(
        pecofence_platform::locale::ui_language(),
    ));
    let instance_name = pecofence_core::brand::var("PECOFENCE_INSTANCE").ok();
    args.instance = instance_name
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(str::to_string);
    let runtime = runtime::Runtime::detect(args.portable, args.instance.as_deref())
        .unwrap_or_else(|error| startup_failure(error));
    // Retain the shared mutex: two distributions must not both manage the desktop.
    // Acquire it before opening/truncating logs or loading configuration.
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

    let file = runtime
        .prepare()
        .unwrap_or_else(|error| startup_failure(error));
    init_logging(file);
    install_panic_hook();
    pecofence_platform::crashlog::install(
        runtime.paths.crash_dir.clone(),
        args.instance.as_deref().unwrap_or("main"),
    );
    tracing::info!(mode = runtime.distribution.mode().as_str(), root = %runtime.distribution.root().display(), "distribution resolved");
    let _ole = OleGuard::init()?;
    let cell = app::App::create(args, runtime)?;
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

fn startup_failure(error: String) -> ! {
    let message = pecofence_core::i18n::format("无法启动 PecoFence：{0}", &[error]);
    eprintln!("{message}");
    window::show_startup_error(&message, "PecoFence");
    std::process::exit(1);
}
