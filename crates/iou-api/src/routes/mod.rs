//! API route handlers

pub mod apps;
pub mod auth;
pub mod compliance;
pub mod context;
pub mod documents;
pub mod graphrag;
pub mod health;
pub mod objects;
pub mod search;
pub mod templates;

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
