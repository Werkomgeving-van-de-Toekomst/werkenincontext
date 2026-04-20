//! Ollama AI Service Integration
//!
//! Local AI service using Ollama for document enrichment,
//! metadata generation, and classification in a Dutch government context.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Current timestamp for generation tracking
fn now_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    format!(
        "{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    )
}

/// Ollama AI service client
pub struct OllamaClient {
    /// Ollama base URL (default: http://localhost:11434)
    base_url: String,

    /// HTTP client
    client: Client,

    /// Default model to use
    default_model: String,

    /// Request timeout
    timeout: Duration,
}

/// Errors from Ollama operations
#[derive(Debug, Error)]
pub enum OllamaError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Ollama API error: {0}")]
    ApiError(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Timeout waiting for response")]
    Timeout,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
            default_model: "llama3.2".to_string(),
            timeout: Duration::from_secs(120),
        }
    }

    /// Create with default localhost URL
    pub fn localhost() -> Self {
        Self::new("http://localhost:11434")
    }

    /// Set the default model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");
        self
    }

    /// Check if Ollama is running and accessible
    pub async fn health_check(&self) -> Result<OllamaHealth, OllamaError> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            Ok(OllamaHealth {
                healthy: true,
                message: "Ollama is running".to_string(),
            })
        } else {
            Ok(OllamaHealth {
                healthy: false,
                message: format!("Ollama returned status: {}", response.status()),
            })
        }
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>, OllamaError> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await?;
        let result: OllamaListResponse = response.json().await?;
        Ok(result.models)
    }

    /// Check if a model exists
    pub async fn model_exists(&self, model: &str) -> Result<bool, OllamaError> {
        let models = self.list_models().await?;
        Ok(models.iter().any(|m| m.name == model || m.name.starts_with(&format!("{}:", model))))
    }

    /// Pull a model from Ollama library
    pub async fn pull_model(&self, model: &str) -> Result<PullProgress, OllamaError> {
        let url = format!("{}/api/pull", self.base_url);
        let body = OllamaPullRequest {
            name: model.to_string(),
            stream: false,
        };

        let response = self.client.post(&url).json(&body).send().await?;
        let result: OllamaPullResponse = response.json().await?;

        if let Some(error) = result.error {
            Err(OllamaError::ApiError(error))
        } else {
            Ok(PullProgress {
                model: model.to_string(),
                status: result.status.unwrap_or("unknown".to_string()),
                completed: result.completed.unwrap_or(0),
                total: result.total.unwrap_or(0),
            })
        }
    }

    /// Generate a chat completion
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, OllamaError> {
        let model = request.model.unwrap_or_else(|| self.default_model.clone());
        let url = format!("{}/api/chat", self.base_url);

        let body = OllamaChatRequest {
            model: model.clone(),
            messages: request.messages,
            stream: false,
            options: request.options,
            format: request.format,
        };

        let response = self.client.post(&url).json(&body).send().await?;
        let result: OllamaChatResponse = response.json().await?;

        if let Some(error) = result.error {
            Err(OllamaError::ApiError(error))
        } else if let Some(message) = result.message {
            Ok(ChatResponse {
                model,
                content: message.content.unwrap_or_default(),
                done: result.done.unwrap_or(true),
                context: result.context,
                prompt_eval_count: result.prompt_eval_count,
                eval_count: result.eval_count,
            })
        } else {
            Err(OllamaError::InvalidResponse("No message in response".to_string()))
        }
    }

    /// Generate a completion (simple prompt)
    pub async fn generate(&self, prompt: impl Into<String>) -> Result<String, OllamaError> {
        let prompt = prompt.into();
        let request = ChatRequest {
            model: None,
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            options: None,
            format: None,
        };

        let response = self.chat(request).await?;
        Ok(response.content)
    }

    /// Generate with JSON output format
    pub async fn generate_json(&self, prompt: impl Into<String>) -> Result<serde_json::Value, OllamaError> {
        let prompt = prompt.into();
        let request = ChatRequest {
            model: None,
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            options: None,
            format: Some("json".to_string()),
        };

        let response = self.chat(request).await?;
        serde_json::from_str(&response.content)
            .map_err(|e| OllamaError::InvalidResponse(format!("JSON parse error: {}", e)))
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaHealth {
    pub healthy: bool,
    pub message: String,
}

/// Ollama model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: Option<u64>,
}

/// List models response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaListResponse {
    pub models: Vec<OllamaModel>,
}

/// Pull model request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaPullRequest {
    pub name: String,
    pub stream: bool,
}

/// Pull model response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaPullResponse {
    pub status: Option<String>,
    pub digest: Option<String>,
    pub total: Option<usize>,
    pub completed: Option<usize>,
    pub error: Option<String>,
}

/// Pull progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullProgress {
    pub model: String,
    pub status: String,
    pub completed: usize,
    pub total: usize,
}

/// Chat request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub options: Option<GenerationOptions>,
    pub format: Option<String>,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Generation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<usize>,
    pub num_predict: Option<usize>,
    pub repeat_penalty: Option<f32>,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(40),
            num_predict: Some(2048),
            repeat_penalty: Some(1.1),
        }
    }
}

/// Ollama chat request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    pub options: Option<GenerationOptions>,
    pub format: Option<String>,
}

/// Ollama chat response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaChatResponse {
    pub model: String,
    pub created_at: String,
    pub message: Option<OllamaChatMessage>,
    pub done: Option<bool>,
    pub context: Option<Vec<usize>>,
    pub prompt_eval_count: Option<usize>,
    pub eval_count: Option<usize>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaChatMessage {
    pub role: String,
    pub content: Option<String>,
}

/// Chat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub model: String,
    pub content: String,
    pub done: bool,
    pub context: Option<Vec<usize>>,
    pub prompt_eval_count: Option<usize>,
    pub eval_count: Option<usize>,
}

// ============================================
// DUTCH GOVERNMENT AI SERVICE
// ============================================

/// AI service for Dutch government document enrichment
pub struct NederlandseAIDienst {
    /// Ollama client
    client: OllamaClient,

    /// Service configuration
    config: AIConfig,
}

/// AI service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// Default model for general tasks
    pub general_model: String,

    /// Model for classification (faster, smaller)
    pub classification_model: String,

    /// Model for summarization
    pub summarization_model: String,

    /// Language (nl-NL, nl-NL)
    pub language: String,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            general_model: "llama3.2".to_string(),
            classification_model: "llama3.2:1b".to_string(),
            summarization_model: "llama3.2".to_string(),
            language: "nl-NL".to_string(),
        }
    }
}

impl NederlandseAIDienst {
    /// Create a new AI service
    pub fn new(client: OllamaClient) -> Self {
        Self {
            client,
            config: AIConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(client: OllamaClient, config: AIConfig) -> Self {
        Self { client, config }
    }

    /// Create with default localhost Ollama
    pub fn localhost() -> Self {
        Self::new(OllamaClient::localhost())
    }

    /// Generate document summary
    pub async fn genereer_samenvatting(&self, document: &DocumentInhoud) -> Result<DocumentSamenvatting, OllamaError> {
        let prompt = self.build_summary_prompt(document);
        let response = self.client.generate(prompt).await?;

        Ok(DocumentSamenvatting {
            samenvatting: response,
            gegenereerd_op: now_timestamp(),
            model: self.config.summarization_model.clone(),
        })
    }

    /// Extract metadata from document
    pub async fn extraheer_metadata(&self, document: &DocumentInhoud) -> Result<AIDocumentMetadata, OllamaError> {
        let prompt = self.build_metadata_prompt(document);
        let json = self.client.generate_json(prompt).await?;

        Ok(AIDocumentMetadata {
            titel: json["titel"].as_str().unwrap_or(&document.titel).to_string(),
            beschrijving: json["beschrijving"].as_str().map(|s| s.to_string()),
            documenttype: json["documenttype"].as_str().map(|s| s.to_string()),
            trefwoorden: json["trefwoorden"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        })
    }

    /// Classify document according to Woo (Wet open overheid)
    pub async fn classificeer_wo(&self, document: &DocumentInhoud) -> Result<WooClassificatie, OllamaError> {
        let prompt = self.build_woo_classification_prompt(document);
        let json = self.client.generate_json(prompt).await?;

        Ok(WooClassificatie {
            woo_relevant: json["woo_relevant"].as_bool().unwrap_or(false),
            categorie: json["categorie"].as_str().map(|s| s.to_string()),
            openbaarheid: json["openbaarheid"].as_str().map(|s| s.to_string()),
            redengeving: json["redengeving"].as_str().map(|s| s.to_string()),
        })
    }

    /// Extract entities from document (persons, organizations, locations)
    pub async fn extraheer_entiteiten(&self, document: &DocumentInhoud) -> Result<Vec<Entiteit>, OllamaError> {
        let prompt = self.build_entity_extraction_prompt(document);
        let json = self.client.generate_json(prompt).await?;

        let entiteiten = json["entiteiten"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        Some(Entiteit {
                            entiteit_type: v["type"].as_str().map(|s| s.to_string()).unwrap_or_default(),
                            naam: v["naam"].as_str().map(|s| s.to_string()).unwrap_or_default(),
                            rol: v["rol"].as_str().map(|s| s.to_string()),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(entiteiten)
    }

    /// Suggest retention period according to Archiefwet
    pub async fn stel_bewaartermijnvoor(&self, document: &DocumentInhoud) -> Result<BewaartermijnSuggestie, OllamaError> {
        let prompt = self.build_retention_prompt(document);
        let json = self.client.generate_json(prompt).await?;

        Ok(BewaartermijnSuggestie {
            bewaartermijn: json["bewaartermijn"].as_str().map(|s| s.to_string()).unwrap_or_default(),
            selectielijst: json["selectielijst"].as_str().map(|s| s.to_string()),
            redengeving: json["redengeving"].as_str().map(|s| s.to_string()),
        })
    }

    /// Build summary prompt
    fn build_summary_prompt(&self, document: &DocumentInhoud) -> String {
        format!(
            r#"Je bent een AI-assistent voor een Nederlandse overheidsorganisatie.
Je taak is om een beknopte samenvatting te maken van het volgende document.

DOCUMENT TITEL: {}
DOCUMENT TYPE: {:?}
INHOUD:

{}

INSTRUCTIES:
1. Schrijf een samenvatting van maximaal 150 woorden in het Nederlands
2. Focus op de kernpunten en besluiten
3. Gebruik professionele taal die past bij overheidscommunicatie
4. Vermijd onnodige details

SAMENVATTING:"#,
            document.titel,
            document.document_type.clone().unwrap_or_else(|| "Onbekend".to_string()),
            document.inhoud
        )
    }

    /// Build metadata extraction prompt
    fn build_metadata_prompt(&self, document: &DocumentInhoud) -> String {
        format!(
            r#"Je bent een AI-assistent voor een Nederlandse overheidsorganisatie.
Extraheer gestructureerde metadata uit het volgende document.

DOCUMENT TITEL: {}
INHOUD: {}

Geef antwoord in JSON-formaat:
{{
  "titel": "verbeterde titel indien nodig",
  "beschrijving": "korte beschrijving van het document",
  "documenttype": "type document (bijv. besluit, notitie, brief, rapport)",
  "trefwoorden": ["trefwoord1", "trefwoord2", ...]
}}"#,
            document.titel, document.inhoud
        )
    }

    /// Build Woo classification prompt
    fn build_woo_classification_prompt(&self, document: &DocumentInhoud) -> String {
        format!(
            r#"Classificeer het volgende document volgens de Wet open overheid (Woo).

DOCUMENT TITEL: {}
INHOUD: {}

Geef antwoord in JSON-formaat:
{{
  "woo_relevant": true/false,
  "categorie": "categorie indien van toepassing",
  "openbaarheid": "openbaar/deelopenbaar/niet_openbaar",
  "redengeving": "korte toelichting"
}}"#,
            document.titel, document.inhoud
        )
    }

    /// Build entity extraction prompt
    fn build_entity_extraction_prompt(&self, document: &DocumentInhoud) -> String {
        format!(
            r#"Extraheer alle genoemde entiteiten (personen, organisaties, locaties) uit het volgende document.

DOCUMENT TITEL: {}
INHOUD: {}

Geef antwoord in JSON-formaat:
{{
  "entiteiten": [
    {{"type": "persoon/organisatie/locatie", "naam": "naam", "rol": "rol indien van toepassing"}},
    ...
  ]
}}"#,
            document.titel, document.inhoud
        )
    }

    /// Build retention period prompt
    fn build_retention_prompt(&self, document: &DocumentInhoud) -> String {
        format!(
            r#"Bepaal de bewaartermijn volgens de Archiefwet voor het volgende document.

DOCUMENT TITEL: {}
DOCUMENT TYPE: {:?}
INHOUD: {}

Geef antwoord in JSON-formaat:
{{
  "bewaartermijn": "bijv. 10 jaar, 20 jaar, permanent",
  "selectielijst": "nummer van de selectielijst indien van toepassing",
  "redengeving": "toelichting bij deze termijn"
}}"#,
            document.titel,
            document.document_type.clone().unwrap_or_else(|| "Onbekend".to_string()),
            document.inhoud
        )
    }
}

// ============================================
// DOCUMENT TYPES
// ============================================

/// Document content for AI processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInhoud {
    /// Document title
    pub titel: String,

    /// Document content/text
    pub inhoud: String,

    /// Document type (optional)
    pub document_type: Option<String>,

    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Document summary result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSamenvatting {
    /// Summary text
    pub samenvatting: String,

    /// When generated (UNIX timestamp)
    pub gegenereerd_op: String,

    /// Model used
    pub model: String,
}

/// Extracted document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIDocumentMetadata {
    /// Document title
    pub titel: String,

    /// Description
    pub beschrijving: Option<String>,

    /// Document type
    pub documenttype: Option<String>,

    /// Keywords
    pub trefwoorden: Vec<String>,
}

/// Woo classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WooClassificatie {
    /// Whether document is Woo-relevant
    pub woo_relevant: bool,

    /// Category
    pub categorie: Option<String>,

    /// Openness level
    pub openbaarheid: Option<String>,

    /// Reasoning
    pub redengeving: Option<String>,
}

/// Extracted entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entiteit {
    /// Entity type
    pub entiteit_type: String,

    /// Name
    pub naam: String,

    /// Role (optional)
    pub rol: Option<String>,
}

/// Retention period suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BewaartermijnSuggestie {
    /// Retention period
    pub bewaartermijn: String,

    /// Selection list reference
    pub selectielijst: Option<String>,

    /// Reasoning
    pub redengeving: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_client_creation() {
        let client = OllamaClient::localhost();
        assert_eq!(client.base_url, "http://localhost:11434");
        assert_eq!(client.default_model, "llama3.2");
    }

    #[test]
    fn test_ollama_client_with_model() {
        let client = OllamaClient::localhost().with_model("mistral");
        assert_eq!(client.default_model, "mistral");
    }

    #[test]
    fn test_ai_service_creation() {
        let service = NederlandseAIDienst::localhost();
        assert_eq!(service.config.language, "nl-NL");
    }

    #[test]
    fn test_document_inhoud() {
        let doc = DocumentInhoud {
            titel: "Test Document".to_string(),
            inhoud: "Test inhoud".to_string(),
            document_type: Some("Notitie".to_string()),
            metadata: None,
        };

        assert_eq!(doc.titel, "Test Document");
        assert_eq!(doc.document_type, Some("Notitie".to_string()));
    }

    #[test]
    fn test_generation_options_default() {
        let opts = GenerationOptions::default();
        assert_eq!(opts.temperature, Some(0.7));
        assert_eq!(opts.top_p, Some(0.9));
    }

    #[test]
    fn test_build_summary_prompt() {
        let service = NederlandseAIDienst::localhost();
        let doc = DocumentInhoud {
            titel: "Besluit".to_string(),
            inhoud: "De gemeente besluit tot...".to_string(),
            document_type: Some("Besluit".to_string()),
            metadata: None,
        };

        let prompt = service.build_summary_prompt(&doc);
        assert!(prompt.contains("Besluit"));
        assert!(prompt.contains("gemeente besluit"));
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: Some("llama3.2".to_string()),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "You are a helpful assistant.".to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
            ],
            options: None,
            format: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("llama3.2"));
        assert!(json.contains("Hello"));
    }
}
