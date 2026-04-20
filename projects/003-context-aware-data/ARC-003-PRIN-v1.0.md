# Context-Aware Data Architecture Principles

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Command**: `/arckit.principles`

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-003-PRIN-v1.0 |
| **Document Type** | Context-Aware Data Architecture Principles |
| **Project** | Context-Aware Data Architecture (Project 003) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.0 |
| **Created Date** | 2026-04-19 |
| **Last Modified** | 2026-04-19 |
| **Review Cycle** | Annual |
| **Next Review Date** | 2027-04-19 |
| **Owner** | Enterprise Architect |
| **Reviewed By** | PENDING |
| **Approved By** | PENDING |
| **Distribution** | Project Team, Architecture Team, MinJus Leadership |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---------|------|--------|---------|-------------|---------------|
| 1.0 | 2026-04-19 | ArcKit AI | Initial creation from `/arckit.principles` command | PENDING | PENDING |

---

## Executive Summary

This document establishes the architecture principles governing **context-aware data** systems within the Ministry of Justice & Security. These principles extend the global IOU-Modern architecture principles with specific guidance for capturing, storing, and utilizing contextual metadata to transform raw data into meaningful information.

**Scope**: All systems that capture, process, or serve contextual metadata for government information

**Authority**: Enterprise Architecture Board, MinJus

**Compliance**: Mandatory for all context-aware data initiatives

**Philosophy**: **Data without context is noise; context transforms data into information.** These principles ensure that contextual metadata is captured systematically, stored efficiently, and utilized effectively while maintaining compliance with Woo, AVG, and Archiefwet.

**Alignment**: These principles extend ARC-000-PRIN (IOU-Global Principles) and must be read in conjunction with that document.

---

## I. Foundational Principles

### C1: Context by Design (NON-NEGOTIABLE)

**Principle Statement**:
Contextual metadata MUST be considered a first-class architectural concern from the initial design phase, not added as an afterthought.

**Rationale**:
Retroactively adding context to existing data is expensive, error-prone, and often incomplete. The Ministry's guidance on "Data versus Informatie" emphasizes that context is essential for meaningful information use in government decision-making.

**Implications**:
- Data models MUST include contextual metadata elements from initial design
- Context requirements MUST be defined alongside functional requirements
- Context-captuur mechanisms MUST be designed before implementation
- Testing MUST include validation of contextual data quality

**Validation Gates**:
- [ ] Context requirements documented in data model
- [ ] Context-capture mechanism designed before implementation
- [ ] Context validation included in test plan
- [ ] Privacy impact assessment (DPIA) considers context as personal data

**MinJus Alignment**:
- Extends P1 (Privacy by Design) - Context IS personal data when it identifies persons
- Supports P2 (Open Government) - Context enables proper Woo classification
- Aligns with P5 (Domain-Driven) - Context defines domain boundaries

---

### C2: Minimal Overhead, Maximum Value

**Principle Statement**:
Context-captuur MUST impose minimal overhead on information managers while maximizing the value of captured context.

**Rationale**:
Information managers have existing workloads. Excessive overhead leads to resistance, poor data quality, or abandonment. The principle balances comprehensiveness with usability.

**Implications**:
- Context-captuur MUST NOT exceed 2 minutes additional effort per entity
- At least 50% of context fields MUST be auto-populated where possible
- Context fields MUST be prioritized (mandatory vs optional) based on value
- UI/UX MUST optimize for efficient context entry

**Validation Gates**:
- [ ] Time-and-motion analysis confirms < 2 minutes overhead
- [ ] Auto-population mechanisms identified and implemented
- [ ] Field prioritization documented with rationale
- [ ] Usability testing validates acceptable burden

**MinJus Alignment**:
- Supports P6 (Human-in-the-Loop) - Technology assists, humans validate
- Extends P10 (Observability) - Context itself is observability of data

---

## II. Context Structure Principles

### C3: Explicit Context Model

**Principle Statement**:
All context-aware systems MUST use an explicit, documented context model that defines what context is, how it's captured, and how it's utilized.

**Rationale**:
Without an explicit model, "context" becomes ambiguous and inconsistent. An explicit model enables shared understanding, consistent implementation, and effective utilization.

**Implications**:
- Context model MUST document context types, sources, and relationships
- Model MUST distinguish between core context (always required) and extended context
- Model MUST align with MinJus guidance on Data→Informatie transformation
- Model MUST be versioned with change management

**Validation Gates**:
- [ ] Context model documented and approved
- [ ] Context types clearly defined with examples
- [ ] Core vs extended context distinguished
- [ ] Model version controlled with change log

**MinJus Alignment**:
- Extends P5 (Domain-Driven) - Context model IS the domain model
- Supports P8 (Interoperability) - Explicit model enables cross-system sharing

---

### C4: Context Layering

**Principle Statement**:
Contextual metadata MUST be organized in layers from core (minimal required) to extended (optional enriched), enabling flexible implementation.

**Rationale**:
Not all use cases require the same depth of context. Layering allows progressive enhancement, balancing overhead against value.

**Implications**:
- **Core Layer**: Who, what, when, where (minimum required)
- **Domain Layer**: Domain-specific context (policy, case, project references)
- **Semantic Layer**: Relationships, classifications, meaning
- **Provenance Layer**: Origin, changes, custody history
- Systems MAY implement partial layers based on use case
- Core layer MANDATORY for all context-aware systems

**Validation Gates**:
- [ ] Layer structure defined in context model
- [ ] Core layer requirements specified
- [ ] Layer dependencies documented
- [ ] Extension points defined for future layers

**MinJus Alignment**:
- Supports P3 (Archival Integrity) - Provenance layer enables archival compliance
- Extends P1 (Privacy by Design) - Layering enables privacy-by-default

---

### C5: Context-Entegrity

**Principle Statement**:
Contextual metadata MUST maintain referential integrity with its associated data entity. Orphaned, inconsistent, or contradictory context is prohibited.

**Rationale**:
Context without integrity is worse than no context - it creates confusion and potential legal issues. Government decisions depend on reliable context.

**Implications**:
- Context MUST be linked to data entity via immutable reference
- Context changes MUST be versioned with audit trail
- Context deletion MUST follow same retention rules as data entity
- Contradictory context MUST be flagged and resolved

**Validation Gates**:
- [ ] Referential integrity enforced at data level
- [ ] Context versioning implemented
- [ ] Orphan detection and prevention mechanisms
- [ ] Contradiction detection and resolution process

**MinJus Alignment**:
- Directly supports P3 (Archival Integrity) - Context requires archival integrity
- Extends P10 (Observability) - Context integrity IS observability

---

## III. Context Capture Principles

### C6: Capture at Source

**Principle Statement**:
Contextual metadata MUST be captured as close to the data creation point as possible, minimizing reconstruction and inference.

**Rationale**:
Reconstructed context is error-prone, incomplete, and expensive. Source-capture ensures accuracy and completeness while the context is known.

**Implications**:
- Context fields MUST be available at data entry point
- Context capture MUST be part of standard workflow, not separate task
- Systems MAY use defaults, templates, and suggestions to reduce burden
- Systems MUST prevent data entry without required context

**Validation Gates**:
- [ ] Context fields integrated into data entry UI
- [ ] Workflow includes context validation
- [ ] Required context enforced before save
- [ ] User feedback confirms minimal disruption

**MinJus Alignment**:
- Supports P5 (Domain-Driven) - Source IS within domain context
- Extends P6 (Human-in-the-Loop) - Human provides context at creation

---

### C7: Context Inference as Last Resort

**Principle Statement**:
When context cannot be captured at source, systems MAY use automated inference, but MUST clearly distinguish between captured and inferred context.

**Rationale**:
Inferred context is better than no context but carries risk. Transparency about inference sources enables proper interpretation and risk management.

**Implications**:
- Inferred context MUST be labeled as such
- Inference method and confidence score MUST be recorded
- Inferred context MUST be reviewable and correctable by humans
- Systems MUST prefer captured context over inferred when both exist

**Validation Gates**:
- [ ] Inference methods documented
- [ ] Confidence thresholds defined
- [ ] Human review mechanism for high-stakes inferences
- [ ] Audit trail captures inference decisions

**MinJus Alignment**:
- Extends P6 (Human-in-the-Loop) - Human oversight of inference
- Supports P1 (Privacy by Design) - Inference may create personal data

---

## IV. Context Storage Principles

### C8: Context Co-Location

**Principle Statement**:
Contextual metadata MUST be stored with its associated data entity, ensuring atomic access and preventing orphaning.

**Rationale**:
Co-located storage ensures context is always available when data is accessed, simplifies queries, and prevents orphaning.

**Implications**:
- Context stored in same database/record as data entity
- Context included in data export/import operations
- Context replication matches data replication strategy
- Backup and restore include context automatically

**Validation Gates**:
- [ ] Data model includes context as first-class fields
- [ ] Query patterns access data with context atomically
- [ ] Export/import includes context
- [ ] Backup验证 includes context

**MinJus Alignment**:
- Supports P3 (Archival Integrity) - Co-location prevents loss
- Extends P4 (Sovereign Technology) - Storage choice remains open

---

### C9: Context Indexing

**Principle Statement**:
Contextual metadata MUST be indexed for efficient querying, enabling context-based search, filtering, and analysis.

**Rationale**:
The value of context is realized through querying and analysis. Without indexing, context becomes a storage burden with no retrieval benefit.

**Implications**:
- Core context fields MUST be indexed
- Domain-specific context MAY be indexed based on query patterns
- Index design MUST support common context queries (who, when, where, domain)
- Full-text search SHOULD include context fields

**Validation Gates**:
- [ ] Index strategy documented for context fields
- [ ] Query performance validated for common patterns
- [ ] Full-text search includes context
- [ ] Index maintenance strategy defined

**MinJus Alignment**:
- Supports P2 (Open Government) - Context enables Woo queries
- Extends P5 (Domain-Driven) - Domain queries require context

---

## V. Context Privacy Principles

### C10: Context IS Personal Data (NON-NEGOTIABLE)

**Principle Statement**:
Any contextual metadata that identifies or can identify a natural person MUST be treated as personal data under AVG/GDPR.

**Rationale**:
Context such as "who created this", "who approved this", or "related to case X" can identify individuals. Privacy law applies to contextual metadata.

**Implications**:
- Context fields MUST be classified for personal data content
- Personal context MUST have appropriate access controls
- Personal context MUST be included in DPIA
- Personal context MUST respect data subject rights (access, deletion, correction)

**Validation Gates**:
- [ ] Privacy classification for all context fields
- [ ] DPIA includes context-capture impact
- [ ] Access controls enforce privacy requirements
- [ ] Data subject rights extend to context

**MinJus Alignment**:
- Directly extends P1 (Privacy by Design) - Context requires privacy design
- Supports P7 (Data Sovereignty) - Personal context remains in EU

---

### C11: Context Minimization

**Principle Statement**:
Context-captuur MUST collect only the contextual metadata that is necessary for the stated purpose, avoiding excessive or speculative collection.

**Rationale**:
AVG/GDPR requires data minimization. Excessive context increases privacy risk without proportional benefit.

**Implications**:
- Required context fields justified by business purpose
- Optional context fields evaluated for necessity
- "Just in case" context collection prohibited
- Context retention defined and enforced

**Validation Gates**:
- [ ] Each context field justified with business purpose
- [ ] Excessive fields eliminated
- [ ] Retention periods defined and enforced
- [ ] Collection necessity review performed

**MinJus Alignment**:
- Directly supports P1 (Privacy by Design) - Minimization is privacy-by-design
- Extends P3 (Archival Integrity) - Retention serves archival purpose

---

## VI. Context Quality Principles

### C12: Context Validation

**Principle Statement**:
Contextual metadata MUST be validated for completeness, consistency, and correctness at capture and on-change.

**Rationale**:
Invalid context is misleading and can cause incorrect decisions. Validation ensures context quality and reliability.

**Implications**:
- Required context fields enforced at capture
- Context values validated against allowable ranges/references
- Cross-field consistency checks performed
- Validation rules documented and versioned

**Validation Gates**:
- [ ] Validation rules defined for each context field
- [ ] Required field enforcement implemented
- [ ] Consistency checks performed
- [ ] Validation feedback provided to users

**MinJus Alignment**:
- Extends P10 (Observability) - Validation IS observability
- Supports P3 (Archival Integrity) - Quality ensures archival value

---

### C13: Context Freshness

**Principle Statement**:
Contextual metadata MUST be maintained to reflect current reality. Stale context MUST be identified and either updated or archived.

**Rationale**:
Stale context (e.g., "current owner" who left the organization) misleads and causes errors. Freshness ensures context reliability.

**Implications**:
- Timestamps recorded for all context capture
- Context aging monitored and reported
- Update workflows defined for time-sensitive context
- Historical context preserved on change (versioning)

**Validation Gates**:
- [ ] Timestamp strategy implemented
- [ ] Freshness monitoring defined
- [ ] Update workflows documented
- [ ] Historical context preserved

**MinJus Alignment**:
- Supports P3 (Archival Integrity) - Freshness captures provenance
- Extends P10 (Observability) - Timestamps enable audit

---

## VII. Context Utilization Principles

### C14: Context-First Search

**Principle Statement**:
Search and discovery systems MUST prioritize contextual metadata in relevance ranking, enabling users to find information by context before content.

**Rationale**:
Government information is often retrieved by context (case number, policy domain, creator) rather than content keywords. Context-first search aligns with real-world usage.

**Implications**:
- Search UI MUST include context filters
- Relevance algorithm weights context appropriately
- Context facets MUST be visible and navigable
- Popular context queries identified and optimized

**Validation Gates**:
- [ ] Context filters implemented in search UI
- [ ] Relevance algorithm weights context
- [ ] Faceted navigation includes context
- [ ] Query analytics validate context-first patterns

**MinJus Alignment**:
- Supports P2 (Open Government) - Context enables Woo disclosure
- Extends P5 (Domain-Driven) - Domain IS primary context

---

### C15: Context Visualization

**Principle Statement**:
Contextual metadata MUST be presented to users in intuitive, visual ways that communicate relationships and provenance effectively.

**Rationale**:
Raw context metadata is difficult to interpret. Visualization helps users understand relationships, provenance, and meaning.

**Implications**:
- Context visualization for entity relationships
- Provenance timeline visualization
- Domain context graphical representation
- Context comparison and difference views

**Validation Gates**:
- [ ] Relationship visualization implemented
- [ ] Provenance timeline available
- [ ] Domain context visualized
- [ ] Usability testing validates effectiveness

**MinJus Alignment**:
- Supports P2 (Open Government) - Visualization enables transparency
- Extends P9 (Accessibility) - Visual context aids understanding

---

## VIII. Context Governance Principles

### C16: Context Stewardship

**Principle Statement**:
Each context type MUST have an identified steward responsible for defining, maintaining, and evolving that context definition.

**Rationale**:
Context without ownership becomes ambiguous or obsolete. Stewardship ensures context remains relevant and valuable.

**Implications**:
- Context stewards identified and documented
- Steward responsibilities defined (definition, validation, evolution)
- Steward consultation required for context changes
- Context change review process established

**Validation Gates**:
- [ ] All context types have stewards
- [ ] Steward responsibilities documented
- [ ] Change review process defined
- [ ] Steward consultation recorded

**MinJus Alignment**:
- Supports P8 (Interoperability) - Stewardship enables consistency
- Extends P10 (Observability) - Stewards ensure context quality

---

### C17: Context Evolution

**Principle Statement**:
Context model and field definitions MUST evolve through structured change management, maintaining backward compatibility where possible.

**Rationale**:
Context needs evolve with business and regulatory changes. Structured evolution prevents breaking changes and maintains continuity.

**Implications**:
- Context model versioning implemented
- Backward compatibility assessed for changes
- Migration strategy defined for breaking changes
- Deprecation process for obsolete context

**Validation Gates**:
- [ ] Versioning strategy implemented
- [ ] Backward compatibility analysis performed
- [ ] Migration plans defined
- [ ] Deprecation process documented

**MinJus Alignment**:
- Supports P8 (Interoperability) - Versioning enables coexistence
- Extends P13 (Maintainability) - Evolution maintains system health

---

## Principle Conflicts and Resolution

| Conflict | Principles | Resolution | Rationale |
|----------|------------|------------|-----------|
| Completeness vs. Overhead | C2 (Minimal Overhead) vs. C6 (Capture at Source) | Layering (C4) - Core required, extended optional | Balance mandatory capture with optional depth |
| Context vs. Privacy | C6 (Capture at Source) vs. C10 (Context IS Personal Data) | Classification - Only necessary context captured | Privacy overrides comprehensive capture |
| Inference vs. Accuracy | C7 (Inference as Last Resort) vs. C12 (Context Validation) | Labeling - Inferred context marked as such | Transparency enables risk assessment |
| Freshness vs. Provenance | C13 (Context Freshness) vs. C3 (Archival Integrity) | Versioning - Both current and historical preserved | Both requirements served through versioning |

---

## Compliance Mapping

| Principle | Woo | AVG | Archiefwet | WCAG 2.1 | MinJus P1-P10 |
|-----------|-----|-----|------------|----------|--------------|
| C1: Context by Design | ✓ | ✓ | ✓ | | P1, P5 |
| C2: Minimal Overhead | | | | | P6 |
| C3: Explicit Model | ✓ | | ✓ | | P5, P8 |
| C4: Context Layering | | | ✓ | | P1, P3 |
| C5: Context-Entegrity | | | ✓ | | P3, P10 |
| C6: Capture at Source | ✓ | | ✓ | | P5 |
| C7: Inference Last Resort | | ✓ | | | P6, P10 |
| C8: Context Co-Location | | | ✓ | | P3, P4 |
| C9: Context Indexing | ✓ | | | | P2, P5 |
| C10: Context IS Personal Data | | ✓ | | | P1, P7 |
| C11: Context Minimization | | ✓ | | | P1 |
| C12: Context Validation | | ✓ | ✓ | | P10 |
| C13: Context Freshness | | | ✓ | | P3, P10 |
| C14: Context-First Search | ✓ | | | ✓ | P2, P9 |
| C15: Context Visualization | ✓ | | | ✓ | P2, P9 |
| C16: Context Stewardship | ✓ | ✓ | ✓ | | P5, P8 |
| C17: Context Evolution | | | ✓ | | P8, P13 |

---

## Decision Framework

When making context-aware architecture decisions, apply principles in priority order:

1. **Legal Compliance**: C10 (Context IS Personal Data), C11 (Context Minimization) — Non-negotiable
2. **Foundation**: C1 (Context by Design), C5 (Context-Entegrity) — Critical for success
3. **User Experience**: C2 (Minimal Overhead), C15 (Context Visualization) — Adoption depends on this
4. **Data Quality**: C3 (Explicit Model), C12 (Context Validation), C13 (Context Freshness) — Essential for value
5. **Operations**: C8 (Context Co-Location), C9 (Context Indexing), C16 (Context Stewardship) — Enabling factors

**Conflict Resolution**: If principles conflict, Legal > Foundation > User Experience > Data Quality > Operations.

---

## Exception Process

### Requesting Context Principle Exceptions

Context principles are mandatory unless a documented exception is approved by the Enterprise Architecture Board.

**Valid Exception Reasons**:

- Technical constraints that prevent compliance
- Regulatory or legal requirements requiring deviation
- Transitional state during migration
- Pilot/proof-of-concept with defined end date

**Exception Request Requirements**:

- [ ] Justification with business/technical rationale
- [ ] Alternative approach and compensating controls
- [ ] Risk assessment and mitigation plan
- [ ] Expiration date (exceptions are time-bound)
- [ ] Impact assessment on Data→Informatie transformation

**Approval Process**:

1. Submit exception request to Enterprise Architecture team
2. Review by Context Steward Council
3. CIO approval for exceptions to critical principles
4. Document exception in project architecture documentation
5. Regular review of exceptions (quarterly)

---

## Principle Summary Checklist

| Principle | Category | Criticality | Validation |
|-----------|----------|-------------|------------|
| C1: Context by Design | Foundational | CRITICAL | Requirements review |
| C2: Minimal Overhead | Foundational | HIGH | Time measurement |
| C3: Explicit Model | Structure | CRITICAL | Model documentation |
| C4: Context Layering | Structure | HIGH | Layer definition |
| C5: Context-Entegrity | Structure | CRITICAL | Referential checks |
| C6: Capture at Source | Capture | HIGH | Workflow review |
| C7: Inference Last Resort | Capture | MEDIUM | Labeling verification |
| C8: Context Co-Location | Storage | HIGH | Data model review |
| C9: Context Indexing | Storage | MEDIUM | Query performance |
| C10: Context IS Personal Data | Privacy | CRITICAL | DPIA, privacy review |
| C11: Context Minimization | Privacy | CRITICAL | Necessity review |
| C12: Context Validation | Quality | HIGH | Validation rules |
| C13: Context Freshness | Quality | MEDIUM | Timestamp checks |
| C14: Context-First Search | Utilization | HIGH | Search analytics |
| C15: Context Visualization | Utilization | MEDIUM | Usability testing |
| C16: Context Stewardship | Governance | HIGH | Steward assignment |
| C17: Context Evolution | Governance | MEDIUM | Change management |

---

## External References

| Document | Type | Source | Key Extractions | Path |
|----------|------|--------|-----------------|------|
| Data versus informatie en het belang van context | Presentatie | MinJus | Data→Informatie transformatie model | external/README.md |
| Strategische laag (Capabilities) | Document | MinJus | Domein context structure | external/README.md |
| Motivatielaag | Document | MinJus | Driver analysis for context | external/README.md |
| ARC-000-PRIN-v1.0 | Principles | Global | Base IOU-Modern principles | /projects/000-global/ |
| ARC-003-STKE-v1.0 | Stakeholders | Project 003 | Stakeholder drivers and goals | /projects/003-context-aware-data/ |
| Algemene verordening gegevensbescherming | Wet | EU | Privacy requirements for context | https://gdpr-info.eu/ |
| Archiefwet 1995 | Wet | NL | Archival requirements for context | https://wetten.overheid.nl/ |
| Wet open overheid | Wet | NL | Transparency requirements | https://wetten.overheid.nl/ |

---

**Generated by**: ArcKit `/arckit.principles` command
**Generated on**: 2026-04-19
**ArcKit Version**: 4.3.1
**Project**: Context-Aware Data Architecture (Project 003)
**AI Model**: claude-opus-4-7
