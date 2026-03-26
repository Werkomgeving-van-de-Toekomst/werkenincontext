# Research Findings: 3D Buildings Enhancements

## Codebase Context

IOU Modern is a Rust-based data exploration application:
- **Backend:** Rust with Axum web framework
- **Frontend:** Rust with Dioxus framework (compiles to WebAssembly for web)
- **Mapping:** MapLibre GL JS for 3D visualization
- **External API:** 3DBAG (Dutch 3D Building API)

## Key Files Identified

- `crates/iou-frontend/src/pages/data_verkenner.rs` - Map initialization, building fetch
- `crates/iou-frontend/src/components/map_3d.rs` - 3D map component with initialization
- `crates/iou-api/src/routes/buildings_3d.rs` - 3DBAG proxy, coordinate conversion

## Architecture Notes

**Current Flow:**
1. Frontend fetches from `/api/buildings-3d?bbox={RD}&limit={n}`
2. Backend proxies to 3DBAG API
3. Backend converts CityJSON to GeoJSON
4. Frontend renders via MapLibre GL JS

**Existing Issues:**
- `buildings_3d.rs` has `use_proj = false` - coordinate conversion uses simplified formula
- Duplicate map initialization paths between `data_verkenner.rs` and `map_3d.rs`

## Testing

### Testing Framework

**Rust Backend:**
- Standard Rust testing (`#[test]` and `#[tokio::test]`)
- Built-in assert macros
- No external testing frameworks

**Frontend (Dioxus):**
- `approx = "0.5"` for floating point comparison
- `serial_test = "3"` for sequential test execution

### Test Conventions

**Location:** Tests embedded in source files using `#[cfg(test)]` modules
- Pattern: `mod tests { ... }` at end of each `.rs` file
- No separate `tests/` directories

**Naming:** `test_{feature}` or `{feature}_test` pattern
- Async tests use `#[tokio::test]`
- Test utilities prefixed with `create_test_`

### Running Tests

```bash
cargo test                    # All tests
cargo test -p iou-api         # API crate tests
cargo test -p iou-frontend    # Frontend crate tests
```

### Test Utilities Found

- `create_test_request()` - Creates test HTTP requests
- `create_test_template()` - Creates test templates
- Helper functions embedded in test modules

### Notable Gaps

- No integration test suite
- No frontend component testing framework
- Limited CI/CD test automation
- No mocking frameworks detected
