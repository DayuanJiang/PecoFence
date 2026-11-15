//! 社区插件 in the app. A plugin is installed in `<config>\plugins\<id>\<version>\`, and
//! `<config>\plugins\<id>\current.txt` names the version in use (writing it is the switch; its
//! `plugin.json` gives the permissions); a plugin's fences run `plugin:<id>` (its `main.js`),
//! and its data lives in `<config>\plugins-data\<id>\`. The index is downloaded when
//! 设置 › 小工具 opens, and once a day while a plugin is installed; downloads run on threads
//! of their own and come back through `drain_plugins`.

use super::{App, HWND};
use pecofence_core::i18n;
use pecofence_core::plugins::{self as list, Entry, Index, Manifest, Update};
use pecofence_platform::window;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub(super) const TIMER_PLUGINS_CHECK: usize = 101;
/// A test instance's index (`http://127.0.0.1:` only); its files are on the same server.
const URL_VAR: &str = "PECOFENCE_PLUGINS_URL";

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Status {
    #[default]
    Idle,
    Loading,
    Offline,
    Ready,
}

#[derive(Default)]
pub(super) struct Plugins {
    index: Option<Index>,
    status: Status,
    /// Plugins being downloaded.
    busy: std::collections::HashSet<String>,
    /// The row the page opens (插件详情 in a plugin fence's menu), numbered so that the page
    /// opens it again after the user closed it.
    open: Option<(u32, String)>,
    done: Arc<Mutex<Vec<Done>>>,
}

enum Done {
    Index(Result<Index, String>),
    /// A download: Err(true, why) when a file is not the reviewed one, Err(false, why) when the
    /// network or the disk failed.
    Downloaded(String, bool, Result<(), (bool, String)>),
}

/// What a fence's script path names when it is a plugin: `plugin:<id>`.
pub(super) fn plugin_id(path: &str) -> Option<&str> {
    path.strip_prefix("plugin:").filter(|id| list::valid_id(id))
}

/// Whether this edition shows plugins (the Store edition does not, for now).
fn shown() -> bool {
    !pecofence_platform::process::is_packaged()
}

fn test_url(test_instance: bool) -> Option<String> {
    let url = pecofence_core::brand::var(URL_VAR).ok();
    url.filter(|u| test_instance && u.starts_with("http://127.0.0.1:"))
}

/// The first address that answers (each gets 8 s).
fn get_first(urls: &[String]) -> Result<Vec<u8>, String> {
    let mut errors = Vec::new();
    for url in urls {
        match super::updates::get_in_time(url, "") {
            Ok(body) => return Ok(body),
            Err(e) => errors.push(e),
        }
    }
    Err(errors.join("; "))
}

fn fetch_index(test: Option<String>) -> Result<Index, String> {
    let urls = test.map_or_else(|| list::index_urls().to_vec(), |u| vec![u]);
    let (index, dropped) = list::parse_index(&get_first(&urls)?)?;
    for why in dropped {
        tracing::warn!(why, "plugins: index entry left out");
    }
    Ok(index)
}

/// Downloads `entry`'s files at the index's `commit` into `dir` (made afresh) and checks each
/// against the index (size, SHA-256) and `plugin.json` against the entry.
fn download(
    entry: &Entry,
    commit: &str,
    test: Option<&str>,
    dir: &Path,
) -> Result<(), (bool, String)> {
    let io = |e: std::io::Error| (false, e.to_string());
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).map_err(io)?;
    let id = &entry.manifest.id;
    for file in entry
        .files
        .iter()
        .filter(|f| list::INSTALL_FILES.contains(&f.name.as_str()))
    {
        let name = &file.name;
        let urls = match test {
            Some(index) => vec![list::test_file_url(index, commit, id, name)],
            None => list::file_urls(commit, id, name).to_vec(),
        };
        let bytes = get_first(&urls).map_err(|e| (false, e))?;
        if bytes.len() as u64 != file.size
            || pecofence_platform::http::sha256_hex(&bytes) != file.sha256
        {
            return Err((
                true,
                format!("{name}: size or SHA-256 differs from the index"),
            ));
        }
        std::fs::write(dir.join(name), &bytes).map_err(io)?;
    }
    let manifest = std::fs::read(dir.join("plugin.json")).map_err(|e| e.to_string());
    let manifest = manifest
        .and_then(|b| list::parse_manifest(&b))
        .map_err(|e| (true, e))?;
    if manifest != entry.manifest {
        return Err((true, "plugin.json differs from the index".into()));
    }
    Ok(())
}

/// Removes `dir`, trying again for a second: a script that was just stopped may still have it
/// as its current folder.
fn remove_dir(dir: &Path) -> std::io::Result<()> {
    for _ in 0..10 {
        match std::fs::remove_dir_all(dir) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                std::thread::sleep(std::time::Duration::from_millis(100))
            }
            _ => return Ok(()),
        }
    }
    std::fs::remove_dir_all(dir)
}

impl App {
    fn plugins_dir(&self) -> PathBuf {
        let config = self.state.config_path_on_disk();
        config.parent().unwrap_or(Path::new(".")).join("plugins")
    }

    pub(super) fn plugin_data_dir(&self, id: &str) -> PathBuf {
        let config = self.state.config_path_on_disk();
        config
            .parent()
            .unwrap_or(Path::new("."))
            .join("plugins-data")
            .join(id)
    }

    /// The folder of the plugin's version in use (`current.txt`).
    fn plugin_dir(&self, id: &str) -> Option<PathBuf> {
        let dir = self.plugins_dir().join(id);
        let version = std::fs::read_to_string(dir.join("current.txt")).ok()?;
        let version = version.trim();
        pecofence_core::updates::parse_version(version).map(|_| dir.join(version))
    }

    /// The file a fence's script path runs: a plugin's `main.js` (a web plugin's `index.html`),
    /// or the path itself.
    pub(super) fn script_file(&self, path: &str) -> PathBuf {
        match plugin_id(path) {
            Some(id) => {
                let web = self.plugin_manifest(id).is_some_and(|m| m.layer == "web");
                let main = if web { "index.html" } else { "main.js" };
                self.plugin_dir(id).unwrap_or_default().join(main)
            }
            None => PathBuf::from(path),
        }
    }

    /// 网页层: the installed web plugin fence `fence` shows (none when it is taken down).
    pub(super) fn web_plugin_of(
        &self,
        fence: pecofence_core::FenceId,
    ) -> Option<crate::web_plugin::Plugin> {
        let spec = self.state.config.scripts.get(&fence)?;
        let id = plugin_id(&spec.path)?;
        let m = self.plugin_manifest(id).filter(|m| m.layer == "web")?;
        let dir = self.plugin_dir(id)?;
        let ok = dir.join("index.html").is_file() && self.plugin_disabled(id).is_none();
        ok.then(|| crate::web_plugin::Plugin {
            id: id.to_string(),
            dir,
            network: m.permissions.network,
            background: m.permissions.background,
            clipboard_read: m.permissions.clipboard >= list::Clipboard::Read,
        })
    }

    /// The installed plugin's `plugin.json` (it stays while the plugin is taken down).
    pub(super) fn plugin_manifest(&self, id: &str) -> Option<Manifest> {
        let bytes = std::fs::read(self.plugin_dir(id)?.join("plugin.json")).ok()?;
        list::parse_manifest(&bytes).ok().filter(|m| m.id == id)
    }

    /// Why the plugin was taken down, when it was.
    fn plugin_disabled(&self, id: &str) -> Option<String> {
        let text = std::fs::read_to_string(self.plugins_dir().join(id).join("disabled.txt"));
        text.ok().map(|t| t.trim().to_string())
    }

    fn installed_plugins(&self) -> Vec<Manifest> {
        let dirs = std::fs::read_dir(self.plugins_dir())
            .into_iter()
            .flatten()
            .flatten();
        let mut ids: Vec<String> = dirs
            .map(|d| d.file_name().to_string_lossy().to_string())
            .collect();
        ids.sort();
        ids.iter()
            .filter(|id| list::valid_id(id))
            .filter_map(|id| self.plugin_manifest(id))
            .collect()
    }

    fn plugin_fences(&self, id: &str) -> Vec<pecofence_core::FenceId> {
        let path = format!("plugin:{id}");
        let scripts = self.state.config.scripts.iter();
        let mut fences: Vec<_> = scripts
            .filter(|(fence, s)| s.path == path && self.state.fence(**fence).is_some())
            .map(|(fence, _)| *fence)
            .collect();
        fences.sort();
        fences
    }

    /// A plugin fence that cannot run: the text it shows instead.
    pub(super) fn plugin_blocked_text(&self, id: &str) -> Option<String> {
        if let Some(why) = self.plugin_disabled(id) {
            return Some(i18n::format("已下架：{0}", &[why]));
        }
        let missing = !self.script_file(&format!("plugin:{id}")).is_file();
        missing.then(|| {
            i18n::text("这个插件还没有安装。右键这个栅栏，选“插件详情”去安装。").to_string()
        })
    }

    /// The name a plugin fence's footer and title show.
    pub(super) fn plugin_name(&self, id: &str) -> String {
        let lang = i18n::language().tag();
        let name = self
            .plugin_manifest(id)
            .map(|m| list::pick(&m.name, lang).to_string());
        name.unwrap_or_else(|| id.to_string())
    }

    /// Whether a row of plugin `id` may start a program: its installed `plugin.json` says so
    /// (the script's header must allow it too).
    pub(super) fn plugin_may_run(&self, id: &str) -> bool {
        self.plugin_manifest(id).is_some_and(|m| m.permissions.run)
    }

    /// At start (`first`: a minute later) and then once a day.
    pub(super) fn arm_plugins_check(&self, first: bool) {
        let ms = if first {
            60 * 1000
        } else {
            24 * 60 * 60 * 1000
        };
        window::set_timer(self.control.hwnd(), TIMER_PLUGINS_CHECK, ms);
    }

    /// `TIMER_PLUGINS_CHECK`: the daily check, while a plugin is installed (whatever the
    /// edition and 自动检查更新 say).
    pub(super) fn daily_plugins_check(&mut self) {
        self.arm_plugins_check(false);
        let installed = self.installed_plugins().len();
        tracing::info!(installed, "plugins: daily check");
        if shown() && installed > 0 {
            self.fetch_plugins();
        }
        self.migrate_tools(true);
    }

    /// 设置 › 小工具 was shown.
    pub(super) fn plugins_page_shown(&mut self) {
        if shown() {
            self.fetch_plugins();
        }
    }

    fn fetch_plugins(&mut self) {
        if self.plugins.status == Status::Loading {
            return;
        }
        self.plugins.status = Status::Loading;
        let test = test_url(self.instance.is_some());
        self.run_plugin_job(move || Done::Index(fetch_index(test)));
        self.push_settings_state();
    }

    fn run_plugin_job(&self, job: impl FnOnce() -> Done + Send + 'static) {
        let done = self.plugins.done.clone();
        let control = self.control.hwnd().0 as isize;
        let spawned = std::thread::Builder::new()
            .name("pecofence-plugins".into())
            .spawn(move || {
                let result = job();
                if let Ok(mut d) = done.lock() {
                    d.push(result);
                }
                let control = HWND(control as *mut core::ffi::c_void);
                window::post_message(control, super::WM_APP_COMMAND, 0, 0);
            });
        if let Err(e) = spawned {
            tracing::warn!(error = %e, "plugins: thread failed to start");
        }
    }

    pub(super) fn drain_plugins(&mut self) {
        let done = match self.plugins.done.lock() {
            Ok(mut d) if !d.is_empty() => std::mem::take(&mut *d),
            _ => return,
        };
        for d in done {
            match d {
                Done::Index(Ok(index)) => {
                    tracing::info!(plugins = index.plugins.len(), commit = %index.commit, "plugins: index read");
                    self.plugins.index = Some(index);
                    self.plugins.status = Status::Ready;
                    self.apply_plugin_index();
                    self.migrate_tools(true);
                }
                Done::Index(Err(e)) => {
                    tracing::warn!(error = %e, "plugins: index not read");
                    self.plugins.status = Status::Offline;
                }
                Done::Downloaded(id, update, result) => {
                    self.plugin_downloaded(&id, update, result);
                    self.migrate_tools(false);
                }
            }
        }
        self.push_settings_state();
    }

    /// Takes down what the index marks disabled and updates what may update by itself.
    fn apply_plugin_index(&mut self) {
        for installed in self.installed_plugins() {
            let id = installed.id.as_str();
            let Some(entry) = self.plugins.index.as_ref().and_then(|i| i.get(id)) else {
                continue;
            };
            let kind = list::update_kind(&installed, &entry.manifest, env!("CARGO_PKG_VERSION"));
            let (offered, why) = (entry.manifest.version.clone(), entry.disabled.clone());
            let disabled = self.plugin_disabled(id).is_some();
            tracing::info!(
                id,
                installed = installed.version,
                offered,
                ?kind,
                disabled,
                "plugins: update check"
            );
            match why {
                Some(why) if !disabled => self.disable_plugin(id, &why),
                None if kind == Update::Auto && !disabled => self.start_plugin_download(id, true),
                _ => {}
            }
        }
    }

    fn start_plugin_download(&mut self, id: &str, update: bool) {
        let Some(index) = &self.plugins.index else {
            return;
        };
        let (Some(entry), commit) = (index.get(id).cloned(), index.commit.clone()) else {
            return;
        };
        if !self.plugins.busy.insert(id.to_string()) {
            return;
        }
        let temp = self.plugins_dir().join(format!(".new-{id}"));
        let test = test_url(self.instance.is_some());
        let id = id.to_string();
        tracing::info!(
            id,
            update,
            version = entry.manifest.version,
            "plugins: download"
        );
        self.run_plugin_job(move || {
            let result = download(&entry, &commit, test.as_deref(), &temp);
            Done::Downloaded(id, update, result)
        });
    }

    fn plugin_downloaded(&mut self, id: &str, update: bool, result: Result<(), (bool, String)>) {
        self.plugins.busy.remove(id);
        let (dir, temp) = (
            self.plugins_dir().join(id),
            self.plugins_dir().join(format!(".new-{id}")),
        );
        let result = result.and_then(|_| {
            // Stop the plugin's fences, move the download into its version's folder, then
            // switch to it by writing current.txt.
            for fence in self.plugin_fences(id) {
                self.stop_script(fence);
            }
            let manifest = std::fs::read(temp.join("plugin.json")).ok();
            let manifest = manifest.and_then(|b| list::parse_manifest(&b).ok());
            let version = manifest.map(|m| m.version).unwrap_or_default();
            let to = dir.join(&version);
            let _ = std::fs::remove_dir_all(&to);
            let swapped = std::fs::create_dir_all(&dir)
                .and_then(|()| std::fs::rename(&temp, &to))
                .and_then(|()| std::fs::write(dir.join("current.txt"), &version))
                .and_then(|()| std::fs::remove_file(dir.join("disabled.txt")).or(Ok(())));
            swapped.map_err(|e| (false, e.to_string()))
        });
        let _ = std::fs::remove_dir_all(&temp);
        let fences = self.plugin_fences(id);
        match result {
            Err((refused, why)) => {
                tracing::warn!(
                    id,
                    update,
                    refused,
                    why,
                    "plugins: install or update failed"
                );
                // An automatic update tries again the next day.
                if !update {
                    self.settings_error(if refused {
                        i18n::text("下载的文件和审查过的不一样，已经放弃安装。")
                    } else {
                        i18n::text("下载没有完成，检查网络后再试一次。")
                    });
                }
            }
            Ok(()) if fences.is_empty() && !update => self.add_plugin_fence(id),
            Ok(()) => {
                tracing::info!(
                    id,
                    update,
                    fences = fences.len(),
                    "plugins: installed, its fences start again"
                );
                for fence in fences {
                    self.start_script(fence);
                }
            }
        }
    }

    fn disable_plugin(&mut self, id: &str, why: &str) {
        tracing::warn!(id, why, "plugins: taken down by the index");
        let dir = self.plugins_dir().join(id);
        for fence in self.plugin_fences(id) {
            self.stop_script(fence);
        }
        if let Some(current) = self.plugin_dir(id) {
            let _ = std::fs::remove_file(current.join("main.js"));
            let _ = std::fs::remove_file(current.join("index.html"));
        }
        if let Err(e) = std::fs::write(dir.join("disabled.txt"), why) {
            tracing::warn!(id, error = %e, "plugins: disabled.txt not written");
        }
        for fence in self.plugin_fences(id) {
            self.push_tool_view(self.state.host_of(fence));
        }
    }

    /// A fence for installed plugin `id`, beside the settings window as the examples are.
    fn add_plugin_fence(&mut self, id: &str) {
        let at = self
            .state
            .work_areas
            .first()
            .map(|w| ((w.left + w.right) / 2, (w.top + w.bottom) / 2));
        let Some((x, y)) = at else {
            return;
        };
        self.tool_clear_of = self
            .settings
            .as_ref()
            .map(|h| window::window_rect(h.hwnd()));
        let rect = self.place_new_tool(280.0, 240.0, x, y);
        let (title, tool) = (
            self.plugin_name(id),
            super::tools::NewTool::Script(format!("plugin:{id}")),
        );
        let made = self.create_tool_fence(tool, Some(&title), Some(rect), x, y);
        tracing::info!(id, made = made.is_some(), "plugins: fence added");
        self.tool_clear_of = None;
    }

    /// 升级为插件 (`migrate.rs`): installs plugins `ids` without a fence or a notice, from the
    /// index when it is there; else the index is asked for (its arrival calls again).
    pub(super) fn install_for_migration(&mut self, ids: &[&str]) {
        match self.plugins.status {
            Status::Ready if self.plugins.index.is_some() => {
                for id in ids {
                    self.start_plugin_download(id, true);
                }
            }
            Status::Loading => {}
            _ => self.fetch_plugins(),
        }
    }

    fn uninstall_plugin(&mut self, id: &str) {
        let owner = self
            .settings
            .as_ref()
            .map_or(self.control.hwnd(), |h| h.hwnd());
        let text = i18n::format(
            "卸载“{0}”？它的栅栏和它保存的数据都会删掉。",
            &[self.plugin_name(id)],
        );
        if !window::confirm(owner, &text, i18n::text("卸载")) {
            return;
        }
        let fences = self.plugin_fences(id);
        tracing::info!(id, fences = fences.len(), "plugins: uninstall");
        for fence in fences {
            self.stop_script(fence);
            self.delete_fence(fence);
        }
        for dir in [self.plugins_dir().join(id), self.plugin_data_dir(id)] {
            if let Err(e) = remove_dir(&dir) {
                tracing::warn!(id, dir = %dir.display(), error = %e, "plugins: folder not removed");
            }
        }
        crate::web_plugin::delete_profile(id);
    }

    /// 插件详情 in a plugin fence's menu: the plugin's row on the settings page.
    pub(super) fn show_plugin(&mut self, id: &str) {
        let seq = self.plugins.open.as_ref().map_or(1, |(n, _)| n + 1);
        self.plugins.open = Some((seq, id.to_string()));
        self.open_settings_page("tools", Some("plugins"));
        self.push_settings_state();
    }

    /// The page's actions on the plugin rows.
    pub(super) fn on_plugin_action(&mut self, name: &str, v: &serde_json::Value) {
        let id = v
            .get("id")
            .and_then(|i| i.as_str())
            .filter(|i| list::valid_id(i));
        tracing::info!(action = name, id, "plugins: page action");
        match (name, id) {
            _ if !shown() => return,
            ("pluginsRefresh", _) => self.fetch_plugins(),
            ("pluginInstall", Some(_)) if self.held_scripts_wait() => {
                self.settings_error(super::settings::scripts_held_text())
            }
            ("pluginInstall", Some(id)) => {
                // Like the built-in examples, an install switches 小工具 on.
                self.ensure_tool_on(pecofence_core::FenceTool::Script);
                self.start_plugin_download(id, false);
            }
            ("pluginAdd", Some(id)) if self.plugin_manifest(id).is_some() => {
                self.add_plugin_fence(id)
            }
            ("pluginUninstall", Some(id)) => self.uninstall_plugin(id),
            ("pluginPage", Some(id)) => {
                if let Some(index) = &self.plugins.index {
                    let url = list::page_url(&index.commit, id);
                    let _ = pecofence_platform::shell::shell_execute(Path::new(&url), None, None);
                }
            }
            _ => {}
        }
        self.push_settings_state();
    }

    /// 设置 › 小工具's 社区插件 (null in the Store edition): the installed plugins, then the
    /// others of the index (those taken down left out).
    pub(super) fn plugins_json(&self) -> serde_json::Value {
        if !shown() {
            return serde_json::Value::Null;
        }
        let (lang, app) = (i18n::language().tag(), env!("CARGO_PKG_VERSION"));
        let index = self.plugins.index.as_ref();
        let installed = self.installed_plugins();
        let offered = index.map(|i| i.plugins.as_slice()).unwrap_or_default();
        let others = offered
            .iter()
            .filter(|e| e.disabled.is_none())
            .map(|e| &e.manifest);
        let others = others.filter(|o| !installed.iter().any(|m| m.id == o.id));
        let rows = installed.iter().map(|m| {
            (
                Some(m),
                index.and_then(|i| i.get(&m.id)).map(|e| &e.manifest),
            )
        });
        let rows: Vec<serde_json::Value> = rows
            .chain(others.map(|o| (None, Some(o))))
            .filter_map(|(mine, offered)| {
                let id = mine.or(offered)?.id.as_str();
                let reason = self.plugin_disabled(id).filter(|_| mine.is_some());
                let state = match (mine, offered) {
                    _ if self.plugins.busy.contains(id) => "busy",
                    (Some(_), _) if reason.is_some() => "disabled",
                    (Some(m), Some(o)) if list::update_kind(m, o, app) == Update::Confirm => "confirm",
                    (Some(_), _) if self.plugin_fences(id).is_empty() => "add",
                    (Some(_), _) => "installed",
                    (None, Some(o)) if pecofence_core::updates::is_newer(&o.min_pecofence, app) => "newer",
                    (None, _) => "install",
                };
                // A row offering an update describes what it brings.
                let m = if state == "confirm" { offered? } else { mine.or(offered)? };
                Some(serde_json::json!({
                    "id": id, "name": list::pick(&m.name, lang), "description": list::pick(&m.description, lang),
                    "author": m.author, "version": m.version, "installed": mine.is_some(),
                    "network": m.permissions.network, "run": m.permissions.run, "streamable": m.streamable,
                    "clipboard": format!("{:?}", m.permissions.clipboard).to_lowercase(),
                    "readsFiles": m.permissions.files == list::Files::DataRead,
                    "state": state, "reason": reason, "page": index.is_some(),
                }))
            })
            .collect();
        let status = match self.plugins.status {
            Status::Idle | Status::Loading => "loading",
            Status::Offline => "offline",
            Status::Ready => "ready",
        };
        serde_json::json!({ "status": status, "held": self.held_scripts_wait(), "open": self.plugins.open, "list": rows })
    }
}
