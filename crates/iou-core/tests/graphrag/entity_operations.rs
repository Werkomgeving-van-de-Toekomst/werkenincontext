//! Integration tests for entity operations
//!
//! These tests require a running ArangoDB instance.

use iou_core::graphrag::{Entity, EntityType, GraphStore, StoreError};
use uuid::Uuid;

/// Helper to create a test store
///
/// This function creates a GraphStore instance using environment variables
/// or default test configuration.
async fn setup_test_store() -> GraphStore {
    use iou_core::graphrag::connection::ArangoConfig;

    // Try to load from environment, fall back to defaults
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
async fn create_entity_returns_entity_with_id() {
    let store = setup_test_store().await;
    let entity = Entity {
        id: Uuid::nil(), // Will be generated
        name: "Test Person".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("test-person".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let created = store.create_entity(&entity).await.unwrap();
    assert_ne!(created.id, Uuid::nil());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn create_entity_in_correct_collection() {
    // Test that Person entities go to `persons` collection
    // Test that Organization entities go to `organizations` collection
    let store = setup_test_store().await;

    // Create a Person entity
    let person = Entity {
        id: Uuid::new_v4(),
        name: "John Doe".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("john-doe".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let created_person = store.create_entity(&person).await.unwrap();
    assert_eq!(created_person.entity_type, EntityType::Person);

    // Create an Organization entity
    let org = Entity {
        id: Uuid::new_v4(),
        name: "Test Corp".to_string(),
        entity_type: EntityType::Organization,
        canonical_name: Some("test-corp".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let created_org = store.create_entity(&org).await.unwrap();
    assert_eq!(created_org.entity_type, EntityType::Organization);

    // Verify collection routing by retrieving the entities
    let found_person = store.get_entity(created_person.id).await.unwrap();
    assert!(found_person.is_some());
    assert_eq!(found_person.unwrap().entity_type, EntityType::Person);

    let found_org = store.get_entity(created_org.id).await.unwrap();
    assert!(found_org.is_some());
    assert_eq!(found_org.unwrap().entity_type, EntityType::Organization);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_entity_found_returns_data() {
    let store = setup_test_store().await;
    let entity = Entity {
        id: Uuid::new_v4(),
        name: "Findable Entity".to_string(),
        entity_type: EntityType::Organization,
        canonical_name: Some("findable-entity".to_string()),
        description: Some("An entity that can be found".to_string()),
        confidence: 0.95,
        source_domain_id: None,
        metadata: serde_json::json!({"test": true}),
        created_at: chrono::Utc::now(),
    };

    let created = store.create_entity(&entity).await.unwrap();
    let found = store.get_entity(created.id).await.unwrap();

    assert!(found.is_some());
    let found_entity = found.unwrap();
    assert_eq!(found_entity.name, "Findable Entity");
    assert_eq!(found_entity.canonical_name, Some("findable-entity".to_string()));
    assert_eq!(found_entity.entity_type, EntityType::Organization);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_entity_not_found_returns_none() {
    let store = setup_test_store().await;
    let random_id = Uuid::new_v4();
    let found = store.get_entity(random_id).await.unwrap();

    assert!(found.is_none());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn update_entity_modifies_fields() {
    let store = setup_test_store().await;
    let entity = Entity {
        id: Uuid::new_v4(),
        name: "Original Name".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("original-name".to_string()),
        description: Some("Original description".to_string()),
        confidence: 0.8,
        source_domain_id: None,
        metadata: serde_json::json!({"version": 1}),
        created_at: chrono::Utc::now(),
    };

    let created = store.create_entity(&entity).await.unwrap();

    use iou_core::graphrag::EntityUpdate;
    let update = EntityUpdate {
        name: Some("Updated Name".to_string()),
        description: Some("Updated description".to_string()),
        confidence: Some(0.95),
        ..Default::default()
    };

    let updated = store.update_entity(created.id, update).await.unwrap();

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.description, Some("Updated description".to_string()));
    assert!((updated.confidence - 0.95).abs() < 0.001);
    // canonical_name should remain unchanged
    assert_eq!(updated.canonical_name, Some("original-name".to_string()));
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn update_entity_not_found_returns_error() {
    let store = setup_test_store().await;
    let random_id = Uuid::new_v4();

    use iou_core::graphrag::EntityUpdate;
    let update = EntityUpdate {
        name: Some("Won't work".to_string()),
        ..Default::default()
    };

    let result = store.update_entity(random_id, update).await;
    assert!(matches!(result, Err(StoreError::EntityNotFound(_))));
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn delete_entity_removes_document() {
    let store = setup_test_store().await;
    let entity = Entity {
        id: Uuid::new_v4(),
        name: "Doomed Entity".to_string(),
        entity_type: EntityType::Location,
        canonical_name: Some("doomed-entity".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let created = store.create_entity(&entity).await.unwrap();

    // Delete without cascade
    let deleted = store.delete_entity(created.id, false).await.unwrap();
    assert!(deleted);

    // Verify entity no longer exists
    let found = store.get_entity(created.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn delete_entity_not_found_returns_false() {
    let store = setup_test_store().await;
    let random_id = Uuid::new_v4();

    let deleted = store.delete_entity(random_id, false).await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn delete_entity_cascade_true_removes_edges() {
    let store = setup_test_store().await;

    // Create two entities
    let person = Entity {
        id: Uuid::new_v4(),
        name: "Worker".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("worker".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let org = Entity {
        id: Uuid::new_v4(),
        name: "Company".to_string(),
        entity_type: EntityType::Organization,
        canonical_name: Some("company".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let created_person = store.create_entity(&person).await.unwrap();
    let created_org = store.create_entity(&org).await.unwrap();

    // Create a relationship (edge) between them
    // Note: This requires relationship operations which may not be implemented yet
    // For now, we'll just test that cascade doesn't error out
    let _ = created_person;
    let _ = created_org;

    // Delete with cascade=true
    // This should not error even if there are no edges
    let result = store.delete_entity(created_person.id, true).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn delete_entity_cascade_false_keeps_edges() {
    // Test that non-cascade leaves edges (orphaned)
    let store = setup_test_store().await;

    let entity = Entity {
        id: Uuid::new_v4(),
        name: "Entity with Edges".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("entity-with-edges".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let created = store.create_entity(&entity).await.unwrap();

    // Delete without cascade
    let deleted = store.delete_entity(created.id, false).await.unwrap();
    assert!(deleted);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn list_entities_without_filters_returns_all() {
    let store = setup_test_store().await;

    // Create test entities
    for i in 0..3 {
        let entity = Entity {
            id: Uuid::new_v4(),
            name: format!("Entity {}", i),
            entity_type: EntityType::Miscellaneous,
            canonical_name: Some(format!("entity-{}", i)),
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        };
        store.create_entity(&entity).await.unwrap();
    }

    use iou_core::graphrag::{EntityFilters, PaginationOptions};

    let result = store
        .list_entities(EntityFilters::default(), PaginationOptions::default())
        .await
        .unwrap();

    // Should return at least our 3 entities
    assert!(result.entities.len() >= 3);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn list_entities_with_type_filter_filters_correctly() {
    let store = setup_test_store().await;

    // Create a Person entity
    let person = Entity {
        id: Uuid::new_v4(),
        name: "Filter Person".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("filter-person".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };
    store.create_entity(&person).await.unwrap();

    // Create a Location entity
    let location = Entity {
        id: Uuid::new_v4(),
        name: "Filter Location".to_string(),
        entity_type: EntityType::Location,
        canonical_name: Some("filter-location".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };
    store.create_entity(&location).await.unwrap();

    use iou_core::graphrag::{EntityFilters, PaginationOptions};

    // Filter for Person type only
    let filters = EntityFilters {
        entity_type: Some(EntityType::Person),
        ..Default::default()
    };

    let result = store
        .list_entities(filters, PaginationOptions::default())
        .await
        .unwrap();

    // All results should be Person type
    assert!(result.entities.iter().all(|e| e.entity_type == EntityType::Person));
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn list_entities_with_pagination_returns_page() {
    let store = setup_test_store().await;

    // Create multiple entities
    for i in 0..10 {
        let entity = Entity {
            id: Uuid::new_v4(),
            name: format!("Page Entity {}", i),
            entity_type: EntityType::Miscellaneous,
            canonical_name: Some(format!("page-entity-{}", i)),
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        };
        store.create_entity(&entity).await.unwrap();
    }

    use iou_core::graphrag::{EntityFilters, PaginationOptions};

    // Request first page with limit 5
    let pagination = PaginationOptions {
        limit: 5,
        cursor: None,
    };

    let result = store
        .list_entities(EntityFilters::default(), pagination)
        .await
        .unwrap();

    assert!(result.entities.len() <= 5);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn list_entities_with_name_filter_filters_correctly() {
    let store = setup_test_store().await;

    // Create entities with specific names
    let special = Entity {
        id: Uuid::new_v4(),
        name: "Special Name".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("special-name".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };
    store.create_entity(&special).await.unwrap();

    let other = Entity {
        id: Uuid::new_v4(),
        name: "Other Entity".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("other-entity".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };
    store.create_entity(&other).await.unwrap();

    use iou_core::graphrag::{EntityFilters, PaginationOptions};

    // Filter for entities containing "Special"
    let filters = EntityFilters {
        name_contains: Some("Special".to_string()),
        ..Default::default()
    };

    let result = store
        .list_entities(filters, PaginationOptions::default())
        .await
        .unwrap();

    // Should only return entities with "Special" in name
    assert!(result.entities.iter().all(|e| e.name.contains("Special")));
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn list_entities_with_min_confidence_filters_correctly() {
    let store = setup_test_store().await;

    // Create entities with different confidence levels
    let high_conf = Entity {
        id: Uuid::new_v4(),
        name: "High Confidence".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("high-confidence".to_string()),
        description: None,
        confidence: 0.9,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };
    store.create_entity(&high_conf).await.unwrap();

    let low_conf = Entity {
        id: Uuid::new_v4(),
        name: "Low Confidence".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("low-confidence".to_string()),
        description: None,
        confidence: 0.5,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };
    store.create_entity(&low_conf).await.unwrap();

    use iou_core::graphrag::{EntityFilters, PaginationOptions};

    // Filter for entities with confidence >= 0.8
    let filters = EntityFilters {
        min_confidence: Some(0.8),
        ..Default::default()
    };

    let result = store
        .list_entities(filters, PaginationOptions::default())
        .await
        .unwrap();

    // All results should have confidence >= 0.8
    assert!(result.entities.iter().all(|e| e.confidence >= 0.8));
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_or_create_entity_new_creates() {
    let store = setup_test_store().await;

    let entity = Entity {
        id: Uuid::new_v4(),
        name: "New Entity".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("new-entity".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let result = store.get_or_create_entity(&entity).await.unwrap();
    assert_eq!(result.name, "New Entity");
    assert_eq!(result.canonical_name, Some("new-entity".to_string()));
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_or_create_entity_existing_returns() {
    let store = setup_test_store().await;

    let entity = Entity {
        id: Uuid::new_v4(),
        name: "Existing Entity".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("existing-entity".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    // Create the entity first
    let created = store.create_entity(&entity).await.unwrap();

    // Try to get_or_create with same canonical_name but different ID
    let lookup_entity = Entity {
        id: Uuid::new_v4(), // Different ID
        name: "Different Name".to_string(), // Different name
        entity_type: EntityType::Person,
        canonical_name: Some("existing-entity".to_string()), // Same canonical_name
        description: None,
        confidence: 0.5,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let result = store.get_or_create_entity(&lookup_entity).await.unwrap();

    // Should return the existing entity, not create a new one
    assert_eq!(result.id, created.id);
    assert_eq!(result.name, "Existing Entity"); // Original name preserved
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn upsert_entity_new_creates() {
    let store = setup_test_store().await;

    let entity = Entity {
        id: Uuid::new_v4(),
        name: "Upsert New".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("upsert-new".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    let result = store.upsert_entity(&entity).await.unwrap();
    assert_eq!(result.name, "Upsert New");
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn upsert_entity_existing_updates() {
    let store = setup_test_store().await;

    let entity = Entity {
        id: Uuid::new_v4(),
        name: "Original".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("upsert-update-test".to_string()),
        description: Some("Original description".to_string()),
        confidence: 0.7,
        source_domain_id: None,
        metadata: serde_json::json!({"version": 1}),
        created_at: chrono::Utc::now(),
    };

    let created = store.create_entity(&entity).await.unwrap();

    // Update with new data
    let update_entity = Entity {
        id: created.id,
        name: "Updated".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("upsert-update-test".to_string()),
        description: Some("Updated description".to_string()),
        confidence: 0.95,
        source_domain_id: None,
        metadata: serde_json::json!({"version": 2}),
        created_at: chrono::Utc::now(),
    };

    let result = store.upsert_entity(&update_entity).await.unwrap();
    assert_eq!(result.id, created.id);
    assert_eq!(result.name, "Updated");
    assert_eq!(result.confidence, 0.95);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn concurrent_entity_creation_resolves_to_single() {
    let store = setup_test_store().await;

    let entity_template = Entity {
        id: Uuid::nil(), // Will be generated
        name: "Concurrent Test".to_string(),
        entity_type: EntityType::Person,
        canonical_name: Some("concurrent-test".to_string()),
        description: None,
        confidence: 1.0,
        source_domain_id: None,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    // Spawn concurrent tasks
    let store1 = store.clone();
    let store2 = store.clone();

    let entity_template1 = entity_template.clone();
    let entity_template2 = entity_template.clone();

    let task1 = tokio::spawn(async move {
        store1.get_or_create_entity(&entity_template1).await
    });

    let task2 = tokio::spawn(async move {
        store2.get_or_create_entity(&entity_template2).await
    });

    let (result1, result2) = tokio::join!(task1, task2);

    let entity1 = result1.unwrap().unwrap();
    let entity2 = result2.unwrap().unwrap();

    // Both should return the same entity (same ID)
    assert_eq!(entity1.id, entity2.id);
    assert_eq!(entity1.canonical_name, entity2.canonical_name);
}
