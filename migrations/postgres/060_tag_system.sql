-- ============================================================
-- Tag System Migration
-- ============================================================
-- Creates tables for flexible tag-based classification
-- Supports free tags, controlled vocabularies, and usage tracking
--
-- Tags are separate from the TEXT[] tags on information_objects
-- to enable proper relationships, statistics, and management
-- ============================================================

-- Enable required extensions if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================
-- TAGS table
-- ============================================================
-- Core tag entity with type, organization scope, and usage tracking
CREATE TABLE IF NOT EXISTS tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    normalized_name VARCHAR NOT NULL UNIQUE,
    tag_type VARCHAR NOT NULL CHECK (tag_type IN (
        'free',           -- User-created free tags
        'controlled',     -- Managed vocabulary
        'policy',         -- Policy agenda tags
        'subsidy',        -- Subsidie categories
        'permit',         -- Vergunning types
        'department',     -- Department tags
        'project_phase',  -- Project phases
        'case_status',    -- Zaak statussen
        'custom'          -- Extension by organization
    )),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    description TEXT,
    color VARCHAR,  -- Hex color for UI
    parent_tag_id UUID REFERENCES tags(id) ON DELETE SET NULL,
    usage_count INTEGER DEFAULT 0,
    is_system_tag BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for tags
CREATE INDEX IF NOT EXISTS idx_tags_normalized_name ON tags(normalized_name);
CREATE INDEX IF NOT EXISTS idx_tags_type ON tags(tag_type);
CREATE INDEX IF NOT EXISTS idx_tags_organization ON tags(organization_id) WHERE organization_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tags_parent ON tags(parent_tag_id) WHERE parent_tag_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tags_usage ON tags(usage_count DESC);
CREATE INDEX IF NOT EXISTS idx_tags_system ON tags(is_system_tag) WHERE is_system_tag = TRUE;

-- Full-text search on tag name
CREATE INDEX IF NOT EXISTS idx_tags_name_fts ON tags USING gin(to_tsvector('dutch', name));

-- ============================================================
-- OBJECT_TAGS table
-- ============================================================
-- Links tags to information_objects with confidence and auto-assign tracking
CREATE TABLE IF NOT EXISTS object_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    object_id UUID NOT NULL REFERENCES information_objects(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    tagged_by UUID NOT NULL,  -- User ID who tagged
    confidence DECIMAL(3,2) DEFAULT 1.00 CHECK (confidence BETWEEN 0 AND 1),
    is_auto_assigned BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(object_id, tag_id)
);

-- Indexes for object_tags
CREATE INDEX IF NOT EXISTS idx_object_tags_object ON object_tags(object_id);
CREATE INDEX IF NOT EXISTS idx_object_tags_tag ON object_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_object_tags_tagged_by ON object_tags(tagged_by);
CREATE INDEX IF NOT EXISTS idx_object_tags_auto ON object_tags(is_auto_assigned) WHERE is_auto_assigned = TRUE;

-- ============================================================
-- DOMAIN_TAGS table
-- ============================================================
-- Links tags to information_domains
CREATE TABLE IF NOT EXISTS domain_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_id UUID NOT NULL REFERENCES information_domains(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    tagged_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(domain_id, tag_id)
);

-- Indexes for domain_tags
CREATE INDEX IF NOT EXISTS idx_domain_tags_domain ON domain_tags(domain_id);
CREATE INDEX IF NOT EXISTS idx_domain_tags_tag ON domain_tags(tag_id);

-- ============================================================
-- FUNCTIONS
-- ============================================================

-- Function to increment tag usage count
CREATE OR REPLACE FUNCTION increment_tag_usage()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE tags SET usage_count = usage_count + 1, updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.tag_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function to decrement tag usage count
CREATE OR REPLACE FUNCTION decrement_tag_usage()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE tags SET usage_count = GREATEST(usage_count - 1, 0), updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.tag_id;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- Function to normalize tag name (lowercase, spaces to dashes)
CREATE OR REPLACE FUNCTION normalize_tag_name(tag_name TEXT)
RETURNS VARCHAR AS $$
BEGIN
    RETURN regexp_replace(
        lower(tag_name),
        '[^a-z0-9]+',
        '-',
        'g'
    );
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- TRIGGERS
-- ============================================================

-- Auto-update usage_count on object_tags insert
DROP TRIGGER IF EXISTS trigger_increment_tag_usage ON object_tags;
CREATE TRIGGER trigger_increment_tag_usage
    AFTER INSERT ON object_tags
    FOR EACH ROW
    EXECUTE FUNCTION increment_tag_usage();

-- Auto-update usage_count on object_tags delete
DROP TRIGGER IF EXISTS trigger_decrement_tag_usage ON object_tags;
CREATE TRIGGER trigger_decrement_tag_usage
    AFTER DELETE ON object_tags
    FOR EACH ROW
    EXECUTE FUNCTION decrement_tag_usage();

-- Auto-normalize tag name on insert
DROP TRIGGER IF EXISTS trigger_normalize_tag_name ON tags;
CREATE TRIGGER trigger_normalize_tag_name
    BEFORE INSERT ON tags
    FOR EACH ROW
    EXECUTE FUNCTION
    BEGIN
        IF NEW.normalized_name IS NULL THEN
            NEW.normalized_name := normalize_tag_name(NEW.name);
        END IF;
    END;
$$ LANGUAGE plpgsql;

-- Auto-update updated_at on tags
DROP TRIGGER IF EXISTS trigger_update_tags_updated_at ON tags;
CREATE TRIGGER trigger_update_tags_updated_at
    BEFORE UPDATE ON tags
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================
-- VIEWS
-- ============================================================

-- View: Tag statistics with usage breakdown
CREATE OR REPLACE VIEW v_tag_stats AS
SELECT
    t.id,
    t.name,
    t.normalized_name,
    t.tag_type,
    t.usage_count,
    t.organization_id,
    COUNT(DISTINCT ot.object_id) as objects_count,
    COUNT(DISTINCT dt.domain_id) as domains_count,
    MAX(ot.created_at) as last_used
FROM tags t
LEFT JOIN object_tags ot ON t.id = ot.tag_id
LEFT JOIN domain_tags dt ON t.id = dt.tag_id
GROUP BY t.id, t.name, t.normalized_name, t.tag_type, t.usage_count, t.organization_id;

-- View: Popular tags for autocomplete/suggestions
CREATE OR REPLACE VIEW v_popular_tags AS
SELECT * FROM v_tag_stats
WHERE usage_count > 0
ORDER BY usage_count DESC, last_used DESC
LIMIT 1000;

-- View: Tags by organization
CREATE OR REPLACE VIEW v_organization_tags AS
SELECT
    o.id as organization_id,
    o.name as organization_name,
    t.id as tag_id,
    t.name as tag_name,
    t.tag_type,
    t.usage_count
FROM organizations o
LEFT JOIN tags t ON t.organization_id = o.id OR t.organization_id IS NULL
WHERE t.is_active IS NOT FALSE  -- Exclude explicitly inactive tags if we add that field
ORDER BY o.name, t.tag_type, t.name;

-- ============================================================
-- INITIAL DATA
-- ============================================================

-- Insert common system tags for Dutch government
INSERT INTO tags (name, tag_type, description, is_system_tag) VALUES
    -- Policy areas
    ('Ruimtelijke Ordening', 'policy', 'Ruimtelijke ordening en vergunningen', TRUE),
    ('Economie', 'policy', 'Economische zaken en werkgelegenheid', TRUE),
    ('Sport', 'policy', 'Sport en recreatie', TRUE),
    ('Cultuur', 'policy', 'Cultuur en erfgoed', TRUE),
    ('Milieu', 'policy', 'Milieu en duurzaamheid', TRUE),
    ('Verkeer', 'policy', 'Verkeer en vervoer', TRUE),
    ('Sociale Zaken', 'policy', 'Welzijn en zorg', TRUE),
    ('Onderwijs', 'policy', 'Onderwijs en jeugd', TRUE),
    ('Financiën', 'policy', 'Financiën en belastingen', TRUE),
    ('Openbare Orde', 'policy', 'Veiligheid en openbare orde', TRUE),
    -- Subsidie types
    ('Kennisontwikkeling', 'subsidy', 'Subsidie voor kennisontwikkeling', TRUE),
    ('Infrastructuur', 'subsidy', 'Subsidie voor infrastructuurprojecten', TRUE),
    ('Evenementen', 'subsidy', 'Evenementensubsidie', TRUE),
    -- Project phases
    ('Initiatief', 'project_phase', 'Projectinitiatief fase', TRUE),
    ('Voorbereiding', 'project_phase', 'Voorbereidingsfase', TRUE),
    ('Uitvoering', 'project_phase', 'Uitvoeringsfase', TRUE),
    ('Evaluatie', 'project_phase', 'Evaluatiefase', TRUE),
    ('Afgerond', 'project_phase', 'Project afgerond', TRUE)
ON CONFLICT (normalized_name) DO NOTHING;
