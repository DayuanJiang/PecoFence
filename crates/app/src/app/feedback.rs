//! The 「反馈」 section of the settings page's 「关于」 page: what it may send along with the user's text (system facts
//! and the shareable log, see `crate::share_log`) and the offer to report a crash on the next
//! start. The page itself posts to the feedback service; nothing here touches the network.

use super::*;

/// The feedback service (the Cloudflare Worker in `services/feedback`), which files each report
/// as an issue in a private repository. `PECOFENCE_FEEDBACK_URL` points a test instance
/// elsewhere (e.g. `wrangler dev`).
const FEEDBACK_URL: &str = "https://api.pecofence.jiang.jp/feedback";
/// How much of each shareable log goes with a report, at most: its first and last lines.
const LOG_EXCERPT_BYTES: usize = 24 * 1024;
/// The first lines of a longer log that go with it: the startup lines (monitors, config,
/// desktop generation, icon host), which the end of a long run no longer shows.
const LOG_HEAD_BYTES: usize = 4 * 1024;
/// The feedback service refuses a system fact longer than this (UTF-16 units).
const FACT_MAX: usize = 200;

/// Feedback-page state the app keeps between page loads.
#[derive(Default)]
pub(super) struct FeedbackState {
    /// The page to show once the freshly opened settings page is ready (`Some(crash)`).
    pub(super) focus: Option<bool>,
    /// The previous run ended in a panic or an unhandled exception (its shareable log says so).
    previous_run_crashed: bool,
    /// The crash balloon is up; clicking it opens the feedback form.
    crash_offer_pending: bool,
}

/// Whether a shareable log records a crash: the panic hook's `PANIC` or crashlog's line.
fn log_records_crash(log: &str) -> bool {
    log.lines().any(|line| {
        line.contains(": PANIC location=") || line.contains(": CRASH (unhandled exception)")
    })
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

/// A run's log within `max` bytes: all of it, or its first lines and its last lines with a
/// line saying how many were left out between them.
fn excerpt(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_string();
    }
    let head = head(text, LOG_HEAD_BYTES);
    // Room for the "left out" line.
    let tail = tail(&text[head.len()..], max.saturating_sub(head.len() + 64));
    let skipped = text[head.len()..text.len() - tail.len()].lines().count();
    format!("{head}--- {skipped} lines left out ---\n{tail}")
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

impl App {
    fn share_log(&self, suffix: &str) -> Option<String> {
        let dir = crate::log_dir()?;
        let name = pecofence_core::brand::log_file_name(self.instance.as_deref(), suffix);
        std::fs::read(dir.join(name))
            .ok()
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }

    /// At startup: if the last run crashed, offer to report it from a tray balloon.
    pub(super) fn offer_crash_feedback(&mut self) {
        let crashed = self
            .share_log(pecofence_core::brand::PREVIOUS_SHARE_LOG)
            .is_some_and(|log| log_records_crash(&log));
        self.feedback.previous_run_crashed = crashed;
        if !crashed {
            return;
        }
        tracing::info!("previous run crashed; offering to send feedback");
        if let Some(t) = &self.tray {
            t.show_info(
                "PecoFence",
                pecofence_core::i18n::text("PecoFence 上次意外关闭了。点击这里报告问题。"),
                true,
            );
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
    pub(super) fn feedback_info_json(&self) -> serde_json::Value {
        let install = if pecofence_platform::process::is_packaged() {
            "Microsoft Store"
        } else {
            "ZIP / winget"
        };
        let displays: Vec<String> = monitors::enumerate()
            .iter()
            .map(|m| {
                let b = m.bounds;
                format!(
                    "{}x{} {:.0}%{}",
                    b.right - b.left,
                    b.bottom - b.top,
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
            "showDesktop {:?}, zorder {:?}, hideIcons {}, quickHide {}, peek {} (dim {}, {}, active {}), autostart {}, iconSize {}",
            settings.show_desktop,
            settings.zorder,
            on_off(settings.hide_real_icons),
            on_off(settings.quick_hide.enabled),
            on_off(settings.peek.enabled),
            on_off(settings.peek.dim),
            settings.peek.chosen_hotkey(),
            self.peek_hotkey
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
                "{} fences: {} visible, {} cloaked, {} topmost, {} below the desktop, {} off screen; hidden by double-click {}, peek {}, show desktop {}; host {}",
                w.fences,
                w.visible,
                w.cloaked,
                w.topmost,
                w.below_host,
                w.off_screen,
                on_off(w.quick_hidden),
                on_off(w.peeking),
                on_off(w.show_desktop),
                w.host.map_or("none".to_string(), |k| format!("{k:?}")),
            ),
            None => "unknown".to_string(),
        };
        let elevated = match process::is_elevated(process::current_pid()) {
            Some(true) => "yes",
            Some(false) => "no",
            None => "unknown",
        };
        let startup = format!(
            "started {} after Explorer, elevated {elevated}, config in {}",
            seconds_after_explorer().map_or("?".to_string(), |s| format!("{s} s")),
            self.state.storage_kind()
        );
        let system = serde_json::json!({
            "app": format!("PecoFence {} ({install})", env!("CARGO_PKG_VERSION")),
            "windows": format!("build {}", pecofence_platform::desktop::os_build()),
            "displays": fact(displays.join(", ")),
            "settings": fact(behavior),
            "state": fact(windows),
            "startup": fact(startup),
            "language": pecofence_core::i18n::language().tag(),
            "theme": format!(
                "{:?}, {}",
                settings.theme_style,
                if self.theme_mode == ThemeMode::Dark { "dark" } else { "light" }
            ),
            "workspace": format!(
                "{} fences, {} items",
                self.state.fences().len(),
                self.state.workspace_item_count()
            ),
            "memory": memory,
            "gpu": fact(self.ctx.stack.adapter().unwrap_or_else(|| "unknown".to_string())),
        });
        let mut log = String::new();
        if self.feedback.previous_run_crashed
            && let Some(previous) = self.share_log(pecofence_core::brand::PREVIOUS_SHARE_LOG)
        {
            log.push_str("--- previous run (crashed) ---\n");
            log.push_str(&excerpt(&previous, LOG_EXCERPT_BYTES));
            log.push_str("\n--- this run ---\n");
        }
        if let Some(current) = self.share_log(pecofence_core::brand::SHARE_LOG) {
            log.push_str(&excerpt(&current, LOG_EXCERPT_BYTES));
        }
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
        })
    }

    /// `{"type": "action", "name": "feedback…"}` messages from the page.
    pub(super) fn on_feedback_action(&mut self, name: &str, v: &serde_json::Value) {
        match name {
            "feedbackInfo" => {
                if let Some(h) = &self.settings {
                    h.post_json(&self.feedback_info_json().to_string());
                }
            }
            "feedbackResult" => {
                let status = v.get("status").and_then(|s| s.as_u64()).unwrap_or(0);
                if (200..300).contains(&status) {
                    tracing::info!(status, "feedback sent");
                    // A crash report went out; the next one starts from this run.
                    self.feedback.previous_run_crashed = false;
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
        let cut = excerpt(&log, LOG_EXCERPT_BYTES);
        assert!(cut.len() <= LOG_EXCERPT_BYTES);
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

    #[test]
    fn facts_fit_the_service_limit() {
        assert_eq!(fact("short".to_string()), "short");
        let long = fact("x".repeat(500));
        assert_eq!(long.encode_utf16().count(), FACT_MAX);
        assert!(long.ends_with('…'));
    }
}
