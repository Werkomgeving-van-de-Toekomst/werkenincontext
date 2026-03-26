Now I'll generate the section content. Based on my analysis:

1. **Dependencies**: Section 01 (Foundation & Types) - provides core data structures like Entity, ExtractionResult, ExtractionOptions, and error types

2. **Tests**: From the TDD plan, Section 04 has 8 test stubs covering API responses, tool calling, confidence scoring, retry logic, timeout, cost tracking, and budget controls

3. **Implementation details**: From the main plan, Section 04 involves creating llm_extractor.rs with Claude API integration, tool schema definition, extract_entities method with confidence scoring from logprobs, timeout handling, and cost tracking

Here's the self-contained section content:</think># Section 04: LLM Extractor

## Overview

This section implements the Claude API integration for entity extraction. The LLM extractor serves as the enhancement layer in the hybrid extraction system, processing uncertain entities and enriching extracted information when baseline extraction lacks confidence.

### Purpose

The LLM extractor is invoked when:
- Baseline extraction confidence is below 0.7
- Missing contextual information (title, role, department)
- Ambiguous entity detection requiring semantic understanding

### Design Goals

| Goal | Description |
|------|-------------|
| Structured Output | Use Claude tool calling for guaranteed JSON schema compliance |
| Confidence Tracking | Calculate confidence scores from API logprobs |
| Cost Control | Track token usage and enforce per-document budgets |
| Fault Tolerance | Timeout handling prevents hanging on API failures |
| Retry Logic | Low-confidence results trigger alternative prompts |

---

## Dependencies

This section depends on **Section 01: Foundation & Types**, which provides:

- `Entity` type from `iou_core::graphrag`
- `ExtractionResult` and `ExtractionStats` types
- `ExtractionOptions` configuration
- `ExtractionError` error type
- `PersonStakeholder` and `OrganizationStakeholder` convenience wrappers

These types are defined in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/mod.rs` and related modules.

---

## File Structure

```
crates/iou-ai/src/stakeholder/
├── llm_extractor.rs         # Main implementation (NEW)
└── mod.rs                   # Public API exports (MODIFY)
```

---

## Data Structures

### Claude Tool Schema

The tool schema defines the structured output format for entity extraction:

```rust
/// Tool definition for Claude entity extraction
const ENTITY_EXTRACTION_TOOL: &str = r#"{
    "name": "extract_entities",
    "description": "Extract person and organization entities from Dutch government text",
    "input_schema": {
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
    }
}"#;
```

### LLM Extractor Configuration

```rust
/// Claude API client configuration
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

impl Default for ClaudeExtractorConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY")
                .expect("ANTHROPIC_API_KEY must be set"),
            model: std::env::var("CLAUDE_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
            max_retries: 2,
            timeout: Duration::from_secs(10),
            max_cost_per_document: 0.10,
            max_llm_calls: 10,
        }
    }
}
```

### Cost Tracking

```rust
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
    /// Calculate cost based on Claude Sonnet 4.5 pricing
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
}
```

### Extraction Context

```rust
/// Context provided to LLM for entity extraction
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
pub struct TextSegment {
    pub text: String,
    pub position: TextPosition,
    pub reason: FocusReason,
}

pub enum FocusReason {
    LowConfidence(f32),
    MissingContext,
    AmbiguousMatch,
}
```

### Error Types

```rust
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
}
```

---

## Implementation

### Main Extractor Structure

File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/llm_extractor.rs`

```rust
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

use crate::stakeholder::{
    ExtractionContext, ExtractionResult, ExtractionOptions,
    Entity, PersonStakeholder, OrganizationStakeholder,
    ExtractionError,
};

/// Claude API integration for entity extraction
pub struct ClaudeExtractor {
    config: ClaudeExtractorConfig,
    client: Arc<Client>,
    cost_tracker: Arc<std::sync::RwLock<CostTracker>>,
}

impl ClaudeExtractor {
    /// Create a new Claude extractor with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ClaudeExtractorConfig::default())
    }
    
    /// Create a new Claude extractor with custom configuration
    pub fn with_config(config: ClaudeExtractorConfig) -> Result<Self> {
        Ok(Self {
            config,
            client: Arc::new(Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?),
            cost_tracker: Arc::new(std::sync::RwLock::new(CostTracker::default())),
        })
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
            let tracker = self.cost_tracker.read()
                .map_err(|_| LlmExtractionError::Internal("Lock poisoned".into()))?;
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
        let response = timeout(
            self.config.timeout,
            self.execute_request(request)
        )
        .await
        .map_err(|_| LlmExtractionError::Timeout(self.config.timeout))??;
        
        // Parse tool use results
        let mut entities = self.parse_tool_results(response)?;
        
        // Retry if average confidence is low
        if self.needs_retry(&entities, context) {
            entities = self.retry_with_alternative_prompt(text, context).await?;
        }
        
        Ok(entities)
    }
    
    /// Build the Claude API request with tool schema
    fn build_extraction_request(
        &self,
        text: &str,
        context: &ExtractionContext,
    ) -> Result<ClaudeRequest, LlmExtractionError> {
        // Implementation details...
        todo!("Build request with system prompt, user message, and tool schema")
    }
    
    /// Execute the HTTP request to Claude API
    async fn execute_request(
        &self,
        request: ClaudeRequest,
    ) -> Result<ClaudeResponse, LlmExtractionError> {
        // Implementation details...
        todo!("Send POST to Anthropic API and parse response")
    }
    
    /// Parse tool use results into Entity types
    fn parse_tool_results(
        &self,
        response: ClaudeResponse,
    ) -> Result<Vec<Entity>, LlmExtractionError> {
        // Implementation details...
        todo!("Extract tool_use blocks and deserialize JSON")
    }
    
    /// Calculate confidence from logprobs
    fn calculate_confidence(&self, logprobs: &[f32]) -> f32 {
        // Aggregate logprobs into a 0-1 confidence score
        // Typical approach: exp(mean(logprobs))
        logprobs.iter()
            .map(|lp| lp.exp())
            .sum::<f32>() / logprobs.len() as f32
    }
    
    /// Check if retry is needed due to low confidence
    fn needs_retry(&self, entities: &[Entity], context: &ExtractionContext) -> bool {
        if entities.is_empty() {
            return false;
        }
        
        let avg_confidence: f32 = entities.iter()
            .filter_map(|e| e.metadata.get("confidence"))
            .filter_map(|v| serde_json::from_value::<f32>(v.clone()).ok())
            .sum::<f32>() / entities.len() as f32;
        
        avg_confidence < context.confidence_threshold
    }
    
    /// Retry extraction with an alternative prompt strategy
    async fn retry_with_alternative_prompt(
        &self,
        text: &str,
        context: &ExtractionContext,
    ) -> Result<Vec<Entity>, LlmExtractionError> {
        // Implementation details...
        todo!("Use different prompt emphasizing precision")
    }
    
    /// Get current cost tracker state
    pub fn get_cost_tracker(&self) -> CostTracker {
        self.cost_tracker.read()
            .map(|t| t.clone())
            .unwrap_or_default()
    }
    
    /// Reset cost tracker (useful between documents)
    pub fn reset_cost_tracker(&self) {
        if let Ok(mut tracker) = self.cost_tracker.write() {
            *tracker = CostTracker::default();
        }
    }
}

/// Check if extraction is needed based on confidence
pub fn should_extract_with_llm(confidence: f32, threshold: f32) -> bool {
    confidence < threshold
}
```

---

## Tests

### Test Stubs

File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/llm_extractor_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Mock, Server};
    use serde_json::json;

    fn create_mock_extractor(base_url: &str) -> ClaudeExtractor {
        let mut config = ClaudeExtractorConfig::default();
        // Override with mock server URL
        ClaudeExtractor::with_config(config).unwrap()
    }

    #[tokio::test]
    async fn test_claude_api_returns_structured_entity_list() {
        // Test that Claude API call returns structured Entity list
        // Setup: Mock Claude API response with tool_use block
        // Expected: Vec<Entity> with correct types and metadata
        todo!("Assert entities parsed from tool_use response")
    }

    #[tokio::test]
    async fn test_tool_calling_produces_valid_json() {
        // Test that tool calling produces valid JSON for entities
        // Setup: Mock response with various entity structures
        // Expected: All entities deserialize correctly
        todo!("Assert JSON schema compliance")
    }

    #[tokio::test]
    async fn test_confidence_from_logprobs_in_valid_range() {
        // Test confidence score calculated from logprobs is between 0.0 and 1.0
        // Setup: Mock response with various logprob values
        // Expected: Confidence always in [0.0, 1.0]
        todo!("Assert 0.0 <= confidence <= 1.0")
    }

    #[tokio::test]
    async fn test_low_confidence_triggers_retry() {
        // Test low-confidence entities trigger retry with different prompt
        // Setup: First response has low avg confidence, second has high
        // Expected: Two API calls made, final result has high confidence
        todo!("Assert retry on low confidence")
    }

    #[tokio::test]
    async fn test_timeout_prevents_hanging() {
        // Test timeout prevents hanging on API failure
        // Setup: Mock server that doesn't respond
        // Expected: Timeout error after configured duration
        todo!("Assert timeout error returned")
    }

    #[tokio::test]
    async fn test_cost_tracking_returns_token_count_and_cost() {
        // Test cost tracking returns token count and estimated cost
        // Setup: Mock response with known token counts
        // Expected: CostTracker has correct values
        todo!("Assert token counts and cost calculated")
    }

    #[tokio::test]
    async fn test_max_llm_calls_limit_prevents_runaway() {
        // Test max_llm_calls limit prevents runaway API usage
        // Setup: Configure limit of 3, trigger 4 potential calls
        // Expected: 4th call returns CallLimitExceeded error
        todo!("Assert error after max calls")
    }

    #[tokio::test]
    async fn test_max_cost_limit_prevents_overruns() {
        // Test max_cost_per_document limit prevents cost overruns
        // Setup: Configure $0.05 limit, mock responses exceeding it
        // Expected: BudgetExceeded error returned
        todo!("Assert error when budget exceeded")
    }

    #[tokio::test]
    async fn test_api_key_failure_returns_appropriate_error() {
        // Test API key authentication failure returns appropriate error
        // Setup: Mock 401 response from API
        // Expected: Authentication error variant
        todo!("Assert Authentication error on 401")
    }

    #[tokio::test]
    fn test_should_extract_with_llm_threshold() {
        // Test the utility function for LLM extraction decision
        assert!(should_extract_with_llm(0.5, 0.7));
        assert!(!should_extract_with_llm(0.8, 0.7));
        assert!(!should_extract_with_llm(0.7, 0.7));
    }
}
```

---

## Configuration

### Environment Variables

The following environment variables should be configured:

```bash
# Required
ANTHROPIC_API_KEY=sk-ant-xxxxx

# Optional (with defaults)
CLAUDE_MODEL=claude-sonnet-4-20250514
EXTRACTION_MAX_LLM_CALLS=10
EXTRACTION_MAX_COST_PER_DOCUMENT=0.10
EXTRACTION_LLM_TIMEOUT=10  # seconds
```

### Integration with ExtractionOptions

The `ExtractionOptions` type (from Section 01) already contains LLM-related fields:

```rust
pub struct ExtractionOptions {
    // ... existing fields ...
    
    /// Maximum LLM calls per document (cost control)
    pub max_llm_calls: usize,
    
    /// Maximum cost per document in USD (cost control)
    pub max_cost_per_document: f32,
    
    /// Timeout for LLM API calls
    pub llm_timeout: Duration,
}
```

The `ClaudeExtractor` should be initialized with values from `ExtractionOptions`:

```rust
impl ClaudeExtractor {
    pub fn from_options(options: &ExtractionOptions) -> Result<Self> {
        let config = ClaudeExtractorConfig {
            max_llm_calls: options.max_llm_calls,
            max_cost_per_document: options.max_cost_per_document,
            timeout: options.llm_timeout,
            ..Default::default()
        };
        Self::with_config(config)
    }
}
```

---

## Success Criteria

Implementation is complete when:

| Criterion | Test/Verification |
|-----------|-------------------|
| Claude API returns structured entities | `test_claude_api_returns_structured_entity_list` |
| Tool calling produces valid JSON | `test_tool_calling_produces_valid_json` |
| Confidence scores from logprobs in [0, 1] | `test_confidence_from_logprobs_in_valid_range` |
| Low confidence triggers retry | `test_low_confidence_triggers_retry` |
| Timeout prevents hanging | `test_timeout_prevents_hanging` |
| Cost tracking accurate | `test_cost_tracking_returns_token_count_and_cost` |
| max_llm_calls enforced | `test_max_llm_calls_limit_prevents_runaway` |
| max_cost_per_document enforced | `test_max_cost_limit_prevents_overruns` |
| Auth failures handled | `test_api_key_failure_returns_appropriate_error` |

---

## Next Steps

After completing this section:

1. **Section 05: Normalization & Deduplication** - Integrate this extractor's output with entity normalization and deduplication logic
2. **Section 06: Main Extractor Implementation** - Combine baseline and LLM extraction into the unified pipeline

---

*Section 04 generated: 2026-03-16*