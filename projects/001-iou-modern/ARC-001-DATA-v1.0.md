# Data Model: IOU-Modern

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Command**: `/arckit.data-model`

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-001-DATA-v1.0 |
| **Document Type** | Data Model |
| **Project** | IOU-Modern (Project 001) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.0 |
| **Created Date** | 2026-03-20 |
| **Last Modified** | 2026-03-20 |
| **Review Cycle** | Quarterly |
| **Next Review Date** | 2026-06-20 |
| **Owner** | Data Architect |
| **Reviewed By** | PENDING |
| **Approved By** | PENDING |
| **Distribution** | Architecture Team, Development Team, Data Governance Committee |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---------|------|--------|---------|-------------|---------------|
| 1.0 | 2026-03-20 | ArcKit AI | Initial creation from `/arckit:data-model` command | PENDING | PENDING |

---

## Executive Summary

### Overview

IOU-Modern is a context-driven information management platform for Dutch government organizations. This data model defines the core entities required to support information domain management (Zaak, Project, Beleid, Expertise), document processing with AI agents, knowledge graph capabilities (GraphRAG), and compliance tracking for Dutch government regulations (Woo, AVG, Archiefwet).

The system uses a hybrid database architecture: PostgreSQL for transactional data with ACID guarantees and Row-Level Security (RLS), and DuckDB for analytical queries and full-text search. All PII (personally identifiable information) is tracked for GDPR compliance, with automatic classification and retention policies applied to all information objects.

### Model Statistics

- **Total Entities**: 15 entities defined (E-001 through E-015)
- **Total Attributes**: 87 attributes across all entities
- **Total Relationships**: 18 relationships mapped
- **Data Classification**:
  - 🟢 Openbaar: 1 entity (published Woo documents)
  - 🟡 Intern: 8 entities (operational data)
  - 🟠 Vertrouwelijk: 5 entities (users, roles, permissions)
  - 🔴 Geheim: 1 entity (audit trail with PII)

### Compliance Summary

- **GDPR/DPA 2018 Status**: COMPLIANT with DPIA REQUIRED
- **PII Entities**: 5 entities contain personally identifiable information (User, InformationObject for PII-tagged content, Entity with Person type, DomainRelation for stakeholder tracking)
- **Data Protection Impact Assessment (DPIA)**: REQUIRED - System processes citizen data (PII) for government services, uses AI/ML for document processing and entity extraction
- **Data Retention**: 20 years maximum (driven by Archiefwet for formal decisions)
- **Cross-Border Transfers**: NO - All data processed within Netherlands/EU (adequacy decision applies)

### Key Data Governance Stakeholders

- **Data Owner (Business)**: Information Manager (Informatiebeheerder) - Accountable for data quality and usage per Archiefwet
- **Data Steward**: Domain Owners per organization (Zaak-, Project-, Beleids-, Expertise-eigenaren)
- **Data Custodian (Technical)**: Database Team - Manages PostgreSQL/DuckDB infrastructure
- **Data Protection Officer**: Privacy Officer (FG) - Ensures AVG/GDPR compliance

---

## Visual Entity-Relationship Diagram (ERD)

```mermaid
erDiagram
    %% Core Domain Hierarchy
    ORGANIZATION ||--o{ DEPARTMENT : contains
    ORGANIZATION ||--o{ INFORMATION_DOMAIN : owns
    DEPARTMENT ||--o{ USER : employs

    %% User and Role Management (PII)
    USER ||--o{ USER_ROLE : has
    ROLE ||--o{ USER_ROLE : assigned_to

    %% Information Domains
    INFORMATION_DOMAIN ||--o{ INFORMATION_OBJECT : contains
    INFORMATION_DOMAIN ||--o{ INFORMATION_DOMAIN : may_contain
    INFORMATION_DOMAIN ||--o{ DOCUMENT : generates

    %% Document Workflow
    TEMPLATE ||--o{ DOCUMENT : defines
    DOCUMENT ||--o{ AUDIT_TRAIL : logs

    %% GraphRAG Entities
    INFORMATION_OBJECT ||--o{ ENTITY : extracts
    ENTITY ||--o{ RELATIONSHIP : has
    ENTITY ||--o{ COMMUNITY : member_of
    INFORMATION_DOMAIN ||--o{ DOMAIN_RELATION : relates_to
    INFORMATION_DOMAIN ||--o{ CONTEXT_VECTOR : embeds

    ORGANIZATION {
        uuid id PK
        string name
        string short_name
        organization_type type
        uuid parent_organization_id FK
        string website
        timestamp created_at
    }

    DEPARTMENT {
        uuid id PK
        uuid organization_id FK
        string name
        string code
        uuid parent_department_id FK
        uuid manager_user_id FK
        timestamp created_at
    }

    USER {
        uuid id PK
        uuid organization_id FK
        string email UK "PII"
        string display_name "PII"
        string first_name "PII"
        string last_name "PII"
        uuid department_id FK
        string job_title
        string phone "PII"
        boolean is_active
        timestamp last_login
        timestamp created_at
    }

    ROLE {
        uuid id PK
        uuid organization_id FK
        string name
        string description
        jsonb permissions
        boolean is_system_role
        timestamp created_at
    }

    USER_ROLE {
        uuid id PK
        uuid user_id FK
        uuid role_id FK
        uuid scope_domain_id FK
        date valid_from
        date valid_until
        uuid assigned_by FK
        timestamp assigned_at
    }

    INFORMATION_DOMAIN {
        uuid id PK
        domain_type type "Zaak|Project|Beleid|Expertise"
        string name
        text description
        domain_status status
        uuid organization_id FK
        uuid owner_user_id FK
        uuid parent_domain_id FK
        jsonb metadata
        timestamp created_at
        timestamp updated_at
    }

    INFORMATION_OBJECT {
        uuid id PK
        uuid domain_id FK
        object_type type "Document|Email|Chat|Besluit|Data"
        string title
        text description
        string content_location
        text content_text
        string mime_type
        bigint file_size
        classification classification "Openbaar|Intern|Vertrouwelijk|Geheim"
        varchar retention_period
        boolean is_woo_relevant
        timestamp woo_publication_date
        privacy_level privacy_level "Openbaar|Normaal|Bijzonder|Strafrechtelijk"
        text[] tags
        jsonb metadata
        integer version
        uuid previous_version_id FK
        uuid created_by FK
        timestamp created_at
        timestamp updated_at
    }

    DOCUMENT {
        uuid id PK
        varchar domain_id FK
        varchar document_type
        document_state state
        varchar current_version_key
        varchar previous_version_key
        decimal compliance_score
        decimal confidence_score
        timestamp created_at
        timestamp updated_at
    }

    TEMPLATE {
        varchar id PK
        string name
        varchar domain_id FK
        varchar document_type
        text content
        jsonb required_variables
        jsonb optional_sections
        integer version
        timestamp created_at
        timestamp updated_at
        boolean is_active
    }

    AUDIT_TRAIL {
        uuid id PK
        uuid document_id FK
        varchar agent_name
        varchar action
        jsonb details
        timestamp timestamp
        integer execution_time_ms
    }

    ENTITY {
        uuid id PK
        string name
        entity_type type "Person|Organization|Location|Law|Date|Money|Policy|Miscellaneous"
        string canonical_name
        text description
        float confidence
        uuid source_domain_id FK
        jsonb metadata
        timestamp created_at
    }

    RELATIONSHIP {
        uuid id PK
        uuid source_entity_id FK
        uuid target_entity_id FK
        relationship_type type "WorksFor|LocatedIn|SubjectTo|RefersTo|RelatesTo|OwnerOf|ReportsTo|CollaboratesWith|Follows|PartOf"
        float weight
        float confidence
        text context
        uuid source_domain_id FK
        timestamp created_at
    }

    COMMUNITY {
        uuid id PK
        string name
        text description
        integer level
        uuid parent_community_id FK
        uuid[] member_entity_ids
        text summary
        text[] keywords
        timestamp created_at
    }

    DOMAIN_RELATION {
        uuid id PK
        uuid from_domain_id FK
        uuid to_domain_id FK
        domain_relation_type type "SharedEntities|SameCommunity|SemanticSimilarity|TemporalOverlap|SharedStakeholders|ManualLink"
        float strength
        discovery_method method "Automatic|Manual|AiSuggestion|RuleBased"
        uuid[] shared_entities
        text explanation
        timestamp created_at
    }

    CONTEXT_VECTOR {
        uuid id PK
        uuid domain_id FK
        vector embedding
        string model_name
        string model_version
        timestamp created_at
    }
```

**Diagram Notes**:

- **Cardinality**: `||` = exactly one, `o{` = zero or more, `|{` = one or more, `}|..|{` = zero or more (optional many)
- **Primary Keys (PK)**: Uniquely identify each record
- **Foreign Keys (FK)**: Reference other entities
- **Unique Keys (UK)**: Must be unique but not primary identifier
- **PII indicators**: Mark attributes containing personally identifiable information

---

## Entity Catalog

### Entity E-001: Organization

**Description**: Represents a Dutch government organization (Rijk, Provincie, Gemeente, Waterschap, ZBO, or other public body). Organizations are the top-level containers for information domains and user accounts.

**Source Requirements**:

- Derived from domain model `crates/iou-core/src/organization.rs`
- Database schema: `migrations/postgres/001_create_initial_schema.sql` (not yet created in PostgreSQL, implicit in DuckDB)

**Business Context**: Organizations are the legal entities responsible for information management per the Archiefwet. Each organization owns one or more information domains and is responsible for classifying, retaining, and publishing government information.

**Data Ownership**:

- **Business Owner**: Informatiebeheerder (Information Manager) of each organization - Accountable for data accuracy and usage
- **Technical Owner**: Database Team - Maintains database and schema
- **Data Steward**: Domain Administrator per organization

**Data Classification**: INTERNAL

**Volume Estimates**:

- **Initial Volume**: ~500 records (all Dutch government organizations)
- **Growth Rate**: +5 records per year (new organizations, mergers)
- **Peak Volume**: ~1,000 records at Year 10
- **Average Record Size**: 2 KB

**Data Retention**:

- **Active Period**: Indefinite (organizations are legal entities)
- **Archive Period**: Permanent
- **Total Retention**: Permanent (historical record for archival purposes)
- **Deletion Policy**: Never delete (append-only for audit trail)

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| name | VARCHAR(255) | Yes | No | Full organization name | Non-empty, 1-255 chars | None | Derived |
| short_name | VARCHAR(50) | No | No | Short name/abbreviation | 1-50 chars if provided | NULL | Derived |
| organization_type | ENUM | Yes | No | Type of government body | One of: Rijk, Provincie, Gemeente, Waterschap, Gemeenschappelijk, Zbo, Overig | None | Derived |
| parent_organization_id | UUID | No | No | Parent organization (for hierarchy) | Must reference valid Organization.id or NULL | NULL | Derived |
| website | VARCHAR(255) | No | No | Official website URL | Valid URL format if provided | NULL | Derived |
| logo_url | VARCHAR(255) | No | No | Logo image URL | Valid URL format if provided | NULL | Derived |
| primary_color | VARCHAR(7) | No | No | Brand color (hex) | Hex color format #RRGGBB if provided | NULL | Derived |
| secondary_color | VARCHAR(7) | No | No | Secondary brand color | Hex color format #RRGGBB if provided | NULL | Derived |
| created_at | TIMESTAMP | Yes | No | Record creation time | ISO 8601, auto-set | NOW() | Derived |

**Attribute Notes**:

- **PII Attributes**: None
- **Encrypted Attributes**: None
- **Derived Attributes**: None
- **Audit Attributes**: created_at only (organizations don't change frequently)

#### Relationships

**Outgoing Relationships** (this entity references others):

- has_children: E-001 → E-001 (one-to-many, self-reference)
  - Foreign Key: parent_organization_id references E-001.id
  - Description: Hierarchical relationship (e.g., Ministerie van BZK → Rijksoverheid)
  - Cascade Delete: NO - Cannot delete parent if children exist
  - Orphan Check: REQUIRED - child can exist without parent (top-level orgs)

**Incoming Relationships** (other entities reference this):

- owns: E-002 (InformationDomain) → E-001
  - Description: Each information domain belongs to exactly one organization
  - Usage: Data ownership and access control scoping

- employs: E-005 (User) → E-001
  - Description: Each user belongs to exactly one organization
  - Usage: Multi-tenancy and access control

- contains: E-004 (Department) → E-001
  - Description: Departments are sub-units of organizations
  - Usage: Organizational structure and hierarchy

#### Indexes

**Primary Key**:

- `pk_organization` on `id` (clustered index)

**Foreign Keys**:

- `fk_organization_parent` on `parent_organization_id`
  - References: E-001.id
  - On Delete: RESTRICT (cannot delete parent organization with children)
  - On Update: CASCADE

**Unique Constraints**:

- `uk_organization_name` on `name` (organization names must be unique within system)

**Performance Indexes**:

- `idx_organization_type` on `organization_type` (filter by org type)
- `idx_organization_parent` on `parent_organization_id` (hierarchical queries)

#### Privacy & Compliance

**GDPR/DPA 2018 Considerations**:

- **Contains PII**: NO
- **PII Attributes**: None
- **Legal Basis for Processing**: Public task (GDPR Art 6(1)(e)) - Government organizations must maintain registry for transparency and accountability
- **Data Subject Rights**: Not applicable (no personal data)
- **Data Breach Impact**: LOW - Organization data is public information
- **Cross-Border Transfers**: None (Dutch government only)
- **Data Protection Impact Assessment (DPIA)**: NOT_REQUIRED

**Sector-Specific Compliance**:

- **Archiefwet**: Organizations are record creators; must be tracked for provenance
- **Gemeentewet**: Municipalities are legal entities with specific responsibilities
- **Provinciewet**: Provinces are legal entities with specific responsibilities

**Audit Logging**:

- **Access Logging**: Required (log who views organization data)
- **Change Logging**: Required (log all modifications - organization changes are rare but significant)
- **Retention of Logs**: 7 years

---

### Entity E-002: InformationDomain

**Description**: The central organizing unit for government information. Each domain represents a context for organizing information: Zaak (executive work like permits, subsidies), Project (temporary collaborations), Beleid (policy development), or Expertise (knowledge sharing and collaboration).

**Source Requirements**:

- Domain model: `crates/iou-core/src/domain.rs`
- Database schema: `migrations/postgres/001_create_initial_schema.sql`

**Business Context**: Information domains are the core abstraction in IOU-Modern. They group related information objects (documents, emails, decisions, data) and enable context-aware search, knowledge graph discovery, and automated compliance checking.

**Data Ownership**:

- **Business Owner**: Domain Owner (Zaak-, Project-, Beleids-, Expertise-eigenaar) - Accountable for domain data quality
- **Technical Owner**: Database Team
- **Data Steward**: Information Manager per organization

**Data Classification**: VERTRAWELIJK (may contain sensitive government information)

**Volume Estimates**:

- **Initial Volume**: ~100,000 records (active cases, projects, policies)
- **Growth Rate**: +10,000 records per month
- **Peak Volume**: ~500,000 records at Year 5
- **Average Record Size**: 5 KB (including metadata)

**Data Retention**:

- **Active Period**: Until domain completion + 2 years
- **Archive Period**: 18 years (total 20 per Archiefwet for formal decisions)
- **Total Retention**: 20 years (driven by Archiefwet)
- **Deletion Policy**: Soft delete (status = Gearchiveerd), hard delete after 20 years

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| domain_type | ENUM | Yes | No | Type of domain | One of: Zaak, Project, Beleid, Expertise | None | Derived |
| name | VARCHAR(255) | Yes | No | Domain name | Non-empty, 1-255 chars | None | Derived |
| description | TEXT | No | No | Domain description | Free text | NULL | Derived |
| status | ENUM | Yes | No | Domain status | One of: Concept, Actief, Afgerond, Gearchiveerd | Actief | Derived |
| organization_id | UUID | Yes | No | Owner organization | Must reference valid Organization.id | None | Derived |
| owner_user_id | UUID | No | Yes | Domain owner | Must reference valid User.id or NULL | NULL | Derived |
| parent_domain_id | UUID | No | No | Parent domain (for hierarchy) | Must reference valid InformationDomain.id or NULL | NULL | Derived |
| metadata | JSONB | No | No | Flexible metadata | Any valid JSON | {} | Derived |
| created_at | TIMESTAMPTZ | Yes | No | Creation timestamp | ISO 8601, auto-set | NOW() | Derived |
| updated_at | TIMESTAMPTZ | Yes | No | Last update timestamp | ISO 8601, auto-update | NOW() | Derived |

**Attribute Notes**:

- **PII Attributes**: owner_user_id (indirect - references User entity with PII)
- **Encrypted Attributes**: None
- **Derived Attributes**: None
- **Audit Attributes**: created_at, updated_at for change tracking

#### Relationships

**Outgoing Relationships**:

- belongs_to: E-002 → E-001 (many-to-one)
  - Foreign Key: organization_id references E-001.id
  - Description: Each domain belongs to exactly one organization
  - Cascade Delete: YES - if organization deleted, delete all domains (rare)
  - Orphan Check: REQUIRED - domain must have organization

- owned_by: E-002 → E-005 (many-to-one, optional)
  - Foreign Key: owner_user_id references E-005.id
  - Description: Each domain has one owner (user responsible)
  - Cascade Delete: SET NULL - if user deleted, set owner to NULL
  - Orphan Check: OPTIONAL - domain can exist without explicit owner

- parent_of: E-002 → E-002 (one-to-many, self-reference)
  - Foreign Key: parent_domain_id references E-002.id
  - Description: Hierarchical domains (e.g., sub-projects)
  - Cascade Delete: NO - cannot delete parent with children
  - Orphan Check: OPTIONAL - top-level domains have no parent

**Incoming Relationships**:

- contains: E-003 (InformationObject) → E-002
  - Description: Each information object belongs to exactly one domain

- generates: E-008 (Document) → E-002
  - Description: Each generated document is associated with a domain

#### Indexes

**Primary Key**:

- `pk_information_domains` on `id` (clustered index)

**Foreign Keys**:

- `fk_domains_org` on `organization_id` → E-001.id (CASCADE)
- `fk_domains_owner` on `owner_user_id` → E-005.id (SET NULL)
- `fk_domains_parent` on `parent_domain_id` → E-002.id (RESTRICT)

**Unique Constraints**:

- None (domains can have duplicate names within org, but recommend uniqueness)

**Performance Indexes**:

- `idx_domains_type` on `domain_type` (filter by domain type)
- `idx_domains_org` on `organization_id` (multi-tenancy queries)
- `idx_domains_status` on `status` (active domain queries)
- `idx_domains_parent` on `parent_domain_id` (hierarchical queries)

**Full-Text Indexes**:

- `ftx_domains_search` on `name`, `description` (domain search)

#### Privacy & Compliance

**GDPR/DPA 2018 Considerations**:

- **Contains PII**: INDIRECT (via owner_user_id and metadata that may contain stakeholder names)
- **PII Attributes**: owner_user_id (references User with PII)
- **Legal Basis for Processing**: Public task (GDPR Art 6(1)(e)) - Government must organize information for service delivery
- **Data Subject Rights**:
  - **Right to Access**: Users can view domains they own or participate in
  - **Right to Rectification**: Domain owners can update domain metadata
  - **Right to Erasure**: Not applicable (domains are government records, cannot be deleted per AVG right to erasure)
  - **Right to Portability**: Export domain metadata in JSON format
- **Data Breach Impact**: MEDIUM - Domain metadata may contain sensitive government information
- **Cross-Border Transfers**: None
- **Data Protection Impact Assessment (DPIA)**: NOT_REQUIRED for domain entity itself, but required for overall system

**Sector-Specific Compliance**:

- **Archiefwet**: Domains represent record series; must have retention schedules
- **Woo**: Domain status and metadata may need to be published (especially for Besluit domains)

**Audit Logging**:

- **Access Logging**: Required (log domain access for compliance)
- **Change Logging**: Required (domain status changes are significant)
- **Retention of Logs**: 7 years

---

### Entity E-003: InformationObject

**Description**: Represents an individual piece of information within a domain: document, email, chat message, formal decision (besluit), or dataset. Each object has automatic compliance metadata (classification, retention, Woo relevance, privacy level).

**Source Requirements**:

- Domain model: `crates/iou-core/src/objects.rs`
- Database schema: `migrations/postgres/001_create_initial_schema.sql`

**Business Context**: Information objects are the fundamental unit of government information. They are automatically classified for compliance (Woo, AVG, Archiefwet) and can be searched, related via knowledge graph, and exported for public access.

**Data Ownership**:

- **Business Owner**: Domain Owner (via InformationDomain)
- **Technical Owner**: Database Team
- **Data Steward**: Records Management Team

**Data Classification**: VARIES (per object classification: Openbaar, Intern, Vertrouwelijk, Geheim)

**Volume Estimates**:

- **Initial Volume**: ~1,000,000 records
- **Growth Rate**: +50,000 records per month
- **Peak Volume**: ~5,000,000 records at Year 5
- **Average Record Size**: 10 KB (including content_text and metadata)

**Data Retention**:

- **Active Period**: Per retention_period field (default by object type: Besluit=20y, Document=10y, Email=5y, Chat=1y, Data=10y)
- **Archive Period**: Per Archiefwet guidelines
- **Total Retention**: 1-20 years (depends on object type and legal requirements)
- **Deletion Policy**: Hard delete after retention period (for non-permanent records)

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| domain_id | UUID | Yes | No | Parent domain | Must reference valid InformationDomain.id | None | Derived |
| object_type | ENUM | Yes | No | Type of object | One of: Document, Email, Chat, Besluit, Data | None | Derived |
| title | VARCHAR(500) | Yes | No | Object title | Non-empty, 1-500 chars | None | Derived |
| description | TEXT | No | No | Object description | Free text | NULL | Derived |
| content_location | VARCHAR(1000) | Yes | No | Storage location | File path or S3 URI | None | Derived |
| content_text | TEXT | No | No | Extracted text content | Full-text searchable | NULL | Derived |
| mime_type | VARCHAR(100) | No | No | MIME type | Valid MIME type if provided | NULL | Derived |
| file_size | BIGINT | No | No | File size in bytes | Non-negative integer | NULL | Derived |
| classification | ENUM | Yes | No | Security classification | One of: Openbaar, Intern, Vertrouwelijk, Geheim | Intern | Derived |
| retention_period | VARCHAR(50) | No | No | Retention period | "20 jaar", "10 jaar", etc. | NULL | Derived |
| is_woo_relevant | BOOLEAN | Yes | No | Woo publication flag | true/false | false | Derived |
| woo_publication_date | TIMESTAMPTZ | No | No | Woo publication date | ISO 8601 if provided | NULL | Derived |
| privacy_level | ENUM | Yes | No | GDPR privacy level | One of: Openbaar, Normaal, Bijzonder, Strafrechtelijk | Normaal | Derived |
| tags | TEXT[] | No | No | Searchable tags | Array of strings | [] | Derived |
| metadata | JSONB | No | No | Flexible metadata | Any valid JSON (entities, NLP results) | {} | Derived |
| version | INTEGER | Yes | No | Object version | Positive integer | 1 | Derived |
| previous_version_id | UUID | No | No | Previous version | References InformationObject.id or NULL | NULL | Derived |
| created_by | UUID | Yes | Yes | Creator user | Must reference valid User.id | None | Derived |
| created_at | TIMESTAMPTZ | Yes | No | Creation timestamp | ISO 8601, auto-set | NOW() | Derived |
| updated_at | TIMESTAMPTZ | Yes | No | Last update timestamp | ISO 8601, auto-update | NOW() | Derived |

**Attribute Notes**:

- **PII Attributes**: created_by (references User entity), content_text and metadata may contain PII (privacy_level field tracks this)
- **Encrypted Attributes**: None (encryption handled at storage layer via S3/MinIO)
- **Derived Attributes**: None
- **Audit Attributes**: created_at, updated_at, created_by, version

#### Relationships

**Outgoing Relationships**:

- belongs_to: E-003 → E-002 (many-to-one)
  - Foreign Key: domain_id references E-002.id
  - Description: Each object belongs to exactly one domain
  - Cascade Delete: YES - if domain deleted, delete all objects
  - Orphan Check: REQUIRED - object must have domain

- created_by_user: E-003 → E-005 (many-to-one)
  - Foreign Key: created_by references E-005.id
  - Description: Each object was created by a user
  - Cascade Delete: SET NULL - if user deleted, set to NULL (preserve object)
  - Orphan Check: OPTIONAL - system processes can create objects

- version_of: E-003 → E-003 (many-to-one, self-reference)
  - Foreign Key: previous_version_id references E-003.id
  - Description: Version chain for object history
  - Cascade Delete: NO - preserve version history
  - Orphan Check: OPTIONAL - first version has no previous

**Incoming Relationships**:

- extracts: E-011 (Entity) → E-003
  - Description: Entities are extracted from information objects

#### Indexes

**Primary Key**:

- `pk_information_objects` on `id` (clustered index)

**Foreign Keys**:

- `fk_objects_domain` on `domain_id` → E-002.id (CASCADE)
- `fk_objects_creator` on `created_by` → E-005.id (SET NULL)
- `fk_objects_previous` on `previous_version_id` → E-003.id (RESTRICT)

**Unique Constraints**:

- None (objects can have duplicate titles within domain)

**Performance Indexes**:

- `idx_objects_domain` on `domain_id` (domain queries)
- `idx_objects_type` on `object_type` (filter by type)
- `idx_objects_title` on `title` (title search)
- `idx_objects_classification` on `classification` (security filtering)
- `idx_objects_created_at` on `created_at DESC` (recent objects)
- `idx_objects_woo` on `is_woo_relevant` (Woo filtering)

**Full-Text Indexes**:

- `ftx_objects_search` on `title`, `description`, `content_text` (full-text search)
- `idx_objects_tags` on `tags` using gin (tag search)
- `idx_objects_metadata` on `metadata` using gin (metadata queries)

#### Privacy & Compliance

**GDPR/DPA 2018 Considerations**:

- **Contains PII**: YES - content_text and metadata may contain personal data
- **PII Attributes**: content_text, metadata, created_by (indirect), tags (may contain personal names)
- **Legal Basis for Processing**: Public task (GDPR Art 6(1)(e)) - Government must maintain records for service delivery and legal obligations
- **Data Subject Rights**:
  - **Right to Access**: Search and retrieve objects containing personal data via SAR endpoint
  - **Right to Rectification**: Update object metadata if personal data is incorrect
  - **Right to Erasure**: Anonymize or delete after retention period (with legal holds)
  - **Right to Portability**: Export in original format (PDF, etc.)
  - **Right to Object**: Implement opt-out for non-essential data processing
- **Data Breach Impact**: HIGH - Objects may contain sensitive personal data
- **Cross-Border Transfers**: None (data processed in Netherlands/EU)
- **Data Protection Impact Assessment (DPIA)**: REQUIRED - Systematic processing of personal data in government records

**Sector-Specific Compliance**:

- **Woo**: All objects must be assessed for Woo relevance; is_woo_relevant flag required
- **Archiefwet**: Retention periods mandatory per object type; 20 years for Besluit, 10 years for Documents
- **AVG**: privacy_level must be set; special category data (Bijzonder) requires additional protection

**Audit Logging**:

- **Access Logging**: Required (log all access to objects with privacy_level >= Normaal)
- **Change Logging**: Required (log all modifications with before/after values)
- **Retention of Logs**: 7 years

---

### Entity E-004: Department

**Description**: Represents a department or unit within an organization. Departments provide organizational structure and are used for user assignment and data ownership delegation.

**Source Requirements**:

- Domain model: `crates/iou-core/src/organization.rs`
- Database schema: Not yet in PostgreSQL migrations (implicit in code)

**Business Context**: Departments represent the functional units within government organizations (e.g., "Dienst Stedelijke Ontwikkeling", "Sector Ruimte"). They are used for access control and workflow routing.

**Data Ownership**:

- **Business Owner**: Organization Management
- **Technical Owner**: Database Team
- **Data Steward**: HR Department

**Data Classification**: INTERN

**Volume Estimates**:

- **Initial Volume**: ~5,000 records (departments across all organizations)
- **Growth Rate**: +50 records per year (reorganizations)
- **Peak Volume**: ~7,000 records at Year 10
- **Average Record Size**: 1 KB

**Data Retention**:

- **Active Period**: Indefinite
- **Archive Period**: Permanent (historical record)
- **Total Retention**: Permanent
- **Deletion Policy**: Soft delete only (mark as inactive)

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| organization_id | UUID | Yes | No | Parent organization | Must reference valid Organization.id | None | Derived |
| name | VARCHAR(255) | Yes | No | Department name | Non-empty, 1-255 chars | None | Derived |
| code | VARCHAR(50) | No | No | Department code | 1-50 chars if provided | NULL | Derived |
| parent_department_id | UUID | No | No | Parent department | Must reference valid Department.id or NULL | NULL | Derived |
| manager_user_id | UUID | No | Yes | Department manager | Must reference valid User.id or NULL | NULL | Derived |
| created_at | TIMESTAMPTZ | Yes | No | Creation timestamp | ISO 8601, auto-set | NOW() | Derived |

**Attribute Notes**:

- **PII Attributes**: manager_user_id (references User entity)
- **Encrypted Attributes**: None
- **Derived Attributes**: None
- **Audit Attributes**: created_at

#### Relationships

**Outgoing Relationships**:

- belongs_to_org: E-004 → E-001 (many-to-one)
  - Foreign Key: organization_id references E-001.id
  - Cascade Delete: YES
  - Orphan Check: REQUIRED

- parent_of: E-004 → E-004 (one-to-many, self-reference)
  - Foreign Key: parent_department_id references E-004.id
  - Cascade Delete: NO
  - Orphan Check: OPTIONAL

- managed_by: E-004 → E-005 (many-to-one, optional)
  - Foreign Key: manager_user_id references E-005.id
  - Cascade Delete: SET NULL

**Incoming Relationships**:

- employs: E-005 (User) → E-004
  - Description: Users belong to departments

#### Indexes

**Primary Key**: `pk_departments` on `id`

**Foreign Keys**:
- `fk_depts_org` on `organization_id` → E-001.id
- `fk_depts_parent` on `parent_department_id` → E-004.id
- `fk_depts_manager` on `manager_user_id` → E-005.id

**Performance Indexes**:
- `idx_depts_org` on `organization_id`
- `idx_depts_code` on `code` (department code lookup)

#### Privacy & Compliance

- **Contains PII**: INDIRECT (via manager_user_id)
- **DPIA**: NOT_REQUIRED for department entity

---

### Entity E-005: User

**Description**: System user representing a government employee or contractor. Contains PII including name, email, and phone number.

**Source Requirements**:

- Domain model: `crates/iou-core/src/organization.rs`
- Database schema: Not yet in PostgreSQL migrations (implicit in code)

**Business Context**: Users interact with the system to create, manage, and access information objects. Each user belongs to one organization and may have multiple roles with different permissions.

**Data Ownership**:

- **Business Owner**: HR Department
- **Technical Owner**: IAM Team
- **Data Steward**: Data Protection Officer

**Data Classification**: VERTRAWELIJK (contains PII)

**Volume Estimates**:

- **Initial Volume**: ~50,000 records
- **Growth Rate**: +500 records per month
- **Peak Volume**: ~100,000 records at Year 5
- **Average Record Size**: 2 KB

**Data Retention**:

- **Active Period**: While employed + 2 years
- **Archive Period**: 5 years
- **Total Retention**: 7 years (per AVG and employment law)
- **Deletion Policy**: Anonymize PII after retention period (keep user_id for referential integrity)

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| organization_id | UUID | Yes | No | Employer organization | Must reference valid Organization.id | None | Derived |
| email | VARCHAR(255) | Yes | Yes | Email address | RFC 5322 format, unique per org | None | Derived |
| display_name | VARCHAR(255) | Yes | Yes | Display name | Non-empty, 1-255 chars | None | Derived |
| first_name | VARCHAR(100) | No | Yes | First name | 1-100 chars if provided | NULL | Derived |
| last_name | VARCHAR(100) | No | Yes | Last name | 1-100 chars if provided | NULL | Derived |
| department_id | UUID | No | No | Department | Must reference valid Department.id or NULL | NULL | Derived |
| job_title | VARCHAR(255) | No | No | Job title | 1-255 chars if provided | NULL | Derived |
| phone | VARCHAR(20) | No | Yes | Phone number | E.164 format if provided | NULL | Derived |
| avatar_url | VARCHAR(500) | No | No | Avatar image URL | Valid URL if provided | NULL | Derived |
| is_active | BOOLEAN | Yes | No | Active status flag | true/false | true | Derived |
| last_login | TIMESTAMPTZ | No | Yes | Last login timestamp | ISO 8601 | NULL | Derived |
| created_at | TIMESTAMPTZ | Yes | No | Creation timestamp | ISO 8601, auto-set | NOW() | Derived |

**Attribute Notes**:

- **PII Attributes**: email, display_name, first_name, last_name, phone, last_login (behavioral data)
- **Encrypted Attributes**: RECOMMENDED for email, phone (encrypt at rest)
- **Derived Attributes**: None
- **Audit Attributes**: created_at, last_login

#### Relationships

**Outgoing Relationships**:

- employed_by: E-005 → E-001 (many-to-one)
  - Foreign Key: organization_id references E-001.id

- belongs_to: E-005 → E-004 (many-to-one, optional)
  - Foreign Key: department_id references E-004.id

- has_roles: E-005 → E-007 (one-to-many)
  - Description: User has multiple role assignments

**Incoming Relationships**:

- owns: E-002 (InformationDomain) → E-005
- created_objects: E-003 (InformationObject) → E-005
- manages: E-004 (Department) → E-005

#### Indexes

**Primary Key**: `pk_users` on `id`

**Foreign Keys**:
- `fk_users_org` on `organization_id` → E-001.id
- `fk_users_dept` on `department_id` → E-004.id

**Unique Constraints**:
- `uk_users_email` on `email`, `organization_id` (unique email per org)

**Performance Indexes**:
- `idx_users_active` on `is_active` (active user queries)
- `idx_users_name` on `display_name` (name search)

#### Privacy & Compliance

**GDPR/DPA 2018 Considerations**:

- **Contains PII**: YES - email, display_name, first_name, last_name, phone
- **Legal Basis for Processing**: Contract (GDPR Art 6(1)(b)) - Employment contract
- **Data Subject Rights**:
  - **Right to Access**: Users can view their own profile
  - **Right to Rectification**: Users can update their own data
  - **Right to Erasure**: Anonymize after retention period (with legal holds for audit)
  - **Right to Portability**: Export user data in JSON format
  - **Right to Object**: Opt-out of non-essential processing
- **Data Breach Impact**: HIGH - Employee PII
- **DPIA**: NOT_REQUIRED for employee data processing (standard HR processing)

**Audit Logging**:
- **Access Logging**: Required (log who views user profiles)
- **Change Logging**: Required (log all PII modifications)

---

### Entity E-006: Role

**Description**: Defines a role with associated permissions. Roles are assigned to users via UserRole entity and support scoped permissions (domain-specific or global).

**Source Requirements**:

- Domain model: `crates/iou-core/src/organization.rs`
- Permissions enum in code: `Permission` enum

**Business Context**: Roles implement RBAC (Role-Based Access Control). System roles include DomainRead, ObjectCreate, ComplianceApprove, WooPublish, etc. Roles can be scoped to specific domains for fine-grained access control.

**Data Ownership**:

- **Business Owner**: Security Officer
- **Technical Owner**: IAM Team

**Data Classification**: VERTRAWELIJK

**Volume Estimates**:

- **Initial Volume**: ~50 roles
- **Growth Rate**: +5 roles per year
- **Peak Volume**: ~100 roles at Year 10
- **Average Record Size**: 3 KB

**Data Retention**:

- **Active Period**: Indefite
- **Archive Period**: Permanent (audit trail)
- **Total Retention**: Permanent
- **Deletion Policy**: Soft delete only

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| organization_id | UUID | Yes | No | Owner organization | Must reference valid Organization.id | None | Derived |
| name | VARCHAR(100) | Yes | No | Role name | Non-empty, unique per org | None | Derived |
| description | TEXT | No | No | Role description | Free text | NULL | Derived |
| permissions | JSONB | Yes | No | Permission list | Array of permission strings | [] | Derived |
| is_system_role | BOOLEAN | Yes | No | System role flag | true/false | false | Derived |
| created_at | TIMESTAMPTZ | Yes | No | Creation timestamp | ISO 8601, auto-set | NOW() | Derived |

**Attribute Notes**:

- **PII Attributes**: None
- **Permissions format**: `["DomainCreate", "DomainRead", "ObjectCreate", ...]`

#### Relationships

**Outgoing Relationships**:

- belongs_to: E-006 → E-001 (many-to-one)

**Incoming Relationships**:

- assigned_to_users: E-007 (UserRole) → E-006

#### Indexes

**Primary Key**: `pk_roles` on `id`

**Unique Constraints**:
- `uk_roles_name` on `name`, `organization_id`

**Performance Indexes**:
- `idx_roles_system` on `is_system_role` (system role queries)

#### Privacy & Compliance

- **Contains PII**: NO
- **DPIA**: NOT_REQUIRED

---

### Entity E-007: UserRole

**Description**: Junction entity assigning a role to a user with optional domain scope and validity period. Supports time-bound role assignments (e.g., project-specific access).

**Source Requirements**:

- Domain model: `crates/iou-core/src/organization.rs`

**Business Context**: User roles can be scoped to specific domains (e.g., "Reviewer for domain X") and time-bound (valid_from, valid_until). This enables temporary access grants and project-specific permissions.

**Data Ownership**:

- **Business Owner**: Data Owners (who assign roles)
- **Technical Owner**: IAM Team

**Data Classification**: VERTRAWELIJK

**Volume Estimates**:

- **Initial Volume**: ~100,000 records (users have multiple roles)
- **Growth Rate**: +1,000 records per month
- **Peak Volume**: ~200,000 records at Year 5
- **Average Record Size**: 1 KB

**Data Retention**:

- **Active Period**: While role assignment is active + 7 years
- **Archive Period**: Permanent
- **Total Retention**: Permanent (access audit trail)
- **Deletion Policy**: Soft delete only

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| user_id | UUID | Yes | Yes | User | Must reference valid User.id | None | Derived |
| role_id | UUID | Yes | No | Role | Must reference valid Role.id | None | Derived |
| scope_domain_id | UUID | No | No | Domain scope (optional) | Must reference valid InformationDomain.id or NULL | NULL | Derived |
| valid_from | DATE | Yes | No | Valid from date | ISO 8601 date | Current date | Derived |
| valid_until | DATE | No | No | Valid until date | ISO 8601 date, >= valid_from | NULL | Derived |
| assigned_by | UUID | Yes | Yes | Assigning user | Must reference valid User.id | None | Derived |
| assigned_at | TIMESTAMPTZ | Yes | No | Assignment timestamp | ISO 8601, auto-set | NOW() | Derived |

**Attribute Notes**:

- **PII Attributes**: user_id (references User), assigned_by (references User)
- **Validation**: `is_valid()` method checks current date against valid_from/valid_until

#### Relationships

**Outgoing Relationships**:

- for_user: E-007 → E-005 (many-to-one)
- for_role: E-007 → E-006 (many-to-one)
- scoped_to: E-007 → E-002 (many-to-one, optional)

#### Indexes

**Primary Key**: `pk_user_roles` on `id`

**Foreign Keys**:
- `fk_ur_user` on `user_id` → E-005.id (CASCADE)
- `fk_ur_role` on `role_id` → E-006.id (CASCADE)
- `fk_ur_scope` on `scope_domain_id` → E-002.id (CASCADE)
- `fk_ur_assigned_by` on `assigned_by` → E-005.id (SET NULL)

**Performance Indexes**:
- `idx_ur_user` on `user_id` (user's roles lookup)
- `idx_ur_role` on `role_id` (role's users lookup)
- `idx_ur_validity` on `valid_from`, `valid_until` (validity checks)

#### Privacy & Compliance

- **Contains PII**: INDIRECT (references User)
- **DPIA**: NOT_REQUIRED

---

### Entity E-008: Document

**Description**: Represents a generated document in the multi-agent document creation pipeline. Tracks document state (draft → review → approved → published), compliance scores, and version references.

**Source Requirements**:

- Domain model: `crates/iou-core/src/document.rs`
- Database schema: `migrations/postgres/001_create_initial_schema.sql`

**Business Context**: Documents are generated by AI agents (Research, Content, Compliance, Review) and go through a workflow with states including draft, submitted, in_review, changes_requested, approved, published, rejected, archived. Compliance and confidence scores determine if human approval is required.

**Data Ownership**:

- **Business Owner**: Domain Owner
- **Technical Owner**: Document Pipeline Team

**Data Classification**: VARIES (based on domain classification)

**Volume Estimates**:

- **Initial Volume**: ~10,000 records
- **Growth Rate**: +1,000 records per month
- **Peak Volume**: ~100,000 records at Year 5
- **Average Record Size**: 2 KB (metadata only, content in S3)

**Data Retention**:

- **Active Period**: Same as parent InformationDomain
- **Archive Period**: 20 years (per Archiefwet)
- **Total Retention**: 20 years
- **Deletion Policy**: Hard delete after retention period

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| domain_id | VARCHAR | Yes | No | Parent domain | Domain identifier | None | Derived |
| document_type | VARCHAR | Yes | No | Document type | Template identifier | None | Derived |
| state | ENUM | Yes | No | Document state | draft, submitted, in_review, changes_requested, approved, published, rejected, archived | draft | Derived |
| current_version_key | VARCHAR | Yes | No | Current version S3 key | S3 object key | None | Derived |
| previous_version_key | VARCHAR | No | No | Previous version S3 key | S3 object key or NULL | NULL | Derived |
| compliance_score | DECIMAL(3,2) | No | No | Compliance score | 0.00-1.00 range | NULL | Derived |
| confidence_score | DECIMAL(3,2) | No | No | Confidence score | 0.00-1.00 range | NULL | Derived |
| created_at | TIMESTAMPTZ | Yes | No | Creation timestamp | ISO 8601, auto-set | NOW() | Derived |
| updated_at | TIMESTAMPTZ | Yes | No | Last update timestamp | ISO 8601, auto-update | NOW() | Derived |

**Attribute Notes**:

- **PII Attributes**: None (PII in document content stored in S3)
- **State Machine**: Document flows through states per workflow

#### Relationships

**Outgoing Relationships**:

- belongs_to_domain: E-008 → E-002 (via domain_id string reference)

**Incoming Relationships**:

- logged_in: E-010 (AuditTrail) → E-008

#### Indexes

**Primary Key**: `pk_documents` on `id`

**Performance Indexes**:
- `idx_documents_domain` on `domain_id`
- `idx_documents_state` on `state`
- `idx_documents_type` on `document_type`

#### Privacy & Compliance

- **Contains PII**: INDIRECT (document content may contain PII)
- **DPIA**: NOT_REQUIRED for document metadata entity

---

### Entity E-009: Template

**Description**: Document template used for generating documents. Templates define required variables, optional sections, and content structure. Active templates are used by the Content agent.

**Source Requirements**:

- Domain model: `crates/iou-core/src/document.rs`
- Database schema: `migrations/postgres/001_create_initial_schema.sql`

**Business Context**: Templates are Markdown documents with variable placeholders (Tera syntax). Each template is specific to a domain and document type (e.g., "woo_besluit" for Zaak domains).

**Data Ownership**:

- **Business Owner**: Content Manager
- **Technical Owner**: Document Pipeline Team

**Data Classification**: INTERN

**Volume Estimates**:

- **Initial Volume**: ~50 templates
- **Growth Rate**: +5 templates per year
- **Peak Volume**: ~100 templates at Year 10
- **Average Record Size**: 50 KB (template content)

**Data Retention**:

- **Active Period**: Indefite
- **Archive Period**: Permanent
- **Total Retention**: Permanent
- **Deletion Policy**: Soft delete only

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | VARCHAR | Yes | No | Template identifier | Unique template ID | None | Derived |
| name | VARCHAR | Yes | No | Template name | Human-readable name | None | Derived |
| domain_id | VARCHAR | Yes | No | Domain scope | Domain identifier | None | Derived |
| document_type | VARCHAR | Yes | No | Document type | Type identifier | None | Derived |
| content | TEXT | Yes | No | Template content | Markdown with Tera syntax | None | Derived |
| required_variables | JSONB | No | No | Required variables | Array of variable names | {} | Derived |
| optional_sections | JSONB | No | No | Optional sections | Array of section names | {} | Derived |
| version | INTEGER | Yes | No | Template version | Positive integer | 1 | Derived |
| created_at | TIMESTAMPTZ | Yes | No | Creation timestamp | ISO 8601, auto-set | NOW() | Derived |
| updated_at | TIMESTAMPTZ | Yes | No | Last update timestamp | ISO 8601, auto-update | NOW() | Derived |
| is_active | BOOLEAN | Yes | No | Active flag | true/false | true | Derived |

**Attribute Notes**:

- **PII Attributes**: None

#### Relationships

**Outgoing Relationships**:

- defines: E-009 → E-002 (via domain_id string reference)

**Incoming Relationships**:

- used_by: E-008 (Document) → E-009

#### Indexes

**Primary Key**: `pk_templates` on `id`

**Performance Indexes**:
- `idx_templates_domain` on `domain_id`
- `idx_templates_type` on `document_type`
- `idx_templates_active` on `is_active` WHERE is_active = true

#### Privacy & Compliance

- **Contains PII**: NO
- **DPIA**: NOT_REQUIRED

---

### Entity E-010: AuditTrail

**Description**: Audit trail entry logging all agent actions in the document creation pipeline. Provides observability and compliance traceability.

**Source Requirements**:

- Domain model: `crates/iou-core/src/document.rs`
- Database schema: `migrations/postgres/001_create_initial_schema.sql`

**Business Context**: Every agent action (Research, Content, Compliance, Review) is logged with timestamp, execution time, and details. This enables debugging, compliance auditing, and performance analysis.

**Data Ownership**:

- **Business Owner**: Compliance Officer
- **Technical Owner**: DevOps Team

**Data Classification**: VERTRAWELIJK (contains operational data)

**Volume Estimates**:

- **Initial Volume**: ~100,000 records
- **Growth Rate**: +10,000 records per month
- **Peak Volume**: ~1,000,000 records at Year 5
- **Average Record Size**: 2 KB

**Data Retention**:

- **Active Period**: 1 year
- **Archive Period**: 6 years
- **Total Retention**: 7 years (compliance standard)
- **Deletion Policy**: Hard delete after 7 years

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| document_id | UUID | Yes | No | Document reference | Must reference valid Document.id | None | Derived |
| agent_name | VARCHAR | Yes | No | Agent name | Research, Content, Compliance, Review | None | Derived |
| action | VARCHAR | Yes | No | Action performed | Action description | None | Derived |
| details | JSONB | No | No | Action details | Any valid JSON | {} | Derived |
| timestamp | TIMESTAMPTZ | Yes | No | Action timestamp | ISO 8601, auto-set | NOW() | Derived |
| execution_time_ms | INTEGER | No | No | Execution time | Milliseconds, non-negative | NULL | Derived |

**Attribute Notes**:

- **PII Attributes**: None (details should not contain PII)
- **Purpose**: Compliance, debugging, performance analysis

#### Relationships

**Outgoing Relationships**:

- for_document: E-010 → E-008 (many-to-one)

#### Indexes

**Primary Key**: `pk_audit_trail` on `id`

**Foreign Keys**:
- `fk_audit_document` on `document_id` → E-008.id (CASCADE)

**Performance Indexes**:
- `idx_audit_document` on `document_id`
- `idx_audit_timestamp` on `timestamp DESC`
- `idx_audit_agent` on `agent_name`

#### Privacy & Compliance

- **Contains PII**: NO
- **DPIA**: NOT_REQUIRED
- **Purpose**: Compliance audit trail

---

### Entity E-011: Entity

**Description**: Named entity extracted from information objects via NER (Named Entity Recognition). Entities include Person, Organization, Location, Law, Date, Money, Policy, and Miscellaneous.

**Source Requirements**:

- Domain model: `crates/iou-core/src/graphrag.rs`
- NER implementation: `crates/iou-ai/src/ner.rs`

**Business Context**: Entities are extracted from documents by the AI pipeline and used to build knowledge graphs, discover relationships, and enable semantic search. Person-type entities contain PII.

**Data Ownership**:

- **Business Owner**: Domain Owner (via source domain)
- **Technical Owner**: Knowledge Graph Team

**Data Classification**: VARIES (Person entities contain PII)

**Volume Estimates**:

- **Initial Volume**: ~1,000,000 records
- **Growth Rate**: +50,000 entities per month
- **Peak Volume**: ~5,000,000 entities at Year 5
- **Average Record Size**: 1 KB

**Data Retention**:

- **Active Period**: Same as source InformationObject
- **Archive Period**: 20 years
- **Total Retention**: 20 years
- **Deletion Policy**: Hard delete after retention period

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| name | VARCHAR | Yes | Yes | Entity name | Entity text | None | Derived |
| entity_type | ENUM | Yes | No | Entity type | Person, Organization, Location, Law, Date, Money, Policy, Miscellaneous | None | Derived |
| canonical_name | VARCHAR | No | Yes | Canonical name | Standardized name if available | NULL | Derived |
| description | TEXT | No | No | Entity description | Free text | NULL | Derived |
| confidence | FLOAT | No | No | Extraction confidence | 0.0-1.00 range | NULL | Derived |
| source_domain_id | UUID | No | No | Source domain | Domain where entity was found | NULL | Derived |
| metadata | JSONB | No | No | Additional metadata | Any valid JSON | {} | Derived |
| created_at | TIMESTAMPTZ | Yes | No | Extraction timestamp | ISO 8601, auto-set | NOW() | Derived |

**Attribute Notes**:

- **PII Attributes**: name, canonical_name (for Person-type entities)
- **Special Handling**: Person entities require additional protection

#### Relationships

**Outgoing Relationships**:

- from_domain: E-011 → E-002 (many-to-one, optional)

**Incoming Relationships**:

- has_relationships: E-012 (Relationship) → E-011 (both source and target)
- member_of: E-011 → E-013 (many-to-many)

#### Indexes

**Primary Key**: `pk_entities` on `id`

**Performance Indexes**:
- `idx_entities_type` on `entity_type` (type filtering)
- `idx_entities_name` on `name` (name search)
- `idx_entities_domain` on `source_domain_id` (domain queries)

#### Privacy & Compliance

**GDPR/DPA 2018 Considerations**:

- **Contains PII**: YES - Person entities contain names of individuals
- **PII Attributes**: name, canonical_name (for entity_type = Person)
- **Legal Basis for Processing**: Public task (GDPR Art 6(1)(e)) - Knowledge extraction for government information management
- **Data Subject Rights**:
  - **Right to Access**: Search and retrieve Person entities containing personal data
  - **Right to Rectification**: Update entity metadata if incorrect
  - **Right to Erasure**: Delete Person entities after retention period
  - **Right to Object**: Opt-out of entity extraction for non-essential processing
- **Data Breach Impact**: MEDIUM - Person entities contain individual names
- **DPIA**: REQUIRED - Systematic processing of personal data via NER

**Sector-Specific Compliance**:

- **Woo**: Entity extraction may reveal stakeholder relationships that need to be published

**Audit Logging**:

- **Access Logging**: Required for Person entities
- **Change Logging**: Required (entity corrections)

---

### Entity E-012: Relationship

**Description**: Relationship between two entities, discovered by GraphRAG or manually entered. Relationships include WorksFor, LocatedIn, SubjectTo, RefersTo, etc.

**Source Requirements**:

- Domain model: `crates/iou-core/src/graphrag.rs`

**Business Context**: Relationships define how entities are connected. They are used for knowledge graph visualization, semantic search, and automatic domain relationship discovery.

**Data Ownership**:

- **Business Owner**: Knowledge Graph Team
- **Technical Owner**: Knowledge Graph Team

**Data Classification**: INTERN

**Volume Estimates**:

- **Initial Volume**: ~5,000,000 records
- **Growth Rate**: +250,000 relationships per month
- **Peak Volume**: ~25,000,000 relationships at Year 5
- **Average Record Size**: 1 KB

**Data Retention**:

- **Active Period**: Same as source entities
- **Archive Period**: 20 years
- **Total Retention**: 20 years
- **Deletion Policy**: Hard delete after retention period

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| source_entity_id | UUID | Yes | No | Source entity | Must reference valid Entity.id | None | Derived |
| target_entity_id | UUID | Yes | No | Target entity | Must reference valid Entity.id | None | Derived |
| relationship_type | ENUM | Yes | No | Relationship type | WorksFor, LocatedIn, SubjectTo, RefersTo, RelatesTo, OwnerOf, ReportsTo, CollaboratesWith, Follows, PartOf, Unknown | None | Derived |
| weight | FLOAT | No | No | Relationship weight | 0.0-1.00 range (strength) | NULL | Derived |
| confidence | FLOAT | No | No | Confidence score | 0.0-1.00 range | NULL | Derived |
| context | TEXT | No | No | Relationship context | Free text describing context | NULL | Derived |
| source_domain_id | UUID | No | No | Source domain | Domain where relationship was found | NULL | Derived |
| created_at | TIMESTAMPTZ | Yes | No | Creation timestamp | ISO 8601, auto-set | NOW() | Derived |

**Attribute Notes**:

- **PII Attributes**: None (indirect via entity references)

#### Relationships

**Outgoing Relationships**:

- from_entity: E-012 → E-011 (many-to-one)
- to_entity: E-012 → E-011 (many-to-one)
- from_domain: E-012 → E-002 (many-to-one, optional)

#### Indexes

**Primary Key**: `pk_relationships` on `id`

**Foreign Keys**:
- `fk_rel_source` on `source_entity_id` → E-011.id (CASCADE)
- `fk_rel_target` on `target_entity_id` → E-011.id (CASCADE)
- `fk_rel_domain` on `source_domain_id` → E-002.id (CASCADE)

**Performance Indexes**:
- `idx_rel_source` on `source_entity_id` (entity's outgoing relationships)
- `idx_rel_target` on `target_entity_id` (entity's incoming relationships)
- `idx_rel_type` on `relationship_type` (type filtering)

#### Privacy & Compliance

- **Contains PII**: INDIRECT (via entity references)
- **DPIA**: NOT_REQUIRED

---

### Entity E-013: Community

**Description**: Cluster of related entities discovered by GraphRAG community detection. Communities represent groups of entities that are densely connected.

**Source Requirements**:

- Domain model: `crates/iou-core/src/graphrag.rs`

**Business Context**: Communities are used to organize knowledge graphs and discover thematic clusters. Each community has a summary and keywords for human interpretation.

**Data Ownership**:

- **Business Owner**: Knowledge Graph Team

**Data Classification**: INTERN

**Volume Estimates**:

- **Initial Volume**: ~10,000 communities
- **Growth Rate**: +500 communities per month
- **Peak Volume**: ~50,000 communities at Year 5
- **Average Record Size**: 3 KB

**Data Retention**:

- **Active Period**: Same as source entities
- **Archive Period**: 20 years
- **Total Retention**: 20 years
- **Deletion Policy**: Hard delete after retention period

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| name | VARCHAR | Yes | No | Community name | Human-readable name | None | Derived |
| description | TEXT | No | No | Community description | Free text | NULL | Derived |
| level | INTEGER | Yes | No | Community level | Hierarchy level (0 = top) | 0 | Derived |
| parent_community_id | UUID | No | No | Parent community | Must reference valid Community.id or NULL | NULL | Derived |
| member_entity_ids | UUID[] | Yes | No | Member entities | Array of entity IDs | [] | Derived |
| summary | TEXT | No | No | Community summary | AI-generated summary | NULL | Derived |
| keywords | TEXT[] | No | No | Community keywords | Array of keywords | [] | Derived |
| created_at | TIMESTAMPTZ | Yes | No | Creation timestamp | ISO 8601, auto-set | NOW() | Derived |

**Attribute Notes**:

- **PII Attributes**: None

#### Relationships

**Outgoing Relationships**:

- parent_of: E-013 → E-013 (one-to-many, self-reference)
- has_members: E-013 → E-011 (many-to-many via member_entity_ids)

**Incoming Relationships**:

- member_of: E-011 → E-013

#### Indexes

**Primary Key**: `pk_communities` on `id`

**Performance Indexes**:
- `idx_communities_level` on `level` (hierarchical queries)
- `idx_communities_parent` on `parent_community_id` (hierarchical queries)

#### Privacy & Compliance

- **Contains PII**: NO
- **DPIA**: NOT_REQUIRED

---

### Entity E-014: DomainRelation

**Description**: Relationship between two information domains, discovered automatically via GraphRAG or manually linked. Used for cross-domain discovery and knowledge graph navigation.

**Source Requirements**:

- Domain model: `crates/iou-core/src/graphrag.rs`

**Business Context**: Domain relations enable users to discover related domains (e.g., a Project domain related to a Beleid domain). They are discovered via shared entities, semantic similarity, or manual linking.

**Data Ownership**:

- **Business Owner**: Knowledge Graph Team

**Data Classification**: INTERN

**Volume Estimates**:

- **Initial Volume**: ~50,000 relations
- **Growth Rate**: +2,000 relations per month
- **Peak Volume**: ~200,000 relations at Year 5
- **Average Record Size**: 2 KB

**Data Retention**:

- **Active Period**: Same as source domains
- **Archive Period**: 20 years
- **Total Retention**: 20 years
- **Deletion Policy**: Hard delete after retention period

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| from_domain_id | UUID | Yes | No | Source domain | Must reference valid InformationDomain.id | None | Derived |
| to_domain_id | UUID | Yes | No | Target domain | Must reference valid InformationDomain.id | None | Derived |
| relation_type | ENUM | Yes | No | Relation type | SharedEntities, SameCommunity, SemanticSimilarity, TemporalOverlap, SharedStakeholders, ManualLink | None | Derived |
| strength | FLOAT | No | No | Relation strength | 0.0-1.00 range | NULL | Derived |
| discovery_method | ENUM | Yes | No | Discovery method | Automatic, Manual, AiSuggestion, RuleBased | None | Derived |
| shared_entities | UUID[] | No | No | Shared entity IDs | Array of entity IDs | [] | Derived |
| explanation | TEXT | No | No | Relation explanation | Human-readable explanation | NULL | Derived |
| created_at | TIMESTAMPTZ | Yes | No | Creation timestamp | ISO 8601, auto-set | NOW() | Derived |

**Attribute Notes**:

- **PII Attributes**: None

#### Relationships

**Outgoing Relationships**:

- from_domain: E-014 → E-002 (many-to-one)
- to_domain: E-014 → E-002 (many-to-one)

#### Indexes

**Primary Key**: `pk_domain_relations` on `id`

**Foreign Keys**:
- `fk_dr_from` on `from_domain_id` → E-002.id (CASCADE)
- `fk_dr_to` on `to_domain_id` → E-002.id (CASCADE)

**Performance Indexes**:
- `idx_dr_from` on `from_domain_id` (domain's outgoing relations)
- `idx_dr_to` on `to_domain_id` (domain's incoming relations)
- `idx_dr_type` on `relation_type` (type filtering)

#### Privacy & Compliance

- **Contains PII**: NO
- **DPIA**: NOT_REQUIRED

---

### Entity E-015: ContextVector

**Description**: Embedding vector for semantic search. Each domain can have a context vector generated by an embedding model, enabling semantic similarity search.

**Source Requirements**:

- Domain model: `crates/iou-core/src/graphrag.rs`

**Business Context**: Context vectors enable semantic search (find similar domains by meaning, not just keywords). Generated by embedding models (e.g., OpenAI, sentence-transformers).

**Data Ownership**:

- **Business Owner**: Knowledge Graph Team

**Data Classification**: INTERN

**Volume Estimates**:

- **Initial Volume**: ~100,000 vectors
- **Growth Rate**: +5,000 vectors per month
- **Peak Volume**: ~500,000 vectors at Year 5
- **Average Record Size**: 5 KB (including vector)

**Data Retention**:

- **Active Period**: Same as source domain
- **Archive Period**: 20 years
- **Total Retention**: 20 years
- **Deletion Policy**: Hard delete after retention period

#### Attributes

| Attribute | Type | Required | PII | Description | Validation Rules | Default | Source Req |
|-----------|------|----------|-----|-------------|------------------|---------|------------|
| id | UUID | Yes | No | Unique identifier | UUID v4 format | Auto-generated | Derived |
| domain_id | UUID | Yes | No | Domain reference | Must reference valid InformationDomain.id | None | Derived |
| embedding | VECTOR | Yes | No | Embedding vector | Float array (model-dependent size) | None | Derived |
| model_name | VARCHAR | Yes | No | Model name | e.g., "text-embedding-3-small" | None | Derived |
| model_version | VARCHAR | Yes | No | Model version | e.g., "1.0.0" | None | Derived |
| created_at | TIMESTAMPTZ | Yes | No | Creation timestamp | ISO 8601, auto-set | NOW() | Derived |

**Attribute Notes**:

- **PII Attributes**: None
- **Vector Storage**: Requires pgvector extension for PostgreSQL

#### Relationships

**Outgoing Relationships**:

- for_domain: E-015 → E-002 (many-to-one)

#### Indexes

**Primary Key**: `pk_context_vectors` on `id`

**Foreign Keys**:
- `fk_cv_domain` on `domain_id` → E-002.id (CASCADE)

**Performance Indexes**:
- `idx_cv_embedding` on `embedding` using ivfflat (vector similarity search)

#### Privacy & Compliance

- **Contains PII**: NO (vectors are derived from domain text, not directly identifying)
- **DPIA**: NOT_REQUIRED

---

## Data Governance Matrix

| Entity | Business Owner | Data Steward | Technical Custodian | Sensitivity | Compliance | Quality SLA | Access Control |
|--------|----------------|--------------|---------------------|-------------|------------|-------------|----------------|
| E-001: Organization | Informatiebeheerder | Domain Administrator | Database Team | INTERNAL | None | 100% accuracy | Role: All authenticated (read-only) |
| E-002: InformationDomain | Domain Owner | Information Manager | Database Team | VERTRAWELIJK | Archiefwet, Woo | 99% completeness | Role: Domain members, Org admin |
| E-003: InformationObject | Domain Owner | Records Management Team | Database Team | VARIES | Woo, AVG, Archiefwet | 95% metadata completeness | Role: Based on classification |
| E-004: Department | HR Manager | Org Administrator | Database Team | INTERNAL | None | 100% accuracy | Role: Org members (read-only) |
| E-005: User | HR Manager | Data Protection Officer | IAM Team | VERTRAWELIJK | AVG | 99% accuracy, 95% completeness | Role: User (self), HR (manage) |
| E-006: Role | Security Officer | IAM Team | IAM Team | VERTRAWELIJK | None | 100% accuracy | Role: Security admin, Org admin |
| E-007: UserRole | Data Owner | IAM Team | IAM Team | VERTRAWELIJK | AVG (access logging) | 99% validity accuracy | Role: Security admin, Data Owner |
| E-008: Document | Domain Owner | Document Manager | Document Pipeline Team | VARIES | Woo, Archiefwet | 95% compliance score | Role: Domain members (based on state) |
| E-009: Template | Content Manager | Template Manager | Document Pipeline Team | INTERNAL | None | 100% syntax validity | Role: Template admin, Content manager |
| E-010: AuditTrail | Compliance Officer | DevOps Team | DevOps Team | VERTRAWELIJK | Compliance standards | 100% completeness | Role: Compliance officer, DevOps |
| E-011: Entity | Domain Owner | Knowledge Graph Team | Knowledge Graph Team | VARIES | AVG (Person entities) | 90% extraction accuracy | Role: Based on entity type |
| E-012: Relationship | Knowledge Graph Team | Knowledge Graph Team | Knowledge Graph Team | INTERNAL | None | 85% confidence average | Role: All authenticated (read-only) |
| E-013: Community | Knowledge Graph Team | Knowledge Graph Team | Knowledge Graph Team | INTERNAL | None | N/A | Role: All authenticated (read-only) |
| E-014: DomainRelation | Knowledge Graph Team | Knowledge Graph Team | Knowledge Graph Team | INTERNAL | None | N/A | Role: All authenticated (read-only) |
| E-015: ContextVector | Knowledge Graph Team | Knowledge Graph Team | Knowledge Graph Team | INTERNAL | None | N/A | Role: System access only |

**Governance Notes**:

- **Business Owner**: Accountable for data quality, accuracy, and appropriate usage
- **Data Steward**: Responsible for enforcing governance policies and resolving data quality issues
- **Technical Custodian**: Manages database infrastructure, backups, security controls
- **Sensitivity**: Classification drives access controls and encryption requirements
- **Compliance**: Regulatory frameworks that apply to this entity
- **Quality SLA**: Measurable quality targets (accuracy, completeness, timeliness)
- **Access Control**: Roles/groups permitted to view or modify data

---

## CRUD Matrix

**Purpose**: Shows which components/systems can Create, Read, Update, Delete each entity

| Entity | Web API | Admin Portal | AI Agents | Analytics | Background Jobs | Public API |
|--------|----------|--------------|-----------|-----------|------------------|------------|
| E-001: Organization | CR-- | CRUD | ---- | -R-- | ---- | -R-- |
| E-002: InformationDomain | CRUD | CRUD | CR-- | -R-- | --U- | ---- |
| E-003: InformationObject | CRUD | CRUD | CR-- | -R-- | --U- | -R-- (Woo filter) |
| E-004: Department | CR-- | CRUD | ---- | -R-- | ---- | ---- |
| E-005: User | CR-- | CRUD | ---- | -R-- | ---- | ---- |
| E-006: Role | CR-- | CRUD | ---- | -R-- | ---- | ---- |
| E-007: UserRole | CRUD | CRUD | ---- | -R-- | --U- (expiry) | ---- |
| E-008: Document | CRUD | CRUD | CRUD | -R-- | --U- (state) | ---- |
| E-009: Template | CR-- | CRUD | -R-- | -R-- | ---- | ---- |
| E-010: AuditTrail | CR-- | -R-- | CR-- | -R-- | ---- | ---- |
| E-011: Entity | ---- | -R-- | CRUD | -R-- | --U- | ---- |
| E-012: Relationship | ---- | -R-- | CRUD | -R-- | --U- | ---- |
| E-013: Community | ---- | -R-- | CRUD | -R-- | --U- | ---- |
| E-014: DomainRelation | CR-- | CRUD | CRUD | -R-- | --U- | ---- |
| E-015: ContextVector | ---- | ---- | CRUD | -R-- | --U- | ---- |

**Legend**:

- **C** = Create (can insert new records)
- **R** = Read (can query existing records)
- **U** = Update (can modify existing records)
- **D** = Delete (can remove records)
- **-** = No access

**Access Control Implications**:

- Components with **C** access require input validation and business rule enforcement
- Components with **U** access require audit logging (before/after values)
- Components with **D** access require authorization checks and soft delete patterns
- Components with **R** only should use read-only database connections

**Security Considerations**:

- **Least Privilege**: Each component has minimum necessary permissions
- **Separation of Duties**: Critical operations (e.g., delete) restricted to admin roles
- **Audit Trail**: All CUD operations logged with timestamp, user, before/after values

---

## Data Integration Mapping

### Upstream Systems (Data Sources)

#### Integration INT-001: Existing Government Systems

**Source System**: Municipality case management systems (e.g., Sqills, Centric)

**Integration Type**: Batch ETL (nightly)

**Data Flow Direction**: [Source System] → [IOU-Modern]

**Entities Affected**:

- **E-002 (InformationDomain)**: Receives case/project data from source systems
  - Source Fields: Source.case_id → domain metadata
  - Update Frequency: Daily batch
  - Data Quality SLA: 95% accuracy, 24-hour latency

- **E-003 (InformationObject)**: Receives documents from source systems
  - Source Fields: Source.document_path → content_location
  - Update Frequency: Daily batch
  - Data Quality SLA: 95% completeness

**Data Mapping**:
| Source Field | Source Type | Target Entity | Target Attribute | Transformation |
|--------------|-------------|---------------|------------------|----------------|
| case_id | VARCHAR | E-002 | metadata->>'source_case_id' | Direct mapping |
| case_type | VARCHAR | E-002 | domain_type | Map to Zaak/Project |
| title | VARCHAR | E-003 | title | Trim whitespace |
| document_path | VARCHAR | E-003 | content_location | Add S3 prefix |

**Data Quality Rules**:

- **Validation**: Reject records with missing title or case_id
- **Deduplication**: Check for existing domain by source_case_id before creating
- **Error Handling**: Failed records logged to error table for manual review

**Reconciliation**:

- **Frequency**: Daily at 02:00 UTC
- **Method**: Compare record counts between source and target
- **Tolerance**: <5% variance acceptable (due to timing)

---

### Downstream Systems (Data Consumers)

#### Integration INT-101: Woo Publication Portal

**Target System**: Woo (Wet open overheid) publication platform

**Integration Type**: Real-time API

**Data Flow Direction**: [IOU-Modern] → [Woo Portal]

**Entities Shared**:

- **E-003 (InformationObject)**: Provides Woo-relevant documents for publication
  - Update Frequency: Real-time (webhook on publication)
  - Sync Method: REST API push
  - Data Latency SLA: <1 hour

**Data Mapping**:
| Source Entity | Source Attribute | Target Field | Target Type | Transformation |
|---------------|------------------|--------------|-------------|----------------|
| E-003 | title | woo.document_title | VARCHAR | Direct mapping |
| E-003 | content_text | woo.document_content | TEXT | Markdown to HTML |
| E-003 | woo_publication_date | woo.publication_date | TIMESTAMP | Direct mapping |
| E-003 | metadata->>'woo_decision' | woo.decision | JSON | Extract from metadata |

**Data Quality Assurance**:

- **Pre-send Validation**: Ensure is_woo_relevant = true and classification = Openbaar
- **Retry Logic**: 3 retries with exponential backoff on failure
- **Monitoring**: Alert if sync latency exceeds 1 hour

---

### Master Data Management (MDM)

**Source of Truth** (which system is authoritative for each entity):

| Entity | System of Record | Rationale | Conflict Resolution |
|--------|------------------|-----------|---------------------|
| E-001: Organization | IOU-Modern | Organizations mastered here | This system wins on conflict |
| E-002: InformationDomain | IOU-Modern | Domains created here | No conflicts (unique to IOU-Modern) |
| E-003: InformationObject | IOU-Modern | Objects stored in S3/MinIO | Source systems have read-only access |
| E-005: User | HR System (upstream) | Employee data managed in HR | HR system wins on conflict |
| E-011: Entity | IOU-Modern | Extracted by AI | AI extraction wins (can be corrected manually) |

**Data Lineage**:

- **E-003 (InformationObject)**: Created in [Source Systems] → Ingested by [IOU-Modern] → Published to [Woo Portal]
- **E-011 (Entity)**: Extracted from [E-003] → Linked to [E-002] → Used in [E-014 DomainRelation]

---

## Privacy & Compliance

### GDPR / UK Data Protection Act 2018 Compliance

#### PII Inventory

**Entities Containing PII**:

- **E-005 (User)**: email, display_name, first_name, last_name, phone, last_login
- **E-007 (UserRole)**: References User entities (PII via relationship)
- **E-011 (Entity)**: name, canonical_name for entity_type = Person (named individuals extracted from documents)
- **E-003 (InformationObject)**: content_text and metadata may contain personal data (tracked via privacy_level field)

**Total PII Attributes**: 11 attributes across 4 entities

**Special Category Data** (sensitive PII under GDPR Article 9):

- **None explicitly modeled**: Bijzonder and Strafrechtelijk privacy levels are supported in E-003 but not pre-defined
- When privacy_level = Bijzonder or Strafrechtelijk, additional protection required:
  - Access logging mandatory
  - Data minimization applied
  - DPIA review required

#### Legal Basis for Processing

| Entity | Purpose | Legal Basis | Notes |
|--------|---------|-------------|-------|
| E-005: User | Employee authentication, access control | Contract (GDPR Art 6(1)(b)) | Employment contract with government organization |
| E-003: InformationObject | Government record-keeping | Public task (GDPR Art 6(1)(e)) | Legal obligation under Archiefwet |
| E-011: Entity (Person) | Knowledge extraction for semantic search | Public task (GDPR Art 6(1)(e)) | Government information management |
| E-002: InformationDomain | Case/project management | Public task (GDPR Art 6(1)(e)) | Service delivery to citizens |

**Consent Management** (if applicable):

- **Opt-in Required**: None currently (all processing based on public task or contract)
- **Future**: Marketing communications (if added) would require explicit consent

#### Data Subject Rights Implementation

**Right to Access (Subject Access Request)**:

- **Endpoint**: `/api/v1/subject-access-request`
- **Authentication**: Multi-factor authentication required (DigiD support planned)
- **Response Format**: JSON containing all personal data
- **Response Time**: Within 30 days (GDPR requirement)
- **Entities Included**: E-005, E-003 (filtered by user's data), E-011 (Person entities where name matches)

**Right to Rectification**:

- **Endpoint**: `/api/v1/user/profile` (PUT)
- **UI**: Users can update their own profile via account settings
- **Admin Override**: Admin portal for data steward corrections
- **Propagation**: Updates synced within 1 hour

**Right to Erasure (Right to be Forgotten)**:

- **Method**: Anonymization (set PII fields to "Gewanonymeerd [entity_type]")
- **Process**:
  1. Data subject submits erasure request via support ticket
  2. Data Protection Officer reviews request (legal obligations check)
  3. If approved, anonymize PII within 30 days
  4. Preserve non-PII data for archival (legal holds)
- **Exceptions**: Cannot delete if legal obligation to retain (e.g., Archiefwet for formal decisions)

**Right to Data Portability**:

- **Endpoint**: `/api/v1/data-export`
- **Format**: JSON or CSV (machine-readable)
- **Scope**: E-005 user profile, E-003 objects created by user

**Right to Object**:

- **Marketing**: Not applicable (no marketing processing)
- **Profiling**: Opt-out available for entity extraction (E-011)

**Right to Restrict Processing**:

- **Flag**: E-005.processing_restricted (to be added)
- **Effect**: Data retained but not used in AI processing

#### Data Retention Schedule

| Entity | Active Retention | Archive Retention | Total Retention | Legal Basis | Deletion Method |
|--------|------------------|-------------------|-----------------|-------------|-----------------|
| E-005: User | Active + 2 years | 5 years | 7 years | AVG, employment law | Anonymize PII |
| E-003: InformationObject (Besluit) | Active | 20 years | 20 years | Archiefwet | Hard delete after 20 years |
| E-003: InformationObject (Document) | Active | 10 years | 10 years | Archiefwet | Hard delete after 10 years |
| E-003: InformationObject (Email) | Active | 5 years | 5 years | Archiefwet | Hard delete after 5 years |
| E-011: Entity (Person) | Same as source object | Same as source object | 20 years max | AVG | Hard delete with source |
| E-010: AuditTrail | 1 year | 6 years | 7 years | Compliance standards | Hard delete after 7 years |

**Retention Policy Enforcement**:

- **Automated Deletion**: Batch job runs monthly to delete data past retention period
- **Audit Trail**: Deletion events logged (entity ID, deletion date, reason)
- **Legal Hold**: Flag to prevent deletion during litigation or investigation

#### Cross-Border Data Transfers

**Data Locations**:

- **Primary Database**: Netherlands (EU region)
- **Backup Storage**: Netherlands (EU region)
- **S3/MinIO Storage**: On-premises or Netherlands (EU region)
- **Downstream Systems**: Netherlands/EU only (Woo portal)

**UK-EU Data Transfers**:

- **Adequacy Decision**: UK-EU adequacy decision in effect (2025)
- **Standard Contractual Clauses (SCCs)**: Not required for UK-EU transfers

**International Transfers**:

- **None**: All data processed within Netherlands/EU

#### Data Protection Impact Assessment (DPIA)

**DPIA Required**: YES

**Triggers for DPIA** (GDPR Article 35):

- ✅ Large-scale processing of special category data (potentially in E-003 with privacy_level = Bijzonder)
- ✅ Systematic monitoring (E-010 audit trail tracks all user actions)
- ✅ Automated decision-making with legal effects (E-008 document approval based on compliance_score)

**DPIA Status**: IN_PROGRESS (this document supports DPIA creation)

**DPIA Summary**:

- **Privacy Risks Identified**:
  1. Unauthorized access to PII in E-005 (User) entities
  2. PII leakage via E-011 (Entity) Person names
  3. Insufficient access controls for E-003 (InformationObject) with Vertrouwelijk classification
  4. Automated document approval (E-008) may publish incorrect information
- **Mitigation Measures**:
  1. Row-Level Security (RLS) in PostgreSQL for organization-level isolation
  2. Encryption at rest (AES-256) and in transit (TLS 1.3)
  3. Access logging for all PII-accessing operations
  4. Human approval required for Woo-relevant documents regardless of compliance_score
- **Residual Risk**: MEDIUM (mitigations reduce risks to acceptable level)
- **ICO Consultation Required**: NO (residual risk not high)

#### ICO Registration & Notifications

**ICO Registration**: REQUIRED (for UK deployments)

**Data Breach Notification**:

- **Breach Detection**: Automated monitoring, security alerts
- **Notification Deadline**: Within 72 hours if high risk to rights and freedoms (UK GDPR)
- **Data Subject Notification**: Without undue delay if high risk
- **Breach Log**: All breaches logged in incident management system

---

### Sector-Specific Compliance

#### Wet open overheid (Woo)

**Applicability**: APPLICABLE

**Woo-Relevant Entities**:

- **E-003 (InformationObject)**: All objects must be assessed for Woo relevance
  - `is_woo_relevant`: Boolean flag for Woo relevance
  - `woo_publication_date`: Date when document was published
  - `classification`: Determines if document can be published (Openbaar) or must be withheld

**Woo Controls**:

- **Automatic Assessment**: AI agent assesses Woo relevance and suggests disclosure_class
- **Human Review**: All Woo-relevant documents require human approval before publication
- **Publication**: Automatic publication to Woo portal for Openbaar documents
- **Withholding**: Documents with classification != Openbaar are not published

**Woo Metadata**:

- **Refusal Grounds**: Tracked in metadata for non-openbaar documents (e.g., Persoonlijke levenssfeer, Bedrijfsgegevens)
- **Publication Date**: Tracked for compliance with publication timelines

---

#### Archiefwet (Dutch Archives Act)

**Applicability**: APPLICABLE

**Retention Requirements**:

- **Besluit (Formal decisions)**: 20 years retention, then transfer to Nationaal Archief
- **Documenten**: 10 years retention
- **Email**: 5 years retention
- **Chat**: 1 year retention

**Archival Values**:

- **Permanent**: Besluit documents with historical value
- **Tijdelijk**: Most administrative documents

**Selection List**:

- **Selectielijst (2021)**: Reference for retention periods per document type
- **Metadata Tracking**: `selection_list_ref` field for selectielijst reference

---

#### AVG (GDPR Netherlands)

**Applicability**: APPLICABLE

**Privacy Levels**:

- **Openbaar**: No personal data
- **Normaal**: Regular personal data (GDPR Article 6)
- **Bijzonder**: Special category data (GDPR Article 9) - health, ethnicity, religion, etc.
- **Strafrechtelijk**: Criminal offense data (GDPR Article 10)

**Legal Basis Tracking**:

- **Wettelijke verplichting**: Legal obligation (most government processing)
- **Openbaar gezag**: Public authority
- **Gerechtvaardigd belang**: Legitimate interest

**Data Subject Rights**:

- Implemented via API endpoints for access, rectification, erasure, portability

---

## Data Quality Framework

### Quality Dimensions

#### Accuracy

**Definition**: Data correctly represents the real-world entity or event

**Quality Targets**:
| Entity | Attribute | Accuracy Target | Measurement Method | Owner |
|--------|-----------|-----------------|-------------------|-------|
| E-005: User | email | 99% valid emails | Bounce rate monitoring | HR Department |
| E-003: InformationObject | classification | 95% accurate | AI agent vs human review | Domain Owner |
| E-011: Entity | name | 90% accurate | NER precision metric | Knowledge Graph Team |

**Validation Rules**:

- **Email**: RFC 5322 format, MX record exists
- **Classification**: Must be one of Openbaar, Intern, Vertrouwelijk, Geheim
- **UUID**: Must be valid UUID v4 format

#### Completeness

**Definition**: All required data elements are populated

**Quality Targets**:
| Entity | Required Fields Completeness | Target | Current | Owner |
|--------|------------------------------|--------|---------|-------|
| E-001: Organization | name, organization_type | 100% | 100% | Domain Administrator |
| E-003: InformationObject | title, domain_id, classification | 100% | TBD | Domain Owner |
| E-005: User | email, display_name | 100% | TBD | HR Department |

**Missing Data Handling**:

- **Required Fields**: Reject record creation if missing
- **Optional Fields**: Allow NULL, track completeness % for reporting

#### Consistency

**Definition**: Data is consistent across systems and does not contradict itself

**Quality Targets**:

- **Referential Integrity**: 100% (all foreign keys reference valid records)
- **Status Consistency**: 99% (document state transitions are valid)

**Reconciliation Process**:

- **Frequency**: Weekly referential integrity check
- **Method**: Database foreign key validation
- **Discrepancy Resolution**: Automated fix if <1% variance, manual review if >1%

#### Timeliness

**Definition**: Data is up-to-date and available when needed

**Quality Targets**:
| Entity | Update Frequency | Staleness Tolerance | Current Latency | Owner |
|--------|------------------|---------------------|-----------------|-------|
| E-003: InformationObject | Real-time | <1 hour | TBD | Integration Team |
| E-011: Entity | Batch (daily) | <24 hours | TBD | Knowledge Graph Team |
| E-008: Document | Real-time | <5 minutes | TBD | Document Pipeline Team |

#### Uniqueness

**Definition**: No duplicate records exist

**Deduplication Rules**:
| Entity | Unique Key | Deduplication Logic | Duplicate Resolution |
|--------|------------|---------------------|----------------------|
| E-005: User | email, organization_id | Case-insensitive email match | Merge records, keep oldest user_id |
| E-011: Entity | name, entity_type, domain_id | Exact match within domain | Merge, keep highest confidence |

#### Validity

**Definition**: Data conforms to defined formats, ranges, and business rules

**Validation Rules**:
| Attribute | Format/Range | Invalid Example | Handling |
|-----------|--------------|-----------------|----------|
| email | RFC 5322 | "not-an-email" | Reject on insert |
| domain_type | Enum | "Invalid" | Reject on insert |
| classification | Enum | "Unknown" | Reject on insert |
| compliance_score | 0.0-1.0 | 1.5 | Reject on insert |

---

## Requirements Traceability

**Purpose**: Ensure all domain model requirements are documented

| Requirement ID | Requirement Description | Entity | Attributes | Status | Notes |
|----------------|------------------------|--------|------------|--------|-------|
| DM-001 | Store government organizations | E-001: Organization | All attributes | ✅ Implemented | Rijk, Provincie, Gemeente, etc. |
| DM-002 | Store organizational structure (departments) | E-004: Department | All attributes | ✅ Implemented | Hierarchical departments |
| DM-003 | Store users with PII | E-005: User | email, display_name, first_name, last_name, phone | ✅ Implemented | PII tracked for AVG compliance |
| DM-004 | Implement role-based access control | E-006: Role, E-007: UserRole | All attributes | ✅ Implemented | Scoped to domains |
| DM-005 | Store information domains (Zaak, Project, Beleid, Expertise) | E-002: InformationDomain | All attributes | ✅ Implemented | Four domain types |
| DM-006 | Store information objects with compliance metadata | E-003: InformationObject | All attributes | ✅ Implemented | Woo, AVG, Archiefwet compliance |
| DM-007 | Support document workflow | E-008: Document | state, compliance_score, confidence_score | ✅ Implemented | Multi-state workflow |
| DM-008 | Store document templates | E-009: Template | All attributes | ✅ Implemented | Markdown with Tera |
| DM-009 | Audit trail for document workflow | E-010: AuditTrail | All attributes | ✅ Implemented | Full observability |
| DM-010 | Extract and store named entities | E-011: Entity | All attributes | ✅ Implemented | NER-based extraction |
| DM-011 | Store entity relationships | E-012: Relationship | All attributes | ✅ Implemented | GraphRAG support |
| DM-012 | Cluster entities into communities | E-013: Community | All attributes | ✅ Implemented | Hierarchical clustering |
| DM-013 | Discover cross-domain relationships | E-014: DomainRelation | All attributes | ✅ Implemented | Automatic discovery |
| DM-014 | Support semantic search via embeddings | E-015: ContextVector | All attributes | ✅ Implemented | Vector similarity |
| DM-015 | GDPR compliance - PII tracking | E-003, E-005, E-011 | privacy_level, PII attributes | ✅ Implemented | Per-entity PII tracking |
| DM-016 | Woo compliance - publication tracking | E-003: InformationObject | is_woo_relevant, woo_publication_date | ✅ Implemented | Woo publication support |
| DM-017 | Archiefwet compliance - retention periods | E-003: InformationObject | retention_period | ✅ Implemented | Type-based retention |

**Coverage Summary**:

- **Total Requirements**: 17
- **Requirements Implemented**: 17 (100%)
- **Coverage**: 100%

---

## Implementation Guidance

### Database Technology Recommendation

**Recommended Database**: PostgreSQL 15+ (transactional) + DuckDB (analytical)

**Rationale**:

- **PostgreSQL**:
  - Use Case: Transactional data (E-001 through E-015)
  - Benefits: ACID guarantees, mature tooling, Row-Level Security (RLS), pgvector for embeddings
  - Compliance: GDPR-compliant tooling, encryption at rest

- **DuckDB**:
  - Use Case: Analytics, full-text search, vector similarity
  - Benefits: In-memory analytical queries, SQL-compatible, Parquet export
  - Integration: Hybrid architecture with PostgreSQL as primary, DuckDB for analytics

**Chosen Technology**: Hybrid PostgreSQL + DuckDB

- **Justification**: PostgreSQL for transactional integrity and compliance, DuckDB for analytics performance
- **Cloud Provider**: AWS RDS (PostgreSQL) + S3 (DuckDB files) or self-hosted
- **High Availability**: Multi-AZ deployment for PostgreSQL, S3 redundancy for DuckDB

---

### Schema Migration Strategy

**Migration Tool**: Diesel ORM (Rust) or custom SQL migrations

**Versioning**:

- **Schema Version**: v1.0.0
- **Migration Scripts**: Stored in `migrations/postgres/` directory
- **Naming Convention**: `NNN_description.sql` (e.g., `001_create_initial_schema.sql`)

**Migration Process**:

1. **Development**: Create migration in local environment
2. **Testing**: Run on test database, verify data integrity
3. **Peer Review**: Code review for data loss and performance
4. **Staging**: Deploy to staging, run smoke tests
5. **Production**: Deploy during maintenance window

---

### Backup and Recovery

**Backup Strategy**:

- **PostgreSQL**: Daily full backup + continuous WAL archiving
- **DuckDB**: Export to Parquet daily, stored in S3 with versioning
- **Backup Retention**: 30 days online, 7 years archival

**Recovery Point Objective (RPO)**: <1 hour

**Recovery Time Objective (RTO)**: <4 hours

**Disaster Recovery**:

- **Multi-Region**: Not required (Netherlands-only deployment)
- **Failover**: Standby replica for automatic failover
- **DR Testing**: Quarterly DR drills

---

### Data Archival

**Archival Policy**:

- **Active Data**: Frequently accessed (last 2 years) → PostgreSQL + S3 hot storage
- **Archived Data**: Rarely accessed (2-20 years) → S3 Glacier
- **Deleted Data**: Past retention period → Permanent deletion

---

### Testing Data Strategy

**Test Data Requirements**:

- **Volume**: 10% of production data volume
- **Diversity**: All entity types, edge cases

**Data Anonymization** (for non-production):

- **E-005 (User)**: Anonymize email, name, phone
- **E-011 (Entity)**: Anonymize Person entity names
- **Referential Integrity**: Maintain while anonymizing

---

## Appendix

### Glossary

- **PII (Personally Identifiable Information)**: Data that can identify an individual
- **AVG**: Algemene verordening gegevensbescherming (GDPR Netherlands)
- **Woo**: Wet open overheid (Government Information Act)
- **Archiefwet**: Dutch Archives Act governing record retention
- **GraphRAG**: Graph-based Retrieval Augmented Generation
- **NER**: Named Entity Recognition
- **RLS**: Row-Level Security (PostgreSQL feature)
- **DPIA**: Data Protection Impact Assessment
- **S3**: Simple Storage Service (object storage)
- **WAL**: Write-Ahead Log (PostgreSQL transaction log)

### References

- [Archiefwet 1995](https://wetten.overheid.nl/BWBR0006646/2021-01-01) - Dutch Archives Act
- [Wet open overheid](https://wetten.overheid.nl/BWBR0036881/2022-05-01) - Government Information Act
- [AVG/GDPR](https://gdpr-info.eu/) - General Data Protection Regulation
- [PostgreSQL Documentation](https://www.postgresql.org/docs/) - Database documentation
- [DuckDB Documentation](https://duckdb.org/docs/) - Analytical database documentation
- [Rust crates/iou-core](../../crates/iou-core/src/) - Domain model source code

---

**Document End**

*This data model is a living document and should be updated as requirements evolve, new entities are added, or compliance regulations change.*

## External References

| Document | Type | Source | Key Extractions | Path |
|----------|------|--------|-----------------|------|
| *None provided* | — | — | — | — |

---

**Generated by**: ArcKit `/arckit:data-model` command
**Generated on**: 2026-03-20
**ArcKit Version**: 4.3.1
**Project**: IOU-Modern (Project 001)
**AI Model**: Claude Opus 4.6
