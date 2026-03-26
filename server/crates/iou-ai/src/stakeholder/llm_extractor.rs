//! LLM-based entity extraction using Claude API
//!
//! This module provides Claude API integration for enhanced entity extraction.
//! It serves as the enhancement layer in the hybrid extraction system, processing
//! uncertain entities and enriching extracted information when baseline extraction
//! lacks confidence.

use anyhow::Result;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout as tokio_timeout;
use uuid::Uuid;

use crate::stakeholder::{
    extractor::ExtractionOptions,
    mention::{MentionRelationship, MentionType, TextPosition},
    types::{OrgType, PersonStakeholder, OrganizationStakeholder},
    ExtractionError,
};
use iou_core::graphrag::{Entity, EntityType};

/// Claude API client configuration
#[derive(Debug, Clone)]
pub struct ClaudeExtractorConfig {
    /// Anthropic API key
    pub api_key: String,

    /// Model identifier (default: claude-sonnet-4-20250514)
    pub model: String,

    /// Maximum retries for low-confidence results
    pub max_retries: usize,

    /// Request timeout
    pub timeout: Duration,

    /// Maximum cost per document (USD)
    pub max_cost_per_document: f32,

    /// Maximum LLM calls per document
    pub max_llm_calls: usize,
}

impl ClaudeExtractorConfig {
    /// Create config from ExtractionOptions
    pub fn from_options(options: &ExtractionOptions) -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY")
                .unwrap_or_else(|_| String::new()),
            model: std::env::var("CLAUDE_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
            max_retries: 2,
            timeout: options.llm_timeout,
            max_cost_per_document: options.max_cost_per_document,
            max_llm_calls: options.max_llm_calls,
        }
    }
}

impl Default for ClaudeExtractorConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY")
                .unwrap_or_else(|_| String::new()),
            model: std::env::var("CLAUDE_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
            max_retries: 2,
            timeout: Duration::from_secs(10),
            max_cost_per_document: 0.10,
            max_llm_calls: 10,
        }
    }
}

/// Token usage and cost tracking
#[derive(Debug, Clone, Default)]
pub struct CostTracker {
    /// Input tokens consumed
    pub input_tokens: u32,

    /// Output tokens consumed
    pub output_tokens: u32,

    /// Total estimated cost in USD
    pub total_cost_usd: f32,

    /// Number of API calls made
    pub api_calls: usize,
}

impl CostTracker {
    /// Calculate cost based on Claude Sonnet 4 pricing
    /// Input: $3/1M tokens, Output: $15/1M tokens
    pub fn calculate_cost(&mut self, input: u32, output: u32) -> f32 {
        self.input_tokens += input;
        self.output_tokens += output;
        self.api_calls += 1;

        let input_cost = (input as f32 / 1_000_000.0) * 3.0;
        let output_cost = (output as f32 / 1_000_000.0) * 15.0;
        self.total_cost_usd = input_cost + output_cost;

        self.total_cost_usd
    }

    /// Check if budget has been exceeded
    pub fn exceeds_budget(&self, max_cost: f32) -> bool {
        self.total_cost_usd > max_cost
    }

    /// Check if call limit has been reached
    pub fn exceeds_call_limit(&self, max_calls: usize) -> bool {
        self.api_calls >= max_calls
    }

    /// Get total tokens consumed
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

/// Context provided to LLM for entity extraction
#[derive(Debug, Clone)]
pub struct ExtractionContext {
    /// Document ID being processed
    pub document_id: Uuid,

    /// Partial results from baseline extraction
    pub baseline_entities: Vec<Entity>,

    /// Specific text segments requiring LLM analysis
    pub focus_segments: Vec<TextSegment>,

    /// Confidence threshold from baseline
    pub confidence_threshold: f32,
}

/// Text segment requiring LLM analysis
#[derive(Debug, Clone)]
pub struct TextSegment {
    pub text: String,
    pub position: TextPosition,
    pub reason: FocusReason,
}

/// Reason why a segment needs LLM analysis
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusReason {
    LowConfidence(f32),
    MissingContext,
    AmbiguousMatch,
}

/// Errors specific to LLM extraction
#[derive(Debug, thiserror::Error)]
pub enum LlmExtractionError {
    #[error("API authentication failed: {0}")]
    Authentication(String),

    #[error("API request timed out after {0:?}")]
    Timeout(Duration),

    #[error("Cost budget exceeded: ${0:.4} > ${1:.4}")]
    BudgetExceeded(f32, f32),

    #[error("Call limit exceeded: {0} >= {1}")]
    CallLimitExceeded(usize, usize),

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Retry limit reached for low-confidence results")]
    RetryLimitReached,

    #[error("HTTP client error: {0}")]
    HttpClient(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("No API key configured")]
    NoApiKey,
}

impl From<reqwest::Error> for LlmExtractionError {
    fn from(e: reqwest::Error) -> Self {
        LlmExtractionError::HttpClient(e.to_string())
    }
}

/// Claude API integration for entity extraction
pub struct ClaudeExtractor {
    config: ClaudeExtractorConfig,
    client: Arc<Client>,
    cost_tracker: Arc<tokio::sync::RwLock<CostTracker>>,
}

impl ClaudeExtractor {
    /// Create a new Claude extractor with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ClaudeExtractorConfig::default())
    }

    /// Create a new Claude extractor with custom configuration
    pub fn with_config(config: ClaudeExtractorConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(anyhow::anyhow!("ANTHROPIC_API_KEY must be set").into());
        }

        Ok(Self {
            config,
            client: Arc::new(
                Client::builder()
                    .timeout(Duration::from_secs(30))
                    .build()?,
            ),
            cost_tracker: Arc::new(tokio::sync::RwLock::new(CostTracker::default())),
        })
    }

    /// Create extractor from ExtractionOptions
    pub fn from_options(options: &ExtractionOptions) -> Result<Self> {
        Self::with_config(ClaudeExtractorConfig::from_options(options))
    }

    /// Extract entities from text using Claude tool calling
    ///
    /// This method sends the text to Claude with the entity extraction tool
    /// schema, ensuring structured output that can be parsed into Entity types.
    pub async fn extract_entities(
        &self,
        text: &str,
        context: &ExtractionContext,
    ) -> Result<Vec<Entity>, LlmExtractionError> {
        // Check budgets before proceeding
        {
            let tracker = self.cost_tracker.read().await;
            if tracker.exceeds_budget(self.config.max_cost_per_document) {
                return Err(LlmExtractionError::BudgetExceeded(
                    tracker.total_cost_usd,
                    self.config.max_cost_per_document,
                ));
            }
            if tracker.exceeds_call_limit(self.config.max_llm_calls) {
                return Err(LlmExtractionError::CallLimitExceeded(
                    tracker.api_calls,
                    self.config.max_llm_calls,
                ));
            }
        }

        // Build request with tool schema
        let request = self.build_extraction_request(text, context)?;

        // Execute with timeout
        let response = tokio_timeout(
            self.config.timeout,
            self.execute_request(&request),
        )
        .await
        .map_err(|_| LlmExtractionError::Timeout(self.config.timeout))?
        ?;

        // Parse tool use results
        let mut entities = self.parse_tool_results(response, context)?;

        // Retry if average confidence is low
        if self.needs_retry(&entities, context) {
            entities = self.retry_with_alternative_prompt(text, context).await?;
        }

        Ok(entities)
    }

    /// Extract entities with simplified interface (no context)
    pub async fn extract_from_text(
        &self,
        text: &str,
    ) -> Result<Vec<Entity>, LlmExtractionError> {
        let context = ExtractionContext {
            document_id: Uuid::new_v4(),
            baseline_entities: Vec::new(),
            focus_segments: Vec::new(),
            confidence_threshold: 0.7,
        };
        self.extract_entities(text, &context).await
    }

    /// Build the Claude API request with tool schema
    fn build_extraction_request(
        &self,
        text: &str,
        _context: &ExtractionContext,
    ) -> Result<ClaudeRequest, LlmExtractionError> {
        let system_prompt = r#"Je bent een expert in het extraheren van entiteiten uit Nederlandse overheidsdocumenten.

Je taak is om personen en organisaties te identificeren en te extraheren uit de tekst. Let op:
- Nederlandse namen met tussenvoegsels (van, van der, de, etc.)
- Afkortingen van ministeries (MinFin, BZK, etc.)
- Titels en rollen van personen
- Organisatietypes (ministeries, agentschappen, gemeenten)

Geef alleen entiteiten terug waar je zeker van bent, met een realistische betrouwbaarheidsScore tussen 0.5 en 1.0."#;

        Ok(ClaudeRequest {
            model: self.config.model.clone(),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: text.to_string(),
            }],
            system: Some(system_prompt.to_string()),
            tools: vec![extract_entities_tool()],
            max_tokens: 4096,
        })
    }

    /// Execute the HTTP request to Claude API
    async fn execute_request(
        &self,
        request: &ClaudeRequest,
    ) -> Result<ClaudeResponse, LlmExtractionError> {
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| LlmExtractionError::HttpClient(e.to_string()))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| LlmExtractionError::HttpClient(e.to_string()))?;

        if !status.is_success() {
            if status.as_u16() == 401 {
                return Err(LlmExtractionError::Authentication(
                    "Invalid API key".to_string(),
                ));
            }
            return Err(LlmExtractionError::ApiError {
                status: status.as_u16(),
                message: response_text,
            });
        }

        let api_response: ClaudeResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                LlmExtractionError::InvalidResponse(format!("Parse error: {}", e))
            })?;

        // Track cost
        if let Some(usage) = &api_response.usage {
            let mut tracker = self.cost_tracker.write().await;
            tracker.calculate_cost(usage.input_tokens, usage.output_tokens);
        }

        Ok(api_response)
    }

    /// Parse tool use results into Entity types
    fn parse_tool_results(
        &self,
        response: ClaudeResponse,
        context: &ExtractionContext,
    ) -> Result<Vec<Entity>, LlmExtractionError> {
        let mut entities = Vec::new();

        for block in &response.content {
            if block.r#type == "tool_use" {
                if let Some(tool_input) = &block.input {
                    // Parse persons
                    if let Some(persons) = tool_input.get("persons").and_then(|v| v.as_array()) {
                        for person_value in persons {
                            if let Ok(person_data) =
                                serde_json::from_value::<PersonData>(person_value.clone())
                            {
                                let mut person_stakeholder =
                                    PersonStakeholder::new(person_data.name, person_data.confidence);

                                if let Some(title) = person_data.title {
                                    person_stakeholder = person_stakeholder.with_title(title);
                                }
                                if let Some(role) = person_data.role {
                                    person_stakeholder = person_stakeholder.with_role(role);
                                }
                                if let Some(department) = person_data.department {
                                    person_stakeholder = person_stakeholder.with_department(department);
                                }

                                entities.push(person_stakeholder.into_entity());
                            }
                        }
                    }

                    // Parse organizations
                    if let Some(orgs) = tool_input.get("organizations").and_then(|v| v.as_array()) {
                        for org_value in orgs {
                            if let Ok(org_data) =
                                serde_json::from_value::<OrganizationData>(org_value.clone())
                            {
                                let mut org_stakeholder = OrganizationStakeholder::new(
                                    org_data.name.clone(),
                                    org_data.confidence,
                                );

                                if let Some(short_name) = org_data.short_name {
                                    org_stakeholder = org_stakeholder.with_short_name(short_name);
                                }

                                // Parse org_type
                                if let Some(org_type_str) = org_data.org_type {
                                    let org_type = match org_type_str.to_lowercase().as_str() {
                                        "ministry" | "ministerie" => OrgType::Ministry,
                                        "agency" | "dienst" | "agentschap" => OrgType::Agency,
                                        "municipal" | "gemeente" => OrgType::Municipal,
                                        _ => OrgType::Other,
                                    };
                                    org_stakeholder = org_stakeholder.with_org_type(org_type);
                                }

                                entities.push(org_stakeholder.into_entity());
                            }
                        }
                    }
                }
            }
        }

        Ok(entities)
    }

    /// Calculate confidence from logprobs (simplified - Claude doesn't provide logprobs in standard API)
    fn calculate_confidence(&self, _logprobs: &[f32]) -> f32 {
        // Since Claude doesn't provide logprobs in the standard API,
        // we rely on the confidence score returned by the model in the tool output
        0.8 // Default fallback
    }

    /// Check if retry is needed due to low confidence
    fn needs_retry(&self, entities: &[Entity], context: &ExtractionContext) -> bool {
        if entities.is_empty() {
            return false;
        }

        let avg_confidence: f32 = entities
            .iter()
            .map(|e| e.confidence)
            .sum::<f32>()
            / entities.len() as f32;

        avg_confidence < context.confidence_threshold && entities.len() < self.config.max_retries
    }

    /// Retry extraction with an alternative prompt strategy
    async fn retry_with_alternative_prompt(
        &self,
        text: &str,
        context: &ExtractionContext,
    ) -> Result<Vec<Entity>, LlmExtractionError> {
        // For now, return empty results on retry
        // In a full implementation, this would use a different prompt emphasizing precision
        tracing::debug!("Retrying extraction with alternative prompt");
        Ok(Vec::new())
    }

    /// Get current cost tracker state
    pub async fn get_cost_tracker(&self) -> CostTracker {
        self.cost_tracker.read().await.clone()
    }

    /// Reset cost tracker (useful between documents)
    pub async fn reset_cost_tracker(&self) {
        let mut tracker = self.cost_tracker.write().await;
        *tracker = CostTracker::default();
    }
}

/// Check if extraction is needed based on confidence
pub fn should_extract_with_llm(confidence: f32, threshold: f32) -> bool {
    confidence < threshold
}

// ===== API Types =====

/// Tool definition for Claude entity extraction
fn extract_entities_tool() -> Tool {
    Tool {
        name: "extract_entities".to_string(),
        description: "Extract person and organization entities from Dutch government text"
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "persons": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "title": {"type": "string"},
                            "role": {"type": "string"},
                            "department": {"type": "string"},
                            "confidence": {"type": "number"}
                        },
                        "required": ["name", "confidence"]
                    }
                },
                "organizations": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "short_name": {"type": "string"},
                            "org_type": {"type": "string"},
                            "confidence": {"type": "number"}
                        },
                        "required": ["name", "confidence"]
                    }
                }
            }
        }),
    }
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    tools: Vec<Tool>,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct Tool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    id: String,
    content: Vec<ContentBlock>,
    usage: Option<Usage>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    r#type: String,
    #[serde(default)]
    input: Option<serde_json::Value>,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct PersonData {
    name: String,
    title: Option<String>,
    role: Option<String>,
    department: Option<String>,
    confidence: f32,
}

#[derive(Debug, Deserialize)]
struct OrganizationData {
    name: String,
    short_name: Option<String>,
    org_type: Option<String>,
    confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_extract_with_llm_threshold() {
        assert!(should_extract_with_llm(0.5, 0.7));
        assert!(!should_extract_with_llm(0.8, 0.7));
        assert!(!should_extract_with_llm(0.7, 0.7));
    }

    #[test]
    fn test_should_extract_with_llm_boundary() {
        assert!(should_extract_with_llm(0.69, 0.7));
        assert!(!should_extract_with_llm(0.71, 0.7));
    }

    #[test]
    fn test_cost_tracker_calculates_correctly() {
        let mut tracker = CostTracker::default();

        // 1000 input tokens at $3/1M = $0.003
        // 500 output tokens at $15/1M = $0.0075
        let cost = tracker.calculate_cost(1000, 500);

        assert_eq!(tracker.input_tokens, 1000);
        assert_eq!(tracker.output_tokens, 500);
        assert_eq!(tracker.api_calls, 1);
        assert!((cost - 0.0105).abs() < 0.0001);
    }

    #[test]
    fn test_cost_tracker_exceeds_budget() {
        let mut tracker = CostTracker::default();

        tracker.calculate_cost(100_000, 50_000); // ~$1.05

        assert!(tracker.exceeds_budget(1.0));
        assert!(!tracker.exceeds_budget(2.0));
    }

    #[test]
    fn test_cost_tracker_call_limit() {
        let mut tracker = CostTracker::default();

        for _ in 0..5 {
            tracker.calculate_cost(100, 50);
        }

        assert!(tracker.exceeds_call_limit(4));
        assert!(!tracker.exceeds_call_limit(6));
    }

    #[test]
    fn test_cost_tracker_total_tokens() {
        let mut tracker = CostTracker::default();

        tracker.calculate_cost(1000, 500);

        assert_eq!(tracker.total_tokens(), 1500);
    }

    #[test]
    fn test_claude_config_default() {
        let config = ClaudeExtractorConfig::default();

        assert_eq!(config.max_retries, 2);
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.max_cost_per_document, 0.10);
        assert_eq!(config.max_llm_calls, 10);
    }

    #[test]
    fn test_focus_reason_partial_eq() {
        assert_eq!(
            FocusReason::LowConfidence(0.5),
            FocusReason::LowConfidence(0.5)
        );
        assert_eq!(FocusReason::MissingContext, FocusReason::MissingContext);
        // assert_ne doesn't work without Eq, use PartialEq directly
        assert!(
            FocusReason::LowConfidence(0.5) != FocusReason::LowConfidence(0.6)
        );
    }

    #[tokio::test]
    async fn test_extractor_requires_api_key() {
        let config = ClaudeExtractorConfig {
            api_key: String::new(),
            ..Default::default()
        };

        let result = ClaudeExtractor::with_config(config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extractor_accepts_valid_api_key() {
        let config = ClaudeExtractorConfig {
            api_key: "sk-ant-test-key".to_string(),
            ..Default::default()
        };

        let result = ClaudeExtractor::with_config(config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reset_cost_tracker() {
        let extractor = ClaudeExtractor::with_config(ClaudeExtractorConfig {
            api_key: "sk-ant-test-key".to_string(),
            ..Default::default()
        })
        .unwrap();

        // Simulate some cost
        {
            let mut tracker = extractor.cost_tracker.write().await;
            tracker.calculate_cost(1000, 500);
        }

        // Verify cost was recorded
        let tracker = extractor.get_cost_tracker().await;
        assert_eq!(tracker.input_tokens, 1000);

        // Reset
        extractor.reset_cost_tracker().await;

        // Verify reset
        let tracker = extractor.get_cost_tracker().await;
        assert_eq!(tracker.input_tokens, 0);
        assert_eq!(tracker.output_tokens, 0);
        assert_eq!(tracker.api_calls, 0);
    }

    #[test]
    fn test_llm_extraction_error_display() {
        let err = LlmExtractionError::Timeout(Duration::from_secs(10));
        assert!(err.to_string().contains("timed out"));

        let err = LlmExtractionError::BudgetExceeded(0.15, 0.10);
        assert!(err.to_string().contains("0.1500"));
        assert!(err.to_string().contains("0.1000"));

        let err = LlmExtractionError::CallLimitExceeded(11, 10);
        assert!(err.to_string().contains("11 >= 10"));

        let err = LlmExtractionError::NoApiKey;
        assert!(err.to_string().contains("API key"));
    }
}
