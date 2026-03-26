Now I have enough context to generate the section content. Let me create the comprehensive, self-contained section for database schema.</think># Section 01: Database Schema

## Overview

This section creates the database migration script that adds all new tables required for the enhanced document workflow system, including multi-stage approvals, delegation, escalation tracking, and version history extensions.

**Dependencies:** None (this is the foundation section)

**Files to create:**
- `/Users/marc/Projecten/iou-modern/migrations/040_enhanced_workflow.sql`

**Tests location:**
- `/Users/marc/Projecten/iou-modern/crates/iou-api/tests/database/schema_040.rs`

## Background

The IOU-Modern system uses a dual database architecture:
- **DuckDB** for embedded analytics and local data storage
- **Supabase PostgreSQL** for realtime features and remote collaboration

The migration script must be compatible with both databases. The SQL syntax in this migration uses PostgreSQL-compatible syntax that works with Supabase and should be adapted for DuckDB-specific features where necessary.

### Existing Relevant Tables

From migration 030, these tables already exist and will be extended:

```sql
-- Existing documents table
CREATE TABLE documents (
    id UUID PRIMARY KEY,
    domain_id VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    state VARCHAR NOT NULL,
    -- ... other fields
);

-- Existing document_versions table (will be extended)
CREATE TABLE document_versions (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    storage_key VARCHAR NOT NULL,
    format VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_by VARCHAR,
    change_summary TEXT,
    is_current BOOLEAN NOT NULL DEFAULT FALSE,
    compliance_score FLOAT,
    UNIQUE(document_id, version_number)
);
```

## Tests

Write these tests BEFORE implementing the migration. The tests verify that the schema can be created, queried, and enforces the required constraints.

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/tests/database/schema_040.rs`

### Test Requirements

1. **approval_stages table can be created and queried**
   - Verify table exists after migration
   - Insert a test stage and query it back
   - Validate all columns are accessible

2. **delegations table enforces unique constraint on (from_user_id, to_user_id, document_types)**
   - Insert identical delegation twice
   - Expect constraint violation error
   - Verify different document_types allow same user pair

3. **document_approval_stages enforces uniqueness on (document_id, stage_id)**
   - Attempt to insert duplicate document-stage combination
   - Verify unique constraint prevents duplicates

4. **document_approvals enforces uniqueness on (stage_instance_id, approver_id)**
   - Attempt duplicate approval from same approver
   - Verify constraint prevents duplicates

5. **approval_escalations can track escalation history**
   - Insert escalation record
   - Query escalations by document_id
   - Verify escalation_type and notification_channel are stored

6. **document_versions extended columns (is_compressed, parent_version_id, diff_summary) are nullable**
   - Insert version record without extended columns
   - Update to add extended columns
   - Verify NULL is accepted

7. **foreign key constraints prevent orphan records**
   - Attempt to insert document_approval_stages with non-existent document_id
   - Verify foreign key violation
   - Attempt to insert document_approvals with non-existent stage_instance_id
   - Verify foreign key violation

8. **indexes improve query performance for stage lookups**
   - Create test data (100+ stages)
   - Query with EXPLAIN ANALYZE
   - Verify index usage on (domain_id, document_type) lookup

### Test Stub Structure

```rust
// File: crates/iou-api/tests/database/schema_040.rs

use sqlx::{PgPool, Row};
use uuid::Uuid;

#[sqlx::test]
async fn test_approval_stages_table_exists(pool: PgPool) -> Result<(), sqlx::Error> {
    // Verify table exists
    let result = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_name = 'approval_stages'
        )"
    )
    .fetch_one(&pool)
    .await?;
    
    assert!(result.get::<bool, _>("exists"));
    Ok(())
}

#[sqlx::test]
async fn test_delegations_unique_constraint(pool: PgPool) -> Result<(), sqlx::Error> {
    // Setup: insert delegation
    let from_user = Uuid::new_v4();
    let to_user = Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO delegations (id, from_user_id, to_user_id, delegation_type, starts_at, is_active, created_at, created_by)
        VALUES ($1, $2, $3, 'temporary', NOW(), true, NOW(), $2)"
    )
    .bind(Uuid::new_v4())
    .bind(from_user)
    .bind(to_user)
    .execute(&pool)
    .await?;
    
    // Attempt duplicate - should fail
    let result = sqlx::query(
        "INSERT INTO delegations (id, from_user_id, to_user_id, delegation_type, starts_at, is_active, created_at, created_by)
        VALUES ($1, $2, $3, 'temporary', NOW(), true, NOW(), $2)"
    )
    .bind(Uuid::new_v4())
    .bind(from_user)
    .bind(to_user)
    .execute(&pool)
    .await;
    
    assert!(result.is_err());
    Ok(())
}

// ... additional tests as specified above
```

## Implementation

### Migration File

**File:** `/Users/marc/Projecten/iou-modern/migrations/040_enhanced_workflow.sql`

```sql
-- Migration: Enhanced Document Workflow Schema
-- Version: 040
-- Description: Adds multi-stage approvals, delegation, escalation, and version history extensions
-- Dependencies: Requires migrations 001-039 to be applied

-- ============================================
-- 1. APPROVAL STAGES DEFINITIONS
-- ============================================

-- Approval stage definitions (configured per domain/document type)
-- These are templates, not per-document instances
CREATE TABLE IF NOT EXISTS approval_stages (
    id VARCHAR(50) PRIMARY KEY,
    domain_id VARCHAR(50),
    document_type VARCHAR(100) NOT NULL,
    stage_name VARCHAR(200) NOT NULL,
    stage_order INTEGER NOT NULL,
    approval_type VARCHAR(20) NOT NULL,
        -- Values: 'sequential', 'parallel_any', 'parallel_all', 'parallel_majority'
    approvers JSONB NOT NULL,
        -- Array of approver objects: [{"user_id": "uuid"}, {"role": "manager"}]
    sla_hours INTEGER NOT NULL DEFAULT 72,
    expiry_action VARCHAR(50) NOT NULL DEFAULT 'notify_only',
        -- Values: 'notify_only', 'return_to_draft', 'auto_approve', 'escalate_to'
    is_optional BOOLEAN DEFAULT false,
    condition TEXT,
        -- Optional expression language for conditional stages
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for looking up stages by domain and document type
CREATE INDEX IF NOT EXISTS idx_approval_stages_domain_type 
ON approval_stages(domain_id, document_type);

-- Index for stage ordering within a document type
CREATE INDEX IF NOT EXISTS idx_approval_stages_order 
ON approval_stages(document_type, stage_order);

-- ============================================
-- 2. DELEGATIONS
-- ============================================

-- Active delegations between users
CREATE TABLE IF NOT EXISTS delegations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_user_id UUID NOT NULL,
    to_user_id UUID NOT NULL,
    delegation_type VARCHAR(20) NOT NULL,
        -- Values: 'temporary', 'permanent', 'bulk'
    document_types JSONB DEFAULT '[]',
        -- Array of document types this delegation applies to
        -- Empty array means all document types
    document_id UUID,
        -- If set, delegation applies only to this specific document
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ,
        -- NULL for permanent delegations
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    
    -- Prevent duplicate delegations for same users and document scope
    UNIQUE(from_user_id, to_user_id, document_types, document_id)
);

-- Index for finding active delegations from a user
CREATE INDEX IF NOT EXISTS idx_delegations_from_active 
ON delegations(from_user_id, is_active) 
WHERE is_active = true;

-- Index for finding delegations received by a user
CREATE INDEX IF NOT EXISTS idx_delegations_to_active 
ON delegations(to_user_id, is_active) 
WHERE is_active = true;

-- Index for time-based expiry queries
CREATE INDEX IF NOT EXISTS idx_delegations_ends_at 
ON delegations(ends_at) 
WHERE ends_at IS NOT NULL;

-- ============================================
-- 3. DOCUMENT APPROVAL STAGES (INSTANCES)
-- ============================================

-- Per-document stage instance state tracking
CREATE TABLE IF NOT EXISTS document_approval_stages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    stage_id VARCHAR(50) NOT NULL,
        -- References approval_stages.id
    stage_status VARCHAR(20) NOT NULL DEFAULT 'pending',
        -- Values: 'pending', 'in_progress', 'completed', 'skipped', 'expired'
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    deadline TIMESTAMPTZ,
    approvers JSONB NOT NULL,
        -- Resolved list of approver UUIDs for this specific document
    approvals_received JSONB DEFAULT '[]',
        -- Array of approval records
    UNIQUE(document_id, stage_id)
);

-- Index for querying document stages
CREATE INDEX IF NOT EXISTS idx_doc_stages_document 
ON document_approval_stages(document_id);

-- Index for finding in-progress stages (for expiry checker)
CREATE INDEX IF NOT EXISTS idx_doc_stages_status 
ON document_approval_stages(stage_status) 
WHERE stage_status = 'in_progress';

-- Index for deadline-based queries
CREATE INDEX IF NOT EXISTS idx_doc_stages_deadline 
ON document_approval_stages(deadline) 
WHERE deadline IS NOT NULL;

-- ============================================
-- 4. INDIVIDUAL APPROVAL RESPONSES
-- ============================================

-- Individual approval responses per stage
CREATE TABLE IF NOT EXISTS document_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stage_instance_id UUID NOT NULL REFERENCES document_approval_stages(id) ON DELETE CASCADE,
    approver_id UUID NOT NULL,
    delegated_from UUID,
        -- If approval was made via delegation, references original approver
    decision VARCHAR(20),
        -- Values: 'approved', 'rejected', 'delegated'
    comment TEXT,
    responded_at TIMESTAMPTZ,
    UNIQUE(stage_instance_id, approver_id)
);

-- Index for querying approvals by stage
CREATE INDEX IF NOT EXISTS idx_approvals_stage 
ON document_approvals(stage_instance_id);

-- ============================================
-- 5. APPROVAL ESCALATIONS
-- ============================================

-- Escalation log for tracking notifications sent
CREATE TABLE IF NOT EXISTS approval_escalations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    stage_instance_id UUID REFERENCES document_approval_stages(id),
    escalation_type VARCHAR(50) NOT NULL,
        -- Values: 'approaching_deadline', 'deadline_passed', 'custom'
    notification_channel VARCHAR(50) NOT NULL,
        -- Values: 'websocket', 'email', 'webhook'
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'pending'
        -- Values: 'pending', 'delivered', 'failed', 'acknowledged'
);

-- Index for escalation history by document
CREATE INDEX IF NOT EXISTS idx_escalations_document 
ON approval_escalations(document_id);

-- Index for pending escalations (retry logic)
CREATE INDEX IF NOT EXISTS idx_escalations_status 
ON approval_escalations(status) 
WHERE status = 'pending';

-- ============================================
-- 6. EXTEND EXISTING TABLES
-- ============================================

-- Extend document_versions table with new columns
-- Note: These columns are nullable for backward compatibility

-- Add is_compressed flag for version storage optimization
ALTER TABLE document_versions 
ADD COLUMN IF NOT EXISTS is_compressed BOOLEAN DEFAULT false;

-- Add parent version reference for version tree traversal
ALTER TABLE document_versions 
ADD COLUMN IF NOT EXISTS parent_version_id UUID REFERENCES document_versions(id);

-- Add diff summary for quick preview without loading full content
ALTER TABLE document_versions 
ADD COLUMN IF NOT EXISTS diff_summary JSONB;

-- Create index for parent version lookups
CREATE INDEX IF NOT EXISTS idx_versions_parent 
ON document_versions(parent_version_id) 
WHERE parent_version_id IS NOT NULL;

-- ============================================
-- 7. VIEWS FOR COMMON QUERIES
-- ============================================

-- View: Active stages requiring attention
-- Returns all in-progress stages with deadline info
CREATE OR REPLACE VIEW v_active_approval_stages AS
SELECT
    das.id,
    das.document_id,
    d.domain_id,
    d.document_type,
    das.stage_id,
    ast.stage_name,
    das.stage_status,
    das.started_at,
    das.deadline,
    das.approvers,
    das.approvals_received,
    -- Calculate remaining hours
    EXTRACT(EPOCH FROM (das.deadline - NOW())) / 3600 AS hours_remaining,
    -- Count escalations
    (SELECT COUNT(*) FROM approval_escalations ae 
     WHERE ae.stage_instance_id = das.id) AS escalation_count
FROM document_approval_stages das
JOIN documents d ON das.document_id = d.id
LEFT JOIN approval_stages ast ON das.stage_id = ast.id
WHERE das.stage_status = 'in_progress';

-- View: Delegation summary for a user
CREATE OR REPLACE VIEW v_user_delegations AS
SELECT
    u_from.id AS user_id,
    u_from.display_name AS user_name,
    u_to.id AS delegate_to_id,
    u_to.display_name AS delegate_to_name,
    d.delegation_type,
    d.document_types,
    d.document_id,
    d.starts_at,
    d.ends_at,
    d.is_active,
    CASE 
        WHEN d.ends_at IS NULL THEN true
        WHEN d.ends_at > NOW() THEN true
        ELSE false
    END AS is_currently_valid
FROM delegations d
JOIN users u_from ON d.from_user_id = u_from.id
JOIN users u_to ON d.to_user_id = u_to.id;

-- ============================================
-- 8. MIGRATION COMPLETE
-- ============================================

-- Record migration in schema_migrations table if it exists
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'schema_migrations') THEN
        INSERT INTO schema_migrations (version, applied_at)
        VALUES (40, NOW())
        ON CONFLICT (version) DO NOTHING;
    END IF;
END $$;
```

### DuckDB Adaptations

For DuckDB compatibility, make these adjustments:

1. Replace `gen_random_uuid()` with `uuid()`
2. Replace `JSONB` with `JSON` (DuckDB uses standard JSON)
3. Replace `TIMESTAMPTZ` with `TIMESTAMP`
4. Replace unique indexes with `WHERE` clauses using standard filtered indexes if supported
5. Use DuckDB's array syntax instead of PostgreSQL array literals

### Rollback Script

For testing and development, include a rollback procedure:

```sql
-- Rollback script for migration 040

-- Drop views
DROP VIEW IF EXISTS v_user_delegations;
DROP VIEW IF EXISTS v_active_approval_stages;

-- Drop indexes
DROP INDEX IF EXISTS idx_versions_parent;
DROP INDEX IF EXISTS idx_escalations_status;
DROP INDEX IF EXISTS idx_escalations_document;
DROP INDEX IF EXISTS idx_approvals_stage;
DROP INDEX IF EXISTS idx_doc_stages_deadline;
DROP INDEX IF EXISTS idx_doc_stages_status;
DROP INDEX IF EXISTS idx_doc_stages_document;
DROP INDEX IF EXISTS idx_delegations_ends_at;
DROP INDEX IF EXISTS idx_delegations_to_active;
DROP INDEX IF EXISTS idx_delegations_from_active;
DROP INDEX IF EXISTS idx_approval_stages_order;
DROP INDEX IF EXISTS idx_approval_stages_domain_type;

-- Drop extended columns
ALTER TABLE document_versions DROP COLUMN IF EXISTS diff_summary;
ALTER TABLE document_versions DROP COLUMN IF EXISTS parent_version_id;
ALTER TABLE document_versions DROP COLUMN IF EXISTS is_compressed;

-- Drop tables (order matters for foreign keys)
DROP TABLE IF EXISTS approval_escalations;
DROP TABLE IF EXISTS document_approvals;
DROP TABLE IF EXISTS document_approval_stages;
DROP TABLE IF EXISTS delegations;
DROP TABLE IF EXISTS approval_stages;
```

## Verification Steps

After applying the migration, verify:

1. **Table Creation**
   - All 5 new tables exist
   - All indexes are created
   - Views are queryable

2. **Constraint Enforcement**
   - Foreign key relationships work
   - Unique constraints prevent duplicates
   - CASCADE deletes work correctly

3. **Backward Compatibility**
   - Existing documents table unchanged
   - Existing document_versions records work without new columns
   - Queries on old tables still function

4. **Performance**
   - Index usage confirmed via EXPLAIN ANALYZE
   - View queries return acceptable performance

## Next Steps

After completing this section:
- **section-02-core-types** - Create Rust types matching these tables
- **section-03-config-system** - Build configuration loading for approval stages
- **section-04-multi-stage-engine** - Implement the workflow engine using these tables

## Implementation Notes

### Changes from Original Plan

The following adjustments were made during implementation based on code review:

1. **JSON vs JSONB**: Used `JSON` instead of `JSONB` for broader compatibility
2. **TIMESTAMP**: Used `TIMESTAMP` instead of `TIMESTAMPTZ`
3. **UUID generation**: Removed `DEFAULT gen_random_uuid()` from PRIMARY KEY - handled by application layer
4. **Unique constraint fix**: Removed `document_types` from UNIQUE constraint due to JSON binary comparison issues
5. **CHECK constraints added**: All enum-like columns now have CHECK constraints for data validation
6. **FK enforcement**: Added actual foreign key for `stage_id` (was just a comment)
7. **CASCADE fixes**:
   - Added `ON DELETE CASCADE` to `approval_escalations.stage_instance_id`
   - Added `ON DELETE RESTRICT` to `document_versions.parent_version_id`
8. **Additional indexes**:
   - `idx_escalations_stage_instance` on `approval_escalations(stage_instance_id)`
   - `idx_approvals_approver` on `document_approvals(approver_id)`
9. **Delegation time range check**: Added CHECK constraint ensuring `ends_at > starts_at` or NULL

### Files Modified/Created

- `migrations/040_enhanced_workflow.sql` - Migration with all fixes applied
- `crates/iou-api/tests/database/mod.rs` - Module declaration
- `crates/iou-api/tests/database/schema_040.rs` - Test stubs (implementation deferred)
- `crates/iou-api/tests/common.rs` - Added database module import

## Notes

- The migration uses `IF NOT EXISTS` throughout to support re-running during development
- Extended columns on `document_versions` are nullable to preserve backward compatibility
- The `document_types` JSON field in `delegations` allows flexible scope matching
- CHECK constraints ensure database-level validation for all enum-like columns
- Consider adding database triggers in a future migration for automatic timestamp updates