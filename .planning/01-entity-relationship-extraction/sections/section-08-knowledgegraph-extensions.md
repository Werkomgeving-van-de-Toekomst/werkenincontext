Now I have all the context I need to generate the section content. Let me create the content for section-08-knowledgegraph-extensions:

# Section 08: KnowledgeGraph Extensions

## Overview

This section extends the existing `KnowledgeGraph` implementation in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/graphrag.rs` with stakeholder-specific query methods. These methods enable efficient retrieval of stakeholder entities, their relationships to documents, and influence metrics.

## Dependencies

This section depends on:

1. **Section 01 (Foundation & Types)** - Provides the `PersonStakeholder`, `OrganizationStakeholder`, and `MentionRelationship` types that these methods will query
2. **Section 07 (Pipeline Integration)** - Ensures that entities and mention relationships are being added to the `KnowledgeGraph` so there is data to query

## Existing KnowledgeGraph API

The `KnowledgeGraph` struct in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/graphrag.rs` currently provides:

- `add_entity(entity: Entity) -> NodeIndex` - Adds an entity and returns its node index
- `add_relationship(relationship: Relationship) -> bool` - Adds a relationship between entities
- `get_entity(id: Uuid) -> Option<&Entity>` - Retrieves a single entity by ID
- `entities() -> Vec<&Entity>` - Returns all entities
- `relationships() -> Vec<&Relationship>` - Returns all relationships
- `neighbors(entity_id: Uuid) -> Vec<&Entity>` - Finds direct neighbors of an entity
- `related_entities(entity_id: Uuid) -> Vec<(&Entity, &Relationship)>` - Finds related entities with their relationships

## Tests FIRST

Before implementing, write the following tests in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/graphrag.rs` (in the `tests` module):

```rust
#[cfg(test)]
mod stakeholder_tests {
    use super::*;
    use iou_core::graphrag::{EntityType, RelationshipType};

    // Helper function to create a test entity
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

    // Helper to create a mention relationship
    fn create_mention_relationship(entity_id: Uuid, document_id: Uuid) -> Relationship {
        Relationship {
            id: Uuid::new_v4(),
            source_entity_id: entity_id,
            target_entity_id: document_id,
            relationship_type: RelationshipType::RefersTo,
            weight: 1.0,
            confidence: 1.0,
            context: None,
            source_domain_id: None,
            created_at: chrono::Utc::now(),
        }
    }

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
        graph.add_relationship(create_mention_relationship(person_id, doc_id));
        graph.add_relationship(create_mention_relationship(org_id, doc_id));
        
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
        graph.add_relationship(create_mention_relationship(person_id, doc1));
        graph.add_relationship(create_mention_relationship(person_id, doc2));
        
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
        graph.add_relationship(create_mention_relationship(person_id, doc1));
        graph.add_relationship(create_mention_relationship(person_id, doc2));
        graph.add_relationship(create_mention_relationship(person_id, doc3));
        
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
        graph.add_relationship(create_mention_relationship(person_id, doc1));
        graph.add_relationship(create_mention_relationship(person_id, doc2));
        
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
        
        graph.add_relationship(create_mention_relationship(p1_id, doc1));
        graph.add_relationship(create_mention_relationship(p1_id, doc2));
        graph.add_relationship(create_mention_relationship(p1_id, doc3));
        graph.add_relationship(create_mention_relationship(p2_id, doc4));
        
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
    fn test_methods_dont_duplicate_existing_functionality() {
        // Ensure new methods use existing underlying graph structures
        let mut graph = KnowledgeGraph::new();
        let entity = create_test_entity("Test Entity", EntityType::Person);
        let entity_id = entity.id;
        let doc_id = Uuid::new_v4();
        
        graph.add_entity(entity);
        graph.add_relationship(create_mention_relationship(entity_id, doc_id));
        
        // Verify we can still use existing methods
        assert!(graph.get_entity(entity_id).is_some());
        assert_eq!(graph.entities().len(), 1);
        
        // Verify new methods work
        assert_eq!(graph.get_stakeholder_documents(entity_id).len(), 1);
    }
}
```

## Implementation

### File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/graphrag.rs`

Add the following methods to the `KnowledgeGraph` impl block:

```rust
impl KnowledgeGraph {
    // ... existing methods ...
    
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
            let (source, target) = self.graph.edge_endpoints(edge).unwrap();
            let relationship = &self.graph[edge];
            
            // Check if this is a mention relationship pointing to our document
            if target == self.entity_index.get(&document_id).copied().unwrap_or_else(|| {
                // Document might not be in the graph as an entity,
                // so we check by UUID directly
                return NodeIndex::end(); // Sentinel value
            }) || relationship.target_entity_id == document_id {
                if let Some(&entity_idx) = self.entity_index.get(&relationship.source_entity_id) {
                    stakeholders.push(&self.graph[entity_idx]);
                }
            } else if relationship.target_entity_id == document_id {
                // Direct UUID check for relationships
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
        
        jaro_winkler(&normalize(a), &normalize(b))
    }
}
```

### Add the InfluenceMetrics struct

Add this struct definition before the `KnowledgeGraph` impl (or in an appropriate location):

```rust
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
```

### Add strsim dependency

Ensure `strsim` is added to `/Users/marc/Projecten/iou-modern/crates/iou-ai/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
strsim = "0.11"
```

## Implementation Notes

### Relationship Type Convention

For mention relationships, the implementation uses the existing `RelationshipType::RefersTo` to indicate a stakeholder is mentioned in a document. The convention is:

- `source_entity_id`: The stakeholder entity (Person or Organization)
- `target_entity_id`: The document ID (document entities are added to the graph during extraction)

### PageRank Implementation

The current PageRank implementation is simplified for Phase 1. A production implementation would:

1. Run the full iterative PageRank algorithm on the entire graph
2. Handle dangling nodes properly
3. Use a damping factor (typically 0.85)

For Phase 1, the degree centrality approximation is sufficient.

### Name Similarity

The `calculate_name_similarity` method uses Jaro-Winkler similarity with Dutch name prefix normalization. This handles:

- Case insensitivity
- Dutch prefixes (van, van der, de, ten, te)
- Spelling variations
- Partial name matches

### Performance Considerations

- `get_document_stakeholders`: O(E) where E is the number of edges
- `get_stakeholder_documents`: O(degree) for the entity
- `get_stakeholder_influence`: O(degree) for the entity
- `find_stakeholders_by_name`: O(N * name_comparison) where N is the number of entities

For large graphs, consider:

1. Adding an index from document IDs to stakeholder entities
2. Caching influence metrics
3. Using a more efficient similarity search (e.g., n-gram indexing)

## Success Criteria

1. All tests pass
2. `get_document_stakeholders` returns all entities for a document
3. `get_stakeholder_documents` returns all document IDs for an entity
4. `get_stakeholder_influence` returns calculated metrics
5. `find_stakeholders_by_name` returns matches above threshold
6. Methods integrate with existing `KnowledgeGraph` without duplicating functionality

## Integration with Section 09 (API Endpoints)

These methods will be used by Section 09 to expose stakeholder data via REST API endpoints:

- `GET /stakeholders/:id/documents` will use `get_stakeholder_documents`
- `GET /documents/:id/stakeholders` will use `get_document_stakeholders`
- `GET /stakeholders/search?q=` will use `find_stakeholders_by_name`
- Influence metrics will be included in `GET /stakeholders/:id` response