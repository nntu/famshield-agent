use childprotect_updater::{ReleaseManifest, StagedUpdateEngine, UpdatePackageVerifier};

#[test]
fn test_update_package_verifier_checksum() {
    use sha2::{Digest, Sha256};
    let payload = b"ChildProtect Update Binary Payload";
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let calculated_sha256 = format!("{:x}", hasher.finalize());

    let res = UpdatePackageVerifier::verify_checksum(payload, &calculated_sha256);
    assert!(res.is_ok());

    let bad_res = UpdatePackageVerifier::verify_checksum(payload, "invalidchecksum00000000000000000000000000000000000000000000000000");
    assert!(bad_res.is_err());
}

#[test]
fn test_staged_update_success_and_rollback() {
    use sha2::{Digest, Sha256};
    let payload = b"New Version Binary 0.2.0";
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let sha256 = format!("{:x}", hasher.finalize());

    let engine = StagedUpdateEngine::new("0.1.0");
    let manifest = ReleaseManifest {
        version: "0.2.0".into(),
        channel: "stable".into(),
        sha256,
        binary_url: "https://dl.childprotect.cloud/0.2.0.bin".into(),
        signature_hex: "abcd".into(),
        min_supported_version: "0.1.0".into(),
    };

    // 1. Success case: health check passes
    let ok = engine.stage_and_apply(&manifest, payload, || true);
    assert!(ok.is_ok());

    // 2. Rollback case: health check fails -> triggers automated rollback
    let fail = engine.stage_and_apply(&manifest, payload, || false);
    assert!(fail.is_err());
    let err_msg = fail.unwrap_err().to_string();
    assert!(err_msg.contains("rollback"));
}
