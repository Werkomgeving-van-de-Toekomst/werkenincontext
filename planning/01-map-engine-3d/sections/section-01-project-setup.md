Now I have all the context needed. Let me generate the section content for `section-01-project-setup`.

Looking at the section manifest and implementation plan, `section-01-project-setup` is about:
1. Creating new component files (map_3d.rs, layer_control_3d.rs)
2. Updating components/mod.rs exports
3. Adding MapLibre CSS/JS resources to Dioxus.toml

From the TDD plan, the tests for Step 1 (Project Setup) are:
- Test: mod.rs exports Map3D component
- Test: mod.rs exports LayerControl3D component

Let me generate the self-contained section content:

---

# Section 1: Project Setup

## Overview

This section establishes the foundational files and configuration needed for the 3D map engine integration. We create placeholder component files and configure external resource loading (MapLibre GL JS CSS and JavaScript).

**Dependencies:** None

**Blocked by:** None

**Blocks:** section-02-config-structures, section-03-terrain-encoding, section-04-map3d-component

## Files to Create/Modify

### 1. New File: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/map_3d.rs`

Create a new component file that will eventually contain the Map3D Dioxus component. For now, create a stub with module documentation.

**Stub content:**

```rust
//! Map3D Component - MapLibre GL JS wrapper
//!
//! This component provides a 3D-capable map interface using MapLibre GL JS.
//! It renders terrain elevation data and GeoJSON layers with 3D navigation
//! support (pitch, rotate, zoom).

// Placeholder - implementation in section-04-map3d-component
```

### 2. New File: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/layer_control_3d.rs`

Create a new component file for the layer control UI. For now, create a stub with module documentation.

**Stub content:**

```rust
//! LayerControl3D Component - Layer toggle control for Map3D
//!
//! This component provides a UI for toggling GeoJSON layer visibility
//! on the 3D map. Each layer has a checkbox and styled label.

// Placeholder - implementation in section-07-layer-control
```

### 3. Modify: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/mod.rs`

Update the module exports to include the new components. First, read the existing file to understand its current structure, then add the new exports.

**Add to mod.rs:**

```rust
// 3D Map Components (Map Engine 3D Upgrade)
pub mod map_3d;
pub mod layer_control_3d;

// Re-export main components for convenience
pub use map_3d::Map3D;
pub use layer_control_3d::LayerControl3D;
```

Note: The actual `pub use` lines should be added after the components are implemented in later sections. For this setup step, only the `pub mod` declarations are strictly necessary.

### 4. Modify: `/Users/marc/Projecten/iou-modern/Cargo.toml` (web resources)

The Dioxus framework loads external CSS/JS resources through its configuration. Add MapLibre GL JS resources to the web resource configuration.

**Add to the appropriate Dioxus configuration file:**

```toml
[web.resource]
# MapLibre GL JS - Version 5.0.0
style = [
    "https://unpkg.com/maplibre-gl@5.0.0/dist/maplibre-gl.css",
]
script = [
    "https://unpkg.com/maplibre-gl@5.0.0/dist/maplibre-gl.js",
]
```

**Note:** The exact location of this configuration depends on the Dioxus project structure. It may be in `Cargo.toml` under `[package.metadata.dioxus]` or in a separate `Dioxus.toml` file. Verify the existing project structure before modifying.

## Tests

Write these tests BEFORE implementing any functionality.

### Test File: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/mod_test.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_exports_map3d_module() {
        // Verify the map_3d module is accessible
        // This will compile successfully if pub mod map_3d; exists
        let _ = map_3d;
    }

    #[test]
    fn test_mod_exports_layer_control_3d_module() {
        // Verify the layer_control_3d module is accessible
        // This will compile successfully if pub mod layer_control_3d; exists
        let _ = layer_control_3d;
    }
}
```

**Expected behavior:**
- Tests should compile and pass after stub files are created and mod.rs is updated
- These tests verify module structure, not component functionality

## Verification Checklist

After completing this section:

- [ ] `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/map_3d.rs` exists with module documentation
- [ ] `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/layer_control_3d.rs` exists with module documentation
- [ ] `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/mod.rs` exports both new modules
- [ ] MapLibre CSS is configured in Dioxus web resources
- [ ] MapLibre JS is configured in Dioxus web resources
- [ ] `cargo test` passes without errors
- [ ] `cargo build --target wasm32-unknown-unknown` succeeds

## Next Steps

After completing this section:
1. Proceed to `section-02-config-structures` to define configuration types
2. Or proceed to `section-03-terrain-encoding` (parallelizable, no dependencies)

## Notes

- MapLibre GL JS version is pinned to 5.0.0 for stability
- The unpkg CDN is used as the default CDN source
- Component implementations are deferred to later sections
- This section is intentionally minimal to establish structure without premature implementation