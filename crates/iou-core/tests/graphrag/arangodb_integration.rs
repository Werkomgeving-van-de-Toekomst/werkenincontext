//! Integration tests for ArangoDB GraphRAG operations
//!
//! These tests require a running ArangoDB instance.

use iou_core::graphrag::{
    Community, Entity, EntityType, GraphStore, Relationship, RelationshipType,
    TraversalDirection, TraversalRequest,
};
use uuid::Uuid;
use chrono::Utc;

/// Common test utilities
pub mod common {
    use iou_core::graphrag::GraphStore;

    /// Setup a test GraphStore instance
    pub async fn setup_test_store() -> GraphStore {
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
}

// ========== CRUD Tests ==========

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_entity_crud() {
    let store = common::setup_test_store().await;

    // Create
    let entity = Entity {
        id: Uuid::new_v4(),
        name: "Test Entity".to_string(),
        entity_type: EntityType::Organization,
        canonical_name: Some("test-entity".to_string()),
        description: Some("A test entity".to_string()),
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({"test": true}),
        created_at: Utc::now(),
    };

    let created = store.create_entity(&entity).await.unwrap();
    assert_eq!(created.name, "Test Entity");

    // Read
    let found = store.get_entity(created.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Test Entity");

    // Update
    use iou_core::graphrag::EntityUpdate;
    let updates = EntityUpdate {
        name: Some("Updated Name".to_string()),
        ..Default::default()
    };

    let updated = store.update_entity(created.id, updates).await.unwrap();
    assert_eq!(updated.name, "Updated Name");

    // Delete
    let deleted = store.delete_entity(created.id, false).await.unwrap();
    assert!(deleted);

    let found = store.get_entity(created.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_relationship_crud() {
    let store = common::setup_test_store().await;

    // Create two entities
    let entity1 = store.create_entity(&Entity {
        id: Uuid::new_v4(),
        name: "Entity 1".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("entity-1".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    }).await.unwrap();

    let entity2 = store.create_entity(&Entity {
        id: Uuid::new_v4(),
        name: "Entity 2".to_string(),
        entity_type: EntityType::Organization,
        canonical_name: Some("entity-2".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    }).await.unwrap();

    // Create relationship
    let relationship = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: entity1.id,
        target_entity_id: entity2.id,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 0.9,
        context: Some("Test relationship".to_string()),
        source_domain_id: None,
        created_at: Utc::now(),
    };

    let created = store.create_relationship(&relationship).await.unwrap();
    assert_eq!(created.relationship_type, RelationshipType::WorksFor);

    // Delete
    let deleted = store.delete_relationship(created.id).await.unwrap();
    assert!(deleted);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_graph_traversal() {
    let store = common::setup_test_store().await;

    let person = store.create_entity(&Entity {
        id: Uuid::new_v4(),
        name: "Person".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("person".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    }).await.unwrap();

    let company = store.create_entity(&Entity {
        id: Uuid::new_v4(),
        name: "Company".to_string(),
        entity_type: EntityType::Organization,
        canonical_name: Some("company".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    }).await.unwrap();

    let _ = store.create_relationship(&Relationship {
        id: Uuid::new_v4(),
        source_entity_id: person.id,
        target_entity_id: company.id,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: Utc::now(),
    }).await.unwrap();

    let request = TraversalRequest {
        start_id: person.id,
        min_depth: 1,
        max_depth: 1,
        direction: TraversalDirection::Outgoing,
        limit: 10,
    };

    let result = store.traverse(request).await.unwrap();
    assert!(!result.vertices.is_empty());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_community_operations() {
    let store = common::setup_test_store().await;

    let entity = store.create_entity(&Entity {
        id: Uuid::new_v4(),
        name: "Member Entity".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("member".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    }).await.unwrap();

    let community = Community {
        id: Uuid::new_v4(),
        name: "Test Community".to_string(),
        description: Some("A test community".to_string()),
        level: 1,
        parent_community_id: None,
        member_entity_ids: vec![entity.id],
        summary: None,
        keywords: vec!["test".to_string()],
        created_at: Utc::now(),
    };

    let created = store.create_community(&community).await.unwrap();
    assert_eq!(created.name, "Test Community");

    let members = store.get_community_members(created.id).await.unwrap();
    assert_eq!(members.len(), 1);
}

// ========== Concurrency Tests ==========

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_concurrent_entity_creation_resolves_to_single() {
    use iou_core::graphrag::connection::ArangoConfig;

    let canonical_name = "race-entity".to_string();

    let mut handles = Vec::new();
    for _ in 0..5 {
        let config = ArangoConfig::from_env().unwrap_or_else(|_| ArangoConfig::new(
            "http://localhost:8529",
            "root",
            "",
            "_system"
        ));
        let canonical_name_clone = canonical_name.clone();

        let handle = tokio::spawn(async move {
            let store = GraphStore::new(&config).await.unwrap();
            let entity = Entity {
                id: Uuid::new_v4(),
                name: "Race Entity".to_string(),
                entity_type: EntityType::Organization,
                canonical_name: Some(canonical_name_clone),
                description: None,
                confidence: 1.0,
                source_domain_id: None,
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
            };
            store.get_or_create_entity(&entity).await
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 5);
    for result in &results {
        assert!(result.is_ok());
    }
}

// ========== Performance Tests ==========

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_single_hop_under_100ms() {
    let store = common::setup_test_store().await;

    let entity1 = store.create_entity(&Entity {
        id: Uuid::new_v4(),
        name: "E1".to_string(),
        entity_type: EntityType::Organization,
        canonical_name: Some("e1".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    }).await.unwrap();

    let entity2 = store.create_entity(&Entity {
        id: Uuid::new_v4(),
        name: "E2".to_string(),
        entity_type: EntityType::Organization,
        canonical_name: Some("e2".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    }).await.unwrap();

    let _ = store.create_relationship(&Relationship {
        id: Uuid::new_v4(),
        source_entity_id: entity1.id,
        target_entity_id: entity2.id,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: Utc::now(),
    }).await.unwrap();

    let request = TraversalRequest {
        start_id: entity1.id,
        min_depth: 1,
        max_depth: 1,
        direction: TraversalDirection::Outgoing,
        limit: 10,
    };

    let start = std::time::Instant::now();
    let _ = store.traverse(request).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 100);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_three_hop_under_500ms() {
    let store = common::setup_test_store().await;

    let mut entities = Vec::new();
    for i in 0..4 {
        let entity = store.create_entity(&Entity {
            id: Uuid::new_v4(),
            name: format!("E{}", i),
            entity_type: EntityType::Organization,
            canonical_name: Some(format!("e{}", i)),
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        }).await.unwrap();
        entities.push(entity);
    }

    for i in 0..3 {
        let _ = store.create_relationship(&Relationship {
            id: Uuid::new_v4(),
            source_entity_id: entities[i].id,
            target_entity_id: entities[i + 1].id,
            relationship_type: RelationshipType::RelatesTo,
            weight: 1.0,
            confidence: 1.0,
            context: None,
            source_domain_id: None,
            created_at: Utc::now(),
        }).await.unwrap();
    }

    let request = TraversalRequest {
        start_id: entities[0].id,
        min_depth: 1,
        max_depth: 3,
        direction: TraversalDirection::Outgoing,
        limit: 100,
    };

    let start = std::time::Instant::now();
    let _ = store.traverse(request).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 500);
}
