//! Template management API routes
//!
//! Provides CRUD operations for document templates.

use axum::{
    extract::{Extension, Path, Query},
    response::{IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::db::Database;
use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use iou_core::document::Template;

// ============================================
// Request/Response Types
// ============================================

/// List templates query parameters
#[derive(Debug, Deserialize)]
pub struct ListTemplatesParams {
    pub domain_id: Option<String>,
}

/// Template list response
#[derive(Debug, Serialize)]
pub struct TemplateListResponse {
    pub templates: Vec<TemplateDto>,
}

/// Template DTO for API responses
#[derive(Debug, Serialize)]
pub struct TemplateDto {
    pub id: String,
    pub name: String,
    pub domain_id: String,
    pub document_type: String,
    pub version: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create template request
#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub domain_id: String,
    pub document_type: String,
    pub content: String,
    pub required_variables: Vec<String>,
    pub optional_sections: Vec<String>,
}

/// Create template response
#[derive(Debug, Serialize)]
pub struct CreateTemplateResponse {
    pub template_id: String,
    pub version: i32,
}

/// Update template request
#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub content: Option<String>,
    pub required_variables: Option<Vec<String>>,
    pub optional_sections: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

// ============================================
// Route Handlers
// ============================================

/// GET /api/templates?domain_id={id}
pub async fn list_templates(
    Extension(_db): Extension<Arc<Database>>,
    Query(_params): Query<ListTemplatesParams>,
) -> Result<Json<TemplateListResponse>, ApiError> {
    // TODO: Query templates from database
    // For now, return empty list
    let templates: Vec<TemplateDto> = vec![];

    Ok(Json(TemplateListResponse { templates }))
}

/// GET /api/templates/{id}
pub async fn get_template(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<String>,
) -> Result<Json<Template>, ApiError> {
    let template = db
        .get_template_async(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Template not found".to_string()))?;

    Ok(Json(template))
}

/// POST /api/templates
pub async fn create_template(
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<CreateTemplateRequest>,
) -> Result<Json<CreateTemplateResponse>, ApiError> {
    // Validate template content
    if req.content.is_empty() {
        return Err(ApiError::Validation(
            "Template content cannot be empty".to_string(),
        ));
    }

    // Check for duplicate domain_id + document_type
    let template_exists = db
        .template_exists_async(req.domain_id.clone(), req.document_type.clone())
        .await?;

    if template_exists {
        return Err(ApiError::Validation(
            "Template for this domain and type already exists".to_string(),
        ));
    }

    let template_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let template = Template {
        id: template_id.clone(),
        name: req.name,
        domain_id: req.domain_id,
        document_type: req.document_type,
        content: req.content,
        required_variables: req.required_variables,
        optional_sections: req.optional_sections,
        version: 1,
        created_at: now,
        updated_at: now,
        is_active: true,
    };

    db.create_template(&template)?;

    tracing::info!(
        template_id = %template_id,
        "Template created"
    );

    Ok(Json(CreateTemplateResponse {
        template_id,
        version: 1,
    }))
}

/// PUT /api/templates/{id}
pub async fn update_template(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<String>,
    Json(_req): Json<UpdateTemplateRequest>,
) -> Result<Json<TemplateDto>, ApiError> {
    // Verify template exists
    let mut template = db
        .get_template_async(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Template not found".to_string()))?;

    // Update template fields
    // TODO: Apply updates from _req
    template.updated_at = Utc::now();
    template.version += 1;

    db.update_template(&template)?;

    tracing::info!(
        template_id = %template.id,
        "Template updated"
    );

    Ok(Json(TemplateDto {
        id: template.id,
        name: template.name,
        domain_id: template.domain_id,
        document_type: template.document_type,
        version: template.version,
        is_active: template.is_active,
        created_at: template.created_at,
        updated_at: template.updated_at,
    }))
}

/// DELETE /api/templates/{id}
pub async fn delete_template(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify template exists
    let mut template = db
        .get_template_async(id.clone())
        .await?
        .ok_or_else(|| ApiError::NotFound("Template not found".to_string()))?;

    // Soft delete (set is_active = false)
    template.is_active = false;
    template.updated_at = Utc::now();

    db.update_template(&template)?;

    tracing::info!(
        template_id = %id,
        "Template deleted"
    );

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_template_request_serialization() {
        let req = CreateTemplateRequest {
            name: "Test Template".to_string(),
            domain_id: "test_domain".to_string(),
            document_type: "woo_besluit".to_string(),
            content: "# {{ reference }}\n\nTest content.".to_string(),
            required_variables: vec!["reference".to_string()],
            optional_sections: vec![],
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Test Template"));
        assert!(json.contains("reference"));
    }

    #[test]
    fn test_update_template_request_partial() {
        let req = UpdateTemplateRequest {
            name: Some("Updated Name".to_string()),
            content: None,
            required_variables: None,
            optional_sections: None,
            is_active: Some(false),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Updated Name"));
    }
}
