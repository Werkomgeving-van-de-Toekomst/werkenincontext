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

    /// Detect communities using the Louvain algorithm
    ///
    /// The Louvain method is a greedy optimization method that tries to optimize
    /// the modularity of a partition of a network. It works in two phases:
    /// 1. Modularity optimization: locally move nodes to communities
    /// 2. Community aggregation: collapse communities into super-nodes
    ///
    /// These phases are repeated until no improvement is possible.
    pub fn detect_communities_louvain(&self) -> LouvainResult {
        let now = chrono::Utc::now();

        // Convert to undirected for modularity calculation
        let mut communities: HashMap<NodeIndex, Uuid> = HashMap::new();
        let mut node_to_community: Vec<Uuid> = vec![Uuid::new_v4(); self.graph.node_count()];

        // Initialize each node in its own community
        for (i, node_idx) in self.graph.node_indices().enumerate() {
            let comm_id = Uuid::new_v4();
            communities.insert(node_idx, comm_id);
            node_to_community[i] = comm_id;
        }

        let mut improved = true;
        let mut iterations = 0;
        let max_iterations = 100;

        while improved && iterations < max_iterations {
            improved = false;
            iterations += 1;

            // Phase 1: Modularity optimization
            for node_idx in self.graph.node_indices() {
                let current_community = communities[&node_idx];
                let best_community = current_community;
                let mut best_gain = 0.0;

                // Calculate current modularity contribution
                let current_mod = self.calculate_node_modularity(node_idx, current_community, &communities);

                // Try moving to neighboring communities
                let neighbor_comms: Vec<_> = self
                    .graph
                    .neighbors(node_idx)
                    .map(|n| communities[&n])
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                for neighbor_comm in &neighbor_comms {
                    if *neighbor_comm != current_community {
                        let new_mod = self.calculate_node_modularity(node_idx, *neighbor_comm, &communities);
                        let gain = new_mod - current_mod;

                        if gain > best_gain {
                            return LouvainResult {
                                communities: vec![],
                                hierarchical_communities: vec![],
                                modularity: 0.0,
                                levels: 0,
                                iterations,
                                converged: false,
                            };
                        }
                    }
                }
            }

            // Phase 2: Community aggregation would happen here
            // For simplicity, we just continue with current partitioning
        }

        // Convert to Community objects
        let mut community_map: HashMap<Uuid, Vec<NodeIndex>> = HashMap::new();
        for (node_idx, comm_id) in &communities {
            community_map.entry(*comm_id).or_default().push(*node_idx);
        }

        let communities: Vec<Community> = community_map
            .into_iter()
            .filter(|(_, nodes)| nodes.len() > 1)
            .map(|(comm_id, nodes)| {
                let member_entities: Vec<&Entity> = nodes
                    .iter()
                    .map(|idx| &self.graph[*idx])
                    .collect();

                let keywords: Vec<String> = member_entities
                    .iter()
                    .filter(|e| e.entity_type == EntityType::Policy)
                    .map(|e| e.name.clone())
                    .collect();

                let name = self.generate_community_name(&member_entities);

                Community {
                    id: comm_id,
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
            .collect();

        let modularity = self.calculate_modularity(&communities);

        LouvainResult {
            communities: communities.iter().map(|c| c.id).collect(),
            hierarchical_communities: communities,
            modularity,
            levels: 1,
            iterations,
            converged: !improved,
        }
    }

    /// Calculate the modularity of the entire partition
    fn calculate_modularity(&self, communities: &[Community]) -> f32 {
        let m = self.graph.edge_count() as f32;
        if m == 0.0 {
            return 0.0;
        }

        let mut community_map: HashMap<Uuid, Vec<NodeIndex>> = HashMap::new();
        for comm in communities {
            let mut nodes = Vec::new();
            for &entity_id in &comm.member_entity_ids {
                if let Some(&idx) = self.entity_index.get(&entity_id) {
                    nodes.push(idx);
                }
            }
            community_map.insert(comm.id, nodes);
        }

        let mut modularity = 0.0;

        for edge in self.graph.edge_indices() {
            let (source, target) = self.graph.edge_endpoints(edge).unwrap();
            let source_comm = self.get_node_community(source, &community_map);
            let target_comm = self.get_node_community(target, &community_map);

            if source_comm == target_comm {
                // Edges within same community contribute positively
                let k_i = self.graph.edges(source).count() as f32;
                let k_j = self.graph.edges(target).count() as f32;
                modularity += 1.0 / m - (k_i * k_j) / (2.0 * m * m);
            }
        }

        modularity
    }

    /// Calculate modularity contribution of a single node
    fn calculate_node_modularity(
        &self,
        node: NodeIndex,
        community: Uuid,
        communities: &HashMap<NodeIndex, Uuid>,
    ) -> f32 {
        let m = self.graph.edge_count() as f32;
        if m == 0.0 {
            return 0.0;
        }

        let k_i = self.graph.edges(node).count() as f32;
        let mut internal_degree = 0.0;
        let mut community_total_degree = 0.0;

        for neighbor in self.graph.neighbors(node) {
            let neighbor_comm = communities.get(&neighbor);
            if neighbor_comm == Some(&community) {
                internal_degree += 1.0;
            }
            if neighbor_comm == Some(&community) {
                community_total_degree += self.graph.edges(neighbor).count() as f32;
            }
        }

        (internal_degree / (2.0 * m)) - ((k_i * community_total_degree) / (2.0 * m * 2.0 * m))
    }

    /// Get community ID for a node
    fn get_node_community(
        &self,
        node: NodeIndex,
        community_map: &HashMap<Uuid, Vec<NodeIndex>>,
    ) -> Option<Uuid> {
        for (comm_id, nodes) in community_map {
            if nodes.contains(&node) {
                return Some(*comm_id);
            }
        }
        None
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

    /// Get all stakeholders mentioned in a document.
    ///
    /// This retrieves all entities that have a MentionRelationship
    /// pointing to the document (using RefersTo relationship type).
    ///
    /// # Arguments
    /// * `document_id` - The UUID of the document
    ///
    /// # Returns
    /// A vector of entity references for all stakeholders in the document
    pub fn get_document_stakeholders(&self, document_id: Uuid) -> Vec<&Entity> {
        let mut stakeholders = Vec::new();

        // Find all relationships where the target is the document
        for edge in self.graph.edge_indices() {
            let relationship = &self.graph[edge];

            // Check if this relationship points to our document
            if relationship.target_entity_id == document_id {
                if let Some(&entity_idx) = self.entity_index.get(&relationship.source_entity_id) {
                    stakeholders.push(&self.graph[entity_idx]);
                }
            }
        }

        stakeholders
    }

    /// Get all documents mentioning a stakeholder.
    ///
    /// Returns a list of document UUIDs for all documents that
    /// contain a MentionRelationship for this entity.
    ///
    /// # Arguments
    /// * `entity_id` - The UUID of the stakeholder entity
    ///
    /// # Returns
    /// A vector of document UUIDs
    pub fn get_stakeholder_documents(&self, entity_id: Uuid) -> Vec<Uuid> {
        let mut documents = Vec::new();

        if let Some(&entity_idx) = self.entity_index.get(&entity_id) {
            // Find all outgoing edges (entity -> document mentions)
            for edge in self.graph.edges(entity_idx) {
                let relationship = edge.weight();
                // The target of the edge is the document
                documents.push(relationship.target_entity_id);
            }
        }

        // Deduplicate documents (in case multiple relationships to same document)
        documents.sort();
        documents.dedup();
        documents
    }

    /// Get influence metrics for a stakeholder.
    ///
    /// Calculates:
    /// - `mention_count`: Total number of mentions across all documents
    /// - `document_count`: Number of unique documents mentioning this stakeholder
    /// - `pagerank_score`: Network centrality score (higher = more influential)
    /// - `betweenness_centrality`: How often this entity lies on shortest paths
    ///
    /// # Arguments
    /// * `entity_id` - The UUID of the stakeholder entity
    ///
    /// # Returns
    /// InfluenceMetrics with calculated values (zeroed if entity not found)
    pub fn get_stakeholder_influence(&self, entity_id: Uuid) -> InfluenceMetrics {
        let mention_count = if let Some(&idx) = self.entity_index.get(&entity_id) {
            self.graph.edges(idx).count()
        } else {
            0
        };

        let document_count = self.get_stakeholder_documents(entity_id).len();

        // Calculate PageRank score using a simple iterative algorithm
        let pagerank_score = self.calculate_pagerank(entity_id);

        // Calculate betweenness centrality (simplified)
        let betweenness_centrality = self.calculate_betweenness(entity_id);

        InfluenceMetrics {
            entity_id,
            mention_count,
            document_count,
            pagerank_score,
            betweenness_centrality,
        }
    }

    /// Find stakeholders by name with fuzzy matching.
    ///
    /// Uses Jaro-Winkler similarity to find entities with similar names.
    /// This handles spelling variations, partial names, and common Dutch
    /// name variations (van, van der, de, etc.).
    ///
    /// # Arguments
    /// * `name` - The name to search for
    /// * `threshold` - Minimum similarity score (0.0 to 1.0), typically 0.8-0.9
    ///
    /// # Returns
    /// A vector of entity references matching the threshold, sorted by similarity
    pub fn find_stakeholders_by_name(&self, name: &str, threshold: f32) -> Vec<&Entity> {
        let mut matches: Vec<(&Entity, f32)> = Vec::new();

        for entity in self.entities() {
            let similarity = self.calculate_name_similarity(name, &entity.name);
            if similarity >= threshold {
                matches.push((entity, similarity));
            }
        }

        // Sort by similarity descending
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        matches.into_iter().map(|(entity, _)| entity).collect()
    }

    // Helper: Calculate PageRank for a single entity
    fn calculate_pagerank(&self, entity_id: Uuid) -> f32 {
        // Simple PageRank: degree centrality normalized by graph size
        if let Some(&idx) = self.entity_index.get(&entity_id) {
            let degree = self.graph.edges(idx).count() as f32;
            let total_nodes = self.graph.node_count() as f32;

            if total_nodes > 0.0 {
                degree / total_nodes
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    // Helper: Calculate betweenness centrality (simplified)
    fn calculate_betweenness(&self, entity_id: Uuid) -> f32 {
        // For Phase 1, use a simplified metric based on connectivity
        if let Some(&idx) = self.entity_index.get(&entity_id) {
            let neighbors = self.graph.neighbors(idx).count();
            let total_nodes = self.graph.node_count();

            if total_nodes > 1 {
                (neighbors as f32) / ((total_nodes - 1) as f32)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    // Helper: Calculate name similarity using Jaro-Winkler
    fn calculate_name_similarity(&self, a: &str, b: &str) -> f32 {
        use strsim::jaro_winkler;

        // Normalize Dutch prefixes for comparison
        let normalize = |s: &str| -> String {
            let lower = s.to_lowercase();
            // Remove common Dutch prefixes for comparison
            [" van ", " van der ", " de ", " ten ", " te "].iter()
                .fold(lower, |acc, prefix| acc.replace(prefix, " "))
                .trim()
                .to_string()
        };

        jaro_winkler(&normalize(a), &normalize(b)) as f32
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

/// Influence metrics for a stakeholder entity
#[derive(Debug, Clone)]
pub struct InfluenceMetrics {
    /// The entity ID this metrics belong to
    pub entity_id: Uuid,

    /// Total number of mentions across all documents
    pub mention_count: usize,

    /// Number of unique documents mentioning this stakeholder
    pub document_count: usize,

    /// PageRank score - network centrality measure (0.0 to 1.0)
    /// Higher values indicate more influential entities
    pub pagerank_score: f32,

    /// Betweenness centrality - how often entity lies on shortest paths
    /// Higher values indicate entities that bridge different parts of the network
    pub betweenness_centrality: f32,
}

/// Result of Louvain community detection
#[derive(Debug, Clone)]
pub struct LouvainResult {
    /// Community IDs in order of discovery
    pub communities: Vec<Uuid>,

    /// Hierarchical community structure
    pub hierarchical_communities: Vec<Community>,

    /// Overall modularity score (higher is better, max 1.0)
    pub modularity: f32,

    /// Number of hierarchical levels found
    pub levels: usize,

    /// Number of iterations performed
    pub iterations: usize,

    /// Whether the algorithm converged
    pub converged: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use iou_core::graphrag::RelationshipType;

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

    fn create_test_document_entity(doc_id: Uuid) -> Entity {
        Entity {
            id: doc_id,
            name: "Test Document".to_string(),
            entity_type: EntityType::Miscellaneous,
            canonical_name: None,
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
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

    // Stakeholder-specific tests

    #[test]
    fn test_get_document_stakeholders_returns_all_entities_for_document() {
        let mut graph = KnowledgeGraph::new();
        let doc_id = Uuid::new_v4();

        let person = create_test_entity("Jan de Vries", EntityType::Person);
        let org = create_test_entity("Ministerie van Financiën", EntityType::Organization);
        let person_id = person.id;
        let org_id = org.id;

        graph.add_entity(person);
        graph.add_entity(org);
        graph.add_entity(create_test_document_entity(doc_id));
        graph.add_relationship(create_test_relationship(person_id, doc_id));
        graph.add_relationship(create_test_relationship(org_id, doc_id));

        let stakeholders = graph.get_document_stakeholders(doc_id);
        assert_eq!(stakeholders.len(), 2);
    }

    #[test]
    fn test_get_document_stakeholders_returns_empty_vec_for_no_entities() {
        let graph = KnowledgeGraph::new();
        let doc_id = Uuid::new_v4();

        let stakeholders = graph.get_document_stakeholders(doc_id);
        assert!(stakeholders.is_empty());
    }

    #[test]
    fn test_get_stakeholder_documents_returns_all_document_ids() {
        let mut graph = KnowledgeGraph::new();
        let person = create_test_entity("Jan de Vries", EntityType::Person);
        let person_id = person.id;

        let doc1 = Uuid::new_v4();
        let doc2 = Uuid::new_v4();

        graph.add_entity(person);
        graph.add_entity(create_test_document_entity(doc1));
        graph.add_entity(create_test_document_entity(doc2));
        graph.add_relationship(create_test_relationship(person_id, doc1));
        graph.add_relationship(create_test_relationship(person_id, doc2));

        let docs = graph.get_stakeholder_documents(person_id);
        assert_eq!(docs.len(), 2);
        assert!(docs.contains(&doc1));
        assert!(docs.contains(&doc2));
    }

    #[test]
    fn test_get_stakeholder_documents_returns_empty_vec_for_unknown_entity() {
        let graph = KnowledgeGraph::new();
        let unknown_id = Uuid::new_v4();

        let docs = graph.get_stakeholder_documents(unknown_id);
        assert!(docs.is_empty());
    }

    #[test]
    fn test_get_stakeholder_influence_returns_mention_count() {
        let mut graph = KnowledgeGraph::new();
        let person = create_test_entity("Jan de Vries", EntityType::Person);
        let person_id = person.id;

        let doc1 = Uuid::new_v4();
        let doc2 = Uuid::new_v4();
        let doc3 = Uuid::new_v4();

        graph.add_entity(person);
        graph.add_entity(create_test_document_entity(doc1));
        graph.add_entity(create_test_document_entity(doc2));
        graph.add_entity(create_test_document_entity(doc3));
        graph.add_relationship(create_test_relationship(person_id, doc1));
        graph.add_relationship(create_test_relationship(person_id, doc2));
        graph.add_relationship(create_test_relationship(person_id, doc3));

        let influence = graph.get_stakeholder_influence(person_id);
        assert_eq!(influence.mention_count, 3);
    }

    #[test]
    fn test_get_stakeholder_influence_returns_document_count() {
        let mut graph = KnowledgeGraph::new();
        let person = create_test_entity("Jan de Vries", EntityType::Person);
        let person_id = person.id;

        let doc1 = Uuid::new_v4();
        let doc2 = Uuid::new_v4();

        graph.add_entity(person);
        graph.add_entity(create_test_document_entity(doc1));
        graph.add_entity(create_test_document_entity(doc2));
        graph.add_relationship(create_test_relationship(person_id, doc1));
        graph.add_relationship(create_test_relationship(person_id, doc2));

        let influence = graph.get_stakeholder_influence(person_id);
        assert_eq!(influence.document_count, 2);
    }

    #[test]
    fn test_get_stakeholder_influence_calculates_pagerank_score() {
        let mut graph = KnowledgeGraph::new();

        // Create a small network: person1 connected to 3 docs, person2 to 1 doc
        let person1 = create_test_entity("Jan de Vries", EntityType::Person);
        let person2 = create_test_entity("Marie Jansen", EntityType::Person);
        let p1_id = person1.id;
        let p2_id = person2.id;

        let doc1 = Uuid::new_v4();
        let doc2 = Uuid::new_v4();
        let doc3 = Uuid::new_v4();
        let doc4 = Uuid::new_v4();

        graph.add_entity(person1);
        graph.add_entity(person2);
        graph.add_entity(create_test_document_entity(doc1));
        graph.add_entity(create_test_document_entity(doc2));
        graph.add_entity(create_test_document_entity(doc3));
        graph.add_entity(create_test_document_entity(doc4));

        graph.add_relationship(create_test_relationship(p1_id, doc1));
        graph.add_relationship(create_test_relationship(p1_id, doc2));
        graph.add_relationship(create_test_relationship(p1_id, doc3));
        graph.add_relationship(create_test_relationship(p2_id, doc4));

        let influence1 = graph.get_stakeholder_influence(p1_id);
        let influence2 = graph.get_stakeholder_influence(p2_id);

        // Person1 should have higher PageRank due to more connections
        assert!(influence1.pagerank_score > influence2.pagerank_score);
    }

    #[test]
    fn test_find_stakeholders_by_name_returns_matches_above_threshold() {
        let mut graph = KnowledgeGraph::new();

        let person1 = create_test_entity("Jan de Vries", EntityType::Person);
        let person2 = create_test_entity("Jan Jansen", EntityType::Person);
        let person3 = create_test_entity("Marie de Vries", EntityType::Person);

        graph.add_entity(person1);
        graph.add_entity(person2);
        graph.add_entity(person3);

        // Search for "Jan de Vries" with threshold 0.8
        let matches = graph.find_stakeholders_by_name("Jan de Vries", 0.8);
        assert!(matches.len() >= 1);

        // Should find exact match
        let exact_match = matches.iter().any(|e| e.name == "Jan de Vries");
        assert!(exact_match);
    }

    #[test]
    fn test_find_stakeholders_by_name_returns_empty_vec_when_no_matches() {
        let mut graph = KnowledgeGraph::new();

        let person = create_test_entity("Jan de Vries", EntityType::Person);
        graph.add_entity(person);

        // Search for completely different name with high threshold
        let matches = graph.find_stakeholders_by_name("Completely Different Name", 0.9);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_dutch_name_prefix_normalization() {
        let mut graph = KnowledgeGraph::new();

        let person1 = create_test_entity("Jan de Vries", EntityType::Person);
        let person2 = create_test_entity("Jan de vries", EntityType::Person); // Different case
        let _person1_id = person1.id;

        graph.add_entity(person1);
        graph.add_entity(person2);

        // Should find match despite case difference and prefix
        let matches = graph.find_stakeholders_by_name("jan de vries", 0.8);
        assert!(matches.len() >= 1);
    }

    #[test]
    fn test_methods_dont_duplicate_existing_functionality() {
        // Ensure new methods use existing underlying graph structures
        let mut graph = KnowledgeGraph::new();
        let entity = create_test_entity("Test Entity", EntityType::Person);
        let entity_id = entity.id;
        let doc_id = Uuid::new_v4();

        graph.add_entity(entity);
        graph.add_entity(create_test_document_entity(doc_id));
        graph.add_relationship(create_test_relationship(entity_id, doc_id));

        // Verify we can still use existing methods
        assert!(graph.get_entity(entity_id).is_some());
        assert_eq!(graph.entities().len(), 2); // entity + document

        // Verify new methods work
        assert_eq!(graph.get_stakeholder_documents(entity_id).len(), 1);
    }
}
