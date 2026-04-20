//! Tags API endpoints
//!
//! Provides CRUD operations for tag management including
//! tagging objects and domains with search and suggestions.

use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use iou_core::{
    Tag, TagType, ObjectTag, TagSuggestion, TagStats,
    TagRepository, TagError,
};

use crate::{
    error::ApiError,
    middleware::auth::{AuthContext, require_permission, Permission},
    supabase::SupabasePool,
};

// =============================================================================
// Request/Response Types
// =============================================================================

/// Create tag request
#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub tag_type: TagType,
    pub organization_id: Option<Uuid>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub parent_tag_id: Option<Uuid>,
    pub is_system_tag: Option<bool>,
}

/// Update tag request
#[derive(Debug, Deserialize)]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,  // None = no change, Some(None) = clear
    pub color: Option<Option<String>>,
}

/// Tag object request (tag an object)
#[derive(Debug, Deserialize)]
pub struct TagObjectRequest {
    pub object_id: Uuid,
    pub tag_id: Uuid,
    pub confidence: Option<f32>,
    pub is_auto_assigned: Option<bool>,
}

/// Untag object request
#[derive(Debug, Deserialize)]
pub struct UntagObjectRequest {
    pub object_id: Uuid,
    pub tag_id: Uuid,
}

/// Tag domain request
#[derive(Debug, Deserialize)]
pub struct TagDomainRequest {
    pub domain_id: Uuid,
    pub tag_id: Uuid,
}

/// List tags query parameters
#[derive(Debug, Deserialize)]
pub struct ListTagsQuery {
    pub tag_type: Option<TagType>,
    pub organization_id: Option<Uuid>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub search: Option<String>,
}

/// Tag response
#[derive(Debug, Serialize)]
pub struct TagResponse {
    pub id: Uuid,
    pub name: String,
    pub normalized_name: String,
    pub tag_type: TagType,
    pub organization_id: Option<Uuid>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub parent_tag_id: Option<Uuid>,
    pub usage_count: i32,
    pub is_system_tag: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Tag> for TagResponse {
    fn from(tag: Tag) -> Self {
        Self {
            id: tag.id,
            name: tag.name,
            normalized_name: tag.normalized_name,
            tag_type: tag.tag_type,
            organization_id: tag.organization_id,
            description: tag.description,
            color: tag.color,
            parent_tag_id: tag.parent_tag_id,
            usage_count: tag.usage_count,
            is_system_tag: tag.is_system_tag,
            created_at: tag.created_at,
            updated_at: tag.updated_at,
        }
    }
}

/// Tags list response
#[derive(Debug, Serialize)]
pub struct TagsListResponse {
    pub tags: Vec<TagResponse>,
    pub total_count: i32,
    pub limit: u32,
    pub offset: u32,
}

/// Tag statistics response
#[derive(Debug, Serialize)]
pub struct TagStatsResponse {
    pub tag_id: Uuid,
    pub tag_name: String,
    pub usage_count: i32,
    pub objects_count: i32,
    pub domains_count: i32,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub trending_score: f32,
}

impl From<TagStats> for TagStatsResponse {
    fn from(stats: TagStats) -> Self {
        Self {
            tag_id: stats.tag_id,
            tag_name: stats.tag_name,
            usage_count: stats.usage_count,
            objects_count: stats.objects_count,
            domains_count: stats.domains_count,
            last_used: stats.last_used,
            trending_score: stats.trending_score,
        }
    }
}

// =============================================================================
// Handlers
// =============================================================================

/// Create a new tag
///
/// Creates a new tag with the specified properties.
/// Requires `TagCreate` permission.
pub async fn create_tag(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<TagResponse>, ApiError> {
    require_permission(&auth, Permission::TagCreate)?;

    let repo = TagRepository::new(pool.into_inner());

    let tag = repo.create_tag(
        req.name,
        req.tag_type,
        req.organization_id,
        req.description,
        req.color,
        req.parent_tag_id,
        req.is_system_tag.unwrap_or(false),
    ).await?;

    Ok(Json(TagResponse::from(tag)))
}

/// Get tag by ID
///
/// Retrieves a single tag by its ID.
/// Requires `TagRead` permission.
pub async fn get_tag(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(tag_id): Path<Uuid>,
) -> Result<Json<TagResponse>, ApiError> {
    require_permission(&auth, Permission::TagRead)?;

    let repo = TagRepository::new(pool.into_inner());
    let tag = repo.get_tag(tag_id).await?;

    Ok(Json(TagResponse::from(tag)))
}

/// List tags
///
/// Lists tags with optional filtering by type, organization, and search.
/// Requires `TagRead` permission.
pub async fn list_tags(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<ListTagsQuery>,
) -> Result<Json<TagsListResponse>, ApiError> {
    require_permission(&auth, Permission::TagRead)?;

    let repo = TagRepository::new(pool.into_inner());

    let tags = if let Some(search_term) = query.search {
        repo.search_tags(&search_term, query.limit.map(|l| l as i32)).await?
    } else {
        repo.list_tags(
            query.tag_type,
            query.organization_id,
            query.limit.map(|l| l as i32),
            query.offset.map(|o| o as i32),
        ).await?
    };

    let total_count = tags.len() as i32;
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);

    Ok(Json(TagsListResponse {
        tags: tags.into_iter().map(TagResponse::from).collect(),
        total_count,
        limit,
        offset,
    }))
}

/// Update tag
///
/// Updates an existing tag's properties.
/// Requires `TagUpdate` permission.
pub async fn update_tag(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(tag_id): Path<Uuid>,
    Json(req): Json<UpdateTagRequest>,
) -> Result<Json<TagResponse>, ApiError> {
    require_permission(&auth, Permission::TagUpdate)?;

    let repo = TagRepository::new(pool.into_inner());
    let tag = repo.update_tag(
        tag_id,
        req.name,
        req.description,
        req.color,
    ).await?;

    Ok(Json(TagResponse::from(tag)))
}

/// Delete tag
///
/// Deletes a tag. This will cascade to object_tags and domain_tags.
/// Requires `TagDelete` permission.
pub async fn delete_tag(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(tag_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, ApiError> {
    require_permission(&auth, Permission::TagDelete)?;

    let repo = TagRepository::new(pool.into_inner());
    repo.delete_tag(tag_id).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// Tag an object
///
/// Associates a tag with an information object.
/// Requires `ObjectUpdate` permission.
pub async fn tag_object(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<TagObjectRequest>,
) -> Result<Json<TagResponse>, ApiError> {
    require_permission(&auth, Permission::ObjectUpdate)?;

    let repo = TagRepository::new(pool.into_inner());

    repo.tag_object(
        req.object_id,
        req.tag_id,
        auth.user_id,
        req.confidence,
        req.is_auto_assigned,
    ).await?;

    // Return the updated tag
    let tag = repo.get_tag(req.tag_id).await?;
    Ok(Json(TagResponse::from(tag)))
}

/// Untag an object
///
/// Removes a tag from an information object.
/// Requires `ObjectUpdate` permission.
pub async fn untag_object(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<UntagObjectRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    require_permission(&auth, Permission::ObjectUpdate)?;

    let repo = TagRepository::new(pool.into_inner());
    repo.untag_object(req.object_id, req.tag_id).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// Get tags for an object
///
/// Lists all tags associated with a specific object.
/// Requires `ObjectRead` permission.
pub async fn get_object_tags(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(object_id): Path<Uuid>,
) -> Result<Json<Vec<TagResponse>>, ApiError> {
    require_permission(&auth, Permission::ObjectRead)?;

    let repo = TagRepository::new(pool.into_inner());
    let tags = repo.get_object_tags(object_id).await?;

    Ok(Json(tags.into_iter().map(TagResponse::from).collect()))
}

/// Get tag suggestions for an object
///
/// Returns suggested tags based on domain and usage patterns.
/// Requires `ObjectRead` permission.
pub async fn get_tag_suggestions(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(object_id): Path<Uuid>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<TagSuggestion>>, ApiError> {
    require_permission(&auth, Permission::ObjectRead)?;

    let repo = TagRepository::new(pool.into_inner());
    let limit = query.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(10);

    let suggestions = repo.get_tag_suggestions(object_id, Some(limit)).await?;

    Ok(Json(suggestions))
}

/// Get popular tags
///
/// Returns frequently used tags across all objects and domains.
/// Requires `TagRead` permission.
pub async fn get_popular_tags(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<TagStatsResponse>>, ApiError> {
    require_permission(&auth, Permission::TagRead)?;

    let repo = TagRepository::new(pool.into_inner());
    let limit = query.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50);

    let stats = repo.get_popular_tags(Some(limit)).await?;

    Ok(Json(stats.into_iter().map(TagStatsResponse::from).collect()))
}

/// Get tag statistics
///
/// Returns usage statistics for a specific tag.
/// Requires `TagRead` permission.
pub async fn get_tag_stats(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(tag_id): Path<Uuid>,
) -> Result<Json<TagStatsResponse>, ApiError> {
    require_permission(&auth, Permission::TagRead)?;

    let repo = TagRepository::new(pool.into_inner());
    let stats = repo.get_tag_stats(tag_id).await?;

    Ok(Json(TagStatsResponse::from(stats)))
}

impl From<TagError> for ApiError {
    fn from(err: TagError) -> Self {
        match err {
            TagError::TagNotFound(id) => ApiError::NotFound(format!("Tag not found: {}", id)),
            TagError::ObjectNotFound(id) => ApiError::NotFound(format!("Object not found: {}", id)),
            TagError::DatabaseError(e) => ApiError::Internal(format!("Database error: {}", e)),
            _ => ApiError::Internal(format!("Tag error: {}", err)),
        }
    }
}
