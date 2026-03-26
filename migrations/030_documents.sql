-- Migration: Document metadata schema
-- Version: 030
-- Description: Creates tables for document creation agents system

-- Documents table
CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY,
    domain_id VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    state VARCHAR NOT NULL,  -- Uses WorkflowStatus values: Draft, Submitted, Approved, Rejected, Published
    trust_level VARCHAR NOT NULL,  -- Low, Medium, High

    -- Storage references
    current_version_key VARCHAR NOT NULL,
    previous_version_key VARCHAR,

    -- Scores
    compliance_score FLOAT NOT NULL DEFAULT 0.0,
    confidence_score FLOAT NOT NULL DEFAULT 0.0,

    -- Request context (JSON)
    request_context JSON,

    -- Audit timestamps
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    published_at TIMESTAMP,

    -- Approval information
    approved_by VARCHAR,
    approval_notes TEXT
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_documents_domain ON documents(domain_id);
CREATE INDEX IF NOT EXISTS idx_documents_state ON documents(state);
CREATE INDEX IF NOT EXISTS idx_documents_domain_state ON documents(domain_id, state);
CREATE INDEX IF NOT EXISTS idx_documents_created ON documents(created_at DESC);

-- Audit trail table
CREATE TABLE IF NOT EXISTS document_audit (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    agent_name VARCHAR NOT NULL,
    action VARCHAR NOT NULL,
    details JSON,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    execution_time_ms INTEGER
);

CREATE INDEX IF NOT EXISTS idx_audit_document ON document_audit(document_id);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON document_audit(timestamp DESC);

-- Document versions table for full history and rollback support
CREATE TABLE IF NOT EXISTS document_versions (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    storage_key VARCHAR NOT NULL,
    format VARCHAR NOT NULL,  -- Markdown, ODF, PDF
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_by VARCHAR,  -- Agent name or User ID
    change_summary TEXT,
    is_current BOOLEAN NOT NULL DEFAULT FALSE,
    compliance_score FLOAT,
    UNIQUE(document_id, version_number)
);

CREATE INDEX IF NOT EXISTS idx_versions_document ON document_versions(document_id);
CREATE INDEX IF NOT EXISTS idx_versions_current ON document_versions(document_id, is_current);

-- Templates table
CREATE TABLE IF NOT EXISTS templates (
    id UUID PRIMARY KEY,
    name VARCHAR NOT NULL,
    domain_id VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    content TEXT NOT NULL,
    required_variables JSON,  -- Array of strings
    optional_sections JSON,    -- Array of strings
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_templates_domain_type
ON templates(domain_id, document_type)
WHERE is_active = TRUE;

-- Domain configuration table
CREATE TABLE IF NOT EXISTS domain_configs (
    domain_id VARCHAR PRIMARY KEY,
    trust_level VARCHAR NOT NULL,  -- Low, Medium, High
    required_approval_threshold FLOAT DEFAULT 0.8,
    auto_approval_threshold FLOAT DEFAULT 0.95,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Insert default domain configurations
INSERT INTO domain_configs (domain_id, trust_level, required_approval_threshold, auto_approval_threshold)
VALUES
    ('default', 'Low', 0.8, 0.95)
ON CONFLICT (domain_id) DO NOTHING;
