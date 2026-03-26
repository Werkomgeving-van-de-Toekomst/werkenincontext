-- Document Creation Agent Tables
-- Adds tables for the multi-agent document creation system

-- Documents table for tracking document generation through the agent pipeline
CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY,
    domain_id VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    state VARCHAR NOT NULL DEFAULT 'draft', -- draft, submitted, in_review, approved, published, rejected, archived
    current_version_key VARCHAR NOT NULL DEFAULT '',
    previous_version_key VARCHAR,
    compliance_score REAL NOT NULL DEFAULT 0.0,
    confidence_score REAL NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Templates table for document templates
CREATE TABLE IF NOT EXISTS templates (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL,
    domain_id VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    content TEXT NOT NULL,
    required_variables VARCHAR[] NOT NULL DEFAULT [],
    optional_sections VARCHAR[] NOT NULL DEFAULT [],
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_active BOOLEAN NOT NULL DEFAULT true
);

-- Audit trail for document creation pipeline
CREATE TABLE IF NOT EXISTS audit_trail (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL,
    agent_name VARCHAR NOT NULL,
    action VARCHAR NOT NULL,
    details JSON NOT NULL DEFAULT '{}',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
    execution_time_ms INTEGER
);

-- Domain configurations for trust levels
CREATE TABLE IF NOT EXISTS domain_configs (
    domain_id VARCHAR PRIMARY KEY,
    trust_level VARCHAR NOT NULL DEFAULT 'medium', -- low, medium, high
    required_approval_threshold REAL NOT NULL DEFAULT 0.8,
    auto_approval_threshold REAL NOT NULL DEFAULT 0.95
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_documents_domain_id ON documents(domain_id);
CREATE INDEX IF NOT EXISTS idx_documents_state ON documents(state);
CREATE INDEX IF NOT EXISTS idx_templates_domain_type ON templates(domain_id, document_type);
CREATE INDEX IF NOT EXISTS idx_audit_trail_document_id ON audit_trail(document_id);
