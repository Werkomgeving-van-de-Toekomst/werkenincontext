# Architecture Diagram: Sequence - Validation Engine Flow

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Command**: `/arckit:diagram sequence`

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-002-DIAG-007-v1.0 |
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
| **Distribution** | Project Team, Architecture Team, Data Governance |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---------|------|--------|---------|-------------|---------------|
| 1.0 | 2026-04-19 | ArcKit AI | Initial creation from `/arckit:diagram sequence` command | PENDING | PENDING |

---

## Diagram Purpose

This sequence diagram shows the validation engine flow for metadata entities, including schema validation, constraint checking, TOOI/MDTO compliance, PII detection, and validation error reporting.

---

## Validation Engine Flow

```mermaid
sequenceDiagram
    autonumber

    participant Client as API Client
    participant API as REST API
    participant Validator as Validation Engine
    participant Schema as Schema Validator
    participant Constraint as Constraint Checker
    participant TOOI as TOOI Validator
    participant MDTO as MDTO Validator
    participant PII as PII Detector
    participant Report as Report Generator
    participant DB as ArangoDB

    Note over Client,DB: Entity Creation Request

    Client->>API: POST /api/v2/entities
    activate API
    API->>API: Extract entity from request
    API->>Validator: validate(entity, entity_type)

    Note over Client,DB: Phase 1: Schema Validation

    activate Validator
    Validator->>Schema: validate_schema(entity, entity_type)
    activate Schema

    Schema->>Schema: Check required fields
    Schema->>Schema: Validate field types
    Schema->>Schema: Check enum values

    alt Missing required field
        Schema-->>Validator: Err(ValidationException::MissingField)
        Validator-->>API: Err(ValidationException::MissingField)
        API-->>Client: 400 Bad Request
        deactivate API
        Note right of Client: {"error": "Field 'naam' is required"}
    end

    alt Invalid field type
        Schema-->>Validator: Err(ValidationException::InvalidType)
        Validator-->>API: Err(ValidationException::InvalidType)
        API-->>Client: 400 Bad Request
        deactivate API
        Note right of Client: {"error": "Field 'geldig_vanaf' must be DateTime"}
    end

    Schema-->>Validator: Ok(schema_valid)
    deactivate Schema

    Note over Client,DB: Phase 2: Constraint Checking

    Validator->>Constraint: check_constraints(entity)
    activate Constraint

    Constraint->>Constraint: Validate time validity
    Note right of Constraint: geldig_tot > geldig_vanaf

    Constraint->>Constraint: Check organization exists
    Constraint->>DB: GET organisaties/{org_id}
    DB-->>Constraint: Organization or null

    Constraint->>Constraint: Validate reference integrity
    Constraint->>DB: Check referenced entities exist
    DB-->>Constraint: References valid or null

    Constraint->>Constraint: Check business rules

    alt Time validity invalid
        Constraint-->>Validator: Err(ValidationException::InvalidValidity)
        Validator-->>API: Err(ValidationException::InvalidValidity)
        API-->>Client: 400 Bad Request
        deactivate API
        Note right of Client: {"error": "geldig_tot must be after geldig_vanaf"}
    end

    alt Organization not found
        Constraint-->>Validator: Err(ValidationException::OrganizationNotFound)
        Validator-->>API: Err(ValidationException::OrganizationNotFound)
        API-->>Client: 404 Not Found
        deactivate API
        Note right of Client: {"error": "Organization not found"}
    end

    alt Reference not found
        Constraint-->>Validator: Err(ValidationException::ReferenceNotFound)
        Validator-->>API: Err(ValidationException::ReferenceNotFound)
        API-->>Client: 400 Bad Request
        deactivate API
        Note right of Client: {"error": "Referenced entity not found"}
    end

    Constraint-->>Validator: Ok(constraints_valid)
    deactivate Constraint

    Note over Client,DB: Phase 3: TOOI Compliance (Conditional)

    alt entity_type requires TOOI validation
        Validator->>TOOI: validate_tooi(entity)
        activate TOOI

        TOOI->>TOOI: Check TOOI URI format
        TOOI->>TOOI: Validate against TOOI schema
        TOOI->>TOOI: Check mandatory TOOI fields

        alt TOOI validation failed
            TOOI-->>Validator: Err(ValidationException::TOOICompliance)
            Note right of Validator: Non-blocking warning
        end

        TOOI-->>Validator: Ok(tooi_warnings)
        deactivate TOOI
    end

    Note over Client,DB: Phase 4: MDTO Compliance (Conditional)

    alt entity_type requires MDTO validation
        Validator->>MDTO: validate_mdto(entity)
        activate MDTO

        MDTO->>MDTO: Check MDTO URI format
        MDTO->>MDTO: Validate against MDTO schema
        MDTO->>MDTO: Check mandatory MDTO fields

        alt MDTO validation failed
            MDTO-->>Validator: Err(ValidationException::MDTOCompliance)
            Note right of Validator: Non-blocking warning
        end

        MDTO-->>Validator: Ok(mdto_warnings)
        deactivate MDTO
    end

    Note over Client,DB: Phase 5: PII Detection

    Validator->>PII: detect_pii(entity)
    activate PII

    PII->>PII: Scan text fields for patterns
    Note right of PII: BSN, email, phone,<br/>IBAN, passport

    PII->>PII: Calculate confidence score
    PII->>PII: Classify privacy level

    alt High-confidence PII detected
        PII-->>Validator: Ok(PIIResult {
            detected: true,
            fields: ["omschrijving"],
            confidence: 0.95,
        })
        Validator->>Validator: Check privacy_level matches
        Note right of Validator: If PII detected,<br/>privacy_level must be avg
    end

    PII-->>Validator: Ok(pii_result)
    deactivate PII

    Note over Client,DB: Phase 6: Report Generation

    Validator->>Report: generate_report(validation_results)
    activate Report

    Report->>Report: Compile errors
    Report->>Report: Compile warnings
    Report->>Report: Calculate score
    Report->>Report: Add recommendations

    Report-->>Validator: ValidationReport
    deactivate Report

    Validator-->>API: Ok(ValidationReport)
    deactivate Validator

    Note over Client,DB: Phase 7: Decision

    API->>API: Check validation_result.has_errors()

    alt Validation failed with errors
        API-->>Client: 400 Bad Request
        deactivate API
        Note right of Client: {
          "valid": false,
          "errors": [...],
          "warnings": [...]
        }
    end

    Note over Client,DB: Validation Success

    API->>DB: INSERT entity
    DB-->>API: Entity created
    API->>DB: INSERT audit_log
    Note right of DB: validation_report stored

    API-->>Client: 201 Created
    deactivate API
    deactivate API
    Note right of Client: {
      "valid": true,
      "entity_id": "...",
      "warnings": [...]
    }
```

---

## Validation Pipeline

### Validation Stages

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   Request   │ ──▶│ Schema Valid │ ──▶│  Constraints │
└─────────────┘    └──────────────┘    └──────┬──────┘
                                              │
                                              ▼
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   Response   │◀──│   Report     │◀───│   PII Check │
└─────────────┘    └──────────────┘    └──────┬──────┘
                                              │
                                    ┌─────────┴─────────┐
                                    ▼                   ▼
                             ┌──────────┐        ┌──────────┐
                             │   TOOI   │        │   MDTO   │
                             │(Optional)│        │(Optional)│
                             └──────────┘        └──────────┘
```

### Validation Categories

| Category | Blocking | Description |
|----------|----------|-------------|
| **Schema** | Yes | Required fields, data types |
| **Constraints** | Yes | Business rules, referential integrity |
| **TOOI** | No | TOOI standard compliance (warnings) |
| **MDTO** | No | MDTO standard compliance (warnings) |
| **PII** | No | Personal data detection (informational) |

---

## Validation Rules

### Schema Validation

```rust
pub fn validate_schema(
    entity: &Value,
    entity_type: EntityType,
) -> Result<SchemaValidation, ValidationError> {
    let schema = get_schema(entity_type)?;

    // Required fields
    for field in schema.required_fields() {
        if !entity.has(field) {
            return Err(ValidationError::MissingField(field));
        }
    }

    // Field types
    for (field, expected_type) in schema.field_types() {
        let value = entity.get(field)
            .ok_or(ValidationError::MissingField(field))?;

        if !check_type(value, expected_type) {
            return Err(ValidationError::InvalidType {
                field,
                expected: expected_type,
                actual: type_of(value),
            });
        }
    }

    // Enum values
    for (field, allowed_values) in schema.enum_fields() {
        let value = entity.get(field).unwrap();
        if !allowed_values.contains(value) {
            return Err(ValidationError::InvalidEnumValue {
                field,
                value: value.clone(),
                allowed: allowed_values.clone(),
            });
        }
    }

    Ok(SchemaValidation::Valid)
}
```

### Constraint Validation

```rust
pub fn check_constraints(
    entity: &Value,
    db: &Database,
) -> Result<ConstraintsValid, ValidationError> {
    // Time validity
    let geldig_vanaf = entity.get("geldig_vanaf").unwrap();
    let geldig_tot = entity.get("geldig_tot").unwrap();

    if geldig_tot <= geldig_vanaf {
        return Err(ValidationError::InvalidValidity);
    }

    // Organization exists
    let org_id = entity.get("organisatie_id").unwrap();
    if !db.organization_exists(org_id) {
        return Err(ValidationError::OrganizationNotFound);
    }

    // Reference integrity
    if let Some(references) = extract_references(entity) {
        for (edge_type, target_key) in references {
            if !db.entity_exists(&target_key) {
                return Err(ValidationError::ReferenceNotFound {
                    edge_type,
                    target_key,
                });
            }
        }
    }

    // Business rules
    validate_business_rules(entity)?;

    Ok(ConstraintsValid)
}
```

### PII Detection

```rust
pub fn detect_pii(entity: &Value) -> PIIResult {
    let mut detected_fields = Vec::new();
    let mut confidence = 0.0;

    // Scan text fields
    for (field, value) in entity.as_object().unwrap() {
        if is_text_field(field) {
            let text = value.as_str().unwrap();

            // BSN pattern
            if let Some(match_) = scan_bsn(text) {
                detected_fields.push((field, "BSN", match_.confidence));
                confidence = confidence.max(match_.confidence);
            }

            // Email pattern
            if let Some(match_) = scan_email(text) {
                detected_fields.push((field, "email", match_.confidence));
                confidence = confidence.max(match_.confidence);
            }

            // Phone pattern
            if let Some(match_) = scan_phone(text) {
                detected_fields.push((field, "phone", match_.confidence));
                confidence = confidence.max(match_.confidence);
            }

            // IBAN pattern
            if let Some(match_) = scan_iban(text) {
                detected_fields.push((field, "IBAN", match_.confidence));
                confidence = confidence.max(match_.confidence);
            }
        }
    }

    PIIResult {
        detected: !detected_fields.is_empty(),
        fields: detected_fields,
        confidence,
    }
}
```

---

## Validation Report

```rust
pub struct ValidationReport {
    pub valid: bool,
    pub score: f64,  // 0.0 - 1.0
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub recommendations: Vec<String>,
    pub pii_detected: bool,
    pub pii_fields: Vec<String>,
    pub pii_confidence: f64,
}

#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
    pub severity: ErrorSeverity,  // error, warning, info
}

#[derive(Debug, Serialize)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub category: WarningCategory,  // tooi, mdto, pii
    pub suggestion: Option<String>,
}
```

---

## Error Codes

| Code | Message | Severity | Field |
|------|---------|----------|-------|
| `MISSING_REQUIRED_FIELD` | Field is required | error | * |
| `INVALID_FIELD_TYPE` | Field has invalid type | error | * |
| `INVALID_ENUM_VALUE` | Value not in allowed set | error | * |
| `INVALID_TIME_VALIDITY` | geldig_tot must be after geldig_vanaf | error | * |
| `ORGANIZATION_NOT_FOUND` | Organization does not exist | error | organisatie_id |
| `REFERENCE_NOT_FOUND` | Referenced entity does not exist | error | * |
| `TOOI_COMPLIANCE_WARNING` | TOOI standard compliance issue | warning | * |
| `MDTO_COMPLIANCE_WARNING` | MDTO standard compliance issue | warning | * |
| `PII_DETECTED` | Personal data detected | info | * |
| `PRIVACY_LEVEL_MISMATCH` | PII detected but privacy_level != avg | warning | privacy_level |

---

## Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Schema validation | <10ms | Required fields, type checks |
| Constraint validation | <50ms | Includes DB lookups |
| PII detection | <100ms | Regex pattern matching |
| TOOI validation | <200ms | External schema check |
| MDTO validation | <200ms | External schema check |
| Total validation | <500ms | All phases combined |

---

## Related Documents

- **ARC-002-REQ-v1.1**: FR-MREG-4 (Validation)
- **ARC-002-DIAG-003**: Component Diagram (Validation Engine)
- **ARC-002-ADR-004**: BSW Alignment (validation requirements)
