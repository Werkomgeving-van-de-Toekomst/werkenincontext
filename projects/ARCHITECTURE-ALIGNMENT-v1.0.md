# Architecture Alignment Plan: Projects 002 & 003

**Document ID**: ARC-ALIGN-001-v1.0
**Created**: 2026-04-20
**Status**: DRAFT
**Projects**: Metadata Registry (002) + Context-Aware Data (003)

## Executive Summary

This document identifies architectural inconsistencies between the Metadata Registry Service (Project 002) and Context-Aware Data Architecture (Project 003) and provides a path to alignment.

## Critical Inconsistencies Found

### 1. Technology Stack Divergence

| Component | Project 002 (Metadata) | Project 003 (Context) | Impact |
|-----------|----------------------|----------------------|--------|
| **Backend Language** | Rust | Java 21 + Spring Boot | HIGH |
| **Database** | ArangoDB (graph) | PostgreSQL 15 + pgvector | HIGH |
| **Frontend** | Dioxus (WebAssembly) | React 18 | MEDIUM |
| **AI/ML Services** | N/A | Python + FastAPI | LOW |
| **API Protocol** | REST v2 + GraphQL | REST + GraphQL | LOW |

### 2. Architecture Documentation Gaps

| Document Type | Project 002 | Project 003 |
|---------------|-------------|-------------|
| Architecture Principles | ❌ Missing | ✅ ARC-003-PRIN (17 principles) |
| Stakeholder Analysis | ✅ ARC-002-STKE | ✅ ARC-003-STKE |
| Detailed Design (DLD) | ✅ ARC-002-DLD | ❌ Missing |
| Data Model | Integrated in DLD | ✅ ARC-003-DATA |
| API Specification | ✅ ARC-002-API | ❌ Missing |
| Database Design | ✅ ARC-002-DB | ❌ Missing |
| Security Design | ✅ ARC-002-SEC | ❌ Missing |
| High-Level Design | Review document | ✅ ARC-003-HLD |
| Implementation Plan | ❌ Missing | ✅ ARC-003-PLAN |

### 3. Integration Concerns

**Current Integration Design:**
```
┌─────────────────────┐         ┌─────────────────────┐
│  Metadata Registry  │         │  Context-Aware      │
│  (Rust + ArangoDB)  │ ◄────► │  (Java + Postgres)  │
│                     │ Sync   │                     │
└─────────────────────┘ Types  └─────────────────────┘
```

**Problems:**
1. Different programming languages require separate build/deploy pipelines
2. Database incompatibility (ArangoDB vs PostgreSQL) prevents direct queries
3. Different transaction models complicate data consistency
4. Separate technology stacks increase operational overhead

## Alignment Options

### Option A: Converge to Common Stack (Recommended)

**Approach**: Both projects use the same technology stack

**Target Stack:**
| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Backend | Rust | Performance, type safety, WASM |
| Database | ArangoDB | Native graph, proven in NL government |
| Frontend | Dioxus | Single codebase with backend |
| API | REST + GraphQL | Both protocols supported |

**Migration Required:**
- Project 003: Rewrite Java Spring Boot → Rust
- Project 003: Migrate PostgreSQL schema → ArangoDB collections
- Project 003: Rewrite React UI → Dioxus

**Effort**: ~6-9 months
**Cost**: €1.2-1.8M
**Benefits**:
- Single build/deploy pipeline
- Shared code and libraries
- Lower operational complexity
- Consistent developer experience

---

### Option B: Service-Oriented Separation

**Approach**: Accept technology differences and design for service boundaries

**Integration Pattern:**
```
┌─────────────────────┐         ┌─────────────────────┐
│  Metadata Registry  │         │  Context-Aware      │
│  (Rust + ArangoDB)  │ ◄────► │  (Java + Postgres)  │
│                     │  API   │                     │
└─────────────────────┘ Gateway └─────────────────────┘
         ▲                               ▲
         │                               │
         └─────────── API Gateway ───────┘
                    (Kong)
```

**Benefits:**
- No migration required
- Teams can work independently
- Technology-appropriate choices per domain

**Drawbacks:**
- Higher operational complexity
- Consistency guarantees harder
- Two developer skill sets required

---

### Option C: Database Unification Only

**Approach**: Both projects use PostgreSQL, keep different languages

**Target Stack:**
| Component | Project 002 | Project 003 |
|-----------|-------------|-------------|
| Backend | Rust | Java |
| Database | PostgreSQL 15 + pgvector | PostgreSQL 15 + pgvector |
| Frontend | Dioxus | React |

**Benefits:**
- Shared database enables direct queries
- Single database technology to operate
- Easier data consistency

**Drawbacks:**
- Loss of graph capabilities in Project 002
- Still two languages to maintain

---

## Recommendations

### Short-Term Actions (Next 2 Weeks)

1. **Add Missing Documents to Project 002:**
   - Create ARC-002-PRIN (Architecture Principles)
   - Document design rationale from ADRs

2. **Add Missing Documents to Project 003:**
   - Create ARC-003-DLD (Detailed Design)
   - Create ARC-003-API (API Specification)
   - Create ARC-003-DB (Database Design)
   - Create ARC-003-SEC (Security Design)

3. **Create Integration Architecture Document:**
   - Document cross-project APIs
   - Define data synchronization patterns
   - Specify error handling and recovery

### Medium-Term Decisions (Next 2 Months)

**Decision Point**: Choose alignment option (A, B, or C)

**Decision Criteria**:
- Available budget for migration
- Team skills and hiring capacity
- Timeline constraints
- Risk tolerance

### Long-Term Roadmap (6-12 Months)

**If Option A (Convergence) selected:**
1. Phase 1: Establish Rust microservices foundation
2. Phase 2: Migrate Project 003 backend
3. Phase 3: Migrate Project 003 database
4. Phase 4: Migrate Project 003 frontend

**If Option B (Separation) selected:**
1. Phase 1: Implement API Gateway
2. Phase 2: Define service contracts
3. Phase 3: Implement async messaging
4. Phase 4: Implement distributed transactions

## Architectural Principles for Alignment

### P1: Technology Consistency
Where feasible, use the same technology stack across related projects.

### P2: Service Boundaries
If technologies must differ, design clear service boundaries with stable contracts.

### P3: Data Portability
Ensure data can be exported and migrated between systems using standard formats.

### P4: Developer Experience
Minimize the number of programming languages and frameworks developers must learn.

### P5: Operational Simplicity
Prefer solutions that reduce the number of distinct technologies to operate.

## Next Steps

1. **Stakeholder Review**: Present this document to architecture board
2. **Decision Workshop**: Facilitate decision on alignment option
3. **Resource Planning**: Budget and team allocation based on decision
4. **Execution**: Begin implementation of chosen option

---

**Document Owner**: Enterprise Architect
**Review Date**: 2026-05-20
**Related ADRs**: ARC-002-ADR-001 through 005, ARC-003-ADR-001 through 003
