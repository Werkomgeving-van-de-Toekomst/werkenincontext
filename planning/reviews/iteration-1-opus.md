# Opus Review

**Model:** claude-opus-4
**Generated:** 2026-03-01T12:09:00Z

---

# Document Creation Agents Implementation Plan - Review

## Executive Summary

This is a well-structured plan with a clear architectural vision. However, I've identified several significant architectural concerns, missing security considerations, and potential footguns that should be addressed before implementation begins.

---

## 1. Critical Architectural Issues

### 1.1 State Machine Duplication (Section 2.3)

**Issue:** The plan defines a new `DocumentState` enum that conflicts with the existing `WorkflowStatus` enum in `/Users/marc/Projecten/iou-modern/crates/iou-core/src/workflows.rs`.

**Existing states:** `Draft`, `Submitted`, `InReview`, `ChangesRequested`, `Approved`, `Published`, `Rejected`, `Archived`

**Plan states:** `Drafting`, `PendingApproval`, `Approved`, `Rejected`, `Published`

**Recommendation:** Reuse the existing `WorkflowStatus` enum. The plan's `Drafting` should map to `Draft`, and `PendingApproval` should map to `Submitted` or `InReview`. Create a mapping layer rather than duplicating state management logic.

### 1.2 Missing iou-storage Crate Dependencies

**Issue:** The plan proposes creating a new `iou-storage` crate, but no storage-related dependencies exist in the workspace.

**Current Cargo.toml (line 37-38):**
```toml
duckdb = { version = "1.1", features = ["bundled", "json", "parquet"] }
```

**Missing:** No S3/MinIO client library (e.g., `rust-s3`, `aws-sdk-s3`) is declared.

**Recommendation:** Add to workspace dependencies:
```toml
rust-s3 = { version = "0.5", features = ["tokio-rustls"] }
# or
aws-sdk-s3 = { version = "1.0", features = ["tokio"] }
```

### 1.3 Knowledge Graph Schema Undefined (Section 10.3)

**Issue:** The Research Agent needs to query GraphRAG, but the schema for what constitutes "documents of same type in domain" is unclear.

**Existing GraphRAG implementation** (`/Users/marc/Projecten/iou-modern/crates/iou-core/src/graphrag.rs`):
- Has `Entity` with `EntityType` enum (Person, Organization, Location, Law, Date, Money, Policy, Miscellaneous)
- No `Document` entity type exists

**Recommendation:** Define a schema for:
1. How documents are stored as entities in the knowledge graph
2. How to query "similar documents by type and domain"
3. What attributes a document entity should have (type, domain, created_at, etc.)

---

## 2. Security Concerns

### 2.1 PII Detection is Not Redaction (Section 4.3)

**Issue:** The Compliance Agent output includes "Redacted version if PII found" but there's no specification of:
- What redaction method (black bars, `[REDACTED]`, hash?)
- Whether redacted content is reversible
- Storage of original vs redacted versions

**Critical Footgun:** If PII is detected but the redaction is reversible or reversible patterns remain, this creates a major privacy violation under AVG/GDPR.

**Recommendation:** Explicitly define:
1. Redaction format must be irreversible
2. Separate storage for redacted vs original documents
3. Access controls for who can view original (unredacted) content

### 2.2 Auto-Approval with High Trust Level (Section 3.1)

**Issue:** High trust domains auto-approve if `confidence > 0.95`. This is dangerous because:
1. AI confidence scores are often poorly calibrated
2. No human review could publish non-compliant government documents
3. The 0.95 threshold lacks empirical justification

**Recommendation:**
1. Require human approval for ALL Woo-relevant documents regardless of confidence
2. Auto-approval should only apply to clearly non-sensitive internal documents
3. Add a "dry run" mode to test auto-approval before enabling it

### 2.3 Missing Authentication on Approval Endpoint (Section 5.1)

**Issue:** The `/api/documents/{id}/approve` endpoint has no specification for:
- Authentication requirements
- Authorization (who can approve?)
- Approval trail (who approved which document)

**Recommendation:** Specify:
1. Authentication middleware requirements
2. Role-based access control (only users with "approver" role can approve)
3. MFA requirements for approval actions in high-sensitivity domains

### 2.4 Template Injection Vulnerability (Section 3.2)

**Issue:** Using Tera templates with user-supplied context could lead to template injection if context is not properly sanitized.

**Recommendation:** Specify:
1. Template sandboxing
2. Input sanitization for all context variables
3. Allowlist of available template functions/filters

---

## 3. Missing Considerations

### 3.1 Document Versioning and History

**Issue:** The `DocumentMetadata` has `current_version_key` and `previous_version_key`, but no provision for:
- Full version history
- Diff between versions
- Rollback capability
- Version comparison UI (referenced in ProvisaManager but not in plan)

**Recommendation:** Add a `document_versions` table with:
```sql
CREATE TABLE document_versions (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id),
    version_number INTEGER NOT NULL,
    storage_key VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    created_by VARCHAR,
    change_summary TEXT,
    is_current BOOLEAN NOT NULL
);
```

### 3.2 Concurrent Document Editing

**Issue:** No handling for multiple users trying to edit/approve the same document simultaneously.

**Recommendation:** Add:
1. Optimistic locking with version numbers
2. Explicit checkout/checkin mechanism
3. Conflict detection and resolution UI

### 3.3 Document Lifecycle After Publication

**Issue:** What happens after `Published` state?
- Can documents be unpublished?
- How are Woo-mandated publication periods handled?
- When do documents move to archival storage?

**Recommendation:** Define post-publication workflow:
1. Unpublication process with justification
2. Automatic archival after retention period
3. Integration with PROVISA retention system

### 3.4 Rate Limiting and Resource Management

**Issue:** No mention of:
- Rate limiting on document creation
- Queue management for expensive AI operations
- Cost controls for AI API calls

**Recommendation:** Add:
1. Per-domain rate limits
2. Job queue for pipeline execution
3. Cost tracking and budget alerts

### 3.5 Localization and Multi-Language Support

**Issue:** Dutch government documents may need to support multiple languages (Dutch, English, Frisian). The plan assumes Dutch-only templates.

**Recommendation:** Specify:
1. Template language variants
2. Language selection in DocumentRequest
3. Translation workflow support

### 3.6 Error Recovery and Retry Logic

**Issue:** The "fail-fast" approach (Section 1.2) means one agent failure stops the entire pipeline. This creates:
- Poor user experience for transient failures
- No partial results for debugging
- No automatic retry for recoverable errors

**Recommendation:** Define:
1. Distinguish between permanent and transient errors
2. Automatic retry with exponential backoff for transient errors
3. Checkpoint/restart capability for long-running pipelines

---

## 4. Performance Concerns

### 4.1 Agent Sequential Bottleneck (Section 2.2)

**Issue:** Sequential execution of four agents means total latency is the sum of all agent execution times. For complex documents, this could exceed the 30-second SLA (Section 9).

**Recommendation:**
1. Identify parallelizable operations (e.g., compliance checks could run alongside content generation in some cases)
2. Add timeout configurations per agent
3. Consider streaming responses for long-running operations

### 4.2 Markdown to ODF Conversion (Section 10.2)

**Issue:** The plan leaves this as an "open question" with three options. Each has significant trade-offs:
- **Pandoc via CLI**: Spawning processes is slow and resource-intensive
- **Custom Rust**: Significant development effort, limited ODF libraries in Rust
- **External service**: Adds network latency, cost, and privacy concerns

**Recommendation:** Benchmark all three options during Phase 2. Consider a hybrid: Pandoc for initial implementation, with a path to native Rust if performance is inadequate.

### 4.3 Audit Query Performance (Section 9)

**Issue:** Target is "< 100ms" for audit queries, but the `document_audit` table design could grow very large.

**Recommendation:**
1. Add partitioning by `created_at`
2. Consider a separate audit log storage system for long-term retention
3. Add audit trail archiving strategy

---

## 5. Integration Issues

### 5.1 Duplicate Compliance Types

**Issue:** The plan defines new compliance types (`ComplianceResult`, `RefusalGround`, `PiiLocation`) that overlap with existing types in `/Users/marc/Projecten/iou-modern/crates/iou-core/src/compliance.rs`.

**Existing types:**
- `WooMetadata`, `WooRefusalGround`, `WooDisclosureClass`
- `AvgMetadata`, `PrivacyLevel`
- `ComplianceStatus`, `ComplianceIssue`

**Recommendation:** Reuse existing compliance types. Only extend if truly needed.

### 5.2 Knowledge Graph Client Undefined

**Issue:** The agents take `&GraphRagClient` as a parameter, but no such client exists in the codebase. There's only `KnowledgeGraph` in `iou-ai`.

**Recommendation:** Define a `GraphRagClient` trait that abstracts:
1. Entity queries
2. Relationship traversal
3. Similarity search
4. Community detection queries

### 5.3 Missing AI Provider Configuration

**Issue:** The plan mentions using AI/ML services but doesn't specify:
1. Which AI provider (OpenAI, Anthropic, local models?)
2. API key management
3. Fallback providers
4. Cost tracking

**Recommendation:** Add a section on AI provider configuration and management.

---

## 6. Ambiguous Requirements

### 6.1 Trust Level Assignment Criteria

**Issue:** Section 3.1 defines `TrustLevel` but not:
1. Who decides the trust level for a domain?
2. How is trust level changed?
3. Is trust level per-domain, per-user, or per-document-type?

**Recommendation:** Specify governance model for trust level assignment.

### 6.2 Template Variable Sources (Section 3.2)

**Issue:** The `VariableSource` enum includes `KnowledgeGraph` but it's unclear:
1. How variables are extracted from the knowledge graph
2. What happens when a variable is not found
3. How conflicting sources (UserInput vs KnowledgeGraph) are resolved

**Recommendation:** Define variable resolution priority and conflict handling.

### 6.3 "Context" in DocumentRequest (Section 3.1)

**Issue:** `context: HashMap<String, String>` is overly permissive. This could lead to:
- Inconsistent variable names across documents
- No validation of required vs optional context
- Difficult debugging

**Recommendation:** Define a schema for context validation per document type.

---

## 7. Minor Issues

### 7.1 SQL Index Missing

The `documents` table has an index on `domain_id`, but no index on `(domain_id, state)` which would be more efficient for common queries like "all pending documents for a domain".

### 7.2 UUID v4 for All IDs

The plan uses UUID v4 (random) for all IDs. Consider:
- UUID v7 (time-ordered) for better database indexing
- Or use domain-specific IDs (e.g., `DOC-2025-0001`)

### 7.3 No Document Cancellation

The state machine has `Rejected` but no explicit `Cancelled` state for documents that are abandoned before completion.

### 7.4 Migration File Naming

The migration file is named `030_documents.sql` but there's no indication if migrations 020-029 exist or what the current migration version is.

---

## 8. Recommendations for Next Steps

1. **Address state machine duplication** before writing any code
2. **Define the Knowledge Graph schema** for document storage and retrieval
3. **Clarify security model** - specifically PII redaction and approval authorization
4. **Prototype the Markdown to ODF conversion** early to validate the 30-second SLA
5. **Add a section on observability** - metrics, logging, and alerting
6. **Define error types** - create a comprehensive `AgentError` enum covering all failure modes
7. **Add a PoC phase** before Phase 1 to validate AI provider integration

---

## 9. Positive Aspects Worth Noting

1. **Clear separation of concerns** between agents is well-designed
2. **Maker-checker pattern** is appropriate for government documents
3. **Audit trail requirements** are thorough
4. **Integration with existing compliance types** shows good awareness of the codebase
5. **Phase-based implementation** is well-structured with clear deliverables

---

Overall, this is a solid foundation but needs refinement in security, state management, and integration with the existing codebase before implementation should begin.
