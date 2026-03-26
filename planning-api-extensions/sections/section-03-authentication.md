Now I have a complete picture of the existing authentication infrastructure and the documents route. Let me generate the section content.

# Section 3: Authentication Integration

## Overview

This section implements Role-Based Access Control (RBAC) for document API endpoints. The authentication infrastructure already exists in `/Users/marc/Projecten/iou-modern/crates/iou-api/src/middleware/auth.rs`. This section focuses on integrating authentication checks into document operations.

**Dependencies:** None (can be implemented in parallel with Section 1)

**Files Modified:**
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/documents.rs` - Add role checks, organization scoping, audit logging
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/middleware/auth.rs` - Load JWT_SECRET from environment

## Existing Authentication Infrastructure

The codebase already has a comprehensive authentication system:

### AuthContext Structure
```rust
pub struct AuthContext {
    pub user_id: Uuid,
    pub email: String,
    pub organization_id: Uuid,
    pub roles: Vec<Role>,
}
```

### Available Roles
- `Admin` - Full system access
- `Auditor` - Read-only audit access
- `DomainManager` - Domain management
- `DomainEditor` - Domain editing
- `DomainViewer` - Domain read-only
- `ObjectCreator` - Can create documents
- `ObjectEditor` - Can edit documents
- `ObjectApprover` - Can approve documents
- `ComplianceOfficer` - Compliance operations
- `ComplianceReviewer` - Compliance review
- `WooOfficer` - WOO operations
- `WooPublisher` - WOO publishing

### Helper Functions Already Available
- `require_permission(auth, permission)` - Check for specific permission
- `has_any_role(auth, roles)` - Check if user has any of the specified roles

## Tests

Write these tests BEFORE implementing the changes.

### Test: create_document with ObjectCreator role succeeds
**File:** `crates/iou-api/tests/routes/documents_auth_tests.rs`

```rust
#[tokio::test]
async fn create_document_with_object_creator_role_succeeds() {
    // Given: AuthContext with ObjectCreator role
    // When: create_document() is called
    // Then: Request proceeds to handler
    // And: Returns 201 Created
}
```

### Test: create_document with DomainEditor role succeeds
```rust
#[tokio::test]
async fn create_document_with_domain_editor_role_succeeds() {
    // Given: AuthContext with DomainEditor role
    // When: create_document() is called
    // Then: Request proceeds to handler
    // And: Returns 201 Created
}
```

### Test: create_document with DomainViewer role fails
```rust
#[tokio::test]
async fn create_document_with_domain_viewer_role_fails() {
    // Given: AuthContext with DomainViewer role only
    // When: create_document() is called
    // Then: Returns 403 Forbidden
    // And: Error message indicates insufficient permissions
}
```

### Test: approve_document with ObjectApprover role succeeds
```rust
#[tokio::test]
async fn approve_document_with_object_approver_role_succeeds() {
    // Given: AuthContext with ObjectApprover role
    // When: approve_document() is called
    // Then: Request proceeds to handler
}
```

### Test: approve_document without ObjectApprover role fails
```rust
#[tokio::test]
async fn approve_document_without_object_approver_role_fails() {
    // Given: AuthContext without ObjectApprover role
    // When: approve_document() is called
    // Then: Returns 403 Forbidden
}
```

### Test: create_document validates organization match
```rust
#[tokio::test]
async fn create_document_validates_organization_match() {
    // Given: AuthContext with org_id = A
    // And: Request with org_id = B
    // When: create_document() is called
    // Then: Returns 403 Forbidden
    // And: Error message indicates organization mismatch
}
```

### Test: document_query filters by organization
```rust
#[tokio::test]
async fn document_query_filters_by_organization() {
    // Given: AuthContext with org_id = A
    // And: Documents exist for org A and org B
    // When: list_documents() is called
    // Then: Only documents from org A are returned
}
```

### Test: audit_log includes user_id
```rust
#[tokio::test]
async fn audit_log_includes_user_id() {
    // Given: AuthContext with user_id
    // When: Document action is performed
    // Then: Audit entry contains user_id
    // And: Audit entry contains timestamp
}
```

### Test: JWT secret from environment is used
```rust
#[tokio::test]
async fn jwt_secret_from_environment_is_used() {
    // Given: JWT_SECRET environment variable set
    // When: JWT token is validated
    // Then: Uses environment secret, not hardcoded value
}
```

### Test: Invalid JWT token is rejected
```rust
#[tokio::test]
async fn invalid_jwt_token_is_rejected() {
    // Given: Malformed JWT token
    // When: Request is made with invalid token
    // Then: Returns 401 Unauthorized
}
```

## Implementation Tasks

### Task 1: Update JWT Service to Use Environment Secret

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/middleware/auth.rs`

The current implementation uses a hardcoded JWT_SECRET constant. Update the `JwtService` to load the secret from the environment:

```rust
impl JwtService {
    /// Create a new JWT service with secret from environment
    pub fn new_from_env() -> Result<Self, AuthError> {
        let secret = std::env::var("JWT_SECRET")
            .map_err(|_| AuthError::InvalidToken)?;
        
        if secret.len() < 32 {
            return Err(AuthError::InvalidToken);
        }
        
        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        })
    }
    
    // Keep new() for tests, but mark it as test-only
    #[cfg(test)]
    pub fn new() -> Self {
        // ... existing implementation
    }
}
```

Update the middleware to use `new_from_env()` instead of `new()`.

### Task 2: Add Organization ID to CreateDocumentRequest

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/documents.rs`

The `CreateDocumentRequest` needs an `organization_id` field to enable organization-based access control:

```rust
#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub organization_id: Uuid,  // NEW
    pub domain_id: String,
    pub document_type: String,
    pub context: HashMap<String, String>,
}
```

### Task 3: Add Role Check to create_document

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/documents.rs`

Update the `create_document` function signature to extract `AuthContext` and add permission checks:

```rust
pub async fn create_document(
    Extension(db): Extension<Arc<Database>>,
    Extension(auth): Extension<AuthContext>,  // NEW
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<CreateDocumentResponse>, ApiError> {
    // Check roles: require ObjectCreator or DomainEditor
    let has_permission = auth.roles.iter()
        .any(|r| *r == Role::ObjectCreator || *r == Role::DomainEditor);
    if !has_permission {
        return Err(ApiError::Forbidden(
            "Requires ObjectCreator or DomainEditor role".to_string()
        ));
    }

    // Check organization access
    if auth.organization_id != req.organization_id {
        return Err(ApiError::Forbidden(
            "Cannot access documents outside your organization".to_string()
        ));
    }
    
    // ... rest of existing implementation
}
```

### Task 4: Add Role Check to approve_document

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/documents.rs`

Update the `approve_document` function to check for `ObjectApprover` role:

```rust
pub async fn approve_document(
    Extension(db): Extension<Arc<Database>>,
    Extension(auth): Extension<AuthContext>,  // NEW
    Path(id): Path<Uuid>,
    Json(req): Json<ApprovalRequest>,
) -> Result<Json<ApprovalResponse>, ApiError> {
    // Check for ObjectApprover role
    if !auth.roles.iter().any(|r| *r == Role::ObjectApprover) {
        return Err(ApiError::Forbidden(
            "Requires ObjectApprover role".to_string()
        ));
    }
    
    // ... rest of existing implementation
    
    // Update audit trail with user_id
    Ok(Json(ApprovalResponse {
        document_id: id,
        state: format!("{:?}", new_state).to_lowercase(),
        approved_at: if req.approved { Some(Utc::now()) } else { None },
        approved_by: Some(auth.user_id.to_string()),  // CHANGED: was None
    }))
}
```

### Task 5: Add Organization Scoping to Queries

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/documents.rs`

For list/query operations, ensure documents are filtered by organization. The `get_status` function should validate organization access:

```rust
pub async fn get_status(
    Extension(db): Extension<Arc<Database>>,
    Extension(auth): Extension<AuthContext>,  // NEW
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentStatusResponse>, ApiError> {
    let document = db
        .get_document_async(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("No document with ID {}", id)))?;
    
    // NEW: Check organization access
    // Note: This requires DocumentMetadata to include organization_id
    // If DocumentMetadata doesn't have it, we need to add it first
    // For now, we can check via domain_id (assuming domains are org-scoped)
    
    // ... rest of existing implementation
}
```

**Note:** If `DocumentMetadata` does not already include `organization_id`, you will need to add it to the struct and database schema first. Check the existing database schema.

### Task 6: Update Audit Trail with User Context

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/documents.rs`

Ensure audit trail entries include the `user_id` from the `AuthContext`. This may require updating the `AuditEntry` structure:

```rust
// When creating audit entries, include user_id:
let audit_entry = AuditEntry {
    user_id: Some(auth.user_id),  // NEW: Track who performed the action
    agent_name: "api".to_string(),
    action: "document_created".to_string(),
    timestamp: Utc::now(),
    details: serde_json::json!({
        "document_id": document_id,
        "document_type": req.document_type,
    }),
};
```

## Testing Checklist

After implementation, verify:

- [ ] JWT_SECRET environment variable is loaded correctly
- [ ] Users without ObjectCreator/DomainEditor role cannot create documents (403)
- [ ] Users with ObjectCreator/DomainEditor role can create documents (201)
- [ ] Users without ObjectApprover role cannot approve documents (403)
- [ ] Users with ObjectApprover role can approve documents (200)
- [ ] Users cannot access documents from other organizations (403/404)
- [ ] Audit trail includes user_id for all actions
- [ ] Invalid JWT tokens are rejected with 401
- [ ] Expired JWT tokens are rejected with 401

## Notes

1. The authentication middleware (`auth_middleware`) must be applied to the document routes in the router setup (typically in `main.rs`). This section assumes the middleware is already wired up.

2. The `AuthContext` is available via `Extension<AuthContext>` extractor because the middleware inserts it into the request extensions.

3. Organization-based access control requires that documents are associated with an organization. If `DocumentMetadata` does not already have an `organization_id` field, this needs to be added as a prerequisite (possibly in a database migration).

4. The JWT secret must be at least 32 characters for security. The implementation should validate this at startup.

5. For testing, you may need to create test helpers that generate valid JWT tokens with specific roles. Consider creating a `tests/helpers/auth.rs` module with utilities like:

```rust
pub fn create_test_token(user_id: Uuid, org_id: Uuid, roles: Vec<Role>) -> String {
    // Create and return a signed JWT token for testing
}
```