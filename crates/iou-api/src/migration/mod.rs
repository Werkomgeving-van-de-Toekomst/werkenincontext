//! User Data Migration Module
//!
//! Handles migration of user data from DuckDB to Supabase Auth.

mod user_migration;

pub use user_migration::{
    UserMigrator,
    MigrationReport,
    MigrationError,
    MigrationConfig,
};
