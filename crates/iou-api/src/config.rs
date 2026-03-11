//! Configuration for the API server

use iou_core::storage::S3Config;

/// Server configuration
#[allow(dead_code)]
pub struct Config {
    /// Host to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// Path to DuckDB database file
    pub database_path: String,
    /// JWT secret for authentication
    pub jwt_secret: String,
    /// S3 storage configuration (re-exported from iou-core)
    pub s3: S3Config,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()?,
            database_path: std::env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "data/iou-modern.duckdb".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "development-secret-change-in-production".to_string()),
            s3: S3Config {
                access_key: std::env::var("S3_ACCESS_KEY").unwrap_or_default(),
                secret_key: std::env::var("S3_SECRET_KEY").unwrap_or_default(),
                bucket: std::env::var("S3_BUCKET").unwrap_or_default(),
                endpoint: std::env::var("S3_ENDPOINT").ok(),
                region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
                path_style: std::env::var("S3_PATH_STYLE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
        })
    }
}
