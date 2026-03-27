Now I have all the context. Let me generate the section content for `section-03-terrain-encoding`. Based on the manifest, this section covers:

> Implement elevation_to_terrain_rgb() and terrain_rgb_to_elevation() functions with full test coverage.

From the plan, this is about Terrain-RGB encoding for elevation data. The key content is in section 5.2 of the plan and section 5.2 of the TDD plan.

# Section 03: Terrain Encoding

## Overview

This section implements the Terrain-RGB encoding/decoding functions used to convert elevation data into the format expected by MapLibre GL JS for terrain rendering. These are pure Rust functions with no external dependencies, making them ideal for TDD.

## Background

### Why Terrain-RGB Encoding?

MapLibre GL JS expects terrain tiles in a specific encoded format called Terrain-RGB. This format stores elevation values in the RGB channels of PNG tiles, where each pixel's color represents the height at that location.

### The Encoding Formula

The standard Mapbox/MapLibre Terrain-RGB encoding uses the following formula:

```
elevation_meters = -10000 + ((R * 65536 + G * 256 + B) * 0.1)
```

This encoding supports:
- Range: -10,000m to +6,553.5m (approximately -10km to +6.5km)
- Precision: 0.1m (10cm)

For Flevoland:
- Minimum elevation: approximately -6.7m (below sea level)
- Maximum elevation: approximately +5m (above sea level)
- Both well within the supported range

## Dependencies

This section has no dependencies on other sections. It implements pure Rust functions that can be developed and tested independently.

## Files to Create

| File | Purpose |
|------|---------|
| `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/terrain_encoding.rs` | Encoding/decoding functions |
| `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/terrain_encoding_test.rs` | Unit tests |

## Tests (Write First)

Create the test file with comprehensive coverage of all edge cases:

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/terrain_encoding_test.rs`

```rust
#[cfg(test)]
mod tests {
    use super::super::terrain_encoding::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_elevation_to_terrain_rgb_minimum() {
        // -10000m should encode to (0, 0, 0)
        let (r, g, b) = elevation_to_terrain_rgb(-10000.0);
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn test_elevation_to_terrain_rgb_sea_level() {
        // 0m (sea level) encoding
        let (r, g, b) = elevation_to_terrain_rgb(0.0);
        // 0 = -10000 + (encoded * 0.1)
        // encoded = 100000
        // 100000 = 65536 * 1 + 34464
        // 34464 = 256 * 134 + 160
        let expected = ((100000 >> 16) & 0xFF, (100000 >> 8) & 0xFF, 100000 & 0xFF);
        assert_eq!((r, g, b), expected);
    }

    #[test]
    fn test_elevation_to_terrain_rgb_flevoland_negative() {
        // Test typical Flevoland negative elevation
        let (r, g, b) = elevation_to_terrain_rgb(-5.5);
        let decoded = terrain_rgb_to_elevation(r, g, b);
        assert_relative_eq!(decoded, -5.5, epsilon = 0.1);
    }

    #[test]
    fn test_elevation_to_terrain_rgb_flevoland_positive() {
        // Test typical Flevoland positive elevation
        let (r, g, b) = elevation_to_terrain_rgb(3.2);
        let decoded = terrain_rgb_to_elevation(r, g, b);
        assert_relative_eq!(decoded, 3.2, epsilon = 0.1);
    }

    #[test]
    fn test_encoding_round_trip() {
        // Round-trip: encode then decode should return original
        let original = -10000.0;
        let (r, g, b) = elevation_to_terrain_rgb(original);
        let decoded = terrain_rgb_to_elevation(r, g, b);
        assert_relative_eq!(original, decoded, epsilon = 0.1);
    }

    #[test]
    fn test_encoding_round_trip_sea_level() {
        let original = 0.0;
        let (r, g, b) = elevation_to_terrain_rgb(original);
        let decoded = terrain_rgb_to_elevation(r, g, b);
        assert_relative_eq!(original, decoded, epsilon = 0.1);
    }

    #[test]
    fn test_encoding_round_trip_positive() {
        let original = 1234.5;
        let (r, g, b) = elevation_to_terrain_rgb(original);
        let decoded = terrain_rgb_to_elevation(r, g, b);
        assert_relative_eq!(original, decoded, epsilon = 0.1);
    }

    #[test]
    fn test_fractional_elevation_rounding() {
        // Test that fractional values are handled correctly
        // The encoding has 0.1m precision, so some rounding is expected
        let original = 1.23;
        let (r, g, b) = elevation_to_terrain_rgb(original);
        let decoded = terrain_rgb_to_elevation(r, g, b);
        assert_relative_eq!(original, decoded, epsilon = 0.1);
    }

    #[test]
    fn test_terrain_rgb_to_elevation_max() {
        // Maximum encodable value: (255, 255, 255)
        // elevation = -10000 + ((255*65536 + 255*256 + 255) * 0.1)
        let expected = -10000.0 + ((255*65536 + 255*256 + 255) as f64 * 0.1);
        let actual = terrain_rgb_to_elevation(255, 255, 255);
        assert_relative_eq!(expected, actual);
    }

    #[test]
    fn test_elevation_clamping_negative() {
        // Values below -10000 should be clamped to 0,0,0
        let (r, g, b) = elevation_to_terrain_rgb(-15000.0);
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn test_elevation_clamping_positive() {
        // Values above maximum should be clamped to 255,255,255
        let (r, g, b) = elevation_to_terrain_rgb(10000.0);
        assert_eq!((r, g, b), (255, 255, 255));
    }
}
```

## Implementation

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/terrain_encoding.rs`

```rust
//! Terrain-RGB encoding for MapLibre GL JS terrain tiles.
//!
//! This module implements the Mapbox Terrain-RGB specification for encoding
//! elevation values into RGB pixel data. MapLibre uses this format to render
//! 3D terrain from raster elevation tiles.
//!
//! # Encoding Formula
//!
//! The standard encoding formula is:
//! ```text
//! elevation_meters = -10000 + ((R * 65536 + G * 256 + B) * 0.1)
//! ```
//!
//! This provides:
//! - Range: -10,000m to approximately +6,553m
//! - Precision: 0.1m (10cm)

/// Encodes an elevation value in meters to Terrain-RGB color components.
///
/// # Arguments
/// * `elevation_meters` - Elevation in meters (can be negative)
///
/// # Returns
/// A tuple of (R, G, B) values, each in the range 0-255
///
/// # Encoding
/// The elevation is normalized to a 24-bit unsigned integer:
/// - Add 10,000 to shift minimum to 0
/// - Multiply by 10 for 0.1m precision
/// - Extract R, G, B from high, mid, low bytes
///
/// # Example
/// ```rust
/// use iou_frontend::components::terrain_encoding::elevation_to_terrain_rgb;
///
/// // Sea level encodes to a specific RGB value
/// let (r, g, b) = elevation_to_terrain_rgb(0.0);
/// ```
pub fn elevation_to_terrain_rgb(elevation_meters: f64) -> (u8, u8, u8) {
    // Shift and scale: -10000m becomes 0
    let normalized = (elevation_meters + 10000.0) * 10.0;
    
    // Clamp to valid range [0, 2^24 - 1]
    let clamped = normalized.clamp(0.0, 16777215.0);
    
    // Convert to integer
    let encoded = clamped as u32;
    
    // Extract RGB components
    let r = ((encoded >> 16) & 0xFF) as u8;
    let g = ((encoded >> 8) & 0xFF) as u8;
    let b = (encoded & 0xFF) as u8;
    
    (r, g, b)
}

/// Decodes Terrain-RGB color components back to elevation in meters.
///
/// # Arguments
/// * `r` - Red channel value (0-255)
/// * `g` - Green channel value (0-255)
/// * `b` - Blue channel value (0-255)
///
/// # Returns
/// Elevation in meters (can be negative)
///
/// # Example
/// ```rust
/// use iou_frontend::components::terrain_encoding::terrain_rgb_to_elevation;
///
/// let elevation = terrain_rgb_to_elevation(129, 48, 160);
/// ```
pub fn terrain_rgb_to_elevation(r: u8, g: u8, b: u8) -> f64 {
    // Combine RGB into 24-bit value
    let encoded = (r as u32) * 65536 + (g as u32) * 256 + (b as u32);
    
    // Reverse the encoding: scale and shift back
    let elevation = (encoded as f64 * 0.1) - 10000.0;
    
    elevation
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests are defined in terrain_encoding_test.rs
    // This module exists for doctests and any quick inline tests
}
```

## Module Updates

Update `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/mod.rs` to include the new module:

```rust
// ... existing imports ...

pub mod terrain_encoding;

// ... rest of file ...
```

## Cargo.toml Dependencies

Add to `/Users/marc/Projecten/iou-modern/crates/iou-frontend/Cargo.toml`:

```toml
[dev-dependencies]
approx = "0.5"  # For floating point comparison in tests
```

## Running Tests

```bash
# Run all terrain encoding tests
cargo test -p iou-frontend terrain_encoding

# Run with output for debugging
cargo test -p iou-frontend terrain_encoding -- --nocapture

# Run specific test
cargo test -p iou-frontend test_encoding_round_trip
```

## Acceptance Criteria

This section is complete when:

1. All tests pass: `cargo test -p iou-frontend terrain_encoding`
2. Round-trip encoding (encode then decode) preserves values within 0.1m precision
3. Flevoland elevation range (-7m to +5m) encodes/decodes correctly
4. Edge cases (minimum, maximum, negative, sea level) are handled
5. Functions are documented with rustdoc comments
6. Module is properly exported from components/mod.rs

## Implementation Notes

### Actual Implementation (2026-03-03)

**File Created:**
- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/terrain_encoding.rs` (with inline tests)

**Key Deviation from Plan:**

The `elevation_to_terrain_rgb` function returns `Option<(u8, u8, u8)>` instead of `(u8, u8, u8)`. This was chosen during code review for safer error handling:

- Returns `None` for NaN and Infinity inputs
- Prevents silent data corruption from invalid float values
- More explicit API for error handling

**Test Organization:**

Tests are in a `#[cfg(test)]` module within `terrain_encoding.rs` (idiomatic Rust) rather than a separate `terrain_encoding_test.rs` file as originally planned.

**Test Count:** 19 tests (15 original + 4 for NaN/Infinity edge cases)

**All acceptance criteria met.**