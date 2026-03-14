//! Mock S3 client for testing upload/download flows

use iou_core::storage::{S3Client, S3Config, S3Error};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock S3 client that stores data in memory for testing
pub struct MockS3Client {
    storage: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    fail_uploads: Arc<RwLock<bool>>,
    fail_downloads: Arc<RwLock<bool>>,
    upload_count: Arc<RwLock<usize>>,
}

impl MockS3Client {
    /// Create a new mock S3 client
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            fail_uploads: Arc::new(RwLock::new(false)),
            fail_downloads: Arc::new(RwLock::new(false)),
            upload_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Upload data to a key
    pub async fn upload(&self, key: &str, data: Vec<u8>) -> Result<(), S3Error> {
        *self.upload_count.write().await += 1;
        if *self.fail_uploads.read().await {
            return Err(S3Error::UploadFailed(
                "Simulated upload failure".to_string(),
            ));
        }
        self.storage.write().await.insert(key.to_string(), data);
        Ok(())
    }

    /// Download data from a key
    pub async fn download(&self, key: &str) -> Result<Vec<u8>, S3Error> {
        if *self.fail_downloads.read().await {
            return Err(S3Error::DownloadFailed(
                "Simulated download failure".to_string(),
            ));
        }
        self.storage
            .read()
            .await
            .get(key)
            .cloned()
            .ok_or_else(|| S3Error::NotFound(format!("Key not found: {}", key)))
    }

    /// Check if a key exists
    pub async fn contains_key(&self, key: &str) -> bool {
        self.storage.read().await.contains_key(key)
    }

    /// Get the number of uploads performed
    pub async fn upload_count(&self) -> usize {
        *self.upload_count.read().await
    }

    /// Set whether uploads should fail
    pub async fn set_fail_uploads(&self, fail: bool) {
        *self.fail_uploads.write().await = fail;
    }

    /// Set whether downloads should fail
    pub async fn set_fail_downloads(&self, fail: bool) {
        *self.fail_downloads.write().await = fail;
    }

    /// Clear all stored data
    pub async fn clear(&self) {
        self.storage.write().await.clear();
        *self.upload_count.write().await = 0;
    }

    /// Get all stored keys
    pub async fn keys(&self) -> Vec<String> {
        self.storage.read().await.keys().cloned().collect()
    }
}

impl Default for MockS3Client {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_s3_upload_and_download() {
        let client = MockS3Client::new();
        let data = b"test data".to_vec();

        client.upload("test_key", data.clone()).await.unwrap();
        assert!(client.contains_key("test_key").await);

        let downloaded = client.download("test_key").await.unwrap();
        assert_eq!(downloaded, data);
    }

    #[tokio::test]
    async fn mock_s3_upload_failure() {
        let client = MockS3Client::new();
        client.set_fail_uploads(true).await;

        let result = client.upload("test_key", vec![1, 2, 3]).await;
        assert!(result.is_err());
        assert!(!client.contains_key("test_key").await);
    }

    #[tokio::test]
    async fn mock_s3_download_failure() {
        let client = MockS3Client::new();
        client.set_fail_downloads(true).await;

        let result = client.download("test_key").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn mock_s3_not_found() {
        let client = MockS3Client::new();

        let result = client.download("nonexistent").await;
        assert!(matches!(result, Err(S3Error::NotFound(_))));
    }

    #[tokio::test]
    async fn mock_s3_upload_count() {
        let client = MockS3Client::new();

        assert_eq!(client.upload_count().await, 0);

        client.upload("key1", vec![1]).await.unwrap();
        assert_eq!(client.upload_count().await, 1);

        client.upload("key2", vec![2]).await.unwrap();
        assert_eq!(client.upload_count().await, 2);
    }

    #[tokio::test]
    async fn mock_s3_clear() {
        let client = MockS3Client::new();

        client.upload("key1", vec![1]).await.unwrap();
        client.upload("key2", vec![2]).await.unwrap();

        assert_eq!(client.keys().await.len(), 2);

        client.clear().await;

        assert_eq!(client.keys().await.len(), 0);
        assert_eq!(client.upload_count().await, 0);
    }
}
