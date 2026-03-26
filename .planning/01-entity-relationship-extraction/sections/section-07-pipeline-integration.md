Now I have a comprehensive understanding. Let me generate the section content for section-07-pipeline-integration:

# Section 07: Document Pipeline Integration

## Overview

This section integrates the stakeholder extraction system into the existing document creation pipeline. The extraction is invoked after the Content Agent generates document content, with extracted entities being added to the KnowledgeGraph and mention relationships being created to link entities to documents.

**Files to modify:**
- `crates/iou-ai/src/agents/pipeline.rs` - Add extraction call after Content Agent
- `crates/iou-ai/src/agents/mod.rs` - Re-export extraction types

**Dependencies:**
- **Section 01 (Foundation & Types):** Core data structures including `PersonStakeholder`, `OrganizationStakeholder`, `ExtractionResult`
- **Section 06 (Main Extractor):** The main `StakeholderExtractor` implementation

## Architecture

The extraction is integrated into the document pipeline as a non-blocking step after Content Agent completion:

```
Document Request
        │
        ▼
┌───────────────────┐
│ Research Agent    │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Content Agent     │ ← Generates GeneratedDocument
└───────────────────┘
        │
        ▼
┌───────────────────────────────────────────┐
│ Stakeholder Extraction (NEW)              │
│ • Extract persons/organizations           │
│ • Add entities to KnowledgeGraph          │
│ • Create mention relationships            │
│ • Non-blocking: logs error if fails       │
└───────────────────────────────────────────┘
        │
        ▼
┌───────────────────┐
│ Compliance Agent  │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Review Agent      │
└───────────────────┘
```

### Integration Behavior

The extraction step is **non-blocking** to ensure document creation is not impacted by extraction failures. If extraction fails:
- A warning is logged
- The pipeline continues with remaining agents
- Document content is not lost

## Implementation

### 1. Update PipelineConfig

**File:** `crates/iou-ai/src/agents/pipeline.rs`

Add stakeholder extraction configuration to the `PipelineConfig` struct:

```rust
use crate::stakeholder::{StakeholderExtractor, ExtractionOptions};

/// Configuration for pipeline execution
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Maximum number of maker-checker iterations
    pub max_iterations: usize,

    /// Maximum retries per agent for transient errors
    pub max_retries: u32,

    /// Initial backoff duration for transient errors
    pub initial_backoff: Duration,

    /// Maximum backoff duration
    pub max_backoff: Duration,

    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,

    /// Enable checkpoint/restart capability
    pub enable_checkpoints: bool,

    /// Enable stakeholder extraction (NEW)
    pub enable_stakeholder_extraction: bool,

    /// Stakeholder extraction options (NEW)
    pub stakeholder_options: ExtractionOptions,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_iterations: 3,
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(16),
            backoff_multiplier: 2.0,
            enable_checkpoints: true,
            enable_stakeholder_extraction: false,  // Disabled by default
            stakeholder_options: ExtractionOptions::default(),
        }
    }
}
```

### 2. Add Extractor to AgentPipeline

**File:** `crates/iou-ai/src/agents/pipeline.rs`

Add the stakeholder extractor to the pipeline structures:

```rust
/// Agent pipeline with all dependencies
pub struct AgentPipeline {
    /// Knowledge graph client (for Research and Content agents)
    pub kg_client: Arc<KnowledgeGraph>,

    /// Template engine (for Content agent)
    pub template_engine: Arc<TemplateEngine>,

    /// Domain configuration
    pub domain_config: Arc<DomainConfig>,

    /// Stakeholder extractor (NEW - Optional)
    pub stakeholder_extractor: Option<Arc<dyn StakeholderExtractor>>,
}

impl AgentPipeline {
    /// Create a new agent pipeline
    pub fn new(
        kg_client: Arc<KnowledgeGraph>,
        template_engine: Arc<TemplateEngine>,
        domain_config: Arc<DomainConfig>,
    ) -> Self {
        Self {
            kg_client,
            template_engine,
            domain_config,
            stakeholder_extractor: None,  // Disabled by default
        }
    }

    /// Set the stakeholder extractor (NEW)
    pub fn with_stakeholder_extractor(
        mut self,
        extractor: Arc<dyn StakeholderExtractor>,
    ) -> Self {
        self.stakeholder_extractor = Some(extractor);
        self
    }
}
```

### 3. Add Extraction Call After Content Agent

**File:** `crates/iou-ai/src/agents/pipeline.rs`

In `AgentPipelineWithConfig::execute_document_pipeline`, after the Content Agent completes (approximately line 251), add the stakeholder extraction:

```rust
impl AgentPipelineWithConfig {
    pub async fn execute_document_pipeline(
        &self,
        request: &DocumentRequest,
        template: &Template,
    ) -> Result<PipelineResult, PipelineError> {
        let started_at = Utc::now();
        let mut agent_results = Vec::new();
        let mut iteration = 0;

        loop {
            iteration += 1;

            if iteration > self.config.max_iterations {
                return Ok(PipelineResult { /* ... */ });
            }

            // ... Research Agent execution ...

            // Execute Content Agent
            let (generated_document, content_result) = self.execute_agent_with_retry(
                &(request, &research_context, template),
                "Content",
                |(req, research, tmpl)| self.execute_content(req, research, tmpl),
            ).await?;

            agent_results.push(content_result);

            // NEW: Execute Stakeholder Extraction after Content Agent
            if self.config.enable_stakeholder_extraction {
                if let Some(extractor) = &self.stakeholder_extractor {
                    let extraction_start = std::time::Instant::now();

                    match extractor.extract(&generated_document, &self.config.stakeholder_options).await {
                        Ok(extraction_result) => {
                            // Add extracted entities to knowledge graph
                            let mut entity_ids = Vec::new();

                            for person in extraction_result.persons {
                                let entity: Entity = person.into();
                                self.kg_client.add_entity(entity.clone()).await;
                                entity_ids.push(entity.id);
                            }

                            for org in extraction_result.organizations {
                                let entity: Entity = org.into();
                                self.kg_client.add_entity(entity.clone()).await;
                                entity_ids.push(entity.id);
                            }

                            // Create mention relationships
                            for mention in extraction_result.mentions {
                                let relationship: Relationship = mention.into();
                                self.kg_client.add_relationship(relationship).await;
                            }

                            let extraction_time = extraction_start.elapsed();

                            // Log extraction stats
                            tracing::info!(
                                "Stakeholder extraction complete for document {}: {} persons, {} organizations, {} mentions ({}ms)",
                                request.id,
                                extraction_result.persons.len(),
                                extraction_result.organizations.len(),
                                extraction_result.mentions.len(),
                                extraction_time.as_millis()
                            );

                            // Track in agent results for audit trail
                            agent_results.push(AgentExecutionResult {
                                agent_name: "StakeholderExtraction".to_string(),
                                success: true,
                                started_at: Utc::now(),
                                completed_at: Utc::now(),
                                execution_time_ms: extraction_time.as_millis() as u64,
                                retry_count: 0,
                                data: serde_json::json!({
                                    "persons_extracted": extraction_result.persons.len(),
                                    "organizations_extracted": extraction_result.organizations.len(),
                                    "mentions_created": extraction_result.mentions.len(),
                                    "stats": extraction_result.stats,
                                }),
                                errors: vec![],
                            });
                        }
                        Err(e) => {
                            // Non-blocking: log error and continue
                            tracing::warn!(
                                "Stakeholder extraction failed for document {}: {}, continuing pipeline",
                                request.id,
                                e
                            );

                            agent_results.push(AgentExecutionResult {
                                agent_name: "StakeholderExtraction".to_string(),
                                success: false,
                                started_at: Utc::now(),
                                completed_at: Utc::now(),
                                execution_time_ms: 0,
                                retry_count: 0,
                                data: serde_json::Value::Null,
                                errors: vec![e.to_string()],
                            });
                        }
                    }
                }
            }

            // Execute Compliance Agent
            let (compliance_result, compliance_exec_result) = self.execute_agent_with_retry(
                &generated_document,
                "Compliance",
                |doc| self.execute_compliance(doc),
            ).await?;

            // ... rest of pipeline ...
        }
    }
}
```

### 4. Update Module Exports

**File:** `crates/iou-ai/src/agents/mod.rs`

The stakeholder module should already be exported. Verify that the following are accessible:

```rust
pub use error::{PipelineError, ErrorSeverity};
pub use pipeline::{
    AgentPipeline, AgentPipelineWithConfig, PipelineConfig,
    AgentExecutionResult, PipelineCheckpoint, PipelineResult,
};
```

The `StakeholderExtractor` trait and related types from section-01 should already be exported via the stakeholder module.

## Tests

### Test Stubs

Write the following tests in `crates/iou-ai/src/agents/pipeline_tests.rs` (extending existing tests):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::stakeholder::{MockStakeholderExtractor, ExtractionResult, ExtractionStats};

    fn create_pipeline_with_extractor() -> AgentPipelineWithConfig {
        let kg = Arc::new(KnowledgeGraph::new());
        let engine = Arc::new(TemplateEngine::new().unwrap());
        let domain_config = Arc::new(create_test_domain_config());
        let config = PipelineConfig {
            enable_stakeholder_extraction: true,
            ..Default::default()
        };

        let mock_extractor = Arc::new(MockStakeholderExtractor::new());

        AgentPipelineWithConfig {
            kg_client: kg,
            template_engine: engine,
            domain_config,
            config,
            stakeholder_extractor: Some(mock_extractor),
        }
    }

    #[tokio::test]
    async fn test_stakeholder_extractor_called_after_content_agent() {
        let pipeline = create_pipeline_with_extractor();
        let request = create_test_request();
        let template = create_test_template();

        let mock_extractor = pipeline.stakeholder_extractor.as_ref().unwrap();
        mock_extractor.set_expected_result(ExtractionResult {
            persons: vec![],
            organizations: vec![],
            mentions: vec![],
            stats: ExtractionStats::default(),
            processing_time_ms: 100,
        });

        let result = pipeline.execute_document_pipeline(&request, &template).await;

        assert!(result.is_ok());

        // Verify extraction was called
        assert!(mock_extractor.was_called());
    }

    #[tokio::test]
    async fn test_extracted_entities_added_to_knowledge_graph() {
        let kg = Arc::new(KnowledgeGraph::new());
        let engine = Arc::new(TemplateEngine::new().unwrap());
        let domain_config = Arc::new(create_test_domain_config());
        let config = PipelineConfig {
            enable_stakeholder_extraction: true,
            ..Default::default()
        };

        let mock_extractor = Arc::new(MockStakeholderExtractor::with_entities(
            2,  // persons
            1,  // organizations
        ));

        let pipeline = AgentPipelineWithConfig {
            kg_client: kg.clone(),
            template_engine: engine,
            domain_config,
            config,
            stakeholder_extractor: Some(mock_extractor),
        };

        let request = create_test_request();
        let template = create_test_template();

        pipeline.execute_document_pipeline(&request, &template).await.unwrap();

        // Verify entities were added to graph
        let entities = kg.entities();
        assert!(entities.len() >= 3);  // 2 persons + 1 organization
    }

    #[tokio::test]
    async fn test_mention_relationships_created_for_each_entity() {
        let pipeline = create_pipeline_with_extractor();

        let mock_extractor = pipeline.stakeholder_extractor.as_ref().unwrap();
        mock_extractor.set_expected_result(ExtractionResult {
            persons: vec![],
            organizations: vec![],
            mentions: vec![/* 3 mentions */],
            stats: ExtractionStats::default(),
            processing_time_ms: 100,
        });

        let request = create_test_request();
        let template = create_test_template();

        let result = pipeline.execute_document_pipeline(&request, &template).await.unwrap();

        // Verify mention relationships were tracked
        let extraction_result = result.agent_results.iter()
            .find(|r| r.agent_name == "StakeholderExtraction");
        assert!(extraction_result.is_some());

        let data = extraction_result.unwrap().data;
        assert_eq!(data["mentions_created"], 3);
    }

    #[tokio::test]
    async fn test_extraction_stats_logged_with_correct_counts() {
        let pipeline = create_pipeline_with_extractor();

        let mock_extractor = pipeline.stakeholder_extractor.as_ref().unwrap();
        mock_extractor.set_expected_result(ExtractionResult {
            persons: vec![],
            organizations: vec![],
            mentions: vec![],
            stats: ExtractionStats {
                total_entities: 5,
                high_confidence: 2,
                medium_confidence: 2,
                low_confidence: 1,
                llm_calls_made: 1,
                api_calls_made: 2,
            },
            processing_time_ms: 100,
        });

        let request = create_test_request();
        let template = create_test_template();

        let result = pipeline.execute_document_pipeline(&request, &template).await.unwrap();

        let extraction_result = result.agent_results.iter()
            .find(|r| r.agent_name == "StakeholderExtraction")
            .unwrap();

        assert_eq!(extraction_result.data["stats"]["total_entities"], 5);
        assert_eq!(extraction_result.data["stats"]["high_confidence"], 2);
        assert_eq!(extraction_result.data["stats"]["medium_confidence"], 2);
    }

    #[tokio::test]
    async fn test_pipeline_continues_if_extraction_fails() {
        let kg = Arc::new(KnowledgeGraph::new());
        let engine = Arc::new(TemplateEngine::new().unwrap());
        let domain_config = Arc::new(create_test_domain_config());
        let config = PipelineConfig {
            enable_stakeholder_extraction: true,
            ..Default::default()
        };

        let failing_extractor = Arc::new(MockStakeholderExtractor::failing());

        let pipeline = AgentPipelineWithConfig {
            kg_client: kg,
            template_engine: engine,
            domain_config,
            config,
            stakeholder_extractor: Some(failing_extractor),
        };

        let request = create_test_request();
        let template = create_test_template();

        let result = pipeline.execute_document_pipeline(&request, &template).await;

        // Pipeline should succeed despite extraction failure
        assert!(result.is_ok());

        let pipeline_result = result.unwrap();

        // Verify extraction failure was logged
        let extraction_result = pipeline_result.agent_results.iter()
            .find(|r| r.agent_name == "StakeholderExtraction");
        assert!(extraction_result.is_some());
        assert!(!extraction_result.unwrap().success);
    }

    #[tokio::test]
    async fn test_generated_document_text_accessible_to_extractor() {
        let pipeline = create_pipeline_with_extractor();
        let request = create_test_request();
        let template = create_test_template();

        let mock_extractor = pipeline.stakeholder_extractor.as_ref().unwrap();

        pipeline.execute_document_pipeline(&request, &template).await.unwrap();

        // Verify extractor received the generated document
        assert!(mock_extractor.received_document());
    }

    #[tokio::test]
    async fn test_pipeline_config_enable_stakeholder_extraction_controls_execution() {
        let kg = Arc::new(KnowledgeGraph::new());
        let engine = Arc::new(TemplateEngine::new().unwrap());
        let domain_config = Arc::new(create_test_domain_config());
        let config = PipelineConfig {
            enable_stakeholder_extraction: false,  // Disabled
            ..Default::default()
        };

        let mock_extractor = Arc::new(MockStakeholderExtractor::new());

        let pipeline = AgentPipelineWithConfig {
            kg_client: kg,
            template_engine: engine,
            domain_config,
            config,
            stakeholder_extractor: Some(mock_extractor),
        };

        let request = create_test_request();
        let template = create_test_template();

        pipeline.execute_document_pipeline(&request, &template).await.unwrap();

        // Extractor should NOT have been called
        assert!(!mock_extractor.was_called());
    }

    #[tokio::test]
    async fn test_pipeline_config_stakeholder_options_passed_to_extractor() {
        let kg = Arc::new(KnowledgeGraph::new());
        let engine = Arc::new(TemplateEngine::new().unwrap());
        let domain_config = Arc::new(create_test_domain_config());

        let mut options = ExtractionOptions::default();
        options.use_llm = true;
        options.confidence_threshold = 0.8;

        let config = PipelineConfig {
            enable_stakeholder_extraction: true,
            stakeholder_options: options,
            ..Default::default()
        };

        let mock_extractor = Arc::new(MockStakeholderExtractor::new());

        let pipeline = AgentPipelineWithConfig {
            kg_client: kg,
            template_engine: engine,
            domain_config,
            config,
            stakeholder_extractor: Some(mock_extractor),
        };

        let request = create_test_request();
        let template = create_test_template();

        pipeline.execute_document_pipeline(&request, &template).await.unwrap();

        // Verify options were passed
        let received_options = mock_extractor.received_options();
        assert!(received_options.is_some());
        assert_eq!(received_options.unwrap().use_llm, true);
        assert_eq!(received_options.unwrap().confidence_threshold, 0.8);
    }
}
```

### Mock Extractor for Testing

Create a mock extractor in `crates/iou-ai/src/stakeholder/mock.rs`:

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;

use crate::stakeholder::{StakeholderExtractor, ExtractionResult, ExtractionOptions, ExtractionError};
use crate::agents::content::GeneratedDocument;

pub struct MockStakeholderExtractor {
    called: Arc<AtomicBool>,
    expected_result: Option<ExtractionResult>,
    fail: bool,
}

impl MockStakeholderExtractor {
    pub fn new() -> Self {
        Self {
            called: Arc::new(AtomicBool::new(false)),
            expected_result: None,
            fail: false,
        }
    }

    pub fn with_entities(persons: usize, organizations: usize) -> Self {
        Self {
            called: Arc::new(AtomicBool::new(false)),
            expected_result: Some(ExtractionResult {
                persons: vec![/* mock persons */],
                organizations: vec![/* mock organizations */],
                mentions: vec![/* mock mentions */],
                stats: ExtractionStats::default(),
                processing_time_ms: 100,
            }),
            fail: false,
        }
    }

    pub fn set_expected_result(&self, result: ExtractionResult) {
        // Store expected result
    }

    pub fn failing() -> Self {
        Self {
            called: Arc::new(AtomicBool::new(false)),
            expected_result: None,
            fail: true,
        }
    }

    pub fn was_called(&self) -> bool {
        self.called.load(Ordering::SeqCst)
    }

    pub fn received_document(&self) -> bool {
        // Track if document was received
        true
    }

    pub fn received_options(&self) -> Option<&ExtractionOptions> {
        // Return received options
        None
    }
}

#[async_trait]
impl StakeholderExtractor for MockStakeholderExtractor {
    async fn extract(
        &self,
        _document: &GeneratedDocument,
        _options: &ExtractionOptions,
    ) -> Result<ExtractionResult, ExtractionError> {
        self.called.store(true, Ordering::SeqCst);

        if self.fail {
            return Err(ExtractionError::ExtractionFailed("Mock failure".to_string()));
        }

        Ok(self.expected_result.clone().unwrap_or_default())
    }

    async fn normalize_entities(
        &self,
        _entities: Vec<iou_core::graphrag::Entity>,
    ) -> Result<Vec<iou_core::graphrag::Entity>, ExtractionError> {
        unimplemented!()
    }

    async fn deduplicate_entities(
        &self,
        _entities: Vec<iou_core::graphrag::Entity>,
    ) -> Result<Vec<iou_core::graphrag::Entity>, ExtractionError> {
        unimplemented!()
    }
}
```

## Success Criteria

- Documents are processed with stakeholder extraction when enabled
- Extracted entities appear in KnowledgeGraph after pipeline execution
- Mention relationships are created linking entities to documents
- Extraction stats are logged with correct counts
- Pipeline continues without losing document content if extraction fails
- GeneratedDocument text is accessible to the extractor
- `PipelineConfig.enable_stakeholder_extraction` controls execution
- `PipelineConfig.stakeholder_options` are passed to the extractor

## Integration Points

This section connects to:
- **Section 06 (Main Extractor):** Uses the `StakeholderExtractor` trait
- **Section 08 (KnowledgeGraph Extensions):** Adds entities and relationships to the graph
- **Section 09 (API Endpoints):** Entities added here will be queryable via API

## Error Handling

The integration implements resilient error handling:

| Failure Mode | Behavior | Impact |
|--------------|----------|--------|
| Extractor not configured | Skip extraction | No entities extracted |
| Extraction fails | Log warning, continue pipeline | Document created without entities |
| Entity add to graph fails | Log error | Mention may not be created |
| KnowledgeGraph unavailable | Log error | Extraction runs but entities not stored |

## Configuration

Stakeholder extraction is **disabled by default** to avoid breaking existing workflows. To enable:

```rust
let config = PipelineConfig {
    enable_stakeholder_extraction: true,
    stakeholder_options: ExtractionOptions {
        use_llm: true,
        confidence_threshold: 0.5,
        enable_normalization: true,
        ..Default::default()
    },
    ..Default::default()
};

let pipeline = AgentPipeline::new(kg, engine, domain_config)
    .with_config(kg, engine, domain_config, config)
    .with_stakeholder_extractor(Arc::new(extractor));
```

## Existing Pipeline Context

The current pipeline flow (from `crates/iou-ai/src/agents/pipeline.rs`) is:

1. Research Agent → `ResearchContext`
2. Content Agent → `GeneratedDocument`
3. Compliance Agent → `ComplianceResult`
4. Review Agent → `ReviewDecision`
5. Approval/Iteration logic

This section inserts stakeholder extraction between steps 2 and 3, using the `GeneratedDocument.content` field as input for entity extraction.

The `GeneratedDocument` structure (from `crates/iou-ai/src/agents/content.rs`) provides:
- `document_id: Uuid` - For linking mentions to documents
- `content: String` - The Markdown text to extract entities from
- `variables: Vec<TemplateVariable>` - Context that may aid extraction
- `entity_links: Vec<EntityLink>` - Existing entity references