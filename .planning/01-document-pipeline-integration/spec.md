# Spec: Document Pipeline Integration

**Split:** 01-document-pipeline-integration
**Created:** 2026-03-14
**Estimated Duration:** 1-2 sprints

## Overview

Complete the multi-agent document pipeline by integrating S3 storage, real-time WebSocket updates, document preview, and audit logging. The pipeline orchestration code exists but has TODO markers for critical integration points.

## Context

### Current State

The agent pipeline (`crates/iou-ai/src/agents/pipeline.rs`) implements the full workflow:
1. **Research Agent** - Queries knowledge graph, determines structure
2. **Content Agent** - Generates document from template
3. **Compliance Agent** - Validates Woo rules, detects PII
4. **Review Agent** - Quality check, decides approval

However, critical integration is incomplete:
- Line 288: `TODO: Store document in S3`
- Line 289: `TODO: Update database state`
- Line 290: `TODO: Add audit trail logging`
- Line 511: `TODO: Store checkpoint in persistent storage`
- Line 532: `TODO: Load checkpoint from persistent storage`

### Existing Infrastructure

- **S3 Client:** Complete implementation in `crates/iou-storage/src/s3.rs`
- **Template Engine:** Tera-based in `crates/iou-ai/src/templates.rs`
- **WebSocket:** Infrastructure exists but not integrated with pipeline
- **Database:** PostgreSQL with DuckDB for analytics

## Requirements

### Core Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| PIPE-01 | S3 Document Storage | Store generated documents in S3/MinIO with versioning |
| PIPE-02 | Real-time Status Updates | Broadcast pipeline status via WebSocket to connected clients |
| PIPE-03 | Document Preview | Render document preview before final approval |
| PIPE-04 | Audit Logging | Complete audit trail for all pipeline stages |
| PIPE-05 | Checkpoint Persistence | Save/load pipeline checkpoints for recovery |
| PIPE-06 | Database State Updates | Update document status in PostgreSQL after each stage |

### User Stories

1. **As a content creator**, I want to see real-time progress of my document so I know when it's ready for review.
2. **As a content creator**, I want to preview my document before publishing so I can catch errors early.
3. **As a compliance officer**, I want a complete audit trail so I can verify Woo/GDPR compliance.
4. **As a system administrator**, I want documents stored reliably in S3 so they persist across deployments.

### Acceptance Criteria

- [ ] Documents are stored in S3 with proper key structure (`documents/{id}/v{version}.{format}`)
- [ ] WebSocket clients receive status updates for each pipeline stage
- [ ] Frontend displays a progress indicator showing current stage
- [ ] Document preview renders the same output as final generation
- [ ] All pipeline stages log user, timestamp, and result to audit table
- [ ] Checkpoints are saved after each agent completes
- [ ] Pipeline can resume from last checkpoint after failure

## Technical Details

### Storage Keys

```
documents/{document_id}/v{version}.md          # Original
documents/{document_id}/v{version}.redacted.md # Redacted (if PII)
documents/{document_id}/checkpoint.json        # Pipeline state
```

### WebSocket Message Format

```json
{
  "type": "pipeline_status",
  "document_id": "uuid",
  "stage": "Research|Content|Compliance|Review|Complete",
  "status": "Started|Progress|Complete|Failed",
  "timestamp": "ISO8601",
  "metadata": {}
}
```

### Audit Log Schema

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Unique audit entry |
| document_id | UUID | Related document |
| stage | varchar | Pipeline stage name |
| action | varchar | Action performed |
| user_id | UUID | User who initiated |
| timestamp | timestamptz | When action occurred |
| result | json | Action result data |

## Dependencies

### From Codebase

- `iou-storage` S3 client (already implemented)
- `iou-core` document types
- `iou-api` WebSocket infrastructure

### External Services

- S3-compatible storage (AWS S3 or MinIO)
- PostgreSQL database
- WebSocket server (Axum)

## Out of Scope

- Template editing (covered in Split 02)
- Advanced AI features (embeddings, ML-based NER)
- Analytics dashboards
- External integrations (PDF, government APIs)

## Constraints

- Must use existing S3 client implementation
- Must maintain Woo/GDPR compliance in audit logs
- WebSocket reconnection must be graceful
- Preview must use same rendering engine as production (Tera)

## Success Metrics

| Metric | Target |
|--------|--------|
| Pipeline success rate | >95% |
| Average pipeline duration | <30 seconds |
| WebSocket message delivery | >99% |
| Audit log completeness | 100% |

---

## References

- Original requirements: `.planning/future-features-requirements.md`
- Interview: `.planning/deep_project_interview.md`
- Pipeline code: `crates/iou-ai/src/agents/pipeline.rs`
- S3 client: `crates/iou-storage/src/s3.rs`
