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
/// `Some((R, G, B))` with values in range 0-255, or `None` if the input is NaN or infinite
///
/// # Encoding
/// The elevation is normalized to a 24-bit unsigned integer:
/// - Add 10,000 to shift minimum to 0
/// - Multiply by 10 for 0.1m precision
/// - Extract R, G, B from high, mid, low bytes
///
/// # Errors
/// Returns `None` for:
/// - `NaN` (not a number) values
/// - Positive or negative infinity
///
/// # Example
/// ```rust
/// use iou_frontend::components::terrain_encoding::elevation_to_terrain_rgb;
///
/// // Sea level encodes to a specific RGB value
/// let rgb = elevation_to_terrain_rgb(0.0);
/// assert_eq!(rgb, Some((1, 134, 160)));
///
/// // Invalid values return None
/// assert_eq!(elevation_to_terrain_rgb(f64::NAN), None);
/// assert_eq!(elevation_to_terrain_rgb(f64::INFINITY), None);
/// ```
pub fn elevation_to_terrain_rgb(elevation_meters: f64) -> Option<(u8, u8, u8)> {
    // Reject NaN and infinity explicitly
    if !elevation_meters.is_finite() {
        return None;
    }

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

    Some((r, g, b))
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
    use approx::assert_relative_eq;

    #[test]
    fn test_elevation_to_terrain_rgb_minimum() {
        // -10000m should encode to (0, 0, 0)
        let result = elevation_to_terrain_rgb(-10000.0);
        assert_eq!(result, Some((0, 0, 0)));
    }

    #[test]
    fn test_elevation_to_terrain_rgb_sea_level() {
        // 0m (sea level) encoding
        let result = elevation_to_terrain_rgb(0.0);
        // 0 = -10000 + (encoded * 0.1)
        // encoded = 100000
        // 100000 = 65536 * 1 + 34464
        // 34464 = 256 * 134 + 160
        let encoded: u32 = 100000;
        let expected = ((encoded >> 16) & 0xFF, (encoded >> 8) & 0xFF, encoded & 0xFF);
        assert_eq!(result, Some((expected.0 as u8, expected.1 as u8, expected.2 as u8)));
    }

    #[test]
    fn test_elevation_to_terrain_rgb_flevoland_negative() {
        // Test typical Flevoland negative elevation
        let result = elevation_to_terrain_rgb(-5.5);
        assert!(result.is_some());
        let (r, g, b) = result.unwrap();
        let decoded = terrain_rgb_to_elevation(r, g, b);
        assert_relative_eq!(decoded, -5.5, epsilon = 0.1);
    }

    #[test]
    fn test_elevation_to_terrain_rgb_flevoland_positive() {
        // Test typical Flevoland positive elevation
        let result = elevation_to_terrain_rgb(3.2);
        assert!(result.is_some());
        let (r, g, b) = result.unwrap();
        let decoded = terrain_rgb_to_elevation(r, g, b);
        assert_relative_eq!(decoded, 3.2, epsilon = 0.1);
    }

    #[test]
    fn test_encoding_round_trip() {
        // Round-trip: encode then decode should return original
        let original = -10000.0;
        let result = elevation_to_terrain_rgb(original);
        assert!(result.is_some());
        let (r, g, b) = result.unwrap();
        let decoded = terrain_rgb_to_elevation(r, g, b);
        assert_relative_eq!(original, decoded, epsilon = 0.1);
    }

    #[test]
    fn test_encoding_round_trip_sea_level() {
        let original = 0.0;
        let result = elevation_to_terrain_rgb(original);
        assert!(result.is_some());
        let (r, g, b) = result.unwrap();
        let decoded = terrain_rgb_to_elevation(r, g, b);
        assert_relative_eq!(original, decoded, epsilon = 0.1);
    }

    #[test]
    fn test_encoding_round_trip_positive() {
        let original = 1234.5;
        let result = elevation_to_terrain_rgb(original);
        assert!(result.is_some());
        let (r, g, b) = result.unwrap();
        let decoded = terrain_rgb_to_elevation(r, g, b);
        assert_relative_eq!(original, decoded, epsilon = 0.1);
    }

    #[test]
    fn test_fractional_elevation_rounding() {
        // Test that fractional values are handled correctly
        // The encoding has 0.1m precision, so some rounding is expected
        let original = 1.23;
        let result = elevation_to_terrain_rgb(original);
        assert!(result.is_some());
        let (r, g, b) = result.unwrap();
        let decoded = terrain_rgb_to_elevation(r, g, b);
        assert_relative_eq!(original, decoded, epsilon = 0.1);
    }

    #[test]
    fn test_terrain_rgb_to_elevation_max() {
        // Maximum encodable value: (255, 255, 255)
        // elevation = -10000 + ((255*65536 + 255*256 + 255) * 0.1)
        let expected = -10000.0 + ((255 * 65536 + 255 * 256 + 255) as f64 * 0.1);
        let actual = terrain_rgb_to_elevation(255, 255, 255);
        assert_relative_eq!(expected, actual);
    }

    #[test]
    fn test_elevation_clamping_negative() {
        // Values below -10000 should be clamped to 0,0,0
        let result = elevation_to_terrain_rgb(-15000.0);
        assert_eq!(result, Some((0, 0, 0)));
    }

    #[test]
    fn test_elevation_clamping_positive() {
        // Values above maximum should be clamped to 255,255,255
        // Maximum encodable: -10000 + (16777215 * 0.1) ≈ 1,667,721m
        let result = elevation_to_terrain_rgb(2000000.0);
        assert_eq!(result, Some((255, 255, 255)));
    }

    #[test]
    fn test_flevoland_range_negative() {
        // Flevoland minimum elevation: approximately -6.7m
        let test_elevations = [-6.7, -6.0, -5.0, -3.5];
        for elevation in test_elevations {
            let result = elevation_to_terrain_rgb(elevation);
            assert!(result.is_some());
            let (r, g, b) = result.unwrap();
            let decoded = terrain_rgb_to_elevation(r, g, b);
            assert_relative_eq!(elevation, decoded, epsilon = 0.1);
        }
    }

    #[test]
    fn test_flevoland_range_positive() {
        // Flevoland maximum elevation: approximately +5m
        let test_elevations = [0.0, 1.0, 2.5, 4.0, 5.0];
        for elevation in test_elevations {
            let result = elevation_to_terrain_rgb(elevation);
            assert!(result.is_some());
            let (r, g, b) = result.unwrap();
            let decoded = terrain_rgb_to_elevation(r, g, b);
            assert_relative_eq!(elevation, decoded, epsilon = 0.1);
        }
    }

    #[test]
    fn test_sea_level_exact_value() {
        // 0m should encode to exactly (1, 134, 160)
        // Because: 0 = -10000 + (100000 * 0.1)
        // 100000 = 1*65536 + 134*256 + 160
        let result = elevation_to_terrain_rgb(0.0);
        assert_eq!(result, Some((1, 134, 160)));
    }

    #[test]
    fn test_decode_known_rgb_values() {
        // Test some known RGB -> elevation conversions
        assert_eq!(terrain_rgb_to_elevation(0, 0, 0), -10000.0);
        assert_relative_eq!(terrain_rgb_to_elevation(1, 134, 160), 0.0, epsilon = 0.1);
    }

    #[test]
    fn test_nan_returns_none() {
        // NaN should return None instead of silently encoding to min value
        let result = elevation_to_terrain_rgb(f64::NAN);
        assert_eq!(result, None, "NaN should return None");
    }

    #[test]
    fn test_positive_infinity_returns_none() {
        // Positive infinity should return None
        let result = elevation_to_terrain_rgb(f64::INFINITY);
        assert_eq!(result, None, "Positive infinity should return None");
    }

    #[test]
    fn test_negative_infinity_returns_none() {
        // Negative infinity should return None
        let result = elevation_to_terrain_rgb(f64::NEG_INFINITY);
        assert_eq!(result, None, "Negative infinity should return None");
    }

    #[test]
    fn test_valid_values_always_some() {
        // All valid finite values should return Some
        let test_values = [-10000.0, -100.0, 0.0, 100.0, 5000.0, 1000000.0];
        for value in test_values {
            let result = elevation_to_terrain_rgb(value);
            assert!(
                result.is_some(),
                "Valid value {} should return Some, got None",
                value
            );
        }
    }
}
