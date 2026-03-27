# Code Review Interview: Section-02-Foundation

## Date
2026-03-13

## Reviewers
- AI Code Reviewer (Claude Opus 4.6)
- User (decisions on critical issues)

---

## Issues Discussed and Decisions

### 1. Domain Type Case Mismatch (Issue #3 - Critical)

**Finding:** Rust code converts domain types to lowercase (`zaak`) but PostgreSQL CHECK constraint expects titlecase (`Zaak`).

**Decision:** Fix in Rust code - remove `.to_lowercase()` from `domain_type_to_string()` function.

**Action Taken:** Updated `crates/iou-api/src/domain_dual_write.rs` to preserve titlecase matching PostgreSQL schema.

---

### 2. ID Mismatch Handling (Issue #5 - Critical)

**Finding:** When IDs mismatch between databases, only a warning is logged instead of treating it as an error.

**Decision:** Yes, fail on mismatch - return `DualWriteResult::Failed` when IDs don't match.

**Action Taken:** Updated `crates/iou-api/src/dual_write.rs` to return error on ID mismatch.

---

### 3. Auto-Fixed Issues (Low-Risk Improvements)

The following issues were automatically fixed:

#### a. Connection Pool Size Config (Issue #2)
**Fix:** Made pool size configurable via `SUPABASE_MAX_CONNECTIONS` env var with smart default (cpu_count * 2, clamped 5-50).

#### b. Error Context in Supabase Connection (Issue #6)
**Fix:** Added context to connection errors showing database hostname.

#### c. Test Database Paths (Issue #7)
**Fix:** Use `CARGO_TARGET_TMPDIR` or system temp directory with UUID suffix for portable test paths.

#### d. Health Check Retry Logic (Issue #10)
**Fix:** Added exponential backoff retry (3 attempts) to `health_check()` method.

---

## Issues Deferred or Not Addressed

### Hardcoded Credentials in Docker Compose (Issue #1)
**Status:** Documented in README - requires runtime environment variables to be set.
**Rationale:** Default values allow for local development; production deployments should use secrets management.

### Missing Transaction Support (Issue #4)
**Status:** Deferred to future section.
**Rationale:** Full distributed transaction support is complex; partial success tracking is sufficient for initial migration phase.

### Migration Function No-Op (Issue #9)
**Status:** Documented as placeholder.
**Rationale:** SQLx CLI or external migration tools recommended for production.

### Module Structure/Circular Dependencies (Issue #8)
**Status:** Accepted as current architecture.
**Rationale:** The `search_types.rs` module cleanly resolves the circular dependency without major restructuring.

---

## Test Updates

Updated `test_domain_type_to_string()` to expect titlecase values (`Zaak`, not `zaak`) to match the fix.

---

## Compilation Status

Library compiles successfully. Binary has pre-existing compilation errors unrelated to this section's changes.
