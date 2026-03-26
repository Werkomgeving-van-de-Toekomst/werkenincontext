/// Test fixture helpers for stakeholder extraction tests

use iou_ai::agents::content::GeneratedDocument;
use iou_ai::stakeholder::ExtractionOptions;

/// Load a document fixture by name
pub fn load_document(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/documents/{}.txt",
        env!("CARGO_MANIFEST_DIR"),
        name
    ))
    .unwrap_or_else(|_| String::new())
}

/// Load expected extraction results
pub fn load_expected(name: &str) -> serde_json::Value {
    let content = std::fs::read_to_string(format!(
        "{}/tests/fixtures/expected/{}.json",
        env!("CARGO_MANIFEST_DIR"),
        name
    ));

    match content {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|_| serde_json::Value::Null),
        Err(_) => serde_json::Value::Null,
    }
}

/// Create a GeneratedDocument from fixture text
pub fn create_test_document(text: &str) -> GeneratedDocument {
    GeneratedDocument {
        document_id: uuid::Uuid::new_v4(),
        content: text.to_string(),
        variables: vec![],
        entity_links: vec![],
        sections: vec![],
        generated_at: chrono::Utc::now(),
    }
}

/// Default extraction options for testing
pub fn test_extraction_options() -> ExtractionOptions {
    ExtractionOptions {
        use_llm: false,  // Disable LLM for fast unit tests
        confidence_threshold: 0.5,
        enable_normalization: false,  // Disable for isolated tests
        max_llm_calls: 0,
        max_cost_per_document: 0.0,
        llm_timeout: std::time::Duration::from_secs(1),
    }
}
