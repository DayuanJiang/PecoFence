//! 插件的轻量层: what PecoFence does for a plugin's `pf` calls (the plugin host checked them
//! against the installed manifest already). They run on worker threads; what needs this
//! thread's windows (the folder dialog, the corner card, the shell, the system switch) goes to
//! the UI thread as a job and its answer comes back.

use super::{App, HWND, WM_APP_COMMAND};
use pecofence_core::FenceId;
use pecofence_core::plugins::Manifest;
use pecofence_platform::{clipboard, sysinfo, window};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Work a worker thread hands to the UI thread.
pub(super) type UiJob = Box<dyn FnOnce(&mut App) + Send>;

const MAX_TEXT: usize = 256 * 1024;
const MAX_DATA: u64 = 200 * 1024 * 1024;
const MAX_PICTURE: u32 = 4096;
const MAX_FILE: u64 = 20 * 1024 * 1024;
const RUN_TIME: Duration = Duration::from_secs(30);
const RUN_OUTPUT: usize = 1024 * 1024;

pub(super) struct AppHost {
    control: isize,
    pub(super) jobs: Arc<Mutex<Vec<UiJob>>>,
    /// The plugin fences that ran: the installed manifest of each (read once a start).
    pub(super) plugins: Mutex<HashMap<FenceId, Arc<Manifest>>>,
    /// `<config>\plugins-data`.
    data: PathBuf,
    cpu: Mutex<Option<sysinfo::CpuTimes>>,
}

fn io(e: std::io::Error) -> String {
    e.to_string()
}

fn text_of(v: &Value) -> String {
    v.as_str().map_or_else(|| v.to_string(), str::to_string)
}

impl AppHost {
    pub(super) fn new(control: HWND, jobs: Arc<Mutex<Vec<UiJob>>>, config_dir: &Path) -> Self {
        Self {
            control: control.0 as isize,
            jobs,
            plugins: Mutex::default(),
            data: config_dir.join("plugins-data"),
            cpu: Mutex::default(),
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
        match name {
            "data.read" => match std::fs::read(&file) {
                Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).into()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Null),
                Err(e) => Err(io(e)),
            },
            "data.write" => {
                let text = text_of(&args[1]);
                self.room(&dir, &file, text.len() as u64)?;
                std::fs::create_dir_all(&dir).map_err(io)?;
                std::fs::write(&file, text).map_err(io)?;
                Ok(Value::Null)
            }
            "data.remove" => match std::fs::remove_file(&file) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(io(e)),
                _ => Ok(Value::Null),
            },
            _ => {
                let entries = std::fs::read_dir(&dir).into_iter().flatten().flatten();
                let mut names: Vec<String> = entries
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect();
                names.sort();
                Ok(json!(names))
            }
        }
    }

    /// Err when `bytes` more in `file` would take the data folder past 200 MB.
    fn room(&self, dir: &Path, file: &Path, bytes: u64) -> Result<(), String> {
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
                let Some((w, h, bgra)) = clipboard::read_image() else {
                    return Ok(Value::Null);
                };
                if w > MAX_PICTURE || h > MAX_PICTURE {
                    return Err(format!(
                        "the picture is larger than 4096 x 4096 ({w} x {h})"
                    ));
                }
                let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
                let ms = ms.unwrap_or_default().as_millis();
                let name = arg.map_or_else(|| format!("clipboard-{ms}.png"), str::to_string);
                let dir = self.data_dir(fence)?;
                let file = dir.join(&name);
                self.room(&dir, &file, bgra.len() as u64 / 4)?;
                std::fs::create_dir_all(&dir).map_err(io)?;
                let _com = pecofence_platform::com::MtaGuard::init();
                clipboard::save_png(&file, w, h, &bgra).map_err(|e| e.to_string())?;
                let size = std::fs::metadata(&file).map_or(0, |m| m.len());
                if size > MAX_FILE {
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
                    MAX_PICTURE,
                    MAX_PICTURE,
                    false,
                );
                let p = picture.map_err(|e| format!("{}: {e}", file.display()))?;
                let ok = clipboard::set_image(self.control(), p.width, p.height, &p.bgra);
                ok.then_some(Value::Null)
                    .ok_or_else(|| "the clipboard is busy".into())
            }
            _ => {
                // The shell's data object lives on the UI thread, as Explorer's copy does.
                let paths: Vec<PathBuf> = (args[0].as_array().into_iter().flatten())
                    .filter_map(Value::as_str)
                    .map(PathBuf::from)
                    .collect();
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
        let begun = Instant::now();
        // Both pipes are read to the end (a full pipe would stall the program); 1 MB is kept.
        let drain = |pipe: Option<Box<dyn Read + Send>>| {
            std::thread::spawn(move || {
                let (mut kept, mut chunk) = (Vec::new(), [0u8; 16 * 1024]);
                if let Some(mut pipe) = pipe {
                    while let Ok(n @ 1..) = pipe.read(&mut chunk) {
                        let room = RUN_OUTPUT.saturating_sub(kept.len());
                        kept.extend_from_slice(&chunk[..n.min(room)]);
                    }
                }
                kept
            })
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
                let _ = child.kill();
                tracing::info!(%fence, program, "pf.run: the program ran 30 s and was ended");
                return Err(format!("{program} ran longer than 30 s and was ended"));
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        let mut bytes = out.join().unwrap_or_default();
        bytes.extend(err.join().unwrap_or_default());
        bytes.truncate(RUN_OUTPUT);
        let output = match std::str::from_utf8(&bytes) {
            Ok(s) => s.to_string(),
            Err(_) => pecofence_platform::wide::from_ansi(&bytes),
        };
        tracing::info!(%fence, program, ?code, ms = begun.elapsed().as_millis(), bytes = bytes.len(), "pf.run: the program ended");
        Ok(json!({ "code": code, "output": output }))
    }

    /// `pf.folder.list` / `read`: inside a folder the user picked for this fence, read only.
    fn folder(&self, fence: FenceId, name: &str, args: &Value) -> Result<Value, String> {
        let picked = self.on_ui(move |app| Ok(app.plugin_folders(fence)))?;
        let asked = args[0].as_str().unwrap_or_default();
        let path = std::fs::canonicalize(asked).map_err(|e| format!("{asked}: {e}"))?;
        let inside = (picked.iter().filter_map(|f| std::fs::canonicalize(f).ok()))
            .any(|f| path.starts_with(f));
        if !inside {
            tracing::info!(%fence, name, "plugin folder call refused: not inside a picked folder");
            return Err(format!(
                "{name}: {asked} is not inside a folder the user picked"
            ));
        }
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
            n if n.starts_with("clipboard.") => self.clipboard(fence, name, &args),
            n if n.starts_with("sys.") => Ok(self.sys(name)),
            "run" => self.run(fence, &args),
            "folder.pick" => self.on_ui(move |app| Ok(app.pick_plugin_folder(fence))),
            "folder.list" | "folder.read" => self.folder(fence, name, &args),
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
