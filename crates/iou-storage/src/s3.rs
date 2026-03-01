//! S3 client wrapper for document storage operations
//!
//! This module provides an async S3 client using AWS SDK for Rust,
//! compatible with AWS S3, MinIO, Garage, and other S3-compatible storage.

use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_config::Region;
use aws_sdk_s3::{Client, Config};
use aws_smithy_types::byte_stream::ByteStream;
use thiserror::Error;
use std::sync::Arc;

pub use crate::config::StorageConfig as S3Config;

#[derive(Error, Debug)]
pub enum S3Error {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Not found: {bucket}/{key}")]
    NotFound { bucket: String, key: String },

    #[error("S3 operation failed: {0}")]
    OperationFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, S3Error>;

/// S3 client wrapper with convenient methods for document operations
pub struct S3Client {
    config: S3Config,
    client: Arc<Client>,
    bucket: String,
}

impl S3Client {
    /// Create a new S3 client from configuration
    pub async fn new(config: S3Config) -> Result<Self> {
        let region_str = config.region.as_deref().unwrap_or("us-east-1").to_string();
        let region = Region::new(region_str);

        // Create credentials provider
        let creds = aws_sdk_s3::config::Credentials::new(
            &config.access_key_id,
            &config.secret_access_key,
            None,
            None,
            "iou-storage",
        );

        // Build S3 config
        let s3_config = Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(region)
            .endpoint_url(config.endpoint.clone())
            .credentials_provider(creds)
            .force_path_style(config.force_path_style)
            .build();

        let client = Arc::new(Client::from_conf(s3_config));
        let bucket = config.bucket.clone();

        Ok(Self {
            config,
            client,
            bucket,
        })
    }

    /// Create a new S3 client synchronously (for testing/development)
    ///
    /// NOTE: This creates a client without async config loading.
    /// Use `new()` for production.
    pub fn new_sync(config: S3Config) -> Result<Self> {
        let region_str = config.region.as_deref().unwrap_or("us-east-1").to_string();
        let region = Region::new(region_str);

        let creds = aws_sdk_s3::config::Credentials::new(
            &config.access_key_id,
            &config.secret_access_key,
            None,
            None,
            "iou-storage",
        );

        let s3_config = Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(region)
            .endpoint_url(config.endpoint.clone())
            .credentials_provider(creds)
            .force_path_style(config.force_path_style)
            .build();

        let client = Arc::new(Client::from_conf(s3_config));
        let bucket = config.bucket.clone();

        Ok(Self {
            config,
            client,
            bucket,
        })
    }

    /// Check if client is properly configured
    pub fn is_ready(&self) -> bool {
        !self.config.access_key_id.is_empty() && !self.config.secret_access_key.is_empty()
    }

    /// Generate a storage key for a document
    pub fn document_key(document_id: &str, version: i32, format: &str) -> String {
        format!("documents/{}/v{}.{}", document_id, version, format)
    }

    /// Generate a storage key for a redacted document
    pub fn redacted_document_key(document_id: &str, version: i32, format: &str) -> String {
        format!("documents/{}/v{}.redacted.{}", document_id, version, format)
    }

    /// Get the bucket name
    pub fn bucket_name(&self) -> &str {
        &self.bucket
    }
}

/// Async storage operations trait
#[async_trait]
pub trait StorageOperations: Send + Sync {
    /// Put data to S3
    async fn put(&self, key: &str, data: Vec<u8>, content_type: &str) -> Result<String>;

    /// Get data from S3
    async fn get(&self, key: &str) -> Result<Vec<u8>>;

    /// Delete data from S3
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if key exists in S3
    async fn exists(&self, key: &str) -> Result<bool>;
}

#[async_trait]
impl StorageOperations for S3Client {
    async fn put(&self, key: &str, data: Vec<u8>, content_type: &str) -> Result<String> {
        let body = ByteStream::from(data);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(body)
            .send()
            .await
            .map_err(|e| S3Error::OperationFailed(format!("put failed: {}", e)))?;

        Ok(format!("{}/{}", self.bucket, key))
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                // Check for not found variants
                if e.to_string().contains("NoSuchKey") || e.to_string().contains("NotFound") {
                    S3Error::NotFound {
                        bucket: self.bucket.clone(),
                        key: key.to_string(),
                    }
                } else {
                    S3Error::OperationFailed(format!("get failed: {}", e))
                }
            })?;

        let data = output
            .body
            .collect()
            .await
            .map_err(|e| S3Error::OperationFailed(format!("read body failed: {}", e)))?
            .into_bytes();

        Ok(data.to_vec())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| S3Error::OperationFailed(format!("delete failed: {}", e)))?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("NotFound") || err_str.contains("NoSuchKey") {
                    Ok(false)
                } else {
                    Err(S3Error::OperationFailed(format!("exists check failed: {}", e)))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_key_generation() {
        let key = S3Client::document_key("uuid-here", 1, "md");
        assert_eq!(key, "documents/uuid-here/v1.md");

        let redacted_key = S3Client::redacted_document_key("uuid-here", 1, "md");
        assert_eq!(redacted_key, "documents/uuid-here/v1.redacted.md");
    }

    #[test]
    fn test_s3_client_sync_creation() {
        let config = S3Config::test_mock();
        let client = S3Client::new_sync(config).unwrap();
        assert!(client.is_ready());
        assert_eq!(client.bucket_name(), "test-bucket");
    }
}
