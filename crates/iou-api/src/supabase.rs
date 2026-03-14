//! Supabase database connection layer
//!
//! Provides PostgreSQL connection pool for self-hosted Supabase
//! as part of the hybrid DuckDB + Supabase architecture.

use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::Result;

/// Supabase database connection pool
#[derive(Clone)]
pub struct SupabasePool {
    pool: PgPool,
}

impl SupabasePool {
    /// Create a new Supabase connection pool from DATABASE_URL
    pub async fn new(database_url: &str) -> Result<Self> {
        // Get max connections from environment or use sensible default
        let max_connections = std::env::var("SUPABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| {
                // Use num_cpus::get() * 2 as a reasonable default, between 5 and 50
                let cpu_count = std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(4);
                (cpu_count * 2).clamp(5, 50) as u32
            });

        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to connect to Supabase at {}: {}",
                    database_url.split('@').last().unwrap_or(database_url),
                    e
                )
            })?;

        tracing::info!("Connected to Supabase PostgreSQL (max connections: {})", max_connections);

        Ok(Self { pool })
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
        // Note: In production, use sqlx-cli or a proper migration tool
        // This is a simplified version for development
        tracing::info!("Migrations should be run via sqlx-cli or external tool");
        Ok(())
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
}
