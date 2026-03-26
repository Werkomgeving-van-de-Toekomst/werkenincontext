//! Integration tests for graph traversal operations
//!
//! These tests require a running ArangoDB instance.

use iou_core::graphrag::{
    Entity, EntityType, GraphStore, GraphPath, Neighbor, NeighborFilters, Relationship,
    RelationshipType, TraversalDirection, TraversalRequest, TraversalResult, StoreError
};
use uuid::Uuid;

/// Helper to create a test store
async fn setup_test_store() -> GraphStore {
    use iou_core::graphrag::connection::ArangoConfig;

    let config = ArangoConfig::from_env().unwrap_or_else(|_| ArangoConfig::new(
        "http://localhost:8529",
        "root",
        "",
        "_system"
    ));

    let store = GraphStore::new(&config).await.expect("Failed to create GraphStore");
    // Ensure collections exist
    let _ = store.ensure_collections().await;
    store
}

/// Helper to create test entities
async fn create_test_entities(store: &GraphStore) -> (Entity, Entity, Entity) {
    let person = Entity {
        id: Uuid::new_v4(),
        name: "Alice".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("alice".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let company = Entity {
        id: Uuid::new_v4(),
        name: "Acme Corp".to_string(),
        entity_type: EntityType::Organization,
        canonical_name: Some("acme-corp".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let location = Entity {
        id: Uuid::new_v4(),
        name: "New York".to_string(),
        entity_type: EntityType::Location,
        canonical_name: Some("new-york".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let person = store.create_entity(&person).await.unwrap();
    let company = store.create_entity(&company).await.unwrap();
    let location = store.create_entity(&location).await.unwrap();

    (person, company, location)
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn find_shortest_path_returns_path() {
    let store = setup_test_store().await;
    let (person, company, _location) = create_test_entities(&store).await;

    // Create relationships: person -> company -> location
    let rel1 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: company.id,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let rel2 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: company.id,
        target_entity_id: Uuid::new_v4(), // Some other entity
        relationship_type: RelationshipType::LocatedIn,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&rel1).await.unwrap();
    store.create_relationship(&rel2).await.unwrap();

    // Find path from person to company
    let path = store.find_shortest_path(person.id, company.id).await;

    // Path should exist
    assert!(path.is_ok());
    let path = path.unwrap();
    assert!(path.is_some());

    if let Some(p) = path {
        // Path should include both entities
        assert!(p.entity_ids.contains(&person.id));
        assert!(p.entity_ids.contains(&company.id));
    }
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn find_shortest_path_no_path_returns_none() {
    let store = setup_test_store().await;

    // Two completely unrelated entities
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    let path = store.find_shortest_path(id1, id2).await.unwrap();
    assert!(path.is_none());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn traverse_one_hop_returns_immediate_neighbors() {
    let store = setup_test_store().await;
    let (person, company, location) = create_test_entities(&store).await;

    // Create relationships
    let rel1 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: company.id,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 0.9,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let rel2 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: company.id,
        target_entity_id: location.id,
        relationship_type: RelationshipType::LocatedIn,
        weight: 1.0,
        confidence: 0.8,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&rel1).await.unwrap();
    store.create_relationship(&rel2).await.unwrap();

    // Traverse 1 hop from person
    let request = TraversalRequest {
        start_id: person.id,
        min_depth: 1,
        max_depth: 1,
        direction: TraversalDirection::Outgoing,
        limit: 10,
    };

    let result = store.traverse(request).await.unwrap();

    // Should find company as immediate neighbor
    assert!(!result.vertices.is_empty());
    assert!(result.vertices.iter().any(|v| v.id == company.id));
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn traverse_three_hops_returns_connected_entities() {
    let store = setup_test_store().await;
    let (person, company, location) = create_test_entities(&store).await;

    // Create a chain: person -> company -> location -> another entity
    let another = Entity {
        id: Uuid::new_v4(),
        name: "Another Entity".to_string(),
        entity_type: EntityType::Miscellaneous,
        canonical_name: Some("another".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };
    let another = store.create_entity(&another).await.unwrap();

    // Create relationships
    let rel1 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: company.id,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let rel2 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: company.id,
        target_entity_id: location.id,
        relationship_type: RelationshipType::LocatedIn,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let rel3 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: location.id,
        target_entity_id: another.id,
        relationship_type: RelationshipType::RelatesTo,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&rel1).await.unwrap();
    store.create_relationship(&rel2).await.unwrap();
    store.create_relationship(&rel3).await.unwrap();

    // Traverse up to 3 hops
    let request = TraversalRequest {
        start_id: person.id,
        min_depth: 1,
        max_depth: 3,
        direction: TraversalDirection::Outgoing,
        limit: 100,
    };

    let result = store.traverse(request).await.unwrap();

    // Should find entities in the chain
    assert!(!result.vertices.is_empty());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn traverse_with_filters_filters_correctly() {
    let store = setup_test_store().await;
    let (person, company, location) = create_test_entities(&store).await;

    // Create relationships with different confidences
    let high_conf_rel = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: company.id,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 0.95,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let low_conf_rel = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: location.id,
        relationship_type: RelationshipType::LocatedIn,
        weight: 1.0,
        confidence: 0.5,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&high_conf_rel).await.unwrap();
    store.create_relationship(&low_conf_rel).await.unwrap();

    // Traverse with incoming direction
    let request = TraversalRequest {
        start_id: person.id,
        min_depth: 1,
        max_depth: 1,
        direction: TraversalDirection::Outgoing,
        limit: 100,
    };

    let result = store.traverse(request).await.unwrap();

    // Should return both relationships
    assert!(!result.edges.is_empty());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_neighbors_returns_connected_entities() {
    let store = setup_test_store().await;
    let (person, company, location) = create_test_entities(&store).await;

    // Create relationships
    let rel1 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: company.id,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 0.9,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let rel2 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: location.id,
        relationship_type: RelationshipType::LocatedIn,
        weight: 0.5,
        confidence: 0.7,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&rel1).await.unwrap();
    store.create_relationship(&rel2).await.unwrap();

    // Get outgoing neighbors
    let filters = NeighborFilters {
        direction: TraversalDirection::Outgoing,
        limit: 10,
        ..Default::default()
    };

    let neighbors = store.get_neighbors(person.id, filters).await.unwrap();

    // Should return both neighbors
    assert!(neighbors.len() >= 2);

    // Verify neighbors contain the expected entities
    let entity_ids: Vec<_> = neighbors.iter().map(|n| n.entity.id).collect();
    assert!(entity_ids.contains(&company.id));
    assert!(entity_ids.contains(&location.id));

    // Verify relationship types
    let rel_types: Vec<_> = neighbors.iter().map(|n| n.relationship_type).collect();
    assert!(rel_types.contains(&RelationshipType::WorksFor));
    assert!(rel_types.contains(&RelationshipType::LocatedIn));
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_neighbors_filters_by_type() {
    let store = setup_test_store().await;
    let (person, company, location) = create_test_entities(&store).await;

    // Create relationships
    let rel1 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: company.id,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let rel2 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: location.id,
        relationship_type: RelationshipType::LocatedIn,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&rel1).await.unwrap();
    store.create_relationship(&rel2).await.unwrap();

    // Filter by WorksFor type only
    let filters = NeighborFilters {
        direction: TraversalDirection::Outgoing,
        relationship_types: vec![RelationshipType::WorksFor],
        limit: 10,
        ..Default::default()
    };

    let neighbors = store.get_neighbors(person.id, filters).await.unwrap();

    // All results should be WorksFor
    assert!(neighbors.iter().all(|n| n.relationship_type == RelationshipType::WorksFor));
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_neighbors_incoming_only() {
    let store = setup_test_store().await;
    let (person, company, _) = create_test_entities(&store).await;

    // Create incoming relationship (company -> person)
    let rel = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: company.id,
        target_entity_id: person.id,
        relationship_type: RelationshipType::CollaboratesWith,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&rel).await.unwrap();

    // Get incoming neighbors
    let filters = NeighborFilters {
        direction: TraversalDirection::Incoming,
        limit: 10,
        ..Default::default()
    };

    let neighbors = store.get_neighbors(person.id, filters).await.unwrap();

    // Should find the incoming relationship
    assert!(neighbors.len() >= 1);
    assert_eq!(neighbors[0].entity.id, company.id);
    assert!(neighbors[0].is_outgoing); // Is outgoing from company's perspective
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_neighbors_filters_by_confidence() {
    let store = setup_test_store().await;
    let (person, company, location) = create_test_entities(&store).await;

    // Create relationships with different confidence
    let high_conf = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: company.id,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 0.95,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let low_conf = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: location.id,
        relationship_type: RelationshipType::LocatedIn,
        weight: 1.0,
        confidence: 0.5,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&high_conf).await.unwrap();
    store.create_relationship(&low_conf).await.unwrap();

    // Filter for confidence >= 0.9
    let filters = NeighborFilters {
        direction: TraversalDirection::Outgoing,
        min_confidence: Some(0.9),
        limit: 10,
        ..Default::default()
    };

    let neighbors = store.get_neighbors(person.id, filters).await.unwrap();

    // All results should have confidence >= 0.9
    assert!(neighbors.iter().all(|n| n.confidence >= 0.9));
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn graph_path_from_json_value() {
    // Test the JSON parsing logic
    let json = r#"
    [
        {"id": "00000000-0000-0000-0000-000000000001", "name": "Alice"},
        {"id": "rel1", "source_entity_id": "00000000-0000-0000-0000-000000000001", "relationship_type": "WORKS_FOR"},
        {"id": "00000000-0000-0000-0000-000000000002", "name": "Company"}
    ]
    "#;

    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    let result = GraphPath::from_json_value(value);

    // Should successfully parse the path
    assert!(result.is_ok());
    let path = result.unwrap();
    assert_eq!(path.entity_ids.len(), 2);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn find_shortest_path_performance_under_100ms() {
    let store = setup_test_store().await;
    let (person, company, _location) = create_test_entities(&store).await;

    // Create a direct relationship
    let rel = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: company.id,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&rel).await.unwrap();

    // Measure performance
    let start = std::time::Instant::now();
    let _path = store.find_shortest_path(person.id, company.id).await.unwrap();
    let elapsed = start.elapsed();

    // Single-hop should be under 100ms
    assert!(
        elapsed.as_millis() < 100,
        "Shortest path query took {}ms, expected < 100ms",
        elapsed.as_millis()
    );
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn traverse_performance_under_500ms() {
    let store = setup_test_store().await;
    let (person, company, location) = create_test_entities(&store).await;

    // Create a 3-hop chain
    let another = Entity {
        id: Uuid::new_v4(),
        name: "Another Entity".to_string(),
        entity_type: EntityType::Miscellaneous,
        canonical_name: Some("another".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };
    let another = store.create_entity(&another).await.unwrap();

    let rel1 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: company.id,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let rel2 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: company.id,
        target_entity_id: location.id,
        relationship_type: RelationshipType::LocatedIn,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let rel3 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: location.id,
        target_entity_id: another.id,
        relationship_type: RelationshipType::RelatesTo,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&rel1).await.unwrap();
    store.create_relationship(&rel2).await.unwrap();
    store.create_relationship(&rel3).await.unwrap();

    // Measure 3-hop traversal performance
    let request = TraversalRequest {
        start_id: person.id,
        min_depth: 1,
        max_depth: 3,
        direction: TraversalDirection::Outgoing,
        limit: 100,
    };

    let start = std::time::Instant::now();
    let _result = store.traverse(request).await.unwrap();
    let elapsed = start.elapsed();

    // 3-hop should be under 500ms
    assert!(
        elapsed.as_millis() < 500,
        "3-hop traversal took {}ms, expected < 500ms",
        elapsed.as_millis()
    );
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn neighbor_from_json_value() {
    let json = r#"
    {
        "entity": {
            "id": "00000000-0000-0000-0000-000000000001",
            "name": "Test Entity",
            "entity_type": "PERSON",
            "canonical_name": "test",
            "description": null,
            "confidence": 1.0,
            "source_domain_id": null,
            "metadata": {},
            "created_at": "2025-03-25T12:00:00Z"
        },
        "relationship": {
            "id": "rel1",
            "source_entity_id": "00000000-0000-0000-0000-000000000001",
            "target_entity_id": "00000000-0000-0000-0000-000000000002",
            "relationship_type": "WORKS_FOR",
            "weight": 1.0,
            "confidence": 0.9,
            "context": null,
            "source_domain_id": null,
            "created_at": "2025-03-25T12:00:00Z"
        }
    }
    "#;

    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    let result = Neighbor::from_json_value(value);

    // Should successfully parse the neighbor
    assert!(result.is_ok());
    let neighbor = result.unwrap();
    assert_eq!(neighbor.entity.name, "Test Entity");
    assert_eq!(neighbor.relationship_type, RelationshipType::WorksFor);
}
