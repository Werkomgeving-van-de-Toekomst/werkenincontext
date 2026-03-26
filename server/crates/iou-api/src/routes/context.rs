//! Context and domain endpoints

use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::db::Database;
use crate::error::ApiError;
use iou_core::api_types::{ContextResponse, CreateDomainRequest, CreateDomainResponse};
use iou_core::domain::{DomainStatus, DomainType, InformationDomain};

/// Query parameters for listing domains
#[derive(Debug, Deserialize)]
pub struct ListDomainsQuery {
    #[serde(rename = "type")]
    pub domain_type: Option<String>,
    pub status: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
}

fn default_limit() -> i32 {
    50
}

/// GET /context/:id - Get complete context for a domain
pub async fn get_context(
    Path(domain_id): Path<Uuid>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Json<ContextResponse>, ApiError> {
    // Get the main domain
    let domain = db
        .get_domain(domain_id)?
        .ok_or_else(|| ApiError::NotFound(format!("Domain {} not found", domain_id)))?;

    // TODO: Fetch related data
    // - Related domains
    // - Recent objects
    // - Recommended apps
    // - Stakeholders
    // - Compliance status

    let response = ContextResponse {
        current_domain: domain,
        domain_details: None,
        related_domains: vec![],
        recent_objects: vec![],
        recommended_apps: vec![],
        stakeholders: vec![],
        compliance_status: None,
    };

    Ok(Json(response))
}

/// GET /domains - List all domains
pub async fn list_domains(
    Query(params): Query<ListDomainsQuery>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Json<Vec<InformationDomain>>, ApiError> {
    let domain_type = params.domain_type.as_ref().map(|t| match t.as_str() {
        "zaak" => DomainType::Zaak,
        "project" => DomainType::Project,
        "beleid" => DomainType::Beleid,
        "expertise" => DomainType::Expertise,
        _ => DomainType::Zaak,
    });

    let status = params.status.as_ref().map(|s| match s.as_str() {
        "concept" => DomainStatus::Concept,
        "actief" => DomainStatus::Actief,
        "afgerond" => DomainStatus::Afgerond,
        "gearchiveerd" => DomainStatus::Gearchiveerd,
        _ => DomainStatus::Actief,
    });

    let domains = db.list_domains(domain_type, status, params.limit, params.offset)?;

    Ok(Json(domains))
}

/// POST /domains - Create a new domain
pub async fn create_domain(
    Extension(db): Extension<Arc<Database>>,
    Json(request): Json<CreateDomainRequest>,
) -> Result<Json<CreateDomainResponse>, ApiError> {
    // Create the domain
    let domain = InformationDomain::new(request.domain_type, request.name, request.organization_id);

    // TODO: Apply AI suggestions for metadata
    let ai_suggestions = vec![];

    // Save to database
    db.create_domain(&domain)?;

    Ok(Json(CreateDomainResponse {
        domain,
        ai_suggestions,
    }))
}
