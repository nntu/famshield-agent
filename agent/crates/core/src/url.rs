//! URL utilities, shared network defaults, and normalization for NATS and Central Server connections.

pub const DEFAULT_SERVER_URL: &str = "https://famshield.ttcantho.io.vn";
pub const DEFAULT_NATS_URL: &str = "wss://famshield.ttcantho.io.vn/nats";
pub const DEFAULT_VERSION_CHECK_URL: &str = "https://famshield.ttcantho.io.vn/version.json";
pub const DEFAULT_DOWNLOAD_LATEST_URL: &str = "https://famshield.ttcantho.io.vn/download/latest";

pub const DEFAULT_DNS_BIND_IP: &str = "127.0.0.2";
pub const DEFAULT_DNS_BIND_ADDR: &str = "127.0.0.2:53";
pub const DEFAULT_DNS_UPSTREAM_IP: &str = "1.1.1.1";
pub const DEFAULT_DNS_UPSTREAM_ADDR: &str = "1.1.1.1:53";
pub const DEFAULT_FALLBACK_UPSTREAM_IPS: &[&str] = &["8.8.8.8", "9.9.9.9"];

/// Returns `true` when a configuration field uses the `"auto"` sentinel keyword.
#[inline]
pub fn is_auto_keyword(value: &str) -> bool {
    value.trim().eq_ignore_ascii_case("auto")
}

/// Returns `true` for Windows system processes that act as generic DNS forwarders
/// (e.g. `svchost.exe` hosting `Dnscache`), where attribution should fall back to foreground app.
#[inline]
pub fn is_system_dns_forwarder(proc_name: &str) -> bool {
    let base = proc_name.trim();
    base.eq_ignore_ascii_case("svchost.exe")
        || base.eq_ignore_ascii_case("dnscache")
        || base.eq_ignore_ascii_case("services.exe")
        || base.eq_ignore_ascii_case("system")
}

/// Normalize any server or NATS input URL into a valid WebSocket URL for `async-nats`.
///
/// Converts:
/// - `https://famshield.ttcantho.io.vn/nats` -> `wss://famshield.ttcantho.io.vn/nats`
/// - `https://famshield.ttcantho.io.vn` -> `wss://famshield.ttcantho.io.vn/nats`
/// - `http://famshield.ttcantho.io.vn` -> `wss://famshield.ttcantho.io.vn/nats`
/// - `ws://famshield.ttcantho.io.vn:9222` -> `wss://famshield.ttcantho.io.vn/nats`
/// - `http://127.0.0.1:8080` -> `ws://127.0.0.1:9222`
/// - `http://localhost:8080` -> `ws://localhost:9222`
/// - `ws://127.0.0.1:9222` -> `ws://127.0.0.1:9222`
pub fn normalize_nats_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return DEFAULT_NATS_URL.to_string();
    }

    if let Some(rest) = trimmed.strip_prefix("https://") {
        let rest = rest.trim_end_matches('/');
        if rest.ends_with("/nats") {
            format!("wss://{}", rest)
        } else {
            format!("wss://{}/nats", rest)
        }
    } else if let Some(rest) = trimmed.strip_prefix("http://") {
        let host_only = rest
            .split('/')
            .next()
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("");
        if host_only == "127.0.0.1" || host_only == "localhost" {
            format!("ws://{}:9222", host_only)
        } else {
            let cleaned = rest.trim_end_matches('/');
            if cleaned.ends_with("/nats") {
                format!("wss://{}", cleaned)
            } else {
                format!("wss://{}/nats", cleaned)
            }
        }
    } else if let Some(rest) = trimmed.strip_prefix("wss://") {
        let rest = rest.trim_end_matches('/');
        if rest.ends_with("/nats") || rest.contains(':') {
            format!("wss://{}", rest)
        } else {
            format!("wss://{}/nats", rest)
        }
    } else if let Some(rest) = trimmed.strip_prefix("ws://") {
        let rest = rest.trim_end_matches('/');
        let host_only = rest
            .split('/')
            .next()
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("");
        if host_only != "127.0.0.1" && host_only != "localhost" && !rest.ends_with("/nats") {
            format!("wss://{}/nats", host_only)
        } else {
            format!("ws://{}", rest)
        }
    } else {
        let host_only = trimmed
            .split('/')
            .next()
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("");
        if host_only == "127.0.0.1" || host_only == "localhost" {
            format!("ws://{}:9222", host_only)
        } else {
            let cleaned = trimmed.trim_end_matches('/');
            if cleaned.ends_with("/nats") {
                format!("wss://{}", cleaned)
            } else {
                format!("wss://{}/nats", cleaned)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_nats_url() {
        assert_eq!(
            normalize_nats_url("https://famshield.ttcantho.io.vn/nats"),
            "wss://famshield.ttcantho.io.vn/nats"
        );
        assert_eq!(
            normalize_nats_url("https://famshield.ttcantho.io.vn"),
            "wss://famshield.ttcantho.io.vn/nats"
        );
        assert_eq!(
            normalize_nats_url("http://famshield.ttcantho.io.vn"),
            "wss://famshield.ttcantho.io.vn/nats"
        );
        assert_eq!(
            normalize_nats_url("ws://famshield.ttcantho.io.vn:9222"),
            "wss://famshield.ttcantho.io.vn/nats"
        );
        assert_eq!(
            normalize_nats_url("http://127.0.0.1:8080"),
            "ws://127.0.0.1:9222"
        );
        assert_eq!(
            normalize_nats_url("ws://127.0.0.1:9222"),
            "ws://127.0.0.1:9222"
        );
        assert_eq!(
            normalize_nats_url("wss://famshield.ttcantho.io.vn/nats"),
            "wss://famshield.ttcantho.io.vn/nats"
        );
        assert_eq!(
            normalize_nats_url(""),
            "wss://famshield.ttcantho.io.vn/nats"
        );
    }
}
