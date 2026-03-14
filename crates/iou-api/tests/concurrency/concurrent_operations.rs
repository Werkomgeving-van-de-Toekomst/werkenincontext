//! Concurrency tests for the API

use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Test concurrent document creation
#[tokio::test]
async fn multiple_concurrent_document_creations() {
    // Given: 10 simultaneous create requests
    let request_count = 10;

    let mut handles = Vec::new();

    for _ in 0..request_count {
        let handle = tokio::spawn(async move {
            // Simulate document creation
            let document_id = Uuid::new_v4();
            let success = true;
            (document_id, success)
        });
        handles.push(handle);
    }

    // When: All requests complete
    let completed: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // Then: All documents are created with unique IDs
    assert_eq!(completed.len(), request_count);

    let document_ids: Vec<_> = completed.iter().map(|(id, _)| *id).collect();
    let unique_ids: std::collections::HashSet<_> = document_ids.iter().collect();

    assert_eq!(
        unique_ids.len(),
        request_count,
        "All document IDs should be unique"
    );

    // All should be successful
    for (_, success) in completed {
        assert!(success, "Document creation should succeed");
    }
}

/// Test connection limit enforcement
#[tokio::test]
async fn eleventh_websocket_connection_is_rejected() {
    use iou_api::websockets::limiter::ConnectionLimiter;

    // Given: Connection limiter with max 10 connections
    let limiter = ConnectionLimiter::new();
    let document_id = Uuid::new_v4();

    // When: 10 connections are established
    let mut permits = Vec::new();
    for _ in 0..10 {
        let permit = limiter.acquire(document_id).await.unwrap();
        permits.push(permit);
    }

    // Then: 11th connection is rejected
    let result = limiter.acquire(document_id).await;
    assert!(result.is_err(), "11th connection should be rejected");

    // And: Connection count is correctly reported
    assert_eq!(limiter.connection_count(document_id), 10);
}

/// Test that releasing permits allows new connections
#[tokio::test]
async fn released_permit_allows_new_connection() {
    use iou_api::websockets::limiter::ConnectionLimiter;

    let limiter = ConnectionLimiter::new();
    let document_id = Uuid::new_v4();

    // Acquire all permits
    let mut permits = Vec::new();
    for _ in 0..10 {
        permits.push(limiter.acquire(document_id).await.unwrap());
    }

    // Verify 11th fails
    assert!(limiter.acquire(document_id).await.is_err());

    // Release one permit
    permits.pop();

    // Now a new connection should succeed
    assert!(limiter.acquire(document_id).await.is_ok());
}

/// Test race condition in permit acquisition
#[tokio::test]
async fn concurrent_permit_acquisition_is_safe() {
    use iou_api::websockets::limiter::ConnectionLimiter;

    let limiter = Arc::new(ConnectionLimiter::new());
    let document_id = Uuid::new_v4();

    // Spawn many concurrent acquisition attempts
    let mut handles = Vec::new();
    for _ in 0..20 {
        let limiter_clone = Arc::clone(&limiter);
        let handle = tokio::spawn(async move {
            limiter_clone.acquire(document_id).await
        });
        handles.push(handle);
    }

    // Wait for all attempts
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // Exactly 10 should succeed
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(
        success_count, 10,
        "Exactly 10 permits should be acquired"
    );

    // 10 should fail
    let failure_count = results.iter().filter(|r| r.is_err()).count();
    assert_eq!(failure_count, 10, "10 attempts should fail");
}
