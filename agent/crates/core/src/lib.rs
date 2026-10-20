pub mod decision;
pub mod domain;
pub mod error;
pub mod evaluator;
pub mod keyword;
pub mod policy;
pub mod traits;
pub mod vietnamese;

pub use decision::{Action, Decision, ReasonCode};
pub use domain::{
    ApplicationCategory, ChildId, DeviceId, ExecutableName, MonotonicUsage, NormalizedDomain,
    TenantId,
};
pub use error::CoreError;
pub use evaluator::{DomainEvaluationContext, PolicyEvaluator, ProcessEvaluationContext};
pub use keyword::{
    default_keyword_rules, normalize_text, KeywordCategory, KeywordMatch, KeywordMatcher,
    KeywordRule, KeywordSeverity,
};
pub use policy::Policy;
pub use traits::{Clock, MockClock, SystemClock};
pub use vietnamese::{canonical_vietnamese, decode_telex, decode_vni, is_word_boundary};
