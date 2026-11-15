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
use std::collections::{BTreeMap, HashMap, HashSet};
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

impl Installed {
    /// Its main file is there and the index has not taken it down: a fence can run it.
    pub(super) fn usable(&self) -> bool {
        self.ready && self.disabled.is_none()
    }
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
    /// The plugin whose install warning the page shows again (the index changed what it may do
    /// while the warning was open): in the one state sent right after the refused 安装.
    ask: Option<String>,
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
/// wrong or missing signature, or a serial lower than `serial` of the index kept, counts as no
/// index there).
fn fetch_index(test: Option<String>, serial: u64) -> Result<(Index, Vec<u8>), String> {
    let urls = test
        .clone()
        .map_or_else(|| list::index_urls().to_vec(), |u| vec![u]);
    let mut errors = Vec::new();
    for url in urls {
        let got = super::updates::get_in_time(&url, "").and_then(|bytes| {
            let sig = super::updates::get_in_time(&format!("{url}.sig"), "");
            match sig {
                Ok(sig) if list::signed_by(&list::INDEX_KEY, &bytes, &sig) => Ok(bytes),
                Err(_) if test.is_some() => Ok(bytes),
                Ok(_) => Err("the signature does not match".to_string()),
                Err(e) => Err(format!("no signature: {e}")),
            }
        });
        let got = got.and_then(|bytes| Ok((list::parse_index(&bytes)?, bytes)));
        let got = got.and_then(|((index, dropped), bytes)| match index.serial {
            older if older < serial => Err(format!(
                "an older index (serial {older}, the one kept has {serial})"
            )),
            _ => Ok(((index, dropped), bytes)),
        });
        match got {
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

/// Writes `path` as `<path>.new` and renames that over it: a failure, or PecoFence ending halfway,
/// leaves the file before whole.
fn write_whole(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut temp = path.as_os_str().to_owned();
    temp.push(".new");
    let mut f = std::fs::File::create(&temp)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    drop(f);
    std::fs::rename(&temp, path)
}

/// Names `version` the one in use (`current.txt`, written whole: a failure leaves the version
/// before in use).
fn write_current(dir: &Path, version: &str) -> std::io::Result<()> {
    write_whole(&dir.join("current.txt"), version.as_bytes())
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
    Some(Installed {
        ready: vdir.join(manifest.main_file()).is_file(),
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
    /// `plugins-data` beside the configuration: a folder per plugin id for its data.
    pub(super) fn plugins_data(&self) -> PathBuf {
        let config = self.state.config_path_on_disk();
        config
            .parent()
            .unwrap_or(Path::new("."))
            .join("plugins-data")
    }

    /// The installed plugin `id` (the in-memory table; no file is read).
    pub(super) fn plugin(&self, id: &str) -> Option<&Installed> {
        self.plugins.installed.get(id)
    }

    /// The file a fence's script path runs: a plugin's `main.js` (a web plugin's `index.html`),
    /// or the path itself.
    pub(super) fn script_file(&self, path: &str) -> PathBuf {
        match plugin_id(path) {
            Some(id) => self
                .plugin(id)
                .map_or_else(PathBuf::new, |p| p.dir.join(p.manifest.main_file())),
            None => PathBuf::from(path),
        }
    }

    /// 网页层: the installed web plugin fence `fence` shows (none when it is taken down).
    pub(super) fn web_plugin_of(&self, fence: FenceId) -> Option<crate::web_plugin::Plugin> {
        let spec = self.state.config.scripts.get(&fence)?;
        let id = plugin_id(&spec.path)?;
        let p = self.plugin(id).filter(|p| p.manifest.layer == "web")?;
        p.usable().then(|| crate::web_plugin::Plugin {
            id: id.to_string(),
            dir: p.dir.clone(),
            network: p.manifest.permissions.network.clone(),
            background: p.manifest.permissions.background,
            clipboard_read: p.manifest.permissions.clipboard.reads(),
        })
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
        self.migrate_tools(true);
    }

    /// Asks for the plugin index (also each time 设置 › 小工具 is shown).
    pub(super) fn fetch_plugins(&mut self) {
        if self.plugins.status == Status::Loading {
            return;
        }
        self.plugins.status = Status::Loading;
        let test = test_url(self.instance.is_some());
        let serial = self.plugins.index.as_ref().map_or(0, |i| i.serial);
        self.run_plugin_job(move || Done::Index(fetch_index(test, serial)));
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
                self.restart_web_page(fence);
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
                    tracing::info!(plugins = index.plugins.len(), serial = index.serial, commit = %index.commit, "plugins: index read");
                    // Written whole: a torn file would read as no index at the next start, and an
                    // older index served again would no longer be refused by its serial.
                    let saved = std::fs::create_dir_all(&self.plugins.root)
                        .and_then(|()| write_whole(&self.plugins.root.join("index.json"), &bytes));
                    if let Err(e) = saved {
                        tracing::warn!(error = %e, "plugins: index not saved");
                    }
                    self.plugins.index = Some(index);
                    self.plugins.status = Status::Ready;
                    self.apply_plugin_index();
                    self.migrate_tools(true);
                }
                Done::Index(Err(e)) => {
                    tracing::warn!(error = %e, "plugins: index not read");
                    self.plugins.status = Status::Offline;
                }
                Done::Downloaded(id, n, by_user, result) => {
                    self.plugin_downloaded(&id, n, by_user, result);
                    self.migrate_tools(false);
                }
            }
        }
        self.push_settings_state();
    }

    /// Takes down what the index marks disabled and updates what may update by itself.
    fn apply_plugin_index(&mut self) {
        let app = env!("CARGO_PKG_VERSION");
        // A first install under way of a plugin taken down now: its result will be dropped.
        let index = self.plugins.index.as_ref();
        let down: Vec<String> = (self.plugins.busy.keys())
            .filter(|id| {
                index
                    .and_then(|i| i.get(id))
                    .is_some_and(|e| e.disabled.is_some())
            })
            .cloned()
            .collect();
        for id in down {
            tracing::warn!(
                id,
                "plugins: taken down by the index during its download, cancelled"
            );
            self.plugins.busy.remove(&id);
        }
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
            // Its code is gone (a take-down took it, a download after the listing again failed):
            // it comes again whatever failed.txt says, until it is there.
            let gone = !mine.ready;
            tracing::info!(
                id,
                installed = mine.manifest.version,
                offered,
                ?kind,
                disabled,
                failed,
                gone,
                "plugins: update check"
            );
            match why {
                Some(why) if !disabled => self.disable_plugin(&id, &why),
                None if disabled => self.relist_plugin(&id, kind, failed),
                None if gone || (kind == Update::Auto && !failed) => {
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
        let mine = self.plugin(id);
        let app = env!("CARGO_PKG_VERSION");
        // Not asked for by the user, an installed plugin changes only as an automatic update
        // does, or comes again as it was (its code gone, the same version as it was OK'd).
        let unasked = |p: &Installed| {
            let same = !p.ready && list::shown_key(&p.manifest) == list::shown_key(&entry.manifest);
            list::update_kind(&p.manifest, &entry.manifest, app) != Update::Auto && !same
        };
        let refused = if entry.disabled.is_some() || mine.is_some_and(|p| p.disabled.is_some()) {
            Some("taken down")
        } else if list::needs_newer_app(&entry.manifest, app) {
            Some("needs a newer PecoFence")
        } else if mine.is_some_and(|p| p.ready && p.manifest.version == entry.manifest.version) {
            Some("this version is installed")
        } else if !by_user && mine.is_some_and(unasked) {
            Some("not asked for, and more than an automatic update")
        } else {
            None
        };
        if let Some(why) = refused {
            let version = &entry.manifest.version;
            return tracing::warn!(id, by_user, version, why, "plugins: download refused");
        }
        self.plugins.downloads += 1;
        let n = self.plugins.downloads;
        self.plugins.busy.insert(id.to_string(), n);
        let temp = self.plugins.root.join(format!(".new-{id}-{n}"));
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
        let temp = self.plugins.root.join(format!(".new-{id}-{n}"));
        let ours = self.plugins.busy.get(id) == Some(&n);
        if ours {
            self.plugins.busy.remove(id);
        }
        // Checked again right before the switch: a plugin the index takes down is not installed.
        let index = self.plugins.index.as_ref();
        let down = index
            .and_then(|i| i.get(id))
            .is_some_and(|e| e.disabled.is_some())
            || self.plugin(id).is_some_and(|p| p.disabled.is_some());
        if !ours || down {
            tracing::info!(
                id,
                n,
                down,
                "plugins: a cancelled download ended, its files are dropped"
            );
            let _ = std::fs::remove_dir_all(&temp);
            return;
        }
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
                // A web plugin's page, its rules and its main.js are the new version's.
                for fence in fences {
                    self.restart_web_page(fence);
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
        let fences = self.plugin_fences(id);
        for fence in &fences {
            self.stop_script(*fence);
        }
        let dir = self.plugins.root.join(id);
        let to = dir.join(version);
        let _ = std::fs::remove_dir_all(&to);
        let swapped = std::fs::create_dir_all(&dir)
            .and_then(|()| std::fs::rename(temp, &to))
            .and_then(|()| write_current(&dir, version));
        if let Err(e) = swapped {
            // The version before is still the one in use: its fences run it again.
            tracing::warn!(id, version, error = %e, fences = fences.len(), "plugins: switch failed, the fences start again");
            self.plugins.reload(id);
            for fence in fences {
                // A web plugin's main.js starts with its page.
                self.restart_web_page(fence);
                self.start_script(fence);
            }
            return Err((false, e.to_string()));
        }
        let _ = std::fs::remove_file(dir.join("failed.txt"));
        if before.is_none() {
            // A web plugin installed again keeps the profile its uninstall could not delete.
            crate::web_plugin::keep_profile(id);
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
        let dir = self.plugins.root.join(&id);
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
        if let Err(e) = write_current(&dir, &before) {
            return tracing::warn!(id, error = %e, "plugins: could not go back");
        }
        // The version before is in use from here on: its fences start again whatever comes.
        if let Err(e) = std::fs::write(dir.join("failed.txt"), &failed) {
            tracing::warn!(id, failed, error = %e, "plugins: failed.txt not written, the next check may offer the failed version again");
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
            self.close_plugin_input(fence);
            self.stop_script(fence);
        }
        disable_files(&self.plugins.root.join(id), why);
        self.plugins.reload(id);
        for fence in self.plugin_fences(id) {
            self.push_tool_view(self.state.host_of(fence));
        }
    }

    /// The index lists plugin `id` again after taking it down: `disabled.txt` goes, and its code
    /// comes back as an update would (by itself unless it asks for more; its fences and their
    /// settings stayed). Its code went with the take-down: it comes again at every index read
    /// until it is there (`apply_plugin_index`), whatever failed.txt says.
    fn relist_plugin(&mut self, id: &str, kind: Update, failed: bool) {
        let removed = std::fs::remove_file(self.plugins.root.join(id).join("disabled.txt"));
        self.plugins.reload(id);
        let gone = self.plugin(id).is_some_and(|p| !p.ready);
        let again = removed.is_ok() && (gone || (kind == Update::Auto && !failed));
        tracing::info!(id, ?kind, failed, gone, again, error = ?removed.err(), "plugins: listed again after a take-down");
        if again {
            self.start_plugin_download(id, false);
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
        let made = self.plugin_fence_at(id, x, y);
        tracing::info!(id, made = made.is_some(), "plugins: fence added");
        self.tool_clear_of = None;
    }

    /// A fence of plugin `id` titled after it, placed near (`x`, `y`).
    fn plugin_fence_at(&mut self, id: &str, x: i32, y: i32) -> Option<FenceId> {
        let rect = self.place_new_tool(280.0, 240.0, x, y);
        let title = self.plugin_name(id);
        let tool = super::tools::NewTool::Script(format!("plugin:{id}"));
        self.create_tool_fence(tool, Some(&title), Some(rect), x, y)
    }

    /// 升级为插件 (`migrate.rs`): installs plugins `ids` without a fence or a notice, from the
    /// index when it is there; else the index is asked for (its arrival calls again).
    pub(super) fn install_for_migration(&mut self, ids: &[&str]) {
        match self.plugins.status {
            Status::Ready if self.plugins.index.is_some() => {
                for id in ids {
                    self.start_plugin_download(id, false);
                }
            }
            Status::Loading => {}
            _ => self.fetch_plugins(),
        }
    }

    /// 新建便签 in the fence, desktop and tray menus (`notes`): a fence of plugin `id` near
    /// (`x`, `y`) when it is installed, else 设置 › 小工具 at its row, to install it there.
    pub(super) fn new_plugin_fence(&mut self, id: &str, x: i32, y: i32) {
        let ready = self.plugin(id).is_some_and(Installed::usable);
        tracing::info!(id, ready, "plugins: new fence from a menu");
        if !ready {
            return self.show_plugin(id, None);
        }
        self.plugin_fence_at(id, x, y);
    }

    fn uninstall_plugin(&mut self, id: &str) {
        let owner = self
            .settings
            .as_ref()
            .map_or(self.control.hwnd(), |h| h.hwnd());
        let fences = self.plugin_fences(id);
        let notes = self.state.config.notes.iter();
        let notes = notes
            .filter(|(f, text)| fences.contains(f) && !text.trim().is_empty())
            .count();
        let text = if notes > 0 {
            i18n::format(
                "卸载“{0}”？它的栅栏和它保存的数据都会删掉，包括 {1} 个便签里的文字。",
                &[self.plugin_name(id), notes.to_string()],
            )
        } else {
            i18n::format(
                "卸载“{0}”？它的栅栏和它保存的数据都会删掉。",
                &[self.plugin_name(id)],
            )
        };
        tracing::info!(id, fences = fences.len(), notes, "plugins: uninstall asked");
        if !window::confirm(owner, &text, i18n::text("卸载")) {
            return;
        }
        let cancelled = self.plugins.busy.remove(id);
        tracing::info!(id, fences = fences.len(), ?cancelled, "plugins: uninstall");
        self.plugins.trial.remove(id);
        for fence in fences {
            self.close_plugin_input(fence);
            self.stop_script(fence);
            self.delete_fence(fence);
        }
        crate::web_plugin::delete_profile(id);
        self.forget_light(id);
        for dir in [self.plugins.root.join(id), self.plugins_data().join(id)] {
            if let Err(e) = remove_dir(&dir) {
                tracing::warn!(id, dir = %dir.display(), error = %e, "plugins: folder not removed");
            }
        }
        self.plugins.reload(id);
    }

    /// At start (`remove_left_script_data`, with its guard): deletes what is kept on disk for
    /// plugin fences that are gone for good, in neither the configuration's scripts nor a
    /// snapshot's: their cached views (`plugins-host\<fence>.view.json`, the plugin host's) and
    /// the data files their plugin named after them (`plugins-data\<id>\<fence>-*`). Until
    /// then an undo or a snapshot brings a deleted fence back with them.
    pub(super) fn drop_files_of_deleted_fences(&self) {
        let (views, data) = (self.views_dir(), self.plugins_data());
        let c = &self.state.config;
        let scripts = (c.scripts.iter()).chain(c.snapshots.iter().flat_map(|s| s.scripts.iter()));
        let keep: HashSet<String> = scripts
            .filter(|(_, s)| plugin_id(&s.path).is_some())
            .map(|(f, _)| f.to_string())
            .collect();
        let sweep = move || {
            // The files in `dir` that `fence` names a fence of, which is not kept.
            let remove_stale = |dir: &Path, fence: fn(&str) -> Option<&str>| {
                let mut gone = 0;
                for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                    let name = e.file_name().to_string_lossy().to_ascii_lowercase();
                    let f = fence(&name).filter(|f| uuid::Uuid::try_parse(f).is_ok());
                    if f.is_some_and(|f| !keep.contains(f))
                        && std::fs::remove_file(e.path()).is_ok()
                    {
                        gone += 1;
                    }
                }
                gone
            };
            let views = remove_stale(&views, |n| n.strip_suffix(".view.json"));
            let named: fn(&str) -> Option<&str> =
                |n| n.get(..36).filter(|_| n.as_bytes().get(36) == Some(&b'-'));
            let plugins = std::fs::read_dir(&data).into_iter().flatten().flatten();
            let data: usize = plugins.map(|p| remove_stale(&p.path(), named)).sum();
            tracing::info!(
                views,
                data,
                kept = keep.len(),
                "plugins: cached views and data files of fences that went deleted, deleted"
            );
        };
        let spawned = std::thread::Builder::new()
            .name("pecofence-plugin-views".into())
            .spawn(sweep);
        if let Err(e) = spawned {
            tracing::warn!(error = %e, "plugins: view sweep did not start");
        }
    }

    /// `plugins-host` beside the configuration, where the plugin host caches each fence's view.
    fn views_dir(&self) -> PathBuf {
        let config = self.state.config_path_on_disk();
        let dir = config.parent().unwrap_or(Path::new("."));
        dir.join("plugins-host")
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
                // The warning showed this version and these permissions; when the index has
                // changed them since, the warning is shown again.
                let shown = v.get("shown").and_then(|s| s.as_str());
                let now = self.plugins.index.as_ref().and_then(|i| i.get(id));
                let now = now.map(|e| list::shown_key(&e.manifest));
                if shown.is_none() || shown != now.as_deref() {
                    tracing::warn!(
                        id,
                        shown,
                        now,
                        "plugins: install refused, the warning showed another entry; shown again"
                    );
                    self.plugins.ask = Some(id.to_string());
                } else {
                    // Like the built-in examples, an install switches 小工具 on.
                    self.ensure_tool_on(pecofence_core::FenceTool::Script);
                    self.start_plugin_download(id, true);
                }
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
        // Asked once: a page opened later shows no warning by itself.
        self.plugins.ask = None;
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
                // The warning of an update by another author names the one before.
                let before = mine.map(|p| &p.manifest.author).filter(|a| state == "confirm" && **a != m.author);
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
                    "author": m.author, "authorBefore": before, "version": m.version, "installed": mine.is_some(), "layer": m.layer,
                    "network": p.network, "links": p.links, "system": p.system, "background": p.background,
                    "folders": p.folders, "notify": p.notify,
                    "clipboard": format!("{:?}", p.clipboard).to_lowercase(),
                    "state": state, "reason": reason, "page": index.is_some(),
                    "settings": settings, "fences": fences, "shown": list::shown_key(m),
                }))
            })
            .collect();
        let status = match self.plugins.status {
            Status::Idle | Status::Loading => "loading",
            Status::Offline => "offline",
            Status::Ready => "ready",
        };
        serde_json::json!({ "status": status, "held": self.held_scripts_wait(), "open": self.plugins.open, "ask": self.plugins.ask, "list": rows })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_file_written_whole_replaces_the_one_before_and_leaves_no_temporary_file() {
        let dir = std::env::temp_dir().join(format!("pf-whole-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("index.json");
        for text in ["first", "second"] {
            super::write_whole(&path, text.as_bytes()).unwrap();
        }
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
        assert!(!dir.join("index.json.new").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
