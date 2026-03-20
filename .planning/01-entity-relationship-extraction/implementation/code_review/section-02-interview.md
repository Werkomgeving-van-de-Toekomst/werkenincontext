# Code Review Interview: Section 02 - Baseline Extraction

**Date:** 2026-03-16
**Section:** section-02-baseline-extraction
**Reviewer:** feature-dev:code-reviewer

---

## User Decisions

### 1. rust-bert Integration (CRITICAL)
**Decision:** Defer rust-bert integration

**Rationale:** The plan specifies rust-bert for Dutch NER, but it requires libtorch (heavy dependency). Deferring to a later section allows the baseline extraction to be lightweight and fast. The regex-based approach already covers common Dutch government entity patterns effectively.

**Action Taken:** Added comment to baseline.rs documenting the deferral. The `enable_ner` flag is retained for future use.

---

### 2. MentionDetector Module (CRITICAL)
**Decision:** Remove orphaned module

**Rationale:** The `mention_detector.rs` module was created but never called from `baseline.rs`. Since the current `MentionRelationshipWrapper` doesn't include mention_type fields, and the stats tracking is now properly implemented, we decided to remove the orphaned module rather than integrate it partially.

**Action Taken:** Deleted `crates/iou-ai/src/stakeholder/mention_detector.rs` and removed module declaration from `mod.rs`.

---

### 3. Confidence Calculation Bug (CRITICAL)
**Decision:** Fix confidence calculation

**Rationale:** The original implementation checked patterns across the entire document instead of near the extracted entity. This would cause false confidence boosts (e.g., a document mentioning "dr. Someone" at the top and "Jan de Vries" at the bottom would incorrectly give Jan the doctor's confidence boost).

**Action Taken:** Modified `calculate_person_confidence()` to accept a `position` parameter and check patterns only within a 200-character window (100 before/after) the matched entity position.

---

## Auto-Applied Fixes

### 4. Stats Population (CRITICAL)
**Fix:** Modified `extract()` to properly track statistics:
- Added `ExtractionStats` initialization at start of extraction
- Call `stats.track_confidence()` for each extracted entity
- Return populated stats instead of `Default::default()`

### 5. Processing Time Measurement (CRITICAL)
**Fix:** Added actual processing time measurement:
- Added `std::time::Instant::now()` at start of extraction
- Set `processing_time_ms` to actual elapsed time instead of hardcoded 0

### 6. Unused Methods (IMPORTANT)
**Fix:** Removed `find_text_position()` and `extract_context()` methods that were never called, along with the unused `TextPosition` import.

### 7. Performance Test (SUGGESTION)
**Fix:** Added `test_baseline_extraction_completes_in_under_500ms_for_typical_document()` test to verify the <500ms performance target.

---

## Deferred Items

The following issues were noted but deferred to future sections:

1. **Async trait implementation** - The `BaselineExtractor::extract()` is synchronous but `StakeholderExtractor` trait requires async. Can be addressed when integrating into the main pipeline.

2. **Relationship pattern role capture** - The relationship pattern matches "X, minister van Y" but doesn't capture the role. Can be enhanced in a future section.

3. **Duplicate entity detection** - Will be addressed in section-05 (normalization-deduplication).

4. **Regex character range** - The `[A-Z][a-zÀ-ÿ]` pattern works but could be improved with Unicode property classes. Low priority.

---

## Test Results

All tests pass:
- 16 baseline tests
- 81 total stakeholder tests
- Performance test confirms <500ms target (typically completes in <10ms)

---

## Files Modified

1. `crates/iou-ai/src/stakeholder/baseline.rs` - Fixed confidence calculation, added stats tracking, added performance test, removed unused methods
2. `crates/iou-ai/src/stakeholder/mod.rs` - Removed mention_detector module declaration
3. `crates/iou-ai/src/stakeholder/mention_detector.rs` - Deleted (orphaned module)
