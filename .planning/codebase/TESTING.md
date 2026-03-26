# Testing Patterns

**Analysis Date:** 2026-03-08

## Test Framework

**Runner:**
- Rust built-in test runner (`#[test]`)
- No external test framework detected
- Config via Cargo.toml dev-dependencies

**Assertion Library:**
- `assert_eq!`, `assert_ne!`, `assert!` macros
- `approx` crate for floating-point comparisons
- `serial_test` for integration tests with sequential execution

**Run Commands:**
```bash
cargo test              # Run all tests
cargo test --release    # Run tests in release mode
cargo test <name>       # Run specific test
cargo test -- --nocapture # Show test output
```

## Test File Organization

**Location:**
- Unit tests co-located with production code (inline `#[cfg(test)]` modules)
- Integration tests in separate files when needed
- Test files follow same naming convention as production code

**Naming:**
- Test functions: snake_case with `test_` prefix (e.g., `test_elevation_to_terrain_rgb_minimum`)
- Test modules: `mod tests` within each file
- Integration tests: function names describe the scenario

**Structure:**
```
src/
├── components/
│   ├── terrain_encoding.rs
│   └── mod.rs
└── pages/
    └── data_verkenner.rs
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_elevation_to_terrain_rgb_minimum() {
        // Arrange
        let elevation = -10000.0;

        // Act
        let result = elevation_to_terrain_rgb(elevation);

        // Assert
        assert_eq!(result, Some((0, 0, 0)));
    }
}
```

**Patterns:**
- Arrange-Act-Assert (AAA) pattern commonly used
- Helper functions for test data creation
- Test data constants defined at module level

## Mocking

**Framework:** No dedicated mocking framework
**Patterns:**
- Manual implementation of test doubles
- Interface design via traits for testability
- Conditional compilation for platform-specific code

**Examples:**
```rust
// Platform-specific implementations tested separately
#[cfg(target_arch = "wasm32")]
async fn fetch_json_impl<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    // WASM implementation
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
async fn fetch_json_impl<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    // Desktop implementation
}
```

**What to Mock:**
- External HTTP requests
- Database connections
- File system operations
- Time-dependent functions

**What NOT to Mock:**
- Simple utility functions
- Data structures
- Core business logic
- Pure functions

## Fixtures and Factories

**Test Data:**
```rust
fn create_test_document(content: &str) -> GeneratedDocument {
    GeneratedDocument {
        document_id: Uuid::new_v4(),
        content: content.to_string(),
        variables: vec![],
        entity_links: vec![],
        sections: vec![],
        generated_at: Utc::now(),
    }
}

fn create_test_compliance(score: f32) -> ComplianceResult {
    ComplianceResult {
        is_compliant: score >= 0.8,
        score,
        refusal_grounds: vec![],
        pii_detected: vec![],
        accessibility_issues: vec![],
        issues: vec![],
        redacted_content: None,
        assessed_at: Utc::now(),
        original_storage_key: None,
        redacted_storage_key: None,
    }
}
```

**Location:**
- Test helper functions in `#[cfg(test)]` modules
- Common test utilities at module level

## Coverage

**Requirements:** Not explicitly enforced
**View Coverage:**
```bash
cargo test -- --nocapture  # Manual inspection
# No coverage tool detected in project
```

## Test Types

**Unit Tests:**
- Scope: Individual functions and modules
- Location: Inline with production code
- Focus: Logic correctness, edge cases, error handling
- Examples: Terrain encoding functions, component rendering

**Integration Tests:**
- Scope: Multiple modules working together
- Location: Separate files or `#[tokio::test]` async functions
- Focus: API endpoints, database operations, async workflows
- Examples: Review agent workflow, document generation pipeline

**E2E Tests:**
- Framework: Not detected
- Implementation: Manual test scripts or external tools

## Common Patterns

**Async Testing:**
```rust
#[tokio::test]
async fn test_review_agent_completeness_check_pass() {
    let document = create_test_document("# Test\n\n## Introduction\n\nContent.\n\n## Conclusion\n\nEnd.");
    let research = ResearchContext {
        mandatory_sections: vec!["Introduction".to_string(), "Conclusion".to_string()],
        ..create_test_research()
    };
    let compliance = create_test_compliance(0.9);

    let result = execute_review_agent(&document, &compliance, &research).await.unwrap();

    assert!(result.completeness_issues.is_empty());
}
```

**Error Testing:**
```rust
#[test]
fn test_nan_returns_none() {
    let result = elevation_to_terrain_rgb(f64::NAN);
    assert_eq!(result, None, "NaN should return None");
}
```

**Property Testing:**
- No property testing framework detected
- Manual property-based testing with loops

---

*Testing analysis: 2026-03-08*