//! `pecofence-plugin-check <plugin dir> [--seconds 60] [--fixture answers.json] [--lang en]
//! [--settings '{"key": value}'] [--json result.json]`
//!
//! Runs a plugin's `main.js` for a while in PecoFence's own plugin host (`crates/plugin-host`)
//! with fake host functions, on any system: `pf.http` answers come from the fixture or say
//! "offline", the clipboard and the computer's state are fixed values, nothing reaches Windows or
//! the network. It drives the fence through what a user does (resize, hide and show, settings, a
//! time change, a click on every row and cell with an id), then leaves it at rest, and reports
//! every pf call, every refused permission, the hosts the plugin tried, how often it showed a
//! view, its memory and its slowest call. The plugin repository's checks run it on every pull
//! request. Ends with `RESULT: PASS` or `RESULT: FAIL` (exit code 0 or 1); WARN lines are the
//! soft rules of the repository's AGENTS.md.

use pecofence_core::FenceId;
use pecofence_core::plugins::{Manifest, fence_settings};
use pecofence_plugin_host::{self as host, Command, Host, Note, Output, Start, Stopped};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

const OFFLINE: &str =
    "offline (pecofence-plugin-check has no network; give answers with --fixture)";

/// What the plugin did, as the host saw it.
#[derive(Default)]
struct Log {
    calls: BTreeMap<String, u64>,
    refused: Vec<String>,
    /// Requests by host: all of them, and those at rest.
    hosts: BTreeMap<String, (u64, u64)>,
    last_view: Value,
    rest_show_calls: usize,
    js_ms: f64,
    rest_js_ms: f64,
    slowest: (f64, String),
    memory: usize,
    data: BTreeMap<String, String>,
    /// Errors that escaped a handler, a timer or a Promise ("<run>: <message> <stack line>"),
    /// and how often each came.
    errors: BTreeMap<String, usize>,
}

struct Fake {
    /// Address prefixes and their answers ({status, headers, body}).
    fixture: Vec<(String, Value)>,
    rest: AtomicBool,
    log: Mutex<Log>,
}

impl Fake {
    fn log(&self) -> MutexGuard<'_, Log> {
        self.log.lock().unwrap_or_else(|e| e.into_inner())
    }
}

fn url_host(url: &str) -> String {
    let rest = url.split_once("://").map_or("", |(_, r)| r);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority.rsplit('@').next().unwrap_or_default();
    host.split(':')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// At most 160 characters of a JSON value, for the report.
fn short(v: &Value) -> String {
    let s = v.to_string();
    match s.char_indices().nth(160) {
        Some((i, _)) => format!("{}…", &s[..i]),
        None => s,
    }
}

impl Host for Fake {
    fn call(&self, _: FenceId, name: &str, args: Value) -> Result<Value, String> {
        // The plugin host checked the call against the manifest already (`host::refusal`, and
        // again at each redirect); refused calls come to `note`.
        let arg = |i: usize| args.get(i).and_then(Value::as_str).unwrap_or_default();
        let mut log = self.log();
        match name {
            "http.get" | "http.post" => {
                let url = arg(0);
                let answer = self
                    .fixture
                    .iter()
                    .filter(|(p, _)| url.starts_with(p.as_str()));
                let (_, a) = answer.max_by_key(|(p, _)| p.len()).ok_or(OFFLINE)?;
                let field = |k: &str, or: Value| a.get(k).cloned().unwrap_or(or);
                Ok(json!({
                    "status": field("status", json!(200)),
                    "headers": field("headers", json!({})),
                    "body": field("body", json!("")),
                }))
            }
            "data.read" => Ok(log.data.get(arg(0)).map_or(Value::Null, |t| json!(t))),
            "data.write" => {
                let text = args.get(1).map_or(String::new(), |v| match v {
                    Value::String(s) => s.clone(),
                    v => v.to_string(),
                });
                log.data.insert(arg(0).to_string(), text);
                Ok(Value::Null)
            }
            "data.list" => Ok(json!(log.data.keys().collect::<Vec<_>>())),
            "data.remove" => Ok(log
                .data
                .remove(arg(0))
                .map_or(json!(false), |_| json!(true))),
            // The shapes of the app's answers (crates/app/src/app/light_host.rs).
            "clipboard.text" => Ok(json!({ "text": "text on the clipboard", "truncated": false })),
            "clipboard.files" => Ok(json!([r"C:\Users\me\Desktop\notes.txt"])),
            "sys.cpu" => Ok(json!(12)),
            "sys.memory" => Ok(json!(38)),
            "sys.drives" => Ok(json!([
                { "letter": "C", "free": 200_000_000_000u64, "size": 500_000_000_000u64 }
            ])),
            "sys.ip" => Ok(json!("192.168.1.20")),
            "sys.network" => Ok(json!([{
                "up": true, "gateway": true, "physical": true, "addresses": ["192.168.1.20"]
            }])),
            "sys.battery" => Ok(json!({ "percent": 80, "plugged": true })),
            "run" => Ok(json!({ "code": 0, "output": "" })),
            "folder.list" => Ok(json!([])),
            "folder.read" => Ok(json!("")),
            // Nothing on the clipboard as a picture, the user cancels the folder dialog, and
            // opening, notices and clipboard writes only happen.
            "clipboard.image" | "folder.pick" | "openSystem" | "open" | "notify" => Ok(Value::Null),
            n if n.starts_with("clipboard.set") => Ok(Value::Null),
            _ => Err(format!("{name} is not a pf function")),
        }
    }

    fn note(&self, _: FenceId, note: Note) {
        let rest = self.rest.load(Ordering::Relaxed);
        let mut log = self.log();
        match note {
            Note::Call {
                name,
                args,
                refused,
            } => {
                *log.calls.entry(name.clone()).or_default() += 1;
                if name.starts_with("http.") {
                    let host = url_host(args.get(0).and_then(Value::as_str).unwrap_or_default());
                    let n = log.hosts.entry(host).or_default();
                    n.0 += 1;
                    n.1 += u64::from(rest);
                }
                if name == "show" {
                    log.last_view = args.get(0).cloned().unwrap_or_default();
                    log.rest_show_calls += usize::from(rest);
                }
                if let Some(why) = refused {
                    log.refused.push(format!("{name} {}: {why}", short(&args)));
                }
            }
            Note::Ran { what, ms, memory } => {
                log.js_ms += ms;
                if rest {
                    log.rest_js_ms += ms;
                }
                if ms > log.slowest.0 {
                    log.slowest = (ms, what);
                }
                log.memory = log.memory.max(memory);
            }
            Note::Error { what, message, at } => {
                *log.errors
                    .entry(format!("{what}: {message} {at}"))
                    .or_default() += 1;
            }
        }
    }
}

/// `pf.time.format` in UTC, in the shapes Windows' formats have.
fn format(ms: i64, kind: &str) -> String {
    let secs = ms.div_euclid(1000);
    let (days, day_secs) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let (h, m, s) = (day_secs / 3600, day_secs / 60 % 60, day_secs % 60);
    // Days since 1970 to a calendar date (Howard Hinnant's civil_from_days).
    let z = days + 719_468;
    let (era, doe) = (z.div_euclid(146_097), z.rem_euclid(146_097));
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(mo <= 2);
    let week = [
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
    ];
    match kind {
        "seconds" => format!("{h}:{m:02}:{s:02}"),
        "date" => format!("{y}-{mo:02}-{d:02}"),
        "short" => format!("{mo}/{d}"),
        "weekday" => week[days.rem_euclid(7) as usize].to_string(),
        _ => format!("{h}:{m:02}"),
    }
}

/// The events a user's clicks send for the view's rows and cells with an id.
fn clicks(view: &Value) -> Vec<(&'static str, Value)> {
    let mut out = Vec::new();
    let rows = view.get("rows").and_then(Value::as_array).cloned();
    for row in rows.unwrap_or_default().iter().take(20) {
        let cells = row.get("cells").and_then(Value::as_array).cloned();
        for item in std::iter::once(row).chain(cells.unwrap_or_default().iter()) {
            let Some(id) = item.get("id").cloned() else {
                continue;
            };
            out.push(("click", json!({ "id": id })));
            out.push(("dblclick", json!({ "id": id })));
            if item.get("check").is_some() || item.get("todo").is_some() {
                out.push(("change", json!({ "id": id, "value": true })));
            }
            if item.get("slider").is_some() {
                out.push(("change", json!({ "id": id, "value": 0.5 })));
            }
        }
        if let Some(input) = row.get("input") {
            let id = input.get("id").or(row.get("id")).cloned();
            out.push(("text", json!({ "id": id, "text": "typed by the checker" })));
        }
    }
    out
}

struct Args {
    dir: PathBuf,
    seconds: f64,
    fixture: Option<PathBuf>,
    lang: String,
    /// The fence's saved settings (the manifest's defaults for the rest).
    settings: serde_json::Map<String, Value>,
    json: Option<PathBuf>,
}

fn args() -> Result<Args, String> {
    let mut a = Args {
        dir: PathBuf::new(),
        seconds: 60.0,
        fixture: None,
        lang: "en".into(),
        settings: serde_json::Map::new(),
        json: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        let mut value = || it.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--seconds" => a.seconds = value()?.parse().map_err(|_| "--seconds takes a number")?,
            "--fixture" => a.fixture = Some(value()?.into()),
            "--lang" => a.lang = value()?,
            "--settings" => {
                let v = serde_json::from_str(&value()?).map_err(|e| format!("--settings: {e}"))?;
                a.settings = v;
            }
            "--json" => a.json = Some(value()?.into()),
            _ if !arg.starts_with('-') && a.dir.as_os_str().is_empty() => a.dir = arg.into(),
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    if a.dir.as_os_str().is_empty() || a.seconds < 5.0 {
        return Err(
            "usage: pecofence-plugin-check <plugin dir> [--seconds 60 (at least 5)] \
                    [--fixture answers.json] [--lang en] [--settings JSON] [--json result.json]"
                .into(),
        );
    }
    Ok(a)
}

fn read_json(path: &Path) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    serde_json::from_slice(bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn main() {
    std::process::exit(match args().and_then(check) {
        Ok(passed) => i32::from(!passed),
        Err(e) => {
            println!("FAIL: {e}\nRESULT: FAIL");
            1
        }
    })
}

/// Runs the plugin and prints the report; whether it passed.
fn check(a: Args) -> Result<bool, String> {
    let manifest: Manifest = serde_json::from_value(read_json(&a.dir.join("plugin.json"))?)
        .map_err(|e| format!("plugin.json: {e}"))?;
    let main = a.dir.join("main.js");
    if !main.exists() && manifest.layer == "web" {
        println!(
            "PASS: {} is a web plugin without main.js: nothing to run\nRESULT: PASS",
            manifest.id
        );
        return Ok(true);
    }
    let code = std::fs::read_to_string(&main).map_err(|e| format!("main.js: {e}"))?;
    let fixture = match &a.fixture {
        Some(path) => match read_json(path)? {
            Value::Object(m) => m.into_iter().collect(),
            _ => return Err("the fixture is an object of addresses and answers".into()),
        },
        None => Vec::new(),
    };
    let fake = Arc::new(Fake {
        fixture,
        rest: AtomicBool::new(false),
        log: Mutex::default(),
    });
    let (out_tx, out_rx) = std::sync::mpsc::channel();
    let send_out = move |o: Output| _ = out_tx.send((Instant::now(), o));
    let (tx, thread) = host::spawn(fake.clone(), format, send_out).map_err(|e| e.to_string())?;
    let fence = FenceId::new_v4();
    let settings = fence_settings(&manifest, &a.settings);
    let begun = Instant::now();
    let send = |c: Command| _ = tx.send(c);
    send(Command::Start(Box::new(Start {
        fence,
        id: manifest.id.clone(),
        version: manifest.version.clone(),
        permissions: manifest.permissions.clone(),
        code,
        init: json!({
            "lang": a.lang, "theme": "dark", "settings": settings,
            "fence": { "id": fence.to_string(), "width": 320.0, "height": 240.0 },
        }),
        shown: true,
        cache: None,
        number: 1,
    })));
    let wait = |part: f64| {
        let at = begun + Duration::from_secs_f64(a.seconds * part);
        std::thread::sleep(at.saturating_duration_since(Instant::now()));
    };
    // What a user does in the first half; the second half is the plugin at rest.
    wait(0.1);
    send(Command::Event(
        fence,
        "resize",
        json!({ "width": 480.0, "height": 300.0 }),
    ));
    wait(0.15);
    send(Command::Shown(fence, false));
    wait(0.25);
    send(Command::Shown(fence, true));
    wait(0.3);
    send(Command::Event(fence, "settings", settings));
    wait(0.35);
    send(Command::Event(fence, "timechange", Value::Null));
    wait(0.4);
    let view = fake.log().last_view.clone();
    let clicks = clicks(&view);
    let clicked = clicks.len();
    for (name, data) in clicks {
        send(Command::Event(fence, name, data));
        std::thread::sleep(Duration::from_millis(50));
    }
    if manifest.permissions.clipboard.reads() {
        send(Command::Event(fence, "clipboard", Value::Null));
    }
    wait(0.5);
    let rest = Instant::now();
    fake.rest.store(true, Ordering::Relaxed);
    wait(1.0);
    let rest_secs = rest.elapsed().as_secs_f64();
    send(Command::Quit);
    thread.join().map_err(|_| "the plugin thread panicked")?;

    let (mut shows, mut stops) = (Vec::new(), Vec::new());
    for (at, o) in out_rx.try_iter() {
        match o {
            Output::Show(..) => shows.push(at),
            Output::Stopped(_, _, Stopped::TooLong) => stops.push("a run took over 50 ms".into()),
            Output::Stopped(_, _, Stopped::TooBusy) => {
                stops.push("over 200 ms of JavaScript in one second".into())
            }
            Output::Stopped(_, _, Stopped::TooMuchMemory) => stops.push("it used over 8 MB".into()),
            Output::Stopped(_, _, Stopped::Error(e)) => stops.push(format!("an error: {e}")),
            // A web plugin's main.js posting to its page: nothing to check here.
            Output::Post(..) => {}
        }
    }
    let log = fake.log();
    let m = &manifest;
    let (mut fails, mut warns, mut info) = (Vec::new(), Vec::new(), Vec::new());
    let calls: Vec<String> = log.calls.iter().map(|(k, v)| format!("{k} {v}")).collect();
    let calls = if calls.is_empty() {
        "none".into()
    } else {
        calls.join(", ")
    };
    info.push(format!("pf calls: {calls}"));
    info.push(format!("{clicked} clicks on rows and cells with an id"));
    info.push(format!("last view: {}", short(&log.last_view)));
    for (host, (all, at_rest)) in &log.hosts {
        let declared = m
            .permissions
            .network
            .iter()
            .any(|h| h.eq_ignore_ascii_case(host));
        let state = if declared {
            "declared"
        } else {
            "NOT DECLARED, refused"
        };
        info.push(format!(
            "host {host}: {all} request(s), {at_rest} at rest ({state})"
        ));
        if declared && rest_secs >= 30.0 && *at_rest as f64 * 60.0 / rest_secs > 1.0 {
            warns.push(format!("more than one request a minute to {host} at rest"));
        }
    }
    let per_second = |n: usize| n as f64 / rest_secs;
    let rest_views = per_second(shows.iter().filter(|t| **t >= rest).count());
    let rest_calls = per_second(log.rest_show_calls);
    let rest_ms = log.rest_js_ms / rest_secs;
    info.push(format!(
        "views shown: {} in all; at rest {rest_views:.2} a second ({rest_calls:.2} pf.show calls)",
        shows.len()
    ));
    info.push(format!(
        "JavaScript: {:.1} ms in all, {rest_ms:.2} ms a second at rest, slowest run {:.2} ms ({}); \
         memory of the engine at most {} KB",
        log.js_ms,
        log.slowest.0,
        log.slowest.1,
        log.memory / 1024
    ));
    match shows.first().map(|t| (*t - begun).as_secs_f64()) {
        None => fails.push("the plugin never showed a view (pf.show)".to_string()),
        Some(s) if s > 1.0 => {
            warns.push(format!("the first view came after {s:.1} s (at most 1 s)"))
        }
        Some(_) => {}
    }
    let soft = [
        (
            rest_calls > 1.0,
            "more than one pf.show a second at rest".to_string(),
        ),
        (
            rest_ms > 10.0,
            "more than 10 ms of JavaScript a second at rest".into(),
        ),
        (
            log.slowest.0 > 20.0,
            format!(
                "a run of {:.1} ms ({}): keep each under 20 ms",
                log.slowest.0, log.slowest.1
            ),
        ),
        (
            log.memory > 4 << 20,
            "the engine used more than 4 MB".into(),
        ),
    ];
    warns.extend(soft.into_iter().filter(|(over, _)| *over).map(|(_, w)| w));
    let mut refused: BTreeMap<&String, usize> = BTreeMap::new();
    for r in &log.refused {
        *refused.entry(r).or_default() += 1;
    }
    fails.extend(refused.iter().map(|(r, n)| match n {
        1 => format!("refused: {r}"),
        n => format!("refused {n} times: {r}"),
    }));
    fails.extend(
        stops
            .iter()
            .map(|s| format!("PecoFence stopped the plugin: {s}")),
    );
    fails.extend(log.errors.iter().map(|(e, n)| match n {
        1 => format!("an error escaped {e}"),
        n => format!("an error escaped {n} times {e}"),
    }));
    let v = env!("CARGO_PKG_VERSION");
    println!(
        "PASS: ran {} {} ({} layer) for {:.0} s in PecoFence {v}'s plugin host",
        m.id, m.version, m.layer, a.seconds
    );
    let lines = [("INFO", &info), ("WARN", &warns), ("FAIL", &fails)];
    for (kind, list) in lines {
        list.iter().for_each(|l| println!("{kind}: {l}"));
    }
    if let Some(path) = &a.json {
        let result = json!({
            "plugin": m.id, "version": m.version, "seconds": a.seconds, "calls": log.calls,
            "refused": log.refused, "hosts": log.hosts, "views": shows.len(),
            "viewsPerSecondAtRest": rest_views, "showCallsPerSecondAtRest": rest_calls,
            "jsMs": log.js_ms, "jsMsPerSecondAtRest": rest_ms, "slowestMs": log.slowest.0,
            "slowest": log.slowest.1, "memoryBytes": log.memory, "stopped": stops,
            "errors": log.errors,
            "warnings": warns, "failures": fails, "lastView": log.last_view,
        });
        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        std::fs::write(path, text).map_err(|e| e.to_string())?;
    }
    println!("RESULT: {}", if fails.is_empty() { "PASS" } else { "FAIL" });
    Ok(fails.is_empty())
}
