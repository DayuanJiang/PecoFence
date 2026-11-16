//! The light layer of plugins (插件的轻量层): each plugin fence runs its plugin's `main.js` in a
//! QuickJS runtime of its own (at most 8 MB, 50 ms per call and 200 ms a second, no `eval` or
//! `Function`), all of them on one plugin thread. A plugin talks to PecoFence through `pf` (see
//! `prelude.js`): it shows a view, sets timers that PecoFence aligns to the clock, gets events,
//! and asks for what only the app can do (`api.rs`), each call checked against its manifest and
//! done on a worker thread, its answer a Promise. Nothing here touches Windows.

use pecofence_core::FenceId;
use pecofence_core::plugins::Permissions;
use rquickjs::{CaughtError, Context, Ctx, Exception, Function, Object, Persistent, Runtime};
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod api;
pub mod view;
pub use api::{Host, MAX_ANSWER, Note, perform, refusal, url_host};
pub use view::{Row, View};

pub const MEMORY_LIMIT: usize = 8 * 1024 * 1024;
pub const CALL_LIMIT: Duration = Duration::from_millis(50);
/// JavaScript time a plugin may use in a second: starts, timers, events, answers and the
/// Promise jobs they queue, together.
pub const BUDGET: Duration = Duration::from_millis(200);
/// `pf.show` takes effect at most 10 times a second: a view that comes sooner waits, and a
/// later one takes its place.
const SHOW_GAP: Duration = Duration::from_millis(100);
const MAX_TIMERS: usize = 20;
/// Requests (`pf.http`) a plugin may have going at once, and calls of any kind (a web plugin's
/// page too).
pub const MAX_REQUESTS: usize = 4;
pub const MAX_CALLS: usize = 16;
/// What `call` names the run of main.js itself: an error there stops the plugin (in any other
/// run it is logged and the plugin goes on).
const MAIN: &str = "main.js";
/// The different error messages a plugin's log gets in full; the rest only in the debug log.
const LOGGED_ERRORS: usize = 100;
/// The errors of a start that escape a handler and the app hears of ([`Output::Errored`]): the
/// app takes an update back after this many in its first minute.
pub const ERRORS_TOLD: u32 = 3;
/// The last view goes into the fence's cache file at most this often (and when it stops).
const CACHE_GAP: Duration = Duration::from_secs(60);
/// The events PecoFence sends ([`Command::Event`] takes one of these names).
pub const EVENTS: [&str; 23] = [
    "start",
    "show",
    "hide",
    "resize",
    "click",
    "dblclick",
    "change",
    "text",
    "settings",
    "clipboard",
    "timechange",
    "stop",
    "message",
    "theme",
    // Interface 2: a command of the fence menu (`view.menu`) or a shortcut ({id, via}), what
    // was dropped on the fence ({files: [{name, size, folder}], text, url, row, drop}), the media
    // session changed, a program of `apps` started, ended or came to the front ({running, front}).
    "command",
    "drop",
    "media",
    "lang",
    "apps",
    // A multi-line box's Ctrl+Enter ({id}); the pointer onto and off a `hover: "event"` row
    // ({row}); the wheel over a `wheel: true` row or cell ({id, steps}). The last three are paced.
    "submit",
    "enter",
    "leave",
    "wheel",
];
/// `enter` and `leave` reach a plugin at most 10 times a second, `wheel` 20 times: what comes
/// sooner waits, the pointer's last row and the wheel's steps added up.
const HOVER_GAP: Duration = Duration::from_millis(100);
const WHEEL_GAP: Duration = Duration::from_millis(50);
/// The most bytes one `pf.post` may carry (as JSON; a web plugin page's message too), and the
/// most messages and bytes that may wait to go out from one run of the plugin.
pub const MAX_POST: usize = 1024 * 1024;
const MAX_POSTS: usize = 100;
const MAX_POSTED: usize = 4 * 1024 * 1024;
/// `pf.time.format(unix_ms, kind)`: the time in Windows' region format.
pub type Format = fn(i64, &str) -> String;

/// Fence `fence`'s plugin starts: its code and what `pf` says about the fence.
#[derive(Default)]
pub struct Start {
    pub fence: FenceId,
    /// The plugin's id and version, and what its installed manifest allows.
    pub id: String,
    pub version: String,
    pub permissions: Permissions,
    pub code: String,
    /// `pf.lang`, `pf.theme`, `pf.settings` and `pf.fence` ({id, width, height}: the content
    /// area in DIPs), as a web plugin's page gets them.
    pub init: serde_json::Value,
    pub shown: bool,
    /// Where the fence's last view is kept (`plugins-host\<fence>.view.json`): shown at start
    /// until the plugin's first `pf.show`.
    pub cache: Option<PathBuf>,
    /// The app's number of this start: every [`Output`] of the runtime carries it, so what an
    /// earlier runtime of the fence sent late (before an update) is told apart.
    pub number: u32,
}

pub enum Command {
    Start(Box<Start>),
    /// Shown or hidden: `show` / `hide`, and the timers wait while it is hidden (unless the
    /// plugin runs in the background).
    Shown(FenceId, bool),
    /// An event of [`EVENTS`]: `timechange`, `settings` (the new settings), `resize` ({width,
    /// height}), `click` ({id}), `clipboard`, ...
    Event(FenceId, &'static str, serde_json::Value),
    /// A web plugin page's `pf.post`: main.js gets `message`. The `Arc` goes once it has, so
    /// the app counts the posts still waiting (`Arc::strong_count`).
    Message(FenceId, serde_json::Value, std::sync::Arc<()>),
    Stop(FenceId),
    /// Every plugin gets `stop`, then the thread ends.
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stopped {
    TooLong,
    /// More than [`BUDGET`] of JavaScript time in a second.
    TooBusy,
    TooMuchMemory,
    Error(String),
}

/// What a plugin's runtime sends, with its fence and its [`Start::number`].
#[derive(Debug, PartialEq)]
pub enum Output {
    Show(FenceId, u32, View),
    Stopped(FenceId, u32, Stopped),
    /// `pf.post(data)`, as JSON: for the page of the web plugin whose main.js this is.
    Post(FenceId, u32, String),
    /// An error that escaped a handler of this start (it was logged, the plugin goes on): one
    /// for each of the first [`ERRORS_TOLD`].
    Errored(FenceId, u32),
}

/// What reaches the plugin thread: the app's commands and the answers of the worker threads.
enum Msg {
    Cmd(Command),
    Done(FenceId, u32, Result<Value, String>),
}

/// Call numbers are unique across plugins: an answer for a plugin that was started again
/// finds no call of its number.
static NEXT_CALL: AtomicU32 = AtomicU32::new(1);

/// Starts the plugin thread; `out` gets what the plugins show (on that thread).
pub fn spawn(
    host: Arc<dyn Host>,
    format: Format,
    out: impl Fn(Output) + Send + 'static,
) -> std::io::Result<(Sender<Command>, std::thread::JoinHandle<()>)> {
    let (tx, rx) = std::sync::mpsc::channel();
    let thread = std::thread::Builder::new()
        .name("pecofence-plugins-js".into())
        .spawn(move || run(rx, host, format, out))?;
    Ok((tx, thread))
}

struct Timer {
    id: u32,
    step: u64,
    aligned: bool,
    /// `pf.after`: runs once.
    once: bool,
    next: Instant,
}

impl Timer {
    /// Just after the next boundary of its step on the clock (as the native clock arms its
    /// timer), or a step from now.
    fn arm(&mut self) {
        let wait = if self.aligned {
            let ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            self.step - ms % self.step + 20
        } else {
            self.step
        };
        self.next = Instant::now() + Duration::from_millis(wait);
    }
}

/// The pointer's row and the wheel's steps, until they may go to the plugin.
#[derive(Default)]
struct Paced {
    /// The row the plugin was told the pointer is on, and the one it is on now (the row's id).
    told: Option<Value>,
    on: Option<Value>,
    hover_at: Option<Instant>,
    /// The steps not told yet, by the id of the row or cell.
    wheel: Vec<(Value, i64)>,
    wheel_at: Option<Instant>,
}

impl Paced {
    /// Takes `enter` ({row}), `leave` and `wheel` ({id, steps}) from the app.
    fn take(&mut self, name: &str, data: &Value) {
        match name {
            "enter" => self.on = Some(data.get("row").cloned().unwrap_or_default()),
            "leave" => self.on = None,
            _ => {
                let id = data.get("id").cloned().unwrap_or_default();
                let steps = data.get("steps").and_then(Value::as_i64).unwrap_or(0);
                match self.wheel.iter_mut().find(|(i, _)| *i == id) {
                    Some((_, n)) => *n += steps,
                    None => self.wheel.push((id, steps)),
                }
            }
        }
    }

    /// The events that may go out now: `leave` of the row told before and `enter` of the new
    /// one, and the wheel's steps by id.
    fn due(&mut self, now: Instant) -> Vec<(&'static str, Value)> {
        let mut out = Vec::new();
        let free = |at: Option<Instant>, gap| at.is_none_or(|at| now >= at + gap);
        if self.on != self.told && free(self.hover_at, HOVER_GAP) {
            self.hover_at = Some(now);
            if let Some(row) = self.told.take() {
                out.push(("leave", json!({ "row": row })));
            }
            if let Some(row) = self.on.clone() {
                out.push(("enter", json!({ "row": row })));
            }
            self.told = self.on.clone();
        }
        if !self.wheel.is_empty() && free(self.wheel_at, WHEEL_GAP) {
            self.wheel_at = Some(now);
            for (id, steps) in std::mem::take(&mut self.wheel) {
                if steps != 0 {
                    out.push(("wheel", json!({ "id": id, "steps": steps })));
                }
            }
        }
        out
    }

    fn next_due(&self) -> Option<Instant> {
        let hover = (self.on != self.told).then(|| self.hover_at.map(|at| at + HOVER_GAP));
        let wheel = (!self.wheel.is_empty()).then(|| self.wheel_at.map(|at| at + WHEEL_GAP));
        // Waiting with no time yet: due at once.
        [hover, wheel]
            .into_iter()
            .flatten()
            .map(|at| at.unwrap_or_else(Instant::now))
            .min()
    }
}

/// What the plugin's calls into `pf` leave for the host.
#[derive(Default)]
struct Shared {
    timers: Vec<Timer>,
    view: Option<View>,
    /// The JSON of the last view shown, for the cache file.
    json: Option<String>,
    /// The calls on worker threads, by number: their names.
    calls: HashMap<u32, String>,
    /// The view fields of the wrong type logged already (each once).
    dropped: HashSet<String>,
    /// What `pf.post` sent in this run, as JSON.
    posts: Vec<String>,
    /// The number of the last drop on the fence: `pf.drop.*` reads its files (the app keeps them).
    last_drop: Option<u64>,
}

struct Plugin {
    fence: FenceId,
    number: u32,
    id: String,
    /// The run going on or made last (`call`'s `what`): the log of a stop names it.
    last_run: String,
    // The context is dropped before its runtime.
    ctx: Context,
    rt: Runtime,
    deadline: Rc<Cell<Option<Instant>>>,
    shared: Rc<RefCell<Shared>>,
    shown: bool,
    sent: Option<Instant>,
    host: Arc<dyn Host>,
    /// `permissions.background`: timers and the clipboard go on while the fence is hidden.
    background: bool,
    reads_clipboard: bool,
    /// A click, dblclick, command or submit handler runs (the user did something): `pf.open`,
    /// `pf.launch.open` and `pf.launch.pick` work only then.
    click: Rc<Cell<bool>>,
    /// A drop handler runs, of the drop with this number: `pf.launch.add` works only then, and
    /// goes to the app with the number (the app adds that drop's files, not a later one's).
    dropping: Rc<Cell<Option<u64>>>,
    /// The JavaScript time used in the second that began then.
    second: (Instant, Duration),
    cache: Option<PathBuf>,
    /// When the cache file was last written, and whether a newer view is not in it yet.
    cached: Option<Instant>,
    unsaved: bool,
    /// The Promises rejected in this run with nobody to handle them yet (an async handler that
    /// threw), with their errors: one that an `await` or a `.catch` takes later in the run goes
    /// out again. Emptied at the end of every run, so none outlives the runtime. Then the
    /// messages of the errors logged so far.
    rejected: Rc<RefCell<Vec<Rejected>>>,
    logged: HashSet<String>,
    /// The errors that escaped a handler, and how many went out as [`Output::Errored`].
    errors: (u32, u32),
    paced: Paced,
}

type Rejected = (Persistent<rquickjs::Value<'static>>, String);

fn stopped(text: String) -> Stopped {
    if text.contains("interrupted") {
        Stopped::TooLong
    } else if text.contains("out of memory") || text.contains("Allocation") {
        Stopped::TooMuchMemory
    } else {
        Stopped::Error(text.lines().next().unwrap_or_default().to_string())
    }
}

/// The view kept in `path`, if any.
fn cached_view(path: &std::path::Path) -> Option<View> {
    view::parse(&std::fs::read_to_string(path).ok()?).ok()
}

impl Plugin {
    fn start(
        s: Start,
        host: Arc<dyn Host>,
        tx: Sender<Msg>,
        format: Format,
    ) -> Result<Self, Stopped> {
        let fail = |e: rquickjs::Error| Stopped::Error(e.to_string());
        let rt = Runtime::new().map_err(fail)?;
        rt.set_memory_limit(MEMORY_LIMIT);
        let deadline: Rc<Cell<Option<Instant>>> = Rc::default();
        let late = deadline.clone();
        rt.set_interrupt_handler(Some(Box::new(move || {
            late.get().is_some_and(|d| Instant::now() > d)
        })));
        let rejected: Rc<RefCell<Vec<Rejected>>> = Rc::default();
        let to = rejected.clone();
        rt.set_host_promise_rejection_tracker(Some(Box::new(
            move |ctx, promise, reason, handled| {
                let promise = Persistent::save(&ctx, promise);
                if handled {
                    return to.borrow_mut().retain(|(p, _)| *p != promise);
                }
                // The text first: an error's `message` or `stack` getter is the plugin's code and
                // may reject another Promise, which comes back here.
                let text = match reason.as_exception() {
                    Some(e) => format!(
                        "Error: {}\n{}",
                        e.message().unwrap_or_default(),
                        e.stack().unwrap_or_default()
                    ),
                    None => format!("Error: {reason:?}"),
                };
                to.borrow_mut().push((promise, text));
            },
        )));
        let ctx = Context::full(&rt).map_err(fail)?;
        let p = &s.permissions;
        // The view of a plugin that may read the clipboard can hold copied texts: it is never
        // kept on disk, and one an earlier version kept goes.
        let cache = match &s.cache {
            Some(path) if p.clipboard.reads() => {
                let gone = std::fs::remove_file(path).is_ok();
                tracing::info!(fence = %s.fence, gone, "plugin view not cached: it may read the clipboard");
                None
            }
            cache => cache.clone(),
        };
        let mut plugin = Plugin {
            fence: s.fence,
            number: s.number,
            id: s.id.clone(),
            last_run: MAIN.to_string(),
            ctx,
            rt,
            deadline,
            shared: Rc::default(),
            shown: s.shown,
            sent: None,
            host: host.clone(),
            background: p.background,
            reads_clipboard: p.clipboard.reads(),
            click: Rc::default(),
            dropping: Rc::default(),
            second: (Instant::now(), Duration::ZERO),
            cache,
            cached: None,
            unsaved: false,
            rejected,
            logged: HashSet::new(),
            errors: (0, 0),
            paced: Paced::default(),
        };
        let (shared, click, fence) = (plugin.shared.clone(), plugin.click.clone(), s.fence);
        let dropping = plugin.dropping.clone();
        let agent = format!("PecoFence-plugin/{} {}", s.id, s.version);
        let (id, permissions) = (s.id, Arc::new(s.permissions));
        let noting = host.wants_notes();
        let note = {
            let host = host.clone();
            move |name: &str, args: Value| {
                let note = Note::Call {
                    name: name.into(),
                    args,
                    refused: None,
                };
                host.note(fence, note);
            }
        };
        plugin.call(MAIN, move |ctx| {
            let n = Object::new(ctx.clone())?;
            n.set("lang", s.init["lang"].as_str().unwrap_or_default())?;
            n.set("theme", s.init["theme"].as_str().unwrap_or_default())?;
            n.set("settings", s.init["settings"].to_string())?;
            n.set("fence", s.init["fence"].to_string())?;
            let (to, note_, plugin) = (shared.clone(), note.clone(), id.clone());
            let show = move |json: String| {
                if noting {
                    note_("show", json!([serde_json::from_str::<Value>(&json).unwrap_or_default()]));
                }
                match view::read(&json) {
                    Ok((view, dropped)) => {
                        let mut s = to.borrow_mut();
                        for field in dropped {
                            if s.dropped.insert(field.clone()) {
                                tracing::info!(%fence, plugin, field, "plugin view: a field of the wrong type is left out, the rest shows");
                            }
                        }
                        (s.view, s.json) = (Some(view), Some(json));
                    }
                    Err(e) => tracing::debug!(error = %e, "plugin view not read"),
                }
            };
            n.set("show", Function::new(ctx.clone(), show)?)?;
            let to = shared.clone();
            let post = move |ctx: Ctx<'_>, json: String| {
                let mut s = to.borrow_mut();
                let waiting = s.posts.iter().map(String::len).sum::<usize>();
                if json.len() > MAX_POST || s.posts.len() >= MAX_POSTS || waiting + json.len() > MAX_POSTED {
                    tracing::info!(%fence, bytes = json.len(), waiting, posts = s.posts.len(), "plugin post refused: too big or too much at once");
                    let why = "pf.post: a message is at most 1 MB, and at most 100 messages or 4 MB go out at once";
                    return Err(Exception::throw_range(&ctx, why));
                }
                s.posts.push(json);
                Ok(())
            };
            n.set("post", Function::new(ctx.clone(), post)?)?;
            let (to, note_) = (shared.clone(), note.clone());
            let timer = move |ctx: Ctx<'_>, id: u32, seconds: f64, align: String, once: bool| {
                note_(if once { "after" } else { "every" }, json!([seconds, align]));
                let mut s = to.borrow_mut();
                if s.timers.len() >= MAX_TIMERS {
                    tracing::info!(%fence, max = MAX_TIMERS, "plugin timer refused: it has the most timers");
                    return Err(Exception::throw_range(&ctx, "a plugin has at most 20 timers"));
                }
                let unit = match align.as_str() {
                    "second" => 1000,
                    "minute" => 60_000,
                    _ => 0,
                };
                // At least a second; aligned, a whole number of its unit.
                let step = (seconds.max(1.0) * 1000.0) as u64;
                let step = if unit > 0 {
                    step.div_ceil(unit) * unit
                } else {
                    step
                };
                let mut timer = Timer {
                    id,
                    step,
                    aligned: unit > 0,
                    once,
                    next: Instant::now(),
                };
                timer.arm();
                s.timers.push(timer);
                Ok(())
            };
            n.set("timer", Function::new(ctx.clone(), timer)?)?;
            let (to, note_) = (shared.clone(), note.clone());
            let cancel = move |id: u32| {
                note_("cancel", json!([id]));
                to.borrow_mut().timers.retain(|t| t.id != id)
            };
            n.set("cancel", Function::new(ctx.clone(), cancel)?)?;
            let format = move |ms: f64, kind: String| {
                let kind = if kind.is_empty() { "short".into() } else { kind };
                format(ms as i64, &kind)
            };
            n.set("format", Function::new(ctx.clone(), format)?)?;
            // Everything that reaches the app: checked against the manifest, then done by the
            // host on a worker thread; the answer comes back as `Msg::Done`.
            let to = shared.clone();
            let call = move |ctx: Ctx<'_>, name: String, args: String| {
                let mut args: Value = serde_json::from_str(&args).unwrap_or(json!([]));
                let mut refused = refusal(&permissions, &name, &args);
                let by_user = ["open", "launch.open", "launch.pick", "media.control"].contains(&name.as_str());
                if refused.is_none() && by_user && !click.get() {
                    refused = Some(format!("pf.{name} works only in a click, dblclick, command or submit handler"));
                }
                if name == "launch.add" {
                    match dropping.get() {
                        Some(drop) => args = json!([args.get(0).cloned().unwrap_or_default(), drop]),
                        None if refused.is_none() => {
                            refused = Some("pf.launch.add works only in a drop handler".into());
                        }
                        None => {}
                    }
                }
                // [file number, name or null, the last drop's number]: that drop's file.
                if name.starts_with("drop.") {
                    match to.borrow().last_drop {
                        Some(drop) => args = json!([args.get(0), args.get(1), drop]),
                        None if refused.is_none() => {
                            refused = Some(format!("pf.{name} needs a drop on the fence first"));
                        }
                        None => {}
                    }
                }
                let mut s = to.borrow_mut();
                let requests = s.calls.values().filter(|c| c.starts_with("http.")).count();
                let full = name.starts_with("http.") && requests >= MAX_REQUESTS;
                if refused.is_none() && (full || s.calls.len() >= MAX_CALLS) {
                    refused = Some(format!(
                        "{name}: too many calls at once (at most {MAX_REQUESTS} requests and {MAX_CALLS} calls)"
                    ));
                }
                let n = Note::Call { name: name.clone(), args: args.clone(), refused: refused.clone() };
                host.note(fence, n);
                if let Some(why) = refused {
                    tracing::info!(%fence, plugin = id, name, why, "plugin call refused");
                    return Err(Exception::throw_message(&ctx, &why));
                }
                let number = NEXT_CALL.fetch_add(1, Ordering::Relaxed);
                tracing::debug!(%fence, plugin = id, name, number, "plugin call");
                s.calls.insert(number, name.clone());
                let (host, permissions, tx, agent) =
                    (host.clone(), permissions.clone(), tx.clone(), agent.clone());
                let worker = std::thread::Builder::new()
                    .name("pecofence-plugin-call".into())
                    .spawn(move || {
                        let result = perform(&*host, fence, &permissions, &agent, &name, args);
                        let _ = tx.send(Msg::Done(fence, number, result));
                    });
                if worker.is_err() {
                    s.calls.remove(&number);
                    return Err(Exception::throw_message(&ctx, "no thread for the call"));
                }
                Ok(number)
            };
            n.set("call", Function::new(ctx.clone(), call)?)?;
            ctx.globals().set("__pf", n)?;
            ctx.eval::<(), _>(include_str!("prelude.js"))?;
            ctx.eval::<(), _>(s.code)?;
            Ok(())
        })?;
        plugin.emit("start", None)?;
        if !plugin.shown {
            plugin.emit("hide", None)?;
        }
        Ok(plugin)
    }

    /// Runs `f` in the plugin, then the promise jobs it queued, within [`CALL_LIMIT`] and what
    /// is left of [`BUDGET`] in this second. A limit stops the plugin, and so does an error in
    /// main.js itself ([`MAIN`]); an error that escapes a handler, a timer or a Promise is
    /// logged ([`Self::escaped`]) and the plugin goes on.
    fn call(
        &mut self,
        what: &str,
        f: impl for<'js> FnOnce(&Ctx<'js>) -> rquickjs::Result<()>,
    ) -> Result<(), Stopped> {
        let begun = Instant::now();
        self.last_run = what.to_string();
        if begun >= self.second.0 + Duration::from_secs(1) {
            self.second = (begun, Duration::ZERO);
        }
        let left = BUDGET.saturating_sub(self.second.1);
        self.deadline.set(Some(begun + CALL_LIMIT.min(left)));
        let mut result = self
            .ctx
            .with(|ctx| f(&ctx).map_err(|e| CaughtError::from_error(&ctx, e).to_string()));
        while result.is_ok() {
            match self.rt.execute_pending_job() {
                Ok(true) => {}
                Ok(false) => break,
                Err(job) => {
                    result = Err(job.0.with(|ctx| {
                        CaughtError::from_error(&ctx, rquickjs::Error::Exception).to_string()
                    }))
                }
            }
        }
        self.deadline.set(None);
        let rejected = std::mem::take(&mut *self.rejected.borrow_mut());
        for (_, text) in rejected {
            // Out of memory or time inside an async function: a limit all the same.
            if result.is_ok() && !matches!(stopped(text.clone()), Stopped::Error(_)) {
                result = Err(text);
            } else {
                self.escaped(what, &text);
            }
        }
        let took = begun.elapsed();
        self.second.1 += took;
        let noting = self.host.wants_notes();
        let memory = if noting || result.is_err() {
            self.rt.memory_usage().memory_used_size as usize
        } else {
            0
        };
        if noting {
            let (what, ms) = (what.into(), took.as_secs_f64() * 1000.0);
            self.host.note(self.fence, Note::Ran { what, ms, memory });
        }
        let Err(text) = result else {
            return Ok(());
        };
        match stopped(text.clone()) {
            // Out of memory, QuickJS may have no room left for its error and throws null.
            Stopped::Error(_) if memory > MEMORY_LIMIT / 10 * 9 => Err(Stopped::TooMuchMemory),
            Stopped::TooLong if left < CALL_LIMIT => Err(Stopped::TooBusy),
            Stopped::Error(_) if what != MAIN => {
                self.escaped(what, &text);
                Ok(())
            }
            why => Err(why),
        }
    }

    /// An error escaped run `what`: the log gets the plugin, the run, the message and the first
    /// line of its stack (each message once in full), the checker a note; the plugin goes on.
    fn escaped(&mut self, what: &str, text: &str) {
        let mut lines = text.lines();
        let message = lines.next().unwrap_or_default().to_string();
        let at = lines.map(str::trim).find(|l| !l.is_empty());
        let at = at.unwrap_or_default().to_string();
        let first = self.logged.len() < LOGGED_ERRORS && self.logged.insert(message.clone());
        self.errors.0 += 1;
        let (fence, plugin) = (self.fence, &self.id);
        if first {
            tracing::warn!(%fence, plugin, run = what, error = message, line = at, "plugin error: it escaped a handler, the plugin goes on");
        } else {
            tracing::debug!(%fence, plugin, run = what, error = message, "plugin error again");
        }
        let what = what.to_string();
        self.host.note(fence, Note::Error { what, message, at });
    }

    fn emit(&mut self, name: &str, data: Option<String>) -> Result<(), Stopped> {
        let name = name.to_string();
        self.call(&name.clone(), move |ctx| {
            let emit: Function = ctx.globals().get("__pf_emit")?;
            emit.call((name, data))
        })
    }

    /// A worker's answer to call `number`: its Promise resolves (Ok) or rejects (Err).
    fn done(&mut self, number: u32, result: Result<Value, String>) -> Result<(), Stopped> {
        let Some(name) = self.shared.borrow_mut().calls.remove(&number) else {
            return Ok(());
        };
        if let Err(why) = &result {
            tracing::info!(fence = %self.fence, name, why, "plugin call failed");
        }
        let (ok, text) = match result {
            Ok(value) => (true, value.to_string()),
            Err(why) => (false, why),
        };
        self.call(&format!("answer to {name}"), move |ctx| {
            let done: Function = ctx.globals().get("__pf_done")?;
            done.call((number, ok, text))
        })
    }

    fn set_shown(&mut self, shown: bool) -> Result<(), Stopped> {
        if std::mem::replace(&mut self.shown, shown) == shown {
            return Ok(());
        }
        if shown && !self.background {
            // Back after a while: every timer starts from now (aligned ones at the next boundary).
            self.shared
                .borrow_mut()
                .timers
                .iter_mut()
                .for_each(Timer::arm);
        }
        tracing::debug!(fence = %self.fence, shown, background = self.background, "plugin shown or hidden, its timers follow");
        self.emit(if shown { "show" } else { "hide" }, None)
    }

    fn runs(&self) -> bool {
        self.shown || self.background
    }

    /// Runs the timers that are due (none while the fence is hidden, unless in the background).
    fn tick(&mut self, now: Instant) -> Result<(), Stopped> {
        if !self.runs() {
            return Ok(());
        }
        let due: Vec<(u32, bool)> = {
            let mut s = self.shared.borrow_mut();
            let due: Vec<(u32, bool)> = (s.timers.iter_mut().filter(|t| t.next <= now))
                .map(|t| {
                    t.arm();
                    (t.id, t.once)
                })
                .collect();
            s.timers
                .retain(|t| !(t.once && due.contains(&(t.id, true))));
            due
        };
        for (id, once) in due {
            self.call("timer", move |ctx| {
                let tick: Function = ctx.globals().get("__pf_tick")?;
                tick.call((id, once))
            })?;
        }
        Ok(())
    }

    /// The last view shown, when the one before it went out long enough ago.
    fn view_to_send(&mut self, now: Instant) -> Option<View> {
        if self.sent.is_some_and(|at| now < at + SHOW_GAP) {
            return None;
        }
        let view = self.shared.borrow_mut().view.take()?;
        self.sent = Some(now);
        self.unsaved = true;
        if self.cached.is_none_or(|at| now >= at + CACHE_GAP) {
            self.save_view(false);
        }
        Some(view)
    }

    /// Writes the last view into the cache file (on a thread of its own, unless `now`: the
    /// plugin stops).
    fn save_view(&mut self, now: bool) {
        let json = self.shared.borrow().json.clone();
        let (Some(path), Some(json), true) = (self.cache.clone(), json, self.unsaved) else {
            return;
        };
        (self.cached, self.unsaved) = (Some(Instant::now()), false);
        let write = move || {
            let _ = path.parent().map(std::fs::create_dir_all);
            if let Err(e) = std::fs::write(&path, json) {
                tracing::debug!(error = %e, "plugin view not cached");
            }
        };
        if now {
            write();
        } else {
            let _ = std::thread::Builder::new()
                .name("pecofence-plugin-cache".into())
                .spawn(write);
        }
    }

    fn stop(mut self) {
        let _ = self.emit("stop", None);
        self.save_view(true);
    }

    fn next_due(&self) -> Option<Instant> {
        let s = self.shared.borrow();
        let timers = s.timers.iter().filter(|_| self.runs()).map(|t| t.next);
        let view = s.view.as_ref().and(self.sent).map(|at| at + SHOW_GAP);
        timers.chain(view).chain(self.paced.next_due()).min()
    }

    /// The pointer's row and the wheel's steps that may go to the plugin now.
    fn pace(&mut self, now: Instant) -> Result<(), Stopped> {
        for (name, data) in self.paced.due(now) {
            tracing::debug!(fence = %self.fence, name, %data, "plugin told of the pointer or the wheel");
            self.emit(name, Some(data.to_string()))?;
        }
        Ok(())
    }
}

/// The plugin thread: commands from the app, the plugins' timers and answers, their views out.
pub fn run(rx: Receiver<Command>, host: Arc<dyn Host>, format: Format, out: impl Fn(Output)) {
    // The commands and the workers' answers come in through one queue.
    let (tx, inbox) = std::sync::mpsc::channel::<Msg>();
    let forward = tx.clone();
    let forwarder = std::thread::Builder::new()
        .name("pecofence-plugins-in".into())
        .stack_size(64 * 1024)
        .spawn(move || {
            while let Ok(cmd) = rx.recv() {
                if forward.send(Msg::Cmd(cmd)).is_err() {
                    return;
                }
            }
            let _ = forward.send(Msg::Cmd(Command::Quit));
        });
    if let Err(e) = forwarder {
        return tracing::warn!(error = %e, "plugin thread: no thread for its commands");
    }
    let mut plugins: Vec<Plugin> = Vec::new();
    loop {
        let due = plugins.iter().filter_map(Plugin::next_due).min();
        let msg = match due {
            Some(at) => inbox.recv_timeout(at.saturating_duration_since(Instant::now())),
            None => inbox.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        let at = |fence: FenceId, plugins: &[Plugin]| plugins.iter().position(|p| p.fence == fence);
        let mut failed: Vec<(FenceId, u32, Stopped)> = Vec::new();
        let (msg, _waiting) = match msg {
            Ok(Msg::Cmd(Command::Message(fence, data, waiting))) => (
                Ok(Msg::Cmd(Command::Event(fence, "message", data))),
                Some(waiting),
            ),
            msg => (msg, None),
        };
        match msg {
            Err(RecvTimeoutError::Timeout) => {}
            Ok(Msg::Cmd(Command::Quit)) | Err(RecvTimeoutError::Disconnected) => {
                tracing::info!(plugins = plugins.len(), "plugin thread: every plugin stops");
                plugins.into_iter().for_each(Plugin::stop);
                return;
            }
            Ok(Msg::Done(fence, number, result)) => {
                if let Some(i) = at(fence, &plugins)
                    && let Err(why) = plugins[i].done(number, result)
                {
                    failed.push((fence, plugins[i].number, why));
                }
            }
            Ok(Msg::Cmd(Command::Start(s))) => {
                let (fence, number, id) = (s.fence, s.number, s.id.clone());
                if let Some(i) = at(fence, &plugins) {
                    plugins.remove(i).stop();
                }
                let begun = Instant::now();
                match Plugin::start(*s, host.clone(), tx.clone(), format) {
                    Ok(p) => {
                        let used = p.rt.memory_usage().memory_used_size;
                        let cached = p.shared.borrow().view.is_none();
                        let cached = cached.then(|| p.cache.as_deref().and_then(cached_view));
                        tracing::info!(%fence, ms = begun.elapsed().as_millis(), used, cached = matches!(cached, Some(Some(_))), "plugin started");
                        if let Some(Some(view)) = cached {
                            out(Output::Show(fence, number, view));
                        }
                        plugins.push(p);
                    }
                    Err(why) => {
                        tracing::warn!(%fence, plugin = id, run = MAIN, ?why, "plugin did not start");
                        failed.push((fence, number, why))
                    }
                }
            }
            Ok(Msg::Cmd(Command::Stop(fence))) => {
                if let Some(i) = at(fence, &plugins) {
                    plugins.remove(i).stop();
                    tracing::info!(%fence, "plugin stopped");
                }
            }
            Ok(Msg::Cmd(Command::Shown(fence, shown))) => {
                if let Some(i) = at(fence, &plugins)
                    && let Err(why) = plugins[i].set_shown(shown)
                {
                    failed.push((fence, plugins[i].number, why));
                }
            }
            Ok(Msg::Cmd(Command::Message(..))) => {}
            Ok(Msg::Cmd(Command::Event(fence, name, data)))
                if matches!(name, "enter" | "leave" | "wheel") =>
            {
                // Goes out below, when its pace allows.
                if let Some(p) = plugins.iter_mut().find(|p| p.fence == fence) {
                    p.paced.take(name, &data);
                }
            }
            Ok(Msg::Cmd(Command::Event(fence, name, data))) => {
                if let Some(p) = plugins.iter_mut().find(|p| p.fence == fence) {
                    if name == "timechange" {
                        let mut s = p.shared.borrow_mut();
                        s.timers
                            .iter_mut()
                            .filter(|t| t.aligned)
                            .for_each(Timer::arm);
                    }
                    // The clipboard only for a plugin that may read it and runs now; the
                    // media session (the app tells only plugins that may see it) the same.
                    let told = match name {
                        "clipboard" => p.reads_clipboard && p.runs(),
                        "media" | "apps" => p.runs(),
                        _ => true,
                    };
                    if !told {
                        tracing::debug!(%fence, name, reads = p.reads_clipboard, runs = p.runs(), "plugin not told of the clipboard or the media session");
                    }
                    let drop = data.get("drop").and_then(Value::as_u64);
                    let files = data["files"].as_array().is_some_and(|f| !f.is_empty());
                    let data = (!data.is_null()).then(|| data.to_string());
                    p.click
                        .set(matches!(name, "click" | "dblclick" | "command" | "submit"));
                    p.dropping.set(drop.filter(|_| name == "drop"));
                    // `pf.drop.*` reads the last drop with files (a text or a link has none).
                    if name == "drop" && files {
                        p.shared.borrow_mut().last_drop = drop;
                    }
                    let result = if told { p.emit(name, data) } else { Ok(()) };
                    p.click.set(false);
                    p.dropping.set(None);
                    if let Err(why) = result {
                        failed.push((fence, p.number, why));
                    }
                }
            }
        }
        let now = Instant::now();
        for p in &mut plugins {
            if let Err(why) = p.tick(now).and_then(|_| p.pace(now)) {
                failed.push((p.fence, p.number, why));
            }
        }
        for (fence, number, why) in failed {
            // The run that broke a limit or failed (a plugin that did not start was in main.js).
            let (plugin, run) = at(fence, &plugins).map_or((String::new(), MAIN.into()), |i| {
                (plugins[i].id.clone(), plugins[i].last_run.clone())
            });
            tracing::warn!(%fence, plugin, number, run, ?why, "plugin stopped by PecoFence");
            if let Some(i) = at(fence, &plugins) {
                plugins.remove(i).save_view(true);
            }
            out(Output::Stopped(fence, number, why));
        }
        for p in &mut plugins {
            if let Some(view) = p.view_to_send(now) {
                out(Output::Show(p.fence, p.number, view));
            }
            let posts = std::mem::take(&mut p.shared.borrow_mut().posts);
            for data in posts {
                out(Output::Post(p.fence, p.number, data));
            }
            while p.errors.1 < p.errors.0.min(ERRORS_TOLD) {
                p.errors.1 += 1;
                out(Output::Errored(p.fence, p.number));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pecofence_core::plugins::Clipboard;
    use std::sync::Mutex;
    use std::sync::mpsc::channel;

    /// Answers every call with its name and arguments after `delay`, and keeps the notes.
    #[derive(Default)]
    struct Fake {
        delay: Duration,
        notes: Mutex<Vec<Note>>,
    }

    impl Host for Fake {
        fn call(&self, _: FenceId, name: &str, args: Value) -> Result<Value, String> {
            std::thread::sleep(self.delay);
            if name == "http.get" {
                let url = args[0].as_str().unwrap_or_default();
                return Ok(json!({ "status": 200, "headers": {}, "body": url }));
            }
            Ok(json!([name, args]))
        }

        fn note(&self, _: FenceId, note: Note) {
            self.notes.lock().unwrap().push(note);
        }
    }

    fn format(ms: i64, kind: &str) -> String {
        format!("{kind} {ms}")
    }

    /// Runs `code` in a plugin thread for `ms` (after `then` was sent) and returns what came out.
    fn outputs(code: &str, shown: bool, ms: u64, then: Vec<Command>) -> Vec<Output> {
        run_with(
            code,
            Permissions::default(),
            Arc::new(Fake::default()),
            shown,
            ms,
            then,
        )
    }

    fn run_with(
        code: &str,
        permissions: Permissions,
        host: Arc<dyn Host>,
        shown: bool,
        ms: u64,
        then: Vec<Command>,
    ) -> Vec<Output> {
        let fence = FenceId::new_v4();
        let (tx, rx) = channel();
        let (out_tx, out_rx) = channel();
        let thread =
            std::thread::spawn(move || run(rx, host, format, move |o| out_tx.send(o).unwrap()));
        let start = Start {
            fence,
            id: "test".into(),
            version: "1.0.0".into(),
            permissions,
            code: code.into(),
            init: json!({
                "lang": "zh-CN", "theme": "dark", "settings": { "seconds": true },
                "fence": { "id": fence.to_string(), "width": 240.0, "height": 104.0 },
            }),
            shown,
            cache: None,
            number: 7,
        };
        tx.send(Command::Start(Box::new(start))).unwrap();
        for cmd in then {
            std::thread::sleep(Duration::from_millis(ms));
            let cmd = match cmd {
                Command::Shown(_, s) => Command::Shown(fence, s),
                Command::Event(_, n, d) => Command::Event(fence, n, d),
                Command::Message(_, d, w) => Command::Message(fence, d, w),
                c => c,
            };
            tx.send(cmd).unwrap();
        }
        std::thread::sleep(Duration::from_millis(ms + 150));
        tx.send(Command::Quit).unwrap();
        thread.join().unwrap();
        out_rx.try_iter().collect()
    }

    fn texts(out: &[Output]) -> Vec<String> {
        let rows = out.iter().filter_map(|o| match o {
            Output::Show(_, _, v) => Some(
                v.rows
                    .iter()
                    .map(|r| r.text.clone())
                    .collect::<Vec<_>>()
                    .join("|"),
            ),
            _ => None,
        });
        rows.collect()
    }

    fn stop_of(out: &[Output]) -> Option<Stopped> {
        out.iter().find_map(|o| match o {
            Output::Stopped(_, _, why) => Some(why.clone()),
            _ => None,
        })
    }

    fn event(name: &'static str) -> Command {
        Command::Event(FenceId::new_v4(), name, json!({ "id": "a" }))
    }

    #[test]
    fn every_output_carries_the_number_of_its_start() {
        let code = "pf.show({ rows: [{ text: 'a' }] }); pf.post(1); pf.on('click', () => { for (;;) {} });";
        let out = outputs(code, true, 30, vec![event("click")]);
        let numbers: Vec<u32> = (out.iter())
            .map(|o| match o {
                Output::Show(_, n, _) | Output::Post(_, n, _) | Output::Stopped(_, n, _) => *n,
                Output::Errored(_, n) => *n,
            })
            .collect();
        assert_eq!(numbers, [7, 7, 7], "{out:?}");
    }

    #[test]
    fn an_error_in_a_handler_or_timer_is_logged_and_the_plugin_goes_on() {
        let code = r#"
            let n = 0;
            pf.on('click', () => { throw new Error('click boom'); });
            pf.on('start', async () => { await 0; throw new TypeError('async boom'); });
            pf.every(1, () => { n++; if (n === 1) throw new Error('tick boom'); pf.show({ rows: [{ text: 'tick ' + n }] }); });
        "#;
        let fake = Arc::new(Fake::default());
        let out = run_with(
            code,
            Permissions::default(),
            fake.clone(),
            true,
            1100,
            vec![event("click"), event("click")],
        );
        assert_eq!(stop_of(&out), None, "{out:?}");
        assert!(texts(&out).contains(&"tick 2".to_string()), "{out:?}");
        // The app hears of the first three of the four (three in the first minute after an
        // update take it back).
        let told = out.iter().filter(|o| matches!(o, Output::Errored(_, 7)));
        assert_eq!(told.count(), 3, "{out:?}");
        let notes = fake.notes.lock().unwrap();
        let errors: Vec<(String, String)> = (notes.iter())
            .filter_map(|n| match n {
                Note::Error { what, message, at } => {
                    assert!(!at.is_empty(), "{message} has no stack line");
                    Some((what.clone(), message.clone()))
                }
                _ => None,
            })
            .collect();
        for (what, message) in [
            ("start", "Error: async boom"),
            ("click", "Error: click boom"),
            ("timer", "Error: tick boom"),
        ] {
            assert!(
                errors.contains(&(what.into(), message.into())),
                "{what} {message} not in {errors:?}"
            );
        }
    }

    #[test]
    fn a_rejection_handled_later_in_the_run_is_no_error() {
        let code = r#"
            const f = async () => { throw new Error('caught later'); };
            pf.on('click', async () => {
                try { await f(); } catch (e) {}
                Promise.reject(new Error('caught too')).catch(() => {});
                const p = Promise.reject(new Error('caught after a turn'));
                await 0;
                p.catch(() => {});
                Promise.reject(new Error('nobody catches this'));
                pf.show({ rows: [{ text: 'done' }] });
            });
        "#;
        let fake = Arc::new(Fake::default());
        let out = run_with(
            code,
            Permissions::default(),
            fake.clone(),
            true,
            30,
            vec![event("click")],
        );
        assert_eq!((stop_of(&out), texts(&out)), (None, vec!["done".into()]));
        let notes = fake.notes.lock().unwrap();
        let errors: Vec<&str> = (notes.iter())
            .filter_map(|n| match n {
                Note::Error { message, .. } => Some(message.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(errors, ["Error: nobody catches this"]);
    }

    #[test]
    fn an_error_whose_message_rejects_a_promise_is_logged() {
        // Its getter runs while the rejection is told and rejects a Promise of its own, which is
        // told (and logged) too.
        let code = r#"
            class Sly extends Error {
                get message() { Promise.reject(new Error('inner')); return 'sly'; }
            }
            let n = 0;
            pf.on('click', async () => { pf.show({ rows: [{ text: 'click ' + ++n }] }); throw new Sly(); });
        "#;
        let fake = Arc::new(Fake::default());
        let clicks = vec![event("click"), event("click")];
        let out = run_with(code, Permissions::default(), fake.clone(), true, 30, clicks);
        assert_eq!(stop_of(&out), None, "{out:?}");
        assert_eq!(texts(&out).last().map(String::as_str), Some("click 2"));
        let notes = fake.notes.lock().unwrap();
        let errors: Vec<&str> = (notes.iter())
            .filter_map(|n| match n {
                Note::Error { message, .. } => Some(message.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            errors,
            ["Error: inner", "Error: sly"].repeat(2),
            "{errors:?}"
        );
    }

    #[test]
    fn an_error_in_main_js_itself_stops_the_plugin() {
        let out = outputs("pf.show({ rows: [] }); null.x;", true, 30, vec![]);
        assert!(
            matches!(stop_of(&out), Some(Stopped::Error(e)) if e.contains("null")),
            "{out:?}"
        );
        // Out of memory in an async function is a limit all the same. (Big pieces: the one that
        // fails leaves room for the error; with no room at all QuickJS rejects with null.)
        let code = "pf.on('start', async () => { await 0; const a = []; for (;;) a.push('x'.repeat(100000) + a.length); });";
        assert_eq!(
            stop_of(&outputs(code, true, 50, vec![])),
            Some(Stopped::TooMuchMemory)
        );
    }

    #[test]
    fn the_pointer_and_the_wheel_reach_the_plugin_paced() {
        let (mut p, t0) = (Paced::default(), Instant::now());
        let ms = |n| t0 + Duration::from_millis(n);
        let (none, at) = (Vec::<(&str, Value)>::new(), |r: &str| json!({ "row": r }));
        p.take("enter", &at("a"));
        assert_eq!(p.due(t0), [("enter", at("a"))]);
        // Onto b, onto c within 100 ms: one change after the gap, to where the pointer is now.
        for row in ["b", "c"] {
            p.take("leave", &Value::Null);
            p.take("enter", &at(row));
        }
        assert_eq!((p.due(ms(50)), p.next_due()), (none.clone(), Some(ms(100))));
        assert_eq!(p.due(ms(100)), [("leave", at("a")), ("enter", at("c"))]);
        // Off c and back before the plugin was told: nothing to tell.
        p.take("leave", &Value::Null);
        p.take("enter", &at("c"));
        assert_eq!((p.due(ms(300)), p.next_due()), (none.clone(), None));
        // The wheel: 20 times a second, the steps added up by id; steps that cancel out go nowhere.
        let wheel = |id: &str, steps: i64| json!({ "id": id, "steps": steps });
        for steps in [1, 1, 1, -1] {
            p.take("wheel", &wheel("v", steps));
        }
        p.take("wheel", &wheel("w", 1));
        p.take("wheel", &wheel("w", -1));
        assert_eq!(p.due(ms(300)), [("wheel", wheel("v", 2))]);
        p.take("wheel", &wheel("v", -3));
        assert_eq!(p.due(ms(320)), none);
        assert_eq!(p.due(ms(350)), [("wheel", wheel("v", -3))]);
    }

    #[test]
    fn enter_leave_and_wheel_go_to_the_handlers() {
        let code = "const got = []; for (const n of ['enter', 'leave', 'wheel', 'submit'])
            pf.on(n, (e) => { got.push(n + ' ' + JSON.stringify(e)); pf.show({ rows: [{ text: got.join(' ') }] }); });";
        let f = FenceId::new_v4();
        let out = outputs(
            code,
            true,
            30,
            vec![
                Command::Event(f, "enter", json!({ "row": "r1" })),
                Command::Event(f, "leave", json!({ "row": "r1" })),
                Command::Event(f, "wheel", json!({ "id": "v", "steps": -2 })),
                Command::Event(f, "submit", json!({ "id": "t" })),
            ],
        );
        // The leave 30 ms after the enter waits for the 100 ms between them.
        assert_eq!(
            texts(&out).last().map(String::as_str),
            Some(
                r#"enter {"row":"r1"} wheel {"id":"v","steps":-2} submit {"id":"t"} leave {"row":"r1"}"#
            ),
            "{out:?}"
        );
    }

    #[test]
    fn the_theme_event_updates_pf_theme() {
        let code = "pf.on('theme', () => pf.show({ rows: [{ text: pf.theme }] }));";
        let theme = Command::Event(FenceId::new_v4(), "theme", json!("light"));
        assert_eq!(texts(&outputs(code, true, 30, vec![theme])), ["light"]);
    }

    #[test]
    fn a_web_plugins_main_js_and_its_page_pass_messages() {
        let code = "pf.on('message', (m) => pf.post({ got: m.n + 1 }));";
        let waiting = Arc::new(());
        let message = Command::Message(FenceId::new_v4(), json!({ "n": 1 }), waiting.clone());
        // Looked at while the plugin still runs (it quits at 210 ms): once main.js had the post,
        // it no longer counts as waiting (only this and the probe's own are left).
        let probe = waiting.clone();
        let probe = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            Arc::strong_count(&probe)
        });
        let out = outputs(code, true, 30, vec![message]);
        assert_eq!(probe.join().unwrap(), 2);
        let posts: Vec<Value> = (out.iter())
            .filter_map(|o| match o {
                Output::Post(_, _, data) => serde_json::from_str(data).ok(),
                _ => None,
            })
            .collect();
        assert_eq!(posts, [serde_json::json!({ "got": 2 })]);
    }

    #[test]
    fn posts_waiting_to_go_out_are_capped() {
        let code = r#"
            const big = 'x'.repeat(900 * 1024), out = [];
            const tryIt = (what, f) => { try { f(); out.push(what + ' ok'); } catch (e) { out.push(what + ' ' + e.name); } };
            tryIt('huge', () => pf.post('x'.repeat(1100 * 1024)));
            for (let i = 0; i < 5; i++) tryIt('big' + i, () => pf.post(big));
            pf.after(1, () => {
                tryIt('later', () => pf.post(big));
                for (let i = 0; i < 101; i++) tryIt('small' + i, () => pf.post(i));
                pf.show({ rows: [{ text: out.join(' ') }] });
            });
        "#;
        let out = outputs(code, true, 1100, vec![]);
        let text = texts(&out).join("");
        // Four of 900 KB fit into 4 MB, the fifth waits for none: refused; after the run they
        // went out and there is room again; the 101st message of one run is refused.
        for want in [
            "huge RangeError",
            "big3 ok",
            "big4 RangeError",
            "later ok",
            "small98 ok",
            "small99 RangeError",
        ] {
            assert!(text.contains(want), "{want} not in {text}");
        }
        let posts = out.iter().filter(|o| matches!(o, Output::Post(..))).count();
        assert_eq!(posts, 4 + 1 + 99);
    }

    #[test]
    fn an_endless_loop_is_stopped_after_50_ms() {
        let begun = Instant::now();
        let out = outputs(
            "pf.on('start', () => { while (true) {} });",
            true,
            50,
            vec![],
        );
        assert_eq!(stop_of(&out), Some(Stopped::TooLong));
        assert!(begun.elapsed() < Duration::from_millis(1000));
        let out = outputs(
            "pf.every(1, () => { for (;;) {} }); pf.show({ rows: [{ text: 'a' }] });",
            true,
            1300,
            vec![],
        );
        assert_eq!(
            (texts(&out), stop_of(&out)),
            (vec!["a".to_string()], Some(Stopped::TooLong))
        );
    }

    #[test]
    fn more_than_200_ms_a_second_stops_the_plugin() {
        // Each click takes 25 ms, half the 50 ms a run may take (a slow machine adds to it): the
        // ninth in the same second goes over the budget, and twelve of them leave no doubt.
        let code =
            "pf.on('click', () => { const end = Date.now() + 25; while (Date.now() < end) {} });";
        let clicks = (0..12).map(|_| event("click")).collect();
        let out = outputs(code, true, 5, clicks);
        assert_eq!(stop_of(&out), Some(Stopped::TooBusy));
        // Spread over seconds (at most three clicks, 75 ms, in one), the same work is fine.
        let out = outputs(code, true, 400, (0..4).map(|_| event("click")).collect());
        assert_eq!(stop_of(&out), None);
    }

    #[test]
    fn a_memory_hog_is_stopped_at_8_mb() {
        // 64 KB a step: 8 MB comes long before the 50 ms a run may take, also on a slow machine.
        let out = outputs(
            "const a = []; while (true) a.push('x'.repeat(65536) + a.length);",
            true,
            50,
            vec![],
        );
        assert_eq!(stop_of(&out), Some(Stopped::TooMuchMemory));
    }

    #[test]
    fn no_eval_and_no_function_constructors() {
        let code = r#"
            const refused = (f) => { try { f(); return 'ran'; } catch (e) { return 'refused'; } };
            pf.show({ rows: [{ text: [
                typeof eval, refused(() => Function('return 1')()),
                refused(() => (function () {}).constructor('return 1')()),
                refused(() => (async function () {}).constructor('return 1')),
                refused(() => (function* () {}).constructor('yield 1')),
                refused(() => (async function* () {}).constructor('yield 1')),
                ['require', 'process', 'fetch', 'std', 'os', 'setTimeout', '__pf'].filter((n) => n in globalThis).join(','),
                typeof (() => 1) === 'function' && (() => 1) instanceof Function,
            ].join(' ') }] });"#;
        let out = outputs(code, true, 50, vec![]);
        let all = "undefined refused refused refused refused refused  true";
        assert_eq!(texts(&out), vec![all.to_string()]);
    }

    #[test]
    fn pf_tells_the_fence_and_shows_merge_to_ten_a_second() {
        let code = r#"
            pf.on('start', () => pf.show({ rows: [{ text: [pf.lang, pf.theme, pf.settings.seconds, pf.fence.width, pf.time.format(1500, 'time'), pf.time.format(1500)].join(' ') }] }));
            pf.on('resize', () => pf.show({ rows: [{ text: 'w' + pf.fence.width }] }));
        "#;
        let size = |w: f32| {
            Command::Event(
                FenceId::new_v4(),
                "resize",
                serde_json::json!({ "width": w, "height": 120.0 }),
            )
        };
        let out = outputs(code, true, 30, vec![size(300.0), size(301.0)]);
        // The first view at once; the two that came within 100 ms of it as one, the last.
        assert_eq!(
            texts(&out),
            ["zh-CN dark true 240 time 1500 short 1500", "w301"]
        );
    }

    #[test]
    fn timers_wait_while_the_fence_is_hidden() {
        let code = r#"
            let n = 0;
            pf.every(1, () => pf.show({ rows: [{ text: 'tick ' + ++n }] }), { align: 'second' });
            pf.on('show', () => pf.show({ rows: [{ text: 'shown ' + n }] }));
        "#;
        let out = outputs(
            code,
            false,
            1300,
            vec![Command::Shown(FenceId::new_v4(), true)],
        );
        let texts = texts(&out);
        assert_eq!(texts[0], "shown 0", "{texts:?}");
        assert!(texts.len() >= 2 && texts[1] == "tick 1", "{texts:?}");
        // In the background they go on.
        let background = Permissions {
            background: true,
            ..Permissions::default()
        };
        let out = run_with(
            code,
            background,
            Arc::new(Fake::default()),
            false,
            1300,
            vec![],
        );
        assert!(
            super::tests::texts(&out).contains(&"tick 1".to_string()),
            "{out:?}"
        );
    }

    #[test]
    fn after_runs_once_cancel_stops_and_at_most_20_timers() {
        let code = r#"
            let n = 0, m = 0, made = 0, refused = '';
            pf.every(2.5, () => pf.show({ rows: [{ text: [n, m, made, refused].join(' ') }] }));
            pf.after(1, () => { n++; });
            const t = pf.every(1, () => { m++; });
            pf.cancel(t);
            try { for (let i = 0; i < 25; i++) { pf.every(60, () => {}); made++; } } catch (e) { refused = e.name; }
        "#;
        let out = outputs(code, true, 2700, vec![]);
        // The view's timer, the after, and 18 of the 25: 20 timers (the cancelled one is gone).
        assert_eq!(texts(&out), ["1 0 18 RangeError"]);
    }

    #[test]
    fn calls_answer_through_promises_and_refusals_throw_naming_the_permission() {
        let code = r#"
            const out = [];
            const show = () => pf.show({ rows: [{ text: out.join(' | ') }] });
            const tryIt = (what, f) => { try { f(); } catch (e) { out.push(what + ' refused: ' + e.message); } };
            pf.on('start', async () => {
                tryIt('http', () => pf.http.get('https://example.com/'));
                tryIt('clipboard', () => pf.clipboard.text());
                tryIt('open', () => pf.open('https://a.com/'));
                tryIt('data', () => pf.data.read('../x'));
                const r = await pf.http.get('https://a.com/x');
                out.push('http ' + r.status + ' ' + r.body);
                out.push(JSON.stringify(await pf.sys.battery()));
                show();
            });
            pf.on('click', () => { pf.open('https://a.com/'); out.push('opened'); show(); });
        "#;
        let p = Permissions {
            network: vec!["a.com".into()],
            ..Permissions::default()
        };
        let fake = Arc::new(Fake {
            delay: Duration::from_millis(20),
            ..Fake::default()
        });
        let out = run_with(code, p, fake.clone(), true, 200, vec![event("click")]);
        let text = texts(&out).join("\n");
        for want in [
            "http refused: http.get needs an https address with example.com in permissions.network",
            "clipboard refused: clipboard.text needs permissions.clipboard read",
            "open refused: pf.open works only in a click, dblclick, command or submit handler",
            "data refused: data.read needs a name",
            "http 200 https://a.com/x",
            r#"["sys.battery",[]]"#,
            "opened",
        ] {
            assert!(text.contains(want), "{want:?} not in {text}");
        }
        let notes = fake.notes.lock().unwrap();
        let refused = notes.iter().filter(|n| {
            matches!(
                n,
                Note::Call {
                    refused: Some(_),
                    ..
                }
            )
        });
        assert_eq!(refused.count(), 4);
        assert!(
            notes
                .iter()
                .any(|n| matches!(n, Note::Ran { what, .. } if what == "answer to http.get"))
        );
    }

    #[test]
    fn media_control_only_in_a_click_and_the_media_event_only_while_it_runs() {
        let code = r#"
            let why = '', seen = 0, done = '';
            const draw = () => pf.show({ rows: [{ text: why + '|' + seen + '|' + done }] });
            try { pf.media.control('playpause'); } catch (e) { why = e.message; }
            pf.on('click', () => { pf.media.control('next').then((a) => { done = JSON.stringify(a); draw(); }); });
            pf.on('media', () => { seen++; draw(); });
            pf.on('show', draw);
            draw();
        "#;
        let p = Permissions {
            media: pecofence_core::plugins::Media::Control,
            ..Permissions::default()
        };
        let run = |shown: bool, then: Vec<Command>| {
            let out = run_with(code, p.clone(), Arc::new(Fake::default()), shown, 600, then);
            texts(&out).last().cloned().unwrap_or_default()
        };
        let text = run(true, vec![event("click"), event("media")]);
        assert_eq!(
            text,
            r#"pf.media.control works only in a click, dblclick, command or submit handler|1|["media.control",["next"]]"#
        );
        // Hidden, it hears nothing of the media session.
        let shown = Command::Shown(FenceId::new_v4(), true);
        let text = run(false, vec![event("media"), shown]);
        assert!(text.ends_with("|0|"), "{text}");
    }

    #[test]
    fn at_most_four_requests_at_once_and_the_clipboard_only_for_readers() {
        let code = r#"
            let made = 0, why = '', seen = 0;
            for (let i = 0; i < 6; i++) { try { pf.http.get('https://a.com/' + i); made++; } catch (e) { why = e.message; } }
            pf.on('clipboard', () => { seen++; });
            pf.every(1, () => pf.show({ rows: [{ text: made + ' ' + seen + ' ' + why }] }));
        "#;
        let p = Permissions {
            network: vec!["a.com".into()],
            ..Permissions::default()
        };
        let fake = Arc::new(Fake {
            delay: Duration::from_millis(300),
            ..Fake::default()
        });
        let out = run_with(code, p.clone(), fake, true, 600, vec![event("clipboard")]);
        let text = texts(&out).join("\n");
        assert!(
            text.starts_with("4 0 http.get: too many calls at once"),
            "{text}"
        );
        let p = Permissions {
            clipboard: Clipboard::Read,
            ..p
        };
        let out = run_with(
            code,
            p,
            Arc::new(Fake::default()),
            true,
            600,
            vec![event("clipboard")],
        );
        assert!(texts(&out).join("\n").contains("4 1 "), "{out:?}");
    }

    #[test]
    fn launch_calls_only_with_the_permission_add_in_a_drop_open_by_the_user() {
        let code = r#"
            const out = [];
            const tryIt = (what, f) => { try { f(); out.push(what + ' ok'); } catch (e) { out.push(what + ' refused: ' + e.message); } };
            const show = () => pf.show({ rows: [{ text: out.join(' | ') }], menu: [
                { id: 'add', title: 'Add a program…\nsecond line' }, { separator: true }, { title: 'no id' },
                ...Array.from({ length: 9 }, (_, i) => ({ id: 'm' + i, title: 'x'.repeat(50) })),
            ] });
            pf.on('start', () => { tryIt('add', () => pf.launch.add([0])); tryIt('open', () => pf.launch.open('a')); tryIt('list', () => pf.launch.list()); show(); });
            pf.on('drop', (d) => { tryIt('drop add', () => pf.launch.add([d.files.length - 1, 0], 99)); show(); });
            pf.on('command', (c) => { tryIt(c.via + ' open', () => pf.launch.open('a')); tryIt('pick', () => pf.launch.pick('program')); show(); });
        "#;
        let p = Permissions {
            launch: true,
            ..Permissions::default()
        };
        let drop = json!({ "files": [{ "name": "a.exe", "size": 3, "folder": false }], "drop": 7 });
        let then = vec![
            Command::Event(FenceId::new_v4(), "drop", drop),
            Command::Event(
                FenceId::new_v4(),
                "command",
                json!({ "id": "x", "via": "key" }),
            ),
        ];
        let fake = Arc::new(Fake::default());
        let out = run_with(code, p, fake.clone(), true, 150, then);
        let text = texts(&out).join("\n");
        // One call for the whole drop, with the number of the drop it was made in (not the
        // plugin's own second argument).
        let added = fake.notes.lock().unwrap().iter().find_map(|n| match n {
            Note::Call {
                name,
                args,
                refused: None,
            } if name == "launch.add" => Some(args.clone()),
            _ => None,
        });
        assert_eq!(added, Some(json!([[0, 0], 7])));
        for want in [
            "add refused: pf.launch.add works only in a drop handler",
            "open refused: pf.launch.open works only in a click, dblclick, command or submit handler",
            "list ok",
            "drop add ok",
            "key open ok",
            "pick ok",
        ] {
            assert!(text.contains(want), "{want:?} not in {text}");
        }
        // The menu: entries with an id and a title or separators, at most 8, one line of 40.
        let Some(Output::Show(_, _, view)) = out.iter().rfind(|o| matches!(o, Output::Show(..)))
        else {
            panic!("no view");
        };
        assert_eq!(view.menu.len(), view::MAX_MENU);
        assert_eq!(view.menu[0].title, "Add a program…");
        assert!(view.menu[1].separator && view.menu[2].id == "m0");
        assert_eq!(view.menu[2].title.chars().count(), 40);
        // Without permissions.launch every launch call is refused.
        let out = outputs(code, true, 100, vec![]);
        assert!(texts(&out)[0].contains("list refused: launch.list needs permissions.launch"));
    }

    #[test]
    fn drop_calls_read_the_last_drop_and_apps_front_needs_apps() {
        let code = r#"
            const out = [];
            const tryIt = (what, f) => { try { f(); out.push(what + ' ok'); } catch (e) { out.push(what + ' refused: ' + e.message); } };
            const show = () => pf.show({ rows: [{ text: out.join(' | ') }] });
            pf.on('start', () => { tryIt('early', () => pf.drop.read(0)); tryIt('front', () => pf.apps.front()); show(); });
            pf.on('drop', () => { out.push('dropped'); show(); });
            pf.on('click', () => { tryIt('text', () => pf.drop.text(0)); tryIt('save', () => pf.drop.save(0, 'a.png')); tryIt('bad', () => pf.drop.save(0, '../a')); show(); });
        "#;
        let p = Permissions {
            drop: vec!["files".into()],
            apps: vec!["spotify.exe".into()],
            ..Permissions::default()
        };
        let fence = FenceId::new_v4();
        let drop = json!({ "files": [{ "name": "a.png", "size": 3, "folder": false }], "drop": 5 });
        // A text dropped after the file: `pf.drop.*` still reads the file's drop.
        let then = vec![
            Command::Event(fence, "drop", drop),
            Command::Event(
                fence,
                "drop",
                json!({ "files": [], "text": "t", "drop": 6 }),
            ),
            Command::Event(fence, "click", json!({ "id": "x" })),
        ];
        // Without `apps`, pf.apps.front is refused.
        let none = Permissions::default();
        let out = texts(&run_with(
            code,
            none,
            Arc::new(Fake::default()),
            true,
            50,
            vec![],
        ));
        let refused = "front refused: apps.front needs permissions.apps";
        assert!(out.join("\n").contains(refused), "{out:?}");
        let fake = Arc::new(Fake::default());
        let text = texts(&run_with(code, p, fake.clone(), true, 150, then)).join("\n");
        for want in [
            "early refused: pf.drop.read needs a drop on the fence first",
            "front ok",
            "text ok",
            "save ok",
            "bad refused: drop.save needs permissions.drop files",
        ] {
            assert!(text.contains(want), "{want:?} not in {text}");
        }
        // After the handler too: the app gets the last drop's number with the call.
        let calls: Vec<Value> = (fake.notes.lock().unwrap().iter())
            .filter_map(|n| match n {
                Note::Call {
                    name,
                    args,
                    refused: None,
                } if name.starts_with("drop.") => Some(args.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(calls, [json!([0, null, 5]), json!([0, "a.png", 5])]);
    }

    /// Answers `drop.read` with a file of this many bytes in base64, `drop.text` with its text.
    struct Dropped(usize);

    impl Host for Dropped {
        fn call(&self, _: FenceId, name: &str, _: Value) -> Result<Value, String> {
            let n = if name == "drop.read" {
                self.0.div_ceil(3) * 4
            } else {
                self.0
            };
            Ok(json!("A".repeat(n)))
        }
    }

    #[test]
    fn a_dropped_file_of_1_mb_fits_the_plugin_and_4_mb_would_not() {
        let code = r#"pf.on('drop', async () => {
            const b = await pf.drop.read(0); const t = await pf.drop.text(0);
            pf.show({ rows: [{ text: b.length + ' ' + t.length }] }); });"#;
        let p = Permissions {
            drop: vec!["files".into()],
            ..Permissions::default()
        };
        let drop = json!({ "files": [{ "name": "a.bin", "size": 1, "folder": false }], "drop": 1 });
        let run = |bytes: usize| {
            let then = vec![Command::Event(FenceId::new_v4(), "drop", drop.clone())];
            run_with(code, p.clone(), Arc::new(Dropped(bytes)), true, 400, then)
        };
        // The app's limit (MAX_DROP_READ in crates/app/src/app/light_host.rs).
        let out = run(1024 * 1024);
        assert_eq!(texts(&out), ["1398104 1048576"], "{out:?}");
        assert_eq!(stop_of(&run(4 * 1024 * 1024)), Some(Stopped::TooMuchMemory));
    }

    #[test]
    fn the_last_view_is_cached_and_shown_at_the_next_start() {
        let dir = std::env::temp_dir().join(format!("pf-cache-{}", std::process::id()));
        let path = dir.join("fence.view.json");
        let _ = std::fs::remove_dir_all(&dir);
        let start = |code: &str| {
            let (tx, rx) = channel();
            let (out_tx, out_rx) = channel();
            let host: Arc<dyn Host> = Arc::new(Fake::default());
            let thread =
                std::thread::spawn(move || run(rx, host, format, move |o| out_tx.send(o).unwrap()));
            let s = Start {
                code: code.into(),
                shown: true,
                cache: Some(path.clone()),
                ..Start::default()
            };
            tx.send(Command::Start(Box::new(s))).unwrap();
            std::thread::sleep(Duration::from_millis(300));
            drop(tx);
            thread.join().unwrap();
            texts(&out_rx.try_iter().collect::<Vec<_>>())
        };
        assert_eq!(
            start(
                "pf.after(60, () => {}); pf.on('start', () => pf.show({ rows: [{ text: 'old' }] }));"
            ),
            ["old"]
        );
        assert_eq!(
            start("pf.after(60, () => pf.show({ rows: [{ text: 'new' }] }));"),
            ["old"]
        );
        assert_eq!(
            start("pf.on('start', () => pf.show({ rows: [{ text: 'now' }] }));"),
            ["now"]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_plugin_that_reads_the_clipboard_keeps_no_view_on_disk() {
        let dir = std::env::temp_dir().join(format!("pf-cache-clip-{}", std::process::id()));
        let path = dir.join("fence.view.json");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, r#"{"rows":[{"text":"copied"}]}"#).unwrap();
        let (tx, rx) = channel();
        let (out_tx, out_rx) = channel();
        let host: Arc<dyn Host> = Arc::new(Fake::default());
        let thread =
            std::thread::spawn(move || run(rx, host, format, move |o| out_tx.send(o).unwrap()));
        let s = Start {
            code: "pf.on('start', () => pf.show({ rows: [{ text: 'copied too' }] }));".into(),
            shown: true,
            cache: Some(path.clone()),
            permissions: Permissions {
                clipboard: Clipboard::Read,
                ..Permissions::default()
            },
            ..Start::default()
        };
        tx.send(Command::Start(Box::new(s))).unwrap();
        std::thread::sleep(Duration::from_millis(300));
        drop(tx);
        thread.join().unwrap();
        assert_eq!(
            texts(&out_rx.try_iter().collect::<Vec<_>>()),
            ["copied too"]
        );
        assert!(
            !path.exists(),
            "the old view stays, or the new one was written"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
