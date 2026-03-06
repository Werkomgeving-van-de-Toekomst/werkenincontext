//! Terrain tile serving with TMS/XYZ coordinate conversion
//!
//! AHN terrain tiles use TMS coordinates (y from bottom),
//! but MapLibre requests XYZ coordinates (y from top).
//! This handler converts between the two schemes.

use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use std::path::PathBuf;

/// Get the full path to the static directory
fn static_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap()
        .join("static")
        .join("terrain")
}

/// Parse tile coordinates from path string "z/x/y.png"
fn parse_tile_coords(s: &str) -> Result<(u32, u32, u32), StatusCode> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 3 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let z: u32 = parts[0].parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let x: u32 = parts[1].parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    // Remove .png extension from y
    let y_str = parts[2].strip_suffix(".png")
        .ok_or(StatusCode::BAD_REQUEST)?;
    let y: u32 = y_str.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok((z, x, y))
}

/// Serve a terrain tile with TMS/XYZ coordinate conversion
///
/// Converts XYZ tile coordinates (y from top) to TMS (y from bottom)
/// to match AHN tile storage format.
pub async fn get_terrain_tile(
    Path(tile_path): Path<String>,
) -> Result<Response, StatusCode> {
    let (z, x, y) = parse_tile_coords(&tile_path)?;

    let max_y = (1 << z) - 1; // 2^zoom - 1
    let y_tms = max_y - y;     // Convert XYZ to TMS

    let tile_file = static_dir()
        .join(z.to_string())
        .join(x.to_string())
        .join(format!("{}.png", y_tms));

    // Return blank 1x1 PNG if tile doesn't exist
    let data = if tile_file.exists() {
        tokio::fs::read(&tile_file)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        // 1x1 transparent PNG (no elevation)
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk start
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 pixels
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, // 8-bit grayscale
            0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, // IDAT chunk
            0x54, 0x08, 0x1D, 0x01, 0x00, 0x00, 0xFF, 0xFF,
            0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, // IEND
        ]
    };

    Ok((
        [(header::CONTENT_TYPE, "image/png")],
        data,
    ).into_response())
}

/// Serve the tilejson.json file for terrain tiles
pub async fn get_tilejson() -> Result<Response, StatusCode> {
    let tilejson_path = static_dir().join("tilejson.json");

    if !tilejson_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let content = tokio::fs::read_to_string(&tilejson_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        [(header::CONTENT_TYPE, "application/json")],
        content,
    ).into_response())
}
