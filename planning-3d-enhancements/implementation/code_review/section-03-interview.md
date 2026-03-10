# Code Review Interview: Section 03 - Backend WGS84 Bbox Endpoint

## Interview Date
2025-03-07

## User Decisions

### 1. Bbox Order Validation (CRITICAL)
**Issue:** No validation that min_lon < max_lon and min_lat < max_lat
**Decision:** Fix it (recommended)
**Fix:** Add validation after parsing coordinates

### 2. HTTP Status Code for Errors (CRITICAL)
**Issue:** Returns 200 OK with error field instead of 400 Bad Request
**Decision:** Implement proper 400 errors
**Fix:** Add ApiError enum with IntoResponse trait

### 3. Whitespace Trimming (CRITICAL)
**Issue:** Coordinates with spaces fail to parse
**Decision:** Auto-fix whitespace
**Fix:** Add .trim() when splitting coordinates

## Auto-Fixes Applied

### 4. NaN/Infinity Validation (HIGH)
**Fix:** Add explicit is_finite() check after parsing each coordinate

### 5. Proj Instance Reuse (HIGH - Performance)
**Fix:** Add OnceLock for WGS84->RD Proj instance

### 6. Stronger Test Assertions (MEDIUM)
**Fix:** Use known reference points with tighter tolerance in tests

## Items Deferred (Let Go)

- Missing test: Limit parameter enforcement (nice to have)
- Missing test: Response format validation (nice to have)
- Missing test: Both bbox parameters present (low priority)
- ApiError enum completeness (partial implementation for 400 errors)
