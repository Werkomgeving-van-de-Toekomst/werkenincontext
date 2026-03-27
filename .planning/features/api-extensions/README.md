# API Extensions

## Overview

Vier uitbreidingen op de bestaande document API: pipeline executor integratie, authentication checks, S3 storage integratie, en WebSocket support.

## Status

📋 **Planned** - Design complete, awaiting implementation

## Key Features

1. **Pipeline executor integration** - Koppel agent orchestrator aan document API
2. **Authentication** - RBAC checks op document endpoints
3. **S3 storage** - Directe S3 integratie voor documenten
4. **WebSocket** - Real-time status updates voor document creation

## API Endpoints

```
POST   /api/documents              - Create (with orchestrator)
GET    /api/documents/:id/stream   - WebSocket status stream
PUT    /api/documents/:id/approve  - Approve (with RBAC)
GET    /api/documents/:id/versions - Version history
```

## Dependencies

```
Section 1: Foundation (Configuration, S3 client, WebSocket setup)
    ↓
Section 2: Orchestrator Integration
    ↓
Section 3: Authentication
    ↓
Section 4: S3 Storage
    ↓
Section 5: WebSocket
```

## RBAC Roles

| Role | Permissions |
|------|-------------|
| `ObjectCreator` | Create documents |
| `DomainEditor` | Edit in domain |
| `ObjectApprover` | Approve documents |
| `ObjectViewer` | View documents |

## S3 Integration

- Client abstraction in `iou-core/src/storage/`
- Supports both AWS S3 and MinIO
- Direct upload URLs voor grote bestanden
- Versioning support in S3 bucket

## WebSocket Events

```javascript
{
  "type": "agent_complete" | "approval_required" | "document_ready",
  "document_id": "uuid",
  "data": { ... }
}
```

## Technology Stack

| Component | Technology |
|-----------|-----------|
| WebSocket | tokio-tungstenite |
| S3 | rust-s3 |
| Channels | tokio::sync::broadcast |
| Auth | JWT middleware |

## Files to Create

- `crates/iou-core/src/storage/mod.rs`
- `crates/iou-core/src/storage/s3.rs`
- `crates/iou-api/src/websockets/mod.rs`
- `crates/iou-api/src/orchestrator/mod.rs`

## Related Documents

- Original plan: `../../planning-api-extensions/claude-plan.md`
- Agent orchestration: `../agent-orchestration/`
- Document workflow: `../document-workflow/`

---

*Source: planning-api-extensions/claude-plan.md*
