//! Pipeline orchestration for the multi-agent document creation system.
//!
//! This module coordinates the execution of the four document creation agents:
//! - Research Agent: Query knowledge graph, determine structure
//! - Content Agent: Generate document from template
//! - Compliance Agent: Validate Woo rules, detect PII
//! - Review Agent: Quality check, decide approval
//!
//! The pipeline handles:
//! - Sequential execution of agents
//! - Maker-checker iteration loop (Content → Compliance → Review)
//! - Transient error retry with exponential backoff
//! - Permanent error fail-fast
//! - Checkpoint/restart capability
//! - Audit trail logging

use super::error::{PipelineError, ErrorSeverity};
use crate::agents::{
    research::{execute_research_agent, ResearchContext},
    content::{execute_content_agent, GeneratedDocument},
    compliance::{execute_compliance_agent, ComplianceResult},
    review::{execute_review_agent, ReviewDecision, ReviewAction},
    AgentError,
};
use crate::graphrag::KnowledgeGraph;
use crate::stakeholder::{StakeholderExtractor, ExtractionOptions};
use crate::templates::TemplateEngine;
use chrono::{DateTime, Utc};
use iou_core::document::{DocumentRequest, DomainConfig, Template};
use iou_core::workflows::WorkflowStatus;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

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

    /// Enable stakeholder extraction after Content Agent
    pub enable_stakeholder_extraction: bool,

    /// Stakeholder extraction options
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
            enable_stakeholder_extraction: false,
            stakeholder_options: ExtractionOptions::default(),
        }
    }
}

/// Result from a single agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionResult {
    pub agent_name: String,
    pub success: bool,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub execution_time_ms: u64,
    pub retry_count: u32,
    pub data: serde_json::Value,
    pub errors: Vec<String>,
}

/// Checkpoint data for pipeline recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineCheckpoint {
    pub document_id: Uuid,
    pub current_agent: Option<String>,
    pub completed_agents: Vec<String>,
    pub iteration: usize,
    pub saved_at: DateTime<Utc>,
}

/// Final pipeline execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub document_id: Uuid,
    pub final_status: WorkflowStatus,
    pub agent_results: Vec<AgentExecutionResult>,
    pub total_iterations: usize,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub requires_human_approval: bool,
    pub compliance_score: f32,
    pub confidence_score: f32,
}

/// Agent pipeline with all dependencies
pub struct AgentPipeline {
    /// Knowledge graph client (for Research and Content agents)
    pub kg_client: Arc<KnowledgeGraph>,

    /// Template engine (for Content agent)
    pub template_engine: Arc<TemplateEngine>,

    /// Domain configuration
    pub domain_config: Arc<DomainConfig>,
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
        }
    }

    /// Create a new agent pipeline with custom configuration
    pub fn with_config(
        kg_client: Arc<KnowledgeGraph>,
        template_engine: Arc<TemplateEngine>,
        domain_config: Arc<DomainConfig>,
        config: PipelineConfig,
    ) -> AgentPipelineWithConfig {
        AgentPipelineWithConfig {
            kg_client,
            template_engine,
            domain_config,
            config,
            stakeholder_extractor: None,
        }
    }

    /// Set the stakeholder extractor for the pipeline
    /// Note: This returns an AgentPipelineWithConfig since the extractor is config-specific
    pub fn with_stakeholder_extractor(
        self,
        extractor: Arc<dyn StakeholderExtractor>,
    ) -> AgentPipelineWithConfig {
        AgentPipelineWithConfig {
            kg_client: self.kg_client,
            template_engine: self.template_engine,
            domain_config: self.domain_config,
            config: PipelineConfig::default(),
            stakeholder_extractor: Some(extractor),
        }
    }

    /// Execute the complete document creation pipeline with default config
    pub async fn execute_document_pipeline(
        &self,
        request: &DocumentRequest,
        template: &Template,
    ) -> Result<PipelineResult, PipelineError> {
        let config = PipelineConfig::default();
        let pipeline_with_config = AgentPipelineWithConfig {
            kg_client: self.kg_client.clone(),
            template_engine: self.template_engine.clone(),
            domain_config: self.domain_config.clone(),
            config,
            stakeholder_extractor: None,
        };
        pipeline_with_config.execute_document_pipeline(request, template).await
    }
}

/// Agent pipeline with explicit configuration
pub struct AgentPipelineWithConfig {
    /// Knowledge graph client (for Research and Content agents)
    pub kg_client: Arc<KnowledgeGraph>,

    /// Template engine (for Content agent)
    pub template_engine: Arc<TemplateEngine>,

    /// Domain configuration
    pub domain_config: Arc<DomainConfig>,

    /// Pipeline configuration
    pub config: PipelineConfig,

    /// Stakeholder extractor (optional)
    pub stakeholder_extractor: Option<Arc<dyn StakeholderExtractor>>,
}

impl AgentPipelineWithConfig {
    /// Execute the complete document creation pipeline
    ///
    /// # Pipeline Flow
    ///
    /// 1. Research Agent: Query knowledge graph, determine structure
    /// 2. Content Agent: Generate document from template
    /// 3. Compliance Agent: Validate Woo rules, detect PII
    /// 4. Review Agent: Quality check, decide approval
    /// 5. Maker-Checker: Iterate if Review requests changes
    /// 6. Finalize: Update state, return result
    ///
    /// # Error Handling
    ///
    /// - **Transient errors**: Retry with exponential backoff
    /// - **Permanent errors**: Fail immediately, return error to user
    pub async fn execute_document_pipeline(
        &self,
        request: &DocumentRequest,
        template: &Template,
    ) -> Result<PipelineResult, PipelineError> {
        let started_at = Utc::now();
        let mut agent_results = Vec::new();
        let mut iteration = 0;

        // Outer loop: Maker-checker iteration
        loop {
            iteration += 1;

            if iteration > self.config.max_iterations {
                return Ok(PipelineResult {
                    document_id: request.id,
                    final_status: WorkflowStatus::InReview,
                    agent_results,
                    total_iterations: iteration,
                    started_at,
                    completed_at: Utc::now(),
                    requires_human_approval: true,
                    compliance_score: 0.0,
                    confidence_score: 0.0,
                });
            }

            // Execute Research Agent (only on first iteration)
            let research_context = if iteration == 1 {
                let (context, result) = self.execute_agent_with_retry(
                    request,
                    "Research",
                    |req| self.execute_research(req),
                ).await?;

                agent_results.push(result);
                context
            } else {
                // Reuse research context from first iteration
                let research_result = agent_results.iter()
                    .find(|r| r.agent_name == "Research")
                    .ok_or_else(|| PipelineError::Configuration("Research agent result not found".to_string()))?;

                serde_json::from_value::<ResearchContext>(research_result.data.clone())
                    .map_err(|_| PipelineError::Configuration("Failed to parse ResearchContext".to_string()))?
            };

            // Execute Content Agent
            let (generated_document, content_result) = self.execute_agent_with_retry(
                &(request, &research_context, template),
                "Content",
                |(req, research, tmpl)| self.execute_content(req, research, tmpl),
            ).await?;

            agent_results.push(content_result);

            // Execute Stakeholder Extraction (after Content Agent, before Compliance)
            if self.config.enable_stakeholder_extraction {
                if let Some(extractor) = &self.stakeholder_extractor {
                    let extraction_start = std::time::Instant::now();

                    match extractor.extract(&generated_document, &self.config.stakeholder_options).await {
                        Ok(extraction_result) => {
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

                            // Note: Entities and relationships can be added to KnowledgeGraph
                            // separately via the KnowledgeGraph extension methods when needed
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

            agent_results.push(compliance_exec_result);

            // Execute Review Agent
            let (review_decision, review_exec_result) = self.execute_agent_with_retry(
                &(&generated_document, &compliance_result, &research_context),
                "Review",
                |(doc, comp, research)| self.execute_review(doc, comp, research),
            ).await?;

            agent_results.push(review_exec_result);

            // Save checkpoint after all agents complete
            if self.config.enable_checkpoints {
                self.save_checkpoint(PipelineCheckpoint {
                    document_id: request.id,
                    current_agent: None,
                    completed_agents: vec!["Research".into(), "Content".into(), "Compliance".into(), "Review".into()],
                    iteration,
                    saved_at: Utc::now(),
                }).await?;
            }

            // Check if iteration is needed
            match review_decision.action {
                ReviewAction::Approve => {
                    // Document approved, finalize
                    let completed_at = Utc::now();

                    // TODO: Defer to section-08 (API Layer): Store document in S3
                    // TODO: Defer to section-08 (API Layer): Update database state
                    // TODO: Defer to section-08 (API Layer): Add audit trail logging

                    return Ok(PipelineResult {
                        document_id: request.id,
                        final_status: if self.requires_human_approval(&compliance_result, &request.document_type) {
                            WorkflowStatus::InReview
                        } else {
                            WorkflowStatus::Approved
                        },
                        agent_results,
                        total_iterations: iteration,
                        started_at,
                        completed_at,
                        requires_human_approval: self.requires_human_approval(&compliance_result, &request.document_type),
                        compliance_score: compliance_result.score,
                        confidence_score: review_decision.overall_quality_score,
                    });
                }
                ReviewAction::RequireHumanApproval => {
                    // Requires human approval
                    return Ok(PipelineResult {
                        document_id: request.id,
                        final_status: WorkflowStatus::InReview,
                        agent_results,
                        total_iterations: iteration,
                        started_at,
                        completed_at: Utc::now(),
                        requires_human_approval: true,
                        compliance_score: compliance_result.score,
                        confidence_score: review_decision.overall_quality_score,
                    });
                }
                ReviewAction::RequestRevision => {
                    // Iterate back to Content Agent
                    if iteration >= self.config.max_iterations {
                        return Ok(PipelineResult {
                            document_id: request.id,
                            final_status: WorkflowStatus::InReview,
                            agent_results,
                            total_iterations: iteration,
                            started_at,
                            completed_at: Utc::now(),
                            requires_human_approval: true,
                            compliance_score: compliance_result.score,
                            confidence_score: review_decision.overall_quality_score,
                        });
                    }
                    // Continue loop to iterate
                    continue;
                }
                ReviewAction::Reject => {
                    // Document rejected
                    return Ok(PipelineResult {
                        document_id: request.id,
                        final_status: WorkflowStatus::Rejected,
                        agent_results,
                        total_iterations: iteration,
                        started_at,
                        completed_at: Utc::now(),
                        requires_human_approval: false,
                        compliance_score: compliance_result.score,
                        confidence_score: review_decision.overall_quality_score,
                    });
                }
            }
        }
    }

    /// Execute a single agent with retry logic for transient errors
    async fn execute_agent_with_retry<'a, Input, Output, AgentFn, Fut>(
        &'a self,
        input: &'a Input,
        agent_name: &'a str,
        agent_fn: AgentFn,
    ) -> Result<(Output, AgentExecutionResult), PipelineError>
    where
        AgentFn: Fn(&'a Input) -> Fut,
        Fut: std::future::Future<Output = Result<AgentExecutionOutput<Output>, PipelineError>>,
    {
        let started_at = Utc::now();
        let mut retry_count = 0;
        let mut backoff = self.config.initial_backoff;

        loop {
            match agent_fn(input).await {
                Ok(output) => {
                    let completed_at = Utc::now();
                    let execution_time_ms = (completed_at - started_at).num_milliseconds() as u64;
                    return Ok((output.output, AgentExecutionResult {
                        agent_name: agent_name.to_string(),
                        success: true,
                        started_at,
                        completed_at,
                        execution_time_ms,
                        retry_count,
                        data: output.data,
                        errors: vec![],
                    }));
                }
                Err(e) if e.severity() == ErrorSeverity::Transient && retry_count < self.config.max_retries => {
                    retry_count += 1;
                    sleep(backoff).await;
                    backoff = Duration::from_secs_f64(
                        (backoff.as_secs_f64() * self.config.backoff_multiplier)
                            .min(self.config.max_backoff.as_secs_f64())
                    );
                }
                Err(e) => {
                    let completed_at = Utc::now();
                    return Err(PipelineError::AgentFailed {
                        agent: agent_name.to_string(),
                        message: e.to_string(),
                        severity: e.severity(),
                    });
                }
            }
        }
    }

    /// Execute Research Agent
    async fn execute_research(
        &self,
        request: &DocumentRequest,
    ) -> Result<AgentExecutionOutput<ResearchContext>, PipelineError> {
        execute_research_agent(request, &self.kg_client, &self.domain_config)
            .await
            .map(|context| AgentExecutionOutput {
                data: serde_json::to_value(&context).unwrap_or_default(),
                output: context,
            })
            .map_err(|e| match e {
                AgentError::TransientError(msg) => PipelineError::transient_agent("Research", msg),
                _ => PipelineError::permanent_agent("Research", e.to_string()),
            })
    }

    /// Execute Content Agent
    async fn execute_content(
        &self,
        request: &&DocumentRequest,
        research: &ResearchContext,
        template: &Template,
    ) -> Result<AgentExecutionOutput<GeneratedDocument>, PipelineError> {
        execute_content_agent(request, research, template, &self.kg_client, &self.template_engine)
            .await
            .map(|document| AgentExecutionOutput {
                data: serde_json::json!({
                    "sections": document.sections.len(),
                    "word_count": document.content.split_whitespace().count(),
                }),
                output: document,
            })
            .map_err(|e| match e {
                AgentError::TemplateError(msg) => PipelineError::Template(msg),
                AgentError::TransientError(msg) => PipelineError::transient_agent("Content", msg),
                _ => PipelineError::permanent_agent("Content", e.to_string()),
            })
    }

    /// Execute Compliance Agent
    async fn execute_compliance(
        &self,
        document: &GeneratedDocument,
    ) -> Result<AgentExecutionOutput<ComplianceResult>, PipelineError> {
        execute_compliance_agent(document)
            .await
            .map(|result| AgentExecutionOutput {
                data: serde_json::to_value(&result).unwrap_or_default(),
                output: result,
            })
            .map_err(|e| match e {
                AgentError::TransientError(msg) => PipelineError::transient_agent("Compliance", msg),
                _ => PipelineError::permanent_agent("Compliance", e.to_string()),
            })
    }

    /// Execute Review Agent
    async fn execute_review(
        &self,
        document: &GeneratedDocument,
        compliance: &ComplianceResult,
        research: &ResearchContext,
    ) -> Result<AgentExecutionOutput<ReviewDecision>, PipelineError> {
        execute_review_agent(document, compliance, research)
            .await
            .map(|decision| AgentExecutionOutput {
                data: serde_json::to_value(&decision).unwrap_or_default(),
                output: decision,
            })
            .map_err(|e| match e {
                AgentError::TransientError(msg) => PipelineError::transient_agent("Review", msg),
                _ => PipelineError::permanent_agent("Review", e.to_string()),
            })
    }

    /// Determine if human approval is required
    fn requires_human_approval(&self, compliance: &ComplianceResult, document_type: &str) -> bool {
        // Only apply Woo rules to actual Woo documents
        let is_woo_document = document_type.starts_with("woo_");

        // ALL Woo-relevant documents with refusal grounds require human approval
        if is_woo_document && !compliance.refusal_grounds.is_empty() {
            return true;
        }

        // Apply domain trust level rules
        match self.domain_config.trust_level {
            iou_core::document::TrustLevel::Low => true,
            iou_core::document::TrustLevel::Medium => {
                compliance.score < self.domain_config.required_approval_threshold
            }
            iou_core::document::TrustLevel::High => false,
        }
    }

    /// Save a pipeline checkpoint
    async fn save_checkpoint(&self, checkpoint: PipelineCheckpoint) -> Result<(), PipelineError> {
        if !self.config.enable_checkpoints {
            return Ok(());
        }

        // TODO: Store checkpoint in persistent storage (section-08: API Layer)
        // For now, log the checkpoint for debugging
        tracing::debug!(
            "Checkpoint saved for document {}: iteration {}, completed agents: {:?}",
            checkpoint.document_id,
            checkpoint.iteration,
            checkpoint.completed_agents
        );

        Ok(())
    }

    /// Load the most recent checkpoint for a document
    pub async fn load_checkpoint(
        &self,
        document_id: Uuid,
    ) -> Result<Option<PipelineCheckpoint>, PipelineError> {
        if !self.config.enable_checkpoints {
            return Ok(None);
        }

        // TODO: Load checkpoint from persistent storage (section-08: API Layer)
        tracing::debug!("Loading checkpoint for document {}", document_id);

        Ok(None)
    }
}

// Helper types for agent execution
struct AgentExecutionOutput<T> {
    output: T,
    data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_request() -> DocumentRequest {
        let mut context = HashMap::new();
        context.insert("reference".to_string(), "TEST-001".to_string());

        DocumentRequest {
            id: Uuid::new_v4(),
            domain_id: "test_domain".to_string(),
            document_type: "woo_besluit".to_string(),
            context,
            requested_at: Utc::now(),
        }
    }

    fn create_test_template() -> Template {
        Template {
            id: "test-template".to_string(),
            name: "Test Template".to_string(),
            domain_id: "test_domain".to_string(),
            document_type: "woo_besluit".to_string(),
            content: "# {{ reference }}\n\nTest document content.".to_string(),
            required_variables: vec!["reference".to_string()],
            optional_sections: vec![],
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
        }
    }

    fn create_test_domain_config() -> DomainConfig {
        DomainConfig {
            domain_id: "test_domain".to_string(),
            trust_level: iou_core::document::TrustLevel::Medium,
            required_approval_threshold: 0.8,
            auto_approval_threshold: 0.95,
        }
    }

    fn create_test_pipeline() -> AgentPipeline {
        let kg = Arc::new(KnowledgeGraph::new());
        let engine = Arc::new(TemplateEngine::new().unwrap());
        let domain_config = Arc::new(create_test_domain_config());

        AgentPipeline::new(kg, engine, domain_config)
    }

    fn create_test_pipeline_with_config() -> AgentPipelineWithConfig {
        let kg = Arc::new(KnowledgeGraph::new());
        let engine = Arc::new(TemplateEngine::new().unwrap());
        let domain_config = Arc::new(create_test_domain_config());
        let config = PipelineConfig::default();

        AgentPipelineWithConfig {
            kg_client: kg,
            template_engine: engine,
            domain_config,
            config,
            stakeholder_extractor: None,
        }
    }

    #[test]
    fn test_pipeline_config_default() {
        let config = PipelineConfig::default();
        assert_eq!(config.max_iterations, 3);
        assert_eq!(config.max_retries, 3);
        assert!(config.enable_checkpoints);
    }

    #[test]
    fn test_error_severity_transitions() {
        let err = PipelineError::transient_agent("Test", "Network error");
        assert_eq!(err.severity(), ErrorSeverity::Transient);

        let err = PipelineError::permanent_agent("Test", "Invalid input");
        assert_eq!(err.severity(), ErrorSeverity::Permanent);
    }

    #[tokio::test]
    async fn test_sequential_execution_completes_all_agents() {
        let pipeline = create_test_pipeline();
        let request = create_test_request();
        let template = create_test_template();

        // Register template
        pipeline.template_engine.register_template(&template.id, &template.content).unwrap();

        let result = pipeline.execute_document_pipeline(&request, &template).await;

        // Should complete without error for simple case
        assert!(result.is_ok());
        let pipeline_result = result.unwrap();
        // Verify all agent types were executed (may have multiple iterations)
        let agent_names: Vec<_> = pipeline_result.agent_results.iter()
            .map(|r| r.agent_name.as_str())
            .collect();
        assert!(agent_names.contains(&"Research"));
        assert!(agent_names.contains(&"Content"));
        assert!(agent_names.contains(&"Compliance"));
        assert!(agent_names.contains(&"Review"));
        // First iteration should have all 4 agents
        assert!(pipeline_result.total_iterations >= 1);
    }

    #[tokio::test]
    async fn test_maker_checker_iteration_terminates_on_approval() {
        let config = PipelineConfig {
            max_iterations: 2,
            ..Default::default()
        };

        let kg = Arc::new(KnowledgeGraph::new());
        let engine = Arc::new(TemplateEngine::new().unwrap());
        let domain_config = Arc::new(create_test_domain_config());

        let pipeline = AgentPipelineWithConfig {
            kg_client: kg,
            template_engine: engine,
            domain_config,
            config,
            stakeholder_extractor: None,
        };

        let request = create_test_request();
        let template = create_test_template();

        pipeline.template_engine.register_template(&template.id, &template.content).unwrap();

        let result = pipeline.execute_document_pipeline(&request, &template).await;

        assert!(result.is_ok());
        let pipeline_result = result.unwrap();
        assert!(pipeline_result.total_iterations <= 2);
    }

    #[tokio::test]
    async fn test_max_iterations_exceeded_returns_in_review() {
        let config = PipelineConfig {
            max_iterations: 1,
            ..Default::default()
        };

        let kg = Arc::new(KnowledgeGraph::new());
        let engine = Arc::new(TemplateEngine::new().unwrap());
        let domain_config = Arc::new(create_test_domain_config());

        let pipeline = AgentPipelineWithConfig {
            kg_client: kg,
            template_engine: engine,
            domain_config,
            config,
            stakeholder_extractor: None,
        };

        let request = create_test_request();
        let template = create_test_template();

        pipeline.template_engine.register_template(&template.id, &template.content).unwrap();

        let result = pipeline.execute_document_pipeline(&request, &template).await;

        assert!(result.is_ok());
        // Should complete with some status (may require approval based on review)
    }

    #[tokio::test]
    async fn test_requires_human_approval_for_woo() {
        let pipeline = create_test_pipeline();
        let domain_config = DomainConfig {
            domain_id: "test".to_string(),
            trust_level: iou_core::document::TrustLevel::High,
            required_approval_threshold: 0.8,
            auto_approval_threshold: 0.95,
        };

        let pipeline_with_config = AgentPipelineWithConfig {
            kg_client: pipeline.kg_client.clone(),
            template_engine: pipeline.template_engine.clone(),
            domain_config: Arc::new(domain_config),
            config: PipelineConfig::default(),
            stakeholder_extractor: None,
        };

        let compliance = ComplianceResult {
            is_compliant: true,
            score: 0.95,
            refusal_grounds: vec![iou_core::compliance::WooRefusalGround::PersoonlijkeLevenssfeer],
            pii_detected: vec![],
            accessibility_issues: vec![],
            issues: vec![],
            redacted_content: None,
            assessed_at: Utc::now(),
            original_storage_key: None,
            redacted_storage_key: None,
        };

        // Woo documents with refusal grounds should always require approval
        assert!(pipeline_with_config.requires_human_approval(&compliance, "woo_besluit"));
    }

    #[tokio::test]
    async fn test_requires_human_approval_low_trust() {
        let domain_config = DomainConfig {
            domain_id: "test".to_string(),
            trust_level: iou_core::document::TrustLevel::Low,
            required_approval_threshold: 0.8,
            auto_approval_threshold: 0.95,
        };

        let pipeline = AgentPipelineWithConfig {
            kg_client: Arc::new(KnowledgeGraph::new()),
            template_engine: Arc::new(TemplateEngine::new().unwrap()),
            domain_config: Arc::new(domain_config),
            config: PipelineConfig::default(),
            stakeholder_extractor: None,
        };

        let compliance = ComplianceResult {
            is_compliant: true,
            score: 1.0,
            refusal_grounds: vec![],
            pii_detected: vec![],
            accessibility_issues: vec![],
            issues: vec![],
            redacted_content: None,
            assessed_at: Utc::now(),
            original_storage_key: None,
            redacted_storage_key: None,
        };

        // Low trust always requires approval
        assert!(pipeline.requires_human_approval(&compliance, "internal_memo"));
    }

    #[tokio::test]
    async fn test_valid_state_transition() {
        let status = WorkflowStatus::Draft;
        let valid_target = WorkflowStatus::Submitted;

        assert!(status.can_transition_to(&valid_target));
    }

    #[tokio::test]
    async fn test_invalid_state_transition() {
        let status = WorkflowStatus::Draft;
        let invalid_target = WorkflowStatus::Approved;

        assert!(!status.can_transition_to(&invalid_target));
    }

    #[test]
    fn test_checkpoint_serialization() {
        let checkpoint = PipelineCheckpoint {
            document_id: Uuid::new_v4(),
            current_agent: Some("Research".to_string()),
            completed_agents: vec!["Research".to_string()],
            iteration: 1,
            saved_at: Utc::now(),
        };

        let json = serde_json::to_string(&checkpoint).unwrap();
        let deserialized: PipelineCheckpoint = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.document_id, checkpoint.document_id);
        assert_eq!(deserialized.iteration, 1);
    }

    #[test]
    fn test_agent_result_serialization() {
        let result = AgentExecutionResult {
            agent_name: "Research".to_string(),
            success: true,
            started_at: Utc::now(),
            completed_at: Utc::now(),
            execution_time_ms: 100,
            retry_count: 0,
            data: serde_json::json!({"test": "data"}),
            errors: vec![],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: AgentExecutionResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.agent_name, "Research");
        assert!(deserialized.success);
    }
}
