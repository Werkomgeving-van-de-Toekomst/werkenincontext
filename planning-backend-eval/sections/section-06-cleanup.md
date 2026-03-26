# Section 06: Cleanup - Legacy Code Removal and Migration Finalization

## Overview

**Phase 5 | Week 12**

This section covers the final cleanup phase of the database migration. At this point, the cutover to Supabase is complete, the ETL pipeline is stable, and performance meets or exceeds baselines. This phase focuses on removing legacy code, reducing technical debt, updating documentation, and ensuring team readiness.

**Dependencies:** Requires completion of section-05-stabilization (monitoring operational, performance validated).

---

## Context from Previous Phases

By the time this section begins, the following has been accomplished:

- **Supabase** is the primary database for all transactional operations
- **DuckDB** serves only analytics workloads via ETL from Supabase
- **Real-time subscriptions** are handled by Supabase Realtime
- **Authentication** is managed by Supabase Auth with RLS policies
- **ETL pipeline** runs stably (batch or CDC-based) from Supabase to DuckDB
- **Performance** meets or exceeds Phase 0 baselines

The cleanup phase ensures no legacy code remains and the team is fully trained on the new architecture.

---

## Tests

Define tests to verify successful cleanup. These tests ensure legacy code is removed and the new architecture is stable.

### Custom WebSocket Removal Tests

**File:** `/Users/marc/Projecten/iou-modern/backend/tests/cleanup/websocket_removal.rs`

```rust
// Test: Custom WebSocket removal - verify no references remain
#[test]
fn test_no_legacy_websocket_imports() {
    // Grep codebase for legacy WebSocket module imports
    // Assert: No references to old websocket module
}

// Test: Custom WebSocket removal - verify Supabase Realtime used instead
#[test]
fn test_supabase_realtime_active() {
    // Verify real-time subscriptions use Supabase client
    // Assert: Subscriptions configured for Supabase channels
}
```

### DuckDB Transactional Query Removal Tests

**File:** `/Users/marc/Projecten/iou-modern/backend/tests/cleanup/duckdb_analytics_only.rs`

```rust
// Test: DuckDB transactional queries removed - verify only analytics queries remain
#[test]
fn test_duckdb_read_only_analytics() {
    // Verify all DuckDB queries are read-only or analytics-specific
    // Assert: No INSERT/UPDATE/DELETE to DuckDB from application code
}

// Test: Verify all writes go through Supabase
#[test]
fn test_all_writes_to_supabase() {
    // Check that write operations use Supabase client
    // Assert: No direct DuckDB write paths in API handlers
}
```

### Documentation Completeness Tests

**File:** `/Users/marc/Projecten/iou-modern/backend/tests/cleanup/documentation.rs`

```rust
// Test: Documentation completeness - verify all docs updated
#[test]
fn test_architecture_documentation_updated() {
    // Verify architecture docs reference Supabase + DuckDB hybrid
    // Assert: No references to DuckDB as primary database
}

// Test: API documentation updated
#[test]
fn test_api_docs_reflect_supabase() {
    // Verify OpenAPI/specs reference Supabase schema
    // Assert: Authentication examples use Supabase patterns
}
```

### Team Training Completion Tests

**File:** `/Users/marc/Projecten/iou-modern/backend/tests/cleanup/training.rs`

```rust
// Test: Team training completion - verify knowledge transfer
// (This is a manual checklist, not an automated test)

// Checklist items:
// - [ ] All developers can navigate Supabase dashboard
// - [ ] All developers understand RLS policy structure
// - [ ] All developers can debug real-time subscription issues
// - [ ] All developers know ETL pipeline troubleshooting steps
// - [ ] Runbook for common issues documented
```

### Final Integration Tests

**File:** `/Users/marc/Projecten/iou-modern/backend/tests/cleanup/final_integration.rs`

```rust
// Test: Full system smoke test - verify all components work together
#[tokio::test]
async fn test_full_system_smoke() {
    // 1. User login via Supabase Auth
    // 2. Create document (writes to Supabase)
    // 3. Verify real-time update received
    // 4. Trigger ETL manually
    // 5. Verify document queryable in DuckDB analytics
    // 6. Verify audit trail complete
}
```

---

## Implementation

### Task 1: Remove Custom WebSocket Code

**Objective:** Eliminate legacy WebSocket implementation now that Supabase Realtime is operational.

**Files to Remove:**

| File Path | Action |
|-----------|--------|
| `/Users/marc/Projecten/iou-modern/backend/src/websocket/mod.rs` | Delete entire module |
| `/Users/marc/Projecten/iou-modern/backend/src/websocket/handler.rs` | Delete |
| `/Users/marc/Projecten/iou-modern/backend/src/websocket/broadcast.rs` | Delete |
| `/Users/marc/Projecten/iou-modern/backend/src/websocket/ state.rs` | Delete |

**Files to Modify:**

**File:** `/Users/marc/Projecten/iou-modern/backend/src/main.rs`

```rust
// REMOVE:
// mod websocket;
// use websocket::WebSocketServer;

// REMOVE from startup:
// let ws_server = WebSocketServer::new(config.clone()).await?;
// ws_server.start().await?;

// REPLACE with Supabase Realtime client initialization:
use supabase_realtime::RealtimeClient;

let realtime_client = RealtimeClient::new(&config.supabase_url, &config.supabase_key)?;
```

**File:** `/Users/marc/Projecten/iou-modern/backend/src/lib.rs`

```rust
// REMOVE:
// pub mod websocket;

// VERIFY: No re-exports of WebSocket types
```

**File:** `/Users/marc/Projecten/iou-modern/backend/Cargo.toml`

```toml
# REMOVE dependencies that were only used by WebSocket:
# tokio-tungstenite = "..."
# futures-util = "..."
# (keep if still used elsewhere)

# VERIFY supabase-realtime-rs or equivalent is present
```

**Verification Steps:**

1. Grep for remaining references: `rg -i "websocket" backend/src/`
2. Ensure all real-time functionality still works via Supabase
3. Run integration tests for collaboration features

---

### Task 2: Remove DuckDB Transactional Queries

**Objective:** Ensure DuckDB is used exclusively for analytics. All transactional operations must go through Supabase.

**Files to Audit and Modify:**

**File:** `/Users/marc/Projecten/iou-modern/backend/src/repository/duckdb.rs`

```rust
// REMOVE any transactional methods:
// - pub async fn insert_document(...) -> Result<...>
// - pub async fn update_document(...) -> Result<...>
// - pub async fn delete_document(...) -> Result<...>

// KEEP only analytics methods:
pub async fn query_compliance_overview(&self) -> Result<Vec<ComplianceMetric>> { ... }
pub async fn query_domain_statistics(&self) -> Result<DomainStats> { ... }
pub async fn full_text_search(&self, query: &str) -> Result<Vec<SearchResult>> { ... }
pub async fn entity_similarity_search(&self, embedding: &[f32]) -> Result<Vec<Entity>> { ... }

// RENAME module to reflect analytics-only:
// FROM: mod duckdb;
// TO: mod analytics;
```

**File:** `/Users/marc/Projecten/iou-modern/backend/src/service/document_service.rs`

```rust
// VERIFY: All write operations go to Supabase repository

// BEFORE cleanup:
let doc = self.duckdb_repo.create_document(...).await?;  // REMOVE

// AFTER cleanup:
let doc = self.supabase_repo.create_document(...).await?;  // CORRECT
```

**Analytics-Only DuckDB Module Structure:**

**File:** `/Users/marc/Projecten/iou-modern/backend/src/repository/analytics/mod.rs`

```rust
//! DuckDB analytics repository.
//!
//! This module provides read-only analytics queries against DuckDB.
//! The database is populated via ETL from Supabase (see etl module).
//! No direct writes to DuckDB should occur from application code.

use duckdb::{Connection, params};
use crate::models::{ComplianceMetric, DomainStats, SearchResult};

pub struct AnalyticsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl AnalyticsRepository {
    /// Query compliance overview metrics.
    /// Returns aggregated statistics for dashboard display.
    pub async fn compliance_overview(&self) -> Result<Vec<ComplianceMetric>> { ... }

    /// Search documents using full-text index.
    /// Leverages Dutch tsvector configuration.
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> { ... }

    /// Vector similarity search for GraphRAG entities.
    pub async fn similarity_search(&self, embedding: &[f32]) -> Result<Vec<Entity>> { ... }
}
```

**Verification Steps:**

1. Search for write operations: `rg "(INSERT|UPDATE|DELETE).*duckdb" backend/src/ -i`
2. Ensure all database writes route through Supabase repository
3. Verify ETL is the only source of DuckDB data updates

---

### Task 3: Optimize ETL Pipeline

**Objective:** Fine-tune the ETL pipeline for efficiency and reliability after stabilization phase learnings.

**File:** `/Users/marc/Projecten/iou-modern/backend/src/etl/coordinator.rs`

```rust
//! ETL Coordinator: Supabase → DuckDB data synchronization.
//!
//! After stabilization, apply optimizations based on production metrics.

use tokio::time::{interval, Duration};
use crate::repository::supabase::SupabaseRepository;
use crate::repository::analytics::AnalyticsRepository;

pub struct EtlCoordinator {
    supabase: SupabaseRepository,
    analytics: AnalyticsRepository,
    interval: Duration,
    batch_size: usize,
}

impl EtlCoordinator {
    /// Run ETL with optimized batch processing.
    pub async fn run(&self) -> Result<EtlMetrics> {
        // Apply lessons from stabilization:
        // - Batch size tuned for memory vs latency
        // - Parallel processing for independent tables
        // - Incremental sync using watermark timestamps
        // - Dead letter queue for failed records
    }

    /// Checkpoint for idempotency.
    /// Stores last successful sync timestamp.
    async fn save_checkpoint(&self, timestamp: i64) -> Result<()> { ... }
}
```

**Optimization Areas:**

1. **Batch Size Tuning:** Adjust based on memory profiling
2. **Parallel Table Processing:** Independent tables sync concurrently
3. **Watermark-Based Incremental Sync:** Only process changed records
4. **Dead Letter Queue:** Isolate failed records for retry

**File:** `/Users/marc/Projecten/iou-modern/backend/src/etl/config.rs`

```rust
/// ETL configuration with production-tuned defaults.

#[derive(Debug, Clone)]
pub struct EtlConfig {
    /// Sync interval (default: 5 minutes)
    pub interval: Duration,

    /// Records per batch (default: 1000)
    pub batch_size: usize,

    /// Enable parallel table processing (default: true)
    pub parallel_tables: bool,

    /// Max retries for failed records (default: 3)
    pub max_retries: u32,
}

impl Default for EtlConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(300),  // 5 minutes
            batch_size: 1000,
            parallel_tables: true,
            max_retries: 3,
        }
    }
}
```

---

### Task 4: Update Documentation

**Objective:** Ensure all documentation reflects the new hybrid architecture.

**Files to Update:**

**File:** `/Users/marc/Projecten/iou-modern/docs/architecture/database.md`

```markdown
# Database Architecture

## Hybrid Approach

IOU-Modern uses a hybrid database architecture:

- **Supabase (PostgreSQL):** Primary transactional database
  - User authentication and authorization
  - Document metadata and workflow states
  - Real-time subscriptions for collaboration
  - Row-Level Security (RLS) for multi-tenant isolation

- **DuckDB:** Analytics database
  - Read-only materialized views
  - Full-text search with Dutch language support
  - Vector similarity search for GraphRAG
  - Data science and BI exports

## Data Flow

```
API Writes → Supabase (Primary)
                 ↓
              ETL (Batch/CDC)
                 ↓
              DuckDB (Analytics)
```

## Connection Details

- Supabase: `postgresql://user:pass@host:5432/ioumodern`
- DuckDB: `/data/analytics.duckdb` (local file)
```

**File:** `/Users/marc/Projecten/iou-modern/docs/operations/runbooks/etl troubleshooting.md`

```markdown
# ETL Troubleshooting

## Common Issues

### Lag Between Supabase and DuckDB

**Symptom:** Analytics show stale data.

**Check:**
1. ETL service running: `systemctl status iou-etl`
2. Last checkpoint: `SELECT * FROM etl_checkpoint ORDER BY ts DESC LIMIT 1;`

**Resolution:**
- Manual trigger: `curl -X POST /admin/etl/trigger`
- Check logs: `journalctl -u iou-etl -f`
```

**File:** `/Users/marc/Projecten/iou-modern/docs/development/onboarding.md`

```markdown
# Developer Onboarding

## Database Setup

1. **Supabase (Primary):**
   ```bash
   docker-compose up -d supabase
   export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/postgres"
   ```

2. **DuckDB (Analytics):**
   ```bash
   # File created automatically by ETL
   ls -la data/analytics.duckdb
   ```

## Running Locally

```bash
cargo run
# ETL runs in background, syncs every 5 minutes
```

## Common Tasks

- **Query analytics:** Connect to `data/analytics.duckdb` using DuckDB CLI
- **Check RLS policies:** Supabase dashboard → Authentication → Policies
- **View real-time logs:** `docker-compose logs -f realtime`
```

---

### Task 5: Decommission Old Authentication

**Objective:** Remove legacy JWT middleware and user management code now that Supabase Auth handles everything.

**Files to Remove:**

| File Path | Action |
|-----------|--------|
| `/Users/marc/Projecten/iou-modern/backend/src/auth/jwt.rs` | Delete |
| `/Users/marc/Projecten/iou-modern/backend/src/auth/middleware.rs` | Delete (custom) |
| `/Users/marc/Projecten/iou-modern/backend/src/auth/users.rs` | Delete (local user store) |

**Files to Modify:**

**File:** `/Users/marc/Projecten/iou-modern/backend/src/auth/mod.rs`

```rust
//! Authentication module - Supabase Auth integration.
//!
//! User authentication is handled by Supabase Auth.
//! This module provides JWT verification middleware using Supabase keys.

use jwt_verify::Verifier;
use crate::config::SupabaseConfig;

/// Verify JWT token issued by Supabase Auth.
pub fn verify_token(token: &str, config: &SupabaseConfig) -> Result<Claims> {
    // Verify against Supabase JWT secret
    // Extract user claims (sub, role, org_id)
}

/// Axum middleware for authentication.
pub fn auth_layer(config: SupabaseConfig) -> AuthLayer {
    AuthLayer::new(config)
}
```

**File:** `/Users/marc/Projecten/iou-modern/backend/src/state.rs`

```rust
// REMOVE: legacy user store
// pub struct UserStore { ... }

// REPLACE with Supabase client reference
use supabase_rs::Client;

pub struct AppState {
    pub supabase: Client,
    pub analytics: AnalyticsRepository,
    // ... other state
}
```

---

### Task 6: Team Training and Knowledge Transfer

**Objective:** Ensure all team members are proficient with the new architecture.

**Training Checklist:**

| Topic | Delivery Method | Completion |
|-------|-----------------|------------|
| Supabase dashboard navigation | Hands-on lab | [ ] |
| RLS policy structure and debugging | Code review session | [ ] |
| Real-time subscription troubleshooting | Demo + runbook | [ ] |
| ETL pipeline operations | Runbook walkthrough | [ ] |
| PostgreSQL query optimization | Workshop | [ ] |
| Backup and recovery procedures | Drill | [ ] |

**Documentation Deliverables:**

1. **Runbook:** `/Users/marc/Projecten/iou-modern/docs/operations/runbooks/`
   - ETL troubleshooting
   - Real-time subscription issues
   - RLS policy debugging
   - Backup recovery

2. **Architecture Diagrams:**
   - Hybrid database architecture
   - Data flow (Supabase → ETL → DuckDB)
   - Authentication flow

3. **Code Examples:**
   - Adding new RLS policies
   - Creating real-time subscriptions
   - Writing analytics queries

---

## Validation Checklist

Before marking this section complete, verify:

- [ ] Custom WebSocket code removed, no references remain
- [ ] DuckDB used for analytics only (no transactional queries)
- [ ] All writes route through Supabase repository
- [ ] ETL pipeline optimized based on stabilization learnings
- [ ] Documentation updated (architecture, runbooks, onboarding)
- [ ] Legacy authentication code removed
- [ ] Team training completed
- [ ] All cleanup tests pass
- [ ] Full system smoke test passes
- [ ] No technical debt markers remaining in code

---

## Rollback Considerations

**Note:** After cleanup phase, rollback becomes **extremely complex**. The following changes are difficult to reverse:

- Custom WebSocket code deletion (would require restore from backup)
- DuckDB transactional query removal (would require code restore)

**Recommendation:** Maintain a tagged release before cleanup (`v-pre-cleanup`) for emergency rollback if critical issues emerge in the first week after cleanup.

**Rollback Procedure (Emergency Only):**

1. Revert to `v-pre-cleanup` tag
2. Restore data from most recent backup
3. Re-enable dual-write temporarily
4. Reconcile any data differences
5. Investigate root cause before re-migrating

---

## Success Criteria

Section 06 is complete when:

1. All legacy code (WebSocket, DuckDB writes, custom auth) removed
2. DuckDB exclusively serves analytics queries
3. ETL pipeline runs without manual intervention
4. Documentation fully updated and accurate
5. All team members trained on new architecture
6. Full integration test suite passes
7. No unresolved technical debt from migration

---

## Next Steps

After completing this section:

1. **Archive migration artifacts:** Move migration scripts and tools to `/docs/migration/` for reference
2. **Post-mortem:** Conduct retrospective on migration process
3. **Monitoring review:** Ensure all alerts are tuned for normal operation
4. **Feature development:** Resume normal feature development cadence