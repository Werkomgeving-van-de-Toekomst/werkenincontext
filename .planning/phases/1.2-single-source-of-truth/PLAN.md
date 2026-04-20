# Phase 1.2: Single Source of Truth Registry (ADR-013)

> **MSP Requirement**: IHH02 - Single Source of Truth
> **Duration**: 6 weeks
> **Status**: ✅ COMPLETED (2026-04-19)

## Overview

Implement a central data registry that serves as the single source of truth for all information objects across the organization, ensuring data consistency and preventing duplicate/conflicting records.

## Success Criteria

| Criterion | Description | Status |
|-----------|-------------|--------|
| REG-01 | Central registry provides discoverable entity catalog | ✅ Complete |
| REG-02 | All data sources registered with quality metrics | ✅ Complete |
| REG-03 | Generic API for source system reads | ✅ Complete |
| REG-04 | Data quality scoring and tracking | ✅ Complete |
| REG-05 | Registry search and filtering | ✅ Complete |

## Tasks

| ID | Task | Duration | Owner | Status |
|----|------|----------|-------|--------|
| 1.2.1 | Design central data registry schema | 1 week | Data Architect | ✅ Complete |
| 1.2.2 | Implement registry service API | 2 weeks | Backend Lead | ✅ Complete |
| 1.2.3 | Create source read API templates | 1 week | Integration Lead | ✅ Complete |
| 1.2.4 | Implement data quality metrics | 1 week | Data Engineer | ✅ Complete |
| 1.2.5 | Registry search functionality | 1 week | Frontend Lead | ✅ Complete |

## Implementation Summary

### Module Structure

Created comprehensive registry module in `crates/iou-core/src/registry/`:

```
registry/
├── mod.rs              # Module exports
├── entity.rs           # DataEntity, EntityType, EntityMatch (270 lines)
├── source.rs           # DataSource, SourceType, SourceConnection (375 lines)
├── quality.rs          # QualityScore, QualityMetric, QualityThreshold (290 lines)
├── api.rs              # Request/Response types for all endpoints (445 lines)
├── service.rs          # RegistryService with CRUD operations (525 lines)
├── source_reader.rs    # Read templates for all source types (650 lines)
├── quality_engine.rs   # Quality assessment with rules engine (680 lines)
└── search.rs           # Full-text search with filtering (720 lines)
```

### Deliverables

#### 1.2.1: Data Model (entity.rs, source.rs, quality.rs)
- 10 entity types: Citizen, Organization, Case, Document, Policy, Location, Building, Asset, Event, Other
- 8 source types: Database, Api, Soap, File, Legacy, MessageQueue, ObjectStorage, GraphQL
- Quality scoring with 4 dimensions: completeness, accuracy, consistency, timeliness
- Full test coverage for all types

#### 1.2.2: Registry Service API (api.rs, service.rs)

**API Endpoints:**
```
GET    /api/v1/registry/sources              - List all data sources
POST   /api/v1/registry/sources              - Register new data source
GET    /api/v1/registry/sources/{id}         - Get source details
PUT    /api/v1/registry/sources/{id}         - Update source
DELETE /api/v1/registry/sources/{id}         - Deactivate source

GET    /api/v1/registry/entities             - Search entities
GET    /api/v1/registry/entities/{id}        - Get entity by canonical ID
GET    /api/v1/registry/sources/{id}/entities - Get entities from source

GET    /api/v1/registry/quality              - Get quality metrics
POST   /api/v1/registry/quality/assess       - Assess entity quality

POST   /api/v1/registry/sync/{source_id}     - Trigger sync from source
GET    /api/v1/registry/sync/{source_id}/status - Get sync status
```

**Service Operations:**
- Source CRUD with validation and conflict detection
- Entity search with filtering by type, source, quality
- Entity upsert with deduplication via external ID index
- Quality metrics aggregation by source/entity type
- Sync job triggering with status tracking

#### 1.2.3: Source Read Templates (source_reader.rs)

**Template Types:**
- `DatabaseTemplate` - SQL queries, stored procedures, batch fetching
- `ApiTemplate` - REST API with pagination (page/offset/cursor/link-header)
- `SoapTemplate` - SOAP/WSDL with XPath extraction
- `FileTemplate` - CSV, JSON, XML, Excel parsing
- `LegacyTemplate` - Fixed-width, delimited, regex parsing
- `GraphQLTemplate` - GraphQL queries with variables

**Builder Pattern:**
```rust
DatabaseTemplateBuilder::new()
    .query("SELECT * FROM citizens")
    .batch_size(1000)
    .build()

ApiTemplateBuilder::new()
    .endpoint("/api/citizens")
    .pagination(PaginationConfig::page_based("page", "limit", 100))
    .build()
```

**Presets:** `TemplatePresets::postgresql_table()`, `rest_api_collection()`, `csv_file()`

#### 1.2.4: Quality Metrics Engine (quality_engine.rs)

**Quality Rules:**
- Required fields, pattern matching, allowed values, ranges
- Date/time, email, URL validation
- Cross-field validation, metadata completeness
- Custom JavaScript expressions

**Quality Assessment:**
- Per-dimension scoring with rule weights
- Issue tracking with severity (Critical, High, Medium, Low)
- Recommendations for improvement

**Aggregation:**
- Average, min, max, median scores
- Score distribution (excellent/good/acceptable/poor)
- Issue summary by severity/dimension

**Default Rule Sets:**
- `DefaultRuleSets::citizen()` - Name, email format, metadata completeness
- `DefaultRuleSets::organization()` - Name, website URL, KvK number

#### 1.2.5: Registry Search (search.rs)

**Search Capabilities:**
- Full-text search across name, external_id, metadata
- Filter by entity type, source, quality score, date ranges
- Metadata filters with dot notation (e.g., `address.city`)
- Sorting by any field with direction
- Pagination with page info

**Query Builder:**
```rust
SearchQueryBuilder::new()
    .text("John")
    .entity_type(EntityType::Citizen)
    .quality_min(0.8)
    .sort(SortField::Name, SortDirection::Asc)
    .paginate(0, 20)
    .include_total(true)
    .build()
```

**Faceted Search:**
- Entity type distribution
- Source distribution
- Quality rating distribution
- Date range buckets (last 24h, 7d, 30d, 90d, older)

**Autocomplete:**
- Typeahead suggestions for entity names
- Search suggestion types: EntityName, EntityType, FieldName, Operator, Value

## Data Model

### data_sources Table
```sql
CREATE TABLE data_sources (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    source_type VARCHAR(50), -- 'database', 'api', 'file', 'legacy'
    connection_string TEXT, -- encrypted
    owner_id UUID,
    organization_id UUID,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now()
);
```

### data_entities Table
```sql
CREATE TABLE data_entities (
    id UUID PRIMARY KEY,
    source_id UUID REFERENCES data_sources(id),
    entity_type VARCHAR(100), -- 'citizen', 'case', 'document', etc.
    external_id VARCHAR(255), -- source system ID
    canonical_id UUID, -- registry's unified ID
    name VARCHAR(500),
    metadata JSONB,
    quality_score DECIMAL(3,2),
    last_synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now()
);
```

## Code Statistics

| Module | Lines | Purpose |
|--------|-------|---------|
| entity.rs | 270 | Core entity types |
| source.rs | 375 | Data source types |
| quality.rs | 290 | Quality scoring types |
| api.rs | 445 | API request/response types |
| service.rs | 525 | Registry service |
| source_reader.rs | 650 | Read templates |
| quality_engine.rs | 680 | Quality rules engine |
| search.rs | 720 | Search functionality |
| **Total** | **3,955** | |

## Test Coverage

All modules include comprehensive unit tests:
- Type validation/serialization
- CRUD operations
- Quality assessment
- Search filtering and sorting
- Template building

## Next Steps

1. **Integration**: Connect registry to actual backend storage (PostgreSQL)
2. **API Routes**: Expose endpoints via Axum/Actix-web
3. **Frontend**: Build admin UI for registry management
4. **Monitoring**: Add metrics for sync operations and quality trends
5. **Documentation**: API documentation with OpenAPI/Swagger

---
*Phase completed: 2026-04-19*
