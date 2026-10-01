//! Release policy only. Network, process and file replacement belong outside core.

use crate::distribution::DistributionMode;
use serde::{Deserialize, Serialize};

pub const METADATA_FILE: &str = "release-info.json";
pub const MAX_PACKAGE_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReleaseInfo {
    pub schema: u32,
    pub repository: String,
    pub version: String,
    pub tag: String,
}

impl ReleaseInfo {
    pub fn parse(text: &str, current: &str) -> Result<Self, String> {
        let info: Self = serde_json::from_str(text.trim_start_matches('\u{feff}'))
            .map_err(|e| format!("Invalid {METADATA_FILE}: {e}"))?;
        if info.schema != 1
            || !valid_repository(&info.repository)
            || info.version != current
            || info.tag != format!("v{current}")
        {
            return Err("Package identity/version does not match this application".into());
        }
        numeric_version(current)?;
        Ok(info)
    }

    pub fn api_url(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/releases/latest",
            self.repository
        )
    }

    pub fn releases_url(&self) -> String {
        format!("https://github.com/{}/releases", self.repository)
    }
}

pub fn valid_repository(value: &str) -> bool {
    let parts: Vec<_> = value.split('/').collect();
    parts.len() == 2
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.len() <= 100
                && *part != "."
                && *part != ".."
                && part
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
        })
}

/// Stable releases are ordered numerically, never lexicographically. A stable
/// version may replace a prerelease of the same numeric version, but not a newer one.
fn numeric_version(value: &str) -> Result<([u32; 3], bool), String> {
    let without_build = value.split_once('+').map_or(value, |(base, _)| base);
    let (base, pre) = without_build
        .split_once('-')
        .map_or((without_build, false), |(base, _)| (base, true));
    if value.is_empty()
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".-+".contains(&c))
    {
        return Err("Invalid release version".into());
    }
    let parts = base.split('.').collect::<Vec<_>>();
    if parts.len() != 3
        || parts.iter().any(|p| {
            p.is_empty()
                || (p.len() > 1 && p.starts_with('0'))
                || !p.bytes().all(|c| c.is_ascii_digit())
        })
    {
        return Err("Expected a major.minor.patch version".into());
    }
    let mut numbers = [0; 3];
    for (index, part) in parts.iter().enumerate() {
        numbers[index] = part.parse().map_err(|_| "Version component is too large")?;
    }
    Ok((numbers, pre))
}

pub fn asset_name(version: &str, mode: DistributionMode) -> Result<String, String> {
    numeric_version(version)?;
    if version.contains(['-', '+']) {
        return Err("Only stable update releases are supported".into());
    }
    let suffix = match mode {
        DistributionMode::Portable => "portable.zip",
        DistributionMode::Installed => "setup.exe",
        _ => return Err("This distribution does not install GitHub updates".into()),
    };
    Ok(format!("pecofence-v{version}-x64-{suffix}"))
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Release {
    pub version: String,
    pub page: String,
    pub asset: String,
    pub url: String,
    pub checksum_url: String,
    pub bytes: u64,
    /// Optional extra integrity check supplied by the GitHub API. The matching
    /// .sha256 asset is always required, even when this digest is available.
    pub digest: Option<String>,
}

impl Release {
    /// Revalidate a saved download before offering it again after restart.
    pub fn validate(&self, info: &ReleaseInfo, mode: DistributionMode) -> Result<(), String> {
        let json = serde_json::json!({
            "tag_name": format!("v{}", self.version), "draft": false, "prerelease": false,
            "assets": [
                {"name":self.asset,"browser_download_url":self.url,"size":self.bytes,"state":"uploaded","digest":self.digest.as_ref().map(|hash|format!("sha256:{hash}"))},
                {"name":format!("{}.sha256",self.asset),"browser_download_url":self.checksum_url,"size":128,"state":"uploaded"}
            ]
        });
        match check_release(info, mode, &json.to_string())? {
            CheckResult::Available(found) if &found == self => Ok(()),
            _ => Err("Saved update is not a newer compatible release".into()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CheckResult {
    UpToDate,
    Available(Release),
}

#[derive(Deserialize)]
struct ApiRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<ApiAsset>,
}

#[derive(Deserialize)]
struct ApiAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    state: String,
    digest: Option<String>,
}

pub fn check_release(
    info: &ReleaseInfo,
    mode: DistributionMode,
    json: &str,
) -> Result<CheckResult, String> {
    let api: ApiRelease =
        serde_json::from_str(json).map_err(|e| format!("Invalid release response: {e}"))?;
    if api.draft || api.prerelease {
        return Err("The latest release is not a published stable release".into());
    }
    let version = api
        .tag_name
        .strip_prefix('v')
        .ok_or("Expected a v-prefixed release tag")?;
    let asset = asset_name(version, mode)?;
    let (remote, _) = numeric_version(version)?;
    let (current, prerelease) = numeric_version(&info.version)?;
    if remote < current || (remote == current && !prerelease) {
        return Ok(CheckResult::UpToDate);
    }
    let checksum = format!("{asset}.sha256");
    let find = |name: &str| -> Result<&ApiAsset, String> {
        let matches = api
            .assets
            .iter()
            .filter(|a| a.name == name)
            .collect::<Vec<_>>();
        if matches.len() != 1 || matches[0].state != "uploaded" {
            return Err(format!("Release is missing one complete {name} asset"));
        }
        Ok(matches[0])
    };
    let package = find(&asset)?;
    let hash_file = find(&checksum)?;
    let base = format!("{}/download/{}", info.releases_url(), api.tag_name);
    if package.browser_download_url != format!("{base}/{asset}")
        || hash_file.browser_download_url != format!("{base}/{checksum}")
        || package.size == 0
        || package.size > MAX_PACKAGE_BYTES
        || hash_file.size == 0
        || hash_file.size > 4096
    {
        return Err("Release asset URL or size failed validation".into());
    }
    let digest = package
        .digest
        .as_deref()
        .map(|value| {
            let hash = value
                .strip_prefix("sha256:")
                .ok_or("Unsupported package digest")?;
            if hash.len() != 64 || !hash.bytes().all(|c| c.is_ascii_hexdigit()) {
                return Err("Invalid package digest");
            }
            Ok(hash.to_ascii_lowercase())
        })
        .transpose()?;
    Ok(CheckResult::Available(Release {
        version: version.into(),
        page: format!("{}/tag/{}", info.releases_url(), api.tag_name),
        asset,
        url: package.browser_download_url.clone(),
        checksum_url: hash_file.browser_download_url.clone(),
        bytes: package.size,
        digest,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn info(version: &str) -> ReleaseInfo {
        ReleaseInfo::parse(&json!({"schema":1,"repository":"Contributor/PecoFence","version":version,"tag":format!("v{version}")}).to_string(), version).unwrap()
    }

    fn response(version: &str, mode: DistributionMode) -> Value {
        let name = asset_name(version, mode).unwrap();
        let base = format!("https://github.com/Contributor/PecoFence/releases/download/v{version}");
        json!({"tag_name":format!("v{version}"),"draft":false,"prerelease":false,"assets":[
            {"name":name,"browser_download_url":format!("{base}/{name}"),"size":1234,"state":"uploaded","digest":format!("sha256:{}", "ab".repeat(32))},
            {"name":format!("{name}.sha256"),"browser_download_url":format!("{base}/{name}.sha256"),"size":120,"state":"uploaded"}
        ]})
    }

    #[test]
    fn metadata_is_bound_to_current_binary_and_repository() {
        let source = info("0.1.3");
        assert_eq!(
            source.api_url(),
            "https://api.github.com/repos/Contributor/PecoFence/releases/latest"
        );
        for repo in [
            "",
            "owner",
            "a/b/c",
            "../b",
            "a/..",
            "a/b?c",
            "a/b#c",
            "a/b%20",
            "a\\b",
            "a/b\n",
            "https://github.com/a/b",
        ] {
            assert!(!valid_repository(repo), "{repo}");
        }
        for changed in [
            json!({"schema":2,"repository":"a/b","version":"0.1.3","tag":"v0.1.3"}),
            json!({"schema":1,"repository":"a/b","version":"0.1.2","tag":"v0.1.2"}),
            json!({"schema":1,"repository":"a/b","version":"0.1.3","tag":"latest"}),
            json!({"schema":1,"repository":"a/b","version":"0.1.3","tag":"v0.1.3","url":"https://elsewhere"}),
        ] {
            assert!(ReleaseInfo::parse(&changed.to_string(), "0.1.3").is_err());
        }
    }

    #[test]
    fn compares_numbers_and_never_downgrades() {
        for (current, remote, newer) in [
            ("0.1.3", "0.1.3", false),
            ("0.2.0", "0.1.9", false),
            ("0.1.9", "0.1.10", true),
            ("0.2.0-rc.1", "0.2.0", true),
            ("0.2.1-rc.1", "0.2.0", false),
        ] {
            let result = check_release(
                &info(current),
                DistributionMode::Portable,
                &response(remote, DistributionMode::Portable).to_string(),
            )
            .unwrap();
            assert_eq!(matches!(result, CheckResult::Available(_)), newer);
        }
    }

    #[test]
    fn selects_the_distribution_and_requires_its_checksum() {
        for mode in [DistributionMode::Portable, DistributionMode::Installed] {
            let result =
                check_release(&info("0.1.3"), mode, &response("0.2.0", mode).to_string()).unwrap();
            let CheckResult::Available(release) = result else {
                panic!("expected update")
            };
            assert_eq!(release.asset, asset_name("0.2.0", mode).unwrap());
            assert_eq!(release.digest, Some("ab".repeat(32)));
        }
        for mode in [
            DistributionMode::Msix,
            DistributionMode::Unmarked,
            DistributionMode::Installed,
        ] {
            assert!(
                check_release(
                    &info("0.1.3"),
                    mode,
                    &response("0.2.0", DistributionMode::Portable).to_string()
                )
                .is_err()
            );
        }
    }

    #[test]
    fn rejects_untrusted_or_incomplete_release_metadata() {
        let original = response("0.2.0", DistributionMode::Portable);
        let mut cases = Vec::new();
        for (field, value) in [
            ("draft", json!(true)),
            ("prerelease", json!(true)),
            ("tag_name", json!("v01.2.0")),
            ("tag_name", json!("v0.2.0-rc.1")),
            ("tag_name", json!("v0.2.0+build")),
            ("tag_name", json!("0.2.0")),
        ] {
            let mut changed = original.clone();
            changed[field] = value;
            cases.push(changed);
        }
        for (field, value) in [
            (
                "browser_download_url",
                json!("https://github.com/Other/PecoFence/releases/download/v0.2.0/file"),
            ),
            ("size", json!(0)),
            ("size", json!(MAX_PACKAGE_BYTES + 1)),
            ("digest", json!("sha256:bad")),
            ("state", json!("new")),
        ] {
            let mut changed = original.clone();
            changed["assets"][0][field] = value;
            cases.push(changed);
        }
        let mut missing = original.clone();
        missing["assets"].as_array_mut().unwrap().pop();
        cases.push(missing);
        let mut duplicate = original.clone();
        duplicate["assets"]
            .as_array_mut()
            .unwrap()
            .push(original["assets"][0].clone());
        cases.push(duplicate);
        for bad in cases {
            assert!(
                check_release(&info("0.1.3"), DistributionMode::Portable, &bad.to_string())
                    .is_err(),
                "{bad}"
            );
        }
    }
}
