Now I have all the context needed. Let me extract the relevant content for section-03-research-agent. Looking at the index.md, the section summary is:

**section-03-research-agent**: GraphRAG integration: Query similar documents, extract structure patterns, identify mandatory/optional sections, retrieve domain context, return ResearchContext.

Let me extract the relevant tests from claude-plan-tdd.md and implementation details from claude-plan.md.</think># Section 3: Research Agent

## Overview

The Research Agent is the first agent in the document creation pipeline. Its purpose is to analyze the document request, query the knowledge graph (GraphRAG) for similar documents and context, determine the appropriate document structure, and identify mandatory versus optional sections.

**Dependencies:**
- section-01-foundation (core domain types, storage client, GraphRAG Document entity)

**Blocks:** section-04-content-agent (depends on ResearchContext output)

## Tests

### 4.1 Research Agent Tests

Extracted from TDD plan, these tests verify the Research Agent correctly queries GraphRAG and analyzes document structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_research_agent_queries_graphrag_for_similar_documents() {
        // Given: A document request and mock GraphRAG client
        let request = create_test_request("woo_besluit");
        let mock_kg = MockGraphRagClient::new();
        mock_kg.expect_similar_documents()
            .with(mockdoc::type_eq("woo_besluit"))
            .returning(|_| Ok(vec![create_similar_doc()]));

        // When: Executing research agent
        let result = execute_research_agent(&request, &mock_kg, &test_domain_config()).await;

        // Then: GraphRAG was queried and results returned
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(!context.similar_documents.is_empty());
    }

    #[tokio::test]
    async fn test_execute_research_agent_extracts_structure_patterns_from_results() {
        // Given: Similar documents with known structure
        let request = create_test_request("woo_besluit");
        let mock_kg = MockGraphRagClient::with_structure_pattern(vec![
            "header", "references", "consideration", "decision"
        ]);

        // When: Executing research agent
        let result = execute_research_agent(&request, &mock_kg, &test_domain_config()).await;

        // Then: Structure patterns are extracted
        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.required_sections, vec!["header", "references", "consideration", "decision"]);
    }

    #[tokio::test]
    async fn test_execute_research_agent_identifies_mandatory_vs_optional_sections() {
        // Given: Documents with mixed mandatory/optional sections
        let request = create_test_request("woo_info");
        let mock_kg = MockGraphRagClient::new();

        // When: Executing research agent
        let result = execute_research_agent(&request, &mock_kg, &test_domain_config()).await;

        // Then: Sections are correctly classified
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(context.mandatory_sections.contains(&"header".to_string()));
        assert!(context.optional_sections.contains(&"attachments".to_string()));
    }

    #[tokio::test]
    async fn test_execute_research_agent_handles_empty_graphrag_results_gracefully() {
        // Given: No similar documents found
        let request = create_test_request("unknown_type");
        let mock_kg = MockGraphRagClient::empty();

        // When: Executing research agent
        let result = execute_research_agent(&request, &mock_kg, &test_domain_config()).await;

        // Then: Returns default structure instead of error
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(!context.required_sections.is_empty()); // Has defaults
    }

    #[tokio::test]
    async fn test_execute_research_agent_returns_research_context_with_required_fields() {
        // Given: Valid inputs
        let request = create_test_request("woo_besluit");
        let mock_kg = MockGraphRagClient::new();

        // When: Executing research agent
        let result = execute_research_agent(&request, &mock_kg, &test_domain_config()).await;

        // Then: All required fields present
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(!context.document_type.is_empty());
        assert!(!context.domain_context.is_empty());
        assert!(context.timestamp <=Utc::now());
    }
}
```

## Implementation Details

### Purpose and Scope

The Research Agent is responsible for:
1. Querying GraphRAG for documents of similar type within the domain
2. Extracting common structure patterns from similar documents
3. Identifying mandatory versus optional sections based on domain rules
4. Retrieving related entities from the knowledge graph for context
5. Returning a structured `ResearchContext` for the Content Agent

### Domain Types (from section-01-foundation)

The Research Agent uses these core types defined in the foundation:

```rust
// From iou-core/src/document.rs
pub type DocumentId = Uuid;
pub use crate::workflows::WorkflowStatus as DocumentState;

pub struct DocumentRequest {
    pub id: DocumentId,
    pub domain_id: String,
    pub document_type: String,
    pub context: HashMap<String, String>,
    pub requested_at: DateTime<Utc>,
}

pub struct DomainConfig {
    pub domain_id: String,
    pub trust_level: TrustLevel,
    pub required_approval_threshold: f32,
    pub auto_approval_threshold: f32,
}
```

### Research Agent Types

Define these types in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/research.rs`:

```rust
use iou_core::document::{DocumentRequest, DomainConfig};
use serde::{Deserialize, Serialize};

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
    pub entity_id: String,
    pub entity_type: String,  // Person, Organization, Location, etc.
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableSource {
    UserInput,
    KnowledgeGraph,
    AgentGenerated,
    Default,
}
```

### Research Agent Function Signature

```rust
use crate::agents::AgentError;
use crate::graphrag::GraphRagClient;

/// Executes the Research Agent phase
///
/// # Arguments
/// * `request` - The original document request
/// * `kg_client` - GraphRAG client for knowledge graph queries
/// * `domain_config` - Domain configuration including trust level
///
/// # Returns
/// `ResearchContext` containing all discovered information
pub async fn execute_research_agent(
    request: &DocumentRequest,
    kg_client: &GraphRagClient,
    domain_config: &DomainConfig,
) -> Result<ResearchContext, AgentError>;
```

### Agent Error Type

Define in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/mod.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("GraphRAG query failed: {0}")]
    GraphRagQueryFailed(String),
    
    #[error("No similar documents found and no default template available")]
    NoSimilarDocuments,
    
    #[error("Invalid document type: {0}")]
    InvalidDocumentType(String),
    
    #[error("Domain configuration not found: {0}")]
    DomainNotFound(String),
    
    #[error("AI provider error: {0}")]
    AiProviderError(String),
    
    #[error("Transient error (will retry): {0}")]
    TransientError(String),
}
```

### Key Behaviors to Implement

1. **Query GraphRAG for Similar Documents**
   - Construct semantic search query from document_type and context
   - Filter by domain_id to ensure domain-relevant results
   - Retrieve top N similar documents (default: 5)

2. **Extract Structure Patterns**
   - Analyze section headings from similar documents
   - Identify common patterns (e.g., 90%+ occurrence = mandatory)
   - Extract section order for logical flow

3. **Identify Mandatory vs Optional Sections**
   - Use domain configuration for mandatory section rules
   - Cross-reference with PROVISA guidelines
   - Classify sections based on occurrence frequency

4. **Retrieve Related Entities**
   - Extract entity references from request context
   - Query GraphRAG for related entities
   - Score relevance by connection strength

5. **Handle Empty Results**
   - When no similar documents found, use domain defaults
   - Log the absence for monitoring
   - Never fail - always return a valid ResearchContext

### GraphRAG Integration

The Research Agent queries the GraphRAG Document entity. The Document entity schema should be defined in section-01-foundation:

```rust
// Example GraphRAG Document entity structure
pub struct GraphRAGDocument {
    pub id: String,
    pub domain_id: String,
    pub document_type: String,
    pub title: String,
    pub content: String,
    pub sections: Vec<DocumentSection>,
    pub entities: Vec<String>,  // Entity IDs
    pub metadata: HashMap<String, String>,
}

pub struct DocumentSection {
    pub heading: String,
    pub content: String,
    pub level: u8,  // h1, h2, h3, etc.
}
```

### Default Document Structures

When no similar documents are found, fall back to these default structures:

```rust
/// Default section structures for common document types
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
            vec!["attachments".to_string(), "footnotes".to_string()]
        ),
        "woo_info" => (
            vec![
                "header".to_string(),
                "description".to_string(),
                "availability".to_string(),
                "contact".to_string(),
            ],
            vec!["exceptions".to_string(), "costs".to_string()]
        ),
        _ => (
            vec!["header".to_string(), "content".to_string()],
            vec!["attachments".to_string()]
        )
    }
}
```

### Configuration

Add to agent configuration in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/config.rs`:

```rust
pub struct ResearchAgentConfig {
    /// Maximum number of similar documents to retrieve
    pub max_similar_documents: usize,
    
    /// Minimum similarity score threshold (0.0 - 1.0)
    pub similarity_threshold: f32,
    
    /// Minimum occurrence frequency for mandatory section (0.0 - 1.0)
    pub mandatory_threshold: f32,
    
    /// Whether to use AI provider for enhanced analysis
    pub use_ai_enhancement: bool,
}

impl Default for ResearchAgentConfig {
    fn default() -> Self {
        Self {
            max_similar_documents: 5,
            similarity_threshold: 0.7,
            mandatory_threshold: 0.9,
            use_ai_enhancement: true,
        }
    }
}
```

## File Structure

Create/modify these files:

```
/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/
├── mod.rs                    # Agent exports, AgentError type
├── research.rs               # Research Agent implementation (NEW)
├── config.rs                 # Agent configuration (NEW or extend existing)
```

## Implementation Checklist

1. Create `ResearchContext`, `DomainContext`, and related types
2. Implement `execute_research_agent` function
3. Integrate with GraphRAG client for document queries
4. Add structure pattern extraction logic
5. Implement mandatory/optional section classification
6. Add entity resolution from knowledge graph
7. Implement fallback to default structures
8. Add error handling for transient vs permanent errors
9. Write unit tests for all behaviors
10. Add integration tests with mock GraphRAG client