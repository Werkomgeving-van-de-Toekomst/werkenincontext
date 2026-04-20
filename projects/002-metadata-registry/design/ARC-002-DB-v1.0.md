# Database Design: Metadata Registry Service

> **Document ID**: ARC-002-DB-v1.0 | **Status**: DRAFT

## Document Control

| Field | Value |
|-------|-------|
| **Document Type** | Database Design Specification |
| **Project** | Metadata Registry Service (Project 002) |
| **Version** | 1.0 |
| **Created Date** | 2026-04-19 |

---

## 1. Database Overview

### 1.1 Technology

**Database**: ArangoDB 3.11.x (Community Edition)
**Deployment Mode**: Cluster (1 coordinator, 3 DB servers)
**Sharding**: Hash-based on `_key`

### 1.2 Naming Conventions

- Collection names: `snake_case`
- Document keys: `{type}-{id}` (e.g., `evt-001`)
- Edge collections: `{from}_{to}` (e.g., `gebeurtenis_gegevensproduct`)

---

## 2. Collection Specifications

### 2.1 Document Collections

#### gebeurtenis (Events)

```javascript
{
  "_key": "evt-001",
  "naam": "Burger aanvraag uitkering",
  "omschrijving": "Aanvraag van een uitkering door een burger",
  "gebeurtenistype": "aanvraag",  // enum: aanvraag, beschikking, melding, taakuitvoering, andere
  "geldig_vanaf": "2024-01-01T00:00:00Z",
  "geldig_tot": "9999-12-31T23:59:59Z",
  "organisatie_id": "org-123",
  "aangemaakt_door": "user-456",
  "aangemaakt_op": "2024-04-19T10:00:00Z",
  "gewijzigd_door": "user-789",
  "gewijzigd_op": "2024-04-19T11:00:00Z"
}
```

**Indexes:**
```javascript
// Primary (automatic)
db.gebeurtenis.ensureIndex({ type: "primary", fields: ["_key"] });

// Time validity (persistent)
db.gebeurtenis.ensureIndex({
  type: "persistent",
  fields: ["geldig_vanaf", "geldig_tot"],
  unique: false
});

// Organization (persistent)
db.gebeurtenis.ensureIndex({
  type: "persistent",
  fields: ["organisatie_id"],
  unique: false
});

// Full-text search
db.gebeurtenis.ensureFullTextIndex(["naam", "omschrijving"]);
```

---

#### informatieobject (Information Objects - BSW)

```javascript
{
  "_key": "info-001",
  "dataobject_id": "do-001",
  "naam": "Besluit ook openbaar",
  "omschrijving": "Besluit over openbaarmaking van documenten",
  "objecttype": "besluit",
  
  // Woo required
  "informatiecategorie": "besluit",
  "documenttype": "pdf",
  
  // Security classification
  "beveiligingsniveau": "openbaar",
  "privacy_level": "geen",
  
  // BSW status
  "status": "dynamisch",
  "workflow_status": null,
  "workflow_id": null,
  
  // Optional references
  "zaak_id": "zaak-001",
  "samenvatting": null,
  
  // Time validity
  "geldig_vanaf": "2024-01-01T00:00:00Z",
  "geldig_tot": "9999-12-31T23:59:59Z",
  
  // Ownership
  "organisatie_id": "org-123",
  "aangemaakt_door": "user-456",
  "aangemaakt_op": "2024-04-19T10:00:00Z"
}
```

**Indexes:**
```javascript
// Primary
db.informatieobject.ensureIndex({ type: "primary", fields: ["_key"] });

// Time validity
db.informatieobject.ensureIndex({
  type: "persistent",
  fields: ["geldig_vanaf", "geldig_tot"],
  unique: false
});

// Organization (multi-tenancy)
db.informatieobject.ensureIndex({
  type: "persistent",
  fields: ["organisatie_id"],
  unique: false
});

// Zaak lookup
db.informatieobject.ensureIndex({
  type: "persistent",
  fields: ["zaak_id"],
  unique: false,
  sparse: true
});

// Status (BSW)
db.informatieobject.ensureIndex({
  type: "persistent",
  fields: ["status"],
  unique: false
});

// Full-text search
db.informatieobject.ensureFullTextIndex(["naam", "omschrijving", "samenvatting"]);
```

---

#### informatieobject_catalogus (Catalog)

```javascript
{
  "_key": "cat-001",
  "informatieobject_id": "info-001",
  "locatie_uri": "cdd+://archive.nl/doc-001",
  "zoek_index": "besluit openbaarmaking documenten",
  "context_metadata": {
    "zaak_id": "zaak-001",
    "werkproces": "publicatie",
    "domein": "woo",
    "labels": ["openbaar", "woo"]
  },
  "geldig_vanaf": "2024-01-01T00:00:00Z",
  "geldig_tot": "9999-12-31T23:59:59Z"
}
```

---

#### informatieobject_recht (Object Rights)

```javascript
{
  "_key": "recht-001",
  "informatieobject_id": "info-001",
  "user_id": "user-789",
  "recht_type": "lezen",
  "granted_by": "user-456",
  "granted_at": "2024-04-19T10:00:00Z",
  "geldig_vanaf": "2024-04-19T10:00:00Z",
  "geldig_tot": null
}
```

**Indexes:**
```javascript
// User access lookup
db.informatieobject_recht.ensureIndex({
  type: "persistent",
  fields: ["user_id", "recht_type"],
  unique: false
});

// Object rights lookup
db.informatieobject_recht.ensureIndex({
  type: "persistent",
  fields: ["informatieobject_id"],
  unique: false
});
```

---

#### zaak (Cases - BSW)

```javascript
{
  "_key": "zaak-001",
  "zaaknummer": "2024-001",
  "zaaktype": "uitkering",
  "startdatum": "2024-01-01",
  "einddatum": null,
  "status": "open",
  "organisatie_id": "org-123",
  "aangemaakt_door": "user-456",
  "aangemaakt_op": "2024-04-19T10:00:00Z"
}
```

---

#### audit (Audit Trail)

```javascript
{
  "_key": "audit-001",
  "actie": "create",
  "entity_type": "gebeurtenis",
  "entity_id": "evt-001",
  "organisatie_id": "org-123",
  "uitgevoerd_door": "user-456",
  "uitgevoerd_op": "2024-04-19T10:00:00Z",
  "reden": "Initial creation",
  "wijzigingen": {
    "naam": ["", "Burger aanvraag uitkering"],
    "gebeurtenistype": ["", "aanvraag"]
  },
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "ip_adres": "192.168.1.100",
  "user_agent": "Mozilla/5.0..."
}
```

**Indexes:**
```javascript
// Entity audit trail
db.audit.ensureIndex({
  type: "persistent",
  fields: ["entity_type", "entity_id", "uitgevoerd_op"],
  unique: false
});

// Organization audit
db.audit.ensureIndex({
  type: "persistent",
  fields: ["organisatie_id", "uitgevoerd_op"],
  unique: false
});

// TTL index (7 years retention)
db.audit.ensureIndex({
  type: "ttl",
  fields: ["uitgevoerd_op"],
  expiryDays: 2555  // 7 years
});
```

---

### 2.2 Edge Collections

#### gebeurtenis_gebeurtenis_gegevensproduct

```javascript
{
  "_from": "gebeurtenis/evt-001",
  "_to": "gegevensproduct/gdp-001",
  "relatie_type": "produceert",
  "geldig_vanaf": "2024-01-01T00:00:00Z",
  "geldig_tot": "9999-12-31T23:59:59Z"
}
```

#### inherits_from (Metadata Inheritance - BSW)

```javascript
{
  "_from": "informatieobject/info-001",
  "_to": "zaak/zaak-001",
  "inheritance_type": "overerven",
  "fields": ["beveiligingsniveau", "privacy_level"],
  "geldig_vanaf": "2024-01-01T00:00:00Z",
  "geldig_tot": "9999-12-31T23:59:59Z"
}
```

---

## 3. AQL Queries

### 3.1 Time-Valid Entity Lookup

```aql
FOR doc IN gebeurtenis
  FILTER doc.organisatie_id == @org_id
  FILTER doc.geldig_vanaf <= @now
  FILTER doc.geldig_tot >= @now
  SORT doc.aangemaakt_op DESC
  LIMIT @limit
  RETURN doc
```

**Bind Parameters:**
```javascript
{
  "org_id": "org-123",
  "now": "2024-04-19T10:00:00Z",
  "limit": 20
}
```

---

### 3.2 Graph Traversal with Time Validity

```aql
WITH gebeurtenis, gegevensproduct, elementaire_set, enkelvoudig_gegeven
FOR v, e, p IN 1..3 OUTBOUND @start_id
  GRAPH 'gghm_v2'
  FILTER e.geldig_vanaf <= @now
  FILTER e.geldig_tot >= @now
  FILTER v.geldig_vanaf <= @now
  FILTER v.geldig_tot >= @now
  RETURN {
    vertex: v,
    edge: e,
    path: p,
    depth: LENGTH(p.edges)
  }
```

---

### 3.3 Context-Aware Search

```aql
FOR doc IN informatieobject
  SEARCH doc.naam IN @query_tokens 
     OR doc.omschrijving IN @query_tokens
     OR doc.samenvatting IN @query_tokens
  FILTER doc.geldig_vanaf <= @now
  FILTER doc.geldig_tot >= @now
  
  // Context scoring
  LET context_score = 
    (@context_zaak_id != null AND doc.zaak_id == @context_zaak_id ? 2.0 : 0.0) +
    (@context_org_id != null AND doc.organisatie_id == @context_org_id ? 1.0 : 0.0) +
    (@context_domein != null AND @context_domein IN doc.context_metadata.labels ? 0.5 : 0.0)
  
  LET relevance = BM25(doc)
  LET final_score = relevance * (1.0 + context_score)
  
  SORT final_score DESC
  LIMIT @limit
  RETURN MERGE(doc, { final_score, context_score })
```

---

### 3.4 Object-Level Authorization Check

```aql
// Get entity with access check
WITH informatieobject, informatieobject_recht, zaak_recht
FOR entity IN INFORMATIONOBJECT
  FILTER entity._key == @entity_key
  
  // Check direct grants
  LET direct_grant = FIRST(
    FOR recht IN INFORMATIEOBJECT_RECHT
      FILTER recht._to == @entity_key
      FILTER recht.user_id == @user_id
      FILTER recht.recht_type == @required Recht
      FILTER recht.geldig_vanaf <= @now
      FILTER (recht.geldig_tot == null OR recht.geldig_tot >= @now)
      RETURN true
  )
  
  // Check inherited zaak rights
  LET inherited_grant = FIRST(
    FOR entity IN 1..1 OUTBOUND @entity_key inherits_from
      FOR recht IN ZAAK_RECHT
        FILTER recht._to == entity._id
        FILTERrecht.user_id == @user_id
        FILTER recht.recht_type == @required Recht
        RETURN true
  )
  
  // Check organization rights
  LET org_grant = FIRST(
    FOR recht IN ORGANISATIE_RECHT
      FILTER recht._to == entity.organisatie_id
      FILTER recht.user_id == @user_id
      FILTERrecht.recht_type == @required Recht
      RETURN true
  )
  
  FILTER direct_grant == true OR inherited_grant == true OR org_grant == true
  
  RETURN entity
```

---

## 4. Replication Strategy

### 4.1 ArangoSync Configuration

```javascript
// Master configuration (EU-Central)
{
  "endpoint": "tcp://arangodb.metadata-registry.svc:8529",
  "database": "metadata_registry",
  "username": "root",
  "authentication": "jwt",
  "maxTransactions": 1000
}

// Follower configuration (EU-West)
{
  "endpoint": "tcp://arangodb-dr.metadata-registry-dr.svc:8529",
  "database": "metadata_registry",
  "username": "root",
  "master": {
    "endpoint": "tcp://arangodb.metadata-registry.svc:8529",
    "authentication": "jwt",
    "token": "<sync-token>"
  },
  "syncFrequency": 60,
  "autoStart": true
}
```

### 4.2 Replication Lag Monitoring

```javascript
db._query(`
  FOR sync IN _sync
    SORT sync.lastTick DESC
    LIMIT 1
    RETURN {
      last_tick: sync.lastTick,
      processed: sync.processedTransactions,
      time: DATE_TIMESTAMP(sync.lastTick * 1000)
    }
`)
```

---

## 5. Backup Strategy

### 5.1 Backup Schedule

| Type | Frequency | Retention | Location |
|------|-----------|-----------|----------|
| Snapshot | Hourly | 30 days | Primary region S3 |
| Incremental | Daily | 90 days | Primary region S3 |
| Full backup | Weekly | 7 years | DR region object storage |

### 5.2 Backup Command

```bash
arangodump \
  --server.endpoint tcp://arangodb:8529 \
  --server.database metadata_registry \
  --server.username root \
  --output /backup/snapshot-$(date +%Y%m%d-%H%M%S) \
  --include-system-collections \
  --threads 4
```

### 5.3 Restore Command

```bash
arangorestore \
  --server.endpoint tcp://arangodb:8529 \
  --server.database metadata_registry_new \
  --server.username root \
  --input /backup/snapshot-20240419-100000 \
  --create-database \
  --threads 4
```

---

## 6. Performance Optimization

### 6.1 Query Optimization Tips

1. **Use indexes**: Ensure filters use indexed fields
2. **Limit result sets**: Always use `LIMIT` in queries
3. **Avoid deep traversals**: Limit graph depth to 3-5 hops
4. **Use subqueries**: For complex filtering conditions
5. **Batch operations**: Use `FOR ... IN` for bulk operations

### 6.2 Sharding Strategy

```
Shard Key: _key (hash)
Shards per Collection: 9 (3 DB servers × 3 shards)
Replication Factor: 2
```

---

## 7. Monitoring Queries

### 7.1 Collection Statistics

```javascript
db._query(`
  FOR c IN _collections
    FILTER c.name NOT STARTS WITH '_')
    RETURN {
      collection: c.name,
      documents: c.count(),
      indexes: c.indexes().length,
      size: c.figures().indexes.size
    }
`)
```

### 7.2 Slow Query Log

```javascript
db._query(`
  FOR q IN _queries
    FILTER q.runtime >= 1000000  // 1 second
    SORT q.runtime DESC
    LIMIT 10
    RETURN {
      query: q.query_string,
      runtime: q.runtime,
      started: q.started
    }
`)
```

---

## 8. Related Documents

- ARC-002-DLD-v1.0: Detailed Design
- ARC-002-API-v1.0: API Design
- ARC-002-ADR-002: ArangoDB ADR
