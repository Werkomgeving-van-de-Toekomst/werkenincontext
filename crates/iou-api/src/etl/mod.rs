//! ETL Pipeline for Supabase to DuckDB
//!
//! This module handles the Extract, Transform, Load (ETL) pipeline
//! that synchronizes data from the primary Supabase database
//! to the analytics DuckDB database.

mod config;
mod outbox;
mod pipeline;
mod tables;

pub use config::{EtlConfig, EtlSchedule};
pub use outbox::{OutboxConfig, OutboxEvent, OutboxProcessor, OutboxProcessResult, OutboxStats};
pub use pipeline::{EtlPipeline, EtlMetrics, EtlError};
pub use tables::{TableSync, TableSyncResult};
