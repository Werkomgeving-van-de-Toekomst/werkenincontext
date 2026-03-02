//! API client for document creation and management endpoints
//!
//! Provides type-safe API calls to the document REST API.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

const API_BASE: &str = "http://localhost:8000/api";

// ============================================
// Request/Response Types
// ============================================

/// Request payload for document creation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreateDocumentRequest {
    pub domain_id: String,
    pub document_type: String,
    #[serde(default)]
    pub context: serde_json::Map<String, serde_json::Value>,
}

/// Response for successful document creation
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CreateDocumentResponse {
    pub document_id: Uuid,
    pub state: String,
    pub estimated_completion: Option<String>,
}

/// Document status response
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DocumentStatus {
    pub document_id: Uuid,
    pub document_type: String,
    pub state: String,
    pub current_agent: Option<String>,
    pub compliance_score: f32,
    pub confidence_score: f32,
    pub requires_approval: bool,
    #[serde(default)]
    pub errors: Vec<String>,
}

/// Approval request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub approved: bool,
    pub comments: Option<String>,
}

/// Approval response
#[derive(Clone, Debug, Deserialize)]
pub struct ApprovalResponse {
    pub document_id: Uuid,
    pub state: String,
    pub approved_at: Option<String>,
    pub approved_by: Option<String>,
}

/// Audit trail response
#[derive(Clone, Debug, Deserialize)]
pub struct AuditTrailResponse {
    pub document_id: Uuid,
    pub audit_trail: Vec<AuditEntry>,
}

/// Audit entry
#[derive(Clone, Debug, Deserialize)]
pub struct AuditEntry {
    pub agent: String,
    pub action: String,
    pub timestamp: String,
    pub details: serde_json::Value,
}

/// Template
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub domain_id: String,
    pub document_type: String,
    pub content: String,
    #[serde(default)]
    pub required_variables: Vec<String>,
    #[serde(default)]
    pub optional_sections: Vec<String>,
    pub version: i32,
    #[serde(default)]
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl Template {
    pub fn empty() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            domain_id: String::new(),
            document_type: String::new(),
            content: String::new(),
            required_variables: Vec::new(),
            optional_sections: Vec::new(),
            version: 1,
            is_active: true,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}

/// Template list response
#[derive(Clone, Debug, Deserialize)]
pub struct TemplateListResponse {
    pub templates: Vec<TemplateDto>,
}

/// Template DTO for API responses
#[derive(Clone, Debug, Deserialize)]
pub struct TemplateDto {
    pub id: String,
    pub name: String,
    pub domain_id: String,
    pub document_type: String,
    pub version: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Create template request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub domain_id: String,
    pub document_type: String,
    pub content: String,
    #[serde(default)]
    pub required_variables: Vec<String>,
    #[serde(default)]
    pub optional_sections: Vec<String>,
}

/// Create template response
#[derive(Clone, Debug, Deserialize)]
pub struct CreateTemplateResponse {
    pub template_id: String,
    pub version: i32,
}

// ============================================
// API Client Functions
// ============================================

/// Create a new document
pub async fn create_document(request: &CreateDocumentRequest) -> Result<CreateDocumentResponse, String> {
    let url = format!("{API_BASE}/documents/create");
    post_json(&url, request).await
}

/// Get document status by ID
pub async fn get_document_status(document_id: Uuid) -> Result<DocumentStatus, String> {
    let url = format!("{API_BASE}/documents/{}/status", document_id);
    fetch_json(&url).await
}

/// Approve or reject a document
pub async fn approve_document(
    document_id: Uuid,
    request: &ApprovalRequest,
) -> Result<ApprovalResponse, String> {
    let url = format!("{API_BASE}/documents/{}/approve", document_id);
    post_json(&url, request).await
}

/// Get audit trail for a document
pub async fn get_audit_trail(document_id: Uuid) -> Result<AuditTrailResponse, String> {
    let url = format!("{API_BASE}/documents/{}/audit", document_id);
    fetch_json(&url).await
}

/// List all templates
pub async fn list_templates(domain_id: Option<String>) -> Result<Vec<TemplateDto>, String> {
    let url = if let Some(domain) = domain_id {
        format!("{API_BASE}/templates?domain_id={}", domain)
    } else {
        format!("{API_BASE}/templates")
    };
    fetch_json(&url).await
}

/// Get a specific template by ID
pub async fn get_template(template_id: String) -> Result<Template, String> {
    let url = format!("{API_BASE}/templates/{}", template_id);
    fetch_json(&url).await
}

/// Create a new template
pub async fn create_template(request: &CreateTemplateRequest) -> Result<CreateTemplateResponse, String> {
    let url = format!("{API_BASE}/templates");
    post_json(&url, request).await
}

/// Update an existing template
pub async fn update_template(template_id: String, template: &Template) -> Result<TemplateDto, String> {
    let url = format!("{API_BASE}/templates/{}", template_id);
    put_json(&url, template).await
}

/// Delete (deactivate) a template
pub async fn delete_template(template_id: String) -> Result<(), String> {
    let url = format!("{API_BASE}/templates/{}", template_id);
    delete(&url).await
}

/// List documents pending approval
pub async fn list_pending_approval() -> Result<Vec<DocumentStatus>, String> {
    // Note: This endpoint doesn't exist yet in the API, but we can implement it
    // by calling get_status for each document or adding a list endpoint
    // For now, return empty
    Ok(Vec::new())
}

// ============================================
// Generic Fetch Helpers
// ============================================

/// Generic GET request
async fn fetch_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    fetch_json_impl(url).await
}

/// Generic POST request with JSON body
async fn post_json<T: serde::de::DeserializeOwned>(
    url: &str,
    body: &impl Serialize,
) -> Result<T, String> {
    post_json_impl(url, body).await
}

/// Generic PUT request with JSON body
async fn put_json<T: serde::de::DeserializeOwned>(
    url: &str,
    body: &impl Serialize,
) -> Result<T, String> {
    put_json_impl(url, body).await
}

/// Generic DELETE request
async fn delete(url: &str) -> Result<(), String> {
    delete_impl(url).await
}

// WASM implementation
#[cfg(target_arch = "wasm32")]
async fn fetch_json_impl<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(web_sys::RequestMode::Cors);

    let request = web_sys::Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {e:?}"))?;

    let window = web_sys::window().expect("no global window");
    let response_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {e:?}"))?;

    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|e| format!("Invalid response type: {e:?}"))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let text_promise = response.text()
        .map_err(|e| format!("Failed to get text promise: {e:?}"))?;

    let text_value = JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("Failed to get response text: {e:?}"))?;

    let text = text_value.as_string()
        .ok_or_else(|| "Response text is not a string".to_string())?;

    serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {e}"))
}

#[cfg(target_arch = "wasm32")]
async fn post_json_impl<T: serde::de::DeserializeOwned>(
    url: &str,
    body: &impl Serialize,
) -> Result<T, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let body_json = serde_json::to_string(body)
        .map_err(|e| format!("Failed to serialize body: {e}"))?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(web_sys::RequestMode::Cors);
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body_json));

    let headers = web_sys::Headers::new()
        .map_err(|e| format!("Failed to create headers: {e:?}"))?;
    headers.append("Content-Type", "application/json")
        .map_err(|e| format!("Failed to set content-type: {e:?}"))?;
    opts.set_headers(&headers);

    let request = web_sys::Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {e:?}"))?;

    let window = web_sys::window().expect("no global window");
    let response_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {e:?}"))?;

    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|e| format!("Invalid response type: {e:?}"))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let text_promise = response.text()
        .map_err(|e| format!("Failed to get text promise: {e:?}"))?;

    let text_value = JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("Failed to get response text: {e:?}"))?;

    let text = text_value.as_string()
        .ok_or_else(|| "Response text is not a string".to_string())?;

    serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {e}"))
}

#[cfg(target_arch = "wasm32")]
async fn put_json_impl<T: serde::de::DeserializeOwned>(
    url: &str,
    body: &impl Serialize,
) -> Result<T, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let body_json = serde_json::to_string(body)
        .map_err(|e| format!("Failed to serialize body: {e}"))?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("PUT");
    opts.set_mode(web_sys::RequestMode::Cors);
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body_json));

    let headers = web_sys::Headers::new()
        .map_err(|e| format!("Failed to create headers: {e:?}"))?;
    headers.append("Content-Type", "application/json")
        .map_err(|e| format!("Failed to set content-type: {e:?}"))?;
    opts.set_headers(&headers);

    let request = web_sys::Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {e:?}"))?;

    let window = web_sys::window().expect("no global window");
    let response_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {e:?}"))?;

    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|e| format!("Invalid response type: {e:?}"))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let text_promise = response.text()
        .map_err(|e| format!("Failed to get text promise: {e:?}"))?;

    let text_value = JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("Failed to get response text: {e:?}"))?;

    let text = text_value.as_string()
        .ok_or_else(|| "Response text is not a string".to_string())?;

    serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {e}"))
}

#[cfg(target_arch = "wasm32")]
async fn delete_impl(url: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let opts = web_sys::RequestInit::new();
    opts.set_method("DELETE");
    opts.set_mode(web_sys::RequestMode::Cors);

    let request = web_sys::Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {e:?}"))?;

    let window = web_sys::window().expect("no global window");
    let response_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {e:?}"))?;

    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|e| format!("Invalid response type: {e:?}"))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    Ok(())
}

// Desktop implementation using reqwest
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
async fn fetch_json_impl<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    extern crate reqwest;

    reqwest::get(url)
        .await
        .map_err(|e| format!("Network error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
async fn post_json_impl<T: serde::de::DeserializeOwned>(
    url: &str,
    body: &impl Serialize,
) -> Result<T, String> {
    extern crate reqwest;

    let client = reqwest::Client::new();
    client.post(url)
        .json(body)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
async fn put_json_impl<T: serde::de::DeserializeOwned>(
    url: &str,
    body: &impl Serialize,
) -> Result<T, String> {
    extern crate reqwest;

    let client = reqwest::Client::new();
    client.put(url)
        .json(body)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
async fn delete_impl(url: &str) -> Result<(), String> {
    extern crate reqwest;

    reqwest::Client::new()
        .delete(url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    Ok(())
}

// Stub for non-WASM builds without desktop feature
#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
async fn fetch_json_impl<T: serde::de::DeserializeOwned>(_url: &str) -> Result<T, String> {
    Err("HTTP client not available".to_string())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
async fn post_json_impl<T: serde::de::DeserializeOwned>(
    _url: &str,
    _body: &impl Serialize,
) -> Result<T, String> {
    Err("HTTP client not available".to_string())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
async fn put_json_impl<T: serde::de::DeserializeOwned>(
    _url: &str,
    _body: &impl Serialize,
) -> Result<T, String> {
    Err("HTTP client not available".to_string())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
async fn delete_impl(_url: &str) -> Result<(), String> {
    Err("HTTP client not available".to_string())
}
