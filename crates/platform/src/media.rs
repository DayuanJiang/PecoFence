//! The current media session, as Windows' media flyout shows it
//! (`GlobalSystemMediaTransportControlsSessionManager`): what plays, its cover, and play/pause,
//! next and previous. Every call waits for WinRT (at most [`WAIT`] each): use it on a worker
//! thread in the MTA.

use crate::media_bindings::{
    DataReader, GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use windows_core::{HRESULT, Result};
use windows_future::IAsyncOperation;

/// The largest cover read (bytes); a bigger one is left out.
const MAX_COVER: u64 = 4 * 1024 * 1024;
/// 100 ns ticks between 1601-01-01 (WinRT's DateTime) and 1970-01-01.
const TICKS_TO_UNIX: i64 = 116_444_736_000_000_000;
/// The longest wait for one answer of the media session; a later one is dropped.
const WAIT: std::time::Duration = std::time::Duration::from_secs(2);

/// A WinRT answer on its way from the thread pool to the media thread: both are in the MTA,
/// where an interface pointer may go to any thread.
struct Answer<T>(Result<T>);
// SAFETY: see above; the media thread is in the MTA (`MtaGuard`).
unsafe impl<T> Send for Answer<T> {}

/// `op`'s answer within [`WAIT`]; one that comes later is dropped (one log line then).
fn wait<T: windows_core::RuntimeType + 'static>(
    op: IAsyncOperation<T>,
    what: &'static str,
) -> Result<T> {
    let (tx, rx) = std::sync::mpsc::channel();
    op.when(move |r| {
        if tx.send(Answer(r)).is_err() {
            tracing::info!(what, "media session: an answer came after 2 s, dropped");
        }
    })?;
    match rx.recv_timeout(WAIT) {
        Ok(answer) => answer.0,
        // HRESULT_FROM_WIN32(ERROR_TIMEOUT)
        Err(_) => Err(windows_core::Error::new(
            HRESULT(0x8007_05B4_u32 as i32),
            format!("{what}: no answer within 2 s"),
        )),
    }
}

/// Where it is and how long it is, in seconds, from the timeline's 100 ns ticks. While it plays
/// the position moves on from the app's last report (`updated`, 1601-based like `now`), only
/// when the app gave that time and a length: an app without a timeline reports 0 for both.
pub fn timeline(
    start: i64,
    end: i64,
    at: i64,
    updated: i64,
    now: i64,
    playing: bool,
) -> (f64, f64) {
    let seconds = |ticks: i64| ticks.max(0) as f64 / 1e7;
    let duration = seconds(end - start);
    let mut position = seconds(at - start);
    if playing && updated > 0 && duration > 0.0 {
        // Apps report the position now and then; it moved on since.
        position += seconds(now - updated);
    }
    if duration > 0.0 {
        position = position.min(duration);
    }
    (position, duration)
}

/// What the current session plays.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Playing {
    /// The app's AppUserModelID: `Spotify.exe`, `MSEdge`, `Microsoft.ZuneMusic_…!Microsoft.ZuneMusic`.
    pub app: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub playing: bool,
    /// Seconds: where it is now (moved on from the app's last report while it plays) and how
    /// long it is (0 when the app does not say).
    pub position: f64,
    pub duration: f64,
    /// The cover as the app gave it (PNG or JPEG bytes) and its hash (FNV-1a).
    pub cover: Option<(u64, Arc<Vec<u8>>)>,
}

impl Playing {
    /// The same item of the same app (its cover is the same, unless it came late).
    pub fn same_item(&self, other: &Playing) -> bool {
        (&self.app, &self.title, &self.artist, &self.album)
            == (&other.app, &other.title, &other.artist, &other.album)
    }
}

pub struct Sessions {
    manager: Manager,
    /// The session whose media properties are watched (by its app) and the watch; `changed` is
    /// set when they change (a new cover under the same title, too).
    watched: Option<(String, windows_core::EventRevoker)>,
    changed: Arc<AtomicBool>,
}

impl Sessions {
    /// The session manager (Windows 10 1809 and later).
    pub fn new() -> Result<Self> {
        let manager = wait(Manager::RequestAsync()?, "RequestAsync")?;
        Ok(Self {
            manager,
            watched: None,
            changed: Arc::default(),
        })
    }

    fn session(&self) -> Option<Session> {
        self.manager.GetCurrentSession().ok()
    }

    /// What plays now, None without a session. `last`: what this returned before, whose cover
    /// is kept for the same item while its media properties did not change.
    pub fn current(&mut self, last: Option<&Playing>) -> Result<Option<Playing>> {
        let Some(session) = self.session() else {
            self.watched = None;
            return Ok(None);
        };
        let app = session.SourceAppUserModelId()?.to_string_lossy();
        if self.watched.as_ref().is_none_or(|(a, _)| *a != app) {
            let changed = self.changed.clone();
            let watch =
                session.MediaPropertiesChanged(move |_, _| changed.store(true, Ordering::Relaxed));
            tracing::debug!(
                app,
                ok = watch.is_ok(),
                "media session: its media properties watched"
            );
            self.watched = watch.ok().map(|w| (app.clone(), w));
        }
        let props_changed = self.changed.swap(false, Ordering::Relaxed);
        let props = wait(
            session.TryGetMediaPropertiesAsync()?,
            "TryGetMediaPropertiesAsync",
        )?;
        let status = session.GetPlaybackInfo()?.PlaybackStatus()?;
        let t = session.GetTimelineProperties()?;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
        let now = now.unwrap_or_default().as_nanos() as i64 / 100 + TICKS_TO_UNIX;
        let playing = status == Status::Playing;
        let (position, duration) = timeline(
            t.StartTime()?.duration,
            t.EndTime()?.duration,
            t.Position()?.duration,
            t.LastUpdatedTime()?.universal_time,
            now,
            playing,
        );
        let mut now = Playing {
            app,
            title: props.Title()?.to_string_lossy(),
            artist: props.Artist()?.to_string_lossy(),
            album: props.AlbumTitle()?.to_string_lossy(),
            playing,
            position,
            duration,
            cover: None,
        };
        // A cover that came late is read on a later call; one changed under the same title
        // (the media properties changed) is read again.
        let same = last.is_some_and(|l| l.same_item(&now));
        let kept = last.filter(|_| same && !props_changed);
        now.cover = match kept.and_then(|l| l.cover.clone()) {
            Some(kept) => Some(kept),
            None => cover(&props)
                .map_err(|e| tracing::debug!(error = %e, "media cover not read"))
                .ok()
                .flatten()
                .map(|bytes| (fnv(&bytes), Arc::new(bytes))),
        };
        if same && props_changed {
            tracing::debug!("media session: its media properties changed, the cover read again");
        }
        if !same {
            let (app, cover) = (&now.app, now.cover.as_ref().map(|c| c.1.len()));
            tracing::debug!(
                app,
                ?cover,
                "media session: another item (cover: its bytes)"
            );
        }
        Ok(Some(now))
    }

    /// `playpause`, `next` or `previous` in the current session: whether the app took it.
    pub fn control(&self, what: &str) -> Result<bool> {
        let Some(s) = self.session() else {
            return Ok(false);
        };
        let done = match what {
            "next" => s.TrySkipNextAsync()?,
            "previous" => s.TrySkipPreviousAsync()?,
            _ => s.TryTogglePlayPauseAsync()?,
        };
        wait(done, "control")
    }
}

/// FNV-1a: a new cover gets a new file name, so the fence draws it anew.
fn fnv(bytes: &[u8]) -> u64 {
    (bytes.iter()).fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// The cover's bytes, when the app gave one of at most [`MAX_COVER`].
fn cover(
    props: &crate::media_bindings::GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Result<Option<Vec<u8>>> {
    let Ok(thumbnail) = props.Thumbnail() else {
        return Ok(None);
    };
    let stream = wait(thumbnail.OpenReadAsync()?, "OpenReadAsync")?;
    let size = stream.Size()?;
    if size == 0 || size > MAX_COVER {
        return Ok(None);
    }
    let reader = DataReader::CreateDataReader(&stream)?;
    let got = wait(reader.LoadAsync(size as u32)?, "LoadAsync")?;
    let mut bytes = vec![0; got as usize];
    reader.ReadBytes(&mut bytes)?;
    Ok(Some(bytes))
}

#[cfg(test)]
mod tests {
    use super::timeline;

    #[test]
    fn the_position_moves_on_only_with_a_report_time_and_a_length() {
        let s = 10_000_000; // a second in ticks
        let now = 13_400_000_000 * s;
        // Playing, reported 5 s ago at 10 s of 180: 15 s.
        assert_eq!(
            timeline(0, 180 * s, 10 * s, now - 5 * s, now, true),
            (15.0, 180.0)
        );
        // Paused: where it was reported.
        assert_eq!(
            timeline(0, 180 * s, 10 * s, now - 5 * s, now, false),
            (10.0, 180.0)
        );
        // Never past the end.
        assert_eq!(
            timeline(0, 180 * s, 178 * s, now - 5 * s, now, true),
            (180.0, 180.0)
        );
        // A start other than 0.
        assert_eq!(
            timeline(20 * s, 200 * s, 30 * s, now, now, true),
            (10.0, 180.0)
        );
        // No timeline (LastUpdatedTime 0): no 1.3e10 s, and the same each second.
        assert_eq!(timeline(0, 0, 0, 0, now, true), (0.0, 0.0));
        assert_eq!(timeline(0, 180 * s, 10 * s, 0, now, true), (10.0, 180.0));
        // A report time without a length (a live stream): it stays.
        assert_eq!(timeline(0, 0, 10 * s, now - 5 * s, now, true), (10.0, 0.0));
    }
}
