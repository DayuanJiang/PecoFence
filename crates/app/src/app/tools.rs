//! 小工具: fences that show a tool in place of desktop items. 便签与待办 shows lines of text (a
//! line starting with `[ ]` is a to-do whose check box ticks with a click); 时钟与日期 shows the
//! time and date.

use super::*;
use crate::fence_window::ScriptView;
use crate::scripts::Outcome;
use pecofence_core::{FenceTool, ScriptSpec};
use std::collections::HashMap;
use std::time::Instant;

/// 脚本栅栏 at run time (nothing of it is saved): each fence's last result and when it came,
/// the fences whose script is running (with the script it runs), and when each is next due.
pub(super) struct ScriptRuns {
    last: HashMap<FenceId, (Outcome, i64)>,
    running: HashMap<FenceId, String>,
    due: HashMap<FenceId, Instant>,
    tx: std::sync::mpsc::Sender<(FenceId, Outcome, i64, u128)>,
    rx: std::sync::mpsc::Receiver<(FenceId, Outcome, i64, u128)>,
}

impl Default for ScriptRuns {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            last: HashMap::new(),
            running: HashMap::new(),
            due: HashMap::new(),
            tx,
            rx,
        }
    }
}

/// A tool fence to make and what it starts with.
pub(super) enum NewTool {
    /// A note with this text.
    Note(String),
    Clock,
    /// A script fence running this file.
    Script(String),
}

impl NewTool {
    fn tool(&self) -> FenceTool {
        match self {
            Self::Note(_) => FenceTool::Note,
            Self::Clock => FenceTool::Clock,
            Self::Script(_) => FenceTool::Script,
        }
    }
}

impl App {
    /// 对话框跳转 and 双击 Ctrl 打开搜索 follow their switches.
    pub(super) fn sync_dialog_jump(&self) {
        let settings = &self.state.config.settings;
        crate::dialog_jump::set_enabled(settings.tools.dialog_jump, self.control.hwnd());
        crate::double_ctrl::set_enabled(settings.peek.double_ctrl, self.control.hwnd());
    }

    /// Makes a tool fence at `rect` (physical px; None = a free spot near (`x`, `y`)), titled
    /// `title` or after the tool, switching the tool on first. None when the fence limit is
    /// reached or no monitor is connected.
    pub(super) fn create_tool_fence(
        &mut self,
        tool: NewTool,
        title: Option<&str>,
        rect: Option<RECT>,
        x: i32,
        y: i32,
    ) -> Option<FenceId> {
        let kind = tool.tool();
        if !self.fence_room_or_notice() {
            return None;
        }
        self.ensure_tool_on(kind);
        let (w, h) = match kind {
            FenceTool::Note => (280.0, 240.0),
            FenceTool::Clock => (240.0, 140.0),
            FenceTool::Script => (280.0, 160.0),
        };
        let rect = rect.unwrap_or_else(|| self.place_new_fence_dip(w, h, x, y, None));
        let default_title = match &tool {
            NewTool::Note(_) => pecofence_core::i18n::text("便签").to_string(),
            NewTool::Clock => pecofence_core::i18n::text("时钟").to_string(),
            NewTool::Script(path) => std::path::Path::new(path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| pecofence_core::i18n::text("脚本").to_string()),
        };
        let title = title
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_string)
            .unwrap_or(default_title);
        let id = match &tool {
            NewTool::Note(text) => self.state.new_note(&title, rect, text)?,
            _ => self.state.new_tool_fence(&title, rect, kind)?,
        };
        if let NewTool::Script(path) = &tool {
            self.state.config.scripts.insert(id, ScriptSpec::new(path));
            self.state.mark_dirty();
        }
        self.resync_windows();
        if let Some(w) = self.fences.get(&id) {
            w.show(true);
        }
        self.schedule_save();
        if kind == FenceTool::Script {
            self.start_script(id);
        }
        Some(id)
    }

    /// 脚本栅栏 for `fence get` / `fence list`: the script and its last run.
    pub(super) fn script_dto(&self, id: FenceId) -> Option<pecofence_ipc::ScriptDto> {
        let spec = self.state.config.scripts.get(&id)?;
        let last = self.script_runs.last.get(&id);
        let outcome = last.map(|(o, _)| match o {
            Outcome::Done(_) => "done",
            Outcome::Failed { .. } => "failed",
            Outcome::TimedOut => "timedOut",
            Outcome::Missing => "missing",
            Outcome::Unsupported => "unsupported",
            Outcome::NotStarted(_) => "notStarted",
        });
        let output = last.and_then(|(o, _)| match o {
            Outcome::Done(t) | Outcome::Failed { text: t, .. } => Some(t.clone()),
            _ => None,
        });
        Some(pecofence_ipc::ScriptDto {
            path: spec.path.clone(),
            interval_minutes: spec.interval_minutes,
            outcome: outcome.map(str::to_string),
            output,
            last_run: last.map(|(_, at)| *at),
            running: self.script_runs.running.contains_key(&id),
        })
    }

    /// Switches `tool` on in the settings when it is off (a new fence of it would have no
    /// window otherwise).
    fn ensure_tool_on(&mut self, tool: FenceTool) {
        if tool.enabled(&self.state.config.settings.tools) {
            return;
        }
        let mut settings = self.state.config.settings.clone();
        match tool {
            FenceTool::Note => settings.tools.notes = true,
            FenceTool::Clock => settings.tools.clock = true,
            FenceTool::Script => settings.tools.scripts = true,
        }
        self.apply_settings(settings, "new tool fence");
    }

    /// A clock fence near (`x`, `y`).
    pub(super) fn new_clock_near(&mut self, x: i32, y: i32) {
        self.create_tool_fence(NewTool::Clock, None, None, x, y);
    }

    /// Pushes what each window's shown fence needs of its tool (note text, clock seconds),
    /// after the tool settings changed.
    pub(super) fn refresh_tool_views(&mut self) {
        let hosts: Vec<FenceId> = self.fences.keys().copied().collect();
        for host in hosts {
            self.push_tool_view(host);
        }
    }

    /// Note text / clock for the fence `host`'s window shows.
    pub(super) fn push_tool_view(&self, host: FenceId) {
        let Some(w) = self.fences.get(&host) else {
            return;
        };
        let shown = w.active_fence();
        let tool = self.state.fence(shown).and_then(|f| f.tool);
        w.set_note(self.state.note_text(shown).map(str::to_string));
        let seconds = self.state.config.settings.tools.clock_seconds;
        w.set_clock((tool == Some(FenceTool::Clock)).then_some(seconds));
        w.set_script((tool == Some(FenceTool::Script)).then(|| self.script_view(shown)));
    }

    /// 脚本栅栏: what fence `id` shows: the last output (or what went wrong) and a footer with
    /// the script's name and when it last ran.
    fn script_view(&self, id: FenceId) -> ScriptView {
        use pecofence_core::i18n;
        let Some(spec) = self
            .state
            .config
            .scripts
            .get(&id)
            .filter(|s| !s.path.is_empty())
        else {
            return ScriptView {
                text: i18n::text("还没有选择脚本。右键这个栅栏，选「选择脚本…」。").to_string(),
                footer: String::new(),
                quiet: false,
            };
        };
        let name = std::path::Path::new(&spec.path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| spec.path.clone());
        let running = self.script_runs.running.contains_key(&id);
        let Some((outcome, at)) = self.script_runs.last.get(&id) else {
            return ScriptView {
                text: String::new(),
                footer: i18n::format("{0} · 正在运行…", &[name]),
                quiet: true,
            };
        };
        let when = pecofence_platform::fileinfo::format_local_time(*at);
        // The body holds the output, or what went wrong; the footer names the script and says
        // when it ran.
        let (text, status) = match outcome {
            Outcome::Done(text) => (text.clone(), Some(i18n::format("{0} 更新", &[when]))),
            // An error's first line says what happened (PowerShell's further lines locate it).
            Outcome::Failed { code, text } => (
                text.lines()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or_default()
                    .to_string(),
                Some(i18n::format(
                    "{0} 运行出错（退出代码 {1}）",
                    &[
                        when,
                        code.map_or_else(|| "?".to_string(), |c| c.to_string()),
                    ],
                )),
            ),
            Outcome::TimedOut => (
                i18n::format("{0} 超过 10 秒没有结束，已停止", &[when]),
                None,
            ),
            Outcome::Missing => (i18n::text("找不到这个文件").to_string(), None),
            Outcome::Unsupported => (
                i18n::text("不能运行这种文件（支持 .ps1、.py、.cmd、.bat、.js、.exe）").to_string(),
                None,
            ),
            Outcome::NotStarted(_) => (
                i18n::text("无法启动，检查是否装了运行它的程序（如 Python）").to_string(),
                None,
            ),
        };
        let status = match (running, status) {
            (true, _) => i18n::format("{0} · 正在运行…", &[name]),
            (false, Some(status)) => format!("{name} · {status}"),
            (false, None) => name,
        };
        ScriptView {
            text,
            footer: status,
            quiet: false,
        }
    }

    /// Runs every script fence whose interval has passed (and those never run yet). Called by
    /// the minute housekeeping, and at once after a change that wants a fresh result.
    pub(super) fn run_due_scripts(&mut self) {
        if !self.state.config.settings.tools.scripts {
            return;
        }
        let now = Instant::now();
        let due: Vec<FenceId> = self
            .state
            .fences()
            .iter()
            .filter(|f| f.tool == Some(FenceTool::Script))
            .map(|f| f.id)
            .filter(|id| !self.script_runs.running.contains_key(id))
            .filter(|id| self.script_runs.due.get(id).is_none_or(|&at| at <= now))
            .collect();
        for id in due {
            self.start_script(id);
        }
    }

    /// Starts fence `id`'s script on a worker thread; the result comes back through
    /// `WM_APP_SCRIPT_DONE` ([`Self::drain_scripts`]).
    pub(super) fn start_script(&mut self, id: FenceId) {
        let Some(spec) = self.state.config.scripts.get(&id).cloned() else {
            return;
        };
        // A run of another script still going: its end starts this one (`drain_scripts`).
        if spec.path.is_empty()
            || self.script_runs.running.contains_key(&id)
            || !self.state.config.settings.tools.scripts
        {
            return;
        }
        let interval = std::time::Duration::from_secs(u64::from(spec.interval_minutes.max(1)) * 60);
        // Due again a little before the full interval: the minute housekeeping that checks is
        // coalesced by up to five seconds.
        self.script_runs.due.insert(
            id,
            Instant::now() + interval.saturating_sub(std::time::Duration::from_secs(10)),
        );
        self.script_runs.running.insert(id, spec.path.clone());
        let ext = std::path::Path::new(&spec.path)
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        tracing::info!(%id, ext, interval_min = spec.interval_minutes, "script fence run started");
        let tx = self.script_runs.tx.clone();
        let control = self.control.hwnd().0 as isize;
        let path = std::path::PathBuf::from(&spec.path);
        let started = std::thread::Builder::new()
            .name("pecofence-script".into())
            .spawn(move || {
                let begun = Instant::now();
                let outcome = crate::scripts::run(&path);
                let _ = tx.send((
                    id,
                    outcome,
                    pecofence_core::now_unix(),
                    begun.elapsed().as_millis(),
                ));
                window::post_message(
                    HWND(control as *mut core::ffi::c_void),
                    crate::commands::WM_APP_SCRIPT_DONE,
                    0,
                    0,
                );
            });
        if let Err(e) = started {
            tracing::warn!(%id, error = %e, "script thread failed");
            self.script_runs.running.remove(&id);
        }
        self.push_tool_view(self.state.host_of(id));
    }

    /// Takes the finished script runs and shows their results.
    pub(super) fn drain_scripts(&mut self) {
        while let Ok((id, outcome, at, ms)) = self.script_runs.rx.try_recv() {
            let ran = self.script_runs.running.remove(&id);
            let current = self.state.config.scripts.get(&id).map(|s| s.path.as_str());
            if ran.as_deref() != current {
                // The fence was given another script (or went) while this one ran: its result
                // is not the fence's, run the new script instead.
                tracing::info!(%id, outcome = outcome.kind(), "script fence result of the old script dropped");
                if current.is_some() {
                    self.script_runs.due.remove(&id);
                    self.start_script(id);
                }
                continue;
            }
            let (lines, code) = match &outcome {
                Outcome::Done(t) => (t.lines().count(), Some(0)),
                Outcome::Failed { code, text } => (text.lines().count(), *code),
                _ => (0, None),
            };
            tracing::info!(%id, outcome = outcome.kind(), ms, lines, code, "script fence run ended");
            self.script_runs.last.insert(id, (outcome, at));
            self.push_tool_view(self.state.host_of(id));
        }
    }

    /// 脚本栅栏: asks for a script (unless `path` is given), then makes a script fence for it
    /// near (`x`, `y`) and runs it.
    pub(super) fn new_script_near(&mut self, x: i32, y: i32, path: Option<std::path::PathBuf>) {
        let path = match path {
            Some(p) => p,
            None => {
                let owner = self.settings.as_ref().map(|h| h.hwnd());
                match pecofence_platform::filedialog::open_script(
                    owner,
                    pecofence_core::i18n::text("选择脚本"),
                ) {
                    Ok(Some(p)) => p,
                    Ok(None) => return,
                    Err(e) => {
                        tracing::warn!(error = %e, "script picker failed");
                        return;
                    }
                }
            }
        };
        let script = NewTool::Script(path.to_string_lossy().to_string());
        self.create_tool_fence(script, None, None, x, y);
    }

    /// 脚本栅栏 example: writes the weather script into the configuration's `scripts` folder
    /// (kept when it is there already) and makes a fence for it.
    pub(super) fn new_script_example(&mut self, x: i32, y: i32) {
        let Some(dir) = self
            .state
            .config_path_on_disk()
            .parent()
            .map(|d| d.join("scripts"))
        else {
            return;
        };
        let path = dir.join(pecofence_core::i18n::text("天气.ps1"));
        if !path.is_file() {
            let chinese = matches!(
                pecofence_core::i18n::language(),
                pecofence_core::i18n::Language::SimplifiedChinese
                    | pecofence_core::i18n::Language::TraditionalChinese
            );
            let written = std::fs::create_dir_all(&dir)
                .and_then(|_| std::fs::write(&path, crate::scripts::weather_example(chinese)));
            if let Err(e) = written {
                tracing::warn!(error = %e, "writing the weather example failed");
                return;
            }
        }
        tracing::info!("weather example script fence");
        self.new_script_near(x, y, Some(path));
    }

    /// Fence menu: another script for fence `id` (runs it at once).
    pub(super) fn pick_script_for(&mut self, id: FenceId) {
        let owner = self.window_for(id).map(|w| w.hwnd());
        let Ok(Some(path)) = pecofence_platform::filedialog::open_script(
            owner,
            pecofence_core::i18n::text("选择脚本"),
        ) else {
            return;
        };
        self.set_script_path(id, &path.to_string_lossy());
    }

    /// Script fence `id` runs `path` from now on (at once); false when it already did or `id`
    /// is no script fence.
    pub(super) fn set_script_path(&mut self, id: FenceId, path: &str) -> bool {
        if self
            .state
            .fence(id)
            .is_none_or(|f| f.tool != Some(FenceTool::Script))
        {
            return false;
        }
        let spec = self
            .state
            .config
            .scripts
            .entry(id)
            .or_insert_with(|| ScriptSpec::new(""));
        if spec.path == path {
            return false;
        }
        spec.path = path.to_string();
        tracing::info!(%id, "script fence script changed");
        self.state.mark_dirty();
        self.schedule_save();
        self.script_runs.last.remove(&id);
        self.script_runs.due.remove(&id);
        self.start_script(id);
        self.push_tool_view(self.state.host_of(id));
        true
    }

    /// 立即运行: the next run is now, whatever the interval says.
    pub(super) fn script_runs_reset(&mut self, id: FenceId) {
        self.script_runs.due.remove(&id);
    }

    /// Fence menu: Explorer on the script's folder with the script selected.
    pub(super) fn open_script_folder(&mut self, id: FenceId) {
        use std::os::windows::process::CommandExt;
        let Some(spec) = self
            .state
            .config
            .scripts
            .get(&id)
            .filter(|s| !s.path.is_empty())
        else {
            return;
        };
        let _ = std::process::Command::new("explorer.exe")
            .raw_arg(format!("/select,\"{}\"", spec.path))
            .spawn();
    }

    /// Fence menu: run fence `id`'s script every `minutes`.
    pub(super) fn set_script_interval(&mut self, id: FenceId, minutes: u32) {
        let Some(spec) = self.state.config.scripts.get_mut(&id) else {
            return;
        };
        let minutes = minutes.clamp(1, 1440);
        if spec.interval_minutes == minutes {
            return;
        }
        spec.interval_minutes = minutes;
        tracing::info!(%id, minutes, "script fence interval set");
        self.state.mark_dirty();
        self.schedule_save();
        // The next run counts from now.
        self.script_runs.due.insert(
            id,
            Instant::now() + std::time::Duration::from_secs(u64::from(minutes) * 60),
        );
    }

    /// A note fence near (`x`, `y`), its editor open. Switches the notes tool on first if it
    /// was off (the fence would have no window otherwise).
    pub(super) fn new_note_near(&mut self, x: i32, y: i32) {
        if let Some(id) = self.create_tool_fence(NewTool::Note(String::new()), None, None, x, y) {
            self.edit_note(id);
        }
    }

    /// Opens the editor over note `fence`'s content.
    pub(super) fn edit_note(&mut self, fence: FenceId) {
        let Some(text) = self.state.note_text(fence).map(str::to_string) else {
            tracing::info!(%fence, "note editor not opened: no such note");
            return;
        };
        let Some(w) = self.window_for(fence) else {
            tracing::info!(%fence, "note editor not opened: the note has no window");
            return;
        };
        let rect = w.content_screen_rect();
        let dpi = monitors::dpi_for_window(w.hwnd());
        tracing::info!(
            %fence,
            rect = ?(rect.left, rect.top, rect.right, rect.bottom),
            dpi,
            chars = text.chars().count(),
            "note editor opened"
        );
        crate::note_edit::open(rect, dpi, fence, &text, self.queue.clone(), self.theme_mode);
    }

    /// Quitting: the text of a note editor still open goes into the note now (its commit through
    /// the command queue would come too late).
    pub(super) fn commit_open_note(&mut self) {
        if let Some((fence, text)) = crate::note_edit::take_open() {
            tracing::info!(%fence, "note editor open at quit: text kept");
            self.note_edited(fence, &text);
        }
    }

    /// A click on a to-do's check box.
    pub(super) fn toggle_note_line(&mut self, fence: FenceId, line: usize) {
        if self.state.toggle_note_line(fence, line) {
            self.push_note(fence);
            self.schedule_save();
        }
    }

    /// The editor closed: its text replaces the note's (line breaks stored as `\n`).
    pub(super) fn note_edited(&mut self, fence: FenceId, text: &str) {
        crate::note_edit::take_committed(fence);
        let text = text.replace("\r\n", "\n");
        if self.state.set_note_text(fence, &text) {
            self.schedule_save();
        }
        self.push_note(fence);
    }

    /// Shows note `fence`'s current text in its window.
    pub(super) fn push_note(&self, fence: FenceId) {
        if let Some(w) = self.window_for(fence)
            && w.active_fence() == fence
        {
            w.set_note(self.state.note_text(fence).map(str::to_string));
        }
    }
}
