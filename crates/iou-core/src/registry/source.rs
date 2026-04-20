//! Data Source Registry
//!
//! Manages all data sources that feed into the central registry.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

/// Data source in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    /// Unique source ID
    pub id: Uuid,

    /// Source name
    pub name: String,

    /// Source type
    pub source_type: SourceType,

    /// Connection details (encrypted)
    pub connection: SourceConnection,

    /// Owner user ID
    pub owner_id: Uuid,

    /// Organization ID
    pub organization_id: Uuid,

    /// Current status
    pub status: SourceStatus,

    /// Last successful sync
    pub last_synced_at: Option<DateTime<Utc>>,

    /// Next scheduled sync
    pub next_sync_at: Option<DateTime<Utc>>,

    /// Entity count in this source
    pub entity_count: usize,

    /// Whether this source is active
    pub is_active: bool,

    /// When source was registered
    pub created_at: DateTime<Utc>,

    /// When source was last updated
    pub updated_at: DateTime<Utc>,
}

impl DataSource {
    /// Create a new data source
    pub fn new(
        name: String,
        source_type: SourceType,
        connection: SourceConnection,
        owner_id: Uuid,
        organization_id: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            source_type,
            connection,
            owner_id,
            organization_id,
            status: SourceStatus::Pending,
            last_synced_at: None,
            next_sync_at: None,
            entity_count: 0,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark source as connected
    pub fn mark_connected(&mut self) {
        self.status = SourceStatus::Connected;
        self.updated_at = Utc::now();
    }

    /// Mark source as failed
    pub fn mark_failed(&mut self, error: String) {
        self.status = SourceStatus::Failed { error };
        self.updated_at = Utc::now();
    }

    /// Update sync timestamps
    pub fn mark_synced(&mut self) {
        self.last_synced_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Schedule next sync
    pub fn schedule_next_sync(&mut self, interval_seconds: u64) {
        let from = self.last_synced_at.unwrap_or(self.created_at);
        self.next_sync_at = Some(from + chrono::Duration::seconds(interval_seconds as i64));
    }

    /// Check if source is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, SourceStatus::Connected | SourceStatus::Syncing)
            && self.is_active
    }

    /// Increment entity count
    pub fn increment_entity_count(&mut self, count: usize) {
        self.entity_count += count;
        self.updated_at = Utc::now();
    }
}

/// Types of data sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Relational database (PostgreSQL, MySQL, etc.)
    Database,

    /// REST API
    Api,

    /// SOAP/WSDL web service
    Soap,

    /// File-based (CSV, JSON, XML)
    File,

    /// Legacy/mainframe system
    Legacy,

    /// Message queue/event stream
    MessageQueue,

    /// S3/storage bucket
    ObjectStorage,

    /// GraphQL API
    GraphQL,
}

/// Connection details for a data source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SourceConnection {
    /// Database connection string
    Database {
        host: String,
        port: u16,
        database: String,
        username: String,
        // Password stored securely elsewhere
        password_ref: String,
    },

    /// API connection
    Api {
        base_url: String,
        api_key_ref: Option<String>,
        auth_type: ApiAuthType,
    },

    /// File connection
    File {
        path: String,
        format: FileFormat,
    },

    /// Legacy system connection
    Legacy {
        endpoint: String,
        protocol: String,
        credentials_ref: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ApiAuthType {
    None,
    Basic,
    Bearer,
    ApiKey,
    OAuth2,
    MutualTls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    Csv,
    Json,
    Xml,
    Excel,
    Parquet,
}

/// Source sync status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum SourceStatus {
    /// Source not yet connected
    Pending,

    /// Source successfully connected
    Connected,

    /// Currently syncing
    Syncing { started_at: DateTime<Utc> },

    /// Sync failed
    Failed { error: String },

    /// Source disabled
    Disabled,

    /// Source unreachable
    Unreachable { since: DateTime<Utc> },
}

/// Source sync result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSyncResult {
    /// Source ID
    pub source_id: Uuid,

    /// Sync status
    pub status: SourceSyncStatus,

    /// Entities synced
    pub entities_synced: usize,

    /// Entities created
    pub entities_created: usize,

    /// Entities updated
    pub entities_updated: usize,

    /// Entities failed
    pub entities_failed: usize,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Sync timestamp
    pub synced_at: DateTime<Utc>,

    /// Error message if failed
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceSyncStatus {
    Success,
    PartialSuccess,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_source_creation() {
        let connection = SourceConnection::Api {
            base_url: "https://api.example.com".to_string(),
            api_key_ref: None,
            auth_type: ApiAuthType::Bearer,
        };

        let source = DataSource::new(
            "Test API".to_string(),
            SourceType::Api,
            connection,
            Uuid::new_v4(),
            Uuid::new_v4(),
        );

        assert_eq!(source.name, "Test API");
        assert_eq!(source.source_type, SourceType::Api);
        assert_eq!(source.status, SourceStatus::Pending);
    }

    #[test]
    fn test_data_source_mark_connected() {
        let mut source = DataSource::new(
            "Test".to_string(),
            SourceType::Database,
            SourceConnection::Database {
                host: "localhost".to_string(),
                port: 5432,
                database: "test".to_string(),
                username: "user".to_string(),
                password_ref: "secret_ref".to_string(),
            },
            Uuid::new_v4(),
            Uuid::new_v4(),
        );

        source.mark_connected();

        assert!(matches!(source.status, SourceStatus::Connected));
        assert!(source.is_healthy());
    }

    #[test]
    fn test_data_source_mark_failed() {
        let mut source = DataSource::new(
            "Test".to_string(),
            SourceType::Api,
            SourceConnection::Api {
                base_url: "https://api.example.com".to_string(),
                api_key_ref: None,
                auth_type: ApiAuthType::Bearer,
            },
            Uuid::new_v4(),
            Uuid::new_v4(),
        );

        source.mark_failed("Connection refused".to_string());

        assert!(matches!(source.status, SourceStatus::Failed { .. }));
        assert!(!source.is_healthy());
    }

    #[test]
    fn test_data_source_schedule_next_sync() {
        let mut source = DataSource::new(
            "Test".to_string(),
            SourceType::Api,
            SourceConnection::Api {
                base_url: "https://api.example.com".to_string(),
                api_key_ref: None,
                auth_type: ApiAuthType::Bearer,
            },
            Uuid::new_v4(),
            Uuid::new_v4(),
        );

        source.mark_synced();
        source.schedule_next_sync(3600); // 1 hour

        assert!(source.next_sync_at.is_some());
    }

    #[test]
    fn test_data_source_entity_count() {
        let mut source = DataSource::new(
            "Test".to_string(),
            SourceType::File,
            SourceConnection::File {
                path: "/data/test.csv".to_string(),
                format: FileFormat::Csv,
            },
            Uuid::new_v4(),
            Uuid::new_v4(),
        );

        assert_eq!(source.entity_count, 0);

        source.increment_entity_count(100);
        assert_eq!(source.entity_count, 100);

        source.increment_entity_count(50);
        assert_eq!(source.entity_count, 150);
    }
}
