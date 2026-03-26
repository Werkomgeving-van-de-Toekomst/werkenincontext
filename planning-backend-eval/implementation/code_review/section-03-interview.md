# Code Review Interview: Section 03

## Date: 2025-03-14

---

## CRITICAL SECURITY: RLS Clearance Function

**Issue:** The `auth.has_clearance()` function grants ALL authenticated users access to 'intern' documents.

**Decision:** AUTO-FIX - Update RLS policy to properly check clearance for 'intern' level.

**Action:** Modify the clearance function to return false for 'intern' unless user has at least 'intern' clearance.

---

## CRITICAL: JWT Default Implementation

**Issue:** `Default` impl for `SupabaseJwtVerifier` panics if env var missing.

**Decision:** AUTO-FIX - Remove the `Default` impl that panics.

**Action:** Remove lines 255-260 from supabase_jwt.rs.

---

## HIGH: Race Condition in Presence Tracker

**Issue:** `update_presence()` has read-modify-write race.

**Decision:** AUTO-FIX - Use `DashMap::entry()` API for atomic updates.

**Action:** Refactor `update_presence()` to use entry API.

---

## HIGH: Realtime Client WebSocket Connection

**Issue:** Realtime client doesn't actually connect to WebSocket - it's a stub.

**Decision:** LET GO - Document as TODO for next section.

**Rationale:** This is expected for section-03. The WebSocket connection will be implemented in section-04 (cutover) when we fully migrate to Supabase real-time. The structure is in place for future implementation.

---

## HIGH: User Migration DuckDB Query

**Issue:** `get_all_duckdb_users()` returns empty vector - not implemented.

**Decision:** LET GO - Document as TODO for production migration.

**Rationale:** This is a placeholder for the actual migration which will be run during the cutover phase (section-04). The structure and types are correct; the actual query will be written when the DuckDB schema is finalized.

---

## MEDIUM: SQL Injection Risk in JSON Building

**Issue:** String concatenation for JSON user metadata.

**Decision:** AUTO-FIX - Use `jsonb_build_object()`.

**Action:** Update migration SQL to use proper JSON building function.

---

## MEDIUM: Hardcoded Instance ID

**Issue:** Uses hardcoded '00000000-0000-0000-0000-000000000000' for instance_id.

**Decision:** AUTO-FIX - Query actual instance_id or make it configurable.

**Action:** Add instance_id to MigrationConfig or query from auth.instances.

---

## LOW: Integration Tests Ignored

**Issue:** All tests marked with `#[ignore]`.

**Decision:** LET GO - Tests are structural scaffolding.

**Rationale:** These tests require actual Supabase instance to run. They serve as documentation of expected behavior. The tests will be enabled and run against a test Supabase instance in section-04.

---

## LOW: Missing Cleanup Task for Presence

**Issue:** `cleanup_stale()` never called automatically.

**Decision:** LET GO - Document for production deployment.

**Rationale:** Cleanup is a production optimization, not a functional requirement. It can be added before production deployment or handled by a separate maintenance service.

---

## AUTO-FIX SUMMARY

The following fixes will be applied:

1. **RLS Clearance Function** - Fix 'intern' clearance logic
2. **JWT Default Impl** - Remove panicking Default impl
3. **Presence Race Condition** - Use DashMap entry API
4. **JSON Building** - Use jsonb_build_object()
5. **Instance ID** - Make configurable or query from DB

---

## ISSUES DEFERRED

The following are documented for future work but not fixed now:

1. WebSocket connection implementation (section-04)
2. DuckDB user query implementation (section-04)
3. Integration tests (enable in section-04)
4. Automatic presence cleanup (production deployment task)
5. Batch size usage (optimization for production)
