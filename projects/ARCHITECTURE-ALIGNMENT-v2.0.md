# Architecture Alignment Plan: All Projects (001, 002, 003)

**Document ID**: ARC-ALIGN-002-v1.0
**Updated**: 2026-04-20
**Projects**: IOU-Modern (001) + Metadata Registry (002) + Context-Aware Data (003)

## Executive Summary

Analysis of all three IOU-Modern projects reveals a more complex alignment picture. Project 001 (main IOU-Modern) already uses Rust and Dioxus, but with different backend framework (Axum) and database (DuckDB/PostgreSQL).

## Complete Technology Stack Comparison

| Component | Project 001 (IOU-Modern) | Project 002 (Metadata) | Project 003 (Context) |
|-----------|------------------------|----------------------|---------------------|
| **Backend** | Rust (Axum) | Rust (actix-web) | Java (Spring Boot) |
| **Database** | DuckDB + PostgreSQL | ArangoDB | PostgreSQL + pgvector |
| **Frontend** | Dioxus (WASM) | Dioxus (WASM) | React 18 |
| **AI/ML** | Rust agents | N/A | Python (FastAPI) |
| **Maps** | Leaflet.js | N/A | N/A |

## Key Findings

### Positive: Partial Alignment Already Exists

1. **Rust Consistency**: Projects 001 and 002 both use Rust
2. **Dioxus Consistency**: Projects 001 and 002 both use Dioxus
3. **Shared Vision**: All three target government information management

### Issues to Resolve

| Issue | Impact | Projects Affected |
|-------|--------|------------------|
| **Framework Split** | Medium | 001 (Axum) vs 002 (actix-web) |
| **Database Fragmentation** | High | DuckDB/PostgreSQL vs ArangoDB vs PostgreSQL+pgvector |
| **Frontend Inconsistency** | Medium | Dioxus (001,002) vs React (003) |
| **AI/ML Scattered** | Low | Rust (001) vs None (002) vs Python (003) |

## Revised Alignment Options

### Option A: Full Convergence to ArangoDB Stack

**Target:**
- All projects use Rust + actix-web + ArangoDB + Dioxus

**Changes Required:**
- Project 001: Axum → actix-web, DuckDB → ArangoDB
- Project 002: No change (already aligned)
- Project 003: Java → Rust, PostgreSQL → ArangoDB, React → Dioxus

**Effort**: 8-10 months | **Cost**: €1.5-2.2M

**Pros:**
- Complete technology consistency
- Strong graph capabilities across all projects
- Single database technology

**Cons:**
- Largest migration effort
- Project 001 (main app) requires significant changes

---

### Option B: Pragmatic Hybrid (NEW - Recommended)

**Target:**
- Unified Rust backend with framework flexibility
- Dioxus for all frontends
- Strategic database selection per project

```
┌─────────────────────────────────────────────────────────────────────┐
│                    IOU-MODERN UNIFIED PLATFORM                      │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Rust Workspace (Unified Types, Traits, Patterns)           │   │
│  │                                                              │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │   │
│  │  │ Project 001  │  │ Project 002  │  │ Project 003  │      │   │
│  │  │  (Axum)      │  │(actix-web)  │  │  (Axum)      │      │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘      │   │
│  │         │                 │                  │              │   │
│  │  ┌────────────────────────────────────────────────────┐    │   │
│  │  │         Shared Crate (common types, errors)       │    │   │
│  │  └────────────────────────────────────────────────────┘    │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Multi-Database Strategy (Per-Project Optimization)         │   │
│  │                                                              │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │   │
│  │  │ Project 001 │  │ Project 002 │  │ Project 003  │         │   │
│  │  │DuckDB+PGSQL │  │  ArangoDB   │  │  ArangoDB   │         │   │
│  │  │ (Analytics) │  │  (Graph)    │  │  (Graph)     │         │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘         │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Unified Dioxus Frontend (Single Codebase)                  │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

**Database Rationale:**
- **Project 001**: Keep DuckDB for analytics, PostgreSQL for transactions (already working)
- **Project 002**: ArangoDB for graph queries (GGHH V2 requirements)
- **Project 003**: ArangoDB for context graph (migrate from PostgreSQL)

**Framework Strategy:**
- **Axum** as default for new work (Project 003, 001)
- **actix-web** remains for Project 002 (already implemented)
- Both frameworks interoperate via shared types

**Effort**: 5-7 months | **Cost**: €800k-1.2M

**Pros:**
- Leverages existing work in Projects 001 and 002
- Minimal changes to Project 001 (already Rust + Dioxus)
- Database choice optimized per project needs
- Shared types enable code reuse

**Cons:**
- Two web frameworks to maintain
- Two database technologies (but justified by use case)

---

### Option C: Service-Oriented Separation

**Target:**
- Accept current differences, add API Gateway

**Effort**: 2-3 months | **Cost**: €200-400k

**Pros:**
- Minimal disruption
- Fastest to implement

**Cons:**
- Permanent fragmentation
- Highest long-term cost

## Recommendation: Option B - Pragmatic Hybrid

**Rationale:**
1. **Project 001 is the reference implementation** - already Rust + Dioxus
2. **Project 002 is aligned** - already Rust + ArangoDB + Dioxus
3. **Only Project 003 needs migration** - migrate to Rust, adopt ArangoDB, use Dioxus

**Migration Priority:**
1. **Phase 1**: Create shared workspace (all projects)
2. **Phase 2**: Migrate Project 003 backend to Rust (Axum for consistency with 001)
3. **Phase 3**: Migrate Project 003 database to ArangoDB
4. **Phase 4**: Migrate Project 003 frontend to Dioxus
5. **Phase 5**: Integrate unified UI components

## Shared Workspace Structure

```
iou-modern/
├── Cargo.toml                 # Workspace root
│
├── crates/
│   ├── shared/                # Common to all projects
│   │   ├── entities.rs        # IouEntity, Temporal, Owned traits
│   │   ├── error.rs           # IouError, IouResult
│   │   ├── api.rs             # API response types
│   │   └── validation.rs      # Common validation
│   │
│   ├── project-001/           # IOU-Modern (keep Axum + DuckDB/PG)
│   │   ├── api/               # Axum server
│   │   ├── agents/            # AI agents
│   │   └── storage/           # S3/MinIO client
│   │
│   ├── project-002/           # Metadata Registry (keep actix-web + ArangoDB)
│   │   ├── api/               # actix-web server
│   │   ├── gghh/              # GGHH V2 entities
│   │   └── repositories/      # ArangoDB repos
│   │
│   └── project-003/           # Context-Aware (migrate to Axum + ArangoDB)
│       ├── api/               # Axum server (NEW)
│       ├── context/           # Context entities
│       ├── inference/         # AI inference (Rust)
│       └── repositories/      # ArangoDB repos (NEW)
│
└── ui/                        # Unified Dioxus app
    ├── src/
    │   ├── iou/               # Project 001 screens
    │   ├── metadata/          # Project 002 screens
    │   ├── context/           # Project 003 screens
    │   └── components/        # Shared components
```

## Implementation Plan (Option B)

### Phase 1: Foundation (Month 1)
- Create unified workspace
- Extract shared types from Projects 001 and 002
- Set up monorepo structure

### Phase 2: Project 003 Backend (Months 2-3)
- Implement context entities in Rust
- Create ArangoDB repositories
- Build Axum API endpoints

### Phase 3: Project 003 Database (Months 3-4)
- Migrate PostgreSQL schema to ArangoDB
- Implement context graph structure
- Data migration scripts

### Phase 4: Project 003 Frontend (Months 5-6)
- Implement Dioxus context screens
- Create shared UI components
- Integrate with unified app

### Phase 5: Integration (Month 7)
- Cross-project API integration
- Unified authentication
- Documentation

## Success Metrics

| Metric | Target |
|--------|--------|
| **Language Consistency** | 100% Rust across all projects |
| **Frontend Consistency** | 100% Dioxus across all projects |
| **Shared Code** | >30% code reuse via shared crate |
| **Build Time** | < 5 min for full workspace |
| **Deployment** | Single Docker image per project |

## Next Steps

1. Review and approve Option B (Pragmatic Hybrid)
2. Create detailed implementation plan
3. Begin Phase 1: Foundation
4. Establish governance for shared crate evolution

---

**Document Owner**: Enterprise Architect
**Review Date**: 2026-05-20
**Related**: ARCHITECTURE-ALIGNMENT-v1.0, MIGRATION-PLAN-v1.0
