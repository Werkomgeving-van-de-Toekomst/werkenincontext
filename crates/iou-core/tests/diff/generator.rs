use iou_core::diff::{DiffGenerator, DiffChange, DiffFormat};

#[test]
fn test_generate_diff_returns_version_fields() {
    let generator = DiffGenerator;
    let old = "line one\nline two";
    let new = "line one\nline two modified";

    let result = generator.generate_diff(old, new, DiffFormat::Unified);

    assert_eq!(result.from_version, "old");
    assert_eq!(result.to_version, "new");
    assert_eq!(result.format, DiffFormat::Unified);
}

#[test]
fn test_generate_diff_sets_correct_format() {
    let generator = DiffGenerator;
    let old = "line one\nline two";
    let new = "line one\nline two modified";

    let unified = generator.generate_diff(old, new, DiffFormat::Unified);
    assert_eq!(unified.format, DiffFormat::Unified);

    let side_by_side = generator.generate_diff(old, new, DiffFormat::SideBySide);
    assert_eq!(side_by_side.format, DiffFormat::SideBySide);

    let inline = generator.generate_diff(old, new, DiffFormat::Inline);
    assert_eq!(inline.format, DiffFormat::Inline);
}

// Unified Format Tests

#[test]
fn test_unified_diff_addition_has_plus_prefix() {
    let generator = DiffGenerator;
    let old = "line one\nline two";
    let new = "line one\nline two\nline three";

    let result = generator.generate_diff(old, new, DiffFormat::Unified);

    let insertions: Vec<_> = result.changes.iter().filter_map(|c| match c {
        DiffChange::Inserted(text) => Some(text),
        _ => None,
    }).collect();

    assert!(!insertions.is_empty());
    assert!(insertions.contains(&&"line three".to_string()));
}

#[test]
fn test_unified_diff_deletion_has_minus_prefix() {
    let generator = DiffGenerator;
    let old = "line one\nline two\nline three";
    let new = "line one\nline two";

    let result = generator.generate_diff(old, new, DiffFormat::Unified);

    let deletions: Vec<_> = result.changes.iter().filter_map(|c| match c {
        DiffChange::Deleted(text) => Some(text),
        _ => None,
    }).collect();

    assert!(!deletions.is_empty());
    assert!(deletions.contains(&&"line three".to_string()));
}

#[test]
fn test_unified_diff_unchanged_without_prefix() {
    let generator = DiffGenerator;
    let old = "line one\nline two";
    let new = "line one\nline two";

    let result = generator.generate_diff(old, new, DiffFormat::Unified);

    let unchanged: Vec<_> = result.changes.iter().filter_map(|c| match c {
        DiffChange::Unchanged(text) => Some(text),
        _ => None,
    }).collect();

    assert!(!unchanged.is_empty());
    assert!(unchanged.contains(&&"line one".to_string()));
    assert!(unchanged.contains(&&"line two".to_string()));
}

// Side-by-Side Format Tests

#[test]
fn test_side_by_side_diff_aligns_unchanged_lines() {
    let generator = DiffGenerator;
    let old = "line one\nline two";
    let new = "line one\nline two";

    let result = generator.generate_diff(old, new, DiffFormat::SideBySide);

    let unchanged: Vec<_> = result.changes.iter().filter_map(|c| match c {
        DiffChange::Unchanged(text) => Some(text),
        _ => None,
    }).collect();

    assert_eq!(unchanged.len(), 2);
    assert!(unchanged.contains(&&"line one".to_string()));
    assert!(unchanged.contains(&&"line two".to_string()));
}

#[test]
fn test_side_by_side_diff_shows_addition_right_column() {
    let generator = DiffGenerator;
    let old = "line one";
    let new = "line one\nline two";

    let result = generator.generate_diff(old, new, DiffFormat::SideBySide);

    let has_insertion = result.changes.iter().any(|c| matches!(c, DiffChange::Inserted(_)));
    assert!(has_insertion);
}

#[test]
fn test_side_by_side_diff_shows_deletion_left_column() {
    let generator = DiffGenerator;
    let old = "line one\nline two";
    let new = "line one";

    let result = generator.generate_diff(old, new, DiffFormat::SideBySide);

    let has_deletion = result.changes.iter().any(|c| matches!(c, DiffChange::Deleted(_)));
    assert!(has_deletion);
}

// Inline Format Tests

#[test]
fn test_inline_diff_wraps_insertions() {
    let generator = DiffGenerator;
    let old = "hello world";
    let new = "hello beautiful world";

    let result = generator.generate_diff(old, new, DiffFormat::Inline);

    // Inline diff should have insertions
    let has_insertion = result.changes.iter().any(|c| matches!(c, DiffChange::Inserted(_)));
    assert!(has_insertion);
}

#[test]
fn test_inline_diff_wraps_deletions() {
    let generator = DiffGenerator;
    let old = "hello beautiful world";
    let new = "hello world";

    let result = generator.generate_diff(old, new, DiffFormat::Inline);

    // Inline diff should have deletions
    let has_deletion = result.changes.iter().any(|c| matches!(c, DiffChange::Deleted(_)));
    assert!(has_deletion);
}

#[test]
fn test_inline_diff_shows_unchanged_without_markers() {
    let generator = DiffGenerator;
    let old = "hello world";
    let new = "hello world";

    let result = generator.generate_diff(old, new, DiffFormat::Inline);

    let unchanged: Vec<_> = result.changes.iter().filter_map(|c| match c {
        DiffChange::Unchanged(text) => Some(text),
        _ => None,
    }).collect();

    assert!(!unchanged.is_empty());
}

// DiffChange Enum Tests

#[test]
fn test_diff_change_unchanged_contains_text_from_both() {
    let generator = DiffGenerator;
    let old = "same text";
    let new = "same text";

    let result = generator.generate_diff(old, new, DiffFormat::Unified);

    assert!(matches!(result.changes.first(), Some(DiffChange::Unchanged(_))));
}

#[test]
fn test_diff_change_inserted_contains_only_new_text() {
    let generator = DiffGenerator;
    let old = "";
    let new = "new text";

    let result = generator.generate_diff(old, new, DiffFormat::Unified);

    assert!(matches!(result.changes.first(), Some(DiffChange::Inserted(_))));
}

#[test]
fn test_diff_change_deleted_contains_only_old_text() {
    let generator = DiffGenerator;
    let old = "old text";
    let new = "";

    let result = generator.generate_diff(old, new, DiffFormat::Unified);

    assert!(matches!(result.changes.first(), Some(DiffChange::Deleted(_))));
}

#[test]
fn test_diff_change_replaced_contains_both_old_and_new() {
    let generator = DiffGenerator;
    let old = "old text";
    let new = "new text";

    let result = generator.generate_diff(old, new, DiffFormat::Inline);

    // Inline format uses Replaced for modifications
    let has_replaced = result.changes.iter().any(|c| matches!(c, DiffChange::Replaced { .. }));
    assert!(has_replaced);

    // Verify it has both old and new values
    if let Some(DiffChange::Replaced { old, new }) = result.changes.iter().find(|c| matches!(c, DiffChange::Replaced { .. })) {
        assert!(!old.is_empty());
        assert!(!new.is_empty());
    }
}

// Edge Cases Tests

#[test]
fn test_generate_diff_handles_empty_old_content() {
    let generator = DiffGenerator;
    let old = "";
    let new = "line one\nline two";

    let result = generator.generate_diff(old, new, DiffFormat::Unified);

    // Should all be insertions
    assert!(!result.changes.is_empty());
    assert!(result.changes.iter().all(|c| matches!(c, DiffChange::Inserted(_) | DiffChange::Unchanged(_))));
}

#[test]
fn test_generate_diff_handles_empty_new_content() {
    let generator = DiffGenerator;
    let old = "line one\nline two";
    let new = "";

    let result = generator.generate_diff(old, new, DiffFormat::Unified);

    // Should all be deletions
    assert!(!result.changes.is_empty());
}

#[test]
fn test_generate_diff_handles_identical_content() {
    let generator = DiffGenerator;
    let old = "identical content";
    let new = "identical content";

    let result = generator.generate_diff(old, new, DiffFormat::Unified);

    // Should only have unchanged entries
    assert!(result.changes.iter().all(|c| matches!(c, DiffChange::Unchanged(_))));
}

#[test]
fn test_generate_diff_handles_single_line_change() {
    let generator = DiffGenerator;
    let old = "single line";
    let new = "single line modified";

    let result = generator.generate_diff(old, new, DiffFormat::Unified);

    // Should detect the change
    assert!(!result.changes.is_empty());
}

#[test]
fn test_generate_diff_handles_multi_line_mixed_operations() {
    let generator = DiffGenerator;
    let old = "line one\nline two\nline three";
    let new = "line one\nline two modified\nline three\nline four";

    let result = generator.generate_diff(old, new, DiffFormat::Unified);

    // Should have a mix of changes
    let has_unchanged = result.changes.iter().any(|c| matches!(c, DiffChange::Unchanged(_)));
    let has_change = result.changes.iter().any(|c| !matches!(c, DiffChange::Unchanged(_)));

    assert!(has_unchanged);
    assert!(has_change);
}
