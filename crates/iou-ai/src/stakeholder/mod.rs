//! Stakeholder extraction for Dutch government documents
//!
//! This module provides entity and relationship extraction from Wet open overheid
//! (Woo) documents, with specific support for:
//! - Dutch name patterns (tussenvoegsels like "van", "van der", "de")
//! - Government organization normalization
//! - PII classification for GDPR compliance

// Feasibility spike modules (section-00)
mod feasibility_tests;
mod rijksoverheid_api_probe;
mod fallback_dict;
mod cost_model;
mod document_probe;

// Foundation types (section-01)
pub mod types;
pub mod extractor;
pub mod result;
pub mod mention;
pub mod normalization;
pub mod error;

// Baseline extraction (section-02)
pub mod baseline;

// Rijksoverheid API client (section-03)
pub mod rijksoverheid;

// LLM extractor (section-04)
pub mod llm_extractor;

// Normalization & Deduplication (section-05)
pub mod normalizer;
pub mod deduplicator;

// Main Extractor (section-06)
pub mod main_extractor;

// Feasibility spike exports
pub use rijksoverheid_api_probe::{probe_rijksoverheid_api, ApiProbeResult};
pub use fallback_dict::get_fallback_canonical_name;
pub use cost_model::CostEstimator;
pub use document_probe::{verify_document_structure, extract_document_text};

// Public API exports (section-01)
pub use types::{PersonStakeholder, OrganizationStakeholder, OrgType};
pub use extractor::{StakeholderExtractor, ExtractionOptions};
pub use result::{ExtractionResult, ExtractionStats, VerificationStatus};
pub use mention::{MentionRelationship, MentionType, TextPosition, ExtractionMethod};
pub use normalization::{DutchNameNormalizer, NameComparison};
pub use error::{ExtractionError, NormalizationError, DeduplicationError};

// Public API exports (section-02)
pub use baseline::{BaselineExtractor, BaselineError, RelationshipMatch};

// Public API exports (section-03)
pub use rijksoverheid::{RijksoverheidClient, OrgInfo};

// Public API exports (section-04)
pub use llm_extractor::{
    ClaudeExtractor, ClaudeExtractorConfig, CostTracker,
    ExtractionContext, TextSegment, FocusReason,
    LlmExtractionError, should_extract_with_llm,
};

// Public API exports (section-05)
pub use normalizer::{EntityNormalizer, CacheStats};
pub use deduplicator::{EntityDeduplicator, jaro_winkler};

// Public API exports (section-06)
pub use main_extractor::MainExtractor;
