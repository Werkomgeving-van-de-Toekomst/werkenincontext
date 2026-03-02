//! LLM Provider Abstraction
//!
//! Supports Mistral AI for content generation.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Errors that can occur during LLM operations
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("API request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("API returned error: {0}")]
    ApiError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("No API key configured")]
    NoApiKey,

    #[error("Rate limited: retry after {0}s")]
    RateLimited(u64),
}

/// Result type for LLM operations
pub type Result<T> = std::result::Result<T, LlmError>;

/// Configuration for LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API key (can be loaded from environment)
    #[serde(skip_serializing)]
    pub api_key: Option<String>,

    /// Model to use
    pub model: String,

    /// API base URL (for custom endpoints)
    pub base_url: Option<String>,

    /// Maximum tokens to generate
    pub max_tokens: usize,

    /// Temperature (0.0 - 1.0)
    pub temperature: f32,
}

impl LlmConfig {
    /// Create config from environment variables
    pub fn from_env() -> Result<Self> {
        let model = std::env::var("LLM_MODEL")
            .unwrap_or_else(|_| "mistral-small-latest".to_string());

        let api_key = std::env::var("LLM_API_KEY").ok();

        let base_url = std::env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| "https://api.mistral.ai".to_string());

        Ok(Self {
            api_key,
            model,
            base_url: Some(base_url),
            max_tokens: 4096,
            temperature: 0.7,
        })
    }

    /// Check if API key is available
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some() || std::env::var("LLM_API_KEY").is_ok()
    }

    /// Get the API key (from config or environment)
    pub fn get_api_key(&self) -> Result<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("LLM_API_KEY").ok())
            .ok_or(LlmError::NoApiKey)
    }
}

/// LLM Backend trait for implementations
#[async_trait]
pub trait LlmBackend: Send + Sync {
    /// Generate text from a prompt
    async fn generate(&self, prompt: &str) -> Result<String>;

    /// Generate with chat history
    async fn chat(&self, messages: &[ChatMessage]) -> Result<String>;

    /// Check if provider is configured
    fn is_configured(&self) -> bool;
}

/// Chat message for conversational LLMs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Mistral AI provider
pub struct MistralProvider {
    client: Client,
    config: LlmConfig,
}

impl MistralProvider {
    pub fn new(config: LlmConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            config,
        })
    }
}

#[async_trait]
impl LlmBackend for MistralProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let api_key = self.config.get_api_key()?;
        let base_url = self.config.base_url.as_ref()
            .map(|s| s.as_str())
            .unwrap_or("https://api.mistral.ai");

        let response = self.client
            .post(format!("{}/v1/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&json!({
                "model": self.config.model,
                "max_tokens": self.config.max_tokens,
                "temperature": self.config.temperature,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }]
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            Ok(json["choices"][0]["message"]["content"]
                .as_str()
                .ok_or_else(|| LlmError::InvalidResponse("Missing content".to_string()))?
                .to_string())
        } else {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            Err(LlmError::ApiError(format!("HTTP {}: {}", status, text)))
        }
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<String> {
        let api_key = self.config.get_api_key()?;
        let base_url = self.config.base_url.as_ref()
            .map(|s| s.as_str())
            .unwrap_or("https://api.mistral.ai");

        let response = self.client
            .post(format!("{}/v1/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&json!({
                "model": self.config.model,
                "max_tokens": self.config.max_tokens,
                "temperature": self.config.temperature,
                "messages": messages
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            Ok(json["choices"][0]["message"]["content"]
                .as_str()
                .ok_or_else(|| LlmError::InvalidResponse("Missing content".to_string()))?
                .to_string())
        } else {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            Err(LlmError::ApiError(format!("HTTP {}: {}", status, text)))
        }
    }

    fn is_configured(&self) -> bool {
        self.config.has_api_key()
    }
}

/// Mock/stub provider for testing
pub struct MockProvider {
    pub response_template: String,
}

#[async_trait]
impl LlmBackend for MockProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        Ok(format!("AI response to: {}", prompt.chars().take(50).collect::<String>()))
    }

    async fn chat(&self, _messages: &[ChatMessage]) -> Result<String> {
        Ok(self.response_template.clone())
    }

    fn is_configured(&self) -> bool {
        true
    }
}

/// Create Mistral provider from config
pub fn create_provider(config: &LlmConfig) -> Result<Box<dyn LlmBackend>> {
    Ok(Box::new(MistralProvider::new(config.clone())?))
}

/// Create Mistral provider from environment
pub fn create_provider_from_env() -> Result<Box<dyn LlmBackend>> {
    let config = LlmConfig::from_env()?;
    create_provider(&config)
}
