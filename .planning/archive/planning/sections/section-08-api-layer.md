Now I have all the context. Let me generate the section content for section-08-api-layer. This section covers the REST API endpoints for document operations. I'll extract the relevant tests from the TDD plan and the implementation details from the main plan.

# Section 08: API Layer

## Overview

This section implements the REST API layer that exposes the document creation pipeline functionality to clients. The API handles document creation requests, status queries, human approval workflows, audit trail access, template management, and document downloads.

**Dependencies:**
- section-07-pipeline-orchestration (must be completed first)
- Existing authentication/authorization middleware in `iou-api`
- Existing workflow state management in `iou-core/src/workflows.rs`

**Files to Create:**
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/documents.rs` - Main document API routes
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/templates.rs` - Template management routes
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/handlers/document_creation.rs` - Request handlers for document operations
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/handlers/approval.rs` - Approval workflow handlers
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/extractors/auth.rs` - Authentication/authorization extractors (if not existing)

**Files to Modify:**
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/main.rs` - Register new routes
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/lib.rs` - Export new modules

---

## Tests

### Phase 5: API Layer Tests

**Location:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/documents/tests.rs` (or inline with `#[cfg(test)]` modules)

#### Authentication/Authorization Tests

```rust
#[cfg(test)]
mod auth_tests {
    use super::*;
    
    /// Test: All endpoints respond to unauthenticated requests with 401
    #[tokio::test]
    async fn unauthenticated_requests_return_401() {
        // Test that all document endpoints require authentication
        // - POST /api/documents/create
        // - GET /api/documents/{id}/status
        // - POST /api/documents/{id}/approve
        // - GET /api/documents/{id}/audit
        // - GET /api/documents/{id}/download
    }
    
    /// Test: Users without document_approver role cannot approve documents
    #[tokio::test]
    async fn approval_requires_approver_role() {
        // Create a test user without document_approver role
        // Attempt to approve a document
        // Verify 403 Forbidden response
    }
    
    /// Test: High-sensitivity domains require MFA for approval
    #[tokio::test]
    async fn high_sensitivity_approval_requires_mfa() {
        // Create a document in a high-sensitivity domain
        // Attempt approval without MFA claim
        // Verify 403 response with MFA required message
    }
    
    /// Test: Approval trail records approved_by user ID
    #[tokio::test]
    async fn approval_records_user_id() {
        // Approve a document with authenticated user
        // Verify audit trail contains user_id
        // Verify documents table updated with approved_by
    }
}
```

#### Document Creation Tests

```rust
#[cfg(test)]
mod creation_tests {
    use super::*;
    
    /// Test: POST /api/documents/create starts pipeline execution
    #[tokio::test]
    async fn create_starts_pipeline() {
        // Submit valid document creation request
        // Verify 200 response with document_id
        // Verify document state is Drafting in database
        // Verify pipeline executor is invoked
    }
    
    /// Test: POST /api/documents/create returns 400 for invalid domain_id
    #[tokio::test]
    async fn invalid_domain_returns_400() {
        // Submit request with non-existent domain_id
        // Verify 400 response with error details
    }
    
    /// Test: GET /api/documents/{id}/status returns current state
    #[tokio::test]
    async fn status_returns_current_state() {
        // Create a document
        // Query status endpoint
        // Verify response contains state, scores, requires_approval
    }
}
```

#### Approval Workflow Tests

```rust
#[cfg(test)]
mod approval_tests {
    use super::*;
    
    /// Test: POST /api/documents/{id}/approve requires approver role
    #[tokio::test]
    async fn approve_requires_role() {
        // Already covered in auth_tests
    }
    
    /// Test: POST /api/documents/{id}/approve transitions state to Approved
    #[tokio::test]
    async fn approve_transitions_state() {
        // Create document in PendingApproval state
        // Submit approval with authorized user
        // Verify state transitions to Approved
        // Verify approved_at timestamp set
    }
    
    /// Test: POST /api/documents/{id}/approve with rejection returns to Draft
    #[tokio::test]
    async fn reject_returns_to_draft() {
        // Create document in PendingApproval state
        // Submit rejection with comments
        // Verify state transitions to Draft
        // Verify approval_notes stored
    }
}
```

#### Audit Trail Tests

```rust
#[cfg(test)]
mod audit_tests {
    use super::*;
    
    /// Test: GET /api/documents/{id}/audit returns complete audit trail
    #[tokio::test]
    async fn audit_returns_complete_trail() {
        // Create and process a document through all agents
        // Query audit endpoint
        // Verify all agent actions recorded
        // Verify timestamps and details present
    }
    
    /// Test: Audit entries are ordered by timestamp
    #[tokio::test]
    async fn audit_ordered_by_timestamp() {
        // Verify audit trail response is DESC ordered by timestamp
    }
}
```

#### Template Management Tests

```rust
#[cfg(test)]
mod template_tests {
    use super::*;
    
    /// Test: GET /api/templates returns templates filtered by domain_id
    #[tokio::test]
    async fn templates_filter_by_domain() {
        // Create templates for multiple domains
        // Query without domain_id filter
        // Query with domain_id filter
        // Verify filtering works correctly
    }
    
    /// Test: POST /api/templates creates new template with validation
    #[tokio::test]
    async fn create_template_validates_input() {
        // Submit valid template
        // Verify 201 response with template_id
        // Submit template with duplicate domain_id + document_type
        // Verify 400 response
    }
}
```

#### Document Download Tests

```rust
#[cfg(test)]
mod download_tests {
    use super::*;
    
    /// Test: GET /api/documents/{id}/download returns correct format
    #[tokio::test]
    async fn download_returns_requested_format() {
        // Create a published document
        // Request download with format=odf
        // Verify Content-Type header
        // Verify binary content
    }
    
    /// Test: GET /api/documents/{id}/download defaults to Markdown
    #[tokio::test]
    async fn download_defaults_to_markdown() {
        // Request download without format parameter
        // Verify Markdown content returned
    }
    
    /// Test: GET /api/documents/{id}/download returns 404 for unpublished
    #[tokio::test]
    async fn download_unpublished_returns_404() {
        // Create document in Draft state
        // Request download
        // Verify 404 response
    }
}
```

#### OpenAPI Documentation Tests

```rust
#[cfg(test)]
mod openapi_tests {
    use super::*;
    
    /// Test: OpenAPI documentation generates correctly
    #[tokio::test]
    async fn openapi_docs_generate() {
        // Access /api-docs or /openapi.json endpoint
        // Verify all document endpoints documented
        // Verify schemas defined for request/response types
    }
}
```

---

## Implementation

### 5.1 API Route Structure

The API layer follows Axum patterns used elsewhere in IOU-Modern. Routes are organized by resource type with consistent error handling.

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/documents.rs`

```rust
//! Document creation and management API routes
//!
//! This module provides REST endpoints for:
//! - Creating new documents via the agent pipeline
//! - Querying document status
//! - Approving/rejecting documents
//! - Accessing audit trails
//! - Downloading generated documents

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use iou_core::document::{DocumentId, DocumentRequest, DocumentMetadata};
use iou_core::workflows::WorkflowStatus;

// Re-export the document router for main.rs registration
pub fn document_routes() -> Router<AppState> {
    Router::new()
        .route("/create", post(create_document))
        .route("/:id/status", get(get_status))
        .route("/:id/approve", post(approve_document))
        .route("/:id/audit", get(get_audit_trail))
        .route("/:id/download", get(download_document))
}
```

### 5.2 Document Creation Endpoint

**Request/Response Types:**

```rust
/// Request payload for document creation
#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub domain_id: String,
    pub document_type: String,
    pub context: std::collections::HashMap<String, String>,
}

/// Response for successful document creation
#[derive(Debug, Serialize)]
pub struct CreateDocumentResponse {
    pub document_id: Uuid,
    pub state: String,  // Maps to WorkflowStatus
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
}

/// Error response for invalid requests
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: String,
}
```

**Handler Implementation:**

```rust
/// POST /api/documents/create
///
/// Initiates the document creation pipeline. The request is validated,
/// a document record is created in Draft state, and the pipeline
/// executor is invoked asynchronously.
///
/// # Authentication
/// Requires valid JWT token (any authenticated user)
///
/// # Returns
/// - 200: Document creation initiated
/// - 400: Invalid request (unknown domain, missing required fields)
/// - 401: Authentication required
pub async fn create_document(
    State(state): State<AppState>,
    Json(req): Json<CreateDocumentRequest>,
    _auth: AuthenticatedUser,  // Extractor from existing auth middleware
) -> Result<Json<CreateDocumentResponse>, ErrorResponse> {
    // 1. Validate domain_id exists in domain_configs table
    let domain_config = state.db
        .get_domain_config(&req.domain_id)
        .await
        .map_err(|_| ErrorResponse {
            error: "Invalid domain_id".to_string(),
            details: format!("Domain '{}' not found", req.domain_id),
        })?;
    
    // 2. Validate document_type has a matching template
    let _template = state.db
        .get_active_template(&req.domain_id, &req.document_type)
        .await
        .map_err(|_| ErrorResponse {
            error: "Invalid document_type".to_string(),
            details: format!("No template found for type '{}' in domain '{}'", 
                           req.document_type, req.domain_id),
        })?;
    
    // 3. Create document record with unique ID
    let document_id = Uuid::new_v4();
    let document = DocumentMetadata {
        id: document_id,
        domain_id: req.domain_id.clone(),
        document_type: req.document_type.clone(),
        state: WorkflowStatus::Draft,
        current_version_key: String::new(),  // Will be set by pipeline
        previous_version_key: None,
        compliance_score: 0.0,
        confidence_score: 0.0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        // ... other fields
    };
    
    // 4. Persist document metadata
    state.db.create_document(&document).await?;
    
    // 5. Invoke pipeline executor (spawn task, don't await)
    let pipeline = state.pipeline.clone();
    tokio::spawn(async move {
        // Pipeline execution happens asynchronously
        // Results will update document state in database
    });
    
    Ok(Json(CreateDocumentResponse {
        document_id,
        state: "Drafting".to_string(),
        estimated_completion: None,  // Could be calculated based on historical data
    }))
}
```

### 5.3 Status Query Endpoint

```rust
/// GET /api/documents/{id}/status
///
/// Returns the current state of a document including scores,
/// current agent being processed, and approval requirements.
///
/// # Authentication
/// Requires valid JWT token
pub async fn get_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: AuthenticatedUser,
) -> Result<Json<DocumentStatusResponse>, ErrorResponse> {
    let document = state.db
        .get_document(id)
        .await
        .map_err(|_| ErrorResponse {
            error: "Document not found".to_string(),
            details: format!("No document with ID {}", id),
        })?;
    
    // Determine if approval is required based on trust_level
    let requires_approval = match document.trust_level {
        TrustLevel::Low => true,
        TrustLevel::Medium => document.compliance_score < domain_config.required_approval_threshold,
        TrustLevel::High => document.is_woo_relevant(),  // ALL Woo docs require approval
    };
    
    Ok(Json(DocumentStatusResponse {
        document_id: document.id,
        state: document.state.to_string(),
        current_agent: document.current_agent,
        compliance_score: document.compliance_score,
        confidence_score: document.confidence_score,
        requires_approval,
        errors: document.errors,
    }))
}

#[derive(Debug, Serialize)]
pub struct DocumentStatusResponse {
    pub document_id: Uuid,
    pub state: String,
    pub current_agent: Option<String>,
    pub compliance_score: f32,
    pub confidence_score: f32,
    pub requires_approval: bool,
    pub errors: Vec<String>,
}
```

### 5.4 Approval Endpoint

**Security Requirements:**
- Authentication: Required (valid JWT)
- Authorization: User must have `document_approver` role
- MFA: Required for high-sensitivity domains

```rust
/// POST /api/documents/{id}/approve
///
/// Approves or rejects a document pending human review.
///
/// # Authentication
/// Requires valid JWT token with `document_approver` role
///
/// # MFA
/// Required for high-sensitivity domains (trust_level = High)
///
/// # Returns
/// - 200: Approval/rejection processed
/// - 401: Authentication required
/// - 403: Insufficient permissions or MFA missing
/// - 404: Document not found
/// - 400: Document not in approvable state
pub async fn approve_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApprovalRequest>,
    auth: ApproverUser,  // Custom extractor that validates document_approver role
) -> Result<Json<ApprovalResponse>, ErrorResponse> {
    // 1. Load document and verify state
    let mut document = state.db.get_document(id).await?;
    
    if document.state != WorkflowStatus::Submitted && 
       document.state != WorkflowStatus::InReview {
        return Err(ErrorResponse {
            error: "Invalid state".to_string(),
            details: format!("Document is in {} state, cannot approve", document.state),
        });
    }
    
    // 2. Verify MFA for high-sensitivity domains
    if document.trust_level == TrustLevel::High && !auth.mfa_verified {
        return Err(ErrorResponse {
            error: "MFA required".to_string(),
            details: "High-sensitivity documents require MFA verification".to_string(),
        });
    }
    
    // 3. Process approval or rejection
    if req.approved {
        document.state = WorkflowStatus::Approved;
        document.approved_by = Some(auth.user_id);
        document.approved_at = Some(Utc::now());
        document.approval_notes = req.comments;
        
        // Trigger final processing (storage, publication)
        state.finalize_document(id).await?;
    } else {
        document.state = WorkflowStatus::Draft;  // Return for revision
        document.approval_notes = req.comments;
        
        // Log rejection for audit
        state.log_audit(id, "Rejection", &auth.user_id, req.comments).await?;
    }
    
    // 4. Persist changes
    state.db.update_document(&document).await?;
    
    Ok(Json(ApprovalResponse {
        document_id: id,
        state: document.state.to_string(),
        approved_at: document.approved_at,
        approved_by: document.approved_by,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    pub approved: bool,
    pub comments: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApprovalResponse {
    pub document_id: Uuid,
    pub state: String,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub approved_by: Option<String>,
}
```

### 5.5 Audit Trail Endpoint

```rust
/// GET /api/documents/{id}/audit
///
/// Returns the complete audit trail for a document, showing
/// all agent actions, state transitions, and human decisions.
///
/// # Authentication
/// Requires valid JWT token
pub async fn get_audit_trail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: AuthenticatedUser,
) -> Result<Json<AuditTrailResponse>, ErrorResponse> {
    // Verify document exists
    let _document = state.db.get_document(id).await?;
    
    // Fetch audit entries ordered by timestamp DESC
    let audit_entries = state.db
        .get_audit_trail(id)
        .await?;
    
    Ok(Json(AuditTrailResponse {
        document_id: id,
        audit_trail: audit_entries
            .into_iter()
            .map(|entry| AuditEntryDto {
                agent: entry.agent_name,
                action: entry.action,
                timestamp: entry.timestamp,
                details: entry.details,
            })
            .collect(),
    }))
}

#[derive(Debug, Serialize)]
pub struct AuditTrailResponse {
    pub document_id: Uuid,
    pub audit_trail: Vec<AuditEntryDto>,
}

#[derive(Debug, Serialize)]
pub struct AuditEntryDto {
    pub agent: String,
    pub action: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: serde_json::Value,
}
```

### 5.6 Template Management Endpoints

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/templates.rs`

```rust
//! Template management API routes
//!
//! Provides CRUD operations for document templates.

use axum::{...};

pub fn template_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_templates))
        .route("/", post(create_template))
        .route("/:id", get(get_template))
        .route("/:id", put(update_template))
}

/// GET /api/templates?domain_id={id}
///
/// Lists available templates, optionally filtered by domain.
pub async fn list_templates(
    State(state): State<AppState>,
    Query(params): Query<ListTemplatesParams>,
    _auth: AuthenticatedUser,
) -> Result<Json<TemplateListResponse>, ErrorResponse> {
    let templates = if let Some(domain_id) = params.domain_id {
        state.db.get_templates_by_domain(&domain_id).await?
    } else {
        state.db.get_all_templates().await?
    };
    
    Ok(Json(TemplateListResponse {
        templates: templates.into_iter().map(|t| TemplateDto {
            id: t.id,
            name: t.name,
            domain_id: t.domain_id,
            document_type: t.document_type,
            version: t.version,
        }).collect(),
    }))
}

/// POST /api/templates
///
/// Creates a new template. Requires template_manager role.
pub async fn create_template(
    State(state): State<AppState>,
    Json(req): Json<CreateTemplateRequest>,
    auth: TemplateManagerUser,  // Requires template_manager role
) -> Result<impl IntoResponse, ErrorResponse> {
    // Validate template content parses as valid Tera template
    tera::Template::new(req.name.clone(), &req.content)
        .map_err(|e| ErrorResponse {
            error: "Invalid template".to_string(),
            details: e.to_string(),
        })?;
    
    // Check for duplicate domain_id + document_type
    if state.db.template_exists(&req.domain_id, &req.document_type).await? {
        return Err(ErrorResponse {
            error: "Template exists".to_string(),
            details: "Template for this domain and type already exists".to_string(),
        });
    }
    
    let template_id = state.db.create_template(req).await?;
    
    Ok((StatusCode::CREATED, Json(CreateTemplateResponse {
        template_id,
        version: 1,
    })))
}
```

### 5.7 Document Download Endpoint

```rust
/// GET /api/documents/{id}/download?format=odf|pdf|md
///
/// Downloads a published document in the specified format.
///
/// # Authentication
/// Requires valid JWT token
///
/// # Returns
/// - 200: Binary file content
/// - 404: Document not found or not published
pub async fn download_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<DownloadParams>,
    _auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ErrorResponse> {
    let document = state.db.get_document(id).await?;
    
    // Only published documents can be downloaded
    if document.state != WorkflowStatus::Published {
        return Err(ErrorResponse {
            error: "Document not available".to_string(),
            details: "Document must be published before download".to_string(),
        });
    }
    
    let format = params.format.unwrap_or(DocumentFormat::Markdown);
    
    // Retrieve from S3 storage
    let storage_key = document.get_storage_key_for_format(&format);
    let content = state.storage.get(&storage_key).await?;
    
    let content_type = match format {
        DocumentFormat::Markdown => "text/markdown",
        DocumentFormat::ODF => "application/vnd.oasis.opendocument.text",
        DocumentFormat::PDF => "application/pdf",
    };
    
    Ok((
        [(axum::http::header::CONTENT_TYPE, content_type)],
        content,
    ))
}

#[derive(Debug, Deserialize)]
pub struct DownloadParams {
    pub format: Option<DocumentFormat>,
}
```

### 5.8 Authentication Extractors

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/extractors/auth.rs`

```rust
//! Authentication and authorization extractors for Axum

use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use serde_json::json;

/// Basic authenticated user (any valid JWT)
pub struct AuthenticatedUser {
    pub user_id: String,
    pub roles: Vec<String>,
}

/// User with document_approver role
pub struct ApproverUser {
    pub user_id: String,
    pub mfa_verified: bool,
}

/// User with template_manager role
pub struct TemplateManagerUser {
    pub user_id: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract JWT from Authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        
        // Validate JWT and extract claims
        // (reuse existing JWT validation from iou-api)
        let claims = validate_jwt(bearer.token())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        
        Ok(AuthenticatedUser {
            user_id: claims.sub,
            roles: claims.roles,
        })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for ApproverUser
where
    S: Send + Sync,
{
    type Rejection = Json<serde_json::Value>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = AuthenticatedUser::from_request_parts(parts, state)
            .await
            .map_err(|_| Json(json!({"error": "Authentication required"})))?;
        
        if !auth.roles.contains(&"document_approver".to_string()) {
            return Err(Json(json!({
                "error": "Insufficient permissions",
                "details": "User lacks 'document_approver' role"
            })));
        }
        
        // Check MFA claim for high-sensitivity approval
        let mfa_verified = auth.roles.contains(&"mfa_verified".to_string());
        
        Ok(ApproverUser {
            user_id: auth.user_id,
            mfa_verified,
        })
    }
}
```

### 5.9 OpenAPI Documentation

The API should expose OpenAPI documentation for all endpoints. Use existing patterns from IOU-Modern:

```rust
// In main.rs or a dedicated docs module
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        create_document,
        get_status,
        approve_document,
        get_audit_trail,
        download_document,
        list_templates,
        create_template,
    ),
    components(schemas(
        CreateDocumentRequest,
        CreateDocumentResponse,
        DocumentStatusResponse,
        ApprovalRequest,
        ApprovalResponse,
        AuditTrailResponse,
        ErrorResponse,
    )),
    tags(
        (name = "documents", description = "Document creation and management"),
        (name = "templates", description = "Template management"),
    )
)]
struct ApiDoc;

// Register in router
let app = Router::new()
    .merge(document_routes())
    .merge(template_routes())
    .merge(SwaggerUi::new("/api-docs").url("/openapi.json", ApiDoc::openapi()));
```

---

## Integration Points

### Existing Authentication Middleware

The document API should integrate with the existing JWT-based authentication in `iou-api/src/middleware/auth.rs`. Do not create a new authentication system - reuse existing extractors and validation logic.

### Existing Workflow State Management

The `DocumentState` type is an alias for `WorkflowStatus` from `iou-core/src/workflows.rs`. Ensure all state transitions use the existing state machine patterns.

### Existing Database Patterns

Use the existing DuckDB connection pool patterns from `iou-core/src/db.rs`. Document queries should follow the same patterns as workflow and compliance queries.

---

## Error Handling

All API endpoints should return consistent error responses:

```rust
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub details: String,
    pub code: Option<String>,  // For programmatic error handling
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.code.as_deref() {
            Some("NOT_FOUND") => StatusCode::NOT_FOUND,
            Some("UNAUTHORIZED") => StatusCode::UNAUTHORIZED,
            Some("FORBIDDEN") => StatusCode::FORBIDDEN,
            Some("INVALID_INPUT") => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        (status, Json(self)).into_response()
    }
}
```

---

## API Summary Table

| Method | Endpoint | Auth | Required Role | Description |
|--------|----------|------|---------------|-------------|
| POST | /api/documents/create | Required | Any user | Create new document |
| GET | /api/documents/{id}/status | Required | Any user | Get document status |
| POST | /api/documents/{id}/approve | Required | document_approver | Approve/reject document |
| GET | /api/documents/{id}/audit | Required | Any user | Get audit trail |
| GET | /api/documents/{id}/download | Required | Any user | Download document |
| GET | /api/templates | Required | Any user | List templates |
| POST | /api/templates | Required | template_manager | Create template |
| GET | /api-docs | Optional | - | OpenAPI documentation |

---

## Implementation Checklist

- [ ] Create document routes module (`documents.rs`)
- [ ] Create template routes module (`templates.rs`)
- [ ] Implement authentication extractors (or verify existing ones suffice)
- [ ] Implement document creation endpoint
- [ ] Implement status query endpoint
- [ ] Implement approval endpoint with role checking
- [ ] Implement audit trail endpoint
- [ ] Implement document download endpoint
- [ ] Implement template management endpoints
- [ ] Register routes in main.rs
- [ ] Add OpenAPI documentation
- [ ] Write integration tests for all endpoints
- [ ] Verify authentication/authorization works correctly
- [ ] Test error handling for all failure modes