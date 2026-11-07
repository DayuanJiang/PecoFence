//! Smart fences (智能栅栏): a fence that shows the live result of a saved search instead of
//! desktop items. The search is a folder (or every drive), the conditions of the sorting rules
//! and a time window; with Everything running it becomes an Everything search, without it the
//! app reads the folder and tests the same conditions itself (`matches`). The files are never
//! moved: the fence lists them where they are.

use crate::rules::{self, Cond, ItemFacts, StrOp, TypeCategory};
use serde::{Deserialize, Serialize};

/// What a smart fence searches. Saved on the fence (`Fence::smart`); versions without smart
/// fences ignore the field and show an empty fence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SmartSpec {
    /// The template it was made from ([`Template::key`]); None for one written by hand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// The folder searched; None = every drive (only with Everything).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    /// Subfolders of `folder` too.
    #[serde(default = "default_true")]
    pub subfolders: bool,
    /// Every condition must hold (the conditions of the sorting rules).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub all_of: Vec<Cond>,
    /// Only what was created or written in the last this many days (the later of the two: a
    /// download keeps the server's date as its write time with some download tools).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within_days: Option<u32>,
    /// Only names that match this text, as the Peek search and 筛选 match names (every word,
    /// pinyin too, not the loosest letters-in-order match). Tested by the app on every file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// An Everything search typed by the user, sent with the folder, the conditions and the
    /// time window, which all still apply (as does the name). Needs Everything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub everything: Option<String>,
    /// Most results shown, newest first.
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_true() -> bool {
    true
}

fn default_limit() -> u32 {
    SmartSpec::DEFAULT_LIMIT
}

impl SmartSpec {
    pub const DEFAULT_LIMIT: u32 = 200;
    /// Most entries the folder reading without Everything looks at.
    pub const MAX_SCANNED: usize = 20_000;
    /// Results asked of Everything when the app filters names itself.
    pub const NAME_FILTER_ASK: u32 = 5_000;

    /// Whether the search can run without Everything: it has a folder to read and no
    /// Everything-only text.
    pub fn works_without_everything(&self) -> bool {
        self.folder.is_some() && self.everything.is_none()
    }

    /// Whether a condition looks at shortcut targets (the target condition, the program type),
    /// which costs a COM round trip per shortcut to read.
    pub fn needs_shortcut_targets(&self) -> bool {
        self.all_of.iter().any(|c| match c {
            Cond::ShortcutTarget { .. } => true,
            Cond::Type(cats) => cats.contains(&TypeCategory::Programs),
            _ => false,
        })
    }

    /// The Everything search text: the folder, the user's own Everything text, the conditions
    /// Everything can test, the time window (`since_date`, see [`everything_date`]) and, over
    /// every drive, the system folders left out. The app tests every condition again on the
    /// answer (`matches`), so a condition left out here only costs results.
    pub fn everything_query(&self, since_date: Option<&str>) -> String {
        let mut terms: Vec<String> = Vec::new();
        match &self.folder {
            Some(folder) => {
                let folder = folder.trim_end_matches('\\');
                if self.subfolders {
                    terms.push(quote(&format!("{folder}\\")));
                } else {
                    terms.push(format!("parent:{}", quote(folder)));
                }
            }
            None => {
                // Every drive: not Windows itself, program data or the Recycle Bin.
                terms.extend(
                    [
                        "!\"C:\\Windows\\\"",
                        "!\\AppData\\",
                        "!\"\\$Recycle.Bin\\\"",
                    ]
                    .map(str::to_string),
                );
            }
        }
        if let Some(raw) = self.everything.as_deref().map(str::trim)
            && !raw.is_empty()
        {
            terms.push(raw.to_string());
        }
        for cond in &self.all_of {
            if let Some(t) = everything_term(cond) {
                terms.push(t);
            }
        }
        if let (Some(_), Some(date)) = (self.within_days, since_date) {
            terms.push(format!("<dm:>={date}|dc:>={date}>"));
        }
        terms.join(" ")
    }

    /// Whether one file found (by Everything or in the folder) belongs in the fence: every
    /// condition holds and it was created or written inside the time window. `newest` (the
    /// later of creation and last write) and `now` are Unix seconds.
    pub fn matches(&self, facts: &ItemFacts, newest: i64, now: i64) -> bool {
        if rules::is_temporary(&facts.file_name, facts.age_secs) {
            return false;
        }
        if let Some(days) = self.within_days
            && newest < now - i64::from(days) * 86_400
        {
            return false;
        }
        if let Some(q) = self.name.as_deref().map(|q| q.trim().to_lowercase())
            && !q.is_empty()
            && !crate::matcher::match_name(&facts.file_name, &q)
                .is_some_and(|m| !crate::matcher::is_loose_score(m.score))
        {
            return false;
        }
        self.all_of.iter().all(|c| rules::cond_matches(c, facts))
    }

    /// How many results to ask Everything for: the name filter and the conditions Everything
    /// cannot test are tested here on the answer, so it takes more than the fence shows.
    pub fn everything_max(&self) -> u32 {
        if self.name.is_some() || self.all_of.iter().any(|c| everything_term(c).is_none()) {
            self.limit.max(Self::NAME_FILTER_ASK)
        } else {
            self.limit
        }
    }
}

/// Quotes an Everything term; a double quote inside becomes the `&quot:` entity.
fn quote(text: &str) -> String {
    format!("\"{}\"", text.replace('"', "&quot:"))
}

/// The Everything form of a rule condition, when it has one: the same files or more (the app
/// tests every condition again on the answer), never fewer.
fn everything_term(cond: &Cond) -> Option<String> {
    Some(match cond {
        Cond::Ext(list) => {
            let list: Vec<&str> = list.iter().map(String::as_str).collect();
            exts_term(&list)?
        }
        Cond::Type(cats) => {
            let mut folders = false;
            let mut all: Vec<&str> = Vec::new();
            for cat in cats {
                match cat {
                    TypeCategory::Folders => folders = true,
                    // A program or a shortcut is told by more than its extension (a shortcut's
                    // target, a setup's name): these extensions take every one of them in.
                    TypeCategory::Programs => {
                        all.extend([".exe", ".msi", ".bat", ".cmd", ".appref-ms", ".lnk"])
                    }
                    TypeCategory::Installers => all.extend([
                        ".msi",
                        ".msix",
                        ".msixbundle",
                        ".appx",
                        ".appxbundle",
                        ".appinstaller",
                        ".exe",
                    ]),
                    TypeCategory::Shortcuts => all.extend([".lnk", ".url", ".website"]),
                    c => all.extend_from_slice(rules::category_exts(*c)),
                }
            }
            match (folders, exts_term(&all)) {
                (true, Some(e)) => format!("<folder:|{e}>"),
                (true, None) => "folder:".into(),
                (false, Some(e)) => e,
                (false, None) => return None,
            }
        }
        Cond::Name { op, value } => match op {
            StrOp::Contains => quote(value),
            StrOp::NotContains => format!("!{}", quote(value)),
            StrOp::StartsWith => format!("startwith:{}", quote(value)),
            StrOp::EndsWith => format!("endwith:{}", quote(value)),
            StrOp::Is => format!("wfn:{}", quote(value)),
        },
        Cond::Glob(g) => format!("wfn:{}", quote(g)),
        Cond::ExactName(names) => {
            let parts: Vec<String> = names.iter().map(|n| format!("wfn:{}", quote(n))).collect();
            format!("<{}>", parts.join("|"))
        }
        Cond::FilesOnly | Cond::PlainFiles => "file:".into(),
        Cond::FoldersOnly => "folder:".into(),
        Cond::SizeMb { min, max } => {
            let bytes = |mb: f64| (mb * 1024.0 * 1024.0).round() as u64;
            let mut t = Vec::new();
            if let Some(m) = min {
                t.push(format!("size:>={}", bytes(*m)));
            }
            if let Some(m) = max {
                t.push(format!("size:<={}", bytes(*m)));
            }
            t.join(" ")
        }
        Cond::ShortcutTarget { .. }
        | Cond::CreatedTime { .. }
        | Cond::CreatedWeekday(_)
        | Cond::Origin(_)
        | Cond::IdleDays { .. } => return None,
    })
}

/// Extensions as one Everything term: `ext:` for plain ones, a name ending for one with a dot
/// inside (`tar.gz`: Everything's `ext:` sees only `gz`), OR'ed. None for an empty list.
fn exts_term(list: &[&str]) -> Option<String> {
    let mut plain: Vec<String> = Vec::new();
    let mut endings: Vec<String> = Vec::new();
    for e in list {
        let e = e.trim().trim_start_matches('.');
        if e.is_empty() {
            continue;
        }
        if e.contains('.') {
            endings.push(format!("endwith:{}", quote(&format!(".{e}"))));
        } else if !plain.iter().any(|p| p.eq_ignore_ascii_case(e)) {
            plain.push(e.to_string());
        }
    }
    let mut parts = endings;
    if !plain.is_empty() {
        parts.insert(0, format!("ext:{}", plain.join(";")));
    }
    match parts.len() {
        0 => None,
        1 => parts.pop(),
        _ => Some(format!("<{}>", parts.join("|"))),
    }
}

/// A local date as Everything's `dm:>=` takes it (`YYYY-MM-DD`, any Everything version and
/// region). The app passes the local date of `now - within_days`; its own test (`matches`) then
/// drops what is older to the second.
pub fn everything_date(d: crate::CivilDate) -> String {
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

/// The searches offered by 「新建智能栅栏」.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Template {
    /// 最近 7 天下载的文件: the Downloads folder, files written in the last 7 days.
    Downloads,
    /// 最近一周的截图: the Screenshots folder, the last 7 days.
    Screenshots,
    /// 大于 1 GB 的文件: every drive (Everything).
    Large,
    /// 所有 .psd 文件: every drive (Everything).
    Psd,
    /// 最近打开的文件: Windows' Recent folder (shortcuts to what was opened).
    Recent,
}

/// Folders a template searches, found by the app (known folders of the user).
#[derive(Clone, Debug, Default)]
pub struct KnownFolders {
    pub downloads: Option<String>,
    pub screenshots: Option<String>,
    pub recent: Option<String>,
}

impl Template {
    pub const ALL: [Template; 5] = [
        Template::Downloads,
        Template::Screenshots,
        Template::Large,
        Template::Psd,
        Template::Recent,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Template::Downloads => "downloads",
            Template::Screenshots => "screenshots",
            Template::Large => "large",
            Template::Psd => "psd",
            Template::Recent => "recent",
        }
    }

    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.key() == key)
    }

    /// The fence's title (and the menu entry).
    pub fn title(self) -> &'static str {
        match self {
            Template::Downloads => crate::i18n::text("最近 7 天下载"),
            Template::Screenshots => crate::i18n::text("最近一周的截图"),
            Template::Large => crate::i18n::text("大于 1 GB 的文件"),
            Template::Psd => crate::i18n::text("所有 .psd 文件"),
            Template::Recent => crate::i18n::text("最近打开的文件"),
        }
    }

    /// Whether it searches every drive, which only Everything can do.
    pub fn needs_everything(self) -> bool {
        matches!(self, Template::Large | Template::Psd)
    }

    /// The search; None when its folder is not known on this computer.
    pub fn spec(self, folders: &KnownFolders) -> Option<SmartSpec> {
        let mut spec = SmartSpec {
            template: Some(self.key().to_string()),
            folder: None,
            subfolders: true,
            all_of: vec![Cond::FilesOnly],
            within_days: None,
            name: None,
            everything: None,
            limit: SmartSpec::DEFAULT_LIMIT,
        };
        match self {
            Template::Downloads => {
                spec.folder = Some(folders.downloads.clone()?);
                spec.within_days = Some(7);
            }
            Template::Screenshots => {
                spec.folder = Some(folders.screenshots.clone()?);
                spec.within_days = Some(7);
                spec.all_of.push(Cond::Type(vec![TypeCategory::Images]));
            }
            Template::Large => {
                spec.all_of.push(Cond::SizeMb {
                    min: Some(1024.0),
                    max: None,
                });
            }
            Template::Psd => spec.all_of.push(Cond::Ext(vec![".psd".into()])),
            Template::Recent => {
                spec.folder = Some(folders.recent.clone()?);
                spec.subfolders = false;
                spec.all_of.push(Cond::Ext(vec![".lnk".into()]));
                spec.limit = 40;
            }
        }
        Some(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folders() -> KnownFolders {
        KnownFolders {
            downloads: Some(r"C:\Users\pf\Downloads".into()),
            screenshots: Some(r"C:\Users\pf\Pictures\Screenshots".into()),
            recent: Some(r"C:\Users\pf\AppData\Roaming\Microsoft\Windows\Recent".into()),
        }
    }

    #[test]
    fn downloads_template_searches_the_folder_tree_by_date() {
        let spec = Template::Downloads.spec(&folders()).unwrap();
        assert_eq!(
            spec.everything_query(Some("2026-10-31")),
            r#""C:\Users\pf\Downloads\" file: <dm:>=2026-10-31|dc:>=2026-10-31>"#
        );
    }

    #[test]
    fn every_drive_leaves_out_system_folders() {
        let q = Template::Large
            .spec(&folders())
            .unwrap()
            .everything_query(None);
        assert_eq!(
            q,
            r#"!"C:\Windows\" !\AppData\ !"\$Recycle.Bin\" file: size:>=1073741824"#
        );
        let q = Template::Psd
            .spec(&folders())
            .unwrap()
            .everything_query(None);
        assert!(q.ends_with("file: ext:psd"), "{q}");
    }

    #[test]
    fn one_folder_without_subfolders_uses_parent() {
        let q = Template::Recent
            .spec(&folders())
            .unwrap()
            .everything_query(None);
        assert_eq!(
            q,
            r#"parent:"C:\Users\pf\AppData\Roaming\Microsoft\Windows\Recent" file: ext:lnk"#
        );
    }

    #[test]
    fn conditions_become_everything_terms() {
        let t = |c: Cond| everything_term(&c);
        assert_eq!(
            t(Cond::Name {
                op: StrOp::StartsWith,
                value: "IMG_".into()
            })
            .unwrap(),
            r#"startwith:"IMG_""#
        );
        assert_eq!(
            t(Cond::Glob("*.tar.gz".into())).unwrap(),
            r#"wfn:"*.tar.gz""#
        );
        assert_eq!(
            t(Cond::Type(vec![TypeCategory::Music])).unwrap(),
            "ext:mp3;flac;wav;m4a;aac;ogg;wma;opus"
        );
        assert_eq!(t(Cond::IdleDays { min: 30 }), None);
    }

    /// What Everything is asked for takes in every file the rule condition does (the app tests
    /// the condition again on the answer): folders together with file types, shortcuts by
    /// their extensions, extensions with a dot inside by the name's ending.
    #[test]
    fn type_and_extension_terms_take_in_every_match() {
        let t = |c: Cond| everything_term(&c).unwrap();
        assert_eq!(
            t(Cond::Type(vec![
                TypeCategory::Folders,
                TypeCategory::Archives
            ])),
            "<folder:|ext:zip;7z;rar;tar;gz;bz2;xz;iso;cab>"
        );
        assert_eq!(t(Cond::Type(vec![TypeCategory::Folders])), "folder:");
        assert_eq!(
            t(Cond::Type(vec![TypeCategory::Shortcuts])),
            "ext:lnk;url;website"
        );
        assert_eq!(
            t(Cond::Type(vec![TypeCategory::Programs])),
            "ext:exe;msi;bat;cmd;appref-ms;lnk"
        );
        assert_eq!(
            t(Cond::Ext(vec![
                "tar.gz".into(),
                ".pdf".into(),
                "PDF".into()
            ])),
            r#"<ext:pdf|endwith:".tar.gz">"#
        );
        assert_eq!(t(Cond::Ext(vec!["tar.gz".into()])), r#"endwith:".tar.gz""#);
        assert_eq!(everything_term(&Cond::Ext(vec![" ".into()])), None);
        // Every category's own test agrees with the term: a file it matches has an extension
        // the term lists (or is a folder).
        for cat in rules::CLASSIFY_ORDER {
            let term = t(Cond::Type(vec![*cat]));
            for ext in rules::category_exts(*cat) {
                assert!(
                    term.contains(ext.trim_start_matches('.')),
                    "{cat:?} {ext} in {term}"
                );
            }
        }
        // A condition Everything cannot test asks for more results.
        let mut spec = Template::Downloads.spec(&folders()).unwrap();
        assert_eq!(spec.everything_max(), spec.limit);
        spec.all_of.push(Cond::IdleDays { min: 30 });
        assert_eq!(spec.everything_max(), SmartSpec::NAME_FILTER_ASK);
    }

    /// The user's own Everything text narrows the search: the folder, the conditions and the
    /// time window go with it, and the name and conditions are tested on the answer as always.
    #[test]
    fn typed_everything_text_keeps_the_conditions_and_the_window() {
        let mut spec = Template::Downloads.spec(&folders()).unwrap();
        spec.everything = Some("  invoice|receipt ".into());
        spec.all_of.push(Cond::Ext(vec!["pdf".into()]));
        assert_eq!(
            spec.everything_query(Some("2026-10-31")),
            r#""C:\Users\pf\Downloads\" invoice|receipt file: ext:pdf <dm:>=2026-10-31|dc:>=2026-10-31>"#
        );
        assert!(!spec.works_without_everything());
        spec.name = Some("2026".into());
        let now = 1_800_000_000;
        let f = |n: &str| ItemFacts {
            file_name: n.into(),
            ..Default::default()
        };
        assert!(spec.matches(&f("invoice 2026.pdf"), now, now));
        assert!(!spec.matches(&f("invoice 2025.pdf"), now, now), "name");
        assert!(
            !spec.matches(&f("invoice 2026.docx"), now, now),
            "condition"
        );
        assert!(
            !spec.matches(&f("invoice 2026.pdf"), now - 8 * 86_400, now),
            "time window"
        );
    }

    #[test]
    fn the_window_and_conditions_decide_a_match() {
        let spec = Template::Screenshots.spec(&folders()).unwrap();
        let now = 1_800_000_000;
        let png = ItemFacts {
            file_name: "Screenshot 2026-11-07.png".into(),
            ..Default::default()
        };
        assert!(spec.matches(&png, now - 3 * 86_400, now));
        assert!(!spec.matches(&png, now - 8 * 86_400, now));
        let txt = ItemFacts {
            file_name: "notes.txt".into(),
            ..Default::default()
        };
        assert!(!spec.matches(&txt, now, now));
        let partial = ItemFacts {
            file_name: "setup.exe.crdownload".into(),
            ..Default::default()
        };
        assert!(
            !Template::Downloads
                .spec(&folders())
                .unwrap()
                .matches(&partial, now, now)
        );
    }

    #[test]
    fn old_configs_and_minimal_json_read() {
        let spec: SmartSpec = serde_json::from_str(r#"{"folder":"D:\\x"}"#).unwrap();
        assert!(spec.subfolders);
        assert_eq!(spec.limit, SmartSpec::DEFAULT_LIMIT);
        assert!(spec.works_without_everything());
        let back: SmartSpec = serde_json::from_value(serde_json::to_value(&spec).unwrap()).unwrap();
        assert_eq!(back, spec);
    }

    #[test]
    fn the_name_filter_matches_like_the_peek_search() {
        let mut spec = Template::Downloads.spec(&folders()).unwrap();
        spec.name = Some("bg".into());
        let now = 1_800_000_000;
        let f = |n: &str| ItemFacts {
            file_name: n.into(),
            ..Default::default()
        };
        assert!(
            spec.matches(&f("季度报告.pdf"), now, now),
            "pinyin initials"
        );
        assert!(!spec.matches(&f("旧合同.pdf"), now, now));
        spec.name = Some("pdf".into());
        assert!(spec.matches(&f("旧合同.pdf"), now, now));
        assert!(
            !spec.matches(&f("photo_d_final.png"), now, now),
            "loose match left out"
        );
        assert_eq!(spec.everything_max(), SmartSpec::NAME_FILTER_ASK);
    }

    #[test]
    fn dates_are_iso() {
        assert_eq!(
            everything_date(crate::CivilDate::new(2026, 3, 5)),
            "2026-03-05"
        );
    }
}
