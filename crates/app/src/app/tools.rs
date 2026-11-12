//! 小工具: fences that show a tool in place of desktop items. 便签与待办 shows lines of text (a
//! line starting with `[ ]` is a to-do whose check box ticks with a click); 时钟与日期 shows the
//! time and date.

use super::*;
use crate::fence_window::{ScriptRowView, ScriptView};
use crate::scripts::Outcome;
use pecofence_core::{FenceTool, ScriptSpec};
use std::collections::HashMap;
use std::time::Instant;

/// 脚本栅栏 at run time (nothing of it is saved): each fence's last result and when it came,
/// the fences whose script is running (with the script it runs), when each is next due, a
/// click that came while its script ran, and the streaming scripts that keep running.
pub(super) struct ScriptRuns {
    last: HashMap<FenceId, (Outcome, i64)>,
    running: HashMap<FenceId, String>,
    due: HashMap<FenceId, Instant>,
    /// The runs clicks asked for while the script was running (their `action=`, None for a
    /// plain refresh), in order: each starts when the run before it ends.
    pending: HashMap<FenceId, std::collections::VecDeque<Option<String>>>,
    /// Streaming scripts: the script, the start's number (a late end of an earlier start is
    /// not this one's) and the process.
    streams: HashMap<FenceId, (String, u64, crate::scripts::Stream)>,
    next_stream: u64,
    /// The runs PecoFence started by itself (the interval came round) and when: their footer
    /// goes on showing the last run's time until the run has taken [`QUIET_RUN`], so a
    /// script that answers at once does not make the footer blink at every run.
    quiet: HashMap<FenceId, Instant>,
    tx: std::sync::mpsc::Sender<ScriptMsg>,
    rx: std::sync::mpsc::Receiver<ScriptMsg>,
}

/// How long a run PecoFence started by itself goes on without 「正在运行…」 in the footer.
const QUIET_RUN: std::time::Duration = std::time::Duration::from_secs(1);

/// What the script workers send back.
enum ScriptMsg {
    /// A run ended (`ms` it took).
    Ran {
        id: FenceId,
        outcome: Outcome,
        at: i64,
        ms: u128,
    },
    /// A streaming script printed a block or ended.
    Stream {
        id: FenceId,
        start: u64,
        event: crate::scripts::StreamEvent,
        at: i64,
    },
    /// A row's `run=` program ended and the row asked for `refresh=true`.
    Refresh(FenceId),
}

impl Default for ScriptRuns {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            last: HashMap::new(),
            running: HashMap::new(),
            due: HashMap::new(),
            pending: HashMap::new(),
            streams: HashMap::new(),
            next_stream: 0,
            quiet: HashMap::new(),
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
    /// 对话框跳转 follows its switch.
    pub(super) fn sync_dialog_jump(&self) {
        let settings = &self.state.config.settings;
        crate::dialog_jump::set_enabled(settings.tools.dialog_jump, self.control.hwnd());
        self.sync_dialog_jump_offer();
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
        if self.held_scripts_wait() {
            // Switching 小工具 on for this fence would start the scripts of an imported
            // configuration: the user turns the switch himself, on the page that lists them.
            tracing::info!(
                ?kind,
                "new tool fence refused: imported scripts wait for 小工具"
            );
            self.open_settings_page("tools", None);
            self.settings_error(super::settings::scripts_held_text());
            return None;
        }
        self.ensure_tool_on(kind);
        let (w, h) = match kind {
            FenceTool::Note => (280.0, 240.0),
            FenceTool::Clock => (240.0, 140.0),
            FenceTool::Script => (280.0, 160.0),
        };
        let rect = rect.unwrap_or_else(|| self.place_new_tool(w, h, x, y));
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
        self.show_new_fence(id);
        self.schedule_save();
        if kind == FenceTool::Script {
            self.start_script(id);
        }
        Some(id)
    }

    /// Where a new tool fence of `w` x `h` DIPs goes: a free spot near (`x`, `y`), or, made
    /// from the settings page, a spot its window does not cover (`place_clear_of`).
    fn place_new_tool(&self, w: f32, h: f32, x: i32, y: i32) -> RECT {
        self.tool_clear_of
            .and_then(|win| self.place_clear_of(w, h, win))
            .unwrap_or_else(|| self.place_new_fence_dip(w, h, x, y, None))
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
            Outcome::Done(t) => Some(t.clone()),
            // As before the two were kept apart: the error text of a run that printed nothing.
            Outcome::Failed { text, error, .. } => {
                Some(if text.is_empty() { error } else { text }.clone())
            }
            _ => None,
        });
        Some(pecofence_ipc::ScriptDto {
            path: spec.path.clone(),
            interval_minutes: spec.interval_minutes,
            outcome: outcome.map(str::to_string),
            output,
            last_run: last.map(|(_, at)| *at),
            // A streaming script runs as long as it is in `streams`.
            running: self.script_runs.running.contains_key(&id)
                || self.script_runs.streams.contains_key(&id),
        })
    }

    /// 小工具 is off and scripts of an imported configuration wait for the user to turn it on
    /// (see `ScriptSpec::held`): nothing else may turn it on.
    pub(super) fn held_scripts_wait(&self) -> bool {
        !self.state.config.settings.tools.enabled && !self.state.config.held_scripts().is_empty()
    }

    /// Switches 小工具 on in the settings when it is off (a new tool fence would have no
    /// window otherwise).
    fn ensure_tool_on(&mut self, tool: FenceTool) {
        if tool.enabled(&self.state.config.settings.tools) {
            return;
        }
        let mut settings = self.state.config.settings.clone();
        settings.tools.enabled = true;
        self.apply_settings(settings, "new tool fence");
    }

    /// Whether the desktop has a clock already (there is one at most).
    pub(super) fn has_clock(&self) -> bool {
        self.state
            .fences()
            .iter()
            .any(|f| f.tool == Some(FenceTool::Clock))
    }

    /// A clock fence near (`x`, `y`), unless the desktop has one.
    pub(super) fn new_clock_near(&mut self, x: i32, y: i32) {
        if self.has_clock() {
            tracing::info!("new clock skipped: the desktop has one");
            return;
        }
        self.create_tool_fence(NewTool::Clock, None, None, x, y);
    }

    /// 时钟与日期: the system time, the time zone or the region's time format changed (`why`).
    /// Every clock shows the new time now; it waited for its next tick, up to a minute.
    pub(super) fn refresh_clocks(&self, why: &'static str) {
        let drawn = self.fences.values().filter(|w| w.clock_changed()).count();
        if drawn > 0 {
            tracing::debug!(clocks = drawn, why, "clocks drawn again");
        }
    }

    /// 新建小工具 (tray and fence menu), command ids `base`..`base + 5`: 时钟 (while the
    /// desktop has none), the script examples, then 用自己的脚本….
    pub(super) fn new_tool_submenu(&self, base: u32) -> PopupMenu {
        use crate::scripts::Example;
        let menu = PopupMenu::new();
        if !self.has_clock() {
            menu.item(base, pecofence_core::i18n::text("时钟"), false, false);
        }
        for (i, kind) in Example::ALL.iter().enumerate() {
            menu.item(base + 1 + i as u32, kind.label(), false, false);
        }
        menu.separator().item(
            base + 1 + Example::ALL.len() as u32,
            pecofence_core::i18n::text("用自己的脚本…"),
            false,
            false,
        );
        menu
    }

    /// Entry `index` of `new_tool_submenu`, for a fence near (`x`, `y`).
    pub(super) fn new_tool_command(&mut self, index: u32, x: i32, y: i32) {
        use crate::scripts::Example;
        match index {
            0 => self.queue.push(Command::NewClock { x, y }),
            i if (i as usize) <= Example::ALL.len() => {
                let example = Example::ALL[i as usize - 1];
                self.queue.push(Command::NewScriptExample { example, x, y });
            }
            _ => self.queue.push(Command::NewScript { x, y }),
        }
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
                rows: ScriptRowView::plain(i18n::text(
                    "还没有选择脚本。右键这个栅栏，选“选择脚本…”。",
                )),
                footer: String::new(),
                footer_short: String::new(),
                quiet: false,
            };
        };
        let name = std::path::Path::new(&spec.path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| spec.path.clone());
        // A run started by the interval says so only once it has taken a while.
        let running = self.script_runs.running.contains_key(&id)
            && self
                .script_runs
                .quiet
                .get(&id)
                .is_none_or(|begun| begun.elapsed() >= QUIET_RUN);
        let Some((outcome, at)) = self.script_runs.last.get(&id) else {
            // The first run: the body says that it runs, where the rows will be.
            return ScriptView {
                rows: Vec::new(),
                footer: name,
                footer_short: String::new(),
                quiet: true,
            };
        };
        let when = pecofence_platform::fileinfo::format_local_time(*at);
        // The body holds the output, or what went wrong; the footer names the script and says
        // when it ran.
        let (text, status) = match outcome {
            Outcome::Done(text) => (text.clone(), Some(i18n::format("{0} 更新", &[when]))),
            Outcome::Failed { code, text, .. } => (
                text.clone(),
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
        // The status alone, for a fence too narrow for the script's name before it.
        let (status, short) = match (running, status) {
            (true, _) => (
                i18n::format("{0} · 正在运行…", &[name]),
                i18n::text("正在运行…").to_string(),
            ),
            (false, Some(status)) => (format!("{name} · {status}"), status),
            (false, None) => (name, String::new()),
        };
        // Only a finished run's output is read as rows with actions; an error shows as text.
        let rows = match outcome {
            Outcome::Done(_) => ScriptRowView::from_output(&text),
            // What went wrong first, then what the script printed until it failed.
            Outcome::Failed { error, .. } => {
                ScriptRowView::failed(pecofence_core::script_output::error_line(error), &text)
            }
            _ => ScriptRowView::plain(&text),
        };
        ScriptView {
            rows,
            footer: status,
            footer_short: short,
            quiet: false,
        }
    }

    /// Runs every script fence whose interval has passed (and those never run yet). Called by
    /// the minute housekeeping, and at once after a change that wants a fresh result.
    pub(super) fn run_due_scripts(&mut self) {
        self.sync_streams();
        if !self.state.config.settings.tools.enabled {
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
            .filter(|id| !self.script_runs.streams.contains_key(id))
            .filter(|id| self.script_runs.due.get(id).is_none_or(|&at| at <= now))
            .collect();
        for id in due {
            // Not the first run: its footer has a time to go on showing.
            if self.script_runs.last.contains_key(&id) {
                self.script_runs.quiet.insert(id, now);
                window::set_timer(
                    self.control.hwnd(),
                    TIMER_SCRIPT_QUIET,
                    QUIET_RUN.as_millis() as u32 + 20,
                );
            }
            self.start_script(id);
        }
    }

    /// `TIMER_SCRIPT_QUIET`: the runs started by the interval that are still going after
    /// [`QUIET_RUN`] say 「正在运行…」 in their footer now.
    pub(super) fn show_long_script_runs(&mut self) {
        let late: Vec<FenceId> = self
            .script_runs
            .quiet
            .iter()
            .filter(|(_, begun)| begun.elapsed() >= QUIET_RUN)
            .map(|(id, _)| *id)
            .collect();
        for id in &late {
            self.script_runs.quiet.remove(id);
            if self.script_runs.running.contains_key(id) {
                tracing::debug!(%id, "script fence: the run takes long, the footer says it is running");
                self.push_tool_view(self.state.host_of(*id));
            }
        }
        if !self.script_runs.quiet.is_empty() {
            window::set_timer(self.control.hwnd(), TIMER_SCRIPT_QUIET, 200);
        }
    }

    /// Starts fence `id`'s script on a worker thread; the result comes back through
    /// `WM_APP_SCRIPT_DONE` ([`Self::drain_scripts`]).
    pub(super) fn start_script(&mut self, id: FenceId) {
        self.start_script_with(id, None);
    }

    /// Runs fence `id`'s script, telling it `action` (a clicked row's `action=`). A run still
    /// going keeps the click for when it ends; a streaming script gets the action on its input.
    pub(super) fn start_script_with(&mut self, id: FenceId, action: Option<String>) {
        let Some(spec) = self.state.config.scripts.get(&id).cloned() else {
            return;
        };
        if spec.path.is_empty() || !self.state.config.settings.tools.enabled {
            return;
        }
        if spec.held {
            // From an imported configuration and not allowed yet, whatever the switch says.
            tracing::debug!(%id, "script fence not started: held since the import");
            return;
        }
        let path = std::path::PathBuf::from(&spec.path);
        if !self.script_runs.streams.contains_key(&id) && crate::scripts::update_old_example(&path)
        {
            tracing::info!(%id, "script fence runs the updated example");
        }
        if crate::scripts::is_streamable(&path) {
            self.start_stream(id, &spec, action);
            return;
        }
        if self.script_runs.running.contains_key(&id) {
            // A click while the script runs waits its turn; refreshes in a row count once.
            let queue = self.script_runs.pending.entry(id).or_default();
            if (action.is_some() || queue.back() != Some(&None)) && queue.len() < 32 {
                queue.push_back(action);
            }
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
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        tracing::info!(
            %id,
            ext,
            interval_min = spec.interval_minutes,
            action = action.is_some(),
            "script fence run started"
        );
        let env = self.script_env(id, action.unwrap_or_default());
        let tx = self.script_runs.tx.clone();
        let control = self.control.hwnd().0 as isize;
        let started = std::thread::Builder::new()
            .name("pecofence-script".into())
            .spawn(move || {
                let begun = Instant::now();
                let outcome = crate::scripts::run(&path, &env);
                let _ = tx.send(ScriptMsg::Ran {
                    id,
                    outcome,
                    at: pecofence_core::now_unix(),
                    ms: begun.elapsed().as_millis(),
                });
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

    /// A streaming script for fence `id`: started when it is not running; a running one gets
    /// `action` as a line on its input.
    fn start_stream(&mut self, id: FenceId, spec: &ScriptSpec, action: Option<String>) {
        let running = self
            .script_runs
            .streams
            .get(&id)
            .is_some_and(|(p, _, _)| *p == spec.path);
        if !running {
            let start = self.script_runs.next_stream;
            self.script_runs.next_stream += 1;
            let env = self.script_env(id, String::new());
            let tx = self.script_runs.tx.clone();
            let control = self.control.hwnd().0 as isize;
            let started = crate::scripts::Stream::start(
                std::path::Path::new(&spec.path),
                &env,
                move |event| {
                    let _ = tx.send(ScriptMsg::Stream {
                        id,
                        start,
                        event,
                        at: pecofence_core::now_unix(),
                    });
                    window::post_message(
                        HWND(control as *mut core::ffi::c_void),
                        crate::commands::WM_APP_SCRIPT_DONE,
                        0,
                        0,
                    );
                },
            );
            match started {
                Ok(stream) => {
                    tracing::info!(%id, start, "script fence stream started");
                    self.script_runs
                        .streams
                        .insert(id, (spec.path.clone(), start, stream));
                }
                Err(outcome) => {
                    tracing::warn!(%id, outcome = outcome.kind(), "script fence stream did not start");
                    self.script_runs
                        .last
                        .insert(id, (outcome, pecofence_core::now_unix()));
                    self.script_runs.due.insert(
                        id,
                        Instant::now()
                            + std::time::Duration::from_secs(
                                u64::from(spec.interval_minutes.max(1)) * 60,
                            ),
                    );
                }
            }
            self.push_tool_view(self.state.host_of(id));
        }
        if let Some(action) = action
            && let Some((_, _, stream)) = self.script_runs.streams.get_mut(&id)
        {
            let sent = stream.send(&action);
            tracing::info!(%id, sent, "script fence stream got an action");
        }
    }

    /// What a run of fence `id`'s script is told (environment variables).
    fn script_env(&self, id: FenceId, action: String) -> crate::scripts::RunEnv {
        let data_dir = self
            .state
            .config_path_on_disk()
            .parent()
            .map(|d| d.join("scripts-data").join(id.to_string()))
            .unwrap_or_default();
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            tracing::warn!(%id, error = %e, "script data folder not made");
        }
        crate::scripts::RunEnv {
            action,
            data_dir,
            theme: if self.theme_mode == ThemeMode::Dark {
                "dark"
            } else {
                "light"
            },
            fence: id.to_string(),
        }
    }

    /// Ends the streaming scripts whose fence went, lost its script or had the tool switched
    /// off.
    pub(super) fn sync_streams(&mut self) {
        let on = self.state.config.settings.tools.enabled;
        let keep: Vec<(FenceId, bool)> = self
            .script_runs
            .streams
            .iter()
            .map(|(id, (path, _, _))| {
                let current = self
                    .state
                    .fence(*id)
                    .is_some_and(|f| f.tool == Some(FenceTool::Script))
                    && self
                        .state
                        .config
                        .scripts
                        .get(id)
                        .is_some_and(|s| s.path == *path);
                (*id, on && current)
            })
            .collect();
        for (id, keep) in keep {
            if !keep {
                self.script_runs.streams.remove(&id);
                tracing::info!(%id, tool_on = on, "script fence stream ended by PecoFence");
            }
        }
    }

    /// Takes the finished script runs and the streaming scripts' blocks and shows them.
    pub(super) fn drain_scripts(&mut self) {
        let mut shown: Vec<FenceId> = Vec::new();
        while let Ok(msg) = self.script_runs.rx.try_recv() {
            match msg {
                ScriptMsg::Ran {
                    id,
                    outcome,
                    at,
                    ms,
                } => {
                    let ran = self.script_runs.running.remove(&id);
                    self.script_runs.quiet.remove(&id);
                    let current = self.state.config.scripts.get(&id).map(|s| s.path.as_str());
                    if ran.as_deref() != current {
                        // The fence was given another script (or went) while this one ran: its
                        // result is not the fence's, run the new script instead.
                        tracing::info!(%id, outcome = outcome.kind(), "script fence result of the old script dropped");
                        self.script_runs.pending.remove(&id);
                        if current.is_some() {
                            self.script_runs.due.remove(&id);
                            self.start_script(id);
                        }
                        continue;
                    }
                    let (lines, code) = match &outcome {
                        Outcome::Done(t) => (t.lines().count(), Some(0)),
                        Outcome::Failed { code, text, .. } => (text.lines().count(), *code),
                        _ => (0, None),
                    };
                    tracing::info!(%id, outcome = outcome.kind(), ms, lines, code, "script fence run ended");
                    if let Outcome::Failed { error, .. } = &outcome {
                        tracing::debug!(
                            %id,
                            error_lines = error.lines().count(),
                            shown = !pecofence_core::script_output::error_line(error).is_empty(),
                            "script fence: a failed run shows its error line above its output"
                        );
                    }
                    self.script_runs.last.insert(id, (outcome, at));
                    shown.push(id);
                    let next = self
                        .script_runs
                        .pending
                        .get_mut(&id)
                        .and_then(|q| q.pop_front());
                    if let Some(action) = next {
                        tracing::info!(%id, action = action.is_some(), "script fence: a click made during the run runs now");
                        self.start_script_with(id, action);
                    }
                }
                ScriptMsg::Stream {
                    id,
                    start,
                    event,
                    at,
                } => {
                    if self
                        .script_runs
                        .streams
                        .get(&id)
                        .is_none_or(|(_, s, _)| *s != start)
                    {
                        continue;
                    }
                    match event {
                        crate::scripts::StreamEvent::Block(text) => {
                            self.script_runs.last.insert(id, (Outcome::Done(text), at));
                        }
                        crate::scripts::StreamEvent::Ended(outcome) => {
                            tracing::info!(%id, outcome = outcome.kind(), "script fence stream ended");
                            self.script_runs.streams.remove(&id);
                            let minutes = self
                                .state
                                .config
                                .scripts
                                .get(&id)
                                .map_or(30, |s| s.interval_minutes.max(1));
                            // Started again at the next interval.
                            self.script_runs.due.insert(
                                id,
                                Instant::now()
                                    + std::time::Duration::from_secs(u64::from(minutes) * 60),
                            );
                            self.script_runs.last.insert(id, (outcome, at));
                        }
                    }
                    shown.push(id);
                }
                ScriptMsg::Refresh(id) => self.refresh_script(id),
            }
        }
        shown.sort();
        shown.dedup();
        for id in shown {
            self.push_tool_view(self.state.host_of(id));
        }
    }

    /// A click on row `index` of script fence `fence`: what its parameters ask for.
    pub(super) fn script_row_clicked(&mut self, fence: FenceId, index: usize) {
        use pecofence_core::script_output::{
            RowAction, ScriptLine, TargetKind, click_allowed, parse,
        };
        let Some((Outcome::Done(text), at)) = self.script_runs.last.get(&fence).cloned() else {
            return;
        };
        let Some(ScriptLine::Row(row)) = parse(&text).into_iter().nth(index) else {
            return;
        };
        if !row.clickable() {
            return;
        }
        let script = self
            .state
            .config
            .scripts
            .get(&fence)
            .map(|s| std::path::PathBuf::from(&s.path));
        let dir = script
            .as_deref()
            .and_then(|s| s.parent().map(|d| d.to_path_buf()));
        // What `href=` / `open=` name, as a path: a relative one is taken from the script's
        // folder.
        let opens = match &row.action {
            Some(RowAction::Href(target) | RowAction::Open(target)) => {
                let mut path = std::path::PathBuf::from(target);
                if path.is_relative()
                    && !target.contains(':')
                    && let Some(d) = &dir
                {
                    path = d.join(path);
                }
                Some((target.clone(), path))
            }
            _ => None,
        };
        let kind = match &row.action {
            Some(RowAction::Href(_)) => "href",
            Some(RowAction::Open(_)) => "open",
            Some(RowAction::Run { .. }) => "run",
            Some(RowAction::Copy(_)) => "copy",
            Some(RowAction::Script(_)) => "action",
            None => "refresh",
        };
        tracing::info!(%fence, index, what = kind, refresh = row.refresh, "script row clicked");
        // A row starts a program only when the script's header allows it: what a script prints
        // can be text from the network that reads as parameters.
        if let Some(action) = &row.action {
            let allows = script.as_deref().is_some_and(crate::scripts::allows_run);
            let target = match &opens {
                Some((target, path)) if !allows => crate::scripts::target_kind(target, path),
                _ => TargetKind::Other,
            };
            let allowed = click_allowed(action, target, allows);
            tracing::debug!(%fence, key = kind, ?target, allows, allowed, "script row: may the click do this");
            if !allowed {
                let script = script
                    .as_deref()
                    .and_then(|s| s.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                tracing::info!(%fence, script, key = kind, "script row refused: the script's header does not allow starting programs");
                if let Some(w) = self.window_for(fence) {
                    w.show_script_row_refused(index);
                }
                return;
            }
        }
        let owner = self.window_for(fence).map(|w| w.hwnd());
        let mut refresh = row.refresh;
        match row.action {
            Some(RowAction::Href(_)) | Some(RowAction::Open(_)) => {
                if let Some((_, path)) = &opens
                    && let Err(e) = shell::shell_execute(path, None, owner)
                {
                    tracing::warn!(%fence, what = kind, error = %e, "script row: open failed");
                }
            }
            Some(RowAction::Run { program, args }) => {
                match crate::scripts::run_program(&program, &args, dir.as_deref()) {
                    Ok(mut child) if refresh => {
                        // The script runs again once the program has done its work.
                        refresh = false;
                        let tx = self.script_runs.tx.clone();
                        let control = self.control.hwnd().0 as isize;
                        let _ = std::thread::Builder::new()
                            .name("pecofence-script-run".into())
                            .spawn(move || {
                                let begun = Instant::now();
                                while begun.elapsed() < std::time::Duration::from_secs(60) {
                                    if !matches!(child.try_wait(), Ok(None)) {
                                        break;
                                    }
                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                }
                                let _ = tx.send(ScriptMsg::Refresh(fence));
                                window::post_message(
                                    HWND(control as *mut core::ffi::c_void),
                                    crate::commands::WM_APP_SCRIPT_DONE,
                                    0,
                                    0,
                                );
                            });
                    }
                    Ok(_) => {}
                    // A program that asks for administrator rights (Task Manager) starts only
                    // through the shell, which shows Windows' prompt.
                    Err(e) if e.raw_os_error() == Some(740) && args.is_empty() => {
                        let started =
                            shell::shell_execute(std::path::Path::new(&program), None, owner);
                        tracing::info!(%fence, ok = started.is_ok(), "script row: program started through the shell (administrator)");
                    }
                    Err(e) => {
                        tracing::warn!(%fence, error = %e, "script row: program did not start")
                    }
                }
            }
            Some(RowAction::Copy(text)) => {
                let copied = pecofence_platform::clipboard::set_text(self.control.hwnd(), &text);
                tracing::info!(%fence, copied, "script row: text copied");
                if let Some(w) = self.window_for(fence) {
                    w.show_script_copy_result(index, copied);
                }
            }
            Some(RowAction::Script(action)) => {
                // A to-do ticks at once; the script's next output has the final say.
                if row.todo.is_some()
                    && let Some(flipped) = pecofence_core::note::toggle(&text, index)
                {
                    self.script_runs
                        .last
                        .insert(fence, (Outcome::Done(flipped), at));
                    self.push_tool_view(self.state.host_of(fence));
                }
                self.start_script_with(fence, Some(action));
                return;
            }
            None => {}
        }
        if refresh {
            self.refresh_script(fence);
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

    /// The weather example (the test script's `new-script-example`).
    pub(super) fn new_script_example(&mut self, x: i32, y: i32) {
        self.new_script_example_of(crate::scripts::Example::Weather, x, y);
    }

    /// 脚本栅栏 example `kind`: writes its script into the configuration's `scripts` folder
    /// (kept when it is there already) and makes a fence for it.
    pub(super) fn new_script_example_of(&mut self, kind: crate::scripts::Example, x: i32, y: i32) {
        let Some(dir) = self
            .state
            .config_path_on_disk()
            .parent()
            .map(|d| d.join("scripts"))
        else {
            return;
        };
        let path = dir.join(kind.file_name());
        if !path.is_file() {
            let chinese = matches!(
                pecofence_core::i18n::language(),
                pecofence_core::i18n::Language::SimplifiedChinese
                    | pecofence_core::i18n::Language::TraditionalChinese
            );
            let text = crate::scripts::example_file_text(kind, chinese);
            let written = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&path, text));
            if let Err(e) = written {
                tracing::warn!(error = %e, example = kind.name(), "writing the example failed");
                return;
            }
        }
        tracing::info!(example = kind.name(), "example script fence");
        // Room for the example's rows (DIPs).
        let (w, h) = match kind {
            crate::scripts::Example::Weather => (280.0, 160.0),
            crate::scripts::Example::GitHub => (360.0, 300.0),
            crate::scripts::Example::Status => (280.0, 300.0),
        };
        let rect = self.place_new_tool(w, h, x, y);
        let script = NewTool::Script(path.to_string_lossy().to_string());
        // Titled as its menu line reads (a file is named `pc-status` in some languages).
        self.create_tool_fence(script, Some(kind.label()), Some(rect), x, y);
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
        // A script the user chose himself is his own.
        spec.held = false;
        tracing::info!(%id, "script fence script changed");
        self.state.mark_dirty();
        self.schedule_save();
        self.script_runs.last.remove(&id);
        self.script_runs.due.remove(&id);
        self.start_script(id);
        self.push_tool_view(self.state.host_of(id));
        true
    }

    /// 立即运行, `refresh=true` and `fence refresh`: runs fence `id`'s script now (after the
    /// run still going); a streaming script starts again.
    pub(super) fn refresh_script(&mut self, id: FenceId) {
        if self.script_runs.streams.remove(&id).is_some() {
            tracing::info!(%id, "script fence stream restarted");
        }
        self.script_runs.due.remove(&id);
        self.start_script_with(id, None);
    }

    /// Fence menu 编辑脚本: the script in Notepad (its own editor may run it instead).
    pub(super) fn edit_script(&mut self, id: FenceId) {
        let Some(spec) = self
            .state
            .config
            .scripts
            .get(&id)
            .filter(|s| !s.path.is_empty())
        else {
            return;
        };
        let started = std::process::Command::new("notepad.exe")
            .arg(&spec.path)
            .spawn();
        tracing::info!(%id, ok = started.is_ok(), "script fence: script opened in Notepad");
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
        // The path goes in between quotes. No file's path holds a quote, but one in an edited
        // or imported configuration can: it would end the argument early and hand Explorer the
        // rest as arguments of its own.
        if spec.path.contains('"') {
            tracing::info!(%id, "script fence: its folder not opened, the path holds a quote");
            return;
        }
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
            self.edit_note(id, None);
        }
    }

    /// Opens the editor of note `fence`, the caret at `caret` (a place in its text, UTF-16
    /// units) or after the last line.
    pub(super) fn edit_note(&mut self, fence: FenceId, caret: Option<usize>) {
        let Some(text) = self.state.note_text(fence).map(str::to_string) else {
            tracing::info!(%fence, "note editor not opened: no such note");
            return;
        };
        // A rolled-up note (「编辑便签」 or 「添加待办」 from its title's menu) opens first: the
        // editor draws nothing itself, the fence shows the text, and a rolled one shows none.
        if self.window_for(fence).is_some_and(|w| w.rests_rolled()) {
            tracing::info!(%fence, "note editor: the rolled-up fence opens first");
            self.set_roll(fence, false);
        }
        let Some(w) = self.window_for(fence) else {
            tracing::info!(%fence, "note editor not opened: the note has no window");
            return;
        };
        let rect = w.content_screen_rect();
        let dpi = monitors::dpi_for_window(w.hwnd());
        let text_size = self.state.fence(fence).map_or(1, text_size_index);
        let font_dip = pecofence_render::fence_chrome::TOOL_TEXT_SIZES[usize::from(text_size)];
        tracing::info!(
            %fence,
            rect = ?(rect.left, rect.top, rect.right, rect.bottom),
            dpi,
            text_size,
            font_dip,
            chars = text.chars().count(),
            ?caret,
            "note editor opened"
        );
        crate::note_edit::open(
            rect,
            dpi,
            fence,
            &text,
            self.queue.clone(),
            self.theme_mode,
            font_dip,
            caret,
        );
    }

    /// 「添加待办」 in a note's menu: its editor opens with an empty to-do after the last
    /// line, the caret in it (feedback #141).
    pub(super) fn add_note_todo(&mut self, fence: FenceId) {
        self.edit_note(fence, None);
        crate::note_edit::add_todo(fence);
    }

    /// The editor of note `fence` holds `text` with this selection now: its window draws
    /// them, and the editor's control follows the caret drawn there (the input method's
    /// window appears at it).
    pub(super) fn note_edit_state(
        &self,
        fence: FenceId,
        text: String,
        anchor: usize,
        active: usize,
    ) {
        if crate::note_edit::open_for() != Some(fence) {
            // The editor closed while this was queued: its text is on the way as the note's.
            return;
        }
        let Some(w) = self.window_for(fence).filter(|w| w.active_fence() == fence) else {
            return;
        };
        w.set_note_edit(Some(crate::fence_window::NoteEditView {
            text,
            anchor,
            active,
            caret_on: true,
        }));
        if let Some((x, top, _)) = w.note_caret_screen() {
            crate::note_edit::place_caret(fence, x, top);
        }
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
        // While the note is edited the tick goes into the editor's text (Ctrl+Z takes it
        // back); the note gets it when the editor closes.
        if crate::note_edit::open_for() == Some(fence) {
            crate::note_edit::toggle_line(fence, line);
            return;
        }
        // Outside the editor the tick is a step for Ctrl+Z in the fence, as a dragged
        // to-do is.
        let before = self.state.note_text(fence).map(str::to_string);
        if self.state.toggle_note_line(fence, line) {
            if let (Some(before), Some(after)) = (before, self.state.note_text(fence)) {
                self.note_line_for_undo(fence, before, after.to_string(), "note to-do ticked");
            }
            self.push_note(fence);
            self.schedule_save();
        }
    }

    /// A to-do was dragged to another line of its note: one saved change, as a tick is,
    /// and one step for Ctrl+Z in the fence.
    pub(super) fn move_note_line(&mut self, fence: FenceId, from: usize, to: usize) {
        let before = self.state.note_text(fence).map(str::to_string);
        if self.state.move_note_line(fence, from, to) {
            if let (Some(before), Some(after)) = (before, self.state.note_text(fence)) {
                self.note_line_for_undo(fence, before, after.to_string(), "note line moved");
            }
            self.push_note(fence);
            self.schedule_save();
        }
    }

    /// A click on a web address in a note: it opens in the default browser. Only what
    /// `note::links` finds there is opened, an `http://` or `https://` address, as a script
    /// fence's `href=` row opens without its header line: a note starts no program and opens
    /// no file.
    pub(super) fn open_note_link(&mut self, fence: FenceId, line: usize, start: usize) {
        use pecofence_core::note::{self, NoteLine};
        let address = self.state.note_text(fence).and_then(|text| {
            let shown = match *note::lines(text).get(line)? {
                NoteLine::Todo { text, .. } | NoteLine::Text(text) => text,
            };
            let wide: Vec<u16> = shown.encode_utf16().collect();
            let (from, to) = note::links(shown).into_iter().find(|l| l.0 == start)?;
            Some(String::from_utf16_lossy(&wide[from..to]))
        });
        let Some(address) = address.filter(|a| pecofence_core::script_output::is_web_address(a))
        else {
            tracing::info!(%fence, line, start, "note link not opened: no web address there");
            return;
        };
        let owner = self.window_for(fence).map(|w| w.hwnd());
        let opened = shell::shell_execute(std::path::Path::new(&address), None, owner);
        // The address itself stays out of the log: it is the user's.
        tracing::info!(
            %fence,
            line,
            chars = address.len(),
            https = address.to_ascii_lowercase().starts_with("https://"),
            ok = opened.is_ok(),
            "note link opened in the browser"
        );
    }

    /// The editor closed: its text replaces the note's (line breaks stored as `\n`).
    pub(super) fn note_edited(&mut self, fence: FenceId, text: &str) {
        crate::note_edit::take_committed(fence);
        let text = text.replace("\r\n", "\n");
        if self.state.set_note_text(fence, &text) {
            self.schedule_save();
        }
        self.push_note(fence);
        // The window draws the stored note again, which has the editor's text by now.
        if let Some(w) = self.window_for(fence) {
            w.set_note_edit(None);
        }
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
