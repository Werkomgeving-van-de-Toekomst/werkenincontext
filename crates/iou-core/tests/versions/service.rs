//! Version storage service tests

use iou_core::versions::{VersionService, VersionError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Test helpers
fn random_document_id() -> Uuid {
    Uuid::new_v4()
}

fn random_user_id() -> Uuid {
    Uuid::new_v4()
}

/// Mock storage backend for testing
#[derive(Clone)]
pub struct MockStorage {
    storage: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn contains_key(&self, key: &str) -> bool {
        self.storage.read().await.contains_key(key)
    }

    pub async fn keys(&self) -> Vec<String> {
        self.storage.read().await.keys().cloned().collect()
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl iou_core::versions::StorageBackend for MockStorage {
    async fn upload(&self, key: &str, data: Vec<u8>, _content_type: &str) -> Result<(), iou_core::storage::S3Error> {
        self.storage.write().await.insert(key.to_string(), data);
        Ok(())
    }

    async fn download(&self, key: &str) -> Result<Vec<u8>, iou_core::storage::S3Error> {
        self.storage
            .read()
            .await
            .get(key)
            .cloned()
            .ok_or_else(|| iou_core::storage::S3Error::NotFound(format!("Key not found: {}", key)))
    }

    async fn exists(&self, key: &str) -> Result<bool, iou_core::storage::S3Error> {
        Ok(self.storage.read().await.contains_key(key))
    }
}

#[tokio::test]
async fn create_version_stores_document_content_in_storage() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage.clone(), 5);
    let document_id = random_document_id();
    let user_id = random_user_id();
    let content = "Document content here";

    let version = service
        .create_version(document_id, content, user_id, "Initial version")
        .await
        .unwrap();

    assert_eq!(version.document_id, document_id);
    // Check that content was stored
    assert!(mock_storage.contains_key(&version.storage_key).await);
}

#[tokio::test]
async fn create_version_creates_version_record_with_metadata() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 5);
    let document_id = random_document_id();
    let user_id = random_user_id();

    let version = service
        .create_version(document_id, "Content here", user_id, "Summary of changes")
        .await
        .unwrap();

    assert_eq!(version.change_summary, Some("Summary of changes".to_string()));
    assert_eq!(version.created_by, user_id.to_string());
    assert!(version.created_at.signed_duration_since(chrono::Utc::now()).num_seconds().abs() < 5);
}

#[tokio::test]
async fn create_version_sets_parent_version_id_to_previous_current_version() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 5);
    let document_id = random_document_id();
    let user_id = random_user_id();

    // Create first version
    let v1 = service
        .create_version(document_id, "First version", user_id, "v1")
        .await
        .unwrap();

    // Create second version
    let v2 = service
        .create_version(document_id, "Second version", user_id, "v2")
        .await
        .unwrap();

    assert_eq!(v2.parent_version_id, Some(v1.id));
}

#[tokio::test]
async fn create_version_increments_version_number() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 5);
    let document_id = random_document_id();
    let user_id = random_user_id();

    let v1 = service
        .create_version(document_id, "v1", user_id, "First")
        .await
        .unwrap();
    let v2 = service
        .create_version(document_id, "v2", user_id, "Second")
        .await
        .unwrap();
    let v3 = service
        .create_version(document_id, "v3", user_id, "Third")
        .await
        .unwrap();

    assert_eq!(v1.version_number, 1);
    assert_eq!(v2.version_number, 2);
    assert_eq!(v3.version_number, 3);
}

#[tokio::test]
async fn create_version_compresses_old_versions_when_threshold_exceeded() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage.clone(), 3);
    let document_id = random_document_id();
    let user_id = random_user_id();

    // Create versions 1-4 (4 > threshold of 3, so v1 should be compressed)
    for i in 1..=4 {
        service
            .create_version(
                document_id,
                &format!("Version {}", i),
                user_id,
                &format!("v{}", i),
            )
            .await
            .unwrap();
    }

    let versions = service.list_versions(document_id).await.unwrap();
    let compressed_count = versions.iter().filter(|v| v.is_compressed).count();

    assert!(compressed_count >= 1, "At least one version should be compressed");
}

#[tokio::test]
async fn list_versions_returns_versions_ordered_by_created_at_desc() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 5);
    let document_id = random_document_id();
    let user_id = random_user_id();

    let v1 = service
        .create_version(document_id, "v1", user_id, "First")
        .await
        .unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let v2 = service
        .create_version(document_id, "v2", user_id, "Second")
        .await
        .unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let v3 = service
        .create_version(document_id, "v3", user_id, "Third")
        .await
        .unwrap();

    let versions = service.list_versions(document_id).await.unwrap();

    assert_eq!(versions.len(), 3);
    assert_eq!(versions[0].id, v3.id); // Most recent first
    assert_eq!(versions[1].id, v2.id);
    assert_eq!(versions[2].id, v1.id);
}

#[tokio::test]
async fn list_versions_includes_version_number_created_by_change_summary() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 5);
    let document_id = random_document_id();
    let user_id = random_user_id();

    service
        .create_version(document_id, "content", user_id, "Test summary")
        .await
        .unwrap();

    let versions = service.list_versions(document_id).await.unwrap();
    let version = &versions[0];

    assert_eq!(version.version_number, 1);
    assert_eq!(version.created_by, user_id.to_string());
    assert_eq!(version.change_summary, Some("Test summary".to_string()));
}

#[tokio::test]
async fn restore_version_fetches_version_content_from_storage() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage.clone(), 5);
    let document_id = random_document_id();
    let user_id = random_user_id();
    let original_content = "Original document content";

    let v1 = service
        .create_version(document_id, original_content, user_id, "v1")
        .await
        .unwrap();

    let restored = service
        .restore_version(document_id, v1.id, user_id)
        .await
        .unwrap();
    assert_eq!(restored.content, original_content);
}

#[tokio::test]
async fn restore_version_updates_document_with_restored_content() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage.clone(), 5);
    let document_id = random_document_id();
    let user_id = random_user_id();

    let v1 = service
        .create_version(document_id, "Version 1", user_id, "v1")
        .await
        .unwrap();
    service
        .create_version(document_id, "Version 2", user_id, "v2")
        .await
        .unwrap();

    // Restore v1 (should create new version with v1's content)
    let restored = service
        .restore_version(document_id, v1.id, user_id)
        .await
        .unwrap();

    assert_eq!(restored.content, "Version 1");
    assert_ne!(restored.id, v1.id); // New version created
}

#[tokio::test]
async fn restore_version_creates_new_version_recording_the_restore() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 5);
    let document_id = random_document_id();
    let user_id = random_user_id();

    let v1 = service
        .create_version(document_id, "v1 content", user_id, "v1")
        .await
        .unwrap();

    let restored = service
        .restore_version(document_id, v1.id, user_id)
        .await
        .unwrap();

    assert_eq!(restored.content, "v1 content");
    assert!(restored.change_summary.contains("Restored from version"));
    assert_eq!(restored.restored_from, v1.id);
}

#[tokio::test]
async fn restore_version_requires_authentication() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 5);
    let document_id = random_document_id();
    let user_id = random_user_id();

    let v1 = service
        .create_version(document_id, "content", user_id, "v1")
        .await
        .unwrap();

    // No user provided (nil UUID) - should fail
    let result = service.restore_version(document_id, v1.id, Uuid::nil()).await;

    assert!(result.is_err());
    assert!(matches!(result, Err(VersionError::Unauthorized(_))));
}

#[tokio::test]
async fn compress_old_versions_compresses_versions_beyond_full_versions_keep() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage.clone(), 3);
    let document_id = random_document_id();
    let user_id = random_user_id();

    // Create 5 versions, threshold is 3, so v1 and v2 should be compressed
    for i in 1..=5 {
        service
            .create_version(
                document_id,
                &format!("Content for version {}", i),
                user_id,
                &format!("v{}", i),
            )
            .await
            .unwrap();
    }

    let versions = service.list_versions(document_id).await.unwrap();
    let compressed: Vec<_> = versions.iter().filter(|v| v.is_compressed).collect();

    assert!(
        compressed.len() >= 2,
        "At least versions 1 and 2 should be compressed"
    );
}

#[tokio::test]
async fn compress_old_versions_sets_is_compressed_flag() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 2);
    let document_id = random_document_id();
    let user_id = random_user_id();

    service
        .create_version(document_id, "v1", user_id, "v1")
        .await
        .unwrap();
    service
        .create_version(document_id, "v2", user_id, "v2")
        .await
        .unwrap();
    service
        .create_version(document_id, "v3", user_id, "v3")
        .await
        .unwrap();

    let versions = service.list_versions(document_id).await.unwrap();
    let v1 = versions.iter().find(|v| v.version_number == 1).unwrap();

    assert!(v1.is_compressed, "v1 should be marked as compressed");
}

#[tokio::test]
async fn compressed_versions_can_be_decompressed_for_content_retrieval() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage.clone(), 1);
    let document_id = random_document_id();
    let user_id = random_user_id();

    let original_content = "This is the original content that will be compressed";
    service
        .create_version(document_id, original_content, user_id, "v1")
        .await
        .unwrap();
    service
        .create_version(document_id, "New content", user_id, "v2")
        .await
        .unwrap();

    let versions = service.list_versions(document_id).await.unwrap();
    let v1 = versions.iter().find(|v| v.version_number == 1).unwrap();

    assert!(v1.is_compressed);

    // Fetch content (should auto-decompress)
    let fetched_content = service.get_version_content(v1.id).await.unwrap();
    assert_eq!(fetched_content, original_content);
}

#[tokio::test]
async fn get_version_returns_version_with_content() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 5);
    let document_id = random_document_id();
    let user_id = random_user_id();

    let content = "Test content for version";
    let version = service
        .create_version(document_id, content, user_id, "Test version")
        .await
        .unwrap();

    let version_content = service.get_version(version.id).await.unwrap();

    assert_eq!(version_content.version.id, version.id);
    assert_eq!(version_content.content, content);
}

#[tokio::test]
async fn get_version_content_decompresses_compressed_versions() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage.clone(), 1);
    let document_id = random_document_id();
    let user_id = random_user_id();

    let original_content = "Content that will be compressed";
    let v1 = service
        .create_version(document_id, original_content, user_id, "v1")
        .await
        .unwrap();

    // Create another version to trigger compression
    service
        .create_version(document_id, "v2", user_id, "v2")
        .await
        .unwrap();

    // Fetch v1 content - should be decompressed
    let content = service.get_version_content(v1.id).await.unwrap();
    assert_eq!(content, original_content);
}

#[tokio::test]
async fn get_version_returns_error_for_nonexistent_version() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 5);
    let nonexistent_id = Uuid::new_v4();

    let result = service.get_version(nonexistent_id).await;

    assert!(matches!(result, Err(VersionError::VersionNotFound(_))));
}

#[tokio::test]
async fn restore_version_returns_error_for_nonexistent_version() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 5);
    let document_id = random_document_id();
    let user_id = random_user_id();
    let nonexistent_id = Uuid::new_v4();

    let result = service.restore_version(document_id, nonexistent_id, user_id).await;

    assert!(matches!(result, Err(VersionError::VersionNotFound(_))));
}

#[tokio::test]
async fn restore_version_returns_error_for_version_belonging_to_different_document() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 5);
    let document_id = random_document_id();
    let other_document_id = random_document_id();
    let user_id = random_user_id();

    let v1 = service
        .create_version(document_id, "content", user_id, "v1")
        .await
        .unwrap();

    // Try to restore v1 but claim it belongs to a different document
    let result = service.restore_version(other_document_id, v1.id, user_id).await;

    assert!(matches!(result, Err(VersionError::InvalidInput(_))));
}

#[tokio::test]
async fn create_version_sets_storage_key_format() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 5);
    let document_id = random_document_id();
    let user_id = random_user_id();

    let version = service
        .create_version(document_id, "content", user_id, "v1")
        .await
        .unwrap();

    // Storage key should be in format: documents/{document_id}/versions/v{version_number}
    assert!(version.storage_key.starts_with("documents/"));
    assert!(version.storage_key.contains(&format!("{}/", document_id)));
    assert!(version.storage_key.ends_with("/v1"));
}

#[tokio::test]
async fn version_record_contains_all_required_fields() {
    let mock_storage = MockStorage::new();
    let service = VersionService::new_with_storage(mock_storage, 5);
    let document_id = random_document_id();
    let user_id = random_user_id();

    let version = service
        .create_version(
            document_id,
            "Test content",
            user_id,
            "Test summary",
        )
        .await
        .unwrap();

    // Verify all fields are populated
    assert_ne!(version.id, Uuid::nil());
    assert_eq!(version.document_id, document_id);
    assert_eq!(version.version_number, 1);
    assert!(!version.storage_key.is_empty());
    assert_eq!(version.format, "markdown");
    assert_ne!(version.created_at, chrono::DateTime::<chrono::Utc>::MIN_UTC);
    assert_eq!(version.created_by, user_id.to_string());
    assert_eq!(version.change_summary, Some("Test summary".to_string()));
    assert!(version.is_current);
    assert!(!version.is_compressed); // First version not compressed
    assert_eq!(version.parent_version_id, None); // First version has no parent
}
