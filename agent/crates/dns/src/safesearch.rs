//! SafeSearch & YouTube Restricted Mode DNS VIP Enforcer.
//!
//! Enforces strict search mode across major search engines and video platforms
//! by rewriting DNS queries to dedicated provider VIP IP addresses.
//!
//! VIP Targets:
//! - Google: `forcesafesearch.google.com` (216.239.38.120)
//! - Bing: `strict.bing.com` (204.79.197.220)
//! - YouTube: `restrictmoderate.youtube.com` (216.239.38.119)
//! - DuckDuckGo: `safe.duckduckgo.com` (52.142.124.215)

use std::net::Ipv4Addr;

pub const GOOGLE_SAFESEARCH_IPV4: Ipv4Addr = Ipv4Addr::new(216, 239, 38, 120);
pub const BING_SAFESEARCH_IPV4: Ipv4Addr = Ipv4Addr::new(204, 79, 197, 220);
pub const YOUTUBE_RESTRICTED_IPV4: Ipv4Addr = Ipv4Addr::new(216, 239, 38, 119);
pub const DUCKDUCKGO_SAFESEARCH_IPV4: Ipv4Addr = Ipv4Addr::new(52, 142, 124, 215);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeSearchCategory {
    Google,
    Bing,
    YouTube,
    DuckDuckGo,
}

impl SafeSearchCategory {
    pub fn vip_ipv4(&self) -> Ipv4Addr {
        match self {
            Self::Google => GOOGLE_SAFESEARCH_IPV4,
            Self::Bing => BING_SAFESEARCH_IPV4,
            Self::YouTube => YOUTUBE_RESTRICTED_IPV4,
            Self::DuckDuckGo => DUCKDUCKGO_SAFESEARCH_IPV4,
        }
    }

    pub fn display_target(&self) -> &'static str {
        match self {
            Self::Google => "forcesafesearch.google.com",
            Self::Bing => "strict.bing.com",
            Self::YouTube => "restrictmoderate.youtube.com",
            Self::DuckDuckGo => "safe.duckduckgo.com",
        }
    }
}

/// Identifies if a domain matches Google, Bing, YouTube, or DuckDuckGo for SafeSearch enforcement.
pub fn match_safesearch_domain(domain: &str) -> Option<SafeSearchCategory> {
    let norm = domain.trim().trim_end_matches('.').to_ascii_lowercase();

    // 1. YouTube
    if norm == "youtube.com"
        || norm == "www.youtube.com"
        || norm == "m.youtube.com"
        || norm == "youtube-nocookie.com"
        || norm == "www.youtube-nocookie.com"
        || norm == "youtubei.googleapis.com"
        || norm == "youtube.googleapis.com"
        || norm == "restrict.youtube.com"
        || norm == "restrictmoderate.youtube.com"
    {
        return Some(SafeSearchCategory::YouTube);
    }

    // 2. Google Search Frontends (excluding accounts, mail, drive, etc.)
    if is_google_search_domain(&norm) {
        return Some(SafeSearchCategory::Google);
    }

    // 3. Bing Search
    if norm == "bing.com"
        || norm == "www.bing.com"
        || norm == "cn.bing.com"
        || norm == "strict.bing.com"
    {
        return Some(SafeSearchCategory::Bing);
    }

    // 4. DuckDuckGo
    if norm == "duckduckgo.com" || norm == "www.duckduckgo.com" || norm == "safe.duckduckgo.com" {
        return Some(SafeSearchCategory::DuckDuckGo);
    }

    None
}

fn is_google_search_domain(norm: &str) -> bool {
    // Specifically exclude non-search Google subdomains to protect login and services
    if norm.starts_with("accounts.")
        || norm.starts_with("mail.")
        || norm.starts_with("drive.")
        || norm.starts_with("docs.")
        || norm.starts_with("play.")
        || norm.starts_with("meet.")
        || norm.starts_with("chat.")
        || norm.starts_with("classroom.")
        || norm.starts_with("cloud.")
        || norm.starts_with("admin.")
        || norm.starts_with("contacts.")
        || norm.starts_with("calendar.")
        || norm.starts_with("photos.")
        || norm.starts_with("myaccount.")
        || norm.starts_with("apis.")
        || norm.starts_with("client")
        || norm.starts_with("update.")
        || norm.starts_with("fonts.")
    {
        return false;
    }

    norm == "google.com"
        || norm == "www.google.com"
        || norm.starts_with("www.google.")
        || norm.starts_with("google.")
        || norm == "search.google.com"
        || norm == "suggestqueries.google.com"
        || norm == "forcesafesearch.google.com"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_safesearch_google_domains() {
        assert_eq!(
            match_safesearch_domain("google.com"),
            Some(SafeSearchCategory::Google)
        );
        assert_eq!(
            match_safesearch_domain("www.google.com"),
            Some(SafeSearchCategory::Google)
        );
        assert_eq!(
            match_safesearch_domain("www.google.com.vn"),
            Some(SafeSearchCategory::Google)
        );
        assert_eq!(
            match_safesearch_domain("google.fr"),
            Some(SafeSearchCategory::Google)
        );
        // Excluded Google services (Login, Drive, Mail) must NOT be rewritten
        assert_eq!(match_safesearch_domain("accounts.google.com"), None);
        assert_eq!(match_safesearch_domain("drive.google.com"), None);
        assert_eq!(match_safesearch_domain("mail.google.com"), None);
        assert_eq!(match_safesearch_domain("classroom.google.com"), None);
    }

    #[test]
    fn test_match_safesearch_youtube_domains() {
        assert_eq!(
            match_safesearch_domain("youtube.com"),
            Some(SafeSearchCategory::YouTube)
        );
        assert_eq!(
            match_safesearch_domain("www.youtube.com"),
            Some(SafeSearchCategory::YouTube)
        );
        assert_eq!(
            match_safesearch_domain("m.youtube.com"),
            Some(SafeSearchCategory::YouTube)
        );
        assert_eq!(
            match_safesearch_domain("youtubei.googleapis.com"),
            Some(SafeSearchCategory::YouTube)
        );
    }

    #[test]
    fn test_match_safesearch_bing_and_duckduckgo() {
        assert_eq!(
            match_safesearch_domain("www.bing.com"),
            Some(SafeSearchCategory::Bing)
        );
        assert_eq!(
            match_safesearch_domain("duckduckgo.com"),
            Some(SafeSearchCategory::DuckDuckGo)
        );
        assert_eq!(match_safesearch_domain("example.com"), None);
    }
}
