use similar::{ChangeTag, TextDiff};

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
                        // Strip trailing newlines for consistency with line-based diffs
                        let value = change.value();
                        let text = value.strip_suffix('\n').unwrap_or(value).to_string();
                        changes.push(DiffChange::Unchanged(text));
                    }
                    ChangeTag::Delete => {
                        pending_delete.push_str(change.value());
                    }
                    ChangeTag::Insert => {
                        pending_insert.push_str(change.value());
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
