//! AI Service Module
//!
//! Local AI integration using Ollama for document enrichment
//! and metadata generation in Dutch government context.

pub mod ollama;

pub use ollama::{
    AIDocumentMetadata, BewaartermijnSuggestie, ChatMessage, ChatRequest, ChatResponse,
    DocumentInhoud, DocumentSamenvatting, Entiteit, GenerationOptions, NederlandseAIDienst,
    OllamaClient, OllamaError, OllamaHealth, OllamaModel, PullProgress, WooClassificatie,
};
