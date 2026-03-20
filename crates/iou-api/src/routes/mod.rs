//! API route handlers

pub mod apps;
pub mod auth;
pub mod buildings_3d;

// Re-export buildings handlers
pub use buildings_3d::{get_buildings_3d, get_buildings_3d_cached};
pub mod compliance;
pub mod context;
pub mod documents;
pub mod graphrag;
pub mod health;
pub mod objects;
pub mod search;
pub mod stakeholder;
pub mod templates;
pub mod terrain;

// Versioned API (RONL Business API Layer)
pub mod v1;

// Re-export document and template handlers for use in main.rs
pub use documents::create_document as documents_create;
pub use documents::get_status as documents_get_status;
pub use documents::approve_document as documents_approve;
pub use documents::get_audit_trail as documents_audit;
pub use documents::download_document as documents_download;
pub use templates::list_templates;
pub use templates::create_template;
pub use templates::get_template;
pub use templates::update_template;
pub use templates::delete_template;

// Re-export stakeholder handlers
pub use stakeholder::{
    get_stakeholder,
    get_stakeholder_documents,
    get_document_stakeholders,
    search_stakeholders,
};
