use childprotect_updater::UpdatePackageVerifier;

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
