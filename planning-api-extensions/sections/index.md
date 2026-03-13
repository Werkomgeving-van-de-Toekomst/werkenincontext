<!-- PROJECT_CONFIG
runtime: rust-cargo
test_command: cargo test
END_PROJECT_CONFIG -->

<!-- SECTION_MANIFEST
section-01-foundation
section-02-orchestrator
section-03-authentication
section-04-s3-storage
section-05-websocket
section-06-integration-testing
section-07-ronl-business-api
END_MANIFEST -->

# Implementation Sections Index

## Dependency Graph

| Section | Depends On | Blocks | Parallelizable |
|---------|------------|--------|----------------|
| section-01-foundation | - | section-02, section-04, section-07 | Yes |
| section-02-orchestrator | 01 | section-05, section-07 | No |
| section-03-authentication | - | section-06, section-07 | Yes |
| section-04-s3-storage | 01 | section-06 | No |
| section-05-websocket | 01, 02 | section-06 | No |
| section-06-integration-testing | 02, 03, 04, 05 | - | No |
| section-07-ronl-business-api | 01, 02, 03 | - | No |

## Execution Order

**Batch 1 (Parallel):**
1. section-01-foundation (no dependencies)
2. section-03-authentication (no dependencies)

**Batch 2 (After 01):**
3. section-02-orchestrator (after foundation)

**Batch 3 (After 01):**
4. section-04-s3-storage (after foundation)

**Batch 4 (After 01, 02):**
5. section-05-websocket (after foundation + orchestrator)

**Batch 5 (After 01, 02, 03):**
7. section-07-ronl-business-api (after foundation, orchestrator, authentication)

**Batch 6 (Final - after 02, 03, 04, 05):**
6. section-06-integration-testing (depends on all previous sections)

## Section Summaries

### section-01-foundation
Setup of base infrastructure for all extensions. Includes adding dependencies (tokio-tungstenite, rust-s3, iou-orchestrator), creating S3 client abstraction, WebSocket handler structure, and configuration loading with S3 validation at startup.

**Files:**
- `crates/iou-core/src/storage/mod.rs` - Storage module
- `crates/iou-core/src/storage/s3.rs` - S3 client with startup validation
- `crates/iou-api/src/websockets/mod.rs` - WebSocket module
- `crates/iou-api/src/orchestrator/mod.rs` - Orchestrator wrapper module
- `crates/iou-api/Cargo.toml` - Add dependencies
- `crates/iou-core/Cargo.toml` - Add rust-s3
- `crates/iou-api/src/config.rs` - S3 config loading

### section-02-orchestrator
Integrates the existing `iou-orchestrator` crate for workflow execution. Creates wrapper around `WorkflowStateMachine`, implements broadcast channel for status updates, adds overall timeout handling (8 min), implements idempotency check, and maps workflow states to document states.

**Files:**
- `crates/iou-api/src/orchestrator/wrapper.rs` - Orchestrator wrapper
- `crates/iou-api/src/orchestrator/types.rs` - State mappings
- `crates/iou-api/src/routes/documents.rs` - Integrate orchestrator

### section-03-authentication
Adds RBAC checks to document endpoints. Includes AuthContext extraction, role-based permission checks (ObjectCreator, DomainEditor, ObjectApprover), organization-based access control, user_id in audit trail, and JWT secret from environment.

**Files:**
- `crates/iou-api/src/routes/documents.rs` - Role checks, org scoping
- `crates/iou-api/src/middleware/auth.rs` - JWT secret from env

### section-04-s3-storage
Implements document storage in S3/MinIO. Includes S3 client with path-style URLs, upload after pipeline completion, streaming download proxy (no presigned URLs), size validation (10MB limit), proper error conversion layer, and startup validation.

**Files:**
- `crates/iou-core/src/storage/s3.rs` - S3 operations with streaming
- `crates/iou-api/src/routes/documents.rs` - Streaming download
- `crates/iou-api/src/main.rs` - Startup validation

### section-05-websocket
Real-time status updates via WebSocket. Includes per-document broadcast subscription with bounded channel (cap 100), connection limit per document (max 10), status message serialization, idle timeout (5 min) with heartbeat, graceful disconnect on completion, and panic recovery.

**Files:**
- `crates/iou-api/src/websockets/documents.rs` - Document WebSocket handler
- `crates/iou-api/src/websockets/types.rs` - Message types
- `crates/iou-api/src/websockets/limiter.rs` - Connection limiter
- `crates/iou-api/src/main.rs` - WebSocket route
- `crates/iou-api/src/orchestrator/wrapper.rs` - Broadcast integration

### section-06-integration-testing
Full integration and testing. Includes wiring all components in main.rs, end-to-end tests for complete flow, unit tests for each component, error handling tests, performance tests for concurrent requests, and mock infrastructure for S3 and pipeline testing.

**Files:**
- `crates/iou-api/src/main.rs` - Wire everything together
- `crates/iou-api/tests/mocks/s3.rs` - Mock S3 client
- `crates/iou-api/tests/mocks/orchestrator.rs` - Mock orchestrator
- `crates/iou-api/tests/helpers/websocket.rs` - WebSocket test helper

### section-07-ronl-business-api
RONL Business API Layer with SSI & Business Rules. Implements Self-Sovereign Identity using EBSI/nl-wallet Verifiable Credentials, multi-tenant tenancy isolation, audit logging, and BPMN/DMN business rules integration via `iou-regels` crate.

**Files:**
- `crates/iou-core/src/ssi/mod.rs` - SSI/VC module
- `crates/iou-core/src/tenancy/mod.rs` - Multi-tenant isolation
- `crates/iou-core/src/audit/mod.rs` - Audit logging
- `crates/iou-api/src/routes/v1/rules.rs` - Rules evaluation endpoints
- `crates/iou-api/src/routes/v1/calculations.rs` - Calculation endpoints
- `crates/iou-api/src/middleware/vc.rs` - VC validation middleware

## Key Architectural Decisions

1. **Orchestrator Integration**: Uses existing `iou-orchestrator` crate instead of creating duplicate executor
2. **Async-Only API**: Removed synchronous mode - all pipeline execution is async with WebSocket updates
3. **Organization Scoping**: All document access is scoped to user's organization
4. **Streaming Downloads**: S3 downloads stream directly to client without buffering
5. **Bounded Channels**: WebSocket broadcast uses bounded channel (100) to prevent memory growth
6. **Connection Limits**: Max 10 WebSocket connections per document
7. **Fail-Fast Validation**: S3 connectivity validated at startup, not on first request
