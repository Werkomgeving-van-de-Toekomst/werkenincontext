// =============================================================================
// Context Entities - Aligned with Metamodel GGHH Overheid V2
// =============================================================================

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use garde::Validate;

use crate::{ContextId, ContextRecordId, Timestamp, OrganisationId};

/// Context: The complete contextual metadata for an information object
///
/// Per Metamodel GGHH V2, Context captures:
/// - Who created/used the information (actor)
/// - When it was created/modified (temporal)
/// - Where it fits in the organization (domain)
/// - Why it exists (purpose/grondslag)
/// - What it relates to (relationships)
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Context {
    /// Unique identifier for this context
    #[garde(skip)]
    pub id: ContextId,

    /// Information object this context describes
    #[garde(skip)]
    pub informatieobject_id: Uuid,

    /// Organization that owns this context
    #[garde(skip)]
    pub organisatie_id: OrganisationId,

    /// Actor who created/last modified the context
    #[garde(skip)]
    pub actor: Actor,

    /// Temporal context (when)
    #[garde(skip)]
    pub temporal: TemporalContext,

    /// Domain context (where in organization)
    #[garde(skip)]
    pub domain: DomainContext,

    /// Purpose context (why - grondslag/beleid)
    #[garde(skip)]
    pub purpose: PurposeContext,

    /// Semantic context (meaning/keywords)
    #[garde(skip)]
    pub semantic: SemanticContext,

    /// Provenance context (origin/lineage)
    #[garde(skip)]
    pub provenance: ProvenanceContext,

    /// Quality metrics
    #[garde(skip)]
    pub quality: QualityMetrics,

    /// Validity period
    #[garde(skip)]
    pub geldigheid: Geldigheid,

    /// Metadata
    #[garde(skip)]
    pub metadata: ContextMetadata,
}

/// Actor: Who created or modified the context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Actor {
    /// Actor type (person, system, service)
    pub actor_type: ActorType,

    /// Actor identifier (employee ID, system ID, etc.)
    pub actor_id: String,

    /// Actor display name
    pub display_name: String,

    /// Actor's organizational unit
    pub organisatie_eenheid: Option<String>,

    /// Actor's role (function)
    pub rol: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, strum::Display)]
pub enum ActorType {
    /// Natural person (medewerker)
    #[strum(serialize = "persoon")]
    Persoon,

    /// System/application
    #[strum(serialize = "systeem")]
    Systeem,

    /// Service/API
    #[strum(serialize = "service")]
    Service,
}

/// Temporal context: When the information was created/modified
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalContext {
    /// Creation timestamp
    pub aangemaakt_op: Timestamp,

    /// Last modified timestamp
    pub gewijzigd_op: Timestamp,

    /// Valid from (optional - for future-dated content)
    pub geldig_vanaf: Option<DateTime<Utc>>,

    /// Valid until (optional - for expired content)
    pub geldig_tot: Option<DateTime<Utc>>,

    /// Reference time for the context (e.g., "as of 2025-01-01")
    pub referentie_tijd: Option<DateTime<Utc>>,
}

/// Domain context: Where in the organization hierarchy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainContext {
    /// Primary domain (Zaak, Project, Beleid, Expertise)
    pub primair_domein: Domein,

    /// Associated domains (cross-cutting)
    pub gerelateerde_domeinen: Vec<Domein>,

    /// Organizational unit path
    pub organisatie_pad: Vec<String>,

    /// Tags for categorization
    pub labels: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, strum::Display)]
pub enum Domein {
    /// Case/Work item (Zaak)
    #[strum(serialize = "zaak")]
    Zaak(ZaakContext),

    /// Project
    #[strum(serialize = "project")]
    Project(ProjectContext),

    /// Policy (Beleid)
    #[strum(serialize = "beleid")]
    Beleid(BeleidContext),

    /// Expertise area
    #[strum(serialize = "expertise")]
    Expertise(ExpertiseContext),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZaakContext {
    pub zaak_id: String,
    pub zaak_type: String,
    pub zaak_fase: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectContext {
    pub project_id: String,
    pub project_naam: String,
    pub project_fase: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeleidContext {
    pub beleid_id: String,
    pub beleidsgebied: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpertiseContext {
    pub expertise_id: String,
    pub expertise_domein: String,
}

/// Purpose context: Why the information exists (legal basis, policy)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PurposeContext {
    /// Legal basis (grondslag) if applicable
    pub grondslagen: Vec<Grondslag>,

    /// Policy references
    pub beleidsreferenties: Vec<Beleidsreferentie>,

    /// Business purpose description
    pub zakelijk_doel: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grondslag {
    /// Grundslag reference (e.g., from wetten)
    pub grondslag_id: String,

    /// Source law/regulation
    pub bron: String,

    /// Article reference
    pub artikel: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Beleidsreferentie {
    pub beleid_id: String,
    pub referentie_type: String,
}

/// Semantic context: Meaning and keywords
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticContext {
    /// Primary keywords
    pub trefwoorden: Vec<String>,

    /// Subject categories (onderwerpen)
    pub onderwerpen: Vec<String>,

    /// Entity mentions (personen, organisaties, locaties)
    pub entiteiten: Vec<Entiteit>,

    /// Summary/description
    pub samenvatting: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entiteit {
    pub entiteit_type: EntiteitType,
    pub naam: String,
    pub identificatie: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, strum::Display)]
pub enum EntiteitType {
    #[strum(serialize = "persoon")]
    Persoon,

    #[strum(serialize = "organisatie")]
    Organisatie,

    #[strum(serialize = "locatie")]
    Locatie,

    #[strum(serialize = "evenement")]
    Evenement,
}

/// Provenance context: Origin and lineage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvenanceContext {
    /// Source system where information originated
    pub bronsysteem: String,

    /// Source reference (ID in source system)
    pub bron_referentie: String,

    /// Data lineage (transformation history)
    pub herkomst: Vec<HerkomstRecord>,

    /// Trust score for source
    pub bron_betrouwbaarheid: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HerkomstRecord {
    pub stap: u32,
    pub systeem: String,
    pub actie: String,
    pub tijdstip: Timestamp,
}

/// Quality metrics for context data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Completeness score (0.0 - 1.0)
    pub volledigheid: f64,

    /// Accuracy score (0.0 - 1.0)
    pub nauwkeurigheid: f64,

    /// Consistency score (0.0 - 1.0)
    pub consistentie: f64,

    /// Timeliness score (0.0 - 1.0)
    pub actualiteit: f64,

    /// Overall quality score (aggregated)
    pub kwaliteit: f64,
}

/// Validity period (Geldigheid) per Archiefwet
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Geldigheid {
    /// Status
    pub status: GeldigheidStatus,

    /// Start date
    pub begindatum: Option<DateTime<Utc>>,

    /// End date
    pub einddatum: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, strum::Display)]
pub enum GeldigheidStatus {
    #[strum(serialize = "actief")]
    Actief,

    #[strum(serialize = "inactief")]
    Inactief,

    #[strum(serialize = "archief")]
    Archief,

    #[strum(serialize = "vervallen")]
    Vervallen,
}

/// Context metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// Created by
    pub aangemaakt_door: String,

    /// Created at
    pub aangemaakt_op: Timestamp,

    /// Last modified by
    pub gewijzigd_door: Option<String>,

    /// Last modified at
    pub gewijzigd_op: Option<Timestamp>,
}

/// Context Record: Versioned context entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextRecord {
    pub id: ContextRecordId,
    pub context_id: ContextId,
    pub versie: u32,
    pub context: Context,
    pub wijzigingsreden: Option<String>,
}
