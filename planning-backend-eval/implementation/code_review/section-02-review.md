# Code Review: Section-02-Foundation

## Review Scope

Reviewing code changes for section-02-foundation implementation, which implements:
1. Docker Compose configuration for self-hosted Supabase
2. PostgreSQL schema migration (migrations/postgres/)
3. Supabase connection module (crates/iou-api/src/supabase.rs)
4. Dual-write pattern (crates/iou-api/src/dual_write.rs)
5. Information domain dual-write implementation (crates/iou-api/src/domain_dual_write.rs)
6. Shared search types (crates/iou-api/src/search_types.rs)
7. Tests for schema equivalence and dual-write

---

## Critical Issues (Confidence: 90-100)

### 1. Hardcoded Database Credentials in Docker Compose
**Confidence: 95**

**File:** `/Users/marc/Projecten/iou-modern/docker-compose.supabase.yml`
**Lines:** 480, 1095, 1125

**Issue:** Default credentials are hardcoded in the docker-compose file:
```yaml
POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-postgres}
```

While environment variable override is supported, the default value `postgres` is insecure. Additionally, the JWT secret has a similar issue:
```yaml
GOTRUE_JWT_SECRET: ${JWT_SECRET:-super-secret-jwt-token-change-in-production}
```

**Recommendation:**
1. Remove default values for sensitive credentials - require explicit environment variables
2. Add validation to ensure these are set before starting services
3. Document the security requirements in the README

```yaml
environment:
  POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:?POSTGRES_PASSWORD must be set}
  GOTRUE_JWT_SECRET: ${JWT_SECRET:?JWT_SECRET must be set}
```

---

### 2. Default Connection Pool Size May Be Too Low
**Confidence: 85**

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/supabase.rs`
**Lines:** 1045

**Issue:** The connection pool is hardcoded to a maximum of 10 connections:
```rust
let pool = PgPoolOptions::new()
    .max_connections(10)
```

**Recommendation:**
Make this configurable via environment variable with a sensible default based on the expected load:
```rust
let max_connections = std::env::var("SUPABASE_MAX_CONNECTIONS")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or_else(|| {
        // Use num_cpus * 2 as a reasonable default
        (num_cpus::get() * 2).max(5).min(50) as u32
    });
```

---

### 3. SQL Injection Vulnerability via Dynamic Domain Type Conversion
**Confidence: 88**

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/domain_dual_write.rs`
**Lines:** 265-272

**Issue:** The helper functions convert enums to lowercase strings for database storage:
```rust
fn domain_type_to_string(dt: &DomainType) -> String {
    dt.to_string().to_lowercase()
}
```

While the current implementation uses parameterized queries (which is good), the CHECK constraint in the PostgreSQL schema uses string literals:
```sql
domain_type VARCHAR NOT NULL CHECK (domain_type IN ('Zaak', 'Project', 'Beleid', 'Expertise'))
```

There's a mismatch: the Rust code converts to lowercase (`zaak`) but the CHECK constraint expects titlecase (`Zaak`).

**Recommendation:**
Either:
1. Update the CHECK constraint to use lowercase values, or
2. Remove the `.to_lowercase()` call in the helper function

The safer option is (2) to maintain consistency with the enum definition.

---

### 4. Missing Transaction Support in Dual-Write Pattern
**Confidence: 92**

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/dual_write.rs`
**Lines:** 489-518

**Issue:** The dual-write pattern does not use transactions across databases:
```rust
async fn dual_write(
    &self,
    duckdb: &Database,
    supabase: &SupabasePool,
) -> DualWriteResult<Self::Id> {
    let (duckdb_result, supabase_result) = tokio::join!(
        self.write_to_duckdb(duckdb),
        self.write_to_supabase(supabase)
    );
```

If one write succeeds and the other fails, there's no automatic rollback mechanism. The code does track this in `DualWriteResult::PartialSuccess`, but there's no reconciliation logic.

**Recommendation:**
1. Implement a reconciliation queue/table to track partial failures
2. Add a periodic reconciliation job to fix inconsistencies
3. Consider implementing a "write-ahead log" pattern for crash recovery

---

### 5. Unhandled ID Mismatch Warning
**Confidence: 80**

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/dual_write.rs`
**Lines:** 497-504

**Issue:** When IDs mismatch between databases, only a warning is logged:
```rust
if d_id != s_id {
    tracing::warn!(
        "ID mismatch in dual-write: DuckDB={}, Supabase={}",
        d_id, s_id
    );
}
```

This is a data integrity issue that should be surfaced more strongly.

**Recommendation:**
Return an error in this case rather than just warning:
```rust
if d_id != s_id {
    return DualWriteResult::Failed(vec![
        anyhow::anyhow!("ID mismatch: DuckDB={}, Supabase={}", d_id, s_id)
    ]);
}
```

---

## Important Issues (Confidence: 80-89)

### 6. Missing Error Context in Supabase Module
**Confidence: 82**

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/supabase.rs`
**Lines:** 1043-1051

**Issue:** Connection errors lack helpful context:
```rust
pub async fn new(database_url: &str) -> Result<Self> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    tracing::info!("Connected to Supabase PostgreSQL");
```

**Recommendation:**
Add context to the error to help debugging:
```rust
pub async fn new(database_url: &str) -> Result<Self> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to connect to Supabase at {}: {}",
                database_url.split('@').last().unwrap_or(database_url),
                e
            )
        })?;

    tracing::info!("Connected to Supabase PostgreSQL");
```

---

### 7. Test Files Have Hardcoded Database Paths
**Confidence: 85**

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/tests/dual_write.rs`
**Line:** 1116

**Issue:** Test creates database in `/tmp/` which may fail on some systems:
```rust
async fn get_test_duckdb() -> Database {
    let db = Database::new("/tmp/test_iou_dual_write.db")
```

**Recommendation:**
Use a temporary directory that's more portable:
```rust
async fn get_test_duckdb() -> Database {
    let temp_dir = std::env::var("CARGO_TARGET_TMPDIR")
        .unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().to_string());
    let db_path = std::path::PathBuf::from(temp_dir)
        .join(format!("test_iou_dual_write_{}.db", uuid::Uuid::new_v4()));
```

---

### 8. Circular Dependency Resolution Could Be Cleaner
**Confidence: 78**

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/search_types.rs`

**Issue:** The `search_types.rs` module was created to resolve circular dependencies between `db.rs` and `routes/search.rs`. While this works, it indicates a potential architectural issue.

**Recommendation:**
Consider restructuring the modules:
1. Create a `types` module at the crate level for shared types
2. Or use a more explicit module hierarchy (e.g., `types::search`)

---

### 9. Migration Function is a No-Op Placeholder
**Confidence: 80**

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/supabase.rs`
**Lines:** 1079-1085

**Issue:** The `run_migrations` function doesn't actually do anything:
```rust
pub async fn run_migrations(&self) -> Result<()> {
    // Note: In production, use sqlx-cli or a proper migration tool
    // This is a simplified version for development
    tracing::info!("Migrations should be run via sqlx-cli or external tool");
    Ok(())
}
```

**Recommendation:**
Either:
1. Remove this function if it's not going to be implemented
2. Or actually implement it using sqlx's migrate functionality:
```rust
pub async fn run_migrations(&self) -> Result<()> {
    sqlx::migrate!("./migrations/postgres")
        .run(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;
    Ok(())
}
```

---

### 10. Missing Health Check Retry Logic
**Confidence: 82**

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/supabase.rs`
**Lines:** 1071-1077

**Issue:** The health check doesn't retry on transient failures:
```rust
pub async fn health_check(&self) -> Result<()> {
    sqlx::query("SELECT 1")
        .fetch_one(&self.pool)
        .await?;
    Ok(())
}
```

**Recommendation:**
Add retry logic with exponential backoff for transient failures:
```rust
pub async fn health_check(&self) -> Result<()> {
    use tokio::time::{sleep, Duration};

    let mut attempts = 0;
    let max_attempts = 3;

    loop {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => return Ok(()),
            Err(e) if attempts < max_attempts => {
                attempts += 1;
                sleep(Duration::from_millis(100 * 2_u64.pow(attempts))).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
}
```

---

## Code Quality Issues

### 11. Inconsistent Module Visibility
**Confidence: 75**

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/lib.rs`

**Issue:** Some modules are public in `lib.rs` but not all need to be:
```rust
pub mod db;
pub mod domain_dual_write;
pub mod dual_write;
pub mod error;
pub mod search_types;
pub mod supabase;
pub mod websockets;
```

**Recommendation:**
Only export what users of the library need. The `domain_dual_write` module might not need to be public if it's only used internally.

---

### 12. Environment Variable Parsing Could Be More Robust
**Confidence: 78**

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/dual_write.rs`
**Lines:** 417-421

**Issue:** The `from_env` function uses `as_deref()` which is good, but the empty string case isn't handled correctly:
```rust
pub fn from_env() -> Self {
    match std::env::var("READ_SOURCE").as_deref() {
        Ok("supabase") => ReadSource::Supabase,
        _ => ReadSource::DuckDb,  // Default to DuckDB for safety
    }
}
```

The test sets `READ_SOURCE` to an empty string which would match the default case. This is actually fine, but the comment in the test is misleading.

**Recommendation:**
Make the test clearer:
```rust
#[test]
fn test_read_source_from_env_default() {
    // Remove the env var to test default behavior
    std::env::remove_var("READ_SOURCE");
    let source = ReadSource::from_env();
    assert_eq!(source, ReadSource::DuckDb);
}
```

---

### 13. PostgreSQL Migration Using Deprecated UUID Function
**Confidence: 85**

**File:** `/Users/marc/Projecten/iou-modern/migrations/postgres/001_create_initial_schema.sql`
**Line:** 1571

**Issue:** The migration uses `gen_random_uuid()` which requires pgcrypto, but also enables `uuid-ossp`:
```sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
```

Then uses:
```sql
id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
```

**Recommendation:**
For PostgreSQL 15+, `gen_random_uuid()` is built-in and doesn't require extensions. Either:
1. Remove the `uuid-ossp` extension if it's not needed elsewhere
2. Or use `uuid_generate_v4()` from `uuid-ossp` for broader compatibility

---

### 14. Missing Index on Updated_at Columns
**Confidence: 80**

**File:** `/Users/marc/Projecten/iou-modern/migrations/postgres/001_create_initial_schema.sql`

**Issue:** Several tables have `updated_at` columns but no indexes on them. If the system needs to query for recently updated records, this will be slow.

**Recommendation:**
Consider adding indexes for common query patterns:
```sql
CREATE INDEX IF NOT EXISTS idx_information_objects_updated_at
    ON information_objects(updated_at DESC);
```

---

### 15. Test Functions Not Following Naming Convention
**Confidence: 75**

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/tests/dual_write.rs`
**Lines:** 1115, 1123

**Issue:** The helper functions use `async fn` but don't need to be async:
```rust
async fn get_test_duckdb() -> Database {
async fn get_test_supabase() -> SupabasePool {
```

Neither function actually uses `.await` inside them before the initialization code.

**Recommendation:**
Remove `async` from these functions or document why they're async (e.g., for future use with async setup).

---

## Positive Observations

1. **Good use of parameterized queries** - All SQL queries use proper parameterization to prevent SQL injection.

2. **Comprehensive test coverage** - The test files cover schema equivalence, dual-write consistency, and update operations.

3. **Well-structured Docker Compose** - The Supabase deployment includes all necessary services with proper health checks and dependencies.

4. **Thoughtful dual-write pattern** - The pattern allows for gradual migration with configurable read source.

5. **Good use of Rust traits** - The `DualWrite` trait is well-designed for extensibility to other domain types.

6. **Proper use of async/await** - The code correctly uses async patterns throughout.

---

## Summary

The section-02-foundation implementation is generally well-structured but has several important issues that should be addressed:

**Critical Priority:**
1. Fix the domain_type case mismatch between Rust and PostgreSQL
2. Implement proper transaction/reconciliation for dual-write failures
3. Remove hardcoded default credentials

**High Priority:**
4. Make connection pool size configurable
5. Implement actual migration functionality
6. Add retry logic to health checks

**Medium Priority:**
7. Fix test database paths for portability
8. Refactor module structure to avoid circular dependencies
9. Add missing indexes for query performance

The code demonstrates good Rust practices overall, with proper use of traits, async/await, and parameterized queries. The dual-write pattern is well-conceived for gradual migration but needs stronger consistency guarantees.

---

## Files Reviewed

- `/Users/marc/Projecten/iou-modern/docker-compose.supabase.yml`
- `/Users/marc/Projecten/iou-modern/migrations/postgres/001_create_initial_schema.sql`
- `/Users/marc/Projecten/iou-modern/migrations/postgres/README.md`
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/supabase.rs`
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/dual_write.rs`
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/domain_dual_write.rs`
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/search_types.rs`
- `/Users/marc/Projecten/iou-modern/crates/iou-api/tests/dual_write.rs`
- `/Users/marc/Projecten/iou-modern/crates/iou-api/tests/schema_equivalence.rs`
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/db.rs` (partial - search type changes)
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/search.rs` (partial - search type changes)
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/lib.rs`
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/main.rs`
- `/Users/marc/Projecten/iou-modern/.env.supabase.example`
