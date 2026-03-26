//! IOU-Core: Domain models voor Informatie Ondersteunde Werkomgeving
//!
//! Dit crate bevat de gedeelde domeinmodellen voor het IOU-Modern systeem,
//! een informatiemanagement platform voor Nederlandse overheidsorganisaties.
//!
//! # Modules
//!
//! ## Always available (WASM-compatible)
//! - [`domain`]: Informatiedomeinen (zaak, project, beleid, expertise)
//! - [`objects`]: Informatieobjecten (documenten, emails, besluiten)
//! - [`compliance`]: Woo, AVG en Archiefwet compliance types
//! - [`organization`]: Organisatie, afdelingen, gebruikers
//! - [`graphrag`]: Knowledge graph types (entities, relationships)
//! - [`workflows`]: Workflow status en definitie types
//! - [`delegation`]: Delegatie types
//! - [`config`]: Configuratie types
//! - [`diff`]: Document diff generatie
//! - [`document`]: Document types
//! - [`sla`]: SLA calculatie types
//! - [`tenancy`]: Multi-tenant context types
//!
//! ## Server-only (requires tokio/sqlx/reqwest)
//! - [`audit`]: Audit logging met PostgreSQL backend
//! - [`storage`]: S3/MinIO storage abstraction
//! - [`versions`]: Document versioning met S3 storage
//! - [`ssi`]: SSI/VC support met DID resolution
//! - [`realtime`]: Realtime WebSocket communicatie
//! - [`escalation`]: Escalatie services

// =============================================================================
// Always-available modules (WASM-compatible)
// =============================================================================

pub mod domain;
pub mod objects;
pub mod compliance;
pub mod organization;
pub mod graphrag;
pub mod api_types;
pub mod workflows;
pub mod delegation;
pub mod config;
pub mod diff;
pub mod document;
pub mod sla;
pub mod tenancy;

// =============================================================================
// Server-only modules (require "server" feature)
// =============================================================================

#[cfg(feature = "server")]
pub mod audit;

#[cfg(feature = "server")]
pub mod storage;

#[cfg(feature = "server")]
pub mod versions;

#[cfg(feature = "server")]
pub mod ssi;

#[cfg(feature = "server")]
pub mod realtime;

#[cfg(feature = "server")]
pub mod escalation;

// =============================================================================
// Re-exports - Always available
// =============================================================================

// Core domain types
pub use domain::{DomainType, InformationDomain, Case, Project, PolicyTopic};
pub use objects::{ObjectType, InformationObject};
pub use compliance::{Classification, WooMetadata, AvgMetadata, RetentionPolicy};
pub use organization::{Organization, Department, User, Role};

// GraphRAG types (only the data structures, not the store)
pub use graphrag::types::{
    Entity,
    EntityType,
    Relationship,
    RelationshipType,
    Community,
    DomainRelation,
    DomainRelationType,
    DiscoveryMethod,
    ContextVector,
    GraphAnalysisResult,
};

// API types
pub use api_types::{
    CreateDomainRequest, CreateDomainResponse,
    CreateObjectRequest, CreateObjectResponse,
    SearchRequest, SearchResponse, SearchResult,
    ContextResponse, DomainDetails, RelatedDomainInfo,
    AppRecommendation, StakeholderInfo, MetadataSuggestion,
    ApiError,
};

// Workflow types
pub use workflows::{
    WorkflowStatus, WorkflowDefinition, WorkflowExecution,
    ApprovalStage, Approver, ApprovalType, ExpiryAction,
    StageInstance, StageStatus, ApprovalResponse, ApprovalDecision,
};

// Delegation types
pub use delegation::{Delegation, DelegationType, ResolvedApprover};

// Configuration types
pub use config::{
    WorkflowConfig, StageConfig, ApproverConfig, ApprovalTypeConfig,
    VersionStorageConfig, SlaConfig, DomainConfig as ConfigDomainConfig, ConfigWatcher,
};

// Document types (AuditEntry is defined here for WASM compatibility)
pub use document::{
    DocumentId, DocumentState, TrustLevel,
    DocumentRequest, DocumentMetadata, AgentResult,
    AuditEntry,  // Re-exported from document.rs for convenience
    StorageRef, DocumentVersion, DocumentFormat,
    Template, TemplateVariable, VariableSource, RenderedDocument,
};

// =============================================================================
// Re-exports - Server-only (these don't conflict with always-available types)
// =============================================================================

// SSI types
#[cfg(feature = "server")]
pub use ssi::{
    VerifiableCredential, VerifiablePresentation, VCValidationError,
    Claims, ClaimValue, DIDResolver, DidMethod, DidDocument, DidKey,
    parse_did, PresentationValidator, UniversalDidResolver,
};

// Audit logging (use AuditEntry from document module, other types from audit)
#[cfg(feature = "server")]
pub use audit::{AuditLogger, AuditBackend, PostgresAuditBackend, SharedAuditLogger, shared_logger, log_shared};
#[cfg(feature = "server")]
pub use audit::{AuditAction, AuditOutcome, AuditFilter, AuditQuery};

// Storage
#[cfg(feature = "server")]
pub use storage::{S3Client, S3Config, S3Error};

// Version storage
#[cfg(feature = "server")]
pub use versions::{VersionService, VersionRecord, VersionContent, RestoreResult, VersionError};

// GraphRAG server types
#[cfg(feature = "server")]
pub use graphrag::{
    ArangoConfig,
    StoreError,
    MigrationValidator, SampleComparison, ValidationResult,
    EntityFilters, EntityUpdate, GraphPath, GraphStore,
    Neighbor, NeighborFilters, PaginatedEntities, PaginationOptions,
    RelationshipDirection, RelationshipQueryOptions,
    TraversalDirection, TraversalRequest, TraversalResult,
};

// Delegation services
#[cfg(feature = "server")]
pub use delegation::{DelegationService, DelegationError, DelegationResolver, ResolutionError};

// Escalation (note: ExpiryAction is exported from workflows, not escalation)
#[cfg(feature = "server")]
pub use escalation::{
    EscalationService,
    EscalationType,
    NotificationChannel,
    EscalationMessage,
    EscalationRecord,
    EscalationStatus,
    EscalationThresholds,
    // ExpiryAction is from workflows module
    PendingEscalation,
    StageDeadlineInfo,
    EscalationError,
};
