//! Shared offline translations for native windows and the settings page.
//!
//! Chinese source messages are stable catalog keys. Only application-owned messages
//! are translated: filenames, user titles, rules and saved snapshot names are data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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
        _ => (&ENGLISH, include_str!("../../../locales/en.json")),
    };
    slot.get_or_init(|| serde_json::from_str(source).expect("validated translation catalog"))
}

pub fn text_in(language: Language, source: &'static str) -> &'static str {
    if language == Language::SimplifiedChinese {
        return source;
    }
    catalog(language)
        .get(source)
        .or_else(|| catalog(Language::English).get(source))
        .map(String::as_str)
        .unwrap_or(source)
}

pub fn text(source: &'static str) -> &'static str {
    text_in(language(), source)
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
    fn resolves_ui_language_and_falls_back_to_english() {
        assert_eq!(Language::System.resolve("ja-JP"), Language::Japanese);
        assert_eq!(
            Language::System.resolve("zh-CN"),
            Language::SimplifiedChinese
        );
        assert_eq!(Language::System.resolve("de-DE"), Language::German);
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
}
