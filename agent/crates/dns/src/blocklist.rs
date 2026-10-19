use std::collections::HashSet;
use std::sync::Arc;
use arc_swap::ArcSwap;

#[derive(Clone, Default)]
pub struct BlocklistStore {
    // Exact domain or parent suffix set (e.g., "tiktok.com")
    domains: Arc<ArcSwap<HashSet<String>>>,
}

impl BlocklistStore {
    pub fn new() -> Self {
        Self {
            domains: Arc::new(ArcSwap::from_pointee(HashSet::new())),
        }
    }

    /// Load or atomically replace the blocked domains list without locking reader threads.
    pub fn replace_blocklist<I, S>(&self, iter: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut set = HashSet::new();
        for item in iter {
            let s = item.as_ref().trim().trim_end_matches('.').to_ascii_lowercase();
            if !s.is_empty() {
                set.insert(s);
            }
        }
        self.domains.store(Arc::new(set));
    }

    /// Check if target domain matches blocklist via exact match or parent subdomain suffix.
    pub fn is_blocked(&self, target_domain: &str) -> bool {
        let current = self.domains.load();
        let norm = target_domain.trim().trim_end_matches('.').to_ascii_lowercase();

        if current.contains(&norm) {
            return true;
        }

        // Subdomain traversal: e.g. "v16.tiktok.com" -> check "tiktok.com", "com"
        let mut s = norm.as_str();
        while let Some(idx) = s.find('.') {
            s = &s[idx + 1..];
            if current.contains(s) {
                return true;
            }
        }

        false
    }
}
