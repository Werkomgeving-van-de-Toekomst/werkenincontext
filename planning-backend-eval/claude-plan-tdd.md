# TDD Plan: Backend Database Selection for IOU-Modern

This document defines the tests to write before implementing each phase of the migration plan. Tests follow Rust's built-in testing framework with `cargo test`.

---

## Introduction

### Validation Tests (Pre-Implementation)

**Goal:** Verify assumptions before starting migration

- **Test:** Performance baseline recording - capture p50/p95/p99 query latencies
- **Test:** Concurrent user capacity measurement - document current limits
- **Test:** WebSocket connection count baseline - record active connections
- **Test:** Database size and growth rate - establish metrics
- **Test:** Authentication documentation verification - catalog existing JWT middleware

---

## Current State Analysis

### Schema Equivalence Tests

**Goal:** Ensure PostgreSQL schema matches DuckDB structure

- **Test:** information_domains table - verify column types match DuckDB
- **Test:** information_objects table - verify column types match DuckDB
- **Test:** documents table - verify column types match DuckDB
- **Test:** templates table - verify column types match DuckDB
- **Test:** audit_trail table - verify column types match DuckDB
- **Test:** View equivalence - v_searchable_objects produces same results
- **Test:** View equivalence - v_compliance_overview produces same results
- **Test:** View equivalence - v_domain_statistics produces same results
- **Test:** View equivalence - v_entity_network produces same results

### SQL Compatibility Tests

**Goal:** Identify and document SQL differences

- **Test:** UUID generation - verify gen_random_uuid() equivalent to uuid()
- **Test:** Array type compatibility - VARCHAR[] vs TEXT[] behavior
- **Test:** Full-text search semantics - compare ILIKE vs tsvector results
- **Test:** JSON function compatibility - -> operator behavior

---

## Migration Strategy

### Phase 0: Assessment Tests

**Goal:** Establish baselines for comparison

- **Test:** Performance baseline suite - run and record all query timings
- **Test:** Authentication flow documentation - verify existing auth implementation
- **Test:** Real-time latency measurement - test with Dutch government network simulation
- **Test:** Analytics query profiling - record typical analytical query patterns

### Phase 1: Foundation Tests

**Goal:** Verify Supabase setup and dual-write consistency

**Supabase Setup:**
- **Test:** Docker Compose deployment - verify all services start
- **Test:** Database connection - verify Rust backend can connect to Supabase
- **Test:** Schema migration - verify all tables created correctly

**Dual-Write Pattern:**
- **Test:** information_domains dual-write - verify writes to both databases succeed
- **Test:** information_objects dual-write - verify writes to both databases succeed
- **Test:** documents dual-write - verify writes to both databases succeed
- **Test:** Partial failure handling - verify rollback if one write fails
- **Test:** Data consistency comparison - verify identical data in both databases
- **Test:** Read toggle functionality - verify switching between databases works
- **Test:** Performance comparison - verify no degradation with dual-write

### Phase 2: Auth & Real-time Tests

**Goal:** Verify authentication and real-time features

**Authentication:**
- **Test:** Supabase Auth configuration - verify JWT issuance
- **Test:** User data migration - verify all users migrated correctly
- **Test:** Password hash compatibility - verify existing passwords work
- **Test:** Session token migration - verify existing sessions remain valid
- **Test:** RLS policy - organization isolation works correctly
- **Test:** RLS policy - user-level access within organization
- **Test:** RLS policy - classification-based filtering
- **Test:** RLS policy - Woo-publication status filtering
- **Test:** RLS policy performance - verify <500ms p95 for policy checks

**Real-time:**
- **Test:** Real-time subscription creation - verify client can subscribe
- **Test:** Real-time document updates - verify updates propagate to subscribers
- **Test:** Real-time presence indicators - verify user presence works
- **Test:** Real-time conflict resolution - verify concurrent edit handling
- **Test:** Real-time latency - verify <200ms for updates
- **Test:** Frontend integration - verify Dioxus can consume real-time updates
- **Test:** Custom WebSocket vs Supabase Realtime - compare functionality

### Phase 3: Cutover Tests

**Goal:** Verify complete migration to Supabase

**Read Migration:**
- **Test:** API endpoint reads from Supabase - verify all endpoints work
- **Test:** Data consistency final check - verify no data loss
- **Test:** Performance meets baseline - verify no degradation

**ETL Pipeline:**
- **Test:** ETL Supabase to DuckDB - verify data transfer completes
- **Test:** ETL idempotency - verify running twice doesn't duplicate
- **Test:** ETL error handling - verify failures are logged and retryable
- **Test:** ETL latency - verify completes within configured window
- **Test:** DuckDB analytics queries - verify still work with ETL data
- **Test:** Transactional outbox pattern - verify events processed in order

**Cleanup:**
- **Test:** Dual-write removal - verify single-write to Supabase works
- **Test:** DuckDB transactional queries removed - verify analytics-only access

### Phase 4: Stabilization Tests

**Goal:** Verify production readiness

**Performance:**
- **Test:** Query performance regression - compare to Phase 0 baseline
- **Test:** RLS policy optimization - verify no policy regressions
- **Test:** ETL pipeline stability - verify runs consistently over time
- **Test:** Concurrent user load - verify supports target concurrency

**Monitoring:**
- **Test:** PostgreSQL query metrics export - verify pg_stat_statements accessible
- **Test:** Replication lag monitoring - verify lag metrics exported
- **Test:** Replication slot WAL size - verify alert triggers at threshold
- **Test:** Real-time subscription count - verify metrics exported
- **Test:** Database connection pool - verify metrics exported
- **Test:** Disk space monitoring - verify alerts trigger

### Phase 5: Cleanup Tests

**Goal:** Verify legacy code removal

- **Test:** Custom WebSocket removal - verify no references remain
- **Test:** DuckDB transactional query removal - verify only analytics queries remain
- **Test:** Documentation completeness - verify all docs updated
- **Test:** Team training completion - verify knowledge transfer

---

## Rollback Procedures Tests

**Goal:** Verify rollback capability

- **Test:** Rollback trigger detection - verify triggers fire correctly
- **Test:** Read toggle rollback - verify switch back to DuckDB works
- **Test:** Data reconciliation script - identifies Supabase-only records
- **Test:** Data reconciliation script - writes missing records to DuckDB
- **Test:** Post-rollback data consistency - verify no data loss

---

## Compliance Tests

### GDPR/AVG

- **Test:** Right to be forgotten - verify user data can be deleted
- **Test:** Data portability - verify user data can be exported
- **Test:** Audit trail continuity - verify migration events logged

### Archiefwet

- **Test:** Retention policy enforcement - verify pg_cron schedule works
- **Test:** Archive deletion - verify documents past destruction date deleted
- **Test:** Long-term storage - verify data export works after 10 years

### Bijhoudingsplicht

- **Test:** Audit trail preservation - verify DuckDB logs correlate with Supabase WAL
- **Test:** Migration audit events - verify all migration changes logged

### Woo

- **Test:** Publication status enforcement - verify RLS filters Woo documents
- **Test:** Woo-index API integration - verify publication triggers work

---

## Operational Tests

### Backup and Recovery

- **Test:** Base backup creation - verify daily backup completes
- **Test:** WAL archive - verify continuous archiving works
- **Test:** Logical export - verify weekly export completes
- **Test:** Backup encryption - verify backups encrypted at rest
- **Test:** Recovery procedure - verify restore from backup works
- **Test:** Recovery time objective - verify RTO < 4 hours
- **Test:** Recovery point objective - verify RPO < 15 minutes

### Full-Text Search Migration

- **Test:** Dutch language configuration - verify 'dutch' text search works
- **Test:** Search relevance comparison - compare ILIKE vs tsvector results
- **Test:** Search performance - verify tsvector performs better than ILIKE
- **Test:** Search index migration - verify existing documents indexed

---

## Risk Assessment Tests

### High-Risk Items

**ETL Consistency:**
- **Test:** Dual-write data consistency - verify no drift during Phase 1
- **Test:** Transactional outbox ordering - verify events processed in order
- **Test:** CDC vs batch ETL comparison - measure latency difference

**RLS Policy Complexity:**
- **Test:** Complex RLS policy performance - verify p95 < 500ms
- **Test:** Multi-organization isolation - verify no cross-org data leakage
- **Test:** Classification-based filtering - verify confidential data protected

**Real-time Performance:**
- **Test:** Dutch government network simulation - verify latency <200ms
- **Test:** High-load real-time - verify 10+ concurrent users work

---

## Success Criteria Tests

The migration is successful when ALL of these tests pass:

1. **Test:** Real-time collaboration latency - verifies <200ms
2. **Test:** Supabase authentication - verifies RLS enforced
3. **Test:** DuckDB analytics - verifies analytics-only workload
4. **Test:** Data loss check - verifies zero data loss during migration
5. **Test:** Performance baseline comparison - verifies meets or exceeds baseline
6. **Test:** GDPR compliance - verifies right to deletion works
7. **Test:** Woo compliance - verifies publication workflow works
8. **Test:** Archiefwet compliance - verifies retention policies enforced
9. **Test:** Team readiness - verifies developers can work with new stack

---

## Test Execution Commands

```bash
# Run all tests
cargo test

# Run migration-specific tests
cargo test --test migration

# Run performance benchmarks
cargo test --test benches -- --nocapture

# Run with coverage (requires tarpaulin)
cargo tarpaulin --out Html

# Run specific test module
cargo test migration::schema_tests
```

---

## CI/CD Integration

**Required CI Tests:**
- All migration tests must pass before merge
- Performance regression tests run on nightly builds
- RLS policy tests run on every commit (security critical)
- Data consistency tests run with test database
