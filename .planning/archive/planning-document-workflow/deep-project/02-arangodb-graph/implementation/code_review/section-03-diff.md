diff --git a/crates/iou-core/src/graphrag/error.rs b/crates/iou-core/src/graphrag/error.rs
new file mode 100644
index 0000000..54e925c
--- /dev/null
+++ b/crates/iou-core/src/graphrag/error.rs
@@ -0,0 +1,230 @@
+//! Error types for graph store operations
+//!
+//! Provides centralized error handling for ArangoDB persistence operations.
+
+use thiserror::Error;
+use uuid::Uuid;
+
+/// Centralized error type for graph store operations
+#[derive(Debug, Error)]
+pub enum StoreError {
+    /// Connection or authentication error
+    #[error("Connection error: {0}")]
+    Connection(String),
+
+    /// Query execution error
+    #[error("Query error: {0}")]
+    Query(String),
+
+    /// Entity not found in database
+    #[error("Entity not found: {0}")]
+    EntityNotFound(Uuid),
+
+    /// Relationship not found in database
+    #[error("Relationship not found: {0}")]
+    RelationshipNotFound(Uuid),
+
+    /// Community not found in database
+    #[error("Community not found: {0}")]
+    CommunityNotFound(Uuid),
+
+    /// Unique constraint violation (e.g., duplicate canonical_name)
+    #[error("Unique constraint violation: {0}")]
+    UniqueViolation(String),
+
+    /// ArangoDB-specific error with error code and message
+    #[error("ArangoDB error [{code}]: {message}")]
+    Arango { code: u16, message: String },
+
+    /// HTTP client error
+    #[error("HTTP client error: {0}")]
+    HttpClient(String),
+
+    /// Insufficient permission for operation
+    #[error("Insufficient permission to {operation}: {permission:?}")]
+    PermissionDenied {
+        permission: String,
+        operation: String,
+    },
+
+    /// Serialization/deserialization error
+    #[error("Serialization error: {0}")]
+    Serialization(String),
+
+    /// Invalid server response
+    #[error("Invalid server response: {0}")]
+    InvalidServer(String),
+}
+
+impl From<arangors::ClientError> for StoreError {
+    fn from(err: arangors::ClientError) -> Self {
+        match err {
+            arangors::ClientError::InsufficientPermission { permission, operation } => {
+                StoreError::PermissionDenied {
+                    permission: format!("{:?}", permission),
+                    operation,
+                }
+            }
+            arangors::ClientError::InvalidServer(msg) => StoreError::InvalidServer(msg),
+            arangors::ClientError::Arango(arango_err) => StoreError::Arango {
+                code: arango_err.error_num(),
+                message: arango_err.message().to_string(),
+            },
+            arangors::ClientError::HttpClient(msg) => StoreError::Connection(msg),
+            arangors::ClientError::Serde(e) => StoreError::Serialization(e.to_string()),
+        }
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn error_display_entity_not_found() {
+        let id = Uuid::new_v4();
+        let err = StoreError::EntityNotFound(id);
+        let display = format!("{}", err);
+
+        assert!(display.contains("Entity not found"));
+        assert!(display.contains(&id.to_string()));
+    }
+
+    #[test]
+    fn error_display_relationship_not_found() {
+        let id = Uuid::new_v4();
+        let err = StoreError::RelationshipNotFound(id);
+        let display = format!("{}", err);
+
+        assert!(display.contains("Relationship not found"));
+        assert!(display.contains(&id.to_string()));
+    }
+
+    #[test]
+    fn error_display_community_not_found() {
+        let id = Uuid::new_v4();
+        let err = StoreError::CommunityNotFound(id);
+        let display = format!("{}", err);
+
+        assert!(display.contains("Community not found"));
+        assert!(display.contains(&id.to_string()));
+    }
+
+    #[test]
+    fn error_display_connection() {
+        let err = StoreError::Connection("connection refused".to_string());
+        let display = format!("{}", err);
+
+        assert_eq!(display, "Connection error: connection refused");
+    }
+
+    #[test]
+    fn error_display_query() {
+        let err = StoreError::Query("syntax error".to_string());
+        let display = format!("{}", err);
+
+        assert_eq!(display, "Query error: syntax error");
+    }
+
+    #[test]
+    fn error_display_unique_violation() {
+        let err = StoreError::UniqueViolation("duplicate canonical_name".to_string());
+        let display = format!("{}", err);
+
+        assert_eq!(display, "Unique constraint violation: duplicate canonical_name");
+    }
+
+    #[test]
+    fn error_display_arango() {
+        let err = StoreError::Arango {
+            code: 1200,
+            message: "duplicate key".to_string(),
+        };
+        let display = format!("{}", err);
+
+        assert!(display.contains("ArangoDB error"));
+        assert!(display.contains("1200"));
+        assert!(display.contains("duplicate key"));
+    }
+
+    #[test]
+    fn error_display_http_client() {
+        let err = StoreError::HttpClient("timeout".to_string());
+        let display = format!("{}", err);
+
+        assert_eq!(display, "HTTP client error: timeout");
+    }
+
+    #[test]
+    fn error_display_permission_denied() {
+        let err = StoreError::PermissionDenied {
+            permission: "rw".to_string(),
+            operation: "insert".to_string(),
+        };
+        let display = format!("{}", err);
+
+        assert!(display.contains("Insufficient permission"));
+        assert!(display.contains("insert"));
+    }
+
+    #[test]
+    fn error_display_serialization() {
+        let err = StoreError::Serialization("invalid JSON".to_string());
+        let display = format!("{}", err);
+
+        assert_eq!(display, "Serialization error: invalid JSON");
+    }
+
+    #[test]
+    fn error_display_invalid_server() {
+        let err = StoreError::InvalidServer("not an ArangoDB instance".to_string());
+        let display = format!("{}", err);
+
+        assert_eq!(display, "Invalid server response: not an ArangoDB instance");
+    }
+
+    #[test]
+    fn store_error_is_send_and_sync() {
+        // Verify that StoreError implements required trait bounds for use in async contexts
+        fn assert_send_sync<T: Send + Sync>() {}
+        assert_send_sync::<StoreError>();
+    }
+
+    #[test]
+    fn store_error_matches_on_connection() {
+        let err = StoreError::Connection("test".to_string());
+        assert!(matches!(err, StoreError::Connection(_)));
+    }
+
+    #[test]
+    fn store_error_matches_on_query() {
+        let err = StoreError::Query("test".to_string());
+        assert!(matches!(err, StoreError::Query(_)));
+    }
+
+    #[test]
+    fn store_error_matches_on_entity_not_found() {
+        let id = Uuid::new_v4();
+        let err = StoreError::EntityNotFound(id);
+        assert!(matches!(err, StoreError::EntityNotFound(_)));
+    }
+
+    #[test]
+    fn store_error_from_http_client() {
+        // The HttpClient variant maps to Connection error
+        let err = StoreError::from(arangors::ClientError::HttpClient(
+            "connection failed".to_string(),
+        ));
+        assert!(matches!(err, StoreError::Connection(_)));
+    }
+
+    #[test]
+    fn store_error_from_serialization() {
+        // Serde errors map to Serialization error
+        // Create a serde_json error by deserializing invalid JSON
+        let serde_err = serde_json::from_str::<serde_json::Value>("{invalid json}")
+            .unwrap_err();
+        let err = StoreError::from(arangors::ClientError::Serde(serde_err));
+        assert!(matches!(err, StoreError::Serialization(_)));
+    }
+}
diff --git a/crates/iou-core/src/graphrag/mod.rs b/crates/iou-core/src/graphrag/mod.rs
new file mode 100644
index 0000000..99fb74d
--- /dev/null
+++ b/crates/iou-core/src/graphrag/mod.rs
@@ -0,0 +1,25 @@
+//! GraphRAG module for knowledge graph persistence and analysis
+//!
+//! This module provides:
+//! - Type definitions for entities, relationships, and communities
+//! - Error types for graph store operations
+//! - ArangoDB-based persistence layer (TODO)
+
+pub mod error;
+pub mod types;
+
+// Re-export common types
+pub use types::{
+    Entity,
+    EntityType,
+    Relationship,
+    RelationshipType,
+    Community,
+    DomainRelation,
+    DomainRelationType,
+    DiscoveryMethod,
+    ContextVector,
+    GraphAnalysisResult,
+};
+
+pub use error::StoreError;
diff --git a/crates/iou-core/src/graphrag/types.rs b/crates/iou-core/src/graphrag/types.rs
new file mode 100644
index 0000000..356a8ee
--- /dev/null
+++ b/crates/iou-core/src/graphrag/types.rs
@@ -0,0 +1,243 @@
+//! GraphRAG types voor kennisgraaf en relaties
+//!
+//! Dit module ondersteunt de automatische detectie van relaties tussen
+//! informatiedomeinen via Named Entity Recognition en graph-analyse.
+
+use chrono::{DateTime, Utc};
+use serde::{Deserialize, Serialize};
+use strum::{Display, EnumString};
+use uuid::Uuid;
+
+/// Entiteit geëxtraheerd uit tekst
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct Entity {
+    pub id: Uuid,
+    pub name: String,
+    pub entity_type: EntityType,
+    pub canonical_name: Option<String>,
+    pub description: Option<String>,
+    pub confidence: f32,
+    pub source_domain_id: Option<Uuid>,
+    pub metadata: serde_json::Value,
+    pub created_at: DateTime<Utc>,
+}
+
+/// Type entiteit (NER labels)
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
+#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
+pub enum EntityType {
+    /// Persoon
+    #[strum(serialize = "PER")]
+    Person,
+    /// Organisatie
+    #[strum(serialize = "ORG")]
+    Organization,
+    /// Locatie
+    #[strum(serialize = "LOC")]
+    Location,
+    /// Wettelijke verwijzing
+    #[strum(serialize = "LAW")]
+    Law,
+    /// Datum
+    #[strum(serialize = "DATE")]
+    Date,
+    /// Geldbedrag
+    #[strum(serialize = "MONEY")]
+    Money,
+    /// Beleidsterme
+    #[strum(serialize = "POLICY")]
+    Policy,
+    /// Overig
+    #[strum(serialize = "MISC")]
+    Miscellaneous,
+}
+
+/// Relatie tussen twee entiteiten
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct Relationship {
+    pub id: Uuid,
+    pub source_entity_id: Uuid,
+    pub target_entity_id: Uuid,
+    pub relationship_type: RelationshipType,
+    pub weight: f32,
+    pub confidence: f32,
+    pub context: Option<String>,
+    pub source_domain_id: Option<Uuid>,
+    pub created_at: DateTime<Utc>,
+}
+
+/// Type relatie tussen entiteiten
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
+#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
+pub enum RelationshipType {
+    /// Werkt voor / is onderdeel van
+    WorksFor,
+    /// Gevestigd in / bevindt zich in
+    LocatedIn,
+    /// Is onderwerp van (wet/beleid)
+    SubjectTo,
+    /// Verwijst naar
+    RefersTo,
+    /// Heeft betrekking op
+    RelatesTo,
+    /// Is eigenaar van
+    OwnerOf,
+    /// Rapporteert aan
+    ReportsTo,
+    /// Werkt samen met
+    CollaboratesWith,
+    /// Volgt op
+    Follows,
+    /// Is onderdeel van
+    PartOf,
+    /// Onbekend
+    Unknown,
+}
+
+/// Community (cluster) van gerelateerde entiteiten
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct Community {
+    pub id: Uuid,
+    pub name: String,
+    pub description: Option<String>,
+    pub level: i32,
+    pub parent_community_id: Option<Uuid>,
+    pub member_entity_ids: Vec<Uuid>,
+    pub summary: Option<String>,
+    pub keywords: Vec<String>,
+    pub created_at: DateTime<Utc>,
+}
+
+/// Relatie tussen twee informatiedomeinen (afgeleid van entiteiten)
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct DomainRelation {
+    pub id: Uuid,
+    pub from_domain_id: Uuid,
+    pub to_domain_id: Uuid,
+    pub relation_type: DomainRelationType,
+    pub strength: f32,
+    pub discovery_method: DiscoveryMethod,
+    pub shared_entities: Vec<Uuid>,
+    pub explanation: Option<String>,
+    pub created_at: DateTime<Utc>,
+}
+
+/// Type relatie tussen domeinen
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
+#[serde(rename_all = "snake_case")]
+pub enum DomainRelationType {
+    /// Domeinen delen dezelfde entiteiten
+    SharedEntities,
+    /// Domeinen behoren tot dezelfde community
+    SameCommunity,
+    /// Domeinen zijn semantisch vergelijkbaar
+    SemanticSimilarity,
+    /// Domeinen overlappen in tijd
+    TemporalOverlap,
+    /// Domeinen hebben dezelfde stakeholders
+    SharedStakeholders,
+    /// Handmatig gelinkt
+    ManualLink,
+}
+
+/// Methode waarmee relatie is ontdekt
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "snake_case")]
+pub enum DiscoveryMethod {
+    /// Automatisch via GraphRAG
+    Automatic,
+    /// Handmatig door gebruiker
+    Manual,
+    /// Via AI suggestie (geaccepteerd)
+    AiSuggestion,
+    /// Via regelgebaseerde matching
+    RuleBased,
+}
+
+/// Context vector voor semantische zoekfunctionaliteit
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct ContextVector {
+    pub id: Uuid,
+    pub domain_id: Uuid,
+    pub embedding: Vec<f32>,
+    pub model_name: String,
+    pub model_version: String,
+    pub created_at: DateTime<Utc>,
+}
+
+impl ContextVector {
+    /// Bereken cosine similarity met andere vector
+    pub fn cosine_similarity(&self, other: &ContextVector) -> f32 {
+        if self.embedding.len() != other.embedding.len() {
+            return 0.0;
+        }
+
+        let dot_product: f32 = self
+            .embedding
+            .iter()
+            .zip(other.embedding.iter())
+            .map(|(a, b)| a * b)
+            .sum();
+
+        let norm_a: f32 = self.embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
+        let norm_b: f32 = other.embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
+
+        if norm_a == 0.0 || norm_b == 0.0 {
+            return 0.0;
+        }
+
+        dot_product / (norm_a * norm_b)
+    }
+}
+
+/// Resultaat van GraphRAG analyse
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct GraphAnalysisResult {
+    pub domain_id: Uuid,
+    pub entities: Vec<Entity>,
+    pub relationships: Vec<Relationship>,
+    pub communities: Vec<Uuid>,
+    pub related_domains: Vec<DomainRelation>,
+    pub keywords: Vec<String>,
+    pub analyzed_at: DateTime<Utc>,
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_cosine_similarity() {
+        let vec1 = ContextVector {
+            id: Uuid::new_v4(),
+            domain_id: Uuid::new_v4(),
+            embedding: vec![1.0, 0.0, 0.0],
+            model_name: "test".to_string(),
+            model_version: "1.0".to_string(),
+            created_at: Utc::now(),
+        };
+
+        let vec2 = ContextVector {
+            embedding: vec![1.0, 0.0, 0.0],
+            ..vec1.clone()
+        };
+
+        let vec3 = ContextVector {
+            embedding: vec![0.0, 1.0, 0.0],
+            ..vec1.clone()
+        };
+
+        // Identieke vectoren: similarity = 1.0
+        assert!((vec1.cosine_similarity(&vec2) - 1.0).abs() < 0.001);
+
+        // Orthogonale vectoren: similarity = 0.0
+        assert!(vec1.cosine_similarity(&vec3).abs() < 0.001);
+    }
+
+    #[test]
+    fn test_entity_type_serialization() {
+        let et = EntityType::Organization;
+        let json = serde_json::to_string(&et).unwrap();
+        assert_eq!(json, "\"ORGANIZATION\"");
+    }
+}
