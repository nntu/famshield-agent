use chrono::{DateTime, Datelike, Timelike, Utc};
use crate::decision::{Decision, ReasonCode};
use crate::domain::{ApplicationCategory, ExecutableName, MonotonicUsage, NormalizedDomain};
use crate::error::CoreError;
use crate::policy::{DayOfWeek, Policy, RuleAction, Schedule, TimeWindow, VisualTrigger};

/// Evaluates processes and network requests against loaded policy.
pub struct PolicyEvaluator {
    policy: Policy,
}

#[derive(Clone, Debug, Default)]
pub struct ProcessEvaluationContext<'a> {
    pub executable_name: Option<&'a ExecutableName>,
    pub canonical_path: Option<&'a str>,
    pub signer_thumbprint: Option<&'a str>,
    pub sha256_hash: Option<&'a str>,
    pub category: Option<ApplicationCategory>,
    pub application_family: Option<&'a str>,
    pub current_usage: Option<MonotonicUsage>,
}

#[derive(Clone, Debug)]
pub struct DomainEvaluationContext<'a> {
    pub domain: &'a NormalizedDomain,
    pub current_usage: Option<MonotonicUsage>,
}

impl<'a> DomainEvaluationContext<'a> {
    pub fn new(domain: &'a NormalizedDomain) -> Self {
        Self {
            domain,
            current_usage: None,
        }
    }

    pub fn with_usage(mut self, usage: MonotonicUsage) -> Self {
        self.current_usage = Some(usage);
        self
    }
}

impl PolicyEvaluator {
    pub fn new(policy: Policy) -> Self {
        Self { policy }
    }

    pub fn policy(&self) -> &Policy {
        &self.policy
    }

    /// Evaluates an application process against policy rules in strict precedence order:
    /// 1. Exact hash rule.
    /// 2. Signer thumbprint rule.
    /// 3. Canonical path rule.
    /// 4. Executable-name rule.
    /// 5. Application-family rule.
    /// 6. Category rule.
    /// 7. Default action.
    pub fn evaluate_process(
        &self,
        ctx: &ProcessEvaluationContext,
        now: DateTime<Utc>,
    ) -> Result<Decision, CoreError> {
        if !self.policy.application.enabled {
            return Ok(Decision::allow(ReasonCode::DefaultActionAllow));
        }

        // Active rules only
        let enabled_rules: Vec<_> = self.policy.application.rules.iter().filter(|r| r.enabled).collect();

        // 1. Exact hash match
        if let Some(hash) = ctx.sha256_hash {
            for rule in &enabled_rules {
                if rule.identity.sha256.iter().any(|h| h.eq_ignore_ascii_case(hash)) {
                    return self.evaluate_application_rule(rule, ctx.current_usage, now);
                }
            }
        }

        // 2. Signer thumbprint match
        if let Some(signer) = ctx.signer_thumbprint {
            for rule in &enabled_rules {
                if rule.identity.signer_thumbprints.iter().any(|s| s.eq_ignore_ascii_case(signer)) {
                    return self.evaluate_application_rule(rule, ctx.current_usage, now);
                }
            }
        }

        // 3. Canonical path match
        if let Some(path) = ctx.canonical_path {
            let lower_path = path.to_ascii_lowercase();
            for rule in &enabled_rules {
                if rule.identity.canonical_paths.iter().any(|p| p.to_ascii_lowercase() == lower_path) {
                    return self.evaluate_application_rule(rule, ctx.current_usage, now);
                }
            }
        }

        // 4. Executable name match
        if let Some(exe) = ctx.executable_name {
            for rule in &enabled_rules {
                if rule.identity.executable_names.iter().any(|e| e.eq_ignore_ascii_case(exe.as_str())) {
                    return self.evaluate_application_rule(rule, ctx.current_usage, now);
                }
            }
        }

        // 5. Application family match
        if let Some(fam) = ctx.application_family {
            for rule in &enabled_rules {
                if rule.identity.application_families.iter().any(|f| f.eq_ignore_ascii_case(fam)) {
                    return self.evaluate_application_rule(rule, ctx.current_usage, now);
                }
            }
        }

        // 6. Category match
        if let Some(cat) = ctx.category {
            for rule in &enabled_rules {
                if rule.identity.categories.contains(&cat) {
                    return self.evaluate_application_rule(rule, ctx.current_usage, now);
                }
            }
        }

        // 7. Default action
        let decision = match self.policy.application.default_action {
            RuleAction::Allow => Decision::allow(ReasonCode::DefaultActionAllow),
            RuleAction::Block => {
                let mut d = Decision::block(ReasonCode::DefaultActionBlock);
                if self.is_visual_verification_enabled(VisualTrigger::AppBlocked) {
                    d = d.with_visual_verification(true);
                }
                d
            }
        };

        Ok(decision)
    }

    /// Evaluates a domain query against web policy rules (subdomain hierarchy match).
    pub fn evaluate_domain(
        &self,
        ctx: &DomainEvaluationContext,
        now: DateTime<Utc>,
    ) -> Result<Decision, CoreError> {
        if !self.policy.web.enabled {
            return Ok(Decision::allow(ReasonCode::DefaultActionAllow));
        }

        let target_domain = ctx.domain.as_str();

        // Exact match or suffix/parent domain match
        // Precedence: explicit Allow or Block rule matched by specificity
        for rule in self.policy.web.rules.iter().filter(|r| r.enabled) {
            for pattern in &rule.domains {
                let pattern_norm = pattern.trim().trim_end_matches('.').to_ascii_lowercase();
                if target_domain == pattern_norm || target_domain.ends_with(&format!(".{}", pattern_norm)) {
                    return self.evaluate_domain_rule(rule, ctx.current_usage, now);
                }
            }
        }

        // Fallback to default WebMode
        match self.policy.web.mode {
            crate::policy::WebMode::AllowByDefault => Ok(Decision::allow(ReasonCode::DefaultActionAllow)),
            crate::policy::WebMode::DenyByDefault => {
                let mut d = Decision::block(ReasonCode::DefaultActionBlock);
                if self.is_visual_verification_enabled(VisualTrigger::DomainBlocked) {
                    d = d.with_visual_verification(true);
                }
                Ok(d)
            }
        }
    }

    fn evaluate_application_rule(
        &self,
        rule: &crate::policy::ApplicationRule,
        usage: Option<MonotonicUsage>,
        now: DateTime<Utc>,
    ) -> Result<Decision, CoreError> {
        match rule.action {
            RuleAction::Block => {
                let mut d = Decision::block(ReasonCode::ExplicitRuleBlock)
                    .with_rule(rule.id.clone(), rule.name.clone());
                if self.is_visual_verification_enabled(VisualTrigger::AppBlocked) {
                    d = d.with_visual_verification(true);
                }
                Ok(d)
            }
            RuleAction::Allow => {
                // Check schedule windows & quota
                self.evaluate_schedule(&rule.schedule, usage, now, &rule.id, &rule.name, VisualTrigger::AppBlocked)
            }
        }
    }

    fn evaluate_domain_rule(
        &self,
        rule: &crate::policy::DomainRule,
        usage: Option<MonotonicUsage>,
        now: DateTime<Utc>,
    ) -> Result<Decision, CoreError> {
        match rule.action {
            RuleAction::Block => {
                let mut d = Decision::block(ReasonCode::ExplicitRuleBlock)
                    .with_rule(rule.id.clone(), rule.name.clone());
                if self.is_visual_verification_enabled(VisualTrigger::DomainBlocked) {
                    d = d.with_visual_verification(true);
                }
                Ok(d)
            }
            RuleAction::Allow => {
                self.evaluate_schedule(&rule.schedule, usage, now, &rule.id, &rule.name, VisualTrigger::DomainBlocked)
            }
        }
    }

    fn evaluate_schedule(
        &self,
        schedule: &Schedule,
        usage: Option<MonotonicUsage>,
        now: DateTime<Utc>,
        rule_id: &str,
        rule_name: &str,
        trigger: VisualTrigger,
    ) -> Result<Decision, CoreError> {
        // 1. Time windows check
        if !schedule.windows.is_empty() {
            let in_window = schedule.windows.iter().any(|w| self.is_time_in_window(w, now));
            if !in_window {
                let mut d = Decision::block(ReasonCode::OutsideAllowedSchedule)
                    .with_rule(rule_id.to_string(), rule_name.to_string());
                if self.is_visual_verification_enabled(trigger) {
                    d = d.with_visual_verification(true);
                }
                return Ok(d);
            }
        }

        // 2. Daily quota check
        if let Some(quota_minutes) = schedule.daily_quota_minutes {
            let quota_seconds = quota_minutes * 60;
            let used_seconds = usage.map(|u| (u.as_millis() / 1000) as u32).unwrap_or(0);

            if used_seconds >= quota_seconds {
                let mut d = Decision::block(ReasonCode::DailyQuotaExceeded)
                    .with_rule(rule_id.to_string(), rule_name.to_string())
                    .with_quota_remaining(0);
                if self.is_visual_verification_enabled(trigger) {
                    d = d.with_visual_verification(true);
                }
                return Ok(d);
            }

            let remaining = quota_seconds - used_seconds;
            return Ok(Decision::allow(ReasonCode::ExplicitRuleAllow)
                .with_rule(rule_id.to_string(), rule_name.to_string())
                .with_quota_remaining(remaining));
        }

        Ok(Decision::allow(ReasonCode::ExplicitRuleAllow).with_rule(rule_id.to_string(), rule_name.to_string()))
    }

    fn is_time_in_window(&self, window: &TimeWindow, now: DateTime<Utc>) -> bool {
        // Convert Utc now to DayOfWeek
        let day = match now.weekday() {
            chrono::Weekday::Mon => DayOfWeek::Mon,
            chrono::Weekday::Tue => DayOfWeek::Tue,
            chrono::Weekday::Wed => DayOfWeek::Wed,
            chrono::Weekday::Thu => DayOfWeek::Thu,
            chrono::Weekday::Fri => DayOfWeek::Fri,
            chrono::Weekday::Sat => DayOfWeek::Sat,
            chrono::Weekday::Sun => DayOfWeek::Sun,
        };

        if !window.days.is_empty() && !window.days.contains(&day) {
            return false;
        }

        // Parse HH:MM
        let current_minutes = now.hour() * 60 + now.minute();

        let start_mins = parse_hh_mm(&window.start).unwrap_or(0);
        let end_mins = parse_hh_mm(&window.end).unwrap_or(24 * 60);

        if start_mins <= end_mins {
            current_minutes >= start_mins && current_minutes < end_mins
        } else {
            // Window wraps around midnight (e.g., 22:00 to 06:00)
            current_minutes >= start_mins || current_minutes < end_mins
        }
    }

    fn is_visual_verification_enabled(&self, trigger: VisualTrigger) -> bool {
        if let Some(ref vv) = self.policy.visual_verification {
            if vv.enabled && vv.triggers.contains(&trigger) {
                return true;
            }
        }
        false
    }
}

fn parse_hh_mm(s: &str) -> Option<u32> {
    let mut parts = s.split(':');
    let h: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    if h > 24 || m >= 60 {
        return None;
    }
    Some(h * 60 + m)
}
