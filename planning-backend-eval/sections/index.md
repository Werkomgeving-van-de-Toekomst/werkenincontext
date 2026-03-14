<!-- PROJECT_CONFIG
runtime: rust-cargo
test_command: cargo test
END_PROJECT_CONFIG -->

<!-- SECTION_MANIFEST
section-01-assessment
section-02-foundation
section-03-auth-realtime
section-04-cutover
section-05-stabilization
section-06-cleanup
END_MANIFEST -->

# Implementation Sections Index

## Overview

This plan outlines the migration from DuckDB to a hybrid Supabase + DuckDB architecture for the IOU-Modern platform. The migration is organized into 6 phases spanning 12 weeks.

## Dependency Graph

| Section | Depends On | Blocks | Parallelizable |
|---------|------------|--------|----------------|
| section-01-assessment | - | section-02 | No |
| section-02-foundation | 01 | section-03 | No |
| section-03-auth-realtime | 02 | section-04 | No |
| section-04-cutover | 03 | section-05 | No |
| section-05-stabilization | 04 | section-06 | No |
| section-06-cleanup | 05 | - | No |

## Execution Order

1. **section-01-assessment** - Establish baselines and document current state
2. **section-02-foundation** - Deploy Supabase and implement dual-write
3. **section-03-auth-realtime** - Implement authentication and real-time features
4. **section-04-cutover** - Migrate primary traffic to Supabase
5. **section-05-stabilization** - Monitor and optimize after cutover
6. **section-06-cleanup** - Remove legacy code and finalize migration

## Section Summaries

### section-01-assessment

**Weeks 1-2 | Phase 0**

Gather performance baselines, document current authentication implementation, validate real-time requirements, characterize analytics workloads, and decide hosting location.

**Key Deliverables:**
- Performance baseline documentation (p50/p95/p99 latencies)
- Current authentication implementation catalog
- Real-time latency measurements with Dutch government network simulation
- Hosting location decision

### section-02-foundation

**Weeks 3-4 | Phase 1**

Deploy self-hosted Supabase alongside DuckDB, create PostgreSQL schema, establish database connections from Rust backend, implement dual-write pattern, and create read toggle for gradual cutover.

**Key Deliverables:**
- Supabase Docker deployment running
- PostgreSQL schema matching DuckDB tables
- Dual-write pattern implemented and tested
- Data consistency validation

### section-03-auth-realtime

**Weeks 5-6 | Phase 2**

Configure Supabase Auth with Row-Level Security (RLS), migrate user data, implement real-time subscriptions, update frontend for real-time updates, and evaluate WebSocket vs Supabase Realtime.

**Key Deliverables:**
- Supabase Auth configured with RLS policies
- User data migrated successfully
- Real-time subscriptions working
- Frontend consuming real-time updates

### section-04-cutover

**Weeks 7-9 | Phase 3**

Migrate primary API traffic to Supabase, implement ETL from Supabase to DuckDB for analytics, configure DuckDB for analytics-only workload, and remove dual-write pattern.

**Key Deliverables:**
- All API endpoints reading from Supabase
- ETL pipeline operational (Supabase → DuckDB)
- DuckDB analytics-only mode configured
- Dual-write pattern removed

### section-05-stabilization

**Weeks 10-11 | Phase 4**

Monitor production performance, optimize slow queries, tune RLS policies, stabilize ETL pipeline, and address production issues.

**Key Deliverables:**
- Performance meeting or exceeding baseline
- ETL latency acceptable
- Monitoring and alerting operational
- No data inconsistencies

### section-06-cleanup

**Week 12 | Phase 5**

Remove legacy code (custom WebSocket, DuckDB transactional queries), optimize ETL pipeline, update documentation, decommission old authentication, and complete team training.

**Key Deliverables:**
- Technical debt reduced
- Documentation complete
- Team trained on new architecture
- Migration fully complete

## Cross-Cutting Concerns

### Compliance

All sections must ensure compliance with:
- **GDPR/AVG** - Right to deletion, data portability, audit trails
- **Archiefwet** - Retention policies, automated deletion
- **Bijhoudingsplicht** - Audit trail continuity during migration
- **Woo** - Publication workflow for public documents
- **BVO** - Multi-factor auth, SIEM integration

### Testing

Each section requires corresponding tests as defined in `claude-plan-tdd.md`:
- Schema equivalence tests
- Data migration tests
- RLS policy tests
- Performance regression tests
- Real-time subscription tests

### Rollback

Each section must support rollback:
- Phase 1-2: Full rollback via read toggle
- Phase 3+: Complex rollback via data reconciliation scripts
