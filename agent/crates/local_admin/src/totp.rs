use sha2::{Digest, Sha256};
use thiserror::Error;
use totp_rs::{Algorithm, Builder, Secret, Totp};

#[derive(Debug, Error)]
pub enum TotpError {
    #[error("TOTP build error: {0}")]
    Build(String),

    #[error("Base32 decoding error: {0}")]
    Base32(String),

    #[error("QR code generation error: {0}")]
    Qr(String),
}

pub struct TotpManager;

impl TotpManager {
    /// Generate a fresh Base32 secret (160-bit), along with QR code Data URL (PNG base64) and raw TOTP auth URL.
    pub fn generate_setup(
        account_name: &str,
    ) -> Result<(String, String, String, Vec<String>), TotpError> {
        let secret = Secret::generate();
        let base32_secret = secret.to_base32();

        let totp = Builder::new()
            .with_algorithm(Algorithm::SHA1)
            .with_digits(6)
            .with_skew(1)
            .with_step_duration(30)
            .with_secret(secret)
            .with_issuer(Some("FamShield"))
            .with_account_name(account_name)
            .build()
            .map_err(|e| TotpError::Build(e.to_string()))?;

        let url = totp.to_url().map_err(|e| TotpError::Build(e.to_string()))?;
        let qr_base64 = totp
            .to_qr_base64()
            .map_err(|e| TotpError::Qr(e.to_string()))?;

        let backup_codes = Self::generate_backup_codes(8);

        Ok((base32_secret, url, qr_base64, backup_codes))
    }

    /// Reconstruct TOTP instance from a stored Base32 secret string.
    pub fn from_base32_secret(base32_secret: &str, account_name: &str) -> Result<Totp, TotpError> {
        let secret =
            Secret::try_from_base32(base32_secret).map_err(|e| TotpError::Base32(e.to_string()))?;

        Builder::new()
            .with_algorithm(Algorithm::SHA1)
            .with_digits(6)
            .with_skew(1)
            .with_step_duration(30)
            .with_secret(secret)
            .with_issuer(Some("FamShield"))
            .with_account_name(account_name)
            .build()
            .map_err(|e| TotpError::Build(e.to_string()))
    }

    /// Verify a 6-digit TOTP code against the secret (tolerates [-1, 0, 1] 30s clock drift).
    pub fn verify_code(
        base32_secret: &str,
        account_name: &str,
        code: &str,
    ) -> Result<bool, TotpError> {
        let totp = Self::from_base32_secret(base32_secret, account_name)?;
        Ok(totp.check_current(code.trim()).is_some())
    }

    /// Generate N cryptographically secure backup codes (format: ABCD-1234).
    pub fn generate_backup_codes(count: usize) -> Vec<String> {
        use rand::Rng;
        let charset: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ"; // exclude 0, 1, I, O to prevent confusion
        let mut rng = rand::thread_rng();
        let mut codes = Vec::with_capacity(count);

        for _ in 0..count {
            let mut part1 = String::with_capacity(4);
            let mut part2 = String::with_capacity(4);
            for _ in 0..4 {
                let idx = rng.gen_range(0..charset.len());
                part1.push(charset[idx] as char);
            }
            for _ in 0..4 {
                let idx = rng.gen_range(0..charset.len());
                part2.push(charset[idx] as char);
            }
            codes.push(format!("{}-{}", part1, part2));
        }

        codes
    }

    /// Hash a backup code with SHA-256 for secure storage.
    pub fn hash_backup_code(code: &str) -> String {
        let normalized = code.trim().to_uppercase().replace('-', "");
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_setup_and_verification() {
        let (secret, url, qr_base64, backup_codes) =
            TotpManager::generate_setup("TestUser").unwrap();
        assert!(!secret.is_empty());
        assert!(url.contains("FamShield"));
        assert!(!qr_base64.is_empty());
        assert_eq!(backup_codes.len(), 8);

        let totp = TotpManager::from_base32_secret(&secret, "TestUser").unwrap();
        let current_code = totp.generate_current().to_string();
        assert_eq!(current_code.len(), 6);

        let is_valid = TotpManager::verify_code(&secret, "TestUser", &current_code).unwrap();
        assert!(is_valid);

        let is_invalid = TotpManager::verify_code(&secret, "TestUser", "000000").unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_backup_codes_hashing() {
        let code1 = "A2B3-C4D5";
        let code2 = "a2b3c4d5";
        let hash1 = TotpManager::hash_backup_code(code1);
        let hash2 = TotpManager::hash_backup_code(code2);
        assert_eq!(hash1, hash2);
    }
}
