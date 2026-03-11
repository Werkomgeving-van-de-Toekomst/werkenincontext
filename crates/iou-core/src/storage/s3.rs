//! S3/MinIO client implementation with startup validation
//!
//! Provides:
//! - Client initialization from environment variables
//! - Connectivity validation at startup
//! - Path-style URL support for MinIO compatibility
//! - Streaming operations for efficient memory usage
//!
//! Full implementation will be completed in Section 4.

use serde::{Deserialize, Serialize};
use std::env;

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

    #[error("S3 connection failed: {0}")]
    ConnectionFailed(String),

    #[error("S3 operation failed: {0}")]
    OperationFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// S3 client wrapper with validation support
///
/// Full implementation in Section 4. This stub provides the basic structure.
#[derive(Debug)]
pub struct S3Client {
    config: S3Config,
}

impl S3Client {
    /// Create a new S3 client from environment variables
    ///
    /// Required environment variables:
    /// - S3_ACCESS_KEY: Access key for S3
    /// - S3_SECRET_KEY: Secret key for S3
    /// - S3_BUCKET: Bucket name
    /// - S3_ENDPOINT: Optional endpoint URL (for MinIO)
    /// - S3_REGION: AWS region (default: us-east-1)
    /// - S3_PATH_STYLE: Use path-style URLs (default: true for MinIO)
    pub fn new_from_env() -> Result<Self, S3Error> {
        let access_key = env::var("S3_ACCESS_KEY")
            .map_err(|_| S3Error::MissingEnvVar("S3_ACCESS_KEY".to_string()))?;
        let _secret_key = env::var("S3_SECRET_KEY")
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
            secret_key: "***".to_string(), // Don't store actual secret
            bucket: bucket_name,
            endpoint,
            region: region_str,
            path_style,
        };

        Ok(Self { config })
    }

    /// Validate S3 connectivity
    ///
    /// Tests connection to S3/MinIO endpoint to verify credentials
    /// and network access. Called at application startup.
    /// Full implementation in Section 4.
    pub async fn validate(&self) -> Result<(), S3Error> {
        // Stub implementation - always succeeds for now
        // Full validation will be implemented in Section 4
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
    fn test_new_from_env_missing_access_key() {
        // Unset the environment variable if it exists
        // Note: remove_var is unsafe in Rust 2024 edition
        unsafe { std::env::remove_var("S3_ACCESS_KEY") };
        unsafe { std::env::remove_var("S3_SECRET_KEY") };
        unsafe { std::env::remove_var("S3_BUCKET") };

        let result = S3Client::new_from_env();
        assert!(result.is_err());
        match result.unwrap_err() {
            S3Error::MissingEnvVar(var) => assert_eq!(var, "S3_ACCESS_KEY"),
            _ => panic!("Expected MissingEnvVar error"),
        }
    }

    #[test]
    fn test_s3config_fields() {
        let config = S3Config {
            access_key: "test_key".to_string(),
            secret_key: "super_secret".to_string(),
            bucket: "test-bucket".to_string(),
            endpoint: Some("http://localhost:9000".to_string()),
            region: "us-east-1".to_string(),
            path_style: true,
        };

        assert_eq!(config.access_key, "test_key");
        assert_eq!(config.bucket, "test-bucket");
        assert_eq!(config.region, "us-east-1");
        assert!(config.path_style);
        assert!(config.endpoint.is_some());
    }
}
