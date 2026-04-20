//! Categories API endpoints
//!
//! Provides CRUD operations for category management including
//! hierarchical operations and tree traversal.

use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use iou_core::{
    Category, CategoryType, ObjectCategory, CategoryNode, CategoryStats,
    CategoryRepository, CategoryError,
};

use crate::{
    error::ApiError,
    middleware::auth::{AuthContext, require_permission, Permission},
    supabase::SupabasePool,
};

// =============================================================================
// Request/Response Types
// =============================================================================

/// Create category request
#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub code: String,
    pub name: String,
    pub category_type: CategoryType,
    pub organization_id: Option<Uuid>,
    pub parent_category_id: Option<Uuid>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

/// Update category request
#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

/// Move category request
#[derive(Debug, Deserialize)]
pub struct MoveCategoryRequest {
    pub new_parent_id: Option<Uuid>,
}

/// Categorize object request
#[derive(Debug, Deserialize)]
pub struct CategorizeObjectRequest {
    pub object_id: Uuid,
    pub category_id: Uuid,
    pub is_primary: Option<bool>,
}

/// Uncategorize object request
#[derive(Debug, Deserialize)]
pub struct UncategorizeObjectRequest {
    pub object_id: Uuid,
    pub category_id: Uuid,
}

/// Categorize domain request
#[derive(Debug, Deserialize)]
pub struct CategorizeDomainRequest {
    pub domain_id: Uuid,
    pub category_id: Uuid,
}

/// List categories query parameters
#[derive(Debug, Deserialize)]
pub struct ListCategoriesQuery {
    pub category_type: Option<CategoryType>,
    pub organization_id: Option<Uuid>,
    pub include_inactive: Option<bool>,
}

/// Category response
#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category_type: CategoryType,
    pub organization_id: Option<Uuid>,
    pub parent_category_id: Option<Uuid>,
    pub level: i32,
    pub path: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Category> for CategoryResponse {
    fn from(cat: Category) -> Self {
        Self {
            id: cat.id,
            code: cat.code,
            name: cat.name,
            description: cat.description,
            category_type: cat.category_type,
            organization_id: cat.organization_id,
            parent_category_id: cat.parent_category_id,
            level: cat.level,
            path: cat.path,
            icon: cat.icon,
            color: cat.color,
            sort_order: cat.sort_order,
            is_active: cat.is_active,
            created_at: cat.created_at,
            updated_at: cat.updated_at,
        }
    }
}

/// Category node response (for tree structure)
#[derive(Debug, Serialize)]
pub struct CategoryNodeResponse {
    pub category: CategoryResponse,
    pub children: Vec<CategoryNodeResponse>,
    pub object_count: i32,
    pub has_objects_in_subtree: bool,
}

impl From<CategoryNode> for CategoryNodeResponse {
    fn from(node: CategoryNode) -> Self {
        Self {
            category: CategoryResponse::from(node.category),
            children: node.children.into_iter().map(CategoryNodeResponse::from).collect(),
            object_count: node.object_count,
            has_objects_in_subtree: node.has_objects_in_subtree,
        }
    }
}

/// Category statistics response
#[derive(Debug, Serialize)]
pub struct CategoryStatsResponse {
    pub category_id: Uuid,
    pub category_name: String,
    pub direct_objects_count: i32,
    pub total_objects_in_subtree: i32,
    pub child_categories_count: i32,
    pub depth: i32,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<CategoryStats> for CategoryStatsResponse {
    fn from(stats: CategoryStats) -> Self {
        Self {
            category_id: stats.category_id,
            category_name: stats.category_name,
            direct_objects_count: stats.direct_objects_count,
            total_objects_in_subtree: stats.total_objects_in_subtree,
            child_categories_count: stats.child_categories_count,
            depth: stats.depth,
            last_used: stats.last_used,
        }
    }
}

// =============================================================================
// Handlers
// =============================================================================

/// Create a new category
///
/// Creates a new category with the specified properties.
/// Requires `CategoryCreate` permission.
pub async fn create_category(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    require_permission(&auth, Permission::CategoryCreate)?;

    let repo = CategoryRepository::new(pool.into_inner());

    let category = repo.create_category(
        req.code,
        req.name,
        req.category_type,
        req.organization_id,
        req.parent_category_id,
        req.description,
        req.icon,
        req.color,
        req.sort_order,
    ).await?;

    Ok(Json(CategoryResponse::from(category)))
}

/// Get category by ID
///
/// Retrieves a single category by its ID.
/// Requires `CategoryRead` permission.
pub async fn get_category(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(category_id): Path<Uuid>,
) -> Result<Json<CategoryResponse>, ApiError> {
    require_permission(&auth, Permission::CategoryRead)?;

    let repo = CategoryRepository::new(pool.into_inner());
    let category = repo.get_category(category_id).await?;

    Ok(Json(CategoryResponse::from(category)))
}

/// Get category by code
///
/// Retrieves a category by its type and code.
/// Requires `CategoryRead` permission.
pub async fn get_category_by_code(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path((category_type, code)): Path<(String, String)>,
) -> Result<Json<CategoryResponse>, ApiError> {
    require_permission(&auth, Permission::CategoryRead)?;

    let category_type = category_type.parse()
        .map_err(|_| ApiError::BadRequest("Invalid category type".to_string()))?;

    let repo = CategoryRepository::new(pool.into_inner());
    let category = repo.get_category_by_code(category_type, &code).await?;

    Ok(Json(CategoryResponse::from(category)))
}

/// List categories
///
/// Lists categories with optional filtering by type and organization.
/// Requires `CategoryRead` permission.
pub async fn list_categories(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<CategoryResponse>>, ApiError> {
    require_permission(&auth, Permission::CategoryRead)?;

    let repo = CategoryRepository::new(pool.into_inner());
    let categories = repo.list_categories(
        query.category_type,
        query.organization_id,
        query.include_inactive.unwrap_or(false),
    ).await?;

    Ok(Json(categories.into_iter().map(CategoryResponse::from).collect()))
}

/// Get category tree
///
/// Returns the hierarchical tree structure for a category type.
/// Requires `CategoryRead` permission.
pub async fn get_category_tree(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(category_type): Path<String>,
) -> Result<Json<CategoryNodeResponse>, ApiError> {
    require_permission(&auth, Permission::CategoryRead)?;

    let category_type = category_type.parse()
        .map_err(|_| ApiError::BadRequest("Invalid category type".to_string()))?;

    let repo = CategoryRepository::new(pool.into_inner());
    let tree = repo.get_category_tree(category_type).await?;

    Ok(Json(CategoryNodeResponse::from(tree)))
}

/// Update category
///
/// Updates an existing category's properties.
/// Requires `CategoryUpdate` permission.
pub async fn update_category(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(category_id): Path<Uuid>,
    Json(req): Json<UpdateCategoryRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    require_permission(&auth, Permission::CategoryUpdate)?;

    let repo = CategoryRepository::new(pool.into_inner());
    let category = repo.update_category(
        category_id,
        req.name,
        req.description,
        req.icon,
        req.color,
        req.sort_order,
        req.is_active,
    ).await?;

    Ok(Json(CategoryResponse::from(category)))
}

/// Move category to new parent
///
/// Moves a category to a new position in the hierarchy.
/// Requires `CategoryUpdate` permission.
pub async fn move_category(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(category_id): Path<Uuid>,
    Json(req): Json<MoveCategoryRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    require_permission(&auth, Permission::CategoryUpdate)?;

    let repo = CategoryRepository::new(pool.into_inner());
    let category = repo.move_category(category_id, req.new_parent_id).await?;

    Ok(Json(CategoryResponse::from(category)))
}

/// Delete category
///
/// Deletes a category. Only allowed if category has no children.
/// Requires `CategoryDelete` permission.
pub async fn delete_category(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(category_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, ApiError> {
    require_permission(&auth, Permission::CategoryDelete)?;

    let repo = CategoryRepository::new(pool.into_inner());
    repo.delete_category(category_id).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// Categorize an object
///
/// Associates a category with an information object.
/// Requires `ObjectUpdate` permission.
pub async fn categorize_object(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CategorizeObjectRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    require_permission(&auth, Permission::ObjectUpdate)?;

    let repo = CategoryRepository::new(pool.into_inner());

    repo.categorize_object(
        req.object_id,
        req.category_id,
        req.is_primary.unwrap_or(false),
        auth.user_id,
    ).await?;

    // Return the updated category
    let category = repo.get_category(req.category_id).await?;
    Ok(Json(CategoryResponse::from(category)))
}

/// Uncategorize an object
///
/// Removes a category from an information object.
/// Requires `ObjectUpdate` permission.
pub async fn uncategorize_object(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<UncategorizeObjectRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    require_permission(&auth, Permission::ObjectUpdate)?;

    let repo = CategoryRepository::new(pool.into_inner());
    repo.uncategorize_object(req.object_id, req.category_id).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// Get categories for an object
///
/// Lists all categories associated with a specific object.
/// Requires `ObjectRead` permission.
pub async fn get_object_categories(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(object_id): Path<Uuid>,
) -> Result<Json<Vec<CategoryResponse>>, ApiError> {
    require_permission(&auth, Permission::ObjectRead)?;

    let repo = CategoryRepository::new(pool.into_inner());
    let categories = repo.get_object_categories(object_id).await?;

    Ok(Json(categories.into_iter().map(CategoryResponse::from).collect()))
}

/// Get category statistics
///
/// Returns usage statistics for a specific category including subtree counts.
/// Requires `CategoryRead` permission.
pub async fn get_category_stats(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(category_id): Path<Uuid>,
) -> Result<Json<CategoryStatsResponse>, ApiError> {
    require_permission(&auth, Permission::CategoryRead)?;

    let repo = CategoryRepository::new(pool.into_inner());
    let stats = repo.get_category_stats(category_id).await?;

    Ok(Json(CategoryStatsResponse::from(stats)))
}

impl From<CategoryError> for ApiError {
    fn from(err: CategoryError) -> Self {
        match err {
            CategoryError::CategoryNotFound(id) => ApiError::NotFound(format!("Category not found: {}", id)),
            CategoryError::ObjectNotFound(id) => ApiError::NotFound(format!("Object not found: {}", id)),
            CategoryError::CycleDetected => ApiError::BadRequest("Cannot create cycle in category hierarchy".to_string()),
            CategoryError::DatabaseError(e) => ApiError::Internal(format!("Database error: {}", e)),
            _ => ApiError::Internal(format!("Category error: {}", err)),
        }
    }
}
