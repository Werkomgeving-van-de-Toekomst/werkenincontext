//! Realtime API endpoints
//!
//! Provides endpoints for managing Supabase Realtime WebSocket subscriptions
//! and querying connection status.

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::realtime::{RealtimeService, ConnectionState, PresenceInfo, PresenceStatus};
use crate::error::ApiError;

/// Create the realtime router
pub fn realtime_router() -> Router {
    Router::new()
        // Subscription management
        .route("/subscribe", post(subscribe))
        .route("/unsubscribe/{channel}", post(unsubscribe))
        .route("/subscriptions", get(list_subscriptions))

        // Connection status
        .route("/status", get(get_status))
        .route("/health", get(health_check))

        // Broadcasting
        .route("/broadcast", post(broadcast))
        .route("/broadcast/{topic}", post(broadcast_to_topic))

        // Presence
        .route("/presence/document/{document_id}", get(get_document_presence))
        .route("/presence/update", post(update_presence))
        .route("/presence/leave/{document_id}", post(leave_document))
}

/// Request to subscribe to a table
#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    /// Table name to subscribe to
    pub table: String,

    /// Optional filter for the subscription (e.g., "id=eq.123")
    pub filter: Option<String>,
}

/// Response for subscription creation
#[derive(Debug, Serialize)]
pub struct SubscribeResponse {
    /// Channel that was subscribed to
    pub channel: String,

    /// Table name
    pub table: String,

    /// Active subscription count
    pub subscription_count: usize,
}

/// Subscribe to table changes
///
/// POST /api/realtime/subscribe
///
/// Request body:
/// ```json
/// {
///   "table": "documents",
///   "filter": "id=eq.123"
/// }
/// ```
async fn subscribe(
    Extension(realtime): Extension<Option<Arc<RealtimeService>>>,
    Json(req): Json<SubscribeRequest>,
) -> Result<Json<SubscribeResponse>, ApiError> {
    let service = realtime.ok_or_else(|| {
        ApiError::ServiceUnavailable("Realtime service is not configured".to_string())
    })?;

    let subscription = service
        .subscribe(&req.table, req.filter.as_deref())
        .await
        .map_err(|e| ApiError::Validation(format!("Subscription failed: {}", e)))?;

    let response = SubscribeResponse {
        channel: subscription.channel().to_string(),
        table: subscription.table().to_string(),
        subscription_count: service.subscription_count(),
    };

    // We don't return the subscription handle to the client
    // In a real implementation, you'd store this somewhere associated with the session

    Ok(Json(response))
}

/// Unsubscribe from a channel
///
/// POST /api/realtime/unsubscribe/:channel
async fn unsubscribe(
    Extension(realtime): Extension<Option<Arc<RealtimeService>>>,
    Path(channel): Path<String>,
) -> Result<StatusCode, ApiError> {
    let _service = realtime.ok_or_else(|| {
        ApiError::ServiceUnavailable("Realtime service is not configured".to_string())
    })?;

    // Note: We'd need to store subscriptions somewhere to actually unsubscribe
    // For now, this is a placeholder

    Ok(StatusCode::NO_CONTENT)
}

/// Active subscriptions response
#[derive(Debug, Serialize)]
pub struct SubscriptionsResponse {
    /// Number of active subscriptions
    pub count: usize,

    /// Connection state
    pub state: String,
}

/// List active subscriptions
///
/// GET /api/realtime/subscriptions
async fn list_subscriptions(
    Extension(realtime): Extension<Option<Arc<RealtimeService>>>,
) -> Result<Json<SubscriptionsResponse>, ApiError> {
    let service = realtime.ok_or_else(|| {
        ApiError::ServiceUnavailable("Realtime service is not configured".to_string())
    })?;

    let state = service.connection_state().await;
    let state_str = match state {
        ConnectionState::Disconnected => "disconnected",
        ConnectionState::Connecting => "connecting",
        ConnectionState::Connected => "connected",
        ConnectionState::Reconnecting => "reconnecting",
    };

    Ok(Json(SubscriptionsResponse {
        count: service.subscription_count(),
        state: state_str.to_string(),
    }))
}

/// Connection status response
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    /// Whether connected to Supabase Realtime
    pub connected: bool,

    /// Connection state
    pub state: String,

    /// Number of active subscriptions
    pub subscription_count: usize,
}

/// Get connection status
///
/// GET /api/realtime/status
async fn get_status(
    Extension(realtime): Extension<Option<Arc<RealtimeService>>>,
) -> Result<Json<StatusResponse>, ApiError> {
    let service = realtime.ok_or_else(|| {
        ApiError::ServiceUnavailable("Realtime service is not configured".to_string())
    })?;

    let state = service.connection_state().await;
    let state_str = match state {
        ConnectionState::Disconnected => "disconnected",
        ConnectionState::Connecting => "connecting",
        ConnectionState::Connected => "connected",
        ConnectionState::Reconnecting => "reconnecting",
    };

    Ok(Json(StatusResponse {
        connected: service.is_connected().await,
        state: state_str.to_string(),
        subscription_count: service.subscription_count(),
    }))
}

/// Health check for realtime service
///
/// GET /api/realtime/health
async fn health_check(
    Extension(realtime): Extension<Option<Arc<RealtimeService>>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let Some(service) = realtime else {
        return Ok(Json(serde_json::json!({
            "status": "disabled",
            "message": "Realtime service is not configured"
        })));
    };

    let connected = service.is_connected().await;

    Ok(Json(serde_json::json!({
        "status": if connected { "healthy" } else { "unhealthy" },
        "connected": connected,
        "subscriptions": service.subscription_count()
    })))
}

/// Broadcast request
#[derive(Debug, Deserialize)]
pub struct BroadcastRequest {
    /// Event name/type
    pub event: String,

    /// Payload to broadcast
    pub payload: serde_json::Value,
}

/// Broadcast to all subscribers
///
/// POST /api/realtime/broadcast
async fn broadcast(
    Extension(realtime): Extension<Option<Arc<RealtimeService>>>,
    Json(req): Json<BroadcastRequest>,
) -> Result<StatusCode, ApiError> {
    let service = realtime.ok_or_else(|| {
        ApiError::ServiceUnavailable("Realtime service is not configured".to_string())
    })?;

    service
        .broadcast("all", &req.event, req.payload)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Broadcast failed: {}", e)))?;

    Ok(StatusCode::ACCEPTED)
}

/// Broadcast to a specific topic
///
/// POST /api/realtime/broadcast/:topic
async fn broadcast_to_topic(
    Extension(realtime): Extension<Option<Arc<RealtimeService>>>,
    Path(topic): Path<String>,
    Json(req): Json<BroadcastRequest>,
) -> Result<StatusCode, ApiError> {
    let service = realtime.ok_or_else(|| {
        ApiError::ServiceUnavailable("Realtime service is not configured".to_string())
    })?;

    service
        .broadcast(&topic, &req.event, req.payload)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Broadcast failed: {}", e)))?;

    Ok(StatusCode::ACCEPTED)
}

/// Presence info for a document
#[derive(Debug, Serialize)]
pub struct DocumentPresenceResponse {
    /// Document ID
    pub document_id: Uuid,

    /// Users currently viewing
    pub viewers: Vec<ViewerInfo>,

    /// Users currently editing
    pub editors: Vec<ViewerInfo>,

    /// Total active users
    pub total_count: usize,
}

/// Information about a user viewing/editing
#[derive(Debug, Serialize)]
pub struct ViewerInfo {
    /// User ID
    pub user_id: Uuid,

    /// User display name
    pub user_name: String,

    /// Current status
    pub status: String,

    /// Last activity timestamp
    pub last_seen: i64,
}

/// Get presence info for a document
///
/// GET /api/realtime/presence/document/:document_id
async fn get_document_presence(
    Extension(realtime): Extension<Option<Arc<RealtimeService>>>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<DocumentPresenceResponse>, ApiError> {
    let service = realtime.ok_or_else(|| {
        ApiError::ServiceUnavailable("Realtime service is not configured".to_string())
    })?;

    let viewers = service.get_document_viewers(&document_id);
    let editors = service.get_document_editors(&document_id);

    let viewer_infos: Vec<ViewerInfo> = viewers
        .iter()
        .map(|p| ViewerInfo {
            user_id: p.user_id,
            user_name: p.user_name.clone(),
            status: p.status.as_str().to_string(),
            last_seen: p.last_seen.timestamp(),
        })
        .collect();

    let editor_infos: Vec<ViewerInfo> = editors
        .iter()
        .map(|p| ViewerInfo {
            user_id: p.user_id,
            user_name: p.user_name.clone(),
            status: p.status.as_str().to_string(),
            last_seen: p.last_seen.timestamp(),
        })
        .collect();

    Ok(Json(DocumentPresenceResponse {
        document_id,
        viewers: viewer_infos,
        editors: editor_infos,
        total_count: service.presence().get_active_count(&document_id),
    }))
}

/// Update presence request
#[derive(Debug, Deserialize)]
pub struct UpdatePresenceRequest {
    /// User ID
    pub user_id: Uuid,

    /// User display name
    pub user_name: String,

    /// Document ID
    pub document_id: Uuid,

    /// Status (viewing, editing, idle)
    pub status: String,

    /// Optional cursor position
    pub cursor_position: Option<usize>,

    /// Optional selection range
    pub selection_range: Option<(usize, usize)>,
}

/// Update user presence on a document
///
/// POST /api/realtime/presence/update
async fn update_presence(
    Extension(realtime): Extension<Option<Arc<RealtimeService>>>,
    Json(req): Json<UpdatePresenceRequest>,
) -> Result<StatusCode, ApiError> {
    let service = realtime.ok_or_else(|| {
        ApiError::ServiceUnavailable("Realtime service is not configured".to_string())
    })?;

    let status = PresenceStatus::from_str(&req.status)
        .ok_or_else(|| ApiError::Validation(format!("Invalid status: {}", req.status)))?;

    let mut info = PresenceInfo::new(req.user_id, req.user_name, req.document_id, status);
    info.cursor_position = req.cursor_position;
    info.selection_range = req.selection_range;

    service.update_presence(info);

    Ok(StatusCode::OK)
}

/// Leave document request
#[derive(Debug, Deserialize)]
pub struct LeaveDocumentRequest {
    /// User ID
    pub user_id: Uuid,

    /// Document ID
    pub document_id: Uuid,
}

/// Remove user presence from a document
///
/// POST /api/realtime/presence/leave/:document_id
async fn leave_document(
    Extension(realtime): Extension<Option<Arc<RealtimeService>>>,
    Path(document_id): Path<Uuid>,
    Json(req): Json<LeaveDocumentRequest>,
) -> Result<StatusCode, ApiError> {
    let service = realtime.ok_or_else(|| {
        ApiError::ServiceUnavailable("Realtime service is not configured".to_string())
    })?;

    service.remove_user_from_document(req.user_id, document_id);

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_response_serialization() {
        let response = StatusResponse {
            connected: true,
            state: "connected".to_string(),
            subscription_count: 5,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"connected\":true"));
        assert!(json.contains("\"state\":\"connected\""));
        assert!(json.contains("\"subscription_count\":5"));
    }

    #[test]
    fn test_document_presence_response_serialization() {
        let id = Uuid::new_v4();
        let response = DocumentPresenceResponse {
            document_id: id,
            viewers: vec![],
            editors: vec![],
            total_count: 0,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"document_id\""));
        assert!(json.contains("\"total_count\":0"));
    }
}
