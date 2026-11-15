//! The one reading of an http or https address that PecoFence both checks and uses: the plugin
//! host checks a plugin's address against its manifest with it, and WinHTTP and the browser get
//! the address it writes back, so the host that was checked is the host that is reached.

use std::fmt;

/// An http or https address read strictly: the host is letters, digits, `.` and `-` (kept in
/// lowercase, without a dot at the end), with an optional port; no user name (`@`) and no
/// backslash before the path, no blank in the host and no control character anywhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebAddress {
    pub secure: bool,
    pub host: String,
    pub port: Option<u16>,
    /// Everything from the first `/`, `?` or `#` on, starting with `/`.
    pub path: String,
}

impl WebAddress {
    /// None for anything else, or anything a browser could read with another host.
    pub fn parse(url: &str) -> Option<Self> {
        let (secure, rest) = match url.strip_prefix("https://") {
            Some(rest) => (true, rest),
            None => (false, url.strip_prefix("http://")?),
        };
        if url.chars().any(char::is_control) {
            return None;
        }
        let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
        let (authority, path) = rest.split_at(end);
        let (host, port) = match authority.split_once(':') {
            Some((host, port)) => {
                let digits =
                    (1..=5).contains(&port.len()) && port.bytes().all(|b| b.is_ascii_digit());
                (
                    host,
                    Some(port.parse::<u16>().ok().filter(|p| digits && *p > 0)?),
                )
            }
            None => (authority, None),
        };
        let host = host.trim_end_matches('.').to_ascii_lowercase();
        let plain = host
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-');
        if !plain || host.split('.').any(str::is_empty) {
            return None;
        }
        let path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        Some(Self {
            secure,
            host,
            port,
            path,
        })
    }
}

impl fmt::Display for WebAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scheme = if self.secure { "https" } else { "http" };
        write!(f, "{scheme}://{}", self.host)?;
        if let Some(port) = self.port {
            write!(f, ":{port}")?;
        }
        f.write_str(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(url: &str) -> Option<String> {
        WebAddress::parse(url).map(|a| a.to_string())
    }

    #[test]
    fn plain_addresses_are_read_and_written_back_normalised() {
        let a = WebAddress::parse("https://API.GitHub.com./repos/x?y=1#z").unwrap();
        assert_eq!(
            (a.secure, a.host.as_str(), a.port, a.path.as_str()),
            (true, "api.github.com", None, "/repos/x?y=1#z")
        );
        assert_eq!(a.to_string(), "https://api.github.com/repos/x?y=1#z");
        assert_eq!(
            read("http://127.0.0.1:8080").as_deref(),
            Some("http://127.0.0.1:8080/")
        );
        assert_eq!(
            read("https://a.com?q=1").as_deref(),
            Some("https://a.com/?q=1")
        );
        // An @ after the host is part of the path (jsDelivr's addresses have one).
        assert_eq!(
            read("https://cdn.jsdelivr.net/gh/a/b@1/x").as_deref(),
            Some("https://cdn.jsdelivr.net/gh/a/b@1/x")
        );
        assert_eq!(
            read("https://a.com/x\\y z").as_deref(),
            Some("https://a.com/x\\y z")
        );
    }

    #[test]
    fn addresses_a_browser_could_read_with_another_host_are_refused() {
        for bad in [
            "https://evil.example\u{0}@api.github.com/",
            "https://evil.com\\@github.com/",
            "https://evil.com@github.com/",
            "https://user:pw@github.com/",
            "https://git\thub.com/",
            "https://github.com /",
            "https://github.com\u{3000}/",
            "https://github.com/\n",
            "https://github.com\\evil.com/",
            "https://[::1]/",
            "https://bücher.de/",
            "https://a..com/",
            "https://.a.com/",
            "https:///x",
            "https://:443/",
            "https://a.com:/",
            "https://a.com:0/",
            "https://a.com:65536/",
            "https://a.com:+80/",
            "https://a.com:80:80/",
            "ftp://a.com/",
            "HTTPS://a.com/",
            "javascript:alert(1)",
            "file:///C:/Windows",
            "",
        ] {
            assert_eq!(read(bad), None, "{bad:?}");
        }
    }
}
