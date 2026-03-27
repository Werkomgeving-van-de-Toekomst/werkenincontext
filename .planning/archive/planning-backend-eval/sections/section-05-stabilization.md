# Section 5: Stabilization</think># Section 5: Stabilization

**Phase 4 | Weeks 10-11**

This section covers the monitoring, optimization, and stabilization activities that occur after the primary traffic cutover to Supabase. The goal is to ensure the system meets or exceeds the performance baseline established in Phase 0 and is production-ready.

## Prerequisites

This section depends on completion of:
- **Section 4 (Cutover)** - All API endpoints must be reading from Supabase, ETL pipeline operational, dual-write pattern removed

## Overview

After completing the cutover to Supabase, the system enters a stabilization period where we monitor performance, optimize queries, tune RLS policies, stabilize the ETL pipeline, and address any production issues. This is a critical period before cleanup begins, as rollback becomes increasingly complex once the system has been running on Supabase for an extended period.

## Success Criteria

The stabilization phase is complete when:
1. Performance meets or exceeds Phase 0 baseline (p50/p95/p99 latencies)
2. ETL latency is acceptable and consistent
3. No data inconsistencies between Supabase and DuckDB
4. Monitoring and alerting are fully operational
5. User feedback is positive

## Tests

Create the following test files to validate stabilization activities.

### Performance Tests

File: `crates/iou-api/tests/stabilization_performance.rs`

```rust
//! Performance regression tests comparing current metrics to Phase 0 baseline.
//! 
//! These tests should run in CI to detect performance regressions before they
//! reach production.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

/// Baseline values captured in Phase 0 (section-01-assessment)
/// These should be loaded from a config file or database
struct PerformanceBaseline {
    query_p50_ms: f64,
    query_p95_ms: f64,
    query_p99_ms: f64,
    concurrent_users: u32,
    database_size_bytes: u64,
}

/// Test: Query performance regression
/// Compares current query latencies against Phase 0 baseline
#[cfg(test)]
mod query_performance_tests {
    use super::*;

    #[tokio::test]
    async fn document_list_query_meets_baseline() {
        // Test that document list query performs within baseline
        // Fetch p50, p95, p99 latencies
        // Assert each is within 10% of baseline
    }

    #[tokio::test]
    async fn search_query_meets_baseline() {
        // Test full-text search performance with tsvector
        // Should be significantly better than DuckDB ILIKE baseline
    }

    #[tokio::test]
    async fn information_domain_query_meets_baseline() {
        // Test domain-related queries
        // Verify joins and aggregations perform well
    }
}

/// Test: RLS policy optimization verification
#[cfg(test)]
mod rls_performance_tests {
    #[tokio::test]
    async fn rls_policy_check_under_500ms_p95() {
        // Verify RLS policy checks complete in <500ms at p95
        // This is critical for user experience
    }

    #[tokio::test]
    async fn multi_organization_query_performance() {
        // Test queries spanning multiple organizations
        // Verify no cross-org data leakage
    }
}

/// Test: Concurrent user load
#[cfg(test)]
mod load_tests {
    #[tokio::test]
    async fn supports_target_concurrent_users() {
        // Verify system supports the target concurrent user count
        // established in Phase 0 baseline
    }
}
```

### Monitoring Tests

File: `crates/iou-api/tests/stabilization_monitoring.rs`

```rust
//! Tests for monitoring and alerting infrastructure.

#[cfg(test)]
mod monitoring_tests {
    /// Test: PostgreSQL query metrics export
    #[tokio::test]
    async fn pg_stat_statements_accessible() {
        // Verify pg_stat_statements extension is available
        // Verify query statistics are being collected
    }

    /// Test: Replication lag monitoring
    #[tokio::test]
    async fn replication_lag_metrics_exported() {
        // Verify replication lag metrics are exported
        // If using CDC for ETL
    }

    /// Test: Replication slot WAL size
    #[tokio::test]
    async fn wal_size_alert_threshold() {
        // Verify alert triggers when WAL size exceeds threshold
        // Default threshold: 1GB
    }

    /// Test: Real-time subscription count
    #[tokio::test]
    async fn realtime_subscription_metrics_exported() {
        // Verify real-time subscription count metrics are exported
        // Useful for detecting abnormal spikes
    }

    /// Test: Database connection pool metrics
    #[tokio::test]
    async fn connection_pool_metrics_exported() {
        // Verify connection pool metrics are exported
        // Active, idle, and max connections
    }

    /// Test: Disk space monitoring
    #[tokio::test]
    async fn disk_space_alerts_trigger() {
        // Verify alerts trigger when disk space < 20%
        // Test with threshold configuration
    }
}
```

### ETL Stability Tests

File: `crates/iou-api/tests/stabilization_etl.rs`

```rust
//! Tests for ETL pipeline stability and consistency.

#[cfg(test)]
mod etl_tests {
    /// Test: ETL pipeline consistency over time
    #[tokio::test]
    async fn etl_produces_consistent_results() {
        // Run ETL multiple times with same source data
        // Verify results are identical
    }

    /// Test: ETL latency measurement
    #[tokio::test]
    async fn etl_completes_within_window() {
        // Verify ETL completes within configured window
        // Default: 5 minutes for batch ETL
    }

    /// Test: ETL error handling
    #[tokio::test]
    async fn etl_handles_partial_failure() {
        // Simulate partial ETL failure
        // Verify recovery and retry logic works
    }

    /// Test: DuckDB analytics queries still work
    #[tokio::test]
    async fn duckdb_analytics_queries_functional() {
        // Verify analytics queries work with ETL data
        // Test views: v_compliance_overview, v_domain_statistics
    }

    /// Test: Transactional outbox processing
    #[tokio::test]
    async fn outbox_events_processed_in_order() {
        // Verify events are processed in order
        // Verify no events are lost or duplicated
    }
}
```

## Implementation Tasks

### Task 1: Monitor Production Performance Metrics

**Objective:** Establish continuous monitoring of key performance indicators.

**Files to create/modify:**

1. `crates/iou-api/src/monitoring/collector.rs`

```rust
//! Collects and exports performance metrics for Supabase/PostgreSQL.
//!
//! This module integrates with PostgreSQL's pg_stat_statements to capture
//! query performance metrics and exports them for monitoring.

use sqlx::PgPool;
use std::time::Duration;

pub struct MetricsCollector {
    pool: PgPool,
    collection_interval: Duration,
}

impl MetricsCollector {
    /// Collect query performance statistics from pg_stat_statements
    pub async fn collect_query_stats(&self) -> Result<QueryStats, Error> {
        // Query pg_stat_statements for:
        // - Query execution counts
        // - Total execution time
        // - Min/max/mean execution times
        // Calculate p50, p95, p99 latencies
    }

    /// Collect database-level metrics
    pub async fn collect_db_stats(&self) -> Result<DbStats, Error> {
        // Connection pool stats
        // Replication lag (if using CDC)
        // WAL size
        // Disk usage
    }

    /// Collect real-time subscription metrics
    pub async fn collect_realtime_stats(&self) -> Result<RealtimeStats, Error> {
        // Active subscription count
        // Message throughput
    }
}
```

2. `crates/iou-api/src/monitoring/alerting.rs`

```rust
//! Alerting configuration and trigger logic.
//!
//! Defines alert thresholds and evaluates metrics against them.

pub struct AlertThresholds {
    pub query_p95_ms: f64,        // Default: 500ms
    pub replication_lag_secs: u64, // Default: 30s
    pub wal_size_bytes: u64,       // Default: 1GB
    pub disk_free_percent: f64,    // Default: 20%
    pub auth_failure_rate: f64,    // Default: 1%
}

pub struct AlertEngine {
    thresholds: AlertThresholds,
}

impl AlertEngine {
    /// Evaluate metrics against thresholds and trigger alerts if needed
    pub fn evaluate(&self, metrics: &SystemMetrics) -> Vec<Alert> {
        // Compare metrics to thresholds
        // Return list of triggered alerts
    }
}
```

**Monitoring Dashboard Setup:**

Configure a monitoring dashboard (Grafana or similar) to display:
- Query latency histogram (p50, p95, p99)
- Replication lag graph
- Connection pool utilization
- Real-time subscription count
- Disk space usage

**Alert Configuration:**

Set up alerts for:
- Query p95 latency > 500ms for 5 minutes
- Replication lag > 30 seconds
- WAL size > 1GB
- Disk free < 20%
- Authentication failure rate > 1%

### Task 2: Optimize Slow Queries

**Objective:** Identify and optimize queries that exceed performance thresholds.

**Files to create/modify:**

1. `migrations/postgres/003_optimization_indexes.sql`

```sql
-- Add indexes based on actual query patterns observed in production

-- Index for common document queries
CREATE INDEX IF NOT EXISTS idx_documents_org_status 
ON documents(organization_id, status) 
WHERE status NOT IN ('archived', 'deleted');

-- Index for full-text search
CREATE INDEX IF NOT EXISTS idx_documents_fts 
ON documents USING gin(to_tsvector('dutch', coalesce(title, '') || ' ' || coalesce(content, '')));

-- Index for audit trail queries
CREATE INDEX IF NOT EXISTS idx_audit_trail_entity_timestamp 
ON audit_trail(entity_type, entity_id, created_at DESC);

-- Partial index for active Woo publications
CREATE INDEX IF NOT EXISTS idx_documents_woo_public 
ON documents(published_at) 
WHERE woo_public = true AND status = 'published';
```

2. `migrations/postgres/004_rls_optimization.sql`

```sql
-- Optimize RLS policies for better performance

-- Replace complex policies with optimized versions
-- Using SECURITY INVOKER functions for common checks

CREATE OR REPLACE FUNCTION check_organization_access(org_id uuid, user_id uuid)
RETURNS boolean AS $$
BEGIN
    -- Check if user belongs to organization
    EXISTS(SELECT 1 FROM organization_members WHERE organization_id = org_id AND user_id = user_id)
    OR
    -- Check if user is system admin
    EXISTS(SELECT 1 FROM user_roles WHERE user_id = user_id AND role = 'admin');
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Updated RLS policy using the function
DROP POLICY IF EXISTS documents_org_policy ON documents;
CREATE POLICY documents_org_policy ON documents
FOR ALL
USING (check_organization_access(organization_id, auth.uid()));
```

**Query Optimization Checklist:**

1. Review `pg_stat_statements` for slow queries (total_exec_time > 1s)
2. Add appropriate indexes based on query patterns
3. Optimize RLS policies to avoid row-by-row checks
4. Use prepared statements for frequently executed queries
5. Consider materialized views for complex analytics queries

### Task 3: Tune RLS Policy Performance

**Objective:** Ensure Row-Level Security policies do not degrade performance.

**Key RLS Policies to Review:**

1. **Organization Isolation** - Users see only their organization's data
2. **User-Level Access** - Additional filtering within organization
3. **Classification-Based** - Confidential data protection
4. **Woo Publication** - Public document filtering

**Files to create/modify:**

1. `migrations/postgres/004_rls_optimization.sql` (optimizes existing policies)

```sql
-- Ensure all RLS policies use indexes efficiently

-- Document access policy
CREATE POLICY documents_select ON documents
FOR SELECT
USING (
    organization_id IN (
        SELECT organization_id FROM user_organizations WHERE user_id = auth.uid()
    )
    AND (
        classification <= (SELECT max_classification FROM users WHERE id = auth.uid())
    )
);

-- Enable RLS on all tables
ALTER TABLE information_domains ENABLE ROW LEVEL SECURITY;
ALTER TABLE information_objects ENABLE ROW LEVEL SECURITY;
ALTER TABLE documents ENABLE ROW LEVEL SECURITY;
ALTER TABLE templates ENABLE ROW LEVEL SECURITY;
```

**Performance Target:**
- RLS policy checks should complete in <500ms at p95 percentile
- Test with realistic data volumes (100k+ rows per organization)

### Task 4: Stabilize ETL Pipeline

**Objective:** Ensure consistent, reliable data transfer from Supabase to DuckDB.

**Files to create/modify:**

1. `crates/iou-api/src/etl/outbox.rs` (expanded with transactional outbox pattern)

```rust
//! ETL pipeline for transferring data from Supabase to DuckDB.
//!
//! This module handles incremental updates and full refreshes.

use sqlx::PgPool;
use duckdb::Connection;

pub struct EtlPipeline {
    source: PgPool,
    destination: DuckDBConnection,
    config: EtlConfig,
}

pub struct EtlConfig {
    pub batch_size: usize,           // Default: 1000 rows
    pub interval_seconds: u64,       // Default: 300 (5 minutes)
    pub use_cdc: bool,               // Use logical decoding if true
    pub outbox_table: String,        // Transactional outbox table name
}

impl EtlPipeline {
    /// Run a single ETL cycle
    pub async fn run_cycle(&self) -> Result<EtlResult, EtlError> {
        // 1. Determine watermark (last successful sync)
        // 2. Fetch changed records from Supabase
        // 3. Apply changes to DuckDB
        // 4. Update watermark
        // 5. Return statistics
    }

    /// Start the ETL scheduler
    pub async fn start_scheduler(&self) -> Result<(), EtlError> {
        // Run ETL cycles on configured interval
        // Handle idempotency for retries
    }
}
```

2. `migrations/postgres/005_outbox_table.sql` (created outbox table)

```rust
//! Transactional outbox pattern implementation.
//!
//! Ensures reliable data transfer between Supabase and DuckDB.

use sqlx::PgPool;
use serde_json::Value;

pub struct OutboxProcessor {
    pool: PgPool,
    batch_size: usize,
}

#[derive(Debug, Clone)]
pub struct OutboxEvent {
    pub id: Uuid,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub payload: Value,
    pub created_at: DateTime<Utc>,
    pub processed: bool,
}

impl OutboxProcessor {
    /// Process pending outbox events
    pub async fn process_pending(&self) -> Result<usize, Error> {
        // 1. Fetch unprocessed events ordered by created_at
        // 2. For each event:
        //    - Write to DuckDB
        //    - Mark as processed
        // 3. Return count of processed events
    }

    /// Write event to outbox (called during transaction)
    pub async fn publish_event(
        &self,
        aggregate_type: &str,
        aggregate_id: Uuid,
        event_type: &str,
        payload: Value,
    ) -> Result<Uuid, Error> {
        // Insert into change_outbox table
    }
}
```

3. `docs/operations/stabilization_runbook.md` (operational procedures)

```sql
-- Create transactional outbox table for CDC
CREATE TABLE IF NOT EXISTS change_outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type TEXT NOT NULL,
    aggregate_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    processed_at TIMESTAMPTZ,
    INDEX (processed, created_at)
);

-- Comment for documentation
COMMENT ON TABLE change_outbox IS 'Transactional outbox for ETL to DuckDB';
```

**ETL Monitoring:**

Track these metrics for the ETL pipeline:
- Cycle duration (time to complete one ETL run)
- Records processed per cycle
- Error rate (failed records / total records)
- Lag time (time between source change and ETL processing)

### Task 5: Address Production Issues

**Objective:** Respond to and resolve any issues that arise in production.

**Common Issues and Solutions:**

1. **Replication Slot Overflow**
   - Symptom: WAL accumulation, disk space pressure
   - Solution: Monitor lag, increase consumer capacity, or switch to batch ETL

2. **RLS Policy Regression**
   - Symptom: Sudden query performance degradation
   - Solution: Review recent policy changes, use `EXPLAIN ANALYZE` to identify bottlenecks

3. **Real-time Connection Spikes**
   - Symptom: Abnormal subscription count
   - Solution: Implement connection limits, investigate client behavior

4. **ETL Failures**
   - Symptom: DuckDB data stale or missing
   - Solution: Check error logs, verify outbox processing, re-run failed cycles

**Runbook Location:**

Document operational procedures in:
`/Users/marc/Projecten/iou-modern/docs/operations/stabilization_runbook.md`

## Rollback Considerations

During stabilization, rollback is still possible but increasingly complex:

**Rollback Triggers (from claude-plan.md):**
- Data inconsistency detected between databases
- Real-time latency exceeds 500ms for >5 minutes
- RLS policy performance degrades (p95 > 1s)
- Authentication failures > 1% of requests

**Rollback Process:**

If rollback is necessary during stabilization:
1. Stop ETL pipeline
2. Switch read toggle back to DuckDB
3. Reconcile data written to Supabase only
4. Investigate root cause
5. Re-migrate after fix validation

**Note:** Document any rollback triggers and decisions for post-mortem analysis.

## Dependencies

This section depends on:
- **Section 1 (Assessment)** - Performance baseline values, current state documentation
- **Section 2 (Foundation)** - Supabase deployment, schema creation
- **Section 3 (Auth & Real-time)** - RLS policies, real-time subscriptions
- **Section 4 (Cutover)** - ETL pipeline, read migration complete

This section blocks:
- **Section 6 (Cleanup)** - Cannot proceed with cleanup until stabilization is complete

## Acceptance Criteria

The stabilization phase is complete when:

1. **Performance**: All query latencies meet or exceed Phase 0 baseline
2. **ETL**: Pipeline runs consistently with <5% error rate
3. **Monitoring**: All metrics collected and alerts configured
4. **RLS**: Policy performance <500ms at p95 percentile
5. **User Feedback**: No significant negative feedback from users
6. **Stability**: System has run for 2 weeks without major incidents

## Next Steps

After stabilization is complete, proceed to **Section 6 (Cleanup)** to:
- Remove custom WebSocket code
- Remove DuckDB transactional queries
- Optimize ETL pipeline
- Update documentation
- Complete team training