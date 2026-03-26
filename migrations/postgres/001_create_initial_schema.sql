-- PostgreSQL Schema for IOU-Modern
-- This migration creates the PostgreSQL schema equivalent to DuckDB
-- for the hybrid Supabase + DuckDB architecture.

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- ============================================================
-- INFORMATION_DOMAINS table
-- ============================================================
CREATE TABLE IF NOT EXISTS information_domains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_type VARCHAR NOT NULL CHECK (domain_type IN ('Zaak', 'Project', 'Beleid', 'Expertise')),
    name VARCHAR NOT NULL,
    description TEXT,
    status VARCHAR NOT NULL DEFAULT 'actief' CHECK (status IN ('concept', 'actief', 'afgerond', 'gearchiveerd')),
    organization_id UUID NOT NULL,
    owner_user_id UUID,
    parent_domain_id UUID REFERENCES information_domains(id) ON DELETE SET NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for information_domains
CREATE INDEX IF NOT EXISTS idx_information_domains_type ON information_domains(domain_type);
CREATE INDEX IF NOT EXISTS idx_information_domains_org ON information_domains(organization_id);
CREATE INDEX IF NOT EXISTS idx_information_domains_parent ON information_domains(parent_domain_id);
CREATE INDEX IF NOT EXISTS idx_information_domains_status ON information_domains(status);

-- ============================================================
-- INFORMATION_OBJECTS table
-- ============================================================
CREATE TABLE IF NOT EXISTS information_objects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_id UUID NOT NULL REFERENCES information_domains(id) ON DELETE CASCADE,
    object_type VARCHAR NOT NULL CHECK (object_type IN ('document', 'email', 'chat', 'besluit', 'data')),
    title VARCHAR NOT NULL,
    description TEXT,
    content_location TEXT,
    content_text TEXT,
    mime_type VARCHAR,
    file_size BIGINT,
    classification VARCHAR DEFAULT 'intern' CHECK (classification IN ('openbaar', 'intern', 'vertrouwelijk', 'geheim')),
    retention_period VARCHAR,
    is_woo_relevant BOOLEAN DEFAULT FALSE,
    woo_publication_date TIMESTAMP WITH TIME ZONE,
    privacy_level VARCHAR DEFAULT 'normaal' CHECK (privacy_level IN ('openbaar', 'normaal', 'bijzonder', 'gevoelig')),
    tags TEXT[] DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    version INTEGER DEFAULT 1,
    previous_version_id UUID REFERENCES information_objects(id) ON DELETE SET NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for information_objects
CREATE INDEX IF NOT EXISTS idx_information_objects_domain ON information_objects(domain_id);
CREATE INDEX IF NOT EXISTS idx_information_objects_type ON information_objects(object_type);
CREATE INDEX IF NOT EXISTS idx_information_objects_title ON information_objects(title);
CREATE INDEX IF NOT EXISTS idx_information_objects_classification ON information_objects(classification);
CREATE INDEX IF NOT EXISTS idx_information_objects_created_at ON information_objects(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_information_objects_tags ON information_objects USING gin(tags);
CREATE INDEX IF NOT EXISTS idx_information_objects_metadata ON information_objects USING gin(metadata);

-- ============================================================
-- DOCUMENTS table (document creation workflow)
-- ============================================================
CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_id VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    state VARCHAR NOT NULL DEFAULT 'draft' CHECK (state IN ('draft', 'submitted', 'in_review', 'changes_requested', 'approved', 'published', 'rejected', 'archived')),
    current_version_key TEXT,
    previous_version_key TEXT,
    compliance_score DECIMAL(3, 2),
    confidence_score DECIMAL(3, 2),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for documents
CREATE INDEX IF NOT EXISTS idx_documents_domain ON documents(domain_id);
CREATE INDEX IF NOT EXISTS idx_documents_state ON documents(state);
CREATE INDEX IF NOT EXISTS idx_documents_type ON documents(document_type);

-- ============================================================
-- TEMPLATES table
-- ============================================================
CREATE TABLE IF NOT EXISTS templates (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL,
    domain_id VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    content TEXT NOT NULL,
    required_variables JSONB DEFAULT '{}',
    optional_sections JSONB DEFAULT '{}',
    version INTEGER DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE
);

-- Indexes for templates
CREATE INDEX IF NOT EXISTS idx_templates_domain ON templates(domain_id);
CREATE INDEX IF NOT EXISTS idx_templates_type ON templates(document_type);
CREATE INDEX IF NOT EXISTS idx_templates_active ON templates(is_active) WHERE is_active = TRUE;

-- ============================================================
-- AUDIT_TRAIL table
-- ============================================================
CREATE TABLE IF NOT EXISTS audit_trail (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    agent_name VARCHAR NOT NULL,
    action VARCHAR NOT NULL,
    details JSONB DEFAULT '{}',
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    execution_time_ms INTEGER
);

-- Indexes for audit_trail
CREATE INDEX IF NOT EXISTS idx_audit_trail_document ON audit_trail(document_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_timestamp ON audit_trail(timestamp DESC);

-- ============================================================
-- VIEWS for search and analytics
-- ============================================================

-- View: searchable_objects - Aggregates searchable text
CREATE OR REPLACE VIEW v_searchable_objects AS
SELECT
    io.id,
    io.object_type,
    io.title,
    io.description,
    io.content_text,
    io.domain_id,
    io.classification,
    io.created_at,
    io.is_woo_relevant,
    -- Concatenate searchable fields for full-text search
    CONCAT_WS(' ',
        COALESCE(io.title, ''),
        COALESCE(io.description, ''),
        COALESCE(io.content_text, '')
    ) as searchable_text
FROM information_objects io;

-- View: compliance_overview - Analytics for compliance metrics
CREATE OR REPLACE VIEW v_compliance_overview AS
SELECT
    id.id as domain_id,
    id.name as domain_name,
    id.domain_type,
    COUNT(io.id) as total_objects,
    COUNT(CASE WHEN io.is_woo_relevant = TRUE THEN 1 END) as woo_relevant_count,
    COUNT(CASE WHEN io.classification = 'openbaar' THEN 1 END) as public_count,
    COUNT(CASE WHEN io.retention_period IS NULL THEN 1 END) as missing_retention
FROM information_domains id
LEFT JOIN information_objects io ON io.domain_id = id.id
GROUP BY id.id, id.name, id.domain_type;

-- View: domain_statistics - Domain type/status distribution
CREATE OR REPLACE VIEW v_domain_statistics AS
SELECT
    domain_type,
    COUNT(*) as count
FROM information_domains
GROUP BY domain_type;

-- View: entity_network - GraphRAG entity relationships
CREATE OR REPLACE VIEW v_entity_network AS
SELECT
    io.id,
    io.title,
    io.metadata->>'entities' as entities
FROM information_objects io
WHERE io.metadata ? 'entities';

-- ============================================================
-- FUNCTIONS and TRIGGERS
-- ============================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for updated_at
DROP TRIGGER IF EXISTS update_information_domains_updated_at ON information_domains;
CREATE TRIGGER update_information_domains_updated_at
    BEFORE UPDATE ON information_domains
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_information_objects_updated_at ON information_objects;
CREATE TRIGGER update_information_objects_updated_at
    BEFORE UPDATE ON information_objects
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_documents_updated_at ON documents;
CREATE TRIGGER update_documents_updated_at
    BEFORE UPDATE ON documents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_templates_updated_at ON templates;
CREATE TRIGGER update_templates_updated_at
    BEFORE UPDATE ON templates
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================
-- DUAL-WRITE SUPPORT
-- ============================================================

-- Function to check data consistency between PostgreSQL and DuckDB (manual)
-- This will be used by the consistency checker
CREATE OR REPLACE FUNCTION check_record_exists(table_name TEXT, record_id UUID)
RETURNS BOOLEAN AS $$
BEGIN
    CASE table_name
        WHEN 'information_domains' THEN
            RETURN EXISTS(SELECT 1 FROM information_domains WHERE id = record_id);
        WHEN 'information_objects' THEN
            RETURN EXISTS(SELECT 1 FROM information_objects WHERE id = record_id);
        WHEN 'documents' THEN
            RETURN EXISTS(SELECT 1 FROM documents WHERE id = record_id);
        ELSE
            RETURN FALSE;
    END CASE;
END;
$$ LANGUAGE plpgsql;

-- Grant necessary permissions (adjust for your setup)
-- GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO postgres;
-- GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO postgres;
