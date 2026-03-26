-- GraphRAG: relationele graaf in PostgreSQL (bron van waarheid / OLTP-graph)
--
-- Patroon: entiteiten en randen als tabellen + B-tree indexen voor 1-hop en type-filters;
-- multi-hop via recursive CTEs in applicatie of BI. DuckDB krijgt dezelfde shape voor OLAP
-- (zie migrations/004_graph_optimization.sql + ETL).

-- ============================================================
-- Tabellen (IF NOT EXISTS voor bestaande omgevingen)
-- ============================================================

CREATE TABLE IF NOT EXISTS entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    entity_type VARCHAR NOT NULL,
    canonical_name VARCHAR,
    description TEXT,
    confidence REAL NOT NULL DEFAULT 1.0,
    source_domain_id UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS entity_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    target_entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    relationship_type VARCHAR NOT NULL,
    weight REAL NOT NULL DEFAULT 1.0,
    confidence REAL NOT NULL DEFAULT 1.0,
    context TEXT,
    source_domain_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT entity_relationships_no_self_loop CHECK (source_entity_id <> target_entity_id)
);

CREATE TABLE IF NOT EXISTS communities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    level INTEGER NOT NULL DEFAULT 0,
    parent_community_id UUID REFERENCES communities(id) ON DELETE SET NULL,
    summary TEXT,
    keywords TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS entity_community_membership (
    entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    community_id UUID NOT NULL REFERENCES communities(id) ON DELETE CASCADE,
    membership_score REAL NOT NULL DEFAULT 1.0,
    PRIMARY KEY (entity_id, community_id)
);

-- ============================================================
-- Indexen: traversaal en GraphRAG-retrieval
-- ============================================================

-- Entiteit op domein / type (context bundles, filters)
CREATE INDEX IF NOT EXISTS idx_entities_source_domain
    ON entities(source_domain_id)
    WHERE source_domain_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_entities_entity_type
    ON entities(entity_type);

CREATE INDEX IF NOT EXISTS idx_entities_canonical_name
    ON entities(canonical_name)
    WHERE canonical_name IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_entities_metadata_gin
    ON entities USING gin(metadata);

-- Uitgaande en inkomende buren (1-hop) — kernpatroon voor “graph zonder graph DB”
CREATE INDEX IF NOT EXISTS idx_entity_relationships_source
    ON entity_relationships(source_entity_id);

CREATE INDEX IF NOT EXISTS idx_entity_relationships_target
    ON entity_relationships(target_entity_id);

CREATE INDEX IF NOT EXISTS idx_entity_relationships_type
    ON entity_relationships(relationship_type);

CREATE INDEX IF NOT EXISTS idx_entity_relationships_source_type
    ON entity_relationships(source_entity_id, relationship_type);

CREATE INDEX IF NOT EXISTS idx_entity_relationships_target_type
    ON entity_relationships(target_entity_id, relationship_type);

CREATE INDEX IF NOT EXISTS idx_entity_relationships_domain
    ON entity_relationships(source_domain_id)
    WHERE source_domain_id IS NOT NULL;

-- Community lookups
CREATE INDEX IF NOT EXISTS idx_entity_community_membership_community
    ON entity_community_membership(community_id);

CREATE INDEX IF NOT EXISTS idx_communities_parent
    ON communities(parent_community_id)
    WHERE parent_community_id IS NOT NULL;

-- (Optioneel later) UNIQUE (source_entity_id, target_entity_id, relationship_type) als je
-- op DB-niveau één rand per type afdwingt; nu toegestaan voor meerdere contexten/mentions.

-- ============================================================
-- Views: SQL-“ontology” / netwerkoverzicht
-- ============================================================

CREATE OR REPLACE VIEW v_entity_graph_degree AS
SELECT
    e.id AS entity_id,
    COALESCE(o.out_degree, 0)::bigint AS out_degree,
    COALESCE(i.in_degree, 0)::bigint AS in_degree,
    (COALESCE(o.out_degree, 0) + COALESCE(i.in_degree, 0))::bigint AS total_degree
FROM entities e
LEFT JOIN (
    SELECT source_entity_id, COUNT(*)::bigint AS out_degree
    FROM entity_relationships
    GROUP BY source_entity_id
) o ON e.id = o.source_entity_id
LEFT JOIN (
    SELECT target_entity_id, COUNT(*)::bigint AS in_degree
    FROM entity_relationships
    GROUP BY target_entity_id
) i ON e.id = i.target_entity_id;

CREATE OR REPLACE VIEW v_relationship_type_stats AS
SELECT
    relationship_type,
    COUNT(*)::bigint AS edge_count,
    AVG(weight)::real AS avg_weight,
    AVG(confidence)::real AS avg_confidence
FROM entity_relationships
GROUP BY relationship_type;
