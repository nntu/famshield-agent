pub mod decision;
pub mod domain;
pub mod error;
pub mod evaluator;
pub mod hex;
pub mod keyword;
pub mod policy;
pub mod traits;
pub mod version;
pub mod vietnamese;

pub use decision::{Action, Decision, ReasonCode};
pub use domain::{
    ApplicationCategory, ChildId, DeviceId, ExecutableName, MonotonicUsage, NormalizedDomain,
    TenantId,
};
pub use error::CoreError;
pub use evaluator::{DomainEvaluationContext, PolicyEvaluator, ProcessEvaluationContext};
pub use hex::{decode_hex, decode_nibble, encode_hex};
pub use keyword::{
    default_keyword_rules, normalize_text, KeywordCategory, KeywordMatch, KeywordMatcher,
    KeywordRule, KeywordSeverity,
};
pub use policy::Policy;
pub use traits::{Clock, MockClock, SystemClock};
pub use version::is_newer_version;
pub use vietnamese::{canonical_vietnamese, decode_telex, decode_vni, is_word_boundary};
