pub mod config;
pub mod decision;
pub mod domain;
pub mod error;
pub mod evaluator;
pub mod hex;
pub mod keyword;
pub mod policy;
pub mod traits;
pub mod url;
pub mod version;
pub mod vietnamese;

pub use config::{AgentConfig, DEFAULT_CONFIG_FILE_NAME, DEFAULT_DNS_BIND_PORT};
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
pub use url::{
    is_auto_keyword, is_system_dns_forwarder, normalize_nats_url, DEFAULT_DNS_BIND_ADDR,
    DEFAULT_DNS_BIND_IP, DEFAULT_DNS_UPSTREAM_ADDR, DEFAULT_DNS_UPSTREAM_IP,
    DEFAULT_DOWNLOAD_LATEST_URL, DEFAULT_FALLBACK_UPSTREAM_IPS, DEFAULT_NATS_URL,
    DEFAULT_SERVER_URL, DEFAULT_VERSION_CHECK_URL,
};
pub use version::is_newer_version;
pub use vietnamese::{canonical_vietnamese, decode_telex, decode_vni, is_word_boundary};
