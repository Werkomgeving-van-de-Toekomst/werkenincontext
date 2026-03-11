//! Connection limiter for WebSocket connections
//!
//! Enforces a maximum number of concurrent WebSocket connections
//! per document to prevent resource exhaustion.

use crate::error::ApiError;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

/// Limits concurrent WebSocket connections per document
pub struct ConnectionLimiter {
    /// Map of document_id to semaphore (max 10 connections per document)
    limits: DashMap<Uuid, Arc<Semaphore>>,
}

impl ConnectionLimiter {
    /// Create a new connection limiter
    pub fn new() -> Self {
        Self {
            limits: DashMap::new(),
        }
    }

    /// Maximum concurrent connections allowed per document
    const MAX_CONNECTIONS: usize = 10;

    /// Acquire a permit for a WebSocket connection to the given document
    ///
    /// Returns an error if the maximum number of connections (10)
    /// has been reached for this document.
    /// The permit is automatically released when dropped.
    pub async fn acquire(
        &self,
        document_id: Uuid,
    ) -> Result<tokio::sync::OwnedSemaphorePermit, ApiError> {
        let semaphore = self
            .limits
            .entry(document_id)
            .or_insert_with(|| Arc::new(Semaphore::new(Self::MAX_CONNECTIONS)));

        semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| ApiError::TooManyRequests(
                "Maximum WebSocket connections (10) reached for this document".to_string(),
            ))
    }

    /// Get the current number of active connections for a document
    pub fn connection_count(&self, document_id: Uuid) -> usize {
        self.limits
            .get(&document_id)
            .map(|s| Self::MAX_CONNECTIONS - s.available_permits())
            .unwrap_or(0)
    }

    /// Remove the limiter entry for a document (cleanup)
    pub fn remove(&self, document_id: &Uuid) {
        self.limits.remove(document_id);
    }
}

impl Default for ConnectionLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tenth_connection_is_accepted() {
        let limiter = ConnectionLimiter::new();
        let document_id = Uuid::new_v4();

        // Acquire 10 permits
        let mut permits = Vec::new();
        for _ in 0..10 {
            let permit = limiter.acquire(document_id).await.unwrap();
            permits.push(permit);
        }

        assert_eq!(limiter.connection_count(document_id), 10);
    }

    #[tokio::test]
    async fn eleventh_connection_is_rejected() {
        let limiter = ConnectionLimiter::new();
        let document_id = Uuid::new_v4();

        // Acquire 10 permits
        let mut permits = Vec::new();
        for _ in 0..10 {
            permits.push(limiter.acquire(document_id).await.unwrap());
        }

        // 11th should fail
        let result = limiter.acquire(document_id).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::TooManyRequests(_) => {}
            _ => panic!("Expected TooManyRequests error"),
        }
    }

    #[tokio::test]
    async fn permit_release_allows_new_connection() {
        let limiter = ConnectionLimiter::new();
        let document_id = Uuid::new_v4();

        // Acquire 10 permits
        let mut permits = Vec::new();
        for _ in 0..10 {
            permits.push(limiter.acquire(document_id).await.unwrap());
        }

        // Drop one permit
        permits.pop();

        // Now we can acquire again
        assert!(limiter.acquire(document_id).await.is_ok());
    }

    #[tokio::test]
    async fn connection_count_increases_with_each_permit() {
        let limiter = ConnectionLimiter::new();
        let document_id = Uuid::new_v4();

        assert_eq!(limiter.connection_count(document_id), 0);

        let _permit1 = limiter.acquire(document_id).await.unwrap();
        assert_eq!(limiter.connection_count(document_id), 1);

        let _permit2 = limiter.acquire(document_id).await.unwrap();
        assert_eq!(limiter.connection_count(document_id), 2);
    }
}
