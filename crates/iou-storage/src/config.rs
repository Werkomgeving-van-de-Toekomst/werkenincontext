//! Storage configuration

use serde::{Deserialize, Serialize};

/// Storage configuration loaded from environment or config file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// S3-compatible endpoint URL
    pub endpoint: String,
    /// Access key ID
    pub access_key_id: String,
    /// Secret access key
    pub secret_access_key: String,
    /// Bucket name for document storage
    pub bucket: String,
    /// Region (optional for MinIO)
    pub region: Option<String>,
    /// Whether to use path-style addressing (required for MinIO)
    pub force_path_style: bool,
}

impl StorageConfig {
    /// Load configuration from environment variables
    ///
    /// # Security
    /// This method requires STORAGE_ACCESS_KEY_ID and STORAGE_SECRET_ACCESS_KEY
    /// to be set. Default credentials are never used in production.
    ///
    /// For development/testing, use `minio_local()` instead.
    pub fn from_env() -> Result<Self, anyhow::Error> {
        Ok(Self {
            endpoint: std::env::var("STORAGE_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
            access_key_id: std::env::var("STORAGE_ACCESS_KEY_ID")
                .expect("STORAGE_ACCESS_KEY_ID must be set"),
            secret_access_key: std::env::var("STORAGE_SECRET_ACCESS_KEY")
                .expect("STORAGE_SECRET_ACCESS_KEY must be set"),
            bucket: std::env::var("STORAGE_BUCKET")
                .unwrap_or_else(|_| "iou-documents".to_string()),
            region: std::env::var("STORAGE_REGION").ok(),
            force_path_style: std::env::var("STORAGE_FORCE_PATH_STYLE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }

    /// Create a config for local MinIO development
    ///
    /// # Warning
    /// Only use this for local development. Never deploy with default credentials.
    pub fn minio_local() -> Self {
        Self {
            endpoint: "http://localhost:9000".to_string(),
            access_key_id: "minioadmin".to_string(),
            secret_access_key: "minioadmin".to_string(),
            bucket: "iou-documents".to_string(),
            region: None,
            force_path_style: true,
        }
    }
}

#[cfg(test)]
impl StorageConfig {
    pub fn test_mock() -> Self {
        Self {
            endpoint: "http://localhost:9000".to_string(),
            access_key_id: "test-key".to_string(),
            secret_access_key: "test-secret".to_string(),
            bucket: "test-bucket".to_string(),
            region: None,
            force_path_style: true,
        }
    }
}
