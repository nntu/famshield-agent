//! URL utilities and normalization for NATS and Central Server connections.

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
        return "wss://famshield.ttcantho.io.vn/nats".to_string();
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
