//! 插件 in the app. A plugin is installed in `<config>\plugins\<id>\<version>\`, and
//! `<config>\plugins\<id>\current.txt` names the version in use (writing it is the switch; the
//! version before stays until the next update, to go back to when the new one fails to start).
//! A plugin's fences run `plugin:<id>`, and its data lives in `<config>\plugins-data\<id>\`.
//! The signed index is downloaded when 设置 › 小工具 opens and once a day while a plugin is
//! installed; the last good one is kept in `plugins\index.json`, and its take-downs apply at
//! start, offline too. Downloads run on threads of their own and come back through
//! `drain_plugins`. What is installed is read into `Plugins::installed` at start and after each
//! change, so drawing a plugin fence reads no files.

use super::{App, HWND};
use pecofence_core::FenceId;
use pecofence_core::i18n;
use pecofence_core::plugins::{self as list, Entry, Index, Manifest, Update};
use pecofence_platform::window;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub(super) const TIMER_PLUGINS_CHECK: usize = 101;
/// A test instance's index (`http://127.0.0.1:` only); its files are on the same server. Its
/// signature is checked only when the server has one.
const URL_VAR: &str = "PECOFENCE_PLUGINS_URL";
/// A new version that stops this soon after an update goes back to the version before.
const TRIAL: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Status {
    #[default]
    Idle,
    Loading,
    Offline,
    Ready,
}

/// An installed plugin, as read from its folder.
pub(super) struct Installed {
    /// The `plugin.json` of the version in use (it stays while the plugin is taken down).
    pub(super) manifest: Manifest,
    /// The folder of the version in use.
    pub(super) dir: PathBuf,
    /// Why the index took it down.
    pub(super) disabled: Option<String>,
    /// Its main.js (a web plugin's index.html) is there.
    pub(super) ready: bool,
    /// A version that stopped right after an update: not taken again by itself.
    failed: Option<String>,
}

#[derive(Default)]
pub(super) struct Plugins {
    root: PathBuf,
    index: Option<Index>,
    status: Status,
    /// The installed plugins by id: the one table the fences, the light layer and the settings
    /// page read.
    pub(super) installed: BTreeMap<String, Installed>,
    /// Downloads under way: the plugin and the download's number. A result whose number is no
    /// longer here (uninstalled or taken down meanwhile) is dropped.
    busy: HashMap<String, u64>,
    downloads: u64,
    /// Plugins updated a moment ago: the version they came from, and when.
    trial: HashMap<String, (String, Instant)>,
    /// Plugins gone back to their version before: their fences start again on the next turn,
    /// after the stops of the failed version still waiting are read.
    restart: Vec<String>,
    /// The row the page opens (插件详情, 插件设置 with its fence), numbered so that the page
    /// opens it again after the user closed it.
    open: Option<(u32, String, Option<FenceId>)>,
    done: Arc<Mutex<Vec<Done>>>,
}

enum Done {
    /// The index, and its bytes to keep.
    Index(Result<(Index, Vec<u8>), String>),
    /// A download (its plugin, number, whether the user asked for it): the version, or Err(true,
    /// why) when a file is not the reviewed one, Err(false, why) when the network or the disk
    /// failed.
    Downloaded(String, u64, bool, Result<String, (bool, String)>),
}

/// What a fence's script path names when it is a plugin: `plugin:<id>`.
pub(super) fn plugin_id(path: &str) -> Option<&str> {
    path.strip_prefix("plugin:").filter(|id| list::valid_id(id))
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

/// The index from the first address whose copy is signed by the plugin repository's key (a
/// wrong or missing signature counts as no index there).
fn fetch_index(test: Option<String>) -> Result<(Index, Vec<u8>), String> {
    let urls = test
        .clone()
        .map_or_else(|| list::index_urls().to_vec(), |u| vec![u]);
    let mut errors = Vec::new();
    for url in urls {
        let got = super::updates::get_in_time(&url, "").and_then(|bytes| {
            let sig = super::updates::get_in_time(&format!("{url}.sig"), "");
            match sig {
                Ok(sig) if list::index_signed(&bytes, &sig) => Ok(bytes),
                Err(_) if test.is_some() => Ok(bytes),
                Ok(_) => Err("the signature does not match".to_string()),
                Err(e) => Err(format!("no signature: {e}")),
            }
        });
        match got.and_then(|bytes| Ok((list::parse_index(&bytes)?, bytes))) {
            Ok(((index, dropped), bytes)) => {
                for why in dropped {
                    tracing::warn!(why, "plugins: index entry left out");
                }
                return Ok((index, bytes));
            }
            Err(e) => {
                tracing::warn!(url, error = %e, "plugins: index not taken from this address");
                errors.push(e);
            }
        }
    }
    Err(errors.join("; "))
}

/// Downloads `entry`'s files at the index's `commit` into `dir` (made afresh) and checks each
/// against the index (size, SHA-256) and `plugin.json` against the entry. Ok: the version.
fn download(
    entry: &Entry,
    commit: &str,
    test: Option<&str>,
    dir: &Path,
) -> Result<String, (bool, String)> {
    let io = |e: std::io::Error| (false, e.to_string());
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).map_err(io)?;
    let id = &entry.manifest.id;
    // This version shows no screenshot.
    for file in entry.files.iter().filter(|f| f.name != "screenshot.png") {
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
        // The index checked the name: parts of letters, digits, '.', '_' and '-'.
        let path = dir.join(name);
        let made = path.parent().map_or(Ok(()), std::fs::create_dir_all);
        made.and_then(|()| std::fs::write(&path, &bytes))
            .map_err(io)?;
    }
    let manifest = std::fs::read(dir.join("plugin.json")).map_err(|e| e.to_string());
    let manifest = manifest
        .and_then(|b| list::parse_manifest(&b))
        .map_err(|e| (true, e))?;
    if manifest != entry.manifest {
        return Err((true, "plugin.json differs from the index".into()));
    }
    Ok(manifest.version)
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

fn read_text(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    Some(text.trim().to_string())
}

/// Plugin `id` as its folder in `root` holds it (None: not installed).
fn read_installed(root: &Path, id: &str) -> Option<Installed> {
    let dir = root.join(id);
    let version = read_text(&dir.join("current.txt"))?;
    pecofence_core::updates::parse_version(&version)?;
    let vdir = dir.join(&version);
    let manifest = list::parse_manifest(&std::fs::read(vdir.join("plugin.json")).ok()?).ok();
    let manifest = manifest.filter(|m| m.id == id && m.version == version)?;
    let main = if manifest.layer == "web" {
        "index.html"
    } else {
        "main.js"
    };
    Some(Installed {
        ready: vdir.join(main).is_file(),
        dir: vdir,
        disabled: read_text(&dir.join("disabled.txt")),
        failed: read_text(&dir.join("failed.txt")),
        manifest,
    })
}

/// Takes plugin folder `dir` down: its code goes (all versions; the `plugin.json` in use stays
/// for the name and 卸载), and `disabled.txt` says why.
fn disable_files(dir: &Path, why: &str) {
    let current = read_text(&dir.join("current.txt")).unwrap_or_default();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if entry.file_name().to_string_lossy() != current {
            let _ = std::fs::remove_dir_all(&path);
            continue;
        }
        for file in std::fs::read_dir(&path).into_iter().flatten().flatten() {
            let p = file.path();
            if file.file_name() != "plugin.json" {
                let _ = std::fs::remove_dir_all(&p).or_else(|_| std::fs::remove_file(&p));
            }
        }
    }
    if let Err(e) = std::fs::write(dir.join("disabled.txt"), why) {
        tracing::warn!(dir = %dir.display(), error = %e, "plugins: disabled.txt not written");
    }
}

impl Plugins {
    /// The installed plugins next to configuration `config`, and the last good index. A plugin
    /// that index takes down is taken down now (no fence runs yet).
    pub(super) fn load(config: &Path) -> Self {
        let root = config.parent().unwrap_or(Path::new(".")).join("plugins");
        let saved = std::fs::read(root.join("index.json")).ok();
        let index = saved
            .and_then(|b| list::parse_index(&b).ok())
            .map(|(i, _)| i);
        let mut p = Plugins {
            root,
            index,
            ..Default::default()
        };
        let dirs = std::fs::read_dir(&p.root).into_iter().flatten().flatten();
        let names: Vec<String> = dirs
            .map(|d| d.file_name().to_string_lossy().to_string())
            .collect();
        // A download that PecoFence quit during left its folder.
        for temp in names.iter().filter(|n| n.starts_with(".new-")) {
            let _ = std::fs::remove_dir_all(p.root.join(temp));
        }
        let ids = names.into_iter().filter(|id| list::valid_id(id));
        for id in ids {
            p.reload(&id);
            let why = p.index.as_ref().and_then(|i| i.get(&id)?.disabled.clone());
            if let Some(why) =
                why.filter(|_| p.installed.get(&id).is_some_and(|i| i.disabled.is_none()))
            {
                tracing::warn!(id, why, "plugins: taken down by the saved index");
                disable_files(&p.root.join(&id), &why);
                p.reload(&id);
            }
        }
        tracing::info!(
            installed = p.installed.len(),
            saved_index = p.index.is_some(),
            "plugins: read"
        );
        p
    }

    /// Reads plugin `id` from its folder again (after an install, update, take-down or
    /// uninstall).
    fn reload(&mut self, id: &str) {
        match read_installed(&self.root, id) {
            Some(p) => self.installed.insert(id.to_string(), p),
            None => self.installed.remove(id),
        };
    }
}

impl App {
    fn plugins_dir(&self) -> PathBuf {
        self.plugins.root.clone()
    }

    pub(super) fn plugin_data_dir(&self, id: &str) -> PathBuf {
        let config = self.state.config_path_on_disk();
        config
            .parent()
            .unwrap_or(Path::new("."))
            .join("plugins-data")
            .join(id)
    }

    /// The installed plugin `id` (the in-memory table; no file is read).
    pub(super) fn plugin(&self, id: &str) -> Option<&Installed> {
        self.plugins.installed.get(id)
    }

    /// The file a fence's script path runs: a plugin's `main.js` (a web plugin's `index.html`),
    /// or the path itself.
    pub(super) fn script_file(&self, path: &str) -> PathBuf {
        match plugin_id(path) {
            Some(id) => self.plugin(id).map_or_else(PathBuf::new, |p| {
                let web = p.manifest.layer == "web";
                p.dir.join(if web { "index.html" } else { "main.js" })
            }),
            None => PathBuf::from(path),
        }
    }

    /// 网页层: the installed web plugin fence `fence` shows (none when it is taken down).
    pub(super) fn web_plugin_of(&self, fence: FenceId) -> Option<crate::web_plugin::Plugin> {
        let spec = self.state.config.scripts.get(&fence)?;
        let id = plugin_id(&spec.path)?;
        let p = self.plugin(id).filter(|p| p.manifest.layer == "web")?;
        (p.ready && p.disabled.is_none()).then(|| crate::web_plugin::Plugin {
            id: id.to_string(),
            dir: p.dir.clone(),
            network: p.manifest.permissions.network.clone(),
        })
    }

    /// `Command::StartWebFence`: the page of the web plugin the window of `host` shows.
    pub(super) fn start_web_fence(&mut self, host: FenceId) {
        let Some(shown) = self.fences.get(&host).map(|w| w.active_fence()) else {
            return;
        };
        let Some(plugin) = self.web_plugin_of(shown) else {
            return;
        };
        if self
            .fences
            .get(&host)
            .and_then(|w| w.web_plugin())
            .as_deref()
            == Some(&plugin.id)
        {
            return;
        }
        // Made once, as for the settings window (it pumps messages until the browser is there).
        if !self.ensure_web_env() {
            return;
        }
        let (Some(w), Some(env)) = (self.fences.get(&host), self.web_env.as_ref()) else {
            return;
        };
        let web =
            crate::web_plugin::WebFence::start(&env.env, &self.ctx.stack, w.hwnd(), shown, plugin);
        w.set_web(Some(web));
    }

    /// The installed plugin's `plugin.json` (it stays while the plugin is taken down).
    pub(super) fn plugin_manifest(&self, id: &str) -> Option<Manifest> {
        self.plugin(id).map(|p| p.manifest.clone())
    }

    fn plugin_fences(&self, id: &str) -> Vec<FenceId> {
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
        match self.plugin(id) {
            Some(Installed {
                disabled: Some(why),
                ..
            }) => Some(i18n::format("已下架：{0}", std::slice::from_ref(why))),
            Some(p) if p.ready => None,
            _ => Some(
                i18n::text("这个插件还没有安装。右键这个栅栏，选“插件详情”去安装。").to_string(),
            ),
        }
    }

    /// The name a plugin fence's footer and title show.
    pub(super) fn plugin_name(&self, id: &str) -> String {
        let lang = i18n::language().tag();
        let name = self.plugin(id).map(|p| list::pick(&p.manifest.name, lang));
        name.unwrap_or(id).to_string()
    }

    /// Whether plugin `id` may start programs: it declares them in `system`, and
    /// 允许插件使用 Windows 底层功能 is on.
    pub(super) fn plugin_may_run(&self, id: &str) -> bool {
        self.state.config.settings.tools.allow_system
            && self
                .plugin(id)
                .is_some_and(|p| !p.manifest.permissions.system.is_empty())
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
        let installed = self.plugins.installed.len();
        tracing::info!(installed, "plugins: daily check");
        if installed > 0 {
            self.fetch_plugins();
        }
    }

    /// 设置 › 小工具 was shown.
    pub(super) fn plugins_page_shown(&mut self) {
        self.fetch_plugins();
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
        for id in std::mem::take(&mut self.plugins.restart) {
            for fence in self.plugin_fences(&id) {
                self.stop_script(fence);
                self.start_script(fence);
            }
        }
        let done = match self.plugins.done.lock() {
            Ok(mut d) if !d.is_empty() => std::mem::take(&mut *d),
            _ => return,
        };
        for d in done {
            match d {
                Done::Index(Ok((index, bytes))) => {
                    tracing::info!(plugins = index.plugins.len(), commit = %index.commit, "plugins: index read");
                    let saved = std::fs::create_dir_all(&self.plugins.root)
                        .and_then(|()| std::fs::write(self.plugins.root.join("index.json"), bytes));
                    if let Err(e) = saved {
                        tracing::warn!(error = %e, "plugins: index not saved");
                    }
                    self.plugins.index = Some(index);
                    self.plugins.status = Status::Ready;
                    self.apply_plugin_index();
                }
                Done::Index(Err(e)) => {
                    tracing::warn!(error = %e, "plugins: index not read");
                    self.plugins.status = Status::Offline;
                }
                Done::Downloaded(id, n, by_user, result) => {
                    self.plugin_downloaded(&id, n, by_user, result)
                }
            }
        }
        self.push_settings_state();
    }

    /// Takes down what the index marks disabled and updates what may update by itself.
    fn apply_plugin_index(&mut self) {
        let app = env!("CARGO_PKG_VERSION");
        let ids: Vec<String> = self.plugins.installed.keys().cloned().collect();
        for id in ids {
            let entry = self.plugins.index.as_ref().and_then(|i| i.get(&id));
            let (Some(entry), Some(mine)) = (entry, self.plugins.installed.get(&id)) else {
                continue;
            };
            let kind = list::update_kind(&mine.manifest, &entry.manifest, app);
            let offered = entry.manifest.version.clone();
            let failed = mine.failed.as_deref() == Some(offered.as_str());
            let (why, disabled) = (entry.disabled.clone(), mine.disabled.is_some());
            tracing::info!(
                id,
                installed = mine.manifest.version,
                offered,
                ?kind,
                disabled,
                failed,
                "plugins: update check"
            );
            match why {
                Some(why) if !disabled => self.disable_plugin(&id, &why),
                None if kind == Update::Auto && !disabled && !failed => {
                    self.start_plugin_download(&id, false)
                }
                _ => {}
            }
        }
    }

    /// Downloads plugin `id` from the index: an install (`by_user`: the page's 安装 or 更新插件;
    /// else an automatic update or a tool brought over from an older version, which adds no
    /// fence and shows no error).
    pub(super) fn start_plugin_download(&mut self, id: &str, by_user: bool) {
        let Some(index) = &self.plugins.index else {
            return;
        };
        let (Some(entry), commit) = (index.get(id).cloned(), index.commit.clone()) else {
            return;
        };
        if self.plugins.busy.contains_key(id) {
            return;
        }
        self.plugins.downloads += 1;
        let n = self.plugins.downloads;
        self.plugins.busy.insert(id.to_string(), n);
        let temp = self.plugins_dir().join(format!(".new-{id}-{n}"));
        let test = test_url(self.instance.is_some());
        let id = id.to_string();
        tracing::info!(
            id,
            n,
            by_user,
            version = entry.manifest.version,
            "plugins: download"
        );
        self.run_plugin_job(move || {
            let result = download(&entry, &commit, test.as_deref(), &temp);
            Done::Downloaded(id, n, by_user, result)
        });
    }

    fn plugin_downloaded(
        &mut self,
        id: &str,
        n: u64,
        by_user: bool,
        result: Result<String, (bool, String)>,
    ) {
        let temp = self.plugins_dir().join(format!(".new-{id}-{n}"));
        if self.plugins.busy.get(id) != Some(&n) {
            tracing::info!(
                id,
                n,
                "plugins: a cancelled download ended, its files are dropped"
            );
            let _ = std::fs::remove_dir_all(&temp);
            return;
        }
        self.plugins.busy.remove(id);
        let before = self.plugin(id).map(|p| p.manifest.version.clone());
        let result =
            result.and_then(|version| self.switch_plugin(id, &temp, &version, before.as_deref()));
        let _ = std::fs::remove_dir_all(&temp);
        let fences = self.plugin_fences(id);
        match result {
            Err((refused, why)) => {
                tracing::warn!(
                    id,
                    by_user,
                    refused,
                    why,
                    "plugins: install or update failed"
                );
                // An automatic update tries again the next day.
                if by_user {
                    self.settings_error(if refused {
                        i18n::text("下载的文件和审查过的不一样，已经放弃安装。")
                    } else {
                        i18n::text("下载没有完成，检查网络后再试一次。")
                    });
                }
            }
            Ok(()) if fences.is_empty() && by_user && before.is_none() => self.add_plugin_fence(id),
            Ok(()) => {
                tracing::info!(
                    id,
                    by_user,
                    fences = fences.len(),
                    "plugins: installed, its fences start again"
                );
                for fence in fences {
                    self.start_script(fence);
                }
            }
        }
    }

    /// Moves a checked download into its version's folder and switches to it (current.txt),
    /// with the plugin's fences stopped. The version before stays to go back to; older ones go.
    fn switch_plugin(
        &mut self,
        id: &str,
        temp: &Path,
        version: &str,
        before: Option<&str>,
    ) -> Result<(), (bool, String)> {
        if before.is_some_and(|b| pecofence_core::updates::is_newer(b, version)) {
            return Err((
                true,
                format!("{version} is lower than the installed {before:?}"),
            ));
        }
        for fence in self.plugin_fences(id) {
            self.stop_script(fence);
        }
        let dir = self.plugins_dir().join(id);
        let to = dir.join(version);
        let _ = std::fs::remove_dir_all(&to);
        let swapped = std::fs::create_dir_all(&dir)
            .and_then(|()| std::fs::rename(temp, &to))
            .and_then(|()| std::fs::write(dir.join("current.txt"), version));
        swapped.map_err(|e| (false, e.to_string()))?;
        for name in ["disabled.txt", "failed.txt"] {
            let _ = std::fs::remove_file(dir.join(name));
        }
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.path().is_dir() && name != version && Some(name.as_str()) != before {
                let _ = std::fs::remove_dir_all(entry.path());
            }
        }
        match before.filter(|b| *b != version) {
            Some(b) => self
                .plugins
                .trial
                .insert(id.to_string(), (b.to_string(), Instant::now())),
            None => self.plugins.trial.remove(id),
        };
        self.plugins.reload(id);
        tracing::info!(id, version, ?before, "plugins: switched to the new version");
        Ok(())
    }

    /// A plugin fence stopped by itself (an error, too long, too much memory). In the first
    /// minute after an update the plugin goes back to the version before, and that stays until
    /// a version newer than the failed one comes.
    pub(super) fn plugin_stopped(&mut self, fence: FenceId) {
        let spec = self.state.config.scripts.get(&fence);
        let Some(id) = spec.and_then(|s| plugin_id(&s.path)).map(str::to_string) else {
            return;
        };
        let Some((before, at)) = self.plugins.trial.remove(&id) else {
            return;
        };
        let dir = self.plugins_dir().join(&id);
        let failed = self
            .plugin(&id)
            .map(|p| p.manifest.version.clone())
            .unwrap_or_default();
        let back = at.elapsed() < TRIAL && dir.join(&before).join("plugin.json").is_file();
        tracing::warn!(
            id,
            failed,
            before,
            back,
            "plugins: the new version stopped after its update"
        );
        if !back {
            return;
        }
        let written = std::fs::write(dir.join("current.txt"), &before)
            .and_then(|()| std::fs::write(dir.join("failed.txt"), &failed));
        if let Err(e) = written {
            return tracing::warn!(id, error = %e, "plugins: could not go back");
        }
        let _ = std::fs::remove_dir_all(dir.join(&failed));
        self.plugins.reload(&id);
        self.plugins.restart.push(id);
        window::post_message(self.control.hwnd(), super::WM_APP_COMMAND, 0, 0);
    }

    fn disable_plugin(&mut self, id: &str, why: &str) {
        tracing::warn!(id, why, "plugins: taken down by the index");
        self.plugins.busy.remove(id);
        self.plugins.trial.remove(id);
        for fence in self.plugin_fences(id) {
            self.stop_script(fence);
        }
        disable_files(&self.plugins_dir().join(id), why);
        self.plugins.reload(id);
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

    /// 新建便签 in the fence, desktop and tray menus (`notes`): a fence of plugin `id` near
    /// (`x`, `y`) when it is installed, else 设置 › 小工具 at its row, to install it there.
    pub(super) fn new_plugin_fence(&mut self, id: &str, x: i32, y: i32) {
        let ready = self
            .plugin(id)
            .is_some_and(|p| p.ready && p.disabled.is_none());
        tracing::info!(id, ready, "plugins: new fence from a menu");
        if !ready {
            return self.show_plugin(id, None);
        }
        let rect = self.place_new_tool(280.0, 240.0, x, y);
        let (title, tool) = (
            self.plugin_name(id),
            super::tools::NewTool::Script(format!("plugin:{id}")),
        );
        self.create_tool_fence(tool, Some(&title), Some(rect), x, y);
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
        let cancelled = self.plugins.busy.remove(id);
        tracing::info!(id, fences = fences.len(), ?cancelled, "plugins: uninstall");
        self.plugins.trial.remove(id);
        for fence in fences {
            self.stop_script(fence);
            self.delete_fence(fence);
        }
        crate::web_plugin::delete_profile(id);
        for dir in [self.plugins_dir().join(id), self.plugin_data_dir(id)] {
            if let Err(e) = remove_dir(&dir) {
                tracing::warn!(id, dir = %dir.display(), error = %e, "plugins: folder not removed");
            }
        }
        self.plugins.reload(id);
    }

    /// 插件详情 and 插件设置 in a plugin fence's menu: the plugin's row on the settings page,
    /// open, at `fence`'s settings for 插件设置.
    pub(super) fn show_plugin(&mut self, id: &str, fence: Option<FenceId>) {
        let seq = self.plugins.open.as_ref().map_or(1, |(n, ..)| n + 1);
        self.plugins.open = Some((seq, id.to_string(), fence));
        self.open_settings_page("tools", Some("plugins"));
        self.push_settings_state();
    }

    /// A setting of a plugin fence changed on the page: saved with the fence, and its plugin
    /// gets `settings`.
    fn set_plugin_setting(&mut self, v: &serde_json::Value) {
        let fence = v.get("fence").and_then(|f| f.as_str());
        let fence = fence.and_then(|f| uuid::Uuid::parse_str(f).ok());
        let (key, value) = (v.get("key").and_then(|k| k.as_str()), v.get("value"));
        let spec = fence.and_then(|f| self.state.config.scripts.get(&f));
        let m = spec.and_then(|s| self.plugin(plugin_id(&s.path)?));
        let (Some(fence), Some(m), Some(key), Some(value)) = (fence, m, key, value) else {
            return tracing::warn!(?fence, key, "plugins: setting for no plugin fence");
        };
        let fits = m.manifest.settings.iter().find(|s| s.key == key);
        if !fits.is_some_and(|s| list::fits(s, value)) {
            return tracing::warn!(%fence, key, "plugins: setting refused, not of its kind");
        }
        let m = m.manifest.clone();
        let Some(spec) = self.state.config.scripts.get_mut(&fence) else {
            return;
        };
        spec.settings.insert(key.to_string(), value.clone());
        let settings = list::fence_settings(&m, &spec.settings);
        tracing::info!(%fence, id = m.id, key, "plugins: fence setting changed");
        self.state.mark_dirty();
        self.schedule_save();
        self.plugin_event(fence, "settings", settings);
    }

    /// The page's actions on the plugin rows.
    pub(super) fn on_plugin_action(&mut self, name: &str, v: &serde_json::Value) {
        let id = v
            .get("id")
            .and_then(|i| i.as_str())
            .filter(|i| list::valid_id(i));
        tracing::info!(action = name, id, "plugins: page action");
        match (name, id) {
            ("pluginsRefresh", _) => self.fetch_plugins(),
            ("pluginInstall", Some(_)) if self.held_scripts_wait() => {
                self.settings_error(super::settings::scripts_held_text())
            }
            ("pluginInstall", Some(id)) => {
                // Like the built-in examples, an install switches 小工具 on.
                self.ensure_tool_on(pecofence_core::FenceTool::Script);
                self.start_plugin_download(id, true);
            }
            ("pluginAdd", Some(id)) if self.plugin(id).is_some() => self.add_plugin_fence(id),
            ("pluginUninstall", Some(id)) => self.uninstall_plugin(id),
            ("pluginSetting", _) => self.set_plugin_setting(v),
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

    /// 设置 › 小工具's plugin list: the installed plugins, then the others of the index (those
    /// taken down left out), each with what it may do and, when installed, its fences'
    /// settings.
    pub(super) fn plugins_json(&self) -> serde_json::Value {
        let (lang, app) = (i18n::language().tag(), env!("CARGO_PKG_VERSION"));
        let index = self.plugins.index.as_ref();
        let installed = &self.plugins.installed;
        let offered = index.map(|i| i.plugins.as_slice()).unwrap_or_default();
        let others = offered
            .iter()
            .filter(|e| e.disabled.is_none() && !installed.contains_key(&e.manifest.id))
            .map(|e| (None, Some(&e.manifest)));
        let rows = installed.values().map(|p| {
            let offered = index.and_then(|i| i.get(&p.manifest.id));
            (Some(p), offered.map(|e| &e.manifest))
        });
        let texts = |t: &BTreeMap<String, String>| list::pick(t, lang).to_string();
        let rows: Vec<serde_json::Value> = rows
            .chain(others)
            .filter_map(|(mine, offered)| {
                let id = mine.map(|p| &p.manifest).or(offered)?.id.as_str();
                let reason = mine.and_then(|p| p.disabled.clone());
                let state = match (mine, offered) {
                    _ if self.plugins.busy.contains_key(id) => "busy",
                    (Some(_), _) if reason.is_some() => "disabled",
                    (Some(p), Some(o)) if list::update_kind(&p.manifest, o, app) == Update::Confirm => "confirm",
                    (Some(_), _) if self.plugin_fences(id).is_empty() => "add",
                    (Some(_), _) => "installed",
                    (None, Some(o)) if list::needs_newer_app(o, app) => "newer",
                    (None, _) => "install",
                };
                // A row offering an update describes what it brings.
                let m = if state == "confirm" { offered? } else { mine.map(|p| &p.manifest).or(offered)? };
                let p = &m.permissions;
                // The settings form of each fence of the installed plugin.
                let form = mine.map(|mine| {
                    let settings: Vec<_> = mine.manifest.settings.iter().map(|s| serde_json::json!({
                        "key": s.key, "type": s.kind, "label": texts(&s.label),
                        "options": s.options.iter().map(|o| serde_json::json!({ "value": o.value, "label": texts(&o.label) })).collect::<Vec<_>>(),
                    })).collect();
                    let fences: Vec<_> = self.plugin_fences(id).into_iter().map(|f| {
                        let saved = self.state.config.scripts.get(&f).map(|s| s.settings.clone()).unwrap_or_default();
                        let title = self.state.fence(f).map(|f| f.title.clone()).unwrap_or_default();
                        serde_json::json!({ "id": f, "title": title, "values": list::fence_settings(&mine.manifest, &saved) })
                    }).collect();
                    (settings, fences)
                });
                let (settings, fences) = form.unwrap_or_default();
                Some(serde_json::json!({
                    "id": id, "name": texts(&m.name), "description": texts(&m.description),
                    "author": m.author, "version": m.version, "installed": mine.is_some(), "layer": m.layer,
                    "network": p.network, "links": p.links, "system": p.system, "background": p.background,
                    "folders": p.folders, "notify": p.notify,
                    "clipboard": format!("{:?}", p.clipboard).to_lowercase(),
                    "state": state, "reason": reason, "page": index.is_some(),
                    "settings": settings, "fences": fences,
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
