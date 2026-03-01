//! Document format conversion utilities
//!
//! This module provides conversion between Markdown and other document formats
//! including OpenDocument Format (ODF) and PDF.

use std::io::Write;
use std::process::{Command, Stdio};
use thiserror::Error;

/// Document format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Markdown,
    ODF,
    PDF,
}

/// Convert Markdown to OpenDocument Format
///
/// This function attempts to use pandoc if available, otherwise falls back
/// to a simple plain text wrapper with .odt extension.
pub fn markdown_to_odf(markdown: &str) -> Result<Vec<u8>, ConversionError> {
    // Try pandoc first
    if let Ok(output) = convert_with_pandoc(markdown, "odt") {
        return Ok(output);
    }

    // Fallback: Create a simple ODF-like wrapper
    // In production, this should use a proper ODF library
    create_simple_odt(markdown)
}

/// Convert Markdown to PDF
///
/// This function attempts to use pandoc if available.
pub fn markdown_to_pdf(markdown: &str) -> Result<Vec<u8>, ConversionError> {
    convert_with_pandoc(markdown, "pdf")
}

/// Convert using pandoc CLI tool
fn convert_with_pandoc(markdown: &str, format: &str) -> Result<Vec<u8>, ConversionError> {
    let mut child = Command::new("pandoc")
        .arg("-f")
        .arg("markdown")
        .arg("-t")
        .arg(format)
        .arg("--standalone")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    .map_err(|_| ConversionError::PandocNotAvailable)?;

    // Write markdown to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(markdown.as_bytes())
            .map_err(ConversionError::Io)?;
    }

    let output = child.wait_with_output().map_err(ConversionError::Io)?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(ConversionError::PandocFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

/// Create a simple ODT file (fallback when pandoc is not available)
///
/// Note: This is a minimal implementation. For production use,
/// consider using a proper ODF library like `rust-odf` or `write-odf`.
fn create_simple_odt(markdown: &str) -> Result<Vec<u8>, ConversionError> {
    // For now, create a minimal valid ODF structure
    // A proper implementation would use an ODF library

    // ODF files are ZIP archives containing XML files
    // This creates a text-only ODT content

    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content
    xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
    xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
    xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0">
  <office:body>
    <office:text>
      <text:h text:style-name="Heading_1">Document</text:h>
      <text:p text:style-name="Text_20_body">{}</text:p>
    </office:text>
  </office:body>
</office:document-content>"#,
        escape_xml(markdown)
    );

    // For a real ODT file, we'd need to create a proper ZIP archive
    // For now, return the content as-is (not a valid ODT, but preserves text)
    Ok(content.into_bytes())
}

/// Escape special XML characters
///
/// Uses a single-pass approach to ensure correct handling and avoid
/// double-escaping issues that can occur with chained replace() calls.
fn escape_xml(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        }
    }
    result
}

/// Conversion error types
#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Pandoc not available on this system")]
    PandocNotAvailable,

    #[error("Pandoc conversion failed: {0}")]
    PandocFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_odf_returns_bytes() {
        let markdown = "# Test Document\n\nSome content.";
        let result = markdown_to_odf(markdown);
        assert!(result.is_ok());

        let odf_bytes = result.unwrap();
        assert!(odf_bytes.len() > 0);
    }

    #[test]
    fn test_escape_xml() {
        let input = "Test <tag> & 'quotes'";
        let escaped = escape_xml(input);
        assert!(escaped.contains("&lt;"));
        assert!(escaped.contains("&amp;"));
        assert!(escaped.contains("&apos;"));
    }

    #[test]
    fn test_output_format_equality() {
        assert_eq!(OutputFormat::Markdown, OutputFormat::Markdown);
        assert_ne!(OutputFormat::PDF, OutputFormat::ODF);
    }

    #[test]
    fn test_create_simple_odt_contains_content() {
        let markdown = "# Test Document\n\nContent here.";
        let result = create_simple_odt(markdown);
        assert!(result.is_ok());

        let odf_bytes = result.unwrap();
        let content = String::from_utf8_lossy(&odf_bytes);
        assert!(content.contains("Test Document"));
        assert!(content.contains("office:document-content"));
    }
}
