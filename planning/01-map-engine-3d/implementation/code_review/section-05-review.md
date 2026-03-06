# Code Review: Section 05 - Terrain Integration

## Critical Issues

### 1. Map Instance Reference Inconsistency (lines 79, 129, 171)
- `build_terrain_init_script` uses `window['map_{}']` with container_id
- `build_terrain_error_script` and `build_set_terrain_exaggeration_script` use `window.map` (global)
- This will cause terrain error handling and exaggeration updates to fail completely
- **Fix**: Use `window['map_{container_id}']` consistently across all scripts

### 2. API Key Placeholder Causes Silent Failures (lines 20-21)
- `unwrap_or_else(|_| "YOUR_KEY_HERE".to_string())` creates an invalid URL that fails silently
- No user feedback that API key is missing - map loads but terrain never works
- **Fix**: Return a Result type or log a clear warning; document the requirement

### 3. No Exaggeration Validation from Environment (lines 33-37)
- `terrain_exaggeration()` accepts any value from TERRAIN_EXAGGERATION without validation
- Values outside [0.1, 5.0] range will cause runtime errors in MapLibre
- The `validate()` method checks `self.terrain_exaggeration` but `terrain_exaggeration()` bypasses this
- **Fix**: Clamp or validate the parsed environment value

### 4. Fragile Custom URL Detection (line 15)
- `!self.terrain_tile_url.contains("maptiler.com")` is unreliable
- If someone sets `TERRAIN_TILE_URL=https://evil-maptiler.com/` it will be used directly without API key
- Also false positive: `https://mymaptiler.com` would incorrectly trigger API key insertion
- **Fix**: Use a proper boolean flag like `custom_terrain_url: bool` in config

## Security Concerns

### 5. JavaScript Injection via URL Interpolation (line 90)
- `const tileUrl = '{}';` directly interpolates `config.terrain_tile_url()`
- If TERRAIN_TILE_URL contains malicious JavaScript, it will execute
- Example: `TERRAIN_TILE_URL='; alert('xss');//`
- **Fix**: JSON-encode the URL or use proper escaping

### 6. Container ID Injection (line 79)
- `window['map_{}']` interpolates container_id without sanitization
- Could break out of bracket notation if container_id contains special characters

## Code Quality Issues

### 7. Test Flakiness Potential
- Environment variable mutations persist between tests in some test runners
- Tests are order-dependent
- **Fix**: Use a test mutex or restore env vars in `Drop`

### 8. Duplicate Code Pattern (lines 55-60, 33-37)
- `from_env()` parses TERRAIN_EXAGGERATION manually
- `terrain_exaggeration()` also parses it
- Parsing logic duplicated; errors handled differently

### 9. Missing TerrainState Integration
- `TerrainState` struct is defined but never integrated with Map3D component
- The plan (section 5.3) shows adding terrain state to Map3DState, but no Map3DState exists
- `TerrainWarning` component is defined but never actually rendered

## Documentation Issues

### 10. Incorrect Environment Variable Documentation (lines 46-47)
- Docs claim `TERRAIN_EXAGGERATION` defaults to 1.5
- But the actual default is read from `self.terrain_exaggeration` which IS 1.5 by default
- Confusing wording: "otherwise uses the configured value" - which value?

## Edge Cases Missing

### 11. No Bounds Checking on Exaggeration Updates (line 168)
- `build_set_terrain_exaggeration_script` accepts any f64 value
- MapLibre will crash or ignore invalid values
- **Fix**: Document valid range or clamp in Rust before sending to JS

### 12. Race Condition in Terrain Loading (lines 102-111)
- `terrain.loaded` event fires immediately after `map.setTerrain()` before tiles load
- MapLibre documentation shows this event doesn't guarantee tiles are rendered
- May show "terrain loaded" prematurely

## Plan Deviations

### 13. Component Interface Mismatch
- Plan shows `icon` component with `lucide::AlertTriangle` (line 211 of plan)
- Implementation uses inline emoji "⚠️" (line 259 of diff)
- Missing lucide integration - inconsistent with project icon system

## Recommendations

1. **Highest Priority**: Fix map reference inconsistency (#1) - this breaks the feature
2. Add Result-based error handling for missing API key (#2)
3. Add exaggeration bounds validation (#3)
4. Implement proper URL escaping (#5)
5. Consider using serde_json::to_string() for safe JS value injection
6. Add integration test for full terrain loading flow (current tests only check string contents)
7. Document the fallback behavior more clearly when API key is missing
8. The Dutch error messages are good - culturally appropriate for Dutch users
