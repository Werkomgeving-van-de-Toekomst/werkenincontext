// =============================================================================
// Context-Store: ArangoDB repository layer for context data
// =============================================================================
//
// Provides persistent storage for context entities using ArangoDB.
// Integrates with Metadata Registry (Project 002) for shared entity types.
//
// Key features:
// - Context CRUD operations
// - Graph traversal for context relationships
// - Temporal queries (geldigheid)
// - Integration with GGHH V2 entities
// =============================================================================

pub mod connection;
pub mod repositories;
pub mod migrations;

pub use connection::*;
pub use repositories::*;

use context_core::{Context, ContextId, ContextRecord, ContextRecordId};
use thiserror::Error;

/// Store error type
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Context not found: {0}")]
    ContextNotFound(ContextId),

    #[error("Database error: {0}")]
    Database(#[from] arangors::error::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Connection pool exhausted")]
    PoolExhausted,

    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

pub type StoreResult<T> = Result<T, StoreError>;

/// Store configuration
#[derive(Debug, Clone)]
pub struct StoreConfig {
    /// ArangoDB connection URL
    pub url: String,

    /// Database name
    pub database: String,

    /// Username
    pub username: String,

    /// Password
    pub password: String,

    /// Connection pool size
    pub pool_size: u32,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8529".to_string(),
            database: "iou_modern".to_string(),
            username: "root".to_string(),
            password: "".to_string(),
            pool_size: 10,
        }
    }
}
