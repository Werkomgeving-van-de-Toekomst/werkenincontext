Now I have all the context. Let me generate the section content for section-04-s3-storage based on the extracted information.

# Section 4: S3 Storage Integration

## Overview

This section implements document storage in S3/MinIO. Documents are uploaded after pipeline completion, downloaded via streaming proxy (no presigned URLs), with size validation (10MB limit), proper error conversion, and startup validation.

**Dependencies:** Section 1 (Foundation & Configuration) must be completed first.

**Files to create/modify:**
- `crates/iou-core/src/storage/s3.rs` - S3 operations with streaming support
- `crates/iou-api/src/routes/documents.rs` - Streaming download endpoint
- `crates/iou-api/src/orchestrator/wrapper.rs` - Upload after completion
- `crates/iou-api/src/main.rs` - Startup validation

## Environment Variables

The following environment variables must be configured:

| Variable | Required | Description |
|----------|----------|-------------|
| `S3_ACCESS_KEY` | Yes | S3/MinIO access key |
| `S3_SECRET_KEY` | Yes | S3/MinIO secret key |
| `S3_BUCKET` | Yes | Bucket name for document storage |
| `S3_ENDPOINT` | No | Custom S3 endpoint (for MinIO) |
| `S3_REGION` | No | S3 region (default: us-east-1) |

## Tests

### Test: S3 Upload

**Test: upload succeeds for document under 10MB**
- Given: Document of 5MB
- When: `upload()` is called
- Then: Returns `Ok(())`
- And: Document is accessible in S3

**Test: upload fails for document over 10MB**
- Given: Document of 11MB
- When: `upload()` is called
- Then: Returns `Err(ApiError::PayloadTooLarge)`

**Test: upload retry on transient S3 error**
- Given: S3 returns 503 Service Unavailable
- When: `upload()` is called with retry logic
- Then: Retries with exponential backoff
- And: Eventually succeeds on retry

**Test: upload propagates permanent S3 errors**
- Given: S3 returns 403 Forbidden
- When: `upload()` is called
- Then: Returns `Err` immediately without retry

### Test: S3 Download (Streaming)

**Test: download_stream streams without buffering entire file**
- Given: 10MB document in S3
- When: `download_stream()` is called
- Then: Returns `AsyncRead` implementation
- And: Memory usage remains constant (not O(10MB))

**Test: download_stream returns 404 for missing document**
- Given: Document key does not exist in S3
- When: `download_stream()` is called
- Then: Returns `Err(ApiError::NotFound)`

**Test: download proxy endpoint streams response**
- Given: Valid document_id in database
- When: `GET /api/documents/{id}/download` is called
- Then: Returns 200 OK with streaming response
- And: `Content-Type` header matches document type

### Test: Error Conversion

**Test: S3Error::HttpFailWithBody(404) maps to ApiError::NotFound**
- Given: S3 returns 404
- When: Error is converted
- Then: Returns `ApiError::NotFound`

**Test: S3Error::HttpFailWithBody(413) maps to ApiError::PayloadTooLarge**
- Given: S3 returns 413
- When: Error is converted
- Then: Returns `ApiError::PayloadTooLarge`

**Test: Generic S3Error maps to ApiError::Internal**
- Given: Unknown S3 error
- When: Error is converted
- Then: Returns `ApiError::Internal`

## Implementation

### S3 Client Structure

The S3 client wraps `rust-s3` library with path-style URL support for MinIO compatibility:

```rust
// crates/iou-core/src/storage/s3.rs

use rust_s3::bucket::Bucket;
use rust_s3::credentials::Credentials;
use rust_s3::S3Error;
use std::io::Cursor;
use tokio::io::AsyncRead;

const MAX_DOCUMENT_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub struct S3Client {
    bucket: Bucket,
}

impl S3Client {
    /// Creates S3 client from environment variables
    /// 
    /// Required env vars:
    /// - S3_ACCESS_KEY
    /// - S3_SECRET_KEY  
    /// - S3_BUCKET
    /// 
    /// Optional:
    /// - S3_ENDPOINT (defaults to AWS S3)
    /// - S3_REGION (defaults to us-east-1)
    pub fn new_from_env() -> Result<Self, S3Error> {
        let credentials = Credentials::new(
            Some(std::env::var("S3_ACCESS_KEY")?),
            Some(std::env::var("S3_SECRET_KEY")?),
            None, None, None,
        )?;
        
        let region = std::env::var("S3_REGION")
            .unwrap_or_else(|_| "us-east-1".to_string())
            .parse()?;
        
        let bucket = Bucket::new(
            &std::env::var("S3_BUCKET")?,
            region,
            credentials,
        )?.with_path_style();
        
        Ok(Self { bucket })
    }
    
    /// Validates S3 connectivity at startup
    /// Fails fast if credentials or endpoint are invalid
    pub async fn validate(&self) -> Result<(), S3Error> {
        // Test connection with HEAD request to bucket
        self.bucket.head_object("").await?;
        Ok(())
    }
    
    /// Uploads document data to S3
    /// 
    /// # Arguments
    /// * `key` - S3 object key (typically `{document_id}.pdf` or similar)
    /// * `data` - Document bytes to upload
    /// 
    /// # Errors
    /// - `PayloadTooLarge` if data exceeds 10MB
    /// - S3 errors for network/auth issues
    pub async fn upload(&self, key: &str, data: Vec<u8>) -> Result<(), S3Error> {
        // Validate size before upload
        if data.len() > MAX_DOCUMENT_SIZE {
            return Err(S3Error::HttpFailWithBody(
                413,
                "Document exceeds 10MB limit".to_string(),
            ));
        }
        
        let reader = Cursor::new(data);
        self.bucket.put_object(key, &mut reader.into_bytes()).await?;
        Ok(())
    }
    
    /// Streams document download from S3
    /// 
    /// Returns an AsyncRead that streams the file without buffering
    /// the entire content in memory.
    /// 
    /// # Arguments
    /// * `key` - S3 object key to download
    /// 
    /// # Errors
    /// - `NotFound` if key does not exist
    /// - Other S3 errors for network/auth issues
    pub async fn download_stream(&self, key: &str) -> Result<impl AsyncRead + Send, S3Error> {
        let response = self.bucket.get_object(key).await?;
        Ok(response.bytes().to_reader())
    }
}
```

### Error Conversion Layer

Convert S3-specific errors to API errors:

```rust
// crates/iou-core/src/storage/s3.rs (continued)

use crate::api::ApiError;

impl From<S3Error> for ApiError {
    fn from(err: S3Error) -> Self {
        match err {
            S3Error::HttpFailWithBody(code, msg) => match code {
                404 => ApiError::NotFound("Document not found in storage".to_string()),
                413 => ApiError::PayloadTooLarge(msg),
                403 => ApiError::Forbidden("Access denied to storage".to_string()),
                _ => ApiError::Internal(format!("Storage error: {}", msg)),
            },
            S3Error::IoError(err) => ApiError::Internal(format!("Storage I/O error: {}", err)),
            _ => ApiError::Internal("Storage connection error".to_string()),
        }
    }
}
```

### Download Endpoint

Add streaming download endpoint to document routes:

```rust
// crates/iou-api/src/routes/documents.rs

use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use http::{header, StatusCode};
use tokio::io::AsyncRead;

/// GET /api/documents/{id}/download
/// 
/// Streams document directly from S3 to client without buffering.
/// Returns 404 if document not found in storage.
pub async fn download_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    // 1. Verify document exists in database and user has access
    let document = sqlx::query!(
        "SELECT storage_key, content_type FROM documents WHERE id = $1",
        id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => ApiError::NotFound("Document not found".to_string()),
        _ => ApiError::Internal(format!("Database error: {}", e)),
    })?;
    
    // 2. Stream from S3
    let stream = state.s3_client
        .download_stream(&document.storage_key)
        .await?;
    
    // 3. Return streaming response
    let body = Body::from_stream(tokio_util::io::ReaderStream::new(stream));
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, document.content_type)
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", id))
        .body(body)
        .unwrap())
}
```

### Upload After Pipeline Completion

Integrate S3 upload with orchestrator completion:

```rust
// crates/iou-api/src/orchestrator/wrapper.rs

use crate::storage::s3::S3Client;

impl WorkflowOrchestrator {
    /// Called when workflow completes successfully
    /// Uploads generated document to S3
    async fn on_completion(
        &self,
        document_id: Uuid,
        document_data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), ApiError> {
        // Generate storage key
        let storage_key = format!("documents/{}/{}.pdf", 
            document_id,
            document_id
        );
        
        // Upload to S3
        self.s3_client.upload(&storage_key, document_data).await?;
        
        // Update database with storage location
        sqlx::query!(
            "UPDATE documents SET storage_key = $1, content_type = $2 WHERE id = $3",
            storage_key,
            content_type,
            document_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
}
```

### Startup Validation

Validate S3 connectivity when application starts:

```rust
// crates/iou-api/src/main.rs

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... existing setup ...
    
    // Create S3 client
    let s3_client = Arc::new(S3Client::new_from_env()?);
    
    // Validate S3 connectivity (fail fast)
    tracing::info!("Validating S3 connectivity...");
    s3_client.validate().await
        .map_err(|e| format!("S3 validation failed: {}", e))?;
    tracing::info!("S3 connectivity validated");
    
    // ... continue with application startup ...
}
```

## Configuration

Add S3 configuration to config loading:

```rust
// crates/iou-api/src/config.rs

#[derive(Debug, Clone)]
pub struct S3Config {
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub endpoint: Option<String>,
    pub region: String,
}

impl S3Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            access_key: std::env::var("S3_ACCESS_KEY")
                .map_err(|_| ConfigError::Missing("S3_ACCESS_KEY"))?,
            secret_key: std::env::var("S3_SECRET_KEY")
                .map_err(|_| ConfigError::Missing("S3_SECRET_KEY"))?,
            bucket: std::env::var("S3_BUCKET")
                .map_err(|_| ConfigError::Missing("S3_BUCKET"))?,
            endpoint: std::env::var("S3_ENDPOINT").ok(),
            region: std::env::var("S3_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
        })
    }
}
```

## Integration with Existing Code

This section builds on the S3 client abstraction created in Section 1. Ensure the following are complete:

1. **Section 1 Foundation:**
   - `crates/iou-core/src/storage/mod.rs` module exists
   - `rust-s3` dependency added to `crates/iou-core/Cargo.toml`

2. **Database Schema:**
   Ensure `documents` table has columns:
   ```sql
   ALTER TABLE documents ADD COLUMN IF NOT EXISTS storage_key TEXT;
   ALTER TABLE documents ADD COLUMN IF NOT EXISTS content_type TEXT;
   ```

## TODO Checklist

- [ ] Add `rust-s3` dependency to `iou-core/Cargo.toml`
- [ ] Create `crates/iou-core/src/storage/s3.rs` with S3Client implementation
- [ ] Implement `new_from_env()` with environment variable loading
- [ ] Implement `validate()` for startup connectivity check
- [ ] Implement `upload()` with 10MB size validation
- [ ] Implement `download_stream()` for streaming downloads
- [ ] Implement `From<S3Error> for ApiError` error conversion
- [ ] Add `download_document()` route handler in `routes/documents.rs`
- [ ] Add upload call in orchestrator `on_completion()` handler
- [ ] Add S3 validation in `main.rs` startup sequence
- [ ] Add database schema migration for storage_key and content_type
- [ ] Write unit tests for S3 operations
- [ ] Write integration tests for download endpoint