//! Version storage service implementation
//!
//! Manages document version lifecycle including creation, storage,
//! compression, and restoration with full audit trail.

use crate::storage::S3Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{Read, Write};
use tokio::sync::RwLock;

/// Storage backend abstraction for testability
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    async fn upload(&self, key: &str, data: Vec<u8>, content_type: &str) -> Result<(), S3Error>;
    async fn download(&self, key: &str) -> Result<Vec<u8>, S3Error>;
    async fn exists(&self, key: &str) -> Result<bool, S3Error>;
}

/// Database backend abstraction for testability
#[async_trait::async_trait]
pub trait DatabaseBackend: Send + Sync {
    async fn create_version_record(&self, record: &VersionRecord) -> Result<VersionRecord, VersionError>;
    async fn get_current_version(&self, document_id: Uuid) -> Result<Option<VersionRecord>, VersionError>;
    async fn list_versions(&self, document_id: Uuid) -> Result<Vec<VersionRecord>, VersionError>;
    async fn get_version(&self, version_id: Uuid) -> Result<Option<VersionRecord>, VersionError>;
    async fn update_version_compression(&self, version_id: Uuid, is_compressed: bool) -> Result<(), VersionError>;
    async fn update_version_current(&self, version_id: Uuid, is_current: bool) -> Result<(), VersionError>;
    async fn get_next_version_number(&self, document_id: Uuid) -> Result<i32, VersionError>;
    async fn update_document_content(&self, document_id: Uuid, content: &str) -> Result<(), VersionError>;
}

/// Version record from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRecord {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version_number: i32,
    pub storage_key: String,
    pub format: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub change_summary: Option<String>,
    pub is_current: bool,
    pub is_compressed: bool,
    pub parent_version_id: Option<Uuid>,
    pub diff_summary: Option<serde_json::Value>,
    pub compliance_score: Option<f32>,
}

/// Version content with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionContent {
    pub version: VersionRecord,
    pub content: String,
}

/// Result of version restoration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub id: Uuid,
    pub document_id: Uuid,
    pub content: String,
    pub restored_from: Uuid,
    pub change_summary: String,
    pub created_at: DateTime<Utc>,
}

/// Version service errors
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("Storage error: {0}")]
    Storage(#[from] S3Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Version not found: {0}")]
    VersionNotFound(Uuid),

    #[error("Document not found: {0}")]
    DocumentNotFound(Uuid),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Decompression error: {0}")]
    Decompression(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Version storage service
pub struct VersionService {
    storage: Arc<dyn StorageBackend>,
    db: Arc<dyn DatabaseBackend>,
    full_versions_keep: i32,
}

impl VersionService {
    /// Create a new version service with storage and database backends
    pub fn new<S, D>(storage: S, db: D, full_versions_keep: i32) -> Self
    where
        S: StorageBackend + 'static,
        D: DatabaseBackend + 'static,
    {
        Self {
            storage: Arc::new(storage),
            db: Arc::new(db),
            full_versions_keep,
        }
    }

    /// Create version service for testing with mock storage
    pub fn new_with_storage<S>(storage: S, full_versions_keep: i32) -> Self
    where
        S: StorageBackend + 'static,
    {
        Self {
            storage: Arc::new(storage),
            db: Arc::new(MockDatabase::new()),
            full_versions_keep,
        }
    }

    /// Create a new version for a document
    pub async fn create_version(
        &self,
        document_id: Uuid,
        content: &str,
        changed_by: Uuid,
        change_summary: &str,
    ) -> Result<VersionRecord, VersionError> {
        // Get current version to set as parent and mark it as non-current
        let current_version = self
            .db
            .get_current_version(document_id)
            .await?;

        let parent_version_id = current_version.as_ref().map(|v| v.id);

        // Mark previous current version as non-current
        if let Some(ref prev) = current_version {
            let _ = self.db.update_version_current(prev.id, false).await;
        }

        // Get next version number
        let version_number = self.db.get_next_version_number(document_id).await?;

        // Generate storage key
        let storage_key = Self::generate_storage_key(document_id, version_number);

        // Upload content to storage
        self.storage
            .upload(&storage_key, content.as_bytes().to_vec(), "text/markdown")
            .await?;

        // Create version record
        let record = VersionRecord {
            id: Uuid::new_v4(),
            document_id,
            version_number,
            storage_key: storage_key.clone(),
            format: "markdown".to_string(),
            created_at: Utc::now(),
            created_by: changed_by.to_string(),
            change_summary: Some(change_summary.to_string()),
            is_current: true,
            is_compressed: false,
            parent_version_id: parent_version_id,
            diff_summary: None, // TODO: Generate using diff service
            compliance_score: None,
        };

        let saved = self.db.create_version_record(&record).await?;

        // Check if we need to compress old versions
        if version_number > self.full_versions_keep {
            let _ = self.compress_old_versions(document_id).await;
        }

        Ok(saved)
    }

    /// List all versions for a document, ordered by created_at DESC
    pub async fn list_versions(
        &self,
        document_id: Uuid,
    ) -> Result<Vec<VersionRecord>, VersionError> {
        self.db.list_versions(document_id).await
    }

    /// Get a specific version by ID with content
    pub async fn get_version(&self, version_id: Uuid) -> Result<VersionContent, VersionError> {
        let record = self
            .db
            .get_version(version_id)
            .await?
            .ok_or(VersionError::VersionNotFound(version_id))?;

        let content = self.get_version_content(version_id).await?;

        Ok(VersionContent {
            version: record,
            content,
        })
    }

    /// Get version content (handles decompression)
    pub async fn get_version_content(&self, version_id: Uuid) -> Result<String, VersionError> {
        let record = self
            .db
            .get_version(version_id)
            .await?
            .ok_or(VersionError::VersionNotFound(version_id))?;

        // For compressed versions, the data is stored at {storage_key}.gz
        let storage_key = if record.is_compressed {
            format!("{}.gz", record.storage_key)
        } else {
            record.storage_key.clone()
        };

        let data = self.storage.download(&storage_key).await?;

        let content = if record.is_compressed {
            Self::decompress(&data)?
        } else {
            String::from_utf8(data)
                .map_err(|e| VersionError::Decompression(format!("UTF-8 error: {}", e)))?
        };

        Ok(content)
    }

    /// Restore a previous version
    pub async fn restore_version(
        &self,
        document_id: Uuid,
        version_id: Uuid,
        restored_by: Uuid,
    ) -> Result<RestoreResult, VersionError> {
        // Validate user has access to document
        if restored_by == Uuid::nil() {
            return Err(VersionError::Unauthorized("Invalid user ID".to_string()));
        }

        // Get version to restore
        let version_to_restore = self
            .db
            .get_version(version_id)
            .await?
            .ok_or(VersionError::VersionNotFound(version_id))?;

        if version_to_restore.document_id != document_id {
            return Err(VersionError::InvalidInput(
                "Version does not belong to document".to_string(),
            ));
        }

        // Fetch content
        let content = self.get_version_content(version_id).await?;

        // Update document with restored content
        self.db
            .update_document_content(document_id, &content)
            .await?;

        // Create new version recording the restore
        let new_version = self
            .create_version(
                document_id,
                &content,
                restored_by,
                &format!("Restored from version {}", version_to_restore.version_number),
            )
            .await?;

        // TODO: Create audit trail entry
        // audit_log.log_restore(document_id, version_id, restored_by).await?;

        Ok(RestoreResult {
            id: new_version.id,
            document_id,
            content,
            restored_from: version_id,
            change_summary: new_version.change_summary.unwrap_or_default(),
            created_at: new_version.created_at,
        })
    }

    /// Compress old versions beyond the full_versions_keep threshold
    async fn compress_old_versions(&self, document_id: Uuid) -> Result<(), VersionError> {
        let versions = self.list_versions(document_id).await?;

        // Versions are ordered DESC (newest first)
        // Compress versions beyond the keep threshold (skip newest N)
        for (i, version) in versions.iter().enumerate() {
            if i >= self.full_versions_keep as usize && !version.is_compressed {
                let _ = self.compress_version(&version.storage_key).await;
                let _ = self
                    .db
                    .update_version_compression(version.id, true)
                    .await;
            }
        }

        Ok(())
    }

    /// Compress a single version's stored content
    async fn compress_version(&self, storage_key: &str) -> Result<(), VersionError> {
        let data = self.storage.download(storage_key).await?;
        let compressed = Self::compress(&data)?;

        // Upload compressed version back
        self.storage
            .upload(&format!("{}.gz", storage_key), compressed, "application/gzip")
            .await?;

        Ok(())
    }

    /// Generate S3 storage key for a version
    fn generate_storage_key(document_id: Uuid, version_number: i32) -> String {
        format!("documents/{}/versions/v{}", document_id, version_number)
    }

    /// Compress data using gzip
    fn compress(data: &[u8]) -> Result<Vec<u8>, VersionError> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(data)
            .map_err(|e| VersionError::Compression(e.to_string()))?;
        encoder
            .finish()
            .map_err(|e| VersionError::Compression(e.to_string()))
    }

    /// Decompress gzip data
    fn decompress(data: &[u8]) -> Result<String, VersionError> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| VersionError::Decompression(e.to_string()))?;
        String::from_utf8(decompressed)
            .map_err(|e| VersionError::Decompression(format!("UTF-8 error: {}", e)))
    }
}

// Mock database for testing - replace with real implementation
struct MockDatabase {
    versions: Arc<RwLock<Vec<VersionRecord>>>,
}

impl MockDatabase {
    fn new() -> Self {
        Self {
            versions: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl DatabaseBackend for MockDatabase {
    async fn create_version_record(
        &self,
        record: &VersionRecord,
    ) -> Result<VersionRecord, VersionError> {
        let mut versions = self.versions.write().await;
        versions.push(record.clone());
        Ok(record.clone())
    }

    async fn get_current_version(
        &self,
        document_id: Uuid,
    ) -> Result<Option<VersionRecord>, VersionError> {
        let versions = self.versions.read().await;
        Ok(versions
            .iter()
            .filter(|v| v.document_id == document_id && v.is_current)
            .max_by_key(|v| v.version_number)
            .cloned())
    }

    async fn list_versions(&self, document_id: Uuid) -> Result<Vec<VersionRecord>, VersionError> {
        let versions = self.versions.read().await;
        let mut result: Vec<_> = versions
            .iter()
            .filter(|v| v.document_id == document_id)
            .cloned()
            .collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at)); // DESC by created_at
        Ok(result)
    }

    async fn get_version(&self, version_id: Uuid) -> Result<Option<VersionRecord>, VersionError> {
        let versions = self.versions.read().await;
        Ok(versions.iter().find(|v| v.id == version_id).cloned())
    }

    async fn update_version_compression(
        &self,
        version_id: Uuid,
        is_compressed: bool,
    ) -> Result<(), VersionError> {
        let mut versions = self.versions.write().await;
        if let Some(v) = versions.iter_mut().find(|v| v.id == version_id) {
            v.is_compressed = is_compressed;
        }
        Ok(())
    }

    async fn update_version_current(
        &self,
        version_id: Uuid,
        is_current: bool,
    ) -> Result<(), VersionError> {
        let mut versions = self.versions.write().await;
        if let Some(v) = versions.iter_mut().find(|v| v.id == version_id) {
            v.is_current = is_current;
        }
        Ok(())
    }

    async fn get_next_version_number(&self, document_id: Uuid) -> Result<i32, VersionError> {
        let versions = self.versions.read().await;
        let max = versions
            .iter()
            .filter(|v| v.document_id == document_id)
            .map(|v| v.version_number)
            .max()
            .unwrap_or(0);
        Ok(max + 1)
    }

    async fn update_document_content(
        &self,
        _document_id: Uuid,
        _content: &str,
    ) -> Result<(), VersionError> {
        // Stub implementation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_storage_key() {
        let document_id = Uuid::new_v4();
        let key = VersionService::generate_storage_key(document_id, 1);
        assert!(key.starts_with("documents/"));
        assert!(key.contains(&document_id.to_string()));
        assert!(key.ends_with("/v1"));
    }
}
