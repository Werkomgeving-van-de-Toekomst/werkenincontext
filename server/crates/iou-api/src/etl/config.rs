//! ETL Configuration

use serde::{Deserialize, Serialize};

/// Configuration for the ETL pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtlConfig {
    /// How often to run the ETL (in seconds)
    pub interval_seconds: u64,

    /// Maximum number of records to transfer per batch
    pub batch_size: usize,

    /// Whether to use incremental updates (false = full refresh)
    pub incremental: bool,

    /// Whether ETL is currently enabled
    pub enabled: bool,
}

impl Default for EtlConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 300, // 5 minutes
            batch_size: 1000,
            incremental: true,
            enabled: true,
        }
    }
}

/// ETL scheduling options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EtlSchedule {
    /// Run ETL continuously on a timer
    Continuous,

    /// Run ETL once manually
    Once,

    /// ETL is disabled
    Disabled,
}
