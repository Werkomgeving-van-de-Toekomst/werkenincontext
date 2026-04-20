-- ============================================
-- Workflow Automation Builder Database Schema
-- Migration: 050_workflow_builder
-- ============================================

-- Enable DuckDB extensions
LOAD json;

-- ============================================
-- 1. WORKFLOW DEFINITIONS
-- ============================================

CREATE SEQUENCE IF NOT EXISTS seq_workflows;

CREATE TABLE IF NOT EXISTS workflows (
    id UUID PRIMARY KEY DEFAULT uuid(),
    name VARCHAR NOT NULL,
    description VARCHAR,
    organization_id UUID NOT NULL,
    created_by_user_id UUID NOT NULL,
    status VARCHAR NOT NULL DEFAULT 'draft', -- draft, active, archived, deprecated
    category VARCHAR,
    tags VARCHAR NOT NULL DEFAULT '[]', -- JSON array
    current_version_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_workflows_org ON workflows(organization_id);
CREATE INDEX idx_workflows_status ON workflows(status);
CREATE INDEX idx_workflows_created_by ON workflows(created_by_user_id);
CREATE INDEX idx_workflows_category ON workflows(category);

-- ============================================
-- 2. WORKFLOW VERSIONS
-- ============================================

CREATE TABLE IF NOT EXISTS workflow_versions (
    id UUID PRIMARY KEY DEFAULT uuid(),
    workflow_id UUID NOT NULL,
    version INTEGER NOT NULL,
    definition VARCHAR NOT NULL, -- JSON as VARCHAR for DuckDB
    backend VARCHAR NOT NULL DEFAULT 'rust', -- rust or camunda
    created_by_user_id UUID NOT NULL,
    status VARCHAR NOT NULL DEFAULT 'draft', -- draft, active, deprecated, archived
    deployed_at TIMESTAMPTZ,
    deprecated_at TIMESTAMPTZ,
    archived_at TIMESTAMPTZ,
    change_description VARCHAR,
    simulation_results VARCHAR, -- JSON as VARCHAR
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_workflow_versions_workflow ON workflow_versions(workflow_id);
CREATE INDEX idx_workflow_versions_status ON workflow_versions(status);
CREATE UNIQUE (workflow_id, version);

-- ============================================
-- 3. WORKFLOW TEMPLATES
-- ============================================

CREATE TABLE IF NOT EXISTS workflow_templates (
    id UUID PRIMARY KEY DEFAULT uuid(),
    name VARCHAR NOT NULL,
    description VARCHAR,
    category VARCHAR,
    tags VARCHAR NOT NULL DEFAULT '[]', -- JSON array
    definition VARCHAR NOT NULL, -- JSON as VARCHAR
    is_system_template BOOLEAN NOT NULL DEFAULT false,
    is_public BOOLEAN NOT NULL DEFAULT true,
    created_by_user_id UUID,
    created_by_org_id UUID,
    usage_count INTEGER NOT NULL DEFAULT 0,
    rating_avg FLOAT,
    rating_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_workflow_templates_category ON workflow_templates(category);
CREATE INDEX idx_workflow_templates_public ON workflow_templates(is_public);

-- ============================================
-- 4. WORKFLOW EXECUTIONS
-- ============================================

CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY DEFAULT uuid(),
    workflow_version_id UUID NOT NULL,
    document_id UUID,
    information_domain_id UUID,
    trigger_type VARCHAR NOT NULL,
    trigger_data VARCHAR, -- JSON as VARCHAR
    status VARCHAR NOT NULL DEFAULT 'running', -- running, completed, failed, cancelled, paused
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    error_message VARCHAR,
    error_details VARCHAR, -- JSON as VARCHAR
    current_stage_id VARCHAR,
    execution_metadata VARCHAR, -- JSON as VARCHAR
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_workflow_executions_version ON workflow_executions(workflow_version_id);
CREATE INDEX idx_workflow_executions_document ON workflow_executions(document_id);
CREATE INDEX idx_workflow_executions_status ON workflow_executions(status);
CREATE INDEX idx_workflow_executions_started ON workflow_executions(started_at DESC);

-- ============================================
-- 5. WORKFLOW STAGE EXECUTIONS
-- ============================================

CREATE TABLE IF NOT EXISTS workflow_stage_executions (
    id UUID PRIMARY KEY DEFAULT uuid(),
    workflow_execution_id UUID NOT NULL,
    stage_id VARCHAR NOT NULL,
    stage_name VARCHAR NOT NULL,
    stage_type VARCHAR NOT NULL, -- automated, approval, notification, condition
    status VARCHAR NOT NULL DEFAULT 'pending', -- pending, in_progress, completed, failed, skipped
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    input_data VARCHAR, -- JSON as VARCHAR
    output_data VARCHAR, -- JSON as VARCHAR
    error_message VARCHAR,
    approver_id UUID,
    approver_name VARCHAR,
    decision VARCHAR,
    comment VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_stage_executions_execution ON workflow_stage_executions(workflow_execution_id);
CREATE INDEX idx_stage_executions_status ON workflow_stage_executions(status);
CREATE INDEX idx_stage_executions_stage ON workflow_stage_executions(stage_id);

-- ============================================
-- 6. WORKFLOW SIMULATIONS
-- ============================================

CREATE TABLE IF NOT EXISTS workflow_simulations (
    id UUID PRIMARY KEY DEFAULT uuid(),
    workflow_version_id UUID NOT NULL,
    simulated_by_user_id UUID NOT NULL,
    test_cases VARCHAR NOT NULL, -- JSON array as VARCHAR
    results VARCHAR NOT NULL, -- JSON as VARCHAR
    warnings VARCHAR NOT NULL DEFAULT '[]', -- JSON array as VARCHAR
    passed_count INTEGER NOT NULL DEFAULT 0,
    failed_count INTEGER NOT NULL DEFAULT 0,
    total_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_simulations_version ON workflow_simulations(workflow_version_id);
CREATE INDEX idx_simulations_user ON workflow_simulations(simulated_by_user_id);
CREATE INDEX idx_simulations_created ON workflow_simulations(created_at DESC);

-- ============================================
-- 7. WORKFLOW AI PARSES
-- ============================================

CREATE TABLE IF NOT EXISTS workflow_ai_parses (
    id UUID PRIMARY KEY DEFAULT uuid(),
    user_input VARCHAR NOT NULL,
    parsed_definition VARCHAR, -- JSON as VARCHAR
    confidence FLOAT,
    clarifications VARCHAR, -- JSON as VARCHAR
    user_id UUID NOT NULL,
    organization_id UUID NOT NULL,
    model_used VARCHAR,
    tokens_used INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_ai_parses_user ON workflow_ai_parses(user_id);
CREATE INDEX idx_ai_parses_org ON workflow_ai_parses(organization_id);
CREATE INDEX idx_ai_parses_created ON workflow_ai_parses(created_at DESC);

-- ============================================
-- 8. WORKFLOW PERMISSIONS
-- ============================================

CREATE TABLE IF NOT EXISTS workflow_permissions (
    id UUID PRIMARY KEY DEFAULT uuid(),
    workflow_id UUID NOT NULL,
    user_id UUID,
    role_id UUID,
    permission_type VARCHAR NOT NULL, -- view, edit, deploy, delete, manage
    granted_by_user_id UUID NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_workflow_permissions_workflow ON workflow_permissions(workflow_id);
CREATE INDEX idx_workflow_permissions_user ON workflow_permissions(user_id);
CREATE INDEX idx_workflow_permissions_role ON workflow_permissions(role_id);

-- ============================================
-- 9. WORKFLOW AUDIT LOG
-- ============================================

CREATE TABLE IF NOT EXISTS workflow_audit_log (
    id UUID PRIMARY KEY DEFAULT uuid(),
    workflow_id UUID,
    workflow_version_id UUID,
    workflow_execution_id UUID,
    action VARCHAR NOT NULL,
    actor_user_id UUID NOT NULL,
    actor_type VARCHAR NOT NULL DEFAULT 'user',
    changes VARCHAR, -- JSON as VARCHAR
    old_value VARCHAR, -- JSON as VARCHAR
    new_value VARCHAR, -- JSON as VARCHAR
    ip_address VARCHAR,
    user_agent VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_workflow ON workflow_audit_log(workflow_id);
CREATE INDEX idx_audit_execution ON workflow_audit_log(workflow_execution_id);
CREATE INDEX idx_audit_actor ON workflow_audit_log(actor_user_id);
CREATE INDEX idx_audit_created ON workflow_audit_log(created_at DESC);

-- ============================================
-- 10. HELPER FUNCTIONS
-- ============================================

-- Get next available version number for a workflow
CREATE OR REPLACE FUNCTION next_workflow_version(p_workflow_id UUID)
RETURNS INTEGER AS $$
    SELECT COALESCE(MAX(version), 0) + 1
    FROM workflow_versions
    WHERE workflow_id = p_workflow_id;
$$ LANGUAGE SQL;

-- Check if workflow version can be deployed
CREATE OR REPLACE FUNCTION can_deploy_workflow_version(p_workflow_id UUID, p_version INTEGER)
RETURNS BOOLEAN AS $$
    SELECT NOT EXISTS (
        SELECT 1 FROM workflow_executions we
        INNER JOIN workflow_versions wv ON we.workflow_version_id = wv.id
        WHERE wv.workflow_id = p_workflow_id
          AND wv.version < p_version
          AND we.status IN ('running', 'paused')
    );
$$ LANGUAGE SQL;

-- ============================================
-- 11. VIEWS
-- ============================================

-- Active workflows with latest version info
CREATE VIEW IF NOT EXISTS v_active_workflows AS
SELECT
    w.id AS workflow_id,
    w.name,
    w.description,
    w.organization_id,
    w.status,
    wv.version AS current_version,
    wv.id AS current_version_id,
    wv.backend,
    wv.deployed_at,
    w.created_at,
    w.updated_at,
    u.display_name AS created_by
FROM workflows w
INNER JOIN workflow_versions wv ON w.current_version_id = wv.id
INNER JOIN users u ON w.created_by_user_id = u.id
WHERE w.status = 'active';

-- Workflow execution summary
CREATE VIEW IF NOT EXISTS v_workflow_execution_summary AS
SELECT
    we.id AS execution_id,
    we.workflow_version_id,
    w.name AS workflow_name,
    we.document_id,
    we.status AS execution_status,
    we.started_at,
    we.completed_at,
    EXTRACT(EPOCH FROM (COALESCE(completed_at, now()) - started_at)) AS duration_seconds,
    COUNT(DISTINCT wse.id) AS stages_total,
    SUM(CASE WHEN wse.status = 'completed' THEN 1 ELSE 0 END) AS stages_completed,
    SUM(CASE WHEN wse.status = 'failed' THEN 1 ELSE 0 END) AS stages_failed
FROM workflow_executions we
INNER JOIN workflow_versions wv ON we.workflow_version_id = wv.id
INNER JOIN workflows w ON wv.workflow_id = w.id
LEFT JOIN workflow_stage_executions wse ON we.id = wse.workflow_execution_id
GROUP BY we.id, w.name, we.document_id, we.status, we.started_at, we.completed_at;
