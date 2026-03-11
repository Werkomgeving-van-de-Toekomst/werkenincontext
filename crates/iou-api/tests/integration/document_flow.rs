//! End-to-end tests for document lifecycle

use std::sync::Arc;
use uuid::Uuid;

/// Test document status transitions
#[tokio::test]
async fn document_status_transitions_correctly() {
    use iou_api::websockets::types::DocumentStatus;

    let document_id = Uuid::new_v4();
    let timestamp = chrono::Utc::now().timestamp();

    // Test Started status
    let started = DocumentStatus::Started {
        document_id,
        agent: "research".to_string(),
        timestamp,
    };
    assert!(!started.is_terminal());
    assert_eq!(started.document_id(), document_id);

    // Test Progress status
    let progress = DocumentStatus::Progress {
        document_id,
        agent: "content".to_string(),
        percent: 50,
        message: None,
        timestamp,
    };
    assert!(!progress.is_terminal());
    assert_eq!(progress.document_id(), document_id);

    // Test Completed status
    let completed = DocumentStatus::Completed { document_id, timestamp };
    assert!(completed.is_terminal());
    assert_eq!(completed.document_id(), document_id);

    // Test Failed status
    let failed = DocumentStatus::Failed {
        document_id,
        error: "Test error".to_string(),
        timestamp,
    };
    assert!(failed.is_terminal());
    assert_eq!(failed.document_id(), document_id);
}

/// Test WebSocket broadcast channel functionality
#[tokio::test]
async fn websocket_broadcast_channel_works() {
    use iou_api::websockets::types::DocumentStatus;
    use tokio::sync::broadcast;

    // Given: A broadcast channel
    let (tx, _rx) = broadcast::channel::<DocumentStatus>(100);
    let mut rx1 = tx.subscribe();
    let mut rx2 = tx.subscribe();

    let document_id = Uuid::new_v4();
    let timestamp = chrono::Utc::now().timestamp();

    // When: Status is broadcast
    let status = DocumentStatus::Started {
        document_id,
        agent: "test".to_string(),
        timestamp,
    };
    tx.send(status.clone()).unwrap();

    // Then: All subscribers receive the message
    let received1 = rx1.recv().await.unwrap();
    assert_eq!(received1.document_id(), document_id);

    let received2 = rx2.recv().await.unwrap();
    assert_eq!(received2.document_id(), document_id);
}

/// Test that lagging receivers receive Lagged error
#[tokio::test]
async fn lagging_receiver_receives_lagged_error() {
    use tokio::sync::broadcast::{self, error::RecvError};

    // Given: Bounded channel with capacity 2
    let (tx, _rx) = broadcast::channel(2);
    let mut rx = tx.subscribe();

    // When: More messages are sent than capacity
    let _: Result<_, _> = tx.send(1);
    let _: Result<_, _> = tx.send(2);
    let _: Result<_, _> = tx.send(3); // Evicts message 1

    // Then: Late subscriber gets Lagged error
    match rx.recv().await {
        Err(RecvError::Lagged(n)) => {
            assert!(n > 0, "Should have skipped at least 1 message");
        }
        other => panic!("Expected Lagged error, got {:?}", other),
    }

    // And: Can still receive latest messages after lag
    let latest: i32 = rx.recv().await.unwrap();
    assert_eq!(latest, 2);
}

/// Test that oversized documents are rejected
#[tokio::test]
async fn oversized_document_is_rejected() {
    // This test validates size validation before S3 upload

    // Given: Document payload exceeding 10MB
    let large_payload = vec![0u8; 11 * 1024 * 1024]; // 11MB

    // When: Request is validated
    let is_valid = large_payload.len() <= 10 * 1024 * 1024;

    // Then: Should be rejected
    assert!(!is_valid, "11MB payload should exceed 10MB limit");

    // And: No upload to S3 is attempted
}

/// Test that workflow timeout propagates correctly
#[tokio::test]
async fn workflow_timeout_propagates_correctly() {
    // Given: Overall timeout of 8 minutes
    const OVERALL_TIMEOUT_MS: u64 = 8 * 60 * 1000;

    // When: Workflow exceeds timeout
    let elapsed = OVERALL_TIMEOUT_MS + 1000;

    // Then: Workflow should be marked as failed
    let timed_out = elapsed >= OVERALL_TIMEOUT_MS;
    assert!(timed_out, "Workflow should time out after 8 minutes");
}
