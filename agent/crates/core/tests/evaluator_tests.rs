use chrono::{TimeZone, Utc};
use famshield_core::{
    decision::{Action, ReasonCode},
    domain::{ApplicationCategory, ExecutableName, NormalizedDomain},
    evaluator::{DomainEvaluationContext, PolicyEvaluator, ProcessEvaluationContext},
    policy::Policy,
    traits::{Clock, MockClock},
};
use std::time::Duration;

const SAMPLE_POLICY: &str = include_str!("../../../../contracts/examples/policy.v1.json");

#[test]
fn test_parse_policy_fixture() {
    let policy: Policy =
        serde_json::from_str(SAMPLE_POLICY).expect("Must parse sample policy fixture");
    assert_eq!(policy.schema_version, 1);
    assert_eq!(policy.policy_id, "school-night-default");
    assert_eq!(policy.application.rules.len(), 1);
    assert_eq!(policy.web.rules.len(), 1);
}

#[test]
fn test_evaluate_process_category_block() {
    let policy: Policy = serde_json::from_str(SAMPLE_POLICY).unwrap();
    let evaluator = PolicyEvaluator::new(policy);

    let exe = ExecutableName::parse("game_client.exe").unwrap();
    let ctx = ProcessEvaluationContext {
        executable_name: Some(&exe),
        category: Some(ApplicationCategory::Game),
        ..Default::default()
    };

    let dt = Utc.with_ymd_and_hms(2026, 10, 19, 18, 0, 0).unwrap();
    let decision = evaluator.evaluate_process(&ctx, dt).unwrap();

    assert_eq!(decision.action, Action::Block);
    assert_eq!(decision.reason, ReasonCode::ExplicitRuleBlock);
    assert!(decision.visual_verification_required);
}

#[test]
fn test_evaluate_domain_blocked() {
    let policy: Policy = serde_json::from_str(SAMPLE_POLICY).unwrap();
    let evaluator = PolicyEvaluator::new(policy);

    let domain = NormalizedDomain::parse("chat.example-social.invalid").unwrap();
    let ctx = DomainEvaluationContext::new(&domain);

    let dt = Utc.with_ymd_and_hms(2026, 10, 19, 10, 0, 0).unwrap();
    let decision = evaluator.evaluate_domain(&ctx, dt).unwrap();

    assert_eq!(decision.action, Action::Block);
    assert_eq!(decision.reason, ReasonCode::ExplicitRuleBlock);
    assert!(decision.visual_verification_required);
}

#[test]
fn test_evaluate_domain_allowed() {
    let policy: Policy = serde_json::from_str(SAMPLE_POLICY).unwrap();
    let evaluator = PolicyEvaluator::new(policy);

    let domain = NormalizedDomain::parse("wikipedia.org").unwrap();
    let ctx = DomainEvaluationContext::new(&domain);

    let dt = Utc.with_ymd_and_hms(2026, 10, 19, 10, 0, 0).unwrap();
    let decision = evaluator.evaluate_domain(&ctx, dt).unwrap();

    assert_eq!(decision.action, Action::Allow);
    assert_eq!(decision.reason, ReasonCode::DefaultActionAllow);
    assert!(!decision.visual_verification_required);
}

#[test]
fn test_mock_clock_advancement() {
    let start = Utc.with_ymd_and_hms(2026, 10, 19, 0, 0, 0).unwrap();
    let mut clock = MockClock::new(start);

    assert_eq!(clock.now_utc(), start);
    clock.advance(Duration::from_secs(3600));
    assert_eq!(
        clock.now_utc(),
        Utc.with_ymd_and_hms(2026, 10, 19, 1, 0, 0).unwrap()
    );
}
