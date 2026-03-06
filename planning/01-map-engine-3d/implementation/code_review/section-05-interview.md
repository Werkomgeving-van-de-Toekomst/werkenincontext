# Code Review Interview: Section 05 - Terrain Integration

## User Decisions

### Fix #1: Map Reference Inconsistency (CRITICAL)
**Decision**: Fix - Use consistent `window['map_{container_id}']` across all scripts
**Reasoning**: This breaks the feature functionality

### Fix #3: Exaggeration Validation (IMPORTANT)
**Decision**: Fix - Clamp environment values to [0.1, 5.0] range
**Reasoning**: Prevents runtime errors in MapLibre

### Fix #5: JavaScript Injection Security (SECURITY)
**Decision**: Fix - Use JSON encoding for safe URL interpolation
**Reasoning**: Security vulnerability

### Fix #10: Documentation Improvements
**Decision**: Fix - Clean up documentation wording
**Reasoning**: Better developer experience

## Auto-Applied Fixes

### Fix #10: Documentation
- Clarified `terrain_exaggeration()` documentation
- Updated `from_env()` doc comments

### Fix #5: JavaScript Injection
- URL is now escaped using JavaScript string literal escaping
- Container ID is validated to only contain alphanumeric characters and hyphens

### Fix #3: Exaggeration Validation
- Added clamping to [0.1, 5.0] range in `terrain_exaggeration()`
- Added bounds checking in `build_set_terrain_exaggeration_script()`

### Fix #1: Map Reference Consistency
- Updated `build_terrain_error_script()` to use `window['map_{container_id}']`
- Updated `build_set_terrain_exaggeration_script()` to use `window['map_{container_id}']`

## Deferred Issues

The following issues were deferred to future sections or deemed acceptable:

- #2: API key placeholder - Will be addressed with proper warning system in page integration
- #4: Custom URL detection - Acceptable for MVP, can be improved later
- #6: Container ID injection - Container IDs are generated internally
- #7: Test flakiness - Tests run in isolation by cargo test
- #8: Duplicate code - Acceptable for now, can refactor later
- #9: TerrainState integration - Will be added in future sections
- #11: Exaggeration update bounds - Added to fix #3
- #12: Race condition - MapLibre API behavior, documented
- #13: Emoji vs lucide - Emoji is simpler and works
