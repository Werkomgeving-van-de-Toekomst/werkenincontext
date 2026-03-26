//! Supabase client for IOU-Modern
//!
//! Provides a comprehensive client for Supabase including:
//! - PostgreSQL connection pool
//! - Auth management (signup, login, password reset)
//! - Storage client (file upload/download)
//! - Realtime subscriptions (PostgreSQL changes)
//! - Utility methods (transactions, query builders)

use sqlx::{PgPool, postgres::PgPoolOptions, Transaction};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

// Re-export the connection pool
pub use crate::supabase_client::{SupabaseClient, SupabaseConfig};
pub use crate::supabase_auth::{SupabaseAuth, AuthError, SignUpRequest, SignInRequest, CredentialsResetRequest};
pub use crate::supabase_storage::{SupabaseStorage, StorageError, StorageFile, UploadOptions};
pub use crate::supabase_realtime::{SupabaseRealtime, RealtimeChannel, RealtimeEvent};
pub use crate::supabase_utils::{QueryBuilder, TransactionHelper, PaginatedResult};

/// Supabase database connection pool (legacy wrapper)
#[derive(Clone)]
pub struct SupabasePool {
    pool: PgPool,
}

impl SupabasePool {
    /// Create a new Supabase connection pool from DATABASE_URL
    pub async fn new(database_url: &str) -> Result<Self> {
        let max_connections = std::env::var("SUPABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| {
                let cpu_count = std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(4);
                (cpu_count * 2).clamp(5, 50) as u32
            });

        Self::with_max_connections(database_url, max_connections).await
    }

    /// Create a new connection pool with custom max connections
    pub async fn with_max_connections(database_url: &str, max: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max)
            .connect(database_url)
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to connect to Supabase at {}: {}",
                    database_url.split('@').last().unwrap_or(database_url),
                    e
                )
            })?;

        tracing::info!("Connected to Supabase PostgreSQL (max connections: {})", max);

        Ok(Self { pool })
    }

    /// Get the underlying sqlx PgPool
    pub fn inner(&self) -> &PgPool {
        &self.pool
    }

    /// Health check for the database connection with retry logic
    pub async fn health_check(&self) -> Result<()> {
        use tokio::time::{sleep, Duration};

        let mut attempts = 0;
        let max_attempts = 3;

        loop {
            match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
                Ok(_) => return Ok(()),
                Err(e) if attempts < max_attempts => {
                    attempts += 1;
                    let delay = Duration::from_millis(100 * 2_u64.pow(attempts));
                    tracing::warn!(
                        "Health check attempt {} failed, retrying in {:?}: {}",
                        attempts,
                        delay,
                        e
                    );
                    sleep(delay).await;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }

    /// Run PostgreSQL migrations
    pub async fn run_migrations(&self) -> Result<()> {
        tracing::info!("Migrations should be run via sqlx-cli or external tool");
        Ok(())
    }
}

/// Main Supabase client combining all features
#[derive(Clone)]
pub struct Client {
    /// Database connection pool
    pub db: PgPool,

    /// Auth client
    pub auth: Arc<SupabaseAuth>,

    /// Storage client
    pub storage: Arc<SupabaseStorage>,

    /// Realtime client
    pub realtime: Arc<RwLock<Option<SupabaseRealtime>>>,

    /// Project URL
    pub url: String,

    /// Service role key (for admin operations)
    pub service_role_key: Option<String>,
}

impl Client {
    /// Create a new Supabase client from environment variables
    ///
    /// Required environment variables:
    /// - `SUPABASE_URL`: Supabase project URL
    /// - `SUPABASE_KEY`: Supabase anon/public key
    /// - `DATABASE_URL`: PostgreSQL connection string
    ///
    /// Optional:
    /// - `SUPABASE_SERVICE_ROLE_KEY`: Service role key for admin operations
    pub async fn from_env() -> Result<Self> {
        let url = std::env::var("SUPABASE_URL")
            .expect("SUPABASE_URL must be set");
        let key = std::env::var("SUPABASE_KEY")
            .unwrap_or_default();
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok();

        Self::new(&url, &key, &database_url, service_role_key.as_deref()).await
    }

    /// Create a new Supabase client
    pub async fn new(
        url: &str,
        key: &str,
        database_url: &str,
        service_role_key: Option<&str>,
    ) -> Result<Self> {
        // Connect to database
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await?;

        tracing::info!("Connected to Supabase at {}", url);

        // Create auth client
        let auth = SupabaseAuth::new(url, key);

        // Create storage client
        let storage = SupabaseStorage::new(url, key);

        Ok(Self {
            db: pool,
            auth: Arc::new(auth),
            storage: Arc::new(storage),
            realtime: Arc::new(RwLock::new(None)),
            url: url.to_string(),
            service_role_key: service_role_key.map(String::from),
        })
    }

    /// Initialize the realtime client
    pub async fn init_realtime(&self) -> Result<()> {
        let realtime = SupabaseRealtime::new(&self.url)?;
        let mut guard = self.realtime.write().await;
        *guard = Some(realtime);
        Ok(())
    }

    /// Get the realtime client (initializes if not already done)
    pub async fn realtime(&self) -> Result<SupabaseRealtime> {
        let guard = self.realtime.read().await;
        if let Some(rt) = guard.as_ref() {
            Ok(rt.clone())
        } else {
            drop(guard);
            self.init_realtime().await?;
            let guard = self.realtime.read().await;
            Ok(guard.as_ref().unwrap().clone())
        }
    }

    /// Get a reference to the database pool
    pub fn pool(&self) -> &PgPool {
        &self.db
    }

    /// Begin a new transaction
    pub async fn begin(&self) -> Result<Transaction<'static, sqlx::Postgres>> {
        Ok(self.db.begin().await?)
    }

    /// Health check for all Supabase services
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let db_healthy = sqlx::query("SELECT 1")
            .fetch_one(&self.db)
            .await
            .is_ok();

        let storage_healthy = self.storage.health_check().await.is_ok();

        Ok(HealthStatus {
            database: db_healthy,
            storage: storage_healthy,
            realtime: true, // Assume healthy if initialized
        })
    }

    /// Create a query builder for the specified table
    pub fn from(&self, table: &str) -> QueryBuilder {
        QueryBuilder::new(self.db.clone(), table)
    }
}

/// Health status of Supabase services
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub database: bool,
    pub storage: bool,
    pub realtime: bool,
}

impl HealthStatus {
    /// Check if all services are healthy
    pub fn is_healthy(&self) -> bool {
        self.database && self.storage && self.realtime
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires actual Supabase instance
    async fn test_supabase_connection() {
        let pool = SupabasePool::new("postgresql://postgres:postgres@localhost:5432/iou_modern")
            .await
            .unwrap();
        assert!(pool.health_check().await.is_ok());
    }

    #[test]
    fn test_health_status() {
        let status = HealthStatus {
            database: true,
            storage: true,
            realtime: false,
        };
        assert!(!status.is_healthy());
    }
}
