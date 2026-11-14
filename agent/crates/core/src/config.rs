//! Optional file-based default configuration (`config.json`) with safe validation
//! and fallback to compiled constants in [`crate::url`].
//!
//! Precedence model:
//! 1. Runtime settings stored in SQLite (`meta` table via Local Admin UI / Cloud / bootstrap).
//! 2. Optional `<data_dir>/config.json` loaded into [`AgentConfig`].
//! 3. Compiled fallback constants in `famshield-core`.

use crate::url::{
    is_auto_keyword, normalize_nats_url, DEFAULT_DNS_BIND_IP, DEFAULT_DNS_UPSTREAM_IP,
    DEFAULT_FALLBACK_UPSTREAM_IPS, DEFAULT_SERVER_URL,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::path::Path;

pub const DEFAULT_CONFIG_FILE_NAME: &str = "config.json";
pub const DEFAULT_DNS_BIND_PORT: u16 = 53;
pub const DEFAULT_SYSTEM_DNS_FORWARDERS: &[&str] =
    &["svchost.exe", "dnscache", "services.exe", "system"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentConfig {
    pub server_url: String,
    pub nats_url: Option<String>,
    pub version_check_url: Option<String>,
    pub download_latest_url: Option<String>,
    pub dns_bind_ip: String,
    pub dns_bind_port: u16,
    pub dns_upstream: String,
    pub fallback_upstreams: Vec<String>,
    pub system_dns_forwarders: Vec<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            server_url: DEFAULT_SERVER_URL.to_string(),
            nats_url: None,
            version_check_url: None,
            download_latest_url: None,
            dns_bind_ip: DEFAULT_DNS_BIND_IP.to_string(),
            dns_bind_port: DEFAULT_DNS_BIND_PORT,
            dns_upstream: DEFAULT_DNS_UPSTREAM_IP.to_string(),
            fallback_upstreams: DEFAULT_FALLBACK_UPSTREAM_IPS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            system_dns_forwarders: DEFAULT_SYSTEM_DNS_FORWARDERS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        }
    }
}

fn is_http_url(s: &str) -> bool {
    let t = s.trim();
    t.starts_with("http://") || t.starts_with("https://")
}

fn is_ws_or_http_url(s: &str) -> bool {
    let t = s.trim();
    is_http_url(t) || t.starts_with("ws://") || t.starts_with("wss://")
}

fn is_valid_ip_or_socket(s: &str) -> bool {
    let t = s.trim();
    t.parse::<IpAddr>().is_ok() || t.parse::<SocketAddr>().is_ok()
}

fn sanitize_optional_url(slot: &mut Option<String>, allow_ws: bool) {
    let valid = match slot.as_deref() {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() || is_auto_keyword(trimmed) {
                None
            } else if (allow_ws && is_ws_or_http_url(trimmed))
                || (!allow_ws && is_http_url(trimmed))
            {
                Some(trimmed.to_string())
            } else {
                None
            }
        }
        None => None,
    };
    *slot = valid;
}

impl AgentConfig {
    /// Parse JSON into [`AgentConfig`] and sanitize invalid fields to safe defaults.
    pub fn from_json_str(raw: &str) -> Result<Self, String> {
        let mut cfg: Self =
            serde_json::from_str(raw).map_err(|e| format!("invalid config JSON: {}", e))?;
        cfg.validate_and_sanitize();
        Ok(cfg)
    }

    /// Load [`AgentConfig`] from `path`, returning `(config, Some(warning))` on read/parse error
    /// and `(Self::default(), None)` if the file does not exist. Never panics.
    pub fn load_with_diagnostics(path: &Path) -> (Self, Option<String>) {
        if !path.exists() {
            return (Self::default(), None);
        }
        match std::fs::read_to_string(path) {
            Ok(content) => match Self::from_json_str(&content) {
                Ok(cfg) => (cfg, None),
                Err(err) => (
                    Self::default(),
                    Some(format!(
                        "Failed to parse config file '{}': {}. Using built-in defaults.",
                        path.display(),
                        err
                    )),
                ),
            },
            Err(err) => (
                Self::default(),
                Some(format!(
                    "Failed to read config file '{}': {}. Using built-in defaults.",
                    path.display(),
                    err
                )),
            ),
        }
    }

    /// Load [`AgentConfig`] from `path`, or create a default `config.json` if it does not exist.
    /// Returns `(config, Some(warning))` on I/O or parse errors, and never panics.
    pub fn load_or_create_with_diagnostics(path: &Path) -> (Self, Option<String>) {
        if !path.exists() {
            let cfg = Self::default();
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    if let Err(err) = std::fs::create_dir_all(parent) {
                        return (
                            cfg,
                            Some(format!(
                                "Failed to create config directory '{}': {}. Using built-in defaults.",
                                parent.display(),
                                err
                            )),
                        );
                    }
                }
            }
            match serde_json::to_string_pretty(&cfg) {
                Ok(json) => {
                    if let Err(err) = std::fs::write(path, format!("{}\n", json)) {
                        return (
                            cfg,
                            Some(format!(
                                "Failed to write default config file '{}': {}. Using built-in defaults.",
                                path.display(),
                                err
                            )),
                        );
                    }
                }
                Err(err) => {
                    return (
                        cfg,
                        Some(format!(
                            "Failed to serialize default config '{}': {}. Using built-in defaults.",
                            path.display(),
                            err
                        )),
                    );
                }
            }
            return (cfg, None);
        }

        Self::load_with_diagnostics(path)
    }

    /// Load [`AgentConfig`] from `path`, silently falling back to [`Self::default`] on error.
    pub fn load_or_default(path: &Path) -> Self {
        Self::load_with_diagnostics(path).0
    }

    /// Validate all fields and reset any malformed values back to compiled defaults.
    pub fn validate_and_sanitize(&mut self) {
        let server_trimmed = self.server_url.trim().to_string();
        if is_http_url(&server_trimmed) {
            self.server_url = server_trimmed;
        } else {
            self.server_url = DEFAULT_SERVER_URL.to_string();
        }

        sanitize_optional_url(&mut self.nats_url, true);
        sanitize_optional_url(&mut self.version_check_url, false);
        sanitize_optional_url(&mut self.download_latest_url, false);

        let bind_trimmed = self.dns_bind_ip.trim().to_string();
        if bind_trimmed.parse::<IpAddr>().is_ok() {
            self.dns_bind_ip = bind_trimmed;
        } else {
            self.dns_bind_ip = DEFAULT_DNS_BIND_IP.to_string();
        }

        if self.dns_bind_port == 0 {
            self.dns_bind_port = DEFAULT_DNS_BIND_PORT;
        }

        let upstream_trimmed = self.dns_upstream.trim().to_string();
        if is_valid_ip_or_socket(&upstream_trimmed) {
            self.dns_upstream = upstream_trimmed;
        } else {
            self.dns_upstream = DEFAULT_DNS_UPSTREAM_IP.to_string();
        }

        let valid_fallbacks: Vec<String> = self
            .fallback_upstreams
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| is_valid_ip_or_socket(s))
            .collect();
        if valid_fallbacks.is_empty() {
            self.fallback_upstreams = DEFAULT_FALLBACK_UPSTREAM_IPS
                .iter()
                .map(|s| (*s).to_string())
                .collect();
        } else {
            self.fallback_upstreams = valid_fallbacks;
        }

        let valid_forwarders: Vec<String> = self
            .system_dns_forwarders
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if valid_forwarders.is_empty() {
            self.system_dns_forwarders = DEFAULT_SYSTEM_DNS_FORWARDERS
                .iter()
                .map(|s| (*s).to_string())
                .collect();
        } else {
            self.system_dns_forwarders = valid_forwarders;
        }
    }

    pub fn dns_bind_addr(&self) -> String {
        format!("{}:{}", self.dns_bind_ip, self.dns_bind_port)
    }

    pub fn dns_upstream_addr(&self) -> String {
        if self.dns_upstream.parse::<SocketAddr>().is_ok() {
            self.dns_upstream.clone()
        } else {
            format!("{}:53", self.dns_upstream)
        }
    }

    pub fn effective_version_check_url(&self) -> String {
        self.version_check_url
            .clone()
            .unwrap_or_else(|| format!("{}/version.json", self.server_url.trim_end_matches('/')))
    }

    pub fn effective_download_latest_url(&self) -> String {
        self.download_latest_url
            .clone()
            .unwrap_or_else(|| format!("{}/download/latest", self.server_url.trim_end_matches('/')))
    }

    pub fn effective_nats_url(&self) -> String {
        match self.nats_url.as_deref() {
            Some(url) => normalize_nats_url(url),
            None => normalize_nats_url(&self.server_url),
        }
    }

    pub fn is_system_dns_forwarder(&self, proc_name: &str) -> bool {
        let base = proc_name.trim();
        self.system_dns_forwarders
            .iter()
            .any(|f| f.eq_ignore_ascii_case(base))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::url::{
        DEFAULT_DNS_BIND_ADDR, DEFAULT_DNS_UPSTREAM_ADDR, DEFAULT_DOWNLOAD_LATEST_URL,
        DEFAULT_NATS_URL, DEFAULT_VERSION_CHECK_URL,
    };

    #[test]
    fn default_config_matches_url_constants() {
        let cfg = AgentConfig::default();
        assert_eq!(cfg.server_url, DEFAULT_SERVER_URL);
        assert_eq!(cfg.dns_bind_addr(), DEFAULT_DNS_BIND_ADDR);
        assert_eq!(cfg.dns_upstream_addr(), DEFAULT_DNS_UPSTREAM_ADDR);
        assert_eq!(cfg.effective_version_check_url(), DEFAULT_VERSION_CHECK_URL);
        assert_eq!(
            cfg.effective_download_latest_url(),
            DEFAULT_DOWNLOAD_LATEST_URL
        );
        assert_eq!(cfg.effective_nats_url(), DEFAULT_NATS_URL);
        assert!(cfg.is_system_dns_forwarder("svchost.exe"));
        assert!(cfg.is_system_dns_forwarder("SVCHOST.EXE"));
        assert!(!cfg.is_system_dns_forwarder("chrome.exe"));
    }

    #[test]
    fn partial_json_overrides_only_specified_fields() {
        let json = r#"{
            "server_url": "https://family.example.com/",
            "dns_bind_ip": "127.0.0.10",
            "dns_bind_port": 5353,
            "dns_upstream": "9.9.9.9"
        }"#;
        let cfg = AgentConfig::from_json_str(json).expect("valid partial config");
        assert_eq!(cfg.server_url, "https://family.example.com/");
        assert_eq!(cfg.dns_bind_addr(), "127.0.0.10:5353");
        assert_eq!(cfg.dns_upstream_addr(), "9.9.9.9:53");
        assert_eq!(
            cfg.effective_version_check_url(),
            "https://family.example.com/version.json"
        );
        assert_eq!(cfg.effective_nats_url(), "wss://family.example.com/nats");
        assert_eq!(cfg.fallback_upstreams, vec!["8.8.8.8", "9.9.9.9"]);
    }

    #[test]
    fn invalid_fields_are_sanitized_to_defaults() {
        let json = r#"{
            "server_url": "ftp://invalid.example.com",
            "nats_url": "auto",
            "version_check_url": "not-a-url",
            "dns_bind_ip": "not-an-ip",
            "dns_bind_port": 0,
            "dns_upstream": "bad-ip",
            "fallback_upstreams": ["bad", "1.0.0.1"],
            "system_dns_forwarders": []
        }"#;
        let cfg = AgentConfig::from_json_str(json).expect("sanitized config");
        assert_eq!(cfg.server_url, DEFAULT_SERVER_URL);
        assert_eq!(cfg.nats_url, None);
        assert_eq!(cfg.version_check_url, None);
        assert_eq!(cfg.dns_bind_ip, DEFAULT_DNS_BIND_IP);
        assert_eq!(cfg.dns_bind_port, DEFAULT_DNS_BIND_PORT);
        assert_eq!(cfg.dns_upstream, DEFAULT_DNS_UPSTREAM_IP);
        assert_eq!(cfg.fallback_upstreams, vec!["1.0.0.1"]);
        assert!(cfg.is_system_dns_forwarder("svchost.exe"));
    }

    #[test]
    fn load_missing_or_malformed_file_falls_back_gracefully() {
        let missing = Path::new("/tmp/famshield_nonexistent_config_12345.json");
        let (cfg, diag) = AgentConfig::load_with_diagnostics(missing);
        assert_eq!(cfg, AgentConfig::default());
        assert!(diag.is_none());

        let tmp = std::env::temp_dir().join("famshield_bad_config_test.json");
        std::fs::write(&tmp, "{ bad json").unwrap();
        let (cfg_bad, diag_bad) = AgentConfig::load_with_diagnostics(&tmp);
        let _ = std::fs::remove_file(&tmp);
        assert_eq!(cfg_bad, AgentConfig::default());
        assert!(diag_bad.is_some());
    }

    #[test]
    fn load_or_create_writes_default_file_when_missing_and_preserves_existing() {
        let dir = std::env::temp_dir().join("famshield_create_config_test");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("config.json");

        let (created, diag) = AgentConfig::load_or_create_with_diagnostics(&path);
        assert!(diag.is_none());
        assert!(path.exists());
        assert_eq!(created, AgentConfig::default());

        let (reloaded, diag2) = AgentConfig::load_or_create_with_diagnostics(&path);
        assert!(diag2.is_none());
        assert_eq!(reloaded, AgentConfig::default());

        std::fs::write(&path, r#"{"dns_bind_port": 5353}"#).unwrap();
        let (custom, diag3) = AgentConfig::load_or_create_with_diagnostics(&path);
        assert!(diag3.is_none());
        assert_eq!(custom.dns_bind_port, 5353);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
