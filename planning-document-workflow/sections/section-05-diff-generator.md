Now I have all the context needed to generate the section content for `section-05-diff-generator`. Based on the index.md, this section covers:

> Text diff generation using the `similar` crate. Supports unified, side-by-side, and inline diff formats. Computes changes between document versions for visualization.

From the dependency graph, this section depends on sections 01 and 02, and blocks section 08.

Let me extract the relevant test and implementation details:

# Section 05: Diff Generator

## Overview

This section implements text diff generation for comparing document versions. The diff generator uses the `similar` crate to compute changes between two document versions and output them in three formats: unified (git-style), side-by-side (parallel view), and inline (highlighted changes).

The diff generator is a foundational component for the version history feature (section-08), enabling users to visualize what changed between document versions.

## Dependencies

This section depends on:
- **section-01-database-schema**: For the `document_versions` table with `diff_summary` column
- **section-02-core-types**: For core document and version types

This section enables:
- **section-08-version-storage**: Uses diff generation to populate `diff_summary` when creating versions

## Files to Create

| File | Purpose |
|------|---------|
| `/Users/marc/Projecten/iou-modern/crates/iou-core/src/diff/generator.rs` | Diff generation logic |
| `/Users/marc/Projecten/iou-modern/crates/iou-core/tests/diff/generator.rs` | Unit tests |

## Tests (Write First)

**Location:** `/Users/marc/Projecten/iou-modern/crates/iou-core/tests/diff/generator.rs`

Before implementing, write tests for:

### Basic Diff Structure Tests
- Test: `generate_diff` returns `DocumentDiff` with `from_version` and `to_version` populated
- Test: `generate_diff` sets the correct `format` field matching the input parameter

### Unified Format Tests
- Test: `unified_diff` produces git-style output with `+` prefix for additions
- Test: `unified_diff` produces git-style output with `-` prefix for deletions
- Test: `unified_diff` shows unchanged lines without prefix
- Test: `unified_diff` includes line numbers for old and new versions in header

### Side-by-Side Format Tests
- Test: `side_by_side_diff` aligns unchanged lines in parallel columns
- Test: `side_by_side_diff` shows additions (right column) and deletions (left column) in parallel
- Test: `side_by_side_diff` handles insertions with empty left column
- Test: `side_by_side_diff` handles deletions with empty right column

### Inline Format Tests
- Test: `inline_diff` wraps insertions in highlight markers (e.g., `<ins>...</ins>`)
- Test: `inline_diff` wraps deletions in highlight markers (e.g., `<del>...</del>`)
- Test: `inline_diff` shows unchanged text without markers

### DiffChange Enum Tests
- Test: `DiffChange::Unchanged` contains text present in both versions
- Test: `DiffChange::Inserted` contains text only in new version
- Test: `DiffChange::Deleted` contains text only in old version
- Test: `DiffChange::Replaced` contains both old and new text for inline format

### Edge Cases Tests
- Test: `generate_diff` handles empty `old_content` (all insertions)
- Test: `generate_diff` handles empty `new_content` (all deletions)
- Test: `generate_diff` handles identical content (no changes, empty changes vec)
- Test: `generate_diff` handles single-line changes
- Test: `generate_diff` handles multi-line changes with mixed operations

## Implementation

### Cargo Dependency

Add to `/Users/marc/Projecten/iou-modern/crates/iou-core/Cargo.toml`:

```toml
[dependencies]
similar = "2.4"
```

### Module Structure

Create `/Users/marc/Projecten/iou-modern/crates/iou-core/src/diff/mod.rs`:

```rust
pub mod generator;

pub use generator::{
    DiffGenerator,
    DiffFormat,
    DocumentDiff,
    DiffChange,
};
```

Update `/Users/marc/Projecten/iou-modern/crates/iou-core/src/lib.rs`:

```rust
pub mod diff;
```

### Core Types

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/diff/generator.rs`

```rust
use similar::{ChangeTag, TextDiff, TextDiffConfig};

/// Format for diff output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffFormat {
    /// Unified diff format (git-style, with + and - prefixes)
    Unified,
    /// Side-by-side diff format (two parallel columns)
    SideBySide,
    /// Inline diff format (changes highlighted inline)
    Inline,
}

/// A single change in the diff
#[derive(Debug, Clone, PartialEq)]
pub enum DiffChange {
    /// Text unchanged between versions
    Unchanged(String),
    /// Text inserted in the new version
    Inserted(String),
    /// Text deleted from the old version
    Deleted(String),
    /// Text replaced (both old and new for inline format)
    Replaced { old: String, new: String },
}

/// Complete diff between two document versions
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentDiff {
    /// Source version identifier
    pub from_version: String,
    /// Target version identifier
    pub to_version: String,
    /// Output format used
    pub format: DiffFormat,
    /// List of changes
    pub changes: Vec<DiffChange>,
}

/// Generates diffs between document versions
pub struct DiffGenerator;

impl DiffGenerator {
    /// Generate a diff between two text contents
    /// 
    /// # Arguments
    /// * `old_content` - The original/older version content
    /// * `new_content` - The new/current version content
    /// * `format` - The desired output format
    /// 
    /// # Returns
    /// A `DocumentDiff` containing all changes between the versions
    pub fn generate_diff(
        &self,
        old_content: &str,
        new_content: &str,
        format: DiffFormat,
    ) -> DocumentDiff {
        match format {
            DiffFormat::Unified => self.unified_diff(old_content, new_content),
            DiffFormat::SideBySide => self.side_by_side_diff(old_content, new_content),
            DiffFormat::Inline => self.inline_diff(old_content, new_content),
        }
    }
    
    /// Generate unified diff format (git-style)
    /// 
    /// Output uses `+` prefix for additions and `-` prefix for deletions.
    fn unified_diff(&self, old: &str, new: &str) -> DocumentDiff {
        let diff = TextDiff::from_lines(old, new);
        let mut changes = Vec::new();
        
        for op in diff.ops() {
            for change in diff.iter_changes(op) {
                let text = change.value().to_string();
                match change.tag() {
                    ChangeTag::Equal => {
                        changes.push(DiffChange::Unchanged(
                            text.strip_suffix('\n').unwrap_or(&text).to_string()
                        ));
                    }
                    ChangeTag::Insert => {
                        changes.push(DiffChange::Inserted(
                            text.strip_suffix('\n').unwrap_or(&text).to_string()
                        ));
                    }
                    ChangeTag::Delete => {
                        changes.push(DiffChange::Deleted(
                            text.strip_suffix('\n').unwrap_or(&text).to_string()
                        ));
                    }
                    ChangeTag::Replace => {
                        // In unified format, replace is treated as delete + insert
                        changes.push(DiffChange::Deleted(
                            text.strip_suffix('\n').unwrap_or(&text).to_string()
                        ));
                    }
                }
            }
        }
        
        DocumentDiff {
            from_version: "old".to_string(),
            to_version: "new".to_string(),
            format: DiffFormat::Unified,
            changes,
        }
    }
    
    /// Generate side-by-side diff format
    /// 
    /// Produces aligned changes suitable for parallel column display.
    fn side_by_side_diff(&self, old: &str, new: &str) -> DocumentDiff {
        let diff = TextDiff::from_lines(old, new);
        let mut changes = Vec::new();
        
        // Use similar's unified diff but structure for side-by-side rendering
        for op in diff.ops() {
            for change in diff.iter_changes(op) {
                let text = change.value().to_string();
                match change.tag() {
                    ChangeTag::Equal => {
                        changes.push(DiffChange::Unchanged(
                            text.strip_suffix('\n').unwrap_or(&text).to_string()
                        ));
                    }
                    ChangeTag::Insert => {
                        changes.push(DiffChange::Inserted(
                            text.strip_suffix('\n').unwrap_or(&text).to_string()
                        ));
                    }
                    ChangeTag::Delete => {
                        changes.push(DiffChange::Deleted(
                            text.strip_suffix('\n').unwrap_or(&text).to_string()
                        ));
                    }
                    ChangeTag::Replace => {
                        changes.push(DiffChange::Replaced {
                            old: text.strip_suffix('\n').unwrap_or(&text).to_string(),
                            new: String::new(), // Filled by corresponding insert
                        });
                    }
                }
            }
        }
        
        DocumentDiff {
            from_version: "old".to_string(),
            to_version: "new".to_string(),
            format: DiffFormat::SideBySide,
            changes,
        }
    }
    
    /// Generate inline diff format
    /// 
    /// Produces inline changes with highlight markers suitable for
    /// HTML rendering (e.g., `<ins>` and `<del>` tags).
    fn inline_diff(&self, old: &str, new: &str) -> DocumentDiff {
        let diff = TextDiff::from_chars(old, new);
        let mut changes = Vec::new();
        
        // Group consecutive changes for better readability
        let mut pending_delete = String::new();
        let mut pending_insert = String::new();
        
        for op in diff.ops() {
            for change in diff.iter_changes(op) {
                match change.tag() {
                    ChangeTag::Equal => {
                        // Flush any pending changes
                        if !pending_delete.is_empty() || !pending_insert.is_empty() {
                            if !pending_delete.is_empty() && !pending_insert.is_empty() {
                                changes.push(DiffChange::Replaced {
                                    old: pending_delete.clone(),
                                    new: pending_insert.clone(),
                                });
                            } else if !pending_delete.is_empty() {
                                changes.push(DiffChange::Deleted(pending_delete.clone()));
                            } else if !pending_insert.is_empty() {
                                changes.push(DiffChange::Inserted(pending_insert.clone()));
                            }
                            pending_delete.clear();
                            pending_insert.clear();
                        }
                        changes.push(DiffChange::Unchanged(change.value().to_string()));
                    }
                    ChangeTag::Delete => {
                        pending_delete.push_str(change.value());
                    }
                    ChangeTag::Insert => {
                        pending_insert.push_str(change.value());
                    }
                    ChangeTag::Replace => {
                        pending_delete.push_str(change.value());
                    }
                }
            }
        }
        
        // Flush remaining changes
        if !pending_delete.is_empty() || !pending_insert.is_empty() {
            if !pending_delete.is_empty() && !pending_insert.is_empty() {
                changes.push(DiffChange::Replaced {
                    old: pending_delete,
                    new: pending_insert,
                });
            } else if !pending_delete.is_empty() {
                changes.push(DiffChange::Deleted(pending_delete));
            } else if !pending_insert.is_empty() {
                changes.push(DiffChange::Inserted(pending_insert));
            }
        }
        
        DocumentDiff {
            from_version: "old".to_string(),
            to_version: "new".to_string(),
            format: DiffFormat::Inline,
            changes,
        }
    }
}

impl Default for DiffGenerator {
    fn default() -> Self {
        Self
    }
}
```

### Test Module Stub

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/tests/diff/generator.rs`

```rust
use iou_core::diff::{DiffGenerator, DiffChange, DiffFormat};
use similar::ChangeTag;

#[tokio::test]
async fn test_generate_diff_returns_version_fields() {
    let generator = DiffGenerator;
    let old = "line one\nline two";
    let new = "line one\nline two modified";
    
    let result = generator.generate_diff(old, new, DiffFormat::Unified);
    
    assert_eq!(result.from_version, "old");
    assert_eq!(result.to_version, "new");
    assert_eq!(result.format, DiffFormat::Unified);
}

// Additional tests as specified in the test section above...
```

## Integration Notes

The diff generator will be integrated with the version storage service (section-08) to:

1. **Auto-generate diff summaries**: When a new version is created, compute a summary of changes for storage in `document_versions.diff_summary`
2. **Support on-demand diff generation**: The API will fetch version content and generate diffs in the requested format
3. **Handle compressed versions**: The diff generator must work with decompressed content from S3/MinIO

## Success Criteria

The diff generator is complete when:

1. All unit tests pass
2. Unified diff output matches standard git-diff format for line-based changes
3. Side-by-side diff properly aligns unchanged lines and shows changes in parallel
4. Inline diff produces output suitable for HTML rendering with `<ins>` and `<del>` markers
5. Edge cases (empty content, identical content, single-line changes) are handled correctly
6. Performance is acceptable for documents up to ~10,000 lines (typical contract document size)

## API Contract for Future Integration

The diff generator will be used by the version storage service as follows:

```rust
// In version storage service (section-08)
use iou_core::diff::DiffGenerator;

pub async fn create_version_with_diff(
    &self,
    document_id: Uuid,
    old_content: Option<String>,  // Previous version content
    new_content: &str,
    // ... other parameters
) -> Result<DocumentVersion> {
    let generator = DiffGenerator;
    
    // Generate diff for summary
    let diff_summary = if let Some(old) = old_content {
        let diff = generator.generate_diff(&old, new_content, DiffFormat::Unified);
        Some(serde_json::to_value(diff)?)
    } else {
        None
    };
    
    // ... store version with diff_summary
}
```

---

## Implementation Notes (2026-03-24)

### Files Actually Created

| File | Status |
|------|--------|
| `crates/iou-core/src/diff/generator.rs` | ✅ Created (212 lines) |
| `crates/iou-core/src/diff/mod.rs` | ✅ Created (8 lines) |
| `crates/iou-core/tests/diff/generator.rs` | ✅ Created (293 lines, 20 tests) |
| `crates/iou-core/tests/diff/mod.rs` | ✅ Created (1 line) |
| `crates/iou-core/src/lib.rs` | ✅ Updated (added `pub mod diff;`) |
| `crates/iou-core/Cargo.toml` | ✅ Updated (added `similar = "2.4"`) |
| `crates/iou-core/tests/mod.rs` | ✅ Updated (added `mod diff;`) |

### Tests Implemented

All 20 tests pass:
- Basic structure: 2 tests
- Unified format: 3 tests
- Side-by-side format: 3 tests
- Inline format: 3 tests
- DiffChange enum: 4 tests
- Edge cases: 5 tests

### Deviations from Plan

1. **Line Numbers Not Implemented**: The plan mentioned line numbers in unified diff headers, but the `DocumentDiff` struct has no fields for them. Deferred to section-08 when actual version IDs exist.

2. **HTML Markers Not Implemented**: The plan said inline diff wraps in `<ins>`/`<del>` tags. The implementation returns structured `DiffChange` enum values instead - better separation of concerns. HTML wrapping is the caller's responsibility.

3. **Hardcoded Version Identifiers**: `from_version` and `to_version` are currently `"old"` and `"new"`. Section-08 will replace these with actual version UUIDs.

4. **Tests Use Synchronous `#[test]`**: Changed from `#[tokio::test]` to `#[test]` since `DiffGenerator` has no async operations (performance improvement).

### Code Review Decisions

- ✅ Fixed: Tests changed from async to sync for performance
- ✅ Fixed: Newline handling inconsistency in inline diff
- ⏭️ Deferred: Code duplication between `unified_diff` and `side_by_side_diff` (minor, acceptable for clarity)
- ⏭️ Deferred: Version identifiers to section-08 integration

### Commit

- **Branch**: main
- **Implementation Commit**: `26c8e96`
- **Documentation Commit**: `06ed1e7`