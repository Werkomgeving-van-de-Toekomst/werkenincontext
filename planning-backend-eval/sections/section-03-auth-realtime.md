Now I have all the context I need. Let me generate the content for section-03-auth-realtime. This section covers Phase 2: Auth & Real-time (Weeks 5-6).

Based on the context:
- Section 03 builds on Section 02 (Foundation - Supabase deployed, dual-write working)
- It covers Supabase Auth configuration, RLS policies, user data migration, real-time subscriptions
- The tests from claude-plan-tdd.md Phase 2 are relevant

Let me create the section content:

# Section 03: Authentication and Real-time Implementation

**Phase 2 | Weeks 5-6**

## Overview

This section implements Supabase Authentication with Row-Level Security (RLS) policies and real-time data synchronization. This is the bridge between the foundation infrastructure and the full cutover to Supabase.

**Dependencies:** This section requires completion of section-02-foundation (Supabase deployed, dual-write pattern operational).

**Key Outcomes:**
- Supabase Auth configured and integrated with existing Rust backend
- Row-Level Security policies enforcing multi-tenant isolation
- User data migrated from DuckDB to Supabase
- Real-time subscriptions operational for collaborative features
- Frontend (Dioxus) consuming real-time updates

---

## Tests FIRST

Before implementing, write tests to verify the following behaviors. Place tests in `/Users/marc/Projecten/iou-modern/backend/tests/migration/auth_realtime.rs`.

### Authentication Tests

```rust
// Test file: backend/tests/migration/auth_realtime.rs

#[cfg(test)]
mod auth_tests {
    use super::*;

    /// Verify Supabase Auth issues valid JWT tokens
    #[tokio::test]
    async fn test_supabase_jwt_issuance() {
        // TODO: Implement test that:
        // 1. Calls Supabase Auth endpoint with valid credentials
        // 2. Receives JWT access token
        // 3. Verifies token signature and claims
        // 4. Confirms token contains user_id and organization_id
    }

    /// Verify existing users can authenticate with migrated credentials
    #[tokio::test]
    async fn test_password_hash_compatibility() {
        // TODO: Implement test that:
        // 1. Selects a sample user from pre-migration data
        // 2. Authenticates via Supabase Auth
        // 3. Verifies authentication succeeds
        // 4. Confirms password hash was correctly migrated
    }

    /// Verify existing sessions remain valid after migration
    #[tokio::test]
    async fn test_session_token_migration() {
        // TODO: Implement test that:
        // 1. Captures existing session tokens
        // 2. Migrates user data
        // 3. Verifies existing tokens still authenticate
        // 4. Confirms session data preserved
    }

    /// Verify user data migration completeness
    #[tokio::test]
    async fn test_user_data_migration() {
        // TODO: Implement test that:
        // 1. Counts users in DuckDB before migration
        // 2. Runs migration
        // 3. Counts users in Supabase after migration
        // 4. Verifies count matches
        // 5. Spot-checks random user records for field accuracy
    }
}
```

### RLS Policy Tests

```rust
#[cfg(test)]
mod rls_tests {
    use super::*;

    /// Verify organization isolation prevents cross-organization data access
    #[tokio::test]
    async fn test_rls_organization_isolation() {
        // TODO: Implement test that:
        // 1. Creates two users in different organizations
        // 2. User A attempts to access User B's documents
        // 3. Verifies access is denied
        // 4. User A accesses their own documents
        // 5. Verifies access succeeds
    }

    /// Verify user-level access within organization
    #[tokio::test]
    async fn test_rls_user_level_access() {
        // TODO: Implement test that:
        // 1. Creates user with limited permissions
        // 2. User attempts to access admin-only resource
        // 3. Verifies access denied
        // 4. User accesses permitted resource
        // 5. Verifies access succeeds
    }

    /// Verify classification-based filtering (confidential documents)
    #[tokio::test]
    async fn test_rls_classification_filtering() {
        // TODO: Implement test that:
        // 1. Creates user without clearance
        // 2. User attempts to access confidential document
        // 3. Verifies access denied
        // 4. User with clearance accesses same document
        // 5. Verifies access succeeds
    }

    /// Verify Woo-publication status filtering
    #[tokio::test]
    async fn test_rls_woo_filtering() {
        // TODO: Implement test that:
        // 1. Creates public and non-public documents
        // 2. Anonymous user queries for documents
        // 3. Verifies only Woo-published documents returned
        // 4. Authenticated user queries
        // 5. Verifies all permitted documents returned
    }

    /// Verify RLS policy performance meets SLA
    #[tokio::test]
    async fn test_rls_policy_performance() {
        // TODO: Implement test that:
        // 1. Executes 100 queries with RLS enforced
        // 2. Measures p50, p95, p99 latency
        // 3. Verifies p95 < 500ms
        // 4. Reports detailed metrics
    }
}
```

### Real-time Tests

```rust
#[cfg(test)]
mod realtime_tests {
    use super::*;

    /// Verify client can create real-time subscription
    #[tokio::test]
    async fn test_realtime_subscription_creation() {
        // TODO: Implement test that:
        // 1. Connects to Supabase Realtime
        // 2. Subscribes to a table channel
        // 3. Verifies subscription confirmation received
        // 4. Confirms connection state is "subscribed"
    }

    /// Verify document updates propagate to subscribers
    #[tokio::test]
    async fn test_realtime_document_updates() {
        // TODO: Implement test that:
        // 1. Client A subscribes to document changes
        // 2. Client B updates a document
        // 3. Client A receives update notification
        // 4. Verifies payload contains correct document state
    }

    /// Verify user presence indicators
    #[tokio::test]
    async fn test_realtime_presence_indicators() {
        // TODO: Implement test that:
        // 1. User A joins a document channel
        // 2. User B joins same channel
        // 3. Both users receive presence updates
        // 4. User A leaves channel
        // 5. User B receives leave notification
    }

    /// Verify concurrent edit conflict resolution
    #[tokio::test]
    async fn test_realtime_conflict_resolution() {
        // TODO: Implement test that:
        // 1. Two users edit same document field simultaneously
        // 2. Verifies last-write-wins or merge strategy applied
        // 3. Confirms no data corruption
        // 4. Both clients see consistent final state
    }

    /// Verify real-time latency meets requirements
    #[tokio::test]
    async fn test_realtime_latency() {
        // TODO: Implement test that:
        // 1. Subscribes to a channel
        // 2. Records timestamp before update
        // 3. Triggers database update
        // 4. Records timestamp when notification received
        // 5. Calculates latency
        // 6. Verifies p95 < 200ms
    }

    /// Verify Dioxus frontend can consume real-time updates
    #[tokio::test]
    async fn test_frontend_realtime_integration() {
        // TODO: Implement test that:
        // 1. Spawns mock Dioxus frontend component
        // 2. Subscribes to real-time channel
        // 3. Triggers backend update
        // 4. Verifies component re-renders with new data
    }

    /// Compare Custom WebSocket vs Supabase Realtime functionality
    #[tokio::test]
    async fn test_websocket_vs_supabase_comparison() {
        // TODO: Implement test that:
        // 1. Documents feature gaps between approaches
        // 2. Tests equivalent operations on both
        // 3. Produces comparison matrix for decision
    }
}
```

### Compliance Tests

```rust
#[cfg(test)]
mod compliance_tests {
    use super::*;

    /// Verify GDPR right to deletion works
    #[tokio::test]
    async fn test_gdpr_right_to_deletion() {
        // TODO: Implement test that:
        // 1. Creates user with associated data
        // 2. Triggers deletion request
        // 3. Verifies all user data deleted
        // 4. Verifies audit trail preserved (deletion logged)
    }

    /// Verify audit trail continuity during migration
    #[tokio::test]
    async fn test_audit_trail_continuity() {
        // TODO: Implement test that:
        // 1. Records pre-migration audit entry count
        // 2. Runs auth migration
        // 3. Verifies DuckDB logs correlate with Supabase WAL
        // 4. Confirms no audit gaps
    }
}
```

---

## Implementation Details

### 1. Supabase Auth Configuration

**File:** `/Users/marc/Projecten/iou-modern/backend/config/supabase_auth.toml`

Configure Supabase Auth with settings appropriate for Dutch government use:

```toml
# Supabase Auth Configuration
[auth]
# JWT settings
jwt_expiry = 3600  # 1 hour
refresh_token_expiry = 2592000  # 30 days

# Email settings (for self-hosted)
[auth.email]
double_confirm_changes = true
enable_signup = false  # Admin-only user creation

# Security settings
[auth.security]
min_password_length = 12
require_uppercase = true
require_lowercase = true
require_number = true
require_special_char = true

# Session settings
[auth.session]
inactivity_timeout = 3600  # 1 hour
max_session_length = 86400  # 24 hours
```

**Key Implementation Tasks:**

1. **Configure JWT Secret:**
   - Generate secure JWT secret using `openssl rand -base64 48`
   - Store in environment variable (`SUPABASE_JWT_SECRET`)
   - Configure Rust backend to verify tokens

2. **Rust Backend JWT Verification Module:**

```rust
// File: backend/src/auth/supabase_jwt.rs

use jsonwebtoken::{decode, Validation, DecodingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SupabaseClaims {
    pub sub: String,        // User ID
    pub aud: String,        // "authenticated"
    pub role: String,       // User role
    pub email: String,
    pub organization_id: String,  // Custom claim
    pub exp: i64,
    pub iat: i64,
}

pub struct SupabaseJwtVerifier {
    decoding_key: DecodingKey,
}

impl SupabaseJwtVerifier {
    pub fn new(jwt_secret: &str) -> Result<Self, AuthError> {
        // TODO: Create verifier with JWT secret
        // Set up validation for Supabase token structure
        unimplemented!()
    }

    pub fn verify(&self, token: &str) -> Result<SupabaseClaims, AuthError> {
        // TODO: Verify token signature and extract claims
        // Check expiration, audience, issuer
        unimplemented!()
    }
}
```

3. **Integration with Existing Auth Middleware:**

```rust
// File: backend/src/auth/middleware.rs

use axum::{extract::Request, middleware::Next, response::Response};
use http::StatusCode;

pub async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Extract Authorization header
    // Verify JWT using SupabaseJwtVerifier
    // Attach claims to request extensions
    // Pass to next handler or return 401
    unimplemented!()
}
```

### 2. Row-Level Security (RLS) Policies

**File:** `/Users/marc/Projecten/iou-modern/backend/db/supabase/migrations/002_rls_policies.sql`

Create RLS policies for each table. This is a critical security component.

```sql
-- Enable RLS on all tables
ALTER TABLE information_domains ENABLE ROW LEVEL SECURITY;
ALTER TABLE information_objects ENABLE ROW LEVEL SECURITY;
ALTER TABLE documents ENABLE ROW LEVEL SECURITY;
ALTER TABLE templates ENABLE ROW LEVEL SECURITY;

-- Helper function to extract organization_id from JWT
CREATE OR REPLACE FUNCTION auth.organization_id() 
RETURNS uuid AS $$
  SELECT nullif(current_setting('request.jwt.claim.organization_id', true), '')::uuid
$$ LANGUAGE sql STABLE;

-- Organization Isolation Policy
-- Users can only read records from their organization
CREATE POLICY org_isolation_read ON information_domains
  FOR SELECT
  USING (organization_id = auth.organization_id());

CREATE POLICY org_isolation_read ON information_objects
  FOR SELECT
  USING (organization_id = auth.organization_id());

CREATE POLICY org_isolation_read ON documents
  FOR SELECT
  USING (organization_id = auth.organization_id());

-- Classification-based filtering
-- Users without CONFIDENTIAL clearance cannot see confidential records
CREATE POLICY classification_filter ON documents
  FOR SELECT
  USING (
    classification != 'CONFIDENTENTIAL' 
    OR EXISTS (
      SELECT 1 FROM user_clearances 
      WHERE user_id = auth.uid() 
      AND clearance_level >= 'CONFIDENTIAL'
    )
  );

-- Woo publication filtering
-- Anonymous/public users can only see Woo-published documents
CREATE POLICY woo_public_read ON documents
  FOR SELECT
  TO public
  USING (woo_published = true);

-- Authenticated users can see their org's documents
CREATE POLICY woo_authenticated_read ON documents
  FOR SELECT
  TO authenticated
  USING (
    woo_published = true 
    OR organization_id = auth.organization_id()
  );
```

**Performance Optimization:**

```sql
-- Create indexes to support RLS policies
CREATE INDEX idx_documents_org ON documents(organization_id);
CREATE INDEX idx_documents_woo ON documents(woo_published) WHERE woo_published = true;
CREATE INDEX idx_documents_classification ON documents(classification);

-- Use partial indexes for common RLS patterns
CREATE INDEX idx_documents_active 
  ON documents(organization_id) 
  WHERE status = 'active';
```

### 3. User Data Migration

**File:** `/Users/marc/Projecten/iou-modern/backend/src/migration/user_migration.rs`

Migrate users from DuckDB to Supabase Auth:

```rust
use sqlx::{PgPool, Pool, Sqlite};
use bcrypt::{hash, verify, DEFAULT_COST};

pub struct UserMigrator {
    duckdb: Pool<Sqlite>,
    supabase: PgPool,
}

impl UserMigrator {
    pub async fn migrate_all_users(&self) -> Result<MigrationReport, MigrationError> {
        // TODO: Implement migration pipeline:
        // 1. Read all users from DuckDB
        // 2. For each user:
        //    a. Extract password hash (verify format)
        //    b. Insert into auth.users table
        //    c. Insert user metadata into public.user_profiles
        //    d. Record migration in audit_trail
        // 3. Return migration report with counts
        unimplemented!()
    }

    async fn verify_password_compatibility(
        &self,
        password_hash: &str,
    ) -> Result<(), MigrationError> {
        // TODO: Verify existing password hash format
        // DuckDB may use different hashing than Supabase
        // May need to rehash or configure Supabase to accept format
        unimplemented!()
    }

    pub async fn rollback_user(&self, user_id: &str) -> Result<(), MigrationError> {
        // TODO: Remove migrated user from Supabase
        // For rollback scenarios
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct MigrationReport {
    pub total_users: usize,
    pub migrated: usize,
    pub failed: Vec<String>,
    pub duration_ms: u64,
}
```

**Password Hash Compatibility:**

Supabase Auth uses bcrypt by default. If DuckDB users use a different hash format:

1. Option A: Rehash on first login (gradual migration)
2. Option B: Pre-migrate all hashes during migration window
3. Option C: Configure Supabase to accept legacy hashes

**Recommended:** Option A with Option B as background task.

### 4. Real-time Subscriptions

**File:** `/Users/marc/Projecten/iou-modern/backend/src/realtime/supabase.rs`

Implement real-time subscription management:

```rust
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

pub struct RealtimeClient {
    /// Supabase Realtime WebSocket URL
    url: String,
    /// JWT token for authentication
    jwt_token: String,
}

impl RealtimeClient {
    pub fn new(url: String, jwt_token: String) -> Self {
        Self { url, jwt_token }
    }

    pub async fn subscribe(
        &self,
        table: &str,
        filter: Option<&str>,
    ) -> Result<SubscriptionHandle, RealtimeError> {
        // TODO: 
        // 1. Connect to Supabase Realtime WebSocket
        // 2. Send subscription message with table and filter
        // 3. Wait for confirmation
        // 4. Return handle for receiving updates
        unimplemented!()
    }

    pub async fn broadcast_document_update(
        &self,
        document_id: &str,
        update_type: UpdateType,
        payload: Value,
    ) -> Result<(), RealtimeError> {
        // TODO: Broadcast update to all subscribers
        // This may be handled automatically by Supabase
        // Or may require explicit broadcast for custom events
        unimplemented!()
    }
}

pub enum UpdateType {
    Created,
    Updated,
    Deleted,
    StatusChanged,
}

pub struct SubscriptionHandle {
    channel: String,
    receiver: tokio::sync::mpsc::Receiver<RealtimeEvent>,
}

#[derive(Debug, Clone)]
pub struct RealtimeEvent {
    pub table: String,
    pub record_type: String,
    pub record: Value,
    pub old_record: Option<Value>,
}
```

**Presence Tracking:**

```rust
// File: backend/src/realtime/presence.rs

pub struct PresenceTracker {
    /// Track users currently viewing/editing documents
    connections: dashmap::DashMap<String, Vec<PresenceInfo>>,
}

#[derive(Debug, Clone)]
pub struct PresenceInfo {
    pub user_id: String,
    pub user_name: String,
    pub document_id: String,
    pub status: PresenceStatus,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum PresenceStatus {
    Viewing,
    Editing,
    Idle,
}

impl PresenceTracker {
    pub fn update_presence(&self, info: PresenceInfo) {
        // TODO: Update user presence state
        // Broadcast to other subscribers
        unimplemented!()
    }

    pub fn get_document_viewers(&self, document_id: &str) -> Vec<PresenceInfo> {
        // TODO: Return list of users viewing a document
        unimplemented!()
    }

    pub fn remove_user(&self, user_id: &str) {
        // TODO: Clean up presence when user disconnects
        unimplemented!()
    }
}
```

### 5. Frontend Integration (Dioxus)

**File:** `/Users/marc/Projecten/iou-modern/frontend/src/realtime/mod.rs`

Dioxus hooks for consuming real-time updates:

```rust
use dioxus::prelude::*;
use gloo_net::websocket::WebSocket;

/// Hook for subscribing to real-time document updates
pub fn use_realtime_documents(document_id: String) -> UseRealtimeDocuments {
    // TODO: 
    // 1. Establish WebSocket connection on mount
    // 2. Subscribe to document channel
    // 3. Update state when changes received
    // 4. Clean up on unmount
    unimplemented!()
}

/// Hook for presence indicators
pub fn use_presence(document_id: String) -> UsePresence {
    // TODO:
    // 1. Subscribe to presence channel
    // 2. Track current viewers/editors
    // 3. Return presence state
    unimplemented!()
}

/// Hook for collaborative editing
pub fn use_collaborative_edit(document_id: String) -> UseCollaborativeEdit {
    // TODO:
    // 1. Subscribe to edit events
    // 2. Broadcast local edits
    // 3. Handle conflict resolution
    // 4. Merge remote edits
    unimplemented!()
}
```

**Component Example:**

```rust
// File: frontend/src/components/document_editor.rs

#[component]
pub fn DocumentEditor(document_id: String) -> Element {
    let realtime = use_realtime_documents(document_id.clone());
    let presence = use_presence(document_id.clone());
    let mut content = use_signal(|| String::new());

    // TODO: 
    // - Display current document content
    // - Show presence indicators (who else is viewing/editing)
    // - Handle real-time updates
    // - Broadcast local changes

    rsx! {
        div { class: "document-editor",
            div { class: "presence-indicators",
                // Show list of active users
            }
            textarea {
                value: "{content}",
                oninput: move |e| content.set(e.value()),
            }
        }
    }
}
```

### 6. WebSocket vs Supabase Realtime Evaluation

**Decision Point:** Evaluate whether to keep custom WebSocket implementation or fully migrate to Supabase Realtime.

**Comparison Framework:**

| Feature | Custom WebSocket | Supabase Realtime | Decision |
|---------|------------------|-------------------|----------|
| Document status broadcasts | Full control | Limited | Hybrid? |
| Presence tracking | Custom implementation | Built-in | Evaluate |
| Conflict resolution | Full control | Last-write-wins | Custom needed? |
| Fine-grained filters | Custom logic | RLS-based | RLS sufficient? |
| Connection management | Full control | Auto-managed | Auto preferred |
| Network conditions | Custom tuning | Standard | Test required |

**Implementation Approach:**

1. **Phase 2A (Week 5):** Implement Supabase Realtime for basic table subscriptions
2. **Phase 2B (Week 6):** Evaluate feature gaps with custom WebSocket
3. **Decision:** Based on evaluation, choose:
   - Full Supabase Realtime migration
   - Hybrid approach (Supabase for basic, custom for advanced features)
   - Keep custom WebSocket (defer Supabase Realtime)

---

## File Structure

Create the following files as part of this section:

```
backend/
├── src/
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── supabase_jwt.rs          # JWT verification
│   │   └── middleware.rs            # Auth middleware
│   ├── migration/
│   │   ├── mod.rs
│   │   └── user_migration.rs        # User migration logic
│   └── realtime/
│       ├── mod.rs
│       ├── supabase.rs              # Supabase Realtime client
│       └── presence.rs              # Presence tracking
├── db/
│   └── supabase/
│       └── migrations/
│           ├── 002_rls_policies.sql # RLS policies
│           └── 003_user_profiles.sql # User profile schema
├── config/
│   └── supabase_auth.toml           # Auth configuration
└── tests/
    └── migration/
        └── auth_realtime.rs         # Integration tests

frontend/
└── src/
    └── realtime/
        └── mod.rs                   # Dioxus realtime hooks
```

---

## Dependencies

This section depends on the following:

### From section-02-foundation:
- Supabase Docker deployment running
- PostgreSQL schema created
- Database connection pool configured in Rust backend
- Dual-write pattern operational

### External Dependencies (Cargo.toml additions):

```toml
# Backend additions
jsonwebtoken = "9"          # JWT verification
bcrypt = "0.16"             # Password hashing
tokio-tungstenite = "0.24"  # WebSocket client
gloo-net = "0.6"            # Frontend WebSocket
dashmap = "6"               # Concurrent presence tracking
```

### Supabase Extensions Required:
- `pg_cron` (for retention policies - may defer to section-04)
- `pgjwt` (for JWT support)

---

## Rollback Procedures

If this section needs to be rolled back:

1. **Auth Rollback:**
   - Switch JWT verification back to existing middleware
   - Drop Supabase user records (keep DuckDB users)

2. **RLS Rollback:**
   - Disable RLS policies: `ALTER TABLE {table} DISABLE ROW LEVEL SECURITY;`
   - Application-level filtering resumes

3. **Real-time Rollback:**
   - Re-enable custom WebSocket endpoints
   - Disconnect Supabase Realtime subscriptions

4. **User Data Rollback:**
   ```rust
   // Run if user migration needs reversal
   UserMigrator::rollback_all_users(&pool).await?;
   ```

---

## Validation Checklist

Complete each item and verify corresponding test passes:

- [ ] Supabase Auth configured and issuing valid JWT tokens
- [ ] Existing users can authenticate with migrated credentials  
- [ ] RLS policy for organization isolation passes test
- [ ] RLS policy for classification filtering passes test
- [ ] RLS policy for Woo publication filtering passes test
- [ ] RLS policy performance meets p95 < 500ms requirement
- [ ] Real-time subscription creation succeeds
- [ ] Document updates propagate to subscribers
- [ ] Presence indicators work correctly
- [ ] Real-time latency meets < 200ms requirement
- [ ] Frontend (Dioxus) can consume real-time updates
- [ ] WebSocket vs Supabase Realtime evaluation complete
- [ ] GDPR right to deletion works with RLS
- [ ] Audit trail continuity verified

---

## Next Steps

After completing this section:

1. **section-04-cutover:** Migrate primary API read traffic to Supabase
2. Implement ETL pipeline from Supabase to DuckDB
3. Configure DuckDB for analytics-only workload
4. Remove dual-write pattern (only after validation)

---

## Implementation Notes (Actual Implementation)

### Files Created

**Backend (crates/iou-api):**
- `src/auth/mod.rs` - Auth module with Supabase JWT integration
- `src/auth/supabase_jwt.rs` - JWT verification for Supabase tokens
- `src/migration/mod.rs` - Migration module
- `src/migration/user_migration.rs` - User migration from DuckDB to Supabase
- `src/realtime/mod.rs` - Real-time module
- `src/realtime/supabase_rt.rs` - Supabase Realtime client (stub)
- `src/realtime/presence.rs` - Presence tracking for collaborative editing
- `tests/migration/auth_realtime.rs` - Integration tests (placeholder)

**Migrations:**
- `migrations/postgres/002_rls_policies.sql` - Row-Level Security policies

**Modified Files:**
- `crates/iou-api/src/lib.rs` - Added auth, migration, realtime, middleware exports
- `crates/iou-api/src/middleware/mod.rs` - Added Role export
- `crates/iou-api/src/dual_write.rs` - Fixed unsafe env var calls

### Deviations from Plan

1. **Config file not created:** `supabase_auth.toml` not created - configuration is done via environment variables and code.

2. **Frontend Dioxus hooks not implemented:** Real-time hooks for Dioxus frontend are deferred to section-04.

3. **Realtime WebSocket connection is a stub:** The `RealtimeClient` structure is in place but doesn't actually connect to WebSocket. This is intentional - the actual connection will be implemented in section-04 (cutover).

4. **DuckDB user migration query is a placeholder:** The `get_all_duckdb_users()` method returns an empty vector. This will be implemented during the actual cutover phase when the DuckDB schema is finalized.

5. **Tests are placeholders:** All integration tests are marked with `#[ignore]` and serve as documentation of expected behavior. They will be enabled in section-04.

### Code Review Fixes Applied

During code review, the following fixes were applied:

1. **RLS Clearance Function:** Fixed to properly check clearance level for 'intern' documents (was granting access to all authenticated users).

2. **JWT Default Implementation:** Removed panicking `Default` impl for `SupabaseJwtVerifier`.

3. **Presence Tracker Race Condition:** Fixed `update_presence()` to use DashMap's entry API for atomic updates.

4. **JSON Building in Migration:** Fixed to use `jsonb_build_object()` instead of string concatenation.

5. **Instance ID Configurable:** Added `instance_id` to `MigrationConfig` to avoid hardcoded value.

### Technical Decisions

1. **Supabase JWT Verification:** Implemented separate `SupabaseJwtVerifier` to handle Supabase-specific JWT format with custom claims (organization_id, clearance, app_roles).

2. **RLS Policy Pattern:** Used a mix of direct organization_id checks and EXISTS subqueries for cross-table relationships. The subquery approach is used for `information_objects` which reference `information_domains` for organization context.

3. **Presence Tracking:** Used DashMap for concurrent access without locks. Cleanup method exists but is not automatically called (manual cleanup or background task needed for production).

4. **Migration Batching:** Config supports `batch_size` but it's not yet used in the implementation. This will be important for production migrations with large user counts.

### Known Limitations

1. No actual WebSocket connection in RealtimeClient
2. User migration from DuckDB returns empty list (placeholder)
3. Integration tests are all ignored (require Supabase instance)
4. No automatic cleanup task for stale presence entries
5. Batch size config defined but not utilized
