# Code Review: Section 02 - Baseline Extraction

**Agent:** feature-dev:code-reviewer
**Date:** 2026-03-16
**Files Changed:** 3 files, ~830 insertions

---

## Summary

The baseline extraction module provides a solid foundation for regex-based Dutch government entity extraction. The implementation demonstrates good Rust practices with proper use of `OnceLock` for lazy pattern compilation and comprehensive test coverage. However, there are several critical issues that prevent this from being production-ready.

---

## CRITICAL Issues (Must Fix)

### 1. Missing rust-bert Integration - Core Feature Not Implemented
**Confidence: 100/100**

**Location:** `baseline.rs:128-149`

**Issue:** The plan explicitly requires rust-bert integration for Dutch NER, but the implementation has a placeholder `enable_ner` flag that does nothing. The `BaselineExtractor` struct stores this flag but never uses it.

**Evidence:**
- Line 136: `enable_ner: bool` - stored but never used
- No `rust-bert` dependency added to Cargo.toml
- Plan section 3 explicitly requires: "Add dependency and implement Dutch NER" with `rust-bert = "0.22"`

**Fix:** Either implement the rust-bert integration or update the plan to defer it to a future section. If deferring, document why the decision was made.

---

### 2. MentionDetector Not Integrated - Feature Completely Orphaned
**Confidence: 100/100**

**Location:** `baseline.rs`, `mention_detector.rs`

**Issue:** The `mention_detector.rs` module was created but is **never called** from `baseline.rs`. The plan requires detecting `MentionType` (Author/Recipient/Subject/Referenced) for each extracted entity, but this never happens.

**Evidence:**
- `baseline.rs:238` returns `MentionRelationshipWrapper` which only has `id`, `entity_id`, `document_id`, `confidence`
- Missing fields: `mention_type`, `context`, `position` from `MentionRelationship`
- `detect_mention_type()` function exists but is never called
- Plan section 5 explicitly shows the integration: "Detect mention type from document context"

**Fix:** Either integrate the mention detector or remove the orphaned module. The mentions should include proper mention_type, context, and position.

---

### 3. Confidence Calculation Bug - Uses Entire Document Text
**Confidence: 95/100**

**Location:** `baseline.rs:323-337`

**Issue:** `calculate_person_confidence()` checks if title/role patterns exist ANYWHERE in the document, not near the extracted entity. This causes false confidence boosts.

**Evidence:**
```rust
fn calculate_person_confidence(&self, _name: &str, text: &str) -> f32 {
    let mut confidence: f32 = 0.70;
    // BUG: Checks entire document, not entity context
    if patterns().title_exact.is_match(text) { confidence += 0.10; }
    if patterns().government_role.is_match(text) { confidence += 0.05; }
    confidence.min(1.0)
}
```

A document mentioning "dr. Someone" at the top and "Jan de Vries" at the bottom will give Jan de Vries the doctor's confidence boost incorrectly.

**Fix:** Check patterns only in a window around the matched position (e.g., 100 characters before/after).

---

### 4. Stats Never Populated - Feature Incomplete
**Confidence: 100/100**

**Location:** `baseline.rs:235`

**Issue:** `ExtractionStats` is returned as `Default::default()` with all zeros. No tracking of high/medium/low confidence entities or API calls.

**Evidence:**
- Line 235: `stats: Default::default()`
- Plan requires: "Extraction statistics" with confidence bucket tracking
- `ExtractionStats` has `track_confidence()` method that's never called

**Fix:** Populate stats properly by calling `stats.track_confidence()` for each extracted entity.

---

### 5. Processing Time Not Measured - Performance Claim Unverifiable
**Confidence: 100/100**

**Location:** `baseline.rs:236`

**Issue:** `processing_time_ms: 0` is hardcoded. The plan targets "<500ms per document" but there's no measurement.

**Evidence:**
- Line 236: `processing_time_ms: 0`
- Plan success criterion: "Baseline extraction completes in <500ms for typical document"

**Fix:** Measure actual processing time using `std::time::Instant`.

---

## IMPORTANT Issues (Should Fix)

### 6. Inefficient Regex Pattern - Title Exact Contains Redundant Alternations
**Confidence: 90/100**

**Location:** `baseline.rs:61`

**Issue:** The title pattern includes both with and without dots redundantly:
```rust
r"(?i)\b(dr\.|prof\.|mr\.|ing\.|ir\.|mr\. dr\.|dr|prof|mr|ing|ir)\b"
```

This creates unnecessary backtracking. Use optional dot pattern: `r"(?i)\b(dr\.?|prof\.?|...)\b"`

---

### 7. Relationship Pattern Doesn't Capture Role
**Confidence: 85/100**

**Location:** `baseline.rs:80-82`, test at line 518

**Issue:** The relationship pattern matches "X, minister van Y" but doesn't capture the role (minister/directeur/etc.). The test comment acknowledges this: "Note: current pattern doesn't capture director role".

**Fix:** Add capturing group for the role, as shown in the plan.

---

### 8. Duplicate Entity Detection Missing
**Confidence: 85/100**

**Location:** `baseline.rs:162-188`

**Issue:** The same entity can be extracted multiple times if it appears multiple times in the document. No deduplication logic.

**Fix:** Use `HashSet` or track by name to prevent duplicate entities in results.

---

### 9. Missing Async Trait Implementation
**Confidence: 95/100**

**Location:** `baseline.rs:146-238`

**Issue:** The plan shows `BaselineExtractor` should implement `StakeholderExtractor` trait which is `async`, but the current `extract()` method is synchronous.

**Evidence:**
- Plan: "StakeholderExtractor trait definition" with `async fn extract()`
- Implementation: `pub fn extract()` - not async
- This prevents `BaselineExtractor` from being used as a trait object

---

### 10. TextPosition Methods Never Called
**Confidence: 100/100**

**Location:** `baseline.rs:340-356`

**Issue:** `find_text_position()` and `extract_context()` methods are defined but never called. Position information is never captured in mentions.

---

### 11. Canonical Name Lookup Bug
**Confidence: 90/100**

**Location:** `baseline.rs:294-296`

**Issue:** `name.contains(abbr)` will match incorrectly. "Ministerie van Financiën" contains "Fin" which would incorrectly map to something else.

**Evidence:**
```rust
.find(|(abbr, _)| *abbr == name || name.contains(abbr))
```

"BZK" in "Ministerie van Binnenlandse Zaken" works, but substring matching is dangerous.

---

## SUGGESTIONS (Minor Polish)

### 12. Regex Character Range for Dutch

**Location:** `baseline.rs:72, 81`

The pattern uses `[A-Z][a-zÀ-ÿ]` but this range is problematic. `À-ÿ` includes characters that aren't all lowercase letters. Consider using Unicode property classes or a more carefully constructed range.

---

### 13. No Performance Test

The plan requires a performance test for <500ms target, but none exists. The `test_baseline_extraction_completes_in_under_500ms` test from the plan is not implemented.

---

### 14. MentionDetector Index Calculation Could Panic

**Location:** `mention_detector.rs:29`

`lines.get(line_num)` returns empty string only because of the `unwrap_or(&"")`, but if `line_num` exceeds `lines.len()`, the default empty string may not be the intended behavior.

---

### 15. Hardcoded Tussenvoegsel List

**Location:** `baseline.rs:76, 86`

The list of Dutch prefixes is duplicated between patterns and doesn't match the comprehensive list in `normalization.rs`.

---

## Missing from Plan

1. **rust-bert integration** - Entirely missing
2. **Performance test** - Not implemented
3. **Person extraction without titles** - Only `person_with_title` is extracted
4. **Deduplication** - Not implemented (deferred to section-05 but should be considered)

---

## Positive Aspects

1. Good use of `OnceLock` for lazy pattern initialization
2. Comprehensive test coverage for implemented features
3. Proper error handling with `thiserror`
4. Clean separation of concerns with dedicated modules
5. Good documentation comments
6. Proper handling of accented characters (ë)

---

## Summary

**Critical Issues:** 5
**Important Issues:** 6
**Suggestions:** 4

The implementation provides a solid foundation but falls short on several key requirements from the plan. The missing rust-bert integration and orphaned `mention_detector` module are the most significant gaps. The confidence calculation bug could lead to incorrect entity confidence scores. Before proceeding to section-03, these critical issues should be addressed.
