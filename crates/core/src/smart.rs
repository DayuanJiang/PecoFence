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
    /// Only what was written in the last this many days.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within_days: Option<u32>,
    /// An Everything search typed by the user, sent as it is in place of the conditions (with
    /// the folder still in front). Needs Everything.
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

    /// The Everything search text: the folder, the conditions Everything can test, the time
    /// window (`since_date`, see [`everything_date`]) and, over every drive, the system folders
    /// left out. The app tests every condition
    /// again on the answer (`matches`), so a condition left out here only costs results.
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
            return terms.join(" ");
        }
        for cond in &self.all_of {
            if let Some(t) = everything_term(cond) {
                terms.push(t);
            }
        }
        if let (Some(_), Some(date)) = (self.within_days, since_date) {
            terms.push(format!("dm:>={date}"));
        }
        terms.join(" ")
    }

    /// Whether one file found (by Everything or in the folder) belongs in the fence: every
    /// condition holds and it was written inside the time window. `mtime` and `now` are Unix
    /// seconds.
    pub fn matches(&self, facts: &ItemFacts, mtime: i64, now: i64) -> bool {
        if rules::is_temporary(&facts.file_name, facts.age_secs) {
            return false;
        }
        if let Some(days) = self.within_days
            && mtime < now - i64::from(days) * 86_400
        {
            return false;
        }
        self.all_of.iter().all(|c| rules::cond_matches(c, facts))
    }
}

/// Quotes an Everything term; a double quote inside becomes the `&quot:` entity.
fn quote(text: &str) -> String {
    format!("\"{}\"", text.replace('"', "&quot:"))
}

/// The Everything form of a rule condition, when it has one.
fn everything_term(cond: &Cond) -> Option<String> {
    let exts = |list: &[&str]| {
        let list: Vec<&str> = list.iter().map(|e| e.trim_start_matches('.')).collect();
        format!("ext:{}", list.join(";"))
    };
    Some(match cond {
        Cond::Ext(list) => {
            let list: Vec<&str> = list.iter().map(String::as_str).collect();
            exts(&list)
        }
        Cond::Type(cats) => {
            let mut all: Vec<&str> = Vec::new();
            for cat in cats {
                match cat {
                    TypeCategory::Folders => return Some("folder:".into()),
                    // Programs and installers depend on shortcut targets and names: the app
                    // tests them on the answer.
                    TypeCategory::Programs | TypeCategory::Installers => return None,
                    c => all.extend_from_slice(rules::category_exts(*c)),
                }
            }
            exts(&all)
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
            r#""C:\Users\pf\Downloads\" file: dm:>=2026-10-31"#
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
    fn typed_everything_text_replaces_the_conditions() {
        let mut spec = Template::Downloads.spec(&folders()).unwrap();
        spec.everything = Some("*.pdf invoice".into());
        assert_eq!(
            spec.everything_query(Some("2026-10-31")),
            r#""C:\Users\pf\Downloads\" *.pdf invoice"#
        );
        assert!(!spec.works_without_everything());
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
        assert_eq!(t(Cond::Type(vec![TypeCategory::Programs])), None);
        assert_eq!(t(Cond::IdleDays { min: 30 }), None);
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
    fn dates_are_iso() {
        assert_eq!(
            everything_date(crate::CivilDate::new(2026, 3, 5)),
            "2026-03-05"
        );
    }
}
