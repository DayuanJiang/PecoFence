//! The shareable log: a second `tracing` layer beside the file log that keeps only what may
//! leave the machine with feedback (「附带诊断日志（不含个人信息）」 on the settings page).
//!
//! Nothing user-provided reaches it, by construction rather than by scrubbing afterwards:
//! - messages are the literals written in the code (`tests::callsites_are_shareable` rejects
//!   info / warn / error callsites that format values into the message);
//! - numbers and booleans are kept;
//! - text is kept only for fields whose values the code defines (enum names, static reasons,
//!   module offsets), each binding pinned by the same test; `error` keeps only its HRESULT /
//!   OS error code and `location` only the crate-relative source position;
//! - any other text survives only when it is numeric in shape (HWNDs, rects, ids). Paths, file
//!   and fence names, titles and every other string become [`OMITTED`].
//!
//! Debug / trace events, other crates' events and the test-script driver are left out. Runs of
//! one message (an anchor error repeating every frame) collapse into a count, and a run stops
//! writing at [`MAX_BYTES`] (error lines a little later).

use std::fmt::{self, Write as _};
use std::io::Write;
use std::sync::Mutex;

use tracing::field::{Field, Visit};
use tracing::{Event, Level, Metadata, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::{FormatTime, SystemTime};
use tracing_subscriber::layer::{Context, Layer};

/// Stands in for a value that may carry user data.
pub const OMITTED: &str = "<omitted>";
/// A run's shareable log stops growing here; the feedback page only sends its tail anyway.
const MAX_BYTES: u64 = 2 * 1024 * 1024;
/// Error lines may go this far past [`MAX_BYTES`]: the next start looks for a `PANIC` or
/// `CRASH` line to offer crash feedback.
const ERROR_RESERVE_BYTES: u64 = 256 * 1024;
/// Identical consecutive messages written out before the rest of the run is only counted.
const REPEATS_SHOWN: u32 = 3;

/// Fields whose text the code itself defines. Binding one of these names to a new expression
/// fails `tests::callsites_are_shareable` until the binding is reviewed and listed there.
const CODE_TEXT_FIELDS: &[&str] = &[
    "access",
    "at",
    "behavior",
    "class",
    "command",
    "effect",
    "family",
    "generation",
    "hotkey",
    "language",
    "memory",
    "method",
    "mode",
    "new",
    "old",
    "outcome",
    "payload",
    "position",
    "prop",
    "queue",
    "reason",
    "report",
    "stack",
    "storage",
    "style",
    "verb",
    "wanted",
    "what",
];

/// The text the shareable log writes for a text value of field `name`.
pub fn safe_text(name: &str, value: &str) -> String {
    let kept = match name {
        "error" => error_codes(value),
        "location" => source_location(value),
        _ if CODE_TEXT_FIELDS.contains(&name) => code_text(value),
        _ => numeric_shape(value).then(|| value.to_string()),
    };
    kept.unwrap_or_else(|| OMITTED.to_string())
}

/// Code-defined text, still refused when it looks like it could be anything else (a path, a
/// quoted string, non-ASCII text).
fn code_text(value: &str) -> Option<String> {
    let plain = value.len() <= 4096
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || " _.:+-()[]{},=#<>!*&|`'\n".contains(c));
    plain.then(|| value.to_string())
}

/// The machine-readable part of an error: `0x80070005` (HRESULT) and `os error 5` (Win32).
fn error_codes(value: &str) -> Option<String> {
    let mut codes = Vec::new();
    let bytes = value.as_bytes();
    let mut i = 0;
    while i + 10 <= bytes.len() {
        let hex = |b: &u8| b.is_ascii_hexdigit();
        if bytes[i] == b'0'
            && bytes[i + 1].eq_ignore_ascii_case(&b'x')
            && bytes[i + 2..i + 10].iter().all(hex)
            && !bytes.get(i + 10).is_some_and(hex)
            && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric())
        {
            codes.push(value[i..i + 10].to_string());
            i += 10;
        } else {
            i += 1;
        }
    }
    let mut rest = value;
    while let Some(at) = rest.find("os error ") {
        rest = &rest[at + "os error ".len()..];
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if !digits.is_empty() {
            codes.push(format!("os error {digits}"));
        }
    }
    (!codes.is_empty()).then(|| codes.join(" "))
}

/// A panic location (`file:line:col`). Source paths are the build machine's, so only the part
/// below the workspace (`crates/app/src/x.rs`), the registry (`serde_json-1.0.140/src/de.rs`)
/// or the toolchain (`/rustc/<hash>/library/...`) is kept.
fn source_location(value: &str) -> Option<String> {
    let path = value.replace('\\', "/");
    let tail = if let Some(at) = path.find("/registry/src/") {
        let rest = &path[at + "/registry/src/".len()..];
        rest.split_once('/').map(|(_, crate_path)| crate_path)?
    } else if let Some(at) = path.find("/rustc/") {
        &path[at..]
    } else {
        &path[path.find("crates/")?..]
    };
    let plain = tail.len() <= 300
        && tail
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_.-/:".contains(c));
    plain.then(|| tail.to_string())
}

/// Numbers, `0x` hex, UUIDs and the few words their `Debug` output wraps them in
/// (`HWND(0x1014e)`, `Some((300, 1280))`, `RECT { left: 0, top: 0, .. }`).
fn numeric_shape(value: &str) -> bool {
    const WORDS: &[&str] = &[
        "Some", "None", "Ok", "Err", "true", "false", "HWND", "HKEY", "HMONITOR", "RECT", "POINT",
        "SIZE", "left", "top", "right", "bottom", "x", "y", "cx", "cy", "w", "h", "width",
        "height", "inf", "NaN",
    ];
    if value.len() > 512 {
        return false;
    }
    let bytes = value.as_bytes();
    let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if !word(c) {
            if !b" ()[]{},.:;+-=#|".contains(&c) {
                return false;
            }
            i += 1;
            continue;
        }
        let start = i;
        if is_uuid_at(bytes, i) {
            i += 36;
        } else if c == b'0'
            && bytes
                .get(i + 1)
                .is_some_and(|b| b.eq_ignore_ascii_case(&b'x'))
        {
            i += 2;
            let digits = i;
            while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                i += 1;
            }
            if i == digits {
                return false;
            }
        } else if c.is_ascii_digit() {
            i = number_end(bytes, i);
        } else {
            while i < bytes.len() && word(bytes[i]) {
                i += 1;
            }
            if !WORDS.contains(&&value[start..i]) {
                return false;
            }
        }
        // A number must not run into letters (`12abc`).
        if bytes.get(i).is_some_and(|&b| word(b)) {
            return false;
        }
    }
    true
}

/// End of the decimal number starting at `i`: digits, an optional fraction, an optional exponent.
fn number_end(bytes: &[u8], mut i: usize) -> usize {
    let digits = |bytes: &[u8], mut i: usize| {
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        i
    };
    i = digits(bytes, i);
    if bytes.get(i) == Some(&b'.') && bytes.get(i + 1).is_some_and(u8::is_ascii_digit) {
        i = digits(bytes, i + 1);
    }
    if bytes.get(i).is_some_and(|b| b.eq_ignore_ascii_case(&b'e')) {
        let sign = usize::from(matches!(bytes.get(i + 1), Some(b'+' | b'-')));
        if bytes.get(i + 1 + sign).is_some_and(u8::is_ascii_digit) {
            i = digits(bytes, i + 1 + sign);
        }
    }
    i
}

/// `8-4-4-4-12` hex digits starting at `i`.
fn is_uuid_at(bytes: &[u8], i: usize) -> bool {
    let Some(candidate) = bytes.get(i..i + 36) else {
        return false;
    };
    candidate.iter().enumerate().all(|(k, b)| match k {
        8 | 13 | 18 | 23 => *b == b'-',
        _ => b.is_ascii_hexdigit(),
    })
}

/// Whether an event goes to the shareable log at all.
fn shared(meta: &Metadata<'_>) -> bool {
    *meta.level() <= Level::INFO
        && meta.target().starts_with("pecofence")
        && meta.target() != "pecofence::test"
        && !meta
            .module_path()
            .is_some_and(|m| m.ends_with("::testscript"))
}

/// One event, already reduced to its shareable text.
#[derive(Default)]
struct SafeFields {
    message: String,
    rest: String,
}

impl SafeFields {
    fn push(&mut self, name: &str, value: &str) {
        let _ = write!(self.rest, " {name}={value}");
    }

    fn text(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.push(field.name(), &safe_text(field.name(), value));
        }
    }
}

impl Visit for SafeFields {
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.push(field.name(), &value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.push(field.name(), &value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.push(field.name(), &value.to_string());
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.push(field.name(), &value.to_string());
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.push(field.name(), &value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.push(field.name(), &value.to_string());
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.text(field, value);
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.text(field, &format!("{value:?}"));
    }
}

/// A run of identical consecutive messages.
struct Run {
    key: (Level, &'static str, String),
    shown: u32,
    hidden: u32,
}

struct Sink {
    out: Box<dyn Write + Send>,
    written: u64,
    run: Option<Run>,
}

impl Sink {
    fn event(&mut self, time: &str, level: Level, target: &'static str, fields: SafeFields) {
        let key = (level, target, fields.message);
        match &mut self.run {
            Some(run) if run.key == key => {
                if run.shown >= REPEATS_SHOWN {
                    run.hidden += 1;
                    return;
                }
                run.shown += 1;
            }
            _ => {
                self.end_run(time);
                self.run = Some(Run {
                    key: key.clone(),
                    shown: 1,
                    hidden: 0,
                });
            }
        }
        let (level, target, message) = key;
        self.write(
            level,
            &format!("{time} {level:>5} {target}: {message}{}\n", fields.rest),
        );
    }

    fn end_run(&mut self, time: &str) {
        if let Some(run) = self.run.take()
            && run.hidden > 0
        {
            let (level, target, _) = run.key;
            self.write(
                level,
                &format!(
                    "{time} {level:>5} {target}: (the previous message repeated {} more times)\n",
                    run.hidden
                ),
            );
        }
    }

    fn write(&mut self, level: Level, line: &str) {
        let limit = if level == Level::ERROR {
            MAX_BYTES + ERROR_RESERVE_BYTES
        } else {
            MAX_BYTES
        };
        if self.written >= limit {
            return;
        }
        let before = self.written;
        self.written += line.len() as u64;
        let line = if self.written < limit {
            line
        } else if level == Level::ERROR {
            "(size limit reached: the rest of this run is not recorded)\n"
        } else {
            "(size limit reached: only errors are recorded from here on)\n"
        };
        let _ = self.out.write_all(line.as_bytes());
        // An error line can be the one that ends the room for the other levels.
        if before < MAX_BYTES && (MAX_BYTES..limit).contains(&self.written) {
            let _ = self
                .out
                .write_all(b"(size limit reached: only errors are recorded from here on)\n");
        }
    }
}

/// The layer; see the module docs.
pub struct ShareLayer {
    sink: Mutex<Sink>,
}

impl ShareLayer {
    pub fn new(out: impl Write + Send + 'static) -> Self {
        Self {
            sink: Mutex::new(Sink {
                out: Box::new(out),
                written: 0,
                run: None,
            }),
        }
    }
}

impl<S: Subscriber> Layer<S> for ShareLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        if !shared(meta) {
            return;
        }
        let mut fields = SafeFields::default();
        event.record(&mut fields);
        let mut time = String::new();
        let _ = SystemTime.format_time(&mut Writer::new(&mut time));
        // A panic while the lock was held must not silence the rest of the run.
        let mut sink = self.sink.lock().unwrap_or_else(|e| e.into_inner());
        sink.event(&time, *meta.level(), meta.target(), fields);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tracing_subscriber::prelude::*;

    #[derive(Clone, Default)]
    struct Shared(Arc<Mutex<Vec<u8>>>);

    impl Write for Shared {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Runs `f` under a subscriber with only the share layer; returns what it wrote, without
    /// the timestamps.
    fn capture(f: impl FnOnce()) -> Vec<String> {
        let out = Shared::default();
        let subscriber = tracing_subscriber::registry().with(ShareLayer::new(out.clone()));
        tracing::subscriber::with_default(subscriber, f);
        let text = String::from_utf8(out.0.lock().unwrap().clone()).unwrap();
        text.lines()
            .map(|l| {
                l.split_once(' ')
                    .map(|(_, rest)| rest.trim_start())
                    .unwrap_or(l)
            })
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn user_text_never_reaches_the_shareable_log() {
        let path =
            std::path::PathBuf::from(r"C:\Users\jane.doe\OneDrive\デスクトップ\游戏\resume.pdf");
        let title = "Jane's taxes 2026";
        let lines = capture(|| {
            tracing::info!(path = %path.display(), folder = ?path, %title, count = 12, "portal refreshed");
            tracing::warn!(error = "拒绝访问。 (0x80070005)", hwnd = ?pecofence_platform::HWND(0x105bc as _), "anchor SetWindowPos failed");
            tracing::warn!(
                error = "The system cannot find the file C:\\Users\\jane.doe\\a.txt. (os error 2)",
                "rename failed"
            );
            tracing::info!(
                reason = "startup",
                new = "Some(IconHost { host: HWND(0x1014e), kind: Progman })",
                "icon host changed"
            );
            tracing::info!(
                reason = r"C:\Users\jane.doe",
                what = "\"quoted\"",
                "reasons are code text only"
            );
            tracing::info!(
                name = "jane.doe",
                id = "1024b90d-d073-490c-9380-8143c3d89172",
                rect = "(0, 0, 3840, 2064)",
                "mixed"
            );
            tracing::error!(
                location = r"C:\Users\builder\src\pecofence\crates\app\src\main.rs:12:5",
                "PANIC"
            );
            tracing::debug!(path = "debug events are left out", "debug");
        });
        assert_eq!(
            lines,
            [
                "INFO pecofence::share_log::tests: portal refreshed path=<omitted> folder=<omitted> title=<omitted> count=12",
                "WARN pecofence::share_log::tests: anchor SetWindowPos failed error=0x80070005 hwnd=HWND(0x105bc)",
                "WARN pecofence::share_log::tests: rename failed error=os error 2",
                "INFO pecofence::share_log::tests: icon host changed reason=startup new=Some(IconHost { host: HWND(0x1014e), kind: Progman })",
                "INFO pecofence::share_log::tests: reasons are code text only reason=<omitted> what=<omitted>",
                "INFO pecofence::share_log::tests: mixed name=<omitted> id=1024b90d-d073-490c-9380-8143c3d89172 rect=(0, 0, 3840, 2064)",
                "ERROR pecofence::share_log::tests: PANIC location=crates/app/src/main.rs:12:5",
            ]
        );
    }

    #[test]
    fn numeric_shapes() {
        for ok in [
            "0",
            "-12.5",
            "1e-3",
            "0x80070005",
            "HWND(0x105bc)",
            "Some((300, 1280))",
            "None",
            "[255, 0, 128]",
            "RECT { left: 0, top: 0, right: 10, bottom: 20 }",
            "1024b90d-d073-490c-9380-8143c3d89172",
        ] {
            assert!(numeric_shape(ok), "{ok}");
        }
        for refused in [
            "jane",
            "\"2024\"",
            "C:\\x",
            "a/b",
            "12abc",
            "0x",
            "face",
            "d073-490c",
            "Some(\"x\")",
            "张三",
            "jane@example.com",
        ] {
            assert!(!numeric_shape(refused), "{refused}");
        }
    }

    #[test]
    fn errors_keep_only_their_codes() {
        assert_eq!(
            error_codes("拒绝访问。 (0x80070005)").as_deref(),
            Some("0x80070005")
        );
        assert_eq!(
            error_codes("Access is denied. (os error 5)").as_deref(),
            Some("os error 5")
        );
        assert_eq!(error_codes("file jane0x12345678.txt is busy"), None);
        assert_eq!(error_codes("expected value at line 1 column 1"), None);
    }

    #[test]
    fn source_locations_drop_the_build_machine_path() {
        assert_eq!(
            source_location(r"C:\Users\me\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\serde_json-1.0.140\src\de.rs:10:2").as_deref(),
            Some("serde_json-1.0.140/src/de.rs:10:2")
        );
        assert_eq!(
            source_location("/rustc/abc123/library/core/src/option.rs:1:2").as_deref(),
            Some("/rustc/abc123/library/core/src/option.rs:1:2")
        );
        assert_eq!(
            source_location(r"crates\app\src\main.rs:5:9").as_deref(),
            Some("crates/app/src/main.rs:5:9")
        );
        assert_eq!(source_location(r"C:\Users\jane\elsewhere\lib.rs:1:1"), None);
    }

    #[test]
    fn repeated_messages_collapse_into_a_count() {
        let lines = capture(|| {
            for hwnd in 0..10 {
                tracing::warn!(hwnd, "anchor SetWindowPos failed");
            }
            tracing::info!("desktop icons already hidden by user/other tool; leaving as is");
        });
        assert_eq!(
            lines,
            [
                "WARN pecofence::share_log::tests: anchor SetWindowPos failed hwnd=0",
                "WARN pecofence::share_log::tests: anchor SetWindowPos failed hwnd=1",
                "WARN pecofence::share_log::tests: anchor SetWindowPos failed hwnd=2",
                "WARN pecofence::share_log::tests: (the previous message repeated 7 more times)",
                "INFO pecofence::share_log::tests: desktop icons already hidden by user/other tool; leaving as is",
            ]
        );
    }

    #[test]
    fn errors_are_kept_past_the_size_limit() {
        let out = Shared::default();
        let mut sink = Sink {
            out: Box::new(out.clone()),
            written: 0,
            run: None,
        };
        let filler = format!("t  INFO pecofence: {}\n", "x".repeat(1000));
        while sink.written < MAX_BYTES {
            sink.write(Level::INFO, &filler);
        }
        sink.write(Level::INFO, "t  INFO pecofence: dropped\n");
        sink.write(
            Level::ERROR,
            "t ERROR pecofence: PANIC location=crates/app/src/x.rs:1:2\n",
        );
        let text = String::from_utf8(out.0.lock().unwrap().clone()).unwrap();
        assert!(text.contains("size limit reached"));
        assert!(!text.contains("dropped"));
        assert!(text.ends_with("PANIC location=crates/app/src/x.rs:1:2\n"));
    }

    #[test]
    fn an_error_line_that_reaches_the_size_limit_says_so() {
        let out = Shared::default();
        let mut sink = Sink {
            out: Box::new(out.clone()),
            written: 0,
            run: None,
        };
        let filler = format!("t  INFO pecofence: {}\n", "x".repeat(1000));
        while sink.written + (filler.len() as u64) < MAX_BYTES {
            sink.write(Level::INFO, &filler);
        }
        let error = format!("t ERROR pecofence: {}\n", "e".repeat(2000));
        sink.write(Level::ERROR, &error);
        sink.write(Level::INFO, "t  INFO pecofence: dropped\n");
        let text = String::from_utf8(out.0.lock().unwrap().clone()).unwrap();
        assert!(!text.contains("dropped"));
        assert!(text.ends_with(&format!(
            "{error}(size limit reached: only errors are recorded from here on)\n"
        )));
    }

    // ---- Source audit: every info / warn / error callsite in the workspace -------------------

    /// Bindings of [`CODE_TEXT_FIELDS`] (and `location`) reviewed as code-defined, whitespace
    /// removed. Before adding one, check that the value cannot hold a path, a file / folder /
    /// fence name, a title or anything else the user typed.
    const REVIEWED: &[(&str, &str)] = &[
        ("access", "access"),      // crashlog: read / write / execute / other
        ("at", "%at"),             // crashlog: `module+rva`
        ("at", "script"),          // settings page script: fixed names
        ("behavior", "?behavior"), // ShowDesktopBehavior enum
        ("class", "%class"),       // window class (under the cursor / above the desktop)
        ("class", "%cls"),         // class of the window that ended a peek
        // Class of the window the fences go above.
        ("class", "%desktop::class_name(h)"),
        ("command", "name"),            // `Command` variant name
        ("effect", "?effect"),          // DropEffect enum
        ("family", "%f.family"),        // installed font family
        ("generation", "?generation"),  // DesktopGeneration enum
        ("generation", "generation"),   // GPU device generation (u64)
        ("hotkey", "%c"),               // Hotkey: modifier and key names ("Ctrl+Alt+Space")
        ("hotkey", "%h"),               // Hotkey
        ("language", "language.tag()"), // BCP 47 tag of a Language variant
        ("location", "%location"),      // panic `file:line`
        ("memory", "%m.summary()"),     // MemoryStats::summary: numbers only
        ("method", "method"),           // IPC method name
        ("mode", "?mode"),              // ThemeMode enum
        ("new", "?resolved"),           // Option<IconHost>: HWNDs + kind enum
        ("old", "?self.host"),          // Option<IconHost>
        ("outcome", "?effect.as_ref().map(|r|(r.dropped,r.effect))"), // (bool, DropEffect)
        ("payload", "payload"),         // panic payload when `&'static str`
        ("position", "?snapshot.position"), // WallpaperPosition enum
        ("prop", "prop"),               // fence property key from our settings page
        ("queue", "stack.queue_path"),  // composition queue kind, static
        ("reason", "reason"),           // `&'static str` reasons (reanchor / sync)
        ("report", "?report"),          // SyncReport: counts
        ("stack", "%frames.join(\"\\n\")"), // crashlog: `module+rva` per frame
        ("storage", "state.storage_kind()"), // fixed names of the config folder's kind
        ("style", "?style"),            // backdrop style enum
        ("verb", "verb"),               // shell verb (canonical name)
        ("verb", "%verb"),
        ("wanted", "%h"),   // Hotkey
        ("what", "what"),   // `&'static str` operation names
        ("what", "e.step"), // ConfigStore::save step name
        ("what", "kind"),   // JavaScript error type from a fixed list
    ];

    struct Callsite {
        at: String,
        target: Option<String>,
        message: Option<String>,
        positional: usize,
        fields: Vec<(String, String)>,
    }

    /// Splits a macro body on top-level commas (strings, char literals, comments and nesting
    /// respected).
    fn split_args(body: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut depth = 0i32;
        let mut chars = body.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '"' => {
                    current.push(c);
                    while let Some(s) = chars.next() {
                        current.push(s);
                        if s == '\\' {
                            if let Some(e) = chars.next() {
                                current.push(e);
                            }
                        } else if s == '"' {
                            break;
                        }
                    }
                }
                '/' if chars.peek() == Some(&'/') => {
                    for s in chars.by_ref() {
                        if s == '\n' {
                            break;
                        }
                    }
                }
                '(' | '[' | '{' => {
                    depth += 1;
                    current.push(c);
                }
                ')' | ']' | '}' => {
                    depth -= 1;
                    current.push(c);
                }
                ',' if depth == 0 => parts.push(std::mem::take(&mut current)),
                _ => current.push(c),
            }
        }
        parts.push(current);
        parts
            .into_iter()
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect()
    }

    /// The body of the macro call whose `(` ends at `open` (exclusive).
    fn macro_body(src: &str, open: usize) -> &str {
        let bytes = src.as_bytes();
        let (mut depth, mut i) = (1, open);
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'"' => {
                    i += 1;
                    while i < bytes.len() && bytes[i] != b'"' {
                        i += if bytes[i] == b'\\' { 2 } else { 1 };
                    }
                }
                b'/' if bytes.get(i + 1) == Some(&b'/') => {
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                }
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            i += 1;
        }
        &src[open..i - 1]
    }

    fn callsites(path: &std::path::Path, src: &str) -> Vec<Callsite> {
        let mut out = Vec::new();
        for level in ["info", "warn", "error"] {
            let needle = format!("tracing::{level}!(");
            for (start, _) in src.match_indices(&needle) {
                let body = macro_body(src, start + needle.len());
                let line = src[..start].matches('\n').count() + 1;
                let mut site = Callsite {
                    at: format!("{}:{line}", path.display()),
                    target: None,
                    message: None,
                    positional: 0,
                    fields: Vec::new(),
                };
                for part in split_args(body) {
                    if let Some(target) = part.strip_prefix("target:") {
                        site.target = Some(target.trim().trim_matches('"').to_string());
                    } else if part.starts_with('"') {
                        site.message = Some(part);
                    } else if site.message.is_some() {
                        site.positional += 1;
                    } else {
                        let (name, expr) = match part.split_once('=') {
                            Some((name, expr)) if !name.contains(['(', '"']) => {
                                (name.trim().to_string(), expr.to_string())
                            }
                            _ => {
                                let name = part.trim_start_matches(['%', '?']).trim();
                                (name.to_string(), part.clone())
                            }
                        };
                        let expr: String = expr.chars().filter(|c| !c.is_whitespace()).collect();
                        site.fields.push((name, expr));
                    }
                }
                out.push(site);
            }
        }
        out
    }

    fn rust_sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n != "target") {
                    rust_sources(&path, out);
                }
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    /// The guarantee the module docs make, checked against the source: no info / warn / error
    /// message formats a value in, and code-text fields only bind reviewed expressions. The
    /// test-script driver is exempt: the layer drops its events.
    #[test]
    fn callsites_are_shareable() {
        let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut files = Vec::new();
        rust_sources(&crates, &mut files);
        let mut failures = Vec::new();
        let mut seen = 0;
        for file in files {
            let name = file.file_name().unwrap().to_string_lossy();
            if name == "bindings.rs" || name == "gpu_bindings.rs" || name == "testscript.rs" {
                continue;
            }
            // This file's own tests log user-like text on purpose.
            if name == "share_log.rs" {
                continue;
            }
            let src = std::fs::read_to_string(&file).unwrap();
            for site in callsites(&file, &src) {
                seen += 1;
                if site.target.as_deref() == Some("pecofence::test") {
                    continue;
                }
                let message = site.message.as_deref().unwrap_or("");
                let literal = message.replace("{{", "").replace("}}", "");
                if site.positional > 0 || literal.contains('{') {
                    failures.push(format!(
                        "{}: message {message} formats values in; pass them as fields",
                        site.at
                    ));
                }
                for (field, expr) in &site.fields {
                    let pinned = CODE_TEXT_FIELDS.contains(&field.as_str()) || field == "location";
                    if pinned && !REVIEWED.contains(&(field.as_str(), expr.as_str())) {
                        failures.push(format!(
                            "{}: `{field} = {expr}` is kept as text in the shareable log; review that it cannot carry user data, then add it to REVIEWED",
                            site.at
                        ));
                    }
                }
            }
        }
        assert!(seen > 150, "callsite scan found only {seen} callsites");
        assert!(failures.is_empty(), "\n{}", failures.join("\n"));
    }
}
