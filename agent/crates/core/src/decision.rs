use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Allow,
    Block,
    Warn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonCode {
    ExplicitRuleBlock,
    ExplicitRuleAllow,
    OutsideAllowedSchedule,
    DailyQuotaExceeded,
    CategoryBlocked,
    DefaultActionBlock,
    DefaultActionAllow,
    SafeSearchEnforced,
    DnsFailureBlocked,
    AdministrativeOverride,
}

impl ReasonCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExplicitRuleBlock => "explicit_rule_block",
            Self::ExplicitRuleAllow => "explicit_rule_allow",
            Self::OutsideAllowedSchedule => "outside_allowed_schedule",
            Self::DailyQuotaExceeded => "daily_quota_exceeded",
            Self::CategoryBlocked => "category_blocked",
            Self::DefaultActionBlock => "default_action_block",
            Self::DefaultActionAllow => "default_action_allow",
            Self::SafeSearchEnforced => "safe_search_enforced",
            Self::DnsFailureBlocked => "dns_failure_blocked",
            Self::AdministrativeOverride => "administrative_override",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub action: Action,
    pub reason: ReasonCode,
    pub matched_rule_id: Option<String>,
    pub matched_rule_name: Option<String>,
    pub quota_remaining_seconds: Option<u32>,
    pub visual_verification_required: bool,
}

impl Decision {
    pub fn allow(reason: ReasonCode) -> Self {
        Self {
            action: Action::Allow,
            reason,
            matched_rule_id: None,
            matched_rule_name: None,
            quota_remaining_seconds: None,
            visual_verification_required: false,
        }
    }

    pub fn block(reason: ReasonCode) -> Self {
        Self {
            action: Action::Block,
            reason,
            matched_rule_id: None,
            matched_rule_name: None,
            quota_remaining_seconds: None,
            visual_verification_required: false,
        }
    }

    pub fn with_rule(mut self, id: String, name: String) -> Self {
        self.matched_rule_id = Some(id);
        self.matched_rule_name = Some(name);
        self
    }

    pub fn with_quota_remaining(mut self, seconds: u32) -> Self {
        self.quota_remaining_seconds = Some(seconds);
        self
    }

    pub fn with_visual_verification(mut self, required: bool) -> Self {
        self.visual_verification_required = required;
        self
    }

    pub fn is_allowed(&self) -> bool {
        self.action == Action::Allow || self.action == Action::Warn
    }

    pub fn is_blocked(&self) -> bool {
        self.action == Action::Block
    }
}
