Now I have all the context. Let me generate the section content for `section-03-backend-wgs84`.

# Section 3: Backend WGS84 Bbox Endpoint

## Overview

This section adds a new API endpoint that accepts WGS84 bbox parameters directly. The frontend can pass map bounds (in WGS84) to the backend, which then converts them to RD coordinates before calling the 3DBAG API. This approach keeps accurate coordinate conversion on the server and simplifies frontend code.

**Dependencies:**
- `section-02-backend-coordinate` must be completed first (enables proj crate for accurate conversion)

**Files to modify:**
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs`

## Tests First

Add the following tests to `buildings_3d.rs` before implementing:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wgs84_bbox_parameter_is_correctly_parsed() {
        // Verify that bbox-wgs84 query parameter is parsed correctly
        // Format: "min_lon,min_lat,max_lon,max_lat"
    }

    #[test]
    fn test_wgs84_bbox_is_converted_to_rd_for_3dbag_api_call() {
        // Verify that WGS84 coordinates are converted to RD
        // using the proj crate before calling 3DBAG
    }

    #[test]
    fn test_limit_parameter_is_respected_with_wgs84_endpoint() {
        // Verify max limit of 150 is enforced
    }

    #[test]
    fn test_invalid_wgs84_coordinates_return_error() {
        // Verify that out-of-bounds coordinates return appropriate error
        // (coordinates outside Netherlands)
    }

    #[test]
    fn test_response_format_is_valid_geojson_feature_collection() {
        // Verify output format matches existing endpoint
    }
}
```

## Implementation

### 1. Add WGS84 Bbox Parameter Support

Modify the buildings_3d handler to accept an alternative `bbox-wgs84` parameter:

**Current endpoint behavior:**
- Accepts RD coordinates: `/api/buildings-3d?bbox=150000,470000,170000,490000&limit=100`

**New behavior to add:**
- Accepts WGS84 coordinates: `/api/buildings-3d?bbox-wgs84=4.5,52.0,5.0,52.5&limit=150`
- Converts WGS84 to RD on backend using proj crate
- Falls back to RD bbox if WGS84 not provided (backward compatibility)

### 2. Coordinate Conversion Function

Add or update the conversion function to handle WGS84 input:

```rust
// In buildings_3d.rs

use proj::Proj;

/// Convert WGS84 bbox to RD (Rijksdriehoeksmeting) coordinates
fn convert_wgs84_to_rd_bbox(min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64)
    -> Result<(f64, f64, f64, f64), ApiError>
{
    let proj = Proj::new_known_crs("EPSG:4326", "EPSG:28992", None)
        .map_err(|e| ApiError::ProjectionError(e.to_string()))?;

    let (min_x, min_y) = proj.convert((min_lon, min_lat))
        .map_err(|e| ApiError::ProjectionError(e.to_string()))?;
    let (max_x, max_y) = proj.convert((max_lon, max_lat))
        .map_err(|e| ApiError::ProjectionError(e.to_string()))?;

    Ok((min_x, min_y, max_x, max_y))
}
```

### 3. Update Handler Logic

Modify the handler to detect and process WGS84 bbox:

```rust
// Handler pseudo-code structure

pub async fn buildings_3d_handler(
    Query(params): Query<BuildingsParams>,
) -> Result<Json<GeoJson>, ApiError> {
    let bbox = if let Some(wgs84_bbox) = params.bbox_wgs84 {
        // Parse WGS84 bbox string: "min_lon,min_lat,max_lon,max_lat"
        let coords: Vec<f64> = wgs84_bbox
            .split(',')
            .map(|s| s.parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ApiError::InvalidBbox)?;

        if coords.len() != 4 {
            return Err(ApiError::InvalidBbox);
        }

        // Convert to RD
        convert_wgs84_to_rd_bbox(coords[0], coords[1], coords[2], coords[3])?
    } else if let Some(rd_bbox) = params.bbox {
        // Use existing RD bbox parsing
        parse_rd_bbox(rd_bbox)?
    } else {
        return Err(ApiError::MissingBbox);
    };

    // Continue with existing 3DBAG API call logic...
}
```

### 4. Update Parameter Struct

Extend the query parameter struct:

```rust
#[derive(Deserialize)]
pub struct BuildingsParams {
    pub bbox: Option<String>,
    #[serde(rename = "bbox-wgs84")]
    pub bbox_wgs84: Option<String>,
    pub limit: Option<usize>,
}
```

## Error Handling

Add appropriate error types:

```rust
pub enum ApiError {
    MissingBbox,
    InvalidBbox,
    ProjectionError(String),
    // ... existing error types
}
```

Return appropriate HTTP status codes:
- `400 Bad Request` for invalid bbox format
- `400 Bad Request` for coordinates outside Netherlands
- `500 Internal Server Error` for projection failures

## Validation

Add basic coordinate validation for Netherlands bounds:

```rust
const NL_MIN_LON: f64 = 3.0;
const NL_MAX_LON: f64 = 7.5;
const NL_MIN_LAT: f64 = 50.5;
const NL_MAX_LAT: f64 = 53.5;

fn validate_wgs84_coords(lon: f64, lat: f64) -> bool {
    lon >= NL_MIN_LON && lon <= NL_MAX_LON &&
    lat >= NL_MIN_LAT && lat <= NL_MAX_LAT
}
```

## Testing Checklist

After implementation:

- [x] Test: `/api/buildings-3d?bbox-wgs84=5.0,52.0,5.5,52.5&limit=10` returns buildings
- [x] Test: Coordinates are accurately converted (buildings align with map position)
- [x] Test: Limit parameter is enforced
- [x] Test: Invalid bbox format returns 400 error
- [x] Test: Coordinates outside Netherlands return 400 error
- [x] Test: Existing RD bbox endpoint still works (backward compatibility)
- [x] Test: Both bbox and bbox-wgs84 present - prioritize one (document choice)

## Actual Implementation (2025-03-07)

**File Modified:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs`

**Changes Made:**

1. **ApiError enum** - Added with `IntoResponse` implementation for proper HTTP status codes:
   ```rust
   pub enum ApiError {
       MissingBbox,
       InvalidBbox(String),
       ProjectionError(String),
   }
   ```

2. **BboxParams struct** - Extended with `bbox_wgs84` field:
   ```rust
   #[serde(rename = "bbox-wgs84")]
   bbox_wgs84: Option<String>,
   ```

3. **wgs84_to_rd() function** - Added for WGS84 to RD conversion using proj crate

4. **parse_and_convert_wgs84_bbox() function** - Added with comprehensive validation:
   - Whitespace trimming (handles "4.5, 52.0, 5.0, 52.5" format)
   - NaN/Infinity validation
   - Bbox order validation (min < max)
   - Netherlands bounds validation

5. **get_buildings_3d() handler** - Updated to:
   - Return `Result<Json<_>, ApiError>` for proper error handling
   - Prioritize `bbox-wgs84` parameter over `bbox`
   - Enforce max limit of 150
   - Return HTTP 400 for invalid input

6. **Tests added** in `wgs84_bbox_tests` module:
   - `test_wgs84_bbox_parameter_is_correctly_parsed`
   - `test_wgs84_bbox_is_converted_to_rd_for_3dbag_api_call`
   - `test_invalid_wgs84_coordinates_return_error`
   - `test_netherlands_bounds_validation`
   - `test_wgs84_to_rd_conversion`
   - `test_wgs84_to_rd_returns_none_on_invalid`

**Code Review Fixes Applied:**
- Fixed bbox order validation (min < max check added)
- Implemented proper HTTP 400 errors via ApiError enum
- Added whitespace trimming for coordinate parsing
- Added NaN/Infinity value validation
- Strengthened test assertions with known reference points

## Next Section

After completing this section, proceed to:
- `section-04-backend-properties` - Extract additional building properties from CityJSON response