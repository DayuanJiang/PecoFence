//! 插件的轻量层: what PecoFence does for a plugin's `pf` calls (the plugin host checked them
//! against the installed manifest already). They run on worker threads; what needs this
//! thread's windows (the folder dialog, the corner card, the shell, the system switch) goes to
//! the UI thread as a job and its answer comes back.

use super::{App, HWND, WM_APP_COMMAND};
use pecofence_core::FenceId;
use pecofence_core::plugins::Manifest;
use pecofence_platform::{clipboard, sysinfo, window};
use pecofence_plugin_host::view::{MAX_IMAGE_BYTES, MAX_IMAGE_SIDE};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Work a worker thread hands to the UI thread.
pub(super) type UiJob = Box<dyn FnOnce(&mut App) + Send>;

const MAX_TEXT: usize = 256 * 1024;
const MAX_DATA: u64 = 200 * 1024 * 1024;
const MAX_FILE: u64 = 20 * 1024 * 1024;
/// The most bytes of a dropped file `pf.drop.read` and `pf.drop.text` give a plugin: in base64
/// with its JSON copy, 4 MB would not fit the plugin's 8 MB.
const MAX_DROP_READ: u64 = 1024 * 1024;
const RUN_TIME: Duration = Duration::from_secs(30);
const RUN_OUTPUT: usize = 1024 * 1024;

pub(super) struct AppHost {
    control: isize,
    pub(super) jobs: Arc<Mutex<Vec<UiJob>>>,
    /// The plugin fences that ran: the installed manifest of each (read once a start).
    pub(super) plugins: Mutex<HashMap<FenceId, Arc<Manifest>>>,
    /// `<config>\plugins-data`.
    data: PathBuf,
    /// Held while a call writes into a data folder, and by an uninstall while it forgets the
    /// plugin ([`Self::forget`]): a write either ends before the folder goes or finds no plugin.
    writing: Mutex<()>,
    cpu: Mutex<Option<sysinfo::CpuTimes>>,
    /// The media session for `pf.media` (its thread starts with the first plugin that may see it).
    pub(super) media: super::media::MediaWatch,
    /// The programs running and the one in front, as last read for `pf.apps` and the `apps`
    /// event (one read every 5 s at most, for all plugins).
    programs: Mutex<Option<(std::time::Instant, Programs)>>,
}

/// The programs running and the program of the window in front (lowercase file names).
#[derive(Clone, Default)]
pub(super) struct Programs(pub Arc<Vec<String>>, pub Option<String>);

fn io(e: std::io::Error) -> String {
    e.to_string()
}

fn text_of(v: &Value) -> String {
    v.as_str().map_or_else(|| v.to_string(), str::to_string)
}

impl AppHost {
    /// `data`: the folder of the plugins' data folders.
    pub(super) fn new(control: HWND, jobs: Arc<Mutex<Vec<UiJob>>>, data: PathBuf) -> Self {
        Self {
            control: control.0 as isize,
            jobs,
            plugins: Mutex::default(),
            data,
            writing: Mutex::default(),
            cpu: Mutex::default(),
            media: Default::default(),
            programs: Mutex::default(),
        }
    }

    fn control(&self) -> HWND {
        HWND(self.control as *mut core::ffi::c_void)
    }

    pub(super) fn manifest(&self, fence: FenceId) -> Option<Arc<Manifest>> {
        self.plugins.lock().ok()?.get(&fence).cloned()
    }

    /// Runs `f` on the UI thread and waits for its answer.
    fn on_ui<T: Send + 'static>(
        &self,
        f: impl FnOnce(&mut App) -> Result<T, String> + Send + 'static,
    ) -> Result<T, String> {
        let (tx, rx) = std::sync::mpsc::channel();
        let job: UiJob = Box::new(move |app| {
            let _ = tx.send(f(app));
        });
        self.jobs
            .lock()
            .map_err(|_| "PecoFence is closing")?
            .push(job);
        window::post_message(self.control(), WM_APP_COMMAND, 0, 0);
        rx.recv().map_err(|_| "PecoFence is closing".to_string())?
    }

    /// The plugin's data folder, `<config>\plugins-data\<id>`.
    fn data_dir(&self, fence: FenceId) -> Result<PathBuf, String> {
        let m = self.manifest(fence).ok_or("the plugin does not run")?;
        Ok(self.data.join(&m.id))
    }

    /// Runs `write` in fence `fence`'s data folder (made first) while its plugin is still
    /// installed: an uninstall waits for it, and a write after the uninstall finds no plugin.
    /// Writes go one at a time, each after the check that `bytes` more in `file` keep the folder
    /// within 200 MB: two at once cannot both pass it.
    pub(super) fn write_data<T>(
        &self,
        fence: FenceId,
        file: &Path,
        bytes: u64,
        write: impl FnOnce(&Path) -> Result<T, String>,
    ) -> Result<T, String> {
        let _writing = self.writing.lock().map_err(|_| "PecoFence is closing")?;
        let dir = self.data_dir(fence)?;
        std::fs::create_dir_all(&dir).map_err(io)?;
        Self::room(&dir, file, bytes)?;
        write(&dir)
    }

    /// Plugin `id` is being uninstalled: its fences' calls find no plugin from now on, and a
    /// write already going ends first.
    pub(super) fn forget(&self, id: &str) {
        let _writing = self.writing.lock();
        if let Ok(mut plugins) = self.plugins.lock() {
            plugins.retain(|_, m| m.id != id);
        }
        tracing::info!(id, "plugin forgotten: its data folder takes no more writes");
    }

    /// Bytes in the data folder, without `except` (a file about to be written again).
    fn data_used(dir: &Path, except: &Path) -> u64 {
        let files = std::fs::read_dir(dir).into_iter().flatten().flatten();
        let files = files.filter(|f| f.path() != except);
        files
            .filter_map(|f| f.metadata().ok())
            .map(|m| m.len())
            .sum()
    }

    fn data(&self, fence: FenceId, name: &str, args: &Value) -> Result<Value, String> {
        let dir = self.data_dir(fence)?;
        let file = dir.join(args[0].as_str().unwrap_or_default());
        let big = || format!("{} is larger than 20 MB", args[0]);
        match name {
            "data.read" => match std::fs::metadata(&file) {
                Ok(m) if m.len() > MAX_FILE => Err(big()),
                Ok(_) => Ok(String::from_utf8_lossy(&std::fs::read(&file).map_err(io)?).into()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Null),
                Err(e) => Err(io(e)),
            },
            "data.write" => {
                let text = text_of(&args[1]);
                if text.len() as u64 > MAX_FILE {
                    return Err(big());
                }
                let bytes = text.len() as u64;
                self.write_data(fence, &file, bytes, |dir| {
                    replace_file(dir, &file, text.as_bytes())
                })?;
                Ok(Value::Null)
            }
            "data.remove" => match std::fs::remove_file(&file) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(io(e)),
                _ => Ok(Value::Null),
            },
            _ => {
                let entries = std::fs::read_dir(&dir).into_iter().flatten().flatten();
                // Without the temporary files of writes going on (no plain name starts with a dot).
                let mut names: Vec<String> = entries
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .filter(|n| !n.starts_with('.'))
                    .collect();
                names.sort();
                Ok(json!(names))
            }
        }
    }

    /// Err when `bytes` more in `file` would take the data folder past 200 MB.
    fn room(dir: &Path, file: &Path, bytes: u64) -> Result<(), String> {
        let used = Self::data_used(dir, file);
        if used + bytes > MAX_DATA {
            tracing::info!(
                used,
                bytes,
                "plugin data refused: the folder would pass 200 MB"
            );
            return Err(format!(
                "the plugin's data folder would pass 200 MB ({used} bytes used)"
            ));
        }
        Ok(())
    }

    fn clipboard(&self, fence: FenceId, name: &str, args: &Value) -> Result<Value, String> {
        let arg = args[0].as_str();
        match name {
            "clipboard.text" => Ok(clipboard::read_text(MAX_TEXT).map_or(
                Value::Null,
                |(text, cut)| json!({ "text": text, "truncated": cut }),
            )),
            "clipboard.files" => Ok(clipboard::read_files().map_or(Value::Null, |paths| {
                json!(
                    paths
                        .iter()
                        .map(|p| p.to_string_lossy())
                        .collect::<Vec<_>>()
                )
            })),
            "clipboard.image" => {
                let (w, h, bgra) = match clipboard::read_image(MAX_IMAGE_SIDE) {
                    None => return Ok(Value::Null),
                    Some(Err((w, h))) => {
                        return Err(format!(
                            "the picture is larger than 4096 x 4096 ({w} x {h})"
                        ));
                    }
                    Some(Ok(picture)) => picture,
                };
                let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
                let ms = ms.unwrap_or_default().as_millis();
                let name = arg.map_or_else(|| format!("clipboard-{ms}.png"), str::to_string);
                let dir = self.data_dir(fence)?;
                let file = dir.join(&name);
                let _com = pecofence_platform::com::MtaGuard::init();
                self.write_data(fence, &file, bgra.len() as u64, |_| {
                    clipboard::save_png(&file, w, h, &bgra).map_err(|e| e.to_string())
                })?;
                let size = std::fs::metadata(&file).map_or(0, |m| m.len());
                if size > MAX_IMAGE_BYTES {
                    let _ = std::fs::remove_file(&file);
                    return Err("the picture is larger than 20 MB as a PNG".into());
                }
                tracing::info!(%fence, w, h, size, "plugin: clipboard picture saved");
                Ok(name.into())
            }
            "clipboard.setText" => {
                let ok = clipboard::set_text(self.control(), &text_of(&args[0]));
                ok.then_some(Value::Null)
                    .ok_or_else(|| "the clipboard is busy".into())
            }
            "clipboard.setImage" => {
                let file = self.data_dir(fence)?.join(arg.unwrap_or_default());
                let _com = pecofence_platform::com::MtaGuard::init();
                let picture = pecofence_platform::wallpaper::decode_native(
                    &file,
                    MAX_IMAGE_SIDE,
                    MAX_IMAGE_SIDE,
                    false,
                );
                let p = picture.map_err(|e| format!("{}: {e}", file.display()))?;
                let ok = clipboard::set_image(self.control(), p.width, p.height, &p.bgra);
                ok.then_some(Value::Null)
                    .ok_or_else(|| "the clipboard is busy".into())
            }
            _ => {
                let paths: Vec<PathBuf> = (args[0].as_array().into_iter().flatten())
                    .filter_map(Value::as_str)
                    .map(PathBuf::from)
                    .collect();
                // A plugin that only writes the clipboard never read a file list from it: it may
                // only put files of its data folder and of the folders the user picked. (One
                // that reads it, a clipboard history, puts back what was copied.)
                let reads = self
                    .manifest(fence)
                    .is_some_and(|m| m.permissions.clipboard.reads());
                if !reads {
                    let mut roots = self.picked(fence)?;
                    roots.push(self.data_dir(fence)?);
                    for p in &paths {
                        inside(&p.to_string_lossy(), &roots).map_err(|why| {
                            tracing::info!(%fence, why, "plugin clipboard.setFiles refused");
                            format!("clipboard.setFiles: {why}")
                        })?;
                    }
                }
                // The shell's data object lives on the UI thread, as Explorer's copy does.
                self.on_ui(move |_| {
                    let paths: Vec<&Path> = paths.iter().map(PathBuf::as_path).collect();
                    clipboard::set_files(&paths, false).map_err(|e| e.to_string())?;
                    Ok(Value::Null)
                })
            }
        }
    }

    fn sys(&self, name: &str) -> Value {
        match name {
            "sys.cpu" => {
                let mut last = self.cpu.lock().ok().and_then(|mut c| c.take());
                if last.is_none() {
                    last = sysinfo::CpuTimes::now();
                    std::thread::sleep(Duration::from_millis(250));
                }
                let now = sysinfo::CpuTimes::now();
                if let Ok(mut c) = self.cpu.lock() {
                    *c = now;
                }
                json!(now.zip(last).map(|(n, l)| n.load_since(&l)))
            }
            "sys.memory" => json!(sysinfo::memory_load()),
            "sys.drives" => json!(
                sysinfo::local_drives()
                    .iter()
                    .map(|d| json!({ "letter": d.letter.to_string(), "free": d.free, "size": d.size }))
                    .collect::<Vec<_>>()
            ),
            "sys.ip" => json!(sysinfo::pick_ipv4(&sysinfo::adapters()).map(|ip| ip.to_string())),
            "sys.network" => json!(
                sysinfo::adapters()
                    .iter()
                    .map(|a| json!({
                        "up": a.up, "gateway": a.gateway, "physical": a.physical,
                        "addresses": a.addresses.iter().map(|ip| ip.to_string()).collect::<Vec<_>>(),
                    }))
                    .collect::<Vec<_>>()
            ),
            _ => sysinfo::battery().map_or(Value::Null, |(percent, plugged)| {
                json!({ "percent": percent, "plugged": plugged })
            }),
        }
    }

    /// `pf.run`: only with 允许插件使用 Windows 底层功能 on, a program the manifest names (the
    /// plugin host checked that), at most 30 s and 1 MB of output.
    fn run(&self, fence: FenceId, args: &Value) -> Result<Value, String> {
        use std::io::Read;
        use std::os::windows::process::CommandExt;
        let on = self.on_ui(|app| Ok(app.state.config.settings.tools.allow_system))?;
        let program = args[0].as_str().unwrap_or_default();
        if !on {
            tracing::info!(%fence, program, "pf.run refused: the system switch is off");
            return Err(
                "pf.run needs 允许插件使用 Windows 底层功能 (设置 › 小工具), which is off".into(),
            );
        }
        let path = if Path::new(program).is_absolute() {
            PathBuf::from(program)
        } else {
            let system = std::env::var_os("SystemRoot")
                .map(PathBuf::from)
                .unwrap_or_default();
            let named = system.join("System32").join(program);
            if named.extension().is_some() {
                named
            } else {
                named.with_extension("exe")
            }
        };
        if !path.is_file() || program.contains(['/', '\\']) && !Path::new(program).is_absolute() {
            return Err(format!("{program}: no such program"));
        }
        let list: Vec<String> = (args[1].as_array().into_iter().flatten())
            .map(text_of)
            .collect();
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut child = std::process::Command::new(&path)
            .args(&list)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(io)?;
        // What the program starts goes with it: the job ends them all when the program ends
        // or runs too long, which closes the pipes, so the readers below finish too.
        let Some(job) = pecofence_platform::job::KillOnCloseJob::new().filter(|j| j.assign(&child))
        else {
            let _ = child.kill();
            tracing::warn!(%fence, program, "pf.run: no job object, the program was ended");
            return Err(format!("{program} could not be run (no job object)"));
        };
        let begun = Instant::now();
        // Both pipes are read to the end (a full pipe would stall the program); 1 MB is kept.
        let drain = |pipe: Option<Box<dyn Read + Send>>| {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let (mut kept, mut chunk) = (Vec::new(), [0u8; 16 * 1024]);
                if let Some(mut pipe) = pipe {
                    while let Ok(n @ 1..) = pipe.read(&mut chunk) {
                        let room = RUN_OUTPUT.saturating_sub(kept.len());
                        kept.extend_from_slice(&chunk[..n.min(room)]);
                    }
                }
                let _ = tx.send(kept);
            });
            rx
        };
        let out = drain(
            child
                .stdout
                .take()
                .map(|p| Box::new(p) as Box<dyn Read + Send>),
        );
        let err = drain(
            child
                .stderr
                .take()
                .map(|p| Box::new(p) as Box<dyn Read + Send>),
        );
        let code = loop {
            if let Some(status) = child.try_wait().map_err(io)? {
                break status.code();
            }
            if begun.elapsed() > RUN_TIME {
                job.terminate();
                let _ = child.wait();
                tracing::info!(%fence, program, "pf.run: the program ran 30 s and was ended with what it started");
                return Err(format!("{program} ran longer than 30 s and was ended"));
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        let left = job.has_processes();
        drop(job);
        let wait = Duration::from_secs(1);
        let mut bytes = out.recv_timeout(wait).unwrap_or_default();
        bytes.extend(err.recv_timeout(wait).unwrap_or_default());
        bytes.truncate(RUN_OUTPUT);
        let output = match std::str::from_utf8(&bytes) {
            Ok(s) => s.to_string(),
            Err(_) => pecofence_platform::wide::from_ansi(&bytes),
        };
        tracing::info!(%fence, program, ?code, ms = begun.elapsed().as_millis(), bytes = bytes.len(), left, "pf.run: the program ended (left: what it started was still running and was ended)");
        Ok(json!({ "code": code, "output": output }))
    }

    /// The programs running and the one in front, read again when the last read is 5 s old or
    /// (`fresh`: the 5 s reads of the `apps` event) at once; what `pf.apps` answers is that read.
    pub(super) fn programs(&self, fresh: bool) -> Programs {
        let Ok(mut last) = self.programs.lock() else {
            return Programs::default();
        };
        match &*last {
            Some((at, p)) if !fresh && at.elapsed() < std::time::Duration::from_secs(5) => {
                p.clone()
            }
            _ => {
                let running = pecofence_platform::process::running_programs();
                let p = Programs(Arc::new(running), front_program());
                tracing::debug!(
                    programs = p.0.len(),
                    fresh,
                    "plugins: the running programs read (kept 5 s)"
                );
                *last = Some((std::time::Instant::now(), p.clone()));
                p
            }
        }
    }

    /// File `args[0]` of drop `args[2]` (the plugin host adds the last drop's number):
    /// `drop.read` gives its bytes in base64 and `drop.text` its text, each up to 1 MB;
    /// `drop.save` copies it into the data folder as `args[1]` (up to 20 MB), so that a larger
    /// file never goes through the plugin's memory. The file is opened once and read up to one
    /// byte past the limit: one that grows meanwhile is refused all the same.
    fn dropped(&self, fence: FenceId, name: &str, args: &Value) -> Result<Value, String> {
        use std::io::Read;
        let (i, drop) = (args[0].as_u64().unwrap_or(u64::MAX), args[2].as_u64());
        let path = self.on_ui(move |app| app.dropped_file(fence, drop, i as usize))?;
        let limit = if name == "drop.save" {
            MAX_FILE
        } else {
            MAX_DROP_READ
        };
        let mut bytes = Vec::new();
        let file = std::fs::File::open(&path).map_err(io)?;
        file.take(limit + 1).read_to_end(&mut bytes).map_err(io)?;
        let size = bytes.len() as u64;
        if size > limit {
            tracing::info!(%fence, name, limit, "plugin: a dropped file refused, too large");
            return Err(format!(
                "{name}: the file has more than {limit} bytes (pf.drop.save copies up to 20 MB into the data folder)"
            ));
        }
        tracing::info!(%fence, name, size, "plugin: a dropped file read");
        match name {
            "drop.text" => drop_text(bytes).map(Value::from),
            "drop.read" => Ok(base64(&bytes).into()),
            _ => {
                let file = self
                    .data_dir(fence)?
                    .join(args[1].as_str().unwrap_or_default());
                let size = bytes.len() as u64;
                self.write_data(fence, &file, size, |dir| replace_file(dir, &file, &bytes))?;
                Ok(Value::Null)
            }
        }
    }

    /// `pf.media.current()`: what plays, or null. Its cover goes into the plugin's data folder
    /// (`media-cover-<hash>.png` or `.jpg`, the one before deleted) for the view to show; `app`
    /// only when it is one of the plugin's `apps`.
    fn media_current(&self, fence: FenceId) -> Result<Value, String> {
        let m = self.manifest(fence).ok_or("the plugin does not run")?;
        let Some(p) = self.media.now.lock().ok().and_then(|n| n.clone()) else {
            return Ok(Value::Null);
        };
        // A cover that cannot be saved is none; the rest still comes.
        let cover = (p.cover.as_ref()).and_then(|(hash, bytes)| {
            let saved = self.save_cover(fence, *hash, bytes);
            if let Err(e) = &saved {
                tracing::info!(%fence, error = e, "plugin: the media cover not saved, given as null");
            }
            saved.ok()
        });
        // The session names its app by its AppUserModelID: `Spotify.exe`, `MSEdge` (msedge.exe),
        // `Chrome`; a Store app's (`Microsoft.ZuneMusic_…!…`) is no program name.
        let apps = &m.permissions.apps;
        let named = |a: &&String| {
            let stem = a.get(..a.len().saturating_sub(4)).unwrap_or_default();
            a.eq_ignore_ascii_case(&p.app) || stem.eq_ignore_ascii_case(&p.app)
        };
        let app = apps.iter().find(named);
        tracing::debug!(%fence, listed = app.is_some(), "plugin: what plays, its app named only when listed in apps");
        Ok(json!({
            "app": app, "title": p.title, "artist": p.artist, "album": p.album,
            "playing": p.playing, "position": p.position, "duration": p.duration, "cover": cover,
        }))
    }

    fn save_cover(&self, fence: FenceId, hash: u64, bytes: &[u8]) -> Result<String, String> {
        let kind = if bytes.starts_with(b"\x89PNG") {
            "png"
        } else {
            "jpg"
        };
        let name = format!("media-cover-{hash:016x}.{kind}");
        let dir = self.data_dir(fence)?;
        let file = dir.join(&name);
        if file.exists() {
            return Ok(name);
        }
        self.write_data(fence, &file, bytes.len() as u64, |dir| {
            let entries = std::fs::read_dir(dir).into_iter().flatten().flatten();
            for old in
                entries.filter(|e| e.file_name().to_string_lossy().starts_with("media-cover-"))
            {
                let _ = std::fs::remove_file(old.path());
            }
            replace_file(dir, &file, bytes)
        })?;
        tracing::debug!(%fence, bytes = bytes.len(), kind, "plugin: the media cover put into its data folder");
        Ok(name)
    }

    /// The folders the user picked for fence `fence`'s plugin.
    fn picked(&self, fence: FenceId) -> Result<Vec<PathBuf>, String> {
        let picked = self.on_ui(move |app| Ok(app.plugin_folders(fence)))?;
        Ok(picked.into_iter().map(PathBuf::from).collect())
    }

    /// `pf.folder.list` / `read`: inside a folder the user picked for this fence, read only.
    fn folder(&self, fence: FenceId, name: &str, args: &Value) -> Result<Value, String> {
        let asked = args[0].as_str().unwrap_or_default();
        let path = inside(asked, &self.picked(fence)?).map_err(|why| {
            tracing::info!(%fence, name, why, "plugin folder call refused");
            format!("{name}: {why}")
        })?;
        if name == "folder.read" {
            let size = std::fs::metadata(&path).map_err(io)?.len();
            if size > MAX_FILE {
                return Err(format!("{asked} is larger than 20 MB"));
            }
            return Ok(String::from_utf8_lossy(&std::fs::read(&path).map_err(io)?).into());
        }
        let entries = std::fs::read_dir(&path).map_err(io)?.flatten().take(10_000);
        let list: Vec<Value> = entries
            .filter_map(|e| {
                let meta = e.metadata().ok()?;
                let modified = meta
                    .modified()
                    .ok()?
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?;
                Some(json!({
                    "name": e.file_name().to_string_lossy(),
                    "path": Path::new(asked).join(e.file_name()).to_string_lossy(),
                    "folder": meta.is_dir(),
                    "size": meta.len(),
                    "modified": modified.as_millis() as u64,
                }))
            })
            .collect();
        Ok(json!(list))
    }
}

/// `asked`, resolved, when it is inside one of `folders`: first by its text, so nothing on disk
/// (or on the network) is touched for a path outside them, then as Windows resolves both, so no
/// link leads out.
pub(super) fn inside(asked: &str, folders: &[PathBuf]) -> Result<PathBuf, String> {
    let lower = |p: &Path| PathBuf::from(p.to_string_lossy().to_lowercase());
    let path = Path::new(asked);
    let up = path
        .components()
        .any(|c| c == std::path::Component::ParentDir);
    let outside = || format!("{asked} is not inside a folder the user picked");
    if !path.is_absolute() || up || !folders.iter().any(|f| lower(path).starts_with(lower(f))) {
        return Err(outside());
    }
    let path = std::fs::canonicalize(path).map_err(|e| format!("{asked}: {e}"))?;
    let mut resolved = folders.iter().filter_map(|f| std::fs::canonicalize(f).ok());
    resolved
        .any(|f| path.starts_with(f))
        .then_some(path)
        .ok_or_else(outside)
}

/// Writes `bytes` to `file` whole: into a temporary file in `dir` first, then renamed over it,
/// so a failed write (a full disk) leaves the file as it was.
/// The program of the window in front (`spotify.exe`, in lowercase).
pub(super) fn front_program() -> Option<String> {
    use pecofence_platform::desktop;
    let pid = desktop::window_pid(desktop::foreground_window());
    let path = pecofence_platform::process::image_path(pid)?;
    Some(path.file_name()?.to_string_lossy().to_lowercase())
}

/// Standard base64 with padding.
fn base64(bytes: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for k in 0..4 {
            let c = ABC[(n >> (18 - 6 * k)) as usize & 63] as char;
            out.push(if k <= chunk.len() { c } else { '=' });
        }
    }
    out
}

fn replace_file(dir: &Path, file: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(1);
    let name = file.file_name().unwrap_or_default().to_string_lossy();
    let temp = dir.join(format!(
        ".{name}.{}.tmp",
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let written = std::fs::write(&temp, bytes).and_then(|_| std::fs::rename(&temp, file));
    if let Err(e) = written {
        let _ = std::fs::remove_file(&temp);
        tracing::info!(file = %file.display(), error = %e, "plugin data not written, the old file stays");
        return Err(io(e));
    }
    Ok(())
}

impl pecofence_plugin_host::Host for AppHost {
    fn call(&self, fence: FenceId, name: &str, args: Value) -> Result<Value, String> {
        let arg = |i: usize| args[i].as_str().unwrap_or_default().to_string();
        match name {
            "http.get" | "http.post" => {
                let options = &args[1];
                let mut agent = String::new();
                let mut lines = String::new();
                for (k, v) in options["headers"].as_object().into_iter().flatten() {
                    let v = v.as_str().unwrap_or_default();
                    if k.eq_ignore_ascii_case("user-agent") {
                        agent = v.to_string();
                    } else if !k.contains(['\r', '\n', ':']) && !v.contains(['\r', '\n']) {
                        lines.push_str(&format!("{k}: {v}\r\n"));
                    }
                }
                let body = options["body"]
                    .as_str()
                    .map(str::as_bytes)
                    .unwrap_or_default();
                let verb = if name == "http.post" { "POST" } else { "GET" };
                let url = arg(0);
                let begun = Instant::now();
                let (status, head, bytes) = pecofence_platform::http::fetch(
                    verb,
                    &url,
                    &agent,
                    lines.trim_end(),
                    body,
                    pecofence_plugin_host::MAX_ANSWER,
                )?;
                tracing::info!(%fence, host = pecofence_plugin_host::url_host(&url), status, bytes = bytes.len(), ms = begun.elapsed().as_millis(), "plugin request answered");
                let headers: serde_json::Map<String, Value> = (head.lines().skip(1))
                    .filter_map(|l| l.split_once(':'))
                    .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().into()))
                    .collect();
                let body = String::from_utf8_lossy(&bytes);
                Ok(json!({ "status": status, "headers": headers, "body": body }))
            }
            n if n.starts_with("data.") => self.data(fence, name, &args),
            n if n.starts_with("drop.") => self.dropped(fence, name, &args),
            n if n.starts_with("clipboard.") => self.clipboard(fence, name, &args),
            n if n.starts_with("sys.") => Ok(self.sys(name)),
            "run" => self.run(fence, &args),
            "folder.pick" => self.on_ui(move |app| Ok(app.pick_plugin_folder(fence))),
            "folder.list" | "folder.read" => self.folder(fence, name, &args),
            n if n.starts_with("launch.") => {
                let name = name.to_string();
                self.on_ui(move |app| app.launch_call(fence, &name, &args))
            }
            "open" | "openSystem" => {
                let target = if name == "open" {
                    arg(0)
                } else {
                    "taskmgr.exe".into()
                };
                self.on_ui(move |app| {
                    let owner = app.window_for(fence).map(|w| w.hwnd());
                    let opened = pecofence_platform::shell::shell_execute(Path::new(&target), None, owner);
                    tracing::info!(%fence, ok = opened.is_ok(), "plugin opened a page or Task Manager");
                    opened.map(|_| Value::Null).map_err(|e| e.to_string())
                })
            }
            "media.current" => self.media_current(fence),
            "media.control" => Ok(json!(self.media.control(&arg(0), self.control)?)),
            "apps.running" => {
                let m = self.manifest(fence).ok_or("the plugin does not run")?;
                let running = self.programs(false).0;
                let apps =
                    (m.permissions.apps.iter()).filter(|a| running.contains(&a.to_lowercase()));
                Ok(json!(apps.collect::<Vec<_>>()))
            }
            // The program of `apps` in front, as the manifest names it, or null.
            "apps.front" => {
                let m = self.manifest(fence).ok_or("the plugin does not run")?;
                let front = self.programs(false).1;
                let app = (m.permissions.apps.iter()).find(|a| front == Some(a.to_lowercase()));
                Ok(json!(app))
            }
            "notify" => {
                let (title, body, sound) =
                    (arg(0), arg(1), args[2]["sound"].as_bool() == Some(true));
                self.on_ui(move |app| {
                    app.plugin_notify(fence, &title, &body, sound);
                    Ok(Value::Null)
                })
            }
            _ => Err(format!("{name}: no such pf function")),
        }
    }

    fn wants_notes(&self) -> bool {
        false
    }
}

/// `pf.drop.text`: a dropped file's text. A file with a NUL or that is not UTF-8 is refused (a
/// lossy string of a binary file grows about six times in JSON, past the plugin's memory).
fn drop_text(bytes: Vec<u8>) -> Result<String, String> {
    match String::from_utf8(bytes) {
        Ok(text) if !text.contains('\0') => Ok(text),
        _ => Err("drop.text: not a text file; use pf.drop.read".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pecofence_plugin_host::Host;

    #[test]
    fn drop_text_takes_text_and_refuses_binary_files() {
        assert_eq!(
            drop_text("à la carte\n".as_bytes().to_vec()),
            Ok("à la carte\n".into())
        );
        for bad in [
            b"PNG\0\x01".to_vec(),
            vec![0xff, 0xfe, 0x41],
            vec![b'a', 0xc3],
        ] {
            assert!(drop_text(bad).unwrap_err().contains("not a text file"));
        }
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pf-light-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A host whose plugin `t` runs in one fence (no UI thread: only calls that need none).
    fn host(data: &Path) -> (AppHost, FenceId) {
        let host = AppHost::new(HWND(std::ptr::null_mut()), Arc::default(), data.into());
        let m: Manifest = serde_json::from_value(json!({
            "id": "t", "version": "1.0.0", "api": 1, "layer": "light",
            "name": { "en": "T" }, "description": { "en": "T" },
            "author": "a", "minPecoFence": "0.4.0", "permissions": {},
        }))
        .unwrap();
        let fence = FenceId::new_v4();
        host.plugins.lock().unwrap().insert(fence, Arc::new(m));
        (host, fence)
    }

    #[test]
    fn dropped_bytes_come_in_standard_base64() {
        let cases = [("", ""), ("M", "TQ=="), ("Ma", "TWE="), ("Man", "TWFu")];
        for (bytes, text) in cases {
            assert_eq!(base64(bytes.as_bytes()), text);
        }
        assert_eq!(base64(&[0xFB, 0xFF]), "+/8=");
    }

    #[test]
    fn data_files_are_replaced_whole_and_hold_at_most_20_mb() {
        let dir = temp("data");
        let (host, fence) = host(&dir);
        let call = |name: &str, args: Value| host.call(fence, name, args);
        call("data.write", json!(["a.txt", "one"])).unwrap();
        call("data.write", json!(["a.txt", "two"])).unwrap();
        assert_eq!(call("data.read", json!(["a.txt"])), Ok(json!("two")));
        // No temporary file is left or listed.
        assert_eq!(call("data.list", json!([])), Ok(json!(["a.txt"])));
        assert_eq!(std::fs::read_dir(dir.join("t")).unwrap().count(), 1);
        let big = "x".repeat(MAX_FILE as usize + 1);
        let why = call("data.write", json!(["big.txt", big])).unwrap_err();
        assert!(why.contains("20 MB"), "{why}");
        std::fs::write(dir.join("t").join("big.txt"), &big).unwrap();
        let why = call("data.read", json!(["big.txt"])).unwrap_err();
        assert!(why.contains("20 MB"), "{why}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn two_writes_at_once_cannot_both_pass_200_mb() {
        let dir = temp("room");
        let (host, fence) = host(&dir);
        std::fs::create_dir_all(dir.join("t")).unwrap();
        let filler = std::fs::File::create(dir.join("t").join("filler")).unwrap();
        filler.set_len(MAX_DATA - 15 * 1024 * 1024).unwrap();
        let text = "x".repeat(10 * 1024 * 1024);
        let start = std::sync::Barrier::new(2);
        let results: Vec<bool> = std::thread::scope(|s| {
            let write = |name: &'static str| {
                let (host, text, start) = (&host, &text, &start);
                s.spawn(move || {
                    start.wait();
                    host.call(fence, "data.write", json!([name, text])).is_ok()
                })
            };
            let both = [write("a.txt"), write("b.txt")];
            both.map(|w| w.join().unwrap()).to_vec()
        });
        assert_eq!(results.iter().filter(|ok| **ok).count(), 1, "{results:?}");
        drop(filler);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn after_an_uninstall_no_write_makes_the_data_folder_again() {
        let dir = temp("forget");
        let (host, fence) = host(&dir);
        host.call(fence, "data.write", json!(["a.txt", "one"]))
            .unwrap();
        host.forget("t");
        std::fs::remove_dir_all(dir.join("t")).unwrap();
        assert!(
            host.call(fence, "data.write", json!(["a.txt", "two"]))
                .is_err()
        );
        assert!(!dir.join("t").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn paths_outside_the_picked_folders_are_refused_before_anything_is_read() {
        let dir = temp("inside");
        let picked = dir.join("Picked");
        std::fs::create_dir_all(picked.join("sub")).unwrap();
        std::fs::create_dir_all(dir.join("Picked2")).unwrap();
        std::fs::write(picked.join("sub").join("a.txt"), "a").unwrap();
        let folders = [picked.clone()];
        let ask = |p: &Path| inside(&p.to_string_lossy(), &folders);
        assert!(ask(&picked.join("sub").join("a.txt")).is_ok());
        let upper = PathBuf::from(picked.to_string_lossy().to_uppercase()).join("SUB");
        assert!(ask(&upper).is_ok(), "{upper:?}");
        for outside in [
            dir.join("Picked2"),
            picked.join("..").join("Picked2"),
            PathBuf::from(r"C:\Windows\win.ini"),
            PathBuf::from("sub"),
        ] {
            let why = ask(&outside).unwrap_err();
            assert!(why.contains("not inside a folder"), "{outside:?}: {why}");
        }
        // A network path is refused at once, without asking the network.
        let begun = Instant::now();
        let why = inside(r"\\pf-no-such-host.invalid\share\x", &folders).unwrap_err();
        assert!(why.contains("not inside a folder"), "{why}");
        assert!(begun.elapsed() < Duration::from_millis(200));
        assert!(inside(r"C:\x", &[]).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
