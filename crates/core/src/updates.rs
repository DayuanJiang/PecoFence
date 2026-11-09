//! In-app updates for the installer and ZIP editions (the Store updates the app itself):
//! whether GitHub's latest release is newer and which file the installer downloads. The
//! network and the hand-off to setup live in the app.

use serde::Deserialize;

pub const REPOSITORY: &str = "DayuanJiang/PecoFence";

/// GitHub's newest published release; drafts and prereleases are never "latest".
pub fn latest_release_url() -> String {
    format!("https://api.github.com/repos/{REPOSITORY}/releases/latest")
}

/// Where the release files of this repository are downloaded from.
pub fn download_base() -> String {
    format!("https://github.com/{REPOSITORY}/releases/download/")
}

#[derive(Debug, Clone, PartialEq)]
pub struct Release {
    pub version: String,
    /// The release page, opened by the ZIP edition.
    pub page: String,
    /// The release notes' first sentence (a notice's body); empty when they have none.
    pub summary: String,
    /// The setup EXE, when the release has one.
    pub setup: Option<SetupAsset>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetupAsset {
    pub name: String,
    pub url: String,
}

#[derive(Deserialize)]
struct ApiRelease {
    tag_name: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    assets: Vec<ApiAsset>,
}

#[derive(Deserialize)]
struct ApiAsset {
    name: String,
    browser_download_url: String,
}

/// The first sentence of release notes in Markdown: the first line with words (headings,
/// list marks and emphasis taken off), up to its first full stop.
pub fn first_sentence(notes: &str) -> String {
    let line = notes
        .lines()
        .map(|l| {
            l.trim()
                .trim_start_matches(['#', '-', '*', '>', ' '])
                .replace("**", "")
        })
        .find(|l| !l.trim().is_empty())
        .unwrap_or_default();
    let end = line
        .char_indices()
        .find(|(_, c)| matches!(c, '。' | '！' | '？'))
        .map(|(i, c)| i + c.len_utf8())
        .or_else(|| line.find(". ").map(|i| i + 1))
        .unwrap_or(line.len());
    line[..end].trim().to_string()
}

pub fn setup_name(version: &str) -> String {
    format!("pecofence-v{version}-x64-setup.exe")
}

/// Parses `releases/latest`. Files only count at their exact place under `download_base`
/// (`<base><tag>/<name>`), so an unexpected response cannot point setup somewhere else; the
/// release page is built from the tag rather than taken from the response.
pub fn parse_release(json: &str, download_base: &str) -> Result<Release, String> {
    let api: ApiRelease = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let version = api.tag_name.strip_prefix('v').unwrap_or(&api.tag_name);
    if parse_version(version).is_none() {
        return Err(format!("unexpected release tag {}", api.tag_name));
    }
    let name = setup_name(version);
    let asset = |wanted: &str| {
        let url = format!("{download_base}{}/{wanted}", api.tag_name);
        api.assets
            .iter()
            .any(|a| a.name == wanted && a.browser_download_url == url)
            .then_some(url)
    };
    let setup = asset(&name).map(|url| SetupAsset { name, url });
    Ok(Release {
        version: version.to_string(),
        summary: first_sentence(api.body.as_deref().unwrap_or("")),
        page: format!(
            "https://github.com/{REPOSITORY}/releases/tag/{}",
            api.tag_name
        ),
        setup,
    })
}

/// `major.minor.patch`, with an optional `-prerelease` (letters, digits, `.`, `-`) that sorts
/// before the release.
fn parse_version(version: &str) -> Option<([u64; 3], bool)> {
    let (numbers, prerelease) = match version.split_once('-') {
        Some((numbers, suffix)) => {
            let allowed = |b: u8| b.is_ascii_alphanumeric() || b == b'.' || b == b'-';
            if suffix.is_empty() || !suffix.bytes().all(allowed) {
                return None;
            }
            (numbers, true)
        }
        None => (version, false),
    };
    let mut parts = numbers.split('.').map(|p| p.parse::<u64>().ok());
    let parsed = [parts.next()??, parts.next()??, parts.next()??];
    parts.next().is_none().then_some((parsed, prerelease))
}

/// Whether `candidate` should be offered to a copy running `current`. A prerelease is only
/// offered to prerelease copies: a tag like `v0.2.0-beta.1` can still be GitHub's "latest".
pub fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some((_, true)), Some((_, false))) => false,
        (Some((new, new_pre)), Some((old, old_pre))) => {
            new > old || (new == old && old_pre && !new_pre)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://github.com/DayuanJiang/PecoFence/releases/download/";

    fn release_json(tag: &str, assets: &[(&str, &str)]) -> String {
        let assets: Vec<_> = assets
            .iter()
            .map(|(name, url)| serde_json::json!({ "name": name, "browser_download_url": url }))
            .collect();
        serde_json::json!({
            "tag_name": tag,
            "html_url": "file:///C:/Windows/System32/calc.exe",
            "assets": assets,
        })
        .to_string()
    }

    #[test]
    fn versions_compare_numerically_and_prereleases_come_first() {
        assert!(is_newer("0.1.4", "0.1.3"));
        assert!(is_newer("0.1.10", "0.1.9"));
        assert!(is_newer("1.0.0", "0.99.99"));
        assert!(is_newer("0.2.0", "0.2.0-beta.1"));
        // Prerelease copies get newer prereleases; suffixes of one version are not ordered.
        assert!(is_newer("0.3.0-beta.1", "0.2.0-beta.1"));
        assert!(!is_newer("0.2.0-beta.2", "0.2.0-beta.1"));
        assert!(!is_newer("0.1.3", "0.1.3"));
        assert!(!is_newer("0.1.2", "0.1.3"));
        assert!(!is_newer("0.2.0-beta.1", "0.2.0"));
        // Release copies are never offered a prerelease, even a higher one.
        assert!(!is_newer("0.3.0-beta.1", "0.2.0"));
        assert!(!is_newer("0.2.0-../../x", "0.1.0-a"));
        assert!(!is_newer("0.2.0-", "0.1.0-a"));
        assert!(!is_newer("0.1", "0.0.1"));
        assert!(!is_newer("0.1.4.1", "0.1.3"));
        assert!(!is_newer("latest", "0.1.3"));
    }

    #[test]
    fn picks_the_setup_from_this_repository() {
        let exe = format!("{BASE}v0.1.4/pecofence-v0.1.4-x64-setup.exe");
        let json = release_json(
            "v0.1.4",
            &[
                (
                    "pecofence-v0.1.4-x64-portable.zip",
                    "https://example.invalid/zip",
                ),
                ("pecofence-v0.1.4-x64-setup.exe", &exe),
            ],
        );
        let release = parse_release(&json, BASE).unwrap();
        assert_eq!(release.version, "0.1.4");
        // Not the response's html_url: that is never opened.
        assert_eq!(
            release.page,
            "https://github.com/DayuanJiang/PecoFence/releases/tag/v0.1.4"
        );
        assert_eq!(
            release.setup,
            Some(SetupAsset {
                name: "pecofence-v0.1.4-x64-setup.exe".into(),
                url: exe,
            })
        );
    }

    #[test]
    fn a_setup_from_elsewhere_is_not_offered() {
        assert_eq!(
            parse_release(&release_json("v0.1.4", &[]), BASE)
                .unwrap()
                .setup,
            None
        );
        let elsewhere = release_json(
            "v0.1.4",
            &[(
                "pecofence-v0.1.4-x64-setup.exe",
                "https://example.invalid/setup.exe",
            )],
        );
        assert_eq!(parse_release(&elsewhere, BASE).unwrap().setup, None);
        let escaping = format!("{BASE}../../../other/repo/releases/download/v0.1.4/");
        let climbing = release_json(
            "v0.1.4",
            &[(
                "pecofence-v0.1.4-x64-setup.exe",
                &format!("{escaping}pecofence-v0.1.4-x64-setup.exe"),
            )],
        );
        assert_eq!(parse_release(&climbing, BASE).unwrap().setup, None);
        assert!(parse_release(&release_json("nightly", &[]), BASE).is_err());
        assert!(parse_release("not json", BASE).is_err());
    }
}
