# Opus Review

**Model:** claude-opus-4
**Generated:** 2026-03-11T18:34:00Z

---

Now I have a comprehensive understanding of the plan and the codebase. Let me provide a thorough review.

---

# Review: Backend Database Selection Evaluation Plan

## Executive Summary

This is a well-structured evaluation plan that demonstrates good understanding of the Dutch government compliance landscape. The recommendation for Supabase is reasonably justified, but there are several critical gaps, technical oversights, and risks that need to be addressed before proceeding.

**Overall Assessment: 7/10** - Solid foundation but needs significant additions on migration complexity, compliance specifics, and operational considerations.

---

## 1. Completeness - Critical Gaps

### 1.1 Missing: Bijhoudingsplicht and Audit Trail Continuity

**Section: Migration Strategy, Phase 1-4**

The plan does not address how the Dutch **Bijhoudingsplicht** (record-keeping obligation) will be maintained during migration. For government systems:

- Every data change must be auditable
- Migration itself creates audit events that must be preserved
- The plan mentions "zero data loss" but not "audit continuity"

**Recommendation:** Add a section on audit trail preservation during migration, including how Supabase's WAL-based audit will be correlated with existing DuckDB audit records.

### 1.2 Missing: Selectielijst Implementation Details

**Section: Compliance Assessment**

The plan mentions retention schedules but lacks implementation detail:

```sql
-- Current schema has this, but no clear migration path to Supabase:
retention_period INTEGER,
retention_trigger VARCHAR,
destruction_date DATE,
```

Supabase/PostgreSQL will need automated retention enforcement, possibly with:

```sql
-- Example missing from plan:
CREATE EXTENSION IF NOT EXISTS pg_cron;
SELECT cron.schedule('check-retention', '0 2 * * *', $$SELECT enforce_retention()$$);
```

### 1.3 Missing: Full-Text Search Migration Strategy

**Section: Current State Analysis**

The plan correctly identifies that current search uses `ILIKE` but doesn't address how Supabase's full-text search will compare:

- DuckDB FTS vs PostgreSQL `tsvector`
- Dutch language support (`dutch` text search configuration)
- Ranking/relevance differences
- Migration of existing search indexes

**Current code (db.rs:346-393):**
```rust
// DuckDB doesn't have built-in FTS like PostgreSQL
let search_pattern = format!("%{}%", query.to_lowercase());
```

Supabase will use completely different search semantics. This is a major functional change not adequately addressed.

### 1.4 Missing: GraphRAG Vector Search Migration

**Section: Current Limitations**

The plan mentions "Vector embeddings stored but similarity search not implemented" but doesn't explain how the hybrid architecture will handle this:

- DuckDB: `FLOAT[]` arrays stored directly
- Supabase: Will need `pgvector` extension
- The ETL direction is unclear: vectors in Supabase or DuckDB?

**Recommendation:** Clarify whether vectors live in Supabase (with `pgvector`) for real-time similarity search, or stay in DuckDB for batch analytics.

### 1.5 Missing: Connection Pooling and Concurrency

**Section: Architecture Overview**

The current DuckDB implementation uses `Arc<Mutex<Connection>>` (single connection):

```rust
// db.rs:49-51
pub struct Database {
    pub(crate) conn: Arc<Mutex<Connection>>,
}
```

Supabase/PostgreSQL will require a proper connection pool. The plan should mention:

- Pool size configuration
- Transaction isolation levels
- Connection lifecycle management
- Impact on existing async wrappers

---

## 2. Accuracy - Technical Concerns

### 2.1 Convex Self-Hosting Reality

**Section: Convex Evaluation**

The plan states "Self-hostable since 2025 (Docker deployment)" but this is misleading. As of early 2026:

- Convex self-hosting is in beta/early access
- Requires THREE separate services (Convex backend, database, deployment)
- Missing some managed features (like Convex Auth) in self-hosted mode
- Migration tooling is immature

**Recommendation:** Downgrade Convex self-hosting claims or add explicit caveats about production readiness.

### 2.2 Supabase Real-time Latency Claims

**Section: Comparison Matrix**

The plan states "100-200ms" for Supabase real-time. This is optimistic for:

- Self-hosted deployments (not the optimized Supabase Cloud)
- Dutch government infrastructure (often slower networks)
- Complex RLS queries affecting subscription performance

**Recommendation:** Add performance testing in Phase 1 with realistic Dutch government network conditions.

### 2.3 PostgreSQL vs DuckDB SQL Compatibility

**Section: DuckDB Evaluation**

The plan claims DuckDB is "PostgreSQL-compatible SQL" but this is incomplete. Key differences that will break migration:

| Feature | DuckDB | PostgreSQL | Impact |
|---------|--------|------------|--------|
| `ILIKE` | Yes | Yes | OK |
| Array types | `VARCHAR[]` | `VARCHAR[]` or `TEXT[]` | Syntax differences |
| JSON functions | `->` operator | `->` operator | Mostly compatible |
| Window functions | Full support | Full support | OK |
| `TIMESTAMPTZ` | Yes | Yes | OK |
| `uuid()` | Yes | `gen_random_uuid()` | **Will break** |

**Code that will break:**
```sql
-- DuckDB (current)
id UUID PRIMARY KEY DEFAULT uuid(),

-- PostgreSQL (Supabase)
id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
```

**Recommendation:** Add a SQL compatibility migration guide.

---

## 3. Risk Assessment - Additional Risks

### 3.1 ETL Consistency Risk (HIGH)

**Section: Risk Assessment**

The plan mentions "ETL complexity" but underestimates the challenge:

**Current pattern:**
```rust
// Dual-write pattern in Phase 1
db_supabase.create_domain(domain)?;
db_duckdb.create_domain(domain)?;
```

**Risks not addressed:**
1. Write amplification (2x database I/O)
2. Partial failure scenarios (one succeeds, one fails)
3. Eventual consistency during dual-write phase
4. Data drift if ETL lags

**Recommendation:** Add a detailed transactional outbox pattern or change data capture (CDC) architecture.

### 3.2 RLS Policy Complexity Risk (HIGH)

**Section: Supabase Evaluation**

Row-Level Security is listed as a strength but the complexity is underestimated:

**Current schema has multi-tenant patterns:**
```sql
organization_id UUID NOT NULL,
owner_user_id UUID,
classification VARCHAR NOT NULL DEFAULT 'intern',
```

**Supabase RLS policies needed:**
- Organization isolation
- User-level access within organization
- Classification-based filtering
- Woo-publication status
- Cross-domain relationships

**Risk:** Complex RLS policies can cause performance issues and are difficult to debug.

**Recommendation:** Add RLS policy prototyping in Phase 1 with performance benchmarking.

### 3.3 Authentication Migration Disruption (HIGH)

**Section: Migration Strategy, Phase 2**

The plan states "Migrate user data to Supabase" but doesn't address:

- Current authentication implementation (what exists?)
- Password hashing algorithm compatibility
- Session token migration
- Multi-factor authentication (if any)
- SSO integration requirements for government

**Grep search found:**
```rust
// crates/iou-api/src/middleware/auth.rs exists but implementation not shown
```

**Recommendation:** Document current auth implementation before Phase 2 planning.

### 3.4 Replication Slot Management (MEDIUM)

**Section: Supabase Weaknesses**

"Real-time via WAL has complexity (replication slot management)" - this is a significant operational burden:

```sql
-- Manual monitoring needed:
SELECT slot_name, slot_type, active,
       pg_size_pretty(wal_status::bigint) as wal_size
FROM pg_replication_slots;
```

**Risk:** Unmanaged slots can cause disk exhaustion and database shutdown.

**Recommendation:** Add monitoring and alerting requirements for replication slots.

### 3.5 Data Sovereignty and Hosting Location (CRITICAL for Dutch Government)

**Section: Compliance Assessment**

Missing consideration: Where will Supabase be hosted?

- Dutch government data often must remain within EU
- Some municipalities require national hosting
- Self-hosting location matters (Rijkshuisstijl hosting requirements)

**Question:** Will this be hosted on:
- Dutch government cloud (Rijkscloud)
- Municipality infrastructure (Gemeente Shared Services)
- Commercial EU provider (Hetzner, Azure NL)
- On-premises?

**Recommendation:** Add hosting location decision criteria.

---

## 4. Recommendation Strength - Analysis

### 4.1 Supabase Recommendation is Well-Justified

**Strengths of the argument:**
- NORA alignment (SQL standard) is correctly prioritized
- SOC 2/HIPAA certifications are relevant (even if not Dutch-specific)
- PostgreSQL ecosystem maturity
- Self-hosting capability for data sovereignty

### 4.2 But Missing Key Consideration: Postgres vs DuckDB Concurrency Model

**Current code pattern:**
```rust
// Single connection with mutex
pub struct Database {
    pub(crate) conn: Arc<Mutex<Connection>>,
}
```

**PostgreSQL pattern:**
```rust
// Connection pool with concurrent access
pg_pool::PgPool::connect(url)
```

This is a fundamental architectural change that affects:
- All database access code
- Transaction handling
- Error recovery
- Performance characteristics

**The migration effort is underestimated.** Every database call in `/Users/marc/Projecten/iou-modern/crates/iou-api/src/db.rs` (1500+ lines) needs reconsideration.

---

## 5. Migration Strategy - Concerns

### 5.1 Timeline Optimism

**8 weeks for full migration is aggressive:**

| Phase | Duration | Reality Check |
|-------|----------|---------------|
| Phase 1: Foundation | 2 weeks | Schema translation alone could take 2 weeks |
| Phase 2: Auth & Real-time | 2 weeks | Auth migration is high-risk |
| Phase 3: Cutover | 2 weeks | ETL stabilization often takes longer |
| Phase 4: Cleanup | 2 weeks | Typically delayed by production issues |

**Recommendation:** Add buffer weeks and consider a phased rollout by domain type (e.g., start with "expertise" domains before "zaak").

### 5.2 Rollback Strategy Missing

**Section: Migration Strategy**

No mention of how to rollback if issues occur:

- Can we revert from Supabase back to DuckDB?
- How to handle data that was written to Supabase but not DuckDB during dual-write?
- What's the rollback trigger criteria?

**Recommendation:** Add rollback procedures with data reconciliation scripts.

### 5.3 Zero-Downtime Claim Unclear

**Phase 1: "zero downtime"**

With `Arc<Mutex<Connection>>` pattern, adding a second database introduces:
- Increased lock contention
- Potential deadlock scenarios
- Transaction spanning both databases (not supported)

**Recommendation:** Clarify what "zero downtime" means and how it's measured.

---

## 6. Compliance - Dutch Government Specifics

### 6.1 Missing: DIGID and EHerkenning Integration

**Section: Compliance Assessment**

Dutch government authentication typically requires:
- **DigiD** for citizen access
- **EHerkenning** for business access
- **OIN** (Organisatie Identificatienummer) for organization identification

Supabase Auth supports OAuth, but DigiD/EHerkenning require specific SAML2/ODIC integration.

**Current plan mentions:** "80+ OAuth providers" but DigiD is NOT a standard OAuth provider.

**Recommendation:** Add explicit DigiD/EHerkenning integration requirements or confirm they're out of scope.

### 6.2 Missing: Woo Publication Requirements

**Section: Compliance Assessment**

The plan mentions `is_woo_relevant` but doesn't address:

- Automatic publication to Woo-index
- API requirements for Woo requests
- Metadata standards for Woo documents
- Proactive publication obligations

**Recommendation:** Add Woo publication workflow integration.

### 6.3 Missing: Basisveiligheid Overheid (BVO)

Not mentioned in the plan. BVO includes:
- Multi-factor authentication
- Secure logging (SIEM integration)
- Regular security assessments
- Encryption standards

**Recommendation:** Add BVO compliance checklist.

---

## 7. Architecture - Hybrid Approach Concerns

### 7.1 Two Databases, Two Problems

The hybrid architecture is sensible but introduces:

```
┌─────────────────────┐     ETL/CDC     ┌─────────────────────┐
│     Supabase        │ ──────────────> │      DuckDB         │
│   (Primary DB)      │                 │   (Analytics DB)    │
└─────────────────────┘                 └─────────────────────┘
         ▲                                        │
         │                                        │
         └────────────────────────────────────────┘
                    Which is source of truth?
```

**Questions not answered:**
1. Which database is authoritative during ETL failure?
2. How to handle analytics queries during ETL window?
3. What's the acceptable staleness for analytics?

### 7.2 WebSocket Deprecation Risk

**Phase 2: "Deprecate custom WebSocket implementation"**

Current WebSocket code is well-structured:
```rust
// crates/iou-api/src/websockets/documents.rs
pub struct WebSocketState {
    pub status_tx: broadcast::Sender<DocumentStatus>,
}
```

Supabase Realtime has different semantics:
- Channel-based subscriptions
- Broadcast to all subscribers
- Less control over message filtering

**Risk:** Replacing custom WebSocket with Supabase Realtime may reduce functionality.

**Recommendation:** Consider keeping custom WebSocket for document workflow and use Supabase Realtime for general data synchronization.

---

## 8. Performance - Missing Analysis

### 8.1 No Performance Baseline

The plan doesn't establish current performance:

- Current query latency (p50, p95, p99)
- Current concurrent user capacity
- Current database size and growth rate
- Current WebSocket connection limits

**Recommendation:** Add performance baseline gathering in pre-migration phase.

### 8.2 Analytics Workload Characterization

"Periodic ETL every N minutes (configurable)" - but what are the actual analytics requirements?

- Dashboard refresh requirements?
- Report generation SLAs?
- Data science query patterns?

**Current DuckDB is excellent for analytics** - moving to PostgreSQL for analytics may be disappointing.

---

## 9. Operational Concerns

### 9.1 Monitoring and Observability

Missing from the plan:

- Supabase self-hosted monitoring stack
- PostgreSQL query performance monitoring
- Replication lag monitoring
- Real-time subscription monitoring

### 9.2 Backup and Recovery

"Single-file deployment (no separate database server)" is a DuckDB advantage lost with Supabase:

- PostgreSQL backup strategy (pg_dump, WAL archiving, PITR)
- Backup encryption requirements
- Recovery testing procedures
- RTO/RPO targets

---

## 10. Specific Actionable Recommendations

### High Priority

1. **Add pre-migration assessment sprint (2 weeks)** to gather:
   - Performance baseline
   - Current authentication implementation details
   - Actual real-time collaboration requirements
   - Analytics workload characterization

2. **Create SQL compatibility matrix** showing all schema differences and migration scripts.

3. **Add RLS policy prototype** with performance benchmarks.

4. **Define ETL consistency strategy** (transactional outbox or CDC).

5. **Clarify hosting location** and data sovereignty requirements.

### Medium Priority

6. **Add Woo publication workflow** to compliance assessment.

7. **Document rollback procedures** with data reconciliation.

8. **Extend timeline** to 12 weeks with buffer phases.

9. **Add DigiD/EHerkenning integration** requirements or confirm out of scope.

10. **Consider keeping DuckDB for analytics** as the primary, not secondary, for analytics workloads.

### Low Priority

11. **Add BVO compliance checklist.**

12. **Document monitoring stack** for Supabase self-hosted.

13. **Create migration playbooks** for common failure scenarios.

---

## Summary Scorecard

| Category | Score | Comments |
|----------|-------|----------|
| Completeness | 6/10 | Missing critical compliance and migration details |
| Accuracy | 7/10 | Some technical claims need verification |
| Risk Assessment | 6/10 | Underestimates ETL and RLS complexity |
| Recommendation Strength | 8/10 | Supabase is well-justified for government use |
| Migration Strategy | 5/10 | Timeline optimistic, lacks rollback planning |
| Compliance | 7/10 | Good NORA coverage, missing DigiD/Woo specifics |

**Overall: 7/10** - The plan has solid foundations but needs significant additional work on migration complexity, compliance specifics, and operational readiness before implementation can begin safely.
