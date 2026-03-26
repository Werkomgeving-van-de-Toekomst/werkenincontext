//! Supabase Client Configuration
//!
//! Configuration and initialization for the Supabase client.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

/// Configuration for Supabase client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    /// Supabase project URL
    pub url: String,

    /// Supabase anon/public key
    pub key: String,

    /// Database connection string
    pub database_url: String,

    /// Service role key for admin operations
    pub service_role_key: Option<String>,

    /// JWT secret for token verification
    pub jwt_secret: Option<String>,

    /// Realtime WebSocket URL (derived from URL if not set)
    pub realtime_url: Option<String>,

    /// Storage URL (derived from URL if not set)
    pub storage_url: Option<String>,
}

impl SupabaseConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let url = env::var("SUPABASE_URL")
            .expect("SUPABASE_URL must be set");

        let key = env::var("SUPABASE_KEY")
            .or_else(|_| env::var("SUPABASE_ANON_KEY"))
            .expect("SUPABASE_KEY or SUPABASE_ANON_KEY must be set");

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let service_role_key = env::var("SUPABASE_SERVICE_ROLE_KEY").ok();

        let jwt_secret = env::var("SUPABASE_JWT_SECRET")
            .or_else(|_| env::var("SUPABASE_JWT"))
            .ok();

        let realtime_url = env::var("SUPABASE_REALTIME_URL").ok();

        let storage_url = env::var("SUPABASE_STORAGE_URL").ok();

        Ok(Self {
            url,
            key,
            database_url,
            service_role_key,
            jwt_secret,
            realtime_url,
            storage_url,
        })
    }

    /// Load from environment with fallback values
    pub fn from_env_with_defaults() -> Self {
        Self {
            url: env::var("SUPABASE_URL").unwrap_or_else(|_| "https://localhost:8000".to_string()),
            key: env::var("SUPABASE_KEY").unwrap_or_else(|_| "default-key".to_string()),
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localhost:5432/db".to_string()),
            service_role_key: env::var("SUPABASE_SERVICE_ROLE_KEY").ok(),
            jwt_secret: env::var("SUPABASE_JWT_SECRET").ok(),
            realtime_url: env::var("SUPABASE_REALTIME_URL").ok(),
            storage_url: env::var("SUPABASE_STORAGE_URL").ok(),
        }
    }

    /// Create configuration explicitly
    pub fn new(
        url: String,
        key: String,
        database_url: String,
    ) -> Self {
        Self {
            url,
            key,
            database_url,
            service_role_key: None,
            jwt_secret: None,
            realtime_url: None,
            storage_url: None,
        }
    }

    /// Get the realtime WebSocket URL
    pub fn get_realtime_url(&self) -> String {
        self.realtime_url.clone().unwrap_or_else(|| {
            format!("{}/realtime/v1/websocket", self.url.trim_end_matches('/'))
        })
    }

    /// Get the storage URL
    pub fn get_storage_url(&self) -> String {
        self.storage_url.clone().unwrap_or_else(|| {
            format!("{}/storage/v1", self.url.trim_end_matches('/'))
        })
    }

    /// Get the REST API URL
    pub fn get_rest_url(&self) -> String {
        format!("{}/rest/v1", self.url.trim_end_matches('/'))
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.url.is_empty() {
            return Err(anyhow::anyhow!("Supabase URL is required"));
        }

        if self.key.is_empty() {
            return Err(anyhow::anyhow!("Supabase key is required"));
        }

        if self.database_url.is_empty() {
            return Err(anyhow::anyhow!("Database URL is required"));
        }

        Ok(())
    }
}

/// Main Supabase client with all features
#[derive(Clone)]
pub struct SupabaseClient {
    /// Client configuration
    config: SupabaseConfig,

    /// Project URL
    pub url: String,

    /// API key
    pub key: String,
}

impl SupabaseClient {
    /// Create a new Supabase client
    pub fn new(config: SupabaseConfig) -> Result<Self> {
        config.validate()?;

        let url = config.url.clone();
        let key = config.key.clone();

        Ok(Self {
            config,
            url,
            key,
        })
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let config = SupabaseConfig::from_env()?;
        Self::new(config)
    }

    /// Get the client configuration
    pub fn config(&self) -> &SupabaseConfig {
        &self.config
    }

    /// Get the realtime URL
    pub fn realtime_url(&self) -> String {
        self.config.get_realtime_url()
    }

    /// Get the storage URL
    pub fn storage_url(&self) -> String {
        self.config.get_storage_url()
    }

    /// Get the REST API URL
    pub fn rest_url(&self) -> String {
        self.config.get_rest_url()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = SupabaseConfig::new(
            "https://test.supabase.co".to_string(),
            "test-key".to_string(),
            "postgresql://localhost:5432/db".to_string(),
        );

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_fails_empty_url() {
        let config = SupabaseConfig::new(
            "".to_string(),
            "test-key".to_string(),
            "postgresql://localhost:5432/db".to_string(),
        );

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_derived_urls() {
        let config = SupabaseConfig::new(
            "https://test.supabase.co".to_string(),
            "test-key".to_string(),
            "postgresql://localhost:5432/db".to_string(),
        );

        assert_eq!(
            config.get_realtime_url(),
            "https://test.supabase.co/realtime/v1/websocket"
        );
        assert_eq!(
            config.get_storage_url(),
            "https://test.supabase.co/storage/v1"
        );
        assert_eq!(
            config.get_rest_url(),
            "https://test.supabase.co/rest/v1"
        );
    }

    #[test]
    fn test_config_from_env_with_defaults() {
        // Clear any existing env vars for this test
        let config = SupabaseConfig::from_env_with_defaults();

        assert_eq!(config.url, "https://localhost:8000");
        assert_eq!(config.key, "default-key");
    }
}
