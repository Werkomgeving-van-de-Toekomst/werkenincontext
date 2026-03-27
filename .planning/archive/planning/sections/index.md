<!-- PROJECT_CONFIG
runtime: rust-cargo
test_command: cargo test
END_PROJECT_CONFIG -->

<!-- SECTION_MANIFEST
section-01-foundation
section-02-template-system
section-03-research-agent
section-04-content-agent
section-05-compliance-agent
section-06-review-agent
section-07-pipeline-orchestration
section-08-api-layer
section-09-frontend-integration
END_MANIFEST -->

# Implementation Sections Index

## Dependency Graph

| Section | Depends On | Blocks | Parallelizable |
|---------|------------|--------|----------------|
| section-01-foundation | - | 02, 03, 05, 06 | Yes |
| section-02-template-system | 01 | 04 | No |
| section-03-research-agent | 01 | 04 | Yes |
| section-04-content-agent | 02, 03 | 07 | No |
| section-05-compliance-agent | 01 | 07 | Yes |
| section-06-review-agent | 01 | 07 | Yes |
| section-07-pipeline-orchestration | 04, 05, 06 | 08 | No |
| section-08-api-layer | 07 | 09 | No |
| section-09-frontend-integration | 08 | - | No |

## Execution Order

### Batch 1: Foundation
1. section-01-foundation (no dependencies)

### Batch 2: Independent Components (parallel after section-01)
2. section-02-template-system (foundation required)
3. section-03-research-agent (foundation required)
4. section-05-compliance-agent (foundation required)
5. section-06-review-agent (foundation required)

### Batch 3: Content Integration
6. section-04-content-agent (requires template-system AND research-agent)

### Batch 4: Pipeline
7. section-07-pipeline-orchestration (requires all agents)

### Batch 5: API
8. section-08-api-layer (requires pipeline)

### Batch 6: Frontend
9. section-09-frontend-integration (requires API)

## Section Summaries

### section-01-foundation
Core infrastructure setup: S3/MinIO client wrapper, DuckDB schema migrations, core domain types in iou-core, GraphRAG Document entity schema, storage client configuration.

### section-02-template-system
Markdown template rendering: Tera integration, template loading from database, variable substitution, conditional sections, Markdown to ODF conversion, initial templates (woo_besluit, woo_info).

### section-03-research-agent
GraphRAG integration: Query similar documents, extract structure patterns, identify mandatory/optional sections, retrieve domain context, return ResearchContext.

### section-04-content-agent
Document generation: Template loading and filling, variable resolution from UserInput/KnowledgeGraph/Defaults, entity linking, domain-specific content generation, Markdown output.

### section-05-compliance-agent
Woo validation and PII detection: Reuse existing compliance types, PII detection with NER, irreversible redaction ([PII: <type>] format), Woo refusal ground validation, WCAG compliance checks, separate storage for redacted/original.

### section-06-review-agent
Quality assurance: Completeness checks, clarity validation, compliance score verification, approval decision logic, iteration request for quality issues.

### section-07-pipeline-orchestration
Agent coordination: Sequential execution, maker-checker iteration loop, fail-fast error handling, transient error retry with exponential backoff, checkpoint/restart capability, audit trail logging.

### section-08-api-layer
REST endpoints: Document creation, status query, approval with auth/MFA, audit trail, template management (CRUD), document download with format selection, OpenAPI documentation.

### section-09-frontend-integration
Dioxus UI: Document creation page, approval workflow queue, audit trail viewer, template management interface, integration with existing compliance dashboard.
