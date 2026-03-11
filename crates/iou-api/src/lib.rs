//! IOU-Modern API Library
//!
//! This library exposes the core types and modules for testing.

pub mod error;
pub mod websockets;

// Re-export commonly used types
pub use websockets::types::DocumentStatus;
pub use websockets::limiter::ConnectionLimiter;
