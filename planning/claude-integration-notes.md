# Integration Notes: Opus Review Feedback

## Date: 2026-03-01
## Reviewer: Claude Opus 4 (via subagent)

---

## Summary

The Opus review identified 18 significant issues across architectural duplication, security gaps, missing considerations, and performance concerns. This document records which suggestions are being integrated into the plan and why.

---

## Integrated Suggestions

### 1. State Machine Duplication (Section 1.1) ✅ INTEGRATING

**Issue:** New `DocumentState` enum conflicts with existing `WorkflowStatus` enum.

**Integration:** Update plan to reuse `WorkflowStatus` from `iou-core/src/workflows.rs`. Mapping:
- `Drafting` → `Draft`
- `PendingApproval` → `Submitted` or `InReview`
- `Approved` → `Approved`
- `Rejected` → `Rejected`
- `Published` → `Published`

**Rationale:** Code reuse prevents duplication bugs and maintains consistency.

### 2. Missing S3 Dependencies (Section 1.2) ✅ INTEGRATING

**Issue:** No S3/MinIO client library in workspace dependencies.

**Integration:** Add to Phase 1 tasks: Add `rust-s3` or `aws-sdk-s3` to workspace Cargo.toml.

**Rationale:** Cannot implement iou-storage crate without S3 client dependency.

### 3. Knowledge Graph Schema Definition (Section 1.3) ✅ INTEGRATING

**Issue:** No Document entity type in existing GraphRAG schema.

**Integration:** Add to Phase 1 tasks: Define Document entity schema for GraphRAG with attributes (type, domain, created_at).

**Rationale:** Research Agent requires ability to query similar documents.

### 4. PII Redaction Specification (Section 2.1) ✅ INTEGRATING

**Issue:** Redaction method undefined, potential AVG/GDPR violation.

**Integration:** Add to Compliance Agent specification:
- Irreversible redaction using `[PII: <type>]` placeholders
- Separate storage for original vs redacted
- Role-based access for viewing originals

**Rationale:** Critical privacy compliance for Dutch government documents.

### 5. Auto-Approval Security (Section 2.2) ✅ INTEGRATING

**Issue:** Auto-approval with 0.95 threshold is dangerous for Woo documents.

**Integration:** Modify approval logic:
- ALL Woo-relevant documents require human approval regardless of confidence
- Auto-approval only for internal, non-sensitive documents
- Add dry-run mode for testing

**Rationale:** Government documents require human oversight for legal compliance.

### 6. Authentication on Approval Endpoint (Section 2.3) ✅ INTEGRATING

**Issue:** No auth/authz specification for approval endpoint.

**Integration:** Add to API specification:
- Authentication middleware requirement
- Role-based access control (approver role only)
- MFA for high-sensitivity domains
- Approval audit trail

**Rationale:** Unauthorized approvals could publish non-compliant documents.

### 7. Document Versioning Table (Section 3.1) ✅ INTEGRATING

**Issue:** No provision for full version history, diffs, or rollback.

**Integration:** Add to database schema: `document_versions` table with version_number, change_summary, rollback capability.

**Rationale:** Government documents need complete version history for accountability.

### 8. Error Recovery and Retry Logic (Section 3.6) ✅ INTEGRATING

**Issue:** Fail-fast approach creates poor UX for transient failures.

**Integration:** Add to Pipeline Orchestration:
- Distinguish permanent vs transient errors
- Exponential backoff retry for transient errors
- Checkpoint/restart capability for long pipelines

**Rationale:** Improves reliability and user experience.

### 9. Duplicate Compliance Types (Section 5.1) ✅ INTEGRATING

**Issue:** New compliance types overlap with existing types.

**Integration:** Modify data models to reuse existing types from `compliance.rs`:
- `WooMetadata`, `WooRefusalGround`, `WooDisclosureClass`
- `AvgMetadata`, `PrivacyLevel`
- `ComplianceStatus`, `ComplianceIssue`

**Rationale:** Code reuse and consistency with existing codebase.

### 10. Audit Query Performance (Section 4.3) ✅ INTEGRATING

**Issue:** Large audit table could miss <100ms SLA.

**Integration:** Add to database schema:
- Partitioning by `created_at`
- Audit trail archiving strategy
- Consider separate audit log storage for long-term retention

**Rationale:** Performance requirement must be met for production.

### 11. SQL Index Optimization (Section 7.1) ✅ INTEGRATING

**Issue:** Missing compound index for common queries.

**Integration:** Add index on `(domain_id, state)` for efficient "all pending documents for domain" queries.

**Rationale:** Simple optimization with significant performance impact.

### 12. AI Provider Configuration (Section 5.3) ✅ INTEGRATING

**Issue:** No specification for AI provider, key management, fallback.

**Integration:** Add new section "AI Provider Configuration" covering:
- Provider options (OpenAI, Anthropic, local models)
- API key management via environment variables
- Fallback providers
- Cost tracking

**Rationale:** Production system needs explicit AI provider strategy.

---

## Suggestions NOT Integrated (with Rationale)

### 1. Template Injection Vulnerability (Section 2.4) ❌ NOT INTEGRATING

**Feedback:** Specify template sandboxing and sanitization.

**Rationale:** Tera templates are sandboxed by design. User context is not template code but data. Sanitization happens at rendering, not at definition. Existing Tera security is sufficient.

### 2. Concurrent Document Editing (Section 3.2) ❌ NOT INTEGRATING

**Feedback:** Add optimistic locking or checkout/checkin.

**Rationale:** The proposed workflow is sequential agent-driven, not collaborative human editing. Human approval is a final decision point, not an editing session. Locking adds complexity without clear benefit for the current design.

### 3. Document Lifecycle After Publication (Section 3.3) ❌ NOT INTEGRATING

**Feedback:** Define unpublication, archival, retention workflow.

**Rationale:** These are separate concerns beyond the scope of the document creation system. They belong in the existing PROVISA management system. The plan correctly focuses on creation through publication.

### 4. Rate Limiting and Resource Management (Section 3.4) ❌ NOT INTEGRATING

**Feedback:** Add rate limits, job queue, cost tracking.

**Rationale:** This is infrastructure-level concern, not application-level. Rate limiting should be handled by API gateway/reverse proxy, not the application. Job queues add complexity not justified by expected load.

### 5. Localization and Multi-Language Support (Section 3.5) ❌ NOT INTEGRATING

**Feedback:** Support Dutch, English, Frisian variants.

**Rationale:** Out of scope for initial implementation. Templates are domain-specific; multi-language can be added via separate templates per language. This is a feature, not a foundational requirement.

### 6. Agent Sequential Bottleneck (Section 4.1) ❌ NOT INTEGRATING

**Feedback:** Identify parallelizable operations, add timeouts.

**Rationale:** Sequential execution is inherent to the workflow design—research must inform content, which must be validated before review. Artificial parallelization would break correctness. Timeouts are already covered in error handling.

### 7. Markdown to ODF Conversion (Section 4.2) ❌ NOT INTEGRATING

**Feedback:** Benchmark options early, consider hybrid approach.

**Rationale:** The plan correctly identifies this as an open question. Implementation decision should be made in Phase 2 based on actual testing. Pre-deciding without benchmarking would be premature optimization.

### 8. Knowledge Graph Client Trait (Section 5.2) ❌ NOT INTEGRATING

**Feedback:** Define GraphRagClient trait abstraction.

**Rationale:** The existing `KnowledgeGraph` struct in `iou-ai` can be extended. Creating a new trait abstraction adds indirection without clear benefit at this stage. YAGNI principle applies.

### 9. Trust Level Assignment Governance (Section 6.1) ❌ NOT INTEGRATING

**Feedback:** Specify who decides, how to change, scope of trust levels.

**Rationale:** This is a policy/business process question, not a technical implementation concern. The technical system stores and enforces trust levels; governance is organizational.

### 10. Template Variable Resolution (Section 6.2) ❌ NOT INTEGRATING

**Feedback:** Define variable extraction, priority, conflict handling.

**Rationale:** The existing `VariableSource` enum with priority order (UserInput > KnowledgeGraph > AgentGenerated > Default) is sufficient. Additional specification would be over-engineering.

### 11. Context HashMap Permissiveness (Section 6.3) ❌ NOT INTEGRATING

**Feedback:** Define schema for context validation per document type.

**Rationale:** Per-document-type schemas would require a schema definition language and validation engine. The HashMap approach is flexible and allows gradual typing. Schema validation can be added later if needed.

### 12. UUID v7 Recommendation (Section 7.2) ❌ NOT INTEGRATING

**Feedback:** Use UUID v7 for time-ordered IDs.

**Rationale:** UUID v4 is used consistently throughout the IOU-Modern codebase. Changing to v7 for one subsystem creates inconsistency. Indexing performance difference is negligible for expected document volumes.

### 13. Document Cancellation State (Section 7.3) ❌ NOT INTEGRATING

**Feedback:** Add explicit `Cancelled` state.

**Rationale:** The existing `Rejected` state can handle documents abandoned before completion. Adding a separate state creates a distinction without meaningful difference in workflow.

---

## Summary Statistics

| Category | Count |
|----------|-------|
| Integrated | 12 |
| Not Integrated | 13 |
| Total Issues | 25 |

**Integration Rate:** 48% (12/25)

The integration focuses on critical security, architectural consistency, and production readiness concerns. Suggestions deferred are primarily around premature optimization, out-of-scope features, or over-engineering.
