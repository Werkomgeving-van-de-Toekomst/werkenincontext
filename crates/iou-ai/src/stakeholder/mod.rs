//! Stakeholder extraction feasibility spike
//!
//! This module validates external dependencies before main implementation:
//! - Rijksoverheid API availability and capabilities
//! - Local fallback dictionary for Dutch government organizations
//! - Cost estimation model for LLM-based entity extraction
//! - GeneratedDocument structure verification

mod feasibility_tests;
mod rijksoverheid_api_probe;
mod fallback_dict;
mod cost_model;
mod document_probe;

pub use rijksoverheid_api_probe::{probe_rijksoverheid_api, ApiProbeResult};
pub use fallback_dict::get_fallback_canonical_name;
pub use cost_model::CostEstimator;
pub use document_probe::{verify_document_structure, extract_document_text};
