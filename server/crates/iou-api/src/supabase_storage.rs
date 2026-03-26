//! Supabase Storage Client
//!
//! Provides file storage operations using Supabase Storage.

use anyhow::Result;
use bytes::Bytes;
use chrono::{DateTime, Utc, Duration};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supabase Storage client
#[derive(Clone)]
pub struct SupabaseStorage {
    /// Supabase project URL
    base_url: String,

    /// API key
    api_key: String,

    /// HTTP client
    http_client: Client,
}

impl SupabaseStorage {
    /// Create a new Supabase Storage client
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            http_client: Client::new(),
        }
    }

    /// Get the storage endpoint URL
    fn storage_url(&self, path: &str) -> String {
        format!("{}/storage/v1/{}", self.base_url, path.trim_start_matches('/'))
    }

    /// Get default headers for requests
    fn headers(&self, bearer_token: Option<&str>) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("apikey", self.api_key.parse().unwrap());

        if let Some(token) = bearer_token {
            headers.insert(
                "Authorization",
                format!("Bearer {}", token).parse().unwrap(),
            );
        }

        headers
    }

    /// Get a reference to a specific bucket
    pub fn bucket(&self, name: &str) -> StorageBucket {
        StorageBucket {
            client: self.clone(),
            name: name.to_string(),
        }
    }

    /// Create a new storage bucket
    pub async fn create_bucket(
        &self,
        name: &str,
        public: bool,
        service_role_key: &str,
    ) -> Result<(), StorageError> {
        let url = self.storage_url("bucket");

        let payload = serde_json::json!({
            "id": name,
            "name": name,
            "public": public,
        });

        let response = self
            .http_client
            .post(&url)
            .header("apikey", service_role_key)
            .header("Authorization", format!("Bearer {}", service_role_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(StorageError::ApiError("Failed to create bucket".to_string()))
        }
    }

    /// List all buckets
    pub async fn list_buckets(&self, bearer_token: Option<&str>) -> Result<Vec<StorageBucketInfo>, StorageError> {
        let url = self.storage_url("bucket");

        let response = self
            .http_client
            .get(&url)
            .headers(self.headers(bearer_token))
            .send()
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;

        if response.status().is_success() {
            response
                .json()
                .await
                .map_err(|e| StorageError::ConnectionError(e.to_string()))
        } else {
            Err(StorageError::ApiError("Failed to list buckets".to_string()))
        }
    }

    /// Health check for storage
    pub async fn health_check(&self) -> Result<()> {
        let url = self.storage_url("bucket");

        let response = self
            .http_client
            .get(&url)
            .header("apikey", &self.api_key)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Connection error: {}", e))?;

        if response.status().is_success() || response.status().as_u16() == 401 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Storage service unavailable"))
        }
    }
}

/// Reference to a specific storage bucket
#[derive(Clone)]
pub struct StorageBucket {
    client: SupabaseStorage,
    name: String,
}

impl StorageBucket {
    /// Get the bucket name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Upload a file to the bucket
    pub async fn upload(
        &self,
        path: &str,
        data: Bytes,
        options: UploadOptions,
        bearer_token: Option<&str>,
    ) -> Result<StorageFile, StorageError> {
        let url = self.client.storage_url(&format!("object/{}/{}", self.name, path));

        let mut headers = self.client.headers(bearer_token);
        if let Some(content_type) = options.content_type {
            headers.insert("Content-Type", content_type.parse().unwrap());
        }

        let mut query = Vec::new();
        if options.upsert {
            query.push(("upsert", "true"));
        }

        let mut request = self.client.http_client.post(&url).headers(headers);

        if !query.is_empty() {
            request = request.query(&query);
        }

        let response = request
            .body(data)
            .send()
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;

        if response.status().is_success() {
            let file_info: StorageFileInner = response
                .json()
                .await
                .map_err(|e| StorageError::ConnectionError(e.to_string()))?;

            Ok(StorageFile {
                id: file_info.id,
                name: file_info.name,
                bucket_id: file_info.bucket_id,
                path: format!("{}/{}", self.name, path),
                size: file_info.metadata.size.unwrap_or(0),
                content_type: file_info.metadata.mimetype,
                created_at: file_info.created_at,
                updated_at: file_info.updated_at,
            })
        } else {
            Err(StorageError::ApiError("Upload failed".to_string()))
        }
    }

    /// Download a file from the bucket
    pub async fn download(&self, path: &str, bearer_token: Option<&str>) -> Result<Bytes, StorageError> {
        let url = self.client.storage_url(&format!("object/{}/{}", self.name, path));

        let response = self
            .client
            .http_client
            .get(&url)
            .headers(self.client.headers(bearer_token))
            .send()
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;

        if response.status().is_success() {
            response
                .bytes()
                .await
                .map_err(|e| StorageError::ConnectionError(e.to_string()))
        } else {
            Err(StorageError::NotFound(format!("File not found: {}", path)))
        }
    }

    /// List files in the bucket
    pub async fn list(
        &self,
        prefix: Option<&str>,
        bearer_token: Option<&str>,
    ) -> Result<Vec<StorageFile>, StorageError> {
        let url = self.client.storage_url(&format!("object/{}", self.name));

        let mut query = Vec::new();
        if let Some(p) = prefix {
            query.push(("prefix", p));
        }
        query.push(("limit", "100"));

        let response = self
            .client
            .http_client
            .get(&url)
            .headers(self.client.headers(bearer_token))
            .query(&query)
            .send()
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;

        if response.status().is_success() {
            let files: Vec<StorageFileInner> = response
                .json()
                .await
                .map_err(|e| StorageError::ConnectionError(e.to_string()))?;

            Ok(files
                .into_iter()
                .map(|f| {
                    let name = f.name.clone();
                    StorageFile {
                        id: f.id,
                        name: name.clone(),
                        bucket_id: f.bucket_id,
                        path: format!("{}/{}", self.name, name),
                        size: f.metadata.size.unwrap_or(0),
                        content_type: f.metadata.mimetype,
                        created_at: f.created_at,
                        updated_at: f.updated_at,
                    }
                })
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// Delete a file from the bucket
    pub async fn delete(&self, path: &str, bearer_token: Option<&str>) -> Result<(), StorageError> {
        let url = self.client.storage_url(&format!("object/{}/{}", self.name, path));

        let response = self
            .client
            .http_client
            .delete(&url)
            .headers(self.client.headers(bearer_token))
            .send()
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(StorageError::ApiError("Failed to delete file".to_string()))
        }
    }

    /// Generate a signed URL for temporary access
    pub fn create_signed_url(&self, path: &str, expires_in: Duration) -> String {
        let expires_at = Utc::now() + expires_in;
        format!(
            "{}/storage/v1/object/sign/{}/{}?expires_at={}",
            self.client.base_url, self.name, path, expires_at.timestamp()
        )
    }

    /// Get public URL for a file (bucket must be public)
    pub fn public_url(&self, path: &str) -> String {
        format!(
            "{}/storage/v1/object/public/{}/{}",
            self.client.base_url, self.name, path
        )
    }
}

/// Options for file uploads
#[derive(Debug, Clone, Default)]
pub struct UploadOptions {
    /// Content type (MIME type) of the file
    pub content_type: Option<String>,
    /// If true, overwrites existing files
    pub upsert: bool,
}

/// Information about a stored file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageFile {
    pub id: Uuid,
    pub name: String,
    pub bucket_id: String,
    pub path: String,
    pub size: u64,
    pub content_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StorageFileInner {
    id: Uuid,
    name: String,
    bucket_id: String,
    metadata: FileMetadata,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileMetadata {
    size: Option<u64>,
    mimetype: Option<String>,
}

/// Information about a storage bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageBucketInfo {
    pub id: String,
    pub name: String,
    pub public: bool,
    pub file_size_limit: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Errors that can occur during storage operations
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Quota exceeded")]
    QuotaExceeded,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_client_creation() {
        let storage = SupabaseStorage::new("https://test.supabase.co/", "test-key");
        assert_eq!(storage.base_url, "https://test.supabase.co");
        assert_eq!(storage.api_key, "test-key");
    }
}
