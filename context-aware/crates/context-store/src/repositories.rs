// =============================================================================
// Context Repositories
// =============================================================================

use arangors::client::reqwest::ReqwestClient;
use async_trait::async_trait;
use std::sync::Arc;

use context_core::{Context, ContextId, ContextRecord, ContextRecordId};
use iou_core::Timestamp;

use crate::{connection::ConnectionPool, StoreError, StoreResult};

/// Repository for Context CRUD operations
#[async_trait]
pub trait ContextRepository: Send + Sync {
    async fn create(&self, context: &Context) -> StoreResult<ContextId>;
    async fn get(&self, id: &ContextId) -> StoreResult<Context>;
    async fn update(&self, context: &Context) -> StoreResult<()>;
    async fn delete(&self, id: &ContextId) -> StoreResult<()>;
    async fn list_by_object(&self, object_id: &uuid::Uuid) -> StoreResult<Vec<Context>>;
    async fn list_by_org(&self, org_id: &str) -> StoreResult<Vec<Context>>;
    async fn search(&self, query: &ContextQuery) -> StoreResult<Vec<Context>>;
}

/// ArangoDB implementation of ContextRepository
pub struct ArangoContextRepository {
    pool: Arc<ConnectionPool>,
}

impl ArangoContextRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }

    /// Get database connection
    async fn db(
        &self,
    ) -> Result<
        arangors::Database<ReqwestClient>,
        mobc::Error<StoreError>,
    > {
        let conn = self.pool.connection().await?;
        Ok(conn.into_inner())
    }
}

#[async_trait]
impl ContextRepository for ArangoContextRepository {
    async fn create(&self, context: &Context) -> StoreResult<ContextId> {
        let db = self.db().await
            .map_err(|e| StoreError::InvalidQuery(e.to_string()))?;

        let collection = db.collection("context").await
            .map_err(|e| StoreError::Database(e.into()))?;

        let result: Context = collection.create_document(context).await
            .map_err(|e| StoreError::Database(e.into()))?;

        Ok(result.id)
    }

    async fn get(&self, id: &ContextId) -> StoreResult<Context> {
        let db = self.db().await
            .map_err(|e| StoreError::InvalidQuery(e.to_string()))?;

        let collection = db.collection("context").await
            .map_err(|e| StoreError::Database(e.into()))?;

        let result: Context = collection.document(id.to_string()).await
            .map_err(|e| StoreError::Database(e.into()))?;

        Ok(result)
    }

    async fn update(&self, context: &Context) -> StoreResult<()> {
        let db = self.db().await
            .map_err(|e| StoreError::InvalidQuery(e.to_string()))?;

        let collection = db.collection("context").await
            .map_err(|e| StoreError::Database(e.into()))?;

        collection.update_document(&context.id.to_string(), context).await
            .map_err(|e| StoreError::Database(e.into()))?;

        Ok(())
    }

    async fn delete(&self, id: &ContextId) -> StoreResult<()> {
        let db = self.db().await
            .map_err(|e| StoreError::InvalidQuery(e.to_string()))?;

        let collection = db.collection("context").await
            .map_err(|e| StoreError::Database(e.into()))?;

        collection.delete_document(&id.to_string()).await
            .map_err(|e| StoreError::Database(e.into()))?;

        Ok(())
    }

    async fn list_by_object(&self, object_id: &uuid::Uuid) -> StoreResult<Vec<Context>> {
        let db = self.db().await
            .map_err(|e| StoreError::InvalidQuery(e.to_string()))?;

        let aql = r#"
            FOR ctx IN context
            FILTER ctx.informatieobject_id == @object_id
                AND @now >= ctx.geldigheid.begindatum
                AND (@now < ctx.geldigheid.einddatum OR ctx.geldigheid.einddatum == null)
            SORT ctx.metadata.aangemaakt_op DESC
            RETURN ctx
        "#;

        let mut cursor = db.query_batch(aql)
            .bind("object_id", object_id.to_string())
            .bind("now", chrono::Utc::now())
            .await
            .map_err(|e| StoreError::Database(e.into()))?;

        cursor.next().await
            .map_err(|e| StoreError::Database(e.into()))?
            .ok_or(StoreError::InvalidQuery("No results".to_string()))
    }

    async fn list_by_org(&self, org_id: &str) -> StoreResult<Vec<Context>> {
        let db = self.db().await
            .map_err(|e| StoreError::InvalidQuery(e.to_string()))?;

        let aql = r#"
            FOR ctx IN context
            FILTER ctx.organisatie_id == @org_id
            SORT ctx.metadata.aangemaakt_op DESC
            LIMIT 100
            RETURN ctx
        "#;

        let mut cursor = db.query_batch(aql)
            .bind("org_id", org_id)
            .await
            .map_err(|e| StoreError::Database(e.into()))?;

        cursor.next().await
            .map_err(|e| StoreError::Database(e.into()))?
            .ok_or(StoreError::InvalidQuery("No results".to_string()))
    }

    async fn search(&self, query: &ContextQuery) -> StoreResult<Vec<Context>> {
        let mut aql = String::from("FOR ctx IN context FILTER 1==1");
        let mut binds = arangors::AqlBindings::default();

        if let Some(object_id) = &query.object_id {
            aql.push_str(&format!(" AND ctx.informatieobject_id == @object_id"));
            binds = binds.bind("object_id", object_id.to_string());
        }

        if let Some(org_id) = &query.organisation_id {
            aql.push_str(&format!(" AND ctx.organisatie_id == @org_id"));
            binds = binds.bind("org_id", org_id.clone());
        }

        if let Some(domein) = &query.domein {
            aql.push_str(&format!(" AND ctx.domain.primair_domein == @domein"));
            binds = binds.bind("domein", domein.to_string());
        }

        aql.push_str(" SORT ctx.metadata.aangemaakt_op DESC");

        if let Some(limit) = query.limit {
            aql.push_str(&format!(" LIMIT {}", limit));
        }

        aql.push_str(" RETURN ctx");

        let db = self.db().await
            .map_err(|e| StoreError::InvalidQuery(e.to_string()))?;

        let mut cursor = db.query_batch(aql)
            .with_bindings(binds)
            .await
            .map_err(|e| StoreError::Database(e.into()))?;

        cursor.next().await
            .map_err(|e| StoreError::Database(e.into()))?
            .ok_or(StoreError::InvalidQuery("No results".to_string()))
    }
}

/// Query parameters for context search
#[derive(Debug, Clone, Default)]
pub struct ContextQuery {
    pub object_id: Option<uuid::Uuid>,
    pub organisation_id: Option<String>,
    pub domein: Option<context_core::Domein>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Repository for ContextRecord (versioned context)
#[async_trait]
pub trait ContextRecordRepository: Send + Sync {
    async fn create(&self, record: &ContextRecord) -> StoreResult<ContextRecordId>;
    async fn get_history(&self, context_id: &ContextId) -> StoreResult<Vec<ContextRecord>>;
    async fn get_version(&self, context_id: &ContextId, version: u32) -> StoreResult<ContextRecord>;
}

pub struct ArangoContextRecordRepository {
    pool: Arc<ConnectionPool>,
}

impl ArangoContextRecordRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }

    async fn db(
        &self,
    ) -> Result<
        arangors::Database<ReqwestClient>,
        mobc::Error<StoreError>,
    > {
        let conn = self.pool.connection().await?;
        Ok(conn.into_inner())
    }
}

#[async_trait]
impl ContextRecordRepository for ArangoContextRecordRepository {
    async fn create(&self, record: &ContextRecord) -> StoreResult<ContextRecordId> {
        let db = self.db().await
            .map_err(|e| StoreError::InvalidQuery(e.to_string()))?;

        let collection = db.collection("context_record").await
            .map_err(|e| StoreError::Database(e.into()))?;

        let result: ContextRecord = collection.create_document(record).await
            .map_err(|e| StoreError::Database(e.into()))?;

        Ok(result.id)
    }

    async fn get_history(&self, context_id: &ContextId) -> StoreResult<Vec<ContextRecord>> {
        let db = self.db().await
            .map_err(|e| StoreError::InvalidQuery(e.to_string()))?;

        let aql = r#"
            FOR rec IN context_record
            FILTER rec.context_id == @context_id
            SORT rec.versie DESC
            RETURN rec
        "#;

        let mut cursor = db.query_batch(aql)
            .bind("context_id", context_id.to_string())
            .await
            .map_err(|e| StoreError::Database(e.into()))?;

        cursor.next().await
            .map_err(|e| StoreError::Database(e.into()))?
            .ok_or(StoreError::InvalidQuery("No results".to_string()))
    }

    async fn get_version(&self, context_id: &ContextId, version: u32) -> StoreResult<ContextRecord> {
        let db = self.db().await
            .map_err(|e| StoreError::InvalidQuery(e.to_string()))?;

        let aql = r#"
            FOR rec IN context_record
            FILTER rec.context_id == @context_id
                AND rec.versie == @version
            RETURN rec
        "#;

        let mut cursor = db.query_batch(aql)
            .bind("context_id", context_id.to_string())
            .bind("version", version)
            .await
            .map_err(|e| StoreError::Database(e.into()))?;

        cursor.next().await
            .map_err(|e| StoreError::Database(e.into()))?
            .ok_or(StoreError::InvalidQuery("No results".to_string()))
    }
}
