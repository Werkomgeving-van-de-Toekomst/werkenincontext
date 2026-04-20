# Architecture Diagram: Sequence - Woo Publication Workflow

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Command**: `/arckit:diagram sequence`

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-002-DIAG-005-v1.0 |
| **Document Type** | Architecture Diagram |
| **Project** | Metadata Registry Service (Project 002) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.0 |
| **Created Date** | 2026-04-19 |
| **Last Modified** | 2026-04-19 |
| **Review Cycle** | On-Demand |
| **Next Review Date** | 2026-05-19 |
| **Owner** | Enterprise Architect |
| **Reviewed By** | PENDING |
| **Approved By** | PENDING |
| **Distribution** | Project Team, Architecture Team, Woo Officers |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---------|------|--------|---------|-------------|---------------|
| 1.0 | 2026-04-19 | ArcKit AI | Initial creation from `/arckit:diagram sequence` command | PENDING | PENDING |

---

## Diagram Purpose

This sequence diagram shows the complete Wet open overheid (Woo) publication workflow, from information object creation to final publication on the Woo portal, including all validation, approval, and publication steps.

---

## Woo Publication Workflow

```mermaid
sequenceDiagram
    autonumber

    participant Steward as Metadata Steward
    participant Admin as Admin UI
    participant API as REST API
    participant Validator as Validation Engine
    participant DB as ArangoDB
    participant WooOfficer as Woo Officer
    participant WooAPI as Woo Portal API
    participant CDD as CDD+ Archive

    Note over Steward,CDD: Phase 1: Creation and Classification

    Steward->>Admin: Create Informatieobject
    Admin->>API: POST /api/v2/informatieobjecten
    API->>Validator: Validate entity
    Validator->>Validator: Check required fields
    Validator->>Validator: Validate Woo category
    Validator-->>API: Validation result
    API->>DB: Create entity (status=dynamisch)
    DB-->>API: Entity created
    API-->>Admin: 201 Created
    Admin-->>Steward: Object created

    Note over Steward,CDD: Phase 2: Woo Relevance Assessment

    Steward->>Admin: Mark for Woo publication
    Admin->>API: PUT /informatieobject/{id}/woo
    API->>DB: Update with woo_relevant=true
    DB->>DB: Set status=woobeoordeling
    API-->>Admin: Assessment required

    Steward->>Admin: Complete Woo assessment
    Admin->>API: POST /woo/beoordeling
    API->>DB: Store WooPublicatie entity
    DB->>DB: Set status=goedkeuring
    API-->>Admin: Assessment saved

    Note over Steward,CDD: Phase 3: AI Enrichment

    API->>API: Trigger AI enrichment
    API->>API: Generate summary (AI)
    API->>API: Detect PII (AI)
    API->>DB: Store AIEnrichment (ongecontroleerd)
    Note right of DB: ai_status=ongecontroleerd

    Steward->>Admin: Review AI enrichment
    Admin->>API: Validate AI results
    API->>DB: Update ai_status=getoetst
    DB-->>API: Validated
    API-->>Admin: AI approved

    Note over Steward,CDD: Phase 4: Woo Officer Approval

    WooOfficer->>Admin: Review publication request
    Admin->>API: GET /woo/queue
    API->>DB: Query status=goedkeuring
    DB-->>API: Pending publications
    API-->>Admin: Approval queue

    WooOfficer->>Admin: Approve publication
    Admin->>API: POST /woo/{id}/approve
    API->>Validator: Validate completeness
    Validator->>Validator: Check Woo fields
    Validator->>Validator: Validate AI status=getoetst
    Validator-->>API: Ready to publish

    API->>DB: Transition status: goedkeuring → te_publiceren
    API->>DB: Transition entity: dynamisch → gepersistent
    Note right of DB: Entity now read-only
    API-->>Admin: Approved for publication

    Note over Steward,CDD: Phase 5: Publication

    API->>WooAPI: POST /publicaties
    WooAPI->>WooAPI: Validate publication
    WooAPI-->>API: 202 Accepted
    API->>DB: Update status=gepubliceerd
    API->>DB: Store publication_date

    WooAPI->>WooAPI: Process publication
    Note right of WooAPI: Async processing
    WooAPI-->>API: Webhook: published
    API->>DB: Update publicatie_url

    Note over Steward,CDD: Phase 6: Archival (Optional)

    opt Retention period expired
        API->>CDD: POST /archive
        CDD->>CDD: Store in archive
        CDD-->>API: Archive reference
        API->>DB: Store archiefmetadata
        API->>DB: Transition status: gepersistent → gearchiveerd
    end

    API-->>Admin: Publication complete
    Admin-->>Steward: Notification: Published
    Admin-->>WooOfficer: Notification: Published
```

---

## Workflow States

### State Transitions

```
┌──────────────┐
│  Creatie     │
│ (dynamisch)  │
└──────┬───────┘
       │ Mark for Woo
       ▼
┌──────────────┐
│ Woobeoordeling│
│ (assessment) │
└──────┬───────┘
       │ Assessment complete
       ▼
┌──────────────┐
│ Goedkeuring   │
│ (approval)   │
└──────┬───────┘
       │ Woo Officer approves
       ▼
┌──────────────┐
│Te_publiceren │
│ (pending)    │
└──────┬───────┘
       │ Submit to Woo
       ▼
┌──────────────┐
│ Gepubliceerd │
│ (published)  │
└──────┬───────┘
       │ Archive
       ▼
┌──────────────┐
│ Gearchiveerd │
└──────────────┘
```

### Status Values

| Status | Description | Mutable | Published |
|--------|-------------|---------|-----------|
| `creatie` | Initial creation | Yes | No |
| `woobeoordeling` | Woo assessment pending | Yes | No |
| `goedkeuring` | Pending Woo Officer approval | Yes | No |
| `te_publiceren` | Approved, pending submission | No | No |
| `gepubliceerd` | Published on Woo portal | No | Yes |
| `gearchiveerd` | Archived in CDD+ | No | Yes |

---

## API Endpoints

### Creation

```http
POST /api/v2/informatieobjecten
Content-Type: application/json

{
  "naam": "Besluit ook openbaar",
  "omschrijving": "...",
  "objecttype": "besluit",
  "informatiecategorie": "besluit",
  "documenttype": "pdf",
  "beveiligingsniveau": "openbaar",
  "organisatie_id": "org-123"
}
```

### Woo Assessment

```http
POST /api/v2/informatieobject/{id}/woo/beoordeling
Content-Type: application/json

{
  "woo_relevant": true,
  "publicatie_binnen": "10_werken",
  "ingang_publicatie": "2024-05-01",
  "redenering": "Besluit van openbaar belang"
}
```

### AI Validation

```http
PUT /api/v2/informatieobject/{id}/ai/validate
Content-Type: application/json

{
  "ai_status": "getoetst",
  "validator_id": "user-456",
  "opmerking": "Summary is accurate"
}
```

### Woo Officer Approval

```http
POST /api/v2/woo/{id}/approve
Authorization: Bearer <woo-officer-token>

{
  "goedgekeurd": true,
  "redenering": "Voldoet aan Woo vereisten"
}
```

### Publication

```http
POST /api/v2/woo/{id}/publish
Authorization: Bearer <service-token>

{
  "publicatie_datum": "2024-05-01",
  "metadata_volledig": true
}
```

---

## Data Structures

### WooPublicatie Entity

```rust
pub struct WooPublicatie {
    pub _key: String,
    pub informatieobject_id: String,

    // Assessment
    pub woo_relevant: bool,
    pub publicatie_binnen: PublicatieBinnen,
    pub ingang_publicatie: Date,
    pub redenering: String,

    // AI Enrichment
    pub samenvatting: Option<String>,
    pub ai_status: AIStatus,
    pub vertrouwensscore: f64,
    pub getoetst_door: Option<String>,
    pub getoetst_op: Option<DateTime>,

    // Approval
    pub goedgekeurd_door: Option<String>,
    pub goedgekeurd_op: Option<DateTime>,
    pub goedkeurings_redenering: Option<String>,

    // Publication
    pub status: WooStatus,
    pub publicatie_datum: Option<Date>,
    pub publicatie_url: Option<String>,
    pub woo_referentie: Option<String>,

    // Audit
    pub organisatie_id: String,
    pub aangemaakt_door: String,
    pub aangemaakt_op: DateTime,
}
```

---

## Error Handling

| Error | Condition | HTTP Status | User Action |
|-------|-----------|-------------|-------------|
| `INVALID_WOO_CATEGORY` | informatiecategorie not valid | 400 | Select valid Woo category |
| `AI_NOT_VALIDATED` | ai_status=ongecontroleerd | 403 | Validate AI results first |
| `MISSING_REQUIRED_FIELDS` | Woo fields incomplete | 400 | Complete assessment |
| `NOT_WOO_RELEVANT` | woo_relevant=false | 400 | Mark as Woo relevant first |
| `ALREADY_PUBLISHED` | status=gepubliceerd | 409 | Cannot re-publish |
| `READ_ONLY_ENTITY` | status=gepersistent | 403 | Cannot modify read-only entity |
| `UNAUTHORIZED` | Non-Woo officer approves | 403 | Get Woo officer approval |

---

## Related Documents

- **ARC-002-REQ-v1.1**: FR-MREG-9 (Woo Publication Workflow)
- **ARC-002-ADR-004**: BSW Alignment (AI validation requirement)
- **ARC-002-DIAG-006**: GitOps Sync Flow
