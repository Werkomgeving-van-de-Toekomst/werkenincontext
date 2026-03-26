# Coding Conventions

**Analysis Date:** 2026-03-08

## Naming Patterns

**Files:**
- Snake_case for Rust files and modules (e.g., `user_info.rs`, `api_client.rs`)
- PascalCase for component files (e.g., `AppCard.rs`, `Header.rs`)
- Directories follow lowercase with underscores (e.g., `src/components`, `src/pages`)

**Functions:**
- Public functions: snake_case with descriptive names (e.g., `fetch_context`, `add_entity`)
- Private functions: snake_case prefixed with underscore (e.g., `_fetch_json_impl`)
- Component functions: PascalCase matching file names (e.g., `fn Header()`, `fn AppCard()`)

**Variables:**
- Local variables: snake_case (e.g., `user`, `query`, `result`)
- Struct/enum fields: snake_case (e.g., `user_name`, `search_query`)
- Constants: SCREAMING_SNAKE_CASE (e.g., `API_BASE`)

**Types:**
- Structs: PascalCase (e.g., `AppState`, `UserInfo`)
- Enums: PascalCase (e.g., `Route`, `ReviewAction`)
- Generics: Single uppercase letters (e.g., `T`, `K`, `V`)

## Code Style

**Formatting:**
- Standard Rust formatting via `rustfmt` (workspace edition = "2024")
- Consistent indentation with 4 spaces
- Trailing commas in multi-line contexts
- 100 character line limit where possible

**Linting:**
- Clippy configured via workspace
- Common lint rules enforced:
  - `dead_code` warnings
  - `unused_variables` warnings
  - `unused_results` warnings
  - `unreachable_code` errors

## Import Organization

**Order:**
1. Standard library imports (e.g., `std::`, `uuid::`)
2. External crate imports (e.g., `dioxus::`, `serde::`, `tracing::`)
3. Internal module imports (e.g., `crate::`, `iou_core::`)

**Path Aliases:**
- Direct imports preferred over glob imports
- Module re-exports organized in `mod.rs` files
- Explicit `pub use` declarations for public interfaces

## Error Handling

**Patterns:**
- `Result<T, E>` for recoverable errors
- `Option<T>` for nullable values
- Custom error types with `thiserror` (e.g., `ConfigError`)
- `anyhow::Result` for high-level functions
- Early returns with `?` operator for error propagation

**Examples:**
```rust
// API error handling
pub async fn fetch_context(domain_id: &str) -> Result<ContextResponse, String> {
    let url = format!("{API_BASE}/context/{domain_id}");
    fetch_json(&url).await
}

// Result-based function
pub fn parse_data(input: &str) -> Result<Data, ParseError> {
    // ... parsing logic
}
```

## Logging

**Framework:** `tracing` with `tracing-subscriber`
**Patterns:**
- `tracing::info!` for general information
- `tracing::debug!` for detailed debugging
- `tracing::warn!` for warnings
- `tracing::error!` for errors
- Structured logging with fields

**Examples:**
```rust
tracing::info!("Starting IOU-Modern API server...");
tracing::info!("DuckDB initialized at: {}", config.database_path);
```

## Comments

**When to Comment:**
- Public API documentation (functions, structs, enums)
- Complex algorithm implementations
- Business logic explanations
- External API integrations
- TODO items (with author and date)

**JSDoc/TSDoc:**
- Comprehensive documentation for public APIs
- Include examples for complex functions
- Document all function parameters and return values
- Document error conditions

**Examples:**
```rust
/// Terrain-RGB encoding for MapLibre GL JS terrain tiles.
///
/// This module implements the Mapbox Terrain-RGB specification for encoding
/// elevation values into RGB pixel data. MapLibre uses this format to render
/// 3D terrain from raster elevation tiles.
///
/// # Encoding Formula
///
/// The standard encoding formula is:
/// ```text
/// elevation_meters = -10000 + ((R * 65536 + G * 256 + B) * 0.1)
/// ```
pub fn elevation_to_terrain_rgb(elevation_meters: f64) -> Option<(u8, u8, u8)> {
    // Implementation
}
```

## Function Design

**Size:**
- Prefer smaller, focused functions (< 50 lines)
- Functions should do one thing well
- Extract complex logic into helper functions

**Parameters:**
- Prefer parameter objects for >3 parameters
- Use trait bounds for generic constraints
- Default parameters where appropriate

**Return Values:**
- Prefer `Result` for fallible operations
- Use `Option` for nullable returns
- Return multiple values via tuples or structs

## Module Design

**Exports:**
- Explicit `pub` declarations
- Re-exports organized in `mod.rs`
- Private implementations in `_` prefixed files

**Barrel Files:**
- Re-export commonly used items
- Hide implementation details
- Provide clean public interfaces

**Examples:**
```rust
// mod.rs
pub use approval_actions::ApprovalActions;
pub use app_card::AppCard;
pub use audit_viewer::AuditTrailViewer;
```

---

*Convention analysis: 2026-03-08*