//! Version storage service for document history tracking
//!
//! This module provides:
//! - Version creation with S3 storage
//! - Version listing with metadata
//! - Version restoration with audit trail
//! - Automatic compression of old versions
//! - Parent-child version tracking

pub mod service;

pub use service::{
    VersionService, VersionRecord, VersionContent, RestoreResult, VersionError,
    StorageBackend, DatabaseBackend,
};
