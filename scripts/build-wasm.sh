#!/bin/bash
# Build script for IOU-Modern WASM frontend
#
# This script builds the WASM-compatible crates for the frontend.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "🔨 Building IOU-Modern for WASM..."
echo "Workspace: frontend/Cargo.toml (WASM-compatible only)"
echo ""

# Build iou-core with wasm feature
echo "Building iou-core..."
cargo build \
    --manifest-path frontend/Cargo.toml \
    --package iou-core \
    --target wasm32-unknown-unknown \
    --release

echo ""
echo "✅ Cargo build complete!"
echo ""
echo "🚀 Running Dioxus bundle..."
echo ""

# Run Dioxus bundler
dx build \
    --manifest-path frontend/Cargo.toml \
    --release

echo ""
echo "✨ Done! Frontend built in ./dist/"
