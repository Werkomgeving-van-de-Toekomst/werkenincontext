//! Integration tests for community operations
//!
//! These tests require a running ArangoDB instance.

use iou_core::graphrag::{
    Community, Entity, EntityType, GraphStore, StoreError
};
use uuid::Uuid;
use chrono::Utc;

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
async fn create_test_entities(store: &GraphStore) -> Vec<Entity> {
    let mut entities = Vec::new();

    for i in 0..3 {
        let entity = Entity {
            id: Uuid::new_v4(),
            name: format!("Entity {}", i),
            entity_type: EntityType::Organization,
            canonical_name: Some(format!("entity-{}", i)),
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };
        entities.push(store.create_entity(&entity).await.unwrap());
    }

    entities
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn create_community_creates_community_vertex() {
    let store = setup_test_store().await;

    let community = Community {
        id: Uuid::new_v4(),
        name: "Test Community".to_string(),
        description: Some("A test community".to_string()),
        level: 1,
        parent_community_id: None,
        member_entity_ids: vec![],
        summary: None,
        keywords: vec!["test".to_string(), "community".to_string()],
        created_at: Utc::now(),
    };

    let created = store.create_community(&community).await.unwrap();

    assert_eq!(created.name, "Test Community");
    assert_eq!(created.description, Some("A test community".to_string()));
    assert_eq!(created.level, 1);
    assert!(!created.id.is_nil());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn create_community_creates_membership_edges() {
    let store = setup_test_store().await;
    let entities = create_test_entities(&store).await;

    let community = Community {
        id: Uuid::new_v4(),
        name: "Test Community".to_string(),
        description: None,
        level: 1,
        parent_community_id: None,
        member_entity_ids: entities.iter().map(|e| e.id).collect(),
        summary: None,
        keywords: vec![],
        created_at: Utc::now(),
    };

    let created = store.create_community(&community).await.unwrap();

    // Check that members were added
    let members = store.get_community_members(created.id).await.unwrap();
    assert_eq!(members.len(), 3);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_community_found_returns_with_members() {
    let store = setup_test_store().await;
    let entities = create_test_entities(&store).await;

    let community = Community {
        id: Uuid::new_v4(),
        name: "Test Community".to_string(),
        description: None,
        level: 1,
        parent_community_id: None,
        member_entity_ids: entities.iter().map(|e| e.id).collect(),
        summary: None,
        keywords: vec![],
        created_at: Utc::now(),
    };

    let created = store.create_community(&community).await.unwrap();

    // Get community
    let found = store.get_community(created.id).await.unwrap();

    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.name, "Test Community");
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_community_not_found_returns_none() {
    let store = setup_test_store().await;

    let random_id = Uuid::new_v4();
    let found = store.get_community(random_id).await.unwrap();

    assert!(found.is_none());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn add_community_member_creates_edge() {
    let store = setup_test_store().await;
    let mut entities = create_test_entities(&store).await;

    let community = Community {
        id: Uuid::new_v4(),
        name: "Test Community".to_string(),
        description: None,
        level: 1,
        parent_community_id: None,
        member_entity_ids: vec![entities[0].id],
        summary: None,
        keywords: vec![],
        created_at: Utc::now(),
    };

    let created = store.create_community(&community).await.unwrap();

    // Add another member
    let result = store.add_community_member(created.id, entities[1].id).await.unwrap();
    assert!(result);

    // Verify both members are present
    let members = store.get_community_members(created.id).await.unwrap();
    assert_eq!(members.len(), 2);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn add_community_member_prevents_duplicates() {
    let store = setup_test_store().await;
    let entities = create_test_entities(&store).await;

    let community = Community {
        id: Uuid::new_v4(),
        name: "Test Community".to_string(),
        description: None,
        level: 1,
        parent_community_id: None,
        member_entity_ids: vec![entities[0].id],
        summary: None,
        keywords: vec![],
        created_at: Utc::now(),
    };

    let created = store.create_community(&community).await.unwrap();

    // Try to add same member again
    let result = store.add_community_member(created.id, entities[0].id).await.unwrap();
    assert!(!result);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn remove_community_member_removes_edge() {
    let store = setup_test_store().await;
    let entities = create_test_entities(&store).await;

    let community = Community {
        id: Uuid::new_v4(),
        name: "Test Community".to_string(),
        description: None,
        level: 1,
        parent_community_id: None,
        member_entity_ids: entities.iter().map(|e| e.id).collect(),
        summary: None,
        keywords: vec![],
        created_at: Utc::now(),
    };

    let created = store.create_community(&community).await.unwrap();

    // Remove first member
    let result = store.remove_community_member(created.id, entities[0].id).await.unwrap();
    assert!(result);

    // Verify only 2 members remain
    let members = store.get_community_members(created.id).await.unwrap();
    assert_eq!(members.len(), 2);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn remove_community_member_non_member_returns_error() {
    let store = setup_test_store().await;
    let entities = create_test_entities(&store).await;

    let community = Community {
        id: Uuid::new_v4(),
        name: "Test Community".to_string(),
        description: None,
        level: 1,
        parent_community_id: None,
        member_entity_ids: vec![entities[0].id],
        summary: None,
        keywords: vec![],
        created_at: Utc::now(),
    };

    let created = store.create_community(&community).await.unwrap();

    // Try to remove non-member (entities[2] was not added)
    let result = store.remove_community_member(created.id, entities[2].id).await.unwrap();
    assert!(!result);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_community_members_returns_all_members() {
    let store = setup_test_store().await;
    let entities = create_test_entities(&store).await;

    let community = Community {
        id: Uuid::new_v4(),
        name: "Test Community".to_string(),
        description: None,
        level: 1,
        parent_community_id: None,
        member_entity_ids: entities.iter().map(|e| e.id).collect(),
        summary: None,
        keywords: vec![],
        created_at: Utc::now(),
    };

    let created = store.create_community(&community).await.unwrap();

    let members = store.get_community_members(created.id).await.unwrap();

    assert_eq!(members.len(), 3);

    // Verify entity IDs match
    let member_ids: Vec<_> = members.iter().map(|m| m.id).collect();
    for entity in &entities {
        assert!(member_ids.contains(&entity.id));
    }
}
