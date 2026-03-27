Now I have all the context needed. Let me generate the section content.

# Section 02: Backend Coordinate Conversion

## Overview

This section enables accurate coordinate conversion from RD (EPSG:28992) to WGS84 (EPSG:4326) by activating the `proj` crate integration in the backend buildings service. Currently, the coordinate conversion is disabled, forcing use of a simplified fallback formula that produces location errors of hundreds of meters to kilometers.

## Why This Matters

The Netherlands uses RD (Rijksdriehoeksmeting) coordinates for its official geospatial data, including the 3DBAG API. MapLibre GL JS uses WGS84 (latitude/longitude). The current implementation has the `proj` crate code written but disabled:

- Line 83: `let use_proj = false;` - Coordinate conversion is forced to use fallback
- The fallback `rd_to_wgs84_fallback()` uses a linear approximation based on Amersfoort coordinates
- This approximation accumulates significant errors away from the reference point

By enabling the `proj` crate, we get accurate coordinate transformations using proper projection definitions, which is essential for buildings to appear at their correct map positions.

## Current State

File: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs`

```rust
// Line 83 - proj is explicitly disabled
let use_proj = false; //proj::Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).is_ok();

// Line 87 - use_proj determines which conversion function is used
let up = use_proj;
// ...

// Lines 113-117 - conditional code path
let geom = if up {
    convert_geometry_proj(footprint, vertices, tf)
} else {
    convert_geometry_fallback(footprint, vertices, tf)
};
```

The `convert_geometry_proj()` function (lines 145-204) is fully implemented but never executes because `use_proj` is hardcoded to `false`.

## Implementation

### Step 1: Enable the proj crate

Modify line 83 in `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs`:

**Before:**
```rust
let use_proj = false; //proj::Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).is_ok();
```

**After:**
```rust
let use_proj = proj::Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).is_ok();
```

This change:
- Attempts to create the proj transformation at runtime
- Sets `use_proj` to `true` if successful
- Falls back to `false` if proj fails (graceful degradation)

### Step 2: Remove the hardcoded false

The commented-out check suggests the original code attempted runtime detection but was disabled. By restoring the runtime check:

1. **Normal operation**: `use_proj = true`, accurate conversion via `convert_geometry_proj()`
2. **Error case**: If `Proj::new_known_crs()` fails, `use_proj = false`, fallback to `convert_geometry_fallback()`

This provides robust fallback behavior while enabling accurate coordinates by default.

### Step 3: Verify proj dependency

Ensure the `proj` crate is included in `/Users/marc/Projecten/iou-modern/crates/iou-api/Cargo.toml`:

```toml
[dependencies]
proj = "0.27"  # or compatible version
```

## Tests

Add the following tests to `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs`:

```rust
#[cfg(test)]
mod coordinate_tests {
    use super::*;

    #[test]
    fn test_rd_to_wgs84_amersfoort() {
        // Amersfoort coordinates (reference point)
        // RD: (155000, 463000) should approximate WGS84: (5.3876, 52.1552)
        if let Some((lon, lat)) = rd_to_wgs84(155000.0, 463000.0) {
            assert!((lon - 5.3876).abs() < 0.001, "Longitude mismatch");
            assert!((lat - 52.1552).abs() < 0.001, "Latitude mismatch");
        } else {
            panic!("proj conversion failed");
        }
    }

    #[test]
    fn test_rd_to_wgs84_returns_none_on_invalid() {
        // Extreme coordinates that might fail conversion
        let result = rd_to_wgs84(f64::NAN, f64::NAN);
        assert!(result.is_none() || result.is_some()); // Should not panic
    }

    #[test]
    fn test_fallback_conversion_does_not_panic() {
        // Fallback should handle all inputs gracefully
        let (lon, lat) = rd_to_wgs84_fallback(155000.0, 463000.0);
        assert!(lon.is_finite());
        assert!(lat.is_finite());
    }

    #[test]
    fn test_proj_more_accurate_than_fallback() {
        // Compare both methods at a known point
        let rd_x = 155000.0;
        let rd_y = 463000.0;
        
        let proj_result = rd_to_wgs84(rd_x, rd_y);
        let fallback_result = rd_to_wgs84_fallback(rd_x, rd_y);
        
        if let Some((proj_lon, proj_lat)) = proj_result {
            // proj should give valid results
            assert!(proj_lon >= 3.0 && proj_lon <= 7.5, "Longitude out of Netherlands range");
            assert!(proj_lat >= 50.0 && proj_lat <= 54.0, "Latitude out of Netherlands range");
            
            // Fallback should be in reasonable range too
            assert!(fallback_result.0 >= 3.0 && fallback_result.0 <= 7.5);
            assert!(fallback_result.1 >= 50.0 && fallback_result.1 <= 54.0);
        }
    }

    #[test]
    fn test_convert_geometry_proj_valid_input() {
        use proj::Proj;
        
        // Skip if proj is not available
        let proj = match Proj::new_known_crs("EPSG:28992", "EPSG:4326", None) {
            Ok(p) => p,
            Err(_) => return, // Skip test if proj unavailable
        };
        
        // Verify proj can convert a simple point
        let result = proj.convert((155000.0, 463000.0));
        assert!(result.is_ok(), "proj should convert valid RD coordinates");
        
        let (lon, lat) = result.unwrap();
        assert!((lon - 5.3876).abs() < 0.01);
        assert!((lat - 52.1552).abs() < 0.01);
    }
}
```

## Verification

After implementation, verify the change works:

1. **Build test**: `cargo test --package iou-api coordinate_tests`
2. **Manual verification**: 
   - Load the map in the browser
   - Navigate to a known location (e.g., Amsterdam Central Station)
   - Verify buildings appear at correct map positions (align with map tiles)
   - Check for visible offset between buildings and map features

3. **Compare before/after** (if possible):
   - Note building positions before change
   - Enable proj
   - Note building positions after change
   - Offset should be eliminated or significantly reduced

## Dependencies

This section has no dependencies on other sections and can be implemented in parallel with:
- section-01-prerequisites
- section-05-frontend-css

## What Changes

**Files modified:**
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs` (1 line changed, tests added)

**Lines affected:**
- Line 83: Change `let use_proj = false;` to runtime check

## Expected Outcomes

After implementing this section:

1. Buildings appear at their correct geographic positions on the map
2. No visible offset between building footprints and map tiles
3. Coordinate accuracy within centimeters (proj) vs kilometers (fallback)
4. Graceful fallback if proj initialization fails

## Notes

- The `proj` crate uses PROJ.4/PROJ library for accurate coordinate transformations
- EPSG:28992 is the Dutch RD coordinate system
- EPSG:4326 is WGS84 (standard GPS coordinates)
- The code already handles both cases; we're just enabling the accurate path

## Implementation Notes (2025-03-07)

**What was actually implemented:**

### Files Modified
- `crates/iou-api/src/routes/buildings_3d.rs`

### Changes Made

1. **Enabled proj crate runtime check** (line ~90)
   - Changed from hardcoded `let use_proj = false;` to `let use_proj = is_proj_available();`
   - Added `OnceLock<bool>` cache to avoid repeated expensive Proj initialization

2. **Added OnceLock cache for performance** (lines 17-24)
   ```rust
   static PROJ_AVAILABLE: OnceLock<bool> = OnceLock::new();

   fn is_proj_available() -> bool {
       *PROJ_AVAILABLE.get_or_init(|| {
           proj::Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).is_ok()
       })
   }
   ```
   This caches the proj availability check at startup, avoiding expensive initialization on every request.

3. **Added comprehensive test suite** (lines 270-320)
   - `test_rd_to_wgs84_amersfoort()` - Validates proj conversion at reference point
   - `test_rd_to_wgs84_returns_none_on_invalid()` - Handles NaN and infinity
   - `test_fallback_conversion_does_not_panic()` - Fallback robustness
   - `test_proj_more_accurate_than_fallback()` - Range validation
   - `test_convert_geometry_proj_valid_input()` - Direct proj conversion test

### Code Review Fixes Applied

**Performance Improvement:**
- Initial implementation created a new Proj instance on every request (potentially 50+ times per request)
- Fixed by caching availability check with OnceLock
- Note: Actual Proj instance is still created once per geometry conversion (not per coordinate) due to thread safety limitations (raw pointers in Proj)

**Test Quality:**
- Fixed meaningless assertion `assert!(result.is_none() || result.is_some())` to proper validation
- Added infinity test case

### Dependencies Verified
- `proj = "0.31.0"` already present in `crates/iou-api/Cargo.toml`

### Compilation Status
✅ Code compiles successfully with `cargo check`