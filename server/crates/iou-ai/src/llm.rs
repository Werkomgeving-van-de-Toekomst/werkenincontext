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

    /// When false (e.g. local SLM / Ollama), requests omit `Authorization` if no key is set.
    #[serde(default = "default_requires_api_key")]
    pub requires_api_key: bool,
}

fn default_requires_api_key() -> bool {
    true
}

/// Standaard Ollama-tag: ~3B, sterk meertalig (o.a. NL/EN), geschikt voor samenvatting/structurering en lichte agent-stappen.
/// Zie `docs/architecture/ollama-models.md` voor alternatieven.
const DEFAULT_SLM_OLLAMA_MODEL: &str = "qwen2.5:3b";

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
            requires_api_key: true,
        })
    }

    /// SLM / OpenAI-compatible local server (Ollama, vLLM, etc.) via `SLM_*` env vars.
    ///
    /// - `SLM_BASE_URL` — default `http://127.0.0.1:11434` (Ollama exposes `/v1/chat/completions` under this root in recent versions; set full host including path prefix if your server differs)
    /// - `SLM_MODEL` — default `qwen2.5:3b` (licht, meertalig); Mistral-voorbeelden: `ministral-3:3b`, `mistral` — zie `docs/architecture/ollama-models.md`
    /// - `SLM_API_KEY` — optional (some gateways require a bearer token)
    pub fn from_slm_env() -> Result<Self> {
        let model = std::env::var("SLM_MODEL")
            .unwrap_or_else(|_| DEFAULT_SLM_OLLAMA_MODEL.to_string());
        let api_key = std::env::var("SLM_API_KEY").ok().filter(|s| !s.is_empty());
        let base_url = std::env::var("SLM_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());

        Ok(Self {
            api_key,
            model,
            base_url: Some(base_url),
            max_tokens: 4096,
            temperature: 0.7,
            requires_api_key: false,
        })
    }

    /// Check if API key is set on this config (after `from_env` / `from_slm_env`).
    pub fn has_api_key(&self) -> bool {
        self.api_key.as_ref().is_some_and(|s| !s.is_empty())
    }

    /// Ready for outbound chat/completions (cloud: needs key; SLM: needs base URL only).
    pub fn is_inference_ready(&self) -> bool {
        let base_ok = self
            .base_url
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if self.requires_api_key {
            base_ok && self.has_api_key()
        } else {
            base_ok
        }
    }

    /// Required bearer token when `requires_api_key` is true; for SLM optional (omit header if absent).
    pub fn get_api_key(&self) -> Result<String> {
        self.api_key
            .clone()
            .filter(|s| !s.is_empty())
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

fn chat_completions_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    format!("{base}/v1/chat/completions")
}

#[async_trait]
impl LlmBackend for MistralProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        if self.config.requires_api_key {
            self.config.get_api_key()?;
        }
        let base_url = self
            .config
            .base_url
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("https://api.mistral.ai");

        let url = chat_completions_url(base_url);
        let mut req = self
            .client
            .post(url)
            .json(&json!({
                "model": self.config.model,
                "max_tokens": self.config.max_tokens,
                "temperature": self.config.temperature,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }]
            }));
        if let Some(key) = self.config.api_key.as_ref().filter(|s| !s.is_empty()) {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let response = req.send().await?;

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
        if self.config.requires_api_key {
            self.config.get_api_key()?;
        }
        let base_url = self
            .config
            .base_url
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("https://api.mistral.ai");

        let url = chat_completions_url(base_url);
        let mut req = self.client.post(url).json(&json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "messages": messages
        }));
        if let Some(key) = self.config.api_key.as_ref().filter(|s| !s.is_empty()) {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let response = req.send().await?;

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
        self.config.is_inference_ready()
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
pub fn create_provider(config: &LlmConfig) -> Result<Box<dyn LlmBackend + Send + Sync>> {
    Ok(Box::new(MistralProvider::new(config.clone())?))
}

/// Create Mistral provider from environment
pub fn create_provider_from_env() -> Result<Box<dyn LlmBackend + Send + Sync>> {
    let config = LlmConfig::from_env()?;
    create_provider(&config)
}

/// OpenAI-compatible chat provider using `SLM_*` environment (local SLM).
pub fn create_slm_provider_from_env() -> Result<Box<dyn LlmBackend + Send + Sync>> {
    let config = LlmConfig::from_slm_env()?;
    create_provider(&config)
}
