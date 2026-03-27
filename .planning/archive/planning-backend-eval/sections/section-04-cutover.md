Now I have all the context needed. Let me generate the section-04-cutover content based on Phase 3 of the migration plan.

---

# Section 04: Cutover to Supabase

**Weeks 7-9 | Phase 3**

## Overview

This section covers the critical cutover phase where primary API traffic migrates from DuckDB to Supabase. This is the most risky phase of the migration as it involves:

1. Switching read traffic from DuckDB to Supabase
2. Implementing ETL pipeline for analytics (Supabase to DuckDB)
3. Removing the dual-write pattern
4. Configuring DuckDB for analytics-only workload

**Dependencies:** This section requires completion of `section-03-auth-realtime` (authentication and real-time features must be operational).

**Key Deliverables:**
- All API endpoints reading from Supabase
- ETL pipeline operational (Supabase to DuckDB)
- DuckDB analytics-only mode configured
- Dual-write pattern removed

---

## Tests

Write the following tests before implementing the cutover. These tests are critical for verifying a safe migration.

### Read Migration Tests

**File:** `/Users/marc/Projecten/iou-modern/backend/tests/migration/read_migration.rs`

```rust
//! Tests for migrating reads from DuckDB to Supabase
//! 
//! These tests verify that all API endpoints work correctly when reading
//! from Supabase instead of DuckDB.

#[cfg(test)]
mod read_migration_tests {
    use super::*;

    /// Test that the information_domains API endpoint reads from Supabase
    #[tokio::test]
    async fn test_information_domains_reads_from_supabase() {
        // TODO: Verify that queries go to Supabase when read toggle is set
        // 1. Set read toggle to Supabase
        // 2. Insert test data into Supabase only
        // 3. Call the API endpoint
        // 4. Assert that Supabase data is returned
    }

    /// Test that the information_objects API endpoint reads from Supabase
    #[tokio::test]
    async fn test_information_objects_reads_from_supabase() {
        // TODO: Verify that queries go to Supabase when read toggle is set
    }

    /// Test that the documents API endpoint reads from Supabase
    #[tokio::test]
    async fn test_documents_reads_from_supabase() {
        // TODO: Verify that queries go to Supabase when read toggle is set
    }

    /// Test that the templates API endpoint reads from Supabase
    #[tokio::test]
    async fn test_templates_reads_from_supabase() {
        // TODO: Verify that queries go to Supabase when read toggle is set
    }

    /// Test that the audit_trail API endpoint reads from Supabase
    #[tokio::test]
    async fn test_audit_trail_reads_from_supabase() {
        // TODO: Verify that queries go to Supabase when read toggle is set
    }

    /// Test that view queries (v_searchable_objects) work on Supabase
    #[tokio::test]
    async fn test_searchable_objects_view_on_supabase() {
        // TODO: Verify that the view produces the same results as DuckDB
        // 1. Query v_searchable_objects on Supabase
        // 2. Compare results to DuckDB version
        // 3. Assert result equivalence
    }

    /// Test full-text search works with PostgreSQL tsvector
    #[tokio::test]
    async fn test_full_text_search_on_supabase() {
        // TODO: Verify that Dutch full-text search works
        // 1. Search for Dutch terms
        // 2. Verify relevance ranking
        // 3. Compare results to ILIKE baseline
    }

    /// Final data consistency check before cutover
    #[tokio::test]
    async fn test_final_data_consistency_check() {
        // TODO: Verify no data loss between databases
        // 1. Count records in both databases
        // 2. Compare sample records for equality
        // 3. Verify all foreign keys valid
        // 4. Assert zero data loss
    }

    /// Test that performance meets baseline after cutover
    #[tokio::test]
    async fn test_performance_meets_baseline() {
        // TODO: Compare query performance to baseline
        // 1. Run standard query suite
        // 2. Measure p50/p95/p99 latencies
        // 3. Compare to Phase 0 baseline
        // 4. Assert no significant degradation
    }
}
```

### ETL Pipeline Tests

**File:** `/Users/marc/Projecten/iou-modern/backend/tests/migration/etl_pipeline.rs`

```rust
//! Tests for the ETL pipeline from Supabase to DuckDB
//! 
//! The ETL pipeline keeps DuckDB synchronized for analytics workloads.

#[cfg(test)]
mod etl_tests {
    use super::*;

    /// Test that ETL transfers data from Supabase to DuckDB
    #[tokio::test]
    async fn test_etl_supabase_to_duckdb() {
        // TODO: Verify complete data transfer
        // 1. Insert test data into Supabase
        // 2. Run ETL job
        // 3. Query DuckDB for transferred data
        // 4. Assert data presence and correctness
    }

    /// Test that ETL is idempotent (can run multiple times safely)
    #[tokio::test]
    async fn test_etl_idempotency() {
        // TODO: Verify no duplicate data on re-run
        // 1. Insert test data into Supabase
        // 2. Run ETL job twice
        // 3. Query DuckDB for record counts
        // 4. Assert no duplicates (upsert logic works)
    }

    /// Test ETL error handling and retry logic
    #[tokio::test]
    async fn test_etl_error_handling() {
        // TODO: Verify graceful failure handling
        // 1. Simulate connection failure mid-transfer
        // 2. Verify error is logged
        // 3. Verify retry mechanism works
        // 4. Verify partial transfers are rolled back
    }

    /// Test ETL latency meets requirements
    #[tokio::test]
    async fn test_etl_latency() {
        // TODO: Verify ETL completes within window
        // 1. Insert large dataset into Supabase
        // 2. Measure ETL execution time
        // 3. Assert completes within configured window (e.g., 5 min)
    }

    /// Test that DuckDB analytics queries work with ETL data
    #[tokio::test]
    async fn test_duckdb_analytics_queries_with_etl() {
        // TODO: Verify analytics queries work
        // 1. Run ETL job
        // 2. Execute v_compliance_overview query
        // 3. Execute v_domain_statistics query
        // 4. Execute v_entity_network query
        // 5. Assert results are valid
    }

    /// Test transactional outbox pattern for event ordering
    #[tokio::test]
    async fn test_transactional_outbox_ordering() {
        // TODO: Verify events processed in order
        // 1. Insert multiple records with timestamps
        // 2. Run ETL
        // 3. Verify DuckDB records maintain order
        // 4. Assert no out-of-order processing
    }

    /// Test ETL handles updates correctly
    #[tokio::test]
    async fn test_etl_handles_updates() {
        // TODO: Verify updates are propagated
        // 1. Insert record via ETL
        // 2. Update record in Supabase
        // 3. Run ETL again
        // 4. Assert DuckDB has updated values
    }

    /// Test ETL handles deletions correctly
    #[tokio::test]
    async fn test_etl_handles_deletions() {
        // TODO: Verify deletions are propagated
        // 1. Insert record via ETL
        // 2. Soft-delete record in Supabase
        // 3. Run ETL again
        // 4. Assert DuckDB reflects deletion
    }
}
```

### Cleanup Tests

**File:** `/Users/marc/Projecten/iou-modern/backend/tests/migration/cleanup.rs`

```rust
//! Tests for removing dual-write pattern and legacy code

#[cfg(test)]
mod cleanup_tests {
    use super::*;

    /// Test that single-write to Supabase works after dual-write removal
    #[tokio::test]
    async fn test_single_write_to_supabase() {
        // TODO: Verify writes go to Supabase only
        // 1. Disable dual-write mode
        // 2. Perform insert/update/delete operations
        // 3. Verify Supabase has the data
        // 4. Verify DuckDB is NOT updated (only via ETL)
    }

    /// Test that DuckDB transactional queries are removed
    #[tokio::test]
    async fn test_duckdb_transactional_queries_removed() {
        // TODO: Verify only analytics queries access DuckDB
        // 1. Scan code for direct DuckDB write operations
        // 2. Assert no transactional write paths to DuckDB exist
        // 3. Verify all DuckDB access is read-only for analytics
    }

    /// Test that analytics still work after cutover
    #[tokio::test]
    async fn test_analytics_work_after_cutover() {
        // TODO: Verify analytics functionality intact
        // 1. Run compliance report
        // 2. Run domain statistics
        // 3. Run entity network analysis
        // 4. Assert all analytics work correctly
    }
}
```

---

## Implementation Details

### 1. API Read Migration

**Objective:** Switch all API endpoints from reading DuckDB to reading Supabase.

**File:** `/Users/marc/Projecten/iou-modern/backend/src/database/mod.rs`

The read toggle mechanism implemented in `section-02-foundation` provides a safe way to switch between databases. During this phase, we permanently switch to Supabase.

```rust
// Example read toggle configuration (already implemented in section-02)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadSource {
    DuckDB,
    Supabase,
}

// Configuration for read source
pub struct DatabaseConfig {
    pub read_source: ReadSource,
    pub write_mode: WriteMode, // DualWrite or SingleWrite
}
```

**Implementation Steps:**

1. **Verify read toggle functionality:**
   - Confirm the toggle from `section-02-foundation` is operational
   - Test switching between DuckDB and Supabase reads
   - Ensure no hard-coded DuckDB references remain

2. **Update repository layer:**
   - Review all repository implementations in `/Users/marc/Projecten/iou-modern/backend/src/repositories/`
   - Ensure all read methods respect the read toggle
   - Add logging to track which database is being queried

3. **Switch read source permanently:**
   - Update configuration to set `read_source = ReadSource::Supabase`
   - Deploy to staging first for verification
   - Monitor metrics for any performance issues

4. **Remove DuckDB read paths (deferred to section-06):**
   - Keep DuckDB connection for analytics only
   - Mark transactional read paths for removal in cleanup phase

### 2. ETL Pipeline Implementation

**Objective:** Implement automated data transfer from Supabase to DuckDB for analytics.

**File:** `/Users/marc/Projecten/iou-modern/backend/src/etl/mod.rs`

The ETL pipeline is the critical link that keeps DuckDB synchronized for analytics workloads.

**ETL Architecture:**

```
Supabase (Primary)          DuckDB (Analytics)
    │                            ▲
    │                            │
    └────────── ETL ──────────────┘
                 (every 5 min)
```

**Key Requirements:**

1. **Idempotent design:** Running ETL multiple times must not create duplicates
2. **Incremental updates:** Only transfer changed records since last run
3. **Error handling:** Failed transfers must be logged and retryable
4. **Ordering preservation:** Maintain transaction order for analytics

**Implementation Stub:**

```rust
// File: /Users/marc/Projecten/iou-modern/backend/src/etl/mod.rs

//! ETL pipeline for syncing Supabase to DuckDB
//!
//! This module handles the transfer of data from the primary Supabase database
//! to the analytics DuckDB database.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use duckdb::Connection;

/// Configuration for the ETL pipeline
#[derive(Debug, Clone)]
pub struct EtlConfig {
    /// How often to run the ETL (in seconds)
    pub interval_seconds: u64,
    /// Maximum number of records to transfer per batch
    pub batch_size: usize,
    /// Whether to use incremental updates (false = full refresh)
    pub incremental: bool,
}

/// The main ETL coordinator
pub struct EtlPipeline {
    supabase_pool: PgPool,
    duckdb_conn: Connection,
    config: EtlConfig,
}

impl EtlPipeline {
    /// Create a new ETL pipeline
    pub fn new(
        supabase_pool: PgPool,
        duckdb_conn: Connection,
        config: EtlConfig,
    ) -> Result<Self> {
        // TODO: Initialize ETL pipeline
        todo!()
    }

    /// Run a single ETL cycle
    pub async fn run_cycle(&self) -> Result<EtlMetrics> {
        // TODO: Execute ETL for all tables
        // 1. Get last run timestamp
        // 2. Fetch changes from Supabase
        // 3. Apply changes to DuckDB (upsert)
        // 4. Update last run timestamp
        // 5. Return metrics (records transferred, time taken)
        todo!()
    }

    /// Start the ETL scheduler
    pub async fn start(&self) -> Result<()> {
        // TODO: Run ETL on a timer
        // Use tokio::time::interval for scheduling
        todo!()
    }

    /// Transfer information_domains table
    async fn sync_information_domains(&self, since: Option<DateTime<Utc>>) -> Result<usize> {
        // TODO: Implement table-specific sync logic
        // Use UPSERT to handle idempotency
        todo!()
    }

    /// Transfer information_objects table
    async fn sync_information_objects(&self, since: Option<DateTime<Utc>>) -> Result<usize> {
        // TODO: Implement table-specific sync logic
        todo!()
    }

    /// Transfer documents table
    async fn sync_documents(&self, since: Option<DateTime<Utc>>) -> Result<usize> {
        // TODO: Implement table-specific sync logic
        // Handle soft-deletes correctly
        todo!()
    }

    /// Transfer templates table
    async fn sync_templates(&self, since: Option<DateTime<Utc>>) -> Result<usize> {
        // TODO: Implement table-specific sync logic
        todo!()
    }

    /// Transfer audit_trail table
    async fn sync_audit_trail(&self, since: Option<DateTime<Utc>>) -> Result<usize> {
        // TODO: Implement table-specific sync logic
        // Ensure audit trail continuity
        todo!()
    }

    /// Rebuild DuckDB views after sync
    async fn rebuild_views(&self) -> Result<()> {
        // TODO: Refresh materialized views
        // - v_searchable_objects
        // - v_compliance_overview
        // - v_domain_statistics
        // - v_entity_network
        todo!()
    }
}

/// Metrics from a single ETL run
#[derive(Debug, Clone)]
pub struct EtlMetrics {
    pub records_transferred: usize,
    pub duration_ms: u64,
    pub tables_synced: Vec<String>,
    pub errors: Vec<String>,
}
```

**Incremental Sync Strategy:**

For incremental updates, use one of these approaches:

1. **Timestamp-based tracking:** Add `updated_at` columns to all tables and query `WHERE updated_at > ?`

2. **CDC (Change Data Capture):** Use PostgreSQL logical replication for real-time sync (more complex, lower latency)

3. **Transactional outbox pattern:** Write events to an outbox table and process in order

**Recommended:** Start with timestamp-based tracking for simplicity. Evaluate CDC in Phase 4 if latency is insufficient.

### 3. DuckDB Analytics-Only Configuration

**Objective:** Configure DuckDB exclusively for analytics workloads.

**File:** `/Users/marc/Projecten/iou-modern/backend/src/database/duckdb.rs`

DuckDB will transition from a primary database to a specialized analytics engine.

**Configuration Changes:**

```rust
// DuckDB configuration for analytics workload
pub struct DuckDbConfig {
    // Path to the database file
    pub path: String,
    
    // Memory limit for analytics queries
    pub memory_limit: Option<String>,
    
    // Number of threads for parallel queries
    pub threads: Option<usize>,
    
    // Enable experimental features for analytics
    pub enable_vector_similarity: bool,
    pub enable_full_text_search: bool,
}
```

**Analytics Queries:**

These queries will continue to use DuckDB after cutover:

- `v_compliance_overview` - Compliance metrics and reports
- `v_domain_statistics` - Domain type distribution and status
- `v_entity_network` - GraphRAG entity relationships
- Full-text search index (consider migrating to PostgreSQL)
- Vector similarity search (if implemented)

### 4. Dual-Write Removal

**Objective:** Remove the dual-write pattern once reads are fully migrated to Supabase.

**File:** `/Users/marc/Projecten/iou-modern/backend/src/database/mod.rs`

**Implementation Steps:**

1. **Verify all reads are from Supabase:**
   - Check metrics to confirm no DuckDB read traffic
   - Run final data consistency tests

2. **Switch to single-write mode:**
   ```rust
   pub enum WriteMode {
       DualWrite,  // Write to both Supabase and DuckDB
       SupabaseOnly,  // Write to Supabase only (ETL syncs to DuckDB)
   }
   ```

3. **Remove dual-write code paths:**
   - Refactor repository write methods to target Supabase only
   - Remove transaction coordination between databases
   - Simplify error handling (no partial rollback concerns)

4. **Update ETL timing:**
   - With single-write, ETL becomes the only source for DuckDB updates
   - Consider reducing ETL interval if analytics needs are urgent
   - Add monitoring for ETL failures

---

## Dependencies

This section requires completion of:

- **section-01-assessment:** Performance baselines must be established for comparison
- **section-02-foundation:** Supabase deployment and dual-write pattern must be operational
- **section-03-auth-realtime:** Authentication and RLS policies must be configured

---

## Rollback Procedures

**Trigger Criteria:**
- Data inconsistency detected between Supabase and DuckDB
- ETL failures causing stale analytics data
- Performance degradation (p95 latency > 2x baseline)
- Authentication failures > 1% of requests

**Rollback Steps:**

1. **Immediate read rollback:**
   - Switch `read_source` configuration back to `DuckDB`
   - Restart backend services to apply configuration

2. **Data reconciliation:**
   ```sql
   -- Find records in Supabase not in DuckDB
   SELECT id, table_name 
   FROM reconciliation_report 
   WHERE synced = false;
   ```

3. **Re-enable dual-write:**
   - Switch `write_mode` back to `DualWrite`
   - Verify writes are succeeding to both databases

4. **Post-rollback verification:**
   - Run data consistency tests
   - Verify performance has recovered
   - Document root cause for fix before re-attempting cutover

**Note:** After cutover, rollback becomes progressively more complex. Consider a 1-week "freeze period" after cutover before moving to section-05.

---

## Compliance Considerations

### Bijhoudingsplicht (Record-Keeping Obligation)

The cutover phase must maintain audit trail continuity:

- All migration events must be logged in the audit_trail table
- Correlate DuckDB audit logs with Supabase WAL (Write-Ahead Log)
- Ensure no audit events are lost during the transition

### Archiefwet (Archives Act)

- Verify retention policies are enforced in Supabase
- Ensure ETL preserves destruction dates and retention schedules
- Test that automated deletion works after cutover

### GDPR/AVG

- Verify right to deletion works via Supabase
- Ensure ETL respects user deletion requests
- Test data portability export from Supabase

---

## Success Criteria

The cutover phase is complete when:

1. **All API endpoints** read from Supabase
2. **ETL pipeline** runs successfully every 5 minutes (or configured interval)
3. **Analytics queries** work correctly on DuckDB with ETL data
4. **Dual-write pattern** is removed
5. **Performance** meets or exceeds baseline (from section-01)
6. **Zero data loss** verified via consistency tests
7. **Monitoring** confirms stable operation for 1 week