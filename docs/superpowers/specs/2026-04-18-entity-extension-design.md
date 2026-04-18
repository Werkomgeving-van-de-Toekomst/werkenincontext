# Entity Extension Design - Metadata Registry V2

> **Project**: IOU-Modern Metadata Registry
> **Date**: 2026-04-18
> **Author**: Brainstorming Session
> **Status**: DRAFT

## Executive Summary

Deze uitbreiding voegt nieuwe kern entiteiten toe aan de Metadata Registry Service:

- **Gebeurtenis (Event)** - Triggers voor gegevenswijzigingen
- **Gegevensproduct (DataProduct)** - Samengestelde gegevenssets
- **ElementaireGegevensset** - Herbruikbare datablokken (Adres, Persoon, Zaak)
- **EnkelvoudigGegeven** - Enkelvoudige velden met tijdsdimensie
- **Waarde (Value)** - Tijdsgebonden waarden met mutatie-tracking
- **Context** - Rijke context bij gebeurtenissen
- **Grondslag** - Wettelijke grondslagen voor verwerking

Daarnaast krijgen alle bestaande entities nieuwe velden voor geldigheid en status.

---

## Table of Contents

1. [Architecture](#1-architecture)
2. [Data Model](#2-data-model)
3. [Validation System](#3-validation-system)
4. [API Design](#4-api-design)
5. [Migration Strategy](#5-migration-strategy)
6. [Implementation Structure](#6-implementation-structure)

---

## 1. Architecture

### 1.1 System Overview

```
     Kern Entiteiten (Nieuw)
     ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
     │  Gebeurtenis    │────▶│   Gegevensproduct │────▶│ElementaireGegevens│
     │  (Event)        │     │  (DataProduct)   │     │     set          │
     └─────────────────┘     └──────────────────┘     └─────────────────┘
           │                       │                       │
           │                       ▼                       ▼
           │              ┌──────────────────┐     ┌─────────────────┐
           │              │ EnkelvoudigGegeven│────▶│   Waarde       │
           │              │ (SimpleDataField) │     │ (Value)        │
           │              └──────────────────┘     └─────────────────┘
           │                       │
           ▼                       ▼
     ┌─────────────────┐     ┌──────────────────┐
     │     Context     │     │    Grondslag     │
     │  (EventContext) │     │ (LegalBasis)     │
     └─────────────────┘     └──────────────────┘
```

### 1.2 Status Model (Nieuw)

Alle entities krijgen een unified status model:

```
        ┌───────────┐
     ┌──│   DRAFT   │──┐
     │  └───────────┘  │
     │                 ▼
     │            ┌───────────┐     corrigeert
     │         ┌──│  ACTIVE   │◀─────────────────┐
     │         │  └───────────┘                   │
     │         │       │                          │
     │         │       ▼                          │
     │         │  ┌───────────┐                   │
     │         └──│  MUTATED  │──────┐            │
     │            └───────────┘      │            │
     │                 │             │            │
     │                 ▼             ▼            │
     │            ┌─────────────────────┐         │
     └───────────▶│   CORRECTED         │─────────┘
                  │  (corrigeert: ref)  │
                  └─────────────────────┘
                          │
                          ▼
                    ┌───────────┐
                    │ DEPRECATED│
                    └───────────┘
```

---

## 2. Data Model

### 2.1 Nieuwe Core Entities

#### Gebeurtenis (Event)

```rust
/// Gebeurtenis die leidt tot verandering in gegevens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gebeurtenis {
    pub _key: String,
    pub naam: String,                    // bijv. "Verhuizing", "Verkeersovertreding"
    pub gebeurtenis_type: GebeurtenisType,
    pub tijdstip: DateTime<Utc>,
    pub bron: GebeurtenisBron,           // Melding burger, Flitspaal, Agent
    pub context_id: Option<String>,      // Reference naar Context
    pub betrokken_producten: Vec<String>, // IDs van betrokken dataproducten
    pub eigenaar: String,
    pub status: EntityStatus,
}

pub enum GebeurtenisType {
    ExterneTrigger,  // Burger melding
    InterneTrigger,  // Systeem initiëerd
    Correctie,       // Foutieve herstel
}

pub enum GebeurtenisBron {
    BurgerMelding,
    Flitspaal,
    Handhaver,
    Systeem,
    Import,
}
```

#### Gegevensproduct (DataProduct)

```rust
/// Gegevensproduct: samengesteld uit elementaire gegevenssets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gegevensproduct {
    pub _key: String,
    pub naam: String,
    pub beschrijving: String,
    pub doelbinding: Vec<String>,        // References naar Doelbinding
    pub elementaire_sets: Vec<String>,   // References naar ElementaireGegevensset
    pub eigenaar: String,
    pub geldig_vanaf: DateTime<Utc>,
    pub geldig_tot: Option<DateTime<Utc>>,
    pub status: EntityStatus,
}
```

#### ElementaireGegevensset

```rust
/// Elementaire gegevensset (bijv. Adres, Persoon, Zaak)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementaireGegevensset {
    pub _key: String,
    pub naam: String,
    pub type_begrip: String,             // Reference naar TypeBegrip
    pub uniek_kenmerk: String,           // BSN, SKN, V-nummer, etc.
    pub domein: GegevensDomein,          // personendomein, zakendomein
    pub eigenaar: String,
    pub geldig_vanaf: DateTime<Utc>,
    pub geldig_tot: Option<DateTime<Utc>>,
    pub status: EntityStatus,
}

pub enum GegevensDomein {
    PersonenDomein,
    Zakendomein,
    Zaakdomein,
    Objectdomein,
}
```

#### EnkelvoudigGegeven

```rust
/// Enkelvoudig gegeven met tijdsdimensie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnkelvoudigGegeven {
    pub _key: String,
    pub naam: String,                    // bijv. "straatnaam", "huisnummer"
    pub elementaire_set_id: String,      // Parent
    pub data_type: DataType,
    pub beschrijving: String,

    // Tijdsdimensie - CORE FEATURE
    pub waardes: Vec<String>,            // References naar WaardeMetTijd
}
```

#### WaardeMetTijd

```rust
/// Waarde met tijdsdimensie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaardeMetTijd {
    pub _key: String,
    pub gegeven_id: String,              // Parent
    pub waarde: serde_json::Value,
    pub geldig_vanaf: DateTime<Utc>,
    pub geldig_tot: Option<DateTime<Utc>>,

    // Mutatie vs Correctie
    pub mutatie_type: MutatieType,
    pub gebeurtenis_id: Option<String>, // Waardoor veranderd?
    pub corrigeert: Option<String>,     // ID van vorige (foute) waarde
}

pub enum MutatieType {
    Mutatie,       // Normale wijziging
    Correctie,     // Foute waarde hersteld
}
```

### 2.2 Context & Grondslag

#### Context

```rust
/// Context bij een gebeurtenis of gegeven
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub _key: String,
    pub context_type: ContextType,
    pub gegevens: serde_json::Value,     // Rijke context data
    pub geldig_vanaf: DateTime<Utc>,
    pub geldig_tot: Option<DateTime<Utc>>,
}

pub enum ContextType {
    ProcesContext,     // Welk werkproces?
    ArchiefContext,    // Archief metadata
    WooContext,        // Openbaarheidsstatus
}
```

#### Grondslag

```rust
/// Wettelijke grondslag voor gegevensverwerking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grondslag {
    pub _key: String,
    pub wetsartikel: String,              // bijv. "AVG Art. 6"
    pub omschrijving: String,
    pub geldig_vanaf: DateTime<Utc>,
    pub geldig_tot: Option<DateTime<Utc>>,
    pub status: EntityStatus,
}
```

### 2.3 Uitbreiding Bestaande Entities

#### MetadataSchema (uitgebreid)

```rust
pub struct MetadataSchema {
    // ... bestaande velden ...

    // NIEUW
    pub geldig_vanaf: DateTime<Utc>,
    pub geldig_tot: Option<DateTime<Utc>>,
    pub eigenaar: String,
    pub status: SchemaStatus,  // → Wordt EntityStatus
}
```

#### AttributeDefinition (uitgebreid)

```rust
pub struct AttributeDefinition {
    // ... bestaande velden ...

    // NIEUW
    pub doelbinding: Option<Vec<String>>,  // Voor welke doelen?
}
```

### 2.4 Nieuwe EntityStatus Enum

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EntityStatus {
    Draft,       // In ontwikkeling, nog niet actief
    Active,      // Geldig en in gebruik
    Mutated,     // Waarde is bijgewerkt
    Corrected,   // Vorige waarde was fout
    Deprecated,  // Niet meer in gebruik
}

impl EntityStatus {
    /// Kan deze status overgaan naar target?
    pub fn can_transition_to(&self, target: &EntityStatus) -> bool {
        matches!(
            (self, target),
            (Self::Draft, Self::Active)
                | (Self::Active, Self::Mutated)
                | (Self::Mutated, Self::Active)
                | (_, Self::Corrected)
                | (Self::Active | Self::Mutated, Self::Deprecated)
        )
    }
}
```

---

## 3. Validation System

### 3.1 Status Transition Rules

| Van | Naar | Voorwaarde |
|-----|------|------------|
| DRAFT | ACTIVE | `eigenaar` set, alle verplichte velden ingevuld |
| ACTIVE | MUTATED | Waarde gewijzigd door Gebeurtenis |
| MUTATED | ACTIVE | Gebeurtenis afgerond, geen open correcties |
| ANY | CORRECTED | `corrigeert` referentie naar vorige versie |
| ACTIVE/MUTATED | DEPRECATED | `geldig_tot` < nu |

### 3.2 Tijdsdimensie Validatie

```rust
pub struct TijdsdimensieValidator;

impl TijdsdimensieValidator {
    /// Check dat periodes niet overlappen binnen zelfde gegeven
    pub async fn check_overlap(
        &self,
        gegeven_id: &str,
        nieuwe_waarde: &WaardeMetTijd,
    ) -> Result<()> {
        let existing = self.get_waardes(gegeven_id).await?;

        for bestaand in existing {
            if self.overlapt(&bestaand, nieuwe_waarde)? {
                return Err(MetadataError::ConstraintViolation {
                    field: "geldigheid".to_string(),
                    reason: format!("Overlapt met bestaande waarde: {}", bestaand._key),
                });
            }
        }

        Ok(())
    }

    fn overlapt(&self, a: &WaardeMetTijd, b: &WaardeMetTijd) -> Result<bool> {
        let a_end = a.geldig_tot.unwrap_or_else(|| DateTime::<Utc>::MAX_UTC());
        let b_end = b.geldig_tot.unwrap_or_else(|| DateTime::<Utc>::MAX_UTC());

        Ok(a.geldig_vanaf < b_end && b.geldig_vanaf < a_end)
    }
}
```

---

## 4. API Design

### 4.1 Nieuwe REST Endpoints

| Methode | Endpoint | Beschrijving |
|---------|----------|--------------|
| **Gebeurtenis** | | |
| GET | `/api/v2/gebeurtenis` | Alle gebeurtenissen (met filter) |
| POST | `/api/v2/gebeurtenis` | Nieuwe gebeurtenis aanmaken |
| GET | `/api/v2/gebeurtenis/:id` | Specifieke gebeurtenis |
| PUT | `/api/v2/gebeurtenis/:id` | Update gebeurtenis |
| **Gegevensproduct** | | |
| GET | `/api/v2/gegevensproducten` | Alle gegevensproducten |
| POST | `/api/v2/gegevensproducten` | Nieuw gegevensproduct |
| GET | `/api/v2/gegevensproducten/:id` | Specifiek product met sets |
| GET | `/api/v2/gekevensproducten/:id/waardes` | Actuele waardes op tijdstip |
| **Elementaire Sets** | | |
| GET | `/api/v2/elementaire-sets` | Alle sets |
| POST | `/api/v2/elementaire-sets` | Nieuwe set |
| GET | `/api/v2/elementaire-sets/:id/fields` | Enkelvoudige gegevens |
| **Waarden (tijdsdimensie)** | | |
| GET | `/api/v2/waardes/:field_id` | Waardes voor veld |
| POST | `/api/v2/waardes` | Nieuwe waarde (mutatie) |
| POST | `/api/v2/waardes/:id/corrigeer`** | | |
| GET | `/api/v2/grondslagen` | Alle wettelijke grondslagen |
| **Validatie** | | |
| POST | `/api/v2/validate/status` | Valideer status transitie |
| POST | `/api/v2/validate/tijdsdimensie` | Check overlappende periodes |

### 4.2 GraphQL Query Examples

```graphql
# Gebeurtenis met context en betrokken producten
query GetGebeurtenis($id: ID!) {
  gebeurtenis(id: $id) {
    naam
    tijdstip
    bron
    context {
      ... on ProcesContext {
        werkproces
        stap
      }
    }
    betrokkenProducten {
      naam
      elementaireSets {
        naam
        enkelvoudigeGegevens {
          naam
          actueleWaarde {
            waarde
            geldigVanaf
          }
        }
      }
    }
  }
}

# Gegevensproduct met volledige samenstelling
query GetGegevensproduct($id: ID!, $op: DateTime!) {
  gegevensproduct(id: $id) {
    naam
    doelbinding
    geldigVanaf
    geldigTot
    status
    waardeOpTijdstip(op: $op) {
      setNaam
      velden {
        naam
        waarde
        mutatieType
      }
    }
  }
}

# Correctie chain
mutation CorrigeerWaarde($id: ID!, $nieuweWaarde: Value!) {
  corrigeerWaarde(id: $id, waarde: $nieuweWaarde) {
    waarde {
      waarde
      geldigVanaf
      mutatieType
      corrigeert
    }
    vorigeFouteWaarde {
      waarde
      deprecated
    }
  }
}
```

### 4.3 Webhook Events (Nieuw)

```rust
pub enum MetadataEventV2 {
    // Nieuwe events
    GebeurtenisGeregistreerd(String),
    GebeurtenisVerwerkt(String),

    GegevensproductGewijzigd(String, Vec<String>), // product_id, gewijzigde_sets
    WaardeGemuteerd(String, String),               // field_id, nieuwe_waarde_id
    WaardeGecorrigeerd(String, String),            // field_id, correctie_id

    // Status transities
    StatusTransitie {
        entity_id: String,
        van: EntityStatus,
        naar: EntityStatus,
    },

    // Geldigheid
    GeldigheidVerlopen(String), // entity_id
}
```

---

## 5. Migration Strategy

### 5.1 Three-Phase Migration

**FASE 1: Schema Uitbreiding (Non-breaking)**

Voeg nieuwe velden toe aan bestaande collecties met defaults:
- `geldig_vanaf` → aangemaakt_op
- `geldig_tot` → null
- `eigenaar` → "system"
- `status` → "ACTIVE"

**FASE 2: Nieuwe Collecties Aanmaken**

Maak nieuwe collecties aan:
- `gebeurtenis`
- `gegevensproduct`
- `elementaire_gegevensset`
- `enkelvoudig_gegeven`
- `waarde`
- `context`
- `grondslag`

**FASE 3: Data Migratie (Achtergrond)**

Migreer bestaande data:
- MetadataSchema → Gegevensproduct
- ValueList → ElementaireGegevensset
- AttributeDefinition → EnkelvoudigGegeven

### 5.2 Deployment Volgorde

```
1. Deploy nieuwe code (met feature flag: v2_entities = false)
2. Run migration 004 (schema uitbreiding)
3. Run migration 005 (nieuwe collecties)
4. Enable feature flag (v2_entities = true)
5. Start background migrator (Fase 3)
6. Monitor: check alle data gemigreerd
7. Deprecate oude API endpoints
```

---

## 6. Implementation Structure

### 6.1 Crate Structure (Uitgebreid)

```
metadata-registry/
├── crates/
│   ├── metadata-core/                  # Gedeelde types (WASM-compatible)
│   │   ├── src/
│   │   │   ├── models.rs               # Uitbreiden met nieuwe entities
│   │   │   ├── status.rs               # Nieuw: EntityStatus, transitions
│   │   │   ├── tijdsdimensie.rs        # Nieuw: tijd validatie
│   │   │   └── graph.rs                # Uitbreiden: edge types
│   │
│   ├── metadata-store/                 # ArangoDB repository layer
│   │   ├── src/
│   │   │   ├── gebeurtenis_repo.rs     # Nieuw
│   │   │   ├── gegevensproduct_repo.rs # Nieuw
│   │   │   ├── elementaire_set_repo.rs # Nieuw
│   │   │   ├── waarde_repo.rs          # Nieuw
│   │   │   ├── context_repo.rs         # Nieuw
│   │   │   └── grondslag_repo.rs       # Nieuw
│   │
│   ├── metadata-validation/            # Validation engine
│   │   ├── src/
│   │   │   ├── status_validator.rs     # Nieuw: status transitions
│   │   │   └── tijd_validator.rs       # Nieuw: tijd overlappen
│   │
│   ├── metadata-api/                   # REST/GraphQL API
│   │   ├── src/
│   │   │   ├── routes_v1.rs            # Bestaand
│   │   │   ├── routes_v2.rs            # Nieuw: nieuwe endpoints
│   │   │   └── graphql_v2.rs           # Nieuw: nieuwe queries/mutations
│   │
│   ├── metadata-admin/                 # Admin UI (Dioxus)
│   │   ├── src/
│   │   │   ├── components/
│   │   │   │   ├── gebeurtenis/        # Nieuw
│   │   │   │   ├── gegevensproduct/    # Nieuw
│   │   │   │   └── tijdslijn/          # Nieuw: tijdlijn view
│   │   │   └── routes.rs               # Uitbreiden
│   │
│   └── metadata-migration/             # Nieuwe crate voor migratie
│       ├── src/
│       │   ├── migrator.rs
│       │   └── rollback.rs
│       └── Cargo.toml
│
└── migrations/
    ├── 004_add_core_fields.js          # ArangoDB migration
    ├── 005_create_new_collections.js
    └── 006_migrate_data.js
```

### 6.2 Edge Relations (Uitgebreid)

| Edge | From | To | Beschrijving |
|------|------|-----|--------------|
| `TRIGGERS` | Gebeurtenis | WaardeMetTijd | Gebeurtenis triggerde mutatie |
| `CORRECTEERT` | WaardeMetTijd | WaardeMetTijd | Correctie verwijst naar foute waarde |
| `HEEFT_CONTEXT` | Gebeurtenis | Context | Context bij gebeurtenis |
| `HEEFT_GRONDSLAG` | Gegevensproduct | Grondslag | Wettelijke basis |
| `BESTAAT UIT` | Gegevensproduct | ElementaireGegevensset | Samenstelling |
| `BEVAT` | ElementaireGegevensset | EnkelvoudigGegeven | Set bevat velden |

---

## Appendix A: Status Transition Matrix

| Huidige | volgende | DRAFT | ACTIVE | MUTATED | CORRECTED | DEPRECATED |
|---------|----------|-------|--------|---------|-----------|------------|
| DRAFT | - | ✓ | ✗ | ✗ | ✗ |
| ACTIVE | - | - | ✓ | ✓ | ✓ |
| MUTATED | - | ✓ | - | ✓ | ✓ |
| CORRECTED | - | - | ✓ | - | ✓ |
| DEPRECATED | - | - | - | - | - |

✓ = Geldige transitie
- = Niet van toepassing

---

**END OF DESIGN DOCUMENT**
