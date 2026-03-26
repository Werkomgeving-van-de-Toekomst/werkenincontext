//! API client for document creation and management endpoints
//!
//! Provides type-safe API calls to the document REST API.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono;

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
// Workflow Stages Types
// ============================================

/// Stage view for API responses
#[derive(Clone, Debug, Deserialize)]
pub struct StageView {
    pub stage_id: String,
    pub id: Uuid,
    pub document_id: Uuid,
    pub stage_name: String,
    pub stage_order: i32,
    pub status: String,
    pub approval_type: String,
    pub approvers: Vec<ApproverView>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub deadline: Option<String>,
    pub sla_hours: i32,
}

/// Approver view
#[derive(Clone, Debug, Deserialize)]
pub struct ApproverView {
    pub user_id: Uuid,
    pub user_name: Option<String>,
    pub status: String,
    pub responded_at: Option<String>,
    pub delegated_from: Option<Uuid>,
}

/// Stage detail view
#[derive(Clone, Debug, Deserialize)]
pub struct StageDetailView {
    #[serde(flatten)]
    pub base: StageView,
    pub approvals_received: Vec<ApprovalResponseView>,
    pub escalation_count: i32,
    pub current_escalation_level: Option<i32>,
}

/// Approval response view
#[derive(Clone, Debug, Deserialize)]
pub struct ApprovalResponseView {
    pub approver_id: Uuid,
    pub delegated_from: Option<Uuid>,
    pub decision: String,
    pub comment: Option<String>,
    pub responded_at: String,
}

/// Approve stage request
#[derive(Clone, Debug, Serialize)]
pub struct ApproveStageRequest {
    pub comment: Option<String>,
}

/// Reject stage request
#[derive(Clone, Debug, Serialize)]
pub struct RejectStageRequest {
    pub reason: String,
    pub comment: Option<String>,
}

/// Delegate stage request
#[derive(Clone, Debug, Serialize)]
pub struct DelegateStageRequest {
    pub to_user_id: Uuid,
    pub reason: Option<String>,
}

// ============================================
// Delegations Types
// ============================================

/// Delegations list response
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DelegationsListResponse {
    pub created: Vec<DelegationView>,
    pub received: Vec<DelegationView>,
}

/// Delegation view
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DelegationView {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub from_user_name: Option<String>,
    pub to_user_id: Uuid,
    pub to_user_name: Option<String>,
    pub delegation_type: String,
    pub document_types: Vec<String>,
    pub document_id: Option<Uuid>,
    pub starts_at: String,
    pub ends_at: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

/// Create delegation request
#[derive(Clone, Debug, Serialize)]
pub struct CreateDelegationRequestApi {
    pub to_user_id: Uuid,
    pub delegation_type: String,
    pub document_types: Option<Vec<String>>,
    pub document_id: Option<Uuid>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
}

/// Reactivate delegation request
#[derive(Clone, Debug, Serialize)]
pub struct ReactivateDelegationRequest {
    pub ends_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================
// Versions Types
// ============================================

/// Version view
#[derive(Clone, Debug, Deserialize)]
pub struct VersionViewApi {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version_number: i32,
    pub created_at: String,
    pub created_by: Uuid,
    pub created_by_name: Option<String>,
    pub change_summary: String,
    pub is_compressed: bool,
    pub parent_version_id: Option<Uuid>,
    pub current: bool,
}

/// Diff response
#[derive(Clone, Debug, Deserialize)]
pub struct DiffResponse {
    pub from_version: String,
    pub to_version: String,
    pub format: String,
    pub changes: Vec<DiffChangeView>,
    pub summary: DiffSummary,
}

/// Diff change view
#[derive(Clone, Debug, Deserialize)]
pub struct DiffChangeView {
    pub change_type: String,
    pub old_text: Option<String>,
    pub new_text: Option<String>,
    pub line_number: Option<usize>,
}

/// Diff summary
#[derive(Clone, Debug, Deserialize)]
pub struct DiffSummary {
    pub additions: i32,
    pub deletions: i32,
    pub unchanged: i32,
}

/// Restore version request
#[derive(Clone, Debug, Serialize)]
pub struct RestoreVersionRequestApi {
    pub comment: Option<String>,
}

/// Restore version response
#[derive(Clone, Debug, Deserialize)]
pub struct RestoreVersionResponseApi {
    pub document_id: Uuid,
    pub restored_from_version: Uuid,
    pub new_version_id: Uuid,
    pub restored_at: String,
    pub restored_by: Uuid,
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
// Workflow Stages API
// ============================================

/// Get all stages for a document
pub async fn get_document_stages(document_id: Uuid) -> Result<Vec<StageView>, String> {
    let url = format!("{API_BASE}/documents/{}/stages", document_id);
    fetch_json(&url).await
}

/// Get detailed stage information
pub async fn get_stage_detail(document_id: Uuid, stage_id: String) -> Result<StageDetailView, String> {
    let url = format!("{API_BASE}/documents/{}/stages/{}", document_id, stage_id);
    fetch_json(&url).await
}

/// Approve a stage
pub async fn approve_stage(
    document_id: Uuid,
    stage_id: String,
    comment: Option<String>,
) -> Result<StageView, String> {
    let url = format!("{API_BASE}/documents/{}/stages/{}/approve", document_id, stage_id);
    let body = ApproveStageRequest { comment };
    post_json(&url, &body).await
}

/// Reject a stage
pub async fn reject_stage(
    document_id: Uuid,
    stage_id: String,
    reason: String,
    comment: Option<String>,
) -> Result<StageView, String> {
    let url = format!("{API_BASE}/documents/{}/stages/{}/reject", document_id, stage_id);
    let body = RejectStageRequest { reason, comment };
    post_json(&url, &body).await
}

/// Delegate a stage approval
pub async fn delegate_stage(
    document_id: Uuid,
    stage_id: String,
    to_user_id: Uuid,
    reason: Option<String>,
) -> Result<StageView, String> {
    let url = format!("{API_BASE}/documents/{}/stages/{}/delegate", document_id, stage_id);
    let body = DelegateStageRequest { to_user_id, reason };
    post_json(&url, &body).await
}

// ============================================
// Delegations API
// ============================================

/// List all delegations for current user
pub async fn list_delegations() -> Result<DelegationsListResponse, String> {
    let url = format!("{API_BASE}/delegations");
    fetch_json(&url).await
}

/// Get a specific delegation
pub async fn get_delegation(delegation_id: Uuid) -> Result<DelegationView, String> {
    let url = format!("{API_BASE}/delegations/{}", delegation_id);
    fetch_json(&url).await
}

/// Create a new delegation
pub async fn create_delegation(request: &CreateDelegationRequestApi) -> Result<DelegationView, String> {
    let url = format!("{API_BASE}/delegations");
    post_json(&url, request).await
}

/// Revoke a delegation
pub async fn revoke_delegation(delegation_id: Uuid) -> Result<(), String> {
    let url = format!("{API_BASE}/delegations/{}", delegation_id);
    delete(&url).await
}

/// Reactivate a delegation
pub async fn reactivate_delegation(
    delegation_id: Uuid,
    ends_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<DelegationView, String> {
    let url = format!("{API_BASE}/delegations/{}/reactivate", delegation_id);
    let body = ReactivateDelegationRequest { ends_at };
    post_json(&url, &body).await
}

// ============================================
// Versions API
// ============================================

/// List all versions for a document
pub async fn list_versions(document_id: Uuid) -> Result<Vec<VersionViewApi>, String> {
    let url = format!("{API_BASE}/documents/{}/versions", document_id);
    fetch_json(&url).await
}

/// Get diff between two versions
pub async fn get_diff(
    document_id: Uuid,
    from: Option<String>,
    to: Option<String>,
    format: Option<String>,
) -> Result<DiffResponse, String> {
    let mut query_parts = Vec::new();
    if let Some(f) = from {
        query_parts.push(format!("from={}", f));
    }
    if let Some(t) = to {
        query_parts.push(format!("to={}", t));
    }
    if let Some(fmt) = format {
        query_parts.push(format!("format={}", fmt));
    }

    let url = if query_parts.is_empty() {
        format!("{API_BASE}/documents/{}/versions/diff", document_id)
    } else {
        format!("{API_BASE}/documents/{}/versions/diff?{}", document_id, query_parts.join("&"))
    };

    fetch_json(&url).await
}

/// Get a specific version
pub async fn get_version(document_id: Uuid, version_id: String) -> Result<VersionViewApi, String> {
    let url = format!("{API_BASE}/documents/{}/versions/{}", document_id, version_id);
    fetch_json(&url).await
}

/// Restore a previous version
pub async fn restore_version(
    document_id: Uuid,
    version_id: String,
    comment: Option<String>,
) -> Result<RestoreVersionResponseApi, String> {
    let url = format!("{API_BASE}/documents/{}/versions/{}/restore", document_id, version_id);
    let body = RestoreVersionRequestApi { comment };
    post_json(&url, &body).await
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
