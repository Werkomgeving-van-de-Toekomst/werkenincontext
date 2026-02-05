//! Information object endpoints

use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    Json,
};
use uuid::Uuid;

use crate::db::Database;
use crate::error::ApiError;
use iou_core::api_types::{CreateObjectRequest, CreateObjectResponse};
use iou_core::objects::InformationObject;

/// GET /objects/:id - Get an information object
pub async fn get_object(
    Path(object_id): Path<Uuid>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Json<InformationObject>, ApiError> {
    let object = db
        .get_object(object_id)?
        .ok_or_else(|| ApiError::NotFound(format!("Object {} not found", object_id)))?;

    Ok(Json(object))
}

/// POST /objects - Create a new information object
pub async fn create_object(
    Extension(db): Extension<Arc<Database>>,
    Json(request): Json<CreateObjectRequest>,
) -> Result<Json<CreateObjectResponse>, ApiError> {
    // TODO: Get current user from JWT
    let created_by = Uuid::new_v4(); // Placeholder

    // Create the object with compliance by design
    let mut object = InformationObject::new(
        request.domain_id,
        request.object_type,
        request.title,
        request.content_location,
        created_by,
    );

    // Apply optional overrides
    if let Some(classification) = request.classification {
        object.classification = classification;
    }
    if let Some(is_woo_relevant) = request.is_woo_relevant {
        object.is_woo_relevant = is_woo_relevant;
    }
    object.description = request.description;
    object.content_text = request.content_text;
    object.mime_type = request.mime_type;
    object.file_size = request.file_size;
    object.tags = request.tags;

    // Apply automatic compliance rules
    if object.retention_period.is_none() {
        object.retention_period = Some(object.default_retention_period());
    }
    if object.should_be_woo_relevant() {
        object.is_woo_relevant = true;
    }

    // TODO: Run business rules engine
    let applied_rules = vec![];

    // TODO: Get AI suggestions for metadata
    let ai_suggestions = vec![];

    // Save to database
    db.create_object(&object)?;

    Ok(Json(CreateObjectResponse {
        object,
        applied_rules,
        ai_suggestions,
    }))
}
