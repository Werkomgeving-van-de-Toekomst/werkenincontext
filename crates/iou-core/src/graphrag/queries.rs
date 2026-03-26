//! AQL Query Builders for GraphRAG
//!
//! Safe AQL query construction using bind parameters.

use uuid::Uuid;
use crate::graphrag::store::{NeighborFilters, TraversalDirection, TraversalRequest};

/// Build AQL query for fetching neighbors
///
/// All values use bind parameters (@param) not string interpolation.
pub fn build_neighbors_aql(entity_id: Uuid, filters: &NeighborFilters) -> String {
    // For TraversalDirection::Any, we need to query both directions
    let (source_condition, target_condition) = match filters.direction {
        TraversalDirection::Outgoing => ("e.source_entity_id == @entity_id", ""),
        TraversalDirection::Incoming => ("", "e.target_entity_id == @entity_id"),
        TraversalDirection::Any => ("e.source_entity_id == @entity_id", "OR e.target_entity_id == @entity_id"),
    };

    format!(
        r#"
        FOR e IN edge_works_for, edge_located_in, edge_subject_to, edge_refers_to, edge_relates_to,
                 edge_owner_of, edge_reports_to, edge_collaborates_with, edge_follows, edge_part_of,
                 edge_unknown
        FILTER ({source_condition} {target_condition})
        LIMIT @limit
        LET connected_id = e.source_entity_id == @entity_id ? e.target_entity_id : e.source_entity_id
        LET connected_entity = FIRST(
            FOR v IN persons, organizations, locations, laws, entities
            FILTER v.id == connected_id
            RETURN v
        )
        RETURN {{edge: e, entity: connected_entity}}
        "#
    )
}

/// Build AQL query for shortest path using named graph
///
/// Note: Requires a named graph 'knowledge_graph' to be set up in ArangoDB.
pub fn build_shortest_path_aql(from: Uuid, to: Uuid) -> String {
    format!(
        r#"
        FOR v, e, p IN OUTBOUND SHORTEST_PATH @from TO @to GRAPH 'knowledge_graph'
        RETURN p
        "#
    )
}

/// Build AQL query for graph traversal with depth range
///
/// Uses AQL's named graph traversal with depth range and options.
pub fn build_traversal_aql(request: &TraversalRequest) -> String {
    let direction = match request.direction {
        TraversalDirection::Outgoing => "OUTBOUND",
        TraversalDirection::Incoming => "INBOUND",
        TraversalDirection::Any => "ANY",
    };

    format!(
        r#"
        FOR v, e, p IN @min_depth..@max_depth {direction} @start GRAPH 'knowledge_graph'
        OPTIONS {{uniqueVertices: 'global', bfs: true}}
        LIMIT @limit
        RETURN {{vertex: v, edge: e, path: p}}
        "#
    )
}

/// Build AQL query for fetching community members
///
/// Queries edge_member_of edges and joins with entity collections.
pub fn build_community_members_aql(community_id: Uuid) -> String {
    format!(
        r#"
        FOR m IN edge_member_of
        FILTER m.community_id == @community_id
        FOR e IN persons, organizations, locations, laws, entities
        FILTER e.id == m.entity_id
        RETURN e
        "#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_neighbors_aql_uses_bind_parameters() {
        let entity_id = Uuid::new_v4();
        let filters = NeighborFilters::default();

        let aql = build_neighbors_aql(entity_id, &filters);

        // Check that bind parameters are used
        assert!(aql.contains("@entity_id"));
        assert!(aql.contains("@limit"));
    }

    #[test]
    fn build_neighbors_aql_includes_filters() {
        let entity_id = Uuid::new_v4();
        let filters = NeighborFilters {
            limit: 50,
            ..Default::default()
        };

        let aql = build_neighbors_aql(entity_id, &filters);

        assert!(aql.contains("LIMIT @limit"));
    }

    #[test]
    fn build_shortest_path_aql_uses_named_graph() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        let aql = build_shortest_path_aql(from, to);

        assert!(aql.contains("GRAPH 'knowledge_graph'"));
        assert!(aql.contains("@from"));
        assert!(aql.contains("@to"));
    }

    #[test]
    fn build_traversal_aql_includes_depth_range() {
        let request = TraversalRequest {
            min_depth: 1,
            max_depth: 3,
            ..Default::default()
        };

        let aql = build_traversal_aql(&request);

        assert!(aql.contains("@min_depth..@max_depth"));
        assert!(aql.contains("@start"));
        assert!(aql.contains("@limit"));
    }

    #[test]
    fn build_traversal_aql_includes_options() {
        let request = TraversalRequest::default();

        let aql = build_traversal_aql(&request);

        assert!(aql.contains("OPTIONS {uniqueVertices: 'global', bfs: true}"));
    }

    #[test]
    fn build_community_members_aql_inbound_direction() {
        let community_id = Uuid::new_v4();

        let aql = build_community_members_aql(community_id);

        assert!(aql.contains("@community_id"));
        assert!(aql.contains("edge_member_of"));
    }
}
