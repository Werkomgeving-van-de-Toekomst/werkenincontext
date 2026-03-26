# 3D Buildings Enhancements - Usage Guide

## Overview

This implementation enhances the IOU Modern Data Verkenner with improved 3D buildings functionality, including:

- Dynamic building loading based on viewport
- Height-based color coding
- Click popups with building information
- WGS84 bbox support for easier integration

## Quick Start

### Starting the Application

```bash
cd /Users/marc/Projecten/iou-modern
cargo run
```

The application will start on the default port (check your `Cargo.toml` for the specific port).

### Accessing the 3D Buildings View

Navigate to the Data Verkenner page in your browser. The 3D buildings layer is now:

1. **Dynamically loaded** - Buildings load as you pan/zoom the map
2. **Color-coded by height** - Green (low), Yellow (medium), Red (high)
3. **Interactive** - Click any building to see its details

## API Endpoints

### Backend: 3D Buildings Proxy

**Endpoint:** `/api/buildings-3d`

**Parameters:**
- `bbox` (string): Bounding box in RD or WGS84 coordinates
  - RD format: `minX,minY,maxX,maxY`
  - WGS84 format: Automatically detected and converted
- `limit` (integer): Maximum buildings to return (default: 150)

**Example:**
```bash
curl "http://localhost:8080/api/buildings-3d?bbox=150000,470000,155000,475000&limit=100"
```

**Response Properties:**
Each building feature includes:
- `bag_id`: Unique BAG identifier (`identificatie`)
- `height`: Roof height (`b3_h_dak_max`)
- `ground_level`: Ground elevation (`b3_h_maaiveld`)
- `floors`: Number of floors (`b3_bouwlagen`)
- `construction_year`: Original construction year (`oorspronkelijkbouwjaar`)

## Color Coding Scheme

| Height Range | Color | Visual |
|--------------|-------|--------|
| 0 - 5 meters | Green | `#22c55e` |
| 5 - 15 meters | Yellow/Orange | `#eab308` |
| 15+ meters | Red | `#ef4444` |

## Popup Information

When clicking a building, the popup displays:
- **BAG ID**: Unique identifier
- **Height**: Maximum roof height in meters
- **Floors**: Number of building floors
- **Construction Year**: When the building was originally built
- **Ground Level**: Elevation at ground level

## Implementation Files

### Backend (Rust/Axum)
- `crates/iou-api/src/routes/buildings_3d.rs` - 3DBAG proxy with property extraction and WGS84 support

### Frontend (Rust/Dioxus)
- `crates/iou-frontend/src/components/density_heatmap.rs` - Heatmap overlay component
- `crates/iou-frontend/src/components/view_toggle.rs` - 2D/3D view toggle
- `crates/iou-frontend/src/pages/data_verkenner.rs` - Main map page with dynamic loading

### Configuration
- Coordinate conversion enabled via `proj` crate in `Cargo.toml`
- Building limit increased from 50 to 150 buildings per viewport

## Development Notes

### API Verification Results

The 3DBAG API provides:
- **Available**: `bag_id`, `construction_year`, height fields
- **Not Available**: Address information (would require separate BAG API)

### Coordinate Systems

- **Input**: Accepts both RD and WGS84 bbox coordinates
- **Backend**: Automatically converts WGS84 to RD for 3DBAG API
- **Detection**: bbox format auto-detected by coordinate values

## Testing

```bash
# Run all tests
cargo test

# Run API-specific tests
cargo test --package iou-api

# Run frontend tests
cargo test --package iou-frontend
```

## Next Steps

Potential future enhancements:
1. Add address lookup via separate BAG API
2. Implement building filtering by attributes
3. Add export functionality for building data
4. Performance optimization for dense urban areas
