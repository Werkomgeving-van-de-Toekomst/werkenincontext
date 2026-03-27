Now I have sufficient context to create the section content. Let me generate it:

---

# Section 9: API Endpoints

## Overview

This section implements the HTTP API layer for the enhanced workflow features, providing REST endpoints for managing multi-stage approval workflows, delegations, and document version history. The API layer exposes the functionality implemented in earlier sections (multi-stage engine, delegation system, SLA/escalation, and version storage) to frontend clients and external integrations.

## Dependencies

This section depends on the following sections being completed first:

- **section-04-multi-stage-engine**: Provides the stage execution logic and state machine
- **section-06-delegation-system**: Provides delegation CRUD and resolution
- **section-07-sla-escalation**: Provides SLA calculator and escalation service
- **section-08-version-storage**: Provides version storage and diff generation

## Files to Create

| File | Purpose |
|------|---------|
| `crates/iou-api/src/routes/workflow_stages.rs` | Workflow stage API endpoints |
| `crates/iou-api/src/routes/delegations.rs` | Delegation management endpoints |
| `crates/iou-api/src/routes/versions.rs` | Version history and diff endpoints |
| `crates/iou-api/tests/routes/workflow_stages.rs` | Stage API tests |
| `crates/iou-api/tests/routes/delegations.rs` | Delegation API tests |
| `crates/iou-api/tests/routes/versions.rs` | Version API tests |

## Files to Modify

| File | Changes |
|------|---------|
| `crates/iou-api/src/routes/mod.rs` | Add module declarations for new route files |
| `crates/iou-api/src/main.rs` | Mount new routers to the main application |

---

## Part 1: Workflow Stages API

### File: `crates/iou-api/src/routes/workflow_stages.rs`

This module implements endpoints for managing multi-stage approval workflows on documents.

**Router Function Signature:**
```rust
use axum::{Router, routing::{get, post}, extract::{Path, State, Extension}, Json};
use uuid::Uuid;
use std::sync::Arc;

use crate::db::Database;
use crate::error::ApiError;
use iou_orchestrator::stage_executor::StageExecutor;
use iou_core::workflows::multi_stage::{StageInstance, ApprovalDecision};

pub fn workflow_stages_router() -> Router<Arc<Database>> {
    Router::new()
        .route("/api/documents/:id/stages", get(get_stages))
        .route("/api/documents/:id/stages/:stage_id", get(get_stage))
        .route("/api/documents/:id/stages/:stage_id/approve", post(approve_stage))
        .route("/api/documents/:id/stages/:stage_id/reject", post(reject_stage))
        .route("/api/documents/:id/stages/:stage_id/delegate", post(delegate_stage))
}
```

**Request/Response Types:**
```rust
/// Summary view of an approval stage
#[derive(Debug, Serialize)]
pub struct StageView {
    pub stage_id: String,
    pub stage_name: String,
    pub stage_order: i32,
    pub status: String,  // "pending", "in_progress", "completed", "skipped", "expired"
    pub approval_type: String,  // "sequential", "parallel_any", "parallel_all", "parallel_majority"
    pub approvers: Vec<ApproverView>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub sla_hours: i32,
}

/// Approver information for display
#[derive(Debug, Serialize)]
pub struct ApproverView {
    pub user_id: Uuid,
    pub user_name: Option<String>,
    pub status: String,  // "pending", "approved", "rejected", "delegated"
    pub responded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub delegated_from: Option<Uuid>,  // Original approver if delegated
}

/// Detailed stage information with all approvals
#[derive(Debug, Serialize)]
pub struct StageDetailView {
    pub base: StageView,
    pub approvals_received: Vec<ApprovalResponseView>,
    pub escalation_count: i32,
    pub current_escalation_level: Option<i32>,
}

/// Individual approval response for display
#[derive(Debug, Serialize)]
pub struct ApprovalResponseView {
    pub approver_id: Uuid,
    pub delegated_from: Option<Uuid>,
    pub decision: String,  // "approved", "rejected", "delegated"
    pub comment: Option<String>,
    pub responded_at: chrono::DateTime<chrono::Utc>,
}

/// Request to approve a stage
#[derive(Debug, Deserialize)]
pub struct ApproveStageRequest {
    pub comment: Option<String>,
}

/// Request to reject a stage
#[derive(Debug, Deserialize)]
pub struct RejectStageRequest {
    pub reason: String,
    pub comment: Option<String>,
}

/// Request to delegate approval to another user
#[derive(Debug, Deserialize)]
pub struct DelegateStageRequest {
    pub to_user_id: Uuid,
    pub reason: Option<String>,
}
```

**Endpoint Handlers:**

```rust
/// GET /api/documents/:id/stages
/// 
/// Returns all approval stages for a document, ordered by stage_order.
/// 
/// Response codes:
/// - 200: Success, returns array of stages
/// - 404: Document not found
/// - 403: User not authorized to view document
async fn get_stages(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<Uuid>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<StageView>>, ApiError> {
    // Verify document exists and user has access
    // Query document_approval_stages for the document
    // For each stage, query document_approvals for approver statuses
    // Return ordered by stage_order
}

/// GET /api/documents/:id/stages/:stage_id
/// 
/// Returns detailed information about a specific stage including all approvals.
/// 
/// Response codes:
/// - 200: Success, returns stage detail
/// - 404: Document or stage not found
async fn get_stage(
    Extension(db): Extension<Arc<Database>>,
    Extension(executor): Extension<Arc<StageExecutor>>,
    Path((id, stage_id)): Path<(Uuid, String)>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<StageDetailView>, ApiError> {
    // Verify document exists and user has access
    // Query stage instance from document_approval_stages
    // Query all approvals from document_approvals
    // Query escalation count from approval_escalations
    // Return detailed view
}

/// POST /api/documents/:id/stages/:stage_id/approve
/// 
/// Records an approval for the current stage. If this approval completes the
/// quorum, the stage transitions to the next stage or final approved state.
/// 
/// Request body: { "comment": "Optional comment" }
/// 
/// Response codes:
/// - 200: Approval recorded, returns updated stage view
/// - 404: Document or stage not found
/// - 403: User is not an authorized approver for this stage
/// - 409: User has already voted on this stage
/// - 400: Stage is not in approvable state
async fn approve_stage(
    Extension(db): Extension<Arc<Database>>,
    Extension(executor): Extension<Arc<StageExecutor>>,
    Path((id, stage_id)): Path<(Uuid, String)>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<ApproveStageRequest>,
) -> Result<Json<StageView>, ApiError> {
    // Verify user is authorized (is in approvers list)
    // Check if user has already voted
    // Record approval via executor.record_approval()
    // If stage completes, trigger next stage transition
    // Send WebSocket notification to relevant parties
    // Return updated stage view
}

/// POST /api/documents/:id/stages/:stage_id/reject
/// 
/// Records a rejection for the current stage. The document workflow
/// transitions to a failure state and no further approvals are accepted.
/// 
/// Request body: { "reason": "Required reason", "comment": "Optional comment" }
/// 
/// Response codes:
/// - 200: Rejection recorded, returns updated document state
/// - 404: Document or stage not found
/// - 403: User is not an authorized approver for this stage
/// - 409: User has already voted on this stage
async fn reject_stage(
    Extension(db): Extension<Arc<Database>>,
    Extension(executor): Extension<Arc<StageExecutor>>,
    Path((id, stage_id)): Path<(Uuid, String)>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<RejectStageRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Verify user is authorized
    // Record rejection via executor.record_approval()
    // Transition document to failed/approval_rejected state
    // Send WebSocket notification
    // Return document state
}

/// POST /api/documents/:id/stages/:stage_id/delegate
/// 
/// Delegates approval authority to another user. The original approver
/// remains in the audit trail but the delegated user must approve.
/// 
/// Request body: { "to_user_id": "uuid", "reason": "Optional reason" }
/// 
/// Response codes:
/// - 200: Delegation recorded, returns updated stage view
/// - 404: Document or stage not found
/// - 403: User is not an authorized approver for this stage
/// - 400: Invalid delegation (self, circular, etc.)
async fn delegate_stage(
    Extension(db): Extension<Arc<Database>>,
    Extension(executor): Extension<Arc<StageExecutor>>,
    Path((id, stage_id)): Path<(Uuid, String)>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<DelegateStageRequest>,
) -> Result<Json<StageView>, ApiError> {
    // Verify user is authorized
    // Create temporary delegation via DelegationService
    // Update stage approvers to include delegated user
    // Send WebSocket notification to delegated user
    // Return updated stage view
}
```

---

## Part 2: Delegations API

### File: `crates/iou-api/src/routes/delegations.rs`

This module implements endpoints for managing approval delegations.

**Router Function Signature:**
```rust
use axum::{Router, routing::{get, post, delete}, extract::{Path, State, Extension, Query}, Json};
use uuid::Uuid;
use std::sync::Arc;

use crate::db::Database;
use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use iou_core::delegation::{Delegation, DelegationType};
use iou_core::delegation::service::DelegationService;

pub fn delegations_router() -> Router<Arc<Database>> {
    Router::new()
        .route("/api/delegations", get(list_delegations).post(create_delegation))
        .route("/api/delegations/:id", get(get_delegation).delete(revoke_delegation))
        .route("/api/delegations/:id/reactivate", post(reactivate_delegation))
        .route("/api/users/:user_id/delegations", get(user_delegations))
}
```

**Request/Response Types:**
```rust
/// View of a delegation for API responses
#[derive(Debug, Serialize)]
pub struct DelegationView {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub from_user_name: Option<String>,
    pub to_user_id: Uuid,
    pub to_user_name: Option<String>,
    pub delegation_type: String,  // "temporary", "permanent", "bulk"
    pub document_types: Vec<String>,
    pub document_id: Option<Uuid>,
    pub starts_at: chrono::DateTime<chrono::Utc>,
    pub ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request to create a delegation
#[derive(Debug, Deserialize)]
pub struct CreateDelegationRequest {
    pub to_user_id: Uuid,
    pub delegation_type: String,  // "temporary", "permanent", "bulk"
    pub document_types: Option<Vec<String>>,  // Required for bulk, optional for others
    pub document_id: Option<Uuid>,  // For single-document delegation
    pub starts_at: Option<chrono::DateTime<chrono::Utc>>,  // Defaults to now
    pub ends_at: Option<chrono::DateTime<chrono::Utc>>,  // Required for temporary
}

/// Request to reactivate a delegation
#[derive(Debug, Deserialize)]
pub struct ReactivateDelegationRequest {
    pub ends_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Query parameters for listing delegations
#[derive(Debug, Deserialize)]
pub struct ListDelegationsQuery {
    pub is_active: Option<bool>,
    pub delegation_type: Option<String>,
}

/// Response for listing delegations
#[derive(Debug, Serialize)]
pub struct DelegationsListResponse {
    pub created: Vec<DelegationView>,   // Delegations created by user
    pub received: Vec<DelegationView>,  // Delegations received by user
}
```

**Endpoint Handlers:**

```rust
/// GET /api/delegations
/// 
/// Lists all delegations for the authenticated user - both those they
/// created and those they received.
/// 
/// Query params: is_active (bool), delegation_type (string)
/// 
/// Response codes:
/// - 200: Success, returns delegations grouped by created/received
/// - 401: Not authenticated
async fn list_delegations(
    Extension(db): Extension<Arc<Database>>,
    Extension(delegation_service): Extension<Arc<DelegationService>>,
    Extension(auth): Extension<AuthContext>,
    Query(params): Query<ListDelegationsQuery>,
) -> Result<Json<DelegationsListResponse>, ApiError> {
    // Query delegations where from_user_id = auth.user_id
    // Query delegations where to_user_id = auth.user_id
    // Apply filters from query params
    // Return grouped response
}

/// GET /api/delegations/:id
/// 
/// Returns detailed information about a specific delegation.
/// 
/// Response codes:
/// - 200: Success, returns delegation detail
/// - 404: Delegation not found
/// - 403: User not authorized to view this delegation
async fn get_delegation(
    Extension(db): Extension<Arc<Database>>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<DelegationView>, ApiError> {
    // Query delegation by ID
    // Verify user is either from_user or to_user or admin
    // Return delegation view with user names resolved
}

/// POST /api/delegations
/// 
/// Creates a new delegation.
/// 
/// Request body: CreateDelegationRequest
/// 
/// Response codes:
/// - 201: Delegation created, returns delegation view
/// - 400: Invalid request (self-delegation, circular, invalid dates)
/// - 404: Target user not found
/// - 422: Validation error (exceeds max delegations, etc.)
async fn create_delegation(
    Extension(db): Extension<Arc<Database>>,
    Extension(delegation_service): Extension<Arc<DelegationService>>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CreateDelegationRequest>,
) -> Result<Json<DelegationView>, ApiError> {
    // Validate from_user != to_user (no self-delegation)
    // Validate ends_at > starts_at for temporary delegations
    // Check for circular delegation chains
    // Verify to_user exists
    // Create delegation via service
    // Return created delegation
}

/// DELETE /api/delegations/:id
/// 
/// Revokes an active delegation. Only the creator (from_user) or an admin
/// can revoke a delegation.
/// 
/// Response codes:
/// - 204: Delegation revoked successfully
/// - 404: Delegation not found
/// - 403: User not authorized to revoke this delegation
async fn revoke_delegation(
    Extension(db): Extension<Arc<Database>>,
    Extension(delegation_service): Extension<Arc<DelegationService>>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, ApiError> {
    // Query delegation
    // Verify auth.user_id == from_user_id or user is admin
    // Revoke via service.revoke_delegation()
    // Return 204 No Content
}

/// POST /api/delegations/:id/reactivate
/// 
/// Reactivates a previously expired or revoked delegation.
/// 
/// Request body: { "ends_at": "optional new end date" }
/// 
/// Response codes:
/// - 200: Delegation reactivated, returns delegation view
/// - 404: Delegation not found
/// - 403: User not authorized
async fn reactivate_delegation(
    Extension(db): Extension<Arc<Database>>,
    Extension(delegation_service): Extension<Arc<DelegationService>>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<ReactivateDelegationRequest>,
) -> Result<Json<DelegationView>, ApiError> {
    // Query delegation
    // Verify auth.user_id == from_user_id or user is admin
    // Update is_active = true, optionally update ends_at
    // Return updated delegation
}

/// GET /api/users/:user_id/delegations
/// 
/// Lists all delegations for a specific user. Admin only endpoint.
/// 
/// Response codes:
/// - 200: Success, returns user's delegations
/// - 403: User is not an admin
async fn user_delegations(
    Extension(db): Extension<Arc<Database>>,
    Extension(auth): Extension<AuthContext>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<DelegationsListResponse>, ApiError> {
    // Verify user has admin role
    // Query delegations for the specified user
    // Return grouped response
}
```

---

## Part 3: Versions API

### File: `crates/iou-api/src/routes/versions.rs`

This module implements endpoints for document version history and diff generation.

**Router Function Signature:**
```rust
use axum::{Router, routing::{get, post}, extract::{Path, State, Extension, Query}, Json};
use uuid::Uuid;
use std::sync::Arc;

use crate::db::Database;
use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use iou_core::versions::service::VersionService;
use iou_core::diff::generator::{DiffGenerator, DiffFormat};
use iou_core::storage::S3Client;

pub fn versions_router() -> Router<Arc<Database>> {
    Router::new()
        .route("/api/documents/:id/versions", get(list_versions))
        .route("/api/documents/:id/versions/diff", get(get_diff))
        .route("/api/documents/:id/versions/:version_id", get(get_version))
        .route("/api/documents/:id/versions/:version_id/restore", post(restore_version))
}
```

**Request/Response Types:**
```rust
/// View of a document version
#[derive(Debug, Serialize)]
pub struct VersionView {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version_number: i32,  // v1, v2, v3...
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Uuid,
    pub created_by_name: Option<String>,
    pub change_summary: String,
    pub is_compressed: bool,
    pub parent_version_id: Option<Uuid>,
    pub current: bool,  // True if this is the current version
}

/// Query parameters for diff generation
#[derive(Debug, Deserialize)]
pub struct DiffQuery {
    pub from: Option<String>,   // Version ID or "current"
    pub to: Option<String>,     // Version ID or "current"
    pub format: Option<String>, // "unified", "side_by_side", "inline"
}

/// Diff response
#[derive(Debug, Serialize)]
pub struct DiffResponse {
    pub from_version: String,
    pub to_version: String,
    pub format: String,
    pub changes: Vec<DiffChangeView>,
    pub summary: DiffSummary,
}

/// Individual diff change
#[derive(Debug, Serialize)]
pub struct DiffChangeView {
    pub change_type: String,  // "unchanged", "inserted", "deleted", "replaced"
    pub old_text: Option<String>,
    pub new_text: Option<String>,
    pub line_number: Option<usize>,
}

/// Summary statistics for a diff
#[derive(Debug, Serialize)]
pub struct DiffSummary {
    pub additions: i32,
    pub deletions: i32,
    pub unchanged: i32,
}

/// Request to restore a version
#[derive(Debug, Deserialize)]
pub struct RestoreVersionRequest {
    pub comment: Option<String>,  // Optional comment explaining the restore
}

/// Response after restoring a version
#[derive(Debug, Serialize)]
pub struct RestoreVersionResponse {
    pub document_id: Uuid,
    pub restored_from_version: Uuid,
    pub new_version_id: Uuid,
    pub restored_at: chrono::DateTime<chrono::Utc>,
    pub restored_by: Uuid,
}
```

**Endpoint Handlers:**

```rust
/// GET /api/documents/:id/versions
/// 
/// Lists all versions of a document, ordered by created_at DESC (newest first).
/// 
/// Response codes:
/// - 200: Success, returns array of versions
/// - 404: Document not found
/// - 403: User not authorized to view document
async fn list_versions(
    Extension(db): Extension<Arc<Database>>,
    Extension(version_service): Extension<Arc<VersionService>>,
    Path(id): Path<Uuid>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<VersionView>>, ApiError> {
    // Verify document exists and user has access
    // Query versions from document_versions table
    // Mark current version (where version_key matches document.current_version_key)
    // Return ordered by created_at DESC
}

/// GET /api/documents/:id/versions/diff
/// 
/// Generates a diff between two document versions.
/// 
/// Query params:
/// - from: Version ID to compare from (defaults to current - 1)
/// - to: Version ID to compare to (defaults to current)
/// - format: "unified", "side_by_side", "inline" (defaults to "unified")
/// 
/// Response codes:
/// - 200: Success, returns diff
/// - 404: Document or version not found
/// - 400: Invalid format or version IDs
async fn get_diff(
    Extension(db): Extension<Arc<Database>>,
    Extension(version_service): Extension<Arc<VersionService>>,
    Extension(diff_generator): Extension<Arc<DiffGenerator>>,
    Extension(s3_client): Extension<Arc<S3Client>>,
    Path(id): Path<Uuid>,
    Extension(auth): Extension<AuthContext>,
    Query(params): Query<DiffQuery>,
) -> Result<Json<DiffResponse>, ApiError> {
    // Verify document access
    // Resolve from/to version IDs (handle "current" default)
    // Fetch version content from S3
    // Decompress if necessary
    // Generate diff using DiffGenerator
    // Return formatted diff
}

/// GET /api/documents/:id/versions/:version_id
/// 
/// Returns detailed information about a specific version, including
/// the diff summary if available.
/// 
/// Response codes:
/// - 200: Success, returns version detail
/// - 404: Version not found
async fn get_version(
    Extension(db): Extension<Arc<Database>>,
    Extension(version_service): Extension<Arc<VersionService>>,
    Path((id, version_id)): Path<(Uuid, Uuid)>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<VersionView>, ApiError> {
    // Query version by ID
    // Verify version belongs to specified document
    // Return version view
}

/// POST /api/documents/:id/versions/:version_id/restore
/// 
/// Restores a previous version of the document. This creates a NEW version
/// recording the restore operation (versions are never overwritten).
/// 
/// Request body: { "comment": "Optional comment" }
/// 
/// Response codes:
/// - 200: Version restored, returns new version info
/// - 404: Version not found
/// - 403: User not authorized to modify document
/// - 400: Cannot restore to current version
async fn restore_version(
    Extension(db): Extension<Arc<Database>>,
    Extension(version_service): Extension<Arc<VersionService>>,
    Path((id, version_id)): Path<(Uuid, Uuid)>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<RestoreVersionRequest>,
) -> Result<Json<RestoreVersionResponse>, ApiError> {
    // Verify document access and edit permission
    // Verify version_id != current version
    // Fetch version content via version_service.restore_version()
    // Update document with restored content
    // Create audit trail entry for restore
    // Send WebSocket notification
    // Return new version info
}
```

---

## Part 4: Main API Integration

### File: `crates/iou-api/src/routes/mod.rs`

Add the new module exports:

```rust
// ... existing imports ...

pub mod workflow_stages;
pub mod delegations;
pub mod versions;

// Re-export new router functions
pub use workflow_stages::workflow_stages_router;
pub use delegations::delegations_router;
pub use versions::versions_router;
```

### File: `crates/iou-api/src/main.rs`

Mount the new routers in the main application. Locate the API router construction (around line 186 in the existing code):

```rust
// In main.rs, where the API router is constructed:
let api = Router::new()
    // ... existing routes ...
    .nest("/workflow_stages", routes::workflow_stages_router())
    .nest("/delegations", routes::delegations_router())
    .nest("/versions", routes::versions_router())
    // ... rest of configuration ...
```

---

## Tests

### File: `crates/iou-api/tests/routes/workflow_stages.rs`

```rust
//! Tests for workflow stages API endpoints

use std::sync::Arc;
use uuid::Uuid;
use axum::http::StatusCode;
use axum::body::Body;
use tower::ServiceExt;

use iou_api::{routes::workflow_stages, error::ApiError};
use iou_api::tests::common::{random_document_id, random_user_id};

/// Test: GET /api/documents/:id/stages returns all stages for document
#[tokio::test]
async fn get_stages_returns_all_stages() {
    // Setup: Create test database with document having 3 stages
    // Execute: GET /api/documents/{id}/stages
    // Assert: Status 200
    // Assert: Response contains 3 stages
    // Assert: Stages ordered by stage_order ASC
}

/// Test: GET /api/documents/:id/stages includes stage status and approvers
#[tokio::test]
async fn get_stages_includes_status_and_approvers() {
    // Setup: Create document with stages in various states
    // Execute: GET /api/documents/{id}/stages
    // Assert: Each stage has status field
    // Assert: Each stage has approvers array
    // Assert: Approver status correctly reflected
}

/// Test: GET /api/documents/:id/stages/:stage_id returns detailed stage info
#[tokio::test]
async fn get_stage_returns_detailed_info() {
    // Setup: Create document with stage
    // Execute: GET /api/documents/{id}/stages/{stage_id}
    // Assert: Status 200
    // Assert: Response includes approvals_received array
    // Assert: Response includes escalation_count
}

/// Test: GET /api/documents/:id/stages/:stage_id includes approvals_received array
#[tokio::test]
async fn get_stage_includes_approvals_received() {
    // Setup: Create stage with 2 existing approvals
    // Execute: GET /api/documents/{id}/stages/{stage_id}
    // Assert: approvals_received has 2 entries
    // Assert: Each approval has approver_id, decision, responded_at
}

/// Test: POST /api/documents/:id/stages/:stage_id/approve records approval
#[tokio::test]
async fn approve_stage_records_approval() {
    // Setup: Create document with in-progress stage, user is approver
    // Execute: POST /api/documents/{id}/stages/{stage_id}/approve
    // Body: { "comment": "Looks good" }
    // Assert: Status 200
    // Assert: Approval recorded in document_approvals
    // Assert: Audit trail entry created
}

/// Test: POST /api/documents/:id/stages/:stage_id/approve requires authentication
#[tokio::test]
async fn approve_stage_requires_authentication() {
    // Setup: No auth context
    // Execute: POST /api/documents/{id}/stages/{stage_id}/approve
    // Assert: Status 401 or 403
}

/// Test: POST /api/documents/:id/stages/:stage_id/approve rejects non-approver
#[tokio::test]
async fn approve_stage_rejects_non_approver() {
    // Setup: Create stage, user is NOT in approvers list
    // Execute: POST /api/documents/{id}/stages/{stage_id}/approve
    // Assert: Status 403
    // Assert: Error message indicates not authorized
}

/// Test: POST /api/documents/:id/stages/:stage_id/reject records rejection
#[tokio::test]
async fn reject_stage_records_rejection() {
    // Setup: Create document with in-progress stage, user is approver
    // Execute: POST /api/documents/{id}/stages/{stage_id}/reject
    // Body: { "reason": "Incomplete", "comment": "More detail needed" }
    // Assert: Status 200
    // Assert: Rejection recorded
    // Assert: Document state changed to failed
}

/// Test: POST /api/documents/:id/stages/:stage_id/reject prevents further approvals
#[tokio::test]
async fn reject_prevents_further_approvals() {
    // Setup: Stage with 1 approval
    // Execute: POST reject
    // Execute: POST approve (by different approver)
    // Assert: Second request returns 400 or 409
    // Assert: No additional approval recorded
}

/// Test: POST approval returns 409 when approver has already voted
#[tokio::test]
async fn approve_returns_conflict_when_already_voted() {
    // Setup: User has already approved this stage
    // Execute: POST /api/documents/{id}/stages/{stage_id}/approve
    // Assert: Status 409
    // Assert: Error message indicates already voted
}

/// Test: POST approval with delegation records delegated_from in audit trail
#[tokio::test]
async fn approve_with_delegation_records_delegated_from() {
    // Setup: User is acting as delegate for original approver
    // Execute: POST approve
    // Assert: Approval recorded
    // Assert: delegated_from field contains original approver ID
}

/// Test: Stage transitions to next stage when quorum met
#[tokio::test]
async fn stage_transitions_when_quorum_met() {
    // Setup: ParallelAll stage with 2 approvers, 1 already approved
    // Execute: POST approve (second approver)
    // Assert: Current stage marked completed
    // Assert: Next stage started
}

/// Test: Document approved when all stages complete
#[tokio::test]
async fn document_approved_when_all_stages_complete() {
    // Setup: Document with 1 stage, 1 of 1 approver
    // Execute: POST approve (final approval)
    // Assert: All stages completed
    // Assert: Document state = Approved
}

/// Test: Get stages for non-existent document returns 404
#[tokio::test]
async fn get_stages_nonexistent_document_returns_404() {
    // Setup: Random document ID
    // Execute: GET /api/documents/{id}/stages
    // Assert: Status 404
}

/// Test: Get non-existent stage returns 404
#[tokio::test]
async fn get_nonexistent_stage_returns_404() {
    // Setup: Valid document, invalid stage_id
    // Execute: GET /api/documents/{id}/stages/invalid_stage
    // Assert: Status 404
}
```

### File: `crates/iou-api/tests/routes/delegations.rs`

```rust
//! Tests for delegation API endpoints

use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Duration};
use axum::http::StatusCode;

use iou_api::tests::common::{random_user_id};

/// Test: GET /api/delegations returns user's created and received delegations
#[tokio::test]
async fn list_delegations_returns_created_and_received() {
    // Setup: Create 2 delegations from user, 1 to user
    // Execute: GET /api/delegations
    // Assert: Status 200
    // Assert: created array has 2 entries
    // Assert: received array has 1 entry
}

/// Test: GET /api/delegations filters by is_active status
#[tokio::test]
async fn list_delegations_filters_by_active() {
    // Setup: Create active and inactive delegations
    // Execute: GET /api/delegations?is_active=true
    // Assert: Only active delegations returned
    // Execute: GET /api/delegations?is_active=false
    // Assert: Only inactive delegations returned
}

/// Test: POST /api/delegations creates delegation with valid input
#[tokio::test]
async fn create_delegation_with_valid_input() {
    // Setup: Authenticated user, valid target user
    // Execute: POST /api/delegations
    // Body: { "to_user_id": "...", "delegation_type": "temporary", "starts_at": now, "ends_at": now+7days }
    // Assert: Status 201
    // Assert: Response includes created delegation
    // Assert: Delegation in database
}

/// Test: POST /api/delegations requires authentication
#[tokio::test]
async fn create_delegation_requires_auth() {
    // Setup: No auth context
    // Execute: POST /api/delegations
    // Assert: Status 401
}

/// Test: POST /api/delegations validates to_user exists
#[tokio::test]
async fn create_delegation_validates_target_user() {
    // Setup: Non-existent target user ID
    // Execute: POST /api/delegations
    // Assert: Status 404
    // Assert: Error indicates user not found
}

/// Test: POST /api/delegations returns 400 for invalid date range
#[tokio::test]
async fn create_delegation_validates_date_range() {
    // Setup: ends_at before starts_at
    // Execute: POST /api/delegations
    // Body: { "ends_at": now, "starts_at": now+1day }
    // Assert: Status 400
    // Assert: Error indicates invalid date range
}

/// Test: DELETE /api/delegations/:id revokes delegation
#[tokio::test]
async fn revoke_delegation_revokes() {
    // Setup: Active delegation created by user
    // Execute: DELETE /api/delegations/{id}
    // Assert: Status 204
    // Assert: Delegation is_active = false in database
}

/// Test: DELETE /api/delegations/:id returns 403 for non-creator
#[tokio::test]
async fn revoke_delegation_returns_403_for_non_creator() {
    // Setup: Delegation created by different user
    // Execute: DELETE /api/delegations/{id}
    // Assert: Status 403
}

/// Test: GET /api/users/:id/delegations returns user's delegations (admin only)
#[tokio::test]
async fn user_delegations_returns_for_admin() {
    // Setup: User with admin role
    // Execute: GET /api/users/{user_id}/delegations
    // Assert: Status 200
    // Assert: Returns target user's delegations
}

/// Test: GET /api/users/:id/delegations returns 403 for non-admin
#[tokio::test]
async fn user_delegations_returns_403_for_non_admin() {
    // Setup: Regular user (not admin)
    // Execute: GET /api/users/{user_id}/delegations
    // Assert: Status 403
}

/// Test: Cannot create self-delegation
#[tokio::test]
async fn cannot_create_self_delegation() {
    // Setup: to_user_id == authenticated user
    // Execute: POST /api/delegations
    // Assert: Status 400
    // Assert: Error indicates cannot delegate to self
}

/// Test: Cannot create circular delegation
#[tokio::test]
async fn cannot_create_circular_delegation() {
    // Setup: A->B delegation exists, try B->A
    // Execute: POST /api/delegations (B delegates to A)
    // Assert: Status 400
    // Assert: Error indicates circular delegation
}

/// Test: Reactivate expired delegation
#[tokio::test]
async fn reactivate_delegation_works() {
    // Setup: Expired delegation
    // Execute: POST /api/delegations/{id}/reactivate
    // Body: { "ends_at": now+7days }
    // Assert: Status 200
    // Assert: Delegation is_active = true
}

/// Test: Bulk delegation applies to specified document types
#[tokio::test]
async fn bulk_delegation_applies_to_document_types() {
    // Setup: Create bulk delegation for ["woo_besluit", "woo_informatie"]
    // Execute: POST /api/delegations
    // Assert: Status 201
    // Assert: Delegation document_types matches request
}
```

### File: `crates/iou-api/tests/routes/versions.rs`

```rust
//! Tests for version API endpoints

use std::sync::Arc;
use uuid::Uuid;
use axum::http::StatusCode;

use iou_api::tests::common::{random_document_id};

/// Test: GET /api/documents/:id/versions returns all versions for document
#[tokio::test]
async fn list_versions_returns_all() {
    // Setup: Create document with 3 versions
    // Execute: GET /api/documents/{id}/versions
    // Assert: Status 200
    // Assert: Response has 3 versions
    // Assert: Ordered by created_at DESC (newest first)
}

/// Test: GET /api/documents/:id/versions includes version metadata
#[tokio::test]
async fn list_versions_includes_metadata() {
    // Setup: Document with versions
    // Execute: GET /api/documents/{id}/versions
    // Assert: Each version has: id, version_number, created_at, created_by, change_summary
    // Assert: One version marked as current=true
}

/// Test: GET /api/documents/:id/versions/diff with from and to params returns diff
#[tokio::test]
async fn get_diff_with_params_returns_diff() {
    // Setup: Document with 2 versions with different content
    // Execute: GET /api/documents/{id}/versions/diff?from=v1&to=v2
    // Assert: Status 200
    // Assert: Response contains changes array
    // Assert: changes has additions/deletions
}

/// Test: GET /api/documents/:id/versions/diff defaults to comparing last two versions
#[tokio::test]
async fn get_diff_defaults_to_last_two() {
    // Setup: Document with 3 versions
    // Execute: GET /api/documents/{id}/versions/diff (no params)
    // Assert: Status 200
    // Assert: Diff compares v2 and v3 (last two)
}

/// Test: GET /api/documents/:id/versions/diff supports format parameter
#[tokio::test]
async fn get_diff_supports_format() {
    // Setup: Document with versions
    // Execute: GET /api/documents/{id}/versions/diff?format=side_by_side
    // Assert: Status 200
    // Assert: Response format = "side_by_side"
    // Execute: GET ...?format=inline
    // Assert: Response format = "inline"
}

/// Test: GET /api/documents/:id/versions/:version_id returns single version
#[tokio::test]
async fn get_version_returns_single() {
    // Setup: Document with versions
    // Execute: GET /api/documents/{id}/versions/{version_id}
    // Assert: Status 200
    // Assert: Response matches requested version
}

/// Test: GET /api/documents/:id/versions/:version_id includes change_summary
#[tokio::test]
async fn get_version_includes_change_summary() {
    // Setup: Version with change_summary = "Fixed typos"
    // Execute: GET /api/documents/{id}/versions/{version_id}
    // Assert: Response change_summary = "Fixed typos"
}

/// Test: POST /api/documents/:id/versions/:version_id/restore restores version
#[tokio::test]
async fn restore_version_restores() {
    // Setup: Document with v1, v2, v3. Restore v2.
    // Execute: POST /api/documents/{id}/versions/{v2_id}/restore
    // Assert: Status 200
    // Assert: Document content matches v2
    // Assert: New version v4 created recording the restore
}

/// Test: POST /api/documents/:id/versions/:version_id/restore requires authentication
#[tokio::test]
async fn restore_version_requires_auth() {
    // Setup: No auth context
    // Execute: POST /api/documents/{id}/versions/{version_id}/restore
    // Assert: Status 401
}

/// Test: POST /api/documents/:id/versions/:version_id/restore creates new version
#[tokio::test]
async fn restore_version_creates_new_version() {
    // Setup: Document has v1, v2, v3
    // Execute: POST restore v1
    // Assert: Document now has v1, v2, v3, v4
    // Assert: v4's parent_version_id points to v1
    // Assert: v4 change_summary indicates restore
}

/// Test: POST restore returns 403 for non-authorized users
#[tokio::test]
async fn restore_version_returns_403_for_unauthorized() {
    // Setup: User without edit permission
    // Execute: POST /api/documents/{id}/versions/{version_id}/restore
    // Assert: Status 403
}

/// Test: Cannot restore current version
#[tokio::test]
async fn cannot_restore_current_version() {
    // Setup: Document with v3 as current
    // Execute: POST /api/documents/{id}/versions/{v3_id}/restore
    // Assert: Status 400
    // Assert: Error indicates cannot restore current version
}

/// Test: Diff summary includes counts
#[tokio::test]
async fn diff_summary_includes_counts() {
    // Setup: Versions with known changes (5 additions, 3 deletions)
    // Execute: GET /api/documents/{id}/versions/diff
    // Assert: Response summary.additions = 5
    // Assert: Response summary.deletions = 3
}

/// Test: Get versions for non-existent document returns 404
#[tokio::test]
async fn list_versions_nonexistent_document_returns_404() {
    // Setup: Random document ID
    // Execute: GET /api/documents/{id}/versions
    // Assert: Status 404
}

/// Test: Get non-existent version returns 404
#[tokio::test]
async fn get_nonexistent_version_returns_404() {
    // Setup: Valid document, invalid version_id
    // Execute: GET /api/documents/{id}/versions/{invalid_id}
    // Assert: Status 404
}

/// Test: Diff with invalid version returns 404
#[tokio::test]
async fn diff_with_invalid_version_returns_404() {
    // Setup: Valid document, invalid from version
    // Execute: GET /api/documents/{id}/versions/diff?from=invalid
    // Assert: Status 404
}
```

---

## Implementation Notes

1. **Authentication Context**: All endpoints should use the existing `AuthContext` from `crate::middleware::auth::AuthContext`. The auth middleware should extract user information from JWT tokens.

2. **WebSocket Notifications**: State changes (approval, rejection, delegation) should trigger WebSocket broadcasts via the existing `broadcast::Sender<StatusMessage>` pattern.

3. **Error Handling**: Use the existing `ApiError` enum for consistent error responses across all endpoints.

4. **Database Access**: The `Arc<Database>` extension provides access to DuckDB for embedded queries. For Supabase operations, use the existing Supabase client configuration.

5. **Service Extensions**: The new route files will need to extend their respective services (StageExecutor, DelegationService, VersionService) with methods for querying by document ID and user ID.

6. **CORS Configuration**: Ensure CORS headers allow frontend origins for all new endpoints.

7. **Rate Limiting**: Consider applying rate limiting to approval/rejection endpoints to prevent abuse.

8. **Audit Logging**: All state-changing operations (approve, reject, delegate, restore) must create audit trail entries via the existing audit system.

9. **Transaction Safety**: When operations affect multiple tables (e.g., approval + stage transition + document state), ensure database transactions are used for consistency.

10. **Null Safety**: Handle optional fields appropriately - use `Option<T>` and return `null` in JSON where appropriate, not empty strings or default values.
---

## Implementation Notes

**Status:** Placeholder implementation - API structure defined, business logic pending service layer integration

**Files Created:**
- ✅ `crates/iou-api/src/routes/workflow_stages.rs` - Route definitions with placeholder handlers
- ✅ `crates/iou-api/src/routes/delegations.rs` - Route definitions with validation
- ✅ `crates/iou-api/src/routes/versions.rs` - Route definitions with placeholder handlers
- ✅ `crates/iou-api/tests/routes/workflow_stages.rs` - Test documentation (placeholder tests)
- ✅ `crates/iou-api/tests/routes/delegations.rs` - Test documentation (placeholder tests)
- ✅ `crates/iou-api/tests/routes/versions.rs` - Test documentation (placeholder tests)

**Files Modified:**
- ✅ `crates/iou-api/src/routes/mod.rs` - Added module declarations
- ✅ `crates/iou-api/src/main.rs` - Routes are defined but not yet mounted (router nesting removed to fix path conflicts)

**Deviations from Original Plan:**

1. **Router mounting approach**: Originally planned to nest routers in main.rs, but this created path conflicts with route definitions. Removed nesting - routes will be mounted directly when ready.

2. **Type representations**: Used String for enum types (status, approval_type, delegation_type) in API responses. This maintains compatibility but loses type safety. Should use proper enums from `iou_core` when service layer is integrated.

3. **Placeholder implementations**: All route handlers return placeholder responses with TODO comments. Basic validation added to `create_delegation` for security (self-delegation check, date range validation).

4. **Service layer integration**: Not integrated yet. Routes have TODO comments indicating where `StageExecutor`, `DelegationService`, and `VersionService` should be used.

**Code Review Fixes Applied:**
- Fixed router path conflicts by removing nesting in main.rs
- Added `chrono::{DateTime, Utc}` import for consistency
- Added basic validation to `create_delegation` (no self-delegation, valid date ranges)
- Cleaned up unused re-exports from routes/mod.rs

**Next Steps for Full Implementation:**
1. Wire up routers to main API router
2. Integrate with service layer (StageExecutor, DelegationService, VersionService)
3. Replace String types with proper enums from iou_core
4. Implement authorization/permission checks
5. Add organization scoping to queries
6. Add audit logging for mutations
7. Implement actual test infrastructure
