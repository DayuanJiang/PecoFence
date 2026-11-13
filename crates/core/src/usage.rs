//! Usage statistics: which features are used and in which order, so that the work on PecoFence
//! goes where the use is. The program calls [`event`], [`event_with`] and [`event_named`] at a
//! few central places; `docs/USAGE_STATS.md` lists them and every name. One recorder for the
//! whole process keeps a report per day. It is off, and the calls do nothing, until the app
//! switches it on ([`start`]). The file `usage.json` and the uploads are the app's side
//! (`app/usage.rs`).
//!
//! Never recorded: file names, paths, search words, note texts, fence titles, any typed text,
//! hotkey combinations, theme names.

use crate::model::Settings;
use crate::rules::RuleSet;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// Records that `name` was used. A name is a string literal of the form `^[a-z0-9_.]{1,60}$`:
/// never a `String`, never anything from a user.
pub fn event(name: &'static str) {
    record(name.into(), String::new());
}

/// The same, with a second fixed word for what varies (`hotkey` and `peek`).
pub fn event_with(name: &'static str, key: &'static str) {
    record(name.into(), key.into());
}

/// The same for a name from one of the code's own tables, which the code only has as text at
/// run time: a command's variant (`LaunchItem`), a CLI method (`fences.setBounds`), a setting's
/// path (`rollUp.hoverPeek`). They are recorded in the spelling of every other name
/// (`launch_item`, `fences.set_bounds`). `key` may be empty.
pub fn event_named(name: &str, key: &str) {
    record(snake(name), snake(key));
}

/// A day keeps this many events (about 110 KB); what comes after them is not recorded.
pub const MAX_EVENTS: usize = 2000;
/// Days the service has not taken yet are kept, this many at most beside today's.
pub const MAX_UNSENT_DAYS: usize = 7;

/// Tells the local date (`2026-11-14`) and the second of that day.
pub type Clock = fn() -> (String, u32);

/// One day's report: an entry of `usage.json`, and the body of an upload.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Report {
    /// The form of a report.
    pub v: u32,
    /// The install number: 32 hex digits made at random at the first start.
    pub id: String,
    /// The local date.
    pub day: String,
    /// The PecoFence version, the edition, the Windows version, the display language.
    pub app: String,
    pub edition: String,
    pub windows: String,
    pub language: String,
    /// [`settings_report`].
    pub settings: Value,
    /// A few counts (fences, items) and how the program ran (start time, memory).
    pub state: Value,
    pub health: Value,
    /// `[second of the local day, name]` or `[second, name, key]`, in order.
    pub events: Vec<Value>,
}

/// What the calls record into while the statistics are on.
struct Recorder {
    /// The days the service has not taken yet, oldest first. The last one is today's.
    days: Vec<Report>,
    clock: Clock,
    /// Something was recorded, or a day began or was sent, since `usage.json` was written.
    unsaved: bool,
}

impl Recorder {
    /// Today's report and the second of the day. A new report begins when the local date is
    /// not the last report's.
    fn today(&mut self) -> (&mut Report, u32) {
        let (day, second) = (self.clock)();
        if self.days.last().is_none_or(|report| report.day != day) {
            let id = self.days.last().map_or_else(new_id, |r| r.id.clone());
            self.days.push(Report {
                v: 1,
                id,
                day,
                ..Report::default()
            });
            if self.days.len() > MAX_UNSENT_DAYS + 1 {
                self.days.remove(0);
            }
            self.unsaved = true;
        }
        let today = self.days.len() - 1;
        (&mut self.days[today], second)
    }
}

static RECORDER: Mutex<Option<Recorder>> = Mutex::new(None);

fn recorder() -> MutexGuard<'static, Option<Recorder>> {
    RECORDER.lock().unwrap_or_else(PoisonError::into_inner)
}

fn record(name: String, key: String) {
    let mut guard = recorder();
    let Some(recorder) = guard.as_mut() else {
        return;
    };
    let (today, second) = recorder.today();
    if today.events.len() >= MAX_EVENTS || !valid(&name) || !(key.is_empty() || valid(&key)) {
        return;
    }
    today.events.push(if key.is_empty() {
        json!([second, name])
    } else {
        json!([second, name, key])
    });
    recorder.unsaved = true;
}

/// Whether `name` may go into a report: `^[a-z0-9_.]{1,60}$`.
fn valid(name: &str) -> bool {
    let allowed = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'.';
    (1..=60).contains(&name.len()) && name.bytes().all(allowed)
}

/// `LaunchItem` as `launch_item`, `fences.setBounds` as `fences.set_bounds`.
fn snake(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_uppercase() && !out.is_empty() && !out.ends_with('.') {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// A new install number: 32 hex digits, 122 of its bits random.
fn new_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

/// Switches the statistics on. `saved` is the text of `usage.json` (empty or unusable: this
/// is a new installation, which gets a new install number). Returns the PecoFence version of
/// the last saved report, empty for a new installation.
pub fn start(saved: &str, clock: Clock) -> String {
    let days: Vec<Report> = serde_json::from_str(saved).unwrap_or_default();
    let last_app = days.last().map(|r| r.app.clone()).unwrap_or_default();
    let mut new = Recorder {
        days,
        clock,
        unsaved: false,
    };
    new.today();
    *recorder() = Some(new);
    last_app
}

/// Switches the statistics off: nothing is recorded from here on, and what was is forgotten.
pub fn stop() {
    *recorder() = None;
}

/// Today's date and whether something changed since `usage.json` was written. `None` while
/// the statistics are off.
pub fn today() -> Option<(String, bool)> {
    let mut guard = recorder();
    let recorder = guard.as_mut()?;
    let day = recorder.today().0.day.clone();
    Some((day, recorder.unsaved))
}

/// Sets what today's report says beside its events: the versions, the settings, the counts.
/// `set` must not call into this module.
pub fn update(set: impl FnOnce(&mut Report)) {
    if let Some(recorder) = recorder().as_mut() {
        set(recorder.today().0);
    }
}

/// The text for `usage.json`, when something changed since this was last asked.
pub fn snapshot() -> Option<String> {
    let mut guard = recorder();
    let recorder = guard.as_mut().filter(|r| r.unsaved)?;
    recorder.unsaved = false;
    serde_json::to_string(&recorder.days).ok()
}

/// The reports to send, as `(day, body)`: every earlier day, and today's too when `today`
/// is set.
pub fn unsent(today: bool) -> Vec<(String, String)> {
    let guard = recorder();
    let Some(recorder) = guard.as_ref() else {
        return Vec::new();
    };
    let count = recorder.days.len() - usize::from(!today);
    let body = |r: &Report| Some((r.day.clone(), serde_json::to_string(r).ok()?));
    recorder.days[..count].iter().filter_map(body).collect()
}

/// The service took the report of `day`: an earlier day is forgotten. Today's stays.
pub fn sent(day: &str) {
    if let Some(recorder) = recorder().as_mut() {
        let today = recorder.days.pop();
        recorder.days.retain(|report| report.day != day);
        recorder.days.extend(today);
        recorder.unsaved = true;
    }
}

/// The settings a report carries, by their paths in `config.json` (the two rule controls
/// under `rules.`): every switch, choice from a fixed list and number. A setting that holds
/// text has no place on this list.
const REPORTED_SETTINGS: &[&str] = &[
    "language",
    "theme",
    "themeStyle",
    "labelColor",
    "titleAlign",
    "tabBar",
    "iconSize",
    "quickHide.enabled",
    "rollUp.hoverPeek",
    "rollUp.clickToExpand",
    "rollUp.hoverSpeed",
    "rollUp.titleOnHover",
    "rollUp.hideInactiveScrollbar",
    "rollUp.pushNeighbors",
    "rollUp.switchTabOnHover",
    "rollUp.hideCountAndChevron",
    "showDesktop",
    "hideRealIcons",
    "keepIconsHidden",
    "hideInboxWhenIconsShown",
    "offerNewFence",
    "perDesktopFences",
    "featureTips",
    "autoCheckUpdates",
    "snapping.enabled",
    "snapping.gapPx",
    "snapping.sizeToCells",
    "snapping.guideLines",
    "zorder",
    "autostart",
    "peek.enabled",
    "peek.dim",
    "peek.doubleCtrl",
    "icons.tintStrength",
    "icons.chameleon",
    "icons.shortcutArrows",
    "tools.enabled",
    "tools.clockSeconds",
    "tools.dialogJump",
    "tools.everything",
    "rules.keepUpdated",
    "rules.unmatched",
];
/// Settings that hold a text or a colour of the user's: a report only says `"set"` or
/// `"unset"`.
const SET_OR_UNSET: &[&str] = &[
    "customTheme",
    "peek.customHotkey",
    "windowsDesktopHotkey",
    "icons.tintRgb",
];

/// The settings as a report carries them, under their paths (see [`REPORTED_SETTINGS`]).
pub fn settings_report(settings: &Settings, rules: &RuleSet) -> Map<String, Value> {
    let mut all = json!(settings);
    all["rules"] = json!({ "keepUpdated": rules.keep_updated, "unmatched": rules.unmatched });
    // An `f32` reads as 0.6000000238418579.
    let strength = (f64::from(settings.icons.tint_strength) * 100.0).round() / 100.0;
    all["icons"]["tintStrength"] = json!(strength);
    let at = |path: &str| path.split('.').fold(&all, |value, part| &value[part]);
    let plain = REPORTED_SETTINGS
        .iter()
        .map(|path| (path.to_string(), at(path).clone()));
    let text = SET_OR_UNSET.iter().map(|path| {
        let set = if at(path).is_null() { "unset" } else { "set" };
        (path.to_string(), json!(set))
    });
    plain.chain(text).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the tests' clock shows.
    static NOW: Mutex<(String, u32)> = Mutex::new((String::new(), 0));

    fn at(day: &str, second: u32) {
        *NOW.lock().unwrap() = (day.to_string(), second);
    }

    fn clock() -> (String, u32) {
        NOW.lock().unwrap().clone()
    }

    /// The reports as they are kept now.
    fn days() -> Vec<Report> {
        recorder().as_ref().unwrap().days.clone()
    }

    fn dates() -> Vec<String> {
        days().into_iter().map(|report| report.day).collect()
    }

    // One test for everything that goes through the process-wide recorder: tests run side by
    // side.
    #[test]
    fn a_report_per_day_from_the_first_start_to_the_switch_off() {
        // Off: nothing is recorded, nothing is there to save or send.
        event("start");
        assert_eq!((today(), snapshot()), (None, None));
        assert!(unsent(true).is_empty());

        // A new installation: a new number, and today's report with the events in order.
        at("2026-11-14", 32_400);
        assert_eq!(start("", clock), "");
        event_with("start", "at_sign_in");
        at("2026-11-14", 32_655);
        event_with("hotkey", "peek");
        event_named("LaunchItem", "");
        event_named("cli", "fences.setBounds");
        event_named("cli", "virtualDesktops.list");
        // A name the form refuses does not get in, whatever it is made of.
        for name in ["C:\\Users\\me", "launch item", "報告", &"x".repeat(61)] {
            event_named(name, "");
            event_named("cli", name);
        }
        update(|report| report.app = "0.3.4".into());
        let first = days().remove(0);
        assert!(first.id.len() == 32 && first.id.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!((first.v, first.day.as_str()), (1, "2026-11-14"));
        assert_eq!(
            json!(first.events),
            json!([
                [32_400, "start", "at_sign_in"],
                [32_655, "hotkey", "peek"],
                [32_655, "launch_item"],
                [32_655, "cli", "fences.set_bounds"],
                [32_655, "cli", "virtual_desktops.list"],
            ])
        );
        // The file is written when something changed, and then once.
        assert_eq!(today(), Some(("2026-11-14".to_string(), true)));
        let text = snapshot().unwrap();
        assert_eq!(serde_json::from_str::<Vec<Report>>(&text).unwrap(), days());
        assert_eq!(today(), Some(("2026-11-14".to_string(), false)));
        assert_eq!(snapshot(), None);

        // The next start on the same day goes on with the same number and report.
        stop();
        assert_eq!(start(&text, clock), "0.3.4");
        assert_eq!(days(), vec![first.clone()]);
        assert!(unsent(false).is_empty());

        // A day keeps MAX_EVENTS events, and the fullest report fits the service's 256 KB.
        for _ in 0..MAX_EVENTS + 10 {
            event_with("settings.set", "roll_up.hide_count_and_chevron");
        }
        assert_eq!(days()[0].events.len(), MAX_EVENTS);
        let full = &unsent(true)[0].1;
        assert!(full.len() < 256 * 1024, "{}", full.len());

        // The local date changes: a new report with the same number. The day before waits to
        // be sent and is forgotten once the service took it; today's stays, sent or not.
        at("2026-11-15", 5);
        event("exit");
        let second = days().remove(1);
        assert_eq!((&second.id, second.day.as_str()), (&first.id, "2026-11-15"));
        assert_eq!(json!(second.events), json!([[5, "exit"]]));
        assert_eq!(
            second.app, "",
            "what a day says beside its events is set anew"
        );
        let waiting = unsent(false);
        assert_eq!(waiting.len(), 1);
        assert_eq!(waiting[0].0, "2026-11-14");
        assert_eq!(
            serde_json::from_str::<Report>(&waiting[0].1).unwrap(),
            days()[0]
        );
        assert_eq!(unsent(true).len(), 2);
        sent("2026-11-15");
        assert_eq!(dates(), ["2026-11-14", "2026-11-15"]);
        sent("2026-11-14");
        assert_eq!(dates(), ["2026-11-15"]);

        // Days nobody took: the oldest go, MAX_UNSENT_DAYS stay beside today's.
        for day in 16..=24 {
            at(&format!("2026-11-{day}"), 0);
            event("start");
        }
        let kept = dates();
        assert_eq!(kept.len(), MAX_UNSENT_DAYS + 1);
        assert_eq!(
            (kept[0].as_str(), kept[7].as_str()),
            ("2026-11-17", "2026-11-24")
        );

        // Switched off, then on again without a usable file: a new number.
        stop();
        event("start");
        assert_eq!(today(), None);
        assert_eq!(start("not a report", clock), "");
        assert!(days()[0].id != first.id && days()[0].events.is_empty());
        stop();
    }

    #[test]
    fn settings_go_out_as_switches_choices_and_numbers() {
        // Every text a user can put into a setting.
        let text = "MARKER C:\\Users\\me";
        let mut settings = Settings {
            custom_theme: Some(text.into()),
            windows_desktop_hotkey: Some(text.into()),
            desktop_path: Some(text.into()),
            ..Settings::default()
        };
        settings.quick_hide.wallpaper_engine_class_whitelist = vec![text.into()];
        settings.peek.custom_hotkey = Some(text.into());
        settings.icons.tint_rgb = Some([1, 2, 3]);
        let report = settings_report(&settings, &RuleSet::default());
        assert!(!json!(report).to_string().contains("MARKER"), "{report:?}");
        assert_eq!(report["language"], "system");
        assert_eq!(report["iconSize"], 48);
        assert_eq!(report["hideRealIcons"], true);
        assert_eq!(report["icons.tintStrength"], 0.6);
        assert_eq!(report["rules.unmatched"], "defaultTarget");
        let plain = settings_report(&Settings::default(), &RuleSet::default());
        for path in SET_OR_UNSET {
            assert_eq!(
                (&report[*path], &plain[*path]),
                (&json!("set"), &json!("unset"))
            );
        }
        // Every path names a setting (a wrong one reads as null), in a form `settings.set`
        // can carry.
        for (path, value) in &plain {
            assert!(!value.is_null() && valid(&snake(path)), "{path}");
        }
    }
}
