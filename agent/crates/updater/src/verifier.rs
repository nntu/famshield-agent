use crate::error::UpdateError;
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use tracing::info;

pub struct UpdatePackageVerifier;

impl UpdatePackageVerifier {
    /// Verifies a DER-encoded ECDSA P-256/SHA-256 signature over canonical manifest bytes.
    pub fn verify_manifest_signature(
        canonical_manifest: &[u8],
        signature_hex: &str,
        verifying_key: &VerifyingKey,
    ) -> Result<(), UpdateError> {
        let signature_bytes = decode_hex(signature_hex.trim())?;
        let signature = Signature::from_der(&signature_bytes).map_err(|error| {
            UpdateError::ManifestInvalid(format!("invalid ECDSA signature encoding: {error}"))
        })?;
        verifying_key
            .verify(canonical_manifest, &signature)
            .map_err(|_| {
                UpdateError::ManifestInvalid("release manifest signature mismatch".into())
            })?;
        info!("Release manifest ECDSA P-256 signature verified successfully");
        Ok(())
    }

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

fn decode_hex(encoded: &str) -> Result<Vec<u8>, UpdateError> {
    if !encoded.as_bytes().chunks_exact(2).remainder().is_empty() {
        return Err(UpdateError::ManifestInvalid(
            "signature must contain an even number of hexadecimal characters".into(),
        ));
    }

    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = decode_nibble(pair[0])?;
            let low = decode_nibble(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn decode_nibble(value: u8) -> Result<u8, UpdateError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(UpdateError::ManifestInvalid(
            "signature contains non-hexadecimal characters".into(),
        )),
    }
}
