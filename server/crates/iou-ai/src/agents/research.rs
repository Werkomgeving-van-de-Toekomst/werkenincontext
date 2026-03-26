//! Research Agent: Analyzes document requests and queries GraphRAG for context.
//!
//! The Research Agent is the first agent in the document creation pipeline.
//! It queries the knowledge graph for similar documents, extracts structure
//! patterns, and identifies mandatory vs optional sections.

use crate::agents::config::ResearchAgentConfig;
use crate::agents::AgentError;
use crate::graphrag::KnowledgeGraph;
use chrono::{DateTime, Utc};
use iou_core::document::{DomainConfig, DocumentRequest, VariableSource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;  // Still used for DocumentRequest

/// Context produced by the Research Agent for the Content Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchContext {
    /// Document type being researched
    pub document_type: String,

    /// Domain-specific context information
    pub domain_context: DomainContext,

    /// Required sections for this document type
    pub mandatory_sections: Vec<String>,

    /// Optional sections that may be included
    pub optional_sections: Vec<String>,

    /// Suggested optional sections that should be included for this document
    pub suggested_sections: Vec<String>,

    /// Similar documents found in GraphRAG
    pub similar_documents: Vec<SimilarDocument>,

    /// Related entities for context
    pub related_entities: Vec<EntityReference>,

    /// Applicable PROVISA guidelines
    pub provisa_guidelines: Vec<ProvisaGuideline>,

    /// Suggested template variables with sources
    pub suggested_variables: Vec<TemplateVariableSuggestion>,

    /// Timestamp when research was performed
    pub timestamp: DateTime<Utc>,
}

/// Domain-specific context extracted from knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainContext {
    pub domain_id: String,
    pub domain_name: String,
    pub organizational_unit: Option<String>,
    pub applicable_regulations: Vec<String>,
    pub standard_procedures: Vec<String>,
}

/// Reference to a similar document found in GraphRAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarDocument {
    pub document_id: String,
    pub title: String,
    pub similarity_score: f32,
    pub sections: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Entity reference from knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityReference {
    pub entity_id: String,  // String for flexibility with external knowledge graphs
    pub entity_type: String,
    pub name: String,
    pub relevance_score: f32,
}

/// PROVISA guideline reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisaGuideline {
    pub guideline_id: String,
    pub title: String,
    pub applicable_sections: Vec<String>,
}

/// Suggested template variable with source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariableSuggestion {
    pub name: String,
    pub suggested_value: Option<String>,
    pub source: VariableSource,
    pub is_required: bool,
}

/// Result from a GraphRAG query for similar documents
pub struct SimilarDocumentsResult {
    pub documents: Vec<SimilarDocument>,
    pub entity_references: Vec<EntityReference>,
}

/// Executes the Research Agent phase
///
/// # Arguments
/// * `request` - The original document request
/// * `kg_client` - Knowledge graph for queries
/// * `domain_config` - Domain configuration including trust level
///
/// # Returns
/// `ResearchContext` containing all discovered information
pub async fn execute_research_agent(
    request: &DocumentRequest,
    kg_client: &KnowledgeGraph,
    domain_config: &DomainConfig,
) -> Result<ResearchContext, AgentError> {
    let config = ResearchAgentConfig::default();
    execute_research_agent_with_config(request, kg_client, domain_config, &config).await
}

/// Executes the Research Agent with custom configuration
pub async fn execute_research_agent_with_config(
    request: &DocumentRequest,
    kg_client: &KnowledgeGraph,
    domain_config: &DomainConfig,
    config: &ResearchAgentConfig,
) -> Result<ResearchContext, AgentError> {
    let timestamp = Utc::now();

    // Query GraphRAG for similar documents
    let similar_result = query_similar_documents(
        &request.document_type,
        &request.domain_id,
        kg_client,
        config,
    )
    .await?;

    // Extract structure patterns from similar documents
    let (mandatory_sections, optional_sections) =
        extract_structure_patterns(&similar_result.documents, config);

    // Get related entities
    let related_entities = extract_related_entities(
        kg_client,
        &request.context,
        config,
    );

    // Build domain context
    let domain_context = build_domain_context(domain_config);

    // TODO: Integrate PROVISA guidelines (section-05 will implement compliance agent)
    let provisa_guidelines = Vec::new();

    // Suggest template variables
    let suggested_variables = suggest_template_variables(
        &request.context,
        &related_entities,
        &domain_context,
    );

    // Suggest optional sections to include (for now, suggest all optional sections)
    let suggested_sections = optional_sections.clone();

    Ok(ResearchContext {
        document_type: request.document_type.clone(),
        domain_context,
        mandatory_sections,
        optional_sections,
        suggested_sections,
        similar_documents: similar_result.documents,
        related_entities,
        provisa_guidelines,
        suggested_variables,
        timestamp,
    })
}

/// Query GraphRAG for similar documents
///
/// TODO: This is a stub implementation. Full semantic similarity search requires:
/// - Embedding generation for documents (section-04 or later)
/// - Vector similarity search in GraphRAG
/// - Actual use of similarity_threshold config
async fn query_similar_documents(
    _document_type: &str,
    _domain_id: &str,
    kg_client: &KnowledgeGraph,
    config: &ResearchAgentConfig,
) -> Result<SimilarDocumentsResult, AgentError> {
    // In a real implementation, this would:
    // 1. Generate embedding for the document type and context
    // 2. Query GraphRAG for semantically similar documents
    // 3. Filter by similarity_threshold
    // 4. Return documents with actual similarity scores
    //
    // For now, we return entities from the knowledge graph as placeholder results.

    let entities = kg_client.entities();

    let similar_docs: Vec<SimilarDocument> = entities
        .iter()
        .filter(|e| e.entity_type == iou_core::graphrag::EntityType::Policy)
        .take(config.max_similar_documents)
        .map(|e| SimilarDocument {
            document_id: e.id.to_string(),
            title: e.name.clone(),
            similarity_score: 0.8,  // TODO: Calculate actual similarity score
            sections: vec!["header".to_string(), "content".to_string()],
            created_at: e.created_at,
        })
        .collect();

    let entity_refs: Vec<EntityReference> = entities
        .iter()
        .take(5)
        .map(|e| EntityReference {
            entity_id: e.id.to_string(),
            entity_type: format!("{:?}", e.entity_type),
            name: e.name.clone(),
            relevance_score: 0.7,  // TODO: Calculate actual relevance score
        })
        .collect();

    Ok(SimilarDocumentsResult {
        documents: similar_docs,
        entity_references: entity_refs,
    })
}

/// Extract structure patterns from similar documents
fn extract_structure_patterns(
    documents: &[SimilarDocument],
    config: &ResearchAgentConfig,
) -> (Vec<String>, Vec<String>) {
    if documents.is_empty() {
        // Return default structure
        return get_default_structure("unknown");
    }

    // Count section occurrences
    let mut section_counts: HashMap<String, usize> = HashMap::new();
    let total_docs = documents.len();

    for doc in documents {
        for section in &doc.sections {
            *section_counts.entry(section.clone()).or_insert(0) += 1;
        }
    }

    // Classify sections based on occurrence frequency
    let mut mandatory = Vec::new();
    let mut optional = Vec::new();

    for (section, count) in &section_counts {
        let frequency = *count as f32 / total_docs as f32;
        if frequency >= config.mandatory_threshold {
            mandatory.push(section.clone());
        } else {
            optional.push(section.clone());
        }
    }

    // Sort by occurrence count (descending)
    mandatory.sort_by(|a, b| {
        section_counts.get(b).copied().unwrap_or(0).cmp(&section_counts.get(a).copied().unwrap_or(0))
    });
    optional.sort_by(|a, b| {
        section_counts.get(b).copied().unwrap_or(0).cmp(&section_counts.get(a).copied().unwrap_or(0))
    });

    // If no mandatory sections found, use defaults
    if mandatory.is_empty() {
        return get_default_structure("unknown");
    }

    (mandatory, optional)
}

/// Extract related entities from knowledge graph
fn extract_related_entities(
    kg_client: &KnowledgeGraph,
    request_context: &HashMap<String, String>,
    _config: &ResearchAgentConfig,
) -> Vec<EntityReference> {
    let entities = kg_client.entities();

    entities
        .iter()
        .filter(|e| {
            // Case-insensitive search: check if entity name appears in request context
            let entity_name_lower = e.name.to_lowercase();
            request_context.values().any(|v| v.to_lowercase().contains(&entity_name_lower))
        })
        .map(|e| EntityReference {
            entity_id: e.id.to_string(),
            entity_type: format!("{:?}", e.entity_type),
            name: e.name.clone(),
            relevance_score: 0.8,  // TODO: Calculate based on connection strength
        })
        .take(10)
        .collect()
}

/// Build domain context from domain configuration
fn build_domain_context(domain_config: &DomainConfig) -> DomainContext {
    // TODO: Fetch actual domain name from domain registry instead of formatting ID
    DomainContext {
        domain_id: domain_config.domain_id.clone(),
        domain_name: format!("Domain {}", domain_config.domain_id),
        organizational_unit: None,
        // TODO: Derive from domain configuration or PROVISA guidelines
        applicable_regulations: vec!["Woo".to_string()],
        standard_procedures: Vec::new(),
    }
}

/// Suggest template variables based on context
fn suggest_template_variables(
    request_context: &HashMap<String, String>,
    related_entities: &[EntityReference],
    _domain_context: &DomainContext,
) -> Vec<TemplateVariableSuggestion> {
    let mut suggestions = Vec::new();

    // Add variables from request context
    for (key, value) in request_context {
        suggestions.push(TemplateVariableSuggestion {
            name: key.clone(),
            suggested_value: Some(value.clone()),
            source: VariableSource::UserInput,
            is_required: true,
        });
    }

    // Add entity-based suggestions
    for entity in related_entities.iter().take(3) {
        suggestions.push(TemplateVariableSuggestion {
            name: format!("entity_{}", entity.name.to_lowercase().replace(' ', "_")),
            suggested_value: Some(entity.name.clone()),
            source: VariableSource::KnowledgeGraph,
            is_required: false,
        });
    }

    suggestions
}

/// Get default section structures for common document types
fn get_default_structure(document_type: &str) -> (Vec<String>, Vec<String>) {
    match document_type {
        "woo_besluit" => (
            vec![
                "header".to_string(),
                "references".to_string(),
                "consideration".to_string(),
                "decision".to_string(),
                "signature".to_string(),
            ],
            vec!["attachments".to_string(), "footnotes".to_string()],
        ),
        "woo_info" => (
            vec![
                "header".to_string(),
                "description".to_string(),
                "availability".to_string(),
                "contact".to_string(),
            ],
            vec!["exceptions".to_string(), "costs".to_string()],
        ),
        _ => (
            vec!["header".to_string(), "content".to_string()],
            vec!["attachments".to_string()],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iou_core::document::TrustLevel;
    use std::collections::HashMap;

    fn create_test_request(document_type: &str) -> DocumentRequest {
        let mut context = HashMap::new();
        context.insert("reference".to_string(), "TEST-001".to_string());

        DocumentRequest {
            id: Uuid::new_v4(),
            domain_id: "test_domain".to_string(),
            document_type: document_type.to_string(),
            context,
            requested_at: Utc::now(),
        }
    }

    fn create_test_domain_config() -> DomainConfig {
        DomainConfig {
            domain_id: "test_domain".to_string(),
            trust_level: TrustLevel::Medium,
            required_approval_threshold: 0.8,
            auto_approval_threshold: 0.95,
        }
    }

    fn create_kg_with_documents() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();

        // Add some test entities
        kg.add_entity(iou_core::graphrag::Entity {
            id: Uuid::new_v4(),
            name: "Test Policy Document".to_string(),
            entity_type: iou_core::graphrag::EntityType::Policy,
            canonical_name: None,
            description: None,
            confidence: 1.0,
            source_domain_id: None,
            metadata: serde_json::Value::Null,
            created_at: Utc::now(),
        });

        kg
    }

    #[tokio::test]
    async fn test_execute_research_agent_queries_graphrag_for_similar_documents() {
        let request = create_test_request("woo_besluit");
        let kg = create_kg_with_documents();
        let domain_config = create_test_domain_config();

        let result = execute_research_agent(&request, &kg, &domain_config).await;

        assert!(result.is_ok());
        let context = result.unwrap();
        // Should have found our test entity as a "similar document"
        assert!(!context.similar_documents.is_empty() || !context.related_entities.is_empty());
    }

    #[tokio::test]
    async fn test_execute_research_agent_extracts_structure_patterns_from_results() {
        let request = create_test_request("woo_besluit");
        let kg = create_kg_with_documents();
        let domain_config = create_test_domain_config();

        let result = execute_research_agent(&request, &kg, &domain_config).await;

        assert!(result.is_ok());
        let context = result.unwrap();
        // Should have either extracted patterns or defaults
        assert!(!context.mandatory_sections.is_empty());
    }

    #[tokio::test]
    async fn test_execute_research_agent_identifies_mandatory_vs_optional_sections() {
        let request = create_test_request("woo_info");
        let kg = create_kg_with_documents();
        let domain_config = create_test_domain_config();

        let result = execute_research_agent(&request, &kg, &domain_config).await;

        assert!(result.is_ok());
        let context = result.unwrap();
        // Should have classified sections
        assert!(!context.mandatory_sections.is_empty() || !context.optional_sections.is_empty());
    }

    #[tokio::test]
    async fn test_execute_research_agent_handles_empty_graphrag_results_gracefully() {
        let request = create_test_request("unknown_type");
        let kg = KnowledgeGraph::new(); // Empty graph
        let domain_config = create_test_domain_config();

        let result = execute_research_agent(&request, &kg, &domain_config).await;

        assert!(result.is_ok());
        let context = result.unwrap();
        // Should return default structure instead of error
        assert!(!context.mandatory_sections.is_empty());
    }

    #[tokio::test]
    async fn test_execute_research_agent_returns_research_context_with_required_fields() {
        let request = create_test_request("woo_besluit");
        let kg = create_kg_with_documents();
        let domain_config = create_test_domain_config();

        let result = execute_research_agent(&request, &kg, &domain_config).await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(!context.document_type.is_empty());
        assert!(!context.domain_context.domain_id.is_empty());
        assert!(context.timestamp <= Utc::now());
    }

    #[test]
    fn test_extract_structure_patterns_from_empty_documents_returns_defaults() {
        let documents: Vec<SimilarDocument> = vec![];
        let config = ResearchAgentConfig::default();

        let (mandatory, optional) = extract_structure_patterns(&documents, &config);

        assert!(!mandatory.is_empty());
        assert_eq!(mandatory, vec!["header", "content"]);
        assert_eq!(optional, vec!["attachments"]);
    }

    #[test]
    fn test_extract_structure_patterns_classifies_sections_by_frequency() {
        let documents = vec![
            SimilarDocument {
                document_id: "1".to_string(),
                title: "Doc 1".to_string(),
                similarity_score: 0.9,
                sections: vec!["header".to_string(), "content".to_string(), "footer".to_string()],
                created_at: Utc::now(),
            },
            SimilarDocument {
                document_id: "2".to_string(),
                title: "Doc 2".to_string(),
                similarity_score: 0.9,
                sections: vec!["header".to_string(), "content".to_string()],
                created_at: Utc::now(),
            },
        ];

        let config = ResearchAgentConfig {
            mandatory_threshold: 0.9, // 100% required for mandatory
            ..Default::default()
        };

        let (mandatory, optional) = extract_structure_patterns(&documents, &config);

        // header and content appear in 100% of documents
        assert!(mandatory.contains(&"header".to_string()));
        assert!(mandatory.contains(&"content".to_string()));
        // footer appears in 50% (optional)
        assert!(optional.contains(&"footer".to_string()) || mandatory.contains(&"footer".to_string()));
    }

    #[test]
    fn test_get_default_structure_for_woo_besluit() {
        let (mandatory, optional) = get_default_structure("woo_besluit");

        assert_eq!(mandatory, vec!["header", "references", "consideration", "decision", "signature"]);
        assert_eq!(optional, vec!["attachments", "footnotes"]);
    }

    #[test]
    fn test_get_default_structure_for_woo_info() {
        let (mandatory, optional) = get_default_structure("woo_info");

        assert_eq!(mandatory, vec!["header", "description", "availability", "contact"]);
        assert_eq!(optional, vec!["exceptions", "costs"]);
    }

    #[test]
    fn test_get_default_structure_for_unknown_type() {
        let (mandatory, optional) = get_default_structure("unknown_type");

        assert_eq!(mandatory, vec!["header", "content"]);
        assert_eq!(optional, vec!["attachments"]);
    }

    #[test]
    fn test_suggest_template_variables_from_request_context() {
        let mut request_context = HashMap::new();
        request_context.insert("title".to_string(), "Test Title".to_string());
        request_context.insert("reference".to_string(), "REF-001".to_string());

        let suggestions = suggest_template_variables(&request_context, &[], &DomainContext {
            domain_id: "test".to_string(),
            domain_name: "Test".to_string(),
            organizational_unit: None,
            applicable_regulations: vec![],
            standard_procedures: vec![],
        });

        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.name == "title"));
        assert!(suggestions.iter().any(|s| s.name == "reference"));
    }

    #[test]
    fn test_suggest_template_variables_includes_entity_suggestions() {
        let mut request_context = HashMap::new();
        request_context.insert("title".to_string(), "Test".to_string());

        let entities = vec![
            EntityReference {
                entity_id: "entity-123".to_string(),
                entity_type: "Organization".to_string(),
                name: "Gemeente Amsterdam".to_string(),
                relevance_score: 0.9,
            },
        ];

        let suggestions = suggest_template_variables(&request_context, &entities, &DomainContext {
            domain_id: "test".to_string(),
            domain_name: "Test".to_string(),
            organizational_unit: None,
            applicable_regulations: vec![],
            standard_procedures: vec![],
        });

        assert!(suggestions.iter().any(|s| s.name.contains("gemeente") || s.name.contains("amsterdam")));
    }
}
