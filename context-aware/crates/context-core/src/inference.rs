// =============================================================================
// Context Inference - AI-powered context enrichment
// =============================================================================

use crate::{Context, Confidence, SemanticContext, Entiteit, EntiteitType};

/// Context inference service trait
pub trait ContextInference: Send + Sync {
    /// Infer semantic context from document content
    async fn infer_semantic(&self, content: &str) -> Result<SemanticContext, InferenceError>;

    /// Extract entities from text
    async fn extract_entities(&self, text: &str) -> Result<Vec<Entiteit>, InferenceError>;

    /// Suggest domain classification
    async fn suggest_domain(&self, context: &Context) -> Result<DomainSuggestion, InferenceError>;

    /// Enrich existing context with inferred data
    async fn enrich(&self, context: &mut Context) -> Result<Confidence, InferenceError>;
}

/// Inference errors
#[derive(Debug, thiserror::Error)]
pub enum InferenceError {
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),

    #[error("Input validation failed: {0}")]
    InvalidInput(String),

    #[error("Inference failed: {0}")]
    InferenceFailed(String),

    #[error("Rate limited")]
    RateLimited,
}

/// Domain suggestion with confidence
#[derive(Debug)]
pub struct DomainSuggestion {
    pub domain: String,
    pub confidence: Confidence,
    pub reasoning: String,
}

/// Entity extraction result
#[derive(Debug)]
pub struct EntityExtraction {
    pub entities: Vec<Entiteit>,
    pub confidence: Confidence,
}

/// Predefined inference models
#[derive(Debug, Clone, Copy)]
pub enum InferenceModel {
    /// Fast, lightweight model for basic inference
    Light,

    /// Standard model for general use
    Standard,

    /// Heavy model for maximum accuracy
    Accurate,
}

impl InferenceModel {
    pub fn name(&self) -> &str {
        match self {
            Self::Light => "context-inference-light",
            Self::Standard => "context-inference-standard",
            Self::Accurate => "context-inference-accurate",
        }
    }
}
