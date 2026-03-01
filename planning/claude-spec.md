# Specification: Document Creation Agents for IOU-Modern

## Executive Summary

This specification defines an AI-powered document creation system for the IOU-Modern platform - a Dutch government information system built with Rust, WebAssembly, and DuckDB. The system uses multi-agent orchestration to generate compliant government documents with configurable human approval workflows.

---

## 1. Project Context

### 1.1 IOU-Modern Architecture

IOU-Modern is an **Informatie Ondersteunde Werkomgeving** for Dutch government organizations:

**Technology Stack:**
- **Backend**: Axum REST API with async/await
- **Database**: DuckDB (embedded analytical database)
- **Frontend**: Dioxus 0.7 (WebAssembly)
- **AI/ML**: Custom Rust implementations for NER, semantic search, and compliance

**Project Structure:**
```
iou-modern/
├── crates/
│   ├── iou-core/       # Shared domain models
│   ├── iou-api/        # REST API (Axum + DuckDB)
│   ├── iou-ai/         # AI services (NER, GraphRAG)
│   ├── iou-regels/     # Open Regels integration
│   └── iou-frontend/   # Dioxus WASM app
├── migrations/         # DuckDB schema
└── data/              # DuckDB database file
```

### 1.2 Existing Integration Points

| Component | Location | Purpose |
|-----------|----------|---------|
| **Compliance AI** | `iou-ai/src/compliance.rs` | Woo validation, refusal grounds |
| **Workflow API** | `iou-api/src/routes/workflows.rs` | State management |
| **PROVISA Manager** | `iou-frontend/src/pages/provisa_manager.rs` | Document versioning |
| **Knowledge Graph** | `GraphRAG endpoints` | Context-aware generation |
| **Compliance Dashboard** | `iou-frontend/src/pages/compliance_dashboard.rs` | Monitoring UI |

---

## 2. Functional Requirements

### 2.1 Agent Pipeline

The system implements a **Sequential Pipeline with Maker-Checker loops**:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Research Agent │ -> │ Content Agent   │ -> │ Compliance Agent│
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                     │
                                                     v
                                              ┌─────────────────┐
                                              │  Review Agent  │ <──┐
                                              └─────────────────┘    │
                                                     │               │
                                                     v               │
                                              ┌─────────────────┐    │
                                              │ Approval Check  │────┘
                                              └─────────────────┘
                                                     │
                                                     v
                                              ┌─────────────────┐
                                              │ Final Document  │
                                              └─────────────────┘
```

#### 2.1.1 Research/Context Agent
- Analyzes document request context
- Queries knowledge graph for relevant domain information
- Determines document structure and required sections
- Identifies applicable PROVISA guidelines

#### 2.1.2 Content Generation Agent
- Populates templates with domain-specific content
- Uses knowledge graph for context-aware generation
- Generates Markdown that will be converted to ODF/PDF
- Handles variable substitution and conditional sections

#### 2.1.3 Compliance Validator Agent
- Validates against Woo (Wet open overheid) rules
- Checks PII detection and redaction requirements
- Validates refusal grounds (Woo articles 5.1 and 5.2)
- Ensures accessibility (WCAG guidelines)
- Assigns confidence scores (0.0-1.0)

#### 2.1.4 Review/Refinement Agent
- Final quality check before human approval
- Checks for consistency, completeness, and clarity
- May iterate with Content Agent if issues found
- Generates audit trail of all decisions

### 2.2 Human Approval Workflow

**Domain-Specific Trust Levels:**
- Each information domain has configurable trust: low/medium/high
- Low trust: Always requires human approval
- Medium trust: Requires approval if compliance score < 0.8
- High trust: Auto-approval for documents with confidence > 0.95

**Approval States:**
- `Drafting` - Agents working on document
- `PendingApproval` - Awaiting human review
- `Approved` - Approved for publication/processing
- `Rejected` - Returned for revision
- `Published` - Final document delivered

### 2.3 Template System

**Format:** Markdown-based templates

**Rationale:**
- Version control friendly
- Simple authoring for non-technical users
- Converted to ODF (ISO/IEC 26300) for final delivery
- Supports conditional sections and variable substitution

**Template Example:**
```markdown
# {{document_type}}

**Ref:** {{reference_number}}
**Date:** {{generated_date}}
**Domain:** {{domain}}

## {{section_1_title}}

{{section_1_content}}

{% if requires_publication_notice %}
## Publicatieplicht

Dit document wordt gepubliceerd conform...
{% endif %}
```

### 2.4 Storage Architecture

**Primary Storage:** S3-compatible object storage
- Documents stored as binary files (ODF/PDF)
- Metadata indexed in DuckDB

**Versioning:** Current + one previous version
- Previous version retained for rollback capability
- Not full history (simplifies storage)

**DuckDB Schema:**
```sql
CREATE TABLE documents (
    id UUID PRIMARY KEY,
    domain_id VARCHAR,
    document_type VARCHAR,
    current_version_id VARCHAR,  -- Pointer to S3
    previous_version_id VARCHAR, -- Pointer to S3
    approval_state VARCHAR,
    trust_level VARCHAR,
    compliance_score FLOAT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);
```

### 2.5 Error Handling

**Fail Fast Philosophy:**
- Any error stops the pipeline immediately
- User notified with specific error details
- No graceful degradation or silent retries
- User can manually fix and retry

---

## 3. Non-Functional Requirements

### 3.1 MVP Priorities

1. **Compliance Validation**
   - Correct Woo rule implementation
   - PII detection before publication
   - Refusal grounds accuracy

2. **Smart Content Generation**
   - Knowledge graph integration
   - Domain-specific content
   - Context-aware templating

3. **Observability/Debugging**
   - Full audit trails for all agent decisions
   - Logging of pipeline states
   - Debug UI for inspection

### 3.2 Deferred to Post-MVP

- Performance optimization (batch processing)
- High concurrency scenarios
- Advanced caching strategies
- Multi-language support (beyond Dutch)

### 3.3 Security Requirements

- HTTPS mandatory (Wet digitale overheid)
- HSTS required
- PII redaction before publication
- Audit logging for all document access

### 3.4 Accessibility

- WCAG 2.1 AA compliance
- Generated documents must be accessible
- ODF format ensures long-term accessibility

---

## 4. API Design (Draft)

### 4.1 Document Creation Endpoints

```
POST /api/documents/create
- Input: document_type, domain_id, context
- Output: document_id, initial_status

GET /api/documents/{id}/status
- Output: current_state, agent_stage, errors

POST /api/documents/{id}/approve
- Input: approval decision, comments
- Output: new_state

GET /api/documents/{id}/audit
- Output: full decision trail
```

### 4.2 Template Management

```
GET /api/templates
- List available templates by domain

POST /api/templates
- Create/update template (Markdown)

GET /api/templates/{id}/preview
- Render template with sample data
```

---

## 5. Integration with Existing Components

### 5.1 Compliance AI Integration

Reuse existing compliance checking:
- Rule-based patterns from `iou-ai/src/compliance.rs`
- Woo relevance assessment
- Disclosure class determination
- Refusal grounds enumeration

### 5.2 PROVISA Integration

- Use PROVISA version comparison for document versions
- Hotspot management for document sections
- AI-powered classification

### 5.3 Knowledge Graph Integration

- Query GraphRAG for domain context
- Use semantic search for template matching
- Retrieve related documents for context

---

## 6. Testing Strategy

Given existing codebase patterns:
- Standard Rust `#[test]` attributes
- Mock data for development
- JSON Schema validation tests
- Compliance rule tests (following `compliance.rs` pattern)

### 6.1 Test Categories

1. **Unit Tests**: Individual agent logic
2. **Integration Tests**: Pipeline orchestration
3. **Compliance Tests**: Woo rule validation
4. **Template Tests**: Rendering and validation
5. **End-to-End Tests**: Full document creation flow

---

## 7. Success Criteria

1. All 4 agent types implemented and functional
2. Domain-specific trust levels configurable
3. Markdown templates convert to valid ODF
4. Compliance validation catches all Woo violations
5. Full audit trail for every generated document
6. Fail-fast error handling works correctly
7. Integration with existing IOU-Modern components verified
