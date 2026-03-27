# Section 05: Stabilization - Code Review

## Critical Issues (Must Fix)

### 1. Migration 003: Indexes on Non-Existent Columns
**File:** `migrations/postgres/003_optimization_indexes.sql`
**Lines:** 9-11, 20-21, 39-41

The migration attempts to create indexes on columns that don't exist in the schema:

- `idx_documents_org_status` references `documents.organization_id` and `documents.status` - but the documents table only has `domain_id` and `state`
- `idx_audit_trail_entity_timestamp` references `audit_trail.entity_type`, `audit_trail.entity_id`, and `audit_trail.created_at` - but the audit_trail table uses `document_id` and `timestamp`

This will cause the migration to fail with "column does not exist" errors.

**Recommendation:** Either fix the column names to match the actual schema, or add a migration to add these columns first.

### 2. Migration 004: Typo in Function Declaration
**File:** `migrations/postgres/004_rls_optimization.sql`
**Line:** 28, 38

The function declarations use `PARALLEL SAFE` but the correct PostgreSQL syntax is `PARALLEL_SAFE` (with underscore). While PostgreSQL may accept this as two separate keywords, it's inconsistent with standard practice.

### 3. Outbox Processor Doesn't Actually Write to DuckDB
**File:** `crates/iou-api/src/etl/outbox.rs`
**Lines:** 87-104

The `process_single_event` function only marks events as processed in PostgreSQL but doesn't actually write them to DuckDB. This defeats the entire purpose of the outbox pattern for CDC/replication.

```rust
async fn process_single_event(&self, event: &sqlx::types::PgRow) -> Result<(), anyhow::Error> {
    let id: Uuid = event.get("id");
    // Mark as processed
    sqlx::query!(...)
    .execute(&self.pool)
    .await?;
    Ok(())
}
```

The function should:
1. Extract event payload
2. Write to DuckDB
3. Then mark as processed

### 4. get_user_organizations Returns Empty Result
**File:** `migrations/postgres/004_rls_optimization.sql`
**Lines:** 32-38

The function returns an empty query result (`RETURN QUERY;`), which means the RLS policy will never match any organization. This effectively breaks all RLS-protected queries.

```sql
CREATE OR REPLACE FUNCTION get_user_organizations(user_id UUID)
RETURNS TABLE(organization_id UUID) AS $$
BEGIN
    RETURN QUERY;  -- Always returns empty!
END;
```

## Important Issues (Should Fix)

### 5. MetricsCollector Returns Placeholder Values
**File:** `crates/iou-api/src/monitoring/collector.rs`
**Lines:** 106-114

Critical monitoring metrics are hardcoded to 0 or None:

```rust
Ok(DbStats {
    replication_lag_secs: None,
    wal_size_bytes: 0,
    disk_usage_bytes: 0,
    disk_free_bytes: 0,
})
```

This makes the monitoring system useless for detecting actual issues.

### 6. All Test Files Are Placeholders
**Files:** `crates/iou-api/tests/stabilization_*.rs`

All three test files contain hardcoded placeholder values:

```rust
let baseline = PerformanceBaseline {
    query_p95_ms: 150.0,
    conn_pool_utilization: 0.6,
    // ... all hardcoded
};
```

Tests should actually exercise the code and verify behavior.

### 7. Percentile Calculations Are Estimates
**File:** `crates/iou-api/src/monitoring/collector.rs`
**Lines:** 87-89

```rust
p50_latency_ms: mean * 0.8,  // Estimate
p95_latency_ms: mean * 1.5,  // Estimate
p99_latency_ms: mean * 2.0,  // Estimate
```

Percentiles calculated from mean are statistically incorrect. Use actual percentiles from pg_stat_statements or a proper histogram.

### 8. user_has_clearance Hardcodes Clearance Level
**File:** `migrations/postgres/004_rls_optimization.sql`
**Lines:** 55-72

```sql
user_clearance := 'intern';  -- Always 'intern'
```

This function always returns the same clearance level regardless of the user, making the authorization check meaningless.

### 9. Redundant Subquery in check_organization_access
**File:** `migrations/postgres/004_rls_optimization.sql`
**Lines:** 13-20

```sql
AND organization_id = (
    SELECT organization_id FROM information_domains
    WHERE id = org_id
    LIMIT 1
)
```

This subquery is redundant since we already know `id = org_id`. It should be simplified.

## Nice to Have Improvements

### 10. AlertEngine Missing auth_failure_rate Evaluation
**File:** `crates/iou-api/src/monitoring/alerting.rs`

The AlertThresholds struct includes `auth_failure_rate` but there's no corresponding evaluation method.

### 11. Unused Import in Test
**File:** `crates/iou-api/tests/stabilization_performance.rs`
**Line:** 3

`use std::time::Duration;` is imported but never used.

## Summary

The section-05 stabilization implementation has several critical issues that will prevent it from working correctly:

1. **Migration failures** due to non-existent columns
2. **Non-functional ETL** (outbox doesn't write to DuckDB)
3. **Broken RLS** (get_user_organizations returns empty)
4. **Useless monitoring** (placeholder values)
5. **No real tests** (all placeholders)

The stabilization layer appears to be a stub implementation that was never completed. The migrations, ETL pipeline, RLS functions, and monitoring system all have critical gaps that need to be addressed before this can be considered production-ready.
