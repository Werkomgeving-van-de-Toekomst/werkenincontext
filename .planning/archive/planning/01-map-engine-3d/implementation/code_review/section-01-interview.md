# Code Review Interview - Section 01

## Interview Decisions

### Critical Fix: Invalid Dioxus.toml Configuration

**Issue**: `[web.resource.map_3d]` is not a valid Dioxus section - resources won't load.

**User Decision**: Implement dynamic JS injection (inject MapLibre CSS/JS via JavaScript eval when Map3D component initializes).

**Action Plan**:
1. Remove the invalid `[web.resource.map_3d]` section from Dioxus.toml
2. Don't add MapLibre to `[web.resource]` (avoid loading it unconditionally)
3. Implement dynamic resource injection in section-04 (Map3D component) using `document::eval()`

**Rationale**: This keeps the 3D map resources truly conditional - they only load when the Map3D component is actually used, not on every page load.

## Auto-Fixes Applied

### Fix 1: Remove invalid Dioxus.toml section
- Remove `[web.resource.map_3d]` section entirely
- Keep original `[web.resource]` unchanged

### Fix 2: Fix comment typo
- Change `/ pub use layer_control_3d::LayerControl3D;` to `// pub use layer_control_3d::LayerControl3D;`

## Items Let Go

- Low-value tests: The placeholder tests are acceptable for now since the modules are empty stubs. Real tests will be added when components are implemented.

## Deferred to Later

- Dynamic JS injection implementation: Will be done in section-04 when building the Map3D component
- Consider adding a note in section-01 about this decision
