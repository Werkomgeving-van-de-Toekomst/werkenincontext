// =============================================================================
// Context Store - Storage abstraction for context data
// =============================================================================

use async_trait::async_trait;
use crate::{Context, ContextId, ContextRecord, InformatieobjectId};

/// Context store trait - abstract storage interface
#[async_trait]
pub trait ContextStore: Send + Sync {
    /// Get context by ID
    async fn get(&self, id: &ContextId) -> Result<Option<Context>, StoreError>;

    /// Get context for an information object
    async fn get_by_informatieobject(&self, info_id: &InformatieobjectId) -> Result<Option<Context>, StoreError>;

    /// Create new context
    async fn create(&self, context: &Context) -> Result<ContextId, StoreError>;

    /// Update existing context
    async fn update(&self, context: &Context) -> Result<(), StoreError>;

    /// Delete context
    async fn delete(&self, id: &ContextId) -> Result<(), StoreError>;

    /// Query contexts by domain
    async fn query_by_domain(&self, domain: &str) -> Result<Vec<Context>, StoreError>;

    /// Search contexts by keywords
    async fn search(&self, query: &str) -> Result<Vec<Context>, StoreError>;
}

/// Store errors
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Context not found: {0}")]
    NotFound(ContextId),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// Informatieobject ID wrapper
pub type InformatieobjectId = uuid::Uuid;
