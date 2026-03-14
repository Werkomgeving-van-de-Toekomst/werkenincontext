//! Supabase Authentication Integration
//!
//! This module provides JWT verification for Supabase Auth tokens
//! and integrates with the existing authentication middleware.

mod supabase_jwt;

pub use supabase_jwt::{
    SupabaseClaims,
    SupabaseJwtVerifier,
    SupabaseAuthError,
};

// Re-export existing auth types for convenience
pub use crate::middleware::{
    AuthContext,
    Role,
};
