//! Purpose API endpoints
//!
//! HTTP handlers for the Purpose Registry (IHH01/IHH02)
//! Manages purposes for data processing according to AVG/GDPR

use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::ApiError,
    middleware::auth::{AuthContext, require_permission, Permission},
    middleware::purpose::{PurposeContext, PurposeState},
    supabase::SupabasePool,
};

use iou_core::purpose::{LawfulBasis, Purpose as CorePurpose, PurposeError, PurposeRegistry};

// =============================================================================
// State
// =============================================================================

/// Purpose service state (shared via Axum Extension/State)
#[derive(Clone)]
pub struct PurposeServiceState {
    pub registry: Arc<PurposeRegistry>,
    pub pool: sqlx::PgPool,
}

impl PurposeServiceState {
    pub fn new(registry: PurposeRegistry, pool: sqlx::PgPool) -> Self {
        Self {
            registry: Arc::new(registry),
            pool,
        }
    }
}

// =============================================================================
// Request/Response Types
// =============================================================================

/// Purpose response
#[derive(Debug, Serialize)]
pub struct PurposeResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub lawful_basis: LawfulBasis,
    pub data_categories: Vec<String>,
    pub owner: String,
    pub requires_approval: bool,
    pub valid_from: Option<NaiveDate>,
    pub valid_until: Option<NaiveDate>,
    pub is_active: bool,
    pub is_valid_now: bool,
    pub is_standard: bool,
}

impl From<CorePurpose> for PurposeResponse {
    fn from(purpose: CorePurpose) -> Self {
        let is_valid_now = purpose.is_valid_now();
        let is_standard = purpose.id.starts_with('P')
            && purpose.id.len() == 4
            && purpose.id[1..].parse::<u32>().map_or(false, |n| (1..=15).contains(&n));

        Self {
            id: purpose.id,
            name: purpose.name,
            description: purpose.description,
            lawful_basis: purpose.lawful_basis,
            data_categories: purpose.data_categories,
            owner: purpose.owner,
            requires_approval: purpose.requires_approval,
            valid_from: purpose.valid_from,
            valid_until: purpose.valid_until,
            is_active: purpose.is_active,
            is_valid_now,
            is_standard,
        }
    }
}

/// Create purpose request
#[derive(Debug, Deserialize)]
pub struct CreatePurposeRequest {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub lawful_basis: LawfulBasis,
    pub data_categories: Vec<String>,
    pub owner: String,
    pub requires_approval: Option<bool>,
    pub valid_from: Option<NaiveDate>,
    pub valid_until: Option<NaiveDate>,
}

/// Update purpose request
#[derive(Debug, Deserialize)]
pub struct UpdatePurposeRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub data_categories: Option<Vec<String>>,
    pub owner: Option<String>,
    pub requires_approval: Option<bool>,
    pub valid_from: Option<Option<NaiveDate>>,
    pub valid_until: Option<Option<NaiveDate>>,
    pub is_active: Option<bool>,
}

/// Purpose validation request
#[derive(Debug, Deserialize)]
pub struct ValidatePurposeRequest {
    pub purpose_id: String,
    pub data_category: Option<String>,
}

/// Purpose validation response
#[derive(Debug, Serialize)]
pub struct ValidatePurposeResponse {
    pub valid: bool,
    pub purpose: Option<PurposeResponse>,
    pub error: Option<String>,
}

/// Purpose approval request
#[derive(Debug, Deserialize)]
pub struct ApprovePurposeRequest {
    pub purpose_id: String,
    pub approved_by: Uuid,
    pub justification: String,
}

/// Purpose approval response
#[derive(Debug, Serialize)]
pub struct PurposeApprovalResponse {
    pub purpose_id: String,
    pub approved: bool,
    pub approved_at: chrono::DateTime<chrono::Utc>,
    pub approved_by: Uuid,
}

/// List purposes query parameters
#[derive(Debug, Deserialize)]
pub struct ListPurposesQuery {
    pub lawful_basis: Option<LawfulBasis>,
    pub include_inactive: Option<bool>,
    pub include_expired: Option<bool>,
    pub data_category: Option<String>,
}

/// Purpose statistics
#[derive(Debug, Serialize)]
pub struct PurposeStatsResponse {
    pub total_purposes: usize,
    pub active_purposes: usize,
    pub standard_purposes: usize,
    pub custom_purposes: usize,
    pub pending_approval: usize,
}

/// Purpose usage for an information object
#[derive(Debug, Serialize)]
pub struct PurposeUsageResponse {
    pub purpose_id: String,
    pub purpose_name: String,
    pub lawful_basis: LawfulBasis,
    pub object_count: i64,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

// =============================================================================
// Handlers
// =============================================================================

/// GET /api/v1/purposes - List all purposes
///
/// Lists purposes with optional filtering.
/// Requires `PurposeRead` permission.
pub async fn list_purposes(
    State(state): State<Arc<PurposeServiceState>>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<ListPurposesQuery>,
) -> Result<Json<Vec<PurposeResponse>>, ApiError> {
    require_permission(&auth, Permission::PurposeRead)?;

    let mut purposes = if query.include_inactive.unwrap_or(false) {
        state.registry.list_all()
    } else {
        state.registry.list_active()
    };

    // Filter by lawful basis
    if let Some(basis) = query.lawful_basis {
        purposes.retain(|p| p.lawful_basis == basis);
    }

    // Filter by data category
    if let Some(category) = query.data_category {
        purposes.retain(|p| p.can_use_data_category(&category));
    }

    // Filter out expired unless requested
    if !query.include_expired.unwrap_or(false) {
        purposes.retain(|p| p.is_valid_now());
    }

    Ok(Json(purposes.into_iter().map(PurposeResponse::from).collect()))
}

/// GET /api/v1/purposes/:id - Get purpose by ID
///
/// Retrieves a single purpose by its ID.
/// Requires `PurposeRead` permission.
pub async fn get_purpose(
    State(state): State<Arc<PurposeServiceState>>,
    Extension(auth): Extension<AuthContext>,
    Path(purpose_id): Path<String>,
) -> Result<Json<PurposeResponse>, ApiError> {
    require_permission(&auth, Permission::PurposeRead)?;

    let purpose = state
        .registry
        .get(&purpose_id)
        .map_err(|e| match e {
            PurposeError::NotFound(id) => ApiError::NotFound(id),
            _ => ApiError::Internal(e.into()),
        })?;

    Ok(Json(PurposeResponse::from(purpose)))
}

/// POST /api/v1/purposes - Create custom purpose
///
/// Creates a new custom purpose (not standard purposes).
/// Requires `PurposeCreate` permission.
pub async fn create_purpose(
    State(state): State<Arc<PurposeServiceState>>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CreatePurposeRequest>,
) -> Result<Json<PurposeResponse>, ApiError> {
    require_permission(&auth, Permission::PurposeCreate)?;

    // Generate ID if not provided
    let id = req.id.unwrap_or_else(|| {
        format!("C{:04}", Uuid::new_v4().simple()[..4].to_uppercase())
    });

    // Prevent creating standard purpose IDs
    if id.starts_with('P') && id.len() == 4 {
        if let Ok(num) = id[1..].parse::<u32>() {
            if (1..=15).contains(&num) {
                return Err(ApiError::BadRequest(
                    "Cannot create standard purposes (P001-P015)".to_string(),
                ));
            }
        }
    }

    let mut purpose = CorePurpose::new(&id, &req.name, &req.description, req.lawful_basis, &req.owner)
        .with_data_categories(req.data_categories)
        .with_validity(req.valid_from, req.valid_until);

    if let Some(approval) = req.requires_approval {
        purpose.requires_approval = approval;
    }

    state
        .registry
        .register(purpose.clone())
        .map_err(|e| match e {
            PurposeError::AlreadyExists(id) => ApiError::Conflict(format!("Purpose already exists: {}", id)),
            _ => ApiError::Internal(e.into()),
        })?;

    Ok(Json(PurposeResponse::from(purpose)))
}

/// PUT /api/v1/purposes/:id - Update purpose
///
/// Updates an existing custom purpose.
/// Requires `PurposeUpdate` permission.
pub async fn update_purpose(
    State(state): State<Arc<PurposeServiceState>>,
    Extension(auth): Extension<AuthContext>,
    Path(purpose_id): Path<String>,
    Json(req): Json<UpdatePurposeRequest>,
) -> Result<Json<PurposeResponse>, ApiError> {
    require_permission(&auth, Permission::PurposeUpdate)?;

    // Get current purpose
    let mut purpose = state
        .registry
        .get(&purpose_id)
        .map_err(|e| match e {
            PurposeError::NotFound(id) => ApiError::NotFound(id),
            _ => ApiError::Internal(e.into()),
        })?;

    // Cannot update standard purposes
    if purpose_id.starts_with('P') && purpose_id.len() == 4 {
        if let Ok(num) = purpose_id[1..].parse::<u32>() {
            if (1..=15).contains(&num) {
                return Err(ApiError::Forbidden(
                    "Cannot update standard purposes".to_string(),
                ));
            }
        }
    }

    // Update fields
    if let Some(name) = req.name {
        purpose.name = name;
    }
    if let Some(description) = req.description {
        purpose.description = description;
    }
    if let Some(categories) = req.data_categories {
        purpose.data_categories = categories;
    }
    if let Some(owner) = req.owner {
        purpose.owner = owner;
    }
    if let Some(approval) = req.requires_approval {
        purpose.requires_approval = approval;
    }
    if let Some(from) = req.valid_from {
        purpose.valid_from = from;
    }
    if let Some(until) = req.valid_until {
        purpose.valid_until = until;
    }
    if let Some(active) = req.is_active {
        purpose.is_active = active;
    }

    // Re-register the updated purpose
    state
        .registry
        .register(purpose.clone())
        .map_err(|e| ApiError::Internal(e.into()))?;

    Ok(Json(PurposeResponse::from(purpose)))
}

/// DELETE /api/v1/purposes/:id - Deactivate purpose
///
/// Deactivates a purpose (doesn't delete, just marks inactive).
/// Requires `PurposeDelete` permission.
pub async fn delete_purpose(
    State(state): State<Arc<PurposeServiceState>>,
    Extension(auth): Extension<AuthContext>,
    Path(purpose_id): Path<String>,
) -> Result<axum::http::StatusCode, ApiError> {
    require_permission(&auth, Permission::PurposeDelete)?;

    // Cannot deactivate standard purposes
    if purpose_id.starts_with('P') && purpose_id.len() == 4 {
        if let Ok(num) = purpose_id[1..].parse::<u32>() {
            if (1..=15).contains(&num) {
                return Err(ApiError::Forbidden(
                    "Cannot deactivate standard purposes".to_string(),
                ));
            }
        }
    }

    let mut purpose = state
        .registry
        .get(&purpose_id)
        .map_err(|e| match e {
            PurposeError::NotFound(id) => ApiError::NotFound(id),
            _ => ApiError::Internal(e.into()),
        })?;

    purpose.is_active = false;

    state
        .registry
        .register(purpose)
        .map_err(|e| ApiError::Internal(e.into()))?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// POST /api/v1/purposes/validate - Validate purpose for use
///
/// Validates that a purpose can be used for a specific data category.
/// Requires `PurposeRead` permission.
pub async fn validate_purpose(
    State(state): State<Arc<PurposeServiceState>>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<ValidatePurposeRequest>,
) -> Result<Json<ValidatePurposeResponse>, ApiError> {
    require_permission(&auth, Permission::PurposeRead)?;

    let result = if let Some(category) = req.data_category {
        state
            .registry
            .validate_for_category(&req.purpose_id, &category)
    } else {
        state.registry.validate(&req.purpose_id)
    };

    match result {
        Ok(purpose) => Ok(Json(ValidatePurposeResponse {
            valid: true,
            purpose: Some(PurposeResponse::from(purpose)),
            error: None,
        })),
        Err(e) => Ok(Json(ValidatePurposeResponse {
            valid: false,
            purpose: None,
            error: Some(e.to_string()),
        })),
    }
}

/// GET /api/v1/purposes/stats - Get purpose statistics
///
/// Returns statistics about purposes in the registry.
/// Requires `PurposeRead` permission.
pub async fn get_purpose_stats(
    State(state): State<Arc<PurposeServiceState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<PurposeStatsResponse>, ApiError> {
    require_permission(&auth, Permission::PurposeRead)?;

    let all_purposes = state.registry.list_all();
    let active_purposes = state.registry.list_active();

    let standard_count = all_purposes
        .iter()
        .filter(|p| p.id.starts_with('P') && p.id.len() == 4)
        .count();

    let custom_count = all_purposes.len() - standard_count;

    let pending_approval = all_purposes
        .iter()
        .filter(|p| p.requires_approval)
        .count();

    Ok(Json(PurposeStatsResponse {
        total_purposes: all_purposes.len(),
        active_purposes: active_purposes.len(),
        standard_purposes: standard_count,
        custom_purposes: custom_count,
        pending_approval,
    }))
}

/// GET /api/v1/purposes/usage - Get purpose usage statistics
///
/// Returns usage statistics for all purposes.
/// Requires `PurposeRead` permission.
pub async fn get_purpose_usage(
    State(state): State<Arc<PurposeServiceState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<PurposeUsageResponse>>, ApiError> {
    require_permission(&auth, Permission::PurposeRead)?;

    let rows = sqlx::query(
        r#"
        SELECT
            p.purpose_id,
            p.purpose_name,
            p.lawful_basis,
            COUNT(DISTINCT io.id) as object_count,
            MAX(io.created_at) as last_used
        FROM information_object_purposes iop
        INNER JOIN purposes p ON iop.purpose_id = p.id::text
        INNER JOIN information_objects io ON iop.object_id = io.id
        GROUP BY p.purpose_id, p.purpose_name, p.lawful_basis
        ORDER BY object_count DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;

    let usage = rows
        .into_iter()
        .map(|row| PurposeUsageResponse {
            purpose_id: row.get("purpose_id"),
            purpose_name: row.get("purpose_name"),
            lawful_basis: row.get("lawful_basis"),
            object_count: row.get("object_count"),
            last_used: row.get("last_used"),
        })
        .collect();

    Ok(Json(usage))
}

/// GET /api/v1/purposes/lawful-bases - List lawful bases
///
/// Returns all available lawful bases for purposes.
pub async fn list_lawful_bases() -> Json<Vec<LawfulBasisInfo>> {
    Json(vec![
        LawfulBasisInfo {
            id: LawfulBasis::WettelijkeVerplichting,
            name: "Wettelijke Verplichting",
            description: "Verwerking is noodzakelijk om te voldoen aan een wettelijke verplichting",
            article: "AVG Art. 6 lid 1 onder c",
        },
        LawfulBasisInfo {
            id: LawfulBasis::Openbaarmaking,
            name: "Openbaarmaking",
            description: "Verwerking is noodzakelijk voor de vervulling van een taak van algemeen belang",
            article: "AVG Art. 6 lid 1 onder e",
        },
        LawfulBasisInfo {
            id: LawfulBasis::Toestemming,
            name: "Toestemming",
            description: "De betrokkene heeft toestemming gegeven voor de verwerking",
            article: "AVG Art. 6 lid 1 onder a",
        },
        LawfulBasisInfo {
            id: LawfulBasis::ContractueleVerplichting,
            name: "Contractuele Verplichting",
            description: "Verwerking is noodzakelijk voor de uitvoering van een overeenkomst",
            article: "AVG Art. 6 lid 1 onder b",
        },
        LawfulBasisInfo {
            id: LawfulBasis::VitaleBelangen,
            name: "Vitale Belangen",
            description: "Verwerking is noodzakelijk ter bescherming van vitale belangen",
            article: "AVG Art. 6 lid 1 onder d",
        },
        LawfulBasisInfo {
            id: LawfulBasis::OpenbaarTaakbelang,
            name: "Openbaar Taakbelang",
            description: "Verwerking is noodzakelijk voor de uitvoering van een taak van algemeen belang",
            article: "AVG Art. 6 lid 1 onder e",
        },
        LawfulBasisInfo {
            id: LawfulBasis::GerechtvaardigdBelang,
            name: "Gerechtvaardigd Belang",
            description: "Verwerking is noodzakelijk voor de behartiging van een gerechtvaardigd belang",
            article: "AVG Art. 6 lid 1 onder f",
        },
    ])
}

#[derive(Debug, Serialize)]
struct LawfulBasisInfo {
    id: LawfulBasis,
    name: &'static str,
    description: &'static str,
    article: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purpose_response_from_core() {
        let purpose = CorePurpose::new(
            "P001",
            "Test Purpose",
            "Test description",
            LawfulBasis::WettelijkeVerplichting,
            "Owner",
        );

        let response = PurposeResponse::from(purpose);

        assert_eq!(response.id, "P001");
        assert_eq!(response.name, "Test Purpose");
        assert!(response.is_valid_now);
        assert!(response.is_standard);
    }

    #[test]
    fn test_lawful_basis_info_serialization() {
        let info = LawfulBasisInfo {
            id: LawfulBasis::WettelijkeVerplichting,
            name: "Wettelijke Verplichting",
            description: "Test",
            article: "AVG Art. 6",
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("wettelijke_verplichting"));
    }
}
