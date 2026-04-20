# Stakeholder Drivers & Goals Analysis: Metadata Registry Service

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Command**: `/arckit.stakeholders`

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-002-STKE-v1.0 |
| **Document Type** | Stakeholder Drivers & Goals Analysis |
| **Project** | Metadata Registry Service (Project 002) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.0 |
| **Created Date** | 2026-04-19 |
| **Last Modified** | 2026-04-19 |
| **Review Cycle** | Quarterly |
| **Next Review Date** | 2026-07-19 |
| **Owner** | Product Owner, IOU-Modern |
| **Reviewed By** | PENDING |
| **Approved By** | PENDING |
| **Distribution** | Project Team, Architecture Team, DPO, Woo Officers, Domain Owners, CIO Office |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---------|------|--------|---------|-------------|---------------|
| 1.0 | 2026-04-19 | ArcKit AI | Initial creation from `/arckit:stakeholders` command | PENDING | PENDING |

---

## Executive Summary

### Purpose

This document identifies key stakeholders for the Metadata Registry Service, their underlying drivers (motivations, concerns, needs), how these drivers manifest into goals, and the measurable outcomes that will satisfy those goals. This analysis ensures stakeholder alignment for a Dutch Government shared service implementing the Metamodel GGHH V2 specification and BSW architecture principles.

### Key Findings

The Metadata Registry Service serves as a critical shared service for Dutch government organizations, with **11 primary stakeholder groups** spanning technical, business, compliance, and regulatory domains. Key findings:

- **Compliance drivers dominate**: 6 of 11 primary drivers are compliance-related (Woo, AVG, Archiefwet, BSW)
- **Tension exists**: Speed of innovation (CIO) vs. Risk avoidance (DPO/Security) requires active management
- **Multi-tenancy complexity**: Different organizational needs require careful governance
- **Context preservation critical**: Per Dutch Ministry of Justice guidance, maintaining context (volledigheid, samenhang, herleidbaarheid) is a primary concern for information managers

### Critical Success Factors

- **Regulatory compliance**: Woo, AVG, and Archiefwet compliance is non-negotiable for government operations
- **BSW alignment**: Full BSW architecture implementation required for future zaak/dossierbeheer
- **Multi-tenancy with isolation**: Organizations must have confidence in data isolation while enabling collaboration
- **Context preservation**: Metadata must capture complete context of creation and use
- **Open-source sovereignty**: Technology stack must align with government digital sovereignty principles

### Stakeholder Alignment Score

**Overall Alignment**: MEDIUM (65%)

The project has strong alignment on compliance requirements but significant tension between:
- Innovation speed vs. risk mitigation
- Centralized control vs. domain autonomy
- Standardization vs. flexibility

Active conflict resolution and communication will be required to maintain alignment through implementation.

---

## Stakeholder Identification

### Internal Stakeholders

| Stakeholder | Role/Department | Influence | Interest | Engagement Strategy |
|-------------|----------------|-----------|----------|---------------------|
| CIO (Chief Information Officer) | Executive, Digital Strategy | HIGH | HIGH | Steering Committee, quarterly reviews |
| DPO (Data Protection Officer) | Legal, Privacy Compliance | HIGH | HIGH | Active involvement, DPIA approval |
| Enterprise Architect | Architecture, Standards | HIGH | HIGH | Design authority, architecture review |
| Woo Officers | Legal, Transparency | HIGH | HIGH | Workflow design, publication approval |
| Information Managers | Operations, Records Management | MEDIUM | HIGH | Requirements input, user acceptance |
| Domain Owners | Business Units | MEDIUM | HIGH | Metadata consumers, collaboration scenarios |
| Security Officer | Information Security | HIGH | MEDIUM | Security architecture, threat modeling |
| Product Owner | IOU-Modern Project | MEDIUM | HIGH | Day-to-day decisions, prioritization |
| DevOps Lead | IT Operations | MEDIUM | HIGH | Deployment, monitoring, support |
| Integration Developers | Technical Teams | LOW | HIGH | API consumers, documentation |

### External Stakeholders

| Stakeholder | Organization | Relationship | Influence | Interest |
|-------------|--------------|--------------|-----------|----------|
| Autoriteit Persoonsgegevens (AP) | Privacy Regulator | Oversight | HIGH | MEDIUM | AVG compliance enforcement |
| Ministerie van BZK | Interior & Kingdom Relations | Standards Owner | HIGH | MEDIUM | BSW standards, digital government |
| Nationaal Archief | Cultural Heritage | Oversight | MEDIUM | MEDIUM | Archiefwet compliance, CDD+ integration |
| Geonovum | Digital Standards | Standards Body | MEDIUM | LOW | TOOI/MDTO standards, interoperability |
| Overheidsorganisaties | Government Organizations | Service Consumers | MEDIUM | HIGH | Multi-tenant service users |

### Stakeholder Power-Interest Grid

```text
                          INTEREST
              Low                         High
        ┌─────────────────────┬─────────────────────┐
        │                     │                     │
        │   KEEP SATISFIED    │   MANAGE CLOSELY    │
   High │                     │                     │
        │  • Nationaal Archief │  • CIO             │
        │  • Geonovum          │  • DPO             │
        │  • Security Officer  │  • Woo Officers    │
 P      │                     │  • Enterprise Arch  │
 O      ├─────────────────────┼─────────────────────┤
 W      │                     │                     │
 E      │      MONITOR        │    KEEP INFORMED    │
 R      │                     │                     │
   Low  │  • AP (after initial) │  • Information Mgrs│
        │                     │  • Domain Owners    │
        │                     │  • Product Owner    │
        │                     │  • DevOps Lead      │
        │                     │  • Integration Devs │
        └─────────────────────┴─────────────────────┘
```

| Stakeholder | Power | Interest | Quadrant | Engagement Strategy |
|-------------|-------|----------|----------|---------------------|
| CIO | HIGH | HIGH | Manage Closely | Steering committee, quarterly business reviews |
| DPO | HIGH | HIGH | Manage Closely | DPIA co-creation, AVG approval gates |
| Enterprise Architect | HIGH | HIGH | Manage Closely | Architecture reviews, standards compliance |
| Woo Officers | HIGH | HIGH | Manage Closely | Workflow design, publication approval |
| Security Officer | HIGH | MEDIUM | Keep Satisfied | Security architecture reviews, threat modeling |
| Nationaal Archief | MEDIUM | MEDIUM | Keep Satisfied | CDD+ integration, archival requirements |
| Geonovum | MEDIUM | LOW | Keep Satisfied | TOOI/MDTO standards alignment |
| Information Managers | MEDIUM | HIGH | Keep Informed | Requirements gathering, UAT |
| Domain Owners | MEDIUM | HIGH | Keep Informed | Collaboration scenarios, search requirements |
| Product Owner | MEDIUM | HIGH | Keep Informed | Daily prioritization, sprint planning |
| DevOps Lead | MEDIUM | HIGH | Keep Informed | Deployment planning, operational readiness |
| Integration Developers | LOW | HIGH | Keep Informed | API documentation, developer support |
| Autoriteit Persoonsgegevens | HIGH | MEDIUM | Keep Satisfied | Compliance reporting, incident notification |

**Quadrant Interpretation:**

- **Manage Closely** (High Power, High Interest): Key decision-makers requiring active engagement and regular communication
- **Keep Satisfied** (High Power, Low/Medium Interest): Influential stakeholders needing periodic updates to maintain support
- **Keep Informed** (Low/Medium Power, High Interest): Engaged stakeholders needing regular communication and involvement
- **Monitor** (Low Power, Low Interest): Minimal engagement required, monitor for changes

---

## Stakeholder Drivers Analysis

### SD-1: CIO - Digital Innovation & Sovereignty

**Stakeholder**: Chief Information Officer, Rijksoverheid (National Government)

**Driver Category**: STRATEGIC | FINANCIAL

**Driver Statement**: "Modernize government information management with sovereign, open-source technology while reducing operational costs through shared service delivery across 500+ government organizations."

**Context & Background**:
The CIO is under pressure from the Ministerie van Binnenlandse Zaken en Koninkrijksrelaties (BZK) to deliver on the Nationale Digitaliseringsstrategie 2024-2027. Key pressures include:
- **Digital Sovereignty Mandate**: Reduce dependence on proprietary vendors (Microsoft, Oracle) by 60% by 2027
- **Cost Reduction**: Shared services must deliver 30% operational cost savings vs. individual implementations
- **BSW Readiness**: Government organizations must be ready for BSW (Beter Samenwerken) architecture by 2028
- **Innovation Acceleration**: New services must launch 50% faster than current 18-month average

**Driver Intensity**: CRITICAL

**Enablers**:
- Metamodel GGHH V2 specification provides standard data model
- Existing IOU-Modern platform as foundation
- Growing Rust ecosystem in Dutch government
- BSW architecture guidance from Geonovum

**Blockers**:
- Resistance to change from individual ministry IT departments
- Skills gap in Rust/ArangoDB within government
- Budget constraints for initial investment
- Legacy system integration complexity

**Related Stakeholders**: Enterprise Architect (alignment), Finance (budget), Ministry BZK (political support)

---

### SD-2: DPO - AVG Compliance & Accountability

**Stakeholder**: Data Protection Officer (Privacy Officer)

**Driver Category**: COMPLIANCE | RISK

**Driver Statement**: "Ensure 100% AVG/GDPR compliance with complete audit trails, data minimization, and demonstrable accountability to avoid AP enforcement actions and fines."

**Context & Background**:
The DPO is accountable to the Autoriteit Persoonsgegevens (Dutch Data Protection Authority) for AVG compliance. Recent AP enforcement actions have resulted in:
- Increased scrutiny of government metadata systems (3 investigations in 2025)
- Fines up to €20 million for non-compliance
- Mandatory DPIA for systems processing personal data on a large scale
- Requirement for separate PII access logging (AVG Article 30)

The Metadata Registry processes potentially sensitive information through:
- Citizen data referenced in zaak/dossier relationships
- Woo publication decisions affecting personal data
- AI enrichment processing citizen information

**Driver Intensity**: CRITICAL

**Enablers**:
- MetadataAuditLog entity tracks all mutations
- PersoonsgebondenTrait for PII classification
- Retention periods per Archiefwet
- Row-Level Security for organization isolation

**Blockers**:
- Context loss between data collection and use (implicit vs. explicit consent)
- Automated deletion after retention periods not yet implemented
- PII access logging design incomplete (separate audit log required)
- DPIA not yet completed

**Related Stakeholders**: Security Officer (controls), Woo Officers (publication conflicts), Information Managers (retention)

---

### SD-3: Woo Officers - Transparency & Publication

**Stakeholder**: Wet open overheid (Woo) Officers

**Driver Category**: COMPLIANCE | OPERATIONAL

**Driver Statement**: "Achieve 100% Woo compliance through automated relevance assessment, human approval workflows, and complete publication audit trails to demonstrate government transparency."

**Context & Background**:
Woo (Wet open overheid, replacing Wob in 2022) requires proactive publication of government information. Key requirements:
- **Automatic Relevance Assessment**: All Informatieobjecten must be assessed for Woo relevance
- **Human Approval Required**: All Woo-relevant documents require human approval before publication
- **Publication Audit Trail**: Complete record of publication decisions and refusals with grounds
- **10-Day Deadline**: Publication requests must be processed within 10 working days
- **Partial Publication Refusal**: Must document specific grounds for partial refusal

Recent Woo violations have resulted in:
- Parliamentary questions
- Negative media coverage
- Legal challenges from citizens and journalists
- Ministerial embarrassment

**Driver Intensity**: CRITICAL

**Enablers**:
- WooPublicatie entity with complete workflow states
- Automatic relevance assessment via is_woo_relevant flag
- Human approval workflow with approvers tracked
- Refusal grounds documentation (redactie field)

**Blockers**:
- PII detection accuracy not yet proven
- Human validation capacity (workflow volume unknown)
- Integration with Woo publication portal not implemented
- Publication deadline (10 days) at risk with manual validation

**Related Stakeholders**: DPO (PII review), Information Managers (document classification), Domain Owners (content approval)

---

### SD-4: Information Managers - Context Preservation

**Stakeholder**: Informatiebeheerders (Records Managers)

**Driver Category**: OPERATIONAL | COMPLIANCE

**Driver Statement**: "Maintain complete context (volledigheid, samenhang, herleidbaarheid) for all government information to ensure duurzame toegankelijkheid and Archiefwet compliance."

**Context & Background**:
Per Dutch Ministry of Justice guidance *"Data versus informatie en het belang van context"*:
- **Volledigheid** (Completeness): All relevant data included with context of creation
- **Samenhang** (Coherence): Data interpreted together, not in isolation; relationships preserved
- **Herleidbaarheid** (Traceability): Source and context preserved for audit trails

Information Managers are concerned that IT systems strip away context, making data:
- **Unreliable**: Without context, incorrect conclusions drawn
- **Unjust**: Decisions made without complete information
- **Non-compliant**: Archiefwet requires provenance and context

Key challenges:
- Current metadata scattered across systems with no relationships
- Loss of "why was this created" context
- Difficulty proving data provenance during audits
- Inability to trace decisions to source information

**Driver Intensity**: HIGH

**Enablers**:
- ContextMetadata entity (zaak_id, werkproces, domein, labels)
- Graph relationships preserving semantic connections
- MetadataAuditLog tracking who/what/when/why
- Metadata inheritance from zaak/dossier
- GitOps maintaining version history

**Blockers**:
- Context of creation fields implicit, not explicit in design
- Business rules governing data not documented
- IT simplification risks not explicitly addressed
- No formal training on context capture requirements

**Related Stakeholders**: Nationaal Archief (archival standards), Domain Owners (context experts), Enterprise Architect (context preservation design)

---

### SD-5: Enterprise Architect - BSW Alignment

**Stakeholder**: Enterprise Architect, Rijksoverheid

**Driver Category**: STRATEGIC | COMPLIANCE

**Driver Statement**: "Ensure full BSW (Beter Samenwerken) architecture alignment to enable government-wide zaak/dossierbeheer and ketensamenwerking across organizational boundaries."

**Context & Background**:
BSW (Beter Samenwerken) is the government-mandated architecture for zaak/dossierbeheer. Key principles:
1. **Informatieobject Centricity**: Everything is an informatieobject (dataobject + metadata)
2. **Dynamic vs Persistent**: Distinguish between "in bewerking" and "gepersisteerd" states
3. **Informatieobject Catalogus**: Metadata + location references (not content storage)
4. **Context-Aware Search**: User context + information origin context combined
5. **Object-Level Authorization**: Collaboration requires grants at individual object level
6. **Metadata Inheritance**: Zaak/dossier metadata inherited by contained objects
7. **AI Enrichment with Validation**: Human validation required before trusted use

Timeline pressure:
- **2026**: Pilot implementations required
- **2027**: 50% of government organizations compliant
- **2028**: 100% compliance mandatory

**Driver Intensity**: CRITICAL

**Enablers**:
- ADR-004: Full BSW architecture alignment decision
- Informatieobject entity with status field
- InformatieobjectRecht for object-level authorization
- Context-aware search algorithm
- AIEnrichment entity with human validation

**Blockers**:
- BSW complexity increases implementation time
- Multi-caretaker scenarios not fully designed
- Workflow integration incomplete
- Organization resistance to new paradigm

**Related Stakeholders**: CIO (BSW readiness), Geonovum (standards), Information Managers (operationalization)

---

### SD-6: Domain Owners - Metadata Discovery

**Stakeholder**: Eigenaren (Domain Owners) - Zaak/Project/Beleid/Expertise

**Driver Category**: OPERATIONAL | CUSTOMER

**Driver Statement**: "Find relevant information quickly through context-aware search that understands my active zaak, domain, and organizational context without requiring technical query skills."

**Context & Background**:
Domain Owners (business owners of Zaak, Project, Beleid, Expertise domains) need to:
- **Find information**: Locate relevant documents, decisions, precedents for their domain
- **Understand context**: Know why information was created, by whom, for what purpose
- **Collaborate**: Share information with other domains while maintaining access control
- **Approve changes**: Validate metadata changes affecting their domain

Current pain points:
- Keyword search returns irrelevant results (no context understanding)
- Cannot find information from other domains without knowing their structure
- Unclear approval processes for metadata changes
- Limited visibility into who is accessing their information

**Driver Intensity**: HIGH

**Enablers**:
- Context-aware search algorithm combining user + information context
- Object-level authorization for collaboration
- Metadata inheritance from zaak/dossier
- Graph traversal for cross-domain discovery

**Blockers**:
- Search algorithm complexity (requires user context capture)
- Multi-tenancy isolation preventing cross-domain discovery
- Admin UI not optimized for domain owner workflows
- No formal approval workflow defined

**Related Stakeholders**: Information Managers (metadata quality), Integration Developers (API access), Security Officer (authorization)

---

### SD-7: Security Officer - Data Isolation

**Stakeholder**: Information Security Officer

**Driver Category**: COMPLIANCE | RISK

**Driver Statement**: "Enforce organization-level data isolation through Row-Level Security while enabling authorized cross-organization collaboration for ketensamenwerking scenarios."

**Context & Background**:
The Metadata Registry serves 500+ government organizations with:
- **Confidentiality Requirements**: Different security levels per organization
- **Data Isolation Mandate**: Organizations must not see each other's data without authorization
- **Collaboration Needs**: Cross-organization zaak/dossier scenarios require selective access
- **AVG Requirements**: Row-Level Security required for AVG Article 32 compliance

Security concerns:
- Data leakage between organizations
- Unauthorized access via API
- Insufficient audit trail for PII access
- Missing authentication/authorization in current implementation
- No MFA for administrative accounts

**Driver Intensity**: HIGH

**Enablers**:
- InformatieobjectRecht entity for object-level grants
- Organizational isolation via organisatie_id
- OAuth 2.0/OIDC authentication planned
- RBAC with roles defined

**Blockers**:
- Authentication/authorization not implemented (HLD BLOCKING-01)
- Row-Level Security not visible in implementation (HLD BLOCKING-03)
- No MFA requirements defined
- PII access logging incomplete (NFR-S-014)

**Related Stakeholders**: DPO (PII protection), CIO (security vs. usability), DevOps Lead (implementation)

---

### SD-8: DevOps Lead - Operational Excellence

**Stakeholder**: DevOps Lead, IT Operations

**Driver Category**: OPERATIONAL | RISK

**Driver Statement**: "Achieve 99.5% uptime with <4 hour RTO and <1 hour RPO while maintaining comprehensive observability for troubleshooting and compliance reporting."

**Context & Background**:
The Metadata Registry is a critical shared service for 500+ organizations. SLA requirements:
- **99.5% uptime** (excluding planned maintenance) = ~21 hours downtime/year allowed
- **4-hour RTO**: Service must recover within 4 hours of disaster
- **1-hour RPO**: Maximum 1 hour data loss acceptable
- **Observability**: Logs, metrics, traces required for compliance (AVG Article 30)

Current concerns:
- No disaster recovery strategy documented
- Metrics and distributed tracing incomplete
- No automated backup verification
- No incident response runbooks
- Multi-region deployment not defined

**Driver Intensity**: MEDIUM

**Enablers**:
- Open-source stack (no vendor lock-in)
- ArangoDB clustering support
- GitOps for version control and recovery

**Blockers**:
- No disaster recovery strategy (HLD BLOCKING-05)
- Observability incomplete (HLD BLOCKING-04)
- No backup automation
- No incident response procedures

**Related Stakeholders**: CIO (SLA compliance), Enterprise Architect (resilience patterns), Security Officer (incident response)

---

### SD-9: Product Owner - Delivery Velocity

**Stakeholder**: Product Owner, IOU-Modern

**Driver Category**: OPERATIONAL | STRATEGIC

**Driver Statement**: "Deliver metadata registry capabilities incrementally to demonstrate value quickly while maintaining quality and addressing stakeholder feedback."

**Context & Background**:
The Product Owner balances competing demands:
- **Speed**: CIO wants fast delivery (18-month pilot deadline)
- **Quality**: DPO/Security require compliance before deployment
- **Scope**: Enterprise Architect wants full BSW alignment
- **Resources**: Limited development team (Rust expertise scarce)

Priority decisions:
- Phase 1: Core GGHH V2 entities + basic Woo workflow
- Phase 2: BSW specific features (catalogus, context search)
- Phase 3: Advanced features (AI enrichment, multi-caretaker)

**Driver Intensity**: HIGH

**Enablers**:
- Modular architecture (6 Rust crates)
- Clear requirements prioritization
- Stakeholder engagement for feedback

**Blockers**:
- Skills gap in Rust/ArangoDB
- HLD blocking conditions must be addressed first
- Competing priorities from stakeholders
- Unclear acceptance criteria for BSW features

**Related Stakeholders**: CIO (delivery speed), Enterprise Architect (scope), DevOps Lead (deployment readiness)

---

### SD-10: Integration Developers - API Usability

**Stakeholder**: Integration Developers (API Consumers)

**Driver Category**: OPERATIONAL | TECHNICAL

**Driver Statement**: "Consume metadata registry APIs reliably with clear documentation, predictable responses, and comprehensive examples to integrate government systems efficiently."

**Context & Background**:
Integration Developers build systems that consume metadata registry APIs:
- **Zaaksystems**: Need case metadata and relationships
- **Document Management**: Need information object catalogus
- **Search Applications**: Need context-aware search
- **Woo Portals**: Need publication workflows

Developer concerns:
- API inconsistencies across endpoints
- Missing or unclear documentation
- Rate limiting and quota enforcement unclear
- No error handling guidance
- Authentication/OIDC integration complexity

**Driver Intensity**: MEDIUM

**Enablers**:
- REST v2 and GraphQL APIs defined
- OpenAPI specification planned
- Example requests/responses documented

**Blockers**:
- Many V2 endpoints return TODO/mock responses
- Authentication not implemented
- No rate limiting strategy documented
- No SDK or client libraries

**Related Stakeholders**: Enterprise Architect (API standards), Product Owner (backlog prioritization), DevOps Lead (API monitoring)

---

### SD-11: Nationaal Archief - Archival Integrity

**Stakeholder**: Nationaal Archief (National Archives)

**Driver Category**: COMPLIANCE | OPERATIONAL

**Driver Statement**: "Ensure government records are preserved according to Archiefwet requirements with complete provenance, proper retention periods, and CDD+ integration for long-term archival."

**Context & Background**:
The Archiefwet 1995 mandates:
- **Retention Periods**: 1, 5, 10, or 20 years depending on document type
- **Provenance**: Complete record of who created/modified records and why
- **Transfer to CDD+**: Records transferred to Nationaal Archief after retention
- **Audit Trail**: All changes logged for 7+ years
- **Deletion Control**: Automated deletion only after retention expires

Archival concerns:
- Informatieobject status tracking (dynamisch → gepersist → gearchiveerd)
- Bewaartermijn entity properly linked
- CDD+ integration only stubbed (NotImplemented)
- No automated deletion after retention expires

**Driver Intensity**: MEDIUM

**Enablers**:
- Bewaartermijn entity with retention periods
- MetadataAuditLog for provenance
- InformatieobjectStatus.Gearchiveerd state
- OpslagType.Gearchiveerd status

**Blockers**:
- CDD+ integration incomplete (HLD ADVISORY-04)
- Automated archival/deletion not implemented
- No archival validation workflows

**Related Stakeholders**: Information Managers (day-to-day archival), DPO (retention compliance), DevOps Lead (CDD+ operations)

---

## Driver-to-Goal Mapping

### Goal G-1: Achieve Full Regulatory Compliance by Q4 2026

**Derived From Drivers**: SD-2 (DPO-AVG), SD-3 (Woo Officers-Woo), SD-11 (Nationaal Archief-Archiefwet)

**Goal Owner**: DPO (with Woo Officers and Nationaal Archief)

**Goal Statement**: "Achieve 100% compliance with AVG/GDPR, Woo, and Archiefwet requirements through automated workflows, complete audit trails, and demonstrable accountability by Q4 2026."

**Why This Matters**: Non-compliance risks AP enforcement, fines up to €20 million, parliamentary questions, and legal challenges. This is the foundational requirement for all government operations.

**Success Metrics**:

- **Primary Metric**: 100% of compliance requirements implemented and validated
- **Secondary Metrics**:
  - DPIA approved by DPO: Q2 2026
  - Zero AVG violations: 0 incidents in first 12 months
  - Woo publication SLA: 95% within 10-day deadline
  - Archiefwet retention: 100% correct retention periods applied

**Baseline**:
- DPIA: Not started
- Woo workflow: Manual process, 40% meet deadline
- Retention tracking: Inconsistent across systems

**Target**:
- DPIA: Approved by DPO (Q2 2026)
- Woo automation: 90% automated, 10-day SLA met
- Retention: 100% automated correct application

**Measurement Method**:
- Compliance assessment checklist (quarterly)
- AP reporting (ongoing)
- Woo publication metrics (dashboard)
- Retention audit (annual)

**Dependencies**:
- HLD blocking conditions addressed (auth/z, RLS, observability)
- DPIA completed and approved
- CDD+ integration for archival

**Risks to Achievement**:
- DPIA approval delayed
- Authentication/authorization implementation takes longer than estimated
- Woo publication volume exceeds human validation capacity

---

### Goal G-2: Enable BSW Architecture Adoption by Q2 2027

**Derived From Drivers**: SD-5 (Enterprise Architect-BSW), SD-4 (Information Managers-Context)

**Goal Owner**: Enterprise Architect

**Goal Statement**: "Deliver full BSW architecture alignment enabling government-wide zaak/dossierbeheer and ketensamenwerking with information object-centric design, context-aware search, and object-level authorization by Q2 2027."

**Why This Matters**: BSW is government-mandated for zaak/dossierbeheer with 100% compliance required by 2028. Organizations cannot collaborate without BSW-aligned metadata services.

**Success Metrics**:

- **Primary Metric**: 100% of BSW principles implemented and operational
- **Secondary Metrics**:
  - BSW entity coverage: All Phase 1-8 entities implemented
  - Context-aware search accuracy: >80% relevance for user queries
  - Object-level authorization: <50ms access check latency
  - Metadata inheritance: 100% of zaak/dossier metadata inherited

**Baseline**:
- BSW implementation: 0% (starting from scratch)
- Current systems: Document-centric, no information object abstraction

**Target**:
- Phase 1-8 entities: 100% implemented by Q2 2027
- Pilot organizations: 5 government organizations using BSW features
- Context search: >80% relevance achieved

**Measurement Method**:
- BSW compliance checklist (Geonovum standard)
- User acceptance testing (pilot organizations)
- Search relevance testing (domain owners)
- Performance testing (authorization latency)

**Dependencies**:
- ADR-004 (BSW alignment) fully implemented
- Multi-caretaker scenarios designed
- Workflow integration complete

**Risks to Achievement**:
- BSW complexity underestimated
- Organization resistance to new paradigm
- Skills gap in graph database operations

---

### Goal G-3: Deliver Cost-Effective Shared Service by Q3 2026

**Derived From Drivers**: SD-1 (CIO-Cost Reduction), SD-8 (DevOps-Operational Excellence)

**Goal Owner**: CIO

**Goal Statement**: "Launch metadata registry as shared service achieving 30% operational cost savings vs. individual implementations while maintaining 99.5% uptime and supporting 100+ organizations by Q3 2026."

**Why This Matters**: The business case for shared services depends on cost savings. Individual ministry implementations would cost €50M total; shared service target is €35M (30% savings).

**Success Metrics**:

- **Primary Metric**: €15M cost savings vs. baseline (30% reduction)
- **Secondary Metrics**:
  - Organizations onboarded: 100+ by Q3 2026, 500+ by 2028
  - Uptime: 99.5% (excluding planned maintenance)
  - RTO: <4 hours for disaster recovery
  - RPO: <1 hour data loss
  - Onboarding time: <2 weeks per organization

**Baseline**:
- Current approach: Individual implementations at €1M per ministry
- Onboarding: Manual processes, 6-8 weeks per organization

**Target**:
- Shared service: €500k per 100 organizations (€5k per org)
- Onboarding: Self-service, <2 weeks per organization
- Disaster recovery: Automated failover within 4 hours

**Measurement Method**:
- Financial reporting (quarterly)
- SLA monitoring (real-time dashboard)
- Onboarding metrics (organization registration)
- Cost per organization (allocated cost model)

**Dependencies**:
- Multi-tenancy fully implemented
- Disaster recovery strategy complete
- Automated onboarding workflows
- Observability for capacity planning

**Risks to Achievement**:
- Cost savings target too aggressive
- Onboarding complexity underestimated
- Uptime target not achievable with current infrastructure

---

### Goal G-4: Enable Context-Aware Information Discovery

**Derived From Drivers**: SD-4 (Information Managers-Context), SD-6 (Domain Owners-Discovery)

**Goal Owner**: Information Managers (with Domain Owners)

**Goal Statement**: "Implement context-aware search combining user context (active zaak, domain, organization) with information origin context to deliver >80% relevance for government professionals seeking information."

**Why This Matters**: Per Dutch Ministry of Justice guidance, using data without context leads to unreliable and unjust decisions. Context preservation and context-aware discovery are fundamental to good government.

**Success Metrics**:

- **Primary Metric**: >80% search relevance for domain owner queries
- **Secondary Metrics**:
  - Context capture: 100% of entities have complete ContextMetadata
  - Search satisfaction: >70% user satisfaction score
  - Cross-domain discovery: 60% of relevant info found outside user's domain
  - Query latency: <200ms p95 for context-aware searches

**Baseline**:
- Current search: Keyword-only, 40% relevance
- Context capture: Inconsistent, often missing

**Target**:
- Context-aware algorithm: Implemented and tuned
- User context capture: Active zaak, domain, org tracked
- Information origin context: zaak_id, werkproces, domein, labels

**Measurement Method**:
- Search relevance testing (domain owner panels)
- User satisfaction surveys (quarterly)
- Context completeness audit (monthly)
- Performance testing (p95 latency)

**Dependencies**:
- ContextMetadata entity properly populated
- Context-aware search algorithm implemented (ADR-004)
- User session context tracking
- Graph traversal performance optimized

**Risks to Achievement**:
- Search relevance target difficult to measure objectively
- User context capture requires integration with other systems
- Graph traversal performance bottlenecks

---

## Goal-to-Outcome Mapping

### Outcome O-1: Zero Compliance Violations

**Supported Goals**: G-1 (Regulatory Compliance)

**Outcome Statement**: "Achieve zero AVG/Woo/Archiefwet violations in first 24 months of operation with 100% audit trail completeness and automated compliance enforcement."

**Measurement Details**:

- **KPI**: Compliance Violation Count
- **Current Value**: 3-5 violations per year (current systems)
- **Target Value**: 0 violations in 24-month period
- **Measurement Frequency**: Monthly (AP reporting), Quarterly (internal)
- **Data Source**: AP correspondence, internal audit logs, compliance assessments
- **Report Owner**: DPO

**Business Value**:

- **Financial Impact**: Avoid fines up to €20M per violation; total savings: €40M avoided
- **Strategic Impact**: Maintain government trust; avoid parliamentary scrutiny; establish compliance leadership
- **Operational Impact**: Reduced manual compliance work; automated enforcement; predictable operations
- **Customer Impact**: Citizen trust in government data handling; transparent operations

**Timeline**:

- **Phase 1 (Months 1-6)**: DPIA approval, implement AVG controls, zero violations target
- **Phase 2 (Months 7-12)**: Woo automation, achieve 95% 10-day SLA, zero Woo complaints
- **Phase 3 (Months 13-24)**: Archiefwet automation, CDD+ integration, zero archival violations
- **Sustainment (Year 2+)**: Maintain zero violations through continuous monitoring

**Stakeholder Benefits**:

- **DPO**: Demonstrates AVG accountability; avoids personal liability; reduces AP scrutiny
- **Woo Officers**: 100% Woo compliance; no refusal challenges; transparent operations
- **Information Managers**: Automated compliance; reduced manual work; archival peace of mind

**Leading Indicators** (early signals of success):
- DPIA approved by Q2 2026
- Automated controls implemented: 80% by Q4 2026
- Audit trail completeness: 100% from launch
- Staff training completion: 100% by Q3 2026

**Lagging Indicators** (final proof of success):
- AP investigations: 0 in 24-month period
- Woo complaints: 0 related to publication timeliness
- Archiefwet violations: 0 in audit findings
- Fines/penalties: €0

---

### Outcome O-2: Government-Wide BSW Adoption

**Supported Goals**: G-2 (BSW Architecture Adoption)

**Outcome Statement**: "Enable 500+ government organizations to adopt BSW-aligned zaak/dossierbeheer with 80% user satisfaction and 50% faster information discovery."

**Measurement Details**:

- **KPI**: BSW Adoption Rate & User Satisfaction
- **Current Value**: 0% (no BSW implementation)
- **Target Value**: 500+ organizations (100% by 2028), 80% user satisfaction
- **Measurement Frequency**: Quarterly (adoption), Monthly (satisfaction)
- **Data Source**: Organization registration, user surveys, Geonovum assessments
- **Report Owner**: Enterprise Architect

**Business Value**:

- **Financial Impact**: Avoid €50M in duplicate implementations; shared service efficiency
- **Strategic Impact**: Government-wide digital transformation; BSW mandate compliance; ketensamenwerking enabled
- **Operational Impact**: Cross-organization collaboration; standardized processes; reduced integration costs
- **Customer Impact**: Citizens experience consistent government services; faster zaak processing

**Timeline**:

- **Phase 1 (Months 1-9)**: BSW Phase 1 implementation (core entities, dynamic/persistent), pilot 5 organizations
- **Phase 2 (Months 10-18)**: BSW Phase 2 implementation (catalogus, context search), onboard 100 organizations
- **Phase 3 (Months 19-24)**: BSW Phase 3 implementation (object authorization, inheritance), onboard 250 organizations
- **Sustainment (Year 2+)**: Complete BSW implementation, onboard 500+ organizations

**Stakeholder Benefits**:

- **CIO**: BSW mandate compliance; shared service delivery; digital sovereignty demonstrated
- **Enterprise Architect**: BSW vision realized; government-wide standards; architecture leadership
- **Information Managers**: Context preservation operationalized; archival compliance automated
- **Domain Owners**: Better information discovery; cross-domain collaboration enabled

**Leading Indicators** (early signals of success):
- BSW entities implemented: 100% by Q2 2027
- Pilot organization success: 5 orgs live by Q4 2026
- Context search relevance: >70% in pilot testing
- Organization onboarding: 100+ orgs by Q3 2027

**Lagging Indicators** (final proof of success):
- Organizations using BSW: 500+ by 2028
- User satisfaction: >80% positive feedback
- Cross-domain queries: 60% find relevant info outside domain
- Information discovery time: 50% reduction vs. current

---

### Outcome O-3: Shared Service Cost Savings

**Supported Goals**: G-3 (Cost-Effective Shared Service)

**Outcome Statement**: "Deliver €15M cost savings through shared service approach while maintaining 99.5% uptime and enabling 100+ organization onboarding."

**Measurement Details**:

- **KPI**: Operational Cost per Organization
- **Current Value**: €1M per ministry (individual implementations)
- **Target Value**: €5k per organization (shared service) = €15M savings for 500 orgs
- **Measurement Frequency**: Quarterly (financial reporting)
- **Data Source**: Financial systems, cost allocation models, organization count
- **Report Owner**: CIO (with Finance)

**Business Value**:

- **Financial Impact**: €15M savings vs. baseline; €10M reinvestment in digital transformation
- **Strategic Impact**: Shared service model validated; template for other government services; digital sovereignty
- **Operational Impact**: Standardized operations; centralized expertise; reduced duplication
- **Customer Impact**: Consistent service quality across organizations; faster innovation deployment

**Timeline**:

- **Phase 1 (Months 1-6)**: Infrastructure setup, first 10 organizations onboarded
- **Phase 2 (Months 7-12)**: 50 organizations onboarded, cost model validated
- **Phase 3 (Months 13-18)**: 100+ organizations onboarded, €15M savings realized
- **Sustainment (Year 2+)**: 500+ organizations, economies of scale maintained

**Stakeholder Benefits**:

- **CIO**: Delivers on digital strategy; cost savings demonstrated; shared service model proven
- **Finance**: Budget efficiency validated; predictable cost model; reduced duplication
- **Organizations**: Lower costs; better service; faster access to innovation

**Leading Indicators** (early signals of success):
- Infrastructure cost: <€5M initial investment
- Onboarding automation: Self-service by Q4 2026
- Cost per org: <€10k by Q2 2026 (trending to €5k)
- Uptime: 99.5% achieved by Q3 2026

**Lagging Indicators** (final proof of success):
- Organizations onboarded: 100+ by Q3 2026, 500+ by 2028
- Cost per organization: €5k (average)
- Total savings: €15M vs. baseline
- Uptime maintained: 99.5% over 24-month period

---

### Outcome O-4: Context Preservation Measured

**Supported Goals**: G-4 (Context-Aware Discovery)

**Outcome Statement**: "Demonstrate 85%+ context preservation (volledigheid, samenhang, herleidbaarheid) through explicit metrics and audit trails proving government decisions based on complete, coherent, traceable information."

**Measurement Details**:

- **KPI**: Context Preservation Score
- **Current Value**: ~40% (scattered systems, implicit context)
- **Target Value**: 85%+ (explicit context capture, graph relationships, audit trails)
- **Measurement Frequency**: Monthly (context completeness), Quarterly (score)
- **Data Source**: Entity audits, relationship analysis, traceability assessments
- **Report Owner**: Information Managers

**Business Value**:

- **Financial Impact**: Avoid decision errors costing €5M+ annually; reduce rework; improve decision quality
- **Strategic Impact**: Align with Dutch Ministry of Justice guidance; demonstrate good government; legal defensibility
- **Operational Impact**: Reduced time finding context; improved information quality; enhanced collaboration
- **Customer Impact**: More reliable government decisions; reduced citizen appeals; improved trust

**Timeline**:

- **Phase 1 (Months 1-6)**: Context capture implemented; volledigheid baseline measured
- **Phase 2 (Months 7-12)**: Graph relationships complete; samenhang measured and improved
- **Phase 3 (Months 13-18)**: Audit trail complete; herleidbaarheid validated
- **Sustainment (Year 2+)**: Maintain 85%+ score; continuous improvement

**Stakeholder Benefits**:

- **Information Managers**: Context preservation operationalized; archival compliance; reduced audit burden
- **Domain Owners**: Better information discovery; context-aware decisions; reduced errors
- **DPO**: AVG accountability demonstrated; decision defensibility; reduced risk

**Leading Indicators** (early signals of success):
- Context completeness: 70% of entities have ContextMetadata by Q4 2026
- Graph relationships: 32 edge collections implemented by Q2 2026
- Audit trail: 100% of mutations logged from launch

**Lagging Indicators** (final proof of success):
- Context Preservation Score: 85%+ by Q4 2027
- Decision errors: 50% reduction vs. baseline
- Audit findings: Zero context-related deficiencies
- User feedback: >70% report better context understanding

---

## Complete Traceability Matrix

### Stakeholder → Driver → Goal → Outcome

| Stakeholder | Driver ID | Driver Summary | Goal ID | Goal Summary | Outcome ID | Outcome Summary |
|-------------|-----------|----------------|---------|--------------|------------|-----------------|
| CIO | SD-1 | Digital innovation & cost savings | G-3 | Cost-effective shared service | O-3 | €15M savings, 99.5% uptime |
| DPO | SD-2 | AVG compliance & accountability | G-1 | Full regulatory compliance | O-1 | Zero compliance violations |
| Woo Officers | SD-3 | Transparency & publication | G-1 | Full regulatory compliance | O-1 | Zero compliance violations |
| Info Managers | SD-4 | Context preservation | G-4 | Context-aware discovery | O-4 | 85%+ context score |
| Enterprise Arch | SD-5 | BSW alignment | G-2 | BSW architecture adoption | O-2 | 500+ orgs adopted |
| Domain Owners | SD-6 | Metadata discovery | G-4 | Context-aware discovery | O-4 | 85%+ context score |
| Security Officer | SD-7 | Data isolation | G-1 | Full regulatory compliance | O-1 | Zero compliance violations |
| DevOps Lead | SD-8 | Operational excellence | G-3 | Cost-effective shared service | O-3 | €15M savings, 99.5% uptime |
| Product Owner | SD-9 | Delivery velocity | G-2 | BSW architecture adoption | O-2 | 500+ orgs adopted |
| Integration Devs | SD-10 | API usability | G-3 | Cost-effective shared service | O-3 | €15M savings, 99.5% uptime |
| Nationaal Archief | SD-11 | Archival integrity | G-1 | Full regulatory compliance | O-1 | Zero compliance violations |

### Conflict Analysis

**Competing Drivers**:

- **Conflict 1**: CIO (SD-1) wants fast delivery for innovation vs. DPO (SD-2) requires thorough compliance review
  - **Impact**: Delivery timeline at risk if DPIA approval delays
  - **Resolution Strategy**: Phased approach - Phase 1 (core GGHH) while DPIA in progress; Phase 2 (full features) post-DPIA approval. Parallel work streams where possible.

- **Conflict 2**: CIO (SD-1) wants lowest cost vs. Security Officer (SD-7) wants comprehensive controls
  - **Impact**: Budget vs. security tradeoff
  - **Resolution Strategy**: Invest in security upfront (authentication, RLS) as non-negotiable; find cost savings in operations (automation, shared infrastructure) not security controls.

- **Conflict 3**: Domain Owners (SD-6) want cross-domain discovery vs. Security Officer (SD-7) wants strict isolation
  - **Impact**: Collaboration vs. security tension
  - **Resolution Strategy**: Object-level authorization (InformatieobjectRecht) enables both - strict isolation by default, selective access via grants. Publish authorization model to both stakeholders.

- **Conflict 4**: Product Owner (SD-9) wants incremental delivery vs. Enterprise Architect (SD-5) wants full BSW alignment
  - **Impact**: Scope vs. timeline tension
  - **Resolution Strategy**: BSW phased implementation - Phase 1: core entities (useful without full BSW), Phase 2-3: BSW-specific features. Manage expectations with both stakeholders.

**Synergies**:

- **Synergy 1**: DPO (SD-2), Woo Officers (SD-3), and Nationaal Archief (SD-11) all drive G-1 (Regulatory Compliance) - achieving G-1 satisfies all three compliance requirements simultaneously.

- **Synergy 2**: Information Managers (SD-4) and Domain Owners (SD-6) both drive G-4 (Context-Aware Discovery) - context preservation benefits both operational quality and user experience.

- **Synergy 3**: CIO (SD-1) and DevOps Lead (SD-8) both drive G-3 (Cost-Effective Shared Service) - operational excellence enables cost savings and reliability.

---

## Communication & Engagement Plan

### Stakeholder-Specific Messaging

#### CIO

**Primary Message**: "The Metadata Registry delivers on the Nationale Digitaliseringsstrategie through sovereign open-source technology, enabling BSW adoption while achieving 30% cost savings across 500+ government organizations."

**Key Talking Points**:
- **Digital Sovereignty**: 100% open-source stack (Rust, ArangoDB, Dioxus) reduces vendor dependence
- **BSW Readiness**: On track for 2028 mandate with phased implementation starting Q4 2026
- **Cost Efficiency**: €15M savings vs. individual implementations; shared service model validated
- **Innovation Speed**: 50% faster delivery through standardized platform and shared services

**Communication Frequency**: Quarterly (Steering Committee), Monthly (Executive Dashboard)

**Preferred Channel**: Executive dashboard + steering committee meetings

**Success Story**: "100 organizations onboarded by Q3 2026 at €5k per organization vs. €1M for individual implementations"

---

#### DPO

**Primary Message**: "The Metadata Registry ensures 100% AVG/GDPR compliance through complete audit trails, automated data minimization, and demonstrable accountability with zero violations target for first 24 months."

**Key Talking Points**:
- **AVG Compliance**: DPIA in progress; all controls designed per Article 25 (privacy by design)
- **Accountability**: Complete audit trail (who, what, when, why) for all metadata mutations
- **PII Protection**: Separate PII access logging; Row-Level Security for organization isolation
- **Rights Support**: Data subject rights (SAR, erasure) supported by design

**Communication Frequency**: Monthly (Compliance Dashboard), Quarterly (AP Briefing)

**Preferred Channel**: Compliance dashboard + quarterly AP briefing preparation

**Success Story**: "Zero AVG violations in first 24 months with 100% audit trail completeness"

---

#### Woo Officers

**Primary Message**: "The Metadata Registry automates Woo compliance with 95% of publications processed within 10-day deadline through automated relevance assessment and human approval workflows."

**Key Talking Points**:
- **Woo Compliance**: 100% of Woo requirements implemented per Wet open overheid
- **Automation**: Automatic relevance assessment; human approval for all Woo-relevant documents
- **Audit Trail**: Complete publication decision history with refusal grounds documented
- **PII Protection**: PII detection before publication; separate audit logging for Woo decisions

**Communication Frequency**: Monthly (Woo Dashboard), As-needed (Publication Issues)

**Preferred Channel**: Woo dashboard + monthly review meetings

**Success Story**: "95% of Woo publications within 10-day deadline vs. 40% baseline with manual process"

---

#### Information Managers

**Primary Message**: "The Metadata Registry preserves context (volledigheid, samenhang, herleidbaarheid) per Dutch Ministry of Justice guidance, ensuring reliable and just government decisions."

**Key Talking Points**:
- **Volledigheid**: Complete context capture (zaak_id, werkproces, domein, labels) for all entities
- **Samenhang**: Graph relationships preserve semantic connections; context-aware search finds related information
- **Herleidbaarheid**: Complete audit trail proves data provenance; GitOps maintains version history
- **Archiefwet**: Automated retention periods; CDD+ integration for long-term archival

**Communication Frequency**: Monthly (Context Dashboard), Quarterly (Training)

**Preferred Channel**: Context dashboard + quarterly training workshops

**Success Story**: "85%+ context preservation score vs. 40% baseline; decisions based on complete, coherent information"

---

#### Enterprise Architect

**Primary Message**: "The Metadata Registry delivers full BSW architecture alignment enabling government-wide zaak/dossierbeheer with information object-centric design and context-aware search."

**Key Talking Points**:
- **BSW Compliance**: All 7 BSW principles implemented per Geonovum guidance
- **Information Objects**: Core abstraction (dataobject + metadata) replacing document-centric approach
- **Context Search**: >80% relevance combining user and information origin context
- **Collaboration**: Object-level authorization enables ketensamenwerking

**Communication Frequency**: Monthly (Architecture Review), Quarterly (BSW Assessment)

**Preferred Channel**: Architecture review board + BSW compliance reports

**Success Story**: "500+ organizations using BSW-aligned metadata by 2028; context search >80% relevance"

---

#### Domain Owners

**Primary Message**: "The Metadata Registry enables context-aware search that understands your active zaak, domain, and organizational context to find relevant information 50% faster."

**Key Talking Points**:
- **Context Awareness**: Search combines your active zaak, domain, and organization with information origin
- **Cross-Domain Discovery**: Find relevant information from other domains without knowing their structure
- **Collaboration**: Share information with other domains while maintaining access control
- **Approval**: Approve metadata changes affecting your domain through workflow

**Communication Frequency**: Monthly (Product Newsletter), Quarterly (User Panels)

**Preferred Channel**: Admin UI + quarterly user panels + training

**Success Story**: "50% faster information discovery with >80% relevance; 60% find relevant info outside their domain"

---

#### Security Officer

**Primary Message**: "The Metadata Registry enforces organization-level data isolation through Row-Level Security while enabling authorized cross-organization collaboration."

**Key Talking Points**:
- **Data Isolation**: Each organization sees only their data unless explicitly authorized
- **Object-Level Authorization**: Fine-grained access control for collaboration scenarios
- **AVG Compliance**: RLS satisfies AVG Article 32 security requirements
- **Audit Trail**: All access logged with PII separately tracked

**Communication Frequency**: Monthly (Security Dashboard), As-needed (Incident Response)

**Preferred Channel**: Security dashboard + monthly architecture reviews

**Success Story**: "Zero data leakage incidents; <50ms authorization check latency"

---

## Change Impact Assessment

### Impact on Stakeholders

| Stakeholder | Current State | Future State | Change Magnitude | Resistance Risk | Mitigation Strategy |
|-------------|---------------|--------------|------------------|-----------------|---------------------|
| Information Managers | Scattered metadata, manual processes, no context tracking | Centralized registry with context preservation, automated compliance | HIGH | MEDIUM | Training, phased rollout, demonstrate benefits early |
| Domain Owners | Keyword search, limited cross-domain discovery | Context-aware search, cross-domain discovery, collaboration | HIGH | LOW | Better search experience, training, gradual rollout |
| Integration Developers | Inconsistent APIs, limited documentation | Standardized REST/GraphQL APIs, comprehensive docs | MEDIUM | LOW | Early API access, developer support, examples |
| DPO | Manual compliance tracking, risk of violations | Automated compliance enforcement, complete audit trails | HIGH | LOW | Reduced risk, automated controls, audit ready |
| Woo Officers | Manual Woo process, 40% meet deadline | Automated workflow, 95% meet deadline | HIGH | LOW | Reduced workload, automation, human control maintained |
| DevOps | No disaster recovery, manual processes | Automated DR, comprehensive observability | MEDIUM | MEDIUM | Training, documented procedures, gradual migration |

### Change Readiness

**Champions** (Enthusiastic supporters):
- **DPO**: Sees AVG compliance automation as major risk reduction
- **Woo Officers**: Automated workflow reduces manual burden
- **Enterprise Architect**: BSW alignment enables strategic vision
- **CIO**: Cost savings and innovation accelerate digital strategy

**Fence-sitters** (Neutral, need convincing):
- **Information Managers**: Will be convinced by context preservation and training
- **Domain Owners**: Will be convinced by better search experience
- **Integration Developers**: Will be convinced by API quality and documentation

**Resisters** (Opposed or skeptical):
- **Individual Ministry IT Leaders**: Fear loss of control; prefer custom solutions
  - *Why they resist*: Shared service reduces their authority and budget
  - *Strategy*: Demonstrate cost savings; show governance participation; maintain some customization options
- **Security Organization**: Fear complexity of multi-tenancy and object-level authorization
  - *Why they resist*: Increased attack surface, complexity
  - *Strategy*: Early involvement in security design; demonstrate controls; comprehensive testing
- **Union Representatives**: Fear job losses from automation
  - *Why they resist*: Reduced manual work (Woo, archival)
  - *Strategy*: Reassign to higher-value work (data quality, context capture); training for new skills

---

## Risk Register (Stakeholder-Related)

### Risk R-1: Stakeholder Resistance to Shared Service

**Related Stakeholders**: Individual Ministry IT Leaders, Security Organization

**Risk Description**: Key stakeholders may resist adopting the shared service model due to loss of control, budget concerns, or preference for custom solutions.

**Impact on Goals**: G-1 (Compliance), G-2 (BSW Adoption), G-3 (Cost Savings)

**Probability**: MEDIUM

**Impact**: HIGH

**Mitigation Strategy**:
- Early stakeholder engagement in design decisions
- Governance board with ministry representation
- Demonstrate cost savings and benefits early
- Maintain some customization options within standard framework

**Contingency Plan**: If resistance prevents adoption, develop hybrid model allowing ministry-specific instances with shared core services

---

### Risk R-2: DPIA Approval Delay

**Related Stakeholders**: DPO, Autoriteit Persoonsgegevens

**Risk Description**: DPIA approval may be delayed or require significant changes, blocking production deployment.

**Impact on Goals**: G-1 (Regulatory Compliance), G-3 (Cost Savings - timeline)

**Probability**: MEDIUM

**Impact**: CRITICAL

**Mitigation Strategy**:
- Start DPIA early (Q2 2026)
- Involve DPO in design from beginning
- Implement AVG controls by default (privacy by design)
- Prepare for AP scrutiny with comprehensive documentation

**Contingency Plan**: If DPIA not approved by Q3 2026, deploy with limited scope (non-PII data only) while addressing AP concerns

---

### Risk R-3: BSW Adoption Too Complex

**Related Stakeholders**: Enterprise Architect, Information Managers, Domain Owners

**Risk Description**: BSW architecture may be too complex for organizations to adopt within timeline, leading to low adoption or implementation failure.

**Impact on Goals**: G-2 (BSW Adoption), G-4 (Context-Aware Discovery)

**Probability**: MEDIUM

**Impact**: HIGH

**Mitigation Strategy**:
- Phased BSW implementation (core entities first, advanced features later)
- Extensive training and documentation
- Pilot organizations to validate approach
- Simplify where possible without compromising BSW principles

**Contingency Plan**: If BSW adoption too slow, focus on GGHH V2 compliance first (baseline) with BSW as optional enhancement

---

### Risk R-4: Context Preservation Not Measurable

**Related Stakeholders**: Information Managers, DPO, Nationaal Archief

**Risk Description**: Context preservation (volledigheid, samenhang, herleidbaarheid) may be difficult to measure objectively, making it hard to demonstrate compliance with Dutch Ministry of Justice guidance.

**Impact on Goals**: G-4 (Context-Aware Discovery), O-4 (Context Preservation Measured)

**Probability**: LOW

**Impact**: MEDIUM

**Mitigation Strategy**:
- Define explicit metrics for each context aspect
- Create context audit checklist
- Implement context completeness monitoring
- Document context preservation mechanisms

**Contingency Plan**: If measurement proves difficult, focus on qualitative assessment with stakeholder validation

---

### Risk R-5: Woo Publication Volume Exceeds Capacity

**Related Stakeholders**: Woo Officers, DPO, Information Managers

**Risk Description**: Human validation capacity for Woo publications may be insufficient, leading to SLA breaches and compliance violations.

**Impact on Goals**: G-1 (Regulatory Compliance), O-1 (Zero Compliance Violations)

**Probability**: MEDIUM

**Impact**: HIGH

**Mitigation Strategy**:
- Model expected publication volume during pilot
- Scale human validation team proportionally
- Implement prioritization for high-risk publications
- Consider automation for low-risk publications

**Contingency Plan**: If volume exceeds capacity, implement tiered service (rush processing vs. standard) with different SLAs

---

## Governance & Decision Rights

### Decision Authority Matrix (RACI)

| Decision Type | Responsible | Accountable | Consulted | Informed |
|---------------|-------------|-------------|-----------|----------|
| Regulatory Compliance (AVG) | DPO | CIO | Woo Officers, Info Managers | All stakeholders |
| Regulatory Compliance (Woo) | Woo Officers | CIO | DPO, Info Managers | All stakeholders |
| Regulatory Compliance (Archiefwet) | Info Managers | CIO | Nationaal Archief, DPO | All stakeholders |
| Architecture Decisions | Enterprise Architect | CIO | Security, DevOps | Product Owner |
| BSW Alignment | Enterprise Architect | CIO | Geonovum, Info Managers | All stakeholders |
| Security Architecture | Security Officer | CIO | DPO, Enterprise Architect | DevOps |
| Budget Approval | CIO | Finance Board | Product Owner | All stakeholders |
| Requirements Prioritization | Product Owner | Steering Committee | Domain Owners, Info Managers | DevOps, Integration Devs |
| Go/No-Go for Production | CIO | Steering Committee | DPO, Security, Enterprise Arch | All stakeholders |
| Organization Onboarding | Product Owner | CIO | DevOps, Info Managers | Onboarding organization |
| Cross-Org Authorization | Security Officer | DPO | Domain Owners | Info Managers |

### Escalation Path

1. **Level 1**: Project Team (Product Owner, Enterprise Architect, Security Officer) - Day-to-day decisions
2. **Level 2**: Steering Committee (CIO, DPO, Woo Officers, Info Managers, Enterprise Architect) - Strategic decisions, conflicts
3. **Level 3**: CIO Board (CIO, Finance Director, Legal Counsel) - Major escalations, budget changes, legal risks

---

## Validation & Sign-off

### Stakeholder Review

| Stakeholder | Review Date | Comments | Status |
|-------------|-------------|----------|--------|
| CIO | PENDING | | AWAITING REVIEW |
| DPO | PENDING | | AWAITING REVIEW |
| Enterprise Architect | PENDING | | AWAITING REVIEW |
| Woo Officers | PENDING | | AWAITING REVIEW |
| Information Managers | PENDING | | AWAITING REVIEW |
| Security Officer | PENDING | | AWAITING REVIEW |
| Product Owner | PENDING | | AWAITING REVIEW |

### Document Approval

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Project Sponsor | | | |
| Business Owner | | | |
| Enterprise Architect | | | |

---

## Appendices

### Appendix A: Dutch Ministry of Justice Guidance on Context

**Source**: "Data versus informatie en het belang van context" (Ministerie van Justitie en Veiligheid)

**Key Principles**:

> *"Gebruik gegevens zonder context = onbetrouwbaar en onrechtvaardig"*
> ("Using data without context = unreliable and unjust")

**Data → Information Transformation**:
- **Data** (waarde): Raw values without interpretation
- **Information** (betekenis in context): Data interpreted with complete context
- **Knowledge**: Actionable understanding derived from information

**Context Requirements**:

1. **Volledigheid** (Completeness):
   - All relevant data included
   - Context of creation documented (who, what, when, why)
   - No selective presentation that misleads

2. **Samenhang** (Coherence):
   - Data interpreted together, not in isolation
   - Relationships and dependencies preserved
   - Semantic meaning maintained

3. **Herleidbaarheid** (Traceability):
   - Source and context preserved
   - Audit trail complete
   - Decisions traceable to supporting information

**IT System Context Loss**:

IT systems naturally strip away context at each layer:
```
Samenleving (Society) → loses context
Bedrijfsdomein (Business) → loses context
IT-domein (IT Systems) → stripped context
```

**Compensation Required**:
- Explicit context capture in design
- Metadata preservation at each layer
- Business rules documented (not implicit)
- Traceability maintained throughout

---

### Appendix B: BSW Architecture Reference

**Source**: BSW (Beter Samenwerken) Architecture Documentation, Geonovum

**Key BSW Concepts**:

1. **Informatieobject**: Core abstraction = dataobject + metadata
2. **Dynamic vs Persistent**: Distinguish "in bewerking" (mutable) from "gepersisteerd" (read-only)
3. **Informatieobject Catalogus**: Metadata + location references (not content)
4. **Context-Aware Search**: User context + information origin context
5. **Object-Level Authorization**: Collaboration requires individual grants
6. **Metadata Inheritance**: Zaak/dossier metadata inherited by contained objects
7. **AI with Validation**: Human validation required before trusted use

**Timeline Requirements**:
- 2026: Pilot implementations
- 2027: 50% government organizations compliant
- 2028: 100% compliance mandatory

---

### Appendix C: Regulatory Requirements Reference

| Regulation | Key Requirements | Metadata Registry Implications |
|------------|------------------|-------------------------------|
| **AVG/GDPR** | Art 25: Privacy by design; Art 30: Records of processing; Art 32: Security of processing | DPIA required; audit trail; RLS; PII logging; data subject rights |
| **Woo** | Automatic relevance assessment; Human approval; 10-day deadline; Partial publication refusal | WooPublicatie entity; workflow; PII detection; refusal grounds |
| **Archiefwet 1995** | Retention periods (1/5/10/20 years); Provenance; Transfer to CDD+ | Bewaartermijn entity; audit trail; CDD+ integration; automated deletion |
| **BSW** | 7 principles for zaak/dossierbeheer | Information object-centric; dynamic/persistent; context search; object grants |

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-19 | ArcKit AI | Initial stakeholder analysis with 11 stakeholders, 4 goals, 4 outcomes |

## External References

| Document | Type | Source | Key Extractions | Path |
|----------|------|--------|-----------------|------|
| *None provided* | — | — | — | — |

---

**Generated by**: ArcKit `/arckit:stakeholders` command
**Generated on**: 2026-04-19
**ArcKit Version**: 4.3.1
**Project**: Metadata Registry Service (Project 002)
**AI Model**: Claude Opus 4.7
