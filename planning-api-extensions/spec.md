# Agent Orchestration API Extensions

## Overview

Uitbreiding van de bestaande document API om de agent pipeline volledig functioneel te maken met authenticatie, storage, en real-time updates.

## Current State

De basis API structuur bestaat in `crates/iou-api/src/routes/documents.rs`:
- POST /api/documents/create - Maakt document record maar roept geen pipeline aan
- GET /api/documents/{id}/status - Status查询
- POST /api/documents/{id}/approve - Goedkeuring workflow
- GET /api/documents/{id}/audit - Audit trail
- GET /api/documents/{id}/download - Download (TODO: S3)

**Ontbrekende functionaliteit (gemarkeerd als TODO):**
1. Pipeline executor integration
2. Authentication checks
3. S3 storage integration
4. Real-time status updates

## Requirements

### 1. Pipeline Executor Integration

**Doel:** Wanneer een document wordt aangemaakt, moet de agent pipeline daadwerkelijk worden uitgevoerd.

**Specificatie:**
- Asynchroon uitvoeren van de pipeline (niet blokkeren voor API response)
- Pipeline stappen: Research → Content → Compliance → Review
- Status updates opslaan in database tijdens processing
- Foutafhandeling voor timeouts en agent failures

**Bestanden:**
- `crates/iou-ai/src/agents/pipeline.rs` - Bestaande pipeline implementation
- `crates/iou-api/src/routes/documents.rs` - Integration punt
- Nieuw: `crates/iou-api/src/workflows/executor.rs` - Async executor wrapper

### 2. Authentication Integration

**Doel:** Beveilig de API endpoints met role-based access control.

**Specificatie:**
- `/api/documents/{id}/approve` vereist `object_approver` role
- `/api/documents/create` vereist authenticated user
- Auth context uitlezen uit JWT middleware
- User ID opslaan in audit trail

**Bestanden:**
- `crates/iou-api/src/middleware/auth.rs` - Bestaande auth module
- `crates/iou-api/src/routes/documents.rs` - Voeg auth checks toe

### 3. S3 Storage Integration

**Doel:** Sla gegenereerde documenten op in S3/MinIO.

**Specificatie:**
- Upload document na pipeline completion
- Ondersteunde formaten: Markdown, ODF, PDF
- Download endpoint haalt uit S3
- Versioning support

**Bestanden:**
- Nieuw: `crates/iou-core/src/storage/mod.rs` - S3 client abstraction
- Nieuw: `crates/iou-core/src/storage/s3.rs` - S3 implementatie
- `crates/iou-api/src/routes/documents.rs` - Upload/download logica

### 4. WebSocket Support

**Doel:** Real-time status updates voor clients die document creatie volgen.

**Specificatie:**
- WebSocket endpoint: /ws/documents/{id}
- Clients subscriben op document ID
- Server push updates bij status veranderingen
- Automatic disconnect na document completion

**Bestanden:**
- Nieuw: `crates/iou-api/src/websockets/mod.rs` - WebSocket handler
- Nieuw: `crates/iou-api/src/websockets/documents.rs` - Document status updates

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
- **Tokio task spawning** voor asynchrone uitvoering
- **Broadcast channel** voor status updates naar WebSocket subscribers
- **Timeout handling** met `tokio::time::timeout`

### Storage
- **rust-s3** crate voor S3 client
- **Environment variables** voor S3 configuratie
- **Presigned URLs** voor secure downloads

### WebSocket
- **tokio-tungstenite** voor WebSocket support
- **Per-document channel** voor isolatie
- **Connection pooling** met automatic cleanup

## Dependencies

**Nieuwe Cargo dependencies:**
```toml
[dependencies]
tokio-tungstenite = "0.21"
futures-util = "0.3"
rust-s3 = "0.33"
```

## Success Criteria

- [ ] Document creatie start automatisch de agent pipeline
- [ ] Approval endpoint werkt alleen met object_approver role
- [ ] Documenten worden opgeslagen in S3 na completion
- [ ] Clients ontvangen real-time status updates via WebSocket
- [ ] Alle tests passsen (`cargo test`)
- [ ] Audit trail bevat alle acties met user IDs

## Open Questions

1. Moet de pipeline synchroon draaien voor testing?
2. Hoe lang moeten WebSocket connecties open blijven bij idle?
3. Moeten we S3 presigned URLs gebruiken of proxy via API?
4. Wat is de maximum grootte voor een document?
