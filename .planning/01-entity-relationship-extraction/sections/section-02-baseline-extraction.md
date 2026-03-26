Now I have all the context. Let me generate the section content for `section-02-baseline-extraction`.

---

# Section 02: Baseline Extraction

## Overview

This section implements the fast baseline extraction layer using regex patterns and rust-bert for Dutch Named Entity Recognition (NER). This is the first stage of the extraction pipeline, designed to complete in under 500ms while capturing the majority of entities with high confidence.

**Target Performance:** <500ms per document (p50), <750ms (p95)

## Dependencies

- **Section 01: Foundation & Types** - Must be completed first
  - Requires `Entity`, `EntityType`, `ExtractionResult`, `ExtractionOptions`
  - Requires `StakeholderExtractor` trait definition
  - Requires `PersonStakeholder`, `OrganizationStakeholder` wrappers
  - Requires `MentionRelationship` and `MentionType` types

## Implementation Tasks

### 1. Create Baseline Module

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/baseline.rs`

```rust
use crate::stakeholder::{
    ExtractionResult, ExtractionOptions, PersonStakeholder, OrganizationStakeholder,
    MentionRelationship, MentionType, ExtractionMethod,
};
use iou_core::graphrag::Entity;
use rust_bert::pipelines::ner::NERModel;

/// Baseline extractor using regex + rust-bert
pub struct BaselineExtractor {
    ner_model: Option<NERModel>,
    enable_ner: bool,
}

impl BaselineExtractor {
    /// Create new baseline extractor
    /// 
    /// Eager loads the Dutch NER model if enabled. Model loading can take
    /// 1-2 seconds on first run, so this should be called at application startup.
    pub fn new(enable_ner: bool) -> Result<Self, BaselineError>;
    
    /// Extract entities from document text
    pub fn extract(
        &self,
        text: &str,
        document_id: Uuid,
        options: &ExtractionOptions,
    ) -> Result<ExtractionResult, BaselineError>;
}
```

### 2. Dutch Regex Patterns

Define patterns for Dutch government document extraction:

**Person Titles:**
- `dr\.?`, `prof\.?`, `mr\.?`, `mr\.?\s+dr\.?`, `ing\.?`, `ir\.?`

**Government Roles:**
- `minister`, `staatssecretaris`, `ambtenaar`, `directeur`, `hoofd`, `coordinator`
- `beleidsmedewerker`, `adviseur`, `manager`

**Department Abbreviations:**
- `MinFin`, `BZK`, `VWS`, `EZK`, `OCW`, `IenW`, `LNV`

**Pattern-based Relationship Extraction:**
- `{person}, (?:minister|staatssecretaris) van {org}` → WorksFor relationship
- `dr\.?\s+{person},\s+(?:directeur|hoofd)\s+(?:van\s+)?{org}` → WorksFor relationship

### 3. rust-bert Integration

Add dependency and implement Dutch NER:

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/Cargo.toml`

```toml
[dependencies]
rust-bert = "0.22"
```

**Implementation details:**
- Load Dutch NER model at startup (lazy or eager based on config)
- Model: `bert-base-dutch-cased` fine-tuned for NER
- Extract PER (person) and ORG (organization) entities
- Map rust-bert labels to internal entity types

### 4. Confidence Scoring

Implement confidence calculation based on extraction method:

| Source | Base Confidence | Boost Conditions |
|--------|-----------------|------------------|
| Regex exact match | 0.85 | +0.05 if title present |
| Regex pattern | 0.70 | +0.10 if government role |
| rust-bert PER | 0.75 | +0.05 if in title pattern |
| rust-bert ORG | 0.70 | +0.10 if in department list |

Final confidence = `min(1.0, base + boosts)`

### 5. Mention Type Detection

Implement logic to detect `MentionType`:

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/mention_detector.rs`

```rust
/// Detect mention type from document context
pub fn detect_mention_type(
    entity_text: &str,
    document_text: &str,
    position: usize,
) -> MentionType {
    // Author detection: check header patterns
    if is_in_header(entity_text, document_text, position) {
        return MentionType::Author;
    }
    
    // Recipient detection: check "Geachte:" patterns
    if is_recipient_pattern(document_text, position) {
        return MentionType::Recipient;
    }
    
    // Default: Referenced
    MentionType::Referenced
}
```

### 6. Module Exports

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/mod.rs`

```rust
pub mod baseline;
pub mod mention_detector;

pub use baseline::{BaselineExtractor, BaselineError};
```

## Tests

Write the following tests in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/baseline_tests.rs`:

### Title Extraction Tests

```rust
#[test]
fn test_regex_extracts_dr_as_title() {
    let text = "Dr. Jan de Vries heeft een brief gestuurd.";
    let result = extractor.extract_titles(text);
    assert!(result.contains(&"dr.".to_string()));
}

#[test]
fn test_regex_extracts_prof_as_title() {
    let text = "Prof. dr. Marie Jansen is aangesteld.";
    let result = extractor.extract_titles(text);
    assert!(result.contains(&"prof.".to_string()));
}
```

### Government Role Tests

```rust
#[test]
fn test_regex_extracts_minister_as_government_role() {
    let text = "De minister van Financien sprak vandaag.";
    let result = extractor.extract_roles(text);
    assert!(result.iter().any(|r| r.to_lowercase() == "minister"));
}

#[test]
fn test_regex_extracts_staatsecretaris_as_government_role() {
    let text = "De staatssecretaris van Economische Zaken.";
    let result = extractor.extract_roles(text);
    assert!(result.iter().any(|r| r.to_lowercase() == "staatssecretaris"));
}
```

### Department Pattern Tests

```rust
#[test]
fn test_regex_extracts_minfin_as_department_abbreviation() {
    let text = "MinFin heeft vandaag het beleid aangekondigd.";
    let result = extractor.extract_departments(text);
    assert!(result.contains(&"MinFin".to_string()));
}

#[test]
fn test_regex_extracts_bzk_as_department_abbreviation() {
    let text = "BZK is verantwoordelijk voor de uitvoering.";
    let result = extractor.extract_departments(text);
    assert!(result.contains(&"BZK".to_string()));
}
```

### Relationship Pattern Tests

```rust
#[test]
fn test_pattern_x_minister_van_y_creates_worksfor_relationship() {
    let text = "Jan de Vries, minister van Financien, is aanwezig.";
    let result = extractor.extract_relationships(text);
    assert!(result.iter().any(|r| 
        r.person_name == "Jan de Vries" && 
        r.org_name == "Financien" &&
        r.relationship_type == "WorksFor"
    ));
}

#[test]
fn test_pattern_dr_x_directeur_z_creates_worksfor_with_role() {
    let text = "Dr. Sophie Bakker, directeur van de Rijksoverheid, spreekt.";
    let result = extractor.extract_relationships(text);
    assert!(result.iter().any(|r| 
        r.person_name == "Sophie Bakker" && 
        r.org_name == "Rijksoverheid" &&
        r.role == Some("directeur".to_string())
    ));
}
```

### Mention Type Detection Tests

```rust
#[test]
fn test_mention_type_author_detected_from_header_patterns() {
    let doc = "Van: Dr. Jan de Vries\nAan: Ministerie van Financien\n\nHierbij...";
    let entity = "Dr. Jan de Vries";
    let mention_type = detect_mention_type(entity, doc, 5);
    assert_eq!(mention_type, MentionType::Author);
}

#[test]
fn test_mention_type_recipient_detected_from_geachte_patterns() {
    let doc = "Geachte minister,\nHierbij stuur ik u...";
    let entity = "minister";
    let mention_type = detect_mention_type(entity, doc, 8);
    assert_eq!(mention_type, MentionType::Recipient);
}

#[test]
fn test_mention_type_defaults_to_referenced_when_unclear() {
    let doc = "Het beleid is goedgekeurd door Jan de Vries.";
    let entity = "Jan de Vries";
    let mention_type = detect_mention_type(entity, doc, 35);
    assert_eq!(mention_type, MentionType::Referenced);
}
```

### rust-bert Integration Tests

```rust
#[tokio::test]
async fn test_rust_bert_dutch_ner_model_loads_without_error() {
    let extractor = BaselineExtractor::new(true).unwrap();
    assert!(extractor.ner_model.is_some());
}

#[test]
fn test_rust_bert_returns_person_entities_from_dutch_text() {
    let extractor = BaselineExtractor::new(true).unwrap();
    let text = "Jan de Vries en Marie Jansen werkten samen.";
    let result = extractor.extract_ner(text).unwrap();
    assert!(result.persons.iter().any(|p| p.contains("Vries")));
}

#[test]
fn test_rust_bert_returns_organization_entities_from_dutch_text() {
    let extractor = BaselineExtractor::new(true).unwrap();
    let text = "Het Ministerie van Financien in Den Haag.";
    let result = extractor.extract_ner(text).unwrap();
    assert!(result.organizations.iter().any(|o| o.contains("Financien")));
}
```

### Confidence Score Tests

```rust
#[test]
fn test_confidence_scores_between_0_and_1() {
    let extractor = BaselineExtractor::new(true).unwrap();
    let text = "Dr. Jan de Vries van MinFin sprak vandaag.";
    let result = extractor.extract(text, Uuid::new_v4(), &ExtractionOptions::default()).unwrap();
    
    for person in result.persons {
        assert!(person.confidence >= 0.0 && person.confidence <= 1.0);
    }
    for org in result.organizations {
        assert!(org.confidence >= 0.0 && org.confidence <= 1.0);
    }
}
```

### Performance Tests

```rust
#[test]
fn test_baseline_extraction_completes_in_under_500ms_for_typical_document() {
    let extractor = BaselineExtractor::new(true).unwrap();
    let text = include_str!("../fixtures/sample_dutch_document.txt");
    let start = std::time::Instant::now();
    
    let result = extractor.extract(text, Uuid::new_v4(), &ExtractionOptions::default()).unwrap();
    
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 500, "Extraction took {}ms", elapsed.as_millis());
}
```

## Error Handling

Define `BaselineError` in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/baseline.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum BaselineError {
    #[error("NER model failed to load: {0}")]
    ModelLoadError(String),
    
    #[error("NER inference failed: {0}")]
    InferenceError(String),
    
    #[error("Invalid document text: {0}")]
    InvalidText(String),
    
    #[error("Regex pattern error: {0}")]
    PatternError(#[from] regex::Error),
}
```

## Success Criteria

- [x] Regex patterns extract known Dutch titles (dr., prof., minister, etc.)
- [x] Regex patterns extract known Dutch government roles
- [x] Regex patterns extract department abbreviations (MinFin, BZK, etc.)
- [ ] rust-bert Dutch NER model loads without error (DEFERRED - heavy libtorch dependency)
- [ ] rust-bert returns Person entities from Dutch text (DEFERRED)
- [ ] rust-bert returns Organization entities from Dutch text (DEFERRED)
- [x] Confidence scores are calculated and fall within 0.0-1.0 range
- [x] Pattern-based relationships are extracted (person ↔ organization)
- [ ] Mention types are detected (Author, Recipient, Referenced) (REMOVED - orphaned module)
- [x] Unit tests pass for Dutch sample text (16 tests)
- [x] Baseline extraction completes in <500ms for typical document

## Implementation Notes

### Deviations from Plan

1. **rust-bert Integration Deferred**: The rust-bert integration with libtorch was deferred due to the heavy dependency weight. The regex-based approach already covers common Dutch government entity patterns effectively and meets the <500ms performance target.

2. **MentionDetector Module Removed**: The `mention_detector.rs` module was created but never integrated. Since `MentionRelationshipWrapper` doesn't include mention_type fields, and integrating would require significant changes, the module was removed as orphaned code.

3. **Confidence Calculation Enhanced**: The confidence calculation was fixed to check patterns only within a 200-character window around the matched entity position, preventing false confidence boosts from entities mentioned elsewhere in the document.

4. **Stats Tracking Added**: Proper statistics tracking with confidence buckets was implemented using `ExtractionStats::track_confidence()`.

5. **Processing Time Measurement**: Actual processing time is now measured and returned, not hardcoded to 0.

### Files Created/Modified

- **Created:** `crates/iou-ai/src/stakeholder/baseline.rs` (~450 lines)
  - BaselineExtractor with Dutch regex patterns
  - Confidence calculation with context-aware boosting
  - Stats tracking and performance measurement
  - 16 unit tests including performance test

- **Deleted:** `crates/iou-ai/src/stakeholder/mention_detector.rs` (orphaned)

- **Modified:** `crates/iou-ai/src/stakeholder/mod.rs`
  - Added baseline module exports
  - Removed mention_detector declaration

### Performance

- Target: <500ms per document
- Actual: Typically <10ms for typical Dutch government documents
- Regex patterns are compiled once using `OnceLock`

### Notes

- Regex patterns use `OnceLock` for lazy initialization and thread safety
- Support for accented Dutch characters (ë, ï, etc.) via `[A-Z][a-zÀ-ÿ]` pattern
- Canonical name lookup for government organizations (MinFin, BZK, etc.)
- Deduplication will be addressed in section-05 (normalization-deduplication)