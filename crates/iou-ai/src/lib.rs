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
pub mod compliance;
pub mod suggestions;
pub mod semantic;

pub use ner::DutchNerExtractor;
pub use graphrag::KnowledgeGraph;
pub use compliance::ComplianceAssessor;
pub use suggestions::MetadataSuggester;
pub use semantic::{SemanticSearchService, cosine_similarity};
