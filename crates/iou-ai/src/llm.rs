//! LLM Provider Abstraction
//!
//! Supports multiple LLM backends for content generation.

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
    /// Provider to use
    pub provider: LlmProvider,

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
        let provider = std::env::var("LLM_PROVIDER")
            .unwrap_or_else(|_| "anthropic".to_string());

        let model = std::env::var("LLM_MODEL")
            .unwrap_or_else(|_| "claude-3-haiku-20240307".to_string());

        let api_key = std::env::var("LLM_API_KEY").ok();

        let base_url = match provider.as_str() {
            "anthropic" => Some("https://api.anthropic.com".to_string()),
            "openai" => Some("https://api.openai.com/v1".to_string()),
            "custom" => std::env::var("LLM_BASE_URL").ok(),
            _ => None,
        };

        Ok(Self {
            provider: provider.parse()?,
            api_key,
            model,
            base_url,
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

/// LLM Provider type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    Anthropic,
    OpenAI,
    Custom(String),
}

impl std::str::FromStr for LlmProvider {
    type Err = LlmError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "anthropic" => Ok(LlmProvider::Anthropic),
            "openai" => Ok(LlmProvider::OpenAI),
            other => Ok(LlmProvider::Custom(other.to_string())),
        }
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

/// Anthropic Claude provider
pub struct AnthropicProvider {
    client: Client,
    config: LlmConfig,
}

impl AnthropicProvider {
    pub fn new(config: LlmConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            config,
        })
    }
}

#[async_trait]
impl LlmBackend for AnthropicProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let api_key = self.config.get_api_key()?;

        let response = self.client
            .post(format!("{}/v1/messages", self.config.base_url.as_ref().unwrap()))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
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
            Ok(json["content"][0]["text"]
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

        let response = self.client
            .post(format!("{}/v1/messages", self.config.base_url.as_ref().unwrap()))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
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
            Ok(json["content"][0]["text"]
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

/// OpenAI provider
pub struct OpenAiProvider {
    client: Client,
    config: LlmConfig,
}

impl OpenAiProvider {
    pub fn new(config: LlmConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            config,
        })
    }
}

#[async_trait]
impl LlmBackend for OpenAiProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let api_key = self.config.get_api_key()?;

        let response = self.client
            .post(format!("{}/chat/completions", self.config.base_url.as_ref().unwrap()))
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

        let response = self.client
            .post(format!("{}/chat/completions", self.config.base_url.as_ref().unwrap()))
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

/// Create LLM provider from config
pub fn create_provider(config: &LlmConfig) -> Result<Box<dyn LlmBackend>> {
    match config.provider {
        LlmProvider::Anthropic => Ok(Box::new(AnthropicProvider::new(config.clone())?)),
        LlmProvider::OpenAI => Ok(Box::new(OpenAiProvider::new(config.clone())?)),
        LlmProvider::Custom(ref name) => {
            // For custom providers, you'd need to implement them
            Err(LlmError::ApiError(format!("Unknown provider: {}", name)))
        }
    }
}
