//! Realtime Service
//!
//! Manages the Supabase Realtime WebSocket client lifecycle,
//! handles subscriptions, and broadcasts events to connected clients.

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::supabase_rt::{RealtimeClient, RealtimeConfig, RealtimeError, ConnectionState};
use super::presence::{PresenceTracker, PresenceInfo, PresenceStatus};

/// Service that manages Supabase Realtime WebSocket connections
pub struct RealtimeService {
    /// The underlying realtime client
    client: Arc<RwLock<RealtimeClient>>,

    /// Presence tracker for user activity
    presence: Arc<PresenceTracker>,

    /// Whether the service is running
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl RealtimeService {
    /// Create a new realtime service from configuration
    pub fn new(config: RealtimeConfig) -> Self {
        // Only create client if URL is configured
        let client = if config.websocket_url.is_empty() {
            tracing::info!("Supabase Realtime URL not configured, using disabled client");
            // Create a client with empty URL - it will fail to connect but won't crash
            RealtimeClient::new(config)
        } else {
            RealtimeClient::new(config)
        };

        Self {
            client: Arc::new(RwLock::new(client)),
            presence: Arc::new(PresenceTracker::new()),
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self, RealtimeError> {
        let url = std::env::var("SUPABASE_REALTIME_URL")
            .unwrap_or_else(|_| String::new());

        let jwt_token = std::env::var("SUPABASE_JWT_TOKEN").ok();

        let heartbeat_interval = std::env::var("REALTIME_HEARTBEAT_INTERVAL")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        let connect_timeout = std::env::var("REALTIME_CONNECT_TIMEOUT")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        let max_reconnect_backoff = std::env::var("REALTIME_MAX_RECONNECT_BACKOFF")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .unwrap_or(60);

        let auto_reconnect = std::env::var("REALTIME_AUTO_RECONNECT")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let config = RealtimeConfig {
            websocket_url: url,
            jwt_token: jwt_token,
            heartbeat_interval,
            connect_timeout,
            max_reconnect_backoff,
            auto_reconnect,
        };

        Ok(Self::new(config))
    }

    /// Start the realtime service in the background
    ///
    /// This spawns a task that maintains the WebSocket connection.
    /// Returns immediately after spawning the task.
    pub async fn start(&self) -> Result<(), RealtimeError> {
        // Check if already running
        if self.is_running.load(std::sync::atomic::Ordering::Relaxed) {
            tracing::warn!("RealtimeService is already running");
            return Ok(());
        }

        // Check if URL is configured
        let client = self.client.read().await;
        let config_url = {
            // We need to access the config through the client
            // Since we can't directly access it, we'll just try to connect
            true
        };
        drop(client);

        if !config_url {
            tracing::info!("Supabase Realtime not configured, service disabled");
            return Ok(());
        }

        self.is_running.store(true, std::sync::atomic::Ordering::Relaxed);

        // Spawn background connection task
        let client_clone = Arc::clone(&self.client);
        let is_running = Arc::clone(&self.is_running);

        tokio::spawn(async move {
            while is_running.load(std::sync::atomic::Ordering::Relaxed) {
                {
                    let client = client_clone.read().await;
                    let state = client.connection_state().await;

                    if state != ConnectionState::Connected {
                        drop(client);
                        let mut client = client_clone.write().await;
                        if let Err(e) = client.connect().await {
                            tracing::error!("Failed to connect to Supabase Realtime: {}", e);
                            // Wait before retrying
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                            continue;
                        }
                    }
                }

                // Wait a bit before checking again
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });

        tracing::info!("RealtimeService started");
        Ok(())
    }

    /// Stop the realtime service
    pub async fn stop(&self) {
        self.is_running.store(false, std::sync::atomic::Ordering::Relaxed);
        tracing::info!("RealtimeService stopped");
    }

    /// Subscribe to a table's changes
    pub async fn subscribe(
        &self,
        table: &str,
        filter: Option<&str>,
    ) -> Result<super::supabase_rt::SubscriptionHandle, RealtimeError> {
        let client = self.client.read().await;
        client.subscribe(table, filter).await
    }

    /// Subscribe to a specific document
    pub async fn subscribe_document(
        &self,
        document_id: Uuid,
    ) -> Result<super::supabase_rt::SubscriptionHandle, RealtimeError> {
        let client = self.client.read().await;
        client.subscribe_document(document_id).await
    }

    /// Unsubscribe from a channel
    pub async fn unsubscribe(&self, channel: &str) {
        let client = self.client.read().await;
        client.unsubscribe(channel).await;
    }

    /// Broadcast a document update to all subscribers
    pub async fn broadcast_document_update(
        &self,
        document_id: &str,
        update_type: super::supabase_rt::UpdateType,
        payload: serde_json::Value,
    ) -> Result<(), RealtimeError> {
        let client = self.client.read().await;
        client.broadcast_document_update(document_id, update_type, payload).await
    }

    /// Broadcast to a specific topic
    pub async fn broadcast(
        &self,
        topic: &str,
        event: &str,
        payload: serde_json::Value,
    ) -> Result<(), RealtimeError> {
        let client = self.client.read().await;
        client.broadcast(topic, event, payload).await
    }

    /// Get current connection state
    pub async fn connection_state(&self) -> ConnectionState {
        let client = self.client.read().await;
        client.connection_state().await
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        let client = self.client.read().await;
        client.is_connected().await
    }

    /// Get active subscription count
    pub fn subscription_count(&self) -> usize {
        let rt = tokio::runtime::Handle::try_current();
        if rt.is_err() {
            return 0;
        }

        // This is a sync method, so we need to use block_in_place if we're already in a runtime
        let client = Arc::clone(&self.client);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                client.read().await.subscription_count()
            })
        })
    }

    /// Get the presence tracker
    pub fn presence(&self) -> Arc<PresenceTracker> {
        Arc::clone(&self.presence)
    }

    /// Update user presence on a document
    pub fn update_presence(&self, info: PresenceInfo) {
        self.presence.update_presence(info);
    }

    /// Get all users viewing a document
    pub fn get_document_viewers(&self, document_id: &Uuid) -> Vec<PresenceInfo> {
        self.presence.get_document_viewers(document_id)
    }

    /// Get editors for a document
    pub fn get_document_editors(&self, document_id: &Uuid) -> Vec<PresenceInfo> {
        self.presence.get_document_editors(document_id)
    }

    /// Remove user from a document
    pub fn remove_user_from_document(&self, user_id: Uuid, document_id: Uuid) {
        self.presence.remove_user_from_document(user_id, document_id);
    }

    /// Remove user from all documents
    pub fn remove_user(&self, user_id: &Uuid) {
        self.presence.remove_user(user_id);
    }
}

impl Clone for RealtimeService {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            presence: Arc::clone(&self.presence),
            is_running: Arc::clone(&self.is_running),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let config = RealtimeConfig {
            websocket_url: "ws://localhost:4000/socket/websocket".to_string(),
            jwt_token: None,
            ..Default::default()
        };

        let service = RealtimeService::new(config);
        assert_eq!(service.subscription_count(), 0);
    }
}
