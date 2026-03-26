//! Integration tests for bulk operations
//!
//! These tests require a running ArangoDB instance.

use iou_core::graphrag::{Entity, EntityType, GraphStore, Relationship, RelationshipType, StoreError};
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

    GraphStore::new(&config).await.expect("Failed to create GraphStore")
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn bulk_create_entities_creates_all() {
    let store = setup_test_store().await;

    // Create test entities with unique canonical names
    let entities: Vec<Entity> = (0..10)
        .map(|i| Entity {
            id: Uuid::new_v4(),
            name: format!("Bulk Entity {}", i),
            entity_type: EntityType::Miscellaneous,
            canonical_name: Some(format!("bulk-entity-{}", Uuid::new_v4())),
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        })
        .collect();

    let result = store.bulk_create_entities(entities.clone()).await.unwrap();

    assert_eq!(result.len(), 10);

    // Verify all entities were created
    for entity in &result {
        let found = store.get_entity(entity.id).await.unwrap();
        assert!(found.is_some());
    }
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn bulk_create_entities_returns_ids() {
    let store = setup_test_store().await;

    let entities: Vec<Entity> = (0..5)
        .map(|i| Entity {
            id: Uuid::nil(), // IDs will be generated
            name: format!("Auto ID Entity {}", i),
            entity_type: EntityType::Person,
            canonical_name: Some(format!("auto-id-entity-{}", Uuid::new_v4())),
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        })
        .collect();

    let result = store.bulk_create_entities(entities).await.unwrap();

    // All returned entities should have non-nil IDs
    for entity in &result {
        assert_ne!(entity.id, Uuid::nil());
    }
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn bulk_create_entities_empty_returns_empty() {
    let store = setup_test_store().await;

    let result = store.bulk_create_entities(vec![]).await.unwrap();

    assert!(result.is_empty());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn bulk_create_relationships_creates_all() {
    let store = setup_test_store().await;

    // Create source and target entities first
    let person1 = Entity {
        id: Uuid::new_v4(),
        name: "Person 1".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some(format!("person-1-{}", Uuid::new_v4())),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let person2 = Entity {
        id: Uuid::new_v4(),
        name: "Person 2".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some(format!("person-2-{}", Uuid::new_v4())),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let org = Entity {
        id: Uuid::new_v4(),
        name: "Organization".to_string(),
        entity_type: EntityType::Organization,
        canonical_name: Some(format!("organization-{}", Uuid::new_v4())),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let created_person1 = store.create_entity(&person1).await.unwrap();
    let created_person2 = store.create_entity(&person2).await.unwrap();
    let created_org = store.create_entity(&org).await.unwrap();

    // Create relationships
    let relationships = vec![
        Relationship {
            id: Uuid::new_v4(),
            source_entity_id: created_person1.id,
            target_entity_id: created_org.id,
            relationship_type: RelationshipType::WorksFor,
            weight: 1.0,
            confidence: 1.0,
            context: None,
            source_domain_id: None,
            created_at: chrono::Utc::now(),
        },
        Relationship {
            id: Uuid::new_v4(),
            source_entity_id: created_person2.id,
            target_entity_id: created_org.id,
            relationship_type: RelationshipType::WorksFor,
            weight: 1.0,
            confidence: 1.0,
            context: None,
            source_domain_id: None,
            created_at: chrono::Utc::now(),
        },
    ];

    let result = store.bulk_create_relationships(relationships.clone()).await.unwrap();

    assert_eq!(result.len(), 2);

    // Verify relationships were created
    for rel in &result {
        let found = store.get_relationship(rel.id).await.unwrap();
        assert!(found.is_some());
    }
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn bulk_create_relationships_empty_returns_empty() {
    let store = setup_test_store().await;

    let result = store.bulk_create_relationships(vec![]).await.unwrap();

    assert!(result.is_empty());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn bulk_delete_entities_removes_all() {
    let store = setup_test_store().await;

    // Create test entities
    let entities: Vec<Entity> = (0..5)
        .map(|_| Entity {
            id: Uuid::new_v4(),
            name: "To Delete".to_string(),
            entity_type: EntityType::Location,
            canonical_name: Some(format!("to-delete-{}", Uuid::new_v4())),
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        })
        .collect();

    let created = store.bulk_create_entities(entities).await.unwrap();
    let ids: Vec<Uuid> = created.iter().map(|e| e.id).collect();

    // Delete them
    let deleted_count = store.bulk_delete_entities(ids.clone()).await.unwrap();

    assert_eq!(deleted_count, 5);

    // Verify all are deleted
    for id in ids {
        let found = store.get_entity(id).await.unwrap();
        assert!(found.is_none());
    }
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn bulk_delete_entities_empty_returns_zero() {
    let store = setup_test_store().await;

    let result = store.bulk_delete_entities(vec![]).await.unwrap();

    assert_eq!(result, 0);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn bulk_delete_entities_non_existent_returns_zero() {
    let store = setup_test_store().await;

    let fake_ids = vec![
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    ];

    let result = store.bulk_delete_entities(fake_ids).await.unwrap();

    assert_eq!(result, 0);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn bulk_delete_entities_partial_deletes_succeeded() {
    let store = setup_test_store().await;

    // Create one real entity
    let entity = Entity {
        id: Uuid::new_v4(),
        name: "Real Entity".to_string(),
        entity_type: EntityType::Location,
        canonical_name: Some(format!("real-entity-{}", Uuid::new_v4())),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let created = store.create_entity(&entity).await.unwrap();

    // Mix real and fake IDs
    let ids = vec![
        created.id,
        Uuid::new_v4(), // Fake
        Uuid::new_v4(), // Fake
    ];

    let result = store.bulk_delete_entities(ids).await.unwrap();

    assert_eq!(result, 1); // Only the real one was deleted

    // Verify real entity is gone
    let found = store.get_entity(created.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn bulk_create_mixed_entity_types() {
    let store = setup_test_store().await;

    // Create entities of different types
    let entities = vec![
        Entity {
            id: Uuid::nil(),
            name: "Person".to_string(),
            entity_type: EntityType::Person,
            canonical_name: Some(format!("mixed-person-{}", Uuid::new_v4())),
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        },
        Entity {
            id: Uuid::nil(),
            name: "Organization".to_string(),
            entity_type: EntityType::Organization,
            canonical_name: Some(format!("mixed-org-{}", Uuid::new_v4())),
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        },
        Entity {
            id: Uuid::nil(),
            name: "Location".to_string(),
            entity_type: EntityType::Location,
            canonical_name: Some(format!("mixed-location-{}", Uuid::new_v4())),
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        },
    ];

    let result = store.bulk_create_entities(entities).await.unwrap();

    assert_eq!(result.len(), 3);

    // Verify each went to the correct collection
    assert_eq!(result[0].entity_type, EntityType::Person);
    assert_eq!(result[1].entity_type, EntityType::Organization);
    assert_eq!(result[2].entity_type, EntityType::Location);

    // Verify they can be retrieved
    for entity in &result {
        let found = store.get_entity(entity.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().entity_type, entity.entity_type);
    }
}
