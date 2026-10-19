use crate::error::UpdateError;
use crate::verifier::UpdatePackageVerifier;
use p256::ecdsa::VerifyingKey;
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub manifest_version: u32,
    pub version: String,
    pub channel: String, // "stable", "beta"
    pub sha256: String,
    pub binary_url: String,
    pub signature_hex: String,
    pub key_id: String,
    pub min_supported_version: String,
}

#[derive(Serialize)]
struct CanonicalReleaseManifest<'a> {
    manifest_version: u32,
    version: &'a str,
    channel: &'a str,
    sha256: &'a str,
    binary_url: &'a str,
    key_id: &'a str,
    min_supported_version: &'a str,
}

impl ReleaseManifest {
    /// Canonical JSON bytes covered by the release signing key.
    pub fn signing_payload(&self) -> Result<Vec<u8>, UpdateError> {
        serde_json::to_vec(&CanonicalReleaseManifest {
            manifest_version: self.manifest_version,
            version: &self.version,
            channel: &self.channel,
            sha256: &self.sha256,
            binary_url: &self.binary_url,
            key_id: &self.key_id,
            min_supported_version: &self.min_supported_version,
        })
        .map_err(|error| UpdateError::ManifestInvalid(error.to_string()))
    }
}

pub struct StagedUpdateEngine {
    current_version: Version,
    trusted_key_id: String,
    verifying_key: VerifyingKey,
}

impl StagedUpdateEngine {
    pub fn new(
        current_version: &str,
        trusted_key_id: impl Into<String>,
        verifying_key_sec1: &[u8],
    ) -> Result<Self, UpdateError> {
        let current_version = Version::parse(current_version).map_err(|error| {
            UpdateError::ManifestInvalid(format!("invalid current version: {error}"))
        })?;
        let verifying_key = VerifyingKey::from_sec1_bytes(verifying_key_sec1).map_err(|error| {
            UpdateError::ManifestInvalid(format!("invalid trusted P-256 public key: {error}"))
        })?;
        Ok(Self {
            current_version,
            trusted_key_id: trusted_key_id.into(),
            verifying_key,
        })
    }

    /// Verifies manifest, channel, and package payload checksum.
    pub fn verify_release(
        &self,
        manifest: &ReleaseManifest,
        payload: &[u8],
    ) -> Result<(), UpdateError> {
        if manifest.manifest_version != 1 {
            return Err(UpdateError::ManifestInvalid(format!(
                "unsupported manifest version: {}",
                manifest.manifest_version
            )));
        }
        if manifest.channel != "stable" && manifest.channel != "beta" {
            return Err(UpdateError::ManifestInvalid(format!(
                "Invalid channel: {}",
                manifest.channel
            )));
        }

        if manifest.key_id != self.trusted_key_id {
            return Err(UpdateError::ManifestInvalid(format!(
                "untrusted release signing key: {}",
                manifest.key_id
            )));
        }

        let release_version = Version::parse(&manifest.version).map_err(|error| {
            UpdateError::ManifestInvalid(format!("invalid release version: {error}"))
        })?;
        if release_version <= self.current_version {
            return Err(UpdateError::ManifestInvalid(format!(
                "release {} is not newer than installed version {}",
                release_version, self.current_version
            )));
        }

        Version::parse(&manifest.min_supported_version).map_err(|error| {
            UpdateError::ManifestInvalid(format!("invalid minimum supported version: {error}"))
        })?;
        if !manifest.binary_url.starts_with("https://") {
            return Err(UpdateError::ManifestInvalid(
                "release binary URL must use HTTPS".into(),
            ));
        }

        UpdatePackageVerifier::verify_manifest_signature(
            &manifest.signing_payload()?,
            &manifest.signature_hex,
            &self.verifying_key,
        )?;

        UpdatePackageVerifier::verify_checksum(payload, &manifest.sha256)?;
        Ok(())
    }

    /// Simulates staged atomic update: validates health check on new binary before swapping,
    /// triggering automated rollback if the new version fails its initial health check.
    pub fn stage_and_apply<F>(
        &self,
        manifest: &ReleaseManifest,
        payload: &[u8],
        health_check: F,
    ) -> Result<(), UpdateError>
    where
        F: FnOnce() -> bool,
    {
        self.verify_release(manifest, payload)?;

        // Run staged self-test
        if !health_check() {
            return Err(UpdateError::ManifestInvalid(
                "Health check failed on newly staged binary; executing automatic rollback to previous version".into(),
            ));
        }

        tracing::info!(
            "Atomic upgrade applied successfully: {} -> {}",
            self.current_version,
            manifest.version
        );
        Ok(())
    }
}
