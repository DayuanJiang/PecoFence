//! 脚本栅栏 video: the native player of a script fence's `video=` row. The script drives it
//! through the row (`play=`, `volume=`, `speed=`, `seek=`), PecoFence plays the file on the
//! graphics card ([`pecofence_platform::video`]) and tells the script how it goes on its
//! standard input.

use super::*;
use crate::scripts::Outcome;
use pecofence_core::script_output::VideoControl;
use pecofence_platform::video::{VideoPlayer, VideoState};
use std::time::{Duration, Instant};

/// The native player of a fence's `video=` row, and what the script was last told of it.
pub(super) struct ScriptVideo {
    /// Shared with the fence's window, which draws its latest picture.
    player: Rc<RefCell<VideoPlayer>>,
    /// What the row asked for last.
    control: VideoControl,
    /// The render device the player was made with ([`RenderStack::generation`]).
    generation: u32,
    /// A move to make once the file is loaded (a move asked before it was, or the place to
    /// resume at after the device was rebuilt).
    pending_seek: Option<f64>,
    /// The last move asked for that was taken (the row repeats it on every output).
    seek_done: Option<(f64, u32)>,
    /// The state line last sent to the script, and when.
    reported: String,
    reported_at: Instant,
    /// The player cannot go on (the device could not be rebuilt, a picture could not be taken).
    failed: bool,
    /// Since when the script's output has no `video=` row any more. The output is read line by
    /// line, so a screen being printed shows without its video row for a moment: the video stops
    /// only when the row stays away for [`GONE`].
    row_gone_since: Option<Instant>,
}

impl ScriptVideo {
    pub(super) fn player(&self) -> &Rc<RefCell<VideoPlayer>> {
        &self.player
    }
}

/// How often the video timer fires: while a picture is wanted (a video playing in view); while a
/// video plays out of view (its pictures are still asked for, not drawn: the engine waits for
/// that, see `VideoPlayer::update`); and otherwise (paused, loading), when only the script is
/// told how it goes.
const TICK_PLAYING_MS: u32 = 16;
const TICK_HIDDEN_MS: u32 = 40;
const TICK_IDLE_MS: u32 = 250;
/// The script is told the position this often while the video plays.
const REPORT: Duration = Duration::from_millis(250);
/// The video row has to be missing from the output this long for the video to stop.
const GONE: Duration = Duration::from_millis(800);

/// What a streaming script's standard input gets of its video, one line per report:
/// `video:<state>:<position>:<duration>` (seconds; the state is `loading`, `playing`, `paused`
/// or `ended`), or `video:error:0:0:<code>` (4: the file is missing or its codec absent, 3: the stream is
/// broken, 0: the player could not start).
fn line(state: &str, position: f64, duration: f64, error: Option<u32>) -> String {
    match error {
        Some(code) => format!("video:{state}:{position:.2}:{duration:.2}:{code}"),
        None => format!("video:{state}:{position:.2}:{duration:.2}"),
    }
}

/// What the script is told of `state`.
fn report(state: &VideoState) -> String {
    if let Some((code, _)) = state.failed {
        return line("error", 0.0, 0.0, Some(code));
    }
    let name = if !state.loaded {
        "loading"
    } else if state.ended {
        "ended"
    } else if state.paused {
        "paused"
    } else {
        "playing"
    };
    line(name, state.position, state.duration, None)
}

/// The state word of a report line: `playing` in `video:playing:1.00:9.00`.
fn state_word(report: &str) -> &str {
    report.split(':').nth(1).unwrap_or("")
}

/// Tells the player what the row asks for: play or pause, the sound level and the speed.
fn apply(player: &VideoPlayer, control: &VideoControl) {
    player.set_playing(control.playing);
    player.set_volume(control.volume);
    player.set_rate(control.speed);
}

impl App {
    /// Fence `id`'s `video=` row as its script last printed it (the first one when there are
    /// several): the player is made for a new file, told to play, pause, change its volume or
    /// move as the row says, and stopped when the row is gone. The fence shows it once the
    /// caller has pushed the tool view.
    pub(super) fn sync_video(&mut self, id: FenceId) {
        let Some(control) = self.last_video_control(id) else {
            // An error or a timeout leaves what plays going on until the script is back; a screen
            // without the row stops it once it has stayed that way ([`GONE`], in the timer).
            if self.script_runs.has_output(id) {
                self.script_runs.video_refused.remove(&id);
                if let Some(video) = self.script_runs.videos.get_mut(&id) {
                    video.row_gone_since.get_or_insert_with(Instant::now);
                    self.arm_video_timer(TICK_PLAYING_MS);
                }
            }
            return;
        };
        // A file that could not be started is not tried again on every output.
        if self.script_runs.video_refused.get(&id) == Some(&control.path) {
            return;
        }
        let stack = self.ctx.stack.clone();
        let same_file = self
            .script_runs
            .videos
            .get(&id)
            .is_some_and(|v| v.control.path == control.path);
        if !same_file {
            self.drop_video(id);
            self.script_runs.video_refused.remove(&id);
            match VideoPlayer::open(&stack.d3d_device(), &control.path, control.playing) {
                Ok(player) => {
                    tracing::info!(%id, file = %control.path, "script fence video opened");
                    self.script_runs.videos.insert(
                        id,
                        ScriptVideo {
                            player: Rc::new(RefCell::new(player)),
                            control: control.clone(),
                            generation: stack.generation(),
                            pending_seek: None,
                            seek_done: None,
                            reported: String::new(),
                            reported_at: Instant::now(),
                            failed: false,
                            row_gone_since: None,
                        },
                    );
                }
                Err(error) => {
                    tracing::warn!(%id, file = %control.path, %error, "script fence video did not start");
                    self.script_runs.video_refused.insert(id, control.path);
                    self.send_to_script(id, &line("error", 0.0, 0.0, Some(0)));
                    return;
                }
            }
        }
        let Some(video) = self.script_runs.videos.get_mut(&id) else {
            return;
        };
        video.row_gone_since = None;
        {
            let player = video.player.borrow();
            apply(&player, &control);
            if let Some(seek) = control.seek
                && video.seek_done != Some(seek)
            {
                video.seek_done = Some(seek);
                // The script waits for the state after a move: it is sent again at once.
                video.reported.clear();
                if player.state().loaded {
                    player.seek(seek.0);
                } else {
                    video.pending_seek = Some(seek.0);
                }
            }
        }
        video.control = control;
        self.arm_video_timer(TICK_PLAYING_MS);
    }

    /// The first `video=` row of fence `id`'s last output.
    fn last_video_control(&self, id: FenceId) -> Option<VideoControl> {
        use pecofence_core::script_output::{ScriptLine, parse};
        let Some((Outcome::Done(text), _)) = self.script_runs.last_of(id) else {
            return None;
        };
        parse(text).iter().find_map(|line| match line {
            ScriptLine::Row(row) => row.video_control(),
            _ => None,
        })
    }

    /// Fence `id` is no longer a script fence of the tools that are on, or has no script: its
    /// video has no one to play for.
    pub(super) fn video_orphaned(&self, id: FenceId) -> bool {
        !self.state.config.settings.tools.enabled
            || self
                .state
                .fence(id)
                .is_none_or(|f| f.tool != Some(pecofence_core::FenceTool::Script))
            || self
                .state
                .config
                .scripts
                .get(&id)
                .is_none_or(|s| s.path.is_empty())
    }

    /// Stops fence `id`'s video and lets go of its player.
    pub(super) fn drop_video(&mut self, id: FenceId) {
        if self.script_runs.videos.remove(&id).is_none() {
            return;
        }
        tracing::info!(%id, "script fence video stopped");
        if let Some(w) = self.fences.get(&self.state.host_of(id)) {
            w.set_script_video(None);
        }
        if self.script_runs.videos.is_empty() && self.script_runs.video_tick_ms != 0 {
            self.control.kill_timer(TIMER_VIDEO);
            self.script_runs.video_tick_ms = 0;
        }
    }

    fn arm_video_timer(&mut self, ms: u32) {
        if self.script_runs.video_tick_ms != ms {
            self.control.set_timer(TIMER_VIDEO, ms);
            self.script_runs.video_tick_ms = ms;
        }
    }

    /// A line for fence `id`'s streaming script; false when it has none running.
    pub(super) fn send_to_script(&mut self, id: FenceId, text: &str) -> bool {
        self.script_runs
            .streams
            .get_mut(&id)
            .is_some_and(|(_, _, stream)| stream.send(text))
    }

    /// The video timer: takes each player's new picture into its fence (when the fence is in
    /// view), makes the moves that waited for a file to load, and tells the script how its video
    /// goes (on a change, and every [`REPORT`] while it plays).
    pub(super) fn tick_videos(&mut self) {
        let stack = self.ctx.stack.clone();
        let ids: Vec<FenceId> = self.script_runs.videos.keys().copied().collect();
        let mut tick = TICK_IDLE_MS;
        for id in ids {
            // The fence was deleted, or made into something else: the video goes with it.
            if self.video_orphaned(id) {
                self.drop_video(id);
                continue;
            }
            let host = self.state.host_of(id);
            let in_view = self
                .fences
                .get(&host)
                .is_some_and(|w| w.active_fence() == id && w.shows_video());
            let Some(video) = self.script_runs.videos.get_mut(&id) else {
                continue;
            };
            if video
                .row_gone_since
                .is_some_and(|since| since.elapsed() >= GONE)
            {
                self.drop_video(id);
                continue;
            }
            // The graphics device was rebuilt and its textures are gone: the player is made
            // again and goes back to where it was.
            if video.generation != stack.generation() {
                let at = video.player.borrow().state().position;
                match VideoPlayer::open(&stack.d3d_device(), &video.control.path, true) {
                    Ok(player) => {
                        apply(&player, &video.control);
                        *video.player.borrow_mut() = player;
                        video.pending_seek = Some(at);
                        video.failed = false;
                    }
                    Err(error) => {
                        tracing::warn!(%id, %error, "script fence video not rebuilt after a device loss");
                        video.failed = true;
                    }
                }
                video.generation = stack.generation();
            }
            let mut state = video.player.borrow().state();
            if state.loaded
                && let Some(at) = video.pending_seek.take()
            {
                video.player.borrow().seek(at);
            }
            if state.failed.is_none() && !video.failed {
                let taken = video.player.borrow_mut().update(in_view);
                match taken {
                    Ok(true) => {
                        tick = TICK_PLAYING_MS;
                        if let Some(w) = self.fences.get(&host) {
                            w.redraw_video();
                        }
                    }
                    Ok(false) if !state.paused || !state.loaded => {
                        tick = tick.min(if in_view {
                            TICK_PLAYING_MS
                        } else {
                            TICK_HIDDEN_MS
                        });
                    }
                    Ok(false) => {}
                    Err(error) => {
                        tracing::warn!(%id, %error, "script fence video picture failed");
                        video.failed = true;
                    }
                }
            }
            if state.failed.is_none() && video.failed {
                state.failed = Some((0, 0));
            }
            let text = report(&state);
            let changed = state_word(&text) != state_word(&video.reported);
            if changed || (state_word(&text) == "playing" && video.reported_at.elapsed() >= REPORT)
            {
                if changed && let Some((code, hresult)) = state.failed {
                    tracing::warn!(%id, code, hresult = format!("{hresult:#x}"), file = %video.control.path, "script fence video cannot be played");
                }
                video.reported = text.clone();
                video.reported_at = Instant::now();
                self.send_to_script(id, &text);
            }
        }
        self.arm_video_timer(tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_lines() {
        let mut state = VideoState::default();
        assert_eq!(report(&state), "video:loading:0.00:0.00");
        state.loaded = true;
        state.position = 12.345;
        state.duration = 60.0;
        assert_eq!(report(&state), "video:playing:12.35:60.00");
        state.paused = true;
        assert_eq!(report(&state), "video:paused:12.35:60.00");
        state.ended = true;
        assert_eq!(report(&state), "video:ended:12.35:60.00");
        state.failed = Some((4, -1));
        assert_eq!(report(&state), "video:error:0.00:0.00:4");
        assert_eq!(state_word("video:playing:1.00:9.00"), "playing");
        assert_eq!(state_word(""), "");
    }
}
