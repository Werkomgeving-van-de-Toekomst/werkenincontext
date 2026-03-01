# Implementation Plan: Document Creation Agents for IOU-Modern

## Document Information

| Field | Value |
|-------|-------|
| **Project** | IOU-Modern (Dutch Government Information System) |
| **Component** | Document Creation Agents |
| **Architecture** | Multi-Agent Sequential Pipeline with Maker-Checker |
| **Tech Stack** | Rust, Axum, DuckDB, Dioxus WASM |

---

## 1. Overview

### 1.1 What We're Building

A multi-agent document creation system for Dutch government organizations. The system generates compliant government documents through a sequential pipeline of four AI agents, with configurable human approval based on domain-specific trust levels.

**Key Capabilities:**
- Generate Woo-compliant documents with automatic PII detection
- Context-aware content generation using knowledge graphs
- Domain-specific trust levels for human approval
- Full audit trails for all agent decisions
- Markdown-based templates converted to ODF/PDF

### 1.2 Why This Architecture

**Sequential Pipeline:** Document creation has clear dependencies. Research must inform structure, which informs content, which must be validated before review.

**Maker-Checker Pattern:** Quality assurance is critical for government documents. The Review Agent can iterate with Content Agent until quality thresholds are met.

**Fail-Fast Error Handling:** Government documents have zero tolerance for errors. Stopping immediately on errors ensures issues are surfaced rather than silently propagated.

**Markdown Templates:** Simpler authoring, version control friendly, convertible to ODF (ISO/IEC 26300) for long-term preservation.

---

## 2. Architecture

### 2.1 System Context

```
┌─────────────────────────────────────────────────────────────────────┐
│                         IOU-Modern Platform                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐         │
│  │   Dioxus     │    │  Axum API    │    │   DuckDB     │         │
│  │  Frontend    │◄──►│   (Rust)     │◄──►│  (Metadata)  │         │
│  └──────────────┘    └──────┬───────┘    └──────────────┘         │
│                              │                                       │
│                              v                                       │
│                       ┌──────────────┐                              │
│                       │   Document   │                              │
│                       │     Agents    │                              │
│                       │   (NEW)      │                              │
│                       └──────┬───────┘                              │
│                              │                                       │
│  ┌──────────────┐    ┌──────┴───────┐    ┌──────────────┐         │
│  │   S3/MinIO   │    │  Knowledge   │    │  Compliance  │         │
│  │   Storage    │◄──►│    Graph     │    │     AI       │         │
│  └──────────────┘    └──────────────┘    └──────────────┘         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Agent Pipeline Flow

```
┌──────────────────────────────────────────────────────────────────┐
│                      Document Creation Pipeline                   │
└──────────────────────────────────────────────────────────────────┘

Request ──► Research Agent ──► Content Agent ──► Compliance Agent
                    │                  │                    │
                    v                  v                    v
              [Context Query]   [Template Fill]    [Woo Check]
              [Structure]       [KG Context]       [PII Detect]
                                                   │
                                                   v
                                          Review Agent ◄─────┐
                                                   │            │
                                                   v            │
                                          Approval Check ─────┘
                                                   │
                                                   v
                                          Final Document
                                                   │
                                                   v
                                          S3 Storage + DB
```

### 2.3 State Machine

**IMPORTANT:** This system reuses the existing `WorkflowStatus` enum from `iou-core/src/workflows.rs` to maintain consistency with the existing workflow system.

**State Mapping:**
```
New Document State    → Existing WorkflowStatus
────────────────────────────────────────────────
Drafting             → Draft
PendingApproval      → Submitted (or InReview for clarity)
Approved             → Approved
Rejected             → Rejected
Published            → Published
```

**State Transitions:**
```
Draft → Submitted → Approved → Published
         │            │
         v            v
      Rejected ←──────┘
         │
         v
      Draft (retry)
```

**State Descriptions:**
- `Draft`: Initial state, all agents processing
- `Submitted`: Awaiting human approval decision (use InReview for more explicit meaning)
- `Approved`: Human approved, ready for final processing
- `Rejected`: Human rejected, back to Draft for revision
- `Published`: Final document delivered to storage

### 2.4 Directory Structure

```
iou-modern/
├── crates/
│   ├── iou-core/
│   │   └── src/
│   │       └── document.rs           # NEW: Document domain models
│   ├── iou-api/
│   │   └── src/
│   │       └── routes/
│   │           └── documents.rs      # NEW: Document API endpoints
│   ├── iou-ai/
│   │   └── src/
│   │       └── agents/               # NEW: Multi-agent system
│   │           ├── mod.rs
│   │           ├── research.rs       # Research Agent
│   │           ├── content.rs        # Content Generation Agent
│   │           ├── compliance.rs     # Compliance Validator Agent
│   │           ├── review.rs         # Review Agent
│   │           ├── pipeline.rs       # Orchestration logic
│   │           └── templates.rs      # Template engine
│   ├── iou-frontend/
│   │   └── src/
│   │       └── pages/
│   │           └── document_creator.rs  # NEW: Document creation UI
│   └── iou-storage/                  # NEW: Storage abstraction crate
│       └── src/
│           ├── mod.rs
│           ├── s3.rs                 # S3 client wrapper
│           └── metadata.rs           # DuckDB metadata operations
├── templates/                        # NEW: Document templates
│   ├── woo_besluit.md
│   ├── woo_info.md
│   └── provisa_notitie.md
└── migrations/
    └── 030_documents.sql             # NEW: Document metadata schema
```

---

## 3. Data Models

### 3.1 Core Domain Types

```rust
/// Unique identifier for a document generation request
pub type DocumentId = Uuid;

/// Current state in the document creation pipeline
/// NOTE: Reuses WorkflowStatus from iou-core/src/workflows.rs
/// Type alias for clarity in document context
pub use crate::workflows::WorkflowStatus as DocumentState;

/// Valid states for document workflow:
/// - Draft: Initial state, all agents processing
/// - Submitted: Awaiting human approval (InReview can also be used for clarity)
/// - Approved: Human approved, ready for final processing
/// - Rejected: Human rejected, back to Draft for revision
/// - Published: Final document delivered
/// - ChangesRequested: Alternative to Rejected when revision is expected
/// - Archived: Historical record, no longer active

/// Trust level determines auto-approval behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrustLevel {
    Low,     // Always requires human approval
    Medium,  // Requires approval if compliance_score < 0.8
    High,    // Auto-approval ONLY for non-Woo documents; ALL Woo-relevant documents require human approval
}

/// IMPORTANT SECURITY NOTE:
/// - ALL Woo-relevant documents require human approval regardless of confidence score
/// - Auto-approval only applies to internal, non-sensitive documents where legal compliance is not a concern
/// - A "dry run" mode should be available for testing auto-approval before enabling it in production

/// Configuration per information domain
pub struct DomainConfig {
    pub domain_id: String,
    pub trust_level: TrustLevel,
    pub required_approval_threshold: f32,  // For Medium trust
    pub auto_approval_threshold: f32,      // For High trust
}

/// Document generation request
pub struct DocumentRequest {
    pub id: DocumentId,
    pub domain_id: String,
    pub document_type: String,
    pub context: HashMap<String, String>,
    pub requested_at: DateTime<Utc>,
}

/// Document metadata stored in DuckDB
pub struct DocumentMetadata {
    pub id: DocumentId,
    pub domain_id: String,
    pub document_type: String,
    pub state: DocumentState,
    pub current_version_key: String,    // S3 object key
    pub previous_version_key: Option<String>,
    pub compliance_score: f32,
    pub confidence_score: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Agent execution result
pub struct AgentResult {
    pub agent_name: String,
    pub success: bool,
    pub data: serde_json::Value,
    pub errors: Vec<String>,
    pub execution_time_ms: u64,
}

/// Audit trail entry for observability
pub struct AuditEntry {
    pub id: Uuid,
    pub document_id: DocumentId,
    pub agent_name: String,
    pub action: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}
```

### 3.2 Template Data

```rust
/// Template metadata
pub struct Template {
    pub id: String,
    pub name: String,
    pub domain_id: String,
    pub document_type: String,
    pub content: String,  // Markdown template
    pub required_variables: Vec<String>,
    pub optional_sections: Vec<String>,
    pub version: i32,
}

/// Template variable value
pub struct TemplateVariable {
    pub name: String,
    pub value: String,
    pub source: VariableSource,
}

#[derive(Debug, Clone)]
pub enum VariableSource {
    UserInput,
    KnowledgeGraph,
    AgentGenerated,
    Default,
}
```

### 3.3 Compliance Data

**NOTE:** Reuses existing compliance types from `iou-core/src/compliance.rs`:
- `WooMetadata`, `WooRefusalGround`, `WooDisclosureClass`
- `AvgMetadata`, `PrivacyLevel`
- `ComplianceStatus`, `ComplianceIssue`

Only extend these types if new functionality is required that doesn't exist.

```rust
/// Woo compliance check result (extends existing ComplianceIssue)
pub struct ComplianceResult {
    pub is_compliant: bool,
    pub score: f32,
    pub refusal_grounds: Vec<WooRefusalGround>,  // Reuse existing type
    pub pii_detected: Vec<PiiLocation>,
    pub accessibility_issues: Vec<AccessibilityIssue>,
}

/// Detected PII location (new type, specific to document workflow)
pub struct PiiLocation {
    pub text: String,
    pub start_index: usize,
    pub end_index: usize,
    pub pii_type: PiiType,  // BSN, Email, Phone, Address, etc.
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub enum PiiType {
    BSN,           // Dutch social security number
    Email,
    PhoneNumber,
    Address,
    IBAN,
    Name,
}
```

### 3.4 Storage Types

```rust
/// S3 object reference
pub struct StorageRef {
    pub bucket: String,
    pub key: String,
    pub version_id: Option<String>,
    pub content_type: String,
    pub size_bytes: u64,
    pub etag: String,
}

/// Document version in S3
pub struct DocumentVersion {
    pub storage_ref: StorageRef,
    pub format: DocumentFormat,
    pub created_at: DateTime<Utc>,
    pub created_by: String,  // Agent or User ID
}

#[derive(Debug, Clone)]
pub enum DocumentFormat {
    Markdown,
    ODF,   // OpenDocument Format
    PDF,
}
```

---

## 4. Agent Specifications

### 4.1 Research Agent

**Purpose:** Analyze request context, query knowledge graph, determine document structure.

**Inputs:**
- `DocumentRequest` - Original request
- Domain configuration
- Knowledge graph client

**Outputs:**
- `ResearchContext` containing:
  - Required sections based on document type
  - Relevant domain information from knowledge graph
  - Applicable PROVISA guidelines
  - Suggested template variables

**Key Behaviors:**
1. Query GraphRAG for documents of same type in domain
2. Extract common structure patterns
3. Identify mandatory vs optional sections
4. Retrieve related entities for context

**Function Signature:**
```rust
pub async fn execute_research_agent(
    request: &DocumentRequest,
    kg_client: &GraphRagClient,
    domain_config: &DomainConfig,
) -> Result<ResearchContext, AgentError>;
```

### 4.2 Content Generation Agent

**Purpose:** Generate document content using template and context.

**Inputs:**
- `ResearchContext` from Research Agent
- Template definition
- Knowledge graph for entity resolution

**Outputs:**
- Generated Markdown document
- Template variable values
- Entity linking metadata

**Key Behaviors:**
1. Load appropriate template for document type
2. Fill required variables from:
   - User input (direct)
   - Knowledge graph (entities, relationships)
   - Agent generation (summaries, descriptions)
3. Handle conditional sections
4. Generate domain-specific content

**Function Signature:**
```rust
pub async fn execute_content_agent(
    research: &ResearchContext,
    template: &Template,
    kg_client: &GraphRagClient,
) -> Result<GeneratedDocument, AgentError>;
```

### 4.3 Compliance Validator Agent

**Purpose:** Validate document against Woo rules and accessibility guidelines.

**Inputs:**
- `GeneratedDocument` from Content Agent
- Compliance rules database

**Outputs:**
- `ComplianceResult` with scores and issues
- Redacted version if PII found
- Approval recommendation

**Key Behaviors:**
1. Run Woo compliance checks (reuse existing `iou-ai/src/compliance.rs`)
2. Detect PII using NER with irreversible redaction
3. Validate refusal grounds (reuse existing `WooRefusalGround`)
4. Check WCAG compliance
5. Calculate overall compliance score

**PII Redaction (CRITICAL for AVG/GDPR compliance):**
- Redaction format: `[PII: <type>]` (e.g., `[PII: BSN]`, `[PII: Email]`)
- Redaction is **irreversible** - original PII cannot be recovered from redacted text
- Separate storage for original vs redacted documents
- Role-based access control for viewing unredacted originals
- Audit trail of who accessed unredacted versions

**Function Signature:**
```rust
pub async fn execute_compliance_agent(
    document: &GeneratedDocument,
    compliance_rules: &ComplianceRules,
) -> Result<ComplianceResult, AgentError>;
```

### 4.4 Review Agent

**Purpose:** Final quality check before human approval.

**Inputs:**
- `GeneratedDocument` from Content Agent
- `ComplianceResult` from Compliance Agent
- Quality thresholds

**Outputs:**
- Review summary
- Approval decision (if auto-approval enabled)
- Iteration request (if quality issues found)

**Key Behaviors:**
1. Check completeness (all required sections present)
2. Check clarity and consistency
3. Verify compliance score meets threshold
4. Decide: approve, iterate, or reject

**Function Signature:**
```rust
pub async fn execute_review_agent(
    document: &GeneratedDocument,
    compliance: &ComplianceResult,
    domain_config: &DomainConfig,
) -> Result<ReviewDecision, AgentError>;
```

### 4.5 Pipeline Orchestration

**Purpose:** Coordinate agent execution in sequence.

**Key Behaviors:**
1. Execute agents sequentially
2. Handle maker-checker iteration (Review → Content)
3. Distinguish between permanent and transient errors
4. Automatic retry with exponential backoff for transient errors
5. Fail-fast on permanent errors
6. Checkpoint/restart capability for long-running pipelines
7. Track all agent results for audit trail
8. Determine if human approval needed

**Error Classification:**
- **Transient errors:** Rate limiting, temporary network issues, AI provider timeouts
  - Action: Retry with exponential backoff (1s, 2s, 4s, 8s, 16s max)
  - Max retries: 3 per agent
- **Permanent errors:** Invalid input, template not found, authentication failure
  - Action: Fail immediately, return error to user

**Function Signature:**
```rust
pub async fn execute_document_pipeline(
    request: DocumentRequest,
    agents: AgentPipeline,
    storage: &StorageBackend,
) -> Result<PipelineResult, PipelineError>;
```

---

## 5. API Specification

### 5.1 Document Creation API

```
POST /api/documents/create

Request:
{
    "domain_id": "gemeente-utrecht",
    "document_type": "woo_besluit",
    "context": {
        "reference_number": "2025-001",
        "requester": "Jan Jansen",
        "subject": "Inzage verzoek"
    }
}

Response (200):
{
    "document_id": "uuid",
    "state": "Drafting",
    "estimated_completion": "2025-03-01T12:00:00Z"
}

Response (400):
{
    "error": "Invalid domain_id",
    "details": "Domain 'unknown' not found"
}
```

```
GET /api/documents/{id}/status

Response (200):
{
    "document_id": "uuid",
    "state": "PendingApproval",
    "current_agent": null,
    "compliance_score": 0.92,
    "confidence_score": 0.88,
    "requires_approval": true,
    "errors": []
}
```

```
POST /api/documents/{id}/approve

Authentication: REQUIRED (valid JWT token)
Authorization: REQUIRED (user must have "document_approver" role)
MFA: REQUIRED for high-sensitivity domains

Request:
{
    "approved": true,
    "comments": "Looks good, proceed with publication"
}

Response (200):
{
    "document_id": "uuid",
    "state": "Approved",
    "approved_at": "2025-03-01T12:05:00Z",
    "approved_by": "user_id"
}

Response (403):
{
    "error": "Insufficient permissions",
    "details": "User lacks 'document_approver' role"
}

Response (401):
{
    "error": "Authentication required"
}
```

```
GET /api/documents/{id}/audit

Response (200):
{
    "document_id": "uuid",
    "audit_trail": [
        {
            "agent": "ResearchAgent",
            "action": "Completed research phase",
            "timestamp": "2025-03-01T12:00:00Z",
            "details": {...}
        },
        {
            "agent": "ContentAgent",
            "action": "Generated document content",
            "timestamp": "2025-03-01T12:01:00Z",
            "details": {...}
        },
        ...
    ]
}
```

### 5.2 Template Management API

```
GET /api/templates

Query params: ?domain_id={id}

Response (200):
{
    "templates": [
        {
            "id": "woo_besluit_2025",
            "name": "Woo Besluit Template",
            "domain_id": "gemeente-utrecht",
            "version": 1
        }
    ]
}
```

```
POST /api/templates

Request:
{
    "name": "Woo Besluit Template",
    "domain_id": "gemeente-utrecht",
    "document_type": "woo_besluit",
    "content": "# {{document_type}}...",
    "required_variables": ["document_type", "reference_number"]
}

Response (201):
{
    "template_id": "uuid",
    "version": 1
}
```

```
GET /api/documents/{id}/download

Query params: ?format=odf|pdf|md

Response (200):
[Binary file content]
```

---

## 6. Database Schema

### 6.1 Documents Table

```sql
CREATE TABLE documents (
    id UUID PRIMARY KEY,
    domain_id VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    state VARCHAR NOT NULL,  -- Drafting, PendingApproval, Approved, Rejected, Published
    trust_level VARCHAR NOT NULL,  -- Low, Medium, High

    -- Storage references
    current_version_key VARCHAR NOT NULL,
    previous_version_key VARCHAR,

    -- Scores
    compliance_score FLOAT NOT NULL DEFAULT 0.0,
    confidence_score FLOAT NOT NULL DEFAULT 0.0,

    -- Request context
    request_context JSON,

    -- Audit
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    published_at TIMESTAMP,

    -- Approval
    approved_by VARCHAR,
    approval_notes TEXT
);

CREATE INDEX idx_documents_domain ON documents(domain_id);
CREATE INDEX idx_documents_state ON documents(state);
CREATE INDEX idx_documents_domain_state ON documents(domain_id, state);  -- Compound index for common queries
CREATE INDEX idx_documents_created ON documents(created_at DESC);
```

### 6.2 Audit Trail Table

```sql
CREATE TABLE document_audit (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id),
    agent_name VARCHAR NOT NULL,
    action VARCHAR NOT NULL,
    details JSON,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    execution_time_ms INTEGER
);

CREATE INDEX idx_audit_document ON document_audit(document_id);
CREATE INDEX idx_audit_timestamp ON document_audit(timestamp DESC);

-- Performance optimization: Partition audit trail by created_at for large-scale deployments
-- Consider separate audit storage for long-term retention
```

### 6.3 Document Versions Table

```sql
-- Provides full version history, diff capability, and rollback support
CREATE TABLE document_versions (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id),
    version_number INTEGER NOT NULL,
    storage_key VARCHAR NOT NULL,
    format VARCHAR NOT NULL,  -- Markdown, ODF, PDF
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_by VARCHAR,  -- Agent name or User ID
    change_summary TEXT,
    is_current BOOLEAN NOT NULL DEFAULT FALSE,
    compliance_score FLOAT,
    UNIQUE(document_id, version_number)
);

CREATE INDEX idx_versions_document ON document_versions(document_id);
CREATE INDEX idx_versions_current ON document_versions(document_id, is_current);
```

### 6.4 Templates Table

```sql
CREATE TABLE templates (
    id UUID PRIMARY KEY,
    name VARCHAR NOT NULL,
    domain_id VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    content TEXT NOT NULL,
    required_variables JSON,  -- Array of strings
    optional_sections JSON,    -- Array of strings
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE UNIQUE INDEX idx_templates_domain_type ON templates(domain_id, document_type) WHERE is_active = TRUE;
```

### 6.5 Domain Configuration Table

```sql
CREATE TABLE domain_configs (
    domain_id VARCHAR PRIMARY KEY,
    trust_level VARCHAR NOT NULL,  -- Low, Medium, High
    required_approval_threshold FLOAT DEFAULT 0.8,
    auto_approval_threshold FLOAT DEFAULT 0.95,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

---

## 7. AI Provider Configuration

### 7.1 Provider Options

The system supports multiple AI providers for agent execution:

| Provider | Use Case | Configuration |
|----------|----------|---------------|
| OpenAI GPT-4 | General purpose, high quality | `OPENAI_API_KEY` environment variable |
| Anthropic Claude | Long context, safety-focused | `ANTHROPIC_API_KEY` environment variable |
| Local Models (Ollama) | Cost-sensitive, data privacy | `OLLAMA_BASE_URL` environment variable |

### 7.2 Configuration Structure

```rust
pub struct AiProviderConfig {
    pub primary_provider: ProviderType,
    pub fallback_providers: Vec<ProviderType>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub cost_tracking_enabled: bool,
}

#[derive(Debug, Clone)]
pub enum ProviderType {
    OpenAI { model: String },
    Anthropic { model: String },
    Ollama { model: String, base_url: String },
}
```

### 7.3 Key Management

- API keys stored in environment variables, never in code
- Keys loaded at startup via `dotenv` or Kubernetes secrets
- No key rotation in initial implementation (future enhancement)

### 7.4 Fallback Strategy

If primary provider fails:
1. Retry with exponential backoff (up to `max_retries`)
2. If still failing, switch to first fallback provider
3. If all providers fail, return error to user with details

### 7.5 Cost Tracking

When enabled, track:
- Tokens consumed per provider
- Estimated cost per document
- Monthly cost totals
- Alerts when approaching budget

---

## 8. Implementation Phases

### Phase 1: Foundation (Prerequisite for all other phases)

**Goal:** Set up core infrastructure.

**Tasks:**
1. Add S3/MinIO client dependency (`rust-s3` or `aws-sdk-s3`) to workspace Cargo.toml
2. Create `iou-storage` crate with S3 abstraction
3. Create database migration for documents schema (including document_versions table)
4. Define core domain models in `iou-core/src/document.rs` (reuse existing WorkflowStatus)
5. Define Document entity schema for GraphRAG
6. Set up storage client configuration

**Deliverables:**
- S3 client dependency in workspace
- `iou-storage` crate with S3 client wrapper
- Document metadata tables in DuckDB
- Core domain types defined (reusing existing types where applicable)
- GraphRAG schema extension for Document entities

### Phase 2: Template System

**Goal:** Implement Markdown template rendering.

**Tasks:**
1. Integrate Tera template engine
2. Create template loading from database
3. Implement variable substitution logic
4. Create initial template set (Woo besluit, Woo info)
5. Implement Markdown to ODF conversion

**Deliverables:**
- Working template system with variable substitution
- Initial document templates
- ODF export capability

### Phase 3: Individual Agents

**Goal:** Implement each agent independently.

**Tasks:**
1. **Research Agent:** GraphRAG queries, structure determination
2. **Content Agent:** Template filling, knowledge graph context
3. **Compliance Agent:** Woo validation, PII detection
4. **Review Agent:** Quality checks, approval decision

**Deliverables:**
- Four working agents with test coverage
- Mock knowledge graph for testing
- Compliance rules database

### Phase 4: Pipeline Orchestration

**Goal:** Wire agents together with state management.

**Tasks:**
1. Implement sequential pipeline execution
2. Add maker-checker iteration logic
3. Implement fail-fast error handling
4. Add audit trail logging
5. Create pipeline state persistence

**Deliverables:**
- Working end-to-end pipeline
- Audit trail for all agent decisions
- State machine implementation

### Phase 5: API Layer

**Goal:** Expose functionality via REST API.

**Tasks:**
1. Implement document creation endpoint
2. Implement status query endpoint
3. Implement approval endpoint
4. Implement audit trail endpoint
5. Implement template management endpoints
6. Implement document download endpoint

**Deliverables:**
- Complete REST API for document operations
- OpenAPI documentation
- API authentication integration

### Phase 6: Frontend Integration

**Goal:** Create UI for document creation and approval.

**Tasks:**
1. Create document creation page in Dioxus
2. Implement approval workflow UI
3. Add audit trail viewer
4. Integrate with existing compliance dashboard
5. Add template management UI

**Deliverables:**
- Document creation interface
- Approval queue interface
- Audit trail viewer component

---

## 9. Testing Strategy

### 9.1 Unit Tests

**Target:** Individual agent logic

- Template variable substitution
- Compliance rule evaluation
- PII detection accuracy
- State transitions

### 9.2 Integration Tests

**Target:** Agent orchestration

- Full pipeline execution
- Error propagation
- Storage integration
- Database operations

### 9.3 Compliance Tests

**Target:** Woo rule validation

- Known Woo-violating documents should fail
- Known Woo-compliant documents should pass
- PII detection edge cases
- Refusal ground accuracy

### 9.4 End-to-End Tests

**Target:** Complete user flows

- Document creation to publication
- Approval workflow
- Rejection and revision
- Document download

---

## 10. Success Criteria

1. **Functional:**
   - All 4 agents execute successfully in sequence
   - Domain-specific trust levels configurable
   - Markdown templates convert to valid ODF
   - Compliance validation catches Woo violations
   - Full audit trail for every document

2. **Non-Functional:**
   - Fail-fast error handling works correctly
   - Integration with existing IOU-Modern components verified
   - Audit query performance < 100ms
   - Document creation completes within 30 seconds

3. **Compliance:**
   - PII detection accuracy > 95%
   - No false negatives on Woo violations
   - All generated documents pass WCAG AA checks

---

## 11. Open Questions / Decisions Needed

### 11.1 S3 Configuration

**Question:** What S3-compatible storage will be used in production?

**Options:**
- AWS S3
- MinIO (self-hosted)
- Azure Blob (via S3 compatibility layer)

**Impact:** Affects configuration structure in `iou-storage`.

### 11.2 Markdown to ODF Conversion

**Question:** Which library for Markdown to ODF conversion?

**Options:**
- Pandoc (via CLI invocation)
- Custom Rust implementation using `odf` crate
- External service (e.g., CloudConvert)

**Impact:** Performance, reliability, deployment complexity.

### 11.3 Knowledge Graph Schema

**Question:** What schema should GraphRAG queries use?

**Status:** Needs investigation of existing GraphRAG schema in IOU-Modern.

---

## Appendix A: Integration Points

### A.1 Existing Compliance AI

**Location:** `iou-ai/src/compliance.rs`

**Reuse:**
- Woo relevance assessment logic
- Disclosure class enumeration
- Existing refusal ground patterns
- Confidence scoring algorithms

### A.2 Existing Workflow API

**Location:** `iou-api/src/routes/workflows.rs`

**Reuse:**
- State machine patterns
- Workflow persistence logic
- Status query patterns

### A.3 Existing PROVISA Manager

**Location:** `iou-frontend/src/pages/provisa_manager.rs`

**Reuse:**
- Version comparison UI patterns
- Document type classification
- Domain selection interface
