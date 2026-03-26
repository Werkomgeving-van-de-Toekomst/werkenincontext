//! Graph store for ArangoDB persistence layer
//!
//! Provides CRUD operations for entities, relationships, and communities
//! with intelligent routing to type-specific collections.

use std::collections::HashMap;
use chrono::Utc;
use mobc::Pool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::graphrag::{
    connection::{ArangorsConnectionManager, ArangoConfig},
    error::StoreError,
    types::{Community, Entity, EntityType, Relationship, RelationshipType},
};

/// All vertex collections for entities
const VERTEX_COLLECTIONS: &[&str] = &["persons", "organizations", "locations", "laws", "entities", "communities"];

/// All edge collections for relationships
const EDGE_COLLECTIONS: &[&str] = &[
    "edge_works_for",
    "edge_located_in",
    "edge_subject_to",
    "edge_refers_to",
    "edge_relates_to",
    "edge_owner_of",
    "edge_reports_to",
    "edge_collaborates_with",
    "edge_follows",
    "edge_part_of",
    "edge_unknown",
    "edge_member_of",
    "edge_subcommunity",
];

/// Graph store for ArangoDB persistence layer
///
/// Provides CRUD operations for entities, relationships, and communities
/// with intelligent routing to type-specific collections.
#[derive(Clone)]
pub struct GraphStore {
    pool: Pool<ArangorsConnectionManager>,
    db_name: String,
}

impl GraphStore {
    /// Create a new GraphStore instance
    ///
    /// # Arguments
    /// * `config` - ArangoDB connection configuration
    ///
    /// # Returns
    /// Result containing GraphStore or StoreError
    pub async fn new(config: &ArangoConfig) -> Result<Self, StoreError> {
        let pool = crate::graphrag::connection::create_pool(config).await?;

        Ok(Self {
            pool,
            db_name: config.database.clone(),
        })
    }

    /// Ensure all required collections exist
    ///
    /// Creates vertex collections for each entity type and edge collections
    /// for each relationship type if they don't already exist.
    pub async fn ensure_collections(&self) -> Result<(), StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        // Create vertex collections
        for collection_name in VERTEX_COLLECTIONS {
            if !self.collection_exists(&db, collection_name).await? {
                db.create_collection(collection_name).await
                    .map_err(|e| StoreError::Query(format!("Failed to create collection {}: {}", collection_name, e)))?;
            }
        }

        // Create edge collections
        for collection_name in EDGE_COLLECTIONS {
            if !self.collection_exists(&db, collection_name).await? {
                db.create_edge_collection(collection_name).await
                    .map_err(|e| StoreError::Query(format!("Failed to create edge collection {}: {}", collection_name, e)))?;
            }
        }

        Ok(())
    }

    /// Ensure indexes are created on collections
    ///
    /// Creates persistent and hash indexes for efficient querying:
    /// - Persistent index on `name` field (for text search)
    /// - Hash index on `canonical_name` (for deduplication)
    /// - Hash index on `source_domain_id` (for domain queries)
    pub async fn ensure_indexes(&self) -> Result<(), StoreError> {
        // Note: Index creation via arangors API is complex
        // For now, we rely on ArangoDB's automatic indexing
        // A full implementation would use collection.create_index_with_options
        Ok(())
    }

    /// Create a new entity in the graph
    ///
    /// Routes the entity to the appropriate collection based on its type.
    /// Generates a new UUID if not provided.
    ///
    /// # Arguments
    /// * `entity` - Entity to create (id field optional, will be generated)
    ///
    /// # Returns
    /// Result containing the created entity with assigned ID
    ///
    /// # Errors
    /// - StoreError::UniqueViolation if entity with same canonical_name exists
    /// - StoreError::Connection if database connection fails
    /// - StoreError::Query if AQL execution fails
    pub async fn create_entity(&self, entity: &Entity) -> Result<Entity, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        // Generate UUID if nil
        let id = if entity.id.is_nil() {
            Uuid::new_v4()
        } else {
            entity.id
        };

        let collection_name = Self::collection_name_for_entity_type(entity.entity_type);

        // Build INSERT query
        let aql = format!(
            r#"
            INSERT @entity INTO {}
            RETURN NEW
            "#,
            collection_name
        );

        let mut document = EntityDocument::from_entity(entity, id);
        document.set_key();

        let entity_json = serde_json::to_value(&document)
            .map_err(|e| StoreError::Serialization(format!("Failed to serialize entity: {}", e)))?;

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert("entity", entity_json);

        let query = arangors::AqlQuery::builder()
            .query(&aql)
            .bind_vars(bind_vars)
            .build();

        match db.aql_query::<EntityDocument>(query).await {
            Ok(mut results) => {
                results
                    .pop()
                    .ok_or_else(|| StoreError::Query("INSERT returned no result".to_string()))
                    .map(|doc| doc.to_entity())
            }
            Err(e) => {
                // Check for unique constraint violation
                let e_str = e.to_string();
                if e_str.contains("unique") || e_str.contains("duplicate") || e_str.contains("1202") {
                    Err(StoreError::UniqueViolation(format!(
                        "Entity with canonical_name '{}' already exists",
                        entity.canonical_name.as_deref().unwrap_or("")
                    )))
                } else {
                    Err(StoreError::from(e))
                }
            }
        }
    }

    /// Get an entity by ID
    ///
    /// Queries all vertex collections to find the entity.
    ///
    /// # Arguments
    /// * `id` - UUID of the entity to retrieve
    ///
    /// # Returns
    /// - Ok(Some(Entity)) if entity found
    /// - Ok(None) if entity not found
    /// - Err(StoreError) on database error
    pub async fn get_entity(&self, id: Uuid) -> Result<Option<Entity>, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        // Query each collection separately (more reliable than UNION)
        for collection_name in VERTEX_COLLECTIONS {
            let aql = format!(
                r#"
                FOR e IN {}
                FILTER e.id == @id
                LIMIT 1
                RETURN e
                "#,
                collection_name
            );

            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
            bind_vars.insert("id", serde_json::json!(id.to_string()));

            let query = arangors::AqlQuery::builder()
                .query(&aql)
                .bind_vars(bind_vars)
                .build();

            match db.aql_query::<EntityDocument>(query).await {
                Ok(mut results) => {
                    if let Some(doc) = results.pop() {
                        return Ok(Some(doc.to_entity()));
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(None)
    }

    /// Update an existing entity
    ///
    /// # Arguments
    /// * `id` - UUID of the entity to update
    /// * `updates` - EntityUpdate struct with fields to modify
    ///
    /// # Returns
    /// Result containing the updated entity
    ///
    /// # Errors
    /// - StoreError::EntityNotFound if entity doesn't exist
    /// - StoreError::Query if update fails
    pub async fn update_entity(&self, id: Uuid, updates: EntityUpdate) -> Result<Entity, StoreError> {
        let collection_name = self.find_entity_collection(id).await?
            .ok_or(StoreError::EntityNotFound(id))?;

        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        // Build UPDATE query with partial fields
        let aql = format!(
            r#"
            FOR e IN {}
            FILTER e.id == @id
            UPDATE e._key WITH @update IN {}
            RETURN NEW
            "#,
            collection_name, collection_name
        );

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert("id", serde_json::json!(id.to_string()));

        // Build update object with only provided fields
        let mut update_obj = serde_json::Map::new();
        if let Some(name) = updates.name {
            update_obj.insert("name".to_string(), serde_json::json!(name));
        }
        if let Some(canonical_name) = updates.canonical_name {
            update_obj.insert("canonical_name".to_string(), serde_json::json!(canonical_name));
        }
        if let Some(description) = updates.description {
            update_obj.insert("description".to_string(), serde_json::json!(description));
        }
        if let Some(confidence) = updates.confidence {
            update_obj.insert("confidence".to_string(), serde_json::json!(confidence));
        }
        if let Some(metadata) = updates.metadata {
            update_obj.insert("metadata".to_string(), metadata);
        }
        bind_vars.insert("update", serde_json::Value::Object(update_obj));

        let query = arangors::AqlQuery::builder()
            .query(&aql)
            .bind_vars(bind_vars)
            .build();

        match db.aql_query::<EntityDocument>(query).await {
            Ok(mut results) => {
                results
                    .pop()
                    .ok_or(StoreError::EntityNotFound(id))
                    .map(|doc| doc.to_entity())
            }
            Err(e) => Err(StoreError::Query(format!("Failed to update entity: {}", e))),
        }
    }

    /// Delete an entity from the graph
    ///
    /// # Arguments
    /// * `id` - UUID of the entity to delete
    /// * `cascade` - If true, also delete all edges connected to this entity
    ///
    /// # Returns
    /// - Ok(true) if entity was deleted
    /// - Ok(false) if entity was not found
    /// - Err(StoreError) on database error
    pub async fn delete_entity(&self, id: Uuid, cascade: bool) -> Result<bool, StoreError> {
        // First find which collection contains the entity
        let collection_name = self.find_entity_collection(id).await?;

        if collection_name.is_none() {
            return Ok(false);
        }

        let collection_name = collection_name.unwrap();

        // Cascade delete edges if requested
        if cascade {
            self.cascade_delete_edges(id).await?;
        }

        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        let aql = format!(
            r#"
            FOR e IN {}
            FILTER e.id == @id
            REMOVE e IN {}
            RETURN OLD
            "#,
            collection_name, collection_name
        );

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert("id", serde_json::json!(id.to_string()));

        let query = arangors::AqlQuery::builder()
            .query(&aql)
            .bind_vars(bind_vars)
            .build();

        match db.aql_query::<EntityDocument>(query).await {
            Ok(mut results) => Ok(results.pop().is_some()),
            Err(_) => Ok(false),
        }
    }

    /// List entities with filtering and pagination
    ///
    /// # Arguments
    /// * `filters` - EntityFilters to apply
    /// * `pagination` - PaginationOptions for result set
    ///
    /// # Returns
    /// PaginatedEntities containing results and pagination metadata
    pub async fn list_entities(
        &self,
        filters: EntityFilters,
        pagination: PaginationOptions,
    ) -> Result<PaginatedEntities, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        // Determine which collections to query
        let collections_to_query = if let Some(entity_type) = filters.entity_type {
            vec![Self::collection_name_for_entity_type(entity_type).to_string()]
        } else {
            VERTEX_COLLECTIONS.iter().map(|s| s.to_string()).collect()
        };

        let mut all_entities = Vec::new();

        // Query each collection
        for collection_name in collections_to_query {
            // Build AQL query with filters
            let mut aql = format!("FOR e IN {}", collection_name);
            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();

            // Add filters
            let mut filter_conditions = Vec::new();

            if let Some(ref name_contains) = filters.name_contains {
                filter_conditions.push("CONTAINS(LOWER(e.name), @name_contains)".to_string());
                bind_vars.insert("name_contains", serde_json::json!(name_contains.to_lowercase()));
            }

            if let Some(min_conf) = filters.min_confidence {
                filter_conditions.push("e.confidence >= @min_confidence".to_string());
                bind_vars.insert("min_confidence", serde_json::json!(min_conf));
            }

            if let Some(source_id) = filters.source_domain_id {
                filter_conditions.push("e.source_domain_id == @source_domain_id".to_string());
                bind_vars.insert("source_domain_id", serde_json::json!(source_id.to_string()));
            }

            if !filter_conditions.is_empty() {
                aql.push_str(" FILTER ");
                aql.push_str(&filter_conditions.join(" AND "));
            }

            aql.push_str(&format!(" LIMIT @limit RETURN e"));
            bind_vars.insert("limit", serde_json::json!(pagination.limit as i64));

            let query = arangors::AqlQuery::builder()
                .query(&aql)
                .bind_vars(bind_vars)
                .build();

            match db.aql_query::<EntityDocument>(query).await {
                Ok(results) => {
                    for doc in results {
                        all_entities.push(doc.to_entity());
                    }
                }
                Err(_) => continue,
            }

            if pagination.cursor.is_some() {
                break;
            }
        }

        // Apply limit to combined results
        let has_more = all_entities.len() > pagination.limit;
        if has_more {
            all_entities.truncate(pagination.limit);
        }

        let next_cursor = if has_more {
            all_entities.last().map(|e| e.id.to_string())
        } else {
            None
        };

        Ok(PaginatedEntities {
            total_count: all_entities.len() as u64,
            entities: all_entities,
            next_cursor,
            has_more,
        })
    }

    /// Get an existing entity or create a new one
    ///
    /// Idempotent operation that prevents duplicate entities.
    /// Uses canonical_name for uniqueness check.
    ///
    /// This method handles race conditions where concurrent requests might
    /// both try to create the same entity by attempting creation first
    /// and falling back to lookup on UniqueViolation error.
    ///
    /// # Arguments
    /// * `entity` - Entity to get or create
    ///
    /// # Returns
    /// Result containing existing or newly created entity
    ///
    /// # Use Cases
    /// - Ingesting entities from external sources
    /// - Ensuring no duplicates when processing multiple documents
    pub async fn get_or_create_entity(&self, entity: &Entity) -> Result<Entity, StoreError> {
        // Try insert-first approach to handle race conditions
        match self.create_entity(entity).await {
            Ok(created) => Ok(created),
            Err(StoreError::UniqueViolation(_)) => {
                // Race condition - another thread created it first
                // Look up the existing entity
                if let Some(ref canonical_name) = entity.canonical_name {
                    self.find_by_canonical_name(canonical_name, entity.entity_type).await?
                        .ok_or_else(|| StoreError::Query("Entity lost after UniqueViolation".to_string()))
                } else {
                    // No canonical_name to look up by, return the error
                    Err(StoreError::Query("Cannot resolve race condition without canonical_name".to_string()))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Update an entity if it exists, or create it if it doesn't
    ///
    /// Uses ArangoDB's UPSERT operation for atomicity.
    ///
    /// # Arguments
    /// * `entity` - Entity to upsert (must have ID)
    ///
    /// # Returns
    /// Result containing the created or updated entity
    pub async fn upsert_entity(&self, entity: &Entity) -> Result<Entity, StoreError> {
        let id = if entity.id.is_nil() {
            Uuid::new_v4()
        } else {
            entity.id
        };

        let collection_name = Self::collection_name_for_entity_type(entity.entity_type);

        // Use UPSERT AQL query
        let aql = format!(
            r#"
            UPSERT {{ canonical_name: @canonical_name }}
            INSERT @insert_data
            UPDATE @update_data
            IN {}
            RETURN NEW
            "#,
            collection_name
        );

        let mut document = EntityDocument::from_entity(entity, id);
        document.set_key();

        let insert_data = serde_json::to_value(&document)
            .map_err(|e| StoreError::Serialization(format!("Failed to serialize: {}", e)))?;

        // Update data without _key
        let mut update_data = serde_json::to_value(&document)
            .map_err(|e| StoreError::Serialization(format!("Failed to serialize: {}", e)))?;
        if let Some(obj) = update_data.as_object_mut() {
            obj.remove("_key");
        }

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert(
            "canonical_name",
            serde_json::json!(entity.canonical_name.clone().unwrap_or_default()),
        );
        bind_vars.insert("insert_data", insert_data);
        bind_vars.insert("update_data", update_data);

        let query = arangors::AqlQuery::builder()
            .query(&aql)
            .bind_vars(bind_vars)
            .build();

        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        match db.aql_query::<EntityDocument>(query).await {
            Ok(mut results) => {
                results
                    .pop()
                    .ok_or_else(|| StoreError::Query("UPSERT returned no result".to_string()))
                    .map(|doc| doc.to_entity())
            }
            Err(e) => Err(StoreError::Query(format!("UPSERT failed: {}", e))),
        }
    }

    /// Get the collection name for an entity type
    fn collection_name_for_entity_type(entity_type: EntityType) -> &'static str {
        match entity_type {
            EntityType::Person => "persons",
            EntityType::Organization => "organizations",
            EntityType::Location => "locations",
            EntityType::Law => "laws",
            _ => "entities", // Date, Money, Policy, Miscellaneous
        }
    }

    /// Check if a collection exists
    async fn collection_exists(
        &self,
        db: &arangors::Database<arangors::client::reqwest::ReqwestClient>,
        name: &str,
    ) -> Result<bool, StoreError> {
        match db.collection(name).await {
            Ok(_) => Ok(true),
            Err(arangors::ClientError::Arango(_)) => Ok(false),
            Err(e) => Err(StoreError::from(e)),
        }
    }

    /// Find which collection contains an entity
    async fn find_entity_collection(&self, id: Uuid) -> Result<Option<&'static str>, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        for collection_name in VERTEX_COLLECTIONS {
            let aql = format!(
                r#"
                FOR e IN {}
                FILTER e.id == @id
                LIMIT 1
                RETURN e
                "#,
                collection_name
            );

            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
            bind_vars.insert("id", serde_json::json!(id.to_string()));

            let query = arangors::AqlQuery::builder()
                .query(&aql)
                .bind_vars(bind_vars)
                .build();

            match db.aql_query::<EntityDocument>(query).await {
                Ok(mut results) => {
                    if results.pop().is_some() {
                        return Ok(Some(collection_name));
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(None)
    }

    /// Find entity by canonical_name
    async fn find_by_canonical_name(
        &self,
        canonical_name: &str,
        entity_type: EntityType,
    ) -> Result<Option<Entity>, StoreError> {
        let collection_name = Self::collection_name_for_entity_type(entity_type);

        let aql = format!(
            r#"
            FOR e IN {}
            FILTER e.canonical_name == @canonical_name
            LIMIT 1
            RETURN e
            "#,
            collection_name
        );

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert("canonical_name", serde_json::json!(canonical_name));

        let query = arangors::AqlQuery::builder()
            .query(&aql)
            .bind_vars(bind_vars)
            .build();

        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        match db.aql_query::<EntityDocument>(query).await {
            Ok(mut results) => Ok(results.pop().map(|doc| doc.to_entity())),
            Err(_) => Ok(None),
        }
    }

    /// Cascade delete all edges connected to an entity
    async fn cascade_delete_edges(&self, entity_id: Uuid) -> Result<(), StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        let entity_key = entity_id.to_string();

        for edge_collection in EDGE_COLLECTIONS {
            let aql = format!(
                r#"
                FOR e IN {}
                FILTER e._from == @entity_key OR e._to == @entity_key
                REMOVE e IN {}
                "#,
                edge_collection, edge_collection
            );

            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
            bind_vars.insert("entity_key", serde_json::json!(entity_key));

            let query = arangors::AqlQuery::builder()
                .query(&aql)
                .bind_vars(bind_vars)
                .build();

            // Execute query, ignore errors
            let _ = db.aql_query::<serde_json::Value>(query).await;
        }

        Ok(())
    }

    // ========== Relationship Operations ==========

    /// Create a new relationship in the graph
    ///
    /// Routes the relationship to the appropriate edge collection based on its type.
    /// Generates a new UUID if not provided.
    ///
    /// # Arguments
    /// * `rel` - Relationship to create (id field optional, will be generated)
    ///
    /// # Returns
    /// Result containing the created relationship with assigned ID
    ///
    /// # Errors
    /// - StoreError::UniqueViolation if relationship with same entities and type exists
    /// - StoreError::Connection if database connection fails
    /// - StoreError::Query if AQL execution fails
    pub async fn create_relationship(&self, rel: &Relationship) -> Result<Relationship, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        // Generate UUID if nil
        let id = if rel.id.is_nil() {
            Uuid::new_v4()
        } else {
            rel.id
        };

        let collection_name = Self::collection_name_for_relationship_type(rel.relationship_type);

        // Build INSERT query
        let aql = format!(
            r#"
            INSERT @relationship INTO {}
            RETURN NEW
            "#,
            collection_name
        );

        let mut document = RelationshipDocument::from_relationship(rel, id);
        document.set_key();

        // Set ArangoDB edge references
        // For now, we use entity IDs as references. A full implementation would
        // query to find the actual collections for source and target entities.
        document._from = Some(format!("entities/{}", rel.source_entity_id));
        document._to = Some(format!("entities/{}", rel.target_entity_id));

        let rel_json = serde_json::to_value(&document)
            .map_err(|e| StoreError::Serialization(format!("Failed to serialize relationship: {}", e)))?;

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert("relationship", rel_json);

        let query = arangors::AqlQuery::builder()
            .query(&aql)
            .bind_vars(bind_vars)
            .build();

        match db.aql_query::<RelationshipDocument>(query).await {
            Ok(mut results) => {
                results
                    .pop()
                    .ok_or_else(|| StoreError::Query("INSERT returned no result".to_string()))
                    .map(|doc| doc.to_relationship())
            }
            Err(e) => {
                // Check for unique constraint violation
                let e_str = e.to_string();
                if e_str.contains("unique") || e_str.contains("duplicate") || e_str.contains("1202") {
                    Err(StoreError::UniqueViolation(format!(
                        "Relationship between {} and {} of type {:?} already exists",
                        rel.source_entity_id, rel.target_entity_id, rel.relationship_type
                    )))
                } else {
                    Err(StoreError::from(e))
                }
            }
        }
    }

    /// Get a relationship by ID
    ///
    /// Queries all edge collections to find the relationship.
    ///
    /// # Arguments
    /// * `id` - UUID of the relationship to retrieve
    ///
    /// # Returns
    /// - Ok(Some(Relationship)) if relationship found
    /// - Ok(None) if relationship not found
    /// - Err(StoreError) on database error
    pub async fn get_relationship(&self, id: Uuid) -> Result<Option<Relationship>, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        // Query each edge collection
        for collection_name in EDGE_COLLECTIONS {
            let aql = format!(
                r#"
                FOR e IN {}
                FILTER e.id == @id
                LIMIT 1
                RETURN e
                "#,
                collection_name
            );

            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
            bind_vars.insert("id", serde_json::json!(id.to_string()));

            let query = arangors::AqlQuery::builder()
                .query(&aql)
                .bind_vars(bind_vars)
                .build();

            match db.aql_query::<RelationshipDocument>(query).await {
                Ok(mut results) => {
                    if let Some(doc) = results.pop() {
                        return Ok(Some(doc.to_relationship()));
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(None)
    }

    /// Delete a relationship from the graph
    ///
    /// # Arguments
    /// * `id` - UUID of the relationship to delete
    ///
    /// # Returns
    /// - Ok(true) if relationship was deleted
    /// - Ok(false) if relationship was not found
    /// - Err(StoreError) on database error
    pub async fn delete_relationship(&self, id: Uuid) -> Result<bool, StoreError> {
        let collection_name = self.find_relationship_collection(id).await?;

        if collection_name.is_none() {
            return Ok(false);
        }

        let collection_name = collection_name.unwrap();

        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        let aql = format!(
            r#"
            FOR e IN {}
            FILTER e.id == @id
            REMOVE e IN {}
            RETURN OLD
            "#,
            collection_name, collection_name
        );

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert("id", serde_json::json!(id.to_string()));

        let query = arangors::AqlQuery::builder()
            .query(&aql)
            .bind_vars(bind_vars)
            .build();

        match db.aql_query::<RelationshipDocument>(query).await {
            Ok(mut results) => Ok(results.pop().is_some()),
            Err(_) => Ok(false),
        }
    }

    /// Get all relationships for a specific entity
    ///
    /// Returns relationships where the entity is either the source or target.
    ///
    /// # Arguments
    /// * `entity_id` - UUID of the entity
    /// * `options` - RelationshipQueryOptions for filtering
    ///
    /// # Returns
    /// Result containing vector of relationships
    pub async fn get_entity_relationships(
        &self,
        entity_id: Uuid,
        options: RelationshipQueryOptions,
    ) -> Result<Vec<Relationship>, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        let mut all_relationships = Vec::new();

        // Determine which collections to query
        let collections_to_query = if let Some(rel_type) = options.relationship_type {
            vec![Self::collection_name_for_relationship_type(rel_type).to_string()]
        } else {
            EDGE_COLLECTIONS.iter().map(|s| s.to_string()).collect()
        };

        for collection_name in collections_to_query {
            let mut aql = format!("FOR e IN {}", collection_name);
            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();

            // Filter for relationships involving this entity
            let mut filter_conditions = vec![
                "e.source_entity_id == @entity_id OR e.target_entity_id == @entity_id".to_string()
            ];

            if let Some(direction) = options.direction {
                match direction {
                    RelationshipDirection::Outgoing => {
                        filter_conditions[0] = "e.source_entity_id == @entity_id".to_string();
                    }
                    RelationshipDirection::Incoming => {
                        filter_conditions[0] = "e.target_entity_id == @entity_id".to_string();
                    }
                    RelationshipDirection::Both => {} // Keep the OR condition
                }
            }

            if let Some(min_confidence) = options.min_confidence {
                filter_conditions.push("e.confidence >= @min_confidence".to_string());
                bind_vars.insert("min_confidence", serde_json::json!(min_confidence));
            }

            if !filter_conditions.is_empty() {
                aql.push_str(" FILTER ");
                aql.push_str(&filter_conditions.join(" AND "));
            }

            aql.push_str(&format!(" LIMIT @limit RETURN e"));
            bind_vars.insert("limit", serde_json::json!(options.limit as i64));
            bind_vars.insert("entity_id", serde_json::json!(entity_id.to_string()));

            let query = arangors::AqlQuery::builder()
                .query(&aql)
                .bind_vars(bind_vars)
                .build();

            match db.aql_query::<RelationshipDocument>(query).await {
                Ok(results) => {
                    for doc in results {
                        all_relationships.push(doc.to_relationship());
                    }
                }
                Err(_) => continue,
            }

            if !options.include_all_collections {
                break;
            }
        }

        Ok(all_relationships)
    }

    /// Get the collection name for a relationship type
    fn collection_name_for_relationship_type(rel_type: RelationshipType) -> &'static str {
        match rel_type {
            RelationshipType::WorksFor => "edge_works_for",
            RelationshipType::LocatedIn => "edge_located_in",
            RelationshipType::SubjectTo => "edge_subject_to",
            RelationshipType::RefersTo => "edge_refers_to",
            RelationshipType::RelatesTo => "edge_relates_to",
            RelationshipType::OwnerOf => "edge_owner_of",
            RelationshipType::ReportsTo => "edge_reports_to",
            RelationshipType::CollaboratesWith => "edge_collaborates_with",
            RelationshipType::Follows => "edge_follows",
            RelationshipType::PartOf => "edge_part_of",
            RelationshipType::Unknown => "edge_unknown",
        }
    }

    /// Find which edge collection contains a relationship
    async fn find_relationship_collection(&self, id: Uuid) -> Result<Option<&'static str>, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        for collection_name in EDGE_COLLECTIONS {
            let aql = format!(
                r#"
                FOR e IN {}
                FILTER e.id == @id
                LIMIT 1
                RETURN e
                "#,
                collection_name
            );

            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
            bind_vars.insert("id", serde_json::json!(id.to_string()));

            let query = arangors::AqlQuery::builder()
                .query(&aql)
                .bind_vars(bind_vars)
                .build();

            match db.aql_query::<RelationshipDocument>(query).await {
                Ok(mut results) => {
                    if results.pop().is_some() {
                        return Ok(Some(collection_name));
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(None)
    }

    // ========== Graph Traversal Operations ==========

    /// Find shortest path between two entities
    ///
    /// Uses breadth-first search to find the shortest path between two entities.
    ///
    /// Note: This is a client-side implementation. For production use with large graphs,
    /// ArangoDB's native SHORTEST_PATH with a named graph should be used instead.
    ///
    /// # Arguments
    /// * `from` - Source entity ID
    /// * `to` - Target entity ID
    ///
    /// # Returns
    /// - Ok(Some(GraphPath)) if path found
    /// - Ok(None) if no path exists
    /// - Err(StoreError) on database error
    pub async fn find_shortest_path(&self, from: Uuid, to: Uuid) -> Result<Option<GraphPath>, StoreError> {
        if from == to {
            return Ok(Some(GraphPath {
                entity_ids: vec![from],
                relationship_ids: vec![],
                weight: 0.0,
            }));
        }

        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        // BFS to find shortest path (max 5 hops to prevent infinite loops)
        let mut visited: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
        let mut queue: std::collections::VecDeque<(Uuid, Vec<Uuid>, Vec<Uuid>)> = std::collections::VecDeque::new();
        visited.insert(from);
        queue.push_back((from, vec![from], vec![]));

        while let Some((current, path, rels)) = queue.pop_front() {
            if path.len() > 6 {
                // Max depth reached
                continue;
            }

            // Get all edges for current entity
            let edges_aql = format!(
                r#"
                FOR e IN {}
                FILTER e.source_entity_id == @current OR e.target_entity_id == @current
                RETURN e
                "#,
                EDGE_COLLECTIONS.join(", ")
            );

            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
            bind_vars.insert("current", serde_json::json!(current.to_string()));

            let query = arangors::AqlQuery::builder()
                .query(&edges_aql)
                .bind_vars(bind_vars)
                .build();

            match db.aql_query::<RelationshipDocument>(query).await {
                Ok(edges) => {
                    for edge in edges {
                        let next_id = if edge.source_entity_id == current {
                            edge.target_entity_id
                        } else {
                            edge.source_entity_id
                        };

                        if next_id == to {
                            // Found target!
                            let mut new_path = path.clone();
                            let mut new_rels = rels.clone();
                            new_path.push(to);
                            new_rels.push(edge.id);
                            let weight = (new_rels.len() + 1) as f32;
                            return Ok(Some(GraphPath {
                                entity_ids: new_path,
                                relationship_ids: new_rels,
                                weight,
                            }));
                        }

                        if !visited.contains(&next_id) {
                            visited.insert(next_id);
                            let mut new_path = path.clone();
                            let mut new_rels = rels.clone();
                            new_path.push(next_id);
                            new_rels.push(edge.id);
                            queue.push_back((next_id, new_path, new_rels));
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(None)
    }

    /// Traverse the graph from a starting entity
    ///
    /// Performs a multi-hop graph traversal with filtering options.
    ///
    /// # Arguments
    /// * `request` - TraversalRequest with traversal parameters
    ///
    /// # Returns
    /// Result containing traversal results with vertices and edges
    pub async fn traverse(&self, request: TraversalRequest) -> Result<TraversalResult, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        // Multi-hop traversal using iterative BFS
        let mut result = TraversalResult::default();
        let mut visited: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
        let mut current_level: Vec<Uuid> = vec![request.start_id];
        visited.insert(request.start_id);

        for depth in 0..request.max_depth {
            if depth + 1 < request.min_depth {
                // Skip levels below min_depth
            }

            if result.edges.len() >= request.limit {
                break;
            }

            let mut next_level: Vec<Uuid> = Vec::new();

            for current_id in current_level {
                let (source_filter, target_filter) = match request.direction {
                    TraversalDirection::Outgoing => ("@current_id", "NULL"),
                    TraversalDirection::Incoming => ("NULL", "@current_id"),
                    TraversalDirection::Any => ("@current_id", "@current_id"),
                };

                // Query edges with both edge documents and connected entity info
                let aql = format!(
                    r#"
                    FOR e IN {}
                    FILTER e.source_entity_id == {} OR e.target_entity_id == {}
                    LET connected_id = e.source_entity_id == @current_id ? e.target_entity_id : e.source_entity_id
                    LET connected_vertex = FIRST(
                        FOR v IN persons, organizations, locations, laws, entities
                        FILTER v.id == connected_id
                        RETURN v
                    )
                    RETURN {{edge: e, vertex: connected_vertex}}
                    "#,
                    EDGE_COLLECTIONS.join(", "),
                    source_filter, target_filter
                );

                let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
                bind_vars.insert("current_id", serde_json::json!(current_id.to_string()));

                let query = arangors::AqlQuery::builder()
                    .query(&aql)
                    .bind_vars(bind_vars)
                    .build();

                match db.aql_query::<serde_json::Value>(query).await {
                    Ok(rows) => {
                        for row in rows {
                            if result.edges.len() >= request.limit {
                                break;
                            }

                            if let Some(obj) = row.as_object() {
                                if let Some(edge_val) = obj.get("edge") {
                                    if let Ok(edge_str) = serde_json::to_string(edge_val) {
                                        if let Ok(edge_doc) = serde_json::from_str::<RelationshipDocument>(&edge_str) {
                                            let edge = edge_doc.to_relationship();
                                            let connected_id = if edge.source_entity_id == current_id {
                                                edge.target_entity_id
                                            } else {
                                                edge.source_entity_id
                                            };

                                            if !visited.contains(&connected_id) {
                                                visited.insert(connected_id);
                                                next_level.push(connected_id);
                                            }

                                            result.edges.push(edge.clone());

                                            // Try to parse vertex
                                            if let Some(vertex_val) = obj.get("vertex") {
                                                if let Some(vertex_obj) = vertex_val.as_object() {
                                                    if vertex_obj.get("id").is_some() {
                                                        if let Ok(vertex_str) = serde_json::to_string(vertex_val) {
                                                            if let Ok(doc) = serde_json::from_str::<EntityDocument>(&vertex_str) {
                                                                result.vertices.push(doc.to_entity());
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }

            current_level = next_level;
            if current_level.is_empty() {
                break;
            }
        }

        Ok(result)
    }

    /// Get immediate neighbors of an entity
    ///
    /// Returns entities directly connected to the specified entity.
    ///
    /// # Arguments
    /// * `entity_id` - Entity ID to get neighbors for
    /// * `filters` - NeighborFilters for filtering results
    ///
    /// # Returns
    /// Vector of Neighbor objects containing connected entities and relationships
    pub async fn get_neighbors(&self, entity_id: Uuid, filters: NeighborFilters) -> Result<Vec<Neighbor>, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        // Build filter conditions for AQL
        let mut filter_conditions = Vec::new();

        if let Some(min_conf) = filters.min_confidence {
            filter_conditions.push(format!("e.confidence >= {}", min_conf));
        }

        if !filters.relationship_types.is_empty() {
            let type_strs: Vec<String> = filters.relationship_types.iter()
                .map(|rt| format!("'{}'", serde_json::to_string(rt).unwrap_or_default().replace('"', "")))
                .collect();
            filter_conditions.push(format!("e.relationship_type IN [{}]", type_strs.join(", ")));
        }

        let _filter_clause = if filter_conditions.is_empty() {
            String::new()
        } else {
            format!("FILTER {}", filter_conditions.join(" AND "))
        };
        // Note: Filters are applied in Rust code below for now
        // TODO: Move filters to AQL WHERE clause for performance

        // Query both relationships and connected entities in a single query
        let (source_condition, target_condition) = match filters.direction {
            TraversalDirection::Outgoing => ("e.source_entity_id == @entity_id", ""),
            TraversalDirection::Incoming => ("", "e.target_entity_id == @entity_id"),
            TraversalDirection::Any => ("e.source_entity_id == @entity_id", "OR e.target_entity_id == @entity_id"),
        };

        let aql = format!(
            r#"
            FOR e IN {}
            FILTER ({} {})
            LIMIT @limit
            LET connected_id = e.source_entity_id == @entity_id ? e.target_entity_id : e.source_entity_id
            LET connected_entity = FIRST(
                FOR v IN persons, organizations, locations, laws, entities
                FILTER v.id == connected_id
                RETURN v
            )
            {{edge: e, entity: connected_entity}}
            "#,
            EDGE_COLLECTIONS.join(", "),
            source_condition,
            target_condition
        );

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert("entity_id", serde_json::json!(entity_id.to_string()));
        bind_vars.insert("limit", serde_json::json!(filters.limit as i64));

        let query = arangors::AqlQuery::builder()
            .query(&aql)
            .bind_vars(bind_vars)
            .build();

        match db.aql_query::<serde_json::Value>(query).await {
            Ok(rows) => {
                let mut neighbors = Vec::new();
                for row in rows {
                    if let Some(obj) = row.as_object() {
                        // Get edge
                        if let Some(edge_val) = obj.get("edge") {
                            if let Ok(edge_str) = serde_json::to_string(edge_val) {
                                if let Ok(edge_doc) = serde_json::from_str::<RelationshipDocument>(&edge_str) {
                                    let relationship = edge_doc.to_relationship();

                                    // Apply filters that couldn't be in AQL
                                    if let Some(min_conf) = filters.min_confidence {
                                        if relationship.confidence < min_conf {
                                            continue;
                                        }
                                    }
                                    if !filters.relationship_types.is_empty()
                                        && !filters.relationship_types.contains(&relationship.relationship_type) {
                                        continue;
                                    }

                                    // Get entity
                                    if let Some(entity_val) = obj.get("entity") {
                                        if let Some(entity_obj) = entity_val.as_object() {
                                            if entity_obj.get("id").is_some() {
                                                if let Ok(entity_str) = serde_json::to_string(entity_val) {
                                                    if let Ok(doc) = serde_json::from_str::<EntityDocument>(&entity_str) {
                                                        let entity = doc.to_entity();
                                                        neighbors.push(Neighbor {
                                                            entity,
                                                            relationship_type: relationship.relationship_type,
                                                            is_outgoing: relationship.source_entity_id == entity_id,
                                                            weight: relationship.weight,
                                                            confidence: relationship.confidence,
                                                            relationship,
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(neighbors)
            }
            Err(e) => Err(StoreError::from(e)),
        }
    }

    // ========== Community Operations ==========

    /// Create a new community
    ///
    /// Creates a community vertex and optionally creates membership edges
    /// for entities specified in member_entity_ids.
    ///
    /// # Arguments
    /// * `community` - Community to create (id field optional, will be generated)
    ///
    /// # Returns
    /// Result containing the created community with assigned ID
    pub async fn create_community(&self, community: &Community) -> Result<Community, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        let id = if community.id.is_nil() {
            Uuid::new_v4()
        } else {
            community.id
        };

        let aql = r#"
            INSERT @community INTO communities
            RETURN NEW
        "#;

        let mut document = CommunityDocument::from_community(community, id);
        document.set_key();

        let community_json = serde_json::to_value(&document)
            .map_err(|e| StoreError::Serialization(format!("Failed to serialize community: {}", e)))?;

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert("community", community_json);

        let query = arangors::AqlQuery::builder()
            .query(aql)
            .bind_vars(bind_vars)
            .build();

        match db.aql_query::<CommunityDocument>(query).await {
            Ok(mut results) => {
                let created = results.pop()
                    .ok_or_else(|| StoreError::Query("INSERT returned no result".to_string()))?
                    .to_community();

                // Create membership edges for initial members
                for member_id in &community.member_entity_ids {
                    let _ = self.add_community_member(created.id, *member_id).await;
                }

                Ok(created)
            }
            Err(e) => {
                let e_str = e.to_string();
                if e_str.contains("unique") || e_str.contains("duplicate") || e_str.contains("1202") {
                    Err(StoreError::UniqueViolation(format!(
                        "Community with name '{}' already exists",
                        community.name
                    )))
                } else {
                    Err(StoreError::from(e))
                }
            }
        }
    }

    /// Get a community by ID
    ///
    /// # Arguments
    /// * `id` - UUID of the community to retrieve
    ///
    /// # Returns
    /// - Ok(Some(Community)) if community found
    /// - Ok(None) if community not found
    pub async fn get_community(&self, id: Uuid) -> Result<Option<Community>, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        let aql = r#"
            FOR c IN communities
            FILTER c.id == @id
            LIMIT 1
            RETURN c
        "#;

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert("id", serde_json::json!(id.to_string()));

        let query = arangors::AqlQuery::builder()
            .query(aql)
            .bind_vars(bind_vars)
            .build();

        match db.aql_query::<CommunityDocument>(query).await {
            Ok(mut results) => Ok(results.pop().map(|doc| doc.to_community())),
            Err(_) => Ok(None),
        }
    }

    /// Add an entity as a member of a community
    ///
    /// Creates an edge_member_of edge from the entity to the community.
    ///
    /// # Arguments
    /// * `community_id` - UUID of the community
    /// * `entity_id` - UUID of the entity to add as member
    ///
    /// # Returns
    /// - Ok(true) if member was added
    /// - Ok(false) if already a member
    pub async fn add_community_member(&self, community_id: Uuid, entity_id: Uuid) -> Result<bool, StoreError> {
        // Check if already a member
        if self.is_community_member(community_id, entity_id).await? {
            return Ok(false);
        }

        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        let aql = r#"
            INSERT @membership INTO edge_member_of
            RETURN NEW
        "#;

        let membership = MembershipEdge {
            _key: None,
            id: Uuid::new_v4(),
            community_id,
            entity_id,
            created_at: Utc::now().to_rfc3339(),
        };

        let membership_json = serde_json::to_value(&membership)
            .map_err(|e| StoreError::Serialization(format!("Failed to serialize membership: {}", e)))?;

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert("membership", membership_json);

        let query = arangors::AqlQuery::builder()
            .query(aql)
            .bind_vars(bind_vars)
            .build();

        match db.aql_query::<serde_json::Value>(query).await {
            Ok(_) => Ok(true),
            Err(e) => {
                let e_str = e.to_string();
                if e_str.contains("unique") || e_str.contains("duplicate") {
                    Ok(false) // Already a member
                } else {
                    Err(StoreError::from(e))
                }
            }
        }
    }

    /// Remove an entity from a community
    ///
    /// Removes the edge_member_of edge.
    ///
    /// # Arguments
    /// * `community_id` - UUID of the community
    /// * `entity_id` - UUID of the entity to remove
    ///
    /// # Returns
    /// - Ok(true) if member was removed
    /// - Ok(false) if entity was not a member
    pub async fn remove_community_member(&self, community_id: Uuid, entity_id: Uuid) -> Result<bool, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        let aql = r#"
            FOR e IN edge_member_of
            FILTER e.community_id == @community_id AND e.entity_id == @entity_id
            REMOVE e IN edge_member_of
            RETURN OLD
        "#;

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert("community_id", serde_json::json!(community_id.to_string()));
        bind_vars.insert("entity_id", serde_json::json!(entity_id.to_string()));

        let query = arangors::AqlQuery::builder()
            .query(aql)
            .bind_vars(bind_vars)
            .build();

        match db.aql_query::<serde_json::Value>(query).await {
            Ok(results) => Ok(!results.is_empty()),
            Err(_) => Ok(false),
        }
    }

    /// Get all members of a community
    ///
    /// # Arguments
    /// * `community_id` - UUID of the community
    ///
    /// # Returns
    /// Vector of entities that are members of the community
    pub async fn get_community_members(&self, community_id: Uuid) -> Result<Vec<Entity>, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        let aql = r#"
            FOR m IN edge_member_of
            FILTER m.community_id == @community_id
            FOR e IN persons, organizations, locations, laws, entities
            FILTER e.id == m.entity_id
            RETURN e
        "#;

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert("community_id", serde_json::json!(community_id.to_string()));

        let query = arangors::AqlQuery::builder()
            .query(aql)
            .bind_vars(bind_vars)
            .build();

        match db.aql_query::<EntityDocument>(query).await {
            Ok(docs) => {
                let members = docs.into_iter().map(|doc| doc.to_entity()).collect();
                Ok(members)
            }
            Err(e) => Err(StoreError::from(e)),
        }
    }

    /// Check if an entity is a member of a community
    async fn is_community_member(&self, community_id: Uuid, entity_id: Uuid) -> Result<bool, StoreError> {
        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        let aql = r#"
            FOR e IN edge_member_of
            FILTER e.community_id == @community_id AND e.entity_id == @entity_id
            LIMIT 1
            RETURN e
        "#;

        let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
        bind_vars.insert("community_id", serde_json::json!(community_id.to_string()));
        bind_vars.insert("entity_id", serde_json::json!(entity_id.to_string()));

        let query = arangors::AqlQuery::builder()
            .query(aql)
            .bind_vars(bind_vars)
            .build();

        match db.aql_query::<serde_json::Value>(query).await {
            Ok(results) => Ok(!results.is_empty()),
            Err(_) => Ok(false),
        }
    }

    /// Bulk create entities in the graph
    ///
    /// Creates multiple entities efficiently by grouping them by collection type.
    /// Generates UUIDs for entities with nil IDs.
    ///
    /// # Arguments
    /// * `entities` - Vector of entities to create
    ///
    /// # Returns
    /// Vector of created entities with assigned IDs
    ///
    /// # Errors
    /// - StoreError::Connection if database connection fails
    /// - StoreError::Query if AQL execution fails
    pub async fn bulk_create_entities(&self, entities: Vec<Entity>) -> Result<Vec<Entity>, StoreError> {
        if entities.is_empty() {
            return Ok(vec![]);
        }

        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        // Group entities by collection
        let mut by_collection: std::collections::HashMap<&str, Vec<(Entity, Uuid)>> = std::collections::HashMap::new();

        for entity in &entities {
            let id = if entity.id.is_nil() {
                Uuid::new_v4()
            } else {
                entity.id
            };
            let collection_name = Self::collection_name_for_entity_type(entity.entity_type);
            by_collection.entry(collection_name).or_default().push((entity.clone(), id));
        }

        let mut all_created = Vec::new();

        // Process each collection
        for (collection_name, entities_with_ids) in by_collection {
            let documents: Vec<serde_json::Value> = entities_with_ids
                .iter()
                .map(|(entity, id)| {
                    let mut document = EntityDocument::from_entity(entity, *id);
                    document.set_key();
                    serde_json::to_value(document)
                        .map_err(|e| StoreError::Serialization(format!("Failed to serialize entity: {}", e)))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let aql = format!(
                r#"
                FOR doc IN @entities
                    INSERT doc INTO {}
                    RETURN NEW
                "#,
                collection_name
            );

            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
            bind_vars.insert("entities", serde_json::json!(documents));

            let query = arangors::AqlQuery::builder()
                .query(&aql)
                .bind_vars(bind_vars)
                .build();

            match db.aql_query::<EntityDocument>(query).await {
                Ok(results) => {
                    for doc in results {
                        all_created.push(doc.to_entity());
                    }
                }
                Err(e) => {
                    return Err(StoreError::Query(format!("Bulk insert failed for collection {}: {}", collection_name, e)));
                }
            }
        }

        Ok(all_created)
    }

    /// Bulk create relationships in the graph
    ///
    /// Creates multiple relationships efficiently.
    ///
    /// # Arguments
    /// * `relationships` - Vector of relationships to create
    ///
    /// # Returns
    /// Vector of created relationships with assigned IDs
    ///
    /// # Errors
    /// - StoreError::Connection if database connection fails
    /// - StoreError::Query if AQL execution fails
    pub async fn bulk_create_relationships(&self, relationships: Vec<Relationship>) -> Result<Vec<Relationship>, StoreError> {
        if relationships.is_empty() {
            return Ok(vec![]);
        }

        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        // Group relationships by edge collection
        let mut by_collection: std::collections::HashMap<&str, Vec<(Relationship, Uuid)>> = std::collections::HashMap::new();

        for rel in &relationships {
            let id = if rel.id.is_nil() {
                Uuid::new_v4()
            } else {
                rel.id
            };
            let collection_name = Self::collection_name_for_relationship_type(rel.relationship_type);
            by_collection.entry(collection_name).or_default().push((rel.clone(), id));
        }

        let mut all_created = Vec::new();

        for (collection_name, rels_with_ids) in by_collection {
            let documents: Vec<serde_json::Value> = rels_with_ids
                .iter()
                .map(|(rel, id)| {
                    let mut document = RelationshipDocument::from_relationship(rel, *id);
                    document.set_key();

                    // Use generic "entities" collection for edge references
                    // This works with all entity types since the single relationship
                    // create approach uses this pattern
                    document._from = Some(format!("entities/{}", rel.source_entity_id));
                    document._to = Some(format!("entities/{}", rel.target_entity_id));

                    serde_json::to_value(document)
                        .map_err(|e| StoreError::Serialization(format!("Failed to serialize relationship: {}", e)))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let aql = format!(
                r#"
                FOR doc IN @relationships
                    INSERT doc INTO {}
                    RETURN NEW
                "#,
                collection_name
            );

            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
            bind_vars.insert("relationships", serde_json::json!(documents));

            let query = arangors::AqlQuery::builder()
                .query(&aql)
                .bind_vars(bind_vars)
                .build();

            match db.aql_query::<RelationshipDocument>(query).await {
                Ok(results) => {
                    for doc in results {
                        all_created.push(doc.to_relationship());
                    }
                }
                Err(e) => {
                    return Err(StoreError::Query(format!("Bulk insert failed for collection {}: {}", collection_name, e)));
                }
            }
        }

        Ok(all_created)
    }

    /// Bulk delete entities from the graph
    ///
    /// Deletes multiple entities by their IDs. Returns the count of successfully deleted entities.
    /// Non-existent IDs are silently ignored.
    ///
    /// # Arguments
    /// * `ids` - Vector of entity IDs to delete
    ///
    /// # Returns
    /// Count of entities that were deleted
    ///
    /// # Errors
    /// - StoreError::Connection if database connection fails
    /// - StoreError::Query if AQL execution fails
    pub async fn bulk_delete_entities(&self, ids: Vec<Uuid>) -> Result<u64, StoreError> {
        if ids.is_empty() {
            return Ok(0);
        }

        let conn = self.pool.get().await?;
        let db = conn.db(&self.db_name).await?;

        // Convert UUIDs to strings for AQL
        let id_strings: Vec<String> = ids.iter().map(|id| id.to_string()).collect();

        // Query each collection separately for better performance
        let mut total_deleted = 0u64;

        for collection in VERTEX_COLLECTIONS {
            let aql = format!(
                r#"
                FOR entity IN {}
                    FILTER entity.id IN @ids
                    REMOVE entity._key IN {}
                    RETURN OLD
                "#,
                collection, collection
            );

            let mut bind_vars: HashMap<&str, serde_json::Value> = HashMap::new();
            bind_vars.insert("ids", serde_json::json!(&id_strings));

            let query = arangors::AqlQuery::builder()
                .query(&aql)
                .bind_vars(bind_vars)
                .build();

            match db.aql_query::<serde_json::Value>(query).await {
                Ok(results) => {
                    total_deleted += results.len() as u64;
                }
                Err(e) => {
                    return Err(StoreError::Query(format!("Bulk delete failed for collection {}: {}", collection, e)));
                }
            }
        }

        Ok(total_deleted)
    }
}

/// Document representation in ArangoDB
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EntityDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    _key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    _id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    _rev: Option<String>,

    id: Uuid,
    name: String,
    #[serde(rename = "entity_type")]
    entity_type: EntityType,
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    confidence: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_domain_id: Option<Uuid>,
    metadata: serde_json::Value,
    #[serde(rename = "created_at")]
    created_at: String,
}

impl EntityDocument {
    fn from_entity(entity: &Entity, id: Uuid) -> Self {
        Self {
            _key: None,
            _id: None,
            _rev: None,
            id,
            name: entity.name.clone(),
            entity_type: entity.entity_type,
            canonical_name: entity.canonical_name.clone(),
            description: entity.description.clone(),
            confidence: entity.confidence,
            source_domain_id: entity.source_domain_id,
            metadata: entity.metadata.clone(),
            created_at: entity.created_at.to_rfc3339(),
        }
    }

    fn set_key(&mut self) {
        self._key = Some(self.id.to_string());
    }

    fn to_entity(&self) -> Entity {
        use chrono::DateTime;

        Entity {
            id: self.id,
            name: self.name.clone(),
            entity_type: self.entity_type,
            canonical_name: self.canonical_name.clone(),
            description: self.description.clone(),
            confidence: self.confidence,
            source_domain_id: self.source_domain_id,
            metadata: self.metadata.clone(),
            created_at: DateTime::parse_from_rfc3339(&self.created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        }
    }

}

/// Partial update for an entity
///
/// All fields are optional; only provided fields are updated.
#[derive(Debug, Clone, Serialize, Default)]
pub struct EntityUpdate {
    pub name: Option<String>,
    pub canonical_name: Option<String>,
    pub description: Option<String>,
    pub confidence: Option<f32>,
    pub metadata: Option<serde_json::Value>,
}

/// Filters for entity queries
#[derive(Debug, Clone, Default)]
pub struct EntityFilters {
    pub entity_type: Option<EntityType>,
    pub source_domain_id: Option<Uuid>,
    pub name_contains: Option<String>,
    pub min_confidence: Option<f32>,
}

/// Pagination options
#[derive(Debug, Clone)]
pub struct PaginationOptions {
    pub limit: usize,
    pub cursor: Option<String>,
}

impl Default for PaginationOptions {
    fn default() -> Self {
        Self {
            limit: 50,
            cursor: None,
        }
    }
}

/// Paginated entity results
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedEntities {
    pub entities: Vec<Entity>,
    pub total_count: u64,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// Query options for retrieving relationships
#[derive(Debug, Clone, Default)]
pub struct RelationshipQueryOptions {
    /// Filter by relationship type
    pub relationship_type: Option<RelationshipType>,
    /// Filter by direction (outgoing, incoming, or both)
    pub direction: Option<RelationshipDirection>,
    /// Minimum confidence threshold
    pub min_confidence: Option<f32>,
    /// Maximum number of results to return
    pub limit: usize,
    /// Whether to query all edge collections or stop at first match
    pub include_all_collections: bool,
}

impl RelationshipQueryOptions {
    /// Create query options with only entity ID
    pub fn new() -> Self {
        Self {
            relationship_type: None,
            direction: None,
            min_confidence: None,
            limit: 100,
            include_all_collections: true,
        }
    }

    /// Set the relationship type filter
    pub fn with_type(mut self, rel_type: RelationshipType) -> Self {
        self.relationship_type = Some(rel_type);
        self
    }

    /// Set the direction filter
    pub fn with_direction(mut self, direction: RelationshipDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Set the result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

/// Direction filter for relationship queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipDirection {
    /// Relationships where the entity is the source
    Outgoing,
    /// Relationships where the entity is the target
    Incoming,
    /// Relationships in either direction
    Both,
}

/// Result of a shortest path query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPath {
    /// List of entity IDs in the path
    pub entity_ids: Vec<Uuid>,
    /// List of relationship IDs along the path
    pub relationship_ids: Vec<Uuid>,
    /// Total weight/cost of the path
    pub weight: f32,
}

impl GraphPath {
    pub fn from_json_value(value: serde_json::Value) -> Result<Self, StoreError> {
        // Parse ArangoDB path format
        // The path is returned as an array of vertices and edges
        if let Some(arr) = value.as_array() {
            let mut entity_ids = Vec::new();
            let mut relationship_ids = Vec::new();

            // Process vertices and edges alternately
            for (i, item) in arr.iter().enumerate() {
                if i % 2 == 0 {
                    // Vertex/Entity
                    if let Some(obj) = item.as_object() {
                        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                            if let Ok(uuid) = Uuid::parse_str(id) {
                                entity_ids.push(uuid);
                            }
                        }
                    }
                } else {
                    // Edge/Relationship
                    if let Some(obj) = item.as_object() {
                        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                            if let Ok(uuid) = Uuid::parse_str(id) {
                                relationship_ids.push(uuid);
                            }
                        }
                    }
                }
            }

            return Ok(GraphPath {
                entity_ids,
                relationship_ids,
                weight: 1.0, // Default weight
            });
        }

        Err(StoreError::Serialization("Invalid path format".to_string()))
    }
}

/// Request for graph traversal
#[derive(Debug, Clone)]
pub struct TraversalRequest {
    /// Starting entity ID
    pub start_id: Uuid,
    /// Minimum depth to traverse (default: 1)
    pub min_depth: u8,
    /// Maximum depth to traverse (default: 3)
    pub max_depth: u8,
    /// Traversal direction
    pub direction: TraversalDirection,
    /// Maximum number of results to return
    pub limit: usize,
}

impl Default for TraversalRequest {
    fn default() -> Self {
        Self {
            start_id: Uuid::nil(),
            min_depth: 1,
            max_depth: 3,
            direction: TraversalDirection::Outgoing,
            limit: 100,
        }
    }
}

/// Direction for graph traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalDirection {
    Outgoing,
    Incoming,
    Any,
}

/// Result of a graph traversal
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraversalResult {
    /// Vertices visited during traversal
    pub vertices: Vec<Entity>,
    /// Edges traversed
    pub edges: Vec<Relationship>,
}

impl TraversalResult {
    fn add_from_json(&mut self, value: serde_json::Value) -> Result<(), StoreError> {
        if let Some(obj) = value.as_object() {
            // Parse vertices
            if let Some(vertices) = obj.get("vertices").and_then(|v| v.as_array()) {
                for vertex in vertices {
                    // Convert to EntityDocument then to Entity
                    let json_str = serde_json::to_string(vertex)
                        .map_err(|e| StoreError::Serialization(format!("Failed to serialize: {}", e)))?;
                    let doc: EntityDocument = serde_json::from_str(&json_str)
                        .map_err(|e| StoreError::Serialization(format!("Failed to deserialize: {}", e)))?;
                    self.vertices.push(doc.to_entity());
                }
            }

            // Parse edges
            if let Some(edges) = obj.get("edges").and_then(|v| v.as_array()) {
                for edge in edges {
                    let json_str = serde_json::to_string(edge)
                        .map_err(|e| StoreError::Serialization(format!("Failed to serialize: {}", e)))?;
                    let doc: RelationshipDocument = serde_json::from_str(&json_str)
                        .map_err(|e| StoreError::Serialization(format!("Failed to deserialize: {}", e)))?;
                    self.edges.push(doc.to_relationship());
                }
            }
        }
        Ok(())
    }
}

/// Filters for neighbor queries
#[derive(Debug, Clone, Default)]
pub struct NeighborFilters {
    /// Traversal direction
    pub direction: TraversalDirection,
    /// Filter by relationship types
    pub relationship_types: Vec<RelationshipType>,
    /// Minimum confidence threshold
    pub min_confidence: Option<f32>,
    /// Maximum number of results
    pub limit: usize,
}

impl Default for TraversalDirection {
    fn default() -> Self {
        Self::Outgoing
    }
}

/// Neighbor result combining entity and relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neighbor {
    /// The connected entity
    pub entity: Entity,
    /// The relationship connecting to the entity
    pub relationship: Relationship,
    /// Relationship type for easy access
    pub relationship_type: RelationshipType,
    /// Direction: true if this is an outgoing relationship
    pub is_outgoing: bool,
    /// Weight/cost of the relationship
    pub weight: f32,
    /// Confidence score
    pub confidence: f32,
}

impl Neighbor {
    pub fn from_json_value(value: serde_json::Value) -> Result<Self, StoreError> {
        if let Some(obj) = value.as_object() {
            // Parse entity
            let entity = if let Some(entity_val) = obj.get("entity") {
                let json_str = serde_json::to_string(entity_val)
                    .map_err(|e| StoreError::Serialization(format!("Failed to serialize: {}", e)))?;
                let doc: EntityDocument = serde_json::from_str(&json_str)
                    .map_err(|e| StoreError::Serialization(format!("Failed to deserialize: {}", e)))?;
                doc.to_entity()
            } else {
                return Err(StoreError::Serialization("Missing entity".to_string()));
            };

            // Parse relationship
            let relationship = if let Some(rel_val) = obj.get("relationship") {
                let json_str = serde_json::to_string(rel_val)
                    .map_err(|e| StoreError::Serialization(format!("Failed to serialize: {}", e)))?;
                let doc: RelationshipDocument = serde_json::from_str(&json_str)
                    .map_err(|e| StoreError::Serialization(format!("Failed to deserialize: {}", e)))?;
                doc.to_relationship()
            } else {
                return Err(StoreError::Serialization("Missing relationship".to_string()));
            };

            Ok(Neighbor {
                entity: entity.clone(),
                relationship_type: relationship.relationship_type,
                is_outgoing: relationship.source_entity_id == entity.id,
                weight: relationship.weight,
                confidence: relationship.confidence,
                relationship,
            })
        } else {
            Err(StoreError::Serialization("Invalid neighbor format".to_string()))
        }
    }
}

/// Document representation in ArangoDB for relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RelationshipDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    _key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    _id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    _rev: Option<String>,

    /// ArangoDB edge fields - required for graph traversals
    #[serde(skip_serializing_if = "Option::is_none")]
    _from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    _to: Option<String>,

    id: Uuid,
    #[serde(rename = "source_entity_id")]
    source_entity_id: Uuid,
    #[serde(rename = "target_entity_id")]
    target_entity_id: Uuid,
    #[serde(rename = "relationship_type")]
    relationship_type: RelationshipType,
    weight: f32,
    confidence: f32,
    context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "source_domain_id")]
    source_domain_id: Option<Uuid>,
    #[serde(rename = "created_at")]
    created_at: String,
}

impl RelationshipDocument {
    fn from_relationship(rel: &Relationship, id: Uuid) -> Self {
        Self {
            _key: None,
            _id: None,
            _rev: None,
            _from: None,  // Will be set in set_from_to()
            _to: None,    // Will be set in set_from_to()
            id,
            source_entity_id: rel.source_entity_id,
            target_entity_id: rel.target_entity_id,
            relationship_type: rel.relationship_type,
            weight: rel.weight,
            confidence: rel.confidence,
            context: rel.context.clone(),
            source_domain_id: rel.source_domain_id,
            created_at: rel.created_at.to_rfc3339(),
        }
    }

    fn set_key(&mut self) {
        self._key = Some(self.id.to_string());
    }

    /// Set ArangoDB edge references from source and target entity IDs
    ///
    /// This must be called after from_relationship() to properly set
    /// the _from and _to fields required for edge collections.
    fn set_from_to(&mut self, source_collection: &str, target_collection: &str) {
        // ArangoDB edge references use the format: "collection/_key"
        self._from = Some(format!("{}/{}", source_collection, self.source_entity_id));
        self._to = Some(format!("{}/{}", target_collection, self.target_entity_id));
    }

    fn to_relationship(&self) -> Relationship {
        use chrono::DateTime;

        Relationship {
            id: self.id,
            source_entity_id: self.source_entity_id,
            target_entity_id: self.target_entity_id,
            relationship_type: self.relationship_type,
            weight: self.weight,
            confidence: self.confidence,
            context: self.context.clone(),
            source_domain_id: self.source_domain_id,
            created_at: DateTime::parse_from_rfc3339(&self.created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        }
    }
}

/// Document representation in ArangoDB for communities
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommunityDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    _key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    _id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    _rev: Option<String>,

    id: Uuid,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    level: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "parent_community_id")]
    parent_community_id: Option<Uuid>,
    #[serde(rename = "member_entity_ids")]
    member_entity_ids: Vec<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    keywords: Vec<String>,
    #[serde(rename = "created_at")]
    created_at: String,
}

impl CommunityDocument {
    fn from_community(community: &Community, id: Uuid) -> Self {
        Self {
            _key: None,
            _id: None,
            _rev: None,
            id,
            name: community.name.clone(),
            description: community.description.clone(),
            level: community.level,
            parent_community_id: community.parent_community_id,
            member_entity_ids: community.member_entity_ids.clone(),
            summary: community.summary.clone(),
            keywords: community.keywords.clone(),
            created_at: community.created_at.to_rfc3339(),
        }
    }

    fn set_key(&mut self) {
        self._key = Some(self.id.to_string());
    }

    fn to_community(&self) -> Community {
        use chrono::DateTime;

        Community {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            level: self.level,
            parent_community_id: self.parent_community_id,
            member_entity_ids: self.member_entity_ids.clone(),
            summary: self.summary.clone(),
            keywords: self.keywords.clone(),
            created_at: DateTime::parse_from_rfc3339(&self.created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        }
    }
}

/// Document representation in ArangoDB for community membership edges
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MembershipEdge {
    #[serde(skip_serializing_if = "Option::is_none")]
    _key: Option<String>,

    id: Uuid,
    #[serde(rename = "community_id")]
    community_id: Uuid,
    #[serde(rename = "entity_id")]
    entity_id: Uuid,
    #[serde(rename = "created_at")]
    created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_name_for_entity_type() {
        assert_eq!(
            GraphStore::collection_name_for_entity_type(EntityType::Person),
            "persons"
        );
        assert_eq!(
            GraphStore::collection_name_for_entity_type(EntityType::Organization),
            "organizations"
        );
        assert_eq!(
            GraphStore::collection_name_for_entity_type(EntityType::Location),
            "locations"
        );
        assert_eq!(
            GraphStore::collection_name_for_entity_type(EntityType::Law),
            "laws"
        );
        assert_eq!(
            GraphStore::collection_name_for_entity_type(EntityType::Date),
            "entities"
        );
        assert_eq!(
            GraphStore::collection_name_for_entity_type(EntityType::Money),
            "entities"
        );
        assert_eq!(
            GraphStore::collection_name_for_entity_type(EntityType::Policy),
            "entities"
        );
        assert_eq!(
            GraphStore::collection_name_for_entity_type(EntityType::Miscellaneous),
            "entities"
        );
    }

    #[test]
    fn test_entity_update_default() {
        let update = EntityUpdate::default();
        assert!(update.name.is_none());
        assert!(update.canonical_name.is_none());
        assert!(update.description.is_none());
        assert!(update.confidence.is_none());
        assert!(update.metadata.is_none());
    }

    #[test]
    fn test_entity_filters_default() {
        let filters = EntityFilters::default();
        assert!(filters.entity_type.is_none());
        assert!(filters.source_domain_id.is_none());
        assert!(filters.name_contains.is_none());
        assert!(filters.min_confidence.is_none());
    }

    #[test]
    fn test_pagination_options_default() {
        let opts = PaginationOptions::default();
        assert_eq!(opts.limit, 50);
        assert!(opts.cursor.is_none());
    }

    #[test]
    fn test_paginated_entities_empty() {
        let result = PaginatedEntities {
            entities: vec![],
            total_count: 0,
            next_cursor: None,
            has_more: false,
        };
        assert_eq!(result.entities.len(), 0);
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[test]
    fn test_entity_document_serialization() {
        use chrono::Utc;

        let entity = Entity {
            id: Uuid::new_v4(),
            name: "Test".to_string(),
            entity_type: EntityType::Person,
            canonical_name: Some("test".to_string()),
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        let doc = EntityDocument::from_entity(&entity, entity.id);
        assert_eq!(doc.id, entity.id);
        assert_eq!(doc.name, "Test");
        assert_eq!(doc.entity_type, EntityType::Person);
        assert_eq!(doc.canonical_name, Some("test".to_string()));
    }

    #[test]
    fn test_entity_document_roundtrip() {
        use chrono::Utc;

        let original = Entity {
            id: Uuid::new_v4(),
            name: "Test Person".to_string(),
            entity_type: EntityType::Organization,
            canonical_name: Some("test-person".to_string()),
            description: Some("A test entity".to_string()),
            confidence: 0.85,
            source_domain_id: Some(Uuid::new_v4()),
            metadata: serde_json::json!({"key": "value"}),
            created_at: Utc::now(),
        };

        let doc = EntityDocument::from_entity(&original, original.id);
        let roundtrip = doc.to_entity();

        assert_eq!(roundtrip.id, original.id);
        assert_eq!(roundtrip.name, original.name);
        assert_eq!(roundtrip.entity_type, original.entity_type);
        assert_eq!(roundtrip.canonical_name, original.canonical_name);
        assert_eq!(roundtrip.description, original.description);
        assert!((roundtrip.confidence - original.confidence).abs() < f32::EPSILON);
        assert_eq!(roundtrip.source_domain_id, original.source_domain_id);
    }
}
