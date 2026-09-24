//! Config persistence: atomic writes, `.bak` rotation, daily backups, load fallbacks (plan §7.5).

use crate::model::{Config, SCHEMA_VERSION};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const MAX_FENCES: usize = 64;
pub const MAX_ITEMS: usize = 5000;
pub const DAILY_BACKUPS_KEPT: usize = 7;

#[derive(Debug)]
pub enum LoadOutcome {
    /// Loaded from the primary file.
    Primary(Config),
    /// Primary missing/corrupt; loaded from `.bak` or a daily backup (path given).
    Recovered(Config, PathBuf),
    /// Nothing usable: fresh default (first run or total loss).
    Fresh(Config),
}

impl LoadOutcome {
    pub fn into_config(self) -> Config {
        match self {
            LoadOutcome::Primary(c) | LoadOutcome::Recovered(c, _) | LoadOutcome::Fresh(c) => c,
        }
    }
}

pub struct ConfigStore {
    dir: PathBuf,
}

impl ConfigStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// Reuse an existing installation's data in place. No copying, rewriting or
    /// renaming of user files is needed just because the product name changed.
    pub fn with_legacy(preferred: impl Into<PathBuf>, legacy: impl Into<PathBuf>) -> Self {
        let preferred = Self::new(preferred);
        let legacy = Self::new(legacy);
        if !preferred.has_saved_data() && legacy.has_saved_data() {
            legacy
        } else {
            preferred
        }
    }

    fn has_saved_data(&self) -> bool {
        self.primary_path().exists() || self.bak_path().exists() || !self.list_backups().is_empty()
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
    pub fn primary_path(&self) -> PathBuf {
        self.dir.join("config.json")
    }
    fn bak_path(&self) -> PathBuf {
        self.dir.join("config.bak")
    }
    fn tmp_path(&self) -> PathBuf {
        self.dir.join("config.json.tmp")
    }
    fn backups_dir(&self) -> PathBuf {
        self.dir.join("backups")
    }

    fn parse(path: &Path) -> Option<Config> {
        Self::parse_file(path).ok()
    }

    /// Reads and validates any config file (import / backup restore), with the reason on error.
    pub fn parse_file(path: &Path) -> Result<Config, String> {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let cfg: Config = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        validate(&cfg)?;
        Ok(migrate(cfg))
    }

    /// Writes the config as pretty JSON to an arbitrary path (export).
    pub fn export_to(cfg: &Config, path: &Path) -> io::Result<()> {
        fs::write(path, Self::to_json(cfg)?)
    }

    /// The file text: pretty JSON with the `$schema` field pointing at the published schema
    /// (serde_json sorts keys, and `$` sorts first, so it is the first line after the brace).
    fn to_json(cfg: &Config) -> io::Result<String> {
        let mut cfg = cfg.clone();
        cfg.schema = Some(crate::model::CONFIG_SCHEMA_URL.to_string());
        serde_json::to_string_pretty(&cfg).map_err(io::Error::other)
    }

    /// Loads with fallbacks: primary → .bak → newest daily backup → default.
    pub fn load(&self) -> LoadOutcome {
        if let Some(c) = Self::parse(&self.primary_path()) {
            return LoadOutcome::Primary(c);
        }
        if let Some(c) = Self::parse(&self.bak_path()) {
            return LoadOutcome::Recovered(c, self.bak_path());
        }
        let mut backups = self.list_backups();
        backups.sort();
        for p in backups.into_iter().rev() {
            if let Some(c) = Self::parse(&p) {
                return LoadOutcome::Recovered(c, p);
            }
        }
        LoadOutcome::Fresh(Config::default())
    }

    /// Daily backups (`backups/YYYY-MM-DD.json`), unsorted.
    pub fn list_backups(&self) -> Vec<PathBuf> {
        fs::read_dir(self.backups_dir())
            .map(|rd| {
                rd.filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| p.extension().is_some_and(|e| e == "json"))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Atomically writes the config: tmp → fsync → rotate old to .bak → rename tmp over primary.
    pub fn save(&self, cfg: &Config) -> io::Result<()> {
        validate(cfg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::create_dir_all(&self.dir)?;
        let json = Self::to_json(cfg)?;
        {
            let mut f = fs::File::create(self.tmp_path())?;
            io::Write::write_all(&mut f, json.as_bytes())?;
            f.sync_all()?;
        }
        let primary = self.primary_path();
        if primary.exists() {
            // Best effort: keep the previous good file as .bak.
            let _ = fs::remove_file(self.bak_path());
            let _ = fs::rename(&primary, self.bak_path());
        }
        fs::rename(self.tmp_path(), &primary)?;
        self.daily_backup(&json)?;
        Ok(())
    }

    /// Writes at most one backup per day and prunes to `DAILY_BACKUPS_KEPT`.
    fn daily_backup(&self, json: &str) -> io::Result<()> {
        let dir = self.backups_dir();
        fs::create_dir_all(&dir)?;
        let today = today_yyyy_mm_dd();
        let path = dir.join(format!("{today}.json"));
        if !path.exists() {
            fs::write(&path, json)?;
        }
        let mut backups = self.list_backups();
        backups.sort();
        while backups.len() > DAILY_BACKUPS_KEPT {
            let oldest = backups.remove(0);
            let _ = fs::remove_file(oldest);
        }
        Ok(())
    }
}

/// Range checks from plan §7.5.
pub fn validate(cfg: &Config) -> Result<(), String> {
    if cfg.schema_version == 0 || cfg.schema_version > SCHEMA_VERSION {
        return Err(format!("unsupported schemaVersion {}", cfg.schema_version));
    }
    if cfg.items.len() > MAX_ITEMS {
        return Err(format!("too many items: {}", cfg.items.len()));
    }
    for layout in &cfg.layouts {
        if layout.fences.len() > MAX_FENCES {
            return Err(format!("too many fences: {}", layout.fences.len()));
        }
        let inboxes = layout
            .fences
            .iter()
            .filter(|f| f.kind == crate::model::FenceKind::Inbox)
            .count();
        if inboxes > 1 {
            return Err(format!("layout has {inboxes} inbox fences"));
        }
        for f in &layout.fences {
            let g = &f.geometry;
            if !(g.w.is_finite() && g.h.is_finite() && g.x.is_finite() && g.y.is_finite()) {
                return Err(format!("fence {} has non-finite geometry", f.title));
            }
            if g.w < 1.0 || g.h < 1.0 || g.w > 8192.0 || g.h > 8192.0 {
                return Err(format!(
                    "fence {} has out-of-range size {}x{}",
                    f.title, g.w, g.h
                ));
            }
            // 0.4–1.8: the per-fence menu offers 0.55 ("更透明") and 1.6 ("更厚实" = a veil).
            if let Some(a) = &f.appearance
                && let Some(o) = a.opacity
                && !(0.2..=2.0).contains(&o)
            {
                return Err(format!("fence {} opacity {o} out of range", f.title));
            }
        }
    }
    Ok(())
}

/// Severity of a [`lint`] finding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LintLevel {
    /// The app would refuse or silently drop this.
    Error,
    /// Loads, but something will not behave as the file suggests.
    Warning,
}

/// One finding of [`lint`].
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lint {
    pub level: LintLevel,
    pub message: String,
    /// What the finding is about (`rule:<name>`, `fence:<title>`, `layout:<n>`, `file`).
    pub subject: String,
}

fn lint_at(out: &mut Vec<Lint>, level: LintLevel, subject: impl Into<String>, message: String) {
    out.push(Lint {
        level,
        message,
        subject: subject.into(),
    });
}

/// Cross-checks a configuration that already passed [`validate`] for things that load but will
/// not work as written: rules whose target fence exists in no layout, portals whose folder is
/// gone, tabs hosted by a missing fence, duplicate fence ids, memberships of unknown items, more
/// snapshots than the app keeps, a missing or foreign `$schema`. Pure apart from the folder
/// existence checks; safe to run on an exported file without the app.
pub fn lint(cfg: &Config) -> Vec<Lint> {
    use crate::model::FenceKind;
    use crate::rules::Target;
    let mut out = Vec::new();
    match cfg.schema.as_deref() {
        None => lint_at(
            &mut out,
            LintLevel::Warning,
            "file",
            format!(
                "no \"$schema\" field; add {:?} for editor validation",
                crate::model::CONFIG_SCHEMA_URL
            ),
        ),
        Some(url) if url != crate::model::CONFIG_SCHEMA_URL => lint_at(
            &mut out,
            LintLevel::Warning,
            "file",
            format!(
                "\"$schema\" is {url:?}; PecoFence publishes {:?}",
                crate::model::CONFIG_SCHEMA_URL
            ),
        ),
        Some(_) => {}
    }
    if cfg.layouts.is_empty() {
        lint_at(
            &mut out,
            LintLevel::Warning,
            "file",
            "no layouts: the app will create a default one on load".into(),
        );
    }
    let mut all_fences: Vec<(&crate::model::Fence, usize)> = Vec::new();
    for (li, layout) in cfg.layouts.iter().enumerate() {
        let subject = format!("layout:{li}");
        if layout.fences.iter().all(|f| f.kind != FenceKind::Inbox) {
            lint_at(
                &mut out,
                LintLevel::Warning,
                &subject,
                "no inbox fence; the app adds one on load".into(),
            );
        }
        let mut seen = std::collections::HashSet::new();
        for f in &layout.fences {
            if !seen.insert(f.id) {
                lint_at(
                    &mut out,
                    LintLevel::Error,
                    format!("fence:{}", f.title),
                    format!("fence id {} appears twice in layout {li}", f.id),
                );
            }
            all_fences.push((f, li));
        }
        for f in &layout.fences {
            let subject = format!("fence:{}", f.title);
            if let Some(host) = f.tab_host {
                match layout.fences.iter().find(|h| h.id == host) {
                    None => lint_at(
                        &mut out,
                        LintLevel::Warning,
                        &subject,
                        format!(
                            "hosted by tab host {host}, which is not in layout {li}; shown as its own window"
                        ),
                    ),
                    Some(h) if h.tab_host.is_some() => lint_at(
                        &mut out,
                        LintLevel::Warning,
                        &subject,
                        format!("tab host {:?} is itself a tab", h.title),
                    ),
                    Some(_) => {}
                }
            }
            if let crate::model::ItemSourceSpec::Folder { path, .. } = &f.source
                && !Path::new(path).is_dir()
            {
                lint_at(
                    &mut out,
                    LintLevel::Warning,
                    &subject,
                    format!(
                        "portal folder {path:?} does not exist (or is not reachable from here)"
                    ),
                );
            }
            if f.kind == FenceKind::FolderPortal
                && !matches!(f.source, crate::model::ItemSourceSpec::Folder { .. })
            {
                lint_at(
                    &mut out,
                    LintLevel::Error,
                    &subject,
                    "portal fence without a folder source".into(),
                );
            }
            let unknown = f
                .items
                .iter()
                .filter(|r| !cfg.items.contains_key(&r.item_id))
                .count();
            if unknown > 0 {
                lint_at(
                    &mut out,
                    LintLevel::Warning,
                    &subject,
                    format!(
                        "{unknown} membership entr{} refer to items missing from \"items\"; they are ignored",
                        if unknown == 1 { "y" } else { "ies" }
                    ),
                );
            }
        }
    }
    for rule in &cfg.rules.list {
        if let Target::Fence(id) = rule.target {
            match all_fences.iter().find(|(f, _)| f.id == id) {
                None => lint_at(
                    &mut out,
                    LintLevel::Warning,
                    format!("rule:{}", rule.name),
                    format!(
                        "target fence {id} exists in no layout; matches fall back to the inbox"
                    ),
                ),
                Some((f, _)) if f.kind == FenceKind::FolderPortal => lint_at(
                    &mut out,
                    LintLevel::Error,
                    format!("rule:{}", rule.name),
                    format!(
                        "target {:?} is a folder portal; rules can only target virtual fences or the inbox",
                        f.title
                    ),
                ),
                Some(_) => {}
            }
        }
        if rule.all_of.is_empty() {
            lint_at(
                &mut out,
                LintLevel::Warning,
                format!("rule:{}", rule.name),
                "no conditions: the rule never matches".into(),
            );
        }
    }
    if let Target::Fence(id) = cfg.rules.default_target
        && !all_fences.iter().any(|(f, _)| f.id == id)
    {
        lint_at(
            &mut out,
            LintLevel::Warning,
            "rules",
            format!(
                "defaultTarget fence {id} exists in no layout; unmatched items go to the inbox"
            ),
        );
    }
    if cfg.snapshots.len() > crate::model::MAX_SNAPSHOTS {
        lint_at(
            &mut out,
            LintLevel::Warning,
            "snapshots",
            format!(
                "{} snapshots; the app keeps at most {}",
                cfg.snapshots.len(),
                crate::model::MAX_SNAPSHOTS
            ),
        );
    }
    out
}

/// Schema migration chain (currently identity).
pub fn migrate(mut cfg: Config) -> Config {
    if cfg.schema_version < SCHEMA_VERSION {
        cfg.schema_version = SCHEMA_VERSION;
    }
    cfg
}

/// Local-date string without pulling in a date crate (civil-from-days algorithm, UTC-based;
/// backups are named for humans so the timezone offset is irrelevant).
pub fn today_yyyy_mm_dd() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn tmpdir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("pecofence-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    fn sample() -> Config {
        let mut c = Config::default();
        c.layouts.push(Layout {
            fingerprint: vec![],
            fences: vec![Fence::new(
                "A",
                FenceKind::Inbox,
                NormGeometry {
                    monitor: "m".into(),
                    x: 1.0,
                    y: 2.0,
                    w: 300.0,
                    h: 200.0,
                    work_w: 1920.0,
                    work_h: 1000.0,
                    anchor: Anchor::LeftTop,
                },
            )],
        });
        c
    }

    #[test]
    fn save_then_load_primary() {
        let store = ConfigStore::new(tmpdir("primary"));
        store.save(&sample()).unwrap();
        assert!(matches!(store.load(), LoadOutcome::Primary(_)));
        assert!(store.primary_path().exists());
        assert!(!store.tmp_path().exists());
        assert_eq!(store.list_backups().len(), 1);
    }

    #[test]
    fn corrupt_primary_falls_back_to_bak() {
        let store = ConfigStore::new(tmpdir("bak"));
        store.save(&sample()).unwrap();
        store.save(&sample()).unwrap(); // creates .bak
        fs::write(store.primary_path(), "{ not json").unwrap();
        match store.load() {
            LoadOutcome::Recovered(c, from) => {
                assert_eq!(c.layouts[0].fences[0].title, "A");
                assert_eq!(from, store.bak_path());
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn nothing_usable_gives_fresh() {
        let store = ConfigStore::new(tmpdir("fresh"));
        assert!(matches!(store.load(), LoadOutcome::Fresh(_)));
    }

    #[test]
    fn renamed_product_reuses_old_configuration_without_changing_it() {
        let parent = tmpdir("rebrand");
        let old = ConfigStore::new(parent.join(crate::brand::LEGACY_DATA_DIR));
        let mut config = sample();
        config.layouts[0].fences[0].title = "My OpenFence files {1}".into();
        config.settings.language = crate::i18n::Language::Japanese;
        old.save(&config).unwrap();
        let original = fs::read(old.primary_path()).unwrap();
        let preferred = parent.join(crate::brand::NAME);
        let selected = ConfigStore::with_legacy(&preferred, old.dir());
        assert_eq!(selected.dir(), old.dir());
        let mut loaded = selected.load().into_config();
        // Saving stamps the published schema URL; everything else is untouched.
        assert_eq!(
            loaded.schema.as_deref(),
            Some(crate::model::CONFIG_SCHEMA_URL)
        );
        loaded.schema = None;
        assert_eq!(
            serde_json::to_value(loaded).unwrap(),
            serde_json::to_value(config).unwrap()
        );
        assert_eq!(fs::read(old.primary_path()).unwrap(), original);
        assert!(!preferred.exists());
    }

    #[test]
    fn current_product_data_wins_and_old_backup_only_installs_still_load() {
        let parent = tmpdir("rebrand-precedence");
        let old = ConfigStore::new(parent.join(crate::brand::LEGACY_DATA_DIR));
        old.save(&sample()).unwrap();
        old.save(&sample()).unwrap();
        fs::remove_file(old.primary_path()).unwrap();
        let preferred = ConfigStore::new(parent.join(crate::brand::NAME));
        let selected = ConfigStore::with_legacy(preferred.dir(), old.dir());
        assert!(matches!(selected.load(), LoadOutcome::Recovered(..)));
        preferred.save(&sample()).unwrap();
        // Even a damaged new config belongs to the new installation. Its normal
        // backup recovery must not silently switch to a stale legacy layout.
        fs::write(preferred.primary_path(), "{ bad json").unwrap();
        assert_eq!(
            ConfigStore::with_legacy(preferred.dir(), old.dir()).dir(),
            preferred.dir()
        );
        assert_eq!(
            ConfigStore::with_legacy(parent.join("fresh"), parent.join("missing")).dir(),
            parent.join("fresh")
        );
    }

    #[test]
    fn validation_rejects_two_inboxes() {
        let mut c = sample();
        let f = c.layouts[0].fences[0].clone();
        c.layouts[0].fences.push(Fence {
            id: uuid::Uuid::new_v4(),
            ..f
        });
        assert!(validate(&c).is_err());
    }

    #[test]
    fn saved_file_starts_with_the_schema_field_and_loads_back() {
        let store = ConfigStore::new(tmpdir("schema"));
        store.save(&sample()).unwrap();
        let text = fs::read_to_string(store.primary_path()).unwrap();
        let second_line = text.lines().nth(1).unwrap_or_default();
        assert!(second_line.contains("\"$schema\""), "{second_line}");
        assert!(text.contains(crate::model::CONFIG_SCHEMA_URL));
        let cfg = ConfigStore::parse_file(&store.primary_path()).unwrap();
        assert_eq!(cfg.schema.as_deref(), Some(crate::model::CONFIG_SCHEMA_URL));
        // A file without the field (older versions) still loads.
        let stripped = text.replacen(
            &format!("\"$schema\": \"{}\",", crate::model::CONFIG_SCHEMA_URL),
            "",
            1,
        );
        assert!(!stripped.contains("$schema"));
        let cfg: Config = serde_json::from_str(&stripped).unwrap();
        assert_eq!(cfg.schema, None);
    }

    #[test]
    fn lint_finds_dangling_targets_and_folders() {
        use crate::rules::{Cond, Rule, Target, TypeCategory};
        let mut c = sample();
        let inbox = c.layouts[0].fences[0].id;
        // Valid rule to the inbox, a rule to a fence that exists nowhere, a rule without
        // conditions, a portal whose folder is gone, a tab hosted by a missing fence.
        c.rules.list.push(Rule::new(
            "ok",
            Target::Fence(inbox),
            vec![Cond::Type(vec![TypeCategory::Images])],
        ));
        let ghost = uuid::Uuid::new_v4();
        c.rules.list.push(Rule::new(
            "ghost",
            Target::Fence(ghost),
            vec![Cond::Type(vec![TypeCategory::Video])],
        ));
        c.rules.list.push(Rule::new("empty", Target::Inbox, vec![]));
        let geometry = c.layouts[0].fences[0].geometry.clone();
        let mut portal = Fence::new("Gone", FenceKind::FolderPortal, geometry.clone());
        portal.source = ItemSourceSpec::Folder {
            path: "Z:\\definitely\\missing\\folder".into(),
            recursive: false,
            filter: None,
        };
        let portal_id = portal.id;
        c.layouts[0].fences.push(portal);
        let mut tab = Fence::new("Tab", FenceKind::Virtual, geometry);
        tab.tab_host = Some(ghost);
        c.layouts[0].fences.push(tab);
        c.rules.list.push(Rule::new(
            "to-portal",
            Target::Fence(portal_id),
            vec![Cond::FoldersOnly],
        ));
        assert!(validate(&c).is_ok(), "lint covers what validate does not");

        let lints = lint(&c);
        let subjects: Vec<&str> = lints.iter().map(|l| l.subject.as_str()).collect();
        assert!(subjects.contains(&"file"), "$schema missing: {lints:?}");
        assert!(subjects.contains(&"rule:ghost"), "{lints:?}");
        assert!(subjects.contains(&"rule:empty"), "{lints:?}");
        assert!(subjects.contains(&"fence:Gone"), "{lints:?}");
        assert!(subjects.contains(&"fence:Tab"), "{lints:?}");
        assert!(!subjects.contains(&"rule:ok"), "{lints:?}");
        let errors: Vec<&Lint> = lints
            .iter()
            .filter(|l| l.level == LintLevel::Error)
            .collect();
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].subject, "rule:to-portal");

        // A clean, saved-and-reloaded config lints without findings.
        let store = ConfigStore::new(tmpdir("lint-clean"));
        let mut clean = sample();
        let clean_inbox = clean.layouts[0].fences[0].id;
        clean.rules.list.push(Rule::new(
            "ok",
            Target::Fence(clean_inbox),
            vec![Cond::Type(vec![TypeCategory::Images])],
        ));
        store.save(&clean).unwrap();
        let reloaded = ConfigStore::parse_file(&store.primary_path()).unwrap();
        assert_eq!(lint(&reloaded), Vec::<Lint>::new());
    }

    #[test]
    fn date_formatting_is_sane() {
        let s = today_yyyy_mm_dd();
        assert_eq!(s.len(), 10);
        assert!(s.starts_with("20"));
    }
}
