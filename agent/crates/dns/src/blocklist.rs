use arc_swap::ArcSwap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// Essential core domains that must never be blocked under any circumstances.
const DEFAULT_CORE_WHITELIST: &[&str] = &[
    "windowsupdate.com",
    "update.microsoft.com",
    "msftconnecttest.com",
    "msftncsi.com",
    "apple.com",
    "icloud.com",
    "google.com",
    "gstatic.com",
    "cloudflare.com",
    "one.one.one.one",
    "dns.google",
    "digicert.com",
    "letsencrypt.org",
    "zalo.me",
    "teams.microsoft.com",
    "zoom.us",
    "vietcombank.com.vn",
    "techcombank.com.vn",
    "momo.vn",
    "gov.vn",
    "edu.vn",
];

#[derive(Clone)]
pub struct BlocklistStore {
    // General threat feeds / domain sinkholes (e.g. 100k - 1M domains)
    domains: Arc<ArcSwap<HashSet<String>>>,
    // Client custom blocked domains (exact domain or parent suffix, e.g. "tiktok.com")
    custom_blocked: Arc<ArcSwap<HashSet<String>>>,
    // Name / brand patterns without dots (e.g. "thiendia" matches "thiendia.us", "thiendia.vn")
    name_patterns: Arc<ArcSwap<HashSet<String>>>,
    // Custom allowed domains / whitelist (ABSOLUTE HIGHEST PRIORITY)
    whitelist: Arc<ArcSwap<HashSet<String>>>,
    // Core system whitelist
    core_whitelist: Arc<ArcSwap<HashSet<String>>>,
    monitor_mode: Arc<AtomicBool>,
    safesearch_enabled: Arc<AtomicBool>,
    youtube_restricted: Arc<AtomicBool>,
    paused_until: Arc<AtomicU64>,
}

impl Default for BlocklistStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BlocklistStore {
    pub fn new() -> Self {
        let mut core_set = HashSet::new();
        for &d in DEFAULT_CORE_WHITELIST {
            core_set.insert(d.to_string());
        }

        Self {
            domains: Arc::new(ArcSwap::from_pointee(HashSet::new())),
            custom_blocked: Arc::new(ArcSwap::from_pointee(HashSet::new())),
            name_patterns: Arc::new(ArcSwap::from_pointee(HashSet::new())),
            whitelist: Arc::new(ArcSwap::from_pointee(HashSet::new())),
            core_whitelist: Arc::new(ArcSwap::from_pointee(core_set)),
            monitor_mode: Arc::new(AtomicBool::new(false)),
            safesearch_enabled: Arc::new(AtomicBool::new(true)),
            youtube_restricted: Arc::new(AtomicBool::new(true)),
            paused_until: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Enable or disable SafeSearch enforcement (Google, Bing, DuckDuckGo).
    pub fn set_safesearch_enabled(&self, enabled: bool) {
        self.safesearch_enabled.store(enabled, Ordering::SeqCst);
    }

    /// Check if SafeSearch is currently active.
    pub fn is_safesearch_enabled(&self) -> bool {
        self.safesearch_enabled.load(Ordering::SeqCst)
    }

    /// Enable or disable YouTube Restricted Mode.
    pub fn set_youtube_restricted(&self, enabled: bool) {
        self.youtube_restricted.store(enabled, Ordering::SeqCst);
    }

    /// Check if YouTube Restricted Mode is currently active.
    pub fn is_youtube_restricted(&self) -> bool {
        self.youtube_restricted.load(Ordering::SeqCst)
    }

    /// Enable or disable Monitor-only Mode (Audit mode: log only, zero blocking).
    pub fn set_monitor_mode(&self, enabled: bool) {
        self.monitor_mode.store(enabled, Ordering::SeqCst);
    }

    /// Check if Monitor Mode is currently active.
    pub fn is_monitor_mode(&self) -> bool {
        self.monitor_mode.load(Ordering::SeqCst)
    }

    /// Temporarily pause blocking for `seconds`. Blocking automatically resumes when timer expires.
    /// Adopted from numa's auto-resume pause to eliminate human error when users disable protection.
    pub fn pause(&self, seconds: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.paused_until.store(now + seconds, Ordering::SeqCst);
    }

    /// Immediately resume blocking (unpause).
    pub fn unpause(&self) {
        self.paused_until.store(0, Ordering::SeqCst);
    }

    /// Check if blocking is currently paused.
    pub fn is_paused(&self) -> bool {
        let until = self.paused_until.load(Ordering::SeqCst);
        if until == 0 {
            return false;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now < until
    }

    /// Remaining seconds of pause, or 0 if not paused.
    pub fn paused_remaining_secs(&self) -> u64 {
        let until = self.paused_until.load(Ordering::SeqCst);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        until.saturating_sub(now)
    }

    /// Check if target domain matches Whitelist (either user whitelist or core system whitelist).
    /// Whitelist has the HIGHEST priority and automatically covers subdomains (e.g., "mail.thiendia.com").
    pub fn is_whitelisted(&self, target_domain: &str) -> bool {
        let norm = target_domain
            .trim()
            .trim_end_matches('.')
            .to_ascii_lowercase();
        if norm.is_empty() {
            return false;
        }

        let wl = self.whitelist.load();
        let core = self.core_whitelist.load();

        if wl.contains(&norm) || core.contains(&norm) {
            return true;
        }

        let mut s = norm.as_str();
        while let Some(idx) = s.find('.') {
            s = &s[idx + 1..];
            if wl.contains(s) || core.contains(s) {
                return true;
            }
        }

        false
    }

    /// Check if target domain matches custom client blocked list (exact or parent subdomain).
    pub fn is_custom_blocked(&self, target_domain: &str) -> bool {
        let norm = target_domain
            .trim()
            .trim_end_matches('.')
            .to_ascii_lowercase();
        if norm.is_empty() {
            return false;
        }

        let custom = self.custom_blocked.load();
        if custom.contains(&norm) {
            return true;
        }

        let mut s = norm.as_str();
        while let Some(idx) = s.find('.') {
            s = &s[idx + 1..];
            if custom.contains(s) {
                return true;
            }
        }

        false
    }

    /// Check if target domain matches any name/brand pattern without dots (e.g. "thiendia").
    /// Matches if any label in domain equals the pattern (e.g. "thiendia.us", "thiendia.vn")
    /// or if a label contains the brand name (e.g. "thiendia-vip.com", "phim-thiendia.com").
    pub fn matches_name_pattern(&self, target_domain: &str) -> bool {
        let norm = target_domain
            .trim()
            .trim_end_matches('.')
            .to_ascii_lowercase();
        if norm.is_empty() {
            return false;
        }

        let patterns = self.name_patterns.load();
        if patterns.is_empty() {
            return false;
        }

        let labels: Vec<&str> = norm.split('.').collect();
        for pat in patterns.iter() {
            for label in &labels {
                if *label == pat.as_str()
                    || label.starts_with(&format!("{}-", pat))
                    || label.ends_with(&format!("-{}", pat))
                {
                    return true;
                }
                if pat.len() >= 4 && label.contains(pat.as_str()) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if target domain matches general threat blocklist.
    pub fn is_in_general_blocklist(&self, target_domain: &str) -> bool {
        let norm = target_domain
            .trim()
            .trim_end_matches('.')
            .to_ascii_lowercase();
        if norm.is_empty() {
            return false;
        }

        let general = self.domains.load();
        if general.contains(&norm) {
            return true;
        }

        let mut s = norm.as_str();
        while let Some(idx) = s.find('.') {
            s = &s[idx + 1..];
            if general.contains(s) {
                return true;
            }
        }

        false
    }

    /// Check if target domain should be blocked according to hierarchical priority:
    /// 1. Whitelist (HIGHEST PRIORITY): If domain or parent is whitelisted, NEVER blocked!
    /// 2. Client Custom Blocked Domains (Priority 2)
    /// 3. Name / Brand Patterns (Priority 3: e.g. "thiendia" blocks "thiendia.us", "thiendia.vn")
    /// 4. General threat/sinkhole blocklist (Priority 4)
    pub fn is_blocked(&self, target_domain: &str) -> bool {
        let norm = target_domain
            .trim()
            .trim_end_matches('.')
            .to_ascii_lowercase();
        if norm.is_empty() {
            return false;
        }

        // PRIORITY 1: Whitelist is absolute — if whitelisted, never block!
        if self.is_whitelisted(&norm) {
            return false;
        }

        // PRIORITY 2: Client Custom Blocked Domains
        if self.is_custom_blocked(&norm) {
            return true;
        }

        // PRIORITY 3: Name / Brand Patterns
        if self.matches_name_pattern(&norm) {
            return true;
        }

        // PRIORITY 4: General Blocklist (threat feeds, sinkholes)
        if self.is_in_general_blocklist(&norm) {
            return true;
        }

        false
    }

    /// Check if target domain should actually be blocked (blocked AND not in monitor mode AND not paused).
    pub fn should_block(&self, target_domain: &str) -> bool {
        !self.is_monitor_mode() && !self.is_paused() && self.is_blocked(target_domain)
    }

    /// Human-readable block reason for auditing and telemetry logging.
    pub fn block_reason(&self, target_domain: &str) -> Option<&'static str> {
        let norm = target_domain
            .trim()
            .trim_end_matches('.')
            .to_ascii_lowercase();
        if self.is_whitelisted(&norm) {
            return None;
        }
        if self.is_custom_blocked(&norm) {
            return Some("custom_domain_rule");
        }
        if self.matches_name_pattern(&norm) {
            return Some("name_pattern_rule");
        }
        if self.is_in_general_blocklist(&norm) {
            return Some("threat_blocklist");
        }
        None
    }

    /// Atomically replace the whitelist (allowed domains).
    pub fn replace_whitelist<I, S>(&self, iter: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut set = HashSet::new();
        for item in iter {
            let s = item
                .as_ref()
                .trim()
                .trim_end_matches('.')
                .to_ascii_lowercase();
            if !s.is_empty() {
                set.insert(s);
            }
        }
        self.whitelist.store(Arc::new(set));
    }

    /// Atomically replace client custom blocked rules (domains and name patterns).
    pub fn replace_custom_blocked<I, S>(&self, iter: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut domain_set = HashSet::new();
        let mut pattern_set = HashSet::new();
        for item in iter {
            let s = item
                .as_ref()
                .trim()
                .trim_end_matches('.')
                .to_ascii_lowercase();
            if s.is_empty() {
                continue;
            }
            if s.contains('.') {
                domain_set.insert(s);
            } else {
                pattern_set.insert(s);
            }
        }
        self.custom_blocked.store(Arc::new(domain_set));
        self.name_patterns.store(Arc::new(pattern_set));
    }

    /// Load or atomically replace the general blocked domains list without locking reader threads.
    /// Automatically deduplicates exact records and prunes redundant subdomains when parent domain is blocked.
    /// Any rule without dots is seamlessly loaded into name_patterns.
    pub fn replace_blocklist<I, S>(&self, iter: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        // 1. Separate domains with dots and name patterns without dots
        let mut raw_set = HashSet::new();
        let mut patterns = HashSet::new();
        for item in iter {
            let s = item
                .as_ref()
                .trim()
                .trim_end_matches('.')
                .to_ascii_lowercase();
            if s.is_empty() {
                continue;
            }
            if s.contains('.') {
                raw_set.insert(s);
            } else {
                patterns.insert(s);
            }
        }

        // 2. Subdomain Pruning: If parent domain is already present, eliminate child subdomains
        let mut pruned_set = HashSet::with_capacity(raw_set.len());
        for domain in &raw_set {
            let mut is_redundant = false;
            let mut s = domain.as_str();
            while let Some(idx) = s.find('.') {
                s = &s[idx + 1..];
                if raw_set.contains(s) {
                    is_redundant = true;
                    break;
                }
            }
            if !is_redundant {
                pruned_set.insert(domain.clone());
            }
        }

        self.domains.store(Arc::new(pruned_set));
        if !patterns.is_empty() {
            let mut existing = (**self.name_patterns.load()).clone();
            existing.extend(patterns);
            self.name_patterns.store(Arc::new(existing));
        }
    }

    /// Number of active unique rules loaded in blocklist (domains + custom + patterns).
    pub fn len(&self) -> usize {
        self.domains.load().len()
            + self.custom_blocked.load().len()
            + self.name_patterns.load().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Exact heap memory consumption of all hash sets in bytes (numa model).
    pub fn heap_bytes(&self) -> usize {
        let per_slot_overhead = std::mem::size_of::<u64>() + std::mem::size_of::<String>() + 1;
        let sets = [
            self.domains.load(),
            self.custom_blocked.load(),
            self.name_patterns.load(),
            self.whitelist.load(),
            self.core_whitelist.load(),
        ];
        sets.iter()
            .map(|s| {
                s.capacity() * per_slot_overhead + s.iter().map(|d| d.capacity()).sum::<usize>()
            })
            .sum()
    }
}
