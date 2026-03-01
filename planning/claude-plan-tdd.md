# TDD Plan: Document Creation Agents for IOU-Modern

## Testing Approach

**Framework:** Standard Rust `#[test]` attributes with `cargo test`
**Location:** Tests colocated in src modules with `#[cfg(test)]` modules
**Mock Data:** Following existing IOU-Modern patterns
**Validation:** JSON Schema validation for API contracts

---

## 1. Overview Tests

### 1.1 System Integration Tests
- Test: All four agents execute sequentially without errors
- Test: Maker-checker iteration completes successfully
- Test: Fail-fast error handling stops pipeline on permanent error
- Test: Transient errors trigger exponential backoff retry

### 1.2 End-to-End Workflow Tests
- Test: Document creation from request to published state
- Test: Human approval workflow for Woo documents
- Test: Rejection and revision cycle returns document to Draft
- Test: Document download in all formats (Markdown, ODF, PDF)

---

## 2. Architecture Tests

### 2.1 State Machine Tests
- Test: Draft → Submitted transition occurs after agents complete
- Test: Submitted → Approved transition requires human approval
- Test: Approved → Published transition stores document in S3
- Test: Rejected state returns document to Draft for revision
- Test: Invalid state transitions are rejected

### 2.2 Agent Pipeline Flow Tests
- Test: Research Agent output is passed to Content Agent
- Test: Content Agent output is passed to Compliance Agent
- Test: Compliance Agent output is passed to Review Agent
- Test: Review Agent can request Content Agent revision
- Test: Agent failure stops pipeline and records error

### 2.3 Directory Structure Tests
- Test: All new crates compile without errors
- Test: Workspace dependencies resolve correctly
- Test: Templates directory is accessible at runtime

---

## 3. Data Model Tests

### 3.1 Core Domain Type Tests
- Test: DocumentId generates valid UUID
- Test: DocumentState correctly maps to WorkflowStatus
- Test: TrustLevel determines approval requirements correctly
- Test: DomainConfig validates threshold ranges (0.0-1.0)
- Test: DocumentRequest serialization/deserialization

### 3.2 Template Data Tests
- Test: Template validates required variables
- Test: TemplateVariable resolves from UserInput source
- Test: TemplateVariable resolves from KnowledgeGraph source
- Test: TemplateVariable falls back to Default source when not found
- Test: VariableSource priority order is enforced

### 3.3 Compliance Data Tests
- Test: ComplianceResult calculates score correctly
- Test: WooRefusalGround reuse from existing compliance module
- Test: PiiLocation records position and type accurately
- Test: PiiType enum covers all required categories (BSN, Email, Phone, Address, IBAN, Name)

### 3.4 Storage Type Tests
- Test: StorageRef holds valid S3 bucket and key
- Test: DocumentVersion tracks created_by correctly
- Test: DocumentFormat enum covers Markdown, ODF, PDF

---

## 4. Agent Specification Tests

### 4.1 Research Agent Tests
- Test: execute_research_agent queries GraphRAG for similar documents
- Test: execute_research_agent extracts structure patterns from results
- Test: execute_research_agent identifies mandatory vs optional sections
- Test: execute_research_agent handles empty GraphRAG results gracefully
- Test: execute_research_agent returns ResearchContext with required fields

### 4.2 Content Generation Agent Tests
- Test: execute_content_agent loads correct template for document type
- Test: execute_content_agent fills variables from UserInput
- Test: execute_content_agent fills variables from KnowledgeGraph
- Test: execute_content_agent handles missing template variables
- Test: execute_content_agent generates valid Markdown output
- Test: execute_content_agent handles conditional sections correctly

### 4.3 Compliance Validator Agent Tests
- Test: execute_compliance_agent detects PII with known patterns
- Test: execute_compliance_agent redacts PII irreversibly using [PII: <type>] format
- Test: execute_compliance_agent validates Woo refusal grounds
- Test: execute_compliance_agent checks WCAG compliance requirements
- Test: execute_compliance_agent returns ComplianceResult with accurate scores
- Test: execute_compliance_agent separates original and redacted document storage

### 4.4 Review Agent Tests
- Test: execute_review_agent checks completeness of required sections
- Test: execute_review_agent validates compliance score meets threshold
- Test: execute_review_agent requires human approval for ALL Woo documents
- Test: execute_review_agent allows auto-approval only for non-Woo documents
- Test: execute_review_agent returns ReviewDecision with proper action

### 4.5 Pipeline Orchestration Tests
- Test: execute_document_pipeline runs all agents sequentially
- Test: execute_document_pipeline implements maker-checker iteration
- Test: execute_document_pipeline distinguishes permanent vs transient errors
- Test: execute_document_pipeline retries transient errors with exponential backoff
- Test: execute_document_pipeline fails fast on permanent errors
- Test: execute_document_pipeline records all agent results in audit trail

---

## 5. API Specification Tests

### 5.1 Document Creation API Tests
- Test: POST /api/documents/create returns 200 with valid document_id
- Test: POST /api/documents/create returns 400 for invalid domain_id
- Test: GET /api/documents/{id}/status returns current state and scores
- Test: POST /api/documents/{id}/approve requires authentication
- Test: POST /api/documents/{id}/approve requires document_approver role
- Test: POST /api/documents/{id}/approve returns 403 for unauthorized users
- Test: POST /api/documents/{id}/approve with MFA for high-sensitivity domains
- Test: GET /api/documents/{id}/audit returns complete audit trail

### 5.2 Template Management API Tests
- Test: GET /api/templates returns templates filtered by domain_id
- Test: POST /api/templates creates new template with validation
- Test: GET /api/documents/{id}/download returns binary content
- Test: GET /api/documents/{id}/download supports format parameter (odf, pdf, md)

### 5.3 Authentication/Authorization Tests
- Test: Unauthenticated requests return 401
- Test: Users without document_approver role cannot approve documents
- Test: MFA is required for high-sensitivity domain approvals
- Test: Approval trail records approved_by user ID

---

## 6. Database Schema Tests

### 6.1 Documents Table Tests
- Test: documents table creation succeeds with all columns
- Test: idx_documents_domain index exists
- Test: idx_documents_state index exists
- Test: idx_documents_domain_state compound index exists
- Test: idx_documents_created index exists for time-based queries

### 6.2 Document Versions Table Tests
- Test: document_versions table creation succeeds
- Test: document_versions stores full version history
- Test: document_versions enforces unique document_id, version_number constraint
- Test: idx_versions_document index exists
- Test: idx_versions_current index exists for current version queries
- Test: Rollback capability works using version_number

### 6.3 Audit Trail Table Tests
- Test: document_audit table creation succeeds
- Test: document_audit records all agent actions
- Test: idx_audit_document index exists
- Test: idx_audit_timestamp index exists with DESC ordering
- Test: Audit partitioning by created_at for performance (if implemented)

### 6.4 Templates Table Tests
- Test: templates table creation succeeds
- Test: idx_templates_domain_type unique constraint works
- Test: is_active flag filters inactive templates

### 6.5 Domain Configuration Table Tests
- Test: domain_configs table creation succeeds
- Test: domain_configs stores trust_level and thresholds correctly

---

## 7. AI Provider Configuration Tests

### 7.1 Provider Selection Tests
- Test: AiProviderConfig loads provider from environment variables
- Test: Primary provider failure triggers fallback to secondary provider
- Test: All providers exhausted returns error with details
- Test: Timeout configuration prevents hanging requests

### 7.2 Cost Tracking Tests
- Test: Tokens consumed are tracked per provider
- Test: Cost per document is calculated accurately
- Test: Monthly cost totals aggregate correctly
- Test: Budget alerts trigger when approaching limit

### 7.3 Key Management Tests
- Test: API keys load from environment at startup
- Test: Missing API keys prevent server startup
- Test: Invalid API keys return appropriate error

---

## 8. Implementation Phase Tests

### Phase 1: Foundation Tests
- Test: rust-s3 dependency resolves in Cargo.toml
- Test: iou-storage crate compiles with S3 client wrapper
- Test: Database migration 030_documents.sql applies successfully
- Test: Core domain types in iou-core/src/document.rs compile
- Test: GraphRAG Document entity schema is defined

### Phase 2: Template System Tests
- Test: Tera template engine initializes without errors
- Test: Template loads from database correctly
- Test: Variable substitution replaces all placeholders
- Test: Conditional sections render correctly based on variables
- Test: Markdown to ODF conversion produces valid ODF file
- Test: Initial templates (woo_besluit, woo_info) load and render

### Phase 3: Individual Agents Tests
- Test: Research Agent queries GraphRAG Document entities
- Test: Research Agent handles empty results gracefully
- Test: Content Agent fills template with variables
- Test: Content Agent handles missing KnowledgeGraph variables
- Test: Compliance Agent detects PII with >95% accuracy
- Test: Compliance Agent redacts PII irreversibly
- Test: Compliance Agent validates Woo rules using existing types
- Test: Review Agent checks all required sections present
- Test: Review Agent enforces human approval for Woo documents

### Phase 4: Pipeline Orchestration Tests
- Test: Sequential execution completes all agents
- Test: Maker-checker iteration loop terminates on approval
- Test: Maker-checker iteration loop terminates after max iterations
- Test: Permanent error stops pipeline immediately
- Test: Transient error triggers retry with exponential backoff
- Test: Audit trail entry created for each agent execution
- Test: Checkpoint saves after each agent for recovery

### Phase 5: API Layer Tests
- Test: All endpoints respond to unauthenticated requests with 401
- Test: POST /api/documents/create starts pipeline execution
- Test: GET /api/documents/{id}/status returns current state
- Test: POST /api/documents/{id}/approve requires approver role
- Test: GET /api/documents/{id}/audit returns audit trail
- Test: POST /api/templates creates template with validation
- Test: GET /api/documents/{id}/download returns correct format
- Test: OpenAPI documentation generates correctly

### Phase 6: Frontend Integration Tests
- Test: Document creation page renders without errors
- Test: Document creation form submits valid request
- Test: Approval workflow UI displays pending documents
- Test: Approval action updates document state
- Test: Audit trail viewer displays all entries
- Test: Template management UI creates and edits templates

---

## 9. Testing Strategy Tests

### 9.1 Unit Tests
- Test: Template variable substitution with edge cases
- Test: Compliance rule evaluation for all Woo refusal grounds
- Test: PII detection for all PiiType variants
- Test: State transition validation for all valid transitions
- Test: Invalid state transitions are rejected

### 9.2 Integration Tests
- Test: Full pipeline execution with mock agents
- Test: Error propagation through all layers
- Test: S3 storage integration with test bucket
- Test: DuckDB operations with test database

### 9.3 Compliance Tests
- Test: Known Woo-violating documents fail compliance
- Test: Known Woo-compliant documents pass compliance
- Test: PII detection for Dutch BSN format
- Test: PII detection for email addresses
- Test: PII detection for Dutch phone numbers
- Test: PII detection for IBAN
- Test: Refusal ground accuracy for Woo articles 5.1 and 5.2
- Test: WCAG AA compliance checks pass

### 9.4 End-to-End Tests
- Test: Complete document creation workflow
- Test: Document approval workflow with MFA
- Test: Document rejection and revision cycle
- Test: Document download in all formats
- Test: Version history and rollback

---

## 10. Success Criteria Tests

### 10.1 Functional Tests
- Test: All 4 agents execute successfully in sequence
- Test: Domain-specific trust levels configure correctly
- Test: Markdown templates convert to valid ODF
- Test: Compliance validation catches Woo violations
- Test: Full audit trail exists for every document

### 10.2 Non-Functional Tests
- Test: Fail-fast error handling works correctly
- Test: Integration with existing IOU-Modern components verified
- Test: Audit query performance completes within <100ms
- Test: Document creation completes within 30 seconds

### 10.3 Compliance Tests
- Test: PII detection accuracy >95%
- Test: No false negatives on Woo violations
- Test: All generated documents pass WCAG AA checks

---

## 11. Security Tests

### 11.1 PII Redaction Tests
- Test: Redacted PII cannot be reversed to original value
- Test: Redacted format follows [PII: <type>] pattern
- Test: Original and redacted documents stored separately
- Test: Only authorized roles can access unredacted originals
- Test: Audit trail records access to unredacted documents

### 11.2 Authentication Tests
- Test: All API endpoints require valid JWT token
- Test: Expired tokens are rejected with 401
- Test: Invalid tokens are rejected with 401

### 11.3 Authorization Tests
- Test: document_approver role required for approval endpoint
- Test: Users without approver role cannot approve documents
- Test: High-sensitivity domains require MFA

### 11.4 Template Injection Tests
- Test: User input cannot execute arbitrary template code
- Test: Template sandbox prevents unsafe operations
- Test: Variable values are properly escaped

---

## Appendix A: Integration Point Tests

### A.1 Existing Compliance AI Tests
- Test: Reuse WooMetadata from iou-core/src/compliance.rs
- Test: Reuse WooRefusalGround from iou-core/src/compliance.rs
- Test: Reuse WooDisclosureClass from iou-core/src/compliance.rs
- Test: Reuse AvgMetadata from iou-core/src/compliance.rs
- Test: Reuse PrivacyLevel from iou-core/src/compliance.rs
- Test: Reuse ComplianceStatus from iou-core/src/compliance.rs
- Test: Reuse ComplianceIssue from iou-core/src/compliance.rs

### A.2 Existing Workflow API Tests
- Test: DocumentState maps to existing WorkflowStatus
- Test: Workflow persistence uses existing patterns
- Test: Status query uses existing patterns

### A.3 Existing PROVISA Manager Tests
- Test: Version comparison UI patterns work with document_versions
- Test: Document type classification matches existing patterns
- Test: Domain selection interface works with new document types
