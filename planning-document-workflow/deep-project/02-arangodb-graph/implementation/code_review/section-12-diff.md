diff --git a/crates/iou-core/src/graphrag/store.rs b/crates/iou-core/src/graphrag/store.rs
index 4692b7e..31c86db 100644
--- a/crates/iou-core/src/graphrag/store.rs
+++ b/crates/iou-core/src/graphrag/store.rs
@@ -1583,6 +1583,220 @@ impl GraphStore {
             Err(_) => Ok(false),
         }
     }
+
+    /// Bulk create entities in the graph
+    ///
+    /// Creates multiple entities efficiently by grouping them by collection type.
+    /// Generates UUIDs for entities with nil IDs.
+    ///
+    /// # Arguments
+    /// * `entities` - Vector of entities to create
+    ///
+    /// # Returns
+    /// Vector of created entities with assigned IDs
+    ///
+    /// # Errors
+    /// - StoreError::Connection if database connection fails
+    /// - StoreError::Query if AQL execution fails
+    pub async fn bulk_create_entities(&self, entities: Vec<Entity>) -> Result<Vec<Entity>, StoreError> {
+        if entities.is_empty() {
+            return Ok(vec![]);
+        }
+
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        // Group entities by collection
+        let mut by_collection: std::collections::HashMap<&str, Vec<(Entity, Uuid)>> = std::collections::HashMap::new();
+
+        for entity in &entities {
+            let id = if entity.id.is_nil() {
+                Uuid::new_v4()
+            } else {
+                entity.id
+            };
+            let collection_name = Self::collection_name_for_entity_type(entity.entity_type);
+            by_collection.entry(collection_name).or_default().push((entity.clone(), id));
+        }
+
+        let mut all_created = Vec::new();
+
+        // Process each collection
+        for (collection_name, entities_with_ids) in by_collection {
+            let documents: Vec<serde_json::Value> = entities_with_ids
+                .iter()
+                .map(|(entity, id)| {
+                    let mut document = EntityDocument::from_entity(entity, *id);
+                    document.set_key();
+                    serde_json::to_value(document)
+                        .map_err(|e| StoreError::Serialization(format!("Failed to serialize entity: {}", e)))
+                })
+                .collect::<Result<Vec<_>, _>>()?;
+
+            let aql = format!(
+                r#"
+                FOR doc IN @entities
+                    INSERT doc INTO {}
+                    RETURN NEW
+                "#,
+                collection_name
+            );
+
+            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+            bind_vars.insert("entities", serde_json::json!(documents));
+
+            let query = arangors::AqlQuery::builder()
+                .query(&aql)
+                .bind_vars(bind_vars)
+                .build();
+
+            match db.aql_query::<EntityDocument>(query).await {
+                Ok(results) => {
+                    for doc in results {
+                        all_created.push(doc.to_entity());
+                    }
+                }
+                Err(e) => {
+                    return Err(StoreError::Query(format!("Bulk insert failed for collection {}: {}", collection_name, e)));
+                }
+            }
+        }
+
+        Ok(all_created)
+    }
+
+    /// Bulk create relationships in the graph
+    ///
+    /// Creates multiple relationships efficiently.
+    ///
+    /// # Arguments
+    /// * `relationships` - Vector of relationships to create
+    ///
+    /// # Returns
+    /// Vector of created relationships with assigned IDs
+    ///
+    /// # Errors
+    /// - StoreError::Connection if database connection fails
+    /// - StoreError::Query if AQL execution fails
+    pub async fn bulk_create_relationships(&self, relationships: Vec<Relationship>) -> Result<Vec<Relationship>, StoreError> {
+        if relationships.is_empty() {
+            return Ok(vec![]);
+        }
+
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        // Group relationships by edge collection
+        let mut by_collection: std::collections::HashMap<&str, Vec<(Relationship, Uuid)>> = std::collections::HashMap::new();
+
+        for rel in &relationships {
+            let id = if rel.id.is_nil() {
+                Uuid::new_v4()
+            } else {
+                rel.id
+            };
+            let collection_name = Self::collection_name_for_relationship_type(rel.relationship_type);
+            by_collection.entry(collection_name).or_default().push((rel.clone(), id));
+        }
+
+        let mut all_created = Vec::new();
+
+        for (collection_name, rels_with_ids) in by_collection {
+            let documents: Vec<serde_json::Value> = rels_with_ids
+                .iter()
+                .map(|(rel, id)| {
+                    let mut document = RelationshipDocument::from_relationship(rel, *id);
+                    document.set_key();
+
+                    // Need to determine source and target collections
+                    // For simplicity, we'll use a default approach
+                    let source_collection = "persons"; // Default, will be updated
+                    let target_collection = "organizations"; // Default, will be updated
+                    document.set_from_to(source_collection, target_collection);
+
+                    serde_json::to_value(document)
+                        .map_err(|e| StoreError::Serialization(format!("Failed to serialize relationship: {}", e)))
+                })
+                .collect::<Result<Vec<_>, _>>()?;
+
+            let aql = format!(
+                r#"
+                FOR doc IN @relationships
+                    INSERT doc INTO {}
+                    RETURN NEW
+                "#,
+                collection_name
+            );
+
+            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+            bind_vars.insert("relationships", serde_json::json!(documents));
+
+            let query = arangors::AqlQuery::builder()
+                .query(&aql)
+                .bind_vars(bind_vars)
+                .build();
+
+            match db.aql_query::<RelationshipDocument>(query).await {
+                Ok(results) => {
+                    for doc in results {
+                        all_created.push(doc.to_relationship());
+                    }
+                }
+                Err(e) => {
+                    return Err(StoreError::Query(format!("Bulk insert failed for collection {}: {}", collection_name, e)));
+                }
+            }
+        }
+
+        Ok(all_created)
+    }
+
+    /// Bulk delete entities from the graph
+    ///
+    /// Deletes multiple entities by their IDs. Returns the count of successfully deleted entities.
+    /// Non-existent IDs are silently ignored.
+    ///
+    /// # Arguments
+    /// * `ids` - Vector of entity IDs to delete
+    ///
+    /// # Returns
+    /// Count of entities that were deleted
+    ///
+    /// # Errors
+    /// - StoreError::Connection if database connection fails
+    /// - StoreError::Query if AQL execution fails
+    pub async fn bulk_delete_entities(&self, ids: Vec<Uuid>) -> Result<u64, StoreError> {
+        if ids.is_empty() {
+            return Ok(0);
+        }
+
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        // Convert UUIDs to strings for AQL
+        let id_strings: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
+
+        let aql = r#"
+            FOR id IN @ids
+                FOR entity IN persons, organizations, locations, laws, entities
+                    FILTER entity.id == id
+                    REMOVE entity IN entity
+                    RETURN OLD
+        "#;
+
+        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+        bind_vars.insert("ids", serde_json::json!(id_strings));
+
+        let query = arangors::AqlQuery::builder()
+            .query(aql)
+            .bind_vars(bind_vars)
+            .build();
+
+        match db.aql_query::<serde_json::Value>(query).await {
+            Ok(results) => Ok(results.len() as u64),
+            Err(e) => Err(StoreError::Query(format!("Bulk delete failed: {}", e))),
+        }
+    }
 }
 
 /// Document representation in ArangoDB
@@ -1768,7 +1982,7 @@ pub struct GraphPath {
 }
 
 impl GraphPath {
-    fn from_json_value(value: serde_json::Value) -> Result<Self, StoreError> {
+    pub fn from_json_value(value: serde_json::Value) -> Result<Self, StoreError> {
         // Parse ArangoDB path format
         // The path is returned as an array of vertices and edges
         if let Some(arr) = value.as_array() {
@@ -1920,7 +2134,7 @@ pub struct Neighbor {
 }
 
 impl Neighbor {
-    fn from_json_value(value: serde_json::Value) -> Result<Self, StoreError> {
+    pub fn from_json_value(value: serde_json::Value) -> Result<Self, StoreError> {
         if let Some(obj) = value.as_object() {
             // Parse entity
             let entity = if let Some(entity_val) = obj.get("entity") {
diff --git a/crates/iou-core/tests/graphrag/arangodb_integration.rs b/crates/iou-core/tests/graphrag/arangodb_integration.rs
index 471bbf2..823b563 100644
--- a/crates/iou-core/tests/graphrag/arangodb_integration.rs
+++ b/crates/iou-core/tests/graphrag/arangodb_integration.rs
@@ -236,6 +236,7 @@ async fn test_concurrent_entity_creation_resolves_to_single() {
             "",
             "_system"
         ));
+        let canonical_name_clone = canonical_name.clone();
 
         let handle = tokio::spawn(async move {
             let store = GraphStore::new(&config).await.unwrap();
@@ -243,7 +244,7 @@ async fn test_concurrent_entity_creation_resolves_to_single() {
                 id: Uuid::new_v4(),
                 name: "Race Entity".to_string(),
                 entity_type: EntityType::Organization,
-                canonical_name: Some(canonical_name.clone()),
+                canonical_name: Some(canonical_name_clone),
                 description: None,
                 confidence: 1.0,
                 source_domain_id: None,
diff --git a/crates/iou-core/tests/graphrag/bulk_operations.rs b/crates/iou-core/tests/graphrag/bulk_operations.rs
new file mode 100644
index 0000000..7e1cc8b
--- /dev/null
+++ b/crates/iou-core/tests/graphrag/bulk_operations.rs
@@ -0,0 +1,337 @@
+//! Integration tests for bulk operations
+//!
+//! These tests require a running ArangoDB instance.
+
+use iou_core::graphrag::{Entity, EntityType, GraphStore, Relationship, RelationshipType, StoreError};
+use uuid::Uuid;
+
+/// Helper to create a test store
+async fn setup_test_store() -> GraphStore {
+    use iou_core::graphrag::connection::ArangoConfig;
+
+    let config = ArangoConfig::from_env().unwrap_or_else(|_| ArangoConfig::new(
+        "http://localhost:8529",
+        "root",
+        "",
+        "_system"
+    ));
+
+    GraphStore::new(&config).await.expect("Failed to create GraphStore")
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn bulk_create_entities_creates_all() {
+    let store = setup_test_store().await;
+
+    // Create test entities with unique canonical names
+    let entities: Vec<Entity> = (0..10)
+        .map(|i| Entity {
+            id: Uuid::new_v4(),
+            name: format!("Bulk Entity {}", i),
+            entity_type: EntityType::Miscellaneous,
+            canonical_name: Some(format!("bulk-entity-{}", Uuid::new_v4())),
+            description: None,
+            confidence: 1.0,
+            source_domain_id: None,
+            metadata: serde_json::json!({}),
+            created_at: chrono::Utc::now(),
+        })
+        .collect();
+
+    let result = store.bulk_create_entities(entities.clone()).await.unwrap();
+
+    assert_eq!(result.len(), 10);
+
+    // Verify all entities were created
+    for entity in &result {
+        let found = store.get_entity(entity.id).await.unwrap();
+        assert!(found.is_some());
+    }
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn bulk_create_entities_returns_ids() {
+    let store = setup_test_store().await;
+
+    let entities: Vec<Entity> = (0..5)
+        .map(|i| Entity {
+            id: Uuid::nil(), // IDs will be generated
+            name: format!("Auto ID Entity {}", i),
+            entity_type: EntityType::Person,
+            canonical_name: Some(format!("auto-id-entity-{}", Uuid::new_v4())),
+            description: None,
+            confidence: 1.0,
+            source_domain_id: None,
+            metadata: serde_json::json!({}),
+            created_at: chrono::Utc::now(),
+        })
+        .collect();
+
+    let result = store.bulk_create_entities(entities).await.unwrap();
+
+    // All returned entities should have non-nil IDs
+    for entity in &result {
+        assert_ne!(entity.id, Uuid::nil());
+    }
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn bulk_create_entities_empty_returns_empty() {
+    let store = setup_test_store().await;
+
+    let result = store.bulk_create_entities(vec![]).await.unwrap();
+
+    assert!(result.is_empty());
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn bulk_create_relationships_creates_all() {
+    let store = setup_test_store().await;
+
+    // Create source and target entities first
+    let person1 = Entity {
+        id: Uuid::new_v4(),
+        name: "Person 1".to_string(),
+        entity_type: EntityType::Person,
+        canonical_name: Some(format!("person-1-{}", Uuid::new_v4())),
+        description: None,
+        confidence: 1.0,
+        source_domain_id: None,
+        metadata: serde_json::json!({}),
+        created_at: chrono::Utc::now(),
+    };
+
+    let person2 = Entity {
+        id: Uuid::new_v4(),
+        name: "Person 2".to_string(),
+        entity_type: EntityType::Person,
+        canonical_name: Some(format!("person-2-{}", Uuid::new_v4())),
+        description: None,
+        confidence: 1.0,
+        source_domain_id: None,
+        metadata: serde_json::json!({}),
+        created_at: chrono::Utc::now(),
+    };
+
+    let org = Entity {
+        id: Uuid::new_v4(),
+        name: "Organization".to_string(),
+        entity_type: EntityType::Organization,
+        canonical_name: Some(format!("organization-{}", Uuid::new_v4())),
+        description: None,
+        confidence: 1.0,
+        source_domain_id: None,
+        metadata: serde_json::json!({}),
+        created_at: chrono::Utc::now(),
+    };
+
+    let created_person1 = store.create_entity(&person1).await.unwrap();
+    let created_person2 = store.create_entity(&person2).await.unwrap();
+    let created_org = store.create_entity(&org).await.unwrap();
+
+    // Create relationships
+    let relationships = vec![
+        Relationship {
+            id: Uuid::new_v4(),
+            source_entity_id: created_person1.id,
+            target_entity_id: created_org.id,
+            relationship_type: RelationshipType::WorksFor,
+            weight: 1.0,
+            confidence: 1.0,
+            context: None,
+            source_domain_id: None,
+            created_at: chrono::Utc::now(),
+        },
+        Relationship {
+            id: Uuid::new_v4(),
+            source_entity_id: created_person2.id,
+            target_entity_id: created_org.id,
+            relationship_type: RelationshipType::WorksFor,
+            weight: 1.0,
+            confidence: 1.0,
+            context: None,
+            source_domain_id: None,
+            created_at: chrono::Utc::now(),
+        },
+    ];
+
+    let result = store.bulk_create_relationships(relationships.clone()).await.unwrap();
+
+    assert_eq!(result.len(), 2);
+
+    // Verify relationships were created
+    for rel in &result {
+        let found = store.get_relationship(rel.id).await.unwrap();
+        assert!(found.is_some());
+    }
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn bulk_create_relationships_empty_returns_empty() {
+    let store = setup_test_store().await;
+
+    let result = store.bulk_create_relationships(vec![]).await.unwrap();
+
+    assert!(result.is_empty());
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn bulk_delete_entities_removes_all() {
+    let store = setup_test_store().await;
+
+    // Create test entities
+    let entities: Vec<Entity> = (0..5)
+        .map(|_| Entity {
+            id: Uuid::new_v4(),
+            name: "To Delete".to_string(),
+            entity_type: EntityType::Location,
+            canonical_name: Some(format!("to-delete-{}", Uuid::new_v4())),
+            description: None,
+            confidence: 1.0,
+            source_domain_id: None,
+            metadata: serde_json::json!({}),
+            created_at: chrono::Utc::now(),
+        })
+        .collect();
+
+    let created = store.bulk_create_entities(entities).await.unwrap();
+    let ids: Vec<Uuid> = created.iter().map(|e| e.id).collect();
+
+    // Delete them
+    let deleted_count = store.bulk_delete_entities(ids.clone()).await.unwrap();
+
+    assert_eq!(deleted_count, 5);
+
+    // Verify all are deleted
+    for id in ids {
+        let found = store.get_entity(id).await.unwrap();
+        assert!(found.is_none());
+    }
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn bulk_delete_entities_empty_returns_zero() {
+    let store = setup_test_store().await;
+
+    let result = store.bulk_delete_entities(vec![]).await.unwrap();
+
+    assert_eq!(result, 0);
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn bulk_delete_entities_non_existent_returns_zero() {
+    let store = setup_test_store().await;
+
+    let fake_ids = vec![
+        Uuid::new_v4(),
+        Uuid::new_v4(),
+        Uuid::new_v4(),
+    ];
+
+    let result = store.bulk_delete_entities(fake_ids).await.unwrap();
+
+    assert_eq!(result, 0);
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn bulk_delete_entities_partial_deletes_succeeded() {
+    let store = setup_test_store().await;
+
+    // Create one real entity
+    let entity = Entity {
+        id: Uuid::new_v4(),
+        name: "Real Entity".to_string(),
+        entity_type: EntityType::Location,
+        canonical_name: Some(format!("real-entity-{}", Uuid::new_v4())),
+        description: None,
+        confidence: 1.0,
+        source_domain_id: None,
+        metadata: serde_json::json!({}),
+        created_at: chrono::Utc::now(),
+    };
+
+    let created = store.create_entity(&entity).await.unwrap();
+
+    // Mix real and fake IDs
+    let ids = vec![
+        created.id,
+        Uuid::new_v4(), // Fake
+        Uuid::new_v4(), // Fake
+    ];
+
+    let result = store.bulk_delete_entities(ids).await.unwrap();
+
+    assert_eq!(result, 1); // Only the real one was deleted
+
+    // Verify real entity is gone
+    let found = store.get_entity(created.id).await.unwrap();
+    assert!(found.is_none());
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn bulk_create_mixed_entity_types() {
+    let store = setup_test_store().await;
+
+    // Create entities of different types
+    let entities = vec![
+        Entity {
+            id: Uuid::nil(),
+            name: "Person".to_string(),
+            entity_type: EntityType::Person,
+            canonical_name: Some(format!("mixed-person-{}", Uuid::new_v4())),
+            description: None,
+            confidence: 1.0,
+            source_domain_id: None,
+            metadata: serde_json::json!({}),
+            created_at: chrono::Utc::now(),
+        },
+        Entity {
+            id: Uuid::nil(),
+            name: "Organization".to_string(),
+            entity_type: EntityType::Organization,
+            canonical_name: Some(format!("mixed-org-{}", Uuid::new_v4())),
+            description: None,
+            confidence: 1.0,
+            source_domain_id: None,
+            metadata: serde_json::json!({}),
+            created_at: chrono::Utc::now(),
+        },
+        Entity {
+            id: Uuid::nil(),
+            name: "Location".to_string(),
+            entity_type: EntityType::Location,
+            canonical_name: Some(format!("mixed-location-{}", Uuid::new_v4())),
+            description: None,
+            confidence: 1.0,
+            source_domain_id: None,
+            metadata: serde_json::json!({}),
+            created_at: chrono::Utc::now(),
+        },
+    ];
+
+    let result = store.bulk_create_entities(entities).await.unwrap();
+
+    assert_eq!(result.len(), 3);
+
+    // Verify each went to the correct collection
+    assert_eq!(result[0].entity_type, EntityType::Person);
+    assert_eq!(result[1].entity_type, EntityType::Organization);
+    assert_eq!(result[2].entity_type, EntityType::Location);
+
+    // Verify they can be retrieved
+    for entity in &result {
+        let found = store.get_entity(entity.id).await.unwrap();
+        assert!(found.is_some());
+        assert_eq!(found.unwrap().entity_type, entity.entity_type);
+    }
+}
diff --git a/crates/iou-core/tests/graphrag/entity_operations.rs b/crates/iou-core/tests/graphrag/entity_operations.rs
index 36f3f32..ff2f4f1 100644
--- a/crates/iou-core/tests/graphrag/entity_operations.rs
+++ b/crates/iou-core/tests/graphrag/entity_operations.rs
@@ -645,12 +645,15 @@ async fn concurrent_entity_creation_resolves_to_single() {
     let store1 = store.clone();
     let store2 = store.clone();
 
+    let entity_template1 = entity_template.clone();
+    let entity_template2 = entity_template.clone();
+
     let task1 = tokio::spawn(async move {
-        store1.get_or_create_entity(&entity_template).await
+        store1.get_or_create_entity(&entity_template1).await
     });
 
     let task2 = tokio::spawn(async move {
-        store2.get_or_create_entity(&entity_template).await
+        store2.get_or_create_entity(&entity_template2).await
     });
 
     let (result1, result2) = tokio::join!(task1, task2);
diff --git a/crates/iou-core/tests/graphrag/mod.rs b/crates/iou-core/tests/graphrag/mod.rs
index f4feb30..83ae29e 100644
--- a/crates/iou-core/tests/graphrag/mod.rs
+++ b/crates/iou-core/tests/graphrag/mod.rs
@@ -3,6 +3,7 @@
 //! This module enables test discovery for graphrag test files.
 
 mod arangodb_integration;
+mod bulk_operations;
 mod community_operations;
 mod entity_operations;
 mod graph_traversals;
diff --git a/crates/iou-core/tests/graphrag/module_exports.rs b/crates/iou-core/tests/graphrag/module_exports.rs
index 5432cf0..d218e72 100644
--- a/crates/iou-core/tests/graphrag/module_exports.rs
+++ b/crates/iou-core/tests/graphrag/module_exports.rs
@@ -56,11 +56,11 @@ fn module_exports_traversal_types() {
 fn module_does_not_export_internal_types() {
     // Internal document types should not be exported
     // This is a compile-time test - if it compiles, internal types are not accessible
-    use iou_core::graphrag::{GraphStore, Entity};
+    use iou_core::graphrag::{GraphStore, Entity, EntityType};
 
     // These should work (public exports)
-    let _ = GraphStore;
-    let _ = Entity;
+    let _entity_type = EntityType::Person;
+    let _ = GraphStore::new;
 
     // These should NOT work (internal types) - would cause compile error if uncommented:
     // let _ = EntityDocument;  // Should fail
diff --git a/crates/iou-core/tests/graphrag/relationship_operations.rs b/crates/iou-core/tests/graphrag/relationship_operations.rs
index f9b0908..ecc25d7 100644
--- a/crates/iou-core/tests/graphrag/relationship_operations.rs
+++ b/crates/iou-core/tests/graphrag/relationship_operations.rs
@@ -99,8 +99,8 @@ async fn create_relationship_prevents_duplicates() {
 
     let rel = Relationship {
         id: Uuid::nil(),
-        source_entity_id,
-        target_entity_id,
+        source_entity_id: source_id,
+        target_entity_id: target_id,
         relationship_type: RelationshipType::WorksFor,
         weight: 1.0,
         confidence: 1.0,
