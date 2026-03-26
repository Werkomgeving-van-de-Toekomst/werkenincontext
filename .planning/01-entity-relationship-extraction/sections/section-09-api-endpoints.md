Now I have all the context needed. Let me generate the section content for section-09-api-endpoints:

# Section 09: API Endpoints

## Overview

This section implements REST API endpoints to expose stakeholder data extracted from documents. The endpoints allow clients to query stakeholders, their relationships to documents, and perform fuzzy searches. Authentication and authorization are enforced, with special handling for PII redaction based on entity classification.

## Dependencies

- **Section 01 (Foundation & Types)**: Core stakeholder data structures (`PersonStakeholder`, `OrganizationStakeholder`, `Entity`, `MentionRelationship`)
- **Section 08 (KnowledgeGraph Extensions)**: Query methods (`get_document_stakeholders`, `get_stakeholder_documents`, `get_stakeholder_influence`, `find_stakeholders_by_name`)
- Existing API infrastructure in `crates/iou-api/`

## Tests

### Unit Tests

#### Route Registration
- Test: GET /stakeholders/:id returns stakeholder for valid ID
- Test: GET /stakeholders/:id returns 404 for invalid ID
- Test: GET /stakeholders/:id/documents returns all documents mentioning stakeholder
- Test: GET /documents/:id/stakeholders returns all stakeholders in document
- Test: GET /stakeholders/search?q= returns results for partial name match
- Test: GET /stakeholders/search?q= returns empty results for no match

#### Pagination
- Test: Pagination returns correct page size
- Test: Pagination returns correct next/prev page links
- Test: Pagination handles out-of-range page numbers gracefully

#### Authentication & Authorization
- Test: Unauthorized requests return 401
- Test: Requests with valid token are processed
- Test: Role-based access control for restricted endpoints

#### Privacy & PII Handling
- Test: Citizen PII redacted in API responses
- Test: Official PII not redacted in API responses
- Test: PII access logged for audit trail

## Implementation

### File Structure

```
crates/iou-api/src/
├── routes/
│   ├── mod.rs                    # Add stakeholder module export
│   └── stakeholder.rs            # NEW: Stakeholder endpoint handlers
└── main.rs                       # Register new routes
```

### Response Types

**File**: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/stakeholder.rs`

```rust
//! Stakeholder API endpoints
//!
//! Provides REST API access to extracted stakeholder entities
//! and their relationships to documents.

use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiError;

/// Stakeholder detail response
#[derive(Debug, Serialize)]
pub struct StakeholderResponse {
    pub id: Uuid,
    pub name: String,
    pub entity_type: String,  // "PERSON" or "ORGANIZATION"
    pub canonical_name: Option<String>,
    pub description: Option<String>,
    pub confidence: f32,
    pub verification_status: String,  // "AUTO_ACCEPTED", "ACCEPTED_FLAGGED", etc.
    pub pii_classification: String,   // "OFFICIAL", "CITIZEN", "NONE"
    pub metadata: serde_json::Value,
    pub influence: InfluenceMetricsResponse,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Influence metrics for API response
#[derive(Debug, Serialize)]
pub struct InfluenceMetricsResponse {
    pub mention_count: usize,
    pub document_count: usize,
    pub pagerank_score: f32,
}

/// Document mention info
#[derive(Debug, Serialize)]
pub struct DocumentMentionResponse {
    pub document_id: Uuid,
    pub document_title: Option<String>,
    pub mention_type: String,  // "AUTHOR", "RECIPIENT", "REFERENCED", "SUBJECT"
    pub context: Option<String>,
    pub confidence: f32,
    pub mentioned_at: chrono::DateTime<chrono::Utc>,
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: usize,
    pub page_size: usize,
    pub total_count: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Search query parameters
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
    #[serde(default = "default_page")]
    page: usize,
    #[serde(default = "default_page_size")]
    page_size: usize,
    #[serde(default = "default_threshold")]
    threshold: Option<f32>,
}

fn default_page() -> usize { 1 }
fn default_page_size() -> usize { 20 }
fn default_threshold() -> Option<f32> { Some(0.7) }

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    page: usize,
    #[serde(default = "default_page_size")]
    page_size: usize,
}
```

### Endpoint Handlers

**File**: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/stakeholder.rs`

```rust
/// GET /stakeholders/:id
///
/// Get detailed information about a specific stakeholder.
///
/// # PII Handling
/// - Citizen PII (email, phone) is redacted unless user has elevated permissions
/// - Official PII is always returned
///
/// # Response
/// Returns 404 if stakeholder not found.
pub async fn get_stakeholder(
    Path(id): Path<Uuid>,
    Extension(knowledge_graph): Extension<Arc<KnowledgeGraph>>,
) -> Result<Json<StakeholderResponse>, ApiError> {
    // Implementation calls KnowledgeGraph methods
    // Applies PII redaction based on pii_classification
    todo!("Implement get_stakeholder")
}

/// GET /stakeholders/:id/documents
///
/// Get all documents that mention this stakeholder.
///
/// # Response
/// Paginated list of documents with mention context.
pub async fn get_stakeholder_documents(
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
    Extension(knowledge_graph): Extension<Arc<KnowledgeGraph>>,
) -> Result<Json<PaginatedResponse<DocumentMentionResponse>>, ApiError> {
    // Implementation calls get_stakeholder_documents
    // Applies pagination logic
    todo!("Implement get_stakeholder_documents")
}

/// GET /documents/:id/stakeholders
///
/// Get all stakeholders mentioned in a specific document.
///
/// # Response
/// List of stakeholders with mention details.
pub async fn get_document_stakeholders(
    Path(id): Path<Uuid>,
    Extension(knowledge_graph): Extension<Arc<KnowledgeGraph>>,
) -> Result<Json<Vec<StakeholderResponse>>, ApiError> {
    // Implementation calls get_document_stakeholders
    // Includes mention_type for each stakeholder
    todo!("Implement get_document_stakeholders")
}

/// GET /stakeholders/search?q=
///
/// Search for stakeholders by name with fuzzy matching.
///
/// # Query Parameters
/// - `q`: Search query (partial name match)
/// - `threshold`: Minimum similarity score (default: 0.7)
/// - `page`: Page number (default: 1)
/// - `page_size`: Results per page (default: 20)
///
/// # Response
/// Paginated list of matching stakeholders.
pub async fn search_stakeholders(
    Query(query): Query<SearchQuery>,
    Extension(knowledge_graph): Extension<Arc<KnowledgeGraph>>,
) -> Result<Json<PaginatedResponse<StakeholderResponse>>, ApiError> {
    // Implementation calls find_stakeholders_by_name with threshold
    // Applies pagination
    todo!("Implement search_stakeholders")
}
```

### PII Redaction Logic

```rust
/// Redact citizen PII from response metadata
fn apply_pii_redaction(
    metadata: &serde_json::Value,
    pii_classification: &str,
    user_has_elevated_access: bool,
) -> serde_json::Value {
    match pii_classification {
        "CITIZEN" if !user_has_elevated_access => {
            // Redact email, phone, address
            redact_pii_fields(metadata)
        }
        _ => metadata.clone(),
    }
}

fn redact_pii_fields(metadata: &serde_json::Value) -> serde_json::Value {
    // Recursively redact known PII fields
    // email -> "***@***.***"
    // phone -> "***-*******"
    // address -> "[REDACTED]"
    todo!("Implement redact_pii_fields")
}
```

### Route Registration

**File**: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/mod.rs`

```rust
// Add to existing module exports
pub mod stakeholder;

// Re-export handlers
pub use stakeholder::{
    get_stakeholder,
    get_stakeholder_documents,
    get_document_stakeholders,
    search_stakeholders,
};
```

**File**: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/main.rs`

```rust
// In the api Router construction (around line 135):
.route("/stakeholders/:id", get(routes::get_stakeholder))
.route("/stakeholders/:id/documents", get(routes::get_stakeholder_documents))
.route("/documents/:id/stakeholders", get(routes::get_document_stakeholders))
.route("/stakeholders/search", get(routes::search_stakeholders))
```

### KnowledgeGraph Access Pattern

The API handlers will need access to the `KnowledgeGraph`. Based on existing patterns in the codebase:

```rust
// In main.rs, add KnowledgeGraph as an extension:
// .layer(Extension(Arc::new(knowledge_graph)))

// The KnowledgeGraph will need to be thread-safe (Arc + RwLock or Arc + async mutex)
// for concurrent access from API handlers.
```

### Authentication Integration

Existing authentication middleware (`optional_auth_middleware`) provides auth context. For stakeholder endpoints:

1. Public endpoints: Basic stakeholder info with citizen PII redacted
2. Authenticated endpoints: Full access based on user role
3. Role-based access: Use existing `Role` enum from middleware

## Success Criteria

- All endpoints return appropriate HTTP status codes
- Fuzzy search works with configurable threshold
- Pagination handles edge cases (empty results, out of range)
- Citizen PII is redacted for non-elevated users
- Official PII is never redacted
- PII access is logged for audit trail
- Integration tests pass with authenticated client