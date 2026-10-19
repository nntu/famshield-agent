use sha2::{Digest, Sha256};
use tracing::info;
use crate::error::UpdateError;

pub struct UpdatePackageVerifier;

impl UpdatePackageVerifier {
    /// Validates an update package against its declared SHA-256 checksum.
    pub fn verify_checksum(package_bytes: &[u8], expected_sha256: &str) -> Result<(), UpdateError> {
        let mut hasher = Sha256::new();
        hasher.update(package_bytes);
        let calculated = format!("{:x}", hasher.finalize());

        if !calculated.eq_ignore_ascii_case(expected_sha256.trim()) {
            return Err(UpdateError::ChecksumMismatch {
                expected: expected_sha256.to_string(),
                calculated,
            });
        }

        info!("Package checksum verified successfully ({})", calculated);
        Ok(())
    }
}
