//! WebSocket support for real-time document status updates
//!
//! This module provides WebSocket handlers for broadcasting document
//! workflow status updates to connected clients.

pub mod documents;
pub mod limiter;
pub mod types;

pub use documents::ws_document_handler;
pub use limiter::ConnectionLimiter;
pub use types::DocumentStatus;
