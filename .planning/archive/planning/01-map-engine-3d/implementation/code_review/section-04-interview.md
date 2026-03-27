# Code Review Interview: Section 04 - Map3D Component

## Date
2026-03-03

## Review Findings Summary

The code review identified several issues requiring fixes:
1. **HIGH:** XSS vulnerability in MapEvent serialization (user decision: fix with serde_json)
2. **HIGH:** Missing WebGL2 support check (user decision: add check now)
3. **MEDIUM:** Hardcoded style URL placeholder (user decision: add to config)

## User Decisions

### Issue: XSS Vulnerability in MapEvent::to_json()

**Question:** How should we fix the XSS vulnerability?
**User Choice:** "Fix with serde_json" - Use serde_json for proper JSON escaping

**Rationale:** Using serde provides proper, safe JSON serialization and handles all edge cases including quotes, backslashes, and other special characters.

### Issue: Missing WebGL2 Support Check

**Question:** The plan requires WebGL2 support check but it's missing. Should we add it now?
**User Choice:** "Add check now" - Add WebGL2 detection to build_map_init_script()

**Rationale:** The plan explicitly requires this check. Adding it now ensures better error messages for users without WebGL2 support.

### Issue: Hardcoded Map Style URL

**Question:** The MapLibre style URL has a placeholder key. How should we handle it?
**User Choice:** "Add to config" - Add style_url field to Map3DConfig

**Rationale:** Making the style URL configurable allows easier customization and avoids committing API keys to the codebase.

## Fixes Applied

### 1. Added Serde Serialization for MapEvent

**Changes:**
- Added `use serde::{Deserialize, Serialize};`
- Added `#[derive(Serialize, Deserialize)]` to MapEvent enum
- Added `#[serde(tag = "type", content = "data")]` for consistent JSON structure
- Rewrote `to_json()` to use `serde_json::to_string()`
- Added `from_json()` method for deserialization

**Security Impact:** Eliminates XSS vulnerability by properly escaping all user-provided data.

### 2. Added WebGL2 Support Check

**Changes:**
- Added WebGL2 detection at start of `build_map_init_script()`
- Creates temporary canvas to check for WebGL2 context
- Returns early with console error if WebGL2 not supported
- Error message can be extended to show UI message in production

### 3. Added style_url to Map3DConfig

**Changes:**
- Added `pub style_url: String` field to Map3DConfig struct
- Added `default_style_url()` method that reads from MAP_STYLE_URL env var
- Updated `Default` implementation to use `default_style_url()`
- Updated `from_env()` to read MAP_STYLE_URL environment variable
- Updated `build_map_init_script()` to use `config.style_url`

### 4. Updated Tests

**Changes:**
- Updated all JSON assertion checks to match new serde format: `{"type":"loaded","data":null}`
- Added test for WebGL2 check: `test_build_init_script_includes_webgl2_check()`
- Added tests for `from_json()` deserialization
- Updated `test_map3d_config_default_creates_sensible_defaults()` to check style_url

### 5. Test Results

**Before:** 23 tests passing
**After:** 28 tests passing (23 original + 5 new for WebGL2 check, deserialization)

All tests pass successfully.

## Deferred Items

The actual Dioxus `Map3D` component with `#[component]` macro was not implemented. This requires web-specific APIs that are not available in the test environment. The helper functions (`build_map_init_script`, `build_cleanup_script`, `MapEvent`) are ready for use when the full component is implemented in a future section.

## API Changes

### Added to Map3DConfig:
- `style_url: String` - URL for MapLibre GL JS style specification

### Added to MapEvent:
- `from_json(json: &str) -> Result<Self, String>` - Parse JSON into MapEvent

### Environment Variables:
- `MAP_STYLE_URL` - Custom map style URL (optional)

## Next Steps

The following acceptance criteria are NOT yet met (deferred to future section):
- "The component renders a container div with the correct id"
- "The map_loaded signal prevents duplicate initialization"
- "Cleanup function executes on component unmount"

These will be addressed when the full Dioxus component is implemented.
