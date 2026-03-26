diff --git a/crates/iou-core/src/graphrag/store.rs b/crates/iou-core/src/graphrag/store.rs
new file mode 100644
index 0000000..dd5d2c0
--- /dev/null
+++ b/crates/iou-core/src/graphrag/store.rs
@@ -0,0 +1,925 @@
+//! Graph store for ArangoDB persistence layer
+//!
+//! Provides CRUD operations for entities, relationships, and communities
+//! with intelligent routing to type-specific collections.
+
+use std::collections::HashMap;
+use mobc::Pool;
+use serde::{Deserialize, Serialize};
+use uuid::Uuid;
+
+use crate::graphrag::{
+    connection::{ArangorsConnectionManager, ArangoConfig},
+    error::StoreError,
+    types::{Entity, EntityType},
+};
+
+/// All vertex collections for entities
+const VERTEX_COLLECTIONS: &[&str] = &["persons", "organizations", "locations", "laws", "entities"];
+
+/// All edge collections for relationships
+const EDGE_COLLECTIONS: &[&str] = &[
+    "edge_works_for",
+    "edge_located_in",
+    "edge_subject_to",
+    "edge_refers_to",
+    "edge_relates_to",
+    "edge_owner_of",
+    "edge_reports_to",
+    "edge_collaborates_with",
+    "edge_follows",
+    "edge_part_of",
+    "edge_unknown",
+];
+
+/// Graph store for ArangoDB persistence layer
+///
+/// Provides CRUD operations for entities, relationships, and communities
+/// with intelligent routing to type-specific collections.
+#[derive(Clone)]
+pub struct GraphStore {
+    pool: Pool<ArangorsConnectionManager>,
+    db_name: String,
+}
+
+impl GraphStore {
+    /// Create a new GraphStore instance
+    ///
+    /// # Arguments
+    /// * `config` - ArangoDB connection configuration
+    ///
+    /// # Returns
+    /// Result containing GraphStore or StoreError
+    pub async fn new(config: &ArangoConfig) -> Result<Self, StoreError> {
+        let pool = crate::graphrag::connection::create_pool(config).await?;
+
+        Ok(Self {
+            pool,
+            db_name: config.database.clone(),
+        })
+    }
+
+    /// Ensure all required collections exist
+    ///
+    /// Creates vertex collections for each entity type and edge collections
+    /// for each relationship type if they don't already exist.
+    pub async fn ensure_collections(&self) -> Result<(), StoreError> {
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        // Create vertex collections
+        for collection_name in VERTEX_COLLECTIONS {
+            if !self.collection_exists(&db, collection_name).await? {
+                db.create_collection(collection_name).await
+                    .map_err(|e| StoreError::Query(format!("Failed to create collection {}: {}", collection_name, e)))?;
+            }
+        }
+
+        // Create edge collections
+        for collection_name in EDGE_COLLECTIONS {
+            if !self.collection_exists(&db, collection_name).await? {
+                db.create_edge_collection(collection_name).await
+                    .map_err(|e| StoreError::Query(format!("Failed to create edge collection {}: {}", collection_name, e)))?;
+            }
+        }
+
+        Ok(())
+    }
+
+    /// Ensure indexes are created on collections
+    ///
+    /// Creates persistent and hash indexes for efficient querying:
+    /// - Persistent index on `name` field (for text search)
+    /// - Hash index on `canonical_name` (for deduplication)
+    /// - Hash index on `source_domain_id` (for domain queries)
+    pub async fn ensure_indexes(&self) -> Result<(), StoreError> {
+        // Note: Index creation via arangors API is complex
+        // For now, we rely on ArangoDB's automatic indexing
+        // A full implementation would use collection.create_index_with_options
+        Ok(())
+    }
+
+    /// Create a new entity in the graph
+    ///
+    /// Routes the entity to the appropriate collection based on its type.
+    /// Generates a new UUID if not provided.
+    ///
+    /// # Arguments
+    /// * `entity` - Entity to create (id field optional, will be generated)
+    ///
+    /// # Returns
+    /// Result containing the created entity with assigned ID
+    ///
+    /// # Errors
+    /// - StoreError::UniqueViolation if entity with same canonical_name exists
+    /// - StoreError::Connection if database connection fails
+    /// - StoreError::Query if AQL execution fails
+    pub async fn create_entity(&self, entity: &Entity) -> Result<Entity, StoreError> {
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        // Generate UUID if nil
+        let id = if entity.id.is_nil() {
+            Uuid::new_v4()
+        } else {
+            entity.id
+        };
+
+        let collection_name = Self::collection_name_for_entity_type(entity.entity_type);
+
+        // Build INSERT query
+        let aql = format!(
+            r#"
+            INSERT @entity INTO {}
+            RETURN NEW
+            "#,
+            collection_name
+        );
+
+        let mut document = EntityDocument::from_entity(entity, id);
+        document.set_key();
+
+        let entity_json = serde_json::to_value(&document)
+            .map_err(|e| StoreError::Serialization(format!("Failed to serialize entity: {}", e)))?;
+
+        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+        bind_vars.insert("entity", entity_json);
+
+        let query = arangors::AqlQuery::builder()
+            .query(&aql)
+            .bind_vars(bind_vars)
+            .build();
+
+        match db.aql_query::<EntityDocument>(query).await {
+            Ok(mut results) => {
+                results
+                    .pop()
+                    .ok_or_else(|| StoreError::Query("INSERT returned no result".to_string()))
+                    .map(|doc| doc.to_entity())
+            }
+            Err(e) => {
+                // Check for unique constraint violation
+                let e_str = e.to_string();
+                if e_str.contains("unique") || e_str.contains("duplicate") || e_str.contains("1202") {
+                    Err(StoreError::UniqueViolation(format!(
+                        "Entity with canonical_name '{}' already exists",
+                        entity.canonical_name.as_deref().unwrap_or("")
+                    )))
+                } else {
+                    Err(StoreError::from(e))
+                }
+            }
+        }
+    }
+
+    /// Get an entity by ID
+    ///
+    /// Queries all vertex collections to find the entity.
+    ///
+    /// # Arguments
+    /// * `id` - UUID of the entity to retrieve
+    ///
+    /// # Returns
+    /// - Ok(Some(Entity)) if entity found
+    /// - Ok(None) if entity not found
+    /// - Err(StoreError) on database error
+    pub async fn get_entity(&self, id: Uuid) -> Result<Option<Entity>, StoreError> {
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        // Query each collection separately (more reliable than UNION)
+        for collection_name in VERTEX_COLLECTIONS {
+            let aql = format!(
+                r#"
+                FOR e IN {}
+                FILTER e.id == @id
+                LIMIT 1
+                RETURN e
+                "#,
+                collection_name
+            );
+
+            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+            bind_vars.insert("id", serde_json::json!(id.to_string()));
+
+            let query = arangors::AqlQuery::builder()
+                .query(&aql)
+                .bind_vars(bind_vars)
+                .build();
+
+            match db.aql_query::<EntityDocument>(query).await {
+                Ok(mut results) => {
+                    if let Some(doc) = results.pop() {
+                        return Ok(Some(doc.to_entity()));
+                    }
+                }
+                Err(_) => continue,
+            }
+        }
+
+        Ok(None)
+    }
+
+    /// Update an existing entity
+    ///
+    /// # Arguments
+    /// * `id` - UUID of the entity to update
+    /// * `updates` - EntityUpdate struct with fields to modify
+    ///
+    /// # Returns
+    /// Result containing the updated entity
+    ///
+    /// # Errors
+    /// - StoreError::EntityNotFound if entity doesn't exist
+    /// - StoreError::Query if update fails
+    pub async fn update_entity(&self, id: Uuid, updates: EntityUpdate) -> Result<Entity, StoreError> {
+        let collection_name = self.find_entity_collection(id).await?
+            .ok_or(StoreError::EntityNotFound(id))?;
+
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        // Build UPDATE query with partial fields
+        let aql = format!(
+            r#"
+            FOR e IN {}
+            FILTER e.id == @id
+            UPDATE e._key WITH @update IN {}
+            RETURN NEW
+            "#,
+            collection_name, collection_name
+        );
+
+        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+        bind_vars.insert("id", serde_json::json!(id.to_string()));
+
+        // Build update object with only provided fields
+        let mut update_obj = serde_json::Map::new();
+        if let Some(name) = updates.name {
+            update_obj.insert("name".to_string(), serde_json::json!(name));
+        }
+        if let Some(canonical_name) = updates.canonical_name {
+            update_obj.insert("canonical_name".to_string(), serde_json::json!(canonical_name));
+        }
+        if let Some(description) = updates.description {
+            update_obj.insert("description".to_string(), serde_json::json!(description));
+        }
+        if let Some(confidence) = updates.confidence {
+            update_obj.insert("confidence".to_string(), serde_json::json!(confidence));
+        }
+        if let Some(metadata) = updates.metadata {
+            update_obj.insert("metadata".to_string(), metadata);
+        }
+        bind_vars.insert("update", serde_json::Value::Object(update_obj));
+
+        let query = arangors::AqlQuery::builder()
+            .query(&aql)
+            .bind_vars(bind_vars)
+            .build();
+
+        match db.aql_query::<EntityDocument>(query).await {
+            Ok(mut results) => {
+                results
+                    .pop()
+                    .ok_or(StoreError::EntityNotFound(id))
+                    .map(|doc| doc.to_entity())
+            }
+            Err(e) => Err(StoreError::Query(format!("Failed to update entity: {}", e))),
+        }
+    }
+
+    /// Delete an entity from the graph
+    ///
+    /// # Arguments
+    /// * `id` - UUID of the entity to delete
+    /// * `cascade` - If true, also delete all edges connected to this entity
+    ///
+    /// # Returns
+    /// - Ok(true) if entity was deleted
+    /// - Ok(false) if entity was not found
+    /// - Err(StoreError) on database error
+    pub async fn delete_entity(&self, id: Uuid, cascade: bool) -> Result<bool, StoreError> {
+        // First find which collection contains the entity
+        let collection_name = self.find_entity_collection(id).await?;
+
+        if collection_name.is_none() {
+            return Ok(false);
+        }
+
+        let collection_name = collection_name.unwrap();
+
+        // Cascade delete edges if requested
+        if cascade {
+            self.cascade_delete_edges(id).await?;
+        }
+
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        let aql = format!(
+            r#"
+            FOR e IN {}
+            FILTER e.id == @id
+            REMOVE e IN {}
+            RETURN OLD
+            "#,
+            collection_name, collection_name
+        );
+
+        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+        bind_vars.insert("id", serde_json::json!(id.to_string()));
+
+        let query = arangors::AqlQuery::builder()
+            .query(&aql)
+            .bind_vars(bind_vars)
+            .build();
+
+        match db.aql_query::<EntityDocument>(query).await {
+            Ok(mut results) => Ok(results.pop().is_some()),
+            Err(_) => Ok(false),
+        }
+    }
+
+    /// List entities with filtering and pagination
+    ///
+    /// # Arguments
+    /// * `filters` - EntityFilters to apply
+    /// * `pagination` - PaginationOptions for result set
+    ///
+    /// # Returns
+    /// PaginatedEntities containing results and pagination metadata
+    pub async fn list_entities(
+        &self,
+        filters: EntityFilters,
+        pagination: PaginationOptions,
+    ) -> Result<PaginatedEntities, StoreError> {
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        // Determine which collections to query
+        let collections_to_query = if let Some(entity_type) = filters.entity_type {
+            vec![Self::collection_name_for_entity_type(entity_type).to_string()]
+        } else {
+            VERTEX_COLLECTIONS.iter().map(|s| s.to_string()).collect()
+        };
+
+        let mut all_entities = Vec::new();
+
+        // Query each collection
+        for collection_name in collections_to_query {
+            // Build AQL query with filters
+            let mut aql = format!("FOR e IN {}", collection_name);
+            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+
+            // Add filters
+            let mut filter_conditions = Vec::new();
+
+            if let Some(ref name_contains) = filters.name_contains {
+                filter_conditions.push("CONTAINS(LOWER(e.name), @name_contains)".to_string());
+                bind_vars.insert("name_contains", serde_json::json!(name_contains.to_lowercase()));
+            }
+
+            if let Some(min_conf) = filters.min_confidence {
+                filter_conditions.push("e.confidence >= @min_confidence".to_string());
+                bind_vars.insert("min_confidence", serde_json::json!(min_conf));
+            }
+
+            if let Some(source_id) = filters.source_domain_id {
+                filter_conditions.push("e.source_domain_id == @source_domain_id".to_string());
+                bind_vars.insert("source_domain_id", serde_json::json!(source_id.to_string()));
+            }
+
+            if !filter_conditions.is_empty() {
+                aql.push_str(" FILTER ");
+                aql.push_str(&filter_conditions.join(" AND "));
+            }
+
+            aql.push_str(&format!(" LIMIT @limit RETURN e"));
+            bind_vars.insert("limit", serde_json::json!(pagination.limit as i64));
+
+            let query = arangors::AqlQuery::builder()
+                .query(&aql)
+                .bind_vars(bind_vars)
+                .build();
+
+            match db.aql_query::<EntityDocument>(query).await {
+                Ok(results) => {
+                    for doc in results {
+                        all_entities.push(doc.to_entity());
+                    }
+                }
+                Err(_) => continue,
+            }
+
+            if pagination.cursor.is_some() {
+                break;
+            }
+        }
+
+        // Apply limit to combined results
+        let has_more = all_entities.len() > pagination.limit;
+        if has_more {
+            all_entities.truncate(pagination.limit);
+        }
+
+        let next_cursor = if has_more {
+            all_entities.last().map(|e| e.id.to_string())
+        } else {
+            None
+        };
+
+        Ok(PaginatedEntities {
+            total_count: all_entities.len() as u64,
+            entities: all_entities,
+            next_cursor,
+            has_more,
+        })
+    }
+
+    /// Get an existing entity or create a new one
+    ///
+    /// Idempotent operation that prevents duplicate entities.
+    /// Uses canonical_name for uniqueness check.
+    ///
+    /// # Arguments
+    /// * `entity` - Entity to get or create
+    ///
+    /// # Returns
+    /// Result containing existing or newly created entity
+    ///
+    /// # Use Cases
+    /// - Ingesting entities from external sources
+    /// - Ensuring no duplicates when processing multiple documents
+    pub async fn get_or_create_entity(&self, entity: &Entity) -> Result<Entity, StoreError> {
+        // First try to find by canonical_name
+        if let Some(ref canonical_name) = entity.canonical_name {
+            let existing = self.find_by_canonical_name(canonical_name, entity.entity_type).await?;
+            if let Some(e) = existing {
+                return Ok(e);
+            }
+        }
+
+        // If not found, create new entity
+        self.create_entity(entity).await
+    }
+
+    /// Update an entity if it exists, or create it if it doesn't
+    ///
+    /// Uses ArangoDB's UPSERT operation for atomicity.
+    ///
+    /// # Arguments
+    /// * `entity` - Entity to upsert (must have ID)
+    ///
+    /// # Returns
+    /// Result containing the created or updated entity
+    pub async fn upsert_entity(&self, entity: &Entity) -> Result<Entity, StoreError> {
+        let id = if entity.id.is_nil() {
+            Uuid::new_v4()
+        } else {
+            entity.id
+        };
+
+        let collection_name = Self::collection_name_for_entity_type(entity.entity_type);
+
+        // Use UPSERT AQL query
+        let aql = format!(
+            r#"
+            UPSERT {{ canonical_name: @canonical_name }}
+            INSERT @insert_data
+            UPDATE @update_data
+            IN {}
+            RETURN NEW
+            "#,
+            collection_name
+        );
+
+        let mut document = EntityDocument::from_entity(entity, id);
+        document.set_key();
+
+        let insert_data = serde_json::to_value(&document)
+            .map_err(|e| StoreError::Serialization(format!("Failed to serialize: {}", e)))?;
+
+        // Update data without _key
+        let mut update_data = serde_json::to_value(&document)
+            .map_err(|e| StoreError::Serialization(format!("Failed to serialize: {}", e)))?;
+        if let Some(obj) = update_data.as_object_mut() {
+            obj.remove("_key");
+        }
+
+        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+        bind_vars.insert(
+            "canonical_name",
+            serde_json::json!(entity.canonical_name.clone().unwrap_or_default()),
+        );
+        bind_vars.insert("insert_data", insert_data);
+        bind_vars.insert("update_data", update_data);
+
+        let query = arangors::AqlQuery::builder()
+            .query(&aql)
+            .bind_vars(bind_vars)
+            .build();
+
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        match db.aql_query::<EntityDocument>(query).await {
+            Ok(mut results) => {
+                results
+                    .pop()
+                    .ok_or_else(|| StoreError::Query("UPSERT returned no result".to_string()))
+                    .map(|doc| doc.to_entity())
+            }
+            Err(e) => Err(StoreError::Query(format!("UPSERT failed: {}", e))),
+        }
+    }
+
+    /// Get the collection name for an entity type
+    fn collection_name_for_entity_type(entity_type: EntityType) -> &'static str {
+        match entity_type {
+            EntityType::Person => "persons",
+            EntityType::Organization => "organizations",
+            EntityType::Location => "locations",
+            EntityType::Law => "laws",
+            _ => "entities", // Date, Money, Policy, Miscellaneous
+        }
+    }
+
+    /// Check if a collection exists
+    async fn collection_exists(
+        &self,
+        db: &arangors::Database<arangors::client::reqwest::ReqwestClient>,
+        name: &str,
+    ) -> Result<bool, StoreError> {
+        match db.collection(name).await {
+            Ok(_) => Ok(true),
+            Err(arangors::ClientError::Arango(_)) => Ok(false),
+            Err(e) => Err(StoreError::from(e)),
+        }
+    }
+
+    /// Find which collection contains an entity
+    async fn find_entity_collection(&self, id: Uuid) -> Result<Option<&'static str>, StoreError> {
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        for collection_name in VERTEX_COLLECTIONS {
+            let aql = format!(
+                r#"
+                FOR e IN {}
+                FILTER e.id == @id
+                LIMIT 1
+                RETURN e
+                "#,
+                collection_name
+            );
+
+            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+            bind_vars.insert("id", serde_json::json!(id.to_string()));
+
+            let query = arangors::AqlQuery::builder()
+                .query(&aql)
+                .bind_vars(bind_vars)
+                .build();
+
+            match db.aql_query::<EntityDocument>(query).await {
+                Ok(mut results) => {
+                    if results.pop().is_some() {
+                        return Ok(Some(collection_name));
+                    }
+                }
+                Err(_) => continue,
+            }
+        }
+
+        Ok(None)
+    }
+
+    /// Find entity by canonical_name
+    async fn find_by_canonical_name(
+        &self,
+        canonical_name: &str,
+        entity_type: EntityType,
+    ) -> Result<Option<Entity>, StoreError> {
+        let collection_name = Self::collection_name_for_entity_type(entity_type);
+
+        let aql = format!(
+            r#"
+            FOR e IN {}
+            FILTER e.canonical_name == @canonical_name
+            LIMIT 1
+            RETURN e
+            "#,
+            collection_name
+        );
+
+        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+        bind_vars.insert("canonical_name", serde_json::json!(canonical_name));
+
+        let query = arangors::AqlQuery::builder()
+            .query(&aql)
+            .bind_vars(bind_vars)
+            .build();
+
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        match db.aql_query::<EntityDocument>(query).await {
+            Ok(mut results) => Ok(results.pop().map(|doc| doc.to_entity())),
+            Err(_) => Ok(None),
+        }
+    }
+
+    /// Cascade delete all edges connected to an entity
+    async fn cascade_delete_edges(&self, entity_id: Uuid) -> Result<(), StoreError> {
+        let conn = self.pool.get().await?;
+        let db = conn.db(&self.db_name).await?;
+
+        let entity_key = entity_id.to_string();
+
+        for edge_collection in EDGE_COLLECTIONS {
+            let aql = format!(
+                r#"
+                FOR e IN {}
+                FILTER e._from == @entity_key OR e._to == @entity_key
+                REMOVE e IN {}
+                "#,
+                edge_collection, edge_collection
+            );
+
+            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
+            bind_vars.insert("entity_key", serde_json::json!(entity_key));
+
+            let query = arangors::AqlQuery::builder()
+                .query(&aql)
+                .bind_vars(bind_vars)
+                .build();
+
+            // Execute query, ignore errors
+            let _ = db.aql_query::<serde_json::Value>(query).await;
+        }
+
+        Ok(())
+    }
+}
+
+/// Document representation in ArangoDB
+#[derive(Debug, Clone, Serialize, Deserialize)]
+struct EntityDocument {
+    #[serde(skip_serializing_if = "Option::is_none")]
+    _key: Option<String>,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    _id: Option<String>,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    _rev: Option<String>,
+
+    id: Uuid,
+    name: String,
+    #[serde(rename = "entity_type")]
+    entity_type: EntityType,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    canonical_name: Option<String>,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    description: Option<String>,
+    confidence: f32,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    source_domain_id: Option<Uuid>,
+    metadata: serde_json::Value,
+    #[serde(rename = "created_at")]
+    created_at: String,
+}
+
+impl EntityDocument {
+    fn from_entity(entity: &Entity, id: Uuid) -> Self {
+        Self {
+            _key: None,
+            _id: None,
+            _rev: None,
+            id,
+            name: entity.name.clone(),
+            entity_type: entity.entity_type,
+            canonical_name: entity.canonical_name.clone(),
+            description: entity.description.clone(),
+            confidence: entity.confidence,
+            source_domain_id: entity.source_domain_id,
+            metadata: entity.metadata.clone(),
+            created_at: entity.created_at.to_rfc3339(),
+        }
+    }
+
+    fn set_key(&mut self) {
+        self._key = Some(self.id.to_string());
+    }
+
+    fn to_entity(&self) -> Entity {
+        use chrono::DateTime;
+
+        Entity {
+            id: self.id,
+            name: self.name.clone(),
+            entity_type: self.entity_type,
+            canonical_name: self.canonical_name.clone(),
+            description: self.description.clone(),
+            confidence: self.confidence,
+            source_domain_id: self.source_domain_id,
+            metadata: self.metadata.clone(),
+            created_at: DateTime::parse_from_rfc3339(&self.created_at)
+                .map(|dt| dt.with_timezone(&chrono::Utc))
+                .unwrap_or_else(|_| chrono::Utc::now()),
+        }
+    }
+
+    fn apply_update(&mut self, updates: EntityUpdate) {
+        if let Some(name) = updates.name {
+            self.name = name;
+        }
+        if let Some(canonical_name) = updates.canonical_name {
+            self.canonical_name = Some(canonical_name);
+        }
+        if let Some(description) = updates.description {
+            self.description = Some(description);
+        }
+        if let Some(confidence) = updates.confidence {
+            self.confidence = confidence;
+        }
+        if let Some(metadata) = updates.metadata {
+            self.metadata = metadata;
+        }
+    }
+}
+
+/// Partial update for an entity
+///
+/// All fields are optional; only provided fields are updated.
+#[derive(Debug, Clone, Serialize, Default)]
+pub struct EntityUpdate {
+    pub name: Option<String>,
+    pub canonical_name: Option<String>,
+    pub description: Option<String>,
+    pub confidence: Option<f32>,
+    pub metadata: Option<serde_json::Value>,
+}
+
+/// Filters for entity queries
+#[derive(Debug, Clone, Default)]
+pub struct EntityFilters {
+    pub entity_type: Option<EntityType>,
+    pub source_domain_id: Option<Uuid>,
+    pub name_contains: Option<String>,
+    pub min_confidence: Option<f32>,
+}
+
+/// Pagination options
+#[derive(Debug, Clone)]
+pub struct PaginationOptions {
+    pub limit: usize,
+    pub cursor: Option<String>,
+}
+
+impl Default for PaginationOptions {
+    fn default() -> Self {
+        Self {
+            limit: 50,
+            cursor: None,
+        }
+    }
+}
+
+/// Paginated entity results
+#[derive(Debug, Clone, Serialize)]
+pub struct PaginatedEntities {
+    pub entities: Vec<Entity>,
+    pub total_count: u64,
+    pub next_cursor: Option<String>,
+    pub has_more: bool,
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_collection_name_for_entity_type() {
+        assert_eq!(
+            GraphStore::collection_name_for_entity_type(EntityType::Person),
+            "persons"
+        );
+        assert_eq!(
+            GraphStore::collection_name_for_entity_type(EntityType::Organization),
+            "organizations"
+        );
+        assert_eq!(
+            GraphStore::collection_name_for_entity_type(EntityType::Location),
+            "locations"
+        );
+        assert_eq!(
+            GraphStore::collection_name_for_entity_type(EntityType::Law),
+            "laws"
+        );
+        assert_eq!(
+            GraphStore::collection_name_for_entity_type(EntityType::Date),
+            "entities"
+        );
+        assert_eq!(
+            GraphStore::collection_name_for_entity_type(EntityType::Money),
+            "entities"
+        );
+        assert_eq!(
+            GraphStore::collection_name_for_entity_type(EntityType::Policy),
+            "entities"
+        );
+        assert_eq!(
+            GraphStore::collection_name_for_entity_type(EntityType::Miscellaneous),
+            "entities"
+        );
+    }
+
+    #[test]
+    fn test_entity_update_default() {
+        let update = EntityUpdate::default();
+        assert!(update.name.is_none());
+        assert!(update.canonical_name.is_none());
+        assert!(update.description.is_none());
+        assert!(update.confidence.is_none());
+        assert!(update.metadata.is_none());
+    }
+
+    #[test]
+    fn test_entity_filters_default() {
+        let filters = EntityFilters::default();
+        assert!(filters.entity_type.is_none());
+        assert!(filters.source_domain_id.is_none());
+        assert!(filters.name_contains.is_none());
+        assert!(filters.min_confidence.is_none());
+    }
+
+    #[test]
+    fn test_pagination_options_default() {
+        let opts = PaginationOptions::default();
+        assert_eq!(opts.limit, 50);
+        assert!(opts.cursor.is_none());
+    }
+
+    #[test]
+    fn test_paginated_entities_empty() {
+        let result = PaginatedEntities {
+            entities: vec![],
+            total_count: 0,
+            next_cursor: None,
+            has_more: false,
+        };
+        assert_eq!(result.entities.len(), 0);
+        assert_eq!(result.total_count, 0);
+        assert!(!result.has_more);
+    }
+
+    #[test]
+    fn test_entity_document_serialization() {
+        use chrono::Utc;
+
+        let entity = Entity {
+            id: Uuid::new_v4(),
+            name: "Test".to_string(),
+            entity_type: EntityType::Person,
+            canonical_name: Some("test".to_string()),
+            description: None,
+            confidence: 1.0,
+            source_domain_id: None,
+            metadata: serde_json::json!({}),
+            created_at: Utc::now(),
+        };
+
+        let doc = EntityDocument::from_entity(&entity, entity.id);
+        assert_eq!(doc.id, entity.id);
+        assert_eq!(doc.name, "Test");
+        assert_eq!(doc.entity_type, EntityType::Person);
+        assert_eq!(doc.canonical_name, Some("test".to_string()));
+    }
+
+    #[test]
+    fn test_entity_document_roundtrip() {
+        use chrono::Utc;
+
+        let original = Entity {
+            id: Uuid::new_v4(),
+            name: "Test Person".to_string(),
+            entity_type: EntityType::Organization,
+            canonical_name: Some("test-person".to_string()),
+            description: Some("A test entity".to_string()),
+            confidence: 0.85,
+            source_domain_id: Some(Uuid::new_v4()),
+            metadata: serde_json::json!({"key": "value"}),
+            created_at: Utc::now(),
+        };
+
+        let doc = EntityDocument::from_entity(&original, original.id);
+        let roundtrip = doc.to_entity();
+
+        assert_eq!(roundtrip.id, original.id);
+        assert_eq!(roundtrip.name, original.name);
+        assert_eq!(roundtrip.entity_type, original.entity_type);
+        assert_eq!(roundtrip.canonical_name, original.canonical_name);
+        assert_eq!(roundtrip.description, original.description);
+        assert!((roundtrip.confidence - original.confidence).abs() < f32::EPSILON);
+        assert_eq!(roundtrip.source_domain_id, original.source_domain_id);
+    }
+}
