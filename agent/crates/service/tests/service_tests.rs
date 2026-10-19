use famshield_service::{dispatcher::IpcDispatcher, ipc::IpcMessage};
use famshield_storage::StorageEngine;

#[tokio::test]
async fn test_ipc_dispatcher_session_hello() {
    let storage = StorageEngine::in_memory().unwrap();
    let dispatcher = IpcDispatcher::new(storage);

    let msg = IpcMessage::SessionHello {
        session_id: 1,
        windows_username: "test_child".into(),
    };

    let reply = dispatcher.dispatch(msg).await;
    match reply {
        IpcMessage::Response {
            success, message, ..
        } => {
            assert!(success);
            assert!(message.contains("Session 1 registered"));
        }
        _ => panic!("Expected Response message"),
    }
}

#[tokio::test]
async fn test_ipc_dispatcher_visual_evidence_storage() {
    let storage = StorageEngine::in_memory().unwrap();
    let dispatcher = IpcDispatcher::new(storage.clone());

    let jpeg_bytes = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10]; // Mock JPEG header
    let msg = IpcMessage::VisualEvidenceSubmission {
        event_id: "evt-violation-99".into(),
        trigger: "app_blocked".into(),
        image_jpeg_bytes: jpeg_bytes,
    };

    let reply = dispatcher.dispatch(msg).await;
    match reply {
        IpcMessage::Response { success, .. } => {
            assert!(success);
        }
        _ => panic!("Expected Response message"),
    }
}

#[tokio::test]
async fn test_ipc_message_framing_roundtrip() {
    use famshield_service::{read_framed_message, write_framed_message};
    use std::io::Cursor;

    let original_msg = IpcMessage::ForegroundHeartbeat {
        session_id: 2,
        executable_name: "game.exe".into(),
        is_idle: false,
        idle_duration_ms: 0,
    };

    let mut stream_buffer = Vec::new();
    write_framed_message(&mut stream_buffer, &original_msg)
        .await
        .unwrap();

    assert!(stream_buffer.len() > 4);

    let mut reader = Cursor::new(stream_buffer);
    let decoded = read_framed_message(&mut reader).await.unwrap();

    assert_eq!(decoded, Some(original_msg));
}

#[tokio::test]
async fn test_ipc_message_max_size_protection() {
    use famshield_service::read_framed_message;
    use std::io::Cursor;

    // Craft a frame header claiming 500 KiB payload (> 256 KiB max quota)
    let oversized_len: u32 = 500 * 1024;
    let mut bad_header = Vec::new();
    bad_header.extend_from_slice(&oversized_len.to_le_bytes());
    bad_header.extend_from_slice(&[0u8; 16]);

    let mut reader = Cursor::new(bad_header);
    let result = read_framed_message(&mut reader).await;

    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("exceeds maximum allowed"));
}

#[test]
fn test_ipc_pipe_sddl_security_format() {
    use famshield_service::IPC_PIPE_SDDL;
    assert!(IPC_PIPE_SDDL.contains("SY")); // LocalSystem
    assert!(IPC_PIPE_SDDL.contains("BA")); // Builtin Administrators
    assert!(IPC_PIPE_SDDL.contains("AU")); // Authenticated Users
}
