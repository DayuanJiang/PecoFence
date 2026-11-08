//! 自由摆放 with no saved cells (a new installation, or the first time it is switched on): the
//! loose icons take the places Windows' own desktop has them at (#204), so showing the fences
//! keeps the order the user sees when the fences are hidden. Explorer's desktop view is read
//! once on a worker; the icons wait in their columns meanwhile (nothing is saved), then take
//! their places, and those cells are saved like any others. Cells already saved are never
//! replaced, and a sort that lines the icons up again is not undone by it.

use super::*;
use crate::fence_window::WindowsIconPlaces;
use pecofence_platform::shell_icons::{self, DesktopIconPlaces};
use std::collections::BTreeMap;

/// `WM_APP + 34`: the worker read Windows' icon places (worker thread → UI thread).
pub(super) const WM_APP_WINDOWS_PLACES: u32 = pecofence_platform::msg::WM_APP + 34;
/// Takes the worker's answer on the UI thread; armed with [`GIVE_UP`] when the read starts,
/// so a worker stuck in a call to a hung Explorer does not keep the icons waiting.
pub(super) const TIMER_WINDOWS_PLACES: usize = 84;
/// How long the worker keeps asking while Explorer has no desktop view yet (sign-in), or one
/// without any icon while 「桌面」 has some.
const READ_FOR: Duration = Duration::from_secs(20);
const RETRY: Duration = Duration::from_millis(500);
const GIVE_UP: Duration = Duration::from_secs(25);
/// How long the start waits for the places before the fences show, so the icons do not stand
/// in their columns for a frame and then jump. Explorer answers in a few ms; at sign-in, with
/// no desktop yet, the fences show after this wait and the read goes on.
const BEFORE_SHOW: Duration = Duration::from_millis(1500);

/// What the worker found, and after how many reads and how long.
type Answer = (
    std::result::Result<DesktopIconPlaces, (&'static str, String)>,
    u32,
    Duration,
);

#[derive(Default)]
pub(super) struct WindowsPlaces {
    state: Stage,
    slot: Arc<Mutex<Option<Answer>>>,
}

#[derive(Default)]
enum Stage {
    /// Read when 自由摆放 next places 「桌面」's icons without saved cells: set at startup and
    /// whenever 自由摆放 or 像桌面一样排列 is switched on.
    #[default]
    Wanted,
    /// The worker is reading.
    Reading(Instant),
    /// Read: given to 「桌面」 until its cells are saved.
    Read(WindowsIconPlaces),
    /// Used, given up, or not needed (the cells were saved already): icons without a cell take
    /// the first free ones.
    Done,
}

impl App {
    /// 自由摆放 or 像桌面一样排列 was switched on: 「桌面」 takes Windows' places if it has
    /// no saved cells (it has none the first time).
    pub(super) fn want_windows_places(&mut self) {
        if !matches!(self.windows_places.state, Stage::Reading(_)) {
            self.windows_places.state = Stage::Wanted;
        }
    }

    /// For [`Self::arrange_desktop_fence`]: the saved cells `free` of 「桌面」 (None: no free
    /// placement), and what goes with them. Without saved cells the first time, the icons stay
    /// in their columns (`free` None, so nothing is saved) while Windows' places are read, then
    /// get those places.
    pub(super) fn windows_places_for(
        &mut self,
        inbox: FenceId,
        free: &mut Option<BTreeMap<ItemId, [u16; 2]>>,
    ) -> Option<WindowsIconPlaces> {
        let cells = free.as_ref()?;
        let icons = self.state.fence(inbox).map_or(0, |f| f.items.len());
        if !cells.is_empty() {
            if let Stage::Read(places) = &self.windows_places.state {
                tracing::info!(
                    places = places.places.len(),
                    cells = cells.len(),
                    "free placement: Windows desktop places used, cells saved"
                );
            }
            self.windows_places.state = Stage::Done;
            return None;
        }
        match &self.windows_places.state {
            Stage::Wanted if icons == 0 => None,
            Stage::Wanted => {
                self.start_windows_places_read(icons);
                *free = None;
                None
            }
            Stage::Reading(_) => {
                *free = None;
                None
            }
            Stage::Read(_) if icons == 0 => None,
            Stage::Read(places) => Some(places.clone()),
            Stage::Done => None,
        }
    }

    fn start_windows_places_read(&mut self, icons: usize) {
        tracing::info!(
            icons,
            "free placement: no saved cells, reading the Windows desktop's icon places"
        );
        self.windows_places.state = Stage::Reading(Instant::now());
        let slot = self.windows_places.slot.clone();
        if let Ok(mut s) = slot.lock() {
            *s = None;
        }
        let control = self.control.hwnd();
        window::set_timer(control, TIMER_WINDOWS_PLACES, GIVE_UP.as_millis() as u32);
        let control = control.0 as isize;
        let spawned = std::thread::Builder::new()
            .name("pecofence-windows-places".into())
            .spawn(move || {
                let started = Instant::now();
                let _com = pecofence_platform::com::MtaGuard::init();
                let mut tries = 0;
                let answer = loop {
                    tries += 1;
                    let read = shell_icons::desktop_icon_places();
                    // At sign-in Explorer may have no desktop yet, or one still empty.
                    let early = match &read {
                        Ok(p) => p.icons.is_empty(),
                        Err(_) => true,
                    };
                    if early && tries == 1 {
                        match &read {
                            Ok(_) => tracing::info!(
                                "free placement: Windows desktop shows no icons yet; asking again"
                            ),
                            Err((reason, error)) => tracing::info!(
                                reason,
                                %error,
                                "free placement: Windows desktop not read yet; asking again"
                            ),
                        }
                    }
                    if !early || started.elapsed() >= READ_FOR {
                        break read;
                    }
                    std::thread::sleep(RETRY);
                };
                if let Ok(mut s) = slot.lock() {
                    *s = Some((answer, tries, started.elapsed()));
                }
                window::post_message(
                    HWND(control as *mut core::ffi::c_void),
                    WM_APP_WINDOWS_PLACES,
                    0,
                    0,
                );
            });
        if let Err(error) = spawned {
            tracing::warn!(%error, "free placement: Windows desktop read thread failed to start");
            self.windows_places.state = Stage::Done;
        }
    }

    /// At startup, before the fences show (and Windows' icons are hidden): waits up to
    /// [`BEFORE_SHOW`] for a read in progress and takes its answer.
    pub(super) fn windows_places_before_show(&mut self) {
        if !matches!(self.windows_places.state, Stage::Reading(_)) {
            return;
        }
        let started = Instant::now();
        let mut answered = false;
        while !answered && started.elapsed() < BEFORE_SHOW {
            std::thread::sleep(Duration::from_millis(5));
            answered = self.windows_places.slot.lock().is_ok_and(|s| s.is_some());
        }
        tracing::info!(
            answered,
            waited_ms = started.elapsed().as_millis(),
            "free placement: Windows desktop read waited for before the fences show"
        );
        self.finish_windows_places();
    }

    /// `TIMER_WINDOWS_PLACES`: the worker's answer, or the end of the wait for it. 「桌面」 then
    /// places its icons: at Windows' places, or from the first free cells when they could not
    /// be read.
    pub(super) fn finish_windows_places(&mut self) {
        let Stage::Reading(since) = self.windows_places.state else {
            return;
        };
        let answer = self
            .windows_places
            .slot
            .lock()
            .ok()
            .and_then(|mut s| s.take());
        let next = match answer {
            None if since.elapsed() < GIVE_UP => {
                // Woken early: wait out the rest.
                let left = GIVE_UP.saturating_sub(since.elapsed()).as_millis() as u32;
                window::set_timer(self.control.hwnd(), TIMER_WINDOWS_PLACES, left.max(1));
                return;
            }
            None => {
                tracing::info!(
                    waited_ms = since.elapsed().as_millis(),
                    "free placement: Windows desktop did not answer; icons take the first free cells"
                );
                Stage::Done
            }
            Some((Err((reason, error)), tries, took)) => {
                tracing::info!(
                    reason,
                    %error,
                    tries,
                    ms = crate::share_log::millis(took),
                    "free placement: Windows desktop icon places not read; icons take the first free cells"
                );
                Stage::Done
            }
            Some((Ok(read), tries, took)) => Stage::Read(self.windows_places_of(read, tries, took)),
        };
        self.windows_places.state = next;
        self.arrange_desktop_fence();
    }

    /// The places of the icons PecoFence knows, by item. Windows lists every desktop icon,
    /// those of the other fences too: 「桌面」 only looks up its own.
    fn windows_places_of(
        &self,
        read: DesktopIconPlaces,
        tries: u32,
        took: Duration,
    ) -> WindowsIconPlaces {
        let mut places = BTreeMap::new();
        for (name, at) in &read.icons {
            if let Some(id) = self.state.item_id_for_path(Path::new(name)) {
                places.insert(id, *at);
            }
        }
        let inbox = self.state.inbox_id().and_then(|id| self.state.fence(id));
        let loose = inbox.map_or(0, |f| f.items.len());
        let loose_found = inbox.map_or(0, |f| {
            f.items
                .iter()
                .filter(|r| places.contains_key(&r.item_id))
                .count()
        });
        let left = read.icons.iter().map(|(_, at)| at.0).min();
        let top = read.icons.iter().map(|(_, at)| at.1).min();
        tracing::info!(
            windows_icons = read.icons.len(),
            known = places.len(),
            loose,
            loose_found,
            spacing = ?read.spacing,
            explorer_dpi = ?read.explorer_dpi,
            auto_arrange = read.auto_arrange,
            left = ?left,
            top = ?top,
            tries,
            ms = crate::share_log::millis(took),
            "free placement: Windows desktop icon places read"
        );
        WindowsIconPlaces {
            spacing: read.spacing,
            explorer_dpi: read.explorer_dpi,
            places,
        }
    }
}
