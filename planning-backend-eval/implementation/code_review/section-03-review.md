# Code Review: Section 03 - Authentication and Real-time Implementation

## CRITICAL SECURITY ISSUES

### 1. JWT Secret Exposure via Environment Variables (HIGH SEVERITY)
**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/auth/supabase_jwt.rs:143-145`

**Issue:** JWT secret loaded directly from environment without validation. The `Default` implementation (lines 255-260) will panic in production if env var is missing, which could crash the server during startup.

**Recommendation:**
- Remove the `Default` impl that panics
- Validate configuration at startup, not on first use
- Consider using a config struct with proper validation

### 2. RLS Policy Clearance Logic Flaw (HIGH SEVERITY)
**File:** `/Users/marc/Projecten/iou-modern/migrations/postgres/002_rls_policies.sql:2144-2156`

**Issue:** The clearance function grants access to 'intern' documents for ALL authenticated users, bypassing the actual clearance check:

```sql
WHEN 'intern' THEN true -- All authenticated users
```

**Recommendation:**
- Actually check clearance level for 'intern' documents
- Add explicit NULL checks in all RLS policies

### 3. SQL Injection Risk in User Migration (MEDIUM SEVERITY)
**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/migration/user_migration.rs:804`

**Issue:** JSON string concatenation is vulnerable if organization_id contains special characters. Also uses hardcoded instance_id.

**Recommendation:**
- Use `jsonb_build_object()` instead of string concatenation
- Query the actual instance_id from auth.instances table

## RACE CONDITIONS & CONCURRENCY ISSUES

### 4. Presence Tracker Not Thread-Safe (MEDIUM SEVERITY)
**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/realtime/presence.rs:1095-1127`

**Issue:** Classic read-modify-write race condition in `update_presence()`. Between `get()` and `insert()`, another thread could update the same document's presence list.

**Recommendation:**
- Use `DashMap::entry()` API for atomic updates
- Or use `by_document.get_mut()` with in-place modification

### 5. No Cleanup Task Scheduled for Stale Presence
**Issue:** The `cleanup_stale()` method exists but is never called automatically.

**Recommendation:**
- Add a `spawn_cleanup_task()` method that uses `tokio::spawn()`
- Store a JoinHandle to cancel on drop

## DATA MIGRATION ISSUES

### 6. Migration is NOT Transactional (HIGH SEVERITY)
**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/migration/user_migration.rs:630-698`

**Issue:** Failed migrations don't roll back successfully migrated users. No resume capability.

**Recommendation:**
- Implement a migration checkpoint table to track progress
- Add `resume_from_checkpoint()` capability
- Consider wrapping batches in transactions

### 7. DuckDB Query Not Implemented (PLACEHOLDER)
**Issue:** The `get_all_duckdb_users()` method returns an empty vector - core migration logic is incomplete.

**Impact:** The migration is a no-op until implemented.

## MISSING ERROR HANDLING

### 8. Realtime Client Never Actually Connects
**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/realtime/supabase_rt.rs`

**Issue:** The `subscribe()` method creates a channel but NEVER connects to WebSocket or sends subscription messages.

**Impact:** Real-time feature is non-functional.

## RLS POLICY ISSUES

### 9. Inconsistent Organization Isolation
**Issue:** `information_objects` uses subquery while `documents` uses direct comparison. Inconsistent patterns make code harder to maintain.

### 10. Missing RLS Policy for Templates UPDATE/DELETE
**Issue:** Templates lack owner-based policies like information_objects.

## TESTING GAPS

### 11. All Integration Tests are Ignored
**Issue:** Every test has `#[ignore]` - tests not validating anything.

### 12. Test Assertions are Placeholders
**Issue:** Assertions like `assert!(true, "...")` always pass.

## PERFORMANCE CONCERNS

### 13. N+1 Query Potential in Presence Tracker
**Issue:** Clones entire presence list for each call.

## SUMMARY

**Critical Issues (must fix):**
1. JWT verification panics on missing env in production
2. RLS clearance function grants 'intern' access to everyone
3. User migration is incomplete (returns empty users)
4. Realtime client doesn't actually connect
5. No transactional safety in migration

**High Priority Issues:**
6. Race conditions in presence tracker
7. SQL injection risk in JSON concatenation
8. All integration tests are ignored/placeholder

**Verdict:** This implementation is **NOT production-ready**. The auth/realtime features appear functional but have critical security vulnerabilities and incomplete implementations.
