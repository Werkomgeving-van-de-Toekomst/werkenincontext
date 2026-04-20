// =============================================================================
// Nebul API Integration
// =============================================================================
//
// Nebul provides OpenAI-compatible API access to multiple LLM models.
// Base URL: https://api.inference.nebul.io/v1
//
// Docs: https://docs.nebul.io/docs/inference-api/getting-started/quick-start

use crate::{
    InferenceConfig, InferenceRequest, ContextInferenceResult, InferenceError, InferenceResult,
    InferredContext, CoreContext, DomainInference, SemanticContext, LegalContext,
    InferredGrondslag, Entity, EntityType, ConfidenceScores, FieldConfidence,
    InferenceMetadata, ContextTypeRequest,
};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Nebul API client
#[derive(Debug, Clone)]
pub struct NebulClient {
    config: InferenceConfig,
    http_client: reqwest::Client,
}

impl NebulClient {
    /// Create a new Nebul client
    pub fn new(config: InferenceConfig) -> Result<Self, InferenceError> {
        if config.nebul_api_key.is_empty() {
            return Err(InferenceError::Config("NEBUL_API_KEY not set".to_string()));
        }

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| InferenceError::Config(format!("HTTP client error: {e}")))?;

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Create with default config from environment
    pub fn from_env() -> Result<Self, InferenceError> {
        Self::new(InferenceConfig::default())
    }

    /// Get the base URL for API requests
    fn base_url(&self) -> &str {
        &self.config.nebul_endpoint
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<NebulModelInfo>, InferenceError> {
        let response = self.http_client
            .get(format!("{}/models", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.config.nebul_api_key))
            .send()
            .await
            .map_err(|e| InferenceError::Api(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(InferenceError::Api(format!("{}: {}", status, body)));
        }

        let models_response: ModelsListResponse = response
            .json()
            .await
            .map_err(|e| InferenceError::Api(format!("JSON parse error: {e}")))?;

        Ok(models_response.data)
    }

    /// Perform context inference on text
    pub async fn infer_context(&self, request: InferenceRequest) -> Result<ContextInferenceResult, InferenceError> {
        let start_time = std::time::Instant::now();
        let text_length = request.text.len();

        // Validate text length
        if text_length > self.config.max_text_length {
            return Err(InferenceError::TextTooLong(text_length, self.config.max_text_length));
        }

        // Build the system prompt for context extraction
        let system_prompt = self.build_system_prompt(&request);

        // Build the user message with the text to analyze
        let user_message = self.build_user_message(&request);

        // Call Nebul API
        let api_response = self.call_chat_completion(&system_prompt, &user_message).await?;

        // Parse the response
        let (context, confidence) = self.parse_inference_response(&api_response, &request)?;

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        // Determine if review is needed
        let requires_review = confidence.overall < self.config.min_confidence
            || (request.options.include_legal && self.config.review_legal_context);
        let review_reason = if requires_review {
            if confidence.overall < self.config.min_confidence {
                Some("Low confidence score".to_string())
            } else {
                Some("Legal context requires review".to_string())
            }
        } else {
            None
        };

        Ok(ContextInferenceResult {
            context,
            confidence,
            metadata: InferenceMetadata {
                model: self.config.model.as_str().to_string(),
                provider: self.config.model.provider().as_str().to_string(),
                inferred_at: Utc::now(),
                processing_time_ms,
                text_length,
                requires_review,
                review_reason,
            },
        })
    }

    fn build_system_prompt(&self, request: &InferenceRequest) -> String {
        let mut prompt = r#"You are a specialized AI assistant for Dutch government context extraction.
Your task is to analyze documents and extract structured contextual metadata.

Extract the following types of information:
"#.to_string();

        for context_type in &request.options.context_types {
            match context_type {
                ContextTypeRequest::Core => {
                    prompt.push_str("- Core: title, creator, classification, description\n");
                }
                ContextTypeRequest::Domain => {
                    prompt.push_str("- Domain: zaak/project/beleid identifiers and types\n");
                }
                ContextTypeRequest::Semantic => {
                    prompt.push_str("- Semantic: keywords (trefwoorden), subjects (onderwerpen), summary\n");
                }
                ContextTypeRequest::Provenance => {
                    prompt.push_str("- Provenance: creation date, author, source system\n");
                }
            }
        }

        if request.options.include_entities {
            prompt.push_str("\nNamed Entities: persons (PERSON), organizations (ORG), locations (LOC), laws (LAW), events (EVENT)\n");
        }

        if request.options.include_legal {
            prompt.push_str("\nLegal References: grondslagen with BWBR identifiers, source (wet), article references\n");
        }

        prompt.push_str(
            r#"

Respond ONLY with valid JSON. No markdown, no explanations, no code blocks.

Format:
{
  "core": {
    "title": "...",
    "creator": "...",
    "classification": "...",
    "description": "..."
  },
  "domain": {
    "type": "zaak|project|beleid",
    "id": "...",
    "sub_type": "...",
    "fase": "..."
  },
  "semantic": {
    "trefwoorden": ["...", "..."],
    "onderwerpen": ["...", "..."],
    "samenvatting": "..."
  },
  "legal": {
    "grondslagen": [
      {
        "grondslag_id": "BWBR...",
        "bron": "...",
        "artikel": "...",
        "confidence": 0.0-1.0
      }
    ]
  },
  "entities": [
    {
      "entity_type": "PERSOON|ORGANISATIE|LOCATIE|WET|EVENEMENT",
      "text": "...",
      "start_offset": 0,
      "end_offset": 0,
      "confidence": 0.0-1.0,
      "metadata": {}
    }
  ],
  "confidence": {
    "overall": 0.0-1.0,
    "by_field": [
      {"field": "...", "confidence": 0.0-1.0, "requires_review": false}
    ]
  }
}

For null/missing values, use null. For empty arrays, use [].
Confidence values must be between 0.0 and 1.0.
"#,
        );

        prompt
    }

    fn build_user_message(&self, request: &InferenceRequest) -> String {
        format!(
            r#"Analyze the following document and extract contextual metadata.

Object Type: {}
Organization ID: {}

Document Text:
---

{}"#,
            request.object_type, request.organisatie_id, request.text
        )
    }

    async fn call_chat_completion(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<ChatCompletionResponse, InferenceError> {
        let request_body = ChatCompletionRequest {
            model: self.config.model.as_str().to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
            temperature: 0.1,  // Low temperature for consistent JSON output
            max_tokens: 4000,
            response_format: Some(ResponseFormat {
                type_: "json_object".to_string(),
            }),
        };

        let response = self.http_client
            .post(format!("{}/chat/completions", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.config.nebul_api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| InferenceError::Api(format!("Request failed: {e}")))?;

        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(InferenceError::RateLimited(60));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(InferenceError::Api(format!("{}: {}", status, body)));
        }

        response
            .json()
            .await
            .map_err(|e| InferenceError::Api(format!("JSON parse error: {e}")))
    }

    fn parse_inference_response(
        &self,
        api_response: &ChatCompletionResponse,
        request: &InferenceRequest,
    ) -> Result<(InferredContext, ConfidenceScores), InferenceError> {
        let content = api_response
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref())
            .ok_or_else(|| InferenceError::Api("Empty response from API".to_string()))?;

        // Parse JSON response
        let parsed: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| InferenceError::Api(format!("Invalid JSON response: {e}")))?;

        // Extract confidence scores
        let confidence_obj = parsed
            .get("confidence")
            .ok_or_else(|| InferenceError::Api("Missing confidence in response".to_string()))?;

        let overall = confidence_obj
            .get("overall")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);

        let by_field: Vec<FieldConfidence> = confidence_obj
            .get("by_field")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        let confidence = ConfidenceScores { overall, by_field };

        // Extract context
        let core = parsed.get("core")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .filter(|c: &CoreContext| {
                c.title.is_some() || c.creator.is_some() || c.classification.is_some()
            });

        let domain = parsed.get("domain")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let semantic = parsed.get("semantic")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let legal = if request.options.include_legal {
            parsed.get("legal").and_then(|v| serde_json::from_value(v.clone()).ok())
        } else {
            None
        };

        let entities = if request.options.include_entities {
            parsed.get("entities")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| serde_json::from_value(v.clone()).ok())
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let context = InferredContext {
            core,
            domain,
            semantic,
            legal,
            entities,
        };

        Ok((context, confidence))
    }
}

#[async_trait]
pub trait ContextInference: Send + Sync {
    async fn infer(&self, request: InferenceRequest) -> InferenceResult<crate::ContextInferenceResult>;
}

#[async_trait]
impl ContextInference for NebulClient {
    async fn infer(&self, request: InferenceRequest) -> InferenceResult<crate::ContextInferenceResult> {
        self.infer_context(request).await
    }
}

// ============================================================================
// Nebul API Types (OpenAI-compatible)
// ============================================================================

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    index: u32,
    message: ResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    role: String,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ModelsListResponse {
    object: String,
    data: Vec<NebulModelInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NebulModelInfo {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

// ============================================================================
// Helper methods
// ============================================================================

pub trait NebulProviderExt {
    fn as_str(&self) -> &'static str;
}

impl NebulProviderExt for crate::NebulProvider {
    fn as_str(&self) -> &'static str {
        match self {
            crate::NebulProvider::OpenAI => "openai",
            crate::NebulProvider::Anthropic => "anthropic",
            crate::NebulProvider::Meta => "meta",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_as_str() {
        assert_eq!(crate::NebulModel::NebulGPT4.as_str(), "gpt-4-turbo");
        assert_eq!(crate::NebulModel::NebulClaudeSonnet.as_str(), "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_provider() {
        assert_eq!(crate::NebulModel::NebulGPT4.provider(), crate::NebulProvider::OpenAI);
        assert_eq!(crate::NebulModel::NebulClaudeSonnet.provider(), crate::NebulProvider::Anthropic);
    }
}
