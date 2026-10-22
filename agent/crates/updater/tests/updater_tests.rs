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
        binary_url: "https://famshield.ttcantho.io.vn/download/0.2.0.bin".into(),
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

#[test]
fn test_online_update_client_check_and_stage() {
    use famshield_updater::{OnlineUpdateClient, UpdateCheckResult};
    use sha2::{Digest, Sha256};

    let signing_key = SigningKey::from_bytes((&[9_u8; 32]).into()).unwrap();
    let verifying_key = signing_key.verifying_key();
    let engine = StagedUpdateEngine::new(
        "0.1.0",
        "key-cloud-2026",
        verifying_key.to_encoded_point(false).as_bytes(),
    )
    .unwrap();

    let client = OnlineUpdateClient::new("https://famshield.ttcantho.io.vn", "stable", engine);
    assert_eq!(
        client.manifest_url(),
        "https://famshield.ttcantho.io.vn/v1/channels/stable/manifest.json"
    );

    let payload = b"FamShield Service v0.2.0 Online Payload";
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let sha256 = format!("{:x}", hasher.finalize());

    let mut manifest = ReleaseManifest {
        manifest_version: 1,
        version: "0.2.0".into(),
        channel: "stable".into(),
        sha256,
        binary_url: "https://famshield.ttcantho.io.vn/download/binaries/famshield-service.exe".into(),
        signature_hex: String::new(),
        key_id: "key-cloud-2026".into(),
        min_supported_version: "0.1.0".into(),
    };
    let sig: Signature = signing_key.sign(&manifest.signing_payload().unwrap());
    manifest.signature_hex = sig
        .to_der()
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    // 1. Check update
    let check_res = client.check_update(&manifest).unwrap();
    match check_res {
        UpdateCheckResult::UpdateAvailable {
            current_version,
            new_version,
            channel,
            ..
        } => {
            assert_eq!(current_version, "0.1.0");
            assert_eq!(new_version, "0.2.0");
            assert_eq!(channel, "stable");
        }
        other => panic!("Expected UpdateAvailable, got: {:?}", other),
    }

    // 2. Verify and stage
    let temp_root = std::env::temp_dir().join("famshield_test_online_staging");
    let app_data = temp_root.join("ProgramData");
    let install_dir = temp_root.join("ProgramFiles");
    let updater = WindowsAtomicUpdater::new("0.1.0", &app_data, &install_dir);

    let staged_file = client
        .verify_and_stage(&manifest, payload, "famshield-service.exe", &updater)
        .unwrap();
    assert!(staged_file.exists());
    assert_eq!(fs::read(&staged_file).unwrap(), payload);

    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn test_offline_update_engine_check_and_apply() {
    use famshield_updater::{OfflineUpdateEngine, UpdateCheckResult};
    use sha2::{Digest, Sha256};

    let temp_root = std::env::temp_dir().join("famshield_test_offline_update");
    let package_dir = temp_root.join("USB_Update_Package");
    let app_data = temp_root.join("ProgramData");
    let install_dir = temp_root.join("ProgramFiles");

    let _ = fs::remove_dir_all(&temp_root);
    fs::create_dir_all(&package_dir).unwrap();
    fs::create_dir_all(&install_dir).unwrap();

    // Active installed version
    let active_bin = install_dir.join("famshield-service.exe");
    fs::write(&active_bin, b"Active Service Binary v0.1.0").unwrap();

    let signing_key = SigningKey::from_bytes((&[8_u8; 32]).into()).unwrap();
    let verifying_key = signing_key.verifying_key();
    let engine = StagedUpdateEngine::new(
        "0.1.0",
        "key-offline-2026",
        verifying_key.to_encoded_point(false).as_bytes(),
    )
    .unwrap();
    let updater = WindowsAtomicUpdater::new("0.1.0", &app_data, &install_dir);

    // Prepare offline package on "USB"
    let new_payload = b"New Offline Service Binary v0.2.0";
    let mut hasher = Sha256::new();
    hasher.update(new_payload);
    let sha256 = format!("{:x}", hasher.finalize());

    let mut manifest = ReleaseManifest {
        manifest_version: 1,
        version: "0.2.0".into(),
        channel: "stable".into(),
        sha256,
        binary_url: "https://famshield.ttcantho.io.vn/download/famshield-service.exe".into(),
        signature_hex: String::new(),
        key_id: "key-offline-2026".into(),
        min_supported_version: "0.1.0".into(),
    };
    let sig: Signature = signing_key.sign(&manifest.signing_payload().unwrap());
    manifest.signature_hex = sig
        .to_der()
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    // Write package files
    fs::write(
        package_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    fs::write(package_dir.join("famshield-service.exe"), new_payload).unwrap();

    // 1. Dry-run verify offline directory
    let check = OfflineUpdateEngine::check_offline_directory(&package_dir, &engine).unwrap();
    match check {
        UpdateCheckResult::UpdateAvailable {
            new_version,
            current_version,
            ..
        } => {
            assert_eq!(current_version, "0.1.0");
            assert_eq!(new_version, "0.2.0");
        }
        other => panic!("Expected UpdateAvailable, got: {:?}", other),
    }

    // 2. Apply offline update
    let report =
        OfflineUpdateEngine::apply_offline_directory(&package_dir, &engine, &updater).unwrap();
    assert_eq!(report.previous_version, "0.1.0");
    assert_eq!(report.new_version, "0.2.0");
    assert_eq!(report.swapped_binary, "famshield-service.exe");

    // Assert active binary was atomically swapped
    assert_eq!(fs::read(&active_bin).unwrap(), new_payload);

    // Assert backup exists
    let backup_bin = report.backup_path;
    assert!(backup_bin.exists());
    assert_eq!(
        fs::read(&backup_bin).unwrap(),
        b"Active Service Binary v0.1.0"
    );

    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn test_offline_update_tampered_payload_rejected() {
    use famshield_updater::OfflineUpdateEngine;
    use sha2::{Digest, Sha256};

    let temp_root = std::env::temp_dir().join("famshield_test_offline_tampered");
    let package_dir = temp_root.join("USB_Tampered");
    let app_data = temp_root.join("ProgramData");
    let install_dir = temp_root.join("ProgramFiles");

    let _ = fs::remove_dir_all(&temp_root);
    fs::create_dir_all(&package_dir).unwrap();
    fs::create_dir_all(&install_dir).unwrap();

    let signing_key = SigningKey::from_bytes((&[8_u8; 32]).into()).unwrap();
    let verifying_key = signing_key.verifying_key();
    let engine = StagedUpdateEngine::new(
        "0.1.0",
        "key-offline-2026",
        verifying_key.to_encoded_point(false).as_bytes(),
    )
    .unwrap();
    let updater = WindowsAtomicUpdater::new("0.1.0", &app_data, &install_dir);

    let legit_payload = b"Legitimate Payload";
    let mut hasher = Sha256::new();
    hasher.update(legit_payload);
    let sha256 = format!("{:x}", hasher.finalize());

    let mut manifest = ReleaseManifest {
        manifest_version: 1,
        version: "0.2.0".into(),
        channel: "stable".into(),
        sha256,
        binary_url: "https://famshield.ttcantho.io.vn/download/famshield-service.exe".into(),
        signature_hex: String::new(),
        key_id: "key-offline-2026".into(),
        min_supported_version: "0.1.0".into(),
    };
    let sig: Signature = signing_key.sign(&manifest.signing_payload().unwrap());
    manifest.signature_hex = sig
        .to_der()
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    fs::write(
        package_dir.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();

    // Attacker replaced the binary on the USB with malicious code!
    fs::write(
        package_dir.join("famshield-service.exe"),
        b"Malicious Injected Trojan",
    )
    .unwrap();

    // Check must fail with ChecksumMismatch!
    let check = OfflineUpdateEngine::check_offline_directory(&package_dir, &engine);
    assert!(check.is_err());

    // Apply must fail and reject
    let apply = OfflineUpdateEngine::apply_offline_directory(&package_dir, &engine, &updater);
    assert!(apply.is_err());

    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn test_watchdog_health_verification_and_auto_rollback() {
    use famshield_updater::WatchdogStatus;
    use sha2::{Digest, Sha256};

    let temp_root =
        std::env::temp_dir().join(format!("famshield_watchdog_test_{}", std::process::id()));
    let app_data = temp_root.join("app_data");
    let install_dir = temp_root.join("install");
    fs::create_dir_all(&app_data).unwrap();
    fs::create_dir_all(&install_dir).unwrap();

    let binary_name = "famshield-service.exe";
    let active_bin = install_dir.join(binary_name);
    fs::write(&active_bin, b"ORIGINAL_V1.0.0_STABLE_BINARY").unwrap();

    let updater = WindowsAtomicUpdater::new("1.0.0", &app_data, &install_dir);

    // Initial state: normal
    assert_eq!(
        updater.check_and_recover_health().unwrap(),
        WatchdogStatus::Normal
    );

    // 1. Stage and swap to a new buggy v1.0.1 binary
    let v101_payload = b"BUGGY_V1.0.1_BINARY_THAT_WILL_CRASH";
    let mut hasher = Sha256::new();
    hasher.update(v101_payload);
    let sha256 = format!("{:x}", hasher.finalize());

    updater
        .stage_binary(binary_name, v101_payload, &sha256)
        .unwrap();
    updater.atomic_swap(binary_name).unwrap();
    updater.record_pending_update(binary_name, "1.0.1").unwrap();

    // Verify active binary is now v1.0.1
    assert_eq!(fs::read(&active_bin).unwrap(), v101_payload);

    // 2. First boot: Watchdog begins observation window (boot_attempts = 1)
    let boot1 = updater.check_and_recover_health().unwrap();
    assert_eq!(
        boot1,
        WatchdogStatus::PendingVerification {
            target_version: "1.0.1".into(),
            boot_attempts: 1,
        }
    );

    // 3. Simulating crash during observation: Service restarts without calling confirm_update_healthy()
    // Second boot: Watchdog detects crash and triggers automated rollback!
    let boot2 = updater.check_and_recover_health().unwrap();
    assert_eq!(
        boot2,
        WatchdogStatus::RolledBack {
            failed_version: "1.0.1".into(),
            restored_version: "1.0.0".into(),
        }
    );

    // Active binary MUST have been restored back to v1.0.0 stable binary!
    let restored_content = fs::read(&active_bin).unwrap();
    assert_eq!(restored_content, b"ORIGINAL_V1.0.0_STABLE_BINARY");

    // 4. Third boot: should now be back to Normal
    let boot3 = updater.check_and_recover_health().unwrap();
    assert_eq!(boot3, WatchdogStatus::Normal);

    // 5. Happy Path: stage new healthy v1.0.2 binary and confirm healthy
    let v102_payload = b"HEALTHY_V1.0.2_BINARY";
    let mut hasher = Sha256::new();
    hasher.update(v102_payload);
    let sha256_v102 = format!("{:x}", hasher.finalize());

    updater
        .stage_binary(binary_name, v102_payload, &sha256_v102)
        .unwrap();
    updater.atomic_swap(binary_name).unwrap();
    updater.record_pending_update(binary_name, "1.0.2").unwrap();

    assert_eq!(
        updater.check_and_recover_health().unwrap(),
        WatchdogStatus::PendingVerification {
            target_version: "1.0.2".into(),
            boot_attempts: 1,
        }
    );

    // Service confirms health after observation window
    updater.confirm_update_healthy().unwrap();
    assert_eq!(
        updater.check_and_recover_health().unwrap(),
        WatchdogStatus::Normal
    );

    let _ = fs::remove_dir_all(&temp_root);
}
