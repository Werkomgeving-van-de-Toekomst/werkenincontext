# Research Summary: Enhanced Document Workflow

This document combines findings from codebase analysis, web research, and testing research for implementing multi-stage approvals, delegation, expiry handling, and version diff features.

## Codebase Analysis

### Existing Workflow Engine

**Key Files:**
- `/Users/marc/Projecten/iou-modern/crates/iou-core/src/workflows.rs` - Core workflow types
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/document_workflow.rs` - Implementation
- `/Users/marc/Projecten/iou-modern/crates/iou-orchestrator/src/state_machine.rs` - State machine

**Current States:**
```rust
pub enum WorkflowStatus {
    Draft, Submitted, InReview, ChangesRequested, 
    Approved, Published, Rejected, Archived
}
```

**Transition Logic:** Strict validation with `can_transition_to()` method. Supports both Rust (built-in) and Camunda (Zeebe) workflow drivers.

### Database Schema

**Key Tables:**
1. **documents** - Core metadata (id, domain_id, state, trust_level, version_keys)
2. **document_audit** - Audit trail (7-year retention for compliance)
3. **document_versions** - Version history with rollback support
4. **templates** - Document templates per domain/type
5. **domain_configs** - Per-domain workflow configuration

**Storage:** DuckDB (embedded) + Supabase PostgreSQL for realtime.

### Approval Queue UI

**Route:** `/documenten/wachtrij` (Dutch interface)

**Features:**
- Document listing with status badges
- Approve/Reject with comments
- Audit trail preview
- Auto-refresh via WebSocket
- Removes from queue after action

### Trust Level System

```rust
pub enum TrustLevel {
    Low,    // Always requires human approval
    Medium, // Requires approval if compliance_score < threshold
    High,   // Auto-approval for non-Woo documents only
}
```

**Rules:**
- Woo documents: Always require human approval
- Internal documents: Auto-approve based on compliance score
- Domain-specific thresholds configurable

### Audit Trail

**Location:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/audit/`

**Features:**
- Write-ahead semantics (entries written BEFORE action)
- Immutable entries with 7-year retention
- Tamper-evident design for BIO/NEN 7510/AVG compliance
- Tracks: user DID, action, outcome, IP, user agent

### Storage Implementation

**S3/MinIO:** Stub implementation with 10MB limit
**Supabase Storage:** Fully implemented with signed URLs, public/private buckets

**Version Management:**
- Current and previous version keys
- Format tracking (Markdown, ODF, PDF)
- Compliance/confidence scores per version

### Current Limitations

1. **Single-stage approval** - No sequential/parallel stages
2. **No delegation** - Cannot reassign approval authority
3. **No expiry/escalation** - No deadline tracking
4. **Basic versioning** - No diff visualization or rollback UI

## Web Research: Best Practices & Libraries

### Multi-Stage Approval Patterns

**Sequential Approval:** Chain of approvers (manager → director → legal)

**Parallel Approval:** Multiple approvers with voting logic:
- ALL must approve
- MAJORITY must approve
- ANY ONE can approve

**Fork-Join Pattern:** Sequential stages with parallel subprocesses

**Recommended Data Model:**
```rust
pub struct ApprovalStage {
    pub stage_id: Uuid,
    pub stage_name: String,
    pub approvers: Vec<Uuid>,
    pub approval_type: ApprovalType, // Sequential, ParallelAny, ParallelAll, ParallelMajority
    pub timeout_hours: i32,
    pub required_count: Option<usize>,
}
```

### Delegation Models

**Types:**
- **Temporary:** Date-range based (vacation, sick leave)
- **Permanent:** Role changes, departures
- **Bulk:** All pending approvals during absence

**Data Model:**
```rust
pub struct Delegation {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub delegation_type: DelegationType,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub document_types: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}
```

**Audit Trail:**
- Always record original approver
- Display "Approved by Jane Doe (delegated from John Smith)"
- Limit delegation chains to N hops for security

### Business Day Calculations

**Recommended Rust Crates:**

1. **`business`** - Most mature, GoCardless Ruby patterns
   ```rust
   use business::Calendar;
   let cal = Calendar::with_holidays(&[xmas_date]);
   cal.add_business_days(start_date, 3);
   ```

2. **`bdays`** - Holiday calendar caching, multiple country support

3. **`workdays`** - Excel WORKDAY equivalent, custom work weeks

**Dutch Holidays:**
- Fixed: New Year's, King's Day (April 27), Christmas
- Variable: Easter, Ascension, Pentecost

### Diff Visualization Libraries

**Backend (Rust):**
1. **`similar`** - Fast, feature-rich, multiple algorithms
   ```rust
   use similar::{ChangeTag, TextDiff};
   let diff = TextDiff::from_lines(old, new);
   ```

2. **`imara-diff`** - SIMD optimized, extremely fast

3. **`dissimilar`** - Minimal dependencies, good for WASM

**Frontend (WASM):**
- Compile `similar` to WASM (minimal dependencies)
- Or JavaScript interop with `diff` npm package

**Diff Format Options:**
```rust
pub enum DiffFormat {
    Unified,    // Git-style
    Inline,     // Within paragraphs
    SideBySide, // Two-column view
}
```

### Version History Patterns

**Storage Approaches:**
1. **Full Version Storage** - Simple, for small docs
2. **Delta Storage** - Efficient, for large docs
3. **Hybrid** (Recommended) - Last N versions full, older compressed

**Version Numbering:**
- Semantic: {major, minor, patch}
- Sequential: v1, v2, v3...
- Timestamp: ISO date based

**Database Schema Extension:**
```sql
CREATE TABLE document_versions (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id),
    version_number VARCHAR(50) NOT NULL,
    storage_key VARCHAR(500) NOT NULL,
    parent_version_id UUID REFERENCES document_versions(id),
    change_summary TEXT,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    is_current BOOLEAN DEFAULT false,
    UNIQUE(document_id, version_number)
);
```

## Testing Analysis

### Test Framework

- **Rust built-in** with `#[tokio::test]` for async
- **No external framework** - native Rust testing
- Test organization: Co-located in `tests/` directories at crate level

### Existing Test Coverage

**Workflow Tests:**
- Basic state transition tests in `iou-core/src/workflows.rs`
- Document flow tests in `iou-api/tests/integration/document_flow.rs`
- WebSocket broadcast functionality
- Size validation (10MB limit)
- Workflow timeout handling (8 minutes)

**Database Tests:**
- Schema equivalence tests (PostgreSQL/DuckDB match)
- Integration tests with real databases
- No separate test databases

**Integration Tests:**
- Auth tests
- Document lifecycle
- Storage tests (S3 upload/download)
- Concurrency testing

### Frontend Testing

**Current State:**
- **No test framework** for Dioxus frontend
- `serial_test = "3"` dependency available
- No test files in frontend crate

### Test Patterns

**Common utilities:**
- `random_document_id()`, `random_user_id()` helpers
- `MockS3Client` for testing without real S3
- `WebSocketTestClient` helper
- Async testing with `tokio::spawn`

**Test organization:**
```
crates/iou-api/tests/
├── integration/     # E2E tests
├── concurrency/    # Concurrency tests
├── cleanup/        # Migration tests
├── mocks/          # Mock implementations
└── helpers/        # Test utilities
```

### Recommendations for Testing New Features

1. **Create dedicated workflow test module** with submodules for each feature
2. **Extend workflow state tests** for multi-stage transitions
3. **Add workflow integration tests** for E2E approval flows
4. **Create test seed data** for different workflow scenarios
5. **Add Dioxus testing** for frontend components
6. **Add performance tests** for large approval workflows
7. **Add security tests** for authorization and audit logging

## Implementation Priority

**Phase 1 (Foundation):**
1. Add `business` crate for SLA calculations
2. Extend `WorkflowStateMachine` for multi-stage approvals
3. Implement delegation data model

**Phase 2 (Core Features):**
4. Add parallel approval support
5. Implement temporary delegation with auto-expiry
6. Basic version history table

**Phase 3 (Advanced):**
7. Add `similar` crate for diff visualization
8. Implement restore functionality
9. Bulk delegation and advanced escalation rules

## Key Files for Modification

| File | Purpose |
|------|---------|
| `iou-orchestrator/src/state_machine.rs` | Core state machine - extend for multi-stage |
| `iou-core/src/workflows.rs` | Workflow types - add stage definitions |
| `iou-core/src/document.rs` | Document metadata - add version history |
| `iou-api/src/routes/documents.rs` | Approval endpoints - extend for delegation |
| `iou-frontend/src/pages/approval_queue.rs` | Frontend UI - add multi-stage/diff views |

## Summary

The IOU-Modern codebase has a solid foundation with:
- Well-structured workflow state machine
- Comprehensive audit trail system
- Dual database architecture (DuckDB + Supabase)
- Real-time WebSocket updates

The main enhancement areas are:
1. Multi-stage approval workflows (extend state machine)
2. Delegation system (new tables + logic)
3. Expiry/escalation (SLA calculation + scheduled jobs)
4. Version diff visualization (`similar` crate + UI components)

Recommended Rust libraries:
- `business` - SLA/business day calculations
- `similar` - Text diffing and visualization
- Existing tokio/axum stack for async operations

Testing approach:
- Extend existing integration test patterns
- Add workflow-specific test modules
- Consider Dioxus testing for frontend components
- Add performance and security tests
