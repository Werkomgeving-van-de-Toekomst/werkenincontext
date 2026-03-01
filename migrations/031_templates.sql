-- Migration: Template storage for document generation
-- Version: 031
-- Description: Creates templates table for dynamic template management

CREATE TABLE IF NOT EXISTS templates (
    id UUID PRIMARY KEY,
    name VARCHAR NOT NULL,
    domain_id VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    content TEXT NOT NULL,
    required_variables JSON,
    optional_sections JSON,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_templates_domain_type
ON templates(domain_id, document_type)
WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_templates_domain ON templates(domain_id);
CREATE INDEX IF NOT EXISTS idx_templates_active ON templates(is_active);

-- Note: Initial templates will be loaded programmatically
-- or can be inserted here with hardcoded content
