//! Delegation system for approval authority transfer
//!
//! Provides types and services for temporary, permanent, and bulk delegations
//! between users, along with resolution logic for determining actual approvers.

// Types are always available (pure data structures)
pub mod types;

// Service and resolver require database (server-only)
#[cfg(feature = "server")]
pub mod service;

#[cfg(feature = "server")]
pub mod resolver;

// Re-export all public types
pub use types::{Delegation, DelegationType, ResolvedApprover};

#[cfg(feature = "server")]
pub use service::{DelegationService, DelegationError};

#[cfg(feature = "server")]
pub use resolver::{DelegationResolver, ResolutionError};
