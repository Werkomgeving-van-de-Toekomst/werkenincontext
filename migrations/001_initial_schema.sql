-- IOU-Modern Database Schema for DuckDB
-- Embedded analytical database - geen server nodig!
--
-- Voordelen DuckDB:
-- - Embedded: geen aparte database server
-- - Analytisch geoptimaliseerd (columnar storage)
-- - Full SQL support inclusief window functions
-- - Goede Rust bindings (duckdb-rs)
-- - Kan direct Parquet/CSV/JSON lezen

-- ============================================
-- 1. ORGANISATIE STRUCTUUR
-- ============================================

CREATE SEQUENCE IF NOT EXISTS seq_organizations;

CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT uuid(),
    name VARCHAR NOT NULL,
    short_name VARCHAR,
    organization_type VARCHAR NOT NULL, -- 'rijk', 'provincie', 'gemeente', 'waterschap', 'zbo'
    parent_organization_id UUID,
    website VARCHAR,
    logo_url VARCHAR,
    primary_color VARCHAR(7),
    secondary_color VARCHAR(7),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE departments (
    id UUID PRIMARY KEY DEFAULT uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR NOT NULL,
    code VARCHAR,
    parent_department_id UUID,
    manager_user_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid(),
    organization_id UUID NOT NULL,
    email VARCHAR UNIQUE NOT NULL,
    display_name VARCHAR NOT NULL,
    first_name VARCHAR,
    last_name VARCHAR,
    department_id UUID,
    job_title VARCHAR,
    phone VARCHAR,
    avatar_url VARCHAR,
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_login TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR NOT NULL,
    description VARCHAR,
    permissions JSON NOT NULL DEFAULT '{}',
    is_system_role BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE user_roles (
    id UUID PRIMARY KEY DEFAULT uuid(),
    user_id UUID NOT NULL,
    role_id UUID NOT NULL,
    scope_domain_id UUID,
    valid_from DATE NOT NULL DEFAULT current_date,
    valid_until DATE,
    assigned_by UUID NOT NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================
-- 2. INFORMATIEDOMEINEN (Context)
-- ============================================

CREATE TABLE information_domains (
    id UUID PRIMARY KEY DEFAULT uuid(),
    domain_type VARCHAR NOT NULL, -- 'zaak', 'project', 'beleid', 'expertise'
    name VARCHAR NOT NULL,
    description VARCHAR,
    status VARCHAR NOT NULL DEFAULT 'actief', -- 'concept', 'actief', 'afgerond', 'gearchiveerd'
    organization_id UUID NOT NULL,
    owner_user_id UUID,
    parent_domain_id UUID,
    metadata JSON NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Zaken (Cases)
CREATE TABLE cases (
    id UUID PRIMARY KEY,
    case_number VARCHAR UNIQUE NOT NULL,
    case_type VARCHAR NOT NULL,
    subject VARCHAR NOT NULL,
    start_date DATE NOT NULL,
    target_date DATE,
    end_date DATE,
    legal_basis VARCHAR,
    retention_period INTEGER,
    disclosure_class VARCHAR
);

-- Projecten
CREATE TABLE projects (
    id UUID PRIMARY KEY,
    project_code VARCHAR UNIQUE NOT NULL,
    project_name VARCHAR NOT NULL,
    start_date DATE,
    planned_end_date DATE,
    actual_end_date DATE,
    budget DECIMAL(12,2),
    project_manager_id UUID
);

-- Beleidsonderwerpen
CREATE TABLE policy_topics (
    id UUID PRIMARY KEY,
    policy_area VARCHAR NOT NULL,
    policy_phase VARCHAR NOT NULL, -- 'agendavorming', 'voorbereiding', 'besluitvorming', 'uitvoering', 'evaluatie'
    responsible_department_id UUID,
    review_date DATE
);

-- ============================================
-- 3. INFORMATIE OBJECTEN
-- ============================================

CREATE TABLE information_objects (
    id UUID PRIMARY KEY DEFAULT uuid(),
    domain_id UUID NOT NULL,
    object_type VARCHAR NOT NULL, -- 'document', 'email', 'chat', 'besluit', 'data'
    title VARCHAR NOT NULL,
    description VARCHAR,
    content_location VARCHAR NOT NULL,
    content_text VARCHAR, -- Extracted text for search
    mime_type VARCHAR,
    file_size BIGINT,
    checksum VARCHAR(64),

    -- Compliance metadata
    classification VARCHAR NOT NULL DEFAULT 'intern', -- 'openbaar', 'intern', 'vertrouwelijk', 'geheim'
    retention_period INTEGER,
    retention_trigger VARCHAR,
    destruction_date DATE,
    is_woo_relevant BOOLEAN NOT NULL DEFAULT false,
    woo_publication_date TIMESTAMPTZ,
    privacy_level VARCHAR NOT NULL DEFAULT 'geen', -- 'geen', 'normaal', 'bijzonder', 'strafrechtelijk'

    -- Vindbaarheid
    tags VARCHAR[] NOT NULL DEFAULT [],
    metadata JSON NOT NULL DEFAULT '{}',

    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    previous_version_id UUID,

    -- Audit
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================
-- 4. RELATIES & NETWERK
-- ============================================

CREATE TABLE stakeholders (
    id UUID PRIMARY KEY DEFAULT uuid(),
    stakeholder_type VARCHAR NOT NULL, -- 'burger', 'bedrijf', 'organisatie', 'intern'
    name VARCHAR NOT NULL,
    contact_email VARCHAR,
    contact_phone VARCHAR,
    organization_name VARCHAR,
    metadata JSON NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE domain_stakeholders (
    domain_id UUID NOT NULL,
    stakeholder_id UUID NOT NULL,
    role_in_domain VARCHAR NOT NULL,
    PRIMARY KEY (domain_id, stakeholder_id)
);

CREATE TABLE domain_relations (
    id UUID PRIMARY KEY DEFAULT uuid(),
    from_domain_id UUID NOT NULL,
    to_domain_id UUID NOT NULL,
    relation_type VARCHAR NOT NULL,
    strength REAL NOT NULL DEFAULT 1.0,
    discovery_method VARCHAR NOT NULL DEFAULT 'manual', -- 'automatic', 'manual', 'ai_suggestion'
    explanation VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================
-- 5. BUSINESS RULES (Compliance by Design)
-- ============================================

CREATE TABLE business_rules (
    id UUID PRIMARY KEY DEFAULT uuid(),
    rule_name VARCHAR NOT NULL,
    rule_category VARCHAR NOT NULL,
    description VARCHAR,
    legal_basis VARCHAR,
    rule_logic JSON NOT NULL,
    applies_to_domain_types VARCHAR[] NOT NULL DEFAULT [],
    applies_to_object_types VARCHAR[] NOT NULL DEFAULT [],
    is_active BOOLEAN NOT NULL DEFAULT true,
    valid_from DATE,
    valid_until DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE rule_executions (
    id UUID PRIMARY KEY DEFAULT uuid(),
    rule_id UUID NOT NULL,
    object_id UUID NOT NULL,
    domain_id UUID NOT NULL,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    success BOOLEAN NOT NULL,
    execution_result JSON NOT NULL DEFAULT '{}'
);

-- ============================================
-- 6. APPS & SERVICES
-- ============================================

CREATE TABLE apps (
    id UUID PRIMARY KEY DEFAULT uuid(),
    name VARCHAR NOT NULL,
    description VARCHAR,
    app_type VARCHAR NOT NULL,
    icon_url VARCHAR,
    endpoint_url VARCHAR NOT NULL,
    relevant_for_domain_types VARCHAR[] NOT NULL DEFAULT [],
    relevant_for_roles VARCHAR[] NOT NULL DEFAULT [],
    required_permissions JSON NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE user_app_usage (
    id UUID PRIMARY KEY DEFAULT uuid(),
    user_id UUID NOT NULL,
    app_id UUID NOT NULL,
    domain_id UUID,
    usage_count INTEGER NOT NULL DEFAULT 1,
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================
-- 7. AI/ML METADATA & SUGGESTIONS
-- ============================================

CREATE TABLE ai_metadata_suggestions (
    id UUID PRIMARY KEY DEFAULT uuid(),
    object_id UUID,
    domain_id UUID,
    suggestion_type VARCHAR NOT NULL,
    field_name VARCHAR NOT NULL,
    suggested_value JSON NOT NULL,
    confidence REAL NOT NULL,
    reasoning VARCHAR,
    source VARCHAR NOT NULL,
    model_name VARCHAR,
    model_version VARCHAR,
    user_feedback VARCHAR, -- 'accepted', 'rejected', 'modified'
    feedback_by UUID,
    feedback_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Context vectors voor semantic search
-- DuckDB heeft geen native vector type, we gebruiken FLOAT[] array
CREATE TABLE context_vectors (
    id UUID PRIMARY KEY DEFAULT uuid(),
    domain_id UUID NOT NULL,
    embedding FLOAT[] NOT NULL,
    model_name VARCHAR NOT NULL,
    model_version VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================
-- 8. GRAPHRAG ENTITEITEN & RELATIES
-- ============================================

CREATE TABLE entities (
    id UUID PRIMARY KEY DEFAULT uuid(),
    name VARCHAR NOT NULL,
    entity_type VARCHAR NOT NULL, -- 'PER', 'ORG', 'LOC', 'LAW', 'DATE', 'MONEY', 'POLICY'
    canonical_name VARCHAR,
    description VARCHAR,
    confidence REAL NOT NULL DEFAULT 1.0,
    source_domain_id UUID,
    metadata JSON NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE entity_relationships (
    id UUID PRIMARY KEY DEFAULT uuid(),
    source_entity_id UUID NOT NULL,
    target_entity_id UUID NOT NULL,
    relationship_type VARCHAR NOT NULL, -- 'WORKS_FOR', 'LOCATED_IN', 'SUBJECT_TO', etc.
    weight REAL NOT NULL DEFAULT 1.0,
    confidence REAL NOT NULL DEFAULT 1.0,
    context VARCHAR,
    source_domain_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE communities (
    id UUID PRIMARY KEY DEFAULT uuid(),
    name VARCHAR NOT NULL,
    description VARCHAR,
    level INTEGER NOT NULL DEFAULT 0,
    parent_community_id UUID,
    summary VARCHAR,
    keywords VARCHAR[] NOT NULL DEFAULT [],
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE entity_community_membership (
    entity_id UUID NOT NULL,
    community_id UUID NOT NULL,
    membership_score REAL NOT NULL DEFAULT 1.0,
    PRIMARY KEY (entity_id, community_id)
);

-- ============================================
-- 9. AUDIT & TRANSPARANTIE
-- ============================================

CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT uuid(),
    user_id UUID,
    action VARCHAR NOT NULL,
    object_type VARCHAR NOT NULL,
    object_id UUID,
    domain_id UUID,
    ip_address VARCHAR,
    user_agent VARCHAR,
    metadata JSON NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================
-- 10. FULL-TEXT SEARCH (DuckDB FTS Extension)
-- ============================================

-- DuckDB heeft een FTS extension die we kunnen gebruiken
-- INSTALL fts;
-- LOAD fts;
-- Dit wordt in Rust code gedaan

-- Maak een view voor full-text search
CREATE VIEW v_searchable_objects AS
SELECT
    id,
    domain_id,
    object_type,
    title,
    description,
    content_text,
    tags,
    classification,
    is_woo_relevant,
    created_at,
    -- Combineer alle doorzoekbare velden
    COALESCE(title, '') || ' ' ||
    COALESCE(description, '') || ' ' ||
    COALESCE(content_text, '') || ' ' ||
    array_to_string(tags, ' ') AS searchable_text
FROM information_objects;

-- ============================================
-- 11. ANALYTISCHE VIEWS
-- ============================================

-- Overzicht per domein type
CREATE VIEW v_domain_statistics AS
SELECT
    domain_type,
    status,
    COUNT(*) as domain_count,
    COUNT(DISTINCT organization_id) as org_count
FROM information_domains
GROUP BY domain_type, status;

-- Compliance dashboard view
CREATE VIEW v_compliance_overview AS
SELECT
    io.domain_id,
    id.name as domain_name,
    id.domain_type,
    COUNT(*) as total_objects,
    SUM(CASE WHEN io.is_woo_relevant THEN 1 ELSE 0 END) as woo_relevant_count,
    SUM(CASE WHEN io.classification = 'openbaar' THEN 1 ELSE 0 END) as public_count,
    SUM(CASE WHEN io.retention_period IS NULL THEN 1 ELSE 0 END) as missing_retention,
    AVG(io.retention_period) as avg_retention_years
FROM information_objects io
JOIN information_domains id ON io.domain_id = id.id
GROUP BY io.domain_id, id.name, id.domain_type;

-- App usage analytics
CREATE VIEW v_app_analytics AS
SELECT
    a.name as app_name,
    a.app_type,
    COUNT(DISTINCT uau.user_id) as unique_users,
    SUM(uau.usage_count) as total_uses,
    MAX(uau.last_used_at) as last_used
FROM apps a
LEFT JOIN user_app_usage uau ON a.id = uau.app_id
WHERE a.is_active = true
GROUP BY a.name, a.app_type;

-- GraphRAG network view
CREATE VIEW v_entity_network AS
SELECT
    e1.name as source_entity,
    e1.entity_type as source_type,
    er.relationship_type,
    e2.name as target_entity,
    e2.entity_type as target_type,
    er.weight,
    er.confidence
FROM entity_relationships er
JOIN entities e1 ON er.source_entity_id = e1.id
JOIN entities e2 ON er.target_entity_id = e2.id;

-- ============================================
-- END SCHEMA
-- ============================================
