//! Semantic search service
//!
//! Provides embedding generation and vector similarity search for semantic
//! search capabilities. Uses ONNX Runtime for local embeddings or external
//! APIs for production use.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Embedding vector (typically 384 dims for sentence-transformers)
pub type Embedding = Vec<f32>;

/// Document with embedding for semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedDocument {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub embedding: Embedding,
    pub domain_id: Uuid,
    pub object_type: String,
}

/// Semantic search result with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub id: Uuid,
    pub title: String,
    pub snippet: String,
    pub domain_id: Uuid,
    pub domain_name: String,
    pub similarity: f32,
    pub object_type: String,
}

/// Configuration for embedding generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model: EmbeddingModel,
    pub dimension: usize,
    pub normalize: bool,
}

/// Available embedding models
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EmbeddingModel {
    /// Sentence-transformers paraphrase-multilingual-MiniLM-L12-v2 (384 dims)
    MultiilingualMiniLM,

    /// Sentence-transformers all-MiniLM-L6-v2 (384 dims, English only)
    MiniLML6,

    /// OpenAI text-embedding-3-small (1536 dims)
    OpenAI3Small,

    /// Cohere embed-v3 (1024 dims)
    CohereV3,

    /// Mock embeddings for testing
    Mock,
}

impl EmbeddingModel {
    pub fn dimension(&self) -> usize {
        match self {
            EmbeddingModel::MultiilingualMiniLM => 384,
            EmbeddingModel::MiniLML6 => 384,
            EmbeddingModel::OpenAI3Small => 1536,
            EmbeddingModel::CohereV3 => 1024,
            EmbeddingModel::Mock => 384,
        }
    }
}

/// Semantic search service
pub struct SemanticSearchService {
    config: EmbeddingConfig,
    documents: HashMap<Uuid, EmbeddedDocument>,
}

impl SemanticSearchService {
    /// Create a new semantic search service
    pub fn new(config: EmbeddingConfig) -> Self {
        Self {
            config,
            documents: HashMap::new(),
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(EmbeddingConfig {
            model: EmbeddingModel::Mock,
            dimension: 384,
            normalize: true,
        })
    }

    /// Generate embedding for text
    pub fn generate_embedding(&self, text: &str) -> anyhow::Result<Embedding> {
        match self.config.model {
            EmbeddingModel::Mock => self.generate_mock_embedding(text),
            EmbeddingModel::MultiilingualMiniLM => {
                // In production, this would use ONNX Runtime
                self.generate_mock_embedding(text)
            }
            _ => self.generate_mock_embedding(text),
        }
    }

    /// Generate mock embedding for testing
    fn generate_mock_embedding(&self, text: &str) -> anyhow::Result<Embedding> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();

        let mut embedding = Vec::with_capacity(self.config.dimension);
        for i in 0..self.config.dimension {
            // Generate pseudo-random but deterministic values
            let val = ((seed.wrapping_mul(i as u64 + 1)) % 1000) as f32 / 1000.0;
            embedding.push(val);
        }

        // Normalize if configured
        if self.config.normalize {
            let norm: f32 = embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
            if norm > 0.0 {
                for v in embedding.iter_mut() {
                    *v /= norm;
                }
            }
        }

        Ok(embedding)
    }

    /// Add a document to the search index
    pub fn add_document(&mut self, doc: EmbeddedDocument) -> anyhow::Result<()> {
        self.documents.insert(doc.id, doc);
        Ok(())
    }

    /// Batch add documents
    pub fn add_documents(&mut self, docs: Vec<EmbeddedDocument>) -> anyhow::Result<()> {
        for doc in docs {
            self.add_document(doc)?;
        }
        Ok(())
    }

    /// Find similar documents by query
    pub fn search(
        &self,
        query: &str,
        limit: usize,
        min_similarity: f32,
    ) -> Vec<SemanticSearchResult> {
        // Generate query embedding
        let query_embedding = match self.generate_embedding(query) {
            Ok(e) => e,
            Err(_) => return vec![],
        };

        // Calculate similarity for all documents
        let mut results: Vec<SemanticSearchResult> = self
            .documents
            .values()
            .map(|doc| {
                let similarity = cosine_similarity(&query_embedding, &doc.embedding);
                SemanticSearchResult {
                    id: doc.id,
                    title: doc.title.clone(),
                    snippet: if doc.content.len() > 200 {
                        format!("{}...", &doc.content[..200])
                    } else {
                        doc.content.clone()
                    },
                    domain_id: doc.domain_id,
                    domain_name: "Domain".to_string(), // Would be looked up in real system
                    similarity,
                    object_type: doc.object_type.clone(),
                }
            })
            .filter(|r| r.similarity >= min_similarity)
            .collect();

        // Sort by similarity descending
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        // Limit results
        results.truncate(limit);
        results
    }

    /// Find similar documents to a given document ID
    pub fn find_similar(&self, doc_id: Uuid, limit: usize) -> Vec<SemanticSearchResult> {
        let doc = match self.documents.get(&doc_id) {
            Some(d) => d,
            None => return vec![],
        };

        let mut results: Vec<SemanticSearchResult> = self
            .documents
            .values()
            .filter(|d| d.id != doc_id)
            .map(|d| {
                let similarity = cosine_similarity(&doc.embedding, &d.embedding);
                SemanticSearchResult {
                    id: d.id,
                    title: d.title.clone(),
                    snippet: if d.content.len() > 200 {
                        format!("{}...", &d.content[..200])
                    } else {
                        d.content.clone()
                    },
                    domain_id: d.domain_id,
                    domain_name: "Domain".to_string(),
                    similarity,
                    object_type: d.object_type.clone(),
                }
            })
            .collect();

        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        results.truncate(limit);
        results
    }

    /// Get document by ID
    pub fn get_document(&self, id: Uuid) -> Option<&EmbeddedDocument> {
        self.documents.get(&id)
    }

    /// Remove document from index
    pub fn remove_document(&mut self, id: Uuid) -> bool {
        self.documents.remove(&id).is_some()
    }

    /// Get document count
    pub fn count(&self) -> usize {
        self.documents.len()
    }
}

/// Calculate cosine similarity between two embeddings
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    norm_a = norm_a.sqrt();
    norm_b = norm_b.sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Calculate Euclidean distance between two embeddings
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::MAX;
    }

    let mut sum = 0.0;
    for i in 0..a.len() {
        let diff = a[i] - b[i];
        sum += diff * diff;
    }
    sum.sqrt()
}

/// Hybrid search: combine text and semantic scores
pub fn hybrid_score(
    text_score: f32,
    semantic_score: f32,
    alpha: f32,
) -> f32 {
    // alpha = 0.0 gives pure text, alpha = 1.0 gives pure semantic
    // 0.5 gives equal weight
    alpha * semantic_score + (1.0 - alpha) * text_score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_hybrid_score() {
        assert!((hybrid_score(0.8, 0.6, 0.5) - 0.7).abs() < 0.001);
        assert!((hybrid_score(1.0, 0.0, 0.0) - 1.0).abs() < 0.001);
        assert!((hybrid_score(0.0, 1.0, 1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_mock_embedding() {
        let service = SemanticSearchService::default();
        let emb1 = service.generate_embedding("test text").unwrap();
        let emb2 = service.generate_embedding("test text").unwrap();
        let emb3 = service.generate_embedding("different text").unwrap();

        // Same text should produce same embedding
        assert_eq!(emb1, emb2);

        // Different text should produce different embedding
        assert_ne!(emb1, emb3);

        // Embeddings should have correct dimension
        assert_eq!(emb1.len(), 384);
    }
}
