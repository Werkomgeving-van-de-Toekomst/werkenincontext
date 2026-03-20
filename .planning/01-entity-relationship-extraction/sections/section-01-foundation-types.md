Now I have a comprehensive understanding of the codebase. Let me create the section content for section-01-foundation-types:

# Section 01: Foundation & Types

## Overview

This section defines the core data structures and public API for the stakeholder extraction system. These types serve as the foundation for all subsequent extraction, normalization, and storage operations.

The key design principle is to extend the existing `Entity` type from `iou-core::graphrag` rather than creating parallel types. Stakeholder-specific attributes are stored in the `metadata` field, with convenience wrappers providing type-safe access.

## Dependencies

**External Crates:**
- `iou-core` - for existing `Entity`, `EntityType`, `Relationship` types
- `uuid` - for UUID generation
- `chrono` - for timestamps
- `serde` - for serialization
- `strum` - for enum string conversions

**No internal dependencies** - this section can be implemented in parallel with Section 00 (Feasibility Spike).

## File Structure

Create the following new module structure:

```
crates/iou-ai/src/stakeholder/
├── mod.rs              # Public API exports
├── types.rs            # Core convenience wrappers
├── extractor.rs        # StakeholderExtractor trait
├── result.rs           # ExtractionResult and stats
├── mention.rs          # MentionRelationship type
├── normalization.rs    # Dutch name normalization
└── error.rs            # Error types
```

## Implementation

### 1. Module Declaration

File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/lib.rs`

Add to the existing pub mod declarations:

```rust
pub mod stakeholder;
```

### 2. Public API Module

File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/mod.rs`

```rust
//! Stakeholder extraction for Dutch government documents
//!
//! This module provides entity and relationship extraction from Wet open overheid
//! (Woo) documents, with specific support for:
//! - Dutch name patterns (tussenvoegsels like "van", "van der", "de")
//! - Government organization normalization
//! - PII classification for GDPR compliance

pub mod types;
pub mod extractor;
pub mod result;
pub mod mention;
pub mod normalization;
pub mod error;

// Re-export public API
pub use types::{PersonStakeholder, OrganizationStakeholder};
pub use extractor::{StakeholderExtractor, ExtractionOptions};
pub use result::{ExtractionResult, ExtractionStats, VerificationStatus};
pub use mention::{MentionRelationship, MentionType, ExtractionMethod};
pub use error::{ExtractionError, NormalizationError, DeduplicationError};
pub use normalization::{DutchNameNormalizer, NameComparison};
```

### 3. Convenience Wrapper Types

File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/types.rs`

```rust
//! Convenience wrappers for stakeholder entities with typed metadata access
//!
//! These wrappers extend the base Entity type with type-safe accessors for
//! stakeholder-specific metadata stored in the Entity.metadata field.

use chrono::{DateTime, Utc};
use iou_core::graphrag::{Entity, EntityType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Convenience wrapper for person entities
///
/// Provides typed access to person-specific metadata stored in Entity.metadata.
/// The underlying Entity is always accessible via the `entity` field.
#[derive(Debug, Clone)]
pub struct PersonStakeholder {
    pub entity: Entity,
}

impl PersonStakeholder {
    /// Metadata key for person title
    const METADATA_TITLE: &str = "title";
    /// Metadata key for person role
    const METADATA_ROLE: &str = "role";
    /// Metadata key for department
    const METADATA_DEPARTMENT: &str = "department";
    /// Metadata key for email
    const METADATA_EMAIL: &str = "email";
    /// Metadata key for phone
    const METADATA_PHONE: &str = "phone";
    /// Metadata key for PII classification
    const METADATA_PII_CLASSIFICATION: &str = "pii_classification";

    /// Create a new person stakeholder
    pub fn new(name: String, confidence: f32) -> Self {
        Self {
            entity: Entity {
                id: Uuid::new_v4(),
                name,
                entity_type: EntityType::Person,
                canonical_name: None,
                description: None,
                confidence,
                source_domain_id: None,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                created_at: Utc::now(),
            }
        }
    }

    /// Get the person's title (dr., prof., etc.)
    pub fn title(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_TITLE)?.as_str()
    }

    /// Set the person's title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        let _ = self.entity.metadataserde_json::json!(Self::METADATA_TITLE, title.into());
        self
    }

    /// Get the person's role
    pub fn role(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_ROLE)?.as_str()
    }

    /// Set the person's role
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        // Implementation would set metadata value
        self
    }

    /// Get the department
    pub fn department(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_DEPARTMENT)?.as_str()
    }

    /// Set the department
    pub fn with_department(mut self, department: impl Into<String>) -> Self {
        // Implementation
        self
    }

    /// Get email address
    pub fn email(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_EMAIL)?.as_str()
    }

    /// Set email address
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        // Implementation
        self
    }

    /// Get phone number
    pub fn phone(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_PHONE)?.as_str()
    }

    /// Set phone number
    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        // Implementation
        self
    }

    /// Convert to base Entity
    pub fn into_entity(self) -> Entity {
        self.entity
    }

    /// Borrow as base Entity
    pub fn as_entity(&self) -> &Entity {
        &self.entity
    }
}

impl From<Entity> for PersonStakeholder {
    fn from(entity: Entity) -> Self {
        assert_eq!(entity.entity_type, EntityType::Person);
        Self { entity }
    }
}

/// Convenience wrapper for organization entities
#[derive(Debug, Clone)]
pub struct OrganizationStakeholder {
    pub entity: Entity,
}

impl OrganizationStakeholder {
    /// Metadata keys for organization attributes
    const METADATA_SHORT_NAME: &str = "short_name";
    const METADATA_ORG_TYPE: &str = "org_type";
    const METADATA_PARENT_ORG: &str = "parent_org";
    const METADATA_LOCATION: &str = "location";

    /// Create a new organization stakeholder
    pub fn new(name: String, confidence: f32) -> Self {
        Self {
            entity: Entity {
                id: Uuid::new_v4(),
                name,
                entity_type: EntityType::Organization,
                canonical_name: None,
                description: None,
                confidence,
                source_domain_id: None,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                created_at: Utc::now(),
            }
        }
    }

    /// Get short name/abbreviation (e.g., "MinFin")
    pub fn short_name(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_SHORT_NAME)?.as_str()
    }

    /// Set short name
    pub fn with_short_name(mut self, short_name: impl Into<String>) -> Self {
        // Implementation
        self
    }

    /// Get organization type
    pub fn org_type(&self) -> Option<OrgType> {
        self.entity.metadata
            .get(Self::METADATA_ORG_TYPE)?
            .as_str()
            .and_then(|s| OrgType::try_from(s).ok())
    }

    /// Set organization type
    pub fn with_org_type(mut self, org_type: OrgType) -> Self {
        // Implementation
        self
    }

    /// Get parent organization
    pub fn parent_org(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_PARENT_ORG)?.as_str()
    }

    /// Set parent organization
    pub fn with_parent_org(mut self, parent: impl Into<String>) -> Self {
        // Implementation
        self
    }

    /// Get location
    pub fn location(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_LOCATION)?.as_str()
    }

    /// Set location
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        // Implementation
        self
    }

    /// Convert to base Entity
    pub fn into_entity(self) -> Entity {
        self.entity
    }
}

impl From<Entity> for OrganizationStakeholder {
    fn from(entity: Entity) -> Self {
        assert_eq!(entity.entity_type, EntityType::Organization);
        Self { entity }
    }
}

/// Type of organization (Dutch government context)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrgType {
    Ministry,
    Agency,
    Other,
}

impl TryFrom<&str> for OrgType {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "ministry" | "ministerie" => Ok(OrgType::Ministry),
            "agency" | "dienst" | "agentschap" => Ok(OrgType::Agency),
            _ => Ok(OrgType::Other),
        }
    }
}

impl AsRef<str> for OrgType {
    fn as_ref(&self) -> &str {
        match self {
            OrgType::Ministry => "ministry",
            OrgType::Agency => "agency",
            OrgType::Other => "other",
        }
    }
}
```

### 4. Extraction Result Types

File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/result.rs`

```rust
//! Result types for stakeholder extraction

use serde::{Deserialize, Serialize};

/// Result of the stakeholder extraction process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// Extracted person entities
    pub persons: Vec<PersonStakeholder>,
    
    /// Extracted organization entities
    pub organizations: Vec<OrganizationStakeholder>,
    
    /// Mention relationships linking entities to documents
    pub mentions: Vec<MentionRelationship>,
    
    /// Extraction statistics
    pub stats: ExtractionStats,
    
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

use super::types::PersonStakeholder;
use super::types::OrganizationStakeholder;
use super::mention::MentionRelationship;

/// Statistics from extraction process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionStats {
    /// Total entities extracted
    pub total_entities: usize,
    
    /// High confidence entities (>= 0.9)
    pub high_confidence: usize,
    
    /// Medium confidence entities (0.7 - 0.9)
    pub medium_confidence: usize,
    
    /// Low confidence entities (0.5 - 0.7)
    pub low_confidence: usize,
    
    /// LLM API calls made
    pub llm_calls_made: usize,
    
    /// Rijksoverheid API calls made
    pub api_calls_made: usize,
}

impl ExtractionStats {
    /// Create new stats with all counters at zero
    pub fn new() -> Self {
        Self {
            total_entities: 0,
            high_confidence: 0,
            medium_confidence: 0,
            low_confidence: 0,
            llm_calls_made: 0,
            api_calls_made: 0,
        }
    }
}

impl Default for ExtractionStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Verification status based on confidence score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationStatus {
    /// Auto-accepted (confidence >= 0.9)
    AutoAccepted,
    
    /// Accepted but flagged for review (0.7 <= confidence < 0.9)
    AcceptedFlagged,
    
    /// Pending manual review (0.5 <= confidence < 0.7)
    PendingReview,
    
    /// Rejected (confidence < 0.5)
    Rejected,
}

impl VerificationStatus {
    /// Determine status from confidence score
    pub fn from_confidence(confidence: f32) -> Self {
        if confidence >= 0.9 {
            VerificationStatus::AutoAccepted
        } else if confidence >= 0.7 {
            VerificationStatus::AcceptedFlagged
        } else if confidence >= 0.5 {
            VerificationStatus::PendingReview
        } else {
            VerificationStatus::Rejected
        }
    }
}
```

### 5. Mention Relationship Type

File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/mention.rs`

```rust
//! Mention relationship types
//!
//! A mention relationship connects an entity to a document where it was
//! referenced, capturing the context of the mention.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A mention of an entity within a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentionRelationship {
    /// Unique identifier for this mention
    pub id: Uuid,
    
    /// ID of the entity being mentioned
    pub entity_id: Uuid,
    
    /// ID of the document containing the mention
    pub document_id: Uuid,
    
    /// How the entity is mentioned in this document
    pub mention_type: MentionType,
    
    /// Surrounding text context (optional)
    pub context: Option<String>,
    
    /// Position within document text
    pub position: Option<TextPosition>,
    
    /// Confidence of this mention extraction
    pub confidence: f32,
    
    /// When this mention was created
    pub created_at: DateTime<Utc>,
}

/// Position of text within a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPosition {
    /// Character offset from start
    pub offset: usize,
    
    /// Length of mention in characters
    pub length: usize,
    
    /// Line number (if available)
    pub line: Option<usize>,
}

/// How the entity is mentioned (populated during extraction)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MentionType {
    /// Primary subject of sentence/document
    Subject,
    
    /// Author of document
    Author,
    
    /// Recipient of document
    Recipient,
    
    /// Referenced in passing (default)
    Referenced,
}

impl Default for MentionType {
    fn default() -> Self {
        MentionType::Referenced
    }
}

/// Method used to extract the entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExtractionMethod {
    /// Regex-based extraction
    Regex,
    
    /// rust-bert NER model
    RustBert,
    
    /// LLM extraction
    LLM,
    
    /// Manually added
    Manual,
}

impl MentionRelationship {
    /// Create a new mention relationship
    pub fn new(
        entity_id: Uuid,
        document_id: Uuid,
        mention_type: MentionType,
        confidence: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_id,
            document_id,
            mention_type,
            context: None,
            position: None,
            confidence,
            created_at: Utc::now(),
        }
    }

    /// Add context to the mention
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    /// Add position information
    pub fn with_position(mut self, position: TextPosition) -> Self {
        self.position = Some(position);
        self
    }
}

// Test stub for serialization
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mention_relationship_serialization() {
        let mention = MentionRelationship::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            MentionType::Author,
            0.95,
        );

        let json = serde_json::to_string(&mention).unwrap();
        let deserialized: MentionRelationship = serde_json::from_str(&json).unwrap();

        assert_eq!(mention.id, deserialized.id);
        assert_eq!(mention.mention_type, deserialized.mention_type);
    }
}
```

### 6. Main Extractor Trait

File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/extractor.rs`

```rust
//! Main stakeholder extraction interface
//!
//! This module defines the public trait that all extractor implementations
//! must fulfill. The trait is object-safe, allowing for dynamic dispatch
//! and boxed trait objects.

use std::time::Duration;
use async_trait::async_trait;

use crate::agents::content::GeneratedDocument;
use iou_core::graphrag::Entity;

use super::result::ExtractionResult;
use super::error::{ExtractionError, NormalizationError, DeduplicationError};

/// Main stakeholder extraction interface
///
/// This trait defines the contract for stakeholder extraction implementations.
/// It is object-safe, meaning it can be used as a trait object (Box<dyn StakeholderExtractor>).
#[async_trait]
pub trait StakeholderExtractor: Send + Sync {
    /// Extract entities from a document
    ///
    /// # Arguments
    /// * `document` - The document to extract entities from
    /// * `options` - Configuration options for extraction
    ///
    /// # Returns
    /// Extraction result containing entities, mentions, and statistics
    async fn extract(
        &self,
        document: &GeneratedDocument,
        options: &ExtractionOptions,
    ) -> Result<ExtractionResult, ExtractionError>;

    /// Normalize entities using external API
    ///
    /// This method resolves entity names to their canonical forms,
    /// particularly for Dutch government organizations.
    async fn normalize_entities(
        &self,
        entities: Vec<Entity>,
    ) -> Result<Vec<Entity>, NormalizationError>;

    /// Deduplicate entities by similarity
    ///
    /// Identifies and merges duplicate entities using string similarity
    /// and clustering algorithms.
    async fn deduplicate_entities(
        &self,
        entities: Vec<Entity>,
    ) -> Result<Vec<Entity>, DeduplicationError>;
}

/// Configuration options for extraction process
#[derive(Debug, Clone)]
pub struct ExtractionOptions {
    /// Whether to use LLM for uncertain cases
    pub use_llm: bool,

    /// Minimum confidence threshold
    pub confidence_threshold: f32,

    /// Whether to normalize via external API
    pub enable_normalization: bool,

    /// Maximum LLM calls per document (cost control)
    pub max_llm_calls: usize,

    /// Maximum cost per document in USD (cost control)
    pub max_cost_per_document: f32,

    /// Timeout for LLM API calls
    pub llm_timeout: Duration,
}

impl Default for ExtractionOptions {
    fn default() -> Self {
        Self {
            use_llm: true,
            confidence_threshold: 0.5,
            enable_normalization: true,
            max_llm_calls: 10,
            max_cost_per_document: 0.10, // $0.10 per document max
            llm_timeout: Duration::from_secs(10),
        }
    }
}

// Test stub for object safety
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_is_object_safe() {
        // This test verifies the trait can be used as a trait object
        // by simply compiling this code
        fn _accept_boxed(_: Box<dyn StakeholderExtractor>) {}
    }
}
```

### 7. Error Types

File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/error.rs`

```rust
//! Error types for stakeholder extraction

use thiserror::Error;

/// Main error type for stakeholder extraction
#[derive(Debug, Error)]
pub enum ExtractionError {
    #[error("Document text is empty or inaccessible")]
    EmptyDocument,

    #[error("Baseline extraction failed: {0}")]
    BaselineFailed(String),

    #[error("LLM extraction failed: {0}")]
    LlmFailed(String),

    #[error("Normalization failed: {0}")]
    NormalizationFailed(#[from] NormalizationError),

    #[error("Deduplication failed: {0}")]
    DeduplicationFailed(#[from] DeduplicationError),

    #[error("Cost limit exceeded: {actual:.4} > {limit:.4} USD")]
    CostLimitExceeded { actual: f32, limit: f32 },

    #[error("API rate limit exceeded")]
    RateLimitExceeded,

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Error during entity normalization
#[derive(Debug, Error)]
pub enum NormalizationError {
    #[error("API request failed: {0}")]
    ApiRequestFailed(String),

    #[error("API timeout after {0}s")]
    ApiTimeout(u64),

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("No canonical name found for: {0}")]
    CanonicalNameNotFound(String),

    #[error("Cache error: {0}")]
    CacheError(String),
}

/// Error during entity deduplication
#[derive(Debug, Error)]
pub enum DeduplicationError {
    #[error("Similarity calculation failed: {0}")]
    SimilarityError(String),

    #[error("Clustering failed: {0}")]
    ClusteringError(String),

    #[error("Merge conflict: {0}")]
    MergeConflict(String),
}
```

### 8. Dutch Name Normalization

File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/normalization.rs`

```rust
//! Dutch name normalization and comparison
//!
//! This module handles Dutch-specific name patterns including:
//! - Tussenvoegsels (van, van der, de, ten, etc.)
//! - Title variations (dr. vs dr)
//! - Phonetic matching for similar names

use std::collections::HashSet;

/// Dutch name normalizer
pub struct DutchNameNormalizer {
    /// Common Dutch prefixes (tussenvoegsels)
    prefixes: HashSet<&'static str>,
}

impl DutchNameNormalizer {
    /// Create a new normalizer with standard Dutch prefixes
    pub fn new() -> Self {
        let prefixes = [
            "van", "van der", "van de", "van den", "de", "den", "ter", "ten",
            "in", "tot", "bij", "onder", "op", "over", "uit", "uijt",
        ].into_iter().collect();

        Self { prefixes }
    }

    /// Normalize a Dutch name for comparison
    ///
    /// This function:
    /// 1. Converts to lowercase
    /// 2. Removes title suffixes (dr., prof., mr., ing., ir.)
    /// 3. Standardizes spacing around prefixes
    /// 4. Trims whitespace
    pub fn normalize(&self, name: &str) -> String {
        let mut normalized = name.to_lowercase();

        // Remove title suffixes
        for title in &["dr.", "dr", "prof.", "prof", "mr.", "mr", "ing.", "ing", "ir.", "ir"] {
            if normalized.ends_with(title) {
                normalized = normalized[..normalized.len() - title.len()].to_string();
            }
        }

        // Trim and normalize whitespace
        normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");

        normalized
    }

    /// Normalize a name with prefix handling
    ///
    /// Treats prefixes case-insensitively for comparison purposes.
    /// "Jan de Vries" and "jan de vries" will produce the same normalized form.
    pub fn normalize_with_prefix(&self, name: &str) -> String {
        let normalized = self.normalize(name);

        // Ensure consistent spacing around prefixes
        let words: Vec<&str> = normalized.split_whitespace().collect();
        let mut result = Vec::new();

        for (i, word) in words.iter().enumerate() {
            if i > 0 && self.prefixes.contains(word) {
                // Prefix - add without space to previous word for sorting purposes
                // This is a simplification; production may need more sophisticated handling
            }
            result.push(*word);
        }

        result.join(" ")
    }

    /// Compare two Dutch names for equivalence
    pub fn are_equivalent(&self, name1: &str, name2: &str) -> bool {
        self.normalize_with_prefix(name1) == self.normalize_with_prefix(name2)
    }

    /// Calculate similarity between two names
    ///
    /// Returns a value between 0.0 (no similarity) and 1.0 (identical)
    pub fn similarity(&self, name1: &str, name2: &str) -> f32 {
        let norm1 = self.normalize_with_prefix(name1);
        let norm2 = self.normalize_with_prefix(name2);

        if norm1 == norm2 {
            return 1.0;
        }

        // Simple Jaro-Winkler-like calculation
        // Full implementation will use strsim crate in Section 05
        jaro_winkler_similarity(&norm1, &norm2)
    }
}

impl Default for DutchNameNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of name comparison
#[derive(Debug, Clone, PartialEq)]
pub enum NameComparison {
    /// Names are identical after normalization
    Identical,
    
    /// Names are very similar (> 0.9 Jaro-Winkler)
    VerySimilar,
    
    /// Names are somewhat similar (> 0.7 Jaro-Winkler)
    Similar,
    
    /// Names are different
    Different,
}

/// Placeholder for Jaro-Winkler similarity
/// Full implementation will be in Section 05 using strsim crate
fn jaro_winkler_similarity(s1: &str, s2: &str) -> f32 {
    if s1 == s2 {
        return 1.0;
    }

    // Simple character overlap for placeholder
    let chars1: HashSet<char> = s1.chars().collect();
    let chars2: HashSet<char> = s2.chars().collect();
    
    let intersection = chars1.intersection(&chars2).count();
    let union = chars1.union(&chars2).count();
    
    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dutch_prefix_normalization() {
        let normalizer = DutchNameNormalizer::new();
        
        // Test that prefixes are handled case-insensitively
        assert_eq!(
            normalizer.normalize("Jan de Vries"),
            normalizer.normalize("jan de vries")
        );
    }

    #[test]
    fn test_title_removal() {
        let normalizer = DutchNameNormalizer::new();
        
        assert_eq!(normalizer.normalize("dr. Jan de Vries"), "jan de vries");
        assert_eq!(normalizer.normalize("prof. Marie Jansen"), "marie jansen");
    }

    #[test]
    fn test_normalization_is_idempotent() {
        let normalizer = DutchNameNormalizer::new();
        let name = "dr. Jan van der Berg";
        
        assert_eq!(
            normalizer.normalize(&normalizer.normalize(name)),
            normalizer.normalize(name)
        );
    }
}
```

## Tests

### Unit Tests

From the TDD plan, the following tests should be implemented in this section:

```rust
// In types.rs tests
#[test]
fn test_person_stakeholder_creates_valid_entity() {
    let person = PersonStakeholder::new("Jan de Vries".to_string(), 0.95);
    
    assert_eq!(person.entity.entity_type, EntityType::Person);
    assert_eq!(person.entity.name, "Jan de Vries");
    assert_eq!(person.entity.confidence, 0.95);
}

#[test]
fn test_person_stakeholder_metadata_accessors() {
    let person = PersonStakeholder::new("Jan de Vries".to_string(), 0.95)
        .with_title("dr.")
        .with_role("minister")
        .with_department("MinFin");
    
    assert_eq!(person.title(), Some("dr."));
    assert_eq!(person.role(), Some("minister"));
    assert_eq!(person.department(), Some("MinFin"));
}

#[test]
fn test_organization_stakeholder_creates_valid_entity() {
    let org = OrganizationStakeholder::new("Ministerie van Financiën".to_string(), 0.98);
    
    assert_eq!(org.entity.entity_type, EntityType::Organization);
    assert_eq!(org.entity.name, "Ministerie van Financiën");
}

#[test]
fn test_organization_stakeholder_metadata() {
    let org = OrganizationStakeholder::new("MinFin".to_string(), 0.95)
        .with_short_name("MinFin")
        .with_org_type(OrgType::Ministry);
    
    assert_eq!(org.short_name(), Some("MinFin"));
    assert_eq!(org.org_type(), Some(OrgType::Ministry));
}

// In extractor.rs tests
#[test]
fn test_stakeholder_extractor_is_object_safe() {
    // Compile-only test - verifies trait can be boxed
    let _: Option<Box<dyn StakeholderExtractor>> = None;
}

// In normalization.rs tests
#[test]
fn test_dutch_prefix_normalization() {
    let normalizer = DutchNameNormalizer::new();
    
    assert_eq!(
        normalizer.normalize("Jan de Vries"),
        normalizer.normalize("jan de vries")
    );
}

#[test]
fn test_dutch_name_equivalence() {
    let normalizer = DutchNameNormalizer::new();
    
    assert!(normalizer.are_equivalent("Jan de Vries", "jan de vries"));
    assert!(normalizer.are_equivalent("dr. Jan de Vries", "jan de vries"));
}

#[test]
fn test_confidence_scores_always_valid() {
    // Property test stub - confidence should always be 0.0-1.0
    for confidence in [0.0, 0.5, 0.9, 1.0] {
        let person = PersonStakeholder::new("Test".to_string(), confidence);
        assert!(person.entity.confidence >= 0.0 && person.entity.confidence <= 1.0);
    }
}
```

## Success Criteria

After implementing this section:

1. All types compile without errors
2. `StakeholderExtractor` trait is object-safe (can be boxed)
3. Error types cover expected failure modes
4. Name normalization handles Dutch prefixes correctly
5. All unit tests pass
6. Module is exported through `iou-ai` lib.rs

## Next Steps

After this section is complete:
- Section 02 (Baseline Extraction) depends on these types
- Section 04 (LLM Extractor) depends on these types
- Section 05 (Normalization) extends the name normalization module