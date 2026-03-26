//! GraphRAG module for knowledge graph persistence and analysis
//!
//! This module provides:
//! - Type definitions for entities, relationships, and communities
//! - ArangoDB-based persistence layer (server-only)

// Types are always available (pure data structures)
pub mod types;

// Server-only modules (require arangors/reqwest)
#[cfg(feature = "server")]
pub mod connection;

#[cfg(feature = "server")]
pub mod error;

#[cfg(feature = "server")]
pub mod migration;

#[cfg(feature = "server")]
pub mod queries;

#[cfg(feature = "server")]
pub mod store;

// Re-export common types (always available)
pub use types::{
    Entity,
    EntityType,
    Relationship,
    RelationshipType,
    Community,
    DomainRelation,
    DomainRelationType,
    DiscoveryMethod,
    ContextVector,
    GraphAnalysisResult,
};

// Re-export server-only types
#[cfg(feature = "server")]
pub use connection::ArangoConfig;

#[cfg(feature = "server")]
pub use error::StoreError;

#[cfg(feature = "server")]
pub use migration::{MigrationValidator, SampleComparison, ValidationResult};

#[cfg(feature = "server")]
pub use store::{
    EntityFilters,
    EntityUpdate,
    GraphPath,
    GraphStore,
    Neighbor,
    NeighborFilters,
    PaginatedEntities,
    PaginationOptions,
    RelationshipDirection,
    RelationshipQueryOptions,
    TraversalDirection,
    TraversalRequest,
    TraversalResult,
};
