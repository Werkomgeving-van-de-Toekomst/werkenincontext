# Integration Notes: Opus Review Feedback

## Reviewed: 2026-03-11

---

## Suggestions to Integrate

### High Priority

#### 1. Audit Trail Continuity (Section: Migration Strategy)
**Feedback:** Missing "Bijhoudingsplicht" - audit trail preservation during migration not addressed.
**Action:** Add section on audit trail preservation, correlating Supabase WAL audit with existing DuckDB audit records.

#### 2. Selectielijst Implementation (Section: Compliance Assessment)
**Feedback:** Retention schedules lack implementation detail.
**Action:** Add pg_cron example for automated retention enforcement.

#### 3. Full-Text Search Migration (Section: Current State Analysis)
**Feedback:** No comparison between DuckDB FTS and PostgreSQL tsvector, especially for Dutch language.
**Action:** Add subsection on search migration strategy with Dutch text search configuration.

#### 4. Vector Search Architecture (Section: Hybrid Architecture)
**Feedback:** Unclear where vectors live - Supabase (pgvector) or DuckDB.
**Action:** Clarify vectors stay in DuckDB for analytics; pgvector optional for real-time similarity.

#### 5. SQL Compatibility Matrix (Section: Migration Strategy)
**Feedback:** PostgreSQL differences (uuid() vs gen_random_uuid()) will break migration.
**Action:** Add SQL compatibility migration guide.

#### 6. DigiD/EHerkenning Requirements (Section: Compliance Assessment)
**Feedback:** Critical for Dutch government - not standard OAuth providers.
**Action:** Add explicit government authentication requirements section.

#### 7. Hosting Location Decision (Section: Compliance Assessment)
**Feedback:** Data sovereignty considerations missing.
**Action:** Add hosting location criteria (Rijkscloud, Gemeente Shared Services, EU provider, on-prem).

#### 8. Pre-Migration Assessment (Section: Migration Strategy)
**Feedback:** No performance baseline or current auth documentation.
**Action:** Add "Phase 0: Assessment" (2 weeks) before Foundation.

#### 9. Rollback Procedures (Section: Risk Assessment)
**Feedback:** No rollback strategy documented.
**Action:** Add rollback procedures with data reconciliation scripts.

#### 10. Monitoring & Backup (New Section)
**Feedback:** Operational concerns missing.
**Action:** Add operational requirements section.

---

### Medium Priority

#### 11. RLS Policy Complexity Warning (Section: Risk Assessment)
**Action:** Add warning about complex RLS policies causing performance issues.

#### 12. ETL Consistency Strategy (Section: Migration Strategy)
**Action:** Detail transactional outbox pattern or CDC architecture.

#### 13. Timeline Extension (Section: Migration Strategy)
**Action:** Extend 8 weeks to 12 weeks with buffer phases.

#### 14. Woo Publication Workflow (Section: Compliance Assessment)
**Action:** Add Woo publication integration requirements.

---

### Low Priority

#### 15. BVO Compliance (Section: Compliance Assessment)
**Action:** Add Basisveiligheid Overheid checklist.

#### 16. WebSocket Deprecation Consideration (Section: Migration Strategy)
**Action:** Note custom WebSocket may provide more functionality than Supabase Realtime.

---

## Suggestions NOT Integrating (with rationale)

#### Convex Self-Hosting Caveats
**Feedback:** Claims are misleading; self-hosting is beta/early access.
**Rationale:** Convex is already "Not recommended" - additional caveats won't change recommendation. The FSL license concern is already documented.

#### Supabase Real-time Latency Claims
**Feedback:** 100-200ms is optimistic for self-hosted deployments.
**Rationale:** Will add "performance testing required" note rather than changing numbers. The comparison table is for estimation purposes.

#### Migration Effort Underestimation
**Feedback:** Every database call in db.rs (1500+ lines) needs reconsideration.
**Rationale:** This is a valid concern but better addressed in detailed implementation planning. The current plan is high-level; adding line-count estimates would be premature.

#### Connection Pooling Architecture Change
**Feedback:** Fundamental change from Arc<Mutex<Connection>> to PgPool.
**Rationale:** Important but this is standard when migrating from embedded to client-server database. Noted implicitly in migration tasks.

#### Analytics Performance Warning
**Feedback:** PostgreSQL may disappoint for analytics vs DuckDB.
**Rationale:** The hybrid architecture specifically keeps DuckDB for analytics - this is already the recommended solution.

---

## Summary

**Integrating:** 16 items across compliance, migration, and operational concerns
**Not integrating:** 5 items (already addressed, premature detail, or better in implementation phase)
**Overall impact:** Moderate - plan needs additional sections but core recommendation remains valid
