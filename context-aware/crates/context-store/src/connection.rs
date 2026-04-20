// =============================================================================
// Connection Pool Management for ArangoDB
// =============================================================================

use arangors::Client;
use mobc::{Connection, Manager};
use std::time::Duration;

use crate::{StoreConfig, StoreError, StoreResult};

/// ArangoDB connection manager for mobc pool
pub struct ArangoConnectionManager {
    client: Client,
    config: StoreConfig,
}

impl ArangoConnectionManager {
    pub fn new(config: StoreConfig) -> StoreResult<Self> {
        let client = Client::new_with_auth(&config.url, &config.username, &config.password)
            .map_err(|e| StoreError::InvalidQuery(e.to_string()))?;

        Ok(Self { client, config })
    }
}

#[async_trait::async_trait]
impl Manager for ArangoConnectionManager {
    type Connection = arangors::Database<arangors::client::reqwest::ReqwestClient>;
    type Error = StoreError;

    async fn create(&self) -> Result<Self::Connection, Self::Error> {
        self.client
            .db(&self.config.database)
            .await
            .map_err(StoreError::Database)
    }

    async fn validate(&self, conn: &mut Self::Connection) -> bool {
        // Simple validation: check if database is accessible
        conn.ping()
            .await
            .map(|_| true)
            .unwrap_or(false)
    }
}

/// Connection pool for ArangoDB
pub struct ConnectionPool {
    pool: mobc::Pool<ArangoConnectionManager>,
}

impl ConnectionPool {
    pub fn new(config: StoreConfig) -> StoreResult<Self> {
        let manager = ArangoConnectionManager::new(config.clone())?;
        let pool = mobc::Pool::builder()
            .max_open(config.pool_size as u64)
            .max_idle(5)
            .get_timeout(Some(Duration::from_secs(30)))
            .build(manager);

        Ok(Self { pool })
    }

    pub async fn connection(
        &self,
    ) -> Result<
        Connection<ArangoConnectionManager>,
        mobc::Error<StoreError>,
    > {
        self.pool.get().await
    }

    pub async fn close(&self) -> Result<(), mobc::Error<StoreError>> {
        self.pool.close().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running ArangoDB instance
    async fn test_connection_pool() {
        let config = StoreConfig::default();
        let pool = ConnectionPool::new(config).unwrap();

        let conn = pool.connection().await;
        assert!(conn.is_ok());
    }
}
