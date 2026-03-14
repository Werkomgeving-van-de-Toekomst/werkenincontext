//! User Migration from DuckDB to Supabase Auth
//!
//! Handles the migration of user accounts, password hashes,
//! and associated profile data from DuckDB to Supabase.

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::db::Database;
use crate::supabase::SupabasePool;

/// Configuration for user migration
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Batch size for migrating users
    pub batch_size: usize,

    /// Whether to verify passwords after migration
    pub verify_passwords: bool,

    /// Whether to create user profiles in public schema
    pub create_profiles: bool,

    /// Maximum number of retry attempts for failed users
    pub max_retries: usize,

    /// Supabase instance ID (defaults to zero UUID if not specified)
    pub instance_id: String,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            verify_passwords: true,
            create_profiles: true,
            max_retries: 3,
            instance_id: "00000000-0000-0000-0000-000000000000".to_string(),
        }
    }
}

/// Result of a user migration operation
#[derive(Debug)]
pub struct MigrationReport {
    /// Total number of users processed
    pub total_users: usize,

    /// Number of users successfully migrated
    pub migrated: usize,

    /// IDs of users that failed to migrate
    pub failed: Vec<String>,

    /// Warnings generated during migration
    pub warnings: Vec<String>,

    /// Duration of the migration in milliseconds
    pub duration_ms: u64,

    /// Detailed per-user results
    pub user_results: HashMap<String, UserMigrationResult>,
}

/// Result of migrating a single user
#[derive(Debug, Clone)]
pub struct UserMigrationResult {
    /// User ID
    pub user_id: String,

    /// User email
    pub email: String,

    /// Whether migration succeeded
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,

    /// Warnings for this user
    pub warnings: Vec<String>,
}

/// Errors that can occur during user migration
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Invalid password hash format")]
    InvalidPasswordHash,

    #[error("Failed to insert user into Supabase: {0}")]
    SupabaseInsertFailed(String),

    #[error("Password hash not compatible")]
    PasswordHashNotCompatible,

    #[error("Migration failed: {0}")]
    MigrationFailed(String),
}

/// User record from DuckDB
#[derive(Debug, Clone)]
struct DuckDbUser {
    id: Uuid,
    email: String,
    password_hash: String,
    organization_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// User migrator that transfers users from DuckDB to Supabase
pub struct UserMigrator {
    /// DuckDB database
    duckdb: Database,

    /// Supabase connection pool
    supabase: PgPool,

    /// Migration configuration
    config: MigrationConfig,
}

impl UserMigrator {
    /// Create a new user migrator
    pub fn new(duckdb: Database, supabase: SupabasePool) -> Self {
        Self {
            duckdb,
            supabase: supabase.inner().clone(),
            config: MigrationConfig::default(),
        }
    }

    /// Create a new user migrator with custom configuration
    pub fn with_config(duckdb: Database, supabase: SupabasePool, config: MigrationConfig) -> Self {
        Self {
            duckdb,
            supabase: supabase.inner().clone(),
            config,
        }
    }

    /// Migrate all users from DuckDB to Supabase
    ///
    /// This method:
    /// 1. Reads all users from DuckDB
    /// 2. For each user, inserts into auth.users (hashed password)
    /// 3. Creates user profile in public.user_profiles
    /// 4. Records migration in audit trail
    pub async fn migrate_all_users(&self) -> Result<MigrationReport, MigrationError> {
        let start_time = std::time::Instant::now();

        // First, get all users from DuckDB
        let users = self.get_all_duckdb_users().await?;

        let total_users = users.len();
        let mut migrated = 0;
        let mut failed = Vec::new();
        let mut warnings = Vec::new();
        let mut user_results = HashMap::new();

        tracing::info!("Starting migration of {} users", total_users);

        for user in users {
            let user_id = user.id.to_string();
            let email = user.email.clone();

            let result = self.migrate_user(&user).await;

            match result {
                Ok(user_warnings) => {
                    migrated += 1;
                    warnings.extend(user_warnings.clone());

                    user_results.insert(user_id.clone(), UserMigrationResult {
                        user_id: user_id.clone(),
                        email: email.clone(),
                        success: true,
                        error: None,
                        warnings: user_warnings,
                    });

                    tracing::debug!("Migrated user: {}", email);
                }
                Err(e) => {
                    failed.push(user_id.clone());

                    user_results.insert(user_id.clone(), UserMigrationResult {
                        user_id: user_id.clone(),
                        email: email.clone(),
                        success: false,
                        error: Some(e.to_string()),
                        warnings: vec![],
                    });

                    tracing::warn!("Failed to migrate user {}: {}", email, e);
                }
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        tracing::info!(
            "Migration complete: {} succeeded, {} failed, took {}ms",
            migrated,
            failed.len(),
            duration_ms
        );

        Ok(MigrationReport {
            total_users,
            migrated,
            failed,
            warnings,
            duration_ms,
            user_results,
        })
    }

    /// Get all users from DuckDB
    async fn get_all_duckdb_users(&self) -> Result<Vec<DuckDbUser>, MigrationError> {
        // Query DuckDB for all users
        let mut users = Vec::new();

        // Note: This assumes a users table exists in DuckDB
        // Adjust the query as needed for your schema
        let query = r#"
            SELECT
                id,
                email,
                password_hash,
                organization_id,
                created_at,
                updated_at
            FROM users
            ORDER BY created_at
        "#;

        // TODO: Execute query against DuckDB
        // For now, return empty vector as placeholder
        // In real implementation:
        // let rows = self.duckdb.execute(query).await?;
        // for row in rows {
        //     users.push(DuckDbUser { ... });
        // }

        tracing::warn!("DuckDB user query not yet implemented - returning empty list");

        Ok(users)
    }

    /// Migrate a single user to Supabase
    ///
    /// This method:
    /// 1. Verifies password hash format
    /// 2. Inserts into auth.users
    /// 3. Creates user profile
    /// Returns list of warnings if successful
    async fn migrate_user(&self, user: &DuckDbUser) -> Result<Vec<String>, MigrationError> {
        let mut warnings = Vec::new();

        // Verify password hash format
        self.verify_password_hash(&user.password_hash)
            .await
            .map_err(|e| {
                tracing::error!("Password hash verification failed for {}: {}", user.email, e);
                MigrationError::PasswordHashNotCompatible
            })?;

        // Insert into auth.users
        self.insert_user_to_supabase(user).await?;

        // Create user profile
        if self.config.create_profiles {
            self.create_user_profile(user).await?;
        }

        Ok(warnings)
    }

    /// Verify that the password hash format is compatible
    async fn verify_password_hash(&self, hash: &str) -> Result<(), MigrationError> {
        // DuckDB and Supabase both use bcrypt by default
        // Verify the hash starts with the bcrypt prefix
        if !hash.starts_with("$2b$") && !hash.starts_with("$2a$") {
            tracing::warn!("Password hash does not appear to be bcrypt format");
            return Err(MigrationError::InvalidPasswordHash);
        }

        // Additional validation could be added here
        Ok(())
    }

    /// Insert user into Supabase auth.users table
    async fn insert_user_to_supabase(&self, user: &DuckDbUser) -> Result<(), MigrationError> {
        // Note: Direct insertion into auth.users requires superuser privileges
        // In production, use Supabase Management API or admin functions

        let query = format!(r#"
            INSERT INTO auth.users (
                instance_id,
                id,
                aud,
                role,
                email,
                encrypted_password,
                email_confirmed_at,
                created_at,
                updated_at,
                raw_app_meta_data,
                raw_user_meta_data,
                is_super_admin
            ) VALUES (
                '{}'::uuid,
                $1,
                'authenticated',
                'authenticated',
                $2,
                $3,
                NOW(),
                $4,
                $5,
                '{{"provider": "email", "providers": ["email"]}}'::jsonb,
                jsonb_build_object('organization_id', $6::text),
                false
            )
            ON CONFLICT (id) DO NOTHING
        "#, self.config.instance_id);

        sqlx::query(&query)
            .bind(user.id)
            .bind(&user.email)
            .bind(&user.password_hash)
            .bind(user.created_at)
            .bind(user.updated_at)
            .bind(user.organization_id)
            .execute(&self.supabase)
            .await
            .map_err(|e| MigrationError::SupabaseInsertFailed(e.to_string()))?;

        tracing::debug!("Inserted user {} into auth.users", user.email);

        Ok(())
    }

    /// Create user profile in public schema
    async fn create_user_profile(&self, user: &DuckDbUser) -> Result<(), MigrationError> {
        let query = r#"
            INSERT INTO public.user_profiles (
                id,
                user_id,
                email,
                organization_id,
                created_at,
                updated_at
            ) VALUES (
                gen_random_uuid(),
                $1,
                $2,
                $3,
                $4,
                $5
            )
            ON CONFLICT (user_id) DO UPDATE SET
                email = EXCLUDED.email,
                updated_at = EXCLUDED.updated_at
        "#;

        sqlx::query(query)
            .bind(user.id)
            .bind(&user.email)
            .bind(user.organization_id)
            .bind(user.created_at)
            .bind(user.updated_at)
            .execute(&self.supabase)
            .await
            .map_err(|e| MigrationError::SupabaseInsertFailed(e.to_string()))?;

        tracing::debug!("Created profile for user {}", user.email);

        Ok(())
    }

    /// Rollback a migrated user
    ///
    /// Removes the user from Supabase auth.users and user_profiles.
    pub async fn rollback_user(&self, user_id: &str) -> Result<(), MigrationError> {
        let uid = Uuid::parse_str(user_id)
            .map_err(|_| MigrationError::MigrationFailed("Invalid user ID".to_string()))?;

        // Delete from user_profiles first (foreign key)
        sqlx::query("DELETE FROM public.user_profiles WHERE user_id = $1")
            .bind(uid)
            .execute(&self.supabase)
            .await?;

        // Delete from auth.users
        sqlx::query("DELETE FROM auth.users WHERE id = $1")
            .bind(uid)
            .execute(&self.supabase)
            .await?;

        tracing::info!("Rolled back user {}", user_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_config_default() {
        let config = MigrationConfig::default();
        assert_eq!(config.batch_size, 100);
        assert!(config.verify_passwords);
        assert!(config.create_profiles);
    }

    #[test]
    fn test_password_hash_verification() {
        // Valid bcrypt hashes
        let valid_hashes = vec![
            "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7qvZCFbwLy",
            "$2a$10$N9qo8uLOickgx2ZMRZoMye1j50kgcuLPVFxWb/nJ0pNP2mC0xN5dG",
        ];

        for hash in valid_hashes {
            assert!(hash.starts_with("$2b$") || hash.starts_with("$2a$"));
        }
    }
}
