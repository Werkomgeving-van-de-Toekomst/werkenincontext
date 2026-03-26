diff --git a/crates/iou-core/src/graphrag/store.rs b/crates/iou-core/src/graphrag/store.rs
index 20f2909..c3160ad 100644
--- a/crates/iou-core/src/graphrag/store.rs
+++ b/crates/iou-core/src/graphrag/store.rs
@@ -4,6 +4,7 @@
 //! with intelligent routing to type-specific collections.
 
 use std::collections::HashMap;
+use chrono::Utc;
 use mobc::Pool;
 use serde::{Deserialize, Serialize};
 use uuid::Uuid;
@@ -11,11 +12,11 @@ use uuid::Uuid;
 use crate::graphrag::{
     connection::{ArangorsConnectionManager, ArangoConfig},
     error::StoreError,
-    types::{Entity, EntityType, Relationship, RelationshipType},
+    types::{Community, Entity, EntityType, Relationship, RelationshipType},
 };
 
 /// All vertex collections for entities
-const VERTEX_COLLECTIONS: &[&str] = &["persons", "organizations", "locations", "laws", "entities"];
+const VERTEX_COLLECTIONS: &[&str] = &["persons", "organizations", "locations", "laws", "entities", "communities"];
 
 /// All edge collections for relationships
 const EDGE_COLLECTIONS: &[&str] = &[
@@ -30,6 +31,8 @@ const EDGE_COLLECTIONS: &[&str] = &[
     "edge_follows",
     "edge_part_of",
     "edge_unknown",
+    "edge_member_of",
+    "edge_subcommunity",
 ];
 
 /// Graph store for ArangoDB persistence layer
@@ -1320,6 +1323,264 @@ impl GraphStore {
             Err(e) => Err(StoreError::from(e)),
         }
     }
+
+    // ========== Community Operations ==========
+
+    /// Create a new community
+    ///
+    /// Creates a community vertex and optionally creates membership edges
+    /// for entities specified in member_entity_ids.
+    ///
+    /// # Arguments
+    /// * `community` - Community to create (id field optional, will be generated)
+    ///
+    /// # Returns
+    /// Result containing the created community with assigned ID
+    pub async fn create_community(&self, community: &Community) -> Result<Community, StoreError> {
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        let id = if community.id.is_nil() {
+            Uuid::new_v4()
+        } else {
+            community.id
+        };
+
+        let aql = r#"
+            INSERT @community INTO communities
+            RETURN NEW
+        "#;
+
+        let mut document = CommunityDocument::from_community(community, id);
+        document.set_key();
+
+        let community_json = serde_json::to_value(&document)
+            .map_err(|e| StoreError::Serialization(format!("Failed to serialize community: {}", e)))?;
+
+        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+        bind_vars.insert("community", community_json);
+
+        let query = arangors::AqlQuery::builder()
+            .query(aql)
+            .bind_vars(bind_vars)
+            .build();
+
+        match db.aql_query::<CommunityDocument>(query).await {
+            Ok(mut results) => {
+                let created = results.pop()
+                    .ok_or_else(|| StoreError::Query("INSERT returned no result".to_string()))?
+                    .to_community();
+
+                // Create membership edges for initial members
+                for member_id in &community.member_entity_ids {
+                    let _ = self.add_community_member(created.id, *member_id).await;
+                }
+
+                Ok(created)
+            }
+            Err(e) => {
+                let e_str = e.to_string();
+                if e_str.contains("unique") || e_str.contains("duplicate") || e_str.contains("1202") {
+                    Err(StoreError::UniqueViolation(format!(
+                        "Community with name '{}' already exists",
+                        community.name
+                    )))
+                } else {
+                    Err(StoreError::from(e))
+                }
+            }
+        }
+    }
+
+    /// Get a community by ID
+    ///
+    /// # Arguments
+    /// * `id` - UUID of the community to retrieve
+    ///
+    /// # Returns
+    /// - Ok(Some(Community)) if community found
+    /// - Ok(None) if community not found
+    pub async fn get_community(&self, id: Uuid) -> Result<Option<Community>, StoreError> {
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        let aql = r#"
+            FOR c IN communities
+            FILTER c.id == @id
+            LIMIT 1
+            RETURN c
+        "#;
+
+        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+        bind_vars.insert("id", serde_json::json!(id.to_string()));
+
+        let query = arangors::AqlQuery::builder()
+            .query(aql)
+            .bind_vars(bind_vars)
+            .build();
+
+        match db.aql_query::<CommunityDocument>(query).await {
+            Ok(mut results) => Ok(results.pop().map(|doc| doc.to_community())),
+            Err(_) => Ok(None),
+        }
+    }
+
+    /// Add an entity as a member of a community
+    ///
+    /// Creates an edge_member_of edge from the entity to the community.
+    ///
+    /// # Arguments
+    /// * `community_id` - UUID of the community
+    /// * `entity_id` - UUID of the entity to add as member
+    ///
+    /// # Returns
+    /// - Ok(true) if member was added
+    /// - Ok(false) if already a member
+    pub async fn add_community_member(&self, community_id: Uuid, entity_id: Uuid) -> Result<bool, StoreError> {
+        // Check if already a member
+        if self.is_community_member(community_id, entity_id).await? {
+            return Ok(false);
+        }
+
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        let aql = r#"
+            INSERT @membership INTO edge_member_of
+            RETURN NEW
+        "#;
+
+        let membership = MembershipEdge {
+            _key: None,
+            id: Uuid::new_v4(),
+            community_id,
+            entity_id,
+            created_at: Utc::now().to_rfc3339(),
+        };
+
+        let membership_json = serde_json::to_value(&membership)
+            .map_err(|e| StoreError::Serialization(format!("Failed to serialize membership: {}", e)))?;
+
+        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+        bind_vars.insert("membership", membership_json);
+
+        let query = arangors::AqlQuery::builder()
+            .query(aql)
+            .bind_vars(bind_vars)
+            .build();
+
+        match db.aql_query::<serde_json::Value>(query).await {
+            Ok(_) => Ok(true),
+            Err(e) => {
+                let e_str = e.to_string();
+                if e_str.contains("unique") || e_str.contains("duplicate") {
+                    Ok(false) // Already a member
+                } else {
+                    Err(StoreError::from(e))
+                }
+            }
+        }
+    }
+
+    /// Remove an entity from a community
+    ///
+    /// Removes the edge_member_of edge.
+    ///
+    /// # Arguments
+    /// * `community_id` - UUID of the community
+    /// * `entity_id` - UUID of the entity to remove
+    ///
+    /// # Returns
+    /// - Ok(true) if member was removed
+    /// - Ok(false) if entity was not a member
+    pub async fn remove_community_member(&self, community_id: Uuid, entity_id: Uuid) -> Result<bool, StoreError> {
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        let aql = r#"
+            FOR e IN edge_member_of
+            FILTER e.community_id == @community_id AND e.entity_id == @entity_id
+            REMOVE e IN edge_member_of
+            RETURN OLD
+        "#;
+
+        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+        bind_vars.insert("community_id", serde_json::json!(community_id.to_string()));
+        bind_vars.insert("entity_id", serde_json::json!(entity_id.to_string()));
+
+        let query = arangors::AqlQuery::builder()
+            .query(aql)
+            .bind_vars(bind_vars)
+            .build();
+
+        match db.aql_query::<serde_json::Value>(query).await {
+            Ok(results) => Ok(!results.is_empty()),
+            Err(_) => Ok(false),
+        }
+    }
+
+    /// Get all members of a community
+    ///
+    /// # Arguments
+    /// * `community_id` - UUID of the community
+    ///
+    /// # Returns
+    /// Vector of entities that are members of the community
+    pub async fn get_community_members(&self, community_id: Uuid) -> Result<Vec<Entity>, StoreError> {
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        let aql = r#"
+            FOR m IN edge_member_of
+            FILTER m.community_id == @community_id
+            FOR e IN persons, organizations, locations, laws, entities
+            FILTER e.id == m.entity_id
+            RETURN e
+        "#;
+
+        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+        bind_vars.insert("community_id", serde_json::json!(community_id.to_string()));
+
+        let query = arangors::AqlQuery::builder()
+            .query(aql)
+            .bind_vars(bind_vars)
+            .build();
+
+        match db.aql_query::<EntityDocument>(query).await {
+            Ok(docs) => {
+                let members = docs.into_iter().map(|doc| doc.to_entity()).collect();
+                Ok(members)
+            }
+            Err(e) => Err(StoreError::from(e)),
+        }
+    }
+
+    /// Check if an entity is a member of a community
+    async fn is_community_member(&self, community_id: Uuid, entity_id: Uuid) -> Result<bool, StoreError> {
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        let aql = r#"
+            FOR e IN edge_member_of
+            FILTER e.community_id == @community_id AND e.entity_id == @entity_id
+            LIMIT 1
+            RETURN e
+        "#;
+
+        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+        bind_vars.insert("community_id", serde_json::json!(community_id.to_string()));
+        bind_vars.insert("entity_id", serde_json::json!(entity_id.to_string()));
+
+        let query = arangors::AqlQuery::builder()
+            .query(aql)
+            .bind_vars(bind_vars)
+            .build();
+
+        match db.aql_query::<serde_json::Value>(query).await {
+            Ok(results) => Ok(!results.is_empty()),
+            Err(_) => Ok(false),
+        }
+    }
 }
 
 /// Document representation in ArangoDB
@@ -1781,6 +2042,89 @@ impl RelationshipDocument {
     }
 }
 
+/// Document representation in ArangoDB for communities
+#[derive(Debug, Clone, Serialize, Deserialize)]
+struct CommunityDocument {
+    #[serde(skip_serializing_if = "Option::is_none")]
+    _key: Option<String>,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    _id: Option<String>,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    _rev: Option<String>,
+
+    id: Uuid,
+    name: String,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    description: Option<String>,
+    level: i32,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    #[serde(rename = "parent_community_id")]
+    parent_community_id: Option<Uuid>,
+    #[serde(rename = "member_entity_ids")]
+    member_entity_ids: Vec<Uuid>,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    summary: Option<String>,
+    keywords: Vec<String>,
+    #[serde(rename = "created_at")]
+    created_at: String,
+}
+
+impl CommunityDocument {
+    fn from_community(community: &Community, id: Uuid) -> Self {
+        Self {
+            _key: None,
+            _id: None,
+            _rev: None,
+            id,
+            name: community.name.clone(),
+            description: community.description.clone(),
+            level: community.level,
+            parent_community_id: community.parent_community_id,
+            member_entity_ids: community.member_entity_ids.clone(),
+            summary: community.summary.clone(),
+            keywords: community.keywords.clone(),
+            created_at: community.created_at.to_rfc3339(),
+        }
+    }
+
+    fn set_key(&mut self) {
+        self._key = Some(self.id.to_string());
+    }
+
+    fn to_community(&self) -> Community {
+        use chrono::DateTime;
+
+        Community {
+            id: self.id,
+            name: self.name.clone(),
+            description: self.description.clone(),
+            level: self.level,
+            parent_community_id: self.parent_community_id,
+            member_entity_ids: self.member_entity_ids.clone(),
+            summary: self.summary.clone(),
+            keywords: self.keywords.clone(),
+            created_at: DateTime::parse_from_rfc3339(&self.created_at)
+                .map(|dt| dt.with_timezone(&chrono::Utc))
+                .unwrap_or_else(|_| chrono::Utc::now()),
+        }
+    }
+}
+
+/// Document representation in ArangoDB for community membership edges
+#[derive(Debug, Clone, Serialize, Deserialize)]
+struct MembershipEdge {
+    #[serde(skip_serializing_if = "Option::is_none")]
+    _key: Option<String>,
+
+    id: Uuid,
+    #[serde(rename = "community_id")]
+    community_id: Uuid,
+    #[serde(rename = "entity_id")]
+    entity_id: Uuid,
+    #[serde(rename = "created_at")]
+    created_at: String,
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
diff --git a/crates/iou-core/tests/graphrag/community_operations.rs b/crates/iou-core/tests/graphrag/community_operations.rs
new file mode 100644
index 0000000..6a7576d
--- /dev/null
+++ b/crates/iou-core/tests/graphrag/community_operations.rs
@@ -0,0 +1,276 @@
+//! Integration tests for community operations
+//!
+//! These tests require a running ArangoDB instance.
+
+use iou_core::graphrag::{
+    Community, Entity, EntityType, GraphStore, StoreError
+};
+use uuid::Uuid;
+use chrono::Utc;
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
+    let store = GraphStore::new(&config).await.expect("Failed to create GraphStore");
+    // Ensure collections exist
+    let _ = store.ensure_collections().await;
+    store
+}
+
+/// Helper to create test entities
+async fn create_test_entities(store: &GraphStore) -> Vec<Entity> {
+    let mut entities = Vec::new();
+
+    for i in 0..3 {
+        let entity = Entity {
+            id: Uuid::new_v4(),
+            name: format!("Entity {}", i),
+            entity_type: EntityType::Organization,
+            canonical_name: Some(format!("entity-{}", i)),
+            description: None,
+            confidence: 1.0,
+            source_domain_id: None,
+            metadata: serde_json::json!({}),
+            created_at: Utc::now(),
+        };
+        entities.push(store.create_entity(&entity).await.unwrap());
+    }
+
+    entities
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn create_community_creates_community_vertex() {
+    let store = setup_test_store().await;
+
+    let community = Community {
+        id: Uuid::new_v4(),
+        name: "Test Community".to_string(),
+        description: Some("A test community".to_string()),
+        level: 1,
+        parent_community_id: None,
+        member_entity_ids: vec![],
+        summary: None,
+        keywords: vec!["test".to_string(), "community".to_string()],
+        created_at: Utc::now(),
+    };
+
+    let created = store.create_community(&community).await.unwrap();
+
+    assert_eq!(created.name, "Test Community");
+    assert_eq!(created.description, Some("A test community".to_string()));
+    assert_eq!(created.level, 1);
+    assert!(!created.id.is_nil());
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn create_community_creates_membership_edges() {
+    let store = setup_test_store().await;
+    let entities = create_test_entities(&store).await;
+
+    let community = Community {
+        id: Uuid::new_v4(),
+        name: "Test Community".to_string(),
+        description: None,
+        level: 1,
+        parent_community_id: None,
+        member_entity_ids: entities.iter().map(|e| e.id).collect(),
+        summary: None,
+        keywords: vec![],
+        created_at: Utc::now(),
+    };
+
+    let created = store.create_community(&community).await.unwrap();
+
+    // Check that members were added
+    let members = store.get_community_members(created.id).await.unwrap();
+    assert_eq!(members.len(), 3);
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn get_community_found_returns_with_members() {
+    let store = setup_test_store().await;
+    let entities = create_test_entities(&store).await;
+
+    let community = Community {
+        id: Uuid::new_v4(),
+        name: "Test Community".to_string(),
+        description: None,
+        level: 1,
+        parent_community_id: None,
+        member_entity_ids: entities.iter().map(|e| e.id).collect(),
+        summary: None,
+        keywords: vec![],
+        created_at: Utc::now(),
+    };
+
+    let created = store.create_community(&community).await.unwrap();
+
+    // Get community
+    let found = store.get_community(created.id).await.unwrap();
+
+    assert!(found.is_some());
+    let found = found.unwrap();
+    assert_eq!(found.name, "Test Community");
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn get_community_not_found_returns_none() {
+    let store = setup_test_store().await;
+
+    let random_id = Uuid::new_v4();
+    let found = store.get_community(random_id).await.unwrap();
+
+    assert!(found.is_none());
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn add_community_member_creates_edge() {
+    let store = setup_test_store().await;
+    let mut entities = create_test_entities(&store).await;
+
+    let community = Community {
+        id: Uuid::new_v4(),
+        name: "Test Community".to_string(),
+        description: None,
+        level: 1,
+        parent_community_id: None,
+        member_entity_ids: vec![entities[0].id],
+        summary: None,
+        keywords: vec![],
+        created_at: Utc::now(),
+    };
+
+    let created = store.create_community(&community).await.unwrap();
+
+    // Add another member
+    let result = store.add_community_member(created.id, entities[1].id).await.unwrap();
+    assert!(result);
+
+    // Verify both members are present
+    let members = store.get_community_members(created.id).await.unwrap();
+    assert_eq!(members.len(), 2);
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn add_community_member_prevents_duplicates() {
+    let store = setup_test_store().await;
+    let entities = create_test_entities(&store).await;
+
+    let community = Community {
+        id: Uuid::new_v4(),
+        name: "Test Community".to_string(),
+        description: None,
+        level: 1,
+        parent_community_id: None,
+        member_entity_ids: vec![entities[0].id],
+        summary: None,
+        keywords: vec![],
+        created_at: Utc::now(),
+    };
+
+    let created = store.create_community(&community).await.unwrap();
+
+    // Try to add same member again
+    let result = store.add_community_member(created.id, entities[0].id).await.unwrap();
+    assert!(!result);
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn remove_community_member_removes_edge() {
+    let store = setup_test_store().await;
+    let entities = create_test_entities(&store).await;
+
+    let community = Community {
+        id: Uuid::new_v4(),
+        name: "Test Community".to_string(),
+        description: None,
+        level: 1,
+        parent_community_id: None,
+        member_entity_ids: entities.iter().map(|e| e.id).collect(),
+        summary: None,
+        keywords: vec![],
+        created_at: Utc::now(),
+    };
+
+    let created = store.create_community(&community).await.unwrap();
+
+    // Remove first member
+    let result = store.remove_community_member(created.id, entities[0].id).await.unwrap();
+    assert!(result);
+
+    // Verify only 2 members remain
+    let members = store.get_community_members(created.id).await.unwrap();
+    assert_eq!(members.len(), 2);
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn remove_community_member_non_member_returns_error() {
+    let store = setup_test_store().await;
+    let entities = create_test_entities(&store).await;
+
+    let community = Community {
+        id: Uuid::new_v4(),
+        name: "Test Community".to_string(),
+        description: None,
+        level: 1,
+        parent_community_id: None,
+        member_entity_ids: vec![entities[0].id],
+        summary: None,
+        keywords: vec![],
+        created_at: Utc::now(),
+    };
+
+    let created = store.create_community(&community).await.unwrap();
+
+    // Try to remove non-member (entities[2] was not added)
+    let result = store.remove_community_member(created.id, entities[2].id).await.unwrap();
+    assert!(!result);
+}
+
+#[tokio::test]
+#[ignore = "Requires ArangoDB"]
+async fn get_community_members_returns_all_members() {
+    let store = setup_test_store().await;
+    let entities = create_test_entities(&store).await;
+
+    let community = Community {
+        id: Uuid::new_v4(),
+        name: "Test Community".to_string(),
+        description: None,
+        level: 1,
+        parent_community_id: None,
+        member_entity_ids: entities.iter().map(|e| e.id).collect(),
+        summary: None,
+        keywords: vec![],
+        created_at: Utc::now(),
+    };
+
+    let created = store.create_community(&community).await.unwrap();
+
+    let members = store.get_community_members(created.id).await.unwrap();
+
+    assert_eq!(members.len(), 3);
+
+    // Verify entity IDs match
+    let member_ids: Vec<_> = members.iter().map(|m| m.id).collect();
+    for entity in &entities {
+        assert!(member_ids.contains(&entity.id));
+    }
+}
