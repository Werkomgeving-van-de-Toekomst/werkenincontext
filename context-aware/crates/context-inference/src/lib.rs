// =============================================================================
// Context-Inference: AI-powered context extraction service
// =============================================================================
//
// Provides intelligent inference of contextual metadata from document content
// using Nebul API with local NLP fallbacks.
//
// Key features:
// - Nebul API integration for high-quality inference
// - Named Entity Recognition (persons, organizations, locations)
// - Legal basis extraction (BWBR references)
// - Confidence scoring and human review workflow
// - Compliance with AVG/GDPR (privacy by design)
// =============================================================================

pub mod nebul;
pub mod ner;
pub mod legal;
pub mod review;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub use nebul::*;
pub use ner::*;
pub use legal::*;
pub use review::*;

/// Inference configuration
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    /// Nebul API key
    pub nebul_api_key: String,

    /// Nebul API endpoint
    pub nebul_endpoint: String,

    /// Minimum confidence for auto-acceptance
    pub min_confidence: f64,

    /// Whether to require human review for legal context
    pub review_legal_context: bool,

    /// Maximum text length for inference (characters)
    pub max_text_length: usize,

    /// Model to use
    pub model: NebulModel,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            nebul_api_key: std::env::var("NEBUL_API_KEY")
                .unwrap_or_default(),
            nebul_endpoint: std::env::var("NEBUL_ENDPOINT")
                .unwrap_or_else(|_| "https://api.nebul.ai/v1".to_string()),
            min_confidence: 0.8,
            review_legal_context: true,
            max_text_length: 100_000,
            model: NebulModel::default(),
        }
    }
}

/// Available Nebul models
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NebulModel {
    NebulGPT4,
    NebulGPT35Turbo,
    NebulClaudeOpus,
    NebulClaudeSonnet,
    NebulLlama3_70b,
}

impl Default for NebulModel {
    fn default() -> Self {
        Self::NebulClaudeSonnet
    }
}

impl NebulModel {
    pub fn as_str(&self) -> &str {
        match self {
            NebulModel::NebulGPT4 => "gpt-4-turbo",
            NebulModel::NebulGPT35Turbo => "gpt-3.5-turbo",
            NebulModel::NebulClaudeOpus => "claude-3-opus-20240229",
            NebulModel::NebulClaudeSonnet => "claude-3-5-sonnet-20241022",
            NebulModel::NebulLlama3_70b => "llama-3-70b",
        }
    }

    pub fn provider(&self) -> NebulProvider {
        match self {
            NebulModel::NebulGPT4 | NebulModel::NebulGPT35Turbo => NebulProvider::OpenAI,
            NebulModel::NebulClaudeOpus | NebulModel::NebulClaudeSonnet => NebulProvider::Anthropic,
            NebulModel::NebulLlama3_70b => NebulProvider::Meta,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NebulProvider {
    OpenAI,
    Anthropic,
    Meta,
}

/// Inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Source text to analyze
    pub text: String,

    /// Information object type
    pub object_type: String,

    /// Organization ID
    pub organisatie_id: String,

    /// Request options
    pub options: InferenceOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceOptions {
    /// Request specific context types
    pub context_types: Vec<ContextTypeRequest>,

    /// Whether to include NER entities
    pub include_entities: bool,

    /// Whether to extract legal references
    pub include_legal: bool,

    /// Confidence threshold
    pub min_confidence: Option<f64>,
}

impl Default for InferenceOptions {
    fn default() -> Self {
        Self {
            context_types: vec![
                ContextTypeRequest::Core,
                ContextTypeRequest::Domain,
                ContextTypeRequest::Semantic,
            ],
            include_entities: true,
            include_legal: true,
            min_confidence: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextTypeRequest {
    Core,
    Domain,
    Semantic,
    Provenance,
}

/// Inference result (successful completion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInferenceResult {
    /// Inferred context
    pub context: InferredContext,

    /// Confidence scores
    pub confidence: ConfidenceScores,

    /// Metadata about the inference
    pub metadata: InferenceMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredContext {
    /// Core context (creator, title, etc.)
    pub core: Option<CoreContext>,

    /// Domain context (zaak, project, etc.)
    pub domain: Option<DomainInference>,

    /// Semantic context (keywords, subjects)
    pub semantic: Option<SemanticContext>,

    /// Legal context (grondslagen)
    pub legal: Option<LegalContext>,

    /// Named entities found
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreContext {
    pub title: Option<String>,
    pub creator: Option<String>,
    pub classification: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DomainInference {
    #[serde(rename = "zaak")]
    Zaak {
        zaak_id: Option<String>,
        zaak_type: Option<String>,
        zaak_fase: Option<String>,
    },
    #[serde(rename = "project")]
    Project {
        project_id: Option<String>,
        project_naam: Option<String>,
    },
    #[serde(rename = "beleid")]
    Beleid {
        beleid_id: Option<String>,
        beleidsgebied: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticContext {
    pub trefwoorden: Vec<String>,
    pub onderwerpen: Vec<String>,
    pub samenvatting: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalContext {
    pub grondslagen: Vec<InferredGrondslag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredGrondslag {
    pub grondslag_id: String,
    pub bron: String,
    pub artikel: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: EntityType,
    pub text: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub confidence: f64,
    pub metadata: Option<EntityMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Persoon,
    Organisatie,
    Locatie,
    Wet,
    Evenement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMetadata {
    pub identificatie: Option<String>,
    pub extra_info: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScores {
    pub overall: f64,
    pub by_field: Vec<FieldConfidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConfidence {
    pub field: String,
    pub confidence: f64,
    pub requires_review: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceMetadata {
    pub model: String,
    pub provider: String,
    pub inferred_at: DateTime<Utc>,
    pub processing_time_ms: u64,
    pub text_length: usize,
    pub requires_review: bool,
    pub review_reason: Option<String>,
}

/// Inference error
#[derive(Debug, thiserror::Error)]
pub enum InferenceError {
    #[error("API error: {0}")]
    Api(String),

    #[error("Rate limited: retry after {0}s")]
    RateLimited(u64),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Text too long: {0} characters (max {1})")]
    TextTooLong(usize, usize),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Result type for inference operations
pub type InferenceResult<T> = std::result::Result<T, InferenceError>;
