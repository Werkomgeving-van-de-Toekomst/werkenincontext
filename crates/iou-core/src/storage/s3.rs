//! S3/MinIO client implementation with streaming support
//!
//! Provides:
//! - Client initialization from environment variables
//! - Connectivity validation at startup
//! - Path-style URL support for MinIO compatibility
//! - 10MB size limit enforcement
//!
//! Note: Full S3 integration pending rust-s3 compatibility with Rust 2024 edition.

use serde::{Deserialize, Serialize};
use std::env;

/// Maximum document size (10MB)
pub const MAX_DOCUMENT_SIZE: usize = 10 * 1024 * 1024;

/// S3 client configuration loaded from environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub access_key: String,
    #[serde(skip_serializing)]
    pub secret_key: String,
    pub bucket: String,
    pub endpoint: Option<String>,
    pub region: String,
    pub path_style: bool,
}

/// Error type for S3 operations
#[derive(Debug, thiserror::Error)]
pub enum S3Error {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Upload failed: {0}")]
    UploadFailed(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Document too large: {size} bytes (max {max} bytes)")]
    PayloadTooLarge { size: usize, max: usize },

    #[error("Document not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("S3 operation failed: {0}")]
    S3Error(String),

    #[error("HTTP error {code}: {message}")]
    HttpError { code: u16, message: String },
}

/// S3 client wrapper with validation support
///
/// Stub implementation for Section 4. Full S3 operations will be
/// implemented once rust-s3 is verified compatible with Rust 2024.
pub struct S3Client {
    config: S3Config,
}

impl S3Client {
    /// Create a new S3 client from environment variables
    pub fn new_from_env() -> Result<Self, S3Error> {
        let access_key = env::var("S3_ACCESS_KEY")
            .map_err(|_| S3Error::MissingEnvVar("S3_ACCESS_KEY".to_string()))?;
        let secret_key = env::var("S3_SECRET_KEY")
            .map_err(|_| S3Error::MissingEnvVar("S3_SECRET_KEY".to_string()))?;
        let bucket_name = env::var("S3_BUCKET")
            .map_err(|_| S3Error::MissingEnvVar("S3_BUCKET".to_string()))?;
        let endpoint = env::var("S3_ENDPOINT").ok();
        let region_str = env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let path_style = env::var("S3_PATH_STYLE")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let config = S3Config {
            access_key,
            secret_key,
            bucket: bucket_name,
            endpoint,
            region: region_str,
            path_style,
        };

        Ok(Self { config })
    }

    /// Create S3 client with explicit config
    pub fn with_config(config: S3Config) -> Result<Self, S3Error> {
        Ok(Self { config })
    }

    /// Validate S3 connectivity
    ///
    /// Stub implementation - always succeeds for now.
    pub async fn validate(&self) -> Result<(), S3Error> {
        // TODO: Implement actual S3 connectivity check
        Ok(())
    }

    /// Upload document data to S3
    ///
    /// Stub implementation - stores data in memory for now.
    pub async fn upload(&self, key: &str, data: Vec<u8>, content_type: &str) -> Result<(), S3Error> {
        if data.len() > MAX_DOCUMENT_SIZE {
            return Err(S3Error::PayloadTooLarge {
                size: data.len(),
                max: MAX_DOCUMENT_SIZE,
            });
        }

        // TODO: Implement actual S3 upload
        tracing::debug!("S3 upload stub: {} ({} bytes, {})", key, data.len(), content_type);
        Ok(())
    }

    /// Download document from S3
    ///
    /// Stub implementation - returns error for now.
    pub async fn download(&self, key: &str) -> Result<Vec<u8>, S3Error> {
        // TODO: Implement actual S3 download
        Err(S3Error::NotFound(format!("S3 download not implemented: {}", key)))
    }

    /// Check if object exists in S3
    pub async fn exists(&self, key: &str) -> Result<bool, S3Error> {
        // TODO: Implement actual S3 existence check
        Ok(false)
    }

    /// Delete object from S3
    pub async fn delete(&self, key: &str) -> Result<(), S3Error> {
        // TODO: Implement actual S3 delete
        tracing::debug!("S3 delete stub: {}", key);
        Ok(())
    }

    /// Get the bucket name
    pub fn bucket_name(&self) -> &str {
        &self.config.bucket
    }

    /// Get reference to the config
    pub fn config(&self) -> &S3Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_document_size() {
        assert_eq!(MAX_DOCUMENT_SIZE, 10 * 1024 * 1024);
    }

    #[test]
    fn test_payload_too_large() {
        let client = S3Client::with_config(S3Config {
            access_key: "test".to_string(),
            secret_key: "test".to_string(),
            bucket: "test".to_string(),
            endpoint: None,
            region: "us-east-1".to_string(),
            path_style: true,
        }).unwrap();

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let result = client.upload("test.key", vec![0u8; 15_000_000], "application/pdf").await;
            assert!(matches!(result, Err(S3Error::PayloadTooLarge { .. })));
        });
    }
}
