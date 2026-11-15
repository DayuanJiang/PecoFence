//! The one reading of an http or https address that PecoFence both checks and uses: the plugin
//! host checks a plugin's address against its manifest with it, and WinHTTP and the browser get
//! the address it writes back, so the host that was checked is the host that is reached.

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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

/// Whether the address WinHTTP connects a plugin's request to (`93.184.215.14`, `2606:2800::1`,
/// with a port or a zone perhaps) is public. Not public: this computer (127/8, ::1), the local
/// network (10/8, 172.16/12, 192.168/16, 169.254/16, 100.64/10 where Tailscale puts its devices,
/// fd00::/8, fe80::/10), 0/8, multicast (224/4, ff00::/8), 240/4 with the broadcast address, and
/// anything unreadable; 64:ff9b::/96 (NAT64) and 2002::/16 (6to4) as the IPv4 address inside
/// them. A name a plugin declared may resolve, or be made to resolve, to these: the router and
/// the computer's own services must stay out of its reach. Public on purpose, as Chromium counts
/// them: 198.18/15 and fc00::/8 (the fake-ip answers of proxy tools such as Clash and sing-box).
pub fn public_ip(text: &str) -> bool {
    let ip = text.parse::<SocketAddr>().map(|a| a.ip()).or_else(|_| {
        let bare = text.split('%').next().unwrap_or_default();
        bare.trim_matches(['[', ']']).parse::<IpAddr>()
    });
    let v4 = |ip: Ipv4Addr| {
        let [a, b, ..] = ip.octets();
        let shared = a == 100 && (64..128).contains(&b);
        !(ip.is_loopback() || ip.is_private() || ip.is_link_local() || shared)
            && !(a == 0 || ip.is_multicast() || a >= 240)
    };
    match ip {
        Ok(IpAddr::V4(ip)) => v4(ip),
        Ok(IpAddr::V6(ip)) => {
            let s = ip.segments();
            let inside = |hi: u16, lo: u16| Ipv4Addr::from((u32::from(hi) << 16) | u32::from(lo));
            if let Some(mapped) = ip.to_ipv4_mapped() {
                v4(mapped)
            } else if s[..6] == [0x64, 0xff9b, 0, 0, 0, 0] {
                v4(inside(s[6], s[7]))
            } else if s[0] == 0x2002 {
                v4(inside(s[1], s[2]))
            } else {
                !(ip.is_loopback() || ip.is_unspecified() || ip.is_multicast())
                    && !(s[0] & 0xff00 == 0xfd00 || ip.is_unicast_link_local())
            }
        }
        Err(_) => false,
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
    fn only_public_addresses_are_public() {
        let public = "93.184.215.14 140.82.112.6:443 2606:2800:21f:cb07::1 [2606:2800::1]:443 \
            172.32.0.1 100.63.255.255 100.128.0.1 198.18.0.5 198.19.255.1 fc00::1 fc00:4000::5 \
            64:ff9b::808:808 2002:808:808::1 223.255.255.1";
        for ip in public.split_whitespace() {
            assert!(public_ip(ip), "{ip}");
        }
        let not = "127.0.0.1 127.1.2.3:443 10.0.0.1 172.16.0.1 172.31.255.255 192.168.1.1 \
            169.254.169.254 0.0.0.0 0.1.2.3 255.255.255.255 ::1 [::1]:443 :: fd00::1 fd12:3456::1 \
            fe80::1 fe80::1%3 [fe80::1%3]:80 ::ffff:127.0.0.1 ::ffff:192.168.0.1 localhost 1.2.3 \
            100.64.0.1 100.101.102.103 100.127.255.255 224.0.0.251 239.255.255.250 240.0.0.1 \
            ff02::1 ff05::fb 64:ff9b::7f00:1 64:ff9b::c0a8:101 2002:c0a8:101::1 2002:7f00:1::";
        for ip in not.split_whitespace().chain([""]) {
            assert!(!public_ip(ip), "{ip}");
        }
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
