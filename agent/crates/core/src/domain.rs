use std::fmt;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::error::CoreError;

/// Validated, lowercased, punycode-normalized FQDN without trailing dot.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NormalizedDomain(String);

impl NormalizedDomain {
    pub fn parse(raw: &str) -> Result<Self, CoreError> {
        let trimmed = raw.trim().trim_end_matches('.');
        if trimmed.is_empty() || trimmed.len() > 253 {
            return Err(CoreError::InvalidDomain(format!("Invalid length: {}", raw)));
        }

        // Basic character validation and lowercasing
        let lower = trimmed.to_ascii_lowercase();
        for label in lower.split('.') {
            if label.is_empty() || label.len() > 63 {
                return Err(CoreError::InvalidDomain(format!("Invalid label in domain: {}", raw)));
            }
            if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                return Err(CoreError::InvalidDomain(format!("Invalid character in domain: {}", raw)));
            }
            if label.starts_with('-') || label.ends_with('-') {
                return Err(CoreError::InvalidDomain(format!("Label starts/ends with hyphen: {}", raw)));
            }
        }

        Ok(NormalizedDomain(lower))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Iterator over parent domains from most specific to least specific.
    /// E.g. "a.b.example.com" -> ["b.example.com", "example.com", "com"]
    pub fn parent_domains(&self) -> impl Iterator<Item = &str> {
        let mut s = self.0.as_str();
        std::iter::from_fn(move || {
            if let Some(idx) = s.find('.') {
                s = &s[idx + 1..];
                if !s.is_empty() {
                    Some(s)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

impl fmt::Display for NormalizedDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Normalized executable filename, lowercased (e.g., "robloxplayerbeta.exe").
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExecutableName(String);

impl ExecutableName {
    pub fn parse(raw: &str) -> Result<Self, CoreError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.len() > 260 {
            return Err(CoreError::InvalidExecutableIdentity("Invalid executable name length".into()));
        }
        Ok(ExecutableName(trimmed.to_ascii_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Official software categories recognized by ChildProtect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationCategory {
    Game,
    Browser,
    Education,
    Communication,
    Media,
    Productivity,
    Development,
    System,
    Unknown,
}

impl ApplicationCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Game => "game",
            Self::Browser => "browser",
            Self::Education => "education",
            Self::Communication => "communication",
            Self::Media => "media",
            Self::Productivity => "productivity",
            Self::Development => "development",
            Self::System => "system",
            Self::Unknown => "unknown",
        }
    }
}

/// Monotonically measured active usage duration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct MonotonicUsage(Duration);

impl MonotonicUsage {
    pub fn from_duration(d: Duration) -> Self {
        MonotonicUsage(d)
    }

    pub fn from_millis(ms: u64) -> Self {
        MonotonicUsage(Duration::from_millis(ms))
    }

    pub fn as_millis(&self) -> u64 {
        self.0.as_millis() as u64
    }

    pub fn as_duration(&self) -> Duration {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChildId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub String);
