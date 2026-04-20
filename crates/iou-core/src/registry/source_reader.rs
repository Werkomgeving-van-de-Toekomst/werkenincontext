//! Source Read API Templates
//!
//! Generic templates for reading data from various source systems.
//! Provides a unified interface for extracting entities from different
//! source types (databases, APIs, files, legacy systems).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::source::{FileFormat, SourceConnection, SourceType};
use crate::registry::{DataEntity, EntityType};

/// Configuration for reading from a source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceReadConfig {
    /// Source ID to read from
    pub source_id: Uuid,

    /// Entity type to map to
    pub entity_type: EntityType,

    /// Read template specific to source type
    pub template: ReadTemplate,

    /// Field mappings from source to registry schema
    pub field_mappings: HashMap<String, FieldMapping>,

    /// Read options
    pub options: ReadOptions,
}

/// Field mapping from source to registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Source field name
    pub source_field: String,

    /// Registry field name
    pub registry_field: String,

    /// Data type transformation
    pub transform: Option<Transform>,

    /// Default value if source field is missing
    pub default_value: Option<serde_json::Value>,
}

/// Data transformation to apply during mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transform {
    /// No transformation
    None,

    /// Convert to string
    ToString,

    /// Parse as integer
    ToInteger,

    /// Parse as float
    ToFloat,

    /// Parse as boolean
    ToBoolean,

    /// Parse as date/time (ISO 8601)
    ToDateTime,

    /// Apply regex extraction
    RegexExtract { pattern: String, group: usize },

    /// Map value through lookup table
    ValueMap { map: HashMap<String, serde_json::Value> },

    /// Concatenate multiple fields
    Concatenate { fields: Vec<String>, separator: String },
}

/// Read template for different source types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source_type")]
pub enum ReadTemplate {
    /// Database query template
    #[serde(rename = "database")]
    Database(DatabaseTemplate),

    /// REST API template
    #[serde(rename = "api")]
    Api(ApiTemplate),

    /// SOAP/WSDL service template
    #[serde(rename = "soap")]
    Soap(SoapTemplate),

    /// File-based template
    #[serde(rename = "file")]
    File(FileTemplate),

    /// Legacy system template
    #[serde(rename = "legacy")]
    Legacy(LegacyTemplate),

    /// GraphQL API template
    #[serde(rename = "graphql")]
    GraphQL(GraphQLTemplate),
}

/// Database read template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTemplate {
    /// SQL query (or stored procedure name)
    pub query: String,

    /// Query type
    pub query_type: DatabaseQueryType,

    /// Batch size for fetching
    pub batch_size: usize,

    /// Parameter bindings
    pub parameters: HashMap<String, DbParameter>,

    /// Schema information (optional, for validation)
    pub schema: Option<DatabaseSchema>,
}

/// Database query type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseQueryType {
    /// Standard SELECT query
    Select,

    /// Stored procedure call
    StoredProcedure,

    /// Custom function call
    Function,

    /// Bulk export operation
    BulkExport,
}

/// Database parameter binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbParameter {
    /// Parameter value
    pub value: serde_json::Value,

    /// SQL type hint
    pub sql_type: Option<String>,
}

/// Database schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSchema {
    /// Table name
    pub table: String,

    /// Column definitions
    pub columns: Vec<ColumnDefinition>,

    /// Primary key columns
    pub primary_keys: Vec<String>,
}

/// Column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    /// Column name
    pub name: String,

    /// Data type
    pub data_type: String,

    /// Whether column is nullable
    pub nullable: bool,

    /// Maximum length (for text columns)
    pub max_length: Option<usize>,
}

/// REST API read template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTemplate {
    /// HTTP method
    pub method: HttpMethod,

    /// API endpoint path (relative to base URL)
    pub endpoint: String,

    /// Path parameters (e.g., {id})
    pub path_params: HashMap<String, String>,

    /// Query parameters
    pub query_params: HashMap<String, ApiParam>,

    /// Request body template (for POST/PUT)
    pub body_template: Option<serde_json::Value>,

    /// Pagination configuration
    pub pagination: Option<PaginationConfig>,

    /// Response format
    pub response_format: ResponseFormat,
}

/// HTTP method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "uppercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// API parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiParam {
    /// Static value or reference to field
    pub value: ParamValue,

    /// Whether parameter is required
    pub required: bool,
}

/// Parameter value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamValue {
    Static(String),
    Field(String),
    Expression(String),
}

/// Pagination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    /// Pagination type
    pub pagination_type: PaginationType,

    /// Page size parameter name
    pub page_size_param: Option<String>,

    /// Page/token parameter name
    pub page_param: String,

    /// Maximum pages to fetch
    pub max_pages: Option<usize>,

    /// Items per page
    pub page_size: usize,
}

/// Pagination type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaginationType {
    PageBased,
    OffsetBased,
    CursorBased,
    LinkHeader,
}

/// Response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    /// JSON path to data array (e.g., "data.items")
    pub data_path: String,

    /// JSON path to total count (e.g., "metadata.total")
    pub total_path: Option<String>,

    /// Whether response is wrapped
    pub wrapped: bool,
}

/// SOAP service template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapTemplate {
    /// WSDL service endpoint
    pub service_endpoint: String,

    /// SOAP action
    pub soap_action: String,

    /// Operation name
    pub operation: String,

    /// Request template (SOAP body)
    pub request_template: String,

    /// XML namespace mapping
    pub namespaces: HashMap<String, String>,

    /// Response extraction (XPath)
    pub data_xpath: String,
}

/// File read template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTemplate {
    /// File format
    pub format: FileFormat,

    /// Whether file has header row (for CSV)
    pub has_header: bool,

    /// Field delimiter (for CSV/TSV)
    pub delimiter: Option<char>,

    /// Quote character (for CSV)
    pub quote_char: Option<char>,

    /// Root element path (for XML/JSON)
    pub root_path: Option<String>,

    /// Record path (for XML/JSON arrays)
    pub record_path: Option<String>,

    /// Encoding
    pub encoding: Option<String>,

    /// Sheet name (for Excel)
    pub sheet_name: Option<String>,
}

/// Legacy system template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyTemplate {
    /// Connection protocol
    pub protocol: String,

    /// Command or transaction code
    pub command: String,

    /// Screen/form identifier
    pub screen_id: Option<String>,

    /// Input fields mapping
    pub input_fields: HashMap<String, String>,

    /// Output parser configuration
    pub output_parser: OutputParser,

    /// Character encoding
    pub encoding: Option<String>,
}

/// Output parser for legacy systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputParser {
    /// Parser type
    pub parser_type: ParserType,

    /// Field positions (for fixed-width)
    pub field_positions: Option<HashMap<String, (usize, usize)>>,

    /// Delimiter (for delimited output)
    pub delimiter: Option<String>,

    /// Regex pattern (for pattern-based parsing)
    pub regex_pattern: Option<String>,
}

/// Parser type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParserType {
    Delimited,
    FixedWidth,
    Regex,
    Custom,
}

/// GraphQL API template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLTemplate {
    /// GraphQL query
    pub query: String,

    /// Variables template
    pub variables: Option<serde_json::Value>,

    /// Operation name (if multiple operations)
    pub operation_name: Option<String>,

    /// Response path to data array
    pub data_path: String,
}

/// Read options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReadOptions {
    /// Maximum number of records to read
    pub max_records: Option<usize>,

    /// Timeout in seconds
    pub timeout_seconds: Option<u64>,

    /// Whether to continue on error
    pub continue_on_error: bool,

    /// Whether to validate schema
    pub validate_schema: bool,

    /// Retry configuration
    pub retry: Option<RetryConfig>,

    /// Enable parallel processing
    pub parallel_processing: bool,

    /// Batch size for parallel processing
    pub batch_size: Option<usize>,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: usize,

    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,

    /// Backoff multiplier
    pub backoff_multiplier: f32,

    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
}

/// Result of a source read operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceReadResult {
    /// Source ID
    pub source_id: Uuid,

    /// Number of records read
    pub records_read: usize,

    /// Number of records successfully mapped
    pub records_mapped: usize,

    /// Number of records failed
    pub records_failed: usize,

    /// Entities created
    pub entities: Vec<DataEntity>,

    /// Errors encountered
    pub errors: Vec<ReadError>,

    /// Read duration
    pub duration_ms: u64,

    /// Timestamp
    pub read_at: DateTime<Utc>,
}

/// Error from reading a record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadError {
    /// Record identifier (if available)
    pub record_id: Option<String>,

    /// Error message
    pub error: String,

    /// Error type
    pub error_type: ReadErrorType,

    /// Raw record data (for debugging)
    pub raw_data: Option<serde_json::Value>,
}

/// Type of read error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadErrorType {
    Connection,
    Authentication,
    Permission,
    Validation,
    Mapping,
    Transformation,
    Timeout,
    Unknown,
}

/// Progress callback for long-running reads
pub type ReadProgressCallback = Box<dyn Fn(ReadProgress) + Send + Sync>;

/// Progress update during read operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadProgress {
    /// Source ID
    pub source_id: Uuid,

    /// Current page/batch
    pub current_page: usize,

    /// Total pages (if known)
    pub total_pages: Option<usize>,

    /// Records processed so far
    pub records_processed: usize,

    /// Estimated total records
    pub estimated_total: Option<usize>,

    /// Current phase
    pub phase: ReadPhase,
}

/// Phase of read operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadPhase {
    Connecting,
    Reading,
    Mapping,
    Completing,
}

// ============================================
// TEMPLATE BUILDER
// ============================================

/// Builder for creating source read templates
pub struct SourceReadTemplateBuilder {
    source_type: SourceType,
    config: HashMap<String, serde_json::Value>,
}

impl SourceReadTemplateBuilder {
    /// Create a new template builder
    pub fn new(source_type: SourceType) -> Self {
        Self {
            source_type,
            config: HashMap::new(),
        }
    }

    /// Build a database template
    pub fn database(self) -> DatabaseTemplateBuilder {
        DatabaseTemplateBuilder::new()
    }

    /// Build an API template
    pub fn api(self) -> ApiTemplateBuilder {
        ApiTemplateBuilder::new()
    }

    /// Build a file template
    pub fn file(self) -> FileTemplateBuilder {
        FileTemplateBuilder::new()
    }
}

/// Builder for database templates
pub struct DatabaseTemplateBuilder {
    query: Option<String>,
    query_type: DatabaseQueryType,
    batch_size: usize,
    parameters: HashMap<String, DbParameter>,
}

impl DatabaseTemplateBuilder {
    pub fn new() -> Self {
        Self {
            query: None,
            query_type: DatabaseQueryType::Select,
            batch_size: 1000,
            parameters: HashMap::new(),
        }
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn query_type(mut self, query_type: DatabaseQueryType) -> Self {
        self.query_type = query_type;
        self
    }

    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    pub fn parameter(mut self, name: impl Into<String>, value: DbParameter) -> Self {
        self.parameters.insert(name.into(), value);
        self
    }

    pub fn build(self) -> ReadTemplate {
        ReadTemplate::Database(DatabaseTemplate {
            query: self.query.expect("query is required"),
            query_type: self.query_type,
            batch_size: self.batch_size,
            parameters: self.parameters,
            schema: None,
        })
    }
}

impl Default for DatabaseTemplateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for API templates
pub struct ApiTemplateBuilder {
    method: HttpMethod,
    endpoint: Option<String>,
    query_params: HashMap<String, ApiParam>,
    response_format: ResponseFormat,
    pagination: Option<PaginationConfig>,
}

impl ApiTemplateBuilder {
    pub fn new() -> Self {
        Self {
            method: HttpMethod::Get,
            endpoint: None,
            query_params: HashMap::new(),
            response_format: ResponseFormat {
                data_path: "data".to_string(),
                total_path: None,
                wrapped: true,
            },
            pagination: None,
        }
    }

    pub fn method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    pub fn query_param(mut self, name: impl Into<String>, value: ParamValue) -> Self {
        self.query_params.insert(
            name.into(),
            ApiParam {
                value,
                required: false,
            },
        );
        self
    }

    pub fn response_format(mut self, data_path: impl Into<String>) -> Self {
        self.response_format.data_path = data_path.into();
        self
    }

    pub fn pagination(mut self, config: PaginationConfig) -> Self {
        self.pagination = Some(config);
        self
    }

    pub fn build(self) -> ReadTemplate {
        ReadTemplate::Api(ApiTemplate {
            method: self.method,
            endpoint: self.endpoint.expect("endpoint is required"),
            path_params: HashMap::new(),
            query_params: self.query_params,
            body_template: None,
            pagination: self.pagination,
            response_format: self.response_format,
        })
    }
}

impl Default for ApiTemplateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for file templates
pub struct FileTemplateBuilder {
    format: FileFormat,
    has_header: bool,
    delimiter: Option<char>,
    quote_char: Option<char>,
    root_path: Option<String>,
    record_path: Option<String>,
    sheet_name: Option<String>,
}

impl FileTemplateBuilder {
    pub fn new() -> Self {
        Self {
            format: FileFormat::Csv,
            has_header: true,
            delimiter: Some(','),
            quote_char: Some('"'),
            root_path: None,
            record_path: None,
            sheet_name: None,
        }
    }

    pub fn format(mut self, format: FileFormat) -> Self {
        self.format = format;
        self
    }

    pub fn has_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = Some(delimiter);
        self
    }

    pub fn root_path(mut self, path: impl Into<String>) -> Self {
        self.root_path = Some(path.into());
        self
    }

    pub fn sheet_name(mut self, name: impl Into<String>) -> Self {
        self.sheet_name = Some(name.into());
        self
    }

    pub fn build(self) -> ReadTemplate {
        ReadTemplate::File(FileTemplate {
            format: self.format,
            has_header: self.has_header,
            delimiter: self.delimiter,
            quote_char: self.quote_char,
            root_path: self.root_path,
            record_path: self.record_path,
            encoding: Some("UTF-8".to_string()),
            sheet_name: self.sheet_name,
        })
    }
}

impl Default for FileTemplateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// PRE-DEFINED TEMPLATES
// ============================================

/// Common template presets
pub struct TemplatePresets;

impl TemplatePresets {
    /// PostgreSQL table read template
    pub fn postgresql_table(table: &str, columns: &[&str]) -> ReadTemplate {
        let query = format!("SELECT {} FROM {}", columns.join(", "), table);
        DatabaseTemplateBuilder::new()
            .query(query)
            .query_type(DatabaseQueryType::Select)
            .batch_size(1000)
            .build()
    }

    /// REST API collection template
    pub fn rest_api_collection(endpoint: &str) -> ReadTemplate {
        ApiTemplateBuilder::new()
            .method(HttpMethod::Get)
            .endpoint(endpoint)
            .response_format("data")
            .pagination(PaginationConfig {
                pagination_type: PaginationType::PageBased,
                page_size_param: Some("limit".to_string()),
                page_param: "page".to_string(),
                max_pages: Some(100),
                page_size: 100,
            })
            .build()
    }

    /// CSV file template
    pub fn csv_file(has_header: bool) -> ReadTemplate {
        FileTemplateBuilder::new()
            .format(FileFormat::Csv)
            .has_header(has_header)
            .delimiter(',')
            .build()
    }

    /// JSON array file template
    pub fn json_array_file() -> ReadTemplate {
        FileTemplateBuilder::new()
            .format(FileFormat::Json)
            .root_path("$")
            .record_path("$")
            .build()
    }

    /// Excel file template
    pub fn excel_file(sheet_name: &str, has_header: bool) -> ReadTemplate {
        FileTemplateBuilder::new()
            .format(FileFormat::Excel)
            .sheet_name(sheet_name)
            .has_header(has_header)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_mapping_serialization() {
        let mapping = FieldMapping {
            source_field: "first_name".to_string(),
            registry_field: "name".to_string(),
            transform: Some(Transform::ToString),
            default_value: Some(serde_json::json!("Unknown")),
        };

        let json = serde_json::to_string(&mapping).unwrap();
        assert!(json.contains("source_field"));
        assert!(json.contains("to_string"));
    }

    #[test]
    fn test_database_template_builder() {
        let template = DatabaseTemplateBuilder::new()
            .query("SELECT * FROM citizens")
            .batch_size(500)
            .build();

        assert!(matches!(template, ReadTemplate::Database(_)));
    }

    #[test]
    fn test_api_template_builder() {
        let template = ApiTemplateBuilder::new()
            .method(HttpMethod::Get)
            .endpoint("/api/citizens")
            .build();

        assert!(matches!(template, ReadTemplate::Api(_)));
    }

    #[test]
    fn test_file_template_builder() {
        let template = FileTemplateBuilder::new()
            .format(FileFormat::Csv)
            .has_header(true)
            .build();

        assert!(matches!(template, ReadTemplate::File(_)));
    }

    #[test]
    fn test_template_presets() {
        let template = TemplatePresets::postgresql_table("citizens", &["id", "name"]);
        assert!(matches!(template, ReadTemplate::Database(_)));

        let template = TemplatePresets::rest_api_collection("/api/entities");
        assert!(matches!(template, ReadTemplate::Api(_)));

        let template = TemplatePresets::csv_file(true);
        assert!(matches!(template, ReadTemplate::File(_)));
    }

    #[test]
    fn test_read_progress() {
        let progress = ReadProgress {
            source_id: Uuid::new_v4(),
            current_page: 1,
            total_pages: Some(10),
            records_processed: 100,
            estimated_total: Some(1000),
            phase: ReadPhase::Reading,
        };

        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("reading"));
    }
}
