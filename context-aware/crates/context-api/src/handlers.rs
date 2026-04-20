// =============================================================================
// Request Handlers for Context API
// =============================================================================

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use context_core::{Context, ContextId};
use context_store::{ContextQuery, ContextRepository, ArangoContextRepository};

use super::{AppState, ApiResult, ApiError};

/// Context query parameters
#[derive(Debug, Deserialize)]
pub struct ContextListParams {
    pub object_id: Option<Uuid>,
    pub org_id: Option<String>,
    pub domein: Option<String>,
    pub limit: Option<usize>,
}

/// Create context request
#[derive(Debug, Deserialize)]
pub struct CreateContextRequest {
    pub informatieobject_id: Uuid,
    pub organisatie_id: String,
    pub actor: ActorRequest,
    pub temporal: TemporalRequest,
    pub domain: DomainRequest,
    pub purpose: Option<PurposeRequest>,
    pub semantic: Option<SemanticRequest>,
    pub provenance: ProvenanceRequest,
}

#[derive(Debug, Deserialize)]
pub struct ActorRequest {
    pub actor_type: String,
    pub actor_id: String,
    pub display_name: String,
    pub organisatie_eenheid: Option<String>,
    pub rol: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TemporalRequest {
    pub geldig_vanaf: Option<chrono::DateTime<chrono::Utc>>,
    pub geldig_tot: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct DomainRequest {
    pub primair_domein: String,
    pub domein_data: serde_json::Value,
    pub labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PurposeRequest {
    pub grondslagen: Vec<GrondslagRequest>,
    pub beleidsreferenties: Vec<BeleidsreferentieRequest>,
    pub zakelijk_doel: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GrondslagRequest {
    pub grondslag_id: String,
    pub bron: String,
    pub artikel: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BeleidsreferentieRequest {
    pub beleid_id: String,
    pub referentie_type: String,
}

#[derive(Debug, Deserialize)]
pub struct SemanticRequest {
    pub trefwoorden: Vec<String>,
    pub onderwerpen: Vec<String>,
    pub samenvatting: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProvenanceRequest {
    pub bronsysteem: String,
    pub bron_referentie: String,
    pub bron_betrouwbaarheid: f64,
}

/// Context response
#[derive(Debug, Serialize)]
pub struct ContextResponse {
    pub id: ContextId,
    pub informatieobject_id: Uuid,
    pub organisatie_id: String,
    pub actor: ActorResponse,
    pub temporal: TemporalResponse,
    pub domain: DomainResponse,
    pub purpose: Option<PurposeResponse>,
    pub semantic: Option<SemanticResponse>,
    pub provenance: ProvenanceResponse,
    pub quality: QualityResponse,
    pub geldigheid: GeldigheidResponse,
    pub metadata: MetadataResponse,
}

#[derive(Debug, Serialize)]
pub struct ActorResponse {
    pub actor_type: String,
    pub actor_id: String,
    pub display_name: String,
    pub organisatie_eenheid: Option<String>,
    pub rol: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TemporalResponse {
    pub aangemaakt_op: chrono::DateTime<chrono::Utc>,
    pub gewijzigd_op: chrono::DateTime<chrono::Utc>,
    pub geldig_vanaf: Option<chrono::DateTime<chrono::Utc>>,
    pub geldig_tot: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct DomainResponse {
    pub primair_domein: String,
    pub domein_data: serde_json::Value,
    pub labels: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PurposeResponse {
    pub grondslagen: Vec<GrondslagResponse>,
    pub beleidsreferenties: Vec<BeleidsreferentieResponse>,
    pub zakelijk_doel: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GrondslagResponse {
    pub grondslag_id: String,
    pub bron: String,
    pub artikel: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BeleidsreferentieResponse {
    pub beleid_id: String,
    pub referentie_type: String,
}

#[derive(Debug, Serialize)]
pub struct SemanticResponse {
    pub trefwoorden: Vec<String>,
    pub onderwerpen: Vec<String>,
    pub samenvatting: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProvenanceResponse {
    pub bronsysteem: String,
    pub bron_referentie: String,
    pub bron_betrouwbaarheid: f64,
}

#[derive(Debug, Serialize)]
pub struct QualityResponse {
    pub volledigheid: f64,
    pub nauwkeurigheid: f64,
    pub consistentie: f64,
    pub actualiteit: f64,
    pub kwaliteit: f64,
}

#[derive(Debug, Serialize)]
pub struct GeldigheidResponse {
    pub status: String,
    pub begindatum: Option<chrono::DateTime<chrono::Utc>>,
    pub einddatum: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct MetadataResponse {
    pub aangemaakt_door: String,
    pub aangemaakt_op: chrono::DateTime<chrono::Utc>,
    pub gewijzigd_door: Option<String>,
    pub gewijzigd_op: Option<chrono::DateTime<chrono::Utc>>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Handlers
// ============================================================================

/// List contexts with optional filters
pub async fn list_contexts(
    State(state): State<AppState>,
    Query(params): Query<ContextListParams>,
) -> ApiResult<Json<Vec<ContextResponse>>> {
    let repo = ArangoContextRepository::new(state.pool.clone());

    let query = ContextQuery {
        object_id: params.object_id,
        organisation_id: params.org_id,
        domein: None, // TODO: parse from params
        limit: params.limit,
        offset: None,
    };

    let contexts = repo.search(&query).await?;

    let responses = contexts.into_iter().map(|c| to_response(c)).collect();

    Ok(Json(responses))
}

/// Get context by ID
pub async fn get_context(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ContextResponse>> {
    let repo = ArangoContextRepository::new(state.pool.clone());
    let context_id = id.parse().map_err(|_| ApiError::InvalidRequest("Invalid context ID".to_string()))?;

    let context = repo.get(&context_id).await?;

    Ok(Json(to_response(context)))
}

/// Create new context
pub async fn create_context(
    State(state): State<AppState>,
    Json(req): Json<CreateContextRequest>,
) -> ApiResult<Json<ContextResponse>> {
    let repo = ArangoContextRepository::new(state.pool.clone());

    let context = to_context(req)?;
    let id = repo.create(&context).await?;

    // Fetch the created context
    let created = repo.get(&id).await?;

    Ok(Json(to_response(created)))
}

/// Update existing context
pub async fn update_context(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CreateContextRequest>,
) -> ApiResult<Json<ContextResponse>> {
    let repo = ArangoContextRepository::new(state.pool.clone());
    let context_id = id.parse().map_err(|_| ApiError::InvalidRequest("Invalid context ID".to_string()))?;

    let mut context = to_context(req)?;
    context.id = context_id;

    repo.update(&context).await?;

    let updated = repo.get(&context_id).await?;

    Ok(Json(to_response(updated)))
}

/// Delete context
pub async fn delete_context(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let repo = ArangoContextRepository::new(state.pool.clone());
    let context_id = id.parse().map_err(|_| ApiError::InvalidRequest("Invalid context ID".to_string()))?;

    repo.delete(&context_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get contexts by information object
pub async fn get_contexts_by_object(
    State(state): State<AppState>,
    Path(object_id): Path<String>,
) -> ApiResult<Json<Vec<ContextResponse>>> {
    let repo = ArangoContextRepository::new(state.pool.clone());
    let uuid = Uuid::parse_str(&object_id).map_err(|_| ApiError::InvalidRequest("Invalid object ID".to_string()))?;

    let contexts = repo.list_by_object(&uuid).await?;

    let responses = contexts.into_iter().map(|c| to_response(c)).collect();

    Ok(Json(responses))
}

/// Get contexts by organization
pub async fn get_contexts_by_org(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> ApiResult<Json<Vec<ContextResponse>>> {
    let repo = ArangoContextRepository::new(state.pool.clone());

    let contexts = repo.list_by_org(&org_id).await?;

    let responses = contexts.into_iter().map(|c| to_response(c)).collect();

    Ok(Json(responses))
}

/// Get context history
pub async fn get_context_history(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    // TODO: Implement using ContextRecordRepository
    Ok(Json(vec![]))
}

/// Get specific version of context
pub async fn get_context_version(
    State(_state): State<AppState>,
    Path((id, version)): Path<(String, u32)>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Implement using ContextRecordRepository
    Ok(Json(serde_json::json!({"message": "Not implemented"})))
}

/// List context types
pub async fn list_context_types(
    State(_state): State<AppState>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    // TODO: Implement
    Ok(Json(vec![]))
}

/// Create context type
pub async fn create_context_type(
    State(_state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Implement
    Ok(Json(serde_json::json!({"message": "Not implemented"})))
}

/// Get context type
pub async fn get_context_type(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Implement
    Ok(Json(serde_json::json!({"message": "Not implemented"})))
}

/// Update context type
pub async fn update_context_type(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Implement
    Ok(Json(serde_json::json!({"message": "Not implemented"})))
}

/// Analyze text for context inference
pub async fn analyze_context(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Implement using context-inference crate
    Ok(Json(serde_json::json!({"message": "Not implemented"})))
}

/// Get quality report
pub async fn get_quality_report(
    State(_state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Implement using context-quality crate
    Ok(Json(serde_json::json!({"message": "Not implemented"})))
}

/// Get quality score for specific context
pub async fn get_quality_score(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Implement using context-quality crate
    Ok(Json(serde_json::json!({"message": "Not implemented"})))
}

/// Health check
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
    })
}

// ============================================================================
// Helper functions
// ============================================================================

fn to_response(context: Context) -> ContextResponse {
    // TODO: Implement proper conversion
    ContextResponse {
        id: context.id,
        informatieobject_id: context.informatieobject_id,
        organisatie_id: context.organisatie_id,
        actor: ActorResponse {
            actor_type: format!("{:?}", context.actor.actor_type),
            actor_id: context.actor.actor_id,
            display_name: context.actor.display_name,
            organisatie_eenheid: context.actor.organisatie_eenheid,
            rol: context.actor.rol,
        },
        temporal: TemporalResponse {
            aangemaakt_op: context.temporal.aangemaakt_op.0,
            gewijzigd_op: context.temporal.gewijzigd_op.0,
            geldig_vanaf: context.temporal.geldig_vanaf,
            geldig_tot: context.temporal.geldig_tot,
        },
        domain: DomainResponse {
            primair_domein: format!("{:?}", context.domain.primair_domein),
            domein_data: serde_json::json!({}),
            labels: context.domain.labels,
        },
        purpose: None, // TODO
        semantic: None, // TODO
        provenance: ProvenanceResponse {
            bronsysteem: context.provenance.bronsyysteem,
            bron_referentie: context.provenance.bron_referentie,
            bron_betrouwbaarheid: context.provenance.bron_betrouwbaarheid,
        },
        quality: QualityResponse {
            volledigheid: context.quality.volledigheid,
            nauwkeurigheid: context.quality.nauwkeurigheid,
            consistentie: context.quality.consistentie,
            actualiteit: context.quality.actualiteit,
            kwaliteit: context.quality.kwaliteit,
        },
        geldigheid: GeldigheidResponse {
            status: format!("{:?}", context.geldigheid.status),
            begindatum: context.geldigheid.begindatum,
            einddatum: context.geldigheid.einddatum,
        },
        metadata: MetadataResponse {
            aangemaakt_door: context.metadata.aangemaakt_door,
            aangemaakt_op: context.metadata.aangemaakt_op.0,
            gewijzigd_door: context.metadata.gewijzigd_door,
            gewijzigd_op: context.metadata.gewijzigd_op.map(|t| t.0),
        },
    }
}

fn to_context(req: CreateContextRequest) -> Result<Context, ApiError> {
    // TODO: Implement proper conversion
    use context_core::*;
    use iou_core::{Id, Timestamp};

    let now = Timestamp::now();

    Ok(Context {
        id: Id::new(),
        informatieobject_id: req.informatieobject_id,
        organisatie_id: req.organisatiew_id,
        actor: Actor {
            actor_type: match req.actor.actor_type.as_str() {
                "persoon" => ActorType::Persoon,
                "systeem" => ActorType::Systeem,
                "service" => ActorType::Service,
                _ => return Err(ApiError::InvalidRequest("Invalid actor type".to_string())),
            },
            actor_id: req.actor.actor_id,
            display_name: req.actor.display_name,
            organisatie_eenheid: req.actor.organisatie_eenheid,
            rol: req.actor.rol,
        },
        temporal: TemporalContext {
            aangemaakt_op: now,
            gewijzigd_op: now,
            geldig_vanaf: req.temporal.geldig_vanaf,
            geldig_tot: req.temporal.geldig_tot,
            referentie_tijd: None,
        },
        domain: DomainContext {
            primair_domein: Domein::Zaak(ZaakContext {
                zaak_id: "TODO".to_string(),
                zaak_type: "TODO".to_string(),
                zaak_fase: None,
            }),
            gerelateerde_domeinen: vec![],
            organisatie_pad: vec![],
            labels: req.domain.labels,
        },
        purpose: PurposeContext {
            grondslagen: vec![],
            beleidsreferenties: vec![],
            zakelijk_doel: None,
        },
        semantic: SemanticContext {
            trefwoorden: vec![],
            onderwerpen: vec![],
            entiteiten: vec![],
            samenvatting: None,
        },
        provenance: ProvenanceContext {
            bronsysteem: req.provenance.bronsyysteem,
            bron_referentie: req.provenance.bron_referentie,
            herkomst: vec![],
            bron_betrouwbaarheid: req.provenance.bron_betrouwbaarheid,
        },
        quality: QualityMetrics {
            volledigheid: 1.0,
            nauwkeurigheid: 1.0,
            consistentie: 1.0,
            actualiteit: 1.0,
            kwaliteit: 1.0,
        },
        geldigheid: Geldigheid {
            status: GeldigheidStatus::Actief,
            begindatum: Some(chrono::Utc::now()),
            einddatum: None,
        },
        metadata: ContextMetadata {
            aangemaakt_door: "system".to_string(),
            aangemaakt_op: now,
            gewijzigd_door: None,
            gewijzigd_op: None,
        },
    })
}
