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
// Re-exports from iou-core
// =============================================================================
pub use iou_core::{Id, Timestamp, OrganisationId};

/// Context ID - uniquely identifies a context record
pub type ContextId = Id<Context>;

/// Context Record ID - uniquely identifies a specific context version
pub type ContextRecordId = Id<ContextRecord>;

/// Confidence score for inference results (0.0 - 1.0)
pub type Confidence = f64;

/// Quality score for context data (0.0 - 1.0)
pub type QualityScore = f64;
