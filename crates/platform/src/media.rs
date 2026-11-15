//! The current media session, as Windows' media flyout shows it
//! (`GlobalSystemMediaTransportControlsSessionManager`): what plays, its cover, and play/pause,
//! next and previous. Every call waits for WinRT: use it on a worker thread in the MTA.

use crate::media_bindings::{
    DataReader, GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
};
use windows_core::Result;

/// The largest cover read (bytes); a bigger one is left out.
const MAX_COVER: u64 = 4 * 1024 * 1024;
/// 100 ns ticks between 1601-01-01 (WinRT's DateTime) and 1970-01-01.
const TICKS_TO_UNIX: i64 = 116_444_736_000_000_000;

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
    /// The cover as the app gave it (PNG or JPEG bytes).
    pub cover: Option<std::sync::Arc<Vec<u8>>>,
}

impl Playing {
    /// The same item of the same app (its cover is the same, unless it came late).
    pub fn same_item(&self, other: &Playing) -> bool {
        (&self.app, &self.title, &self.artist, &self.album)
            == (&other.app, &other.title, &other.artist, &other.album)
    }
}

pub struct Sessions(Manager);

impl Sessions {
    /// The session manager (Windows 10 1809 and later).
    pub fn new() -> Result<Self> {
        Ok(Self(Manager::RequestAsync()?.join()?))
    }

    fn session(&self) -> Option<Session> {
        self.0.GetCurrentSession().ok()
    }

    /// What plays now, None without a session. `last`: what this returned before, whose cover
    /// is kept for the same item.
    pub fn current(&self, last: Option<&Playing>) -> Result<Option<Playing>> {
        let Some(session) = self.session() else {
            return Ok(None);
        };
        let props = session.TryGetMediaPropertiesAsync()?.join()?;
        let status = session.GetPlaybackInfo()?.PlaybackStatus()?;
        let timeline = session.GetTimelineProperties()?;
        let seconds = |ticks: i64| ticks.max(0) as f64 / 1e7;
        let (start, end) = (timeline.StartTime()?.duration, timeline.EndTime()?.duration);
        let duration = seconds(end - start);
        let mut position = seconds(timeline.Position()?.duration - start);
        let playing = status == Status::Playing;
        if playing {
            // Apps report the position now and then; it moved on since.
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
            let now = now.unwrap_or_default().as_nanos() as i64 / 100 + TICKS_TO_UNIX;
            position += seconds(now - timeline.LastUpdatedTime()?.universal_time);
        }
        if duration > 0.0 {
            position = position.min(duration);
        }
        let mut now = Playing {
            app: session.SourceAppUserModelId()?.to_string_lossy(),
            title: props.Title()?.to_string_lossy(),
            artist: props.Artist()?.to_string_lossy(),
            album: props.AlbumTitle()?.to_string_lossy(),
            playing,
            position,
            duration,
            cover: None,
        };
        // A cover that came late is read on a later call.
        let same = last.is_some_and(|l| l.same_item(&now));
        now.cover = match last.filter(|_| same).and_then(|l| l.cover.clone()) {
            Some(kept) => Some(kept),
            None => cover(&props)
                .map_err(|e| tracing::debug!(error = %e, "media cover not read"))
                .ok()
                .flatten()
                .map(std::sync::Arc::new),
        };
        if !same {
            let (app, cover) = (&now.app, now.cover.as_ref().map(|c| c.len()));
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
        done.join()
    }
}

/// The cover's bytes, when the app gave one of at most [`MAX_COVER`].
fn cover(
    props: &crate::media_bindings::GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Result<Option<Vec<u8>>> {
    let Ok(thumbnail) = props.Thumbnail() else {
        return Ok(None);
    };
    let stream = thumbnail.OpenReadAsync()?.join()?;
    let size = stream.Size()?;
    if size == 0 || size > MAX_COVER {
        return Ok(None);
    }
    let reader = DataReader::CreateDataReader(&stream)?;
    let got = reader.LoadAsync(size as u32)?.join()?;
    let mut bytes = vec![0; got as usize];
    reader.ReadBytes(&mut bytes)?;
    Ok(Some(bytes))
}
