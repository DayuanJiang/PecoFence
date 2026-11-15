//! 插件的轻量层 in the app: plugin fences whose plugin's main.js runs on the plugin thread
//! (`pecofence_plugin_host`). The app starts and stops them, tells them when their fence is
//! shown, hidden or resized, when the time, their settings or the clipboard changed and what
//! the user clicked, draws the views they show, does their `pf` calls (`light_host.rs`, also
//! for web plugin pages through [`App::plugin_api`]) and starts a stopped plugin again: after
//! 5 s by itself, after the third stop within an hour only by 重新启动.

use super::light_host::{AppHost, UiJob};
use super::*;
use pecofence_plugin_host as host;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// A web page's `pf` call waiting for its answer.
type Reply = Box<dyn FnOnce(std::result::Result<Value, String>)>;

/// A stopped plugin starts again by itself this long after, unless it stopped this often in
/// the last hour.
const RESTART_AFTER: std::time::Duration = std::time::Duration::from_secs(5);
const STOPS_AN_HOUR: usize = 3;

#[derive(Default)]
pub(super) struct Light {
    thread: Option<(
        std::sync::mpsc::Sender<host::Command>,
        std::thread::JoinHandle<()>,
    )>,
    out: Arc<Mutex<Vec<host::Output>>>,
    /// The fences whose plugin runs: the rows it last showed (None before its first view), and
    /// whether it was last told that the fence shows and how big it is.
    pub(super) running: HashMap<
        FenceId,
        (
            Option<Vec<crate::fence_window::ScriptRowView>>,
            bool,
            (f32, f32),
        ),
    >,
    /// The plugins PecoFence stopped, and why.
    pub(super) stopped: HashMap<FenceId, String>,
    /// When PecoFence stopped each plugin within the last hour.
    stops: HashMap<FenceId, Vec<Instant>>,
    /// The app's side of the `pf` calls, and the work it hands to this thread.
    host: Option<Arc<AppHost>>,
    jobs: Arc<Mutex<Vec<UiJob>>>,
    replies: HashMap<u32, Reply>,
    next_reply: u32,
    /// The hidden window that hears of clipboard changes (made for the first plugin that may
    /// read the clipboard), and whether one came.
    clipboard: Option<(window::Window, window::WindowClass)>,
    clipboard_changed: Arc<AtomicBool>,
    /// The corner card of `pf.notify`.
    pub(super) notice: Option<crate::notice::Notice>,
}

impl App {
    fn light_send(&self, cmd: host::Command) {
        if let Some((tx, _)) = &self.light.thread {
            let _ = tx.send(cmd);
        }
    }

    /// The app's side of the `pf` calls (made with the first plugin or web page call).
    fn light_host(&mut self) -> Arc<AppHost> {
        let config = self.state.config_path_on_disk();
        let dir = config.parent().unwrap_or(std::path::Path::new("."));
        let jobs = self.light.jobs.clone();
        let control = self.control.hwnd();
        (self
            .light
            .host
            .get_or_insert_with(|| Arc::new(AppHost::new(control, jobs, dir))))
        .clone()
    }

    /// Hands `job` to this thread from another one.
    fn post_job(jobs: &Mutex<Vec<UiJob>>, control: isize, job: UiJob) {
        if let Ok(mut jobs) = jobs.lock() {
            jobs.push(job);
        }
        window::post_message(
            HWND(control as *mut core::ffi::c_void),
            WM_APP_COMMAND,
            0,
            0,
        );
    }

    /// Fence `fence` shows its content now (the native clock asks the same before it draws).
    fn light_shown(&self, fence: FenceId) -> (bool, (f32, f32)) {
        let w = self.fences.get(&self.state.host_of(fence));
        let w = w.filter(|w| w.active_fence() == fence);
        w.map_or((false, (0.0, 0.0)), |w| {
            (w.shows_content(), w.content_dips())
        })
    }

    pub(super) fn light_runs(&self, fence: FenceId) -> bool {
        self.light.running.contains_key(&fence) || self.light.stopped.contains_key(&fence)
    }

    /// The installed manifest of fence `fence`'s plugin, read once and kept until it stops.
    fn light_manifest(&mut self, fence: FenceId) -> Option<Arc<pecofence_core::plugins::Manifest>> {
        let host = self.light_host();
        if let Some(m) = host.manifest(fence) {
            return Some(m);
        }
        let spec = self.state.config.scripts.get(&fence)?;
        let id = super::plugins::plugin_id(&spec.path)?.to_string();
        let m = Arc::new(self.plugin_manifest(&id)?);
        if let Ok(mut plugins) = host.plugins.lock() {
            plugins.insert(fence, m.clone());
        }
        Some(m)
    }

    /// Starts plugin `id`'s `main` for fence `fence` (again, when it ran).
    pub(super) fn start_light(&mut self, fence: FenceId, id: &str, main: &std::path::Path) {
        let (Ok(code), Some(m)) = (std::fs::read_to_string(main), self.plugin_manifest(id)) else {
            return;
        };
        let m = Arc::new(m);
        let api = self.light_host();
        if let Ok(mut plugins) = api.plugins.lock() {
            plugins.insert(fence, m.clone());
        }
        if self.light.thread.is_none() {
            let (out, control) = (self.light.out.clone(), self.control.hwnd().0 as isize);
            let spawned = host::spawn(api, pecofence_platform::fileinfo::time_text, move |o| {
                if let Ok(mut out) = out.lock() {
                    out.push(o);
                }
                window::post_message(
                    HWND(control as *mut core::ffi::c_void),
                    WM_APP_COMMAND,
                    0,
                    0,
                );
            });
            match spawned {
                Ok(thread) => self.light.thread = Some(thread),
                Err(e) => return tracing::warn!(error = %e, "plugin thread did not start"),
            }
        }
        let p = &m.permissions;
        if matches!(p.clipboard, Clipboard::Read | Clipboard::Readwrite) {
            self.ensure_clipboard_listener();
        }
        let saved = self
            .state
            .config
            .scripts
            .get(&fence)
            .map(|s| s.settings.clone());
        let settings = pecofence_core::plugins::fence_settings(&m, &saved.unwrap_or_default());
        let (shown, size) = self.light_shown(fence);
        tracing::info!(%fence, id, version = m.version, shown, ?size, background = p.background, "plugin fence starts its main.js");
        self.light.stopped.remove(&fence);
        self.light.running.insert(fence, (None, shown, size));
        let dark = self.theme_mode == ThemeMode::Dark;
        let config = self.state.config_path_on_disk();
        let cache = config
            .parent()
            .map(|d| d.join("plugins-host").join(format!("{fence}.view.json")));
        self.light_send(host::Command::Start(Box::new(host::Start {
            fence,
            id: m.id.clone(),
            version: m.version.clone(),
            permissions: m.permissions.clone(),
            code,
            settings,
            lang: pecofence_core::i18n::language().tag().to_string(),
            theme: if dark { "dark" } else { "light" }.into(),
            size,
            shown,
            cache,
        })));
    }

    pub(super) fn stop_light(&mut self, fence: FenceId) {
        self.light.stopped.remove(&fence);
        self.light.stops.remove(&fence);
        if let Some(Ok(mut plugins)) = self.light.host.as_ref().map(|h| h.plugins.lock()) {
            plugins.remove(&fence);
        }
        if self.light.running.remove(&fence).is_some() {
            self.light_send(host::Command::Stop(fence));
        }
    }

    /// Exit: every plugin gets `stop` (each within its 50 ms), then the thread ends.
    pub(super) fn quit_light(&mut self) {
        if let Some((tx, thread)) = self.light.thread.take() {
            let _ = tx.send(host::Command::Quit);
            let _ = thread.join();
            tracing::info!(plugins = self.light.running.len(), "plugin thread ended");
        }
    }

    /// Tells each plugin whether its fence shows and how big it is, where that changed.
    pub(super) fn sync_light(&mut self) {
        let fences: Vec<FenceId> = self.light.running.keys().copied().collect();
        for fence in fences {
            let (shown, size) = self.light_shown(fence);
            let Some((_, was_shown, was_size)) = self.light.running.get_mut(&fence) else {
                continue;
            };
            let resized = shown && (size.0 - was_size.0).abs() + (size.1 - was_size.1).abs() >= 1.0;
            let toggled = std::mem::replace(was_shown, shown) != shown;
            if resized {
                *was_size = size;
            }
            if toggled {
                self.light_send(host::Command::Shown(fence, shown));
            }
            if resized {
                let size = serde_json::json!({ "width": size.0, "height": size.1 });
                self.light_send(host::Command::Event(fence, "resize", size));
            }
        }
    }

    /// The system time, the time zone or the region's format changed.
    pub(super) fn light_time_changed(&self) {
        for fence in self.light.running.keys() {
            self.light_send(host::Command::Event(
                *fence,
                "timechange",
                serde_json::Value::Null,
            ));
        }
    }

    /// The user did something in plugin fence `fence` (click, dblclick, change, text, or a web
    /// plugin page's `message`): its plugin gets the event. A click in a stopped plugin's
    /// fence (its 重新启动) starts it again.
    pub(super) fn plugin_event(&mut self, fence: FenceId, name: &str, data: Value) {
        if self.light.stopped.contains_key(&fence) && matches!(name, "click" | "dblclick") {
            tracing::info!(%fence, "plugin started again from its fence");
            return self.start_script(fence);
        }
        let Some(name) = host::EVENTS.iter().find(|e| **e == name).copied() else {
            return tracing::warn!(%fence, name, "plugin event of an unknown name dropped");
        };
        if !self.light.running.contains_key(&fence) {
            return tracing::debug!(%fence, name, "plugin event dropped: the plugin does not run");
        }
        tracing::debug!(%fence, name, "plugin event");
        self.light_send(host::Command::Event(fence, name, data));
    }

    /// A web plugin page's `pf.<name>(args)`: the same check against the installed manifest
    /// and the same work as a light plugin's call; `reply` gets the answer on this thread.
    #[allow(dead_code)] // The web layer's page bridge calls it.
    pub(super) fn plugin_api(
        &mut self,
        fence: FenceId,
        name: &str,
        args: Value,
        reply: impl FnOnce(std::result::Result<Value, String>) + 'static,
    ) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
        match name {
            "time.now" => return reply(Ok(json!(now.unwrap_or_default().as_millis() as u64))),
            "time.format" => {
                let kind = args[1]
                    .as_str()
                    .filter(|k| !k.is_empty())
                    .unwrap_or("short");
                let ms = args[0].as_f64().unwrap_or_default() as i64;
                return reply(Ok(pecofence_platform::fileinfo::time_text(ms, kind).into()));
            }
            _ => {}
        }
        let Some(m) = self.light_manifest(fence) else {
            return reply(Err("the plugin of this fence is not installed".into()));
        };
        if let Some(why) = host::refusal(&m.permissions, name, &args) {
            tracing::info!(%fence, plugin = m.id, name, why, "web plugin call refused");
            return reply(Err(why));
        }
        self.light.next_reply += 1;
        let number = self.light.next_reply;
        self.light.replies.insert(number, Box::new(reply));
        let api = self.light_host();
        let (jobs, control) = (self.light.jobs.clone(), self.control.hwnd().0 as isize);
        let (name, agent) = (
            name.to_string(),
            format!("PecoFence-plugin/{} {}", m.id, m.version),
        );
        tracing::debug!(%fence, plugin = m.id, name, "web plugin call");
        let worker = std::thread::Builder::new()
            .name("pecofence-plugin-call".into())
            .spawn(move || {
                let result = host::perform(&*api, fence, &m.permissions, &agent, &name, args);
                let job: UiJob = Box::new(move |app: &mut App| {
                    if let Some(reply) = app.light.replies.remove(&number) {
                        reply(result);
                    }
                });
                Self::post_job(&jobs, control, job);
            });
        if worker.is_err()
            && let Some(reply) = self.light.replies.remove(&number)
        {
            reply(Err("no thread for the call".into()));
        }
    }

    /// The rows of a stopped plugin's fence: why, and when it starts again.
    pub(super) fn light_stopped_rows(
        &self,
        fence: FenceId,
    ) -> Option<Vec<crate::fence_window::ScriptRowView>> {
        use crate::fence_window::{ScriptCellView, ScriptRowView};
        use pecofence_core::i18n;
        let why = std::slice::from_ref(self.light.stopped.get(&fence)?);
        if self.light.stops.get(&fence).map_or(0, Vec::len) < STOPS_AN_HOUR {
            let text = i18n::format("插件停止了：{0}。5 秒后自动重新启动。", why);
            return Some(ScriptRowView::failed(&text, ""));
        }
        let text = i18n::format("插件停止了：{0}。一小时里停了三次，不再自动启动。", why);
        let mut rows = ScriptRowView::failed(&text, "");
        rows.push(ScriptRowView {
            cells: vec![ScriptCellView {
                text: i18n::text("重新启动").to_string(),
                clickable: true,
                ..ScriptCellView::default()
            }],
            ..ScriptRowView::default()
        });
        Some(rows)
    }

    /// A plugin stopped by PecoFence: it starts again in 5 s, unless this is its third stop
    /// within an hour.
    fn light_stopped(&mut self, fence: FenceId, why: String) {
        let stops = self.light.stops.entry(fence).or_default();
        stops.retain(|at| at.elapsed() < std::time::Duration::from_secs(3600));
        stops.push(Instant::now());
        let again = stops.len() < STOPS_AN_HOUR;
        tracing::warn!(%fence, why, stops = stops.len(), again, "plugin fence stopped");
        if self.light.running.remove(&fence).is_none() {
            return;
        }
        self.light.stopped.insert(fence, why);
        if !again {
            return;
        }
        let (jobs, control) = (self.light.jobs.clone(), self.control.hwnd().0 as isize);
        let _ = std::thread::Builder::new()
            .name("pecofence-plugin-restart".into())
            .spawn(move || {
                std::thread::sleep(RESTART_AFTER);
                let job: UiJob = Box::new(move |app: &mut App| {
                    if app.light.stopped.contains_key(&fence) {
                        tracing::info!(%fence, "plugin starts again by itself");
                        app.start_script(fence);
                    }
                });
                Self::post_job(&jobs, control, job);
            });
    }

    /// What the worker threads left for this thread, what the plugins showed, the ones
    /// PecoFence stopped, and a clipboard change.
    pub(super) fn drain_light(&mut self) {
        let jobs = self.light.jobs.lock().map(|mut j| std::mem::take(&mut *j));
        for job in jobs.unwrap_or_default() {
            job(self);
        }
        if self.light.clipboard_changed.swap(false, Ordering::Relaxed) {
            self.light_clipboard_changed();
        }
        let outputs = match self.light.out.lock() {
            Ok(mut out) if !out.is_empty() => std::mem::take(&mut *out),
            _ => return,
        };
        let mut hosts = Vec::new();
        for output in outputs {
            match output {
                host::Output::Show(fence, view) => {
                    if !self.light.running.contains_key(&fence) {
                        continue;
                    }
                    let shown = self.plugin_rows(fence, view);
                    if let Some((rows, ..)) = self.light.running.get_mut(&fence) {
                        *rows = Some(shown);
                    }
                    hosts.push(self.state.host_of(fence));
                }
                host::Output::Stopped(fence, why) => {
                    use pecofence_core::i18n;
                    let why = match why {
                        host::Stopped::TooLong => i18n::text("一次运行超过了 50 毫秒").to_string(),
                        host::Stopped::TooBusy => {
                            i18n::text("每秒合计运行超过了 200 毫秒").to_string()
                        }
                        host::Stopped::TooMuchMemory => {
                            i18n::text("用的内存超过了 8 MB").to_string()
                        }
                        host::Stopped::Error(e) => e,
                    };
                    self.light_stopped(fence, why);
                    hosts.push(self.state.host_of(fence));
                }
            }
        }
        hosts.dedup();
        // A plugin's new view is content refreshing itself, as a script's new output: it does
        // not hold back the memory housekeeping.
        pecofence_render::stack::quiet_draws(|| {
            for host in hosts {
                self.push_tool_view(host);
            }
        });
        // A hide that came without its window message still pauses the plugin.
        self.sync_light();
    }

    /// Makes the hidden window that hears of clipboard changes, once.
    pub(super) fn ensure_clipboard_listener(&mut self) {
        if self.light.clipboard.is_some() {
            return;
        }
        let (changed, control) = (
            self.light.clipboard_changed.clone(),
            self.control.hwnd().0 as isize,
        );
        let made = window::WindowClass::register("PecoFence.PluginClipboard", Default::default())
            .and_then(|class| {
                // One copy can change the clipboard several times (emptied, then filled): the
                // plugins hear of it once, 100 ms after the last change.
                let handler = Box::new(move |hwnd: HWND, message: u32, wparam: usize, _: isize| {
                    match message {
                        WM_CLIPBOARDUPDATE => window::set_timer(hwnd, 1, 100),
                        msg::WM_TIMER if wparam == 1 => {
                            window::kill_timer(hwnd, 1);
                            changed.store(true, Ordering::Relaxed);
                            let control = HWND(control as *mut core::ffi::c_void);
                            window::post_message(control, WM_APP_COMMAND, 0, 0);
                        }
                        _ => return None,
                    }
                    Some(0)
                });
                let w = window::WindowBuilder::new(&class).create(handler)?;
                Ok((w, class))
            });
        match made {
            Ok((w, class)) => {
                let listens = pecofence_platform::clipboard::listen(w.hwnd());
                tracing::info!(listens, "plugins: the clipboard is listened to");
                self.light.clipboard = Some((w, class));
            }
            Err(e) => tracing::warn!(error = %e, "plugins: no clipboard listener"),
        }
    }

    /// The clipboard changed: the running plugins that may read it are told, unless its
    /// content is marked private.
    fn light_clipboard_changed(&mut self) {
        if pecofence_platform::clipboard::is_private() {
            return tracing::info!("clipboard changed, marked private: plugins are not told");
        }
        let Some(api) = self.light.host.clone() else {
            return;
        };
        let readers: Vec<FenceId> = (self.light.running.keys())
            .filter(|f| {
                api.manifest(**f).is_some_and(|m| {
                    matches!(
                        m.permissions.clipboard,
                        Clipboard::Read | Clipboard::Readwrite
                    )
                })
            })
            .copied()
            .collect();
        tracing::debug!(plugins = readers.len(), "clipboard changed: plugins told");
        for fence in readers {
            self.light_send(host::Command::Event(fence, "clipboard", Value::Null));
        }
    }

    /// `pf.folder.pick`: the user picks a folder the plugin fence may read (kept in its
    /// settings under `__folders`); None when cancelled.
    pub(super) fn pick_plugin_folder(&mut self, fence: FenceId) -> Value {
        let owner = self.window_for(fence).map(|w| w.hwnd());
        let title = pecofence_core::i18n::text("选择插件可以读取的文件夹");
        let picked = pecofence_platform::filedialog::pick_folder(owner, title);
        let Ok(Some(path)) = picked else {
            tracing::info!(%fence, cancelled = picked.is_ok(), "plugin folder not picked");
            return Value::Null;
        };
        let path = path.to_string_lossy().to_string();
        if let Some(spec) = self.state.config.scripts.get_mut(&fence) {
            let folders = spec
                .settings
                .entry("__folders")
                .or_insert_with(|| json!([]));
            if let Some(list) = folders.as_array_mut()
                && !list.contains(&json!(path))
            {
                list.push(json!(path));
            }
            self.state.mark_dirty();
            self.schedule_save();
        }
        tracing::info!(%fence, "plugin folder picked");
        json!(path)
    }

    /// The folders the user picked for plugin fence `fence`.
    pub(super) fn plugin_folders(&self, fence: FenceId) -> Vec<String> {
        let spec = self.state.config.scripts.get(&fence);
        let folders = spec.and_then(|s| s.settings.get("__folders")?.as_array().cloned());
        let folders = folders.unwrap_or_default().into_iter();
        folders
            .filter_map(|f| f.as_str().map(str::to_string))
            .collect()
    }

    /// `pf.notify`: PecoFence's corner card with the plugin's name, and a system sound.
    pub(super) fn plugin_notify(&mut self, fence: FenceId, title: &str, body: &str, sound: bool) {
        let id = self
            .light_manifest(fence)
            .map(|m| m.id.clone())
            .unwrap_or_default();
        let title = format!("{} · {title}", self.plugin_name(&id));
        self.light.notice = None;
        let shown = crate::notice::Notice::show(
            &self.ctx,
            &title,
            body,
            pecofence_core::i18n::text("知道了"),
            crate::notice::locale(),
            &self.notice_avoid_rects(),
        );
        match shown {
            Ok(notice) => self.light.notice = Some(notice),
            Err(e) => tracing::warn!(error = %e, "plugin notice not shown"),
        }
        if sound {
            windows_core::link!("user32.dll" "system" fn MessageBeep(kind: u32) -> i32);
            // SAFETY: plain FFI call: the system's information sound (MB_ICONASTERISK).
            unsafe { MessageBeep(0x40) };
        }
        tracing::info!(%fence, plugin = id, sound, "plugin notice shown");
    }

    /// A switch of the plugin fence's settings (its fence menu) turned over.
    pub(super) fn toggle_plugin_setting(&mut self, fence: FenceId, index: usize) {
        let id = self
            .state
            .config
            .scripts
            .get(&fence)
            .and_then(|s| super::plugins::plugin_id(&s.path));
        let Some(m) = id
            .map(str::to_string)
            .and_then(|id| self.plugin_manifest(&id))
        else {
            return;
        };
        let (Some(setting), Some(spec)) = (
            m.settings.get(index),
            self.state.config.scripts.get_mut(&fence),
        ) else {
            return;
        };
        let now = pecofence_core::plugins::fence_settings(&m, &spec.settings);
        let on = !now[&setting.key].as_bool().unwrap_or(false);
        spec.settings.insert(setting.key.clone(), on.into());
        let settings = pecofence_core::plugins::fence_settings(&m, &spec.settings);
        tracing::info!(%fence, key = setting.key, on, "plugin fence setting changed");
        self.state.mark_dirty();
        self.schedule_save();
        self.light_send(host::Command::Event(fence, "settings", settings));
    }
}

use pecofence_core::plugins::Clipboard;
const WM_CLIPBOARDUPDATE: u32 = 0x031D;
