//! Storage abstraction layer for IOU-Modern document system.
//!
//! Provides a unified interface for S3/MinIO storage operations and
//! document metadata persistence.

pub mod config;
pub mod s3;
pub mod metadata;

pub use config::StorageConfig;
pub use s3::{S3Client, S3Error, StorageOperations};
pub use metadata::{MetadataStore, MetadataError};
