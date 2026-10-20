pub mod auth;
mod secret_protection;
pub mod server;
pub mod service;
pub mod totp;
pub mod ui;

pub use auth::{AuthError, MasterPinAuthenticator};
pub use secret_protection::{
    protect_secret as protect_machine_secret, unprotect_secret as unprotect_machine_secret,
};
pub use server::{
    EvidenceItemResponse, LocalAdminServer, LoginRequest, LoginResponse, ServerState,
    StatusResponse,
};
pub use service::{CreateRuleRequest, LocalAdminError, LocalAdminService};
pub use ui::LOCAL_DASHBOARD_HTML;
