# Evaluation Plan: Backend Database Selection for IOU-Modern

## Introduction

This document provides a comprehensive evaluation plan for selecting a backend database solution for the IOU-Modern platform. The evaluation compares three candidates:

1. **DuckDB** (current solution) - Embedded analytical database
2. **Convex** - Reactive backend platform
3. **Supabase** - PostgreSQL-based backend-as-a-service

The goal is to provide a clear recommendation with supporting analysis, risk assessment, and high-level migration considerations.

**Key Decision Factors:**
- Real-time collaboration is a **must-have** requirement
- Decision timeline is **urgent** (< 3 months)
- **Avoid vendor lock-in** (prefer open-source)
- **Self-hosting required** for Dutch government compliance

---

## Current State Analysis

### DuckDB Implementation

The IOU-Modern platform currently uses DuckDB as its primary database with the following schema:

**Core Tables:**
```
information_domains    - Organizational context (Zaak, Project, Beleid, Expertise)
information_objects    - Main document/content with full-text search
documents              - Document metadata with workflow states
templates              - Document templates for automated generation
audit_trail            - Full audit trail for compliance
```

**Key Views:**
```
v_searchable_objects   - Aggregates searchable text
v_compliance_overview  - Analytics for compliance metrics
v_domain_statistics    - Domain type/status distribution
v_entity_network       - GraphRAG entity relationships
```

**Current Limitations:**
1. Full-text search uses basic `ILIKE` instead of DuckDB's FTS extension
2. Single-writer architecture limits concurrent write operations
3. No built-in real-time data synchronization
4. Vector embeddings stored (as `FLOAT[]` arrays) but similarity search not implemented
5. S3 integration uses references only, no direct querying

### SQL Compatibility Notes

**Important for migration:** DuckDB is "PostgreSQL-compatible" but has key differences:

| Feature | DuckDB | PostgreSQL | Migration Impact |
|---------|--------|------------|-------------------|
| UUID generation | `uuid()` | `gen_random_uuid()` | Schema migration required |
| Array types | `VARCHAR[]` | `TEXT[]` or `VARCHAR[]` | Minor syntax differences |
| Full-text search | Basic `ILIKE` | `tsvector` with Dutch config | Major search semantics change |
| Connection model | `Arc<Mutex<Connection>>` | Connection pool (`PgPool`) | Architectural change required |

### Full-Text Search Migration Considerations

The current implementation uses `ILIKE` pattern matching. PostgreSQL's `tsvector` offers:

- **Pros:** Proper relevance ranking, Dutch language stemming (`dutch` text search configuration), better performance
- **Cons:** Different query semantics, requires index rebuild, relevance scoring differs

**Migration approach:** Prototype search in Phase 0 with actual Dutch government documents to validate relevance quality.

### Why Change?

The current DuckDB setup works well for **analytical workloads** but has critical gaps:

| Gap | Impact | Current Workaround |
|-----|--------|-------------------|
| No real-time sync | Users can't collaborate live | Custom WebSocket implementation |
| Single writer | Limits concurrent operations | Mutex-based access pattern |
| No built-in auth | Application-level auth only | Custom JWT middleware |
| Limited transactional performance | Not designed for OLTP | Not addressed |

---

## Candidate Evaluation

### 1. DuckDB (Status Quo)

**Architecture:** In-process, columnar-storage OLAP database
**License:** MIT (fully open-source)
**Language:** SQL (PostgreSQL-compatible)

**Strengths:**
- Excellent analytical query performance (columnar storage)
- Single-file deployment (no separate database server)
- Full SQL support with window functions and CTEs
- Works seamlessly with Rust (official bindings)
- Direct file access supports archival requirements

**Weaknesses:**
- No built-in real-time capabilities
- Single-writer limitation (only one process can write)
- Not designed for transactional/OLTP workloads
- No built-in authentication or authorization
- Limited support for concurrent writes

**Verdict:** **Retain for analytics** as part of hybrid solution

---

### 2. Convex

**Architecture:** Reactive backend platform with document database
**License:** FSL 1.1-Apache 2.0 (converts to Apache after 2 years)
**Language:** TypeScript functions (not SQL)

**Strengths:**
- Reactive queries with automatic real-time updates
- Sub-50ms latency at 5,000 concurrent connections
- Built-in authentication (Convex Auth with 80+ OAuth providers)
- Self-hostable since 2025 (Docker deployment)
- Automatic caching and query optimization
- End-to-end TypeScript type safety

**Weaknesses:**
- **FSL license is not FOSS** for 2 years (concerning for government)
- Non-standard query model (TypeScript, not SQL)
- Vendor lock-in concerns (tight coupling to Convex)
- Smaller ecosystem compared to PostgreSQL
- Migration requires code rewrite

**Verdict:** **Not recommended** due to FSL license and lock-in concerns

---

### 3. Supabase

**Architecture:** PostgreSQL-based backend-as-a-service
**License:** MIT (fully open-source)
**Language:** SQL + PostgreSQL extensions

**Strengths:**
- Built-in real-time via PostgreSQL logical replication
- Row-Level Security (RLS) for granular access control
- Multiple authentication providers (OAuth, SSO, magic links)
- Self-hostable with Docker Compose
- SOC 2 Type 2 and HIPAA compliant
- PostgreSQL ecosystem (proven, mature)
- Standard SQL (NORA alignment)

**Weaknesses:**
- More infrastructure to manage (multiple services)
- PostgreSQL not optimized for analytics (but can add extensions)
- Real-time via WAL has complexity (replication slot management)
- Higher latency than Convex (100-200ms vs <50ms)

**Verdict:** **Recommended** as primary database solution

---

## Comparison Matrix

| Feature | DuckDB | Convex | Supabase | Weight |
|---------|--------|--------|----------|--------|
| **Real-time capabilities** | ✗ | ✓✓✓ | ✓✓ | Critical |
| **Open-source license** | ✓✓✓ | ✗ | ✓✓✓ | Critical |
| **Self-hostable** | N/A | ✓ | ✓✓✓ | Critical |
| **SQL support** | ✓✓✓ | ✗ | ✓✓✓ | High |
| **Auth built-in** | ✗ | ✓✓ | ✓✓✓ | High |
| **Government compliance** | ✓ | ? | ✓✓✓ | High |
| **Analytics performance** | ✓✓✓ | ✓ | ✓ | Medium |
| **Rust integration** | ✓✓✓ | ✗ | ✓ | Medium |
| **Vendor lock-in risk** | None | High | Low | Critical |
| **Real-time latency** | N/A | <50ms | 100-200ms | Medium |

**Scoring:**
- **Supabase:** 8/9 critical/high features met
- **DuckDB:** 3/9 (excellent for analytics niche)
- **Convex:** 5/9 (strong on real-time, weak on lock-in)

---

## Compliance Assessment

### Dutch Government Requirements

**NORA (Nederlandse Overheid Referentie Architectuur):**
- Requires standard, open solutions
- Preference for SQL-based systems
- Building blocks must be interoperable

| Requirement | Supabase | Convex | DuckDB |
|-------------|----------|--------|--------|
| **Standard SQL** | ✓ (PostgreSQL) | ✗ (TypeScript) | ✓ (PostgreSQL-compatible) |
| **Open source** | ✓ (MIT) | ✗ (FSL for 2 years) | ✓ (MIT) |
| **Interoperability** | ✓ (PostgreSQL ecosystem) | Limited | ✓ (SQL standard) |

**Archiefwet (Archives Act):**
- Records must remain findable, readable, usable for 10+ years
- Selectielijsten (retention schedules) must be supportable

| Requirement | Supabase | Convex | DuckDB |
|-------------|----------|--------|--------|
| **Data export** | ✓ (pg_dump) | ✓ (ZIP) | ✓ (file copy) |
| **Long-term storage** | ✓ (standard tools) | Requires export | ✓ (direct file) |
| **Retention policies** | ✓ (SQL + extensions) | Manual | Manual |

**GDPR/AVG:**
- Right to be forgotten
- Data portability
- Audit trails

All three solutions can support GDPR requirements with appropriate application-level implementation. Supabase has advantage with proven SOC 2 Type 2 certification.

**Bijhoudingsplicht (Record-Keeping Obligation):**
- Every data change must be auditable
- Migration itself creates audit events that must be preserved
- Audit trail continuity required during migration (correlate DuckDB logs with Supabase WAL)

**Basisveiligheid Overheid (BVO) - Minimum Security:**
- Multi-factor authentication required
- Secure logging (SIEM integration capability)
- Regular security assessments
- Encryption at rest and in transit

### Dutch Government Authentication

**Critical for government use cases:**

| Provider | Type | Supabase Support | Notes |
|----------|------|------------------|-------|
| **DigiD** | SAML2/OIDC | Not native | Requires custom SAML integration |
| **eHerkenning** | SAML2 | Not native | Requires custom SAML integration |
| **OIN** | Organization ID | Application-level | Store as organization identifier |

**Action:** DigiD/eHerkenning integration is **out of scope** for database evaluation. Assume application-level integration via existing middleware, with Supabase storing the resulting user identity claims.

### Hosting Location Decision

**Data sovereignty requirements:**

| Hosting Option | EU Data Residency | Dutch Control | Recommendation |
|----------------|-------------------|---------------|----------------|
| Rijkscloud | ✓ | ✓ | Preferred if available |
| Gemeente Shared Services | ✓ | ✓ | Good for municipalities |
| Commercial EU (Hetzner/Azure NL) | ✓ | Partial | Acceptable |
| On-premises | ✓ | ✓ | Best control, highest operational burden |

**Decision:** Hosting location must be determined before procurement. Consider existing government infrastructure agreements.

### Selectielijst Implementation

**Retention schedule automation with PostgreSQL:**

```sql
-- Example: Automated retention enforcement
CREATE EXTENSION IF NOT EXISTS pg_cron;

-- Function to check and apply retention rules
CREATE OR REPLACE FUNCTION enforce_retention()
RETURNS void AS $$
BEGIN
  -- Archive documents past retention period
  UPDATE documents
  SET status = 'archived'
  WHERE retention_date < CURRENT_DATE
  AND status = 'active';

  -- Delete documents past destruction date
  DELETE FROM documents
  WHERE destruction_date < CURRENT_DATE;
END;
$$ LANGUAGE plpgsql;

-- Schedule daily check at 2 AM
SELECT cron.schedule('check-retention', '0 2 * * *', $$SELECT enforce_retention()$$);
```

### Woo Publication Requirements

**Wet open overheid (Woo) compliance:**

- Automatic publication to Woo-index required for public documents
- API endpoints for Woo requests
- Metadata standards compliance (DCAT, TOOI)
- Proactive publication obligations

**Integration:** Supabase RLS policies will enforce publication status. Application layer handles Woo-index API interactions.

---

## Recommended Solution: Hybrid Architecture

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        IOU-Modern Platform                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌────────────────────┐          ┌────────────────────┐        │
│  │     Supabase       │          │      DuckDB        │        │
│  │   (Primary DB)     │          │   (Analytics DB)   │        │
│  │                    │          │                    │        │
│  │  • Users/Auth      │          │  • OLAP queries    │        │
│  │  • Documents       │◄────────►│  • Analytics       │        │
│  │  • Real-time       │   ETL    │  • Reporting       │        │
│  │  • RLS policies    │          │  • Data science    │        │
│  └────────────────────┘          └────────────────────┘        │
│           │                               ▲                     │
│           │                               │                     │
│           ▼                               │                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  Rust/Axum Backend                      │   │
│  │                                                          │   │
│  │  • API layer                                            │   │
│  │  • Business logic                                       │   │
│  │  • Orchestrator (workflow)                             │   │
│  │  • WebSocket bridge (if needed)                         │   │
│  │  • ETL coordinator                                      │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

**Supabase (Primary Database):**
- User management and authentication
- Document metadata and workflow states
- Information domains and objects
- Real-time subscriptions for live collaboration
- Row-Level Security for multi-tenant isolation

**DuckDB (Analytics Database):**
- Periodic ETL import from Supabase
- Materialized views for reporting
- Full-text search index
- GraphRAG entity relationship queries
- Vector embeddings for similarity search (FLOAT[] arrays)
- Data science and BI exports

**Sync Mechanism:**
- Batch ETL every N minutes (configurable, starting at 5 minutes)
- Or streaming via CDC (PostgreSQL logical decoding)
- Idempotent design for safe retries

**ETL Consistency Strategy:**

The ETL pipeline must handle:
- Write amplification (dual-write during migration)
- Partial failure scenarios
- Data drift during ETL windows

**Recommended approach: Transactional outbox pattern**

```rust
// Write to outbox table first
INSERT INTO change_outbox (aggregate_type, aggregate_id, event_type, payload)
VALUES ('document', doc_id, 'created', json_payload);

// Background worker processes outbox
// Writes to both Supabase and DuckDB
// Marks as processed only after both succeed
```

**CDC alternative:** Use PostgreSQL logical decoding for near real-time sync (lower latency, higher complexity).

### Vector Search Architecture Decision

**Vectors stay in DuckDB for analytics**

- DuckDB: `FLOAT[]` arrays for batch similarity queries
- Supabase: Optional `pgvector` extension for real-time similarity (defer to Phase 2 evaluation)
- Decision point: Evaluate if real-time vector search is required during Phase 0

---

## Migration Strategy

**Updated Timeline:** 12 weeks (extended from 8 for buffer and assessment)

### Phase 0: Assessment (Weeks 1-2)

**Objective:** Gather baselines and document current state before migration

**Tasks:**
1. **Performance baseline:**
   - Measure current query latency (p50, p95, p99)
   - Document concurrent user capacity
   - Record database size and growth rate
   - Profile WebSocket connection limits

2. **Current authentication documentation:**
   - Document existing JWT middleware implementation
   - Catalog user schema and password hashing
   - Identify session management patterns

3. **Real-time requirements validation:**
   - Confirm actual collaboration features needed
   - Define acceptable latency thresholds
   - Test with realistic Dutch government network conditions

4. **Analytics workload characterization:**
   - Profile typical analytical queries
   - Define dashboard refresh requirements
   - Document data science usage patterns

5. **Hosting location decision:**
   - Evaluate hosting options (Rijkscloud, Gemeente, commercial EU, on-prem)
   - Procurement process initiation

**Validation:**
- Performance baseline documented
- Current implementation fully understood
- Hosting location decided
- Stakeholder sign-off on migration approach

### Phase 1: Foundation (Weeks 3-4)

**Objective:** Set up Supabase alongside DuckDB (zero downtime)

**Tasks:**
1. Deploy self-hosted Supabase (Docker Compose)
2. Create schema in PostgreSQL matching DuckDB tables
3. Set up database connection from Rust backend
4. Implement dual-write pattern (write to both databases)
5. Create read toggle for gradual cutover

**Validation:**
- Can write to both databases simultaneously
- Data consistency between databases
- No performance degradation

### Phase 2: Auth & Real-time (Weeks 5-6)

**Objective:** Implement Supabase auth and real-time features

**Tasks:**
1. Configure Supabase Auth (providers, RLS)
2. Migrate user data to Supabase
3. Implement real-time subscriptions
4. Update frontend to consume real-time updates
5. Evaluate: Keep custom WebSocket for document workflow or migrate to Supabase Realtime

**Note:** Custom WebSocket provides fine-grained control (document status broadcasts) that Supabase Realtime may not match. Consider hybrid approach.

**Validation:**
- Users can authenticate via Supabase
- Real-time updates working
- Multi-user collaboration functional

### Phase 3: Cutover (Weeks 7-9)

**Objective:** Migrate primary traffic to Supabase

**Tasks:**
1. Update API endpoints to read from Supabase
2. Implement ETL from Supabase to DuckDB
3. Configure DuckDB for analytics-only workload
4. Remove dual-write pattern
5. Monitor and optimize

**Validation:**
- All read traffic from Supabase
- Analytics working via DuckDB
- Performance meets SLA

### Phase 4: Stabilization (Weeks 10-11)

**Objective:** Monitor and optimize after cutover

**Tasks:**
1. Monitor production performance metrics
2. Optimize slow queries
3. Tune RLS policy performance
4. Stabilize ETL pipeline
5. Address any production issues

**Validation:**
- Performance meets or exceeds baseline
- ETL latency acceptable
- No data inconsistencies
- User feedback positive

### Phase 5: Cleanup (Week 12)

**Objective:** Remove legacy code and optimize

**Tasks:**
1. Remove custom WebSocket code
2. Remove DuckDB transactional queries
3. Optimize ETL pipeline
4. Update documentation
5. Decommission old authentication

**Validation:**
- Technical debt reduced
- Documentation updated
- Team trained on new architecture

---

## Rollback Procedures

**Trigger Criteria:**
- Data inconsistency detected between databases
- Real-time latency exceeds 500ms for >5 minutes
- RLS policy performance degrades (p95 > 1s)
- Authentication failures > 1% of requests

**Rollback Process:**

1. **Immediate rollback (during Phase 1-2):**
   - Switch read toggle back to DuckDB
   - Stop dual-write
   - Reconcile any data written to Supabase only

2. **Data reconciliation script:**
```sql
-- Find records in Supabase not in DuckDB
SELECT id FROM supabase.documents
WHERE id NOT IN (SELECT id FROM duckdb.documents);

-- Reconcile by writing missing records to DuckDB
```

3. **Post-rollback actions:**
   - Root cause analysis
   - Fix applied to migration approach
   - Re-migrate after fix validation

**Note:** After Phase 3 cutover, rollback becomes significantly more complex. Consider a "freeze period" after cutover before cleanup.

---

## Operational Requirements

### Monitoring Stack

**Required monitoring for self-hosted Supabase:**

| Metric | Tool | Alert Threshold |
|--------|------|-----------------|
| PostgreSQL query performance | pg_stat_statements | p95 > 500ms |
| Replication lag | pg_stat_replication | lag > 30s |
| Replication slot WAL size | pg_replication_slots | WAL > 1GB |
| Real-time subscription count | Custom metrics | Abnormal spike |
| Database connections | pg_stat_activity | > 80% of max |
| Disk space | System metrics | < 20% free |

**Integration:** Export to existing SIEM/log aggregation system for BVO compliance.

### Backup and Recovery

**PostgreSQL backup strategy:**

| Backup Type | Frequency | Retention | RTO | RPO |
|-------------|-----------|-----------|-----|-----|
| Base backup | Daily | 30 days | 4 hours | 15 min |
| WAL archive | Continuous | 30 days | - | - |
| Logical export | Weekly | 90 days | 24 hours | 1 week |

**Requirements:**
- Backup encryption at rest (AES-256)
- Regular recovery testing (monthly)
- Disaster recovery documentation

**Note:** DuckDB's single-file backup advantage is lost. Supabase requires more comprehensive backup infrastructure.

---

## Risk Assessment

### High-Risk Items

| Risk | Impact | Mitigation |
|------|--------|------------|
| **ETL consistency** | Data inconsistency between databases | Transactional outbox pattern, idempotent design, monitoring |
| **RLS policy complexity** | Performance issues, difficult debugging | Prototype in Phase 1, benchmark, simplify policies |
| **Real-time performance** | Latency unacceptable for users | POC testing with Dutch gov networks, optimize queries, add caching |
| **Migration downtime** | Service interruption during cutover | Blue-green deployment, dual-write phase, rollback procedures |
| **Team expertise** | Team unfamiliar with PostgreSQL/Supabase | Training, documentation, gradual migration |

### Medium-Risk Items

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Resource requirements** | Supabase requires more infrastructure | Right-sizing, monitoring, auto-scaling |
| **Regulatory compliance** | Supabase deployment not compliant | Self-hosting, SOC 2 leverage, audit prep |
| **Feature gaps** | Supabase missing some DuckDB features | Hybrid approach, custom functions |

### Low-Risk Items

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Vendor lock-in** | Limited to PostgreSQL | Standard SQL, portable, export tools |
| **Cost increase** | Additional hosting costs | DuckDB reduction, right-sizing |

---

## Success Criteria

The migration will be considered successful when:

1. **Real-time collaboration** is working with <200ms latency
2. **Authentication** is handled by Supabase with RLS enforcement
3. **DuckDB** is successfully used for analytics workloads only
4. **Zero data loss** during migration
5. **Performance** meets or exceeds current benchmarks
6. **Compliance** requirements are satisfied (GDPR, Woo, Archiefwet)
7. **Team** is trained and comfortable with new architecture

---

## Recommendation

**Primary Recommendation: Supabase**

**Summary:** Supabase is the recommended solution for IOU-Modern's primary database, with DuckDB retained for analytics in a hybrid architecture.

**Key Reasons:**
1. Meets all critical requirements (real-time, self-hosting, open-source)
2. PostgreSQL foundation ensures NORA alignment
3. SOC 2/HIPAA compliance supports government requirements
4. Real-time capabilities sufficient for collaboration needs
5. Proven technology with strong ecosystem

**Confidence Level:** High

This recommendation is based on:
- Comprehensive research of all three candidates
- Analysis of current codebase and limitations
- User interview confirming requirements and priorities
- Dutch government compliance requirements
- Technical feasibility assessment

**Next Steps:**
1. Stakeholder review and approval
2. Proof-of-concept for real-time performance validation
3. Detailed migration planning (if approved)
4. Infrastructure procurement for Supabase deployment
