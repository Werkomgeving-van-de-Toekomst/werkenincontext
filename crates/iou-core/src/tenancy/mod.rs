//! Multi-tenant isolation for municipality data

pub mod tenant;

pub use tenant::{TenantContext, TenantId, LoA, TenantError};
