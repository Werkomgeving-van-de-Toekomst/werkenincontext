//! Configuration for the API server

use std::env;

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
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()?,
            database_path: env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "data/iou-modern.duckdb".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "development-secret-change-in-production".to_string()),
        })
    }
}
