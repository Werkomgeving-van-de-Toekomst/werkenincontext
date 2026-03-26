//! ETL Pipeline implementation

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub use super::config::{EtlConfig, EtlSchedule};

/// Errors that can occur during ETL
#[derive(Debug, thiserror::Error)]
pub enum EtlError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Transformation error: {0}")]
    Transformation(String),

    #[error("Load error: {0}")]
    Load(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Metrics from a single ETL run
#[derive(Debug, Clone, Default)]
pub struct EtlMetrics {
    pub records_transferred: usize,
    pub duration_ms: u64,
    pub tables_synced: Vec<String>,
    pub errors: Vec<String>,
}

/// The main ETL coordinator
pub struct EtlPipeline {
    /// Supabase connection pool (source)
    pub supabase_pool: PgPool,

    /// ETL configuration
    pub config: EtlConfig,

    /// Current schedule mode
    pub schedule: EtlSchedule,

    /// Last successful run timestamp
    pub last_run: tokio::sync::RwLock<Option<DateTime<Utc>>>,
}

impl EtlPipeline {
    /// Create a new ETL pipeline
    pub fn new(supabase_pool: PgPool, config: EtlConfig) -> Result<Self> {
        let schedule = if config.enabled {
            EtlSchedule::Continuous
        } else {
            EtlSchedule::Disabled
        };

        Ok(Self {
            supabase_pool,
            config,
            schedule,
            last_run: tokio::sync::RwLock::new(None),
        })
    }

    /// Create from environment variables
    pub fn from_env(supabase_pool: PgPool) -> Result<Self> {
        let interval = std::env::var("ETL_INTERVAL_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);

        let batch_size = std::env::var("ETL_BATCH_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);

        let incremental = std::env::var("ETL_INCREMENTAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);

        let enabled = std::env::var("ETL_ENABLED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);

        let config = EtlConfig {
            interval_seconds: interval,
            batch_size,
            incremental,
            enabled,
        };

        Self::new(supabase_pool, config)
    }
}
