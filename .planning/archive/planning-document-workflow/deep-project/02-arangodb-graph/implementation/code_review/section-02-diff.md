diff --git a/crates/iou-core/Cargo.toml b/crates/iou-core/Cargo.toml
index f268422..169b0ba 100644
--- a/crates/iou-core/Cargo.toml
+++ b/crates/iou-core/Cargo.toml
@@ -46,6 +46,7 @@ sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "chrono", "uu
 arangors = { version = "0.6", features = ["reqwest_async", "rocksdb"] }
 
 # Connection pooling for ArangoDB
+mobc = "0.9"
 mobc-arangors = "0.2"
 
 # Diff generation for document version comparison
diff --git a/crates/iou-core/src/graphrag/connection.rs b/crates/iou-core/src/graphrag/connection.rs
new file mode 100644
index 0000000..17d277b
--- /dev/null
+++ b/crates/iou-core/src/graphrag/connection.rs
@@ -0,0 +1,310 @@
+//! Connection pool management for ArangoDB
+//!
+//! This module provides connection pooling using the `mobc` crate,
+//! supporting both JWT and basic authentication.
+
+use std::time::Duration;
+
+use arangors::connection::Connection;
+use mobc::{async_trait, Manager, Pool};
+
+use crate::graphrag::error::StoreError;
+
+/// Configuration for ArangoDB connection
+#[derive(Debug, Clone)]
+pub struct ArangoConfig {
+    /// ArangoDB server URL (e.g., "http://localhost:8529")
+    pub connection_url: String,
+    /// Username for authentication
+    pub username: String,
+    /// Password or JWT token for authentication
+    pub credential: String,
+    /// Database name to connect to
+    pub database: String,
+    /// Maximum number of open connections in the pool
+    pub pool_max_open: u64,
+    /// Minimum number of idle connections in the pool
+    pub pool_min_idle: u64,
+    /// Timeout in seconds for getting a connection from the pool
+    pub pool_timeout_seconds: u64,
+}
+
+impl ArangoConfig {
+    /// Default connection pool values
+    pub const DEFAULT_MAX_OPEN: u64 = 10;
+    pub const DEFAULT_MIN_IDLE: u64 = 2;
+    pub const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
+
+    /// Create a new ArangoConfig with the given parameters
+    pub fn new(
+        connection_url: impl Into<String>,
+        username: impl Into<String>,
+        credential: impl Into<String>,
+        database: impl Into<String>,
+    ) -> Self {
+        Self {
+            connection_url: connection_url.into(),
+            username: username.into(),
+            credential: credential.into(),
+            database: database.into(),
+            pool_max_open: Self::DEFAULT_MAX_OPEN,
+            pool_min_idle: Self::DEFAULT_MIN_IDLE,
+            pool_timeout_seconds: Self::DEFAULT_TIMEOUT_SECONDS,
+        }
+    }
+
+    /// Load configuration from environment variables
+    ///
+    /// Environment variables:
+    /// - `ARANGODB_URL`: Connection URL (default: "http://localhost:8529")
+    /// - `ARANGODB_USERNAME`: Username (default: "root")
+    /// - `ARANGODB_PASSWORD`: Password (default: "")
+    /// - `ARANGODB_DATABASE`: Database name (default: "_system")
+    pub fn from_env() -> Result<Self, std::env::VarError> {
+        Ok(Self {
+            connection_url: std::env::var("ARANGODB_URL")
+                .unwrap_or_else(|_| "http://localhost:8529".to_string()),
+            username: std::env::var("ARANGODB_USERNAME")
+                .unwrap_or_else(|_| "root".to_string()),
+            credential: std::env::var("ARANGODB_PASSWORD")
+                .unwrap_or_else(|_| String::new()),
+            database: std::env::var("ARANGODB_DATABASE")
+                .unwrap_or_else(|_| "_system".to_string()),
+            pool_max_open: Self::DEFAULT_MAX_OPEN,
+            pool_min_idle: Self::DEFAULT_MIN_IDLE,
+            pool_timeout_seconds: Self::DEFAULT_TIMEOUT_SECONDS,
+        })
+    }
+
+    /// Detect if the credential is a JWT token
+    ///
+    /// JWT tokens are typically long base64 strings (3 segments separated by dots)
+    fn is_jwt_token(&self) -> bool {
+        // JWT tokens have 3 parts separated by dots and are typically long
+        let parts: Vec<&str> = self.credential.split('.').collect();
+        parts.len() == 3 && self.credential.len() > 100
+    }
+}
+
+/// Connection manager for ArangoDB using mobc
+///
+/// This manager handles the creation and validation of ArangoDB connections.
+#[derive(Clone, Debug)]
+pub struct ArangorsConnectionManager {
+    connection_url: String,
+    username: String,
+    credential: String,
+    database: String,
+    use_jwt: bool,
+}
+
+impl ArangorsConnectionManager {
+    /// Create a new connection manager from config
+    pub fn new(config: &ArangoConfig) -> Self {
+        Self {
+            connection_url: config.connection_url.clone(),
+            username: config.username.clone(),
+            credential: config.credential.clone(),
+            database: config.database.clone(),
+            use_jwt: config.is_jwt_token(),
+        }
+    }
+
+    /// Establish a new connection to ArangoDB
+    ///
+    /// Handles JWT or basic authentication based on credential format.
+    pub async fn connect(&self) -> Result<Connection, arangors::ClientError> {
+        let conn = if self.use_jwt {
+            Connection::establish_jwt(&self.connection_url, &self.username, &self.credential).await?
+        } else {
+            Connection::establish_basic_auth(&self.connection_url, &self.username, &self.credential).await?
+        };
+
+        // Verify the database exists and is accessible
+        conn.db(&self.database).await?;
+
+        Ok(conn)
+    }
+}
+
+#[async_trait]
+impl Manager for ArangorsConnectionManager {
+    type Connection = Connection;
+    type Error = arangors::ClientError;
+
+    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
+        self.connect().await
+    }
+
+    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
+        // Verify connection is still valid by checking database access
+        conn.db(&self.database).await?;
+        Ok(conn)
+    }
+}
+
+/// Create a new connection pool for ArangoDB
+///
+/// # Arguments
+/// * `config` - ArangoDB connection configuration
+///
+/// # Returns
+/// A configured mobc Pool ready for use
+///
+/// # Example
+/// ```no_run
+/// use iou_core::graphrag::connection::{create_pool, ArangoConfig};
+///
+/// # #[tokio::main]
+/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
+/// let config = ArangoConfig::new(
+///     "http://localhost:8529",
+///     "username",
+///     "password",
+///     "my_db"
+/// );
+/// let pool = create_pool(&config).await?;
+/// # Ok(())
+/// # }
+/// ```
+pub async fn create_pool(config: &ArangoConfig) -> Result<Pool<ArangorsConnectionManager>, StoreError> {
+    let manager = ArangorsConnectionManager::new(config);
+
+    let pool = Pool::builder()
+        .max_open(config.pool_max_open)
+        .max_idle(config.pool_min_idle)
+        .get_timeout(Some(Duration::from_secs(config.pool_timeout_seconds)))
+        .build(manager);
+
+    Ok(pool)
+}
+
+/// Test connection health by executing a simple query
+///
+/// # Arguments
+/// * `pool` - The connection pool to test
+///
+/// # Returns
+/// `Ok(())` if the connection is healthy, `Err` otherwise
+pub async fn test_connection(pool: &Pool<ArangorsConnectionManager>) -> Result<(), StoreError> {
+    let _conn = pool.get().await?;
+    // If we got here, the connection works
+    Ok(())
+}
+
+/// Get the ArangoDB server version
+///
+/// # Arguments
+/// * `pool` - The connection pool to use
+///
+/// # Returns
+/// The server version string
+///
+/// # Note
+/// This is a simplified version that returns a placeholder.
+/// Full implementation will query the ArangoDB server.
+pub async fn get_database_version(_pool: &Pool<ArangorsConnectionManager>) -> Result<String, StoreError> {
+    // TODO: Implement actual version query
+    // This requires complex type handling with the arangors response types
+    Ok("ArangoDB (version TBD)".to_string())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_config_new() {
+        let config = ArangoConfig::new("http://localhost:8529", "user", "pass", "db");
+        assert_eq!(config.connection_url, "http://localhost:8529");
+        assert_eq!(config.username, "user");
+        assert_eq!(config.credential, "pass");
+        assert_eq!(config.database, "db");
+        assert_eq!(config.pool_max_open, ArangoConfig::DEFAULT_MAX_OPEN);
+        assert_eq!(config.pool_min_idle, ArangoConfig::DEFAULT_MIN_IDLE);
+        assert_eq!(config.pool_timeout_seconds, ArangoConfig::DEFAULT_TIMEOUT_SECONDS);
+    }
+
+    #[test]
+    fn test_config_defaults() {
+        let config = ArangoConfig::new("url", "u", "p", "d");
+        assert_eq!(config.pool_max_open, 10);
+        assert_eq!(config.pool_min_idle, 2);
+        assert_eq!(config.pool_timeout_seconds, 30);
+    }
+
+    #[test]
+    fn test_is_jwt_token() {
+        // Password is not JWT
+        let config = ArangoConfig::new("url", "u", "password", "d");
+        assert!(!config.is_jwt_token());
+
+        // Short token is not JWT
+        let config = ArangoConfig::new("url", "u", "abc.def", "d");
+        assert!(!config.is_jwt_token());
+
+        // JWT-like token with 3 parts and sufficient length
+        // Using a placeholder token with sufficient length
+        let long_segment = "a".repeat(50);
+        let jwt_token = format!("{}.{}.{}", long_segment, long_segment, long_segment);
+        let config = ArangoConfig::new("url", "u", &jwt_token, "d");
+        assert!(config.is_jwt_token());
+    }
+
+    #[test]
+    fn test_manager_new() {
+        let config = ArangoConfig::new("http://localhost:8529", "user", "pass", "test_db");
+        let manager = ArangorsConnectionManager::new(&config);
+
+        assert_eq!(manager.connection_url, "http://localhost:8529");
+        assert_eq!(manager.username, "user");
+        assert_eq!(manager.credential, "pass");
+        assert_eq!(manager.database, "test_db");
+        assert!(!manager.use_jwt); // password is not JWT
+    }
+
+    #[test]
+    fn test_manager_new_with_jwt() {
+        // Create a JWT-like token with 3 parts and sufficient length
+        let long_segment = "a".repeat(50);
+        let jwt_token = format!("{}.{}.{}", long_segment, long_segment, long_segment);
+        let config = ArangoConfig::new("http://localhost:8529", "user", &jwt_token, "test_db");
+        let manager = ArangorsConnectionManager::new(&config);
+
+        assert!(manager.use_jwt);
+    }
+
+    #[test]
+    fn test_arango_config_from_env_defaults() {
+        // Clear env vars for test
+        // SAFETY: remove_var is safe in tests with controlled env vars
+        unsafe {
+            std::env::remove_var("ARANGODB_URL");
+            std::env::remove_var("ARANGODB_USERNAME");
+            std::env::remove_var("ARANGODB_PASSWORD");
+            std::env::remove_var("ARANGODB_DATABASE");
+        }
+
+        let config = ArangoConfig::from_env().unwrap();
+
+        assert_eq!(config.connection_url, "http://localhost:8529");
+        assert_eq!(config.username, "root");
+        assert_eq!(config.credential, "");
+        assert_eq!(config.database, "_system");
+    }
+
+    #[test]
+    fn test_arango_config_clone() {
+        let config = ArangoConfig::new("url", "u", "p", "d");
+        let cloned = config.clone();
+        assert_eq!(config.connection_url, cloned.connection_url);
+    }
+
+    #[test]
+    fn test_connection_manager_clone() {
+        let config = ArangoConfig::new("url", "u", "p", "d");
+        let manager = ArangorsConnectionManager::new(&config);
+        let cloned = manager.clone();
+        assert_eq!(manager.connection_url, cloned.connection_url);
+    }
+}
diff --git a/crates/iou-core/src/graphrag/error.rs b/crates/iou-core/src/graphrag/error.rs
index c07415a..b5d445f 100644
--- a/crates/iou-core/src/graphrag/error.rs
+++ b/crates/iou-core/src/graphrag/error.rs
@@ -77,6 +77,17 @@ impl From<arangors::ClientError> for StoreError {
     }
 }
 
+impl From<mobc::Error<arangors::ClientError>> for StoreError {
+    fn from(err: mobc::Error<arangors::ClientError>) -> Self {
+        match err {
+            mobc::Error::Inner(e) => e.into(),
+            mobc::Error::Timeout => StoreError::Connection("Connection pool timeout".to_string()),
+            mobc::Error::BadConn => StoreError::Connection("Bad connection".to_string()),
+            mobc::Error::PoolClosed => StoreError::Connection("Connection pool closed".to_string()),
+        }
+    }
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
diff --git a/crates/iou-core/src/graphrag/mod.rs b/crates/iou-core/src/graphrag/mod.rs
index 99fb74d..1c8c64c 100644
--- a/crates/iou-core/src/graphrag/mod.rs
+++ b/crates/iou-core/src/graphrag/mod.rs
@@ -5,6 +5,7 @@
 //! - Error types for graph store operations
 //! - ArangoDB-based persistence layer (TODO)
 
+pub mod connection;
 pub mod error;
 pub mod types;
 
