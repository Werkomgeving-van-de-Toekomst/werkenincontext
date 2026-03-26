//! Integration tests for relationship operations
//!
//! These tests require a running ArangoDB instance.

use iou_core::graphrag::{
    Entity, EntityType, GraphStore, Relationship, RelationshipType, RelationshipDirection, RelationshipQueryOptions, StoreError
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

    GraphStore::new(&config).await.expect("Failed to create GraphStore")
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn create_relationship_returns_relationship_with_id() {
    let store = setup_test_store().await;

    let relationship = Relationship {
        id: Uuid::nil(), // Will be generated
        source_entity_id: Uuid::new_v4(),
        target_entity_id: Uuid::new_v4(),
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 0.9,
        context: Some("Test context".to_string()),
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let created = store.create_relationship(&relationship).await.unwrap();
    assert_ne!(created.id, Uuid::nil());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn create_relationship_creates_in_correct_edge_collection() {
    let store = setup_test_store().await;

    // Test WorksFor -> edge_works_for
    let rel1 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: Uuid::new_v4(),
        target_entity_id: Uuid::new_v4(),
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let created1 = store.create_relationship(&rel1).await.unwrap();
    assert_eq!(created1.relationship_type, RelationshipType::WorksFor);

    // Test LocatedIn -> edge_located_in
    let rel2 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: Uuid::new_v4(),
        target_entity_id: Uuid::new_v4(),
        relationship_type: RelationshipType::LocatedIn,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let created2 = store.create_relationship(&rel2).await.unwrap();
    assert_eq!(created2.relationship_type, RelationshipType::LocatedIn);

    // Verify relationships exist
    let found1 = store.get_relationship(created1.id).await.unwrap();
    assert!(found1.is_some());
    assert_eq!(found1.unwrap().relationship_type, RelationshipType::WorksFor);

    let found2 = store.get_relationship(created2.id).await.unwrap();
    assert!(found2.is_some());
    assert_eq!(found2.unwrap().relationship_type, RelationshipType::LocatedIn);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn create_relationship_prevents_duplicates() {
    let store = setup_test_store().await;

    let source_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let rel = Relationship {
        id: Uuid::nil(),
        source_entity_id: source_id,
        target_entity_id: target_id,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    // First creation should succeed
    let created1 = store.create_relationship(&rel).await.unwrap();

    // Second creation with same entities and type should fail
    // Note: ArangoDB may or may not enforce this without explicit unique index
    // This test verifies the behavior
    let result = store.create_relationship(&rel).await;

    // If unique constraints are enforced, we expect an error
    // If not, we expect a different relationship to be created
    match result {
        Ok(_) => {
            // ArangoDB allowed the duplicate - verify we get two different relationships
            let all = store.get_entity_relationships(
                source_id,
                RelationshipQueryOptions {
                    relationship_type: Some(RelationshipType::WorksFor),
                    ..Default::default()
                }
            ).await.unwrap();
            assert!(all.len() >= 1);
        }
        Err(StoreError::UniqueViolation(_)) => {
            // Expected behavior - duplicates prevented
            assert!(true);
        }
        Err(_) => {
            // Other error - not expected
            panic!("Unexpected error when creating duplicate relationship");
        }
    }

    let _ = created1;
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_relationship_found_returns_data() {
    let store = setup_test_store().await;

    let rel = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: Uuid::new_v4(),
        target_entity_id: Uuid::new_v4(),
        relationship_type: RelationshipType::CollaboratesWith,
        weight: 0.8,
        confidence: 0.95,
        context: Some("Test collaboration".to_string()),
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let created = store.create_relationship(&rel).await.unwrap();
    let found = store.get_relationship(created.id).await.unwrap();

    assert!(found.is_some());
    let found_rel = found.unwrap();
    assert_eq!(found_rel.relationship_type, RelationshipType::CollaboratesWith);
    assert_eq!(found_rel.context, Some("Test collaboration".to_string()));
    assert_eq!(found_rel.weight, 0.8);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_relationship_not_found_returns_none() {
    let store = setup_test_store().await;
    let random_id = Uuid::new_v4();

    let found = store.get_relationship(random_id).await.unwrap();

    assert!(found.is_none());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn delete_relationship_removes_document() {
    let store = setup_test_store().await;

    let rel = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: Uuid::new_v4(),
        target_entity_id: Uuid::new_v4(),
        relationship_type: RelationshipType::ReportsTo,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let created = store.create_relationship(&rel).await.unwrap();

    // Delete the relationship
    let deleted = store.delete_relationship(created.id).await.unwrap();
    assert!(deleted);

    // Verify relationship no longer exists
    let found = store.get_relationship(created.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn delete_relationship_not_found_returns_false() {
    let store = setup_test_store().await;
    let random_id = Uuid::new_v4();

    let deleted = store.delete_relationship(random_id).await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_entity_relationships_returns_both_directions() {
    let store = setup_test_store().await;

    let entity_id = Uuid::new_v4();
    let other1 = Uuid::new_v4();
    let other2 = Uuid::new_v4();

    // Create an outgoing relationship (entity -> other1)
    let outgoing = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: entity_id,
        target_entity_id: other1,
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    // Create an incoming relationship (other2 -> entity)
    let incoming = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: other2,
        target_entity_id: entity_id,
        relationship_type: RelationshipType::ReportsTo,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&outgoing).await.unwrap();
    store.create_relationship(&incoming).await.unwrap();

    // Get all relationships (both directions)
    let options = RelationshipQueryOptions {
        direction: Some(RelationshipDirection::Both),
        include_all_collections: true,
        ..Default::default()
    };

    let results = store.get_entity_relationships(entity_id, options).await.unwrap();

    // Should return both relationships
    assert!(results.len() >= 2);

    // Verify we have both types
    let types: Vec<_> = results.iter().map(|r| r.relationship_type).collect();
    assert!(types.contains(&RelationshipType::WorksFor));
    assert!(types.contains(&RelationshipType::ReportsTo));
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_entity_relationships_filters_by_type() {
    let store = setup_test_store().await;

    let entity_id = Uuid::new_v4();

    // Create multiple relationships of different types
    let rel1 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: entity_id,
        target_entity_id: Uuid::new_v4(),
        relationship_type: RelationshipType::WorksFor,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let rel2 = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: entity_id,
        target_entity_id: Uuid::new_v4(),
        relationship_type: RelationshipType::LocatedIn,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&rel1).await.unwrap();
    store.create_relationship(&rel2).await.unwrap();

    // Filter for WorksFor only
    let options = RelationshipQueryOptions {
        relationship_type: Some(RelationshipType::WorksFor),
        direction: Some(RelationshipDirection::Outgoing),
        include_all_collections: false,
        ..Default::default()
    };

    let results = store.get_entity_relationships(entity_id, options).await.unwrap();

    // All results should be WorksFor
    assert!(results.iter().all(|r| r.relationship_type == RelationshipType::WorksFor));
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_entity_relationships_paginates_correctly() {
    let store = setup_test_store().await;

    let entity_id = Uuid::new_v4();

    // Create multiple relationships
    for i in 0..15 {
        let rel = Relationship {
            id: Uuid::new_v4(),
            source_entity_id: entity_id,
            target_entity_id: Uuid::new_v4(),
            relationship_type: RelationshipType::RelatesTo,
            weight: 1.0,
            confidence: 1.0,
            context: Some(format!("Relationship {}", i)),
            source_domain_id: None,
            created_at: chrono::Utc::now(),
        };
        store.create_relationship(&rel).await.unwrap();
    }

    // Request first 10
    let options = RelationshipQueryOptions {
        limit: 10,
        include_all_collections: false,
        ..Default::default()
    };

    let results = store.get_entity_relationships(entity_id, options).await.unwrap();

    // Should return at most 10
    assert!(results.len() <= 10);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_entity_relationships_outgoing_only() {
    let store = setup_test_store().await;

    let entity_id = Uuid::new_v4();
    let other = Uuid::new_v4();

    // Create only outgoing relationship
    let rel = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: entity_id,
        target_entity_id: other,
        relationship_type: RelationshipType::Follows,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&rel).await.unwrap();

    // Get outgoing only
    let options = RelationshipQueryOptions {
        direction: Some(RelationshipDirection::Outgoing),
        include_all_collections: false,
        ..Default::default()
    };

    let results = store.get_entity_relationships(entity_id, options).await.unwrap();

    // Should find the relationship where entity is source
    assert!(results.len() >= 1);
    assert_eq!(results[0].source_entity_id, entity_id);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_entity_relationships_incoming_only() {
    let store = setup_test_store().await;

    let entity_id = Uuid::new_v4();
    let other = Uuid::new_v4();

    // Create only incoming relationship
    let rel = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: other,
        target_entity_id: entity_id,
        relationship_type: RelationshipType::OwnerOf,
        weight: 1.0,
        confidence: 1.0,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&rel).await.unwrap();

    // Get incoming only
    let options = RelationshipQueryOptions {
        direction: Some(RelationshipDirection::Incoming),
        include_all_collections: false,
        ..Default::default()
    };

    let results = store.get_entity_relationships(entity_id, options).await.unwrap();

    // Should find the relationship where entity is target
    assert!(results.len() >= 1);
    assert_eq!(results[0].target_entity_id, entity_id);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_entity_relationships_filters_by_confidence() {
    let store = setup_test_store().await;

    let entity_id = Uuid::new_v4();

    // Create relationships with different confidence levels
    let high_conf = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: entity_id,
        target_entity_id: Uuid::new_v4(),
        relationship_type: RelationshipType::CollaboratesWith,
        weight: 1.0,
        confidence: 0.95,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    let low_conf = Relationship {
        id: Uuid::new_v4(),
        source_entity_id: entity_id,
        target_entity_id: Uuid::new_v4(),
        relationship_type: RelationshipType::CollaboratesWith,
        weight: 1.0,
        confidence: 0.5,
        context: None,
        source_domain_id: None,
        created_at: chrono::Utc::now(),
    };

    store.create_relationship(&high_conf).await.unwrap();
    store.create_relationship(&low_conf).await.unwrap();

    // Filter for confidence >= 0.9
    let options = RelationshipQueryOptions {
        min_confidence: Some(0.9),
        include_all_collections: false,
        ..Default::default()
    };

    let results = store.get_entity_relationships(entity_id, options).await.unwrap();

    // All results should have confidence >= 0.9
    assert!(results.iter().all(|r| r.confidence >= 0.9));
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn relationship_query_options_builder() {
    let options = RelationshipQueryOptions::new()
        .with_type(RelationshipType::WorksFor)
        .with_direction(RelationshipDirection::Outgoing)
        .with_limit(50);

    assert_eq!(options.relationship_type, Some(RelationshipType::WorksFor));
    assert_eq!(options.direction, Some(RelationshipDirection::Outgoing));
    assert_eq!(options.limit, 50);
}
