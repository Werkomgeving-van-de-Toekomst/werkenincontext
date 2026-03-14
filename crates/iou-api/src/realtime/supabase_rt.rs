//! Supabase Realtime Client
//!
//! WebSocket client for subscribing to Supabase Realtime channels.
//! Handles connection, subscription management, and event reception.

use serde_json::Value;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use uuid::Uuid;
use std::time::Duration;

/// Configuration for Supabase Realtime connection
#[derive(Debug, Clone)]
pub struct RealtimeConfig {
    /// Supabase Realtime WebSocket URL
    /// Format: wss://<project-ref>.supabase.co/realtime/v1
    pub websocket_url: String,

    /// JWT token for authentication
    pub jwt_token: Option<String>,

    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,

    /// Connection timeout in seconds
    pub connect_timeout: u64,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            websocket_url: "ws://localhost:4000/socket".to_string(), // Default for local dev
            jwt_token: None,
            heartbeat_interval: 30,
            connect_timeout: 10,
        }
    }
}

/// Realtime client for Supabase
pub struct RealtimeClient {
    /// Configuration
    config: RealtimeConfig,

    /// Active subscription metadata (doesn't include the Receiver)
    subscriptions: dashmap::DashMap<String, SubscriptionInfo>,
}

/// Metadata about an active subscription (can be cloned)
#[derive(Debug, Clone)]
struct SubscriptionInfo {
    channel: String,
    table: String,
    filter: Option<String>,
}

impl RealtimeClient {
    /// Create a new Realtime client
    pub fn new(config: RealtimeConfig) -> Self {
        Self {
            config,
            subscriptions: dashmap::DashMap::new(),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self, RealtimeError> {
        let websocket_url = std::env::var("SUPABASE_REALTIME_URL")
            .unwrap_or_else(|_| "ws://localhost:4000/socket".to_string());

        let jwt_token = std::env::var("SUPABASE_JWT_TOKEN").ok();

        let config = RealtimeConfig {
            websocket_url,
            jwt_token,
            ..Default::default()
        };

        Ok(Self::new(config))
    }

    /// Subscribe to a table's changes
    ///
    /// # Arguments
    /// * `table` - Table name to subscribe to
    /// * `filter` - Optional filter for the subscription (e.g., "id=eq.123")
    ///
    /// # Returns
    /// A subscription handle that can be used to receive events
    pub async fn subscribe(
        &self,
        table: &str,
        filter: Option<&str>,
    ) -> Result<SubscriptionHandle, RealtimeError> {
        let topic = if let Some(f) = filter {
            format!("{}:{}", table, f)
        } else {
            format!("{}:*", table)
        };

        // Create channel for this subscription
        let (tx, rx) = mpsc::channel(100);

        let handle = SubscriptionHandle {
            channel: topic.clone(),
            table: table.to_string(),
            filter: filter.map(String::from),
            receiver: rx,
            sender: tx,
        };

        // Store just the metadata (clone-able)
        self.subscriptions.insert(topic.clone(), SubscriptionInfo {
            channel: topic.clone(),
            table: table.to_string(),
            filter: filter.map(String::from),
        });

        // TODO: Connect to WebSocket and send subscription message
        tracing::info!("Subscribed to table: {} with filter: {:?}", table, filter);

        Ok(handle)
    }

    /// Subscribe to a specific document
    pub async fn subscribe_document(
        &self,
        document_id: Uuid,
    ) -> Result<SubscriptionHandle, RealtimeError> {
        self.subscribe(
            "documents",
            Some(&format!("id=eq.{}", document_id)),
        ).await
    }

    /// Broadcast a document update to all subscribers
    ///
    /// Note: This may be handled automatically by Supabase's
    /// WAL replication. Use this for custom events only.
    pub async fn broadcast_document_update(
        &self,
        document_id: &str,
        update_type: UpdateType,
        payload: Value,
    ) -> Result<(), RealtimeError> {
        // TODO: Implement broadcast via Realtime's broadcast feature
        tracing::debug!(
            "Broadcasting update for document {}: {:?}",
            document_id,
            update_type
        );

        Ok(())
    }

    /// Unsubscribe from a channel
    pub fn unsubscribe(&self, channel: &str) {
        self.subscriptions.remove(channel);
        tracing::info!("Unsubscribed from: {}", channel);
    }

    /// Get active subscription count
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.len()
    }
}

/// Handle for an active subscription
#[derive(Debug)]
pub struct SubscriptionHandle {
    /// Channel topic
    channel: String,

    /// Table name
    table: String,

    /// Filter applied
    filter: Option<String>,

    /// Event receiver
    #[allow(dead_code)]
    receiver: mpsc::Receiver<RealtimeEvent>,

    /// Event sender (for internal use)
    sender: mpsc::Sender<RealtimeEvent>,
}

impl SubscriptionHandle {
    /// Get the channel name
    pub fn channel(&self) -> &str {
        &self.channel
    }

    /// Get the table name
    pub fn table(&self) -> &str {
        &self.table
    }

    /// Receive the next event
    pub async fn recv(&mut self) -> Option<RealtimeEvent> {
        self.receiver.recv().await
    }

    /// Try to receive an event without blocking
    pub fn try_recv(&mut self) -> Option<RealtimeEvent> {
        self.receiver.try_recv().ok()
    }
}

/// Types of updates that can occur
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateType {
    Created,
    Updated,
    Deleted,
    StatusChanged,
}

impl UpdateType {
    /// Parse from string (e.g., "INSERT", "UPDATE")
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "INSERT" => Some(Self::Created),
            "UPDATE" => Some(Self::Updated),
            "DELETE" => Some(Self::Deleted),
            _ => None,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "INSERT",
            Self::Updated => "UPDATE",
            Self::Deleted => "DELETE",
            Self::StatusChanged => "UPDATE",
        }
    }
}

/// Real-time event from Supabase
#[derive(Debug, Clone)]
pub struct RealtimeEvent {
    /// Table that triggered the event
    pub table: String,

    /// Type of record (e.g., "documents")
    pub record_type: String,

    /// The new record data
    pub record: Value,

    /// The old record data (for UPDATE and DELETE)
    pub old_record: Option<Value>,

    /// Type of update
    pub update_type: UpdateType,

    /// Timestamp of the event
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Errors that can occur in realtime operations
#[derive(Debug, thiserror::Error)]
pub enum RealtimeError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Subscription failed: {0}")]
    SubscriptionFailed(String),

    #[error("Invalid message format")]
    InvalidMessage,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    #[error("Send error: {0}")]
    SendError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_type_parsing() {
        assert_eq!(UpdateType::from_str("INSERT"), Some(UpdateType::Created));
        assert_eq!(UpdateType::from_str("UPDATE"), Some(UpdateType::Updated));
        assert_eq!(UpdateType::from_str("DELETE"), Some(UpdateType::Deleted));
        assert_eq!(UpdateType::from_str("INVALID"), None);
    }

    #[test]
    fn test_update_type_as_str() {
        assert_eq!(UpdateType::Created.as_str(), "INSERT");
        assert_eq!(UpdateType::Updated.as_str(), "UPDATE");
        assert_eq!(UpdateType::Deleted.as_str(), "DELETE");
    }

    #[tokio::test]
    async fn test_realtime_client_creation() {
        let config = RealtimeConfig {
            websocket_url: "ws://localhost:4000/socket".to_string(),
            jwt_token: None,
            ..Default::default()
        };

        let client = RealtimeClient::new(config);
        assert_eq!(client.subscription_count(), 0);
    }

    #[tokio::test]
    async fn test_subscription_handle() {
        let (tx, rx) = mpsc::channel(100);

        let mut handle = SubscriptionHandle {
            channel: "documents:*".to_string(),
            table: "documents".to_string(),
            filter: None,
            receiver: rx,
            sender: tx,
        };

        assert_eq!(handle.channel(), "documents:*");
        assert_eq!(handle.table(), "documents");
        assert!(handle.try_recv().is_none());
    }
}
