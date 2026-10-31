# Localization

Settings → General → Display language supports live switching. New installations
follow the Windows **display language**, not the region/date-format setting.
Existing configurations without a `language` field retain Simplified Chinese.

| Setting | Language |
|---|---|
| `system` | Follow Windows; unsupported display languages fall back to English |
| `zh-CN` | Simplified Chinese |
| `zh-TW` | Traditional Chinese |
| `en` | English |
| `ja` | Japanese |
| `ko` | Korean |
| `de` | German |
| `fr` | French |
| `es` | Spanish |
| `pt-BR` | Portuguese (Brazil) |
| `ru` | Russian |
| `it` | Italian |

Chinese display locales using Traditional Chinese resolve to `zh-TW`. Portuguese
display locales use the supported Brazilian Portuguese catalog.

## Catalogs

`locales/*.json` use the original Simplified Chinese application message as a
stable key. Simplified Chinese uses the source text directly. Keep keys intact;
edit the value to improve a translation.

Native code uses `pecofence_core::i18n::text` and `i18n::format`. The settings page
uses the same catalog through `ui/i18n.js`. Catalogs and scripts are embedded into
the executable, so installation does not require copying a locale directory.

Numbered placeholders (`{0}`, `{1}`, etc.) may be reordered for grammar but must
be preserved. `%1` is a Windows drag-description insertion marker and must also
remain intact. Arguments are substituted once; braces inside a filename are literal.

Put a count, name or value inside its whole message (`tf("{0} 个栅栏", n)`) rather
than joining translated fragments, so every language controls word order and
inflection. When the same Chinese text needs two translations, prefix the key with a
context and U+0004, as in gettext: `"新建\x04文件夹"` is the singular New ▸ Folder
command, while `"文件夹"` stays the plural default fence name. Simplified Chinese
shows only the text after the separator. Non-Chinese source text in the settings
page, such as a unit, names its message with `data-i18n="MB"`.

The page captures only its original static text for translation. Dynamic strings
must explicitly use `t()` or `tf()`. Do not translate arbitrary DOM text, user
filenames, rule names, snapshot names or fence titles.

Default group/rule names are localized when created. They are then saved as names
and are not rewritten by later language changes.

## Validation and adding a language

Run `python scripts/check-locales.py` and the core tests after any catalog edit.
The checker verifies source coverage and placeholder parity in every language.
The browser suite also switches through every language while retaining draft data.

To add a language, add its catalog, extend `Language`/`SUPPORTED` in
`crates/core/src/i18n.rs`, map the Windows language in `crates/platform/src/locale.rs`,
and update the settings selector and verification language lists.

Windows-owned dialogs, dates, file-type descriptions and third-party Explorer
menu entries are provided by Windows and retain the system's own language.

## README translations

The root `README.md` is English. Each interface language also has a README under
`docs/readme/README.<language>.md`, sharing the same structure and a localized hero
image in `docs/assets/hero-<language>.png`. When the English README changes, update
every translation in the same change, keep UI terms identical to the language's
catalog in `locales/`, and rerun `python scripts/make-readme-media.py --stills-only`
if the hero text in `HERO_TEXT` changed.
