// =============================================================================
// Context-Core: Shared types and abstractions for context-aware data
// =============================================================================
//
// Implements the core context model aligned with Metamodel GGHH Overheid V2
// "Context" entity and BSW architecture principles.
//
// Key concepts:
// - Context: Who, what, when, where, why of information creation/usage
// - Context layers: Domain, Semantic, Provenance, Temporal
// - Context quality: Accuracy, completeness, consistency scores
// =============================================================================

pub mod entities;
pub mod layers;
pub mod quality;
pub mod store;
pub mod inference;
pub mod bridge;

pub use entities::*;
pub use layers::*;
pub use quality::*;
pub use store::*;
pub use inference::*;
pub use bridge::*;

// =============================================================================
// Core Type Definitions
// =============================================================================

use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Generic ID wrapper for strongly-typed identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id<T>(pub Uuid, std::marker::PhantomData<T>);

impl<T> serde::Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> serde::Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let uuid = Uuid::deserialize(deserializer)?;
        Ok(Id(uuid, std::marker::PhantomData))
    }
}

impl<T> Id<T> {
    pub fn new() -> Self {
        Self(Uuid::new_v4(), std::marker::PhantomData)
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid, std::marker::PhantomData)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Context ID - uniquely identifies a context record
pub type ContextId = Id<Context>;

/// Context Record ID - uniquely identifies a specific context version
pub type ContextRecordId = Id<ContextRecord>;

/// Organisation ID - identifies an organization
pub type OrganisationId = Uuid;

/// Timestamp type for context data
pub type Timestamp = DateTime<Utc>;

/// Confidence score for inference results (0.0 - 1.0)
pub type Confidence = f64;

/// Quality score for context data (0.0 - 1.0)
pub type QualityScore = f64;
