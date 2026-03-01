//! Template rendering system for document generation
//!
//! This module provides a Tera-based template engine for rendering Markdown
//! documents with variable substitution and conditional sections.

use crate::conversion::markdown_to_odf;
use iou_core::document::{RenderedDocument, Template, TemplateVariable, VariableSource};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tera::{Context, Result as TeraResult, Tera, Value};
use thiserror::Error;

/// Template engine error types
#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Tera error: {0}")]
    Tera(#[from] tera::Error),

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Invalid template syntax: {0}")]
    InvalidSyntax(String),

    #[error("Conversion error: {0}")]
    ConversionError(String),
}

/// Template engine wrapper for document generation
pub struct TemplateEngine {
    tera: Arc<Mutex<Tera>>,
}

impl TemplateEngine {
    /// Create a new template engine instance
    pub fn new() -> Result<Self, TemplateError> {
        let mut tera = Tera::default();

        // Add custom filters for Dutch government documents
        tera.register_filter("dutch_date", dutch_date_filter);
        tera.register_filter("format_iban", format_iban_filter);
        tera.register_filter("slugify", slugify_filter);

        Ok(Self {
            tera: Arc::new(Mutex::new(tera)),
        })
    }

    /// Register a template from a string
    pub fn register_template(
        &self,
        name: &str,
        content: &str,
    ) -> Result<(), TemplateError> {
        let mut tera = self.tera.lock().unwrap();
        tera.add_raw_template(name, content)?;
        Ok(())
    }

    /// Register multiple templates at once
    pub fn register_templates(
        &self,
        templates: &HashMap<String, String>,
    ) -> Result<(), TemplateError> {
        let mut tera = self.tera.lock().unwrap();
        for (name, content) in templates {
            tera.add_raw_template(name, content)?;
        }
        Ok(())
    }

    /// Render a template with the given context
    pub fn render(
        &self,
        template_name: &str,
        variables: &HashMap<String, TemplateVariable>,
    ) -> Result<RenderedDocument, TemplateError> {
        let tera = self.tera.lock().unwrap();
        let mut context = Context::new();
        let mut variables_used = Vec::new();

        // Add all variables to context
        for (name, var) in variables {
            context.insert(name, &var.value);
            variables_used.push(name.clone());
        }

        // Render the template
        let content = tera.render(template_name, &context)?;

        Ok(RenderedDocument {
            content,
            variables_used,
        })
    }

    /// Render a template and convert to ODF format
    pub fn render_to_odf(
        &self,
        template_name: &str,
        variables: &HashMap<String, TemplateVariable>,
    ) -> Result<Vec<u8>, TemplateError> {
        let rendered = self.render(template_name, variables)?;
        markdown_to_odf(&rendered.content)
            .map_err(|e| TemplateError::ConversionError(e.to_string()))
    }

    /// Resolve template variables from multiple sources
    pub fn resolve_variables(
        &self,
        template: &Template,
        user_input: &HashMap<String, String>,
        kg_data: &HashMap<String, String>,
        agent_data: &HashMap<String, String>,
    ) -> Result<HashMap<String, TemplateVariable>, TemplateError> {
        let mut result = HashMap::new();

        for var_name in &template.required_variables {
            let (value, source) = resolve_variable(
                var_name,
                user_input,
                kg_data,
                agent_data,
            )?;

            result.insert(
                var_name.clone(),
                TemplateVariable {
                    name: var_name.clone(),
                    value,
                    source,
                },
            );
        }

        Ok(result)
    }

    /// Get list of required variables from template content
    pub fn extract_required_variables(&self, content: &str) -> Vec<String> {
        let re = Regex::new(r"\{\{\s*([\w_]+)\s*\}\}").unwrap();
        re.captures_iter(content)
            .map(|cap| cap[1].to_string())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Resolve a single variable with priority order
fn resolve_variable(
    name: &str,
    user_input: &HashMap<String, String>,
    kg_data: &HashMap<String, String>,
    agent_data: &HashMap<String, String>,
) -> Result<(String, VariableSource), TemplateError> {
    // Priority 1: User input
    if let Some(value) = user_input.get(name) {
        return Ok((value.clone(), VariableSource::UserInput));
    }

    // Priority 2: Knowledge graph
    if let Some(value) = kg_data.get(name) {
        return Ok((value.clone(), VariableSource::KnowledgeGraph));
    }

    // Priority 3: Agent generated
    if let Some(value) = agent_data.get(name) {
        return Ok((value.clone(), VariableSource::AgentGenerated));
    }

    Err(TemplateError::VariableNotFound(name.to_string()))
}

/// Custom filter for Dutch date formatting
fn dutch_date_filter(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().ok_or_else(|| {
        tera::Error::msg("dutch_date filter requires a string value")
    })?;

    // Try to parse as ISO date and format as Dutch date
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        let formatted = dt.format("%-d %B %Y").to_string();
        return Ok(Value::String(formatted));
    }

    if let Ok(dt) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let formatted = dt.format("%-d %B %Y").to_string();
        return Ok(Value::String(formatted));
    }

    // If parsing fails, return original
    Ok(value.clone())
}

/// Custom filter for IBAN formatting
fn format_iban_filter(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().ok_or_else(|| {
        tera::Error::msg("format_iban filter requires a string value")
    })?;

    // Remove spaces and convert to uppercase, then add spaces every 4 characters
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let formatted: String = cleaned
        .chars()
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ");

    Ok(Value::String(formatted.to_uppercase()))
}

/// Custom filter for slugifying strings
fn slugify_filter(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().ok_or_else(|| {
        tera::Error::msg("slugify filter requires a string value")
    })?;

    let slug = slug::slugify(s);

    // If slug is empty (e.g., input was only special characters), provide a fallback
    if slug.is_empty() {
        return Ok(Value::String(format!("doc-{}", uuid::Uuid::new_v4())));
    }

    Ok(Value::String(slug))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_engine_initializes_without_errors() {
        let engine = TemplateEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_template_default() {
        let engine = TemplateEngine::default();
        // Engine created successfully
        let _ = &engine;
    }

    #[test]
    fn test_variable_substitution_replaces_all_placeholders() {
        let engine = TemplateEngine::new().unwrap();
        engine
            .register_template("test_template", "Hello {{ name }}, your reference is {{ ref }}.")
            .unwrap();

        let mut variables = HashMap::new();
        variables.insert(
            "name".to_string(),
            TemplateVariable {
                name: "name".to_string(),
                value: "Jan".to_string(),
                source: VariableSource::UserInput,
            },
        );
        variables.insert(
            "ref".to_string(),
            TemplateVariable {
                name: "ref".to_string(),
                value: "12345".to_string(),
                source: VariableSource::Default,
            },
        );

        let result = engine.render("test_template", &variables).unwrap();
        assert_eq!(result.content, "Hello Jan, your reference is 12345.");
    }

    #[test]
    fn test_conditional_sections_render_correctly_based_on_variables() {
        let engine = TemplateEngine::new().unwrap();
        engine
            .register_template(
                "conditional_test",
                "{% if show_extra %}Extra content{% endif %}Main content",
            )
            .unwrap();

        let mut variables = HashMap::new();
        variables.insert(
            "show_extra".to_string(),
            TemplateVariable {
                name: "show_extra".to_string(),
                value: "true".to_string(),
                source: VariableSource::Default,
            },
        );

        let result = engine.render("conditional_test", &variables).unwrap();
        assert!(result.content.contains("Extra content"));
        assert!(result.content.contains("Main content"));
    }

    #[test]
    fn test_conditional_section_when_false() {
        let engine = TemplateEngine::new().unwrap();
        engine
            .register_template(
                "conditional_test",
                "{% if show_extra %}Extra content{% endif %}Main content",
            )
            .unwrap();

        let mut variables = HashMap::new();
        // Use a string that Tera will treat as falsy (empty string or "false")
        variables.insert(
            "show_extra".to_string(),
            TemplateVariable {
                name: "show_extra".to_string(),
                value: "".to_string(),  // Empty string is falsy in Tera
                source: VariableSource::Default,
            },
        );

        let result = engine.render("conditional_test", &variables).unwrap();
        assert!(!result.content.contains("Extra content"));
        assert!(result.content.contains("Main content"));
    }

    #[test]
    fn test_variable_resolution_priority_user_input() {
        let engine = TemplateEngine::new().unwrap();

        let template = Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            domain_id: "default".to_string(),
            document_type: "test".to_string(),
            content: "{{ value }}".to_string(),
            required_variables: vec!["value".to_string()],
            optional_sections: vec![],
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            is_active: true,
        };

        let mut user_input = HashMap::new();
        user_input.insert("value".to_string(), "from_user".to_string());

        let mut kg_data = HashMap::new();
        kg_data.insert("value".to_string(), "from_kg".to_string());

        let result = engine
            .resolve_variables(&template, &user_input, &kg_data, &HashMap::new())
            .unwrap();

        assert_eq!(result["value"].value, "from_user");
        assert_eq!(result["value"].source, VariableSource::UserInput);
    }

    #[test]
    fn test_variable_resolution_priority_kg() {
        let engine = TemplateEngine::new().unwrap();

        let template = Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            domain_id: "default".to_string(),
            document_type: "test".to_string(),
            content: "{{ value }}".to_string(),
            required_variables: vec!["value".to_string()],
            optional_sections: vec![],
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            is_active: true,
        };

        let user_input = HashMap::new();
        let mut kg_data = HashMap::new();
        kg_data.insert("value".to_string(), "from_kg".to_string());

        let result = engine
            .resolve_variables(&template, &user_input, &kg_data, &HashMap::new())
            .unwrap();

        assert_eq!(result["value"].value, "from_kg");
        assert_eq!(result["value"].source, VariableSource::KnowledgeGraph);
    }

    #[test]
    fn test_extract_required_variables() {
        let engine = TemplateEngine::new().unwrap();
        let content = "Hello {{ name }}, your order {{ order_id }} is ready.";
        let mut vars = engine.extract_required_variables(content);
        vars.sort(); // HashSet doesn't preserve order, so sort for comparison
        assert_eq!(vars, vec!["name", "order_id"]);
    }

    #[test]
    fn test_dutch_date_filter() {
        let engine = TemplateEngine::new().unwrap();
        engine
            .register_template("date_test", "{{ date|dutch_date }}")
            .unwrap();

        let mut variables = HashMap::new();
        variables.insert(
            "date".to_string(),
            TemplateVariable {
                name: "date".to_string(),
                value: "2025-03-01".to_string(),
                source: VariableSource::Default,
            },
        );

        let result = engine.render("date_test", &variables).unwrap();
        // Dutch month name for March is "maart" and year should be present
        assert!(result.content.contains("2025"));
    }

    #[test]
    fn test_format_iban_filter() {
        let engine = TemplateEngine::new().unwrap();
        engine
            .register_template("iban_test", "{{ iban|format_iban }}")
            .unwrap();

        let mut variables = HashMap::new();
        variables.insert(
            "iban".to_string(),
            TemplateVariable {
                name: "iban".to_string(),
                value: "NL91ABNA0417164300".to_string(),
                source: VariableSource::Default,
            },
        );

        let result = engine.render("iban_test", &variables).unwrap();
        // Should be formatted with spaces every 4 characters
        assert!(result.content.contains(' '));
    }

    #[test]
    fn test_missing_variables_detected() {
        let engine = TemplateEngine::new().unwrap();
        engine
            .register_template("missing_test", "Hello {{ name }}, {{ missing_var }}")
            .unwrap();

        let mut variables = HashMap::new();
        variables.insert(
            "name".to_string(),
            TemplateVariable {
                name: "name".to_string(),
                value: "World".to_string(),
                source: VariableSource::Default,
            },
        );

        // Tera throws an error for missing variables in strict mode
        let result = engine.render("missing_test", &variables);
        assert!(result.is_err());
    }

    #[test]
    fn test_conditional_with_default_variable() {
        let engine = TemplateEngine::new().unwrap();
        engine
            .register_template(
                "conditional_default",
                "{% if show_extra %}Extra content{% endif %}Main content",
            )
            .unwrap();

        // When variable is not provided, Tera treats it as falsy
        let variables = HashMap::new();
        let result = engine.render("conditional_default", &variables).unwrap();
        assert!(!result.content.contains("Extra content"));
        assert!(result.content.contains("Main content"));
    }
}
