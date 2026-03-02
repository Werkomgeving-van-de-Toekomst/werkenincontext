//! IOU-AI: AI/ML services voor metadata extractie en kennisgraaf
//!
//! Dit crate biedt:
//! - Named Entity Recognition (NER) voor Nederlandse overheidstekst
//! - GraphRAG voor automatische relatiedetectie
//! - Compliance assessment (Woo, AVG, Archiefwet)
//! - Metadata suggesties
//!
//! # Architectuur
//!
//! We gebruiken een pragmatische aanpak:
//! - Regex-based NER voor bekende patronen (organisaties, wetten, locaties)
//! - petgraph voor graph algorithms (community detection, path finding)
//! - Eenvoudige rule-based classificatie
//!
//! Voor productie kan dit uitgebreid worden met:
//! - ONNX Runtime voor echte ML modellen
//! - Externe embedding API (OpenAI, Cohere)
//! - Fine-tuned mBERT voor Nederlands

pub mod ner;
pub mod graphrag;
pub mod document_entity;
pub mod compliance;
pub mod suggestions;
pub mod semantic;

pub mod templates;
pub mod conversion;

pub mod agents;
pub mod llm;

pub use ner::DutchNerExtractor;
pub use graphrag::KnowledgeGraph;
pub use document_entity::{DocumentEntity, DocumentSection, DocumentEntityMetadata, DocumentSchema};
pub use compliance::ComplianceAssessor;
pub use suggestions::MetadataSuggester;
pub use semantic::{SemanticSearchService, cosine_similarity};
pub use templates::TemplateEngine;
pub use conversion::{markdown_to_odf, markdown_to_pdf, OutputFormat};
pub use agents::{
    AgentError,
    ResearchContext, ResearchAgentConfig, execute_research_agent,
    GeneratedDocument, ContentAgentConfig, execute_content_agent,
    ComplianceResult, ComplianceConfig, execute_compliance_agent,
    ReviewDecision, ReviewAction, ReviewConfig, execute_review_agent,
    QualityIssue, QualityIssueCategory,
    PipelineError, ErrorSeverity, AgentPipeline, PipelineConfig,
    AgentExecutionResult, PipelineCheckpoint, PipelineResult,
};
pub use llm::{LlmConfig, LlmProvider, LlmBackend, LlmError, ChatMessage, create_provider};
