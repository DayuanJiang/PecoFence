//! The light layer of plugins (插件的轻量层): each plugin fence runs its plugin's `main.js` in a
//! QuickJS runtime of its own (at most 8 MB, 50 ms per call, no `eval` or `Function`), all of
//! them on one plugin thread. A plugin talks to PecoFence through `pf` (see `prelude.js`): it
//! shows a view, sets timers that PecoFence aligns to the clock, and gets events. Nothing here
//! touches Windows: the app passes in how times are written and takes the views back.

use pecofence_core::FenceId;
use rquickjs::{CaughtError, Context, Ctx, Function, Object, Runtime};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub mod view;
pub use view::{Row, View};

pub const MEMORY_LIMIT: usize = 8 * 1024 * 1024;
pub const CALL_LIMIT: Duration = Duration::from_millis(50);
/// `pf.show` takes effect at most 10 times a second: a view that comes sooner waits, and a
/// later one takes its place.
const SHOW_GAP: Duration = Duration::from_millis(100);
const MAX_TIMERS: usize = 20;
/// `pf.time.format(unix_ms, kind)`: the time in Windows' region format.
pub type Format = fn(i64, &str) -> String;

/// Fence `fence`'s plugin starts: its code and what `pf` says about the fence.
pub struct Start {
    pub fence: FenceId,
    pub code: String,
    pub settings: serde_json::Value,
    pub lang: String,
    pub theme: String,
    /// The content area in DIPs.
    pub size: (f32, f32),
    pub shown: bool,
}

pub enum Command {
    Start(Box<Start>),
    /// Shown or hidden: `show` / `hide`, and the timers wait while it is hidden.
    Shown(FenceId, bool),
    /// `timechange`, `settings` (the new settings) or `resize` ({width, height}).
    Event(FenceId, &'static str, serde_json::Value),
    Stop(FenceId),
    /// Every plugin gets `stop`, then the thread ends.
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stopped {
    TooLong,
    TooMuchMemory,
    Error(String),
}

#[derive(Debug, PartialEq)]
pub enum Output {
    Show(FenceId, View),
    Stopped(FenceId, Stopped),
}

/// Starts the plugin thread; `out` gets what the plugins show (on that thread).
pub fn spawn(
    format: Format,
    out: impl Fn(Output) + Send + 'static,
) -> std::io::Result<(Sender<Command>, std::thread::JoinHandle<()>)> {
    let (tx, rx) = std::sync::mpsc::channel();
    let thread = std::thread::Builder::new()
        .name("pecofence-plugins-js".into())
        .spawn(move || run(rx, format, out))?;
    Ok((tx, thread))
}

struct Timer {
    id: u32,
    step: u64,
    aligned: bool,
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

/// What the plugin's calls into `pf` leave for the host.
#[derive(Default)]
struct Shared {
    timers: Vec<Timer>,
    view: Option<View>,
}

struct Plugin {
    fence: FenceId,
    // The context is dropped before its runtime.
    ctx: Context,
    rt: Runtime,
    deadline: Rc<Cell<Option<Instant>>>,
    shared: Rc<RefCell<Shared>>,
    shown: bool,
    sent: Option<Instant>,
}

fn stopped(text: String) -> Stopped {
    if text.contains("interrupted") {
        Stopped::TooLong
    } else if text.contains("out of memory") || text.contains("Allocation") {
        Stopped::TooMuchMemory
    } else {
        Stopped::Error(text.lines().next().unwrap_or_default().to_string())
    }
}

impl Plugin {
    fn start(s: Start, format: Format) -> Result<Self, Stopped> {
        let fail = |e: rquickjs::Error| Stopped::Error(e.to_string());
        let rt = Runtime::new().map_err(fail)?;
        rt.set_memory_limit(MEMORY_LIMIT);
        let deadline: Rc<Cell<Option<Instant>>> = Rc::default();
        let late = deadline.clone();
        rt.set_interrupt_handler(Some(Box::new(move || {
            late.get().is_some_and(|d| Instant::now() > d)
        })));
        let ctx = Context::full(&rt).map_err(fail)?;
        let mut plugin = Plugin {
            fence: s.fence,
            ctx,
            rt,
            deadline,
            shared: Rc::default(),
            shown: s.shown,
            sent: None,
        };
        let shared = plugin.shared.clone();
        let fence =
            serde_json::json!({ "id": s.fence.to_string(), "width": s.size.0, "height": s.size.1 });
        plugin.call(move |ctx| {
            let n = Object::new(ctx.clone())?;
            n.set("lang", s.lang)?;
            n.set("theme", s.theme)?;
            n.set("settings", s.settings.to_string())?;
            n.set("fence", fence.to_string())?;
            let to = shared.clone();
            let show = move |json: String| match view::parse(&json) {
                Ok(view) => to.borrow_mut().view = Some(view),
                Err(e) => tracing::debug!(error = %e, "plugin view not read"),
            };
            n.set("show", Function::new(ctx.clone(), show)?)?;
            let to = shared.clone();
            let every = move |id: u32, seconds: f64, align: String| {
                let mut s = to.borrow_mut();
                if s.timers.len() >= MAX_TIMERS {
                    return;
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
                    next: Instant::now(),
                };
                timer.arm();
                s.timers.push(timer);
            };
            n.set("every", Function::new(ctx.clone(), every)?)?;
            let to = shared.clone();
            let cancel = move |id: u32| to.borrow_mut().timers.retain(|t| t.id != id);
            n.set("cancel", Function::new(ctx.clone(), cancel)?)?;
            let format = move |ms: f64, kind: String| format(ms as i64, &kind);
            n.set("format", Function::new(ctx.clone(), format)?)?;
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

    /// Runs `f` in the plugin, then the promise jobs it queued, within [`CALL_LIMIT`].
    fn call(
        &mut self,
        f: impl for<'js> FnOnce(&Ctx<'js>) -> rquickjs::Result<()>,
    ) -> Result<(), Stopped> {
        self.deadline.set(Some(Instant::now() + CALL_LIMIT));
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
        result.map_err(stopped)
    }

    fn emit(&mut self, name: &str, data: Option<String>) -> Result<(), Stopped> {
        let name = name.to_string();
        self.call(move |ctx| {
            let emit: Function = ctx.globals().get("__pf_emit")?;
            emit.call((name, data))
        })
    }

    fn set_shown(&mut self, shown: bool) -> Result<(), Stopped> {
        if std::mem::replace(&mut self.shown, shown) == shown {
            return Ok(());
        }
        if shown {
            // Back after a while: every timer starts from now (aligned ones at the next boundary).
            self.shared
                .borrow_mut()
                .timers
                .iter_mut()
                .for_each(Timer::arm);
        }
        tracing::debug!(fence = %self.fence, shown, "plugin shown or hidden, its timers follow");
        self.emit(if shown { "show" } else { "hide" }, None)
    }

    /// Runs the timers that are due (none while the fence is hidden).
    fn tick(&mut self, now: Instant) -> Result<(), Stopped> {
        if !self.shown {
            return Ok(());
        }
        let due: Vec<u32> = {
            let mut s = self.shared.borrow_mut();
            let due = s.timers.iter_mut().filter(|t| t.next <= now);
            due.map(|t| {
                t.arm();
                t.id
            })
            .collect()
        };
        for id in due {
            self.call(move |ctx| {
                let tick: Function = ctx.globals().get("__pf_tick")?;
                tick.call((id,))
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
        Some(view)
    }

    fn next_due(&self) -> Option<Instant> {
        let s = self.shared.borrow();
        let timers = s.timers.iter().filter(|_| self.shown).map(|t| t.next);
        let view = s.view.as_ref().and(self.sent).map(|at| at + SHOW_GAP);
        timers.chain(view).min()
    }
}

/// The plugin thread: commands from the app, the plugins' timers, their views out.
pub fn run(rx: Receiver<Command>, format: Format, out: impl Fn(Output)) {
    let mut plugins: Vec<Plugin> = Vec::new();
    loop {
        let due = plugins.iter().filter_map(Plugin::next_due).min();
        let cmd = match due {
            Some(at) => rx.recv_timeout(at.saturating_duration_since(Instant::now())),
            None => rx.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        let at = |fence: FenceId, plugins: &[Plugin]| plugins.iter().position(|p| p.fence == fence);
        let mut failed: Vec<(FenceId, Stopped)> = Vec::new();
        match cmd {
            Err(RecvTimeoutError::Timeout) => {}
            Ok(Command::Quit) | Err(RecvTimeoutError::Disconnected) => {
                tracing::info!(plugins = plugins.len(), "plugin thread: every plugin stops");
                for p in &mut plugins {
                    let _ = p.emit("stop", None);
                }
                return;
            }
            Ok(Command::Start(s)) => {
                let fence = s.fence;
                if let Some(i) = at(fence, &plugins) {
                    plugins.remove(i);
                }
                let begun = Instant::now();
                match Plugin::start(*s, format) {
                    Ok(p) => {
                        let used = p.rt.memory_usage().memory_used_size;
                        tracing::info!(%fence, ms = begun.elapsed().as_millis(), used, "plugin started");
                        plugins.push(p);
                    }
                    Err(why) => failed.push((fence, why)),
                }
            }
            Ok(Command::Stop(fence)) => {
                if let Some(i) = at(fence, &plugins) {
                    let _ = plugins.remove(i).emit("stop", None);
                    tracing::info!(%fence, "plugin stopped");
                }
            }
            Ok(Command::Shown(fence, shown)) => {
                if let Some(i) = at(fence, &plugins)
                    && let Err(why) = plugins[i].set_shown(shown)
                {
                    failed.push((fence, why));
                }
            }
            Ok(Command::Event(fence, name, data)) => {
                if let Some(p) = plugins.iter_mut().find(|p| p.fence == fence) {
                    if name == "timechange" {
                        let mut s = p.shared.borrow_mut();
                        s.timers
                            .iter_mut()
                            .filter(|t| t.aligned)
                            .for_each(Timer::arm);
                    }
                    let data = (!data.is_null()).then(|| data.to_string());
                    if let Err(why) = p.emit(name, data) {
                        failed.push((fence, why));
                    }
                }
            }
        }
        let now = Instant::now();
        for p in &mut plugins {
            if let Err(why) = p.tick(now) {
                failed.push((p.fence, why));
            }
        }
        for (fence, why) in failed {
            tracing::warn!(%fence, ?why, "plugin stopped by PecoFence");
            if let Some(i) = at(fence, &plugins) {
                plugins.remove(i);
            }
            out(Output::Stopped(fence, why));
        }
        for p in &mut plugins {
            if let Some(view) = p.view_to_send(now) {
                out(Output::Show(p.fence, view));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    fn format(ms: i64, kind: &str) -> String {
        format!("{kind} {ms}")
    }

    /// Runs `code` in a plugin thread for `ms` (after `then` was sent) and returns what came out.
    fn outputs(code: &str, shown: bool, ms: u64, then: Vec<Command>) -> Vec<Output> {
        let fence = FenceId::new_v4();
        let (tx, rx) = channel();
        let (out_tx, out_rx) = channel();
        let thread = std::thread::spawn(move || run(rx, format, move |o| out_tx.send(o).unwrap()));
        let start = Start {
            fence,
            code: code.into(),
            settings: serde_json::json!({ "seconds": true }),
            lang: "zh-CN".into(),
            theme: "dark".into(),
            size: (240.0, 104.0),
            shown,
        };
        tx.send(Command::Start(Box::new(start))).unwrap();
        for cmd in then {
            std::thread::sleep(Duration::from_millis(ms));
            let cmd = match cmd {
                Command::Shown(_, s) => Command::Shown(fence, s),
                Command::Event(_, n, d) => Command::Event(fence, n, d),
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
            Output::Show(_, v) => Some(
                v.rows
                    .iter()
                    .map(|r| r.text.clone())
                    .collect::<Vec<_>>()
                    .join("|"),
            ),
            Output::Stopped(..) => None,
        });
        rows.collect()
    }

    fn stop_of(out: &[Output]) -> Option<Stopped> {
        out.iter().find_map(|o| match o {
            Output::Stopped(_, why) => Some(why.clone()),
            Output::Show(..) => None,
        })
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
    fn a_memory_hog_is_stopped_at_8_mb() {
        let out = outputs(
            "const a = []; while (true) a.push('x'.repeat(1000) + a.length);",
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
            pf.on('start', () => pf.show({ rows: [{ text: [pf.lang, pf.theme, pf.settings.seconds, pf.fence.width, pf.time.format(1500, 'time')].join(' ') }] }));
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
        assert_eq!(texts(&out), ["zh-CN dark true 240 time 1500", "w301"]);
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
    }
}
