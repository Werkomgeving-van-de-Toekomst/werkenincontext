//! GraphRAG: Knowledge Graph voor automatische relatiedetectie
//!
//! Gebruikt petgraph voor:
//! - Graph representatie van entiteiten en relaties
//! - Community detection via connected components
//! - Path finding voor relatieontdekking
//! - Graph analytics

use std::collections::HashMap;

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{kosaraju_scc, dijkstra};
use petgraph::visit::EdgeRef;
use uuid::Uuid;

use iou_core::graphrag::{
    Community, DomainRelation, DomainRelationType, DiscoveryMethod,
    Entity, EntityType, Relationship,
};

/// Knowledge Graph implementation using petgraph
pub struct KnowledgeGraph {
    /// The graph structure
    graph: DiGraph<Entity, Relationship>,
    /// Map from entity ID to node index
    entity_index: HashMap<Uuid, NodeIndex>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            entity_index: HashMap::new(),
        }
    }

    /// Add an entity to the graph
    pub fn add_entity(&mut self, entity: Entity) -> NodeIndex {
        if let Some(&idx) = self.entity_index.get(&entity.id) {
            return idx;
        }

        let id = entity.id;
        let idx = self.graph.add_node(entity);
        self.entity_index.insert(id, idx);
        idx
    }

    /// Add a relationship between two entities
    pub fn add_relationship(&mut self, relationship: Relationship) -> bool {
        let source_idx = match self.entity_index.get(&relationship.source_entity_id) {
            Some(&idx) => idx,
            None => return false,
        };

        let target_idx = match self.entity_index.get(&relationship.target_entity_id) {
            Some(&idx) => idx,
            None => return false,
        };

        self.graph.add_edge(source_idx, target_idx, relationship);
        true
    }

    /// Get an entity by ID
    pub fn get_entity(&self, id: Uuid) -> Option<&Entity> {
        self.entity_index.get(&id).map(|&idx| &self.graph[idx])
    }

    /// Get all entities
    pub fn entities(&self) -> Vec<&Entity> {
        self.graph.node_weights().collect()
    }

    /// Get all relationships
    pub fn relationships(&self) -> Vec<&Relationship> {
        self.graph.edge_weights().collect()
    }

    /// Find neighbors of an entity
    pub fn neighbors(&self, entity_id: Uuid) -> Vec<&Entity> {
        let idx = match self.entity_index.get(&entity_id) {
            Some(&idx) => idx,
            None => return vec![],
        };

        self.graph
            .neighbors(idx)
            .map(|neighbor_idx| &self.graph[neighbor_idx])
            .collect()
    }

    /// Find related entities (both incoming and outgoing edges)
    pub fn related_entities(&self, entity_id: Uuid) -> Vec<(&Entity, &Relationship)> {
        let idx = match self.entity_index.get(&entity_id) {
            Some(&idx) => idx,
            None => return vec![],
        };

        let mut results = Vec::new();

        // Outgoing edges
        for edge in self.graph.edges(idx) {
            let target_entity = &self.graph[edge.target()];
            let relationship = edge.weight();
            results.push((target_entity, relationship));
        }

        // Incoming edges
        for edge in self.graph.edges_directed(idx, petgraph::Direction::Incoming) {
            let source_entity = &self.graph[edge.source()];
            let relationship = edge.weight();
            results.push((source_entity, relationship));
        }

        results
    }

    /// Detect communities using strongly connected components
    /// This is a simple approach; for better results consider Louvain algorithm
    pub fn detect_communities(&self) -> Vec<Community> {
        let sccs = kosaraju_scc(&self.graph);
        let now = chrono::Utc::now();

        sccs.into_iter()
            .filter(|component| component.len() > 1) // Only consider groups
            .enumerate()
            .map(|(_i, component)| {
                let member_entities: Vec<&Entity> = component
                    .iter()
                    .map(|&idx| &self.graph[idx])
                    .collect();

                // Extract keywords from entity names
                let keywords: Vec<String> = member_entities
                    .iter()
                    .filter(|e| e.entity_type == EntityType::Policy)
                    .map(|e| e.name.clone())
                    .collect();

                // Generate community name based on most common entity type
                let name = self.generate_community_name(&member_entities);

                Community {
                    id: Uuid::new_v4(),
                    name,
                    description: Some(format!("{} gerelateerde entiteiten", member_entities.len())),
                    level: 0,
                    parent_community_id: None,
                    member_entity_ids: member_entities.iter().map(|e| e.id).collect(),
                    summary: None,
                    keywords,
                    created_at: now,
                }
            })
            .collect()
    }

    fn generate_community_name(&self, entities: &[&Entity]) -> String {
        // Count entity types
        let mut type_counts: HashMap<EntityType, usize> = HashMap::new();
        for entity in entities {
            *type_counts.entry(entity.entity_type).or_insert(0) += 1;
        }

        // Find dominant type
        let dominant_type = type_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(t, _)| t);

        match dominant_type {
            Some(EntityType::Organization) => "Organisatienetwerk".to_string(),
            Some(EntityType::Law) => "Wettelijk kader".to_string(),
            Some(EntityType::Location) => "Geografisch cluster".to_string(),
            Some(EntityType::Policy) => "Beleidsdomein".to_string(),
            _ => "Thematisch cluster".to_string(),
        }
    }

    /// Find shortest path between two entities
    pub fn shortest_path(&self, from: Uuid, to: Uuid) -> Option<Vec<&Entity>> {
        let from_idx = self.entity_index.get(&from)?;
        let to_idx = self.entity_index.get(&to)?;

        // Use Dijkstra with uniform weights
        let distances = dijkstra(&self.graph, *from_idx, Some(*to_idx), |_| 1);

        if !distances.contains_key(to_idx) {
            return None;
        }

        // Reconstruct path (simplified - just return if reachable for now)
        // For full path reconstruction, need to track predecessors
        Some(vec![&self.graph[*from_idx], &self.graph[*to_idx]])
    }

    /// Find domains that share entities
    pub fn find_related_domains(
        &self,
        domain_id: Uuid,
        domain_entities: &[Uuid],
    ) -> Vec<DomainRelation> {
        let now = chrono::Utc::now();
        let mut relations = Vec::new();
        let mut domain_overlap: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        // Find entities that belong to other domains
        for entity_id in domain_entities {
            if let Some(&idx) = self.entity_index.get(entity_id) {
                let entity = &self.graph[idx];
                if let Some(other_domain) = entity.source_domain_id {
                    if other_domain != domain_id {
                        domain_overlap
                            .entry(other_domain)
                            .or_default()
                            .push(*entity_id);
                    }
                }
            }
        }

        // Create relations for domains with shared entities
        for (other_domain_id, shared) in domain_overlap {
            let shared_count = shared.len();
            let strength = (shared_count as f32) / (domain_entities.len() as f32);

            relations.push(DomainRelation {
                id: Uuid::new_v4(),
                from_domain_id: domain_id,
                to_domain_id: other_domain_id,
                relation_type: DomainRelationType::SharedEntities,
                strength,
                discovery_method: DiscoveryMethod::Automatic,
                shared_entities: shared,
                explanation: Some(format!(
                    "Domeinen delen {} entiteiten",
                    shared_count
                )),
                created_at: now,
            });
        }

        relations
    }

    /// Get graph statistics
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            node_count: self.graph.node_count(),
            edge_count: self.graph.edge_count(),
            density: self.calculate_density(),
        }
    }

    fn calculate_density(&self) -> f64 {
        let n = self.graph.node_count() as f64;
        if n <= 1.0 {
            return 0.0;
        }
        let e = self.graph.edge_count() as f64;
        e / (n * (n - 1.0))
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph statistics
#[derive(Debug, Clone)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub density: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entity(name: &str, entity_type: EntityType) -> Entity {
        Entity {
            id: Uuid::new_v4(),
            name: name.to_string(),
            entity_type,
            canonical_name: None,
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: chrono::Utc::now(),
        }
    }

    fn create_test_relationship(source_id: Uuid, target_id: Uuid) -> Relationship {
        Relationship {
            id: Uuid::new_v4(),
            source_entity_id: source_id,
            target_entity_id: target_id,
            relationship_type: RelationshipType::RelatesTo,
            weight: 1.0,
            confidence: 1.0,
            context: None,
            source_domain_id: None,
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_add_entity() {
        let mut graph = KnowledgeGraph::new();
        let entity = create_test_entity("Test Entity", EntityType::Organization);
        let id = entity.id;

        graph.add_entity(entity);

        assert!(graph.get_entity(id).is_some());
        assert_eq!(graph.stats().node_count, 1);
    }

    #[test]
    fn test_add_relationship() {
        let mut graph = KnowledgeGraph::new();

        let e1 = create_test_entity("Entity 1", EntityType::Organization);
        let e2 = create_test_entity("Entity 2", EntityType::Location);
        let e1_id = e1.id;
        let e2_id = e2.id;

        graph.add_entity(e1);
        graph.add_entity(e2);

        let rel = create_test_relationship(e1_id, e2_id);
        assert!(graph.add_relationship(rel));

        assert_eq!(graph.stats().edge_count, 1);
    }

    #[test]
    fn test_find_neighbors() {
        let mut graph = KnowledgeGraph::new();

        let e1 = create_test_entity("Entity 1", EntityType::Organization);
        let e2 = create_test_entity("Entity 2", EntityType::Location);
        let e3 = create_test_entity("Entity 3", EntityType::Law);
        let e1_id = e1.id;
        let e2_id = e2.id;
        let e3_id = e3.id;

        graph.add_entity(e1);
        graph.add_entity(e2);
        graph.add_entity(e3);

        graph.add_relationship(create_test_relationship(e1_id, e2_id));
        graph.add_relationship(create_test_relationship(e1_id, e3_id));

        let neighbors = graph.neighbors(e1_id);
        assert_eq!(neighbors.len(), 2);
    }
}
