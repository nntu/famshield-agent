pub mod atomic_swap;
pub mod error;
pub mod offline;
pub mod online;
pub mod staged;
pub mod verifier;

pub use atomic_swap::{UpdateWatchdogState, WatchdogStatus, WindowsAtomicUpdater};
pub use error::UpdateError;
pub use offline::{OfflineUpdateEngine, OfflineUpdateReport};
pub use online::OnlineUpdateClient;
pub use staged::{ReleaseManifest, StagedUpdateEngine, UpdateCheckResult};
pub use verifier::UpdatePackageVerifier;
