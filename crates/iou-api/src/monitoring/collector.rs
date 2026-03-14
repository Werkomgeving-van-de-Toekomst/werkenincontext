//! Collects and exports performance metrics for Supabase/PostgreSQL.

use sqlx::PgPool;
use std::time::Duration;
use tracing::debug;

/// Query performance statistics from pg_stat_statements
#[derive(Debug, Clone)]
pub struct QueryStats {
    pub query_count: i64,
    pub total_exec_time_ms: f64,
    pub min_exec_time_ms: f64,
    pub max_exec_time_ms: f64,
    pub mean_exec_time_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
}

impl Default for QueryStats {
    fn default() -> Self {
        Self {
            query_count: 0,
            total_exec_time_ms: 0.0,
            min_exec_time_ms: 0.0,
            max_exec_time_ms: 0.0,
            mean_exec_time_ms: 0.0,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
        }
    }
}

/// Database-level metrics
#[derive(Debug, Clone)]
pub struct DbStats {
    pub active_connections: i32,
    pub idle_connections: i32,
    pub max_connections: i32,
    pub replication_lag_secs: Option<i64>,
    pub wal_size_bytes: i64,
    pub disk_usage_bytes: i64,
    pub disk_free_bytes: i64,
    pub disk_free_percent: f64,
}

impl Default for DbStats {
    fn default() -> Self {
        Self {
            active_connections: 0,
            idle_connections: 0,
            max_connections: 20,
            replication_lag_secs: None,
            wal_size_bytes: 0,
            disk_usage_bytes: 0,
            disk_free_bytes: 0,
            disk_free_percent: 100.0,
        }
    }
}

/// Real-time subscription metrics
#[derive(Debug, Clone)]
pub struct RealtimeStats {
    pub active_subscriptions: i32,
    pub messages_per_second: f64,
}

impl Default for RealtimeStats {
    fn default() -> Self {
        Self {
            active_subscriptions: 0,
            messages_per_second: 0.0,
        }
    }
}

/// Aggregated system metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub query_stats: QueryStats,
    pub db_stats: DbStats,
    pub realtime_stats: RealtimeStats,
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

/// Collects and exports performance metrics
pub struct MetricsCollector {
    pool: PgPool,
    collection_interval: Duration,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(pool: PgPool, collection_interval: Duration) -> Self {
        Self {
            pool,
            collection_interval,
        }
    }

    /// Collect query performance statistics from pg_stat_statements
    pub async fn collect_query_stats(&self) -> Result<QueryStats, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT
                COALESCE(SUM(calls), 0) as query_count,
                COALESCE(SUM(total_exec_time), 0.0) as total_exec_time,
                COALESCE(MIN(min_exec_time), 0.0) as min_exec_time,
                COALESCE(MAX(max_exec_time), 0.0) as max_exec_time,
                COALESCE(SUM(total_exec_time) / NULLIF(SUM(calls), 0), 0.0) as mean_exec_time
            FROM pg_stat_statements
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let mean = row.mean_exec_time;

                // Estimate percentiles from mean
                // In production, you'd use actual percentile calculations
                // from pg_stat_statements or a dedicated metrics system
                Ok(QueryStats {
                    query_count: row.query_count.unwrap_or(0),
                    total_exec_time_ms: row.total_exec_time.unwrap_or(0.0),
                    min_exec_time_ms: row.min_exec_time.unwrap_or(0.0),
                    max_exec_time_ms: row.max_exec_time.unwrap_or(0.0),
                    mean_exec_time_ms: mean,
                    // Estimates based on typical latency distribution
                    p50_latency_ms: mean * 0.8,
                    p95_latency_ms: mean * 1.5,
                    p99_latency_ms: mean * 2.0,
                })
            }
            None => {
                debug!("pg_stat_statements not available or no data");
                Ok(QueryStats::default())
            }
        }
    }

    /// Collect database-level metrics
    pub async fn collect_db_stats(&self) -> Result<DbStats, sqlx::Error> {
        // Get connection statistics
        let conn_stats = sqlx::query!(
            r#"
            SELECT
                (SELECT count(*) FROM pg_stat_activity WHERE state = 'active') as active_connections,
                (SELECT count(*) FROM pg_stat_activity WHERE state = 'idle') as idle_connections,
                (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_connections
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        // Get database size information
        let size_stats = sqlx::query!(
            r#"
            SELECT
                pg_database_size(current_database()) as db_size_bytes
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(DbStats {
            active_connections: conn_stats
                .as_ref()
                .and_then(|r| r.active_connections)
                .unwrap_or(0),
            idle_connections: conn_stats
                .as_ref()
                .and_then(|r| r.idle_connections)
                .unwrap_or(0),
            max_connections: conn_stats
                .as_ref()
                .and_then(|r| r.max_connections)
                .unwrap_or(20),
            replication_lag_secs: None, // Would require CDC setup
            wal_size_bytes: 0,          // Would require pg_walfile_name() access
            disk_usage_bytes: size_stats
                .as_ref()
                .and_then(|r| r.db_size_bytes)
                .unwrap_or(0),
            disk_free_bytes: 0, // Would require system-level access
            disk_free_percent: 100.0, // Would require df-like access
        })
    }

    /// Collect real-time subscription metrics
    pub async fn collect_realtime_stats(&self) -> Result<RealtimeStats, sqlx::Error> {
        // In production, this would query Supabase Realtime's internal metrics
        // For now, we check if there's a realtime_subscription table
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM (
                SELECT 1
                FROM information_schema.tables
                WHERE table_name = 'realtime_subscription'
                LIMIT 1
            ) as t
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(RealtimeStats {
            active_subscriptions: 0, // Would query actual subscriptions
            messages_per_second: 0.0, // Would calculate from message logs
        })
    }

    /// Collect all system metrics
    pub async fn collect_all(&self) -> Result<SystemMetrics, sqlx::Error> {
        debug!("Collecting system metrics");

        let query_stats = self.collect_query_stats().await?;
        let db_stats = self.collect_db_stats().await?;
        let realtime_stats = self.collect_realtime_stats().await?;

        Ok(SystemMetrics {
            query_stats,
            db_stats,
            realtime_stats,
            collected_at: chrono::Utc::now(),
        })
    }

    /// Get the collection interval
    pub fn interval(&self) -> Duration {
        self.collection_interval
    }
}
