# Specification: Backend Evaluation for IOU-Modern

## Executive Summary

This specification defines a comprehensive evaluation of backend database solutions for the IOU-Modern platform, comparing the current DuckDB implementation with two alternatives: Convex and Supabase.

**Recommendation Preview:** Supabase with DuckDB retained for analytics (hybrid approach).

---

## Project Context

### IOU-Modern Platform

IOU-Modern (Informatie Ondersteunde Werkomgeving) is an information management platform for Dutch government organizations. It helps government agencies:

- Manage information domains (Zaak, Project, Beleid, Expertise)
- Create Woo-compliant documents
- Maintain context-aware relationships (GraphRAG)
- Automate document generation via AI agents

### Current Architecture

| Component | Technology |
|-----------|-------------|
| Backend API | Axum (Rust) |
| Database | DuckDB (embedded analytical) |
| Frontend | Dioxus 0.7 (WebAssembly) |
| AI Agents | Rust multi-agent pipeline |
| Storage | S3/MinIO |
| Real-time | Custom WebSocket implementation |

### Current DuckDB Usage

**Core Tables:**
- `information_domains` - Organizational context
- `information_objects` - Main document/content storage
- `documents` - Document metadata with workflow states
- `templates` - Document templates
- `audit_trail` - Full audit trail

**Current Limitations:**
- Full-text search uses basic `ILIKE` (not DuckDB's FTS extension)
- Single-writer architecture limits concurrent writes
- No built-in real-time synchronization
- Vector embeddings stored but no similarity search implemented

---

## Evaluation Requirements

### Functional Requirements

#### FR1: Real-Time Collaboration (MUST-HAVE)
- Multiple users collaborate on documents simultaneously
- Changes propagate without manual refresh
- Presence indicators (who is viewing/editing)
- Conflict resolution for concurrent edits

#### FR2: User Authentication & Authorization
- Multiple authentication providers
- Role-based access control (RBAC)
- Fine-grained permissions per information domain
- Support for Dutch government identity systems (DigiD, eHerkenning) preferred

#### FR3: Compliance Features
- GDPR compliance (right to be forgotten, data portability)
- Woo compliance (Wet open overheid - transparency)
- Archiefwet compliant (retention policies, automatic deletion)
- Audit logging for all data access
- PII detection and redaction capabilities

#### FR4: Data Model
- Support for complex relational data
- Full-text search capabilities
- JSON/document storage flexibility
- Graph relationship queries (for GraphRAG)
- Large dataset analytics (100K+ records)

### Non-Functional Requirements

#### NFR1: Self-Hosting (HIGH)
- Must be deployable on-premises or on own infrastructure
- No dependency on proprietary cloud services
- Data sovereignty (data stays within EU/Netherlands)

#### NFR2: Rust Stack Compatibility (HIGH)
- Can integrate with existing Rust/Axum backend
- Or can replace it without complete rewrite
- TypeScript/JavaScript SDKs for frontend integration

#### NFR3: Vendor Lock-In (HIGH - AVOID)
- Preference for open-source licenses
- Avoid proprietary dependencies
- Data portability and export capabilities

#### NFR4: Timeline (URGENT)
- Decision needed within 3 months
- Evaluation phase, not implementation

---

## Candidates

### 1. DuckDB (Current Solution)

**Pros:**
- Already implemented
- Single-file deployment
- Excellent for analytics
- Full SQL support
- Works great with Rust
- MIT license

**Cons:**
- No built-in real-time features
- No built-in authentication
- Single-writer limitation
- Not designed for multi-user concurrent writes

**Verdict:** Retain for analytics workloads in hybrid approach

---

### 2. Convex

**Pros:**
- Reactive queries (automatic real-time)
- Sub-50ms latency at 5,000 concurrent connections
- Built-in auth integration
- Zero-config scaling
- Self-hostable since 2025

**Cons:**
- FSL 1.1-Apache 2.0 license (not FOSS for 2 years)
- Requires TypeScript backend logic
- Vendor lock-in concerns
- Non-standard query model

**Verdict:** Not recommended due to FSL license and lock-in concerns

---

### 3. Supabase

**Pros:**
- Self-hostable (Docker/Kubernetes)
- PostgreSQL foundation (proven, reliable)
- Built-in real-time subscriptions
- Built-in auth (multiple providers)
- Row-level security (RLS)
- Edge functions
- MIT license (fully open-source)
- SOC 2 Type 2 and HIPAA compliant

**Cons:**
- More infrastructure to manage
- Postgres not optimized for analytics (can use extensions)
- May require migration from DuckDB schema

**Verdict:** Recommended as primary solution

---

## Research Findings Summary

### Performance Comparison

| Metric | DuckDB | Convex | Supabase |
|--------|--------|--------|----------|
| Real-time latency | N/A | <50ms | 100-200ms p99 |
| Concurrent connections | Limited (single writer) | 5,000+ | PostgreSQL limits |
| Analytics performance | Excellent | Good | Fair |
| Transactional performance | Fair | Good | Excellent |

### Compliance Matrix

| Requirement | Supabase | Convex | DuckDB |
|-------------|----------|--------|--------|
| GDPR compliant | ✓ (SOC 2) | ✓ (self-hosted) | ✓ (embedded) |
| Woo compliant | ✓ | ✓ | ✓ |
| Archiefwet support | Standard tools | Export ZIP | Direct file |
| NORA alignment | Standard SQL | FSL license | Fully open |
| Data portability | PostgreSQL | ZIP export | File-based |

---

## User Interview Insights

### Critical Decisions

1. **Real-time collaboration = Must-have**
   - Eliminates DuckDB-only approach
   - Requires real-time database or significant WebSocket investment

2. **Timeline = Urgent (< 3 months)**
   - Need clear recommendation
   - Proof-of-concept may be needed

3. **Scope = Evaluation only**
   - This plan produces recommendation + high-level migration considerations
   - Implementation planning is separate follow-up

4. **Vendor lock-in = Avoid**
   - Strong preference for open-source
   - Self-hosting capability important
   - Makes Convex less attractive

---

## Recommended Approach: Hybrid Solution

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     IOU-Modern Platform                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────┐         ┌──────────────────┐         │
│  │   Supabase       │         │    DuckDB        │         │
│  │   (Primary DB)   │         │   (Analytics)    │         │
│  │                  │         │                  │         │
│  │  • Users         │         │  • Analytics     │         │
│  │  • Documents     │◄───────►│  • Reporting     │         │
│  │  • Real-time     │  Sync   │  • BI exports    │         │
│  │  • Auth          │         │  • Data science  │         │
│  └──────────────────┘         └──────────────────┘         │
│           │                           ▲                     │
│           │                           │                     │
│           ▼                           │                     │
│  ┌──────────────────────────────────────────────────┐      │
│  │           Rust/Axum Backend                      │      │
│  │  • Syncs data between databases                  │      │
│  │  • Business logic                                │      │
│  │  • Orchestrator integration                      │      │
│  └──────────────────────────────────────────────────┘      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Benefits

1. **Real-time**: Supabase provides built-in real-time subscriptions
2. **Compliance**: PostgreSQL standard tools, SOC 2 certified
3. **Analytics**: DuckDB retained for powerful analytical queries
4. **Flexibility**: Can migrate incrementally
5. **Open-source**: Both use permissive licenses

### Migration Strategy

**Phase 1:** Set up Supabase alongside DuckDB
- Deploy self-hosted Supabase (Docker)
- Create schema for documents, users, domains
- Implement sync mechanism

**Phase 2:** Migrate transactional data to Supabase
- Users and auth
- Documents and workflows
- Real-time subscriptions

**Phase 3:** Optimize DuckDB for analytics
- ETL pipeline from Supabase to DuckDB
- Materialized views for reporting
- Data science notebook integration

---

## Open Questions

1. Should we implement a real-time sync between databases or use batch ETL?
2. What is the data volume for real-time vs analytical queries?
3. Are there budget constraints for hosting additional infrastructure?
4. Which authentication providers need priority (DigiD, eHerkenning, others)?

---

## Deliverables

1. ✓ Detailed comparison matrix
2. ✓ Recommendation with rationale
3. ✓ High-level migration considerations
4. ✓ Risk assessment for each option
5. → Next: Proof-of-concept plan (if recommended)
