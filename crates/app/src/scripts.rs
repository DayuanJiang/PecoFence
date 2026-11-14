//! 脚本栅栏: runs a script of the user's without a console window and collects what it prints.
//! Runs happen on a worker thread; the App shows the result in the fence.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// A run that has not ended by then is stopped.
pub const TIMEOUT: Duration = Duration::from_secs(10);
/// What is kept of the output: enough for a fence, small enough for the UI thread.
const MAX_BYTES: usize = 16 * 1024;
const MAX_LINES: usize = 40;
const MAX_LINE_CHARS: usize = 300;
/// CREATE_NO_WINDOW: no console flashes up on the desktop.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// How a run ended.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// Exit code 0; what it printed (cleaned up).
    Done(String),
    /// Another exit code; what it printed until then (`text`) and what it wrote to stderr.
    Failed {
        code: Option<i32>,
        text: String,
        error: String,
    },
    /// Still running after [`TIMEOUT`]; stopped.
    TimedOut,
    /// The file is gone.
    Missing,
    /// A kind of file we do not know how to run.
    Unsupported,
    /// The interpreter could not be started (not installed, access denied).
    NotStarted(String),
}

impl Outcome {
    /// Short fixed name for the log.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Done(_) => "done",
            Self::Failed { .. } => "failed",
            Self::TimedOut => "timed_out",
            Self::Missing => "missing",
            Self::Unsupported => "unsupported",
            Self::NotStarted(_) => "not_started",
        }
    }
}

/// The interpreter for `path` by its extension, None for one we do not run.
fn command_for(path: &Path) -> Option<Vec<Command>> {
    use std::os::windows::process::CommandExt;
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let file = path.to_string_lossy().to_string();
    Some(match ext.as_str() {
        "ps1" => {
            // Windows PowerShell writes the console code page unless told otherwise.
            let mut c = Command::new("powershell.exe");
            c.args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
            ])
            .arg("-Command")
            // Failed only when the script ended in failure (`exit 3`, an error on its last
            // line): then with its exit code, else 1. A program that failed earlier in the
            // script (a ping that got no answer) leaves $LASTEXITCODE set but is no failure.
            .arg(format!(
                "[Console]::OutputEncoding=[Text.Encoding]::UTF8; & '{}'; \
                 if (-not $?) {{ if ($LASTEXITCODE) {{ exit $LASTEXITCODE }} else {{ exit 1 }} }}",
                file.replace('\'', "''")
            ));
            vec![c]
        }
        "py" | "pyw" => {
            // The py launcher first (python.org installs), then python on PATH.
            let mut py = Command::new("py");
            py.arg("-3").arg(&file);
            let mut python = Command::new("python");
            python.arg(&file);
            for c in [&mut py, &mut python] {
                c.env("PYTHONIOENCODING", "utf-8").env("PYTHONUTF8", "1");
            }
            vec![py, python]
        }
        "cmd" | "bat" => {
            let mut c = Command::new("cmd.exe");
            c.raw_arg(format!("/d /s /c \"chcp 65001 >nul & call \"{file}\"\""));
            vec![c]
        }
        "js" | "mjs" => {
            let mut c = Command::new("node");
            c.arg(&file);
            vec![c]
        }
        "exe" | "com" => vec![Command::new(&file)],
        _ => return None,
    })
}

/// What a run tells the script, as environment variables.
#[derive(Clone, Debug, Default)]
pub struct RunEnv {
    /// `PECOFENCE_ACTION`: the `action=` of the row clicked; empty for a timed run.
    pub action: String,
    /// `PECOFENCE_DATA`: a folder of this fence's own for what the script keeps.
    pub data_dir: std::path::PathBuf,
    /// `PECOFENCE_THEME`: `light` or `dark`.
    pub theme: &'static str,
    /// `PECOFENCE_FENCE`: the fence's id.
    pub fence: String,
}

/// Starts `path` in its own folder without a window, its output piped (and its input when
/// `stdin` is set: a streaming script reads its actions there).
fn spawn(path: &Path, env: &RunEnv, stdin: bool) -> Result<std::process::Child, Outcome> {
    use std::os::windows::process::CommandExt;
    if !path.is_file() {
        return Err(Outcome::Missing);
    }
    let Some(candidates) = command_for(path) else {
        return Err(Outcome::Unsupported);
    };
    let mut last_error = String::new();
    for mut cmd in candidates {
        cmd.stdin(if stdin { Stdio::piped() } else { Stdio::null() })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .env("PECOFENCE", "1")
            .env("PECOFENCE_ACTION", &env.action)
            .env("PECOFENCE_DATA", &env.data_dir)
            .env("PECOFENCE_THEME", env.theme)
            .env("PECOFENCE_FENCE", &env.fence)
            // A streaming Python script's lines would wait in its buffer otherwise.
            .env("PYTHONUNBUFFERED", "1");
        if let Some(dir) = path.parent() {
            cmd.current_dir(dir);
        }
        match cmd.spawn() {
            Ok(child) => return Ok(child),
            Err(e) => last_error = e.to_string(),
        }
    }
    Err(Outcome::NotStarted(last_error))
}

/// Runs `path` once, at most [`TIMEOUT`], in its own folder and without a window.
pub fn run(path: &Path, env: &RunEnv) -> Outcome {
    run_within(path, env, TIMEOUT)
}

/// [`run`] with the time limit given (the tests give a slow CI machine more).
fn run_within(path: &Path, env: &RunEnv, limit: Duration) -> Outcome {
    let mut child = match spawn(path, env, false) {
        Ok(child) => child,
        Err(outcome) => return outcome,
    };
    // The programs the script starts go with it: when it ends (or is stopped) the job ends
    // them, so none keeps running or holds the output pipes open.
    let job = pecofence_platform::job::KillOnCloseJob::new().filter(|job| job.assign(&child));
    // Readers on their own threads: a full pipe would otherwise stop the script.
    let reader = |pipe: Option<Box<dyn Read + Send>>| {
        let (tx, rx) = mpsc::channel();
        if let Some(mut pipe) = pipe {
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                while let Ok(n) = pipe.read(&mut chunk) {
                    if n == 0 {
                        break;
                    }
                    if buf.len() < MAX_BYTES {
                        buf.extend_from_slice(&chunk[..n.min(MAX_BYTES - buf.len())]);
                    }
                }
                let _ = tx.send(buf);
            });
        }
        rx
    };
    let out = reader(
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let err = reader(
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let begun = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if begun.elapsed() >= limit => {
                match &job {
                    Some(job) => job.terminate(),
                    None => {
                        let _ = child.kill();
                    }
                }
                let _ = child.wait();
                break None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(_) => break None,
        }
    };
    // Ends what the script left running, which closes the pipes so the readers finish.
    drop(job);
    let Some(status) = status else {
        return Outcome::TimedOut;
    };
    let wait = Duration::from_millis(1000);
    let stdout = clean(&decode(&out.recv_timeout(wait).unwrap_or_default()));
    let stderr = clean(&decode(&err.recv_timeout(wait).unwrap_or_default()));
    if status.success() {
        Outcome::Done(stdout)
    } else if status.code() == Some(9009) {
        // Windows' "command not found": no such interpreter (the python.exe Store stub too).
        Outcome::NotStarted(stdout)
    } else {
        Outcome::Failed {
            code: status.code(),
            text: stdout,
            error: stderr,
        }
    }
}

/// What a streaming script sends while it runs.
#[derive(Clone, Debug, PartialEq)]
pub enum StreamEvent {
    /// What the fence shows now.
    Block(String),
    /// The script ended (or was ended).
    Ended(Outcome),
}

/// A streaming script (`<swiftbar.type>streamable</swiftbar.type>` in its text) that keeps
/// running; dropping it ends the script and what it started.
pub struct Stream {
    job: pecofence_platform::job::KillOnCloseJob,
    /// Lines for the script's input, written by a thread of their own: a script that does not
    /// read its input then blocks that thread, never the UI.
    input: Option<mpsc::Sender<String>>,
}

/// The first 8 KB of `path`, where a script's header comments sit.
fn header(path: &Path) -> String {
    let mut head = vec![0u8; 8 * 1024];
    let n = std::fs::File::open(path)
        .and_then(|mut f| f.read(&mut head))
        .unwrap_or(0);
    String::from_utf8_lossy(&head[..n]).into_owned()
}

/// The first 8 KB of `path` mark it as a streaming script (the tag sits in the header).
pub fn is_streamable(path: &Path) -> bool {
    pecofence_core::script_output::is_streamable(&header(path))
}

/// The first 8 KB of `path` allow its rows to start programs
/// (`<pecofence.allow>run</pecofence.allow>` in the header).
pub fn allows_run(path: &Path) -> bool {
    pecofence_core::script_output::allows_run(&header(path))
}

/// What a row's `href=` / `open=` value `target` points at, for
/// `script_output::click_allowed`; `path` is the value as a path (a relative one already
/// taken from the script's folder).
pub fn target_kind(target: &str, path: &Path) -> pecofence_core::script_output::TargetKind {
    use pecofence_core::script_output::{TargetKind, is_web_address};
    if is_web_address(target) {
        return TargetKind::Web;
    }
    // Only a path on one of this computer's disks is looked at. Asking a network place
    // (`\\server\share` however its slashes are written, a mapped drive) whether it exists
    // already signs in to that server, and can keep the click waiting. A disc in a CD or DVD
    // drive can name a program of its own to start when it is opened (autorun.inf).
    if matches!(
        pecofence_platform::shell::drive_kind(path),
        "network" | "unc" | "unknown" | "cdrom"
    ) {
        return TargetKind::Other;
    }
    if path.is_dir() {
        return TargetKind::Folder;
    }
    // Windows drops dots and spaces at the end of a name: `a.exe.` opens `a.exe`.
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().trim_end_matches(['.', ' ']).to_string())
        .unwrap_or_default();
    // No extension (the shell would look for a program of that name) or a stream of a file
    // (`a.txt:b.exe`) is nothing to judge by.
    let ext = match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() && !name.contains(':') => {
            ext.to_lowercase()
        }
        _ => return TargetKind::Other,
    };
    // What PecoFence itself runs as a script (.py runs when Python is installed), and what
    // Windows classes as dangerous to open.
    let runs = command_for(Path::new(&name)).is_some()
        || pecofence_platform::shell::is_dangerous_type(&ext);
    if !runs && path.is_file() {
        TargetKind::Document
    } else {
        TargetKind::Other
    }
}

/// What a streaming script shows, line by line, as SwiftBar does: every line is added at
/// once; after a `~~~` line the next line starts a new screen. The newest [`MAX_LINES`] stay.
#[derive(Default)]
struct Screen {
    lines: std::collections::VecDeque<String>,
    reset: bool,
}

impl Screen {
    /// Takes one printed line; the text to show when it changed the screen.
    fn push(&mut self, line: &str) -> Option<String> {
        if line.trim() == "~~~" {
            self.reset = true;
            return None;
        }
        if std::mem::take(&mut self.reset) {
            self.lines.clear();
        }
        self.lines.push_back(line.to_string());
        while self.lines.len() > MAX_LINES {
            self.lines.pop_front();
        }
        Some(self.text())
    }

    fn text(&self) -> String {
        clean(&Vec::from(self.lines.clone()).join("\n"))
    }
}

/// Everything `pipe` sends, up to [`MAX_BYTES`], on a thread of its own.
fn drain(pipe: Option<impl Read + Send + 'static>) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut pipe) = pipe {
            let mut chunk = [0u8; 4096];
            while let Ok(n) = pipe.read(&mut chunk) {
                if n == 0 {
                    break;
                }
                if buf.len() < MAX_BYTES {
                    buf.extend_from_slice(&chunk[..n.min(MAX_BYTES - buf.len())]);
                }
            }
        }
        buf
    })
}

impl Stream {
    /// Starts `path`; `on_event` gets what to show and the end, on a reader thread.
    pub fn start(
        path: &Path,
        env: &RunEnv,
        on_event: impl Fn(StreamEvent) + Send + 'static,
    ) -> Result<Self, Outcome> {
        let mut child = spawn(path, env, true)?;
        // Without a job the script could not be ended with the fence: do not start it.
        let Some(job) = pecofence_platform::job::KillOnCloseJob::new().filter(|j| j.assign(&child))
        else {
            let _ = child.kill();
            return Err(Outcome::NotStarted("job object".into()));
        };
        let input = child.stdin.take().map(|mut stdin| {
            let (tx, rx) = mpsc::channel::<String>();
            std::thread::spawn(move || {
                use std::io::Write;
                for line in rx {
                    if writeln!(stdin, "{line}")
                        .and_then(|_| stdin.flush())
                        .is_err()
                    {
                        break;
                    }
                }
            });
            tx
        });
        let stdout = child.stdout.take();
        let errors = drain(child.stderr.take());
        std::thread::Builder::new()
            .name("pecofence-script-stream".into())
            .spawn(move || {
                use std::io::BufRead;
                let mut screen = Screen::default();
                let mut shown = String::new();
                if let Some(out) = stdout {
                    let mut reader = std::io::BufReader::new(out);
                    let mut line = Vec::new();
                    loop {
                        line.clear();
                        match reader
                            .by_ref()
                            .take(MAX_BYTES as u64)
                            .read_until(b'\n', &mut line)
                        {
                            Ok(0) | Err(_) => break,
                            Ok(_) => {}
                        }
                        let text = decode(&line);
                        if let Some(now) = screen.push(text.trim_end_matches(['\r', '\n'])) {
                            shown = now.clone();
                            on_event(StreamEvent::Block(now));
                        }
                    }
                }
                let status = child.wait().ok();
                let stderr = clean(&decode(&errors.join().unwrap_or_default()));
                // A script that ends by itself leaves what it showed last.
                let outcome = match status.and_then(|s| s.code()) {
                    Some(0) => Outcome::Done(shown),
                    code => Outcome::Failed {
                        code,
                        text: shown,
                        error: stderr,
                    },
                };
                on_event(StreamEvent::Ended(outcome));
            })
            .map_err(|e| Outcome::NotStarted(e.to_string()))?;
        Ok(Self { job, input })
    }

    /// Hands `line` (a clicked row's `action=`) to the script's standard input.
    pub fn send(&mut self, line: &str) -> bool {
        let sent = self
            .input
            .as_ref()
            .is_some_and(|tx| tx.send(line.to_string()).is_ok());
        if !sent {
            self.input = None;
        }
        sent
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        self.input = None;
        self.job.terminate();
    }
}

/// A row's `run=`: `program` with `args` in the script's folder `dir`, without a window. A
/// script file runs through its interpreter (the first one that starts); a relative path is
/// taken from `dir`. Returns the started process (the caller may wait for it before running
/// the script again).
pub fn run_program(
    program: &str,
    args: &[String],
    dir: Option<&Path>,
) -> std::io::Result<std::process::Child> {
    use std::os::windows::process::CommandExt;
    let mut path = std::path::PathBuf::from(program);
    if path.is_relative()
        && let Some(d) = dir
        && d.join(&path).exists()
    {
        path = d.join(&path);
    }
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let file = path.is_file();
    // (program, arguments before the user's): .cmd / .bat and programs run directly (Rust
    // quotes a batch file's arguments for cmd.exe).
    let candidates: Vec<(&str, Vec<std::ffi::OsString>)> = match ext.as_str() {
        "ps1" if file => vec![(
            "powershell.exe",
            [
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ]
            .iter()
            .map(Into::into)
            .chain([path.clone().into_os_string()])
            .collect(),
        )],
        "py" | "pyw" if file => vec![
            ("py", vec!["-3".into(), path.clone().into_os_string()]),
            ("python", vec![path.clone().into_os_string()]),
        ],
        "js" | "mjs" if file => vec![("node", vec![path.clone().into_os_string()])],
        _ => Vec::new(),
    };
    let mut commands: Vec<Command> = candidates
        .into_iter()
        .map(|(exe, pre)| {
            let mut c = Command::new(exe);
            c.args(pre);
            c
        })
        .collect();
    if commands.is_empty() {
        commands.push(Command::new(&path));
    }
    let mut last = None;
    for mut cmd in commands {
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .env("PYTHONIOENCODING", "utf-8")
            .env("PYTHONUTF8", "1");
        if let Some(d) = dir {
            cmd.current_dir(d);
        }
        match cmd.spawn() {
            Ok(child) => return Ok(child),
            Err(e) => last = Some(e),
        }
    }
    Err(last.unwrap_or_else(|| std::io::Error::other("nothing to run")))
}

/// UTF-8 (what the interpreters are asked for), else the system's ANSI code page.
fn decode(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        // Only the last character is cut (the byte limit): UTF-8 up to it.
        Err(e) if e.error_len().is_none() => {
            String::from_utf8_lossy(&bytes[..e.valid_up_to()]).into_owned()
        }
        Err(_) => pecofence_platform::wide::from_ansi(bytes),
    }
}

/// Line breaks as `\n`, colour codes and trailing spaces gone, at most [`MAX_LINES`] lines of
/// [`MAX_LINE_CHARS`], no empty lines at either end.
pub fn clean(text: &str) -> String {
    let text = text.trim_start_matches('\u{feff}');
    let mut lines: Vec<String> = text
        .split('\n')
        .map(|l| {
            let l = strip_escapes(l.trim_end_matches('\r'));
            let l = l.trim_end();
            l.chars().take(MAX_LINE_CHARS).collect()
        })
        .collect();
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    let first = lines
        .iter()
        .position(|l| !l.is_empty())
        .unwrap_or(lines.len());
    lines.drain(..first);
    lines.truncate(MAX_LINES);
    lines.join("\n")
}

/// Drops ANSI escape sequences (`ESC [ ... letter`), which consoles turn into colours.
fn strip_escapes(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// The script fence examples, as (Chinese, English) texts: the comments and labels are in
/// that language (the weather's conditions come from the service as they are).
pub fn example(kind: Example, chinese: bool) -> &'static str {
    match (kind, chinese) {
        (Example::Weather, true) => include_str!("../assets/scripts/weather.zh.ps1"),
        (Example::Weather, false) => include_str!("../assets/scripts/weather.en.ps1"),
        (Example::GitHub, true) => include_str!("../assets/scripts/github.zh.ps1"),
        (Example::GitHub, false) => include_str!("../assets/scripts/github.en.ps1"),
        (Example::Status, true) => include_str!("../assets/scripts/status.zh.ps1"),
        (Example::Status, false) => include_str!("../assets/scripts/status.en.ps1"),
    }
}

/// The text an example's file gets: Windows PowerShell 5.1 reads a script without a BOM in the
/// ANSI code page (the emoji and Chinese text need the BOM), and Notepad keeps CRLF.
pub fn example_file_text(kind: Example, chinese: bool) -> String {
    format!(
        "\u{feff}{}",
        unify_lines(example(kind, chinese)).replace('\n', "\r\n")
    )
}

/// Without a BOM, with LF line ends.
fn unify_lines(text: &str) -> String {
    text.trim_start_matches('\u{feff}').replace("\r\n", "\n")
}

/// Examples as earlier versions wrote them, with the example and language that replace each:
/// 0.3.0 and 0.3.1's PC status showed the IPv4 address of a disconnected adapter and drive C:
/// only. The GitHub and weather examples of 0.3.0 to 0.3.3 printed text from the network
/// unchanged, which could read as a row's parameters (a forged issue title started a program);
/// the PC status of 0.3.2 and 0.3.3 lacks the header line its `run=` row now needs.
const OLD_EXAMPLES: [(&str, Example, bool); 8] = [
    (
        include_str!("../assets/scripts/old/status.zh.0.3.0.ps1"),
        Example::Status,
        true,
    ),
    (
        include_str!("../assets/scripts/old/status.en.0.3.0.ps1"),
        Example::Status,
        false,
    ),
    (
        include_str!("../assets/scripts/old/status.zh.0.3.2.ps1"),
        Example::Status,
        true,
    ),
    (
        include_str!("../assets/scripts/old/status.en.0.3.2.ps1"),
        Example::Status,
        false,
    ),
    (
        include_str!("../assets/scripts/old/github.zh.0.3.0.ps1"),
        Example::GitHub,
        true,
    ),
    (
        include_str!("../assets/scripts/old/github.en.0.3.0.ps1"),
        Example::GitHub,
        false,
    ),
    (
        include_str!("../assets/scripts/old/weather.zh.0.3.0.ps1"),
        Example::Weather,
        true,
    ),
    (
        include_str!("../assets/scripts/old/weather.en.0.3.0.ps1"),
        Example::Weather,
        false,
    ),
];

/// A script file that still holds an earlier version's example unchanged gets the current one,
/// so a fence made from an old example shows its fixes; an edited copy stays as it is. True
/// when the file was replaced.
pub fn update_old_example(path: &Path) -> bool {
    // Only small files can be an example (a script fence may run a large .exe).
    let small = std::fs::metadata(path).is_ok_and(|m| m.len() <= 16 * 1024);
    if !small {
        return false;
    }
    let Ok(now) = std::fs::read_to_string(path) else {
        return false;
    };
    let now = unify_lines(&now);
    let Some(&(_, kind, chinese)) = OLD_EXAMPLES
        .iter()
        .find(|(old, _, _)| unify_lines(old) == now)
    else {
        return false;
    };
    match std::fs::write(path, example_file_text(kind, chinese)) {
        Ok(()) => {
            tracing::info!(
                example = kind.name(),
                chinese,
                "unedited example script of an earlier version replaced by the current one"
            );
            true
        }
        Err(e) => {
            tracing::warn!(example = kind.name(), error = %e, "updating an old example script failed");
            false
        }
    }
}

/// 「添加示例…」: the script fence examples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Example {
    /// Today's weather (wttr.in).
    Weather,
    /// A GitHub repository's newest issues and pull requests: rows that open in the browser.
    GitHub,
    /// The computer's state every 2 seconds (a streaming script): colours, copy, run.
    Status,
}

impl Example {
    pub const ALL: [Self; 3] = [Self::Weather, Self::GitHub, Self::Status];

    /// The example a settings page button names (`name()`).
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.name() == name)
    }

    /// Its line in the menu.
    pub fn label(self) -> &'static str {
        match self {
            Self::Weather => pecofence_core::i18n::text("天气"),
            Self::GitHub => pecofence_core::i18n::text("GitHub 仓库动态"),
            Self::Status => pecofence_core::i18n::text("电脑状态"),
        }
    }

    /// The file it is written to in the configuration's `scripts` folder.
    pub fn file_name(self) -> &'static str {
        match self {
            Self::Weather => pecofence_core::i18n::text("天气.ps1"),
            Self::GitHub => pecofence_core::i18n::text("GitHub 动态.ps1"),
            Self::Status => pecofence_core::i18n::text("电脑状态.ps1"),
        }
    }

    /// For the log.
    pub fn name(self) -> &'static str {
        match self {
            Self::Weather => "weather",
            Self::GitHub => "github",
            Self::Status => "status",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pc_status_selects_a_connected_ipv4_gateway_and_local_disks() {
        use std::os::windows::process::CommandExt;

        let test = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/test-pc-status.ps1");
        let output = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(test)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .expect("run the PC status regression tests in Windows PowerShell");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn an_unedited_old_example_gets_the_current_one() {
        let dir = std::env::temp_dir().join(format!("pf-old-example-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for (old, kind, chinese) in OLD_EXAMPLES {
            // As 0.3.0 wrote it: a BOM and CRLF.
            let written = format!("\u{feff}{}", unify_lines(old).replace('\n', "\r\n"));
            let path = dir.join(format!("{}-{chinese}.ps1", kind.name()));
            std::fs::write(&path, &written).unwrap();
            assert!(update_old_example(&path));
            let now = std::fs::read_to_string(&path).unwrap();
            assert_eq!(now, example_file_text(kind, chinese));
            assert_ne!(now, written);
            // The current one is left alone, and so is an edited old copy.
            assert!(!update_old_example(&path));
            let edited = format!("{written}# my own line\r\n");
            std::fs::write(&path, &edited).unwrap();
            assert!(!update_old_example(&path));
            assert_eq!(std::fs::read_to_string(&path).unwrap(), edited);
        }
        assert!(!update_old_example(&dir.join("missing.ps1")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_stream_screen_follows_swiftbar() {
        let mut s = Screen::default();
        assert_eq!(s.push("a").as_deref(), Some("a"));
        assert_eq!(s.push("b").as_deref(), Some("a\nb"));
        // `~~~` changes nothing until the next line, which starts a new screen.
        assert_eq!(s.push("~~~"), None);
        assert_eq!(s.push("c").as_deref(), Some("c"));
        // SwiftBar's way, `~~~` before each block, works the same.
        s.push("~~~");
        s.push("d");
        assert_eq!(s.push("e").as_deref(), Some("d\ne"));
        // A log that never prints `~~~` shows its newest lines.
        let mut log = Screen::default();
        let mut last = None;
        for i in 0..100 {
            last = log.push(&format!("line {i}"));
        }
        let last = last.unwrap();
        assert_eq!(last.lines().count(), MAX_LINES);
        assert!(last.ends_with("line 99"));
    }

    #[test]
    fn output_is_cleaned_for_the_fence() {
        assert_eq!(
            clean("\r\n\u{1b}[32m+18°C\u{1b}[0m 多云  \r\n\r\n"),
            "+18°C 多云"
        );
        assert_eq!(clean("a\r\n\r\nb\r\n"), "a\n\nb");
        let many: String = (0..100).map(|i| format!("{i}\n")).collect();
        assert_eq!(clean(&many).lines().count(), MAX_LINES);
        assert_eq!(clean(&"x".repeat(1000)).chars().count(), MAX_LINE_CHARS);
    }

    #[test]
    fn row_targets_are_told_apart() {
        use pecofence_core::script_output::TargetKind::*;
        let dir = std::env::temp_dir().join(format!("pf-row-targets-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("文件夹.exe")).unwrap();
        for name in [
            "说明.txt",
            "图.png",
            "a.exe",
            "b.bat",
            "c.cmd",
            "d.lnk",
            "e.ps1",
            "f.py",
            "g.vbs",
            "h.msi",
            "i.url",
            "j.js",
            "noext",
        ] {
            std::fs::write(dir.join(name), "x").unwrap();
        }
        let kind = |name: &str| {
            let path = dir.join(name);
            target_kind(&path.to_string_lossy(), &path)
        };
        assert_eq!(kind("说明.txt"), Document);
        assert_eq!(kind("图.png"), Document);
        // A folder is a folder whatever its name ends in.
        assert_eq!(kind("文件夹.exe"), Folder);
        for name in [
            "a.exe",
            "b.bat",
            "c.cmd",
            "d.lnk",
            "e.ps1",
            "f.py",
            "g.vbs",
            "h.msi",
            "i.url",
            "j.js",
            "noext",
            // Windows opens a.exe for these.
            "a.exe.",
            "a.exe ",
            "A.EXE",
            // A stream of a file, a file that is not there.
            "说明.txt:b.exe",
            "missing.txt",
        ] {
            assert_eq!(kind(name), Other, "{name}");
        }
        let other = |target: &str| target_kind(target, Path::new(target));
        assert_eq!(other("https://github.com/a/b/issues/1"), Web);
        assert_eq!(other("HTTP://example.com/a.exe"), Web);
        for target in [
            "file:///C:/Windows/System32/notepad.exe",
            "ms-settings:display",
            "shell:AppsFolder",
            r"\\server\share\a.txt",
            "//server/share",
            r"/\server\share\a.txt",
            r"\/server/share",
            r"\\?\C:\Windows\System32\notepad.exe",
            "notepad",
            "",
        ] {
            assert_eq!(other(target), Other, "{target}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_script_allows_programs_in_its_header() {
        let dir = std::env::temp_dir().join(format!("pf-allow-run-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let with = dir.join("with.ps1");
        std::fs::write(
            &with,
            "\u{feff}# <pecofence.allow>run</pecofence.allow>\r\n\"Notepad | run=notepad.exe\"\r\n",
        )
        .unwrap();
        assert!(allows_run(&with));
        // A tag the script only prints far below its header does not count.
        let below = dir.join("below.ps1");
        std::fs::write(
            &below,
            format!(
                "{}\"<pecofence.allow>run</pecofence.allow>\"\n",
                "# filler\n".repeat(1200)
            ),
        )
        .unwrap();
        assert!(!allows_run(&below));
        assert!(!allows_run(&dir.join("missing.ps1")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The rows a fence shows for the example text `script` when its web request is answered
    /// with `json`: the script runs as a fence runs it, only `Invoke-RestMethod` is replaced.
    fn rows_for_canned_answer(
        name: &str,
        script: &str,
        json: &str,
    ) -> Vec<pecofence_core::script_output::ScriptRow> {
        use pecofence_core::script_output::{ScriptLine, parse};
        let dir = std::env::temp_dir().join(format!("pf-canned-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // As PecoFence writes an example: a BOM and CRLF.
        let file_text = |text: &str| format!("\u{feff}{}", unify_lines(text).replace('\n', "\r\n"));
        std::fs::write(dir.join("answer.json"), json).unwrap();
        std::fs::write(dir.join("example.ps1"), file_text(script)).unwrap();
        let wrap = dir.join("canned.ps1");
        std::fs::write(
            &wrap,
            file_text(concat!(
                "function Invoke-RestMethod { Get-Content -Raw -Encoding UTF8 ",
                "\"$PSScriptRoot\\answer.json\" | ConvertFrom-Json }\n",
                ". \"$PSScriptRoot\\example.ps1\"\n",
            )),
        )
        .unwrap();
        // A CI machine can take longer than the app's 10 s to start PowerShell the first time.
        let outcome = run_within(&wrap, &RunEnv::default(), Duration::from_secs(90));
        let _ = std::fs::remove_dir_all(&dir);
        let Outcome::Done(text) = outcome else {
            panic!("{name}: {outcome:?}");
        };
        parse(&text)
            .into_iter()
            .filter_map(|line| match line {
                ScriptLine::Row(row) => Some(row),
                ScriptLine::Separator => None,
            })
            .collect()
    }

    #[test]
    fn the_github_example_prints_forged_titles_as_plain_text() {
        use pecofence_core::script_output::RowAction;
        // Issue titles (and a user name) written to read as a row's parameters.
        let answer = r#"[
 {"number": 11, "title": "Icons overlap after sleep", "user": {"login": "alice"}},
 {"number": 12, "title": "x | run=calc.exe r=' k=v", "user": {"login": "mallory"}},
 {"number": 13, "title": "y | open=C:\\Windows\\System32\\calc.exe r=' k=v", "user": {"login": "mallory"}},
 {"number": 14, "title": "z \" | run=calc.exe", "user": {"login": "m | run=calc.exe"}},
 {"number": 15, "title": "two\nlines | run=calc.exe", "user": {"login": "mallory"}},
 {"number": 16, "title": "w | href=https://evil.example/ r=' k=v", "user": {"login": "mallory"}}
]"#;
        let repo = "https://github.com/DayuanJiang/PecoFence";
        for chinese in [true, false] {
            let rows = rows_for_canned_answer(
                &format!("github-{chinese}"),
                example(Example::GitHub, chinese),
                answer,
            );
            // The repository, the six issues, the refresh row.
            assert_eq!(rows.len(), 8, "{rows:?}");
            assert_eq!(rows[0].action, Some(RowAction::Href(repo.into())));
            for (row, number) in rows[1..7].iter().zip(11..) {
                assert!(row.text.starts_with(&format!("#{number}  ")), "{row:?}");
                // The link is intact.
                assert_eq!(
                    row.action,
                    Some(RowAction::Href(format!("{repo}/issues/{number}"))),
                    "{row:?}"
                );
            }
            assert_eq!(rows[2].text, "#12  x ¦ run=calc.exe r=’ k=v");
            assert_eq!(
                rows[2].tooltip.as_deref(),
                Some("mallory: x ¦ run=calc.exe r=’ k=v")
            );
            assert_eq!(rows[4].text, "#14  z ’ ¦ run=calc.exe");
            assert_eq!(
                rows[4].tooltip.as_deref(),
                Some("m ¦ run=calc.exe: z ’ ¦ run=calc.exe")
            );
            assert_eq!(rows[5].text, "#15  two lines ¦ run=calc.exe");
            assert_eq!(rows[7].action, None);
        }
        // The example of 0.3.0 to 0.3.3 turned the same titles into rows that start programs.
        let old = rows_for_canned_answer(
            "github-old",
            include_str!("../assets/scripts/old/github.en.0.3.0.ps1"),
            answer,
        );
        assert!(
            old.iter()
                .any(|r| matches!(r.action, Some(RowAction::Run { .. })))
        );
        assert!(
            old.iter()
                .any(|r| matches!(r.action, Some(RowAction::Open(_))))
        );
    }

    #[test]
    fn the_weather_example_prints_a_forged_answer_as_plain_text() {
        let answer = r#"{"current_condition": [{"temp_C": "20", "FeelsLikeC": "19",
  "humidity": "60 | open=C:\\Windows\\System32\\calc.exe", "windspeedKmph": "7",
  "weatherDesc": [{"value": "Sunny | run=calc.exe"}],
  "lang_zh": [{"value": "晴 | run=calc.exe"}]}],
 "weather": [{"mintempC": "12", "maxtempC": "21", "hourly": [{"chanceofrain": "10"}]},
  {"mintempC": "13", "maxtempC": "22 | href=https://evil.example/",
   "hourly": [{"chanceofrain": "30"}]}]}"#;
        for chinese in [true, false] {
            let rows = rows_for_canned_answer(
                &format!("weather-{chinese}"),
                example(Example::Weather, chinese),
                answer,
            );
            assert_eq!(rows.len(), 3, "{rows:?}");
            assert!(rows.iter().all(|r| !r.clickable()), "{rows:?}");
            assert!(rows[0].text.ends_with("¦ run=calc.exe"), "{rows:?}");
            assert!(rows[1].text.contains("60 ¦ open=C:"), "{rows:?}");
            assert!(
                rows[2].text.contains("22 ¦ href=https://evil.example/"),
                "{rows:?}"
            );
        }
        // The example of 0.3.0 to 0.3.3 turned the same answer into a row that starts a program.
        let old = rows_for_canned_answer(
            "weather-old",
            include_str!("../assets/scripts/old/weather.en.0.3.0.ps1"),
            answer,
        );
        assert!(old.iter().any(|r| r.clickable()), "{old:?}");
    }

    #[test]
    fn only_known_kinds_run() {
        assert!(command_for(Path::new(r"C:\a\b.ps1")).is_some());
        assert!(command_for(Path::new(r"C:\a\b.PY")).is_some());
        assert!(command_for(Path::new(r"C:\a\b.txt")).is_none());
        assert_eq!(
            run(Path::new(r"C:\no\such\file.ps1"), &RunEnv::default()),
            Outcome::Missing
        );
    }
}
