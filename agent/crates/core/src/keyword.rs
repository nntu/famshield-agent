use aho_corasick::{AhoCorasick, MatchKind};
use serde::{Deserialize, Serialize};

/// Categories of monitored keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeywordCategory {
    /// Profanity, vulgarity, swearing (Vietnamese & English).
    Profanity,
    /// Cyberbullying, harassment, threats.
    Bullying,
    /// Self-harm, suicide, depression ideation.
    SelfHarm,
    /// Adult, sexually explicit, NSFW content.
    Adult,
    /// Custom keyword added by parents.
    Custom,
}

impl KeywordCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Profanity => "Chửi bậy / Thô tục",
            Self::Bullying => "Bắt nạt / Đe dọa",
            Self::SelfHarm => "Tự hại / Nguy hiểm",
            Self::Adult => "Người lớn / Khiêu dâm",
            Self::Custom => "Tùy chỉnh phụ huynh",
        }
    }
}

/// Severity level of a detected keyword violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeywordSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl KeywordSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// A monitored keyword pattern definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeywordRule {
    pub id: String,
    pub pattern: String,
    pub category: KeywordCategory,
    pub severity: KeywordSeverity,
    pub enabled: bool,
}

/// A match found in monitored text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeywordMatch {
    pub rule_id: String,
    pub pattern: String,
    pub category: KeywordCategory,
    pub severity: KeywordSeverity,
    pub start_idx: usize,
    pub end_idx: usize,
}

/// Normalizes Vietnamese diacritics, leetspeak digits, and obfuscating punctuation.
pub fn normalize_text(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());

    for ch in raw.chars() {
        for ch_lower in ch.to_lowercase() {
            // Replace Vietnamese diacritics
            match ch_lower {
                'đ' => out.push('d'),
                'á' | 'à' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ấ'
                | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' => out.push('a'),
                'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => {
                    out.push('e')
                }
                'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => out.push('i'),
                'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ớ'
                | 'ờ' | 'ở' | 'ỡ' | 'ợ' => out.push('o'),
                'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => {
                    out.push('u')
                }
                'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => out.push('y'),
                // Common leet-speak replacements
                '0' => out.push('o'),
                '1' => out.push('i'),
                '3' => out.push('e'),
                '4' => out.push('a'),
                '5' => out.push('s'),
                '7' => out.push('t'),
                // Skip common obfuscation delimiters between letters
                '.' | '_' | '-' | '*' | '@' | '#' | '!' | '~' | '/' | '\\' | '|' => {
                    // Ignore symbol to connect obfuscated words like d.m -> dm
                }
                c if c.is_whitespace() => {
                    if !out.ends_with(' ') {
                        out.push(' ');
                    }
                }
                c if c.is_alphanumeric() => {
                    out.push(c);
                }
                _ => {}
            }
        }
    }

    out.trim().to_string()
}

/// Also produces a collapsed non-whitespace version for compact acronyms (e.g. "v c l" -> "vcl").
pub fn collapse_text(normalized: &str) -> String {
    normalized.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Fast multi-pattern keyword matcher using Aho-Corasick.
pub struct KeywordMatcher {
    rules: Vec<KeywordRule>,
    ac: Option<AhoCorasick>,
}

impl Default for KeywordMatcher {
    fn default() -> Self {
        Self::new(default_keyword_rules())
    }
}

impl KeywordMatcher {
    pub fn new(rules: Vec<KeywordRule>) -> Self {
        let mut matcher = Self { rules, ac: None };
        matcher.rebuild();
        matcher
    }

    /// Rebuilds the internal Aho-Corasick automaton from enabled rules.
    pub fn rebuild(&mut self) {
        let patterns: Vec<String> = self
            .rules
            .iter()
            .filter(|r| r.enabled)
            .map(|r| normalize_text(&r.pattern))
            .collect();

        if patterns.is_empty() {
            self.ac = None;
            return;
        }

        self.ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostFirst)
            .ascii_case_insensitive(true)
            .build(&patterns)
            .ok();
    }

    pub fn rules(&self) -> &[KeywordRule] {
        &self.rules
    }

    pub fn add_rule(&mut self, rule: KeywordRule) {
        self.rules.retain(|r| r.id != rule.id);
        self.rules.push(rule);
        self.rebuild();
    }

    pub fn remove_rule(&mut self, id: &str) -> bool {
        let initial_len = self.rules.len();
        self.rules.retain(|r| r.id != id);
        let removed = self.rules.len() < initial_len;
        if removed {
            self.rebuild();
        }
        removed
    }

    pub fn toggle_rule(&mut self, id: &str, enabled: bool) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == id) {
            rule.enabled = enabled;
            self.rebuild();
            true
        } else {
            false
        }
    }

    pub fn toggle_category(&mut self, category: KeywordCategory, enabled: bool) {
        for rule in self.rules.iter_mut().filter(|r| r.category == category) {
            rule.enabled = enabled;
        }
        self.rebuild();
    }

    /// Scans a text buffer and returns all detected keyword violations.
    pub fn scan_text(&self, text: &str) -> Vec<KeywordMatch> {
        let ac = match &self.ac {
            Some(ac) => ac,
            None => return Vec::new(),
        };

        let enabled_rules: Vec<&KeywordRule> = self.rules.iter().filter(|r| r.enabled).collect();
        let normalized = normalize_text(text);
        let collapsed = collapse_text(&normalized);

        let mut matches = Vec::new();

        // 1. Scan normalized text (preserves word spaces)
        for m in ac.find_iter(&normalized) {
            if let Some(rule) = enabled_rules.get(m.pattern().as_usize()) {
                matches.push(KeywordMatch {
                    rule_id: rule.id.clone(),
                    pattern: rule.pattern.clone(),
                    category: rule.category,
                    severity: rule.severity,
                    start_idx: m.start(),
                    end_idx: m.end(),
                });
            }
        }

        // 2. Scan collapsed text (for acronyms separated by spaces e.g. "v c l", "d m")
        for m in ac.find_iter(&collapsed) {
            if let Some(rule) = enabled_rules.get(m.pattern().as_usize()) {
                // If it's a short token and not already matched
                if !matches.iter().any(|existing| existing.rule_id == rule.id) {
                    matches.push(KeywordMatch {
                        rule_id: rule.id.clone(),
                        pattern: rule.pattern.clone(),
                        category: rule.category,
                        severity: rule.severity,
                        start_idx: m.start(),
                        end_idx: m.end(),
                    });
                }
            }
        }

        matches
    }
}

/// Generates a comprehensive default ruleset for Vietnamese and English toxic keywords.
pub fn default_keyword_rules() -> Vec<KeywordRule> {
    vec![
        // Profanity / Vulgarity (Vietnamese)
        KeywordRule {
            id: "prof_vn_dm".into(),
            pattern: "dm".into(),
            category: KeywordCategory::Profanity,
            severity: KeywordSeverity::Medium,
            enabled: true,
        },
        KeywordRule {
            id: "prof_vn_dmm".into(),
            pattern: "dmm".into(),
            category: KeywordCategory::Profanity,
            severity: KeywordSeverity::Medium,
            enabled: true,
        },
        KeywordRule {
            id: "prof_vn_vcl".into(),
            pattern: "vcl".into(),
            category: KeywordCategory::Profanity,
            severity: KeywordSeverity::Medium,
            enabled: true,
        },
        KeywordRule {
            id: "prof_vn_clgt".into(),
            pattern: "clgt".into(),
            category: KeywordCategory::Profanity,
            severity: KeywordSeverity::Medium,
            enabled: true,
        },
        KeywordRule {
            id: "prof_vn_dit_me".into(),
            pattern: "dit me".into(),
            category: KeywordCategory::Profanity,
            severity: KeywordSeverity::High,
            enabled: true,
        },
        KeywordRule {
            id: "prof_vn_du_ma".into(),
            pattern: "du ma".into(),
            category: KeywordCategory::Profanity,
            severity: KeywordSeverity::High,
            enabled: true,
        },
        KeywordRule {
            id: "prof_vn_lon".into(),
            pattern: "con lon".into(),
            category: KeywordCategory::Profanity,
            severity: KeywordSeverity::High,
            enabled: true,
        },
        KeywordRule {
            id: "prof_vn_cac".into(),
            pattern: "cai cac".into(),
            category: KeywordCategory::Profanity,
            severity: KeywordSeverity::High,
            enabled: true,
        },
        // Profanity (English)
        KeywordRule {
            id: "prof_en_fuck".into(),
            pattern: "fuck".into(),
            category: KeywordCategory::Profanity,
            severity: KeywordSeverity::Medium,
            enabled: true,
        },
        KeywordRule {
            id: "prof_en_bitch".into(),
            pattern: "bitch".into(),
            category: KeywordCategory::Profanity,
            severity: KeywordSeverity::Medium,
            enabled: true,
        },
        KeywordRule {
            id: "prof_en_shit".into(),
            pattern: "shit".into(),
            category: KeywordCategory::Profanity,
            severity: KeywordSeverity::Low,
            enabled: true,
        },
        KeywordRule {
            id: "prof_en_asshole".into(),
            pattern: "asshole".into(),
            category: KeywordCategory::Profanity,
            severity: KeywordSeverity::Medium,
            enabled: true,
        },
        // Bullying / Harassment
        KeywordRule {
            id: "bully_vn_chet_di".into(),
            pattern: "chet di".into(),
            category: KeywordCategory::Bullying,
            severity: KeywordSeverity::High,
            enabled: true,
        },
        KeywordRule {
            id: "bully_vn_tao_giet".into(),
            pattern: "tao giet".into(),
            category: KeywordCategory::Bullying,
            severity: KeywordSeverity::Critical,
            enabled: true,
        },
        KeywordRule {
            id: "bully_vn_danh_chet".into(),
            pattern: "danh chet".into(),
            category: KeywordCategory::Bullying,
            severity: KeywordSeverity::High,
            enabled: true,
        },
        KeywordRule {
            id: "bully_vn_con_cho".into(),
            pattern: "con cho".into(),
            category: KeywordCategory::Bullying,
            severity: KeywordSeverity::Medium,
            enabled: true,
        },
        // Self-Harm / Suicide (Highest Priority / Critical)
        KeywordRule {
            id: "harm_vn_tu_tu".into(),
            pattern: "tu tu".into(),
            category: KeywordCategory::SelfHarm,
            severity: KeywordSeverity::Critical,
            enabled: true,
        },
        KeywordRule {
            id: "harm_vn_tu_sat".into(),
            pattern: "tu sat".into(),
            category: KeywordCategory::SelfHarm,
            severity: KeywordSeverity::Critical,
            enabled: true,
        },
        KeywordRule {
            id: "harm_vn_rach_tay".into(),
            pattern: "rach tay".into(),
            category: KeywordCategory::SelfHarm,
            severity: KeywordSeverity::Critical,
            enabled: true,
        },
        KeywordRule {
            id: "harm_vn_muon_chet".into(),
            pattern: "muon chet".into(),
            category: KeywordCategory::SelfHarm,
            severity: KeywordSeverity::Critical,
            enabled: true,
        },
        KeywordRule {
            id: "harm_en_suicide".into(),
            pattern: "suicide".into(),
            category: KeywordCategory::SelfHarm,
            severity: KeywordSeverity::Critical,
            enabled: true,
        },
        KeywordRule {
            id: "harm_en_self_harm".into(),
            pattern: "self harm".into(),
            category: KeywordCategory::SelfHarm,
            severity: KeywordSeverity::Critical,
            enabled: true,
        },
        KeywordRule {
            id: "harm_en_kill_myself".into(),
            pattern: "kill myself".into(),
            category: KeywordCategory::SelfHarm,
            severity: KeywordSeverity::Critical,
            enabled: true,
        },
        // Adult / NSFW
        KeywordRule {
            id: "adult_vn_phim_sex".into(),
            pattern: "phim sex".into(),
            category: KeywordCategory::Adult,
            severity: KeywordSeverity::High,
            enabled: true,
        },
        KeywordRule {
            id: "adult_en_porn".into(),
            pattern: "porn".into(),
            category: KeywordCategory::Adult,
            severity: KeywordSeverity::High,
            enabled: true,
        },
        KeywordRule {
            id: "adult_en_hentai".into(),
            pattern: "hentai".into(),
            category: KeywordCategory::Adult,
            severity: KeywordSeverity::High,
            enabled: true,
        },
        KeywordRule {
            id: "adult_en_jav".into(),
            pattern: "jav".into(),
            category: KeywordCategory::Adult,
            severity: KeywordSeverity::High,
            enabled: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_vietnamese_and_leetspeak() {
        assert_eq!(normalize_text("Đụ má"), "du ma");
        assert_eq!(normalize_text("Đ.M!"), "dm");
        assert_eq!(normalize_text("v_c_l"), "vcl");
        assert_eq!(normalize_text("d*m"), "dm");
        assert_eq!(normalize_text("10n"), "ion"); // 1->i, 0->o
        assert_eq!(normalize_text("c4c"), "cac"); // 4->a
    }

    #[test]
    fn test_keyword_matcher_detects_violations() {
        let matcher = KeywordMatcher::default();

        // Direct matching
        let matches = matcher.scan_text("Thang nay choi game vcl that");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].pattern, "vcl");
        assert_eq!(matches[0].category, KeywordCategory::Profanity);

        // Accents and punctuation obfuscation
        let matches = matcher.scan_text("may d.m may ha buoi");
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern == "dm"));

        // Critical Self-Harm detection
        let matches = matcher.scan_text("Toi cam thay rat buon va muon tu tu");
        assert!(!matches.is_empty());
        let self_harm = matches
            .iter()
            .find(|m| m.category == KeywordCategory::SelfHarm);
        assert!(self_harm.is_some());
        assert_eq!(
            self_harm.map(|m| m.severity),
            Some(KeywordSeverity::Critical)
        );
    }

    #[test]
    fn test_keyword_matcher_toggle_and_custom() {
        let mut matcher = KeywordMatcher::default();

        // Toggle category off
        matcher.toggle_category(KeywordCategory::Profanity, false);
        let matches = matcher.scan_text("vcl that chu");
        assert!(matches.is_empty());

        // Add custom rule
        matcher.add_rule(KeywordRule {
            id: "custom_rule_1".into(),
            pattern: "choi game khuya".into(),
            category: KeywordCategory::Custom,
            severity: KeywordSeverity::Low,
            enabled: true,
        });

        let matches = matcher.scan_text("No dang choi game khuya");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern, "choi game khuya");
        assert_eq!(matches[0].category, KeywordCategory::Custom);
    }
}
