# Architecture Decision Records: IOU-Modern

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Command**: `/arckit:adr`

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-001-ADR-v1.0 |
| **Document Type** | Architecture Decision Records |
| **Project** | IOU-Modern (Project 001) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.0 |
| **Created Date** | 2026-03-20 |
| **Last Modified** | 2026-03-20 |
| **Review Cycle** | Per decision |
| **Owner** | Enterprise Architect |
| **Reviewed By** | PENDING |
| **Approved By** | PENDING |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---------|------|--------|---------|-------------|---------------|
| 1.0 | 2026-03-20 | ArcKit AI | Initial creation from `/arckit:adr` command | PENDING | PENDING |

---

## Decision Register

| ID | Decision | Date | Status | Impact |
|----|----------|------|--------|--------|
| ADR-001 | Rust + WebAssembly for frontend | 2026-03-20 | Accepted | High |
| ADR-002 | PostgreSQL + DuckDB hybrid database | 2026-03-20 | Accepted | High |
| ADR-003 | GraphRAG for knowledge extraction | 2026-03-20 | Accepted | Medium |
| ADR-004 | Human-in-the-loop for Woo publication | 2026-03-20 | Accepted | High |
| ADR-005 | Open-source first technology stack | 2026-03-20 | Accepted | High |
| ADR-006 | Domain-driven information organization | 2026-03-20 | Accepted | High |
| ADR-007 | Row-Level Security (RLS) for multi-tenancy | 2026-03-20 | Accepted | High |
| ADR-008 | MinIO/S3 for document storage | 2026-03-20 | Accepted | Medium |

---

## ADR-001: Rust + WebAssembly for Frontend

**Date**: 2026-03-20
**Status**: Accepted
**Context**: IOU-Modern requires a responsive web interface for government employees. Technology choice impacts performance, maintainability, and digital sovereignty.

### Decision

Use **Rust with WebAssembly (Wasm)** via **Dioxus 0.7** framework for the frontend application.

### Considered Options

| Option | Pros | Cons |
|--------|------|------|
| **Rust + Dioxus (Chosen)** | - Single language (Rust)<br>- Native performance<br>- Small Wasm binaries<br>- Type safety<br>- Digital sovereignty | - Smaller ecosystem than React<br>- Fewer UI components |
| React + TypeScript | - Large ecosystem<br>- Many components<br>- Familiar to devs | - JavaScript dependency<br>- Larger bundle size<br>- Digital sovereignty concerns |
| Vue 3 + TypeScript | - Progressive framework<br>- Good docs | - JavaScript dependency<br>- Smaller ecosystem than React |
| Svelte + TypeScript | - Compiled, small bundles | - JavaScript dependency<br>- Less mature for enterprise |

### Drivers

- **Digital Sovereignty**: Rust is open-source, European roots (Mozilla)
- **Performance**: WebAssembly provides near-native performance
- **Code Sharing**: Backend and frontend can share Rust types
- **Security**: Memory-safe Rust reduces vulnerabilities
- **Bundle Size**: Wasm binaries smaller than JavaScript bundles

### Consequences

**Positive**:
- Single development language reduces context switching
- Type safety across full stack
- Excellent performance for data-heavy UIs
- No JavaScript dependency

**Negative**:
- Smaller ecosystem than JavaScript frameworks
- Fewer ready-made UI components
| - Steeper learning curve for web developers
| - Dioxus framework less mature than React

### Related Decisions

- Enables ADR-002 (PostgreSQL + DuckDB) via shared types
- Supports ADR-005 (Open-source first)

---

## ADR-002: PostgreSQL + DuckDB Hybrid Database

**Date**: 2026-03-20
**Status**: Accepted
**Context**: IOU-Modern needs both transactional data integrity and analytical query performance. Single database technology may not satisfy both requirements.

### Decision

Use **PostgreSQL 15+** for transactional data and **DuckDB** for analytical queries in a hybrid architecture.

### Considered Options

| Option | Pros | Cons |
|--------|------|------|
| **PostgreSQL + DuckDB (Chosen)** | - ACID for transactions<br>- Columnar for analytics<br>- Open-source<br>- Mature tooling | - Complexity of two databases<br>- Data synchronization needed |
| PostgreSQL only | - Single database<br>- Mature tooling<br>- ACID | - Poor analytical performance<br>- Expensive scaling |
| Snowflake/BigQuery | - Excellent analytics | - Vendor lock-in<br>- No sovereignty<br>- High cost |
| ClickHouse | - Excellent analytics | - Poor transaction support<br>- Less mature for OLTP |

### Drivers

- **Compliance**: PostgreSQL has proven AVG/GDPR compliance
- **Performance**: DuckDB provides columnar analytics performance
- **Open Source**: Both technologies are fully open-source
- **Sovereignty**: No vendor lock-in
- **Cost**: No licensing fees

### Architecture

```
┌─────────────────┐
│  Application    │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
┌───▼───┐ ┌──▼─────┐
│  PG   │ │ DuckDB │
│(OLTP) │ │ (OLAP) │
└───────┘ └────────┘
```

### Data Flow

1. **Write Path**: Application writes to PostgreSQL (transactional)
2. **Read Path**:
   - Transactional queries → PostgreSQL
   - Analytics queries → DuckDB (with replicated data)
3. **Sync**: ETL pipeline syncs data from PostgreSQL to DuckDB

### Consequences

**Positive**:
- Optimal performance for both use cases
| - ACID guarantees for transactions
- Columnar performance for analytics
- Open-source, no vendor lock-in

**Negative**:
- Increased complexity (two databases)
- Data synchronization overhead
- Additional infrastructure to maintain

### Related Decisions

- Supported by ADR-001 (Rust type sharing)
- Enables ADR-007 (RLS in PostgreSQL)

---

## ADR-003: GraphRAG for Knowledge Extraction

**Date**: 2026-03-20
**Status**: Accepted
**Context**: IOU-Modern needs to extract entities and relationships from documents to enable knowledge discovery and cross-domain insights.

### Decision

Use **GraphRAG (Graph-based Retrieval Augmented Generation)** for knowledge extraction, using **petgraph** Rust library for graph operations.

### Considered Options

| Option | Pros | Cons |
|--------|------|------|
| **GraphRAG (Chosen)** | - Semantic understanding<br>- Cross-domain discovery<br>- Explainable relationships | - Complex implementation<br>- Privacy concerns (DPIA-004) |
| Simple NER | - Simple to implement<br>- Fast | - No relationships<br>- Limited insights |
| Vector search only | - Simple implementation | - Black box results<br>- No explainability |
| External KG service | - Managed service | - Vendor lock-in<br>- Data sovereignty concerns |

### Drivers

- **Knowledge Discovery**: Government needs cross-domain insights
- **Innovation**: GraphRAG is cutting-edge technology
- **Transparency**: Graph relationships are explainable (vs. black box AI)
- **Control**: Open-source implementation maintains sovereignty

### Implementation

```rust
// GraphRAG pipeline
1. Extract entities (NER)
2. Create nodes for each entity
3. Discover relationships (co-occurrence, semantic)
4. Create edges between nodes
5. Detect communities (clustering)
6. Generate summaries per community
```

### Privacy Safeguards

- Access controls on Person entities
- DPO oversight of relationship discovery
- Opt-out mechanism for entity extraction
- Audit logging of graph queries

### Consequences

**Positive**:
- Enables knowledge discovery across domains
- Explainable AI (relationships visible)
| - Innovative use of technology

**Negative**:
- DPIA required (high-risk processing)
- Privacy concerns (cross-domain disclosure)
- Complexity increases

### Related Decisions

- Requires ADR-004 (Human oversight)
- Informs DPIA (risk DPIA-004: Cross-domain relationship disclosure)

---

## ADR-004: Human-in-the-Loop for Woo Publication

**Date**: 2026-03-20
**Status**: Accepted
**Context**: Woo (Wet open overheid) publication has legal consequences. AI errors could result in wrongful disclosure or withholding of government information.

### Decision

**ALL Woo-relevant documents require human approval before publication**, regardless of AI confidence score.

### Considered Options

| Option | Pros | Cons |
|--------|------|------|
| **Human for ALL Woo (Chosen)** | - Legal liability clear<br>- No accidental disclosure<br>- Public trust | - Higher cost<br>- Slower process |
| Human only if AI uncertain | - Faster for clear cases | - Liability risk<br>- Potential errors |
| Fully automated | - Lowest cost<br>- Fastest | - Unacceptable legal risk<br>- No liability protection |

### Drivers

- **Legal Liability**: Human must make final decision for legal compliance
- **Public Trust**: Citizens expect human oversight of government decisions
- **Woo Compliance**: Wet open overheid requires accuracy in publication decisions
- **Accountability**: Audit trail must show human responsibility

### Implementation

```rust
match document {
    doc if doc.is_woo_relevant => {
        // Always require human approval for Woo
        ApprovalWorkflow::HumanApprovalRequired
    }
    doc if doc.compliance_score > threshold => {
        // Non-Woo documents can be auto-approved
        ApprovalWorkflow::AutoApprove
    }
    _ => ApprovalWorkflow::HumanApprovalRequired
}
```

### Consequences

**Positive**:
- Legal liability protected
- No accidental publication of sensitive information
- Public trust maintained

**Negative**:
- Higher operational cost
- Slower publication process
- Bottleneck if reviewers unavailable

### Related Decisions

- Informs DPIA (risk DPIA-007: Inaccurate Woo classification)
- Supports Principle P6 (Human-in-the-Loop AI)

---

## ADR-005: Open-Source First Technology Stack

**Date**: 2026-03-20
**Status**: Accepted
**Context**: Dutch government requires digital sovereignty. Proprietary vendors create lock-in and may compromise data jurisdiction.

### Decision

**Prioritize open-source software** for all technology choices. Proprietary software only used if no viable open-source alternative exists, with exit strategy.

### Considered Options

| Option | Pros | Cons |
|--------|------|------|
| **Open-source first (Chosen)** | - No vendor lock-in<br>- Digital sovereignty<br>- Code can be audited<br>- No licensing fees | - Higher development cost<br>- Need internal expertise |
| Proprietary SaaS | - Faster to implement<br>- Vendor support | - Lock-in<br>- Sovereignty concerns<br>- Ongoing licensing fees |

### Open-Source Choices Made

| Component | Technology | Alternative (proprietary) |
|------------|-----------|--------------------------|
| Backend | Rust (open-source) | .NET (Microsoft) |
| Frontend | Dioxus (open-source) | React (Meta) |
| Database | PostgreSQL (open-source) | SQL Server (Microsoft) |
| Analytics | DuckDB (open-source) | Snowflake (proprietary) |
| Storage | MinIO (open-source) | AWS S3 (proprietary) |
| Maps | Leaflet (open-source) | Google Maps (proprietary) |

### Exceptions Allowed

- **AI APIs**: May use commercial (OpenAI, Anthropic) if no EU alternative exists
- **Exit Strategy Required**: For each proprietary dependency, document migration path

### Consequences

**Positive**:
- Digital sovereignty maintained
- No vendor lock-in
- Code can be audited for security
| - No licensing fees

**Negative**:
- Higher internal development cost
- Need to maintain open-source expertise
- Some features may be less mature than proprietary alternatives

### Related Decisions

- Enables ADR-001 (Rust + WebAssembly)
- Enables ADR-002 (PostgreSQL + DuckDB)
- Supports Principle P4 (Sovereign Technology)

---

## ADR-006: Domain-Driven Information Organization

**Date**: 2026-03-20
**Status**: Accepted
**Context**: Government information is traditionally organized by folders or departments. This limits cross-domain insights and doesn't reflect how work is actually done.

### Decision

**Organize information by domain type**: Zaak (case), Project, Beleid (policy), Expertise.

### Considered Options

| Option | Pros | Cons |
|--------|------|------|
| **Domain-driven (Chosen)** | - Reflects actual work<br>- Enables cross-domain insights<br>- Semantic organization | - Change management required<br>- New paradigm |
| Folder hierarchy | - Familiar to users<br>- Simple | - Limits insights<br>- Departmental silos |
| Tag-based only | - Flexible | - No structure<br>- Difficult to navigate |

### Domain Types

| Domain Type | Description | Example |
|-------------|-------------|---------|
| **Zaak** | Executive work (permits, subsidies) | Vergunning aanbouw |
| **Project** | Temporary collaboration | Project nieuwbouw centrum |
| **Beleid** | Policy development | Beleid duurzaamheid |
| **Expertise** | Knowledge sharing and collaboration | Expertise ruimtelijke ordening |

### Implementation

```rust
pub enum DomainType {
    Zaak,      // Case/executive work
    Project,   // Temporary collaboration
    Beleid,    // Policy development
    Expertise, // Knowledge sharing
}
```

### Consequences

**Positive**:
- Reflects how government actually works
- Enables cross-domain knowledge discovery
- Better semantic search

**Negative**:
- Change management for users
| - Migration from folder-based systems
- Requires training

### Related Decisions

- Enables ADR-003 (GraphRAG cross-domain discovery)
- Supports Principle P5 (Domain-Driven Organization)

---

## ADR-007: Row-Level Security (RLS) for Multi-Tenancy

**Date**: 2026-03-20
**Status**: Accepted
**Context**: IOU-Modern serves multiple government organizations. Each organization must only see its own data, with controlled cross-organization sharing.

### Decision

Use **PostgreSQL Row-Level Security (RLS)** for organization-level data isolation.

### Considered Options

| Option | Pros | Cons |
|--------|------|------|
| **RLS in PostgreSQL (Chosen)** | - Database-level enforcement<br>- Transparent to application<br>- Proven security | - PostgreSQL-specific |
| Application-level filtering | - Database agnostic | - Error-prone (forgot WHERE clause)<br>- Bypassable |
| Separate databases per org | - Complete isolation | - Complex to manage<br>- Cross-org queries difficult |

### Implementation

```sql
-- Enable RLS on information_domains
ALTER TABLE information_domains ENABLE ROW LEVEL SECURITY;

-- Policy: Users can only see their organization's domains
CREATE POLICY org_isolation ON information_domains
    FOR SELECT
    USING (organization_id = current_setting('app.current_org_id')::uuid);
```

### Consequences

**Positive**:
- Database-level security (cannot be bypassed)
- Transparent to application code
- Proven PostgreSQL feature

**Negative**:
- PostgreSQL vendor lock-in
- Performance overhead for row filtering
| - Complex queries for cross-organization sharing

### Related Decisions

- Supports ADR-002 (PostgreSQL choice)
- Enables Principle P1 (Privacy by Design)

---

## ADR-008: MinIO/S3 for Document Storage

**Date**: 2026-03-20
**Status**: Accepted
**Context**: IOU-Modern stores large document files (PDFs, etc.) that don't fit well in database BLOBs. Need scalable, reliable object storage.

### Decision

Use **MinIO** (open-source S3-compatible) or **AWS S3** for document storage, with S3 API abstraction for portability.

### Considered Options

| Option | Pros | Cons |
|--------|------|------|
| **MinIO (Chosen if on-prem)** | - Open-source<br>- S3 compatible<br>- Self-hosted | - Operational overhead |
| **AWS S3 (Chosen if cloud)** | - Managed service<br>- High availability<br>- S3 standard | - Vendor lock-in<br>- Sovereignty concerns |
| Database BLOB | - Simple | - Poor performance<br>- Database bloat |
| File system | - Simple | - Scalability limits<br>- No replication |

### Decision Criteria

| Deployment | Storage Choice |
|------------|----------------|
| On-premises | MinIO (self-hosted) |
| AWS cloud | S3 (with governance concerns) |
| Azure cloud | Azure Blob (with S3 compatibility layer) |

### Implementation

```rust
// S3 client abstraction (supports MinIO, AWS S3, Azure Blob)
pub trait StorageClient: Send + Sync {
    async fn put_object(&self, key: &str, data: Bytes) -> Result<()>;
    async fn get_object(&self, key: &str) -> Result<Bytes>;
    async fn delete_object(&self, key: &str) -> Result<()>;
}
```

### Consequences

**Positive**:
- Scales to millions of documents
- S3 API provides portability
- CDN integration possible

**Negative**:
- Additional infrastructure to manage
- Eventual consistency (if distributed)

### Related Decisions

- Supports ADR-005 (Open-source preference with MinIO)
- Enables ADR-002 (Database focused on metadata only)

---

## Decision Template

Use this template for future ADRs:

```markdown
## ADR-XXX: [Decision Title]

**Date**: YYYY-MM-DD
**Status**: Proposed / Accepted / Deprecated / Superseded
**Context**: [Background information]

### Decision

[One-sentence decision statement]

### Considered Options

| Option | Pros | Cons |
|--------|------|------|
| **Option 1 (Chosen)** | | |
| Option 2 | | |
| Option 3 | | |

### Drivers

- Driver 1
- Driver 2
- Driver 3

### Consequences

**Positive**:
- Consequence 1
- Consequence 2

**Negative**:
- Consequence 1
- Consequence 2

### Related Decisions

- Links to related ADRs
```

---

## Glossary

| Term | Definition |
|------|------------|
| **ADR** | Architecture Decision Record |
| **RLS** | Row-Level Security (PostgreSQL feature) |
| **GraphRAG** | Graph-based Retrieval Augmented Generation |
| **Woo** | Wet open overheid (Government Information Act) |
| **S3** | Simple Storage Service (object storage API) |
| **MinIO** | Open-source S3-compatible object storage |
| **OLTP** | Online Transaction Processing |
| **OLAP** | Online Analytical Processing |
| **ACID** | Atomicity, Consistency, Isolation, Durability |

---

**END OF ADR**

## Generation Metadata

**Generated by**: ArcKit `/arckit:adr` command
**Generated on**: 2026-03-20
**ArcKit Version**: 4.3.1
**Project**: IOU-Modern (Project 001)
**AI Model**: Claude Opus 4.6
