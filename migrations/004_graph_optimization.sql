-- DuckDB: GraphRAG / kennisgraaf — indexen + analytische views
--
-- Doel: snelle aggregaties en neighborhood-stats voor ranking en dashboards;
-- Postgres blijft de aanbevolen bron voor constraint-rijke writes (FK); DuckDB voor OLAP.

-- Indexen (1-hop traversaal, type-filters)
CREATE INDEX IF NOT EXISTS idx_entities_source_domain ON entities(source_domain_id);
CREATE INDEX IF NOT EXISTS idx_entities_entity_type ON entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_entities_canonical_name ON entities(canonical_name);

CREATE INDEX IF NOT EXISTS idx_entity_relationships_source ON entity_relationships(source_entity_id);
CREATE INDEX IF NOT EXISTS idx_entity_relationships_target ON entity_relationships(target_entity_id);
CREATE INDEX IF NOT EXISTS idx_entity_relationships_type ON entity_relationships(relationship_type);
CREATE INDEX IF NOT EXISTS idx_entity_relationships_source_type ON entity_relationships(source_entity_id, relationship_type);
CREATE INDEX IF NOT EXISTS idx_entity_relationships_target_type ON entity_relationships(target_entity_id, relationship_type);
CREATE INDEX IF NOT EXISTS idx_entity_relationships_domain ON entity_relationships(source_domain_id);

CREATE INDEX IF NOT EXISTS idx_entity_community_membership_community ON entity_community_membership(community_id);

-- Analytische views (materialized refresh = opnieuw aanmaken of batch job indien nodig)
CREATE OR REPLACE VIEW v_entity_graph_degree AS
SELECT
    e.id AS entity_id,
    COALESCE(o.out_degree, 0) AS out_degree,
    COALESCE(i.in_degree, 0) AS in_degree,
    COALESCE(o.out_degree, 0) + COALESCE(i.in_degree, 0) AS total_degree
FROM entities e
LEFT JOIN (
    SELECT source_entity_id AS entity_id, COUNT(*) AS out_degree
    FROM entity_relationships
    GROUP BY source_entity_id
) o ON e.id = o.entity_id
LEFT JOIN (
    SELECT target_entity_id AS entity_id, COUNT(*) AS in_degree
    FROM entity_relationships
    GROUP BY target_entity_id
) i ON e.id = i.entity_id;

CREATE OR REPLACE VIEW v_relationship_type_stats AS
SELECT
    relationship_type,
    COUNT(*) AS edge_count,
    AVG(weight) AS avg_weight,
    AVG(confidence) AS avg_confidence
FROM entity_relationships
GROUP BY relationship_type;
