# Supabase Realtime Integration

This document describes the Supabase Realtime WebSocket integration in the IOU-Modern API.

## Overview

The Supabase Realtime integration provides WebSocket-based real-time data synchronization between the API server and Supabase PostgreSQL database. This allows the application to receive instant notifications when data changes in the database.

## Architecture

```
┌─────────────┐      WebSocket      ┌──────────────────┐
│  IOU-API    │◄─────────────────────►│ Supabase Realtime│
│             │   Phoenix Protocol    │                  │
└──────┬──────┘                      └──────────────────┘
       │
       │
       ▼
┌──────────────────┐
│  RealtimeService │  Manages WebSocket connections,
│                  │  subscriptions, and presence
└──────────────────┘
```

## Components

### RealtimeClient

Core WebSocket client implementing the Supabase Realtime protocol (based on Phoenix protocol).

- **Location**: `crates/iou-api/src/realtime/supabase_rt.rs`
- **Features**:
  - Auto-reconnect with exponential backoff
  - Heartbeat mechanism (30s interval)
  - PostgreSQL changes subscription
  - Custom event broadcasting
  - Graceful shutdown

### RealtimeService

Service layer that manages the RealtimeClient lifecycle.

- **Location**: `crates/iou-api/src/realtime/service.rs`
- **Features**:
  - Background connection management
  - Subscription tracking
  - Presence tracking integration
  - Thread-safe API

### PresenceTracker

Tracks which users are currently viewing or editing documents.

- **Location**: `crates/iou-api/src/realtime/presence.rs`
- **Features**:
  - User activity tracking
  - Automatic cleanup of stale presence
  - Per-document user lists

## Configuration

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `SUPABASE_REALTIME_URL` | WebSocket URL (wss://...) | - | Yes* |
| `SUPABASE_JWT_TOKEN` | JWT token for auth | - | No |
| `REALTIME_HEARTBEAT_INTERVAL` | Heartbeat interval (seconds) | 30 | No |
| `REALTIME_CONNECT_TIMEOUT` | Connection timeout (seconds) | 10 | No |
| `REALTIME_MAX_RECONNECT_BACKOFF` | Max reconnect backoff (seconds) | 60 | No |
| `REALTIME_AUTO_RECONNECT` | Enable auto-reconnect | true | No |

*Required for realtime features; service starts in disabled mode if not set.

### Example Configuration

```bash
# .env
SUPABASE_REALTIME_URL=wss://your-project.supabase.co/realtime/v1/websocket
SUPABASE_JWT_TOKEN=your-jwt-token
REALTIME_HEARTBEAT_INTERVAL=30
REALTIME_AUTO_RECONNECT=true
```

## API Endpoints

All realtime endpoints are prefixed with `/api/realtime/`.

### Connection Status

#### GET /api/realtime/status

Get the current connection status.

**Response:**
```json
{
  "connected": true,
  "state": "connected",
  "subscription_count": 3
}
```

#### GET /api/realtime/health

Health check endpoint for monitoring.

**Response:**
```json
{
  "status": "healthy",
  "connected": true,
  "subscriptions": 3
}
```

### Subscriptions

#### POST /api/realtime/subscribe

Subscribe to a table's changes.

**Request:**
```json
{
  "table": "documents",
  "filter": "id=eq.123"
}
```

**Response:**
```json
{
  "channel": "documents:id=eq.123",
  "table": "documents",
  "subscription_count": 1
}
```

#### POST /api/realtime/unsubscribe/:channel

Unsubscribe from a channel.

#### GET /api/realtime/subscriptions

List active subscriptions.

**Response:**
```json
{
  "count": 3,
  "state": "connected"
}
```

### Broadcasting

#### POST /api/realtime/broadcast

Broadcast a custom event to all subscribers.

**Request:**
```json
{
  "event": "custom_event",
  "payload": { "key": "value" }
}
```

#### POST /api/realtime/broadcast/:topic

Broadcast to a specific topic.

**Request:**
```json
{
  "event": "document_updated",
  "payload": { "id": "123", "status": "approved" }
}
```

### Presence

#### GET /api/realtime/presence/document/:document_id

Get users currently viewing/editing a document.

**Response:**
```json
{
  "document_id": "uuid",
  "viewers": [
    {
      "user_id": "uuid",
      "user_name": "John Doe",
      "status": "editing",
      "last_seen": 1234567890
    }
  ],
  "editors": [
    {
      "user_id": "uuid",
      "user_name": "John Doe",
      "status": "editing",
      "last_seen": 1234567890
    }
  ],
  "total_count": 1
}
```

#### POST /api/realtime/presence/update

Update user presence on a document.

**Request:**
```json
{
  "user_id": "uuid",
  "user_name": "John Doe",
  "document_id": "uuid",
  "status": "editing",
  "cursor_position": 100,
  "selection_range": [100, 150]
}
```

#### POST /api/realtime/presence/leave/:document_id

Remove user presence from a document.

**Request:**
```json
{
  "user_id": "uuid",
  "document_id": "uuid"
}
```

## Programmatic Usage

### Using RealtimeService in Code

```rust
use crate::realtime::RealtimeService;

// Create service from environment
let service = RealtimeService::from_env()?;

// Start the service (spawns background connection task)
service.start().await?;

// Subscribe to table changes
let subscription = service.subscribe("documents", Some("id=eq.123")).await?;

// Receive events
while let Some(event) = subscription.recv().await {
    match event.update_type {
        UpdateType::Created => println!("New record: {:?}", event.record),
        UpdateType::Updated => println!("Updated record: {:?}", event.record),
        UpdateType::Deleted => println!("Deleted record"),
        _ => {}
    }
}

// Update user presence
service.update_presence(PresenceInfo::new(
    user_id,
    "John Doe".to_string(),
    document_id,
    PresenceStatus::Editing,
));

// Broadcast custom event
service.broadcast("documents", "status_changed", json!({"id": "123"})).await?;
```

### Accessing from Axum Handlers

```rust
use axum::extract::Extension;
use std::sync::Arc;

async fn my_handler(
    Extension(realtime): Extension<Option<Arc<RealtimeService>>>,
) -> Result<Json<Value>, ApiError> {
    if let Some(service) = realtime {
        let connected = service.is_connected().await;
        // Use the service...
    } else {
        // Realtime not configured
    }
    Ok(Json(json!({ "status": "ok" })))
}
```

## Phoenix Protocol

The client implements the Supabase Realtime protocol (Phoenix-based).

### Message Format

```json
{
  "event": "phx_join",
  "topic": "realtime:*",
  "payload": {},
  "ref": "unique-ref"
}
```

### Event Types

- `phx_join` - Join a topic
- `phx_leave` - Leave a topic
- `phx_reply` - Server response
- `heartbeat` - Keep-alive (every 30s)
- `postgres_changes` - Database change notification
- `broadcast` - Custom event

## Troubleshooting

### Service Not Starting

If the realtime service doesn't start:

1. Check that `SUPABASE_REALTIME_URL` is set
2. Verify the URL format: `wss://project-ref.supabase.co/realtime/v1/websocket`
3. Check network connectivity to Supabase
4. Review logs for connection errors

### Connection Drops

The client automatically reconnects with exponential backoff:

- Initial attempt: immediate
- Subsequent attempts: 1s, 2s, 4s, ... up to 60s max
- Heartbeat every 30s to detect dead connections

### High Memory Usage

If memory usage grows:

1. Check subscription count - unsubscribe from unused channels
2. Review presence cleanup interval (default 5min timeout)
3. Monitor for connection leaks

## Testing

### Unit Tests

Run realtime module tests:

```bash
cargo test -p iou-api --lib realtime
```

### Integration Tests

Run with actual Supabase:

```bash
# Set environment variables
export SUPABASE_REALTIME_URL=wss://...
export SUPABASE_JWT_TOKEN=...

# Run the server
cargo run -p iou-api
```

### Manual Testing with wscat

```bash
# Install wscat
npm install -g wscat

# Connect to Supabase Realtime
wscat -c "wss://your-project.supabase.co/realtime/v1/websocket?apikey=your-key"

# Send join message
{"event":"phx_join","topic":"realtime:*","payload":{},"ref":"1"}
```

## Future Enhancements

- [ ] Per-subscription event filtering
- [ ] Webhook delivery for offline clients
- [ ] Presence sync across multiple instances
- [ ] Metrics and monitoring dashboard
- [ ] GraphQL subscriptions integration
- [ ] Room-based channels for multi-user collaboration
