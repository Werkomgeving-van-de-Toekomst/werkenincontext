# Pragmatic Hybrid Alignment Status

**Document ID**: ARC-ALIGN-STATUS-v1.1
**Updated**: 2026-04-20
**Status**: 75% Complete

## Current State Summary

| Aspect | Project 001 (IOU-Modern) | Project 002 (Metadata) | Project 003 (Context) | Status |
|--------|-------------------------|----------------------|----------------------|--------|
| **Language** | Rust ✓ | Rust ✓ | Rust ✓ | ✅ 100% |
| **Web Framework** | Axum 0.8 | actix-web 4.0 | Axum 0.8 (planned) | ✅ Aligned |
| **Frontend** | Dioxus | Dioxus (planned) | TBD | ⚠️ 67% |
| **Database** | DuckDB + PostgreSQL | ArangoDB | ArangoDB (planned) | ✅ Per-use |
| **Workspace** | server/ | metadata-registry/ | context-aware/ | ✅ Separate |

## Workspace Structure

```
iou-modern/
├── Cargo.toml                 # Root workspace (shared crates)
├── crates/
│   ├── iou-core/              # Shared domain models ✓
│   ├── iou-regels/            # PROVISA/BPMN rules ✓
│   └── shared/                # Common types (NEW)
│
├── server/                    # Project 001 workspace
│   ├── Cargo.toml             # Axum + DuckDB/PostgreSQL
│   └── crates/
│       ├── iou-api/           # API server ✓
│       ├── iou-storage/       # S3/MinIO ✓
│       ├── iou-orchestrator/  # AI orchestration ✓
│       ├── iou-ai/            # AI agents ✓
│       └── iou-camunda-worker/ # Camunda/Zeebe BPMN ✓
│
├── metadata-registry/         # Project 002 workspace
│   ├── Cargo.toml             # actix-web + ArangoDB
│   └── crates/
│       ├── metadata-core/     # GGHH V2 entities ✓
│       ├── metadata-api/      # REST/GraphQL ✓
│       ├── metadata-store/    # ArangoDB repos ✓
│       ├── metadata-validation/ ✓
│       ├── metadata-gitops/   # ✓
│       ├── metadata-admin/    # Dioxus UI (planned)
│       └── metadata-migration/ ✓
│
├── context-aware/             # Project 003 workspace
│   ├── Cargo.toml             # Axum + ArangoDB
│   └── crates/
│       ├── context-core/      # Entities ✓
│       ├── context-api/       # TODO
│       ├── context-domain/    # TODO
│       ├── context-semantic/  # TODO
│       ├── context-provenance/ # TODO
│       ├── context-inference/ # TODO
│       ├── context-quality/   # TODO
│       └── context-store/     # TODO
│
└── frontend/                  # Unified Dioxus workspace
    └── crates/
        └── iou-frontend/      # Dioxus app
```

## Implementation Status by Crate

### Project 001: IOU-Modern ✅ COMPLETE

| Crate | Status | Notes |
|-------|--------|-------|
| iou-core | ✅ | Shared types, Id, Timestamp |
| iou-regels | ✅ | PROVISA rules engine |
| iou-api | ✅ | Axum REST API |
| iou-storage | ✅ | S3/MinIO client |
| iou-orchestrator | ✅ | Multi-agent orchestration |
| iou-ai | ✅ | AI agent implementations |
| iou-ai-service | ✅ | AI service wrapper |
| iou-camunda-worker | ✅ | Camunda/Zeebe BPMN integration |
| ~~workflow-builder~~ | ❌ | REMOVED: Use Camunda instead |

### Project 002: Metadata Registry ✅ COMPLETE

| Crate | Status | Notes |
|-------|--------|-------|
| metadata-core | ✅ | GGHH V2 entities |
| metadata-api | ✅ | actix-web REST/GraphQL |
| metadata-store | ✅ | ArangoDB repositories |
| metadata-validation | ✅ | Validation engine |
| metadata-gitops | ✅ | Git-based config sync |
| metadata-admin | ⚠️ | Dioxus UI (planned) |
| metadata-migration | ✅ | Database migrations |

### Project 003: Context-Aware Data 🚧 IN PROGRESS

| Crate | Status | Priority |
|-------|--------|----------|
| context-core | ✅ | Base entities (Context, layers, quality) |
| context-api | ✅ | Axum REST endpoints (CRUD, search) |
| context-store | ✅ | ArangoDB repositories (ConnectionPool, migrations) |
| context-domain | 🟡 MEDIUM | Domain context logic |
| context-semantic | 🟡 MEDIUM | Semantic context (NER, entities) |
| context-provenance | 🟡 MEDIUM | Provenance tracking |
| context-inference | ✅ | Nebul API + local NLP (Dutch entities, BWBR) |
| context-quality | 🟡 MEDIUM | Quality scoring |

## Frontend Status

| Project | Framework | Status |
|---------|-----------|--------|
| 001 (IOU-Modern) | Dioxus | ✅ Complete |
| 002 (Metadata) | Dioxus | ⚠️ Planned in metadata-admin |
| 003 (Context) | Dioxus | ❌ Not started |

**Recommendation**: Create unified Dioxus app with per-project modules.

## Completion Plan

### Phase 1: Complete Context-Aware Backend (4-6 weeks)

#### Week 1-2: Core Infrastructure
- [x] context-store: ArangoDB repositories
  - ContextRepository trait
  - ArangoDB implementation
  - Connection pooling
  - Integration with metadata-core types

- [x] context-api: Axum REST API
  - CRUD endpoints for context
  - Integration with context-core entities
  - Error handling and validation

#### Week 3-4: Domain Logic
- [ ] context-domain: Domain context implementation
  - Zaak, Project, Beleid, Expertise handlers
  - Integration with legacy systems
  - Domain-specific validation

- [ ] context-semantic: Semantic context
  - NER pipeline
  - Entity linking
  - Keyword extraction

#### Week 5-6: Advanced Features
- [ ] context-inference: AI inference service
  - Claude API integration
  - spaCy/Candle NLP
  - Confidence scoring
  - Human review workflow

- [ ] context-quality: Quality monitoring
  - Completeness scoring
  - Accuracy tracking
  - Quality dashboard API

### Phase 2: Unified Frontend (3-4 weeks)

#### Week 1-2: Dioxus Foundation
- [ ] Create unified workspace for all UIs
- [ ] Shared components library
- [ ] Navigation/routing structure

#### Week 3-4: Project Modules
- [ ] Port 001 UI to unified structure
- [ ] Implement 002 metadata admin UI
- [ ] Implement 003 context management UI

### Phase 3: Integration (2-3 weeks)

- [ ] Cross-workspace API integration
- [ ] Unified authentication
- [ ] End-to-end testing
- [ ] Documentation

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Context schema changes | LOW | MEDIUM | Use versioned migrations |
| ArangoDB performance | LOW | MEDIUM | Early performance testing |
| AI inference accuracy | MEDIUM | HIGH | Human review workflow |
| Frontend complexity | MEDIUM | MEDIUM | Component library |

## Resource Requirements

| Phase | Duration | Backend Dev | Frontend Dev | Total FTE |
|-------|----------|-------------|--------------|-----------|
| Phase 1 | 4-6 weeks | 2 | 0 | 2 |
| Phase 2 | 3-4 weeks | 0.5 | 1.5 | 2 |
| Phase 3 | 2-3 weeks | 1 | 1 | 2 |
| **Total** | **9-13 weeks** | | | **2** |

## Success Criteria

- [ ] All context-aware crates implemented
- [ ] API tests passing (>80% coverage)
- [ ] Frontend modules for all 3 projects
- [ ] Integration tests passing
- [ ] Performance benchmarks met (<100ms p95)
- [ ] Documentation complete

---

**Next Steps:**
1. Create context-store crate
2. Create context-api crate
3. Begin implementation of Phase 1

**Owner**: Enterprise Architect
**Review**: Weekly during implementation
