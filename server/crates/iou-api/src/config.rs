//! Configuration for the API server

use iou_core::storage::S3Config;

/// Supabase Realtime configuration
#[derive(Debug, Clone)]
pub struct RealtimeConfig {
    /// Supabase Realtime WebSocket URL
    pub websocket_url: String,
    /// JWT token for Supabase Realtime authentication
    pub jwt_token: Option<String>,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Maximum reconnect backoff in seconds
    pub max_reconnect_backoff: u64,
    /// Whether to automatically reconnect on connection loss
    pub auto_reconnect: bool,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            websocket_url: String::new(),
            jwt_token: None,
            heartbeat_interval: 30,
            connect_timeout: 10,
            max_reconnect_backoff: 60,
            auto_reconnect: true,
        }
    }
}

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
    /// Supabase Realtime configuration
    pub realtime: RealtimeConfig,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        let websocket_url = std::env::var("SUPABASE_REALTIME_URL").ok();
        let jwt_token = std::env::var("SUPABASE_JWT_TOKEN").ok();

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
            realtime: RealtimeConfig {
                websocket_url: websocket_url.unwrap_or_default(),
                jwt_token,
                heartbeat_interval: std::env::var("REALTIME_HEARTBEAT_INTERVAL")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                connect_timeout: std::env::var("REALTIME_CONNECT_TIMEOUT")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                max_reconnect_backoff: std::env::var("REALTIME_MAX_RECONNECT_BACKOFF")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
                auto_reconnect: std::env::var("REALTIME_AUTO_RECONNECT")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
        })
    }
}
