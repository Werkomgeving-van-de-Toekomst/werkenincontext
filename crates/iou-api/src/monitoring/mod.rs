//! Monitoring and metrics collection for Supabase/PostgreSQL.
//!
//! This module integrates with PostgreSQL's pg_stat_statements to capture
//! query performance metrics and exports them for monitoring.

mod alerting;
mod collector;

pub use alerting::{Alert, AlertEngine, AlertThresholds};
pub use collector::{DbStats, MetricsCollector, QueryStats, RealtimeStats, SystemMetrics};
