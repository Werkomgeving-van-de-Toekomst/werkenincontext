//! Storage abstraction layer for S3/MinIO integration
//!
//! This module provides a unified interface for document storage operations,
//! supporting S3-compatible backends including AWS S3 and MinIO.

pub mod s3;

pub use s3::{S3Client, S3Config, S3Error};
