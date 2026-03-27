# Interview Transcript: Backend Evaluation for IOU-Modern

**Date:** 2026-03-11
**Goal:** Clarify requirements for DuckDB vs Convex vs Supabase evaluation

---

## Q1: How important is real-time collaboration?

**Answer:** Must-have

**Implication:** Real-time collaboration is a critical requirement for the IOU-Modern platform. This significantly impacts the evaluation:
- DuckDB: Does not have built-in real-time (would require custom WebSocket implementation)
- Convex: Excellent real-time with sub-50ms latency
- Supabase: Has real-time via PostgreSQL CDC (100-200ms p99)

**Notes:** The current codebase has a custom WebSocket implementation for status updates. A database with built-in real-time could simplify this.

---

## Q2: What is the timeline for making this backend decision?

**Answer:** Urgent (< 3 months)

**Implication:** Quick evaluation and decision needed. This means:
- Need clear, actionable recommendation
- Migration plan should be high-level (detailed planning can come later)
- Proof-of-concept testing may be needed before final decision

---

## Q3: What's your preferred approach for this evaluation?

**Answer:** Evaluation only

**Implication:** This plan will produce:
- Detailed comparison and recommendation
- High-level migration considerations
- Risk assessment
- No detailed implementation steps

Implementation planning would be a separate follow-up project.

---

## Q4: How concerned are you about vendor lock-in?

**Answer:** Avoid lock-in

**Implication:** Strong preference for open-source solutions with minimal vendor dependencies:
- **Convex**: FSL 1.1 license is concerning (not FOSS for 2 years)
- **Supabase**: MIT license, fully open-source, self-hostable
- **DuckDB**: MIT license, embedded (no vendor lock-in)

**Additional Consideration:** Dutch government projects often have strict requirements about software licensing and data sovereignty. Self-hosting capability is important.

---

## Summary of Requirements

| Requirement | Value | Impact |
|-------------|-------|--------|
| Real-time collaboration | Must-have | Eliminates DuckDB-only approach |
| Timeline | < 3 months | Need clear recommendation quickly |
| Scope | Evaluation only | No implementation in this plan |
| Vendor lock-in | Avoid | Favors Supabase over Convex |

---

## Preliminary Conclusion

Based on these requirements, **Supabase** emerges as the leading candidate:
- ✓ Real-time subscriptions (PostgreSQL CDC)
- ✓ MIT license, fully open-source
- ✓ Self-hostable (Docker)
- ✓ SOC 2/HIPAA compliant
- ✗ Latency higher than Convex (100-200ms vs <50ms)

**Hybrid approach consideration:** Keep DuckDB for analytics workloads while using Supabase for transactional data and real-time features.
