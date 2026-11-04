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
    /// Another exit code; what it printed (or wrote to stderr when it printed nothing).
    Failed { code: Option<i32>, text: String },
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

/// Runs `path` once, at most [`TIMEOUT`], in its own folder and without a window.
pub fn run(path: &Path) -> Outcome {
    use std::os::windows::process::CommandExt;
    if !path.is_file() {
        return Outcome::Missing;
    }
    let Some(candidates) = command_for(path) else {
        return Outcome::Unsupported;
    };
    let mut last_error = String::new();
    let mut started = None;
    for mut cmd in candidates {
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW);
        if let Some(dir) = path.parent() {
            cmd.current_dir(dir);
        }
        match cmd.spawn() {
            Ok(child) => {
                started = Some(child);
                break;
            }
            Err(e) => last_error = e.to_string(),
        }
    }
    let Some(mut child) = started else {
        return Outcome::NotStarted(last_error);
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
            Ok(None) if begun.elapsed() >= TIMEOUT => {
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
            text: if stdout.is_empty() { stderr } else { stdout },
        }
    }
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

/// The 脚本栅栏 example: today's weather from wttr.in; `chinese` = the comments and labels in
/// Chinese (the conditions come from the service as they are).
pub fn weather_example(chinese: bool) -> &'static str {
    if chinese {
        "\u{feff}# PecoFence 脚本栅栏示例：显示天气（数据来自 wttr.in）。\r\n# 把下面的 Tokyo 改成你所在的城市（英文或拼音），保存后在栅栏菜单里选「立即运行」。\r\n# 脚本打印出来的每一行文字都会显示在栅栏里。\r\n$city = 'Tokyo'\r\n$w = Invoke-RestMethod \"https://wttr.in/$($city)?format=j1&lang=zh\" -TimeoutSec 8\r\n$now = $w.current_condition[0]\r\n$today = $w.weather[0]\r\n$tomorrow = $w.weather[1]\r\n$desc = $now.lang_zh[0].value\r\nif (-not $desc) { $desc = $now.weatherDesc[0].value }\r\n$rain = ($tomorrow.hourly | Measure-Object -Property chanceofrain -Maximum).Maximum\r\n\"$city  $($now.temp_C)°C  $desc\"\r\n\"体感 $($now.FeelsLikeC)°C，湿度 $($now.humidity)%，风 $($now.windspeedKmph) km/h\"\r\n\"今天 $($today.mintempC) 到 $($today.maxtempC)°C，明天 $($tomorrow.mintempC) 到 $($tomorrow.maxtempC)°C，降雨概率 $rain%\"\r\n"
    } else {
        "\u{feff}# PecoFence script fence example: today's weather (data from wttr.in).\r\n# Change Tokyo below to your city, save, then choose \"Run now\" in the fence menu.\r\n# Every line the script prints shows in the fence.\r\n$city = 'Tokyo'\r\n$w = Invoke-RestMethod \"https://wttr.in/$($city)?format=j1\" -TimeoutSec 8\r\n$now = $w.current_condition[0]\r\n$today = $w.weather[0]\r\n$tomorrow = $w.weather[1]\r\n$rain = ($tomorrow.hourly | Measure-Object -Property chanceofrain -Maximum).Maximum\r\n\"$city  $($now.temp_C)°C  $($now.weatherDesc[0].value)\"\r\n\"Feels like $($now.FeelsLikeC)°C, humidity $($now.humidity)%, wind $($now.windspeedKmph) km/h\"\r\n\"Today $($today.mintempC) to $($today.maxtempC)°C, tomorrow $($tomorrow.mintempC) to $($tomorrow.maxtempC)°C, rain $rain%\"\r\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn only_known_kinds_run() {
        assert!(command_for(Path::new(r"C:\a\b.ps1")).is_some());
        assert!(command_for(Path::new(r"C:\a\b.PY")).is_some());
        assert!(command_for(Path::new(r"C:\a\b.txt")).is_none());
        assert_eq!(run(Path::new(r"C:\no\such\file.ps1")), Outcome::Missing);
    }
}
