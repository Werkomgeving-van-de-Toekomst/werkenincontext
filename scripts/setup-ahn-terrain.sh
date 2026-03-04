#!/bin/bash
# Setup AHN Terrain Tiles for Flevoland 3D Map
#
# This script creates terrain tiles from elevation data.
# Since PDOK WMS is unreliable, we use AWS Terrain Tiles instead.

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

echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Terrain Tiles Setup${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo ""
echo "Tile directory: $TILE_DIR"
echo ""

# Create directories
mkdir -p "$TILE_DIR"
mkdir -p "$TEMP_DIR"

echo -e "${YELLOW}Creating terrain tile configuration...${NC}"
echo ""

# AWS Terrain Tiles - gratis, geen API key nodig
# We gebruiken een tile JSON configuratie die direct naar AWS wijst

cat > "$TILE_DIR/tilejson.json" << 'EOF'
{
  "tiles": [
    "https://elevation-tiles-prod.s3.amazonaws.com/v1/terrarium/{z}/{x}/{y}.png"
  ],
  "minzoom": 0,
  "maxzoom": 14,
  "attribution": "AWS Open Data Terrain Tiles",
  "name": "terrain",
  "format": "png"
}
EOF

echo -e "${GREEN}✓ Tile configuration created${NC}"
echo ""

# Create a simple terrain RGB decoder note
cat > "$TILE_DIR/README.md" << 'EOF'
# Terrain Tiles

These tiles use AWS Open Data terrain tiles in Terrarium format.

## Encoding

AWS uses Terrarium encoding:
- R: (height - floor(height)) * 256
- G: floor(height) / 256
- B: floor(height) % 256

## MapLibre Configuration

Use this configuration for MapLibre:

```javascript
map.addSource('terrain', {
  type: 'raster-dem',
  tiles: ['https://elevation-tiles-prod.s3.amazonaws.com/v1/terrarium/{z}/{x}/{y}.png'],
  tileSize: 256,
  attribution: 'AWS Open Data'
});
```

## Note

For Flevoland-specific AHN data with higher accuracy, download from:
- https://www.ahn.nl/nl/download-ahn
- Extract and convert using gdal2tiles.py
EOF

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Option 1: AWS Terrain Tiles (Recommended)${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""
echo "✓ Configuration created for AWS Terrain Tiles"
echo "  - No API key required"
echo "  - Global coverage"
echo "  - Works immediately"
echo ""

echo -e "${YELLOW}Update the map_3d.rs to use AWS tiles:${NC}"
echo ""
echo '  terrain_local_path: "https://elevation-tiles-prod.s3.amazonaws.com/v1/terrarium/{z}/{x}/{y}.png"'
echo ""

# Update the terrain path in the Rust config
echo -e "${YELLOW}Updating map_3d.rs configuration...${NC}"

sed -i.bak 's|/static/terrain/{z}/{x}/{y}.png|https://elevation-tiles-prod.s3.amazonaws.com/v1/terrarium/{z}/{x}/{y}.png|g' \
  "$PROJECT_ROOT/crates/iou-frontend/src/components/map_3d.rs"

rm -f "$PROJECT_ROOT/crates/iou-frontend/src/components/map_3d.rs.bak"

echo -e "${GREEN}✓ Configuration updated${NC}"
echo ""

echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Setup Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo ""
echo "The 3D map now uses AWS Terrain Tiles (no API key needed)."
echo ""
echo "To test:"
echo "  export MAP_3D_ENABLED=true"
echo "  cargo run"
echo ""
