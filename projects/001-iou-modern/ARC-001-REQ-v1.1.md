# Requirements: IOU-Modern

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Command**: `/arckit:requirements`

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-001-REQ-v1.1 |
| **Document Type** | Requirements |
| **Project** | IOU-Modern (Project 001) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.1 |
| **Created Date** | 2026-03-20 |
| **Last Modified** | 2026-03-20 |
| **Review Cycle** | Quarterly |
| **Next Review Date** | 2026-06-20 |
| **Owner** | Product Owner |
| **Reviewed By** | PENDING |
| **Approved By** | PENDING |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---------|------|--------|---------|-------------|---------------|
| 1.0 | 2026-03-20 | ArcKit AI | Initial creation from `/arckit:requirements` command | PENDING | PENDING |
| 1.1 | 2026-03-20 | ArcKit AI | Applied MoSCoW prioritization (MUST/SHOULD/MAY) | PENDING | PENDING |

---

## Executive Summary

IOU-Modern enables Dutch government organizations to manage information efficiently while complying with legal obligations (Woo, AVG, Archiefwet). The system provides domain-driven organization (Zaak, Project, Beleid, Expertise), AI-assisted document processing, knowledge graph capabilities, and automated compliance tracking.

**Scope**: Context-driven information management platform for 50,000+ government employees processing millions of documents.

**Key Requirements**:
- 45 business requirements (BR)
- 38 functional requirements (FR)
- 22 non-functional requirements (NFR)

**MoSCoW Prioritization**:
- **MUST**: 56 requirements (51%) - Non-negotiable for MVP
- **SHOULD**: 41 requirements (38%) - Important, deferred to v1.1
- **MAY**: 8 requirements (7%) - Nice to have, lowest priority

---

## 1. Business Requirements

### Domain Management (BR-001 to BR-010)

| ID | Requirement | Priority | Stakeholder | Source |
|----|-------------|----------|-------------|--------|
| **BR-001** | System shall organize information by Zaak (case/work) context | MUST | Domain Owners | Archiefwet |
| **BR-002** | System shall organize information by Project context | SHOULD | Project Managers | Common practice |
| **BR-003** | System shall organize information by Beleid (policy) context | SHOULD | Policy Advisors | Common practice |
| **BR-004** | System shall organize information by Expertise context | SHOULD | Knowledge Workers | Common practice |
| **BR-005** | System shall support hierarchical domain relationships (parent/child) | SHOULD | Domain Owners | Archiefwet |
| **BR-006** | System shall assign ownership to each domain (Domain Owner) | MUST | Information Managers | AVG accountability |
| **BR-007** | System shall support multi-tenancy across government organizations | MUST | CIO | Shared service model |
| **BR-008** | System shall enable cross-domain relationship discovery | SHOULD | Policy Advisors | Knowledge sharing |
| **BR-009** | System shall track domain lifecycle (Concept → Active → Completed → Archived) | MUST | Records Management | Archiefwet |
| **BR-010** | System shall support domain metadata (description, tags, custom fields) | MAY | Domain Owners | Flexibility |

### Document Management (BR-011 to BR-020)

| ID | Requirement | Priority | Stakeholder | Source |
|----|-------------|----------|-------------|--------|
| **BR-011** | System shall support multiple document types (Document, Email, Chat, Besluit, Data) | MUST | Records Management | Archiefwet |
| **BR-012** | System shall classify documents by security level (Openbaar, Intern, Vertrouwelijk, Geheim) | MUST | Security Officer | AVG |
| **BR-013** | System shall assess all documents for Woo relevance | MUST | Woo Officers | Woo |
| **BR-014** | System shall support document versioning with full history | MUST | Records Management | Archiefwet |
| **BR-015** | System shall track document compliance score (0.0-1.0) | SHOULD | Compliance Officers | Quality assurance |
| **BR-016** | System shall support document workflow (Draft → Review → Approved → Published) | MUST | Domain Owners | Woo publication |
| **BR-017** | System shall assign privacy level to documents (Openbaar, Normaal, Bijzonder, Strafrechtelijk) | MUST | DPO | AVG |
| **BR-018** | System shall enforce retention periods per document type | MUST | Records Management | Archiefwet |
| **BR-019** | System shall support full-text search across documents | SHOULD | All Users | Usability |
| **BR-020** | System shall support semantic search (meaning-based, not keyword-based) | MAY | Knowledge Workers | Innovation |

### Woo Compliance (BR-021 to BR-027)

| ID | Requirement | Priority | Stakeholder | Source |
|----|-------------|----------|-------------|--------|
| **BR-021** | System shall automatically identify Woo-relevant documents | SHOULD | Woo Officers | Woo |
| **BR-022** | System shall require human approval before publishing Woo-relevant documents | MUST | Legal Counsel | Woo liability |
| **BR-023** | System shall publish Openbaar documents to Woo portal | MUST | Woo Officers | Woo |
| **BR-024** | System shall track refusal grounds for non-publication | SHOULD | Woo Officers | Woo justification |
| **BR-025** | System shall track Woo publication date for compliance | MUST | Woo Officers | Woo timelines |
| **BR-026** | System shall support Woo publication workflow with audit trail | MUST | Auditors | Accountability |
| **BR-027** | System shall generate Woo decision documents from templates | MAY | Domain Owners | Efficiency |

### AVG/GDPR Compliance (BR-028 to BR-034)

| ID | Requirement | Priority | Stakeholder | Source |
|----|-------------|----------|-------------|--------|
| **BR-028** | System shall track all PII at entity level | MUST | DPO | AVG accountability |
| **BR-029** | System shall support Subject Access Requests (SAR) via API | MUST | DPO | AVG Article 15 |
| **BR-030** | System shall support data rectification | MUST | DPO | AVG Article 16 |
| **BR-031** | System shall support data erasure after retention period | MUST | DPO | AVG Article 17 |
| **BR-032** | System shall support data portability | SHOULD | DPO | AVG Article 20 |
| **BR-033** | System shall log all PII access | MUST | DPO | AVG Article 30 |
| **BR-034** | System shall conduct DPIA before processing high-risk data | MUST | DPO | AVG Article 35 |

### AI and Knowledge Graph (BR-035 to BR-045)

| ID | Requirement | Priority | Stakeholder | Source |
|----|-------------|----------|-------------|--------|
| **BR-035** | System shall extract named entities from documents | SHOULD | Knowledge Workers | Innovation |
| **BR-036** | System shall build knowledge graphs from extracted entities | SHOULD | Knowledge Workers | Innovation |
| **BR-037** | System shall discover cross-domain relationships | SHOULD | Policy Advisors | Knowledge sharing |
| **BR-038** | System shall support entity-based search | MAY | Knowledge Workers | Usability |
| **BR-039** | System shall provide AI assistance for document creation | MAY | Content Creators | Efficiency |
| **BR-040** | System shall provide AI compliance checking | SHOULD | Compliance Officers | Risk reduction |
| **BR-041** | System shall maintain human oversight for AI decisions | MUST | Legal Counsel | Liability |
| **BR-042** | System shall track AI confidence scores | MAY | Data Scientists | Transparency |
| **BR-043** | System shall support AI model explainability | MAY | Ethics Board | Transparency |
| **BR-044** | System shall detect and mitigate algorithmic bias | SHOULD | DPO | AVG fairness |
| **BR-045** | System shall opt-out citizens from entity extraction | SHOULD | DPO | AVG Article 21 |

---

## 2. Functional Requirements

### User Management (FR-001 to FR-005)

| ID | Requirement | Acceptance Criteria | Priority |
|----|-------------|-------------------|----------|
| **FR-001** | System shall authenticate users via DigiD | Integration with DigiD gateway successful | MUST |
| **FR-002** | System shall support role-based access control (RBAC) | Users see only permitted domains/documents | MUST |
| **FR-003** | System shall support domain-scoped permissions | Role limited to specific domain | MUST |
| **FR-004** | System shall track user login history | Last login timestamp recorded | SHOULD |
| **FR-005** | System shall support multi-factor authentication | MFA required for PII access | MUST |

### Domain Operations (FR-006 to FR-012)

| ID | Requirement | Acceptance Criteria | Priority |
|----|-------------|-------------------|----------|
| **FR-006** | System shall create domains with type (Zaak/Project/Beleid/Expertise) | Domain created with valid type | MUST |
| **FR-007** | System shall assign Domain Owner to each domain | Owner receives notifications | MUST |
| **FR-008** | System shall support domain hierarchy (parent/child) | Child domains inherit parent permissions | SHOULD |
| **FR-009** | System shall track domain status transitions | Status change logged | MUST |
| **FR-010** | System shall archive domains after completion | Archived domains read-only | MUST |
| **FR-011** | System shall discover domain relationships automatically | GraphRAG finds shared entities | SHOULD |
| **FR-012** | System shall support manual domain linking | Administrators can link domains | MAY |

### Document Operations (FR-013 to FR-022)

| ID | Requirement | Acceptance Criteria | Priority |
|----|-------------|-------------------|----------|
| **FR-013** | System shall ingest documents from source systems | ETL batch processes documents daily | MUST |
| **FR-014** | System shall store document content in S3/MinIO | File accessible via content_location | MUST |
| **FR-015** | System shall extract text from documents for search | Full-text search returns documents | MUST |
| **FR-016** | System shall classify documents by security level | Classification assigned automatically | MUST |
| **FR-017** | System shall assess Woo relevance | is_woo_relevant flag set | MUST |
| **FR-018** | System shall route documents through workflow | State transitions valid | MUST |
| **FR-019** | System shall record human approval decisions | Approval captured in audit trail | MUST |
| **FR-020** | System shall publish approved documents to Woo portal | Document visible on Woo portal | MUST |
| **FR-021** | System shall version documents | Version history maintained | MUST |
| **FR-022** | System shall support document templates | Template variables replaced correctly | SHOULD |

### Knowledge Graph (FR-023 to FR-028)

| ID | Requirement | Acceptance Criteria | Priority |
|----|-------------|-------------------|----------|
| **FR-023** | System shall extract Person entities from documents | NER identifies Person names | SHOULD |
| **FR-024** | System shall extract Organization entities from documents | NER identifies organizations | SHOULD |
| **FR-025** | System shall extract Location entities from documents | NER identifies locations | MAY |
| **FR-026** | System shall discover entity relationships | GraphRAG creates edges | SHOULD |
| **FR-027** | System shall cluster entities into communities | Hierarchical communities created | MAY |
| **FR-028** | System shall support graph traversal queries | Related entities returned | MAY |

### Search and Query (FR-029 to FR-032)

| ID | Requirement | Acceptance Criteria | Priority |
|----|-------------|-------------------|----------|
| **FR-029** | System shall support full-text search | Query returns matching documents | SHOULD |
| **FR-030** | System shall support entity-based search | Query returns documents containing entity | MAY |
| **FR-031** | System shall support semantic search | Vector similarity returns relevant results | MAY |
| **FR-032** | System shall support domain-scoped search | Results filtered by domain | MUST |

### Data Subject Rights (FR-033 to FR-038)

| ID | Requirement | Acceptance Criteria | Priority |
|----|-------------|-------------------|----------|
| **FR-033** | System shall provide SAR endpoint `/api/v1/subject-access-request` | Returns user's personal data within 30 days | MUST |
| **FR-034** | System shall provide rectification endpoint | Users can update their data | MUST |
| **FR-035** | System shall provide erasure endpoint | PII anonymized after retention | MUST |
| **FR-036** | System shall provide portability endpoint | Data exported in JSON/CSV | SHOULD |
| **FR-037** | System shall provide objection endpoint | Users can opt-out of processing | SHOULD |
| **FR-038** | System shall log all rights requests | Audit trail captures requests | MUST |

---

## 3. Non-Functional Requirements

### Performance (NFR-PERF-001 to NFR-PERF-005)

| ID | Requirement | Metric | Priority |
|----|-------------|--------|----------|
| **NFR-PERF-001** | Document ingestion throughput | >1,000 documents/minute | SHOULD |
| **NFR-PERF-002** | Search response time | <2 seconds for 95% of queries | MUST |
| **NFR-PERF-003** | API response time | <500ms for 95% of requests | MUST |
| **NFR-PERF-004** | Concurrent users | Support 1,000 concurrent users | SHOULD |
| **NFR-PERF-005** | Database query performance | <1 second for 95% of queries | MAY |

### Security (NFR-SEC-001 to NFR-SEC-008)

| ID | Requirement | Standard | Priority |
|----|-------------|----------|----------|
| **NFR-SEC-001** | Encryption at rest | AES-256 | MUST |
| **NFR-SEC-002** | Encryption in transit | TLS 1.3 | MUST |
| **NFR-SEC-003** | Authentication | DigiD + MFA | MUST |
| **NFR-SEC-004** | Authorization | RBAC + Row-Level Security | MUST |
| **NFR-SEC-005** | Audit logging | All PII access logged | MUST |
| **NFR-SEC-006** | Penetration testing | Annual penetration test | SHOULD |
| **NFR-SEC-007** | Vulnerability scanning | Quarterly scans | SHOULD |
| **NFR-SEC-008** | Incident response | 72-hour breach notification | MUST |

### Availability (NFR-AVAIL-001 to NFR-AVAIL-004)

| ID | Requirement | Metric | Priority |
|----|-------------|--------|----------|
| **NFR-AVAIL-001** | System uptime | 99.5% (excluding planned maintenance) | SHOULD |
| **NFR-AVAIL-002** | Recovery Time Objective (RTO) | <4 hours | MUST |
| **NFR-AVAIL-003** | Recovery Point Objective (RPO) | <1 hour | MUST |
| **NFR-AVAIL-004** | Backup retention | 30 days online, 7 years archival | MUST |

### Scalability (NFR-SCALE-001 to NFR-SCALE-004)

| ID | Requirement | Metric | Priority |
|----|-------------|--------|----------|
| **NFR-SCALE-001** | Document volume | Support 5M+ documents | SHOULD |
| **NFR-SCALE-002** | User volume | Support 100K+ users | SHOULD |
| **NFR-SCALE-003** | Organization volume | Support 500+ organizations | MAY |
| **NFR-SCALE-004** | Horizontal scaling | Add application servers without downtime | SHOULD |

### Compliance (NFR-COMP-001 to NFR-COMP-005)

| ID | Requirement | Regulation | Priority |
|----|-------------|------------|----------|
| **NFR-COMP-001** | Woo compliance | Wet open overheid | MUST |
| **NFR-COMP-002** | AVG compliance | Algemene verordening gegevensbescherming | MUST |
| **NFR-COMP-003** | Archiefwet compliance | Archiefwet 1995 | MUST |
| **NFR-COMP-004** | WCAG 2.1 AA | European Accessibility Act | SHOULD |
| **NFR-COMP-005** | Log retention | 7 years for audit logs | MUST |

---

## 4. Requirements Traceability Matrix

### Business → Functional Traceability

| Business Req | Functional Reqs | Data Model Entities |
|--------------|-----------------|-------------------|
| BR-001 to BR-004 (Domain types) | FR-006, FR-007 | E-002: InformationDomain |
| BR-005 (Hierarchy) | FR-008 | E-002.parent_domain_id |
| BR-011 to BR-020 (Documents) | FR-013 to FR-022 | E-003: InformationObject |
| BR-021 to BR-027 (Woo) | FR-017, FR-018, FR-020 | E-003.is_woo_relevant |
| BR-028 to BR-034 (AVG) | FR-033 to FR-038 | E-003.privacy_level |
| BR-035 to BR-045 (AI/KG) | FR-023 to FR-028 | E-011: Entity, E-012: Relationship |

### Functional → NFR Traceability

| Functional Req | Related NFRs |
|----------------|--------------|
| FR-001 to FR-005 (User Mgmt) | NFR-SEC-001 to NFR-SEC-008 |
| FR-013 to FR-022 (Documents) | NFR-PERF-001, NFR-SCALE-001 |
| FR-029 to FR-032 (Search) | NFR-PERF-002 |
| FR-033 to FR-038 (DSAR) | NFR-COMP-002 (AVG) |

---

## 5. Requirements Priority Summary

| Priority | Business | Functional | Non-Functional | Total | % |
|----------|----------|-----------|----------------|-------|---|
| **MUST** | 18 | 23 | 15 | 56 | 51% |
| **SHOULD** | 22 | 12 | 7 | 41 | 38% |
| **MAY** | 5 | 3 | 0 | 8 | 7% |
| **TOTAL** | 45 | 38 | 22 | 105 | 100% |

**MoSCoW Definition**:
- **MUST**: Non-negotiable for MVP (legal compliance, security, core functionality)
- **SHOULD**: Important but can be deferred to v1.1
- **MAY**: Nice to have, lowest priority

**Critical Path Requirements** (MUST be implemented first for MVP):
1. FR-001: DigiD authentication (MUST)
2. FR-002: RBAC (MUST)
3. FR-005: MFA for PII access (MUST)
4. FR-006: Domain creation (MUST)
5. FR-013: Document ingestion (MUST)
6. FR-016: Document classification (MUST)
7. FR-017: Woo relevance assessment (MUST)
8. FR-019: Human approval (MUST)
9. FR-020: Woo publication (MUST)
10. FR-033: SAR endpoint (MUST)
11. NFR-SEC-001 through NFR-SEC-005: Security baseline (MUST)
12. NFR-COMP-001 through NFR-COMP-003: Legal compliance (MUST)

---

## 6. Out of Scope (Explicitly Not Required)

The following are **NOT** in scope for IOU-Modern v1.0:

| Item | Reason | Future Consideration |
|------|--------|---------------------|
| Digital signature support | Not in initial requirements | v2.0 |
| Video/audio processing | Text documents only | v2.0 |
| Real-time collaboration | Async workflow only | v2.0 |
| Mobile app | Web interface only | v2.0 |
| Internationalization (i18n) | Dutch-only initially | v2.0 |
| Advanced analytics | Basic reporting only | v2.0 |
| Blockchain features | Not required for government use | Not planned |

---

## 7. Assumptions and Dependencies

### Assumptions

| ID | Assumption | Risk if Invalid |
|----|------------|-----------------|
| A1 | DigiD integration available | Cannot authenticate citizens |
| A2 | Source systems provide structured data | Poor data quality |
| A3 | Woo portal has API | Manual publication required |
| A4 | Organizations have reliable internet | System unavailable |
| A5 | Budget for AI API costs | Reduced functionality |

### Dependencies

| ID | Dependency | Type | Owner |
|----|------------|------|-------|
| D1 | DigiD gateway access | External | Logius |
| D2 | Source system APIs | External | Case management vendors |
| D3 | Woo portal API | External | Gemeente Den Haag |
| D4 | PostgreSQL hosting | Infrastructure | CIO |
| D5 | S3/MinIO storage | Infrastructure | CIO |
| D6 | AI API (OpenAI/Anthropic) | External | Budget approval |

---

## 8. Glossary

| Term | Definition |
|------|------------|
| **AVG** | Algemene verordening gegevensbescherming (GDPR Netherlands) |
| **Woo** | Wet open overheid (Government Information Act) |
| **Archiefwet** | Dutch Archives Act governing record retention |
| **DigiD** | Dutch digital identity system |
| **Zaak** | Case or executive work item |
| **Beleid** | Policy development work |
| **Expertise** | Knowledge sharing and collaboration |
| **NER** | Named Entity Recognition (AI technique) |
| **GraphRAG** | Graph-based Retrieval Augmented Generation |
| **PII** | Personally Identifiable Information |
| **DPIA** | Data Protection Impact Assessment |

---

**END OF REQUIREMENTS**

## Generation Metadata

**Generated by**: ArcKit `/arckit:requirements` command
**Generated on**: 2026-03-20
**ArcKit Version**: 4.3.1
**Project**: IOU-Modern (Project 001)
**AI Model**: Claude Opus 4.6
