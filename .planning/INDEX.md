# IOU-Modern Planning Index

## Project Status

| Milestone | Status | Last Updated |
|-----------|--------|-------------|
| v1.0 - 3D Buildings Phase 2 | ✅ Complete | 2026-03-27 |

**Current Position:** Milestone v1.0 complete - ready for next feature

## Document Structure

### Project Level (`./`)
- **[PROJECT.md](PROJECT.md)** - Core value, requirements, context, decisions
- **[REQUIREMENTS.md](REQUIREMENTS.md)** - Validated, active, and out-of-scope requirements
- **[ROADMAP.md](ROADMAP.md)** - Phase execution plan with dependencies
- **[STATE.md](STATE.md)** - Current position, progress metrics, accumulated context

### Phases (`./phases/`)
Detailed implementation plans for each phase:

| Phase | Description | Status |
|-------|-------------|--------|
| [2.1](phases/2.1-building-filtering/) | Building Filter Controls | ✅ Complete |
| [2.2](phases/2.2-view-toggle/) | 2D/3D View Toggle | ✅ Complete |
| [2.3](phases/2.3-density-analysis/) | Density Heatmap | ✅ Complete |
| [2.4](phases/2.4-polish/) | Polish & URL State | ✅ Complete |
| [2-GAP](phases/2-gap-closure/) | UAT Gap Fixes | ✅ 5/5 Complete |

### Features (`./features/`)
Cross-cutting features that span multiple phases:

| Feature | Description | Source | Status |
|---------|-------------|--------|--------|
| [agent-orchestration](features/agent-orchestration/) | AI agent workflow with human-in-the-loop | planning/ | 📋 Planned |
| [document-workflow](features/document-workflow/) | Multi-stage approvals, delegation, escalation | planning-document-workflow/ | 📋 Planned |
| [api-extensions](features/api-extensions/) | Pipeline executor, auth, S3, WebSocket | planning-api-extensions/ | 📋 Planned |
| [backend-eval](features/backend-eval/) | Database selection (DuckDB vs Supabase vs Convex) | planning-backend-eval/ | 📋 Research |
| [3d-map](features/3d-map/) | 3D buildings visualization enhancements | planning-3d-enhancements/ | ✅ Phase 2 Done |

### Research (`./research/`)
Background research and exploratory work:

- **graphrag_interview.md** - GraphRAG stakeholder requirements
- **future-features-requirements.md** - Potential future enhancements

### Archive (`./archive/`)
Superseded or historical planning documents:

- Original deep-plan outputs (superseded by GSD phases)
- Deprecated feature specifications

## Quick Navigation

### For Developers
- Start with [PROJECT.md](PROJECT.md) for context
- Check [STATE.md](STATE.md) for current position
- See [phases/](phases/) for active implementation plans

### For Product Owners
- Review [REQUIREMENTS.md](REQUIREMENTS.md) for validated requirements
- Check [ROADMAP.md](ROADMAP.md) for timeline and dependencies
- See [STATE.md](STATE.md) for progress tracking

### For Architects
- Review [features/](features/) for cross-cutting concerns
- Check [research/](research/) for technical background
- See project decisions in [PROJECT.md](PROJECT.md)

## Phase Progress Summary

```
Phase 1 (Foundation)     ████████████████████ 100%  (Completed 2025-03-07)
Phase 2.1 (Filtering)    ████████████████████ 100%  (Completed 2026-03-08)
Phase 2.2 (View Toggle)  ████████████████████ 100%  (Completed 2026-03-08)
Phase 2.3 (Density)      ████████████████████ 100%  (Completed 2026-03-08)
Phase 2.4 (Polish)       ████████████████████ 100%  (Completed 2026-03-08)
Gap Closure              ████████████████████ 100%  (5/5 complete - 2026-03-27)
Phase 3                  ░░░░░░░░░░░░░░░░░░░░   0%   (Not started)
```

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Complete |
| 🟡 | In Progress |
| 📋 | Planned |
| ⚠️ | Blocked |
| ❌ | Cancelled |

---

*Last updated: 2026-03-27*
