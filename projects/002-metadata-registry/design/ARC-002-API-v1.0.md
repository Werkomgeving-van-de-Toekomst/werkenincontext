# API Design: Metadata Registry Service

> **Document ID**: ARC-002-API-v1.0 | **Status**: DRAFT

## Document Control

| Field | Value |
|-------|-------|
| **Document Type** | API Design Specification |
| **Project** | Metadata Registry Service (Project 002) |
| **Version** | 1.0 |
| **Created Date** | 2026-04-19 |

---

## 1. API Overview

### 1.1 Base URLs

| Environment | Base URL |
|-------------|----------|
| Development | https://api.dev.metadata-registry.nl |
| Test | https://api.test.metadata-registry.nl |
| Production | https://api.metadata-registry.nl |

### 1.2 Authentication

```
Authorization: Bearer <access_token>
```

Token validation via OAuth 2.0 / OpenID Connect.

---

## 2. REST API v2 Specification

### 2.1 Common Headers

```
Content-Type: application/json
Accept: application/json
Authorization: Bearer <token>
X-Request-ID: <uuid>
X-Organisation-ID: <org_id>
```

### 2.2 Common Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 204 | No Content |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Not Found |
| 409 | Conflict |
| 422 | Unprocessable Entity |
| 429 | Rate Limited |
| 500 | Internal Server Error |

### 2.3 Error Response Format

```json
{
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Field 'naam' is required",
    "field": "naam",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

---

## 3. Gebeurtenis (Event) Endpoints

### 3.1 List Gebeurtenissen

```http
GET /api/v2/gebeurtenissen
```

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| organisatie_id | string | No | Filter by organization |
| geldig_op | datetime | No | Filter by validity at timestamp |
| gebeurtenistype | string | No | Filter by event type |
| limit | integer | No | Max results (default: 50, max: 500) |
| offset | integer | No | Pagination offset |
| sort_by | string | No | Sort field (default: aangemaakt_op) |
| sort_order | string | No | asc or desc (default: desc) |

**Request Example:**
```http
GET /api/v2/gebeurtenissen?organisatie_id=org-123&limit=20&geldig_op=2024-04-19T10:00:00Z
```

**Response Example:**
```json
{
  "data": [
    {
      "_key": "evt-001",
      "naam": "Burger aanvraag uitkering",
      "omschrijving": "Aanvraag van een uitkering door een burger",
      "gebeurtenistype": "aanvraag",
      "geldig_vanaf": "2024-01-01T00:00:00Z",
      "geldig_tot": "9999-12-31T23:59:59Z",
      "organisatie_id": "org-123",
      "aangemaakt_door": "user-456",
      "aangemaakt_op": "2024-04-19T10:00:00Z"
    }
  ],
  "pagination": {
    "total": 100,
    "limit": 20,
    "offset": 0
  }
}
```

### 3.2 Get Gebeurtenis

```http
GET /api/v2/gebeurtenissen/{key}
```

**Response Example:**
```json
{
  "_key": "evt-001",
  "naam": "Burger aanvraag uitkering",
  "omschrijving": "Aanvraag van een uitkering door een burger",
  "gebeurtenistype": "aanvraag",
  "geldig_vanaf": "2024-01-01T00:00:00Z",
  "geldig_tot": "9999-12-31T23:59:59Z",
  "organisatie_id": "org-123",
  "aangemaakt_door": "user-456",
  "aangemaakt_op": "2024-04-19T10:00:00Z",
  "gewijzigd_door": "user-789",
  "gewijzigd_op": "2024-04-19T11:00:00Z"
}
```

### 3.3 Create Gebeurtenis

```http
POST /api/v2/gebeurtenissen
Content-Type: application/json
```

**Request Body:**
```json
{
  "naam": "Burger aanvraag uitkering",
  "omschrijving": "Aanvraag van een uitkering door een burger",
  "gebeurtenistype": "aanvraag",
  "geldig_vanaf": "2024-01-01T00:00:00Z",
  "geldig_tot": "9999-12-31T23:59:59Z",
  "organisatie_id": "org-123"
}
```

**Response (201 Created):**
```json
{
  "_key": "evt-001",
  "message": "Gebeurtenis created successfully"
}
```

### 3.4 Update Gebeurtenis

```http
PUT /api/v2/gebeurtenissen/{key}
Content-Type: application/json
```

**Request Body:**
```json
{
  "naam": "Burger aanvraag uitkering (gewijzigd)",
  "omschrijving": "Aanvraag van een uitkering door een burger",
  "gebeurtenistype": "aanvraag",
  "geldig_vanaf": "2024-01-01T00:00:00Z",
  "geldig_tot": "9999-12-31T23:59:59Z"
}
```

**Note**: Cannot update `organisatie_id`, `aangemaakt_door`, `aangemaakt_op`.

### 3.5 Delete Gebeurtenis

```http
DELETE /api/v2/gebeurtenissen/{key}
```

**Response (204 No Content)**

---

## 4. Informatieobject Endpoints

### 4.1 Create Informatieobject

```http
POST /api/v2/informatieobjecten
Content-Type: application/json
```

**Request Body:**
```json
{
  "dataobject_id": "do-001",
  "naam": "Besluit ook openbaar",
  "omschrijving": "Besluit over openbaarmaking van documenten",
  "objecttype": "besluit",
  "informatiecategorie": "besluit",
  "documenttype": "pdf",
  "beveiligingsniveau": "openbaar",
  "privacy_level": "geen",
  "zaak_id": "zaak-001",
  "geldig_vanaf": "2024-01-01T00:00:00Z",
  "geldig_tot": "9999-12-31T23:59:59Z",
  "organisatie_id": "org-123"
}
```

**Enum Values:**

`beveiligingsniveau`: openbaar, intern, vertrouwelijk, zeer_vertrouwelijk

`privacy_level`: geen, laag, middel, hoog

`informatiecategorie`: besluit, rapport, nota, correspondentie, agenda, verslag, andere

### 4.2 Update Informatieobject Status

```http
PATCH /api/v2/informatieobjecten/{key}/status
Content-Type: application/json
```

**Request Body:**
```json
{
  "status": "gepersistent",
  "reden": "Goedgekeurd door Woo officer"
}
```

**Status Transitions:**
- `dynamisch` → `gepersistent`: Requires approval
- `gepersistent` → `gearchiveerd`: Requires archival
- `gearchiveerd`: Final state (no transitions)

---

## 5. Search Endpoints

### 5.1 Full-Text Search

```http
GET /api/v2/search?q={query}&type={type}
```

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| q | string | Yes | Search query |
| type | string | No | Entity type filter |
| limit | integer | No | Max results (default: 20) |
| offset | integer | No | Pagination offset |

**Request Example:**
```http
GET /api/v2/search?q=uitkering&type=gebeurtenis&limit=10
```

**Response Example:**
```json
{
  "data": [
    {
      "entity_type": "gebeurtenis",
      "key": "evt-001",
      "naam": "Burger aanvraag uitkering",
      "score": 0.95,
      "highlight": "Burger aanvraag <em>uitkering</em>"
    }
  ],
  "total": 5
}
```

### 5.2 Context-Aware Search

```http
POST /api/v2/search/context
Content-Type: application/json
```

**Request Body:**
```json
{
  "query": "uitkering",
  "context": {
    "zaak_id": "zaak-001",
    "organisatie_id": "org-123",
    "domein": "sociale_zekerheid"
  },
  "limit": 20
}
```

**Response Example:**
```json
{
  "data": [
    {
      "key": "evt-001",
      "naam": "Burger aanvraag uitkering",
      "context_match": 0.8,
      "text_relevance": 0.95,
      "final_score": 1.71
    }
  ]
}
```

---

## 6. GraphQL API

### 6.1 Schema

Endpoint: `https://api.metadata-registry.nl/graphql`

```graphql
type Query {
  gebeurtenis(key: String!): Gebeurtenis
  gebeurtenissen(
    organisatie_id: String
    geldig_op: DateTime
    limit: Int
    offset: Int
  ): GebeurtenisConnection!
  
  informatieobject(key: String!): Informatieobject
  informatieobjecten(
    organisatie_id: String
    status: InformatieobjectStatus
    limit: Int
  ): InformatieobjectConnection!
  
  search(query: String!, type: String): SearchResultConnection!
}

type Mutation {
  createGebeurtenis(input: GebeurtenisInput!): Gebeurtenis!
  updateGebeurtenis(key: String!, input: GebeurtenisInput!): Gebeurtenis!
  deleteGebeurtenis(key: String!): Boolean!
  
  createInformatieobject(input: InformatieobjectInput!): Informatieobject!
  updateInformatieobjectStatus(
    key: String!
    status: InformatieobjectStatus!
    reden: String
  ): Informatieobject!
}

type Gebeurtenis {
  key: ID!
  naam: String!
  omschrijving: String
  gebeurtenistype: GebeurtenisType
  geldig_vanaf: DateTime!
  geldig_tot: DateTime!
  organisatie_id: String!
  aangemaakt_door: String!
  aangemaakt_op: DateTime!
  
  # Relations
  gegevensproducten: [Gegevensproduct!]!
  context: [Context!]!
  grondslagen: [Grondslag!]!
}

type Informatieobject {
  key: ID!
  dataobject_id: String!
  naam: String!
  objecttype: String!
  informatiecategorie: Informatiecategorie!
  beveiligingsniveau: Beveiligingsniveau!
  privacy_level: PrivacyLevel!
  status: InformatieobjectStatus!
  zaak_id: String
  organisatie_id: String!
  
  # Relations
  zaak: Zaak
  catalogus: InformatieobjectCatalogus
  rechten: [InformatieobjectRecht!]!
  ai_enrichment: AIEnrichment
}

type GebeurtenisConnection {
  edges: [GebeurtenisEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type GebeurtenisEdge {
  node: Gebeurtenis!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}

enum GebeurtenisType {
  AANVRAAG
  BESCHIKKING
  MELDING
  TAAKUITVOERING
  ANDERE
}

enum InformatieobjectStatus {
  DYNAMISCH
  GEPERSISTEERD
  GEARCHIVEERD
}
```

### 6.2 Example Queries

**Get Gebeurtenis with Relations:**
```graphql
query GetGebeurtenis {
  gebeurtenis(key: "evt-001") {
    key
    naam
    gebeurtenistype
    gegevensproducten {
      key
      naam
    }
    grondslagen {
      naam
      type
    }
  }
}
```

**Context-Aware Search:**
```graphql
query SearchContext {
  search(
    query: "uitkering"
    type: "gebeurtenis"
  ) {
    data {
      key
      naam
      score
      ... on Gebeurtenis {
        gebeurtenistype
      }
    }
  }
}
```

**Create Informatieobject:**
```graphql
mutation CreateInformatieobject {
  createInformatieobject(input: {
    dataobject_id: "do-001"
    naam: "Besluit ook openbaar"
    objecttype: "besluit"
    informatiecategorie: BESLUIT
    beveiligingsniveau: OPENBAAR
    privacy_level: GEEN
    organisatie_id: "org-123"
  }) {
    key
    naam
    status
  }
}
```

---

## 7. WebSocket API

### 7.1 Connection

```
wss://api.metadata-registry.nl/ws
```

### 7.2 Authentication

```json
{
  "type": "auth",
  "token": "Bearer <access_token>"
}
```

### 7.3 Events

**Entity Created:**
```json
{
  "type": "entity.created",
  "entity_type": "gebeurtenis",
  "key": "evt-001",
  "data": { ... }
}
```

**Entity Updated:**
```json
{
  "type": "entity.updated",
  "entity_type": "gebeurtenis",
  "key": "evt-001",
  "data": { ... }
}
```

**Sync Complete:**
```json
{
  "type": "sync.complete",
  "timestamp": "2024-04-19T10:00:00Z",
  "files_processed": 10,
  "entities_created": 5,
  "entities_updated": 3
}
```

---

## 8. Rate Limiting

| Tier | Requests | Window |
|------|----------|--------|
| Free | 100 | 1 minute |
| Standard | 1,000 | 1 minute |
| Premium | 10,000 | 1 minute |

**Response Headers:**
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 950
X-RateLimit-Reset: 1713528000
```

---

## 9. Related Documents

- ARC-002-DLD-v1.0: Detailed Design
- ARC-002-DB-v1.0: Database Design
- ARC-002-SEC-v1.0: Security Design
