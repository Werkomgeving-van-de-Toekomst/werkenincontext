# Agent Orchestration API Extensions - Complete Specification

## Overview

Uitbreiding van de bestaande document API om de agent pipeline volledig functioneel te maken met authenticatie, storage, en real-time updates.

## Configuration Decisions (from Interview)

| Configuratie | Waarde |
|--------------|--------|
| Agent timeout | 2 minuten per agent (totaal ~8 min) |
| WebSocket idle timeout | 5 minuten |
| S3 download pattern | API Proxy (geen presigned URLs) |
| Max document size | 10 MB |
| Test execution mode | Synchroon (production: async) |

## Requirements

### 1. Pipeline Executor Integration

**Functionele Requirements:**

1.1 Asynchrone pipeline uitvoering bij document creatie
- API response returned onmiddellijk met document_id
- Pipeline draait op achtergrond via `tokio::spawn`
- Status updates via broadcast channel

1.2 Agent timeout handling
- Per agent: 2 minuten timeout
- Total pipeline: ~8 minuten (4 agents)
- Timeout resulteert in Failed status met error details

1.3 Status opslag tijdens processing
- Database update na elke agent completion
- Current agent bijhouden in document record
- Progress percentage (0, 25, 50, 75, 100%)

1.4 Test mode support
- Feature flag: `PIPELINE_EXECUTION_MODE` (sync/async)
- Test mode: synchroon uitvoeren voor eenvoudiger testing
- Prod mode: asynchroon met broadcast updates

**Bestanden:**
- `crates/iou-ai/src/agents/pipeline.rs` - Bestaande pipeline (reeds async)
- `crates/iou-api/src/routes/documents.rs` - Integration punt
- Nieuw: `crates/iou-api/src/workflows/executor.rs` - Async executor wrapper

### 2. Authentication Integration

**Functionele Requirements:**

2.1 Role-based access control
- `POST /api/documents/create` vereist `ObjectCreator` of `DomainEditor` role
- `POST /api/documents/{id}/approve` vereist `ObjectApprover` role
- `GET /api/documents/{id}` vereist authenticated user

2.2 Auth context extractie
- Gebruik bestaande `AuthContext` uit JWT middleware
- Extract `user_id` voor audit trail
- Check roles met `Role::has_permission()` of `roles.contains()`

2.3 Audit trail enrichment
- Sla `user_id` op bij elke document actie
- Sla `user_email` op voor traceability
- Log role checks bij approve actions

**Bestanden:**
- `crates/iou-api/src/middleware/auth.rs` - Bestaande auth module
- `crates/iou-api/src/routes/documents.rs` - Voeg auth checks toe

### 3. S3 Storage Integration

**Functionele Requirements:**

3.1 Document upload na completion
- Upload document wanneer pipeline Completed status bereikt
- Ondersteunde formaten: Markdown, ODF, PDF
- Max grootte: 10 MB

3.2 Download via API proxy
- Server haalt document uit S3
- Streamt naar client (geen buffering in memory)
- Valideer document access voor download

3.3 S3 client configuratie
- Environment variables: `S3_ENDPOINT`, `S3_ACCESS_KEY`, `S3_SECRET_KEY`, `S3_BUCKET`
- MinIO compatible met path-style URLs
- Connection pooling voor efficiency

**Bestanden:**
- Nieuw: `crates/iou-core/src/storage/mod.rs` - S3 client abstraction
- Nieuw: `crates/iou-core/src/storage/s3.rs` - S3 implementatie
- `crates/iou-api/src/routes/documents.rs` - Upload/download logica

**Dependencies:**
```toml
rust-s3 = { version = "0.37", features = ["tokio", "fail-on-err"] }
```

### 4. WebSocket Support

**Functionele Requirements:**

4.1 WebSocket endpoint
- `GET /ws/documents/{id}` - Subscribe op document updates
- Clients ontvangen real-time status updates
- Automatic disconnect na document completion

4.2 Per-document broadcast channel
- Channel voor elk document (isolate updates)
- Broadcast updates: agent started, progress, completed, failed
- Multiple clients kunnen subscriben opzelfde document

4.3 Connection lifecycle
- Idle timeout: 5 minuten zonder berichten
- Heartbeat ping every 30 seconden
- Graceful disconnect bij completion

**Bestanden:**
- Nieuw: `crates/iou-api/src/websockets/mod.rs` - WebSocket handler
- Nieuw: `crates/iou-api/src/websockets/documents.rs` - Document status updates

**Dependencies:**
```toml
tokio-tungstenite = "0.21"
futures-util = "0.3"
```

## Architecture

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ HTTP/WebSocket
       ▼
┌─────────────────────────────────┐
│         API Server (Axum)        │
│  ┌────────────────────────────┐ │
│  │   routes/documents.rs      │ │
│  │   - create_document()      │ │
│  │   - approve_document()     │ │
│  │   - download_document()    │ │
│  └───────────┬────────────────┘ │
│              │                   │
│  ┌───────────▼────────────────┐ │
│  │   workflows/executor.rs    │ │
│  │   - Async pipeline runner  │ │
│  │   - Broadcast channel      │ │
│  └───────────┬────────────────┘ │
│              │                   │
│  ┌───────────▼────────────────┐ │
│  │  iou-ai/agents/pipeline.rs │ │
│  │  - Research Agent          │ │
│  │  - Content Agent           │ │
│  │  - Compliance Agent        │ │
│  │  - Review Agent            │ │
│  └───────────┬────────────────┘ │
└──────────────┼──────────────────┘
               │
       ┌───────┴────────┐
       ▼                ▼
┌─────────────┐  ┌─────────────┐
│  DuckDB     │  │  S3/MinIO   │
│  (metadata) │  │  (files)    │
└─────────────┘  └─────────────┘
```

## Technical Decisions

### Pipeline Execution
- **Tokio task spawning** voor asynchrone uitvoering in prod
- **Direct await** in test mode (feature flag)
- **Broadcast channel** (`tokio::sync::broadcast`) voor status updates
- **Timeout**: 2 minuten per agent met `tokio::time::timeout`

### Storage
- **rust-s3** crate v0.37+ met tokio feature
- **API Proxy pattern** voor downloads (geen presigned URLs)
- **Stream processing** om memory gebruik te minimaliseren
- **Environment variables** voor S3 configuratie

### WebSocket
- **tokio-tungstenite** voor WebSocket support
- **Per-document channel** voor isolatie
- **5-minute idle timeout** met heartbeat
- **Automatic cleanup** bij completion

## Data Model Changes

### Document Metadata (uitgebreid)
```rust
pub struct DocumentMetadata {
    pub id: Uuid,
    pub domain_id: String,
    pub document_type: String,
    pub state: WorkflowStatus,
    pub current_agent: Option<String>,      // NEW
    pub progress_percent: u8,                // NEW
    pub current_version_key: String,
    pub previous_version_key: Option<String>,
    pub compliance_score: f32,
    pub confidence_score: f32,
    pub created_by: Option<Uuid>,            // NEW
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Audit Entry (uitgebreid)
```rust
pub struct AuditEntry {
    pub document_id: Uuid,
    pub agent_name: String,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<Uuid>,               // NEW
    pub details: serde_json::Value,
}
```

## Success Criteria

- [ ] Document creatie start automatisch de agent pipeline (async)
- [ ] Pipeline timeout na 2 minuten per agent
- [ ] Approval endpoint werkt alleen met ObjectApprover role
- [ ] Documenten worden opgeslagen in S3 na completion (max 10MB)
- [ ] Download proxy haalt uit S3 en streamt naar client
- [ ] Clients ontvangen real-time status updates via WebSocket
- [ ] WebSocket verbinding sluit na 5 min idle
- [ ] Alle tests passsen (`cargo test`)
- [ ] Audit trail bevat alle acties met user IDs
- [ ] Test mode werkt synchroon

## Open Questions (Resolved)

1. ~~Moet de pipeline synchroon draaien voor testing?~~ **Ja** (feature flag)
2. ~~Hoe lang moeten WebSocket connecties open blijven bij idle?~~ **5 minuten**
3. ~~Moeten we S3 presigned URLs gebruiken of proxy via API?~~ **API Proxy**
4. ~~Wat is de maximum grootte voor een document?~~ **10 MB**
