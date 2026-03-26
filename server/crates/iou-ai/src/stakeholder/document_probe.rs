//! Document structure verification module
//!
//! Verifies that GeneratedDocument provides access to text content
//! for entity extraction processing.

use crate::agents::GeneratedDocument;

/// Verify GeneratedDocument has accessible content field
///
/// This function confirms that a GeneratedDocument contains
/// accessible text content that can be processed by entity extraction.
///
/// # Arguments
///
/// * `document` - Reference to a GeneratedDocument
///
/// # Returns
///
/// * `Ok(String)` - The document content if accessible
/// * `Err(String)` - Error message if content is not accessible
pub fn verify_document_structure(document: &GeneratedDocument) -> Result<String, String> {
    // The document has a `content` field that contains the generated Markdown
    if document.content.is_empty() {
        return Err("Document content is empty".to_string());
    }

    Ok(document.content.clone())
}

/// Extract text content from GeneratedDocument for entity extraction
///
/// This is a convenience function that returns the document content
/// as a string slice for processing by entity extraction algorithms.
///
/// # Arguments
///
/// * `document` - Reference to a GeneratedDocument
///
/// # Returns
///
/// * `Some(&str)` - The document content if present and non-empty
/// * `None` - If content is empty
pub fn extract_document_text(document: &GeneratedDocument) -> Option<&str> {
    if document.content.is_empty() {
        None
    } else {
        Some(document.content.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

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

    #[test]
    fn test_verify_document_structure_with_content() {
        let doc = create_test_document("Test document content");
        let result = verify_document_structure(&doc);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test document content");
    }

    #[test]
    fn test_verify_document_structure_with_empty_content() {
        let doc = create_test_document("");
        let result = verify_document_structure(&doc);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Document content is empty");
    }

    #[test]
    fn test_extract_document_text_with_content() {
        let doc = create_test_document("Some content");
        let result = extract_document_text(&doc);
        assert_eq!(result, Some("Some content"));
    }

    #[test]
    fn test_extract_document_text_with_empty_content() {
        let doc = create_test_document("");
        let result = extract_document_text(&doc);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_document_text_handles_multiline() {
        let content = "Line 1\nLine 2\nLine 3";
        let doc = create_test_document(content);
        let result = extract_document_text(&doc);
        assert_eq!(result, Some(content));
    }
}
