//! The type filter of the Peek search (its row of tabs) and of 筛选 (a 类型 token in the box): one
//! of the sorting rules' categories (文档, 图片, …) or one extension (PDF), so a type means the
//! same in the search, in a fence, in Everything's words and in a smart fence made from them.
//! The filter box hands a token on as a `kind:<name>` word in front of the typed text.

use crate::rules::{self, Cond, TypeCategory};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeToken {
    Cat(TypeCategory),
    /// Lower case, without the dot.
    Ext(String),
}

/// The Peek search's tabs after 全部, as Windows 11's Start search lists its types.
pub const TABS: [TypeCategory; 7] = [
    TypeCategory::Folders,
    TypeCategory::Documents,
    TypeCategory::Images,
    TypeCategory::Video,
    TypeCategory::Music,
    TypeCategory::Archives,
    TypeCategory::Programs,
];

/// A category's name in the UI language (the rules' own names).
pub fn category_label(c: TypeCategory) -> &'static str {
    crate::i18n::text(match c {
        TypeCategory::Programs => "程序",
        TypeCategory::Folders => "文件夹",
        TypeCategory::Documents => "文档",
        TypeCategory::Images => "图片",
        TypeCategory::Music => "音乐",
        TypeCategory::Video => "视频",
        TypeCategory::Archives => "压缩包",
        TypeCategory::Shortcuts => "快捷方式",
        TypeCategory::Installers => "安装包",
    })
}

/// 「类型：PDF」, 「类型：文档」.
pub fn label(t: &TypeToken) -> String {
    let name = match t {
        TypeToken::Cat(c) => category_label(*c).to_string(),
        TypeToken::Ext(e) => e.to_uppercase(),
    };
    crate::i18n::format("类型：{0}", &[name])
}

/// Names a typed word may give a category: the Chinese ones, the rules' wire names, a few
/// plain English ones.
const NAMES: &[(&str, TypeCategory)] = &[
    ("程序", TypeCategory::Programs),
    ("应用", TypeCategory::Programs),
    ("programs", TypeCategory::Programs),
    ("apps", TypeCategory::Programs),
    ("文件夹", TypeCategory::Folders),
    ("folders", TypeCategory::Folders),
    ("folder", TypeCategory::Folders),
    ("文档", TypeCategory::Documents),
    ("documents", TypeCategory::Documents),
    ("docs", TypeCategory::Documents),
    ("图片", TypeCategory::Images),
    ("照片", TypeCategory::Images),
    ("images", TypeCategory::Images),
    ("pictures", TypeCategory::Images),
    ("音乐", TypeCategory::Music),
    ("music", TypeCategory::Music),
    ("视频", TypeCategory::Video),
    ("video", TypeCategory::Video),
    ("videos", TypeCategory::Video),
    ("压缩包", TypeCategory::Archives),
    ("archives", TypeCategory::Archives),
    ("快捷方式", TypeCategory::Shortcuts),
    ("shortcuts", TypeCategory::Shortcuts),
    ("安装包", TypeCategory::Installers),
    ("installers", TypeCategory::Installers),
];

/// An extension one of the categories knows, or a program's.
fn known_ext(e: &str) -> bool {
    let dotted = format!(".{e}");
    matches!(e, "exe" | "lnk" | "msi" | "url" | "bat" | "cmd")
        || rules::CLASSIFY_ORDER
            .iter()
            .any(|c| rules::category_exts(*c).contains(&dotted.as_str()))
}

/// The type a typed word names: a category (in Chinese, in the UI language or in English) or a
/// known extension, with or without its dot. None for any other word.
pub fn suggest(word: &str) -> Option<TypeToken> {
    let w = word.trim().trim_start_matches('.').to_lowercase();
    if w.is_empty() {
        return None;
    }
    if let Some((_, c)) = NAMES.iter().find(|(n, _)| *n == w) {
        return Some(TypeToken::Cat(*c));
    }
    if let Some(c) = rules::CLASSIFY_ORDER
        .iter()
        .find(|c| category_label(**c).to_lowercase() == w)
    {
        return Some(TypeToken::Cat(*c));
    }
    known_ext(&w).then_some(TypeToken::Ext(w))
}

/// `kind:pdf`, `kind:documents`, `类型:文档` or `类型：图片` as a type; None for other words.
pub fn parse_word(word: &str) -> Option<TypeToken> {
    let rest = ["kind:", "类型:", "类型："]
        .iter()
        .find_map(|p| word.strip_prefix(p))?;
    if let Some(c) = rules::CLASSIFY_ORDER
        .iter()
        .find(|c| rules::category_name(**c) == rest)
    {
        return Some(TypeToken::Cat(*c));
    }
    suggest(rest)
}

/// The `kind:` word a token is handed on as.
pub fn word(t: &TypeToken) -> String {
    match t {
        TypeToken::Cat(c) => format!("kind:{}", rules::category_name(*c)),
        TypeToken::Ext(e) => format!("kind:{e}"),
    }
}

/// The types in a query (`kind:` words anywhere) and the rest of it.
pub fn split(query: &str) -> (Vec<TypeToken>, String) {
    let mut tokens = Vec::new();
    let mut rest = Vec::new();
    for w in query.split_whitespace() {
        match parse_word(w) {
            Some(t) if !tokens.contains(&t) => tokens.push(t),
            Some(_) => {}
            None => rest.push(w),
        }
    }
    (tokens, rest.join(" "))
}

/// Whether a file `name` (a folder when `is_folder`) is of every type in `tokens`, as the
/// `type` and `ext` rule conditions test it.
pub fn matches(tokens: &[TypeToken], name: &str, is_folder: bool) -> bool {
    let ext = rules::ext_of(name);
    tokens.iter().all(|t| match t {
        TypeToken::Ext(e) => !is_folder && ext == format!(".{e}"),
        TypeToken::Cat(c) => rules::file_in_category(*c, name, is_folder),
    })
}

/// Everything's own words for `tokens` (`ext:pdf`, `folder:`), added to what it is asked.
pub fn everything_words(tokens: &[TypeToken]) -> String {
    tokens
        .iter()
        .map(|t| match t {
            TypeToken::Ext(e) => format!("ext:{e}"),
            TypeToken::Cat(TypeCategory::Folders) => "folder:".to_string(),
            TypeToken::Cat(TypeCategory::Programs) => "ext:exe;msi;bat;cmd;appref-ms;lnk".into(),
            TypeToken::Cat(TypeCategory::Shortcuts) => "ext:lnk;url;website".into(),
            TypeToken::Cat(TypeCategory::Installers) => {
                "ext:msi;msix;msixbundle;appx;appxbundle;appinstaller;exe".into()
            }
            TypeToken::Cat(c) => format!(
                "file: ext:{}",
                rules::category_exts(*c)
                    .iter()
                    .map(|e| e.trim_start_matches('.'))
                    .collect::<Vec<_>>()
                    .join(";")
            ),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The rule condition a token is in a smart fence (「把这次搜索放到桌面」, 把筛选存为智能栅栏).
pub fn cond(t: &TypeToken) -> Cond {
    match t {
        TypeToken::Cat(c) => Cond::Type(vec![*c]),
        TypeToken::Ext(e) => Cond::Ext(vec![format!(".{e}")]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_name_types_and_the_rest_stays_text() {
        assert_eq!(suggest("pdf"), Some(TypeToken::Ext("pdf".into())));
        assert_eq!(suggest(".PSD"), Some(TypeToken::Ext("psd".into())));
        assert_eq!(suggest("图片"), Some(TypeToken::Cat(TypeCategory::Images)));
        assert_eq!(
            suggest("documents"),
            Some(TypeToken::Cat(TypeCategory::Documents))
        );
        assert_eq!(suggest("季度"), None);
        assert_eq!(suggest("report"), None);
        let (types, rest) = split("kind:pdf 季度 类型：图片 kind:pdf");
        assert_eq!(
            types,
            vec![
                TypeToken::Ext("pdf".into()),
                TypeToken::Cat(TypeCategory::Images)
            ]
        );
        assert_eq!(rest, "季度");
        assert_eq!(
            split(&format!(
                "{} 报告",
                word(&TypeToken::Cat(TypeCategory::Documents))
            ))
            .0,
            vec![TypeToken::Cat(TypeCategory::Documents)]
        );
    }

    #[test]
    fn a_type_matches_as_the_rule_conditions_do() {
        let pdf = [TypeToken::Ext("pdf".into())];
        assert!(matches(&pdf, "季度报告.pdf", false));
        assert!(!matches(&pdf, "季度报告.docx", false));
        assert!(!matches(&pdf, "pdf", true));
        let docs = [TypeToken::Cat(TypeCategory::Documents)];
        assert!(matches(&docs, "季度报告.docx", false));
        assert!(matches(&docs, "季度报告.PDF", false));
        assert!(!matches(&docs, "季度海报.png", false));
        let folders = [TypeToken::Cat(TypeCategory::Folders)];
        assert!(matches(&folders, "照片", true));
        assert!(!matches(&folders, "照片.jpg", false));
        assert!(matches(&[], "anything", false));
    }

    #[test]
    fn everything_and_smart_fences_get_the_same_type() {
        assert_eq!(everything_words(&[TypeToken::Ext("pdf".into())]), "ext:pdf");
        assert_eq!(
            everything_words(&[TypeToken::Cat(TypeCategory::Folders)]),
            "folder:"
        );
        assert!(
            everything_words(&[TypeToken::Cat(TypeCategory::Images)])
                .starts_with("file: ext:png;jpg")
        );
        assert_eq!(
            cond(&TypeToken::Cat(TypeCategory::Music)),
            Cond::Type(vec![TypeCategory::Music])
        );
        assert_eq!(
            cond(&TypeToken::Ext("pdf".into())),
            Cond::Ext(vec![".pdf".into()])
        );
    }
}
