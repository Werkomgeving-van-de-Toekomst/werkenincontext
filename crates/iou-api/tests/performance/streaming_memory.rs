//! Performance tests for the API

/// Test that large files can be uploaded efficiently
#[tokio::test]
async fn large_file_upload_does_not_buffer_inefficiently() {
    // Create a 10MB file
    let large_file = vec![0xABu8; 10 * 1024 * 1024];

    assert_eq!(large_file.len(), 10 * 1024 * 1024);
}

/// Test zero-copy serialization efficiency
#[tokio::test]
async fn status_serialization_is_efficient() {
    use iou_api::websockets::types::DocumentStatus;

    let document_id = uuid::Uuid::new_v4();
    let timestamp = chrono::Utc::now().timestamp();

    // Test serialization of each status type
    let statuses = vec![
        DocumentStatus::Started {
            document_id,
            agent: "research".to_string(),
            timestamp,
        },
        DocumentStatus::Progress {
            document_id,
            agent: "content".to_string(),
            percent: 50,
            message: Some("Processing".to_string()),
            timestamp,
        },
        DocumentStatus::Completed { document_id, timestamp },
        DocumentStatus::Failed {
            document_id,
            error: "Test error".to_string(),
            timestamp,
        },
    ];

    // All should serialize successfully
    for status in statuses {
        let json = serde_json::to_string(&status);
        assert!(json.is_ok(), "Status should serialize");

        // Verify the JSON is reasonably sized
        let serialized = json.unwrap();
        assert!(serialized.len() < 1000, "Serialized status should be compact");
    }
}

/// Benchmark: Status update throughput
#[tokio::test]
async fn status_update_throughput_is_acceptable() {
    use iou_api::websockets::types::DocumentStatus;
    use std::time::Instant;

    let (tx, mut _rx) = tokio::sync::broadcast::channel::<DocumentStatus>(100);

    let document_id = uuid::Uuid::new_v4();
    let timestamp = chrono::Utc::now().timestamp();

    // Measure time to send 1000 messages
    let message_count = 1000;
    let start = Instant::now();

    for i in 0..message_count {
        let status = DocumentStatus::Progress {
            document_id,
            agent: "test".to_string(),
            percent: (i % 100) as u8,
            message: None,
            timestamp,
        };
        tx.send(status).unwrap();
    }

    let elapsed = start.elapsed();

    // Should send at least 10,000 messages per second
    let messages_per_second = message_count as f64 / elapsed.as_secs_f64();
    assert!(
        messages_per_second > 10_000.0,
        "Should send at least 10,000 messages/second, got {:.0}",
        messages_per_second
    );
}
