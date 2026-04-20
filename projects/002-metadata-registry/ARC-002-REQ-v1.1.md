# Requirements: Metadata Registry Service

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Command**: `/arckit:requirements`

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-002-REQ-v1.1 |
| **Document Type** | Business and Technical Requirements |
| **Project** | Metadata Registry Service (Project 002) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.1 |
| **Created Date** | 2026-04-19 |
| **Last Modified** | 2026-04-19 |
| **Review Cycle** | Quarterly |
| **Next Review Date** | 2026-07-19 |
| **Owner** | Product Owner |
| **Reviewed By** | [PENDING] |
| **Approved By** | [PENDING] |
| **Distribution** | Project Team, Architecture Team, DPO, Woo Officers |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---------|------|--------|---------|-------------|---------------|
| 1.0 | 2026-04-19 | ArcKit AI | Initial creation from `/arckit:requirements` command | [PENDING] | [PENDING] |
| 1.1 | 2026-04-19 | BSW Review | Added BSW architecture compliance requirements: dynamic/persistent status, information object catalogus, context-aware search, object-level authorization, metadata inheritance, workflow integration, AI validation, multi-caretaker support | [PENDING] | [PENDING] |

---

## Executive Summary

### Business Context

The Metadata Registry Service is a centralized component of IOU-Modern that implements the Dutch Government Metamodel GGHH (Gegevenshuishouding). It provides a unified metadata catalog for all government information assets, enabling compliance with Wet open overheid (Woo), Algemene verordening gegevensbescherming (AVG/GDPR), and the nieuwe Archiefwet 2025 requirements.

**BSW Architecture Principle**: This service implements the BSW (Beter Samenwerken) architecture paradigm where the **informatieobject** (dataobject + metadata) is the core abstraction—not the document or dataobject alone. All information is managed as information objects with complete context metadata for duurzame toegankelijkheid.

The registry serves as the single source of truth for metadata across government organizations, supporting domain-driven information management (Zaak, Project, Beleid, Expertise) while maintaining complete audit trails and time-based validity for all metadata entities.

### Objectives

- Establish a centralized metadata repository compliant with Metamodel GGHH V2 specification
- Implement BSW architecture principles (informatieobject-centric, dynamic/persistent storage)
- Enable automated compliance tracking for Woo, AVG, and Archiefwet 2025 requirements (10-year transfer rule)
- Provide REST/GraphQL APIs for metadata consumption by downstream services
- Support GitOps-based metadata synchronization for version control and auditability
- Deliver an admin UI for metadata stewards to manage the registry
- Enable context-aware search combining user and information origin context

### Expected Outcomes

- 95%+ compliance with Metamodel GGHH specification (all V2 core entities implemented)
- 100% alignment with BSW architecture principles (informatieobject-centric design)
- 100% of metadata changes tracked with audit trail (who, what, when, why)
- <100ms API response time for 95% of metadata queries
- Support for 500+ government organizations with multi-tenancy
- Reduced metadata duplication by 80% across government systems
- Context-aware search with 80%+ relevance accuracy for user queries

### Project Scope

**In Scope**:
- Metamodel GGHH V2 core entities (Gebeurtenis, Gegevensproduct, ElementaireGegevensset, EnkelvoudigGegeven, WaardeMetTijd, Context, Grondslag)
- Phase 1-8 entities (Bedrijfsproces, Wetsbegrip, Beleidsbegrip, Informatieobject, Woo publicatie, etc.)
- BSW-specific entities (Zaak, InformatieobjectCatalogus, InformatieobjectRecht)
- Dynamic/persistent status management per BSW architecture
- Context-aware search combining user and information origin context
- Object-level authorization for collaboration scenarios
- REST API v2 and GraphQL API for metadata access
- ArangoDB-based graph storage with 32 edge collections
- GitOps synchronization service for YAML-based metadata definitions
- Dioxus-based admin UI for metadata management
- Validation engine for TOOI/MDTO standards compliance
- AI enrichment with human validation workflow
- Archive integration (CDD+) for long-term records retention

**Out of Scope**:
- Document content storage (managed by IOU-Modern core system)
- AI enrichment features (Phase 6 - mock implementation only)
- Real-time event streaming (future consideration)
- Multi-language support (Dutch-only initially)
- External government system integrations (handled by integration layer)

---

## Stakeholders

| Stakeholder | Role | Organization | Involvement Level |
|-------------|------|--------------|-------------------|
| Product Owner | Requirements Owner | IOU-Modern | Decision maker |
| Enterprise Architect | Technical Oversight | Architecture | Design authority |
| DPO | Privacy Compliance | Legal | AVG/GDPR requirements |
| Woo Officers | Woo Compliance | Legal | Publication workflow |
| Information Managers | Records Management | Operations | Archiefwet requirements |
| Domain Owners | Metadata Consumers | Business Units | User acceptance |
| DevOps Lead | Operations | IT | Deployment and monitoring |

---

## Business Requirements

### Metamodel Compliance (BR-MREG-001 to BR-MREG-010)

| ID | Requirement | Priority | Stakeholder | Source |
|----|-------------|----------|-------------|--------|
| **BR-MREG-001** | Registry shall implement all Metamodel GGHH V2 core entities | MUST | Enterprise Architect | Overheid 20240530 |
| **BR-MREG-002** | Registry shall support time-based validity (geldig_vanaf/geldig_tot) for all entities | MUST | Information Managers | Archiefwet |
| **BR-MREG-003** | Registry shall maintain graph relationships via edge collections | MUST | Enterprise Architect | GGHH specification |
| **BR-MREG-004** | Registry shall support multi-tenancy across government organizations | MUST | CIO | Shared service model |
| **BR-MREG-005** | Registry shall track complete audit trail for all metadata mutations | MUST | DPO | AVG accountability |
| **BR-MREG-006** | Registry shall validate TOOI/MDTO standard compliance | SHOULD | Information Managers | Interoperability |
| **BR-MREG-007** | Registry shall support GitOps synchronization for metadata definitions | SHOULD | DevOps Lead | Version control |
| **BR-MREG-008** | Registry shall integrate with CDD+ for archival | SHOULD | Information Managers | Archiefwet |
| **BR-MREG-009** | Registry shall support Woo publication workflow | MUST | Woo Officers | Woo compliance |
| **BR-MREG-010** | Registry shall expose REST and GraphQL APIs | MUST | Integration Team | Interoperability |

### Privacy and Compliance (BR-MREG-011 to BR-MREG-018)

| ID | Requirement | Priority | Stakeholder | Source |
|----|-------------|----------|-------------|--------|
| **BR-MREG-011** | Registry shall classify all metadata by AVG privacy category | MUST | DPO | AVG Article 9 |
| **BR-MREG-012** | Registry shall track retention periods (Archiefwet) per entity | MUST | Information Managers | Archiefwet |
| **BR-MREG-013** | Registry shall support PII detection for metadata content | SHOULD | DPO | AVG Article 30 |
| **BR-MREG-014** | Registry shall log all PII access separately | MUST | DPO | AVG Article 30 |
| **BR-MREG-015** | Registry shall support data subject rights (SAR, erasure) | MUST | DPO | AVG Articles 15-17 |
| **BR-MREG-016** | Registry shall enforce Row-Level Security for organization isolation | MUST | Security Officer | AVG security |
| **BR-MREG-017** | Registry shall store all data within EU jurisdiction | MUST | CIO | Data sovereignty |
| **BR-MREG-018** | Registry shall support Woo relevance assessment for Informatieobject | MUST | Woo Officers | Woo |

### Operational Excellence (BR-MREG-019 to BR-MREG-025)

| ID | Requirement | Priority | Stakeholder | Source |
|----|-------------|----------|-------------|--------|
| **BR-MREG-019** | Registry shall achieve 99.5% uptime (excluding planned maintenance) | SHOULD | DevOps Lead | SLA requirement |
| **BR-MREG-020** | Registry shall support horizontal scaling | SHOULD | CIO | Growth planning |
| **BR-MREG-021** | Registry shall recover from failures within 4 hours (RTO) | MUST | DevOps Lead | Business continuity |
| **BR-MREG-022** | Registry shall maintain backups with <1 hour data loss (RPO) | MUST | DevOps Lead | Data protection |
| **BR-MREG-023** | Registry shall use open-source technology stack | MUST | CIO | Sovereign Technology principle |
| **BR-MREG-024** | Registry shall provide comprehensive observability (logs, metrics, traces) | SHOULD | DevOps Lead | Operational monitoring |
| **BR-MREG-025** | Registry shall support automated deployment via CI/CD | SHOULD | DevOps Lead | Delivery velocity |

### BSW Architecture Compliance (BR-MREG-026 to BR-MREG-034)

| ID | Requirement | Priority | Stakeholder | Source |
|----|-------------|----------|-------------|--------|
| **BR-MREG-026** | Registry shall distinguish between dynamic (in bewerking) and persistent (gepersisteerd) information objects | MUST | Enterprise Architect | BSW architectuur |
| **BR-MREG-027** | Registry shall support information object catalogus with location references | MUST | Information Managers | BSW informatiecataloog |
| **BR-MREG-028** | Registry shall support context-aware search combining user and information origin context | MUST | Domain Owners | BSW zoek en vind |
| **BR-MREG-029** | Registry shall support authorization at individual information object level | MUST | Security Officer | BSW samenwerking |
| **BR-MREG-030** | Registry shall support metadata inheritance from zaak/dossier to contained objects | SHOULD | Information Managers | BSW zaakbeheer |
| **BR-MREG-031** | Registry shall support external workflow status integration | SHOULD | DevOps Lead | BSW workflow |
| **BR-MREG-032** | Registry shall require human validation for AI enrichment results before trusted use | MUST | DPO | BSW AI verrijking |
| **BR-MREG-033** | Registry shall support multi-caretaker scenarios for shared information objects | SHOULD | Information Managers | BSW ketensamenwerking |
| **BR-MREG-034** | Registry shall store informatiecategorie per Woo requirements | MUST | Woo Officers | Woo verplichting |

---

## Functional Requirements

### User Personas

#### Persona 1: Metadata Steward (Informatiebeheerder)

- **Role**: Information Manager responsible for metadata quality
- **Goals**: Maintain accurate metadata, ensure compliance, support end users
- **Pain Points**: Scattered metadata sources, manual validation processes, unclear ownership
- **Technical Proficiency**: Medium

#### Persona 2: Domain Owner (Eigenaar)

- **Role**: Business owner of an information domain (Zaak/Project/Beleid)
- **Goals**: Understand domain metadata, approve changes, ensure Woo compliance
- **Pain Points**: Limited visibility into metadata, unclear approval workflows
- **Technical Proficiency**: Low

#### Persona 3: Integration Developer

- **Role**: Developer integrating with metadata registry via APIs
- **Goals**: Reliable API access, clear documentation, predictable responses
- **Pain Points**: API inconsistencies, lack of examples, rate limiting issues
- **Technical Proficiency**: High

#### Persona 4: DPO / Compliance Officer

- **Role**: Data protection and compliance oversight
- **Goals**: Verify AVG compliance, audit metadata changes, approve processing
- **Pain Points**: Limited visibility into metadata processing, incomplete audit trails
- **Technical Proficiency**: Medium

---

### Use Cases

#### UC-MREG-1: Metadata Registration

**Actor**: Metadata Steward

**Preconditions**:
- User authenticated with RBAC role "metadata_steward"
- User has write access to target organization

**Main Flow**:
1. User navigates to admin UI entity creation page
2. User selects entity type (e.g., Informatieobject)
3. User enters required attributes (naam, omschrijving, geldig_vanaf)
4. System validates required fields and data types
5. System checks TOOI/MDTO compliance if applicable
6. User submits entity for creation
7. System generates unique entity ID
8. System stores entity in ArangoDB
9. System logs audit trail entry (who, what, when, why)
10. System confirms successful creation

**Postconditions**:
- Entity stored in database with unique ID
- Audit trail entry created
- Entity searchable via API

**Alternative Flows**:
- **Alt 5a**: If TOOI/MDTO validation fails, display specific errors and prevent creation
- **Alt 6a**: If required field missing, highlight field and prevent submission

**Exception Flows**:
- **Ex 1**: Database connection error - display error and queue for retry

**Business Rules**:
- BR-MREG-001: All entities must conform to GGHH V2 specification
- BR-MREG-005: All changes must be logged in audit trail

**Priority**: CRITICAL

---

#### UC-MREG-2: Metadata Query via REST API

**Actor**: Integration Developer

**Preconditions**:
- Valid API token for organization
- Entity ID or search criteria known

**Main Flow**:
1. Application sends GET request to /api/v2/{entity_type}/{id}
2. System validates API token and organization access
3. System checks Row-Level Security for entity access
4. System retrieves entity from ArangoDB
5. System filters fields based on user permissions
6. System applies time-based validity filter (geldig_vanaf/geldig_tot)
7. System returns entity with HTTP 200

**Postconditions**:
- Client receives entity data
- Access logged in audit trail

**Alternative Flows**:
- **Alt 3a**: If entity not found, return HTTP 404
- **Alt 6a**: If entity expired (geldig_tot < now), return HTTP 404

**Exception Flows**:
- **Ex 1**: Invalid API token - return HTTP 401
- **Ex 2**: Insufficient permissions - return HTTP 403

**Business Rules**:
- BR-MREG-016: Row-Level Security enforced for all queries
- BR-MREG-002: Time-based validity applied automatically

**Priority**: CRITICAL

---

#### UC-MREG-3: Woo Publication Workflow

**Actor**: Domain Owner, Woo Officer

**Preconditions**:
- Informatieobject exists with is_woo_relevant=true
- Domain Owner authenticated

**Main Flow**:
1. Domain Owner requests Woo publication for Informatieobject
2. System validates Woo relevance assessment
3. System checks for PII presence (via AI enrichment)
4. System creates WooPublicatie entity in status "voorgelegd"
5. Woo Officer reviews publication request
6. Woo Officer approves or rejects with grounds
7. If approved, system updates status to "goedgekeurd"
8. System triggers publication to Woo portal
9. System logs decision in audit trail

**Postconditions**:
- WooPublicatie entity created with complete workflow
- Publication executed to Woo portal
- Audit trail records human decision

**Alternative Flows**:
- **Alt 6a**: If rejected, record refusal grounds and notify requester

**Exception Flows**:
- **Ex 1**: PII detected without redaction - block publication and require review

**Business Rules**:
- BR-MREG-009: Human approval required for Woo publication
- BR-MREG-018: PII detection before publication

**Priority**: CRITICAL

---

#### UC-MREG-4: GitOps Synchronization

**Actor**: DevOps Engineer

**Preconditions**:
- Git repository configured with metadata YAML files
- GitOps service running and authenticated

**Main Flow**:
1. Developer commits YAML metadata definition to Git repository
2. GitOps service detects new commit via webhook
3. Service parses YAML files
4. Service validates against GGHH schema
5. Service compares with existing database state
6. Service creates, updates, or deletes entities
7. Service commits changes with audit context
8. Service reports synchronization status

**Postconditions**:
- Database state matches Git repository
- All changes logged with Git commit reference

**Alternative Flows**:
- **Alt 4a**: If validation fails, report error and skip sync

**Exception Flows**:
- **Ex 1**: Merge conflict detected - require manual resolution

**Business Rules**:
- BR-MREG-007: GitOps synchronization for version control
- BR-MREG-005: Audit trail includes Git commit reference

**Priority**: HIGH

---

### Functional Requirements Detail

#### FR-MREG-1: Entity CRUD Operations

**Description**: System shall support Create, Read, Update, Delete operations for all GGHH V2 entities

**Relates To**: BR-MREG-001, UC-MREG-1

**Acceptance Criteria**:
- Given valid entity data, when POST to /api/v2/{entity_type}, then entity created with unique ID
- Given entity exists, when GET /api/v2/{entity_type}/{id}, then entity returned
- Given entity exists, when PUT /api/v2/{entity_type}/{id}, then entity updated
- Given entity exists, when DELETE /api/v2/{entity_type}/{id}, then entity soft-deleted

**Data Requirements**:
- **Inputs**: Entity JSON conforming to GGHH schema
- **Outputs**: Entity JSON with system fields (id, created_at, updated_at)
- **Validations**: Required fields, data types, enum values, referential integrity

**Priority**: MUST

**Complexity**: MEDIUM

**Dependencies**: None

**Assumptions**: ArangoDB provides transactional consistency

---

#### FR-MREG-2: Graph Traversal Queries

**Description**: System shall support efficient graph traversal queries via edge collections

**Relates To**: BR-MREG-003

**Acceptance Criteria**:
- Given entity with relationships, when query with depth parameter, then related entities returned to specified depth
- Given edge collection query, when traversal executed, then path returned with edge types
- Given disconnected entities, when path query executed, then empty result returned

**Data Requirements**:
- **Inputs**: Start entity ID, traversal direction, depth filter, edge types
- **Outputs**: Array of entities with path metadata
- **Validations**: Depth limit max 5, cyclic path detection

**Priority**: MUST

**Complexity**: HIGH

**Dependencies**: FR-MREG-1 (entities must exist)

**Assumptions**: ArangoDB graph traversal performance acceptable for 5-hop queries

---

#### FR-MREG-3: Time-Based Validity Filtering

**Description**: System shall automatically filter entities based on geldig_vanaf and geldig_tot

**Relates To**: BR-MREG-002

**Acceptance Criteria**:
- Given entity with geldig_vanaf in future, when query executed, then entity not returned
- Given entity with geldig_tot in past, when query executed, then entity not returned
- Given entity with current date within validity period, when query executed, then entity returned
- Given query with historical date parameter, when query executed, then validity applied to historical date

**Data Requirements**:
- **Inputs**: Optional geldig_op date parameter (defaults to now)
- **Outputs**: Entities filtered by validity
- **Validations**: Date format ISO 8601

**Priority**: MUST

**Complexity**: MEDIUM

**Dependencies**: FR-MREG-1

**Assumptions**: Index on geldig_vanaf and geldig_tot fields

---

#### FR-MREG-4: TOOI/MDTO Validation

**Description**: System shall validate metadata against TOOI and MDTO standards

**Relates To**: BR-MREG-006

**Acceptance Criteria**:
- Given Informatieobject, when validated, then TOOI code verified against official list
- Given metadata element, when validated, then MDTO schema checked
- Given invalid TOOI code, when validated, then specific error returned

**Data Requirements**:
- **Inputs**: Entity data for validation
- **Outputs**: Validation result with errors/warnings
- **Validations**: TOOI code list, MDTO schema definitions

**Priority**: SHOULD

**Complexity**: MEDIUM

**Dependencies**: FR-MREG-1

**Assumptions**: TOOI/MDTO reference data available

---

#### FR-MREG-5: Row-Level Security

**Description**: System shall enforce organization-level isolation via Row-Level Security

**Relates To**: BR-MREG-016

**Acceptance Criteria**:
- Given user from Organization A, when query entities, then only Organization A entities returned
- Given admin user, when query entities, then all entities returned
- Given user attempting cross-organization access, when unauthorized, then HTTP 403 returned

**Data Requirements**:
- **Inputs**: User token with organization claim
- **Outputs**: Filtered query results
- **Validations**: Organization claim present in token

**Priority**: MUST

**Complexity**: HIGH

**Dependencies**: FR-MREG-1, FR-MREG-2

**Assumptions**: ArangoDB RLS capabilities or application-level filtering

---

#### FR-MREG-6: Audit Trail Logging

**Description**: System shall log all metadata mutations with business context

**Relates To**: BR-MREG-005

**Acceptance Criteria**:
- Given entity created, when operation completes, then audit entry logged
- Given entity updated, when operation completes, then audit entry logged with before/after
- Given entity deleted, when operation completes, then audit entry logged
- Given audit query, when requested, then audit trail returned for entity

**Data Requirements**:
- **Inputs**: Mutation operation, user context, business reason
- **Outputs**: Audit entry stored
- **Validations**: All required audit fields present

**Priority**: MUST

**Complexity**: MEDIUM

**Dependencies**: FR-MREG-1

**Assumptions**: Audit log retained for 7 years

---

#### FR-MREG-7: GraphQL API

**Description**: System shall provide GraphQL API for flexible metadata queries

**Relates To**: BR-MREG-010

**Acceptance Criteria**:
- Given valid GraphQL query, when POST /graphql, then requested data returned
- Given query with relationships, when executed, then nested entities resolved
- Given invalid query, when executed, then validation error returned

**Data Requirements**:
- **Inputs**: GraphQL query document, variables
- **Outputs**: JSON response with data/errors
- **Validations**: Query syntax, field existence, argument types

**Priority**: SHOULD

**Complexity**: HIGH

**Dependencies**: FR-MREG-1, FR-MREG-2

**Assumptions**: GraphQL schema auto-generated from GGHH entities

---

#### FR-MREG-8: Full-Text Search

**Description**: System shall provide full-text search across Informatieobject entities

**Relates To**: BR-MREG-010

**Acceptance Criteria**:
- Given search term, when GET /api/v2/informatieobjecten/zoek, then matching results returned
- Given search with filters, when executed, then results filtered by criteria
- Given search with pagination, when executed, then paginated results returned

**Data Requirements**:
- **Inputs**: Search term, filters, pagination parameters
- **Outputs**: Search results with relevance scores
- **Validations**: Search term length minimum 2 characters

**Priority**: SHOULD

**Complexity**: MEDIUM

**Dependencies**: FR-MREG-1

**Assumptions**: ArangoDB full-text search or external search service

---

#### FR-MREG-9: Woo Publication Workflow

**Description**: System shall implement Woo publication approval workflow

**Relates To**: BR-MREG-009, UC-MREG-3

**Acceptance Criteria**:
- Given Woo-relevant Informatieobject, when publication requested, then WooPublicatie created
- Given WooPublicatie in review, when approved, then status updated to goedgekeurd
- Given approved publication, when publish triggered, then document sent to Woo portal
- Given rejected publication, when rejected, then refusal grounds recorded

**Data Requirements**:
- **Inputs**: Informatieobject ID, refusal grounds (if rejected)
- **Outputs**: WooPublicatie entity with status
- **Validations**: Human approval required, PII check passed

**Priority**: MUST

**Complexity**: HIGH

**Dependencies**: FR-MREG-1, FR-MREG-6

**Assumptions**: Woo portal API available for publication

---

#### FR-MREG-10: GitOps Synchronization

**Description**: System shall synchronize metadata from Git repository

**Relates To**: BR-MREG-007, UC-MREG-4

**Acceptance Criteria**:
- Given Git commit with metadata YAML, when webhook received, then sync initiated
- Given valid YAML entity, when parsed, then entity created/updated in database
- Given deleted YAML file, when detected, then corresponding entity soft-deleted
- Given sync completion, when finished, then status reported

**Data Requirements**:
- **Inputs**: Git webhook payload, repository URL
- **Outputs**: Sync status, error details
- **Validations**: YAML schema validity, GGHH compliance

**Priority**: SHOULD

**Complexity**: HIGH

**Dependencies**: FR-MREG-1, FR-MREG-6

**Assumptions**: Git repository accessible with webhook configuration

---

#### FR-MREG-11: Admin UI

**Description**: System shall provide web-based admin UI for metadata management

**Relates To**: UC-MREG-1

**Acceptance Criteria**:
- Given admin user, when UI accessed, then entity list displayed
- Given create action, when form submitted, then entity created
- Given edit action, when changes saved, then entity updated
- Given delete action, when confirmed, then entity soft-deleted

**Data Requirements**:
- **Inputs**: User actions via browser
- **Outputs**: UI rendering with metadata
- **Validations**: Form validation before submission

**Priority**: SHOULD

**Complexity**: MEDIUM

**Dependencies**: FR-MREG-1, FR-MREG-2

**Assumptions**: Dioxus framework with WebAssembly deployment

---

#### FR-MREG-12: CDD+ Archive Integration

**Description**: System shall support archival to CDD+ system

**Relates To**: BR-MREG-008

**Acceptance Criteria**:
- Given archival request, when POST /api/v2/archiveer/:id, then entity prepared for archive
- Given valid archive request, when processed, then metadata sent to CDD+
- Given archived entity, when queried, then archive location returned

**Data Requirements**:
- **Inputs**: Entity ID for archival
- **Outputs**: Archive confirmation with location
- **Validations**: Retention period met, entity inactive

**Priority**: SHOULD

**Complexity**: MEDIUM

**Dependencies**: FR-MREG-1

**Assumptions**: CDD+ API accessible with authentication

---

#### FR-MREG-13: Multi-Tenancy

**Description**: System shall support multiple organizations with data isolation

**Relates To**: BR-MREG-004

**Acceptance Criteria**:
- Given request from Organization A, when query executed, then only Organization A data returned
- Given organization admin, when user created, then user assigned to organization
- Given cross-organization query, when attempted, then results filtered or rejected

**Data Requirements**:
- **Inputs**: User token with organization claim
- **Outputs**: Organization-scoped data
- **Validations**: Organization claim required

**Priority**: MUST

**Complexity**: HIGH

**Dependencies**: FR-MREG-1, FR-MREG-5

**Assumptions**: Organization identifier in user token

---

#### FR-MREG-14: Value List Management

**Description**: System shall manage standard value lists (provincie, gemeente, etc.)

**Relates To**: BR-MREG-001

**Acceptance Criteria**:
- Given value list query, when GET /api/v2/valuelists/:id, then list items returned
- Given value list item, when validated, then value checked against list
- Given admin user, when value list updated, then new version created

**Data Requirements**:
- **Inputs**: Value list ID, item values
- **Outputs**: Value list items
- **Validations**: Standard government value lists

**Priority**: SHOULD

**Complexity**: LOW

**Dependencies**: FR-MREG-1

**Assumptions**: Standard value lists seeded at initialization

---

#### FR-MREG-15: Subject Access Request (SAR) Support

**Description**: System shall support AVG SAR requests for personal data in metadata

**Relates To**: BR-MREG-015

**Acceptance Criteria**:
- Given SAR request, when submitted, then all PII-containing entities returned
- Given SAR for specific person, when queried, then entities with that person returned
- Given SAR export, when generated, then data provided in machine-readable format

**Data Requirements**:
- **Inputs**: Person identifier (BSN or name)
- **Outputs**: All metadata containing PII for that person
- **Validations**: Authorized requester only

**Priority**: MUST

**Complexity**: MEDIUM

**Dependencies**: FR-MREG-1, FR-MREG-13

**Assumptions**: PII detected and tagged during entity creation

---

#### FR-MREG-16: Dynamic vs Persistent Status Management

**Description**: System shall distinguish between dynamic (in bewerking) and persistent (gepersistent) information objects per BSW architecture

**Relates To**: BR-MREG-026

**Acceptance Criteria**:
- Given entity created, when status=dynamisch, then entity is mutable
- Given entity persisted, when status=gepersistent, then entity becomes read-only
- Given workflow transition, when entity finalized, then status changes to gepersistent
- Given persistent entity, when modification attempted, then error returned
- Given gepersistent entity, when archived, then status becomes gearchiveerd

**Data Requirements**:
- **Fields**: status (enum: dynamisch, gepersistent, gearchiveerd)
- **Transitions**: dynamisch → gepersistent → gearchiveerd (irreversible)
- **Validation**: Status change requires audit trail entry with business reason

**Priority**: MUST

**Complexity**: MEDIUM

**Dependencies**: FR-MREG-1, FR-MREG-6

**Assumptions**: Status changes are logged in audit trail for compliance

---

#### FR-MREG-17: Information Object Catalogus

**Description**: System shall maintain a catalog storing metadata and location references to information objects in archives

**Relates To**: BR-MREG-027

**Acceptance Criteria**:
- Given information object archived, when location determined, then catalog entry created
- Given catalog query, when search executed, then metadata and location URI returned
- Given object location changes, when updated, then catalog locatie_uri updated
- Given catalog entry, when object deleted, then entry removed

**Data Requirements**:
- **Entity**: InformatieobjectCatalogus
- **Fields**: informatieobject_id, locatie_uri, zoek_index, context_metadata
- **Relationships**: Many-to-one with Informatieobject, many-to-one with Zaak

**Priority**: MUST

**Complexity**: HIGH

**Dependencies**: FR-MREG-1, FR-MREG-12

**Assumptions**: Catalog is separate from content storage (CDD+)

---

#### FR-MREG-18: Context-Aware Search

**Description**: System shall provide search combining user context with information origin context per BSW requirements

**Relates To**: BR-MREG-028

**Acceptance Criteria**:
- Given search from zaak context, when query executed, then results prioritized from same zaak
- Given search with werkproces context, when query executed, then results filtered by process
- Given user context (role, organization), when query executed, then results scoped appropriately
- Given search without context, when query executed, then all accessible results returned
- Given context dimensions, when combined, then relevance_score = context_match × text_relevance

**Data Requirements**:
- **Context Dimensions**: User (role, org, active zaak), Information (zaak_id, werkproces, creatie_datum)
- **Search Inputs**: zoekterm, context_filters, pagination
- **Outputs**: Ranked results with context_match_score

**Priority**: MUST

**Complexity**: HIGH

**Dependencies**: FR-MREG-1, FR-MREG-17

**Assumptions**: Context metadata is available for all information objects

---

#### FR-MREG-19: Information Object Level Authorization

**Description**: System shall support authorization grants at individual information object level for collaboration scenarios

**Relates To**: BR-MREG-029

**Acceptance Criteria**:
- Given information object, when shared with user, then specific access granted
- Given shared object, when owner revokes access, then access removed immediately
- Given multi-caretaker scenario, when object archived, then preserved for each caretaker
- Given user with object grant, when query executed, then only accessible objects returned
- Given zaak-level rights, when assigned, then inherited by contained objects

**Data Requirements**:
- **Entity**: InformatieobjectRecht (informatieobject_id, user_id, recht_type: lezen/bewerken)
- **Inheritance**: Zaak-level rights inherit to contained objects
- **Caching**: Rights cached for performance with invalidation on change

**Priority**: MUST

**Complexity**: HIGH

**Dependencies**: FR-MREG-1, FR-MREG-5, FR-MREG-13

**Assumptions**: User identity available from authentication token

---

#### FR-MREG-20: Metadata Inheritance

**Description**: Metadata from zaak/dossier shall be inheritable by contained information objects

**Relates To**: BR-MREG-030

**Acceptance Criteria**:
- Given zaak with metadata, when informatieobject created within zaak, then inherited metadata available
- Given inherited metadata, when zaak metadata updated, then objects can inherit new values
- Given information object, when displayed, then inherited and own metadata both visible
- Given inheritance conflict, when both defined, then object-level metadata takes precedence

**Data Requirements**:
- **Edge Collection**: inherits_from with inheritance_type (overerven, overschrijven)
- **Metadata**: zaak_id, organisatie_id, beveiligingsniveau (inheritable)

**Priority**: SHOULD

**Complexity**: MEDIUM

**Dependencies**: FR-MREG-1, FR-MREG-2

**Assumptions**: Zaak entity exists with inheritable metadata fields

---

#### FR-MREG-21: Workflow Status Integration

**Description**: System shall support external workflow status management for information objects

**Relates To**: BR-MREG-031

**Acceptance Criteria**:
- Given external workflow, when status changes, then entity updated with workflow_status
- Given workflow transition to gepersistent, when triggered, then entity becomes read-only
- Given workflow event, when received, then audit trail updated with workflow context
- Given workflow status query, when requested, then current workflow state returned

**Data Requirements**:
- **Fields**: workflow_status, workflow_id, workflow_last_update
- **Integration**: Event-driven status updates via webhook/message queue

**Priority**: SHOULD

**Complexity**: MEDIUM

**Dependencies**: FR-MREG-1, FR-MREG-6

**Assumptions**: External workflow system sends status change events

---

#### FR-MREG-22: AI Result Validation

**Description**: AI enrichment results must support human validation before trusted use per BSW requirements

**Relates To**: BR-MREG-032

**Acceptance Criteria**:
- Given AI enrichment completed, when result created, then ai_status=ongecontroleerd
- Given human validation, when approved, then ai_status=getoetst and validator_id stored
- Given unvalidated result, when used in critical process (Woo publication), then warning displayed
- Given AI result, when rejected, then alternatieve_waarde stored

**Data Requirements**:
- **AI Fields**: ai_status (ongecontroleerd, getoetst, afgewezen), getoetst_door, getoetst_op, vertrouwensscore (0-1)
- **Validation**: Required for Woo publication automation

**Priority**: MUST

**Complexity**: MEDIUM

**Dependencies**: Phase 6 AI entities (AIEnrichment, AISummary)

**Assumptions**: Human validation workflow exists in admin UI

---

#### FR-MREG-23: Multi-Caretaker Support

**Description**: System shall support information objects archived under multiple caretakers in chain scenarios

**Relates To**: BR-MREG-033

**Acceptance Criteria**:
- Given shared object in chain, when archived, then preserved for each caretaker organization
- Given multi-caretaker object, when queried, then all caretaker locations visible
- Given caretaker separation, when object deleted, then other caretaker copies preserved
- Given transfer between caretakers, when executed, then ownership transferred with audit trail

**Data Requirements**:
- **Entity**: InformatieobjectZorgdrager (informatieobject_id, organisatie_id, rol: primair/secundair)
- **Relationships**: One object can have multiple zorgdrager entries

**Priority**: SHOULD

**Complexity**: MEDIUM

**Dependencies**: FR-MREG-1, FR-MREG-12

**Assumptions**: CDD+ supports multi-caretaker scenarios

---

## Non-Functional Requirements

### Performance Requirements

#### NFR-MREG-P-1: API Response Time

**Requirement**: API response time for metadata queries

- Read operations (single entity): <50ms (95th percentile)
- Read operations (list with pagination): <200ms (95th percentile)
- Graph traversal (depth 1-3): <500ms (95th percentile)
- Write operations (create/update): <100ms (95th percentile)
- Full-text search: <1 second (95th percentile)

**Measurement Method**: APM instrumentation with percentile metrics

**Load Conditions**:
- Peak load: 100 concurrent API requests
- Average load: 20 requests/second
- Data volume: 1M entities in database

**Priority**: CRITICAL

---

#### NFR-MREG-P-2: Database Query Performance

**Requirement**: ArangoDB query performance with indexes

- Single entity lookup by ID: <10ms (average)
- Time-based validity filtered query: <50ms (95th percentile)
- Graph traversal (3 hops, 1000 nodes): <100ms (95th percentile)
- Full-text search query: <500ms (95th percentile)

**Measurement Method**: ArangoDB query profiling and slow query logs

**Priority**: CRITICAL

---

#### NFR-MREG-P-3: Concurrent User Support

**Requirement**: Support concurrent users without degradation

- 100 concurrent API clients
- 50 concurrent admin UI users
- No performance degradation >20% at peak load

**Priority**: HIGH

---

### Availability and Resilience Requirements

#### NFR-MREG-A-1: Availability Target

**Requirement**: System must achieve 99.5% uptime (SLA)

- Maximum planned downtime: 4 hours/month for maintenance
- Maximum unplanned downtime: 22 hours/year
- Maintenance windows: Sunday 02:00-06:00 CET

**Priority**: HIGH

---

#### NFR-MREG-A-2: Disaster Recovery

**RPO (Recovery Point Objective)**: Maximum acceptable data loss = 1 hour

**RTO (Recovery Time Objective)**: Maximum acceptable downtime = 4 hours

**Backup Requirements**:
- Backup frequency: Hourly for ArangoDB
- Backup retention: 30 days online, 7 years archival
- Geographic backup location: Netherlands/EU region

**Failover Requirements**:
- Automatic failover: NO (manual procedure)
- Failover time: <4 hours with documented runbook

**Priority**: MUST

---

#### NFR-MREG-A-3: Fault Tolerance

**Requirement**: System must gracefully degrade when components fail

**Resilience Patterns Required**:
- [x] Retry with exponential backoff for ArangoDB queries
- [x] Timeout on all network calls (30s default)
- [x] Graceful degradation: read-only mode when write DB unavailable
- [x] Circuit breaker for external API calls (CDD+, Woo portal)
- [ ] Bulkhead isolation for critical resources

**Priority**: SHOULD

---

### Scalability Requirements

#### NFR-MREG-S-1: Horizontal Scaling

**Requirement**: System must support horizontal scaling

**Growth Projections**:
- Year 1: 1M entities, 50 organizations, 100K queries/day
- Year 2: 5M entities, 200 organizations, 500K queries/day
- Year 3: 10M entities, 500 organizations, 1M queries/day

**Scaling Triggers**: Auto-scale when CPU >70% or memory >80%

**Priority**: SHOULD

---

#### NFR-MREG-S-2: Data Volume Scaling

**Requirement**: System must handle data growth to 100M entities over 5 years

**Data Archival Strategy**:
- Hot data (active entities): Primary ArangoDB instance
- Warm data (expired entities): Separate ArangoDB database
- Cold data (archived entities): CDD+ or cold storage

**Priority**: SHOULD

---

### Security Requirements

#### NFR-MREG-SEC-1: Authentication

**Requirement**: All users must authenticate via OAuth 2.0 / OpenID Connect

**Multi-Factor Authentication (MFA)**:
- Required for: Admin users, privileged operations
- MFA methods: Authenticator app (TOTP)

**Session Management**:
- Session timeout: 8 hours of inactivity
- Absolute session timeout: 24 hours
- Re-authentication required for: Entity deletion, organization settings

**Priority**: MUST

---

#### NFR-MREG-SEC-2: Authorization

**Requirement**: Role-based access control (RBAC) with least privilege principle

**Roles and Permissions**:
- **viewer**: Read-only access to own organization entities
- **editor**: Create/update entities in own organization
- **admin**: Full access to own organization, user management
- **system_admin**: Full access across all organizations

**Privilege Elevation**: Temporary elevation requires documented approval

**Priority**: MUST

---

#### NFR-MREG-SEC-3: Data Encryption

**Requirement**:
- Data in transit: TLS 1.3+ with strong cipher suites
- Data at rest: ArangoDB encryption (AES-256)
- Key management: Environment variables or HashiCorp Vault

**Encryption Scope**:
- [x] Database encryption at rest (ArangoDB)
- [x] Backup encryption
- [ ] Application-level field encryption for PII (deferred to Phase 2)

**Priority**: MUST

---

#### NFR-MREG-SEC-4: Secrets Management

**Requirement**: No secrets in code or configuration files

**Secrets Storage**: Environment variables in production, HashiCorp Vault for larger deployments

**Secrets Rotation**: Quarterly for ArangoDB credentials, API keys

**Priority**: SHOULD

---

#### NFR-MREG-SEC-5: Vulnerability Management

**Requirement**:
- Dependency scanning in CI/CD pipeline (cargo-audit)
- No critical vulnerabilities in deployment
- Security review for major releases

**Remediation SLA**:
- Critical vulnerabilities: 7 days
- High vulnerabilities: 30 days

**Priority**: SHOULD

---

### Compliance and Regulatory Requirements

#### NFR-MREG-C-1: AVG/GDPR Compliance

**Applicable Regulations**: AVG (Algemene verordening gegevensbescherming) / GDPR

**Compliance Requirements**:
- [x] Data subject rights (SAR endpoint via FR-MREG-15)
- [x] Privacy by design (PII classification, Row-Level Security)
- [x] Data breach notification within 72 hours
- [x] DPIA completed for high-risk processing

**Data Residency**: All data stored within EU (Netherlands region preferred)

**Data Retention**: Metadata retained per Archiefwet requirements (5-20 years)

**Priority**: MUST

---

#### NFR-MREG-C-2: Audit Logging

**Requirement**: Comprehensive audit trail for compliance and forensics

**Audit Log Contents**:
- Who: User identity (user ID, organization)
- What: Action performed (create, update, delete)
- When: Timestamp (UTC, millisecond precision)
- Where: System component (API, UI, GitOps)
- Why: Business context (request ID, Git commit)
- Result: Success/failure with error details

**Log Retention**: 7 years for compliance logs (immutable storage)

**Log Integrity**: Tamper-evident logging (append-only, signed hashes)

**Priority**: MUST

---

#### NFR-MREG-C-3: Woo Compliance

**Requirement**: System must support Wet open overheid (Woo) publication workflow

**Compliance Requirements**:
- [x] Automatic Woo relevance assessment for Informatieobject
- [x] Human approval required before publication
- [x] Refusal grounds tracking for non-publication
- [x] Publication deadline tracking

**Priority**: MUST

---

### Maintainability and Supportability Requirements

#### NFR-MREG-M-1: Observability

**Requirement**: Comprehensive instrumentation for monitoring and troubleshooting

**Telemetry Requirements**:
- **Logging**: Structured JSON logs (tracing), centralized aggregation
- **Metrics**: Prometheus-compatible, RED metrics (Rate, Errors, Duration)
- **Tracing**: OpenTelemetry distributed tracing
- **Dashboards**: Grafana dashboards for key metrics
- **Alerts**: SLO-based alerting with runbooks

**Log Levels**: ERROR, WARN, INFO, DEBUG, TRACE

**Priority**: SHOULD

---

#### NFR-MREG-M-2: Documentation

**Requirement**: Comprehensive documentation for operators and developers

**Documentation Types**:
- [x] API documentation (OpenAPI 3.0 specs)
- [x] Architecture documentation (C4 model)
- [ ] Runbooks for operational procedures (TBD)
- [ ] Troubleshooting guides (TBD)
- [ ] Admin user manual (TBD)

**Documentation Format**: Markdown in repository

**Priority**: SHOULD

---

#### NFR-MREG-M-3: Operational Runbooks

**Requirement**: Runbooks for common operational tasks

**Runbook Coverage**:
- [x] Deployment procedures (docker-compose, Kubernetes)
- [x] Backup and restore procedures
- [ ] Incident response for common failures (TBD)
- [ ] Scaling procedures (TBD)

**Priority**: SHOULD

---

### Portability and Interoperability Requirements

#### NFR-MREG-I-1: API Standards

**Requirement**: All APIs must follow OpenAPI 3.0 standards

**API Design Principles**:
- RESTful design with standard HTTP methods
- JSON request/response format
- Versioning via URL path (/api/v2/)
- Consistent error response format
- HATEOAS links for related resources

**Priority**: MUST

---

#### NFR-MREG-I-2: GraphQL Schema

**Requirement**: GraphQL API with auto-generated schema from GGHH entities

**Schema Design**:
- Entity types for all GGHH V2 entities
- Relationship fields for edge collections
- Filtering and sorting arguments
- Pagination for list queries

**Priority**: SHOULD

---

#### NFR-MREG-I-3: Data Export

**Requirement**: Support for metadata export in standard formats

**Export Formats**: JSON, CSV, YAML

**Export Scope**: Complete organization export or filtered by entity type

**Priority**: COULD

---

## Integration Requirements

### External System Integrations

#### INT-MREG-1: Integration with ArangoDB

**Purpose**: Primary graph database storage

**Integration Type**: Direct connection via arangors driver

**Data Exchanged**:
- **From Service to ArangoDB**: Entity CRUD operations, graph traversals, AQL queries
- **From ArangoDB to Service**: Query results, transaction status

**Integration Pattern**: Request/response with connection pooling

**Authentication**: Username/password (root user for admin, app user for operations)

**Error Handling**: Retry with exponential backoff, circuit breaker for connection failures

**SLA**: <10ms for single entity read, <100ms for graph traversal

**Owner**: DevOps Team

**Priority**: MUST

---

#### INT-MREG-2: Integration with Git Repository

**Purpose**: GitOps-based metadata synchronization

**Integration Type**: Webhook + Git client operations

**Data Exchanged**:
- **From Git to Service**: Webhook notifications on commit, file contents (YAML)
- **From Service to Git**: Status updates (optional)

**Integration Pattern**: Event-driven (webhook triggers sync)

**Authentication**: SSH key or deploy token

**Error Handling**: Queue failed syncs for retry, manual resolution for merge conflicts

**SLA**: Sync initiated within 1 minute of commit

**Owner**: DevOps Team

**Priority**: SHOULD

---

#### INT-MREG-3: Integration with Woo Portal

**Purpose**: Publication of Woo-relevant documents

**Integration Type**: REST API client

**Data Exchanged**:
- **From Service to Woo Portal**: Publication requests, document metadata
- **From Woo Portal to Service**: Publication confirmation, status updates

**Integration Pattern**: Request/response with async status polling

**Authentication**: OAuth 2.0 client credentials

**Error Handling**: Retry for transient failures, manual intervention for publication failures

**SLA**: Publication request submitted within 5 seconds

**Owner**: Woo Officers

**Priority**: MUST

---

#### INT-MREG-4: Integration with CDD+ Archive

**Purpose**: Long-term archival of expired metadata

**Integration Type**: REST API client

**Data Exchanged**:
- **From Service to CDD+**: Archival packages (metadata + references)
- **From CDD+ to Service**: Archive confirmation, retrieval requests

**Integration Pattern**: Batch archival (nightly job)

**Authentication**: Mutual TLS or API key

**Error Handling**: Queue failed archival for retry, alert on repeated failures

**SLA**: Archival initiated within 24 hours of retention expiry

**Owner**: Information Managers

**Priority**: SHOULD

---

## Data Requirements

### Data Entities

#### Entity 1: Gebeurtenis (Event)

**Description**: Represents an event or occurrence in the government context

**Attributes**:
| Attribute | Type | Required | Description | Constraints |
|-----------|------|----------|-------------|-------------|
| _key | String | Yes | Unique identifier | Primary key, auto-generated |
| naam | String | Yes | Event name | Not null |
| omschrijving | String | No | Event description | Text field |
| geldig_vanaf | DateTime | Yes | Valid from date | ISO 8601, indexed |
| geldig_tot | DateTime | No | Valid to date | ISO 8601, indexed, nullable |
| organisatie_id | String | Yes | Owner organization | Foreign key to organization |

**Relationships**:
- Many-to-many with Gebeurtenis via gebeurtenis_product edge
- Many-to-many with Context via gebeurtenis_context edge
- Many-to-many with Grondslag via gebeurtenis_grondslag edge

**Data Volume**: 50K records Year 1, 200K Year 3

**Access Patterns**: Query by validity date, by organization, graph traversal to related entities

**Data Classification**: INTERNAL

**Data Retention**: 20 years (Archiefwet requirement for Besluit)

---

#### Entity 2: Informatieobject (Information Object)

**Description**: BSW-compliant information object (dataobject + metadata) - core abstraction per BSW architecture

**Attributes**:
| Attribute | Type | Required | Description | Constraints |
|-----------|------|----------|-------------|-------------|
| _key | String | Yes | Unique identifier | Primary key |
| naam | String | Yes | Object name | Not null |
| objecttype | String | Yes | Document type | Enum: Document, Email, Chat, Besluit, Data |
| documenttype | String | Yes | Document type classification | Enum: standard types (for search) |
| beveiligingsniveau | String | Yes | Security level | Enum: Openbaar, Intern, Vertrouwelijk, Geheim |
| privacy_level | String | No | Privacy classification | Enum: Openbaar, Normaal, Bijzonder, Strafrechtelijk |
| informatiecategorie | String | Yes | Woo information category | Enum: Woo categories |
| is_woo_relevant | Boolean | Yes | Woo relevance flag | Default: false |
| zaak_id | String | No | Related case/dossier | Foreign key to Zaak |
| samenvatting | Text | No | AI-generated summary | Indexable, validated |
| inhoud | Text | No | Content text | Full-text indexed |
| status | String | Yes | Storage status | Enum: dynamisch, gepersistent, gearchiveerd |
| geldig_vanaf | DateTime | Yes | Valid from date | Indexed |
| geldig_tot | DateTime | No | Valid to date | Indexed, nullable |

**Relationships**:
- Many-to-many with Bewaartermijn via informatieobject_bewaartermijn edge
- Many-to-one with WooPublicatie via woo_informatieobject edge
- Many-to-many with Context via informatieobject_context edge

**Data Volume**: 5M records Year 1, 20M Year 3

**Access Patterns**: Full-text search, Woo relevance query, security filtering

**Data Classification**: varies by beveiligingsniveau

**Data Retention**: Per document type (1-20 years per Archiefwet)

---

#### Entity 3: WooPublicatie

**Description**: Tracks Woo publication workflow for information objects

**Attributes**:
| Attribute | Type | Required | Description | Constraints |
|-----------|------|----------|-------------|-------------|
| _key | String | Yes | Unique identifier | Primary key |
| informatieobject_id | String | Yes | Reference to Informatieobject | Foreign key |
| status | String | Yes | Publication status | Enum: voorgelegd, goedgekeurd, gepubliceerd, afgewezen |
| afgewezen_grond | String | No | Refusal grounds | Required if status=afgewezen |
| goedgekeurd_door | String | No | Approver user ID | Required for approval |
| goedgekeurd_op | DateTime | No | Approval timestamp | |
| gepubliceerd_op | DateTime | No | Publication timestamp | |
| woo_url | String | No | Woo portal URL | After publication |

**Relationships**:
- Many-to-one with Informatieobject
- Many-to-one with Context (via woo_publicatie_context)
- Many-to-one with Grondslag (via woo_publicatie_grondslag)

**Data Volume**: 100K records Year 1 (10% of Informatieobjecten)

**Access Patterns**: Query by status, by approver, by date range

**Data Classification**: OFFICIAL

**Data Retention**: 20 years (permanent record of Woo decisions)

---

#### Entity 4: Zaak (Case/Dossier)

**Description**: BSW case/dossier for grouping and metadata inheritance per zaak/dossierbeheer requirements

**Attributes**:
| Attribute | Type | Required | Description | Constraints |
|-----------|------|----------|-------------|-------------|
| _key | String | Yes | Unique identifier | Primary key, org-unique |
| type_zaak | String | Yes | Case/dossier type | Enum: standard types (Woo required) |
| naam | String | Yes | Dossier name | Short, descriptive (Woo required) |
| omschrijving | Text | No | Dossier description | Context for later retrieval |
| onderwerp | String | No | Subject/topic | Facilitates search |
| organisatie_id | String | Yes | Owner organization | Foreign key |
| geldig_vanaf | DateTime | Yes | Valid from date | ISO 8601, indexed |
| geldig_tot | DateTime | No | Valid to date | ISO 8601, indexed, nullable |

**Relationships**:
- One-to-many with Informatieobject via zaak_informatieobject edge
- Many-to-one with Gebeurtenis (zaak creation trigger)
- Many-to-one with Bedrijfsproces (workflow context)

**Inheritable Metadata**: organisatie_id, beveiligingsniveau (can inherit to contained Informatieobjecten)

**Data Volume**: 500K records Year 1, 2M Year 3

**Access Patterns**: Query by type, by organization, graph traversal to contained objects

**Data Classification**: varies by case type

**Data Retention**: Per applicable retention schedule (10-20 years per Archiefwet)

---

#### Entity 5: InformatieobjectCatalogus (Information Object Catalog)

**Description**: Stores metadata and location references to information objects in archives per BSW informatiecataloog requirements

**Attributes**:
| Attribute | Type | Required | Description | Constraints |
|-----------|------|----------|-------------|-------------|
| _key | String | Yes | Unique identifier | Primary key |
| informatieobject_id | String | Yes | Reference to Informatieobject | Foreign key |
| locatie_uri | String | Yes | Storage location | CDD+, file system, or other |
| locatie_type | String | Yes | Location type | Enum: CDD, file_system, other |
| zoek_index | Text | Yes | Full-text searchable content | Indexed |
| context_metadata | JSON | Yes | Context for search | zaak, werkproces, etc. |
| geindexeerd_op | DateTime | Yes | Index timestamp | ISO 8601 |
| laatst_gebruikt | DateTime | No | Last access timestamp | For relevance scoring |

**Relationships**:
- Many-to-one with Informatieobject
- Many-to-one with Zaak (via catalogus_zaak edge)
- Many-to-one with Organisatie

**Data Volume**: 1:1 with Informatieobject (minus active/dynamic objects)

**Access Patterns**: Context-aware search, location lookup, relevance ranking

**Data Classification**: same as source Informatieobject

**Data Retention**: Synchronized with source object (removed when source deleted/archived)

---

#### Entity 6: InformatieobjectRecht (Information Object Right)

**Description**: Object-level authorization grants for collaboration scenarios per BSW samenwerking requirements

**Attributes**:
| Attribute | Type | Required | Description | Constraints |
|-----------|------|----------|-------------|-------------|
| _key | String | Yes | Unique identifier | Primary key |
| informatieobject_id | String | Yes | Reference to Informatieobject | Foreign key |
| user_id | String | Yes | Granted user identifier | From authentication |
| recht_type | String | Yes | Access right type | Enum: lezen, bewerken |
| granted_by | String | Yes | Grantor user ID | Who granted access |
| granted_at | DateTime | Yes | Grant timestamp | ISO 8601 |
| expires_at | DateTime | No | Expiration timestamp | Nullable |
| inheritance_source | String | No | Source if inherited | From zaak-level grant |

**Relationships**:
- Many-to-one with Informatieobject
- Many-to-one with User (via external auth)
- Many-to-one with Zaak (for inherited rights)

**Data Volume**: 2-5x Informatieobject volume (shared objects)

**Access Patterns**: Authorization check per query, user's accessible objects list

**Data Classification**: INTERNAL (security metadata)

**Data Retention**: 7 years (AVG compliance)

---

### Data Quality Requirements

**Data Accuracy**: Entity attributes must conform to GGHH V2 schema definitions

**Data Completeness**: Required fields must be non-null, referential integrity enforced

**Data Consistency**: Time-based validity must not overlap (geldig_tot < next geldig_vanaf)

**Data Timeliness**: GitOps sync within 1 minute of commit

**Data Lineage**: Audit trail records source of all changes (API, UI, GitOps)

---

### Data Migration Requirements

**Migration Scope**: Seed data for standard value lists (provincie, gemeente, etc.)

**Migration Strategy**: Run database migrations on startup via ArangoDB migration system

**Data Transformation**: YAML to JSON entity conversion for GitOps

**Data Validation**: GGHH schema validation before entity creation

**Rollback Plan**: Migration rollback via ArangoDB snapshot restore

**Migration Timeline**: Initial setup <5 minutes

---

## Constraints and Assumptions

### Technical Constraints

**TC-1**: Must use Rust for API service (existing codebase)

**TC-2**: Must use ArangoDB for graph storage (existing infrastructure)

**TC-3**: Must deploy via Docker or Kubernetes (containerization requirement)

**TC-4**: Must comply with Sovereign Technology principle (open-source only)

---

### Business Constraints

**BC-1**: Must maintain AVG/GDPR compliance at all times (legal requirement)

**BC-2**: Must support Archiefwet retention periods (legal requirement)

**BC-3**: Must support Woo publication workflow (legal requirement)

**BC-4**: Budget limited to open-source solutions (no commercial database licenses)

---

### Assumptions

**A-1**: ArangoDB cluster available for production deployment

**A-2**: Git repository accessible for GitOps configuration

**A-3**: Woo portal API available for publication

**A-4**: CDD+ API available for archival (or manual process acceptable)

**A-5**: Network latency to ArangoDB <5ms (same datacenter)

**A-6**: External workflow system available for integration (optional, SHOULD requirement)

**A-7**: User authentication system provides user identity for object-level authorization

**Validation Plan**:
- Infrastructure validation in Sprint 1
- API integration testing in Sprint 2
- BSW architecture compliance review in Sprint 3
- User acceptance testing with BSW scenarios in Sprint 4

---

## Success Criteria and KPIs

### Business Success Metrics

| Metric | Baseline | Target | Timeline | Measurement Method |
|--------|----------|--------|----------|-------------------|
| GGHH Compliance | 0% | 95% | 6 months | Entity coverage analysis |
| BSW Architecture Alignment | 0% | 100% | 3 months | BSW principles checklist |
| Metadata Duplication | 100% | <20% | 12 months | Cross-system audit |
| Woo Publication Time | 14 days | <5 days | 6 months | Workflow timestamp analysis |
| SAR Response Time | 30 days | <7 days | 3 months | API response tracking |
| Context-Aware Search Accuracy | N/A | 80%+ | 6 months | User feedback scoring |

---

### Technical Success Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| System availability | 99.5% | Uptime monitoring |
| API response time (p95) | <200ms | APM tooling |
| Graph traversal (p95) | <500ms | ArangoDB profiling |
| Error rate | <0.1% | Log aggregation |
| Dynamic/Persistent transition time | <1s | Workflow timing |
| Catalog search latency (p95) | <300ms | Search performance metrics |

---

### User Adoption Metrics

| Metric | Target | Timeline | Measurement Method |
|--------|--------|----------|-------------------|
| Active organizations | 50 | 6 months | Organization registration |
| API calls per day | 10K | 6 months | API analytics |
| Admin UI active users | 100 | 6 months | User activity tracking |
| Zaak/Dossier creation rate | N/A | Baseline +20% | 12 months | Process efficiency analysis |
| Object-level sharing usage | 0 | 30% of users | 6 months | Feature adoption tracking |

---

## Requirements Traceability Matrix

### Business → Functional Traceability

| Business Req | Functional Reqs | Data Model Entities |
|--------------|-----------------|-------------------|
| BR-MREG-001 to BR-MREG-003 (GGHH compliance) | FR-MREG-1, FR-MREG-2, FR-MREG-4 | All GGHH V2 entities |
| BR-MREG-002 (Time-based validity) | FR-MREG-3 | geldig_vanaf/geldig_tot fields |
| BR-MREG-004 (Multi-tenancy) | FR-MREG-5, FR-MREG-13 | organisatie_id field |
| BR-MREG-005 (Audit trail) | FR-MREG-6 | Audit entity |
| BR-MREG-006 (TOOI/MDTO) | FR-MREG-4 | Validation tables |
| BR-MREG-007 (GitOps) | FR-MREG-10 | Git sync state |
| BR-MREG-008 (CDD+ archive) | FR-MREG-12 | Archive metadata |
| BR-MREG-009 (Woo workflow) | FR-MREG-9 | WooPublicatie entity |
| BR-MREG-010 (APIs) | FR-MREG-7, FR-MREG-8 | API endpoint mappings |
| BR-MREG-011 to BR-MREG-018 (Privacy/Compliance) | FR-MREG-5, FR-MREG-15 | privacy_level, PII fields |
| BR-MREG-026 to BR-MREG-034 (BSW compliance) | FR-MREG-16 to FR-MREG-23 | BSW-specific entities and features |

### Functional → NFR Traceability

| Functional Req | Related NFRs |
|----------------|--------------|
| FR-MREG-1 to FR-MREG-3 (CRUD, Graph, Validity) | NFR-MREG-P-1, NFR-MREG-P-2 (Performance) |
| FR-MREG-5, FR-MREG-13 (Security, Multi-tenancy) | NFR-MREG-SEC-1 to NFR-MREG-SEC-5 (Security) |
| FR-MREG-6 (Audit trail) | NFR-MREG-C-2 (Audit logging) |
| FR-MREG-9 (Woo workflow) | NFR-MREG-C-3 (Woo compliance) |
| FR-MREG-11 (Admin UI) | NFR-MREG-U-1 (Usability) |
| FR-MREG-15 (SAR) | NFR-MREG-C-1 (AVG/GDPR) |
| FR-MREG-16 to FR-MREG-23 (BSW features) | NFR-MREG-C-1, NFR-MREG-C-2, NFR-MREG-SEC-1 to NFR-MREG-SEC-5 |

---

## Requirement Conflicts & Resolutions

### Conflict MREG-C-1: Performance vs. Completeness

**Conflicting Requirements**:
- **FR-MREG-2**: Graph traversal queries to depth 5 with <500ms response time
- **NFR-MREG-C-2**: Complete audit trail for all operations

**Stakeholders Involved**:
- **DevOps Lead** (DevOps): Wants FR-MREG-2 for responsive UI
- **DPO** (Compliance): Wants NFR-MREG-C-2 for complete accountability

**Nature of Conflict**:
Deep graph traversals with complete audit logging degrade performance. Logging every edge traversal creates significant overhead.

**Trade-off Analysis**:

| Option | Pros | Cons | Impact |
|--------|------|------|--------|
| **Option 1**: Prioritize Performance (skip audit for read queries) | ✅ Fast graph queries<br>✅ Better UX | ❌ Incomplete audit trail<br>❌ AVG compliance risk | DevOps happy<br>DPO concerned |
| **Option 2**: Prioritize Compliance (audit all queries) | ✅ Complete audit trail<br>✅ AVG compliant | ❌ Slower queries<br>❌ User frustration | DPO happy<br>DevOps concerned |
| **Option 3**: Compromise (audit writes only, async logging) | ✅ Fast queries<br>✅ Complete audit<br>✅ Both satisfied | ❌ Async complexity<br>❌ Slight delay in audit visibility | Both satisfied |

**Resolution Strategy**: COMPROMISE

**Decision**: Option 3 - Async audit logging for read operations, synchronous for writes

**Rationale**:
- DPO accepts async logging for reads as long as audit trail is complete
- DevOps accepts minor complexity for query performance
- Synchronous audit for writes ensures mutation accountability

**Decision Authority**: Product Owner with DPO consultation

**Impact on Requirements**:
- **Unchanged**: FR-MREG-2 - 500ms target maintained
- **Modified**: NFR-MREG-C-2 - Clarified async audit for reads, sync for writes
- **Added**: NFR-MREG-M-4 requirement for async audit queue

**Stakeholder Management**:
- **DPO**: Accepts async logging with guarantee of completeness (no dropped audits)
- **DevOps**: Accepts async queue complexity with monitoring for queue depth

**Future Consideration**:
- Monitor async queue depth and processing lag
- Alert if audit processing falls behind

---

### Conflict MREG-C-2: Open Source vs. AI Features

**Conflicting Requirements**:
- **BR-MREG-023**: Must use open-source technology stack
- **BR-MREG-013**: PII detection via AI (requires commercial APIs)

**Stakeholders Involved**:
- **CIO** (Architecture): Wants BR-MREG-023 for digital sovereignty
- **DPO** (Compliance): Wants BR-MREG-013 for privacy protection

**Nature of Conflict**:
PII detection requires advanced AI models (OpenAI, Anthropic) which are commercial services, conflicting with open-source preference.

**Trade-off Analysis**:

| Option | Pros | Cons | Impact |
|--------|------|------|--------|
| **Option 1**: Open-source only (local models) | ✅ Full sovereignty<br>✅ No external deps | ❌ Lower accuracy<br>❌ Higher infra cost<br>❌ Maintenance burden | CIO happy<br>DPO concerned |
| **Option 2**: Commercial AI APIs | ✅ Best accuracy<br>✅ Low infra cost | ❌ Vendor lock-in<br>❌ Data sovereignty risk | DPO happy<br>CIO concerned |
| **Option 3**: Hybrid (commercial for MVP, local later) | ✅ Fast MVP delivery<br>✅ Exit strategy | ❌ Two migrations<br>❌ Higher total cost | Both somewhat satisfied |

**Resolution Strategy**: PHASE

**Decision**: Option 3 - Phase 1: Commercial APIs (with EU processing guarantee), Phase 2: Local models

**Rationale**:
- DPO accepts commercial APIs with EU data processing guarantee (addresses AVG concern)
- CIO accepts temporary commercial dependency with clear migration path to open-source
- Phase 1 enables fast delivery; Phase 2 fulfills sovereignty principle

**Decision Authority**: Architecture Board with CIO and DPO consultation

**Impact on Requirements**:
- **Modified**: BR-MREG-013 - PII detection via commercial APIs (Phase 1), local models (Phase 2)
- **Added**: FR-MREG-16 - Support pluggable AI backends
- **Added**: BR-MREG-026 - Migration to open-source AI models by Q4 2026

**Stakeholder Management**:
- **CIO**: Accepts temporary commercial dependency with written migration commitment
- **DPO**: Accepts commercial APIs with EU processing guarantee and contract

**Future Consideration**:
- Evaluate local AI model options (mistral.ai, aleph-alpha)
- Budget for Phase 2 migration

---

## Timeline and Milestones

### High-Level Milestones

| Milestone | Description | Target Date | Dependencies |
|-----------|-------------|-------------|--------------|
| Requirements Approval | Stakeholder sign-off on requirements | 2026-05-15 | This document |
| Design Complete | HLD and DLD approved | 2026-06-15 | Requirements |
| Core Implementation | GGHH V2 entities, REST API | 2026-08-15 | Design |
| Woo Workflow | Woo publication feature | 2026-09-30 | Core Implementation |
| GitOps Integration | YAML sync service | 2026-10-31 | Core Implementation |
| UAT Complete | User acceptance testing | 2026-11-30 | All features |
| Production Launch | Go-live to first 10 organizations | 2026-12-15 | UAT |

---

## Budget

### Cost Estimate

| Category | Estimated Cost | Notes |
|----------|----------------|-------|
| Development | 400 hours | Rust developers, 6-month timeline |
| Infrastructure | €2,000/year | ArangoDB Cloud or self-hosted |
| AI APIs (Phase 1) | €500/month | PII detection, summarization |
| Testing | €5,000 | Security testing, performance testing |
| Training | €3,000 | Admin UI training for stewards |
| **Total** | **€32,000** | First year |

### Ongoing Operational Costs

| Category | Annual Cost | Notes |
|----------|-------------|-------|
| Infrastructure | €5,000/year | Hosting, backups, monitoring |
| AI APIs (Phase 1) | €6,000/year | PII detection (optional Phase 2) |
| Support | 0.2 FTE | Maintenance, bug fixes |
| **Total** | **€25,000/year** | From Year 2 onwards |

---

## Approval

### Requirements Review

| Reviewer | Role | Status | Date | Comments |
|----------|------|--------|------|----------|
| Product Owner | Business Sponsor | [ ] Approved | [DATE] | |
| Enterprise Architect | Architecture | [ ] Approved | [DATE] | |
| DPO | Compliance | [ ] Approved | [DATE] | |
| DevOps Lead | Operations | [ ] Approved | [DATE] | |

### Sign-Off

By signing below, stakeholders confirm that requirements are complete, understood, and approved to proceed to design phase.

| Stakeholder | Signature | Date |
|-------------|-----------|------|
| [Name, Role] | _________ | [DATE] |
| [Name, Role] | _________ | [DATE] |

---

## Appendices

### Appendix A: Glossary

| Term | Definition |
|------|------------|
| **GGHH** | Gegevenshuishouding - Information Housekeeping |
| **BSW** | Beter Samenwerken - Better Together architecture program |
| **AVG** | Algemene verordening gegevensbescherming (GDPR Netherlands) |
| **Woo** | Wet open overheid (Government Information Act) |
| **Archiefwet** | Dutch Archives Act 2025: transfers to archives after 10 years (was 20), digital accessibility focus. See: [open-overheid.nl](https://www.open-overheid.nl/actueel/nieuws/2025/02/20/nieuwe-archiefwet-aangenomen-door-tweede-kamer) |
| **TOOI** | Tilla Overheids Identificatiecode - standard government codes |
| **MDTO** | Metadata Standaard Dubbele Taak - metadata standard |
| **CDD+** | Content Delivery Disaster recovery - archival system |
| **RLS** | Row-Level Security - database-level access control |
| **SAR** | Subject Access Request - GDPR data access right |
| **Informatieobject** | Dataobject + metadata - core BSW abstraction |
| **Dynamisch** | Mutable information object status (in bewerking) |
| **Gepersistent** | Read-only information object status |
| **Zorgdrager** | Caretaker organization for archived information objects |

### Appendix B: Reference Documents

**Standards & Laws**:
- [Metamodel GGHH Overheid](https://regels.overheid.nl/standaarden/gghh) - Gegevenshuishouding standaard
- [Nieuwe Archiefwet 2025](https://www.open-overheid.nl/actueel/nieuws/2025/02/20/nieuwe-archiefwet-aangenomen-door-tweede-kamer) - 10-jaar overdrachtsregel, digitale toegankelijkheid
- [Woo (Wet open overheid)](https://wetten.overheid.nl/BWBR0036881/)
- [AVG/GDPR Guidelines](https://gdpr-info.eu/)
- [BSW / Zaakgericht Werken](https://vng-realisatie.github.io/gemma-zaken/standaard/) - VNG Realisatie API documentatie

**IOU-Modern Documents**:
- [IOU-Modern Architecture Principles](../001-iou-modern/ARC-000-PRIN-v1.0.md)
- [IOU-Modern Risk Register](../001-iou-modern/ARC-001-RISK-v1.0.md)
- [IOU-Modern Stakeholder Analysis](../001-iou-modern/ARC-001-STKE-v1.0.md)

### Appendix C: Entity Relationship Summary

**Core V2 Entities** (7):
1. Gebeurtenis (Event)
2. Gegevensproduct (Data Product)
3. ElementaireGegevensset (Elementary Data Set)
4. EnkelvoudigGegeven (Simple Data Element)
5. WaardeMetTijd (Value with Time)
6. Context (Context)
7. Grondslag (Legal Basis)

**Phase 1 Entities** (3):
8. Bedrijfsproces (Business Process)
9. Wetsbegrip (Legal Concept)
10. Beleidsbegrip (Policy Concept)

**Phase 2 Entities** (3):
11. PersoonsgebondenTrait (Personal Data Trait)
12. AVGCategory (AVG Category)
13. Bewaartermijn (Retention Period)

**Phase 3 Entities** (1):
14. AuditMutatie (Audit Mutation)

**Phase 4 Entities** (2):
15. TOOIStandard (TOOI Standard)
16. MDTOStandard (MDTO Standard)

**Phase 5 Entities** (1):
17. Informatieobject (Information Object)

**Phase 6 Entities** (3):
18. AIEnrichment (AI Enrichment Result)
19. AISummary (AI Summary)
20. AITranslation (AI Translation)

**Phase 7 Entities** (2):
21. Archiefbestand (Archive File)
22. ArchiefLocation (Archive Location)

**Phase 8 Entities** (1):
23. WooPublicatie (Woo Publication)

**BSW-Specific Entities** (3):
24. Zaak (Case/Dossier) - for grouping and metadata inheritance
25. InformatieobjectCatalogus (Catalog with location references)
26. InformatieobjectRecht (Object-level authorization grants)

**Additional Edge Collections** (3):
30. inherits_from (Zaak to Informatieobject metadata inheritance)
31. catalogus_zaak (Catalogus to Zaak relationship)
32. informatieobject_zorgdrager (Multi-caretaker support)
33. zaak_informatieobject (Zaak to Informatieobject grouping)
34. object_recht_user (InformatieobjectRecht to User)

**Total**: 26 entities, 34 edge collections

---

**END OF REQUIREMENTS**

## Generation Metadata

**Generated by**: ArcKit `/arckit:requirements` command
**Generated on**: 2026-04-19 09:30:00 GMT
**Modified on**: 2026-04-19 10:00:00 GMT
**ArcKit Version**: 4.3.1
**Project**: Metadata Registry Service (Project 002)
**AI Model**: claude-opus-4-7
**Generation Context**: Based on Metamodel GGHH specification, BSW zaak/dossierbeheer requirements, IOU-Modern architecture principles, stakeholder analysis, and existing implementation in metadata-registry/ directory
**BSW Alignment**: Updated per review against "20260324 Zaak dossierbeheer conform BSW.docx"
