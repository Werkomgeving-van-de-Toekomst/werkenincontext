//! IOU-Core: Domain models voor Informatie Ondersteunde Werkomgeving
//!
//! Dit crate bevat de gedeelde domeinmodellen voor het IOU-Modern systeem,
//! een informatiemanagement platform voor Nederlandse overheidsorganisaties.
//!
//! # Modules
//!
//! - [`domain`]: Informatiedomeinen (zaak, project, beleid, expertise)
//! - [`objects`]: Informatieobjecten (documenten, emails, besluiten)
//! - [`compliance`]: Woo, AVG en Archiefwet compliance types
//! - [`organization`]: Organisatie, afdelingen, gebruikers
//! - [`graphrag`]: Knowledge graph entiteiten en relaties
//! - [`api_types`]: API request/response types

pub mod domain;
pub mod objects;
pub mod compliance;
pub mod organization;
pub mod graphrag;
pub mod api_types;
pub mod workflows;

// Re-exports voor gemakkelijk gebruik
pub use domain::{DomainType, InformationDomain, Case, Project, PolicyTopic};
pub use objects::{ObjectType, InformationObject};
pub use compliance::{Classification, WooMetadata, AvgMetadata, RetentionPolicy};
pub use organization::{Organization, Department, User, Role};
pub use workflows::{WorkflowStatus, WorkflowDefinition, WorkflowExecution};
