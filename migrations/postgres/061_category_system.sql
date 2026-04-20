-- ============================================================
-- Category System Migration
-- ============================================================
-- Creates tables for hierarchical category classification
-- Supports managed taxonomies with tree structures
--
-- Categories provide structured classification unlike free-form tags
-- Supports parent-child relationships and custom metadata
-- ============================================================

-- Enable required extensions if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================
-- CATEGORIES table
-- ============================================================
-- Core category entity with hierarchical support
CREATE TABLE IF NOT EXISTS categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    description TEXT,
    category_type VARCHAR NOT NULL CHECK (category_type IN (
        'document_type',    -- Document types (besluit, rapport, memo, etc.)
        'policy_area',      -- Beleidsvelden
        'subsidy',          -- Subsidie categorieën
        'permit',           -- Vergunning types
        'woo_category',     -- Woo informatiecategorieën
        'retention_schedule', -- Archiefselectielijst
        'department',       -- Afdelingen
        'project',          -- Project categorieën
        'custom'            -- Extension by organization
    )),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    parent_category_id UUID REFERENCES categories(id) ON DELETE SET NULL,
    level INTEGER DEFAULT 0,  -- Depth in hierarchy (0 = root)
    path VARCHAR,  -- Slash-separated path for hierarchy (e.g., "POL/001/SUB")
    icon VARCHAR,  -- Icon name for UI
    color VARCHAR,  -- Hex color for UI
    sort_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(category_type, code),
    UNIQUE(category_type, path)
);

-- Indexes for categories
CREATE INDEX IF NOT EXISTS idx_categories_type ON categories(category_type);
CREATE INDEX IF NOT EXISTS idx_categories_code ON categories(category_type, code);
CREATE INDEX IF NOT EXISTS idx_categories_path ON categories(category_type, path);
CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_category_id) WHERE parent_category_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_categories_level ON categories(level);
CREATE INDEX IF NOT EXISTS idx_categories_active ON categories(is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_categories_sort ON categories(category_type, sort_order);

-- ============================================================
-- OBJECT_CATEGORIES table
-- ============================================================
-- Links categories to information_objects with primary designation
CREATE TABLE IF NOT EXISTS object_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    object_id UUID NOT NULL REFERENCES information_objects(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    is_primary BOOLEAN DEFAULT FALSE,  -- Primary categorization
    assigned_by UUID NOT NULL,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(object_id, category_id)
);

-- Indexes for object_categories
CREATE INDEX IF NOT EXISTS idx_object_categories_object ON object_categories(object_id);
CREATE INDEX IF NOT EXISTS idx_object_categories_category ON object_categories(category_id);
CREATE INDEX IF NOT EXISTS idx_object_categories_primary ON object_categories(is_primary) WHERE is_primary = TRUE;

-- Ensure only one primary category per object
CREATE UNIQUE INDEX IF NOT EXISTS idx_object_categories_unique_primary
    ON object_categories(object_id)
    WHERE is_primary = TRUE;

-- ============================================================
-- DOMAIN_CATEGORIES table
-- ============================================================
-- Links categories to information_domains
CREATE TABLE IF NOT EXISTS domain_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_id UUID NOT NULL REFERENCES information_domains(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    assigned_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(domain_id, category_id)
);

-- Indexes for domain_categories
CREATE INDEX IF NOT EXISTS idx_domain_categories_domain ON domain_categories(domain_id);
CREATE INDEX IF NOT EXISTS idx_domain_categories_category ON domain_categories(category_id);

-- ============================================================
-- CATEGORY_MIGRATIONS table
-- ============================================================
-- Tracks category taxonomy migrations and reorganizations
CREATE TABLE IF NOT EXISTS category_migrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    source_category_id UUID NOT NULL REFERENCES categories(id) ON DELETE SET NULL,
    target_category_id UUID NOT NULL REFERENCES categories(id) ON DELETE SET NULL,
    migration_strategy VARCHAR NOT NULL CHECK (migration_strategy IN (
        'move',    -- Move all objects to target category
        'copy',    -- Copy to target (both keep assignments)
        'remove',  -- Remove old assignment
        'manual'   -- Manual review required
    )),
    status VARCHAR DEFAULT 'pending' CHECK (status IN (
        'pending', 'in_progress', 'completed', 'failed', 'cancelled'
    )),
    affected_objects_count INTEGER DEFAULT 0,
    migrated_objects_count INTEGER DEFAULT 0,
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for category_migrations
CREATE INDEX IF NOT EXISTS idx_category_migrations_status ON category_migrations(status);
CREATE INDEX IF NOT EXISTS idx_category_migrations_source ON category_migrations(source_category_id);
CREATE INDEX IF NOT EXISTS idx_category_migrations_target ON category_migrations(target_category_id);
CREATE INDEX IF NOT EXISTS idx_category_migrations_created_by ON category_migrations(created_by);

-- ============================================================
-- FUNCTIONS
-- ============================================================

-- Recursive function to build category path
CREATE OR REPLACE FUNCTION build_category_path()
RETURNS TRIGGER AS $$
DECLARE
    parent_path VARCHAR;
BEGIN
    IF NEW.parent_category_id IS NOT NULL THEN
        SELECT path INTO parent_path FROM categories WHERE id = NEW.parent_category_id;
        NEW.path := parent_path || '/' || NEW.code;
        NEW.level := (SELECT level FROM categories WHERE id = NEW.parent_category_id) + 1;
    ELSE
        NEW.path := NEW.code;
        NEW.level := 0;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function to get category tree for UI
CREATE OR REPLACE FUNCTION get_category_tree(p_category_type VARCHAR)
RETURNS TABLE (
    id UUID,
    code VARCHAR,
    name VARCHAR,
    level INTEGER,
    path VARCHAR,
    parent_id UUID,
    has_children BOOLEAN
) AS $$
WITH RECURSIVE category_tree AS (
    -- Root nodes
    SELECT
        c.id,
        c.code,
        c.name,
        c.level,
        c.path,
        c.parent_category_id,
        FALSE::BOOLEAN as has_children
    FROM categories c
    WHERE c.category_type = get_category_tree.p_category_type
      AND c.parent_category_id IS NULL
      AND c.is_active = TRUE

    UNION ALL

    -- Child nodes
    SELECT
        c.id,
        c.code,
        c.name,
        c.level,
        c.path,
        c.parent_category_id,
        TRUE::BOOLEAN as has_children
    FROM categories c
    INNER JOIN category_tree ct ON c.parent_category_id = ct.id
    WHERE c.is_active = TRUE
)
SELECT * FROM category_tree
ORDER BY path;
$$ LANGUAGE plpgsql;

-- Function to count objects in category and all children
CREATE OR REPLACE FUNCTION count_category_objects(p_category_id UUID)
RETURNS INTEGER AS $$
DECLARE
    total_count INTEGER := 0;
BEGIN
    WITH RECURSIVE category_descendants AS (
        -- Start with the category itself
        SELECT id FROM categories WHERE id = p_category_id
        UNION ALL
        -- Add all children recursively
        SELECT c.id
        FROM categories c
        INNER JOIN category_descendants cd ON c.parent_category_id = cd.id
    )
    SELECT COUNT(*) INTO total_count
    FROM object_categories oc
    WHERE oc.category_id IN (SELECT id FROM category_descendants);

    RETURN total_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- TRIGGERS
-- ============================================================

-- Auto-build path and level on insert/update
DROP TRIGGER IF EXISTS trigger_build_category_path ON categories;
CREATE TRIGGER trigger_build_category_path
    BEFORE INSERT OR UPDATE OF parent_category_id, code
    ON categories
    FOR EACH ROW
    EXECUTE FUNCTION build_category_path();

-- Auto-update updated_at on categories
DROP TRIGGER IF EXISTS trigger_update_categories_updated_at ON categories;
CREATE TRIGGER trigger_update_categories_updated_at
    BEFORE UPDATE ON categories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================
-- VIEWS
-- ============================================================

-- View: Category statistics with object counts
CREATE OR REPLACE VIEW v_category_stats AS
SELECT
    c.id,
    c.category_type,
    c.code,
    c.name,
    c.level,
    c.path,
    c.parent_category_id,
    COUNT(DISTINCT oc.object_id) as direct_objects_count,
    count_category_objects(c.id) as total_objects_in_subtree,
    COUNT(DISTINCT child.id) as child_count,
    MAX(oc.assigned_at) as last_used
FROM categories c
LEFT JOIN object_categories oc ON c.id = oc.category_id
LEFT JOIN categories child ON c.parent_category_id = c.id
WHERE c.is_active = TRUE
GROUP BY c.id, c.category_type, c.code, c.name, c.level, c.path, c.parent_category_id;

-- View: Active categories by type for UI dropdowns
CREATE OR REPLACE VIEW v_active_categories AS
SELECT
    c.*,
    COALESCE(stats.direct_objects_count, 0) as object_count
FROM categories c
LEFT JOIN v_category_stats stats ON c.id = stats.id
WHERE c.is_active = TRUE
ORDER BY c.category_type, c.sort_order, c.path;

-- ============================================================
-- INITIAL DATA
-- ============================================================

-- Insert common document types
INSERT INTO categories (code, name, category_type, description, sort_order) VALUES
    ('BESLUIT', 'Besluit', 'document_type', 'Formeel besluit', 1),
    ('RAPP', 'Rapport', 'document_type', 'Rapport of notitie', 2),
    ('MEMO', 'Memo', 'document_type', 'Interne memo', 3),
    ('BRIEF', 'Brief', 'document_type', 'Inkomende/uitgaande brief', 4),
    ('EMAIL', 'Email', 'document_type', 'Email correspondentie', 5),
    ('NOTU', 'Notitie', 'document_type', 'Notitie van vergadering', 6),
    ('CONC', 'Concept', 'document_type', 'Concept document', 7),
    ('FORM', 'Formulier', 'document_type', 'Ingevuld formulier', 8),
    ('DATA', 'Dataset', 'document_type', 'Gestructureerde data', 9),
    ('OVERG', 'Overig', 'document_type', 'Overig document', 10)
ON CONFLICT (category_type, code) DO NOTHING;

-- Insert Woo categories (per Woo Besluit)
INSERT INTO categories (code, name, category_type, description, sort_order) VALUES
    ('WOO-INFO', 'Informatie- en inspraakcategorie', 'woo_category', 'Informatie verstrekt op eigen initiatief', 1),
    ('WOO-BESL', 'Besluit', 'woo_category', 'Besluit op Woo verzoek', 2),
    ('WOO-VERLE', 'Verlenging', 'woo_category', 'Verlenging besluittermijn', 3),
    ('WOO-ZIEK', 'Ziekmelding', 'woo_category', 'Ziekmelding ambtenaar', 4)
ON CONFLICT (category_type, code) DO NOTHING;

-- Insert policy areas
INSERT INTO categories (code, name, category_type, description, sort_order) VALUES
    ('POL-RO', 'Ruimtelijke Ordening', 'policy_area', 'Ruimtelijke ordening en vergunningen', 1),
    ('POL-ECO', 'Economie', 'policy_area', 'Economische zaken', 2),
    ('POL-SPO', 'Sport', 'policy_area', 'Sport en recreatie', 3),
    ('POL-CUL', 'Cultuur', 'policy_area', 'Cultuur en erfgoed', 4),
    ('POL-MIL', 'Milieu', 'policy_area', 'Milieu en duurzaamheid', 5),
    ('POL-VRK', 'Verkeer', 'policy_area', 'Verkeer en vervoer', 6),
    ('POL-SOC', 'Sociale Zaken', 'policy_area', 'Welzijn en zorg', 7),
    ('POL-OND', 'Onderwijs', 'policy_area', 'Onderwijs en jeugd', 8),
    ('POL-FIN', 'Financiën', 'policy_area', 'Financiën en belastingen', 9),
    ('POL-VEIL', 'Openbare Orde', 'policy_area', 'Veiligheid', 10)
ON CONFLICT (category_type, code) DO NOTHING;
