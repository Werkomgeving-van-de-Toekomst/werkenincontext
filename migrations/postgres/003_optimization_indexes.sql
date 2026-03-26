-- Performance optimization indexes
-- Section 05: Stabilization
--
-- This migration first adds missing columns needed for RLS and features,
-- then creates indexes based on actual query patterns observed in production.

-- ============================================================
-- Add missing columns for RLS and features
-- ============================================================

-- Add organization_id to documents table (needed for RLS policies)
ALTER TABLE documents ADD COLUMN IF NOT EXISTS organization_id UUID;
ALTER TABLE documents ADD COLUMN IF NOT EXISTS owner_id UUID;
ALTER TABLE documents ADD COLUMN IF NOT EXISTS classification VARCHAR DEFAULT 'intern' CHECK (classification IN ('openbaar', 'intern', 'vertrouwelijk', 'geheim'));
ALTER TABLE documents ADD COLUMN IF NOT EXISTS woo_published BOOLEAN DEFAULT FALSE;
ALTER TABLE documents ADD COLUMN IF NOT EXISTS woo_published_at TIMESTAMP WITH TIME ZONE;

-- Add organization_id to templates table (needed for RLS policies)
ALTER TABLE templates ADD COLUMN IF NOT EXISTS organization_id UUID;

-- Update existing documents to have domain_id copied to organization_id for migration
-- This is a one-time data migration for existing records
UPDATE documents SET organization_id = domain_id::UUID WHERE organization_id IS NULL;

-- ============================================================
-- Performance optimization indexes
-- ============================================================

-- Index for common document queries
-- Covers the pattern: WHERE organization_id = ? AND state NOT IN ('archived', 'rejected')
CREATE INDEX IF NOT EXISTS idx_documents_org_state
ON documents(organization_id, state)
WHERE state NOT IN ('archived', 'rejected');

-- Index for full-text search on information_objects
-- Uses GIN index for efficient tsvector searches
CREATE INDEX IF NOT EXISTS idx_information_objects_fts
ON information_objects USING gin(to_tsvector('dutch', coalesce(title, '') || ' ' || coalesce(content_text, '')));

-- Index for audit trail queries by document and timestamp
-- Common pattern: show recent activity for a document
CREATE INDEX IF NOT EXISTS idx_audit_trail_document_timestamp
ON audit_trail(document_id, timestamp DESC);

-- Partial index for Woo-relevant objects met publicatiedatum (zonder CURRENT_TIMESTAMP:
-- Postgres vereist IMMUTABLE predicaten; tijdsfilter hoort in de query zelf)
CREATE INDEX IF NOT EXISTS idx_information_objects_woo_public
ON information_objects(woo_publication_date)
WHERE is_woo_relevant = true
  AND woo_publication_date IS NOT NULL;

-- Index for information objects by domain with classification filter
-- Optimizes queries that filter by domain and classification
CREATE INDEX IF NOT EXISTS idx_information_objects_domain_classification
ON information_objects(domain_id, classification)
WHERE classification != 'geheim';

-- Index for document state transitions
-- Optimizes queries that track workflow state
CREATE INDEX IF NOT EXISTS idx_documents_state_updated
ON documents(state, updated_at DESC)
WHERE state NOT IN ('archived', 'rejected');

-- Composite index for domain queries with type and status
CREATE INDEX IF NOT EXISTS idx_information_domains_type_status
ON information_domains(domain_type, status, organization_id);

-- Recente objecten: geen NOW() in partial index (niet IMMUTABLE).
-- B-tree op created_at volstaat voor ORDER BY created_at DESC / range filters.
CREATE INDEX IF NOT EXISTS idx_information_objects_recent
ON information_objects(created_at DESC);

-- Index for Woo published documents
CREATE INDEX IF NOT EXISTS idx_documents_woo_published
ON documents(woo_published, organization_id)
WHERE woo_published = true;

-- Index for document classification with RLS
CREATE INDEX IF NOT EXISTS idx_documents_classification
ON documents(organization_id, classification)
WHERE classification IS NOT NULL;
