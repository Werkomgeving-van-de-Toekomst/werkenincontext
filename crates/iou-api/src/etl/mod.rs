//! ETL Pipeline for Supabase to DuckDB
//!
//! This module handles the Extract, Transform, Load (ETL) pipeline
//! that synchronizes data from the primary Supabase database
//! to the analytics DuckDB database.

mod config;
mod pipeline;
mod tables;

pub use config::{EtlConfig, EtlSchedule};
pub use pipeline::{EtlPipeline, EtlMetrics, EtlError};
pub use tables::{TableSync, TableSyncResult};
