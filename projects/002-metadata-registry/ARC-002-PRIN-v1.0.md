# Architecture Principles: Metadata Registry Service

> **Template Origin**: Official | **ArcKit Version**: 4.3.1

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-002-PRIN-v1.0 |
| **Document Type** | Architecture Principles |
| **Project** | Metadata Registry Service (Project 002) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.0 |
| **Created Date** | 2026-04-20 |
| **Owner** | Enterprise Architect |

---

## Introduction

### Purpose

These architecture principles guide the design, implementation, and evolution of the Metadata Registry Service. They ensure alignment with Dutch government standards, particularly the Metamodel GGHH (Gemeentelijk Gegevensmodel Haagse Hoek), and support the broader IOU-Modern initiative.

### Scope

These principles apply to:
- All Metadata Registry Service components
- Integration with Project 003 (Context-Aware Data)
- Future metadata-related services

---

## Architecture Principles

### P1: Sovereign Technology First

**Statement**: Use only open-source technologies with permissive licensing that can be self-hosted within Dutch government infrastructure.

**Rationale**: The Dutch government's Sovereign Technology policy requires independence from proprietary vendors and ensures digital sovereignty.

**Implications**:
- Prefer Rust (MIT/Apache 2.0) over commercial alternatives
- Use ArangoDB (Apache 2.0) instead of commercial graph databases
- Avoid vendor-specific cloud services

**Related ADR**: ARC-002-ADR-005

---

### P2: GGHH V2 Compliance

**Statement**: All metadata must conform to Metamodel GGHH V2 specification, including time validity (tijdsdimensie) and retention (bewaartermijn) concepts.

**Rationale**: GGHH V2 is the Dutch government standard for metadata management, ensuring interoperability across agencies.

**Implications**:
- All entities include `geldig_vanaf` and `geldig_tot` fields
- Retention periods are explicitly defined and enforced
- Graph relationships follow GGHH edge specifications

**Related Requirements**: BR-MREG-001

---

### P3: Type Safety by Default

**Statement**: Leverage compile-time type checking to prevent entire classes of runtime errors.

**Rationale**: Metadata integrity is critical; errors can propagate across government systems.

**Implications**:
- Use Rust's type system for all entity definitions
- Enforce type safety at API boundaries
- Use serde for validated serialization/deserialization

**Related ADR**: ARC-002-ADR-001

---

### P4: Audit Everything

**Statement**: All metadata changes must be logged with who, what, when, and why information.

**Rationale**: Government data requires full provenance for accountability and compliance.

**Implications**:
- Immutable audit log for all mutations
- User identification required for write operations
- Audit logs retained per Archiefwet 1995 requirements

**Related Design**: ARC-002-SEC

---

### P5: Context Co-Location

**Statement**: Contextual metadata must be stored and accessed atomically with its parent entity.

**Rationale**: Ensures referential integrity and prevents stale context references.

**Implications**:
- Context stored in same database transaction as parent entity
- No separate "context service" that could become inconsistent
- Graph edges enforce referential integrity

**Related to Project 003**: Aligns with principle C8 in ARC-003-PRIN

---

### P6: API Version Stability

**Statement**: API changes must be backward compatible within a major version.

**Rationale**: Government systems have long lifecycles; breaking changes cause widespread disruption.

**Implications**:
- Semantic versioning for all APIs
- Deprecation period of minimum 12 months
- v1 API maintained alongside v2

**Related Design**: ARC-002-API

---

### P7: Performance at Scale

**Statement**: The system must support 10M+ information objects with sub-100ms query response times.

**Rationale**: Dutch government generates massive amounts of metadata; performance is critical for usability.

**Implications**:
- ArangoDB with RocksDB storage engine
- Graph queries optimized with proper indexes
- Connection pooling for high concurrency

**Related ADR**: ARC-002-ADR-002

---

### P8: Privacy by Design

**Statement**: Personal data in metadata must be identified, protected, and subject to GDPR controls.

**Rationale**: Metadata often contains personal information; AVG/GDPR compliance is mandatory.

**Implications**:
- PII detection in validation pipeline
- Row-level security for personal data
- DPIA executed for high-risk processing

**Related Design**: ARC-002-SEC

---

### P9: GitOps for Configuration

**Statement**: All metadata type definitions and validation rules stored in Git and synced via automation.

**Rationale**: Enables version control, peer review, and audit trails for metadata governance.

**Implications**:
- YAML definitions in Git repository
- Automated sync on merge to main
- Rollback capability via Git revert

**Related Components**: metadata-gitops crate

---

### P10: Interoperability Standard

**Statement**: Support both REST and GraphQL APIs to accommodate different consumer preferences.

**Rationale**: Government systems have diverse integration requirements.

**Implications**:
- REST API for traditional integrations
- GraphQL API for flexible queries
- Both APIs provide equivalent functionality

**Related Design**: ARC-002-API

---

## Principle Relationships with Project 003

| ARC-002 Principle | ARC-003 Principle | Alignment |
|-------------------|-------------------|-----------|
| P5: Context Co-Location | C8: Context Co-Location | ✅ Identical |
| P8: Privacy by Design | C10: Context IS Personal Data | ✅ Aligned |
| P4: Audit Everything | Layer 4: Provenance | ✅ Aligned |
| P2: GGHH V2 Compliance | C3: Explicit Context Model | ✅ Aligned |
| P7: Performance at Scale | C9: Context Indexing | ✅ Aligned |

---

## Compliance Mapping

### AVG/GDPR
- P8: Privacy by Design → GDPR Article 25
- P4: Audit Everything → GDPR Article 30

### Archiefwet 1995
- P2: GGHH V2 → Retention requirements
- P4: Audit Everything → Provenance requirements

### Woo (Wet open overheid)
- P2: GGHH V2 → Publication metadata
- P10: Interoperability → Public access API

---

## Principle Trade-offs

| Principle | Trade-off | Decision |
|-----------|-----------|----------|
| P3 (Type Safety) vs Development Speed | Rust has slower development than interpreted languages | Type safety prioritized for government data integrity |
| P7 (Performance) vs Cost | High performance requires more infrastructure | Performance prioritized; accept higher infrastructure cost |
| P1 (Sovereign Tech) vs Feature Completeness | Open source may lag commercial features | Sovereignty prioritized; build missing features if needed |

---

## Evolution Principles

These principles will be reviewed annually or when:
- New Dutch government standards are issued
- Security vulnerabilities require technology changes
- Performance requirements increase significantly
- Integration with Project 003 reveals inconsistencies

---

## References

- Metamodel GGHH Overheid 20240530
- Sovereign Technology principles (Forum Standaardisatie)
- AVG/GDPR implementation guidelines
- Archiefwet 1995 requirements
- Project 003: ARC-003-PRIN-v1.0

---

**Document Owner**: Enterprise Architect
**Review Date**: 2027-04-20
**Status**: DRAFT - Pending Architecture Board Review
