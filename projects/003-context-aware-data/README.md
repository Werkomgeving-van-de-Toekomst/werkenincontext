# Context-Aware Data Architecture

**Project ID**: 003
**Created**: 2026-04-19
**Status**: Architecture Design Complete ✅
**Phase**: Implementation Planning
**ADRs**: 3 Complete ✅

## Project Description

This project defines the architecture for **Context-Aware Data** systems within the Ministry of Justice & Security, implementing the Data→Informatie transformation principle where raw data becomes meaningful information through contextual metadata.

The architecture establishes how government information systems can capture, store, and utilize contextual metadata to support the digital rule of law, ensuring decisions are based on complete, interpretable information rather than isolated data points.

## Vision Statement

> "Every government information object carries sufficient contextual metadata to be understandable, interpretable, and actionable—now and in the future—enabling reliable digital governance and the digital rule of law."

---

## Architecture Documents

### Core Documents

| Document | ID | Version | Status | Description |
|----------|----|---------|--------|-------------|
| **Architecture Principles** | ARC-003-PRIN | v1.0 | DRAFT | 17 principles (C1-C17) governing context-aware data |
| **Stakeholder Analysis** | ARC-003-STKE | v1.0 | DRAFT | Stakeholder drivers, goals, and engagement strategy |
| **Data Model** | ARC-003-DATA | v1.0 | DRAFT | 8 entities, 25+ context types across 4 layers |
| **High-Level Design** | ARC-003-HLD | v1.0 | DRAFT | Component architecture, integration, security |
| **Implementation Plan** | ARC-003-PLAN | v1.0 | DRAFT | 12-month roadmap, budget, resources |

### Document Traceability

```
MinJus Guidance (Data vs Informatie)
              ↓
    ARC-003-PRIN (Principles C1-C17)
              ↓
    ┌─────────┴─────────┐
    ↓                   ↓
ARC-003-STKE         ARC-003-DATA
(Stakeholders)      (Data Model)
    ↓                   ↓
    └─────────┬─────────┘
              ↓
       ARC-003-HLD
    (High-Level Design)
              ↓
       ARC-003-PLAN
  (Implementation Plan)
```

### Architecture Decision Records (ADRs)

| ADR | Topic | Status | Key Decision |
|-----|-------|--------|--------------|
| **ADR-001** | PostgreSQL Database Selection | ✅ ACCEPTED | PostgreSQL 15+ with pgvector extension |
| **ADR-002** | Context Co-Location Strategy | ✅ ACCEPTED | Context stored with parent entity for atomic access |
| **ADR-003** | Context Inference Service | ✅ ACCEPTED | Claude API + spaCy with human review workflow |

**ADR Summaries**:

- **ADR-001 (PostgreSQL)**: Selected PostgreSQL over MongoDB, ArangoDB, Oracle, and SQL Server for strong ACID guarantees, pgvector for semantic search, JSON flexibility, and government compliance. Cost savings: €100k+ vs. commercial alternatives.

- **ADR-002 (Co-Location)**: Context must be stored co-located with parent information object in the same PostgreSQL database. Ensures atomic transactions, referential integrity via foreign keys, and compliance with principles C5 (Context-Integrity) and C8 (Context Co-Location).

- **ADR-003 (Inference)**: When context cannot be captured at source, use Anthropic Claude API for LLM-based inference combined with spaCy for NER. Mandatory human review for confidence < 0.9 and all legal context. ROI: 99.9% savings vs. manual capture.

---

## Architecture Overview

### Core Principles

The Context-Aware Data Architecture is governed by 17 principles organized into 8 categories:

| Category | Principles | Focus |
|----------|------------|-------|
| **Foundational** | C1-C2 | Context by Design, Minimal Overhead |
| **Structure** | C3-C5 | Explicit Model, Layering, Integrity |
| **Capture** | C6-C7 | At Source, Inference Last Resort |
| **Storage** | C8-C9 | Co-Location, Indexing |
| **Privacy** | C10-C11 | Context IS Personal Data, Minimization |
| **Quality** | C12-C13 | Validation, Freshness |
| **Utilization** | C14-C15 | Context-First Search, Visualization |
| **Governance** | C16-C17 | Stewardship, Evolution |

### Context Layer Model

```
┌─────────────────────────────────────────────────────────────────┐
│                    CONTEXT LAYER MODEL                          │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  PROVENANCE (Layer 4)                                       │ │
│  │  Who modified, when, why - audit trail for archiving       │ │
│  └─────────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  SEMANTIC (Layer 3)                                        │ │
│  │  Legal basis, relationships, meaning, subject tags         │ │
│  └─────────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  DOMAIN (Layer 2)                                          │ │
│  │  Case number, project code, policy reference              │ │
│  └─────────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  CORE (Layer 1) - MANDATORY                                │ │
│  │  Who, what, when - minimum required for all objects        │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Data Model Summary

**8 Core Entities**:
- **E-001**: InformationObject - The data entity
- **E-002**: Context - Contextual metadata values
- **E-003**: ContextLayer - Layer definitions (Core, Domain, Semantic, Provenance)
- **E-004**: ContextType - Type definitions per layer
- **E-005**: ContextInference - AI-derived context tracking
- **E-006**: ContextQuality - Quality monitoring
- **E-007**: InformationDomain - Domain organization
- **E-008**: ContextStewardship - Governance and ownership

**25+ Context Types** across 4 layers, including:
- Core: creator, created_at, object_type, title, classification
- Domain: case_number, case_status, policy_reference, project_code
- Semantic: legal_basis, subject_tags, relationships, woo_exemption
- Provenance: modified_by, modified_at, approval_chain, source_system

---

## System Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                 CONTEXT-AWARE DATA SYSTEM                       │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Context    │  │   Context    │  │   Context    │          │
│  │   Capture    │  │   Registry   │  │   Portal     │          │
│  │   Service    │  │ (PostgreSQL) │  │    (React)    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│         │                 │                  │                  │
│         ↓                 ↓                  ↓                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Context     │  │   Metadata   │  │   Context    │          │
│  │  Inference   │  │   Registry   │  │   Quality    │          │
│  │  (Claude)    │  │   (External) │  │  Service     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

### Key Technologies

- **Database**: PostgreSQL 15 with pgvector extension
- **API**: Spring Boot 3.2 (Java 21), REST/GraphQL
- **UI**: React 18
- **AI/ML**: Anthropic Claude API, spaCy
- **Infrastructure**: Kubernetes, Docker
- **Monitoring**: Prometheus, Grafana, ELK Stack

---

## Implementation Summary

### 12-Month Roadmap

| Phase | Duration | Budget | Key Deliverables |
|-------|----------|--------|------------------|
| **1. Foundation** | Months 1-3 | €825k | Core context capture, database, basic UI |
| **2. Enhancement** | Months 4-6 | €847k | All context layers, inference, quality |
| **3. Integration** | Months 7-9 | €737k | Legacy systems, Woo, archival workflows |
| **4. Optimization** | Months 10-12 | €539k | Performance, security, handover |
| **Total** | **12 months** | **€2.95M** | **Production system** |

### Resource Requirements

- **Peak Team**: 7 FTE (Phase 2)
- **Total Effort**: ~180 person-months
- **Key Roles**: Project Lead, Backend Dev, Frontend Dev, Data Scientist, Integration Dev

### Success Criteria

**Technical**:
- 99.5% system availability
- < 500ms API response time (p95)
- 95%+ context quality score

**Business**:
- < 2 minutes extra overhead for context capture
- 50%+ automation of context population
- Positive ROI demonstrated

**Compliance**:
- AVG-approved DPIA
- Archiefwet compliant
- Woo compliant

---

## Integration Points

### Dependencies

**Internal (IOU-Modern)**:
- **Project 002**: Metadata Registry - shared context type definitions
- **Project 001**: Global Architecture Principles - alignment required

**External (MinJus)**:
- Zaaksystemen (Centric, Quadrum, etc.) - context source
- Document Management Systems - context source
- Woo Portal - context consumer
- National Archive - archival transfer

### APIs

**Context Capture API**:
- `POST /api/v1/objects/{id}/context` - Create/update context
- `GET /api/v1/objects/{id}/context` - Retrieve context
- `GET /api/v1/context/types` - List context types

**Context Inference API**:
- `POST /api/v1/inference/analyze` - Request AI inference

**Context Quality API**:
- `GET /api/v1/quality/report` - Quality metrics

---

## Governance

### Stakeholders

| Stakeholder | Role | Interest |
|-------------|------|----------|
| **CIO/Directeur I&A** | Sponsor | Strategic value, ROI |
| **Hoofd Informatiebeheer** | Owner | Operational feasibility |
| **Privacy Officer** | Compliance | AVG/GDPR adherence |
| **Informatie Architecten** | Design | Architecture alignment |
| **Archivarissen** | Domain | Provenance requirements |
| **Informatiemanagers** | Users | Usability, efficiency |

### Stewardship Model

Each context type has an assigned steward responsible for:
- Definition and documentation
- Validation rules
- Change management
- Quality assurance

---

## Compliance

### AVG/GDPR

- **DPIA**: Required and in progress
- **Legal Basis**: Legal Obligation (Art 6(1)(c))
- **Data Minimization**: Per principle C11
- **Subject Rights**: Access, rectification, erasure supported

### Archiefwet 1995

- **Retention**: 20 years maximum for decisions
- **Provenance**: Full audit trail maintained
- **Transfer**: Integration with National Archive

### Woo (Wet open overheid)

- **Publication**: Context-enriched metadata
- **Classification**: Woo fields in context model
- **Transparency**: Context-first search capability

---

## Project Structure

```
003-context-aware-data/
├── README.md                          # This file
├── ARC-003-PRIN-v1.0.md              # Architecture Principles
├── ARC-003-STKE-v1.0.md              # Stakeholder Analysis
├── ARC-003-DATA-v1.0.md              # Data Model
├── ARC-003-HLD-v1.0.md               # High-Level Design
├── ARC-003-PLAN-v1.0.md              # Implementation Plan
├── decisions/                         # Architecture Decision Records
│   ├── ARC-003-ADR-001-v1.0.md      # PostgreSQL Database Selection
│   ├── ARC-003-ADR-002-v1.0.md      # Context Co-Location Strategy
│   └── ARC-003-ADR-003-v1.0.md      # Context Inference Service
├── external/                          # External reference documents
│   └── README.md
├── diagrams/                          # Architecture diagrams (future)
└── reviews/                           # Design reviews (future)
```

---

## References

### Internal Documents

- Data versus informatie en het belang van context (MinJus Presentation)
- Strategische laag (Capabilities) document
- Motivatielaag document
- Metadata Registry Gap Analysis

### External Standards

- Archiefwet 1995
- Algemene verordening gegevensbescherming (AVG/GDPR)
- Wet open overheid (Woo)
- Metamodel GGHH Overheid 20240530

### Related Projects

- **Project 001**: IOU-Modern Global Architecture
- **Project 002**: Metadata Registry Service

---

## Contact

**Project Lead**: [To be assigned]
**Enterprise Architect**: Enterprise Architecture Team
**For questions**: Contact the Enterprise Architecture team

---

**Last Updated**: 2026-04-19
**Status**: Architecture Design Complete with ADRs - Ready for Implementation
