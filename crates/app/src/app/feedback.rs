//! The 「反馈」 section of the settings page's 「关于」 page: what it may send along with the user's text (system facts
//! and the shareable log, see `crate::share_log`) and the offer to report a crash on the next
//! start. The page itself posts to the feedback service; nothing here touches the network.

use super::*;
use pecofence_core::config_store::ReadError;

/// The feedback service (the Cloudflare Worker in `services/feedback`), which files each report
/// as an issue in a private repository. `PECOFENCE_FEEDBACK_URL` points a test instance
/// elsewhere (e.g. `wrangler dev`).
const FEEDBACK_URL: &str = "https://api.pecofence.jiang.jp/feedback";
/// How much shareable log goes with a report, at most, all runs together (the service takes
/// 52 KB); a long message leaves less room, see [`log_budget`].
const LOG_MAX_BYTES: usize = 50 * 1024;
/// The service refuses a body over 64 KB (bytes); this leaves one free.
const BODY_MAX_BYTES: usize = 63 * 1024;
/// What a body holds besides the message and the log: 12 system facts of up to 200
/// characters, the e-mail address, the JSON keys, and the escapes the log's lines need.
const REPORT_OVERHEAD_BYTES: usize = 4 * 1024;
/// The first lines of a longer log that go with it: the startup lines (monitors, config,
/// desktop generation, icon host, settings), which the end of a long run no longer shows.
const LOG_HEAD_BYTES: usize = 8 * 1024;
/// At most this much of the stretch a long log leaves out is kept: its warnings, errors and
/// [`KEY_MESSAGES`] lines, the newest when they do not all fit.
const LOG_KEPT_BYTES: usize = 8 * 1024;
/// The previous run's log goes along too when this run's leaves at least this much room (this
/// run just started: the user may have restarted PecoFence after the trouble).
const PREVIOUS_RUN_MIN_BYTES: usize = 4 * 1024;
/// Room for the lines that mark where runs and left-out stretches begin.
const MARKER_BYTES: usize = 128;
/// The start of the messages kept from a left-out stretch besides warnings and errors: the
/// events reports most often turn on. `tests::key_messages_are_logged` pins them to the code.
const KEY_MESSAGES: &[&str] = &[
    "display changed",
    "fence placed on display change",
    "fence DPI changed",
    "fence WM_DPICHANGED",
    "fence rect saved",
    "fences covered",
    "session changed",
    "quitting",
    "work area changed",
    "icon host changed",
    "show desktop",
    "fences visibility toggled",
    "double-click on empty desktop",
    "windows desktop",
    "settings changed",
    "rules applied",
    "fence created",
    "fence deleted",
    "GPU device rebuilt",
    "TaskbarCreated",
    "system suspending",
    "system resumed",
    "session ending",
    "UI thread",
];
/// The feedback service refuses a system fact longer than this (UTF-16 units).
const FACT_MAX: usize = 200;

/// Feedback-page state the app keeps between page loads.
#[derive(Default)]
pub(super) struct FeedbackState {
    /// The page to show once the freshly opened settings page is ready (`Some(crash)`).
    pub(super) focus: Option<bool>,
    /// The previous run ended in a panic or an unhandled exception (its shareable log says so).
    previous_run_crashed: bool,
    /// The previous run's shareable log stops without its last line: it hung and was ended, or
    /// was ended from Task Manager. Its log goes with a report too.
    previous_run_unfinished: bool,
    /// The crash balloon is up; clicking it opens the feedback form.
    crash_offer_pending: bool,
}

/// Whether a shareable log records a crash: the panic hook's `PANIC` or crashlog's line.
fn log_records_crash(log: &str) -> bool {
    log.lines().any(|line| {
        line.contains(": PANIC location=") || line.contains(": CRASH (unhandled exception)")
    })
}

/// The module crashlog's line in a shareable log names as the faulting one
/// (`at=ExplorerPatcher.amd64.dll+0xc0c1`).
fn crash_module(log: &str) -> Option<&str> {
    let line = log
        .lines()
        .find(|line| line.contains(": CRASH (unhandled exception)"))?;
    let at = line
        .split_whitespace()
        .find_map(|w| w.strip_prefix("at="))?;
    at.split_once('+').map(|(module, _)| module)
}

/// ExplorerPatcher's DLL: registered as a shell extension, it loads into every program that
/// shows the shell's folders, PecoFence included (feedback #81).
fn is_explorer_patcher(module: &str) -> bool {
    module.to_ascii_lowercase().starts_with("explorerpatcher.")
}

/// Whether a shareable log ends as a normal exit does (`main` writes `process exiting` last;
/// at a sign-out Windows may end the process before that, after `session ending`), or cannot
/// tell: an empty log, one that stopped at its size limit, or one of a version before the hang
/// watch, which wrote no `process exiting` (the first start after an update from 0.2.3).
fn log_ends_normally(log: &str) -> bool {
    log.trim().is_empty()
        || !log.contains(": hang watch started")
        || [
            ": process exiting",
            ": session ending",
            "(size limit reached",
        ]
        .iter()
        .any(|line| log.contains(line))
}

/// The last whole lines of `text` within `max` bytes.
fn tail(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let mut start = text.len() - max;
    while !text.is_char_boundary(start) {
        start += 1;
    }
    let cut = &text[start..];
    cut.find('\n').map(|i| &cut[i + 1..]).unwrap_or(cut)
}

/// The first whole lines of `text` within `max` bytes.
fn head(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let mut end = max;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].rfind('\n').map(|i| &text[..=i]).unwrap_or("")
}

/// Whether a shareable log line (`time LEVEL target: message fields`) is one an excerpt keeps
/// from the stretch it leaves out.
fn is_key_line(line: &str) -> bool {
    let level = line.split_whitespace().nth(1).unwrap_or("");
    if level == "WARN" || level == "ERROR" {
        return true;
    }
    let message = line.split_once(": ").map_or("", |(_, m)| m);
    KEY_MESSAGES.iter().any(|k| message.starts_with(k))
}

/// The lines of `stretch` an excerpt keeps ([`is_key_line`]), each with the lines that go on
/// from it (a crash's stack, a repeat count), within `max` bytes: the newest when they do not
/// all fit. Returns them and how many lines they are.
fn key_lines(stretch: &str, max: usize) -> (String, usize) {
    let mut records: Vec<String> = Vec::new();
    let mut keeping = false;
    for line in stretch.split_inclusive('\n') {
        // A field written over several lines (a stack) goes on indented; a repeat count names
        // no message of its own.
        if line.starts_with(' ') || line.contains(": (the previous message repeated") {
            if keeping && let Some(last) = records.last_mut() {
                last.push_str(line);
            }
            continue;
        }
        keeping = is_key_line(line);
        if keeping {
            records.push(line.to_string());
        }
    }
    let mut kept = Vec::new();
    let mut bytes = 0;
    for record in records.into_iter().rev() {
        if bytes + record.len() > max {
            break;
        }
        bytes += record.len();
        kept.push(record);
    }
    kept.reverse();
    let text = kept.concat();
    let lines = text.lines().count();
    (text, lines)
}

/// A run's log within `max` bytes: all of it, or its first lines and its last lines, and
/// between them a line saying how many were left out, followed by the warnings, errors and
/// key events among those.
fn excerpt(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_string();
    }
    let head = head(text, LOG_HEAD_BYTES.min(max / 4));
    let rest = &text[head.len()..];
    let room = max.saturating_sub(head.len() + 2 * MARKER_BYTES);
    // The key lines of what the longest tail leaves out take that much room from the tail (up
    // to their share); the shorter tail leaves more out, whose newest key lines may take it.
    let longest = tail(rest, room);
    let share = LOG_KEPT_BYTES.min(room / 4);
    let (first, _) = key_lines(&rest[..rest.len() - longest.len()], share);
    let tail = tail(rest, room - first.len());
    let stretch = &rest[..rest.len() - tail.len()];
    let (kept, kept_lines) = key_lines(stretch, first.len());
    let left = stretch.lines().count() - kept_lines;
    if kept.is_empty() {
        return format!("{head}--- {left} lines left out ---\n{tail}");
    }
    format!(
        "{head}--- {left} lines left out; the {kept_lines} warning, error and key event lines among them follow ---\n{kept}--- end of the lines kept from that stretch ---\n{tail}"
    )
}

/// The room for the log in a report whose message is `message_bytes` long (UTF-8): all of
/// [`LOG_MAX_BYTES`] for the usual short message, less for a long one, so that a 5000-character
/// Chinese message (15 KB) still fits the service's body limit.
fn log_budget(message_bytes: usize) -> usize {
    BODY_MAX_BYTES
        .saturating_sub(message_bytes + REPORT_OVERHEAD_BYTES)
        .min(LOG_MAX_BYTES)
}

/// The log that goes with a report, within `budget` bytes: this run's, after the previous
/// run's when that one crashed or did not end normally (`how`), or when this run's leaves room.
fn report_log(previous: Option<&str>, how: Option<&str>, current: &str, budget: usize) -> String {
    let previous = previous.filter(|p| !p.trim().is_empty());
    let (previous, current) = match (previous, how) {
        (Some(previous), Some(_)) => {
            // The run that went wrong first; this one gets at most half.
            let mine = current.len().min(budget / 2);
            let previous = excerpt(previous, budget.saturating_sub(mine + 2 * MARKER_BYTES));
            let room = budget.saturating_sub(previous.len() + 2 * MARKER_BYTES);
            (Some(previous), excerpt(current, room))
        }
        (Some(previous), None) => {
            let current = excerpt(current, budget.saturating_sub(2 * MARKER_BYTES));
            let room = budget.saturating_sub(current.len() + 2 * MARKER_BYTES);
            let previous = (room >= PREVIOUS_RUN_MIN_BYTES).then(|| excerpt(previous, room));
            (previous, current)
        }
        (None, _) => (None, excerpt(current, budget)),
    };
    let Some(previous) = previous else {
        return current;
    };
    let how = how.map_or(String::new(), |how| format!(" ({how})"));
    format!("--- previous run{how} ---\n{previous}\n--- this run ---\n{current}")
}

/// A system fact cut to what the feedback service accepts.
fn fact(text: String) -> String {
    if text.encode_utf16().count() <= FACT_MAX {
        return text;
    }
    let mut cut: String = text.chars().take(FACT_MAX - 1).collect();
    while cut.encode_utf16().count() > FACT_MAX - 1 {
        cut.pop();
    }
    cut.push('…');
    cut
}

fn on_off(on: bool) -> &'static str {
    if on { "on" } else { "off" }
}

fn yes_no(yes: bool) -> &'static str {
    if yes { "yes" } else { "no" }
}

/// The drive a path is on (`D: (removable)`), or the kind of place without a letter: a report
/// needs to know whether the program or its configuration sit on another drive (os error 17).
fn drive_of(path: &std::path::Path) -> String {
    let kind = shell::drive_kind(path);
    let text = path.to_string_lossy();
    let text = text.strip_prefix(r"\\?\").unwrap_or(&text);
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(letter), Some(':')) if letter.is_ascii_alphabetic() => {
            format!("{}: ({kind})", letter.to_ascii_uppercase())
        }
        _ => kind.to_string(),
    }
}

/// How this copy was installed: the Store, Setup (which leaves its uninstaller next to the
/// program; winget runs the same Setup) or a ZIP; `portable` runs keep their configuration
/// next to the program.
pub(super) fn install_kind(portable: bool) -> String {
    let kind = if pecofence_platform::process::is_packaged() {
        "Microsoft Store"
    } else if std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("unins000.exe").exists()))
        .unwrap_or(false)
    {
        "installer"
    } else {
        "ZIP"
    };
    if portable {
        format!("{kind}, portable")
    } else {
        kind.to_string()
    }
}

impl App {
    fn share_log(&self, suffix: &str) -> Option<String> {
        self.read_share_log(suffix).ok()
    }

    /// A shareable log's text; a file that exists but cannot be read is logged.
    fn read_share_log(&self, suffix: &str) -> std::io::Result<String> {
        let dir = crate::log_dir().ok_or(std::io::ErrorKind::NotFound)?;
        let name = pecofence_core::brand::log_file_name(self.instance.as_deref(), suffix);
        let read = std::fs::read(dir.join(name));
        if let Err(e) = &read
            && e.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(error = %e, "shareable log unreadable");
        }
        read.map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }

    /// At startup: if the last run crashed, offer to report it from a tray balloon.
    pub(super) fn offer_crash_feedback(&mut self) {
        let previous = self.share_log(pecofence_core::brand::PREVIOUS_SHARE_LOG);
        let crashed = previous.as_deref().is_some_and(log_records_crash);
        self.feedback.previous_run_crashed = crashed;
        self.feedback.previous_run_unfinished = !crashed
            && previous
                .as_deref()
                .is_some_and(|log| !log_ends_normally(log));
        if self.feedback.previous_run_unfinished {
            tracing::info!("previous run did not end normally");
        }
        if !crashed {
            return;
        }
        tracing::info!("previous run crashed; offering to send feedback");
        let in_explorer_patcher = previous
            .as_deref()
            .and_then(crash_module)
            .is_some_and(is_explorer_patcher);
        let text = if in_explorer_patcher {
            tracing::info!("the crash was inside ExplorerPatcher");
            pecofence_core::i18n::text(
                "上次因 ExplorerPatcher 出错而关闭。请在它的 Properties › File Explorer 中取消勾选“Register as shell extension”，然后重启资源管理器。",
            )
        } else {
            pecofence_core::i18n::text("PecoFence 上次意外关闭了。点击这里报告问题。")
        };
        if let Some(t) = &self.tray {
            t.show_info("PecoFence", text, true);
            self.feedback.crash_offer_pending = true;
        }
    }

    /// A tray balloon was clicked; only the crash offer reacts to it.
    pub(super) fn on_balloon_click(&mut self) {
        if std::mem::take(&mut self.feedback.crash_offer_pending) {
            self.open_feedback(true);
        }
    }

    /// The balloon went away unclicked (or another one replaced it).
    pub(super) fn on_balloon_gone(&mut self) {
        self.feedback.crash_offer_pending = false;
    }

    /// Opens the settings window at the 「反馈」 form (on 「关于」); `crash` says it reports the last crash.
    pub(super) fn open_feedback(&mut self, crash: bool) {
        self.open_settings();
        let Some(host) = &self.settings else {
            return;
        };
        if host.ready {
            self.post_show_feedback(crash);
        } else {
            // The page is still loading; `ready` delivers the state and then the latest
            // request: this form, or a fence's options.
            self.feedback.focus = Some(crash);
            self.settings_focus_fence = None;
        }
    }

    /// `ready` from a freshly loaded page: show the feedback form if that is why it opened.
    pub(super) fn show_pending_feedback(&mut self) {
        if let Some(crash) = self.feedback.focus.take() {
            self.post_show_feedback(crash);
        }
    }

    fn post_show_feedback(&self, crash: bool) {
        if let Some(h) = &self.settings {
            h.post_json(&serde_json::json!({ "type": "showFeedback", "crash": crash }).to_string());
        }
    }

    /// What the page may send besides the user's own text, shown to them before sending.
    /// `message_bytes`: how long the user's message is now (UTF-8), which the log makes room for.
    pub(super) fn feedback_info_json(&self, message_bytes: usize) -> serde_json::Value {
        let install = install_kind(self.portable);
        let displays: Vec<String> = monitors::enumerate()
            .iter()
            .map(|m| {
                let b = m.bounds;
                format!(
                    "{}x{} @{},{} {:.0}%{}",
                    b.right - b.left,
                    b.bottom - b.top,
                    b.left,
                    b.top,
                    m.scale() * 100.0,
                    if m.primary { " primary" } else { "" }
                )
            })
            .collect();
        let settings = &self.state.config.settings;
        // Task Manager's number first, then where else memory sits: private commit counts
        // pages Windows trimmed, GPU memory the surfaces and textures.
        const MB: f64 = 1024.0 * 1024.0;
        let mem = pecofence_platform::memstats::MemoryStats::current().unwrap_or_default();
        let gpu = match self.ctx.stack.gpu_memory() {
            Some((local, shared)) => format!(
                "GPU {:.1} MB dedicated, {:.1} MB shared",
                local as f64 / MB,
                shared as f64 / MB,
            ),
            None => "GPU unknown".to_string(),
        };
        let memory = format!(
            "{:.1} MB (commit {:.1} MB, {gpu})",
            mem.private_working_set as f64 / MB,
            mem.private_commit as f64 / MB,
        );
        // The settings that change how the fences sit on the desktop.
        let behavior = format!(
            "showDesktop {:?}, zorder {:?}, hideIcons {}, quickHide {}, peek {} (dim {}, {}, active {}), windowsDesktop hotkey {}, autostart {}, iconSize {}",
            settings.show_desktop,
            settings.zorder,
            on_off(settings.hide_real_icons),
            on_off(settings.quick_hide.enabled),
            on_off(settings.peek.enabled),
            on_off(settings.peek.dim),
            settings.peek.chosen_hotkey(),
            self.peek_hotkey
                .map_or_else(|| "none".to_string(), |h| h.to_string()),
            settings
                .windows_desktop_hotkey()
                .map_or_else(|| "none".to_string(), |h| h.to_string()),
            on_off(settings.autostart),
            settings.icon_size,
        );
        // The fence windows as they are while the user writes the report.
        let windows = match self
            .anchor
            .try_borrow()
            .ok()
            .and_then(|a| a.as_ref().map(|a| a.window_report()))
        {
            Some(w) => format!(
                "{} fences: {} visible, {} cloaked, {} topmost, {} below the desktop, {} above other windows, {} off screen; hidden by double-click {}, peek {}, show desktop {}, windows desktop {}; host {}",
                w.fences,
                w.visible,
                w.cloaked,
                w.topmost,
                w.below_host,
                w.above_windows,
                w.off_screen,
                on_off(w.quick_hidden),
                on_off(w.peeking),
                on_off(w.show_desktop),
                on_off(self.windows_desktop),
                w.host.map_or("none".to_string(), |k| format!("{k:?}")),
            ),
            None => "unknown".to_string(),
        };
        let elevated = match process::is_elevated(process::current_pid()) {
            Some(true) => "yes",
            Some(false) => "no",
            None => "unknown",
        };
        let mut startup = format!(
            "started {} after Explorer, elevated {elevated}, config in {}, app on {}, config on {}",
            seconds_after_explorer().map_or("?".to_string(), |s| format!("{s} s")),
            self.state.storage_kind(),
            std::env::current_exe().map_or("?".to_string(), |exe| drive_of(&exe)),
            drive_of(&self.state.config_path()),
        );
        // Files the start could not use, by their OS error (the log may be unreadable too).
        for (what, _, error) in &self.state.load_failures {
            let reason = match error {
                ReadError::Io(e) => e
                    .raw_os_error()
                    .map_or_else(|| format!("{:?}", e.kind()), |c| format!("os error {c}")),
                ReadError::Invalid(_) => "invalid".to_string(),
            };
            startup.push_str(&format!(", {what} unusable ({reason})"));
        }
        // This run so far: what the questions after a report were about.
        let mut session = format!(
            "run {} min, remote desktop {}",
            self.started.elapsed().as_secs() / 60,
            yes_no(sysparams::remote_session()),
        );
        match self.live.first() {
            Some(live) => session.push_str(&format!(
                ", live wallpaper on {} of {} monitors ({})",
                self.live.len(),
                self.state.work_areas.len(),
                live.class()
            )),
            None => session.push_str(", no live wallpaper"),
        }
        match &self.last_display_change {
            Some((at, d)) => session.push_str(&format!(
                ", last display change {} min ago: {} monitors, layout {}, {} fences placed, {} moved",
                at.elapsed().as_secs() / 60,
                d.monitors,
                d.layout,
                d.placed,
                d.moved
            )),
            None => session.push_str(", no display change"),
        }
        // The service takes 12 facts at most: this is the 12th.
        let system = serde_json::json!({
            "app": format!("PecoFence {} ({install})", env!("CARGO_PKG_VERSION")),
            "windows": format!("build {}", pecofence_platform::desktop::os_build()),
            "displays": fact(displays.join(", ")),
            "settings": fact(behavior),
            "state": fact(windows),
            "startup": fact(startup),
            "language": pecofence_core::i18n::language().tag(),
            "theme": format!(
                "{}{}, {}",
                if self.ctx.theme.borrow().liquid_glass { "LiquidGlass" } else { "Fluent" },
                match (&settings.custom_theme, &self.custom_theme_error) {
                    (None, _) => "",
                    (Some(_), None) => " (custom theme)",
                    (Some(_), Some(_)) => " (custom theme not loaded)",
                },
                if self.theme_mode == ThemeMode::Dark { "dark" } else { "light" }
            ),
            "workspace": format!(
                "{} fences, {} items",
                self.state.fences().len(),
                self.state.workspace_item_count()
            ),
            "memory": memory,
            "gpu": fact(self.ctx.stack.adapter().unwrap_or_else(|| "unknown".to_string())),
            "session": fact(session),
        });
        let previous_run = if self.feedback.previous_run_crashed {
            Some("crashed")
        } else if self.feedback.previous_run_unfinished {
            Some("did not end normally")
        } else {
            None
        };
        let previous = self.share_log(pecofence_core::brand::PREVIOUS_SHARE_LOG);
        let current = self
            .read_share_log(pecofence_core::brand::SHARE_LOG)
            // The report still says why it has no log.
            .unwrap_or_else(|e| format!("--- the log could not be read: {e} ---\n"));
        let log = report_log(
            previous.as_deref(),
            previous_run,
            &current,
            log_budget(message_bytes),
        );
        let url = pecofence_core::brand::var("PECOFENCE_FEEDBACK_URL")
            .ok()
            .filter(|u| u.starts_with("https://") || u.starts_with("http://127.0.0.1:"))
            .unwrap_or_else(|| FEEDBACK_URL.to_string());
        serde_json::json!({
            "type": "feedbackInfo",
            "url": url,
            "app": env!("CARGO_PKG_VERSION"),
            "language": pecofence_core::i18n::language().tag(),
            "crashed": self.feedback.previous_run_crashed,
            "system": system,
            "log": log,
            // The page asks again when the message has grown since (see `log_budget`).
            "messageBytes": message_bytes,
        })
    }

    /// `{"type": "action", "name": "feedback…"}` messages from the page.
    pub(super) fn on_feedback_action(&mut self, name: &str, v: &serde_json::Value) {
        match name {
            "feedbackInfo" => {
                let message_bytes =
                    v.get("messageBytes").and_then(|b| b.as_u64()).unwrap_or(0) as usize;
                if let Some(h) = &self.settings {
                    h.post_json(&self.feedback_info_json(message_bytes).to_string());
                }
            }
            "feedbackResult" => {
                let status = v.get("status").and_then(|s| s.as_u64()).unwrap_or(0);
                if (200..300).contains(&status) {
                    tracing::info!(status, "feedback sent");
                    // A crash report went out; the next one starts from this run.
                    self.feedback.previous_run_crashed = false;
                    self.feedback.previous_run_unfinished = false;
                } else {
                    tracing::warn!(status, "feedback not sent");
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crash_lines_are_recognised() {
        assert!(log_records_crash(
            "2026-10-04T12:00:00Z ERROR pecofence: PANIC location=crates/app/src/x.rs:1:2 payload=boom\n"
        ));
        assert!(log_records_crash(
            "t ERROR pecofence_platform::crashlog: CRASH (unhandled exception) code=0xc0000005\n"
        ));
        assert!(!log_records_crash(
            "t  INFO pecofence::app: config loaded fences=7\n"
        ));
    }

    #[test]
    fn the_faulting_module_is_read_from_the_crash_line() {
        // As feedback #81's log has it: the stack follows on the next lines.
        let log = "t  INFO pecofence::app: process started\n\
            t ERROR pecofence_platform::crashlog: CRASH (unhandled exception) code=0xc0000005 at=ExplorerPatcher.amd64.dll+0xc0c1 module_version=26100.8457.70.3 access=write stack=  ExplorerPatcher.amd64.dll+0xc0c1\n  combase.dll+0x1c6e85\n";
        assert_eq!(crash_module(log), Some("ExplorerPatcher.amd64.dll"));
        assert!(is_explorer_patcher("ExplorerPatcher.amd64.dll"));
        assert!(is_explorer_patcher("explorerpatcher.IA-32.dll"));
        let ours = "t ERROR pecofence_platform::crashlog: CRASH (unhandled exception) code=0xc0000005 at=pecofence.exe+0x1f368 stack=  ExplorerPatcher.amd64.dll+0x19b37\n";
        assert!(!crash_module(ours).is_some_and(is_explorer_patcher));
        // A panic names no module.
        assert_eq!(
            crash_module("t ERROR pecofence: PANIC location=crates/app/src/x.rs:1:2\n"),
            None
        );
    }

    #[test]
    fn unfinished_runs_are_recognised() {
        assert!(log_ends_normally(""));
        assert!(log_ends_normally(
            "t  INFO pecofence::app: process started\nt  INFO pecofence: process exiting code=0\n"
        ));
        assert!(log_ends_normally(
            "t  INFO pecofence::app: process started\nt  INFO pecofence::app: session ending: quitting close_app=false\n"
        ));
        assert!(log_ends_normally(
            "t  INFO pecofence::app: process started\n(size limit reached: only errors are recorded from here on)\n"
        ));
        assert!(!log_ends_normally(
            "t  INFO pecofence::app: process started\nt  INFO pecofence_platform::hangwatch: hang watch started\nt  WARN pecofence_platform::hangwatch: UI thread not responding ms=5000\n"
        ));
        // 0.2.3 and earlier wrote no `process exiting`: a log of theirs cannot tell.
        assert!(log_ends_normally(
            "t  INFO pecofence::app: process started\nt  INFO pecofence::app: tray exit\n"
        ));
    }

    #[test]
    fn tails_start_on_a_whole_line() {
        let log = "first line\nsecond line\nthird\n";
        assert_eq!(tail(log, 100), log);
        assert_eq!(tail(log, 14), "third\n");
        assert_eq!(tail("一二三\n四五\n", 8), "四五\n");
    }

    #[test]
    fn excerpts_keep_the_start_and_the_end() {
        let short = "a\nb\n";
        assert_eq!(excerpt(short, 100), short);
        let log: String = (0..10_000).map(|i| format!("line {i:05}\n")).collect();
        let cut = excerpt(&log, LOG_MAX_BYTES);
        assert!(cut.len() <= LOG_MAX_BYTES);
        assert!(cut.starts_with("line 00000\n"));
        assert!(cut.ends_with("line 09999\n"));
        let head_lines = head(&log, LOG_HEAD_BYTES).lines().count();
        let marker = cut.lines().nth(head_lines).unwrap();
        let skipped: usize = marker
            .trim_start_matches("--- ")
            .trim_end_matches(" lines left out ---")
            .parse()
            .unwrap();
        assert_eq!(cut.lines().count() - 1 + skipped, 10_000);
    }

    /// A shareable log line as the share layer writes it.
    fn line(i: usize, level: &str, message: &str) -> String {
        format!(
            "2026-11-02T10:{:02}:{:02}.{i:06}Z {level:>5} pecofence::app: {message}\n",
            i / 60 % 60,
            i % 60
        )
    }

    #[test]
    fn excerpts_keep_the_warnings_and_key_events_they_leave_out() {
        let mut log = String::new();
        for i in 0..4000 {
            let text = match i {
                1500 => line(i, "WARN", "anchor SetWindowPos failed error=0x80070005"),
                1501 => line(i, "WARN", "anchor SetWindowPos failed error=0x80070005"),
                1502 => line(i, "WARN", "(the previous message repeated 7 more times)"),
                1800 => line(
                    i,
                    "INFO",
                    "display changed monitors=2 new_setup=true layout=1",
                ),
                2000 => format!(
                    "{}  combase.dll+0x1c6e85\n  pecofence.exe+0x1f368\n",
                    line(
                        i,
                        "ERROR",
                        "CRASH (unhandled exception) code=0xc0000005 stack=  ntdll.dll+0x1"
                    )
                ),
                _ => line(i, "INFO", "slow command ms=60.5 command=RaiseFence"),
            };
            log.push_str(&text);
        }
        let cut = excerpt(&log, LOG_MAX_BYTES);
        assert!(cut.len() <= LOG_MAX_BYTES, "{}", cut.len());
        assert!(cut.starts_with(&line(0, "INFO", "slow command ms=60.5 command=RaiseFence")));
        assert!(cut.ends_with(&line(
            3999,
            "INFO",
            "slow command ms=60.5 command=RaiseFence"
        )));
        let kept = cut
            .split_once("among them follow ---\n")
            .and_then(|(_, rest)| rest.split_once("--- end of the lines kept"))
            .map(|(kept, _)| kept)
            .unwrap();
        let expected = [1500, 1501, 1502, 1800]
            .iter()
            .map(|&i| log.lines().nth(i).unwrap())
            .chain(log.lines().skip(2000).take(3))
            .collect::<Vec<_>>();
        assert_eq!(kept.lines().collect::<Vec<_>>(), expected);
        // Every line of the log is shown or counted.
        let marker = cut
            .lines()
            .find(|l| l.contains(" lines left out; "))
            .unwrap();
        let left: usize = marker
            .trim_start_matches("--- ")
            .split(' ')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(cut.lines().count() - 2 + left, log.lines().count());
    }

    #[test]
    fn a_full_shareable_log_keeps_its_key_events() {
        // A run's log at its 2 MB limit, of lines as feedback #96 had them, with a display
        // change in the middle.
        let filler = [
            "INFO pecofence::app: slow command ms=62.5 command=FenceMenu",
            "INFO pecofence::app::motion: animation run stuttered frames=27 total_ms=505.7 max_gap_ms=188.4 max_frame_ms=2",
            "INFO pecofence::app::sync: desktop resynced report=SyncReport { added: 0, removed: 0, updated: 1, by_rule: 0, by_default: 0, unrouted: 0, unread: 0 }",
            "INFO pecofence::app::visuals: wallpaper refreshed reason=poll cache_hit=false elapsed_ms=135 last_setting_change_ms=None last_desktop_key_ms=None last_desktops_key_ms=None",
        ];
        let mut log = String::new();
        let mut i = 0;
        while log.len() < 2 * 1024 * 1024 {
            let text = if i == 9000 {
                "INFO pecofence::app::fences: display changed monitors=2 new_setup=true layout=2 from_layout=1 fences_to_place=5".to_string()
            } else if (9001..9006).contains(&i) {
                format!(
                    "INFO pecofence::app::fences: fence placed on display change id=c40efadc-e13d-4d4a-aabc-90153a2b968{} from=Some((0, 0, 320, 400)) want=(1920, 0, 2240, 400) to=(1920, 0, 2320, 500) rolled=false monitor=Some(1) dpi=Some(120) reason=saved for these monitors",
                    i - 9001
                )
            } else {
                filler[i % filler.len()].to_string()
            };
            log.push_str(&format!(
                "2026-11-01T15:{:02}:{:02}.{i:06}Z {text}\n",
                i / 60 % 60,
                i % 60
            ));
            i += 1;
        }
        let started = std::time::Instant::now();
        let cut = excerpt(&log, LOG_MAX_BYTES);
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
        assert!(cut.len() <= LOG_MAX_BYTES);
        assert!(cut.starts_with(log.lines().next().unwrap()));
        assert!(cut.ends_with(&format!("{}\n", log.lines().last().unwrap())));
        assert!(cut.contains(": display changed monitors=2"));
        assert_eq!(cut.matches(": fence placed on display change").count(), 5);
    }

    #[test]
    fn too_many_key_lines_keep_the_newest() {
        let log: String = (0..6000)
            .map(|i| line(i, "WARN", "hiding desktop icons failed"))
            .collect();
        let cut = excerpt(&log, LOG_MAX_BYTES);
        assert!(cut.len() <= LOG_MAX_BYTES);
        let kept = cut
            .split_once("among them follow ---\n")
            .and_then(|(_, rest)| rest.split_once("--- end of the lines kept"))
            .map(|(kept, _)| kept)
            .unwrap();
        assert!(kept.len() <= LOG_KEPT_BYTES);
        // The newest of the left-out stretch: right before the tail.
        let last_kept = kept.lines().last().unwrap();
        let tail_first = cut
            .split_once("--- end of the lines kept from that stretch ---\n")
            .unwrap()
            .1
            .lines()
            .next()
            .unwrap();
        let at = |l: &str| log.lines().position(|x| x == l).unwrap();
        assert_eq!(at(last_kept) + 1, at(tail_first));
    }

    #[test]
    fn key_messages_are_logged() {
        let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut sources = String::new();
        let mut dirs = vec![crates];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() && path.file_name().is_some_and(|n| n != "target") {
                    dirs.push(path);
                } else if path.extension().is_some_and(|e| e == "rs")
                    // The list itself is no log message.
                    && path.file_name().is_some_and(|n| n != "feedback.rs")
                {
                    sources.push_str(&std::fs::read_to_string(&path).unwrap());
                }
            }
        }
        for key in KEY_MESSAGES {
            assert!(
                sources.contains(&format!("\"{key}")),
                "no log message starts with {key:?}"
            );
        }
    }

    #[test]
    fn reports_bring_the_previous_run_along_when_it_matters_or_there_is_room() {
        let long = |tag: &str| -> String {
            (0..20_000)
                .map(|i| line(i, "INFO", &format!("{tag} {i}")))
                .collect()
        };
        let (old, now) = (long("old"), long("now"));
        // Crashed: both go, each within the budget together.
        let log = report_log(Some(&old), Some("crashed"), &now, LOG_MAX_BYTES);
        assert!(log.len() <= LOG_MAX_BYTES, "{}", log.len());
        assert!(log.starts_with("--- previous run (crashed) ---\n"));
        assert!(log.contains("\n--- this run ---\n"));
        assert!(log.contains(": old 19999\n") && log.ends_with(": now 19999\n"));
        // A normal previous run goes only into the room this run leaves.
        assert!(!report_log(Some(&old), None, &now, LOG_MAX_BYTES).contains("previous run"));
        let started = line(1, "INFO", "process started");
        let log = report_log(Some(&old), None, &started, LOG_MAX_BYTES);
        assert!(log.len() <= LOG_MAX_BYTES);
        assert!(log.starts_with("--- previous run ---\n"));
        assert!(log.ends_with(&format!("--- this run ---\n{started}")));
        // No previous run, or an empty one.
        assert_eq!(report_log(None, None, &started, LOG_MAX_BYTES), started);
        assert_eq!(
            report_log(Some(" \n"), Some("crashed"), &started, LOG_MAX_BYTES),
            started
        );
    }

    #[test]
    fn a_full_report_fits_the_feedback_service() {
        // services/feedback: Content-Length (bytes) at most 64 KB, log at most 52 KB (UTF-16).
        const MAX_BODY: usize = 64 * 1024;
        const MAX_LOG: usize = 52 * 1024;
        let old: String = (0..30_000)
            .map(|i| line(i, "WARN", "anchor SetWindowPos failed"))
            .collect();
        // The longest message the page takes: 5000 characters, 3 bytes each in UTF-8.
        let message = "错".repeat(5000);
        assert_eq!(log_budget(0), LOG_MAX_BYTES);
        let log = report_log(Some(&old), Some("crashed"), &old, log_budget(message.len()));
        assert!(log.encode_utf16().count() <= MAX_LOG);
        assert!(log.len() > 40 * 1024, "{}", log.len());
        let system: serde_json::Map<String, serde_json::Value> = (0..12)
            .map(|i| {
                (
                    format!("fact{i:02}"),
                    serde_json::Value::String("x".repeat(FACT_MAX)),
                )
            })
            .collect();
        let report = serde_json::json!({
            "kind": "bug",
            "message": message,
            "email": format!("{}@example.com", "a".repeat(242)),
            "app": "0.2.4",
            "crash": true,
            "language": "zh-CN",
            "system": system,
            "log": log,
        });
        let body = report.to_string();
        assert!(body.len() <= MAX_BODY, "{} bytes", body.len());
    }

    #[test]
    fn facts_fit_the_service_limit() {
        assert_eq!(fact("short".to_string()), "short");
        let long = fact("x".repeat(500));
        assert_eq!(long.encode_utf16().count(), FACT_MAX);
        assert!(long.ends_with('…'));
    }
}
