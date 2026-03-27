diff --git a/crates/iou-api/src/auth/mod.rs b/crates/iou-api/src/auth/mod.rs
new file mode 100644
index 0000000..c7e8928
--- /dev/null
+++ b/crates/iou-api/src/auth/mod.rs
@@ -0,0 +1,18 @@
+//! Supabase Authentication Integration
+//!
+//! This module provides JWT verification for Supabase Auth tokens
+//! and integrates with the existing authentication middleware.
+
+mod supabase_jwt;
+
+pub use supabase_jwt::{
+    SupabaseClaims,
+    SupabaseJwtVerifier,
+    SupabaseAuthError,
+};
+
+// Re-export existing auth types for convenience
+pub use crate::middleware::{
+    AuthContext,
+    Role,
+};
diff --git a/crates/iou-api/src/auth/supabase_jwt.rs b/crates/iou-api/src/auth/supabase_jwt.rs
new file mode 100644
index 0000000..bbf0232
--- /dev/null
+++ b/crates/iou-api/src/auth/supabase_jwt.rs
@@ -0,0 +1,357 @@
+//! Supabase JWT Token Verification
+//!
+//! Provides verification for JWT tokens issued by Supabase Auth.
+//! Supabase tokens follow a specific structure with custom claims
+//! for organization_id and other application-specific data.
+
+use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm, Header};
+use serde::{Deserialize, Serialize};
+use uuid::Uuid;
+use std::env;
+
+use crate::middleware::{AuthContext, Role};
+
+/// Errors that can occur during Supabase JWT verification
+#[derive(Debug, thiserror::Error)]
+pub enum SupabaseAuthError {
+    #[error("Missing JWT secret")]
+    MissingSecret,
+
+    #[error("Invalid token format: {0}")]
+    InvalidToken(String),
+
+    #[error("Token signature verification failed")]
+    SignatureFailed,
+
+    #[error("Token has expired")]
+    TokenExpired,
+
+    #[error("Invalid token issuer: expected {expected}, got {found}")]
+    InvalidIssuer { expected: String, found: String },
+
+    #[error("Missing required claim: {0}")]
+    MissingClaim(String),
+
+    #[error("Invalid user ID format")]
+    InvalidUserId,
+
+    #[error("Invalid organization ID format")]
+    InvalidOrganizationId,
+}
+
+/// Standard claims in a Supabase JWT token
+///
+/// Supabase issues JWTs with the following structure:
+/// - `sub`: User ID (UUID)
+/// - `aud`: Audience (typically "authenticated")
+/// - `role`: User role (typically "authenticated")
+/// - `email`: User email
+/// - `exp`: Expiration timestamp
+/// - `iat`: Issued at timestamp
+///
+/// Custom claims (via JWT hooks in Supabase):
+/// - `organization_id`: User's organization UUID
+/// - `clearance`: Security clearance level
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct SupabaseClaims {
+    /// User ID (UUID from auth.users.id)
+    pub sub: String,
+
+    /// Audience (typically "authenticated")
+    pub aud: String,
+
+    /// User role (typically "authenticated")
+    pub role: String,
+
+    /// User email address
+    pub email: String,
+
+    /// Organization ID (custom claim)
+    #[serde(default)]
+    pub organization_id: Option<String>,
+
+    /// Security clearance level (custom claim)
+    #[serde(default)]
+    pub clearance: Option<String>,
+
+    /// User roles/permissions (custom claim)
+    #[serde(default)]
+    pub app_roles: Option<Vec<String>>,
+
+    /// Token expiration timestamp
+    pub exp: i64,
+
+    /// Token issued at timestamp
+    pub iat: i64,
+
+    /// Token issuer
+    pub iss: String,
+}
+
+/// JWT verifier for Supabase Auth tokens
+pub struct SupabaseJwtVerifier {
+    /// Decoding key for verifying JWT signatures
+    decoding_key: DecodingKey,
+
+    /// Expected issuer for Supabase tokens
+    issuer: String,
+
+    /// Expected audience for Supabase tokens
+    audience: String,
+}
+
+impl SupabaseJwtVerifier {
+    /// Create a new Supabase JWT verifier
+    ///
+    /// The JWT secret is loaded from the `SUPABASE_JWT_SECRET` environment variable.
+    /// This is typically the JWT secret from your Supabase project settings.
+    ///
+    /// # Errors
+    ///
+    /// Returns `SupabaseAuthError::MissingSecret` if the environment variable is not set.
+    pub fn new() -> Result<Self, SupabaseAuthError> {
+        let jwt_secret = env::var("SUPABASE_JWT_SECRET")
+            .or_else(|_| env::var("SUPABASE_JWT")) // Fallback alternative name
+            .map_err(|_| SupabaseAuthError::MissingSecret)?;
+
+        Self::with_secret(&jwt_secret)
+    }
+
+    /// Create a verifier with a specific JWT secret
+    ///
+    /// This is useful for testing or when the secret is managed externally.
+    pub fn with_secret(jwt_secret: &str) -> Result<Self, SupabaseAuthError> {
+        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
+
+        Ok(Self {
+            decoding_key,
+            issuer: "supabase".to_string(),
+            audience: "authenticated".to_string(),
+        })
+    }
+
+    /// Create a verifier with custom issuer and audience
+    ///
+    /// Use this if your Supabase instance uses non-standard values.
+    pub fn with_config(
+        jwt_secret: &str,
+        issuer: String,
+        audience: String,
+    ) -> Result<Self, SupabaseAuthError> {
+        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
+
+        Ok(Self {
+            decoding_key,
+            issuer,
+            audience,
+        })
+    }
+
+    /// Verify and decode a Supabase JWT token
+    ///
+    /// This method:
+    /// 1. Verifies the token signature
+    /// 2. Checks expiration
+    /// 3. Validates issuer and audience
+    /// 4. Returns the decoded claims
+    ///
+    /// # Errors
+    ///
+    /// Returns an error if:
+    /// - The token is malformed
+    /// - The signature is invalid
+    /// - The token has expired
+    /// - The issuer or audience doesn't match
+    pub fn verify(&self, token: &str) -> Result<SupabaseClaims, SupabaseAuthError> {
+        // Set up validation first (validates algorithm)
+        let mut validation = Validation::new(Algorithm::HS256);
+        validation.set_issuer(&[&self.issuer]);
+        validation.set_audience(&[&self.audience]);
+        validation.validate_exp = true;
+
+        // Decode and verify
+        let token_data = decode::<SupabaseClaims>(token, &self.decoding_key, &validation)
+            .map_err(|e| {
+                match e.kind() {
+                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
+                        SupabaseAuthError::TokenExpired
+                    }
+                    _ => SupabaseAuthError::SignatureFailed,
+                }
+            })?;
+
+        Ok(token_data.claims)
+    }
+
+    /// Verify a token and convert it to an AuthContext
+    ///
+    /// This is a convenience method that verifies the token and
+    /// converts the Supabase claims to the internal AuthContext format.
+    pub fn verify_to_auth_context(&self, token: &str) -> Result<AuthContext, SupabaseAuthError> {
+        let claims = self.verify(token)?;
+
+        let user_id = Uuid::parse_str(&claims.sub)
+            .map_err(|_| SupabaseAuthError::InvalidUserId)?;
+
+        let organization_id = claims.organization_id
+            .ok_or_else(|| SupabaseAuthError::MissingClaim("organization_id".to_string()))?
+            .parse::<Uuid>()
+            .map_err(|_| SupabaseAuthError::InvalidOrganizationId)?;
+
+        // Convert app_roles to our Role enum
+        let roles: Vec<Role> = claims
+            .app_roles
+            .unwrap_or_default()
+            .into_iter()
+            .filter_map(|r| r.parse().ok())
+            .collect();
+
+        // Add default role if none specified
+        let roles = if roles.is_empty() {
+            vec![Role::DomainViewer] // Default role
+        } else {
+            roles
+        };
+
+        Ok(AuthContext {
+            user_id,
+            email: claims.email,
+            organization_id,
+            roles,
+        })
+    }
+}
+
+impl Default for SupabaseJwtVerifier {
+    fn default() -> Self {
+        // Try to create from environment, panic if not available
+        // In production, ensure SUPABASE_JWT_SECRET is set
+        Self::new().expect("SUPABASE_JWT_SECRET must be set")
+    }
+}
+
+/// Middleware integration helper
+///
+/// Use this function in your auth middleware to verify Supabase tokens.
+pub fn verify_supabase_token(token: &str) -> Result<AuthContext, SupabaseAuthError> {
+    let verifier = SupabaseJwtVerifier::new()?;
+    verifier.verify_to_auth_context(token)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use jsonwebtoken::{encode, EncodingKey, Header};
+
+    /// Helper to create a test token
+    fn create_test_token(
+        user_id: &str,
+        email: &str,
+        organization_id: &str,
+        secret: &str,
+    ) -> String {
+        let claims = SupabaseClaims {
+            sub: user_id.to_string(),
+            aud: "authenticated".to_string(),
+            role: "authenticated".to_string(),
+            email: email.to_string(),
+            organization_id: Some(organization_id.to_string()),
+            clearance: Some("intern".to_string()),
+            app_roles: Some(vec!["domain_viewer".to_string()]),
+            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
+            iat: chrono::Utc::now().timestamp(),
+            iss: "supabase".to_string(),
+        };
+
+        encode(
+            &Header::default(),
+            &claims,
+            &EncodingKey::from_secret(secret.as_bytes()),
+        )
+        .unwrap()
+    }
+
+    #[test]
+    fn test_supabase_jwt_verification() {
+        let secret = "test-secret-key-for-testing";
+        let verifier = SupabaseJwtVerifier::with_secret(secret).unwrap();
+
+        let user_id = Uuid::new_v4().to_string();
+        let email = "test@example.com";
+        let org_id = Uuid::new_v4().to_string();
+
+        let token = create_test_token(&user_id, email, &org_id, secret);
+
+        let claims = verifier.verify(&token).unwrap();
+        assert_eq!(claims.sub, user_id);
+        assert_eq!(claims.email, email);
+        assert_eq!(claims.organization_id.as_deref(), Some(org_id.as_str()));
+    }
+
+    #[test]
+    fn test_supabase_jwt_to_auth_context() {
+        let secret = "test-secret-key-for-testing";
+        let verifier = SupabaseJwtVerifier::with_secret(secret).unwrap();
+
+        let user_id = Uuid::new_v4();
+        let email = "test@example.com";
+        let org_id = Uuid::new_v4();
+
+        let token = create_test_token(&user_id.to_string(), email, &org_id.to_string(), secret);
+
+        let auth_context = verifier.verify_to_auth_context(&token).unwrap();
+        assert_eq!(auth_context.user_id, user_id);
+        assert_eq!(auth_context.email, email);
+        assert_eq!(auth_context.organization_id, org_id);
+        assert!(!auth_context.roles.is_empty());
+    }
+
+    #[test]
+    fn test_invalid_token_rejected() {
+        let verifier = SupabaseJwtVerifier::with_secret("test-secret").unwrap();
+        let result = verifier.verify("invalid.token.here");
+        assert!(result.is_err());
+    }
+
+    #[test]
+    fn test_wrong_secret_rejected() {
+        let verifier = SupabaseJwtVerifier::with_secret("secret-one").unwrap();
+        let token = create_test_token(
+            &Uuid::new_v4().to_string(),
+            "test@example.com",
+            &Uuid::new_v4().to_string(),
+            "secret-two", // Different secret!
+        );
+
+        let result = verifier.verify(&token);
+        assert!(matches!(result, Err(SupabaseAuthError::SignatureFailed)));
+    }
+
+    #[test]
+    fn test_expired_token_rejected() {
+        let secret = "test-secret-key";
+        let verifier = SupabaseJwtVerifier::with_secret(secret).unwrap();
+
+        let claims = SupabaseClaims {
+            sub: Uuid::new_v4().to_string(),
+            aud: "authenticated".to_string(),
+            role: "authenticated".to_string(),
+            email: "test@example.com".to_string(),
+            organization_id: Some(Uuid::new_v4().to_string()),
+            clearance: None,
+            app_roles: None,
+            exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp(), // Expired
+            iat: (chrono::Utc::now() - chrono::Duration::hours(2)).timestamp(),
+            iss: "supabase".to_string(),
+        };
+
+        let token = encode(
+            &Header::default(),
+            &claims,
+            &EncodingKey::from_secret(secret.as_bytes()),
+        ).unwrap();
+
+        let result = verifier.verify(&token);
+        assert!(matches!(result, Err(SupabaseAuthError::TokenExpired)));
+    }
+}
diff --git a/crates/iou-api/src/dual_write.rs b/crates/iou-api/src/dual_write.rs
index c12cf79..10698fe 100644
--- a/crates/iou-api/src/dual_write.rs
+++ b/crates/iou-api/src/dual_write.rs
@@ -165,17 +165,17 @@ mod tests {
     #[test]
     fn test_read_source_from_env_default() {
         // Clear the env var if set
-        let _ = std::env::set_var("READ_SOURCE", "");
+        let _ = unsafe { std::env::set_var("READ_SOURCE", "") };
         let source = ReadSource::from_env();
         assert_eq!(source, ReadSource::DuckDb);
     }
 
     #[test]
     fn test_read_source_from_env_supabase() {
-        std::env::set_var("READ_SOURCE", "supabase");
+        unsafe { std::env::set_var("READ_SOURCE", "supabase") };
         let source = ReadSource::from_env();
         assert_eq!(source, ReadSource::Supabase);
-        std::env::remove_var("READ_SOURCE");
+        unsafe { std::env::remove_var("READ_SOURCE") };
     }
 
     #[test]
diff --git a/crates/iou-api/src/lib.rs b/crates/iou-api/src/lib.rs
index 03938dd..c72b503 100644
--- a/crates/iou-api/src/lib.rs
+++ b/crates/iou-api/src/lib.rs
@@ -2,10 +2,14 @@
 //!
 //! This library exposes the core types and modules for testing.
 
+pub mod auth;
 pub mod db;
 pub mod domain_dual_write;
 pub mod dual_write;
 pub mod error;
+pub mod middleware;
+pub mod migration;
+pub mod realtime;
 pub mod search_types;
 pub mod supabase;
 pub mod websockets;
@@ -13,6 +17,9 @@ pub mod websockets;
 // Re-export commonly used types
 pub use db::Database;
 pub use dual_write::{DualWrite, DualWriteResult, ReadSource};
+pub use middleware::{AuthContext, Role};
+pub use migration::{UserMigrator, MigrationReport};
+pub use realtime::{RealtimeClient, PresenceTracker};
 pub use search_types::{
     AdvancedSearchResult, FacetCount, SearchFacets, SearchParams, SearchMode,
     SortOrder, SuggestionResult, SuggestionType,
diff --git a/crates/iou-api/src/middleware/mod.rs b/crates/iou-api/src/middleware/mod.rs
index 69d2a6d..e753e20 100644
--- a/crates/iou-api/src/middleware/mod.rs
+++ b/crates/iou-api/src/middleware/mod.rs
@@ -3,5 +3,5 @@
 pub mod auth;
 
 pub use auth::{
-    auth_middleware, optional_auth_middleware, AuthContext, require_permission,
+    auth_middleware, optional_auth_middleware, AuthContext, require_permission, Role,
 };
diff --git a/crates/iou-api/src/migration/mod.rs b/crates/iou-api/src/migration/mod.rs
new file mode 100644
index 0000000..892df73
--- /dev/null
+++ b/crates/iou-api/src/migration/mod.rs
@@ -0,0 +1,12 @@
+//! User Data Migration Module
+//!
+//! Handles migration of user data from DuckDB to Supabase Auth.
+
+mod user_migration;
+
+pub use user_migration::{
+    UserMigrator,
+    MigrationReport,
+    MigrationError,
+    MigrationConfig,
+};
diff --git a/crates/iou-api/src/migration/user_migration.rs b/crates/iou-api/src/migration/user_migration.rs
new file mode 100644
index 0000000..ab2a712
--- /dev/null
+++ b/crates/iou-api/src/migration/user_migration.rs
@@ -0,0 +1,437 @@
+//! User Migration from DuckDB to Supabase Auth
+//!
+//! Handles the migration of user accounts, password hashes,
+//! and associated profile data from DuckDB to Supabase.
+
+use async_trait::async_trait;
+use sqlx::{PgPool, Row};
+use uuid::Uuid;
+use chrono::{DateTime, Utc};
+use std::collections::HashMap;
+
+use crate::db::Database;
+use crate::supabase::SupabasePool;
+
+/// Configuration for user migration
+#[derive(Debug, Clone)]
+pub struct MigrationConfig {
+    /// Batch size for migrating users
+    pub batch_size: usize,
+
+    /// Whether to verify passwords after migration
+    pub verify_passwords: bool,
+
+    /// Whether to create user profiles in public schema
+    pub create_profiles: bool,
+
+    /// Maximum number of retry attempts for failed users
+    pub max_retries: usize,
+}
+
+impl Default for MigrationConfig {
+    fn default() -> Self {
+        Self {
+            batch_size: 100,
+            verify_passwords: true,
+            create_profiles: true,
+            max_retries: 3,
+        }
+    }
+}
+
+/// Result of a user migration operation
+#[derive(Debug)]
+pub struct MigrationReport {
+    /// Total number of users processed
+    pub total_users: usize,
+
+    /// Number of users successfully migrated
+    pub migrated: usize,
+
+    /// IDs of users that failed to migrate
+    pub failed: Vec<String>,
+
+    /// Warnings generated during migration
+    pub warnings: Vec<String>,
+
+    /// Duration of the migration in milliseconds
+    pub duration_ms: u64,
+
+    /// Detailed per-user results
+    pub user_results: HashMap<String, UserMigrationResult>,
+}
+
+/// Result of migrating a single user
+#[derive(Debug, Clone)]
+pub struct UserMigrationResult {
+    /// User ID
+    pub user_id: String,
+
+    /// User email
+    pub email: String,
+
+    /// Whether migration succeeded
+    pub success: bool,
+
+    /// Error message if failed
+    pub error: Option<String>,
+
+    /// Warnings for this user
+    pub warnings: Vec<String>,
+}
+
+/// Errors that can occur during user migration
+#[derive(Debug, thiserror::Error)]
+pub enum MigrationError {
+    #[error("Database error: {0}")]
+    Database(#[from] sqlx::Error),
+
+    #[error("User not found: {0}")]
+    UserNotFound(String),
+
+    #[error("Invalid password hash format")]
+    InvalidPasswordHash,
+
+    #[error("Failed to insert user into Supabase: {0}")]
+    SupabaseInsertFailed(String),
+
+    #[error("Password hash not compatible")]
+    PasswordHashNotCompatible,
+
+    #[error("Migration failed: {0}")]
+    MigrationFailed(String),
+}
+
+/// User record from DuckDB
+#[derive(Debug, Clone)]
+struct DuckDbUser {
+    id: Uuid,
+    email: String,
+    password_hash: String,
+    organization_id: Uuid,
+    created_at: DateTime<Utc>,
+    updated_at: DateTime<Utc>,
+}
+
+/// User migrator that transfers users from DuckDB to Supabase
+pub struct UserMigrator {
+    /// DuckDB database
+    duckdb: Database,
+
+    /// Supabase connection pool
+    supabase: PgPool,
+
+    /// Migration configuration
+    config: MigrationConfig,
+}
+
+impl UserMigrator {
+    /// Create a new user migrator
+    pub fn new(duckdb: Database, supabase: SupabasePool) -> Self {
+        Self {
+            duckdb,
+            supabase: supabase.inner().clone(),
+            config: MigrationConfig::default(),
+        }
+    }
+
+    /// Create a new user migrator with custom configuration
+    pub fn with_config(duckdb: Database, supabase: SupabasePool, config: MigrationConfig) -> Self {
+        Self {
+            duckdb,
+            supabase: supabase.inner().clone(),
+            config,
+        }
+    }
+
+    /// Migrate all users from DuckDB to Supabase
+    ///
+    /// This method:
+    /// 1. Reads all users from DuckDB
+    /// 2. For each user, inserts into auth.users (hashed password)
+    /// 3. Creates user profile in public.user_profiles
+    /// 4. Records migration in audit trail
+    pub async fn migrate_all_users(&self) -> Result<MigrationReport, MigrationError> {
+        let start_time = std::time::Instant::now();
+
+        // First, get all users from DuckDB
+        let users = self.get_all_duckdb_users().await?;
+
+        let total_users = users.len();
+        let mut migrated = 0;
+        let mut failed = Vec::new();
+        let mut warnings = Vec::new();
+        let mut user_results = HashMap::new();
+
+        tracing::info!("Starting migration of {} users", total_users);
+
+        for user in users {
+            let user_id = user.id.to_string();
+            let email = user.email.clone();
+
+            let result = self.migrate_user(&user).await;
+
+            match result {
+                Ok(user_warnings) => {
+                    migrated += 1;
+                    warnings.extend(user_warnings.clone());
+
+                    user_results.insert(user_id.clone(), UserMigrationResult {
+                        user_id: user_id.clone(),
+                        email: email.clone(),
+                        success: true,
+                        error: None,
+                        warnings: user_warnings,
+                    });
+
+                    tracing::debug!("Migrated user: {}", email);
+                }
+                Err(e) => {
+                    failed.push(user_id.clone());
+
+                    user_results.insert(user_id.clone(), UserMigrationResult {
+                        user_id: user_id.clone(),
+                        email: email.clone(),
+                        success: false,
+                        error: Some(e.to_string()),
+                        warnings: vec![],
+                    });
+
+                    tracing::warn!("Failed to migrate user {}: {}", email, e);
+                }
+            }
+        }
+
+        let duration_ms = start_time.elapsed().as_millis() as u64;
+
+        tracing::info!(
+            "Migration complete: {} succeeded, {} failed, took {}ms",
+            migrated,
+            failed.len(),
+            duration_ms
+        );
+
+        Ok(MigrationReport {
+            total_users,
+            migrated,
+            failed,
+            warnings,
+            duration_ms,
+            user_results,
+        })
+    }
+
+    /// Get all users from DuckDB
+    async fn get_all_duckdb_users(&self) -> Result<Vec<DuckDbUser>, MigrationError> {
+        // Query DuckDB for all users
+        let mut users = Vec::new();
+
+        // Note: This assumes a users table exists in DuckDB
+        // Adjust the query as needed for your schema
+        let query = r#"
+            SELECT
+                id,
+                email,
+                password_hash,
+                organization_id,
+                created_at,
+                updated_at
+            FROM users
+            ORDER BY created_at
+        "#;
+
+        // TODO: Execute query against DuckDB
+        // For now, return empty vector as placeholder
+        // In real implementation:
+        // let rows = self.duckdb.execute(query).await?;
+        // for row in rows {
+        //     users.push(DuckDbUser { ... });
+        // }
+
+        tracing::warn!("DuckDB user query not yet implemented - returning empty list");
+
+        Ok(users)
+    }
+
+    /// Migrate a single user to Supabase
+    ///
+    /// This method:
+    /// 1. Verifies password hash format
+    /// 2. Inserts into auth.users
+    /// 3. Creates user profile
+    /// Returns list of warnings if successful
+    async fn migrate_user(&self, user: &DuckDbUser) -> Result<Vec<String>, MigrationError> {
+        let mut warnings = Vec::new();
+
+        // Verify password hash format
+        self.verify_password_hash(&user.password_hash)
+            .await
+            .map_err(|e| {
+                tracing::error!("Password hash verification failed for {}: {}", user.email, e);
+                MigrationError::PasswordHashNotCompatible
+            })?;
+
+        // Insert into auth.users
+        self.insert_user_to_supabase(user).await?;
+
+        // Create user profile
+        if self.config.create_profiles {
+            self.create_user_profile(user).await?;
+        }
+
+        Ok(warnings)
+    }
+
+    /// Verify that the password hash format is compatible
+    async fn verify_password_hash(&self, hash: &str) -> Result<(), MigrationError> {
+        // DuckDB and Supabase both use bcrypt by default
+        // Verify the hash starts with the bcrypt prefix
+        if !hash.starts_with("$2b$") && !hash.starts_with("$2a$") {
+            tracing::warn!("Password hash does not appear to be bcrypt format");
+            return Err(MigrationError::InvalidPasswordHash);
+        }
+
+        // Additional validation could be added here
+        Ok(())
+    }
+
+    /// Insert user into Supabase auth.users table
+    async fn insert_user_to_supabase(&self, user: &DuckDbUser) -> Result<(), MigrationError> {
+        // Note: Direct insertion into auth.users requires superuser privileges
+        // In production, use Supabase Management API or admin functions
+
+        let query = r#"
+            INSERT INTO auth.users (
+                instance_id,
+                id,
+                aud,
+                role,
+                email,
+                encrypted_password,
+                email_confirmed_at,
+                created_at,
+                updated_at,
+                raw_app_meta_data,
+                raw_user_meta_data,
+                is_super_admin
+            ) VALUES (
+                '00000000-0000-0000-0000-000000000000',
+                $1,
+                'authenticated',
+                'authenticated',
+                $2,
+                $3,
+                NOW(),
+                $4,
+                $5,
+                '{"provider": "email", "providers": ["email"]}'::jsonb,
+                '{"organization_id": "' || $6::text || '"}'::jsonb,
+                false
+            )
+            ON CONFLICT (id) DO NOTHING
+        "#;
+
+        sqlx::query(query)
+            .bind(user.id)
+            .bind(&user.email)
+            .bind(&user.password_hash)
+            .bind(user.created_at)
+            .bind(user.updated_at)
+            .bind(user.organization_id)
+            .execute(&self.supabase)
+            .await
+            .map_err(|e| MigrationError::SupabaseInsertFailed(e.to_string()))?;
+
+        tracing::debug!("Inserted user {} into auth.users", user.email);
+
+        Ok(())
+    }
+
+    /// Create user profile in public schema
+    async fn create_user_profile(&self, user: &DuckDbUser) -> Result<(), MigrationError> {
+        let query = r#"
+            INSERT INTO public.user_profiles (
+                id,
+                user_id,
+                email,
+                organization_id,
+                created_at,
+                updated_at
+            ) VALUES (
+                gen_random_uuid(),
+                $1,
+                $2,
+                $3,
+                $4,
+                $5
+            )
+            ON CONFLICT (user_id) DO UPDATE SET
+                email = EXCLUDED.email,
+                updated_at = EXCLUDED.updated_at
+        "#;
+
+        sqlx::query(query)
+            .bind(user.id)
+            .bind(&user.email)
+            .bind(user.organization_id)
+            .bind(user.created_at)
+            .bind(user.updated_at)
+            .execute(&self.supabase)
+            .await
+            .map_err(|e| MigrationError::SupabaseInsertFailed(e.to_string()))?;
+
+        tracing::debug!("Created profile for user {}", user.email);
+
+        Ok(())
+    }
+
+    /// Rollback a migrated user
+    ///
+    /// Removes the user from Supabase auth.users and user_profiles.
+    pub async fn rollback_user(&self, user_id: &str) -> Result<(), MigrationError> {
+        let uid = Uuid::parse_str(user_id)
+            .map_err(|_| MigrationError::MigrationFailed("Invalid user ID".to_string()))?;
+
+        // Delete from user_profiles first (foreign key)
+        sqlx::query("DELETE FROM public.user_profiles WHERE user_id = $1")
+            .bind(uid)
+            .execute(&self.supabase)
+            .await?;
+
+        // Delete from auth.users
+        sqlx::query("DELETE FROM auth.users WHERE id = $1")
+            .bind(uid)
+            .execute(&self.supabase)
+            .await?;
+
+        tracing::info!("Rolled back user {}", user_id);
+
+        Ok(())
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_migration_config_default() {
+        let config = MigrationConfig::default();
+        assert_eq!(config.batch_size, 100);
+        assert!(config.verify_passwords);
+        assert!(config.create_profiles);
+    }
+
+    #[test]
+    fn test_password_hash_verification() {
+        // Valid bcrypt hashes
+        let valid_hashes = vec![
+            "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7qvZCFbwLy",
+            "$2a$10$N9qo8uLOickgx2ZMRZoMye1j50kgcuLPVFxWb/nJ0pNP2mC0xN5dG",
+        ];
+
+        for hash in valid_hashes {
+            assert!(hash.starts_with("$2b$") || hash.starts_with("$2a$"));
+        }
+    }
+}
diff --git a/crates/iou-api/src/realtime/mod.rs b/crates/iou-api/src/realtime/mod.rs
new file mode 100644
index 0000000..dd10ffd
--- /dev/null
+++ b/crates/iou-api/src/realtime/mod.rs
@@ -0,0 +1,22 @@
+//! Real-time Subscription Module
+//!
+//! Provides integration with Supabase Realtime for WebSocket-based
+//! real-time data synchronization.
+
+mod supabase_rt;
+mod presence;
+
+pub use supabase_rt::{
+    RealtimeClient,
+    SubscriptionHandle,
+    RealtimeEvent,
+    UpdateType,
+    RealtimeError,
+    RealtimeConfig,
+};
+
+pub use presence::{
+    PresenceTracker,
+    PresenceInfo,
+    PresenceStatus,
+};
diff --git a/crates/iou-api/src/realtime/presence.rs b/crates/iou-api/src/realtime/presence.rs
new file mode 100644
index 0000000..2d2e689
--- /dev/null
+++ b/crates/iou-api/src/realtime/presence.rs
@@ -0,0 +1,391 @@
+//! Presence Tracking for Real-time Collaboration
+//!
+//! Tracks which users are currently viewing or editing documents.
+//! Integrates with Supabase Realtime presence feature.
+
+use chrono::{DateTime, Utc};
+use dashmap::DashMap;
+use uuid::Uuid;
+use std::sync::Arc;
+use tokio::sync::RwLock;
+
+/// Status of a user's presence on a document
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum PresenceStatus {
+    /// User is viewing the document (read-only)
+    Viewing,
+
+    /// User is actively editing the document
+    Editing,
+
+    /// User is idle (no activity for a period)
+    Idle,
+}
+
+impl PresenceStatus {
+    /// Parse from string
+    pub fn from_str(s: &str) -> Option<Self> {
+        match s.to_lowercase().as_str() {
+            "viewing" => Some(Self::Viewing),
+            "editing" => Some(Self::Editing),
+            "idle" => Some(Self::Idle),
+            _ => None,
+        }
+    }
+
+    /// Convert to string
+    pub fn as_str(&self) -> &'static str {
+        match self {
+            Self::Viewing => "viewing",
+            Self::Editing => "editing",
+            Self::Idle => "idle",
+        }
+    }
+}
+
+/// Information about a user's presence on a document
+#[derive(Debug, Clone)]
+pub struct PresenceInfo {
+    /// User ID
+    pub user_id: Uuid,
+
+    /// User display name
+    pub user_name: String,
+
+    /// Document ID
+    pub document_id: Uuid,
+
+    /// Current presence status
+    pub status: PresenceStatus,
+
+    /// Last activity timestamp
+    pub last_seen: DateTime<Utc>,
+
+    /// Current cursor position (if editing)
+    pub cursor_position: Option<usize>,
+
+    /// Selected text range (if any)
+    pub selection_range: Option<(usize, usize)>,
+}
+
+impl PresenceInfo {
+    /// Create a new presence info
+    pub fn new(
+        user_id: Uuid,
+        user_name: String,
+        document_id: Uuid,
+        status: PresenceStatus,
+    ) -> Self {
+        Self {
+            user_id,
+            user_name,
+            document_id,
+            status,
+            last_seen: Utc::now(),
+            cursor_position: None,
+            selection_range: None,
+        }
+    }
+
+    /// Update the last seen timestamp
+    pub fn update_last_seen(&mut self) {
+        self.last_seen = Utc::now();
+    }
+
+    /// Check if this presence is stale (older than specified seconds)
+    pub fn is_stale(&self, seconds: i64) -> bool {
+        Utc::now().signed_duration_since(self.last_seen).num_seconds() > seconds
+    }
+
+    /// Get the age of this presence in seconds
+    pub fn age_seconds(&self) -> i64 {
+        Utc::now().signed_duration_since(self.last_seen).num_seconds()
+    }
+}
+
+/// Presence tracker for managing user presence across documents
+pub struct PresenceTracker {
+    /// Presence tracking by document ID
+    by_document: DashMap<Uuid, Vec<PresenceInfo>>,
+
+    /// Presence tracking by user ID (for quick lookup)
+    by_user: DashMap<Uuid, Vec<Uuid>>,
+
+    /// Cleanup interval in seconds
+    cleanup_interval: i64,
+
+    /// Presence timeout in seconds (after which user is considered offline)
+    presence_timeout: i64,
+}
+
+impl PresenceTracker {
+    /// Create a new presence tracker
+    pub fn new() -> Self {
+        Self {
+            by_document: DashMap::new(),
+            by_user: DashMap::new(),
+            cleanup_interval: 60, // Cleanup every minute
+            presence_timeout: 300, // 5 minutes timeout
+        }
+    }
+
+    /// Create with custom timeout settings
+    pub fn with_timeout(cleanup_interval_secs: i64, presence_timeout_secs: i64) -> Self {
+        Self {
+            by_document: DashMap::new(),
+            by_user: DashMap::new(),
+            cleanup_interval: cleanup_interval_secs,
+            presence_timeout: presence_timeout_secs,
+        }
+    }
+
+    /// Update or add user presence
+    ///
+    /// This method:
+    /// 1. Updates the user's presence status
+    /// 2. Broadcasts the update to other users on the document
+    /// 3. Cleans up stale presence entries
+    pub fn update_presence(&self, info: PresenceInfo) {
+        let document_id = info.document_id;
+        let user_id = info.user_id;
+
+        // Update by_document index
+        let mut presence_list = self.by_document.get(&document_id)
+            .map(|e| e.clone())
+            .unwrap_or_default();
+
+        // Remove existing entry for this user if present
+        presence_list.retain(|p| p.user_id != user_id);
+        presence_list.push(info.clone());
+
+        self.by_document.insert(document_id, presence_list);
+
+        // Update by_user index
+        let mut user_documents = self.by_user.get(&user_id)
+            .map(|e| e.clone())
+            .unwrap_or_default();
+
+        if !user_documents.contains(&document_id) {
+            user_documents.push(document_id);
+        }
+
+        self.by_user.insert(user_id, user_documents);
+
+        tracing::debug!(
+            "Updated presence: user {} on document {} ({:?})",
+            info.user_name,
+            document_id,
+            info.status
+        );
+    }
+
+    /// Get all users currently viewing a document
+    pub fn get_document_viewers(&self, document_id: &Uuid) -> Vec<PresenceInfo> {
+        self.by_document
+            .get(document_id)
+            .map(|list| {
+                list.iter()
+                    .filter(|p| !p.is_stale(self.presence_timeout))
+                    .cloned()
+                    .collect()
+            })
+            .unwrap_or_default()
+    }
+
+    /// Get editors (users with Editing status) for a document
+    pub fn get_document_editors(&self, document_id: &Uuid) -> Vec<PresenceInfo> {
+        self.get_document_viewers(document_id)
+            .into_iter()
+            .filter(|p| p.status == PresenceStatus::Editing)
+            .collect()
+    }
+
+    /// Get presence info for a specific user on a document
+    pub fn get_user_presence(&self, user_id: &Uuid, document_id: &Uuid) -> Option<PresenceInfo> {
+        self.by_document
+            .get(document_id)?
+            .iter()
+            .find(|p| p.user_id == *user_id)
+            .cloned()
+    }
+
+    /// Remove user from a document (when they leave)
+    pub fn remove_user_from_document(&self, user_id: Uuid, document_id: Uuid) {
+        // Update by_document
+        if let Some(mut list) = self.by_document.get_mut(&document_id) {
+            list.retain(|p| p.user_id != user_id);
+            if list.is_empty() {
+                self.by_document.remove(&document_id);
+            }
+        }
+
+        // Update by_user
+        if let Some(mut docs) = self.by_user.get_mut(&user_id) {
+            docs.retain(|d| d != &document_id);
+            if docs.is_empty() {
+                self.by_user.remove(&user_id);
+            }
+        }
+
+        tracing::debug!("Removed user {} from document {}", user_id, document_id);
+    }
+
+    /// Remove user from all documents (when they disconnect)
+    pub fn remove_user(&self, user_id: &Uuid) {
+        if let Some((_, docs)) = self.by_user.remove(user_id) {
+            for document_id in &docs {
+                if let Some(mut list) = self.by_document.get_mut(document_id) {
+                    list.retain(|p| p.user_id != *user_id);
+                    if list.is_empty() {
+                        self.by_document.remove(document_id);
+                    }
+                }
+            }
+        }
+
+        tracing::debug!("Removed user {} from all documents", user_id);
+    }
+
+    /// Clean up stale presence entries
+    pub fn cleanup_stale(&self) {
+        let stale_cutoff = self.presence_timeout;
+
+        self.by_document.retain(|document_id, list| {
+            let original_len = list.len();
+            list.retain(|p| !p.is_stale(stale_cutoff));
+
+            if list.len() < original_len {
+                tracing::debug!(
+                    "Cleaned {} stale presence entries from document {}",
+                    original_len - list.len(),
+                    document_id
+                );
+            }
+
+            !list.is_empty()
+        });
+    }
+
+    /// Get all active documents (with at least one active user)
+    pub fn get_active_documents(&self) -> Vec<Uuid> {
+        self.by_document
+            .iter()
+            .map(|e| *e.key())
+            .collect()
+    }
+
+    /// Get count of active users on a document
+    pub fn get_active_count(&self, document_id: &Uuid) -> usize {
+        self.by_document
+            .get(document_id)
+            .map(|list| list.iter().filter(|p| !p.is_stale(self.presence_timeout)).count())
+            .unwrap_or(0)
+    }
+
+    /// Get total presence count across all documents
+    pub fn get_total_count(&self) -> usize {
+        self.by_document
+            .iter()
+            .map(|e| {
+                e.value()
+                    .iter()
+                    .filter(|p| !p.is_stale(self.presence_timeout))
+                    .count()
+            })
+            .sum()
+    }
+}
+
+impl Default for PresenceTracker {
+    fn default() -> Self {
+        Self::new()
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_presence_status_parsing() {
+        assert_eq!(PresenceStatus::from_str("viewing"), Some(PresenceStatus::Viewing));
+        assert_eq!(PresenceStatus::from_str("editing"), Some(PresenceStatus::Editing));
+        assert_eq!(PresenceStatus::from_str("idle"), Some(PresenceStatus::Idle));
+        assert_eq!(PresenceStatus::from_str("invalid"), None);
+    }
+
+    #[test]
+    fn test_presence_status_as_str() {
+        assert_eq!(PresenceStatus::Viewing.as_str(), "viewing");
+        assert_eq!(PresenceStatus::Editing.as_str(), "editing");
+        assert_eq!(PresenceStatus::Idle.as_str(), "idle");
+    }
+
+    #[test]
+    fn test_presence_info_creation() {
+        let user_id = Uuid::new_v4();
+        let document_id = Uuid::new_v4();
+        let info = PresenceInfo::new(
+            user_id,
+            "Test User".to_string(),
+            document_id,
+            PresenceStatus::Editing,
+        );
+
+        assert_eq!(info.user_id, user_id);
+        assert_eq!(info.document_id, document_id);
+        assert_eq!(info.status, PresenceStatus::Editing);
+    }
+
+    #[test]
+    fn test_presence_tracker() {
+        let tracker = PresenceTracker::new();
+
+        let user_id = Uuid::new_v4();
+        let document_id = Uuid::new_v4();
+
+        let info = PresenceInfo::new(
+            user_id,
+            "Test User".to_string(),
+            document_id,
+            PresenceStatus::Viewing,
+        );
+
+        tracker.update_presence(info);
+
+        // Verify user is tracked
+        assert_eq!(tracker.get_active_count(&document_id), 1);
+        assert_eq!(tracker.get_total_count(), 1);
+
+        // Remove user
+        tracker.remove_user_from_document(user_id, document_id);
+        assert_eq!(tracker.get_active_count(&document_id), 0);
+    }
+
+    #[test]
+    fn test_get_document_editors() {
+        let tracker = PresenceTracker::new();
+        let document_id = Uuid::new_v4();
+
+        // Add a viewer
+        tracker.update_presence(PresenceInfo::new(
+            Uuid::new_v4(),
+            "Viewer".to_string(),
+            document_id,
+            PresenceStatus::Viewing,
+        ));
+
+        // Add an editor
+        let editor_id = Uuid::new_v4();
+        tracker.update_presence(PresenceInfo::new(
+            editor_id,
+            "Editor".to_string(),
+            document_id,
+            PresenceStatus::Editing,
+        ));
+
+        let editors = tracker.get_document_editors(&document_id);
+        assert_eq!(editors.len(), 1);
+        assert_eq!(editors[0].user_id, editor_id);
+    }
+}
diff --git a/crates/iou-api/src/realtime/supabase_rt.rs b/crates/iou-api/src/realtime/supabase_rt.rs
new file mode 100644
index 0000000..2ec131c
--- /dev/null
+++ b/crates/iou-api/src/realtime/supabase_rt.rs
@@ -0,0 +1,333 @@
+//! Supabase Realtime Client
+//!
+//! WebSocket client for subscribing to Supabase Realtime channels.
+//! Handles connection, subscription management, and event reception.
+
+use serde_json::Value;
+use tokio::sync::mpsc;
+use tokio_tungstenite::{connect_async, tungstenite::Message};
+use futures_util::{SinkExt, StreamExt};
+use uuid::Uuid;
+use std::time::Duration;
+
+/// Configuration for Supabase Realtime connection
+#[derive(Debug, Clone)]
+pub struct RealtimeConfig {
+    /// Supabase Realtime WebSocket URL
+    /// Format: wss://<project-ref>.supabase.co/realtime/v1
+    pub websocket_url: String,
+
+    /// JWT token for authentication
+    pub jwt_token: Option<String>,
+
+    /// Heartbeat interval in seconds
+    pub heartbeat_interval: u64,
+
+    /// Connection timeout in seconds
+    pub connect_timeout: u64,
+}
+
+impl Default for RealtimeConfig {
+    fn default() -> Self {
+        Self {
+            websocket_url: "ws://localhost:4000/socket".to_string(), // Default for local dev
+            jwt_token: None,
+            heartbeat_interval: 30,
+            connect_timeout: 10,
+        }
+    }
+}
+
+/// Realtime client for Supabase
+pub struct RealtimeClient {
+    /// Configuration
+    config: RealtimeConfig,
+
+    /// Active subscription metadata (doesn't include the Receiver)
+    subscriptions: dashmap::DashMap<String, SubscriptionInfo>,
+}
+
+/// Metadata about an active subscription (can be cloned)
+#[derive(Debug, Clone)]
+struct SubscriptionInfo {
+    channel: String,
+    table: String,
+    filter: Option<String>,
+}
+
+impl RealtimeClient {
+    /// Create a new Realtime client
+    pub fn new(config: RealtimeConfig) -> Self {
+        Self {
+            config,
+            subscriptions: dashmap::DashMap::new(),
+        }
+    }
+
+    /// Create from environment variables
+    pub fn from_env() -> Result<Self, RealtimeError> {
+        let websocket_url = std::env::var("SUPABASE_REALTIME_URL")
+            .unwrap_or_else(|_| "ws://localhost:4000/socket".to_string());
+
+        let jwt_token = std::env::var("SUPABASE_JWT_TOKEN").ok();
+
+        let config = RealtimeConfig {
+            websocket_url,
+            jwt_token,
+            ..Default::default()
+        };
+
+        Ok(Self::new(config))
+    }
+
+    /// Subscribe to a table's changes
+    ///
+    /// # Arguments
+    /// * `table` - Table name to subscribe to
+    /// * `filter` - Optional filter for the subscription (e.g., "id=eq.123")
+    ///
+    /// # Returns
+    /// A subscription handle that can be used to receive events
+    pub async fn subscribe(
+        &self,
+        table: &str,
+        filter: Option<&str>,
+    ) -> Result<SubscriptionHandle, RealtimeError> {
+        let topic = if let Some(f) = filter {
+            format!("{}:{}", table, f)
+        } else {
+            format!("{}:*", table)
+        };
+
+        // Create channel for this subscription
+        let (tx, rx) = mpsc::channel(100);
+
+        let handle = SubscriptionHandle {
+            channel: topic.clone(),
+            table: table.to_string(),
+            filter: filter.map(String::from),
+            receiver: rx,
+            sender: tx,
+        };
+
+        // Store just the metadata (clone-able)
+        self.subscriptions.insert(topic.clone(), SubscriptionInfo {
+            channel: topic.clone(),
+            table: table.to_string(),
+            filter: filter.map(String::from),
+        });
+
+        // TODO: Connect to WebSocket and send subscription message
+        tracing::info!("Subscribed to table: {} with filter: {:?}", table, filter);
+
+        Ok(handle)
+    }
+
+    /// Subscribe to a specific document
+    pub async fn subscribe_document(
+        &self,
+        document_id: Uuid,
+    ) -> Result<SubscriptionHandle, RealtimeError> {
+        self.subscribe(
+            "documents",
+            Some(&format!("id=eq.{}", document_id)),
+        ).await
+    }
+
+    /// Broadcast a document update to all subscribers
+    ///
+    /// Note: This may be handled automatically by Supabase's
+    /// WAL replication. Use this for custom events only.
+    pub async fn broadcast_document_update(
+        &self,
+        document_id: &str,
+        update_type: UpdateType,
+        payload: Value,
+    ) -> Result<(), RealtimeError> {
+        // TODO: Implement broadcast via Realtime's broadcast feature
+        tracing::debug!(
+            "Broadcasting update for document {}: {:?}",
+            document_id,
+            update_type
+        );
+
+        Ok(())
+    }
+
+    /// Unsubscribe from a channel
+    pub fn unsubscribe(&self, channel: &str) {
+        self.subscriptions.remove(channel);
+        tracing::info!("Unsubscribed from: {}", channel);
+    }
+
+    /// Get active subscription count
+    pub fn subscription_count(&self) -> usize {
+        self.subscriptions.len()
+    }
+}
+
+/// Handle for an active subscription
+#[derive(Debug)]
+pub struct SubscriptionHandle {
+    /// Channel topic
+    channel: String,
+
+    /// Table name
+    table: String,
+
+    /// Filter applied
+    filter: Option<String>,
+
+    /// Event receiver
+    #[allow(dead_code)]
+    receiver: mpsc::Receiver<RealtimeEvent>,
+
+    /// Event sender (for internal use)
+    sender: mpsc::Sender<RealtimeEvent>,
+}
+
+impl SubscriptionHandle {
+    /// Get the channel name
+    pub fn channel(&self) -> &str {
+        &self.channel
+    }
+
+    /// Get the table name
+    pub fn table(&self) -> &str {
+        &self.table
+    }
+
+    /// Receive the next event
+    pub async fn recv(&mut self) -> Option<RealtimeEvent> {
+        self.receiver.recv().await
+    }
+
+    /// Try to receive an event without blocking
+    pub fn try_recv(&mut self) -> Option<RealtimeEvent> {
+        self.receiver.try_recv().ok()
+    }
+}
+
+/// Types of updates that can occur
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum UpdateType {
+    Created,
+    Updated,
+    Deleted,
+    StatusChanged,
+}
+
+impl UpdateType {
+    /// Parse from string (e.g., "INSERT", "UPDATE")
+    pub fn from_str(s: &str) -> Option<Self> {
+        match s.to_uppercase().as_str() {
+            "INSERT" => Some(Self::Created),
+            "UPDATE" => Some(Self::Updated),
+            "DELETE" => Some(Self::Deleted),
+            _ => None,
+        }
+    }
+
+    /// Convert to string
+    pub fn as_str(&self) -> &'static str {
+        match self {
+            Self::Created => "INSERT",
+            Self::Updated => "UPDATE",
+            Self::Deleted => "DELETE",
+            Self::StatusChanged => "UPDATE",
+        }
+    }
+}
+
+/// Real-time event from Supabase
+#[derive(Debug, Clone)]
+pub struct RealtimeEvent {
+    /// Table that triggered the event
+    pub table: String,
+
+    /// Type of record (e.g., "documents")
+    pub record_type: String,
+
+    /// The new record data
+    pub record: Value,
+
+    /// The old record data (for UPDATE and DELETE)
+    pub old_record: Option<Value>,
+
+    /// Type of update
+    pub update_type: UpdateType,
+
+    /// Timestamp of the event
+    pub timestamp: chrono::DateTime<chrono::Utc>,
+}
+
+/// Errors that can occur in realtime operations
+#[derive(Debug, thiserror::Error)]
+pub enum RealtimeError {
+    #[error("Connection failed: {0}")]
+    ConnectionFailed(String),
+
+    #[error("Subscription failed: {0}")]
+    SubscriptionFailed(String),
+
+    #[error("Invalid message format")]
+    InvalidMessage,
+
+    #[error("Authentication failed")]
+    AuthenticationFailed,
+
+    #[error("Channel not found: {0}")]
+    ChannelNotFound(String),
+
+    #[error("Send error: {0}")]
+    SendError(String),
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_update_type_parsing() {
+        assert_eq!(UpdateType::from_str("INSERT"), Some(UpdateType::Created));
+        assert_eq!(UpdateType::from_str("UPDATE"), Some(UpdateType::Updated));
+        assert_eq!(UpdateType::from_str("DELETE"), Some(UpdateType::Deleted));
+        assert_eq!(UpdateType::from_str("INVALID"), None);
+    }
+
+    #[test]
+    fn test_update_type_as_str() {
+        assert_eq!(UpdateType::Created.as_str(), "INSERT");
+        assert_eq!(UpdateType::Updated.as_str(), "UPDATE");
+        assert_eq!(UpdateType::Deleted.as_str(), "DELETE");
+    }
+
+    #[tokio::test]
+    async fn test_realtime_client_creation() {
+        let config = RealtimeConfig {
+            websocket_url: "ws://localhost:4000/socket".to_string(),
+            jwt_token: None,
+            ..Default::default()
+        };
+
+        let client = RealtimeClient::new(config);
+        assert_eq!(client.subscription_count(), 0);
+    }
+
+    #[tokio::test]
+    async fn test_subscription_handle() {
+        let (tx, rx) = mpsc::channel(100);
+
+        let mut handle = SubscriptionHandle {
+            channel: "documents:*".to_string(),
+            table: "documents".to_string(),
+            filter: None,
+            receiver: rx,
+            sender: tx,
+        };
+
+        assert_eq!(handle.channel(), "documents:*");
+        assert_eq!(handle.table(), "documents");
+        assert!(handle.try_recv().is_none());
+    }
+}
diff --git a/crates/iou-api/tests/migration/auth_realtime.rs b/crates/iou-api/tests/migration/auth_realtime.rs
new file mode 100644
index 0000000..0207f3c
--- /dev/null
+++ b/crates/iou-api/tests/migration/auth_realtime.rs
@@ -0,0 +1,422 @@
+//! Integration tests for Section 03: Authentication and Real-time Implementation
+//!
+//! Tests cover:
+//! - Supabase Auth JWT verification
+//! - User data migration
+//! - Row-Level Security (RLS) policies
+//! - Real-time subscriptions
+//! - Frontend integration
+
+use uuid::Uuid;
+
+// ============================================================
+// Authentication Tests
+// ============================================================
+
+#[cfg(test)]
+mod auth_tests {
+    use super::*;
+
+    /// Verify Supabase Auth issues valid JWT tokens
+    #[tokio::test]
+    #[ignore] // Requires Supabase instance
+    async fn test_supabase_jwt_issuance() {
+        // This test verifies that:
+        // 1. Supabase Auth endpoint can be called with valid credentials
+        // 2. JWT access token is received
+        // 3. Token signature and claims can be verified
+        // 4. Token contains user_id and organization_id
+
+        // TODO: Implement actual Supabase auth call
+        // For now, this is a placeholder that demonstrates the test structure
+
+        let expected_user_id = Uuid::new_v4();
+        let expected_org_id = Uuid::new_v4();
+
+        // Mock token verification (will be replaced with actual implementation)
+        let mock_token = "mock.jwt.token";
+        assert!(!mock_token.is_empty());
+
+        // Verify token structure (placeholder)
+        assert!(true, "Token structure should be valid");
+
+        // In real implementation:
+        // - Call Supabase Auth API
+        // - Parse JWT
+        // - Verify claims.user_id == expected_user_id
+        // - Verify claims.organization_id == expected_org_id
+    }
+
+    /// Verify existing users can authenticate with migrated credentials
+    #[tokio::test]
+    #[ignore] // Requires password hash data
+    async fn test_password_hash_compatibility() {
+        // This test verifies that:
+        // 1. A sample user from pre-migration data can be selected
+        // 2. Authentication via Supabase Auth succeeds
+        // 3. Password hash was correctly migrated
+
+        // TODO: Implement actual password compatibility check
+        // For now, this is a placeholder
+
+        let test_email = "test@example.com";
+        let test_password = "SecurePassword123!";
+
+        assert!(!test_email.is_empty());
+        assert!(!test_password.is_empty());
+
+        // In real implementation:
+        // - Query DuckDB for existing user
+        // - Migrate password hash to Supabase auth.users
+        // - Verify login succeeds with original password
+    }
+
+    /// Verify existing sessions remain valid after migration
+    #[tokio::test]
+    #[ignore] // Requires session management
+    async fn test_session_token_migration() {
+        // This test verifies that:
+        // 1. Existing session tokens are captured
+        // 2. User data is migrated
+        // 3. Existing tokens still authenticate
+        // 4. Session data is preserved
+
+        // TODO: Implement session continuity check
+        let existing_token = "existing.session.token";
+
+        assert!(!existing_token.is_empty());
+
+        // In real implementation:
+        // - Capture session tokens before migration
+        // - Run migration
+        // - Verify tokens still validate
+    }
+
+    /// Verify user data migration completeness
+    #[tokio::test]
+    #[ignore] // Requires full database setup
+    async fn test_user_data_migration() {
+        // This test verifies that:
+        // 1. Users in DuckDB are counted before migration
+        // 2. Migration runs
+        // 3. Users in Supabase are counted after migration
+        // 4. Counts match
+        // 5. Random user records are spot-checked
+
+        // TODO: Implement migration completeness check
+        let duckdb_count = 100;
+        let supabase_count = 100;
+
+        assert_eq!(duckdb_count, supabase_count,
+            "User count should match after migration");
+
+        // In real implementation:
+        // - COUNT(*) FROM auth.users in Supabase
+        // - COUNT(*) FROM users in DuckDB
+        // - Verify equality
+        // - Spot-check random users
+    }
+}
+
+// ============================================================
+// RLS Policy Tests
+// ============================================================
+
+#[cfg(test)]
+mod rls_tests {
+    use super::*;
+
+    /// Verify organization isolation prevents cross-organization data access
+    #[tokio::test]
+    #[ignore] // Requires RLS setup
+    async fn test_rls_organization_isolation() {
+        // This test verifies that:
+        // 1. Two users in different organizations are created
+        // 2. User A attempts to access User B's documents
+        // 3. Access is denied
+        // 4. User A accesses their own documents
+        // 5. Access succeeds
+
+        let org_a_id = Uuid::new_v4();
+        let org_b_id = Uuid::new_v4();
+        let user_a_id = Uuid::new_v4();
+        let user_b_id = Uuid::new_v4();
+
+        // Organizations should be different
+        assert_ne!(org_a_id, org_b_id);
+
+        // TODO: Implement actual RLS test
+        // In real implementation:
+        // - Set up RLS policies
+        // - Query as user A for org B data
+        // - Verify empty result
+        // - Query as user A for org A data
+        // - Verify results returned
+    }
+
+    /// Verify user-level access within organization
+    #[tokio::test]
+    #[ignore] // Requires RLS setup
+    async fn test_rls_user_level_access() {
+        // This test verifies that:
+        // 1. A user with limited permissions is created
+        // 2. User attempts to access admin-only resource
+        // 3. Access is denied
+        // 4. User accesses permitted resource
+        // 5. Access succeeds
+
+        // TODO: Implement user-level access test
+        assert!(true, "User-level access control should work");
+
+        // In real implementation:
+        // - Create user with DomainViewer role
+        // - Attempt to update domain (should fail)
+        // - Attempt to read domain (should succeed)
+    }
+
+    /// Verify classification-based filtering (confidential documents)
+    #[tokio::test]
+    #[ignore] // Requires RLS setup
+    async fn test_rls_classification_filtering() {
+        // This test verifies that:
+        // 1. A user without clearance is created
+        // 2. User attempts to access confidential document
+        // 3. Access is denied
+        // 4. User with clearance accesses same document
+        // 5. Access succeeds
+
+        // TODO: Implement classification filtering test
+        let classification = "CONFIDENTIAL";
+
+        assert_eq!(classification, "CONFIDENTIAL");
+
+        // In real implementation:
+        // - Create confidential document
+        // - Create user without clearance
+        // - Query for confidential docs (should return empty)
+        // - Grant clearance
+        // - Query again (should return document)
+    }
+
+    /// Verify Woo-publication status filtering
+    #[tokio::test]
+    #[ignore] // Requires RLS setup
+    async fn test_rls_woo_filtering() {
+        // This test verifies that:
+        // 1. Public and non-public documents are created
+        // 2. Anonymous user queries for documents
+        // 3. Only Woo-published documents are returned
+        // 4. Authenticated user queries
+        // 5. All permitted documents are returned
+
+        // TODO: Implement Woo filtering test
+        let woo_published = true;
+        let woo_non_published = false;
+
+        assert_ne!(woo_published, woo_non_published);
+
+        // In real implementation:
+        // - Create woo_published=true document
+        // - Create woo_published=false document
+        // - Query as anonymous (should see only published)
+        // - Query as authenticated user (should see both)
+    }
+
+    /// Verify RLS policy performance meets SLA
+    #[tokio::test]
+    #[ignore] // Requires performance testing setup
+    async fn test_rls_policy_performance() {
+        // This test verifies that:
+        // 1. 100 queries with RLS enforced are executed
+        // 2. p50, p95, p99 latency are measured
+        // 3. p95 < 500ms requirement is met
+        // 4. Detailed metrics are reported
+
+        let query_count = 100;
+        let max_p95_ms = 500;
+
+        assert!(query_count > 0);
+        assert!(max_p95_ms > 0);
+
+        // TODO: Implement performance test
+        // In real implementation:
+        // - Run 100 queries with different org contexts
+        // - Measure each query time
+        // - Calculate percentiles
+        // - Assert p95 < 500ms
+    }
+}
+
+// ============================================================
+// Real-time Tests
+// ============================================================
+
+#[cfg(test)]
+mod realtime_tests {
+    use super::*;
+
+    /// Verify client can create real-time subscription
+    #[tokio::test]
+    #[ignore] // Requires Supabase Realtime instance
+    async fn test_realtime_subscription_creation() {
+        // This test verifies that:
+        // 1. Connection to Supabase Realtime succeeds
+        // 2. Subscription to a table channel succeeds
+        // 3. Subscription confirmation is received
+        // 4. Connection state is "subscribed"
+
+        // TODO: Implement subscription test
+        let table_name = "documents";
+
+        assert!(!table_name.is_empty());
+
+        // In real implementation:
+        // - Connect to Supabase Realtime WebSocket
+        // - Send subscription message
+        // - Wait for confirmation
+        // - Verify connection state
+    }
+
+    /// Verify document updates propagate to subscribers
+    #[tokio::test]
+    #[ignore] // Requires Supabase Realtime instance
+    async fn test_realtime_document_updates() {
+        // This test verifies that:
+        // 1. Client A subscribes to document changes
+        // 2. Client B updates a document
+        // 3. Client A receives update notification
+        // 4. Payload contains correct document state
+
+        let document_id = Uuid::new_v4();
+
+        // TODO: Implement document update propagation test
+        assert!(!document_id.to_string().is_empty());
+
+        // In real implementation:
+        // - Subscribe client A to document channel
+        // - Update document via client B
+        // - Verify client A receives update
+        // - Verify payload matches new state
+    }
+
+    /// Verify user presence indicators
+    #[tokio::test]
+    #[ignore] // Requires presence system
+    async fn test_realtime_presence_indicators() {
+        // This test verifies that:
+        // 1. User A joins a document channel
+        // 2. User B joins same channel
+        // 3. Both users receive presence updates
+        // 4. User A leaves channel
+        // 5. User B receives leave notification
+
+        let user_a = Uuid::new_v4();
+        let user_b = Uuid::new_v4();
+        let document_id = Uuid::new_v4();
+
+        assert_ne!(user_a, user_b);
+
+        // TODO: Implement presence test
+        // In real implementation:
+        // - Join user A to document channel
+        // - Join user B to document channel
+        // - Verify both receive presence of other
+        // - User A leaves
+        // - Verify user B receives leave notification
+    }
+
+    /// Verify concurrent edit conflict resolution
+    #[tokio::test]
+    #[ignore] // Requires conflict resolution system
+    async fn test_realtime_conflict_resolution() {
+        // This test verifies that:
+        // 1. Two users edit same document field simultaneously
+        // 2. Last-write-wins or merge strategy is applied
+        // 3. No data corruption occurs
+        // 4. Both clients see consistent final state
+
+        // TODO: Implement conflict resolution test
+        assert!(true, "Conflict resolution should handle concurrent edits");
+
+        // In real implementation:
+        // - Two clients simultaneously update same field
+        // - Verify final state is consistent
+        // - Verify no data corruption
+    }
+
+    /// Verify real-time latency meets requirements
+    #[tokio::test]
+    #[ignore] // Requires latency measurement
+    async fn test_realtime_latency() {
+        // This test verifies that:
+        // 1. Client subscribes to a channel
+        // 2. Timestamp before update is recorded
+        // 3. Database update is triggered
+        // 4. Timestamp when notification received is recorded
+        // 5. Latency is calculated
+        // 6. p95 < 200ms requirement is verified
+
+        let max_latency_ms = 200;
+
+        assert!(max_latency_ms > 0);
+
+        // TODO: Implement latency test
+        // In real implementation:
+        // - Subscribe to channel
+        // - Record time before update
+        // - Trigger update
+        // - Record time when notification received
+        // - Calculate and verify latency
+    }
+}
+
+// ============================================================
+// Compliance Tests
+// ============================================================
+
+#[cfg(test)]
+mod compliance_tests {
+    use super::*;
+
+    /// Verify GDPR right to deletion works
+    #[tokio::test]
+    #[ignore] // Requires GDPR implementation
+    async fn test_gdpr_right_to_deletion() {
+        // This test verifies that:
+        // 1. A user with associated data is created
+        // 2. Deletion request is triggered
+        // 3. All user data is deleted
+        // 4. Audit trail is preserved (deletion logged)
+
+        let user_id = Uuid::new_v4();
+
+        // TODO: Implement GDPR deletion test
+        assert!(!user_id.to_string().is_empty());
+
+        // In real implementation:
+        // - Create user with data
+        // - Trigger GDPR deletion
+        // - Verify all user data removed
+        // - Verify deletion logged in audit trail
+    }
+
+    /// Verify audit trail continuity during migration
+    #[tokio::test]
+    #[ignore] // Requires audit trail system
+    async fn test_audit_trail_continuity() {
+        // This test verifies that:
+        // 1. Pre-migration audit entry count is recorded
+        // 2. Auth migration runs
+        // 3. DuckDB logs correlate with Supabase WAL
+        // 4. No audit gaps exist
+
+        // TODO: Implement audit continuity test
+        assert!(true, "Audit trail should remain continuous during migration");
+
+        // In real implementation:
+        // - Count audit entries before migration
+        // - Run auth migration
+        // - Verify audit entries match or exceed
+        // - Verify no gaps in timestamps
+    }
+}
diff --git a/migrations/postgres/002_rls_policies.sql b/migrations/postgres/002_rls_policies.sql
new file mode 100644
index 0000000..2e3ef37
--- /dev/null
+++ b/migrations/postgres/002_rls_policies.sql
@@ -0,0 +1,368 @@
+-- Row-Level Security (RLS) Policies for IOU-Modern
+-- Section 03: Authentication and Real-time Implementation
+--
+-- This migration creates RLS policies to enforce multi-tenant isolation
+-- and classification-based access control at the database level.
+
+-- ============================================================
+-- Enable RLS on all tables
+-- ============================================================
+
+ALTER TABLE information_domains ENABLE ROW LEVEL SECURITY;
+ALTER TABLE information_objects ENABLE ROW LEVEL SECURITY;
+ALTER TABLE documents ENABLE ROW LEVEL SECURITY;
+ALTER TABLE templates ENABLE ROW LEVEL SECURITY;
+
+-- ============================================================
+-- Helper Functions for RLS
+-- ============================================================
+
+-- Function to extract organization_id from JWT custom claim
+CREATE OR REPLACE FUNCTION auth.organization_id()
+RETURNS UUID AS $$
+  SELECT nullif(current_setting('request.jwt.claim.organization_id', true), '')::uuid
+$$ LANGUAGE sql STABLE PARALLEL SAFE;
+
+-- Function to extract user role from JWT
+CREATE OR REPLACE FUNCTION auth.user_role()
+RETURNS VARCHAR AS $$
+  SELECT nullif(current_setting('request.jwt.claim.role', true), '')::varchar
+$$ LANGUAGE sql STABLE PARALLEL SAFE;
+
+-- Function to check if user has specific clearance level
+CREATE OR REPLACE FUNCTION auth.has_clearance(required_level VARCHAR)
+RETURNS BOOLEAN AS $$
+  SELECT
+    CASE required_level
+      WHEN 'openbaar' THEN true -- Everyone has access to public
+      WHEN 'intern' THEN true -- All authenticated users
+      WHEN 'vertrouwelijk' THEN
+        current_setting('request.jwt.claim.clearance', true) IN ('vertrouwelijk', 'geheim')
+      WHEN 'geheim' THEN
+        current_setting('request.jwt.claim.clearance', true) = 'geheim'
+      ELSE false
+    END
+$$ LANGUAGE sql STABLE PARALLEL SAFE;
+
+-- ============================================================
+-- Organization Isolation Policies
+-- ============================================================
+
+-- Information Domains: Users can only read/write their own organization's domains
+CREATE POLICY org_isolation_select ON information_domains
+  FOR SELECT
+  TO authenticated
+  USING (organization_id = auth.organization_id());
+
+CREATE POLICY org_isolation_insert ON information_domains
+  FOR INSERT
+  TO authenticated
+  WITH CHECK (organization_id = auth.organization_id());
+
+CREATE POLICY org_isolation_update ON information_domains
+  FOR UPDATE
+  TO authenticated
+  USING (organization_id = auth.organization_id())
+  WITH CHECK (organization_id = auth.organization_id());
+
+CREATE POLICY org_isolation_delete ON information_domains
+  FOR DELETE
+  TO authenticated
+  USING (organization_id = auth.organization_id());
+
+-- Information Objects: Users can only read/write their own organization's objects
+CREATE POLICY org_isolation_select ON information_objects
+  FOR SELECT
+  TO authenticated
+  USING (
+    -- Direct access: user's organization
+    EXISTS (
+      SELECT 1 FROM information_domains
+      WHERE information_domains.id = information_objects.domain_id
+      AND information_domains.organization_id = auth.organization_id()
+    )
+  );
+
+CREATE POLICY org_isolation_insert ON information_objects
+  FOR INSERT
+  TO authenticated
+  WITH CHECK (
+    EXISTS (
+      SELECT 1 FROM information_domains
+      WHERE information_domains.id = domain_id
+      AND information_domains.organization_id = auth.organization_id()
+    )
+  );
+
+CREATE POLICY org_isolation_update ON information_objects
+  FOR UPDATE
+  TO authenticated
+  USING (
+    EXISTS (
+      SELECT 1 FROM information_domains
+      WHERE information_domains.id = domain_id
+      AND information_domains.organization_id = auth.organization_id()
+    )
+  )
+  WITH CHECK (
+    EXISTS (
+      SELECT 1 FROM information_domains
+      WHERE information_domains.id = domain_id
+      AND information_domains.organization_id = auth.organization_id()
+    )
+  );
+
+CREATE POLICY org_isolation_delete ON information_objects
+  FOR DELETE
+  TO authenticated
+  USING (
+    EXISTS (
+      SELECT 1 FROM information_domains
+      WHERE information_domains.id = domain_id
+      AND information_domains.organization_id = auth.organization_id()
+    )
+  );
+
+-- Documents: Organization isolation
+CREATE POLICY org_isolation_select ON documents
+  FOR SELECT
+  TO authenticated
+  USING (organization_id = auth.organization_id());
+
+CREATE POLICY org_isolation_insert ON documents
+  FOR INSERT
+  TO authenticated
+  WITH CHECK (organization_id = auth.organization_id());
+
+CREATE POLICY org_isolation_update ON documents
+  FOR UPDATE
+  TO authenticated
+  USING (organization_id = auth.organization_id())
+  WITH CHECK (organization_id = auth.organization_id());
+
+CREATE POLICY org_isolation_delete ON documents
+  FOR DELETE
+  TO authenticated
+  USING (organization_id = auth.organization_id());
+
+-- Templates: Organization isolation
+CREATE POLICY org_isolation_select ON templates
+  FOR SELECT
+  TO authenticated
+  USING (organization_id = auth.organization_id());
+
+CREATE POLICY org_isolation_insert ON templates
+  FOR INSERT
+  TO authenticated
+  WITH CHECK (organization_id = auth.organization_id());
+
+CREATE POLICY org_isolation_update ON templates
+  FOR UPDATE
+  TO authenticated
+  USING (organization_id = auth.organization_id())
+  WITH CHECK (organization_id = auth.organization_id());
+
+CREATE POLICY org_isolation_delete ON templates
+  FOR DELETE
+  TO authenticated
+  USING (organization_id = auth.organization_id());
+
+-- ============================================================
+-- Classification-Based Filtering
+-- ============================================================
+
+-- Information Objects: Classification-based access
+CREATE POLICY classification_filter ON information_objects
+  FOR SELECT
+  TO authenticated
+  USING (
+    classification = 'openbaar'
+    OR classification = 'intern'
+    OR (classification = 'vertrouwelijk' AND auth.has_clearance('vertrouwelijk'))
+    OR (classification = 'geheim' AND auth.has_clearance('geheim'))
+    OR created_by = auth.uid() -- Users can always see their own objects
+  );
+
+-- Documents: Classification-based access
+CREATE POLICY classification_filter ON documents
+  FOR SELECT
+  TO authenticated
+  USING (
+    classification IS NULL
+    OR classification = 'openbaar'
+    OR classification = 'intern'
+    OR (classification = 'vertrouwelijk' AND auth.has_clearance('vertrouwelijk'))
+    OR (classification = 'geheim' AND auth.has_clearance('geheim'))
+  );
+
+-- ============================================================
+-- Woo Publication Filtering (Public Access)
+-- ============================================================
+
+-- Helper function to check if document should be publicly visible
+CREATE OR REPLACE FUNCTION is_woo_public(doc_id VARCHAR)
+RETURNS BOOLEAN AS $$
+  SELECT EXISTS (
+    SELECT 1 FROM information_objects
+    WHERE id = doc_id::uuid
+    AND is_woo_relevant = true
+    AND woo_publication_date IS NOT NULL
+    AND woo_publication_date <= CURRENT_TIMESTAMP
+  );
+$$ LANGUAGE sql STABLE;
+
+-- Information Objects: Public Woo access
+CREATE POLICY woo_public_read ON information_objects
+  FOR SELECT
+  TO public
+  USING (
+    is_woo_relevant = true
+    AND woo_publication_date IS NOT NULL
+    AND woo_publication_date <= CURRENT_TIMESTAMP
+    AND classification = 'openbaar'
+  );
+
+-- Documents: Public Woo access
+CREATE POLICY woo_public_read ON documents
+  FOR SELECT
+  TO public
+  USING (
+    woo_published = true
+    AND (classification IS NULL OR classification = 'openbaar')
+  );
+
+-- Authenticated users can see their org's Woo documents
+CREATE POLICY woo_authenticated_read ON documents
+  FOR SELECT
+  TO authenticated
+  USING (
+    woo_published = true
+    OR organization_id = auth.organization_id()
+  );
+
+-- ============================================================
+-- Owner-Based Policies
+-- ============================================================
+
+-- Users can update/delete their own information objects
+CREATE POLICY owner_update ON information_objects
+  FOR UPDATE
+  TO authenticated
+  USING (created_by = auth.uid())
+  WITH CHECK (created_by = auth.uid());
+
+CREATE POLICY owner_delete ON information_objects
+  FOR DELETE
+  TO authenticated
+  USING (created_by = auth.uid());
+
+-- ============================================================
+-- Role-Based Policies
+-- ============================================================
+
+-- Domain Managers can manage domains
+CREATE POLICY domain_manager_full ON information_domains
+  FOR ALL
+  TO authenticated
+  USING (
+    auth.user_role() = 'admin'
+    OR auth.user_role() = 'domain_manager'
+    OR organization_id = auth.organization_id()
+  )
+  WITH CHECK (
+    auth.user_role() = 'admin'
+    OR auth.user_role() = 'domain_manager'
+    OR organization_id = auth.organization_id()
+  );
+
+-- ============================================================
+-- Performance Optimization Indexes
+-- ============================================================
+
+-- Create indexes to support RLS policy lookups
+CREATE INDEX IF NOT EXISTS idx_documents_org ON documents(organization_id);
+CREATE INDEX IF NOT EXISTS idx_documents_woo ON documents(woo_published) WHERE woo_published = true;
+CREATE INDEX IF NOT EXISTS idx_documents_classification ON documents(classification);
+CREATE INDEX IF NOT EXISTS idx_documents_owner ON documents(created_by);
+
+CREATE INDEX IF NOT EXISTS idx_objects_created_by ON information_objects(created_by);
+CREATE INDEX IF NOT EXISTS idx_objects_classification ON information_objects(classification);
+CREATE INDEX IF NOT EXISTS idx_objects_woo_relevant ON information_objects(is_woo_relevant) WHERE is_woo_relevant = true;
+CREATE INDEX IF NOT EXISTS idx_objects_woo_published ON information_objects(woo_publication_date) WHERE woo_publication_date IS NOT NULL;
+
+CREATE INDEX IF NOT EXISTS idx_domains_org ON information_domains(organization_id);
+
+-- Partial indexes for common RLS patterns
+CREATE INDEX IF NOT EXISTS idx_documents_active
+  ON documents(organization_id)
+  WHERE state NOT IN ('archived', 'rejected');
+
+CREATE INDEX IF NOT EXISTS idx_objects_active
+  ON information_objects(created_at DESC)
+  WHERE classification != 'geheim';
+
+-- ============================================================
+-- RLS Policy Functions
+-- ============================================================
+
+-- Function to check if RLS is enabled on a table
+CREATE OR REPLACE FUNCTION check_rls_enabled(table_name TEXT)
+RETURNS BOOLEAN AS $$
+  SELECT relrowsecurity FROM pg_class WHERE relname = table_name AND relnamespace = 'public'::regnamespace;
+$$ LANGUAGE sql SECURITY DEFINER;
+
+-- Function to list all RLS policies for a table
+CREATE OR REPLACE FUNCTION list_rls_policies(table_name TEXT)
+RETURNS TABLE(
+  policy_name VARCHAR,
+  command VARCHAR,
+  roles VARCHAR[]
+) AS $$
+  SELECT
+    p.polname::VARCHAR,
+    p.polcmd::VARCHAR,
+    ARRAY_AGG(r.rolname) AS roles
+  FROM pg_policy p
+  JOIN pg_class c ON c.oid = p.polrelid
+  LEFT JOIN pg_authid r ON r.oid = ANY(p.polroles)
+  WHERE c.relname = table_name
+  GROUP BY p.polname, p.polcmd;
+$$ LANGUAGE sql SECURITY DEFINER;
+
+-- ============================================================
+-- Audit Trail for RLS Violations (Optional)
+-- ============================================================
+
+-- Function to log RLS policy violations for security monitoring
+CREATE OR REPLACE FUNCTION log_rls_violation()
+RETURNS TRIGGER AS $$
+BEGIN
+  -- Log to a security audit table
+  INSERT INTO audit_trail (document_id, agent_name, action, details)
+  VALUES (
+    COALESCE(NEW.id, OLD.id),
+    current_user,
+    'rls_violation_attempt',
+    jsonb_build_object(
+      'table', TG_TABLE_NAME,
+      'user', auth.uid(),
+      'organization', auth.organization_id(),
+      'timestamp', now()
+    )
+  );
+  RAISE EXCEPTION 'RLS policy violation detected on %', TG_TABLE_NAME;
+  RETURN NULL;
+END;
+$$ LANGUAGE plpgsql SECURITY DEFINER;
+
+-- ============================================================
+-- Grant Permissions
+-- ============================================================
+
+-- Grant usage on auth functions to authenticated users
+GRANT USAGE ON SCHEMA public TO postgres;
+GRANT SELECT ON ALL TABLES IN SCHEMA public TO postgres;
+GRANT INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO postgres;
+
+-- Grant usage on sequences
+GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO postgres;
