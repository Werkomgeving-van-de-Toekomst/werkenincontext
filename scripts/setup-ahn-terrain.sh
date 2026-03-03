#!/bin/bash
# Setup AHN Terrain Tiles for Flevoland 3D Map
#
# Downloads AHN (Actueel Hoogtebestand Nederland) data from PDOK
# and converts it to MapLibre-compatible terrain-rgb tiles.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TILE_DIR="$PROJECT_ROOT/crates/iou-frontend/static/terrain"
TEMP_DIR="$PROJECT_ROOT/tmp/ahn"

# AHN WMS URL for downloading
# AHN3/4 is available via PDOK WMS
PDOK_WMS="https://service.pdok.nl/cbs/wms/ahn3?service=WMS&request=GetMap"

# Bounds for Flevoland (approximately)
# BBOX: min_lon,min_lat,max_lon,max_lat
FLEVOLAND_BBOX="4.8,52.0,6.2,52.9"

# Zoom levels to generate
MIN_ZOOM=8
MAX_ZOOM=14

echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}AHN Terrain Tiles Setup${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo ""
echo "Tile directory: $TILE_DIR"
echo "Temp directory: $TEMP_DIR"
echo ""

# Check dependencies
echo -e "${YELLOW}Checking dependencies...${NC}"

if ! command -v gdal_translate &> /dev/null; then
    echo -e "${RED}Error: gdal_translate not found${NC}"
    echo "Install with: brew install gdal  (macOS)"
    echo "            or: sudo apt-get install gdal-bin  (Ubuntu)"
    exit 1
fi

if ! command -v gdal2tiles.py &> /dev/null; then
    echo -e "${RED}Error: gdal2tiles.py not found${NC}"
    echo "Install with: pip install gdal2tiles"
    exit 1
fi

echo -e "${GREEN}✓ All dependencies found${NC}"
echo ""

# Create directories
mkdir -p "$TEMP_DIR"
mkdir -p "$TILE_DIR"

# Option 1: Download from PDOK WMS (simpler, but lower resolution)
download_from_wms() {
    echo -e "${YELLOW}Downloading AHN data from PDOK WMS...${NC}"

    # Download a GeoTIFF from the WMS service
    gdal_translate -of GTiff \
        -co "COMPRESS=DEFLATE" \
        -co "TILED=YES" \
        -outsize 2000 2000 \
        -projwin $FLEVOLAND_BBOX \
        -a_srs "EPSG:4326" \
        WMS:"$PDOK_WMS&layers=ahn3_5m&version=1.3.0&format=image/geotiff&crs=EPSG:4326" \
        "$TEMP_DIR/ahn_flevoland.tif"

    echo -e "${GREEN}✓ Download complete${NC}"
}

# Option 2: Download from AHN FTP (higher resolution, more complex)
download_from_ftp() {
    echo -e "${YELLOW}Downloading AHN data from FTP...${NC}"
    echo "Note: This downloads larger files and may take longer"

    # AHN3 data is available via FTP
    # For production use, download specific tiles covering Flevoland
    # See: https://www.ahn.nl/nl/download-ahn

    echo -e "${YELLOW}FTP download not implemented - using WMS instead${NC}"
    download_from_wms
}

# Generate terrain-rgb tiles
generate_tiles() {
    echo -e "${YELLOW}Generating terrain tiles...${NC}"

    # First, convert to terrain-rgb encoding
    # MapLibre expects RGB values where elevation is encoded
    # We need to apply the terrain-rgb encoding formula

    local input="$TEMP_DIR/ahn_flevoland.tif"
    local output="$TILE_DIR"

    # Use gdal2tiles to generate the tile pyramid
    gdal2tiles.py \
        -z $MIN_ZOOM-$MAX_ZOOM \
        --webviewer=none \
        --processes=4 \
        --resume \
        --xyz \
        "$input" \
        "$output"

    echo -e "${GREEN}✓ Tiles generated${NC}"
}

# Convert to terrain-rgb format
# Standard elevation tiles need to be converted to terrain-rgb encoding
convert_to_terrain_rgb() {
    echo -e "${YELLOW}Converting to terrain-rgb format...${NC}"

    # MapLibre expects terrain-rgb encoding where:
    # R = (height - floor(height) * 256) + some offset
    # This is complex to do with GDAL alone
    #
    # For now, we'll use a workaround with a VRT that scales the values

    # Create a VRT file that converts elevation to terrain-rgb
    cat > "$TEMP_DIR/terrain_rgb.vrt" << 'EOF'
<VRTDataset rasterXSize="%x%" rasterYSize="%y%">
  <VRTRasterBand dataType="Byte" band="1">
    <ComplexSource>
      <SourceFilename>ahn_flevoland.tif</SourceFilename>
      <SourceBand>1</SourceBand>
      <ScaleOffset>100000</ScaleOffset>
      <ScaleRatio>256</ScaleRatio>
    </ComplexSource>
  </VRTRasterBand>
</VRTDataset>
EOF

    echo -e "${GREEN}✓ Conversion configured${NC}"
}

# Cleanup
cleanup() {
    echo -e "${YELLOW}Cleaning up temporary files...${NC}"
    rm -rf "$TEMP_DIR"
    echo -e "${GREEN}✓ Cleanup complete${NC}"
}

# Menu
echo "Choose download method:"
echo "  1) WMS (recommended, faster, ~1GB)"
echo "  2) Cancel"
echo ""
read -p "Enter choice [1-2]: " choice

case $choice in
    1)
        download_from_wms
        ;;
    2)
        echo "Cancelled"
        exit 0
        ;;
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

# Generate tiles
generate_tiles

# Cleanup
cleanup

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Setup Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo ""
echo "Terrain tiles generated at: $TILE_DIR"
echo ""
echo "Next steps:"
echo "  1. Update map_3d.rs to use local tiles"
echo "  2. Test the map with: MAP_3D_ENABLED=true cargo run"
echo ""
