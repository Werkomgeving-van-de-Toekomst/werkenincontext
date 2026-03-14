# 3D Buildings Enhancements

## Overview
Enhance the existing 3D buildings layer in the IOU Modern Data Verkenner with improved functionality.

## Requirements

### 1. Load More Buildings
- Current limit is 100 buildings
- Increase limit to show more buildings in the viewport
- Consider performance implications

### 2. Dynamic BBox Based on Map Position
- Instead of fixed bbox, calculate bbox based on current map viewport
- Load buildings dynamically as user pans/zooms the map
- Debounce requests to avoid API overload

### 3. Color Coding by Building Height
- Color buildings based on their height
- Create a visual legend for the height ranges
- Example color scheme:
  - Low (0-5m): Green
  - Medium (5-15m): Yellow/Orange
  - High (15m+): Red

### 4. Click Popup with Building Info
- Show popup when clicking on a building
- Display building properties:
  - Building ID (NL.IMBAG.Pand.*)
  - Height (in meters)
  - Number of floors
  - Ground level elevation

## Context
- Frontend: Rust/Dioxus with MapLibre GL JS
- Backend: Rust/Axum with 3DBAG proxy endpoint
- Current endpoint: `/api/buildings-3d?bbox=...&limit=...`
- Buildings are returned as GeoJSON FeatureCollection
