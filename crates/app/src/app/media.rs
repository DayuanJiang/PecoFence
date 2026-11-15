//! 插件的媒体接口 (`pf.media`): Windows' current media session for plugins whose manifest has
//! `media`. A thread of its own reads it once a second while the fence of such a plugin shows
//! (or its plugin runs in the background), and not at all otherwise. When what a fence would
//! show changes, the plugins get `media` and ask `pf.media.current()` (`light_host.rs`).

use super::*;
use pecofence_platform::media::{Playing, Sessions};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};

const READ_EVERY: std::time::Duration = std::time::Duration::from_secs(1);

enum Ask {
    Watch(bool),
    /// `playpause`, `next` or `previous`, and where its answer goes.
    Control(String, Sender<std::result::Result<bool, String>>),
}

#[derive(Default)]
pub(super) struct MediaWatch {
    asks: Mutex<Option<Sender<Ask>>>,
    watching: AtomicBool,
    /// What plays, as last read (kept while nobody watches; read again when one does).
    pub(super) now: Arc<Mutex<Option<Playing>>>,
    /// Set when `now` changed: the UI thread tells the plugins.
    pub(super) changed: Arc<AtomicBool>,
}

impl MediaWatch {
    /// The media thread's queue (the thread starts with the first ask).
    fn asks(&self, control: isize) -> Option<Sender<Ask>> {
        let mut asks = self.asks.lock().ok()?;
        if asks.is_none() {
            let (tx, rx) = std::sync::mpsc::channel();
            let (now, changed) = (self.now.clone(), self.changed.clone());
            let spawned = std::thread::Builder::new()
                .name("pecofence-media".into())
                .spawn(move || watch(rx, now, changed, control));
            match spawned {
                Ok(_) => *asks = Some(tx),
                Err(e) => tracing::warn!(error = %e, "media session: no thread"),
            }
        }
        asks.clone()
    }

    /// Reads the session once a second from now on, or no more.
    pub(super) fn watch(&self, on: bool, control: isize) {
        if self.watching.swap(on, Ordering::Relaxed) == on {
            return;
        }
        tracing::info!(
            on,
            "media session: read for plugins from now on, or no more"
        );
        if let Some(tx) = self.asks(control) {
            let _ = tx.send(Ask::Watch(on));
        }
    }

    /// `pf.media.control`: whether the app playing took it.
    pub(super) fn control(&self, what: &str, control: isize) -> std::result::Result<bool, String> {
        let tx = self.asks(control).ok_or("no media thread")?;
        let (reply, answer) = std::sync::mpsc::channel();
        tx.send(Ask::Control(what.into(), reply))
            .map_err(|_| "no media thread")?;
        let wait = std::time::Duration::from_secs(5);
        (answer.recv_timeout(wait)).map_err(|_| "the media session did not answer".to_string())?
    }
}

/// Whether a fence would show `a` as it shows `b`: the same item, state, whole seconds and
/// cover or none.
fn alike(a: &Option<Playing>, b: &Option<Playing>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => {
            let shown = |p: &Playing| {
                let seconds = (p.position as i64, p.duration as i64);
                (p.playing, seconds, p.cover.is_some())
            };
            a.same_item(b) && shown(a) == shown(b)
        }
        (a, b) => a.is_none() && b.is_none(),
    }
}

/// The media thread: the session manager, read once a second while watched, and the plugins'
/// play / pause / skip.
fn watch(
    rx: Receiver<Ask>,
    now: Arc<Mutex<Option<Playing>>>,
    changed: Arc<AtomicBool>,
    control: isize,
) {
    let _com = pecofence_platform::com::MtaGuard::init();
    let sessions = Sessions::new();
    if let Err(e) = &sessions {
        tracing::warn!(error = %e, "media session manager not available: plugins see nothing playing");
    }
    let (mut watching, mut last) = (false, now.lock().ok().and_then(|n| n.clone()));
    loop {
        let ask = if watching {
            rx.recv_timeout(READ_EVERY)
        } else {
            rx.recv().map_err(|_| RecvTimeoutError::Disconnected)
        };
        match ask {
            Ok(Ask::Watch(on)) => watching = on,
            Ok(Ask::Control(what, reply)) => {
                let took = match &sessions {
                    Ok(s) => s.control(&what).map_err(|e| e.to_string()),
                    Err(_) => Ok(false),
                };
                tracing::info!(
                    what,
                    ?took,
                    "media session: a plugin played, paused or skipped"
                );
                let _ = reply.send(took);
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        let Ok(s) = sessions.as_ref().map_err(|_| ()) else {
            continue;
        };
        if !watching {
            continue;
        }
        let read = match s.current(last.as_ref()) {
            Ok(read) => read,
            Err(e) => {
                tracing::debug!(error = %e, "media session not read this time");
                continue;
            }
        };
        if !alike(&read, &last) {
            let p = read.as_ref();
            tracing::debug!(
                app = p.map(|p| p.app.as_str()),
                playing = p.map(|p| p.playing),
                cover = p.map(|p| p.cover.is_some()),
                "media session changed: plugins told"
            );
            if let Ok(mut now) = now.lock() {
                now.clone_from(&read);
            }
            changed.store(true, Ordering::Relaxed);
            let control = HWND(control as *mut core::ffi::c_void);
            window::post_message(control, WM_APP_COMMAND, 0, 0);
        }
        last = read;
    }
}
