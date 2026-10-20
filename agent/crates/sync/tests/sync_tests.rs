use famshield_sync::{CloudEnrollmentClient, EnrollmentRequest};

#[tokio::test]
async fn test_enrollment_validation() {
    let client = CloudEnrollmentClient::new("https://api.famshield.cloud");

    // Invalid code (not 6 digits)
    let bad_req = EnrollmentRequest {
        enrollment_code: "123".into(),
        device_name: "Desktop-PC".into(),
        os_version: "Windows 11".into(),
    };
    assert!(client.enroll_device(&bad_req).await.is_err());

    // A syntactically valid code must still fail closed if the server is unavailable.
    let valid_req = EnrollmentRequest {
        enrollment_code: "654321".into(),
        device_name: "Desktop-PC".into(),
        os_version: "Windows 11".into(),
    };
    let res = client.enroll_device(&valid_req).await;
    assert!(res.is_err());
}
