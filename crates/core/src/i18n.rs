//! Shared offline translations for native windows and the settings page.
//!
//! Chinese source messages are stable catalog keys. Only application-owned messages
//! are translated: filenames, user titles, rules and saved snapshot names are data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Language {
    #[default]
    #[serde(rename = "system")]
    System,
    #[serde(rename = "zh-CN")]
    SimplifiedChinese,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "zh-TW")]
    TraditionalChinese,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "pt-BR")]
    Portuguese,
    #[serde(rename = "ru")]
    Russian,
    #[serde(rename = "it")]
    Italian,
}

impl Language {
    /// Existing configurations were Chinese-only; preserve their UI until changed.
    pub fn legacy_default() -> Self {
        Self::SimplifiedChinese
    }

    pub fn from_locale(locale: &str) -> Self {
        let locale = locale.to_ascii_lowercase().replace('_', "-");
        match locale.split('-').next().unwrap_or("") {
            "zh" if locale.contains("hant")
                || locale.ends_with("-tw")
                || locale.ends_with("-hk")
                || locale.ends_with("-mo") =>
            {
                Self::TraditionalChinese
            }
            "zh" => Self::SimplifiedChinese,
            "ja" => Self::Japanese,
            "ko" => Self::Korean,
            "de" => Self::German,
            "fr" => Self::French,
            "es" => Self::Spanish,
            "pt" => Self::Portuguese,
            "ru" => Self::Russian,
            "it" => Self::Italian,
            _ => Self::English,
        }
    }

    pub fn resolve(self, system_locale: &str) -> Self {
        if self == Self::System {
            Self::from_locale(system_locale)
        } else {
            self
        }
    }

    pub fn tag(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::SimplifiedChinese => "zh-CN",
            Self::English => "en",
            Self::Japanese => "ja",
            Self::TraditionalChinese => "zh-TW",
            Self::Korean => "ko",
            Self::German => "de",
            Self::French => "fr",
            Self::Spanish => "es",
            Self::Portuguese => "pt-BR",
            Self::Russian => "ru",
            Self::Italian => "it",
        }
    }
}

// Default keeps pure core consumers and tests deterministic. The application selects
// the persisted/system language before it creates default fences or any windows.
static ACTIVE: AtomicU8 = AtomicU8::new(0);
static ENGLISH: OnceLock<HashMap<String, String>> = OnceLock::new();
static JAPANESE: OnceLock<HashMap<String, String>> = OnceLock::new();
static TRADITIONAL: OnceLock<HashMap<String, String>> = OnceLock::new();
static KOREAN: OnceLock<HashMap<String, String>> = OnceLock::new();
static GERMAN: OnceLock<HashMap<String, String>> = OnceLock::new();
static FRENCH: OnceLock<HashMap<String, String>> = OnceLock::new();
static SPANISH: OnceLock<HashMap<String, String>> = OnceLock::new();
static PORTUGUESE: OnceLock<HashMap<String, String>> = OnceLock::new();
static RUSSIAN: OnceLock<HashMap<String, String>> = OnceLock::new();
static ITALIAN: OnceLock<HashMap<String, String>> = OnceLock::new();

pub const SUPPORTED: &[Language] = &[
    Language::SimplifiedChinese,
    Language::TraditionalChinese,
    Language::English,
    Language::Japanese,
    Language::Korean,
    Language::German,
    Language::French,
    Language::Spanish,
    Language::Portuguese,
    Language::Russian,
    Language::Italian,
];

pub fn set_language(language: Language) {
    ACTIVE.store(
        match language {
            Language::English | Language::System => 1,
            Language::Japanese => 2,
            Language::SimplifiedChinese => 0,
            Language::TraditionalChinese => 3,
            Language::Korean => 4,
            Language::German => 5,
            Language::French => 6,
            Language::Spanish => 7,
            Language::Portuguese => 8,
            Language::Russian => 9,
            Language::Italian => 10,
        },
        Ordering::Relaxed,
    );
}

pub fn language() -> Language {
    match ACTIVE.load(Ordering::Relaxed) {
        1 => Language::English,
        2 => Language::Japanese,
        3 => Language::TraditionalChinese,
        4 => Language::Korean,
        5 => Language::German,
        6 => Language::French,
        7 => Language::Spanish,
        8 => Language::Portuguese,
        9 => Language::Russian,
        10 => Language::Italian,
        _ => Language::SimplifiedChinese,
    }
}

pub fn catalog(language: Language) -> &'static HashMap<String, String> {
    let (slot, source) = match language {
        Language::Japanese => (&JAPANESE, include_str!("../../../locales/ja.json")),
        Language::TraditionalChinese => (&TRADITIONAL, include_str!("../../../locales/zh-TW.json")),
        Language::Korean => (&KOREAN, include_str!("../../../locales/ko.json")),
        Language::German => (&GERMAN, include_str!("../../../locales/de.json")),
        Language::French => (&FRENCH, include_str!("../../../locales/fr.json")),
        Language::Spanish => (&SPANISH, include_str!("../../../locales/es.json")),
        Language::Portuguese => (&PORTUGUESE, include_str!("../../../locales/pt-BR.json")),
        Language::Russian => (&RUSSIAN, include_str!("../../../locales/ru.json")),
        Language::Italian => (&ITALIAN, include_str!("../../../locales/it.json")),
        _ => (&ENGLISH, include_str!("../../../locales/en.json")),
    };
    slot.get_or_init(|| serde_json::from_str(source).expect("validated translation catalog"))
}

/// Separates a disambiguating context from the message in a key, as in gettext:
/// `"新建\x04文件夹"` is the singular New ▸ Folder command, while `"文件夹"` stays the
/// plural default fence name. Simplified Chinese shows only the text after it.
pub const CONTEXT: char = '\u{4}';

pub fn text_in(language: Language, source: &'static str) -> &'static str {
    let bare = source
        .split_once(CONTEXT)
        .map_or(source, |(_, message)| message);
    if language == Language::SimplifiedChinese {
        return bare;
    }
    catalog(language)
        .get(source)
        .or_else(|| catalog(Language::English).get(source))
        .map(String::as_str)
        .unwrap_or(bare)
}

/// The Han locale and the GDI face for Windows' UI language `ui` (`zh-TW`, `ja`, `ko`, …):
/// Han characters take the forms of that language, and Simplified Chinese's for every other
/// one. The fence titles (DirectWrite) use the locale, the Peek search list (GDI) the face, so
/// both show 下载 the same way; on an English Windows GDI's own fallback drew it in Japanese or
/// Traditional forms.
pub fn han_for(ui: &str) -> (&'static str, &'static str) {
    match ui {
        "zh-TW" => ("zh-TW", "Microsoft JhengHei UI"),
        "ja" => ("ja-JP", "Yu Gothic UI"),
        "ko" => ("ko-KR", "Malgun Gothic"),
        _ => ("zh-CN", "Microsoft YaHei UI"),
    }
}

pub fn text(source: &'static str) -> &'static str {
    text_in(language(), source)
}

/// A name PecoFence gave in some language for one of `keys` (a first-start fence the user never
/// renamed): the same name in `language`, when that differs.
pub fn retranslated_in(
    language: Language,
    name: &str,
    keys: &[&'static str],
) -> Option<&'static str> {
    keys.iter().find_map(|key| {
        let now = text_in(language, key);
        (name != now && SUPPORTED.iter().any(|l| text_in(*l, key) == name)).then_some(now)
    })
}

/// [`retranslated_in`] the current language.
pub fn retranslated(name: &str, keys: &[&'static str]) -> Option<&'static str> {
    retranslated_in(language(), name, keys)
}

pub fn ui_payload() -> serde_json::Value {
    let active = language();
    let translations = if active == Language::SimplifiedChinese {
        serde_json::json!({})
    } else {
        serde_json::to_value(catalog(active)).expect("serializable translation catalog")
    };
    serde_json::json!({ "locale": active.tag(), "translations": translations })
}

/// Substitute numbered placeholders once. Arguments are never parsed as templates,
/// so braces in filenames and user titles remain literal.
pub fn interpolate(template: &str, arguments: &[String]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        if let Some(end) = rest.find('}')
            && let Ok(index) = rest[1..end].parse::<usize>()
            && let Some(value) = arguments.get(index)
        {
            out.push_str(value);
            rest = &rest[end + 1..];
        } else {
            out.push('{');
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    out
}

pub fn format(source: &'static str, arguments: &[String]) -> String {
    interpolate(text(source), arguments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn han_text_takes_the_forms_of_the_ui_language_and_simplified_for_the_rest() {
        assert_eq!(han_for("zh-CN"), ("zh-CN", "Microsoft YaHei UI"));
        assert_eq!(han_for("zh-TW"), ("zh-TW", "Microsoft JhengHei UI"));
        assert_eq!(han_for("ja"), ("ja-JP", "Yu Gothic UI"));
        assert_eq!(han_for("ko"), ("ko-KR", "Malgun Gothic"));
        for other in ["en", "de", "fr", "es", "pt-BR", "ru", "it", ""] {
            assert_eq!(han_for(other), ("zh-CN", "Microsoft YaHei UI"), "{other}");
        }
    }

    #[test]
    fn first_start_names_follow_the_language_until_renamed() {
        let keys = ["程序", "文件夹", "文件与文档"];
        assert_eq!(
            retranslated_in(Language::English, "程序", &keys),
            Some("Programs")
        );
        assert_eq!(
            retranslated_in(Language::SimplifiedChinese, "Folders", &keys),
            Some("文件夹")
        );
        assert_eq!(
            retranslated_in(Language::Japanese, "Files and documents", &keys),
            Some(text_in(Language::Japanese, "文件与文档"))
        );
        // Already in the language, or a name the user chose: left alone.
        assert_eq!(retranslated_in(Language::English, "Programs", &keys), None);
        assert_eq!(retranslated_in(Language::English, "My apps", &keys), None);
        assert_eq!(retranslated_in(Language::English, "程序 2", &keys), None);
    }

    #[test]
    fn resolves_ui_language_and_falls_back_to_english() {
        assert_eq!(Language::System.resolve("ja-JP"), Language::Japanese);
        assert_eq!(
            Language::System.resolve("zh-CN"),
            Language::SimplifiedChinese
        );
        assert_eq!(Language::System.resolve("de-DE"), Language::German);
        assert_eq!(Language::System.resolve("it-IT"), Language::Italian);
        assert_eq!(Language::System.resolve("ar-SA"), Language::English);
        assert_eq!(
            Language::System.resolve("zh-Hant-HK"),
            Language::TraditionalChinese
        );
        assert_eq!(Language::Japanese.resolve("en-US"), Language::Japanese);
    }

    #[test]
    fn interpolation_allows_reordering_without_interpreting_user_data() {
        assert_eq!(
            interpolate("{1}: {0}", &["report {1}.txt".into(), "File".into()]),
            "File: report {1}.txt"
        );
        assert_eq!(interpolate("{2}", &[]), "{2}");
    }

    #[test]
    fn saved_languages_round_trip_and_old_settings_stay_chinese() {
        let settings = crate::Settings::default();
        assert_eq!(settings.language, Language::System);
        for &language in SUPPORTED {
            let mut settings = settings.clone();
            settings.language = language;
            let json = serde_json::to_string(&settings).unwrap();
            let loaded: crate::Settings = serde_json::from_str(&json).unwrap();
            assert_eq!(loaded.language, language);
        }
        let mut old = serde_json::to_value(&settings).unwrap();
        old.as_object_mut().unwrap().remove("language");
        let loaded: crate::Settings = serde_json::from_value(old).unwrap();
        assert_eq!(loaded.language, Language::SimplifiedChinese);
        assert_eq!(text_in(Language::English, "不存在的消息"), "不存在的消息");
    }

    #[test]
    fn context_keys_show_the_bare_chinese_message() {
        assert_eq!(
            text_in(Language::SimplifiedChinese, "新建\x04文件夹"),
            "文件夹"
        );
        assert_eq!(text_in(Language::English, "新建\x04文件夹"), "Folder");
        assert_eq!(text_in(Language::English, "文件夹"), "Folders");
        assert_eq!(text_in(Language::English, "测试\x04不存在"), "不存在");
    }

    /// The button of the last first-start tip opens the manual. "Manual" alone also reads as
    /// "by hand" in English, Spanish, Portuguese, French and Italian: there the button says
    /// what it does. Simplified Chinese keeps 使用手册.
    #[test]
    fn the_last_tips_button_says_that_it_opens_the_manual() {
        let key = "小技巧\x04使用手册";
        assert_eq!(text_in(Language::SimplifiedChinese, key), "使用手册");
        assert_eq!(text_in(Language::English, key), "Open the manual");
        for tag in ["en", "es", "pt-BR", "fr", "it"] {
            let language = SUPPORTED.iter().copied().find(|l| l.tag() == tag);
            let language = language.unwrap_or_else(|| panic!("{tag}"));
            let (button, noun) = (text_in(language, key), text_in(language, "使用手册"));
            assert!(
                button != noun && button.to_lowercase().contains(&noun.to_lowercase()),
                "{tag}: {button}"
            );
        }
    }

    #[test]
    fn catalogs_have_matching_keys_and_placeholders() {
        let english = catalog(Language::English);
        assert!(!english.is_empty());
        fn placeholders(s: &str) -> Vec<String> {
            let mut result: Vec<_> = s
                .split('{')
                .skip(1)
                .filter_map(|part| part.split_once('}'))
                .filter(|(key, _)| key.parse::<usize>().is_ok())
                .map(|(key, _)| key.to_string())
                .collect();
            result.sort();
            result
        }
        for &language in SUPPORTED {
            if language == Language::SimplifiedChinese {
                continue;
            }
            let translated = catalog(language);
            assert_eq!(english.len(), translated.len(), "{}", language.tag());
            for source in english.keys() {
                let value = translated.get(source).expect(source);
                assert!(!value.trim().is_empty(), "{source}");
                assert_eq!(
                    placeholders(source),
                    placeholders(value),
                    "{}: {source}",
                    language.tag()
                );
            }
        }
    }

    /// The notice after a crash while a menu was built is a Windows banner: 4 lines of about
    /// 36 Latin or 18 Chinese, Japanese or Korean characters, the rest is cut. Measured on
    /// 2026-11-12 with a 24-character file name: every language fits at up to 126 half-width
    /// characters, and the 182 of the earlier English text lost their last 48.
    #[test]
    fn the_menu_crash_notice_fits_the_banner() {
        let source = "右键菜单里的 Windows 菜单项已暂时关闭，可以在托盘菜单里恢复。上次因其他程序的 {0} 出错而关闭。";
        let name = "ContextMenuHandler64.dll".to_string();
        // Half-width characters: Chinese, Japanese and Korean characters take two.
        let width =
            |s: &str| -> usize { s.chars().map(|c| if c >= '\u{2E80}' { 2 } else { 1 }).sum() };
        for &language in SUPPORTED {
            let text = match language {
                Language::SimplifiedChinese => source,
                _ => catalog(language)
                    .get(source)
                    .unwrap_or_else(|| panic!("{}", language.tag())),
            };
            let shown = interpolate(text, std::slice::from_ref(&name));
            assert!(
                width(&shown) <= 130,
                "{}: {} half-width characters: {shown}",
                language.tag(),
                width(&shown)
            );
        }
    }

    /// Every text the app hands to a Windows notification fits the banner in every language:
    /// 4 lines of about 36 Latin or 18 Chinese, Japanese or Korean characters (measured on
    /// 2026-11-12), the rest is cut, and the end of a sentence is often the part that says
    /// what to do. The texts are found in the app's sources: every catalog key in a function
    /// that calls `show_info(` or `.toast(`. Not checked where the sources are not beside the
    /// test (the test boxes run the built tests only).
    #[test]
    fn every_notification_fits_the_banner() {
        fn rust_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    rust_files(&path, out);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    out.push(path);
                }
            }
        }
        // The string literals of a piece of Rust code (`\"` and `\\` unescaped).
        fn literals(code: &str) -> Vec<String> {
            let mut out = Vec::new();
            let mut chars = code.chars();
            while let Some(c) = chars.next() {
                if c == '\'' && chars.as_str().starts_with("\"'") {
                    chars.nth(1); // the character '"'
                } else if c == '"' {
                    let mut literal = String::new();
                    while let Some(c) = chars.next() {
                        match c {
                            '"' => break,
                            '\\' => literal.extend(chars.next()),
                            _ => literal.push(c),
                        }
                    }
                    out.push(literal);
                }
            }
            out
        }
        let app = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/src");
        let mut files = Vec::new();
        rust_files(&app, &mut files);
        if files.is_empty() {
            eprintln!("no app sources at {}: not checked here", app.display());
            return;
        }
        let english = catalog(Language::English);
        let mut sources = std::collections::BTreeSet::new();
        for file in &files {
            let code = std::fs::read_to_string(file).unwrap();
            // `toast` is the app's own short way to the same notification.
            let calls = code.match_indices("show_info(");
            for (call, _) in calls.chain(code.match_indices(".toast(")) {
                // From the start of the function around the call to a little after it.
                let from = code[..call]
                    .rfind("\n    fn ")
                    .max(code[..call].rfind("\n    pub"))
                    .unwrap_or(0);
                let to = code[call..]
                    .char_indices()
                    .nth(700)
                    .map_or(code.len(), |(i, _)| call + i);
                sources.extend(
                    literals(&code[from..to])
                        .into_iter()
                        .filter(|s| english.contains_key(s)),
                );
            }
        }
        assert!(sources.len() >= 30, "only {} texts found", sources.len());
        // Values as long as real ones: the configuration folder, the name an unreadable
        // configuration file gets, and 24 characters for the rest (a file name, a shortcut
        // of four modifiers).
        let values = |source: &str| -> [String; 2] {
            let first = if source.contains("能否写入") {
                r"C:\Users\me\AppData\Roaming\PecoFence"
            } else if source.contains("改名为") {
                "config.unreadable-20261112-66600.json"
            } else {
                "ContextMenuHandler64.dll"
            };
            [first.to_string(), "Ctrl+Alt+Space".to_string()]
        };
        // Half-width characters: Chinese, Japanese and Korean characters take two.
        let width =
            |s: &str| -> usize { s.chars().map(|c| if c >= '\u{2E80}' { 2 } else { 1 }).sum() };
        let mut long = Vec::new();
        for source in &sources {
            for &language in SUPPORTED {
                let text = match language {
                    Language::SimplifiedChinese => source,
                    _ => &catalog(language)[source],
                };
                let shown = interpolate(text, &values(source));
                if width(&shown) > 130 {
                    long.push(format!("{} {}: {shown}", language.tag(), width(&shown)));
                }
            }
        }
        assert!(
            long.is_empty(),
            "longer than 130 half-width characters:\n{}",
            long.join("\n")
        );
    }

    /// The card after a fence is deleted says what happened first and the fence's name last:
    /// a long name wraps, and the last line then held the verb alone (`Fence “…” / deleted`).
    #[test]
    fn the_undo_card_names_the_deleted_fence_last() {
        let source = "已删除栅栏“{0}”";
        for &language in SUPPORTED {
            let text = match language {
                Language::SimplifiedChinese => source,
                _ => &catalog(language)[source],
            };
            let after = text.split("{0}").nth(1).unwrap_or_default();
            assert!(
                !after.chars().any(char::is_alphabetic),
                "{}: {text}",
                language.tag()
            );
        }
    }

    /// A script fence's footer names the script and says how the last run went; when both do
    /// not fit, the name is left out. The status of a failed run stays short enough for both
    /// in a fence 300 px wide (German had 47 characters and lost the name).
    #[test]
    fn the_status_of_a_failed_script_leaves_room_for_its_name() {
        let source = "{0} 运行出错（退出代码 {1}）";
        let values = ["23:59".to_string(), "1".to_string()];
        let width =
            |s: &str| -> usize { s.chars().map(|c| if c >= '\u{2E80}' { 2 } else { 1 }).sum() };
        for &language in SUPPORTED {
            let text = match language {
                Language::SimplifiedChinese => source,
                _ => &catalog(language)[source],
            };
            let shown = interpolate(text, &values);
            assert!(width(&shown) <= 40, "{}: {shown}", language.tag());
        }
    }

    /// One kind of quotation mark per language: a card that names a fence in “ ” above a
    /// button named in 「 」 reads as two programs. The source strings (Simplified Chinese)
    /// are the catalogs' keys.
    #[test]
    fn each_language_uses_one_kind_of_quotation_marks() {
        // The marks a language's strings do not use.
        let foreign = |language: Language| -> &'static [char] {
            match language {
                Language::Japanese | Language::TraditionalChinese => &['“', '”', '‘'],
                Language::German => &['「', '」', '«', '»', '‘'],
                Language::French | Language::Spanish | Language::Russian | Language::Italian => {
                    &['「', '」', '“', '”', '‘']
                }
                // English, Korean, Portuguese and the Simplified Chinese source: “ ”.
                _ => &['「', '」', '«', '»', '‘', '„'],
            }
        };
        for source in catalog(Language::English).keys() {
            let odd: Vec<char> = source
                .chars()
                .filter(|c| foreign(Language::SimplifiedChinese).contains(c))
                .collect();
            assert!(odd.is_empty(), "zh-CN: {odd:?} in {source}");
        }
        for &language in SUPPORTED {
            if language == Language::SimplifiedChinese {
                continue;
            }
            for (source, value) in catalog(language) {
                let odd: Vec<char> = value
                    .chars()
                    .filter(|c| foreign(language).contains(c))
                    .collect();
                assert!(
                    odd.is_empty(),
                    "{}: {odd:?} in {value} ({source})",
                    language.tag()
                );
                // A name in straight " " (\"{0}\" 필터) beside the language's own marks.
                assert!(
                    !value.contains("\"{") && !value.contains("}\""),
                    "{}: a straight \" around a name in {value}",
                    language.tag()
                );
                // A straight ' after a letter is an apostrophe (it's, d'ora, fences'); one
                // that starts a word opens a quotation.
                let chars: Vec<char> = value.chars().collect();
                for (i, c) in chars.iter().enumerate() {
                    let opens = *c == '\''
                        && (i == 0 || !chars[i - 1].is_alphanumeric())
                        && chars.get(i + 1).is_some_and(|n| n.is_alphanumeric());
                    assert!(
                        !opens,
                        "{}: a straight ' as a quotation mark in {value}",
                        language.tag()
                    );
                }
            }
        }
    }
}
