#!/bin/bash
# Generate AHN Terrain Tiles for Flevoland 3D Map
#
# This script downloads AHN data from PDOK and converts it to Terrain-RGB tiles
# suitable for MapLibre GL JS 3D terrain visualization.
#
# Requirements:
#   - gdal (gdal-bin, gdal-dev)
#   - Python 3 with pip
#   - rio-rgbify: pip install rio-rgbify
#   - gdal2tiles.py (comes with gdal)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TILE_DIR="$PROJECT_ROOT/crates/iou-frontend/static/terrain"
TEMP_DIR="$PROJECT_ROOT/tmp/ahn"

# Flevoland bounding box (approximate)
# West: 5.0, East: 6.0, South: 52.0, North: 52.8
BOUNDS="5.0 52.0 6.0 52.8"

echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}AHN Terrain Tiles Generator${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo ""
echo "This script will:"
echo "  1. Download AHN data for Flevoland from PDOK"
echo "  2. Convert to Terrain-RGB format"
echo "  3. Generate map tiles for 3D visualization"
echo ""

# Check dependencies
echo -e "${YELLOW}Checking dependencies...${NC}"

if ! command -v gdalwarp &> /dev/null; then
    echo -e "${RED}✗ gdalwarp not found. Install with: brew install gdal${NC}"
    exit 1
fi
echo -e "${GREEN}✓ gdal found${NC}"

if ! command -v python3 &> /dev/null; then
    echo -e "${RED}✗ python3 not found${NC}"
    exit 1
fi
echo -e "${GREEN}✓ python3 found${NC}"

if ! python3 -c "import rio_rgbify" 2>/dev/null; then
    echo -e "${YELLOW}⚠ rio-rgbify not found. Install with: pip3 install rio-rgbify${NC}"
    echo -e "${YELLOW}  Continuing anyway (will fail if not installed)${NC}"
else
    echo -e "${GREEN}✓ rio-rgbify found${NC}"
fi

echo ""

# Create directories
mkdir -p "$TILE_DIR"
mkdir -p "$TEMP_DIR"

# Step 1: Download AHN data
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 1: Download AHN Data${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""
echo "AHN data can be downloaded from:"
echo "  - https://www.pdok.nl/downloads (PDOK Download service)"
echo "  - https://www.ahn.nl/nl/download-ahn (AHN direct download)"
echo ""
echo "For Flevoland, you need the following AHN bladen:"
echo "  - 25F2 (Noordoost Flevoland)"
echo "  - 31F2 (Zuidoost Flevoland)"
echo "  - 31G1 (Zuidelijk Flevoland)"
echo "  - 25G1 (Noordelijk Flevoland)"
echo ""
echo "Download the GeoTIFF (DEM) files and place them in:"
echo "  $TEMP_DIR"
echo ""

# Check if we have any GeoTIFF files
if ls "$TEMP_DIR"/*.tif 2>/dev/null | grep -q .; then
    echo -e "${GREEN}✓ Found $(ls -1 "$TEMP_DIR"/*.tif 2>/dev/null | wc -l) GeoTIFF files${NC}"
else
    echo -e "${YELLOW}⚠ No GeoTIFF files found in $TEMP_DIR${NC}"
    echo ""
    echo -e "${YELLOW}To download AHN data manually:${NC}"
    echo "  1. Visit: https://www.pdok.nl/downloads"
    echo "  2. Select: Actueel Hoogtebestand Nederland (AHN)"
    echo "  3. Download the relevant bladen for Flevoland"
    echo "  4. Extract and place .tif files in: $TEMP_DIR"
    echo ""
    exit 1
fi

# Step 2: Merge and clip to Flevoland bounds
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 2: Merge and Clip AHN Data${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""

# Create a virtual raster from all input files
echo "Creating virtual raster..."
gdalbuildvrt -overwrite "$TEMP_DIR/ahn_merge.vrt" "$TEMP_DIR"/*.tif

# Clip to bounds and warp to Web Mercator
echo "Warping to Web Mercator (EPSG:3857)..."
gdalwarp \
    -t_srs EPSG:3857 \
    -te -556597.45 6808894.06 -444823.24 6952604.20 \
    -tr 10 10 \
    -r bilinear \
    "$TEMP_DIR/ahn_merge.vrt" \
    "$TEMP_DIR/ahn_flevoland.tif"

echo -e "${GREEN}✓ AHN data prepared${NC}"
echo ""

# Step 3: Convert to Terrain-RGB
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 3: Convert to Terrain-RGB Format${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""

# Use rio-rgbify to convert elevation to Terrain-RGB
rio rgbify \
    --output-format GTiff \
    --min-elevation -10 \
    --max-elevation 10 \
    "$TEMP_DIR/ahn_flevoland.tif" \
    "$TEMP_DIR/ahn_flevoland_rgb.tif"

echo -e "${GREEN}✓ Terrain-RGB conversion complete${NC}"
echo ""

# Step 4: Generate tiles
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 4: Generate Map Tiles${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""

# Clear existing tiles
rm -rf "$TILE_DIR"/*
mkdir -p "$TILE_DIR"

# Generate tiles using gdal2tiles
gdal2tiles.py \
    --zoom=8-14 \
    --webviewer=none \
    --processes=4 \
    "$TEMP_DIR/ahn_flevoland_rgb.tif" \
    "$TILE_DIR"

echo -e "${GREEN}✓ Tiles generated${NC}"
echo ""

# Create tilejson
cat > "$TILE_DIR/tilejson.json" << EOF
{
  "tiles": ["/static/terrain/{z}/{x}/{y}.png"],
  "minzoom": 8,
  "maxzoom": 14,
  "attribution": "AHN - Actueel Hoogtebestand Nederland",
  "name": "ahn-terrain",
  "format": "png"
}
EOF

echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Setup Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo ""
echo "Terrain tiles generated at: $TILE_DIR"
echo ""
echo "To use in Map3D component, the tiles are served at:"
echo "  /static/terrain/{z}/{x}/{y}.png"
echo ""
echo "Clean up temp files with:"
echo "  rm -rf $TEMP_DIR"
echo ""
