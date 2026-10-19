use childprotect_sync::{CloudEnrollmentClient, EnrollmentRequest};

#[tokio::test]
async fn test_enrollment_validation() {
    let client = CloudEnrollmentClient::new("https://api.childprotect.cloud");

    // Invalid code (not 6 digits)
    let bad_req = EnrollmentRequest {
        enrollment_code: "123".into(),
        device_name: "Desktop-PC".into(),
        os_version: "Windows 11".into(),
    };
    assert!(client.enroll_device(&bad_req).await.is_err());

    // Valid 6-digit code
    let valid_req = EnrollmentRequest {
        enrollment_code: "654321".into(),
        device_name: "Desktop-PC".into(),
        os_version: "Windows 11".into(),
    };
    let res = client.enroll_device(&valid_req).await.expect("Enrollment should succeed");
    assert_eq!(res.device_id, "dev-654321");
    assert_eq!(res.tenant_id, "tenant-default");
}
