// =============================================================================
// Context Bridge - Integration with Metadata Registry (Project 002)
// =============================================================================
//
// This module provides the bridge between context-aware data (Project 003)
// and the Metadata Registry (Project 002) that implements Metamodel GGHH V2.
//
// The bridge ensures:
// - Context data aligns with GGHH V2 Context entity
// - Bidirectional sync between working and metadata layers
// - Complete audit trail for Archiefwet 2025 compliance
// =============================================================================

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{Context, ContextId, QualityScore};

/// Bridge record linking context to GGHH Informatieobject
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextBridge {
    /// Unique bridge record ID
    pub bridge_id: Uuid,

    /// IOU-Modern InformationObject reference (Project 001)
    pub information_object_id: Uuid,

    /// GGHH Informatieobject reference (Project 002)
    pub informatieobject_id: Uuid,

    /// Context record reference (Project 003)
    pub context_id: ContextId,

    /// Synchronization status
    pub sync_status: SyncStatus,

    /// Last synchronized timestamp
    pub last_synced_at: DateTime<Utc>,

    /// Sync error message (if failed)
    pub sync_error: Option<String>,

    /// Metadata version from registry
    pub metadata_version: i32,
}

/// Synchronization status between layers
#[derive(Clone, Debug, Serialize, Deserialize, strum::Display)]
pub enum SyncStatus {
    /// Pending synchronization
    #[strum(serialize = "pending")]
    Pending,

    /// Successfully synchronized
    #[strum(serialize = "synced")]
    Synced,

    /// Synchronization failed
    #[strum(serialize = "failed")]
    Failed,

    /// Out of sync - needs update
    #[strum(serialize = "outdated")]
    Outdated,
}

/// GGHH V2 Context entity (from Project 002)
///
/// This represents the official GGHH Context entity that our context model
/// extends and implements.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GghhContext {
    /// Unique identifier (GGHH format)
    pub context_id: String,

    /// Actor who created/modified
    pub actor: GghhActor,

    /// Temporal validity
    pub geldigheid: GghgGeldigheid,

    /// Domain reference
    pub domein: Option<String>,

    /// Context type (from GGHH catalogue)
    pub context_type: String,

    /// Context value
    pub waarde: serde_json::Value,

    /// Quality indicators
    pub kwaliteit: Option<GghhKwaliteit>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GghhActor {
    pub actor_type: String,  // "persoon", "systeem", "service"
    pub actor_id: String,
    pub naam: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GghgGeldigheid {
    pub geldig_vanaf: Option<DateTime<Utc>>,
    pub geldig_tot: Option<DateTime<Utc>>,
    pub status: String,  // "actief", "inactief", "archief"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GghhKwaliteit {
    pub volledigheid: f64,
    pub nauwkeurigheid: f64,
}

/// Convert our Context to GGHH format for metadata registry
impl From<&Context> for GghhContext {
    fn from(context: &Context) -> Self {
        GghhContext {
            context_id: context.id.to_string(),
            actor: GghhActor {
                actor_type: match context.actor.actor_type {
                    crate::entities::ActorType::Persoon => "persoon",
                    crate::entities::ActorType::Systeem => "systeem",
                    crate::entities::ActorType::Service => "service",
                },
                actor_id: context.actor.actor_id.clone(),
                naam: context.actor.display_name.clone(),
            },
            geldigheid: GghgGeldigheid {
                geldig_vanaf: context.geldigheid.begindatum,
                geldig_tot: context.geldigheid.einddatum,
                status: context.geldigheid.status.to_string(),
            },
            domein: Some(format!("{:?}", context.domain.primair_domein)),
            context_type: "context".to_string(),  // Simplified
            waarde: serde_json::to_value(context).unwrap_or_default(),
            kwaliteit: Some(GghhKwaliteit {
                volledigheid: context.quality.volledigheid,
                nauwkeurigheid: context.quality.nauwkeurigheid,
            }),
        }
    }
}

/// Metadata record from registry with context enrichment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichedInformatieobject {
    /// GGHH Informatieobject ID
    pub informatieobject_id: Uuid,

    /// Core metadata from registry
    pub base_metadata: InformatieobjectMetadata,

    /// Enriching context layers
    pub context_layers: Vec<ContextLayerReference>,

    /// Combined quality score (metadata + context)
    pub combined_quality: QualityScore,

    /// Ready for archive transfer (Archiefwet 2025)
    pub archive_ready: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InformatieobjectMetadata {
    pub titel: String,
    pub beschrijving: Option<String>,
    pub creatie_datum: DateTime<Utc>,
    pub auteur: Option<String>,
    pub document_type: String,
    pub woo_relevant: bool,
    pub beveiligingsniveau: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextLayerReference {
    pub layer_id: Uuid,
    pub layer_type: String,  // "core", "domain", "semantic", "provenance"
    pub layer_name: String,
    pub confidence: f64,
}

/// Archive package for 10-year transfer (Archiefwet 2025)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchivePackage {
    /// Package identifier
    pub package_id: Uuid,

    /// Information objects in this package
    pub informatieobject_ids: Vec<Uuid>,

    /// Complete GGHH metadata
    pub gghh_metadata: Vec<GghhContext>,

    /// Context enrichment data
    pub context_data: Vec<Context>,

    /// Quality assessment
    pub quality_assessment: ArchiveQualityAssessment,

    /// Transfer date
    pub transfer_date: DateTime<Utc>,

    /// Target archive service
    pub target_archive: ArchiveService,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchiveQualityAssessment {
    pub metadata_completeness: f64,
    pub context_completeness: f64,
    pub overall_readiness: bool,
    pub gaps: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ArchiveService {
    NationaalArchief,
    RegionaalArchief(String),  // naam
    GemeentelijkArchief(String),  // naam
}
