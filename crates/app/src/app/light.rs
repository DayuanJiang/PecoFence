//! 插件的轻量层 in the app: plugin fences whose plugin's main.js runs on the plugin thread
//! (`pecofence_plugin_host`). The app starts and stops them, tells them when their fence is
//! shown, hidden or resized, when the time changed and when their settings did, and draws the
//! views they show.

use super::*;
use pecofence_plugin_host as host;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
    /// The plugins PecoFence stopped, and why: they wait for 立即运行.
    pub(super) stopped: HashMap<FenceId, String>,
}

impl App {
    fn light_send(&self, cmd: host::Command) {
        if let Some((tx, _)) = &self.light.thread {
            let _ = tx.send(cmd);
        }
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

    /// Starts plugin `id`'s `main` for fence `fence` (again, when it ran).
    pub(super) fn start_light(&mut self, fence: FenceId, id: &str, main: &std::path::Path) {
        if self.hold_light(fence, Some((id, main))) {
            return;
        }
        let (Ok(code), Some(m)) = (std::fs::read_to_string(main), self.plugin_manifest(id)) else {
            return;
        };
        if self.light.thread.is_none() {
            let (out, control) = (self.light.out.clone(), self.control.hwnd().0 as isize);
            let spawned = host::spawn(pecofence_platform::fileinfo::time_text, move |o| {
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
        let saved = self
            .state
            .config
            .scripts
            .get(&fence)
            .map(|s| s.settings.clone());
        let settings = pecofence_core::plugins::fence_settings(&m, &saved.unwrap_or_default());
        let (shown, size) = self.light_shown(fence);
        tracing::info!(%fence, id, version = m.version, shown, ?size, "plugin fence starts its main.js");
        self.light.stopped.remove(&fence);
        self.light.running.insert(fence, (None, shown, size));
        let dark = self.theme_mode == ThemeMode::Dark;
        self.light_send(host::Command::Start(Box::new(host::Start {
            fence,
            code,
            settings,
            lang: pecofence_core::i18n::language().tag().to_string(),
            theme: if dark { "dark" } else { "light" }.into(),
            size,
            shown,
        })));
    }

    /// pf-view's stub of pf-api's `plugin_event` (the same signature): the event goes to the
    /// plugin thread.
    pub(super) fn plugin_event(&mut self, fence: FenceId, name: &str, data: serde_json::Value) {
        let name = ["click", "dblclick", "change", "text"]
            .into_iter()
            .find(|n| *n == name);
        if let Some(name) = name {
            self.light_send(host::Command::Event(fence, name, data));
        }
    }

    pub(super) fn stop_light(&mut self, fence: FenceId) {
        if self.hold_light(fence, None) {
            return;
        }
        self.light.stopped.remove(&fence);
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

    /// What the plugins showed, and the ones PecoFence stopped.
    pub(super) fn drain_light(&mut self) {
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
                        host::Stopped::TooMuchMemory => {
                            i18n::text("用的内存超过了 8 MB").to_string()
                        }
                        host::Stopped::Error(e) => e,
                    };
                    tracing::warn!(%fence, why, "plugin fence stopped");
                    if self.light.running.remove(&fence).is_some() {
                        self.light.stopped.insert(fence, why);
                    }
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
