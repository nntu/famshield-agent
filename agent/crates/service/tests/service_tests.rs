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
