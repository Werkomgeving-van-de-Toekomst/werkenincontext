//! Supabase Realtime Client
//!
//! WebSocket client for subscribing to Supabase Realtime channels.

use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Configuration for Supabase Realtime connection
#[derive(Debug, Clone)]
pub struct RealtimeConfig {
    /// Supabase Realtime WebSocket URL
    pub websocket_url: String,

    /// Bearer auth bearer for authenticated connections
    pub bearer_bearer: Option<String>,

    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,

    /// Reconnect interval in seconds
    pub reconnect_interval: u64,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            websocket_url: "ws://localhost:4000/socket".to_string(),
            bearer_bearer: None,
            heartbeat_interval: 30,
            reconnect_interval: 5,
        }
    }
}

/// Supabase Realtime client for WebSocket connections
#[derive(Clone)]
pub struct SupabaseRealtime {
    config: RealtimeConfig,
    channels: Arc<RwLock<Vec<RealtimeChannel>>>,
}

impl SupabaseRealtime {
    /// Create a new Realtime client
    pub fn new(base_url: &str) -> Result<Self, RealtimeError> {
        let ws_url = if base_url.contains("realtime") {
            base_url.to_string()
        } else {
            format!("{}/realtime/v1/websocket", base_url.trim_end_matches('/'))
        };

        Ok(Self {
            config: RealtimeConfig {
                websocket_url: ws_url,
                ..Default::default()
            },
            channels: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Create with custom configuration
    pub fn with_config(config: RealtimeConfig) -> Self {
        Self {
            config,
            channels: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Set the bearer authentication bearer
    pub fn with_bearer_bearer(mut self, bearer: String) -> Self {
        self.config.bearer_bearer = Some(bearer);
        self
    }

    /// Connect to a specific table and subscribe to changes
    pub async fn subscribe_table(
        &self,
        table: &str,
        filter: Option<&str>,
    ) -> Result<RealtimeChannel, RealtimeError> {
        let topic = if let Some(f) = filter {
            format!("{}:{}", table, f)
        } else {
            format!("{}:*", table)
        };

        self.join_topic(topic).await
    }

    /// Connect to a broadcast channel for custom events
    pub async fn broadcast(&self, channel_name: &str) -> Result<RealtimeChannel, RealtimeError> {
        let topic = format!("broadcast:{}", channel_name);
        self.join_topic(topic).await
    }

    /// Connect to a presence channel for tracking online users
    pub async fn presence(&self, channel_name: &str) -> Result<RealtimeChannel, RealtimeError> {
        let topic = format!("presence:{}", channel_name);
        self.join_topic(topic).await
    }

    /// Join a specific topic
    async fn join_topic(&self, topic: String) -> Result<RealtimeChannel, RealtimeError> {
        let (tx, rx) = mpsc::channel(100);

        let channel = RealtimeChannel {
            topic: topic.clone(),
            sender: tx,
            receiver: Arc::new(RwLock::new(Some(rx))),
            join_ref: Uuid::new_v4().to_string(),
        };

        let mut channels = self.channels.write().await;
        channels.push(channel.clone());

        tracing::info!("Joined Realtime topic: {}", topic);

        Ok(channel)
    }

    /// Remove a channel subscription
    pub async fn leave(&self, topic: &str) -> Result<(), RealtimeError> {
        let mut channels = self.channels.write().await;
        channels.retain(|c| c.topic != topic);

        tracing::info!("Left Realtime topic: {}", topic);

        Ok(())
    }

    /// Get active channel count
    pub async fn channel_count(&self) -> usize {
        self.channels.read().await.len()
    }
}

/// A Realtime channel representing a subscription
#[derive(Clone)]
pub struct RealtimeChannel {
    topic: String,
    sender: mpsc::Sender<RealtimeEvent>,
    receiver: Arc<RwLock<Option<mpsc::Receiver<RealtimeEvent>>>>,
    join_ref: String,
}

impl RealtimeChannel {
    /// Get the channel topic
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// Send a custom event to this channel
    pub async fn send<T: Serialize>(&self, payload: T) -> Result<(), RealtimeError> {
        let event = RealtimeEvent {
            topic: self.topic.clone(),
            event_type: EventType::Broadcast,
            payload: serde_json::to_value(payload)
                .map_err(|e| RealtimeError::SerializationError(e.to_string()))?,
            timestamp: chrono::Utc::now(),
        };

        self.sender
            .send(event)
            .await
            .map_err(|_| RealtimeError::ChannelClosed)?;

        Ok(())
    }

    /// Track presence for this user
    pub async fn track(&self, presence: PresenceData) -> Result<(), RealtimeError> {
        let event = RealtimeEvent {
            topic: self.topic.clone(),
            event_type: EventType::Presence,
            payload: serde_json::to_value(presence)
                .map_err(|e| RealtimeError::SerializationError(e.to_string()))?,
            timestamp: chrono::Utc::now(),
        };

        self.sender
            .send(event)
            .await
            .map_err(|_| RealtimeError::ChannelClosed)?;

        Ok(())
    }
}

/// Realtime event received from Supabase
#[derive(Debug, Clone)]
pub struct RealtimeEvent {
    pub topic: String,
    pub event_type: EventType,
    pub payload: Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Type of realtime event
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    Broadcast,
    Presence,
    System,
}

/// Presence data for tracking users
#[derive(Debug, Clone, Serialize)]
pub struct PresenceData {
    pub user_id: String,
    pub online: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

impl PresenceData {
    pub fn online(user_id: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            online: true,
            metadata: None,
        }
    }

    pub fn offline() -> Self {
        Self {
            user_id: String::new(),
            online: false,
            metadata: None,
        }
    }
}

/// Errors that can occur in realtime operations
#[derive(Debug, thiserror::Error)]
pub enum RealtimeError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Channel closed")]
    ChannelClosed,

    #[error("Serialization error: {0}")]
    SerializationError(String),
}
