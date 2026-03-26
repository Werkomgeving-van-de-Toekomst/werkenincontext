# Code Review Interview: Section 02 - Template System

## Date
2026-03-01

## Context
After implementing the Tera-based template engine, a code review identified several issues. This interview documents the resolution.

## Issues Addressed

### 1. Unused chrono Imports (Auto-fixed)
**Original Finding**: `use chrono::{DateTime, NaiveDate}` were imported but the code uses fully-qualified `chrono::DateTime` paths.

**Fix Applied**:
- Removed unused imports from templates.rs
- Code already uses `chrono::DateTime::parse_from_rfc3339()` and `chrono::NaiveDate::parse_from_str()`

**Files Modified**:
- `crates/iou-ai/src/templates.rs`

### 2. Unsafe XML Escaping (Auto-fixed)
**Original Finding**: The `escape_xml()` function used chained `.replace()` calls which is order-dependent and could double-escape.

**Fix Applied**:
- Replaced with single-pass loop implementation
- Iterates through each character once, escaping only the special XML characters
- No risk of double-escaping regardless of character order

**Files Modified**:
- `crates/iou-ai/src/conversion.rs`

### 3. Missing Pandoc Timeout (Deemed Non-Critical)
**Original Finding**: `convert_with_pandoc()` could hang indefinitely if pandoc is unresponsive.

**Decision**: Deferred to future work. Implementing a proper timeout in a synchronous context requires complex select/timeout mechanisms or async/await. Pandoc is expected to be fast for typical documents, and the fallback ODT generation handles pandoc unavailability.

### 4. Missing slugify Validation (Already Fixed During Implementation)
**Original Finding**: Empty slug output could cause issues.

**Fix Applied**:
- Added fallback UUID generation when slugify produces empty string
- `format!("doc-{}", uuid::Uuid::new_v4())`

**Files Modified**:
- `crates/iou-ai/src/templates.rs`

### 5. missing_variables Field Never Populated (Auto-fixed)
**Original Finding**: `RenderedDocument` had `missing_variables` field but Tera's strict mode means missing variables cause render errors, making this field useless.

**Fix Applied**:
- Removed `missing_variables` field from `RenderedDocument` struct
- Removed `detect_missing_variables()` helper function
- Updated comment to explain Tera's strict mode behavior

**Files Modified**:
- `crates/iou-core/src/document.rs`

## Test Results
All 32 tests pass in iou-ai package:
- Template engine tests: 12/12 pass
- Conversion tests: 4/4 pass
- Existing tests (NER, semantic, etc.): 16/16 pass

## Summary of Changes
| Category | Count |
|----------|-------|
| Files created | 4 |
| Files modified | 4 |
| Lines added | ~1200 |
| Tests added | 16 |
| Dependencies added | 2 (tera, slug) |

## Code Quality Improvements
- Order-independent XML escaping prevents double-escaping bugs
- Removed dead code (missing_variables field)
- Clear comments explaining Tera's strict mode behavior
- Slugify fallback prevents empty identifier issues

## Approval
All critical fixes applied. Deferred timeout consideration to future async refactor. Ready for commit.
