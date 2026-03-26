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
    approvers JSON NOT NULL,
        -- Array of approver objects: [{"user_id": "uuid"}, {"role": "manager"}]
    sla_hours INTEGER NOT NULL DEFAULT 72,
    expiry_action VARCHAR(50) NOT NULL DEFAULT 'notify_only',
        -- Values: 'notify_only', 'return_to_draft', 'auto_approve', 'escalate_to'
    is_optional BOOLEAN DEFAULT false,
    condition TEXT,
        -- Optional expression language for conditional stages
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Index for looking up stages by domain and document type
CREATE INDEX IF NOT EXISTS idx_approval_stages_domain_type
ON approval_stages(domain_id, document_type);

-- Index for stage ordering within a document type
CREATE INDEX IF NOT EXISTS idx_approval_stages_order
ON approval_stages(document_type, stage_order);

-- CHECK constraints for enum-like columns
ALTER TABLE approval_stages
ADD CONSTRAINT chk_approval_type
CHECK (approval_type IN ('sequential', 'parallel_any', 'parallel_all', 'parallel_majority'));

ALTER TABLE approval_stages
ADD CONSTRAINT chk_expiry_action
CHECK (expiry_action IN ('notify_only', 'return_to_draft', 'auto_approve', 'escalate_to'));

-- ============================================
-- 2. DELEGATIONS
-- ============================================

-- Active delegations between users
CREATE TABLE IF NOT EXISTS delegations (
    id UUID PRIMARY KEY,
    from_user_id UUID NOT NULL,
    to_user_id UUID NOT NULL,
    delegation_type VARCHAR(20) NOT NULL,
        -- Values: 'temporary', 'permanent', 'bulk'
    document_types JSON DEFAULT '[]',
        -- Array of document types this delegation applies to
        -- Empty array means all document types
    document_id UUID,
        -- If set, delegation applies only to this specific document
    starts_at TIMESTAMP NOT NULL DEFAULT NOW(),
    ends_at TIMESTAMP,
        -- NULL for permanent delegations
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,

    -- Prevent duplicate delegations for same users and document scope
    -- Note: document_types excluded from UNIQUE due to JSON binary comparison
    UNIQUE(from_user_id, to_user_id, document_id)
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

-- CHECK constraints for delegations
ALTER TABLE delegations
ADD CONSTRAINT chk_delegation_type
CHECK (delegation_type IN ('temporary', 'permanent', 'bulk'));

ALTER TABLE delegations
ADD CONSTRAINT chk_delegation_time_range
CHECK (ends_at IS NULL OR ends_at > starts_at);

-- ============================================
-- 3. DOCUMENT APPROVAL STAGES (INSTANCES)
-- ============================================

-- Per-document stage instance state tracking
CREATE TABLE IF NOT EXISTS document_approval_stages (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    stage_id VARCHAR(50) NOT NULL REFERENCES approval_stages(id),
    stage_status VARCHAR(20) NOT NULL DEFAULT 'pending',
        -- Values: 'pending', 'in_progress', 'completed', 'skipped', 'expired'
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    deadline TIMESTAMP,
    approvers JSON NOT NULL,
        -- Resolved list of approver UUIDs for this specific document
    approvals_received JSON DEFAULT '[]',
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

-- CHECK constraint for stage status
ALTER TABLE document_approval_stages
ADD CONSTRAINT chk_stage_status
CHECK (stage_status IN ('pending', 'in_progress', 'completed', 'skipped', 'expired'));

-- ============================================
-- 4. INDIVIDUAL APPROVAL RESPONSES
-- ============================================

-- Individual approval responses per stage
CREATE TABLE IF NOT EXISTS document_approvals (
    id UUID PRIMARY KEY,
    stage_instance_id UUID NOT NULL REFERENCES document_approval_stages(id) ON DELETE CASCADE,
    approver_id UUID NOT NULL,
    delegated_from UUID,
        -- If approval was made via delegation, references original approver
    decision VARCHAR(20),
        -- Values: 'approved', 'rejected', 'delegated'
    comment TEXT,
    responded_at TIMESTAMP,
    UNIQUE(stage_instance_id, approver_id)
);

-- Index for querying approvals by stage
CREATE INDEX IF NOT EXISTS idx_approvals_stage
ON document_approvals(stage_instance_id);

-- Index for querying approvals by approver
CREATE INDEX IF NOT EXISTS idx_approvals_approver
ON document_approvals(approver_id);

-- CHECK constraint for decision
ALTER TABLE document_approvals
ADD CONSTRAINT chk_decision
CHECK (decision IN ('approved', 'rejected', 'delegated'));

-- ============================================
-- 5. APPROVAL ESCALATIONS
-- ============================================

-- Escalation log for tracking notifications sent
CREATE TABLE IF NOT EXISTS approval_escalations (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    stage_instance_id UUID REFERENCES document_approval_stages(id) ON DELETE CASCADE,
    escalation_type VARCHAR(50) NOT NULL,
        -- Values: 'approaching_deadline', 'deadline_passed', 'custom'
    notification_channel VARCHAR(50) NOT NULL,
        -- Values: 'websocket', 'email', 'webhook'
    sent_at TIMESTAMP NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMP,
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

-- Index for querying escalations by stage instance
CREATE INDEX IF NOT EXISTS idx_escalations_stage_instance
ON approval_escalations(stage_instance_id);

-- CHECK constraints for escalations
ALTER TABLE approval_escalations
ADD CONSTRAINT chk_escalation_type
CHECK (escalation_type IN ('approaching_deadline', 'deadline_passed', 'custom'));

ALTER TABLE approval_escalations
ADD CONSTRAINT chk_notification_channel
CHECK (notification_channel IN ('websocket', 'email', 'webhook'));

ALTER TABLE approval_escalations
ADD CONSTRAINT chk_escalation_status
CHECK (status IN ('pending', 'delivered', 'failed', 'acknowledged'));

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
ADD COLUMN IF NOT EXISTS parent_version_id UUID REFERENCES document_versions(id) ON DELETE RESTRICT;

-- Add diff summary for quick preview without loading full content
ALTER TABLE document_versions
ADD COLUMN IF NOT EXISTS diff_summary JSON;

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
    d.id,
    d.from_user_id,
    d.to_user_id,
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
FROM delegations d;

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
