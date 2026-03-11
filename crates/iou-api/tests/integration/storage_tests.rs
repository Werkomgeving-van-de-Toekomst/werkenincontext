//! S3 storage integration tests

use crate::mocks::MockS3Client;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Test S3 upload functionality
#[tokio::test]
async fn s3_upload_succeeds() {
    let client = MockS3Client::new();
    let key = "test_document.pdf";
    let data = b"test document content".to_vec();

    client.upload(key, data.clone()).await.unwrap();

    assert!(client.contains_key(key).await);

    let downloaded = client.download(key).await.unwrap();
    assert_eq!(downloaded, data);
}

/// Test S3 download functionality
#[tokio::test]
async fn s3_download_returns_correct_data() {
    let client = MockS3Client::new();
    let key = "test_key";
    let original_data = b"original content".to_vec();

    client.upload(key, original_data.clone()).await.unwrap();

    let downloaded = client.download(key).await.unwrap();
    assert_eq!(downloaded, original_data);
}

/// Test S3 not found error
#[tokio::test]
async fn s3_download_nonexistent_returns_error() {
    let client = MockS3Client::new();

    let result = client.download("nonexistent_key").await;

    assert!(result.is_err());
}

/// Test S3 failure simulation
#[tokio::test]
async fn s3_upload_failure_can_be_simulated() {
    let client = MockS3Client::new();
    client.set_fail_uploads(true).await;

    let result = client.upload("test_key", vec![1, 2, 3]).await;

    assert!(result.is_err());
    assert!(!client.contains_key("test_key").await);
}

/// Test S3 download failure simulation
#[tokio::test]
async fn s3_download_failure_can_be_simulated() {
    let client = MockS3Client::new();
    let key = "test_key";
    let data = b"test data".to_vec();

    client.upload(key, data).await.unwrap();
    client.set_fail_downloads(true).await;

    let result = client.download(key).await;

    assert!(result.is_err());
}

/// Test upload with retry on transient error
#[tokio::test]
async fn s3_upload_retry_on_transient_error() {
    let client = MockS3Client::new();
    let key = "retry_key";
    let data = b"retry data".to_vec();

    // Simulate transient failure
    client.set_fail_uploads(true).await;

    let attempts = Arc::new(tokio::sync::Mutex::new(0));
    let attempts_clone = Arc::clone(&attempts);

    // Retry logic
    let mut success = false;
    for attempt in 0..3 {
        *attempts_clone.lock().await += 1;

        let result = client.upload(key, data.clone()).await;
        if result.is_ok() {
            success = true;
            break;
        }

        // After first attempt, disable failure
        if attempt == 0 {
            client.set_fail_uploads(false).await;
            sleep(Duration::from_millis(10)).await;
        }
    }

    assert!(success, "Upload should succeed after retry");
    assert_eq!(*attempts.lock().await, 2, "Should succeed on second attempt");
    assert!(client.contains_key(key).await);
}

/// Test concurrent uploads
#[tokio::test]
async fn s3_concurrent_uploads_succeed() {
    let client = Arc::new(MockS3Client::new());

    // Spawn multiple concurrent uploads
    let mut handles = Vec::new();
    for i in 0..10 {
        let client_clone = Arc::clone(&client);
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_key_{}", i);
            let data = vec![i as u8; 100];
            client_clone.upload(&key, data).await
        });
        handles.push(handle);
    }

    // Wait for all uploads to complete
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // All uploads should succeed
    for result in results {
        assert!(result.is_ok(), "Concurrent upload should succeed");
    }

    // Verify all keys exist
    for i in 0..10 {
        let key = format!("concurrent_key_{}", i);
        assert!(client.contains_key(&key).await);
    }
}

/// Test upload count tracking
#[tokio::test]
async fn s3_upload_count_increments_correctly() {
    let client = MockS3Client::new();

    assert_eq!(client.upload_count().await, 0);

    for i in 1..=5 {
        client.upload(&format!("key_{}", i), vec![i as u8]).await.unwrap();
        assert_eq!(client.upload_count().await, i);
    }
}

/// Test storage clearing
#[tokio::test]
async fn s3_clear_removes_all_data() {
    let client = MockS3Client::new();

    // Add some data
    for i in 0..5 {
        client.upload(&format!("key_{}", i), vec![i as u8]).await.unwrap();
    }

    assert_eq!(client.keys().await.len(), 5);

    // Clear storage
    client.clear().await;

    assert_eq!(client.keys().await.len(), 0);
    assert_eq!(client.upload_count().await, 0);
}

/// Test large file handling
#[tokio::test]
async fn s3_handles_large_files() {
    let client = MockS3Client::new();
    let key = "large_file.bin";

    // Create a 1MB file
    let large_data = vec![0xABu8; 1024 * 1024];

    client.upload(key, large_data.clone()).await.unwrap();

    assert!(client.contains_key(key).await);

    let downloaded = client.download(key).await.unwrap();
    assert_eq!(downloaded.len(), large_data.len());
    assert_eq!(downloaded, large_data);
}
