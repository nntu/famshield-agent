use famshield_updater::{
    ReleaseManifest, StagedUpdateEngine, UpdatePackageVerifier, WindowsAtomicUpdater,
};
use p256::ecdsa::{signature::Signer, Signature, SigningKey};
use std::fs;

#[test]
fn test_update_package_verifier_checksum() {
    use sha2::{Digest, Sha256};
    let payload = b"FamShield Update Binary Payload";
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let calculated_sha256 = format!("{:x}", hasher.finalize());

    let res = UpdatePackageVerifier::verify_checksum(payload, &calculated_sha256);
    assert!(res.is_ok());

    let bad_res = UpdatePackageVerifier::verify_checksum(
        payload,
        "invalidchecksum00000000000000000000000000000000000000000000000000",
    );
    assert!(bad_res.is_err());
}

#[test]
fn test_staged_update_success_and_rollback() {
    use sha2::{Digest, Sha256};
    let payload = b"New Version Binary 0.2.0";
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let sha256 = format!("{:x}", hasher.finalize());

    let signing_key = SigningKey::from_bytes((&[7_u8; 32]).into()).unwrap();
    let verifying_key = signing_key.verifying_key();
    let engine = StagedUpdateEngine::new(
        "0.1.0",
        "release-key-2026",
        verifying_key.to_encoded_point(false).as_bytes(),
    )
    .unwrap();
    let mut manifest = ReleaseManifest {
        manifest_version: 1,
        version: "0.2.0".into(),
        channel: "stable".into(),
        sha256,
        binary_url: "https://dl.famshield.cloud/0.2.0.bin".into(),
        signature_hex: String::new(),
        key_id: "release-key-2026".into(),
        min_supported_version: "0.1.0".into(),
    };
    let signature: Signature = signing_key.sign(&manifest.signing_payload().unwrap());
    manifest.signature_hex = signature
        .to_der()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();

    // 1. Success case: health check passes
    let ok = engine.stage_and_apply(&manifest, payload, || true);
    assert!(ok.is_ok());

    // 2. Rollback case: health check fails -> triggers automated rollback
    let fail = engine.stage_and_apply(&manifest, payload, || false);
    assert!(fail.is_err());
    let err_msg = fail.unwrap_err().to_string();
    assert!(err_msg.contains("rollback"));

    // Any signed field change invalidates the manifest.
    let mut tampered = manifest.clone();
    tampered.sha256 = "00".repeat(32);
    assert!(engine.verify_release(&tampered, payload).is_err());
}

#[test]
fn test_windows_atomic_updater_staging_and_swap() {
    use sha2::{Digest, Sha256};
    let temp_root = std::env::temp_dir().join("famshield_test_updater");
    let app_data = temp_root.join("ProgramData");
    let install_dir = temp_root.join("ProgramFiles");

    let _ = fs::remove_dir_all(&temp_root);
    fs::create_dir_all(&install_dir).unwrap();

    // Create current active binary
    let active_bin = install_dir.join("famshield-service.exe");
    fs::write(&active_bin, b"Version 1.0.0 Active Service Binary").unwrap();

    let updater = WindowsAtomicUpdater::new("1.0.0", &app_data, &install_dir);
    assert_eq!(updater.current_version(), "1.0.0");

    // 1. Stage binary with valid hash
    let new_payload = b"Version 1.1.0 New Staged Service Binary";
    let mut hasher = Sha256::new();
    hasher.update(new_payload);
    let new_sha256 = format!("{:x}", hasher.finalize());

    let staged_path = updater
        .stage_binary("famshield-service.exe", new_payload, &new_sha256)
        .expect("Staging must succeed");
    assert!(staged_path.exists());

    // 2. Atomic swap
    updater
        .atomic_swap("famshield-service.exe")
        .expect("Swap must succeed");

    // Assert active binary now contains new payload
    let active_content = fs::read(&active_bin).unwrap();
    assert_eq!(active_content, new_payload);

    // Assert backup file was preserved
    let backup_bin = app_data.join("backup").join("famshield-service.exe");
    assert!(backup_bin.exists());
    assert_eq!(
        fs::read(&backup_bin).unwrap(),
        b"Version 1.0.0 Active Service Binary"
    );

    // 3. Rollback
    updater
        .rollback("famshield-service.exe")
        .expect("Rollback must succeed");
    assert_eq!(
        fs::read(&active_bin).unwrap(),
        b"Version 1.0.0 Active Service Binary"
    );

    // Clean up test sandbox
    let _ = fs::remove_dir_all(&temp_root);
}
