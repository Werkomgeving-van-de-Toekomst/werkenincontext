# Architecture Decision Record: Multi-Project Architecture Alignment

> **Template Origin**: Official | **ArcKit Version**: 4.3.1

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-ALIGN-ADR-001-v1.0 |
| **Document Type** | Architecture Decision Record |
| **Project** | IOU-Modern (Cross-Project) |
| **Classification** | OFFICIAL |
| **Status** | PROPOSED |
| **Version** | 1.0 |
| **Created Date** | 2026-04-20 |
| **Decision Date** | PENDING |
| **Owner** | Enterprise Architect |

---

## Decision Metadata

| Field | Value |
|-------|-------|
| **Status** | Proposed |
| **Context** | Architectural inconsistencies between Projects 002 and 003 |
| **Decision Type** | Strategic Alignment |
| **Impact Level** | High |
| **Related Projects** | 002 (Metadata Registry), 003 (Context-Aware Data) |
| **Related Decisions** | ARC-002-ADR-001 through 005, ARC-003-ADR-001 through 003 |

---

## Context

### Background

Projects 002 (Metadata Registry) and 003 (Context-Aware Data) are closely related components of IOU-Modern. Both services deal with metadata management and must integrate closely. However, independent technology choices have created architectural inconsistencies.

### Problem Statement

**Current State:**
- Project 002: Rust + ArangoDB + Dioxus
- Project 003: Java/PostgreSQL + Python + React

**Implications:**
1. Two programming languages to maintain
2. Two database technologies to operate
3. Two frontend frameworks
4. Complex integration patterns
5. Duplicate skill requirements
6. Higher operational overhead

### Drivers

**Technical Drivers:**
- Integration complexity between services
- Data consistency across different databases
- Shared component development

**Business Drivers:**
- Operational cost containment
- Team efficiency
- Time-to-market

**Constraint Drivers:**
- Existing investment in Project 002 (partially implemented)
- Project 003 in design phase (not yet implemented)

---

## Decision

### Decision Statement

**Option A: Technology Stack Convergence is RECOMMENDED**

Both projects should use the same technology stack:
- **Backend**: Rust
- **Database**: ArangoDB
- **Frontend**: Dioxus
- **AI/ML**: Rust SDK + Claude API

### Implementation Approach

**Phase 1: Foundation (Months 1-2)**
- Establish shared Rust workspace
- Create unified ArangoDB schema
- Define common crate structure

**Phase 2: Migrate Project 003 Backend (Months 3-5)**
- Rewrite Context Capture Service in Rust
- Implement REST/GraphQL APIs using actix-web
- Create repositories for ArangoDB

**Phase 3: Migrate Project 003 Database (Months 4-6)**
- Convert PostgreSQL schema to ArangoDB collections
- Migrate context data model to graph structure
- Implement pgvector-like functionality using ArangoDB

**Phase 4: Migrate Project 003 Frontend (Months 6-7)**
- Rewrite Context Portal in Dioxus
- Reuse components from Metadata admin UI

**Phase 5: Cleanup (Month 8)**
- Remove legacy Java/Python code
- Update documentation
- Deploy unified platform

---

## Alternatives Considered

### Option B: Service-Oriented Separation

**Description**: Accept technology differences and design for service boundaries.

**Pros:**
- No migration required
- Teams work independently
- Technology-appropriate choices

**Cons:**
- Higher operational complexity
- Consistency harder to guarantee
- Two skill sets to maintain

**Rejected Because**: Long-term operational costs outweigh short-term savings.

---

### Option C: Database Unification Only

**Description**: Both projects use PostgreSQL; keep different languages.

**Pros:**
- Shared database enables direct queries
- Easier data consistency

**Cons:**
- Loss of graph capabilities for Project 002
- Still two languages to maintain

**Rejected Because**: ArangoDB's graph capabilities are core to GGHH V2 implementation.

---

### Option D: Maintain Status Quo

**Description**: Accept inconsistencies and build integration layer.

**Pros:**
- Zero migration cost
- Immediate implementation

**Cons:**
- Permanent technical debt
- Ongoing inefficiency
- Two technology stacks forever

**Rejected Because**: Violates principle of architectural coherence and operational simplicity.

---

## Decision Drivers

| Driver | Weight | Option A | Option B | Option C | Option D |
|--------|--------|----------|----------|----------|----------|
| Operational Simplicity | HIGH | 10 | 4 | 6 | 2 |
| Development Efficiency | HIGH | 9 | 5 | 6 | 3 |
| Integration Complexity | HIGH | 9 | 5 | 7 | 2 |
| Migration Cost | MEDIUM | 3 | 9 | 6 | 10 |
| Team Skills | MEDIUM | 6 | 7 | 5 | 8 |
| Time to Market | MEDIUM | 5 | 9 | 7 | 10 |
| **Weighted Score** | | **42** | **39** | **37** | **35** |

---

## Consequences

### Positive Consequences

1. **Single Codebase**: Shared types, utilities, and patterns
2. **Unified Build**: One Cargo workspace, one CI/CD pipeline
3. **Shared Components**: Reusable Dioxus UI components
4. **Simpler Operations**: One language runtime, one database
5. **Team Mobility**: Developers can work across projects
6. **Consistent Patterns**: Same idioms throughout

### Negative Consequences

1. **Migration Effort**: 6-9 months of development work
2. **Migration Cost**: €1.2-1.8M additional investment
3. **Delay**: Project 003 implementation delayed
4. **Risk**: Migration may introduce bugs
5. **Learning**: Team must learn Rust (if not already skilled)

### Mitigation Strategies

1. **Phased Migration**: Deliver value incrementally
2. **Parallel Operation**: Keep existing systems during migration
3. **Investment**: Budget for training and external support
4. **Testing**: Comprehensive test coverage before cutover
5. **Rollback**: Maintain ability to revert if needed

---

## Implementation Plan

### Dependencies

This decision depends on:
- ARC-002-ADR-001: Rust language selection (already accepted)
- ARC-002-ADR-002: ArangoDB selection (already accepted)
- ARC-003-ADR-001: PostgreSQL selection (would be superseded)

### Timeline

```
Month 1-2: Foundation
├── Establish shared workspace
├── Define unified schema
└── Create common crates

Month 3-5: Backend Migration
├── Context Capture Service (Rust)
├── Context Registry (ArangoDB)
├── Context Inference (Rust SDK)
└── Context Quality (Rust)

Month 4-6: Database Migration
├── Schema conversion
├── Data migration scripts
└── Validation

Month 6-7: Frontend Migration
├── Context Portal (Dioxus)
└── Component library

Month 8: Deployment
├── Cutover to unified stack
├── Documentation
└── Handover
```

### Resource Requirements

| Role | FTE | Duration |
|------|-----|----------|
| Rust Backend Developer | 2 | 8 months |
| Database Developer | 1 | 4 months |
| Frontend Developer | 1 | 4 months |
| DevOps Engineer | 0.5 | 8 months |
| QA Engineer | 0.5 | 6 months |
| **Total** | **5** | **~32 person-months** |

---

## Validation

### Success Criteria

1. ✅ Single Rust workspace compiles
2. ✅ All services deploy from one artifact
3. ✅ ArangoDB stores both metadata and context
4. ✅ API response times < 100ms (p95)
5. ✅ Zero regression in functionality
6. ✅ 80% of code coverage maintained

### Monitoring

- Migration progress tracked weekly
- Performance benchmarks at each phase
- Stakeholder reviews at phase gates

---

## Related Decisions

### Supersedes
- ARC-003-ADR-001: PostgreSQL as Primary Database (would be replaced by ArangoDB)

### Enables
- Shared component library across projects
- Unified deployment pipeline
- Cross-project code reuse

---

## Approval

| Role | Name | Signature | Date |
|------|------|-----------|------|
| **Decision Proposer** | Enterprise Architect | | PENDING |
| **Technical Reviewer** | Lead Architect | | PENDING |
| **Business Reviewer** | Product Owner | | PENDING |
| **Final Approver** | CIO | | PENDING |

---

**Generated by**: ArcKit
**Generated on**: 2026-04-20
**Project**: IOU-Modern Cross-Project Alignment
