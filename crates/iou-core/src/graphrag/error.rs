//! Error types for graph store operations
//!
//! Provides centralized error handling for ArangoDB persistence operations.

use thiserror::Error;
use uuid::Uuid;

/// Centralized error type for graph store operations
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StoreError {
    /// Connection or authentication error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Query execution error
    #[error("Query error: {0}")]
    Query(String),

    /// Entity not found in database
    #[error("Entity not found: {0}")]
    EntityNotFound(Uuid),

    /// Relationship not found in database
    #[error("Relationship not found: {0}")]
    RelationshipNotFound(Uuid),

    /// Community not found in database
    #[error("Community not found: {0}")]
    CommunityNotFound(Uuid),

    /// Unique constraint violation (e.g., duplicate canonical_name)
    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),

    /// ArangoDB-specific error with error code and message
    #[error("ArangoDB error [{code}]: {message}")]
    Arango { code: u16, message: String },

    /// HTTP client error
    #[error("HTTP client error: {0}")]
    HttpClient(String),

    /// Insufficient permission for operation
    #[error("Insufficient permission to {operation}: {permission:?}")]
    PermissionDenied {
        permission: String,
        operation: String,
    },

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid server response
    #[error("Invalid server response: {0}")]
    InvalidServer(String),
}

impl From<arangors::ClientError> for StoreError {
    fn from(err: arangors::ClientError) -> Self {
        match err {
            arangors::ClientError::InsufficientPermission { permission, operation } => {
                StoreError::PermissionDenied {
                    permission: format!("{:?}", permission),
                    operation,
                }
            }
            arangors::ClientError::InvalidServer(msg) => StoreError::InvalidServer(msg),
            arangors::ClientError::Arango(arango_err) => StoreError::Arango {
                code: arango_err.error_num(),
                message: arango_err.message().to_string(),
            },
            arangors::ClientError::HttpClient(msg) => StoreError::Connection(msg),
            arangors::ClientError::Serde(e) => StoreError::Serialization(e.to_string()),
        }
    }
}

impl From<mobc::Error<arangors::ClientError>> for StoreError {
    fn from(err: mobc::Error<arangors::ClientError>) -> Self {
        match err {
            mobc::Error::Inner(e) => e.into(),
            mobc::Error::Timeout => StoreError::Connection("Connection pool timeout".to_string()),
            mobc::Error::BadConn => StoreError::Connection("Bad connection".to_string()),
            mobc::Error::PoolClosed => StoreError::Connection("Connection pool closed".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_entity_not_found() {
        let id = Uuid::new_v4();
        let err = StoreError::EntityNotFound(id);
        let display = format!("{}", err);

        assert!(display.contains("Entity not found"));
        assert!(display.contains(&id.to_string()));
    }

    #[test]
    fn error_display_relationship_not_found() {
        let id = Uuid::new_v4();
        let err = StoreError::RelationshipNotFound(id);
        let display = format!("{}", err);

        assert!(display.contains("Relationship not found"));
        assert!(display.contains(&id.to_string()));
    }

    #[test]
    fn error_display_community_not_found() {
        let id = Uuid::new_v4();
        let err = StoreError::CommunityNotFound(id);
        let display = format!("{}", err);

        assert!(display.contains("Community not found"));
        assert!(display.contains(&id.to_string()));
    }

    #[test]
    fn error_display_connection() {
        let err = StoreError::Connection("connection refused".to_string());
        let display = format!("{}", err);

        assert_eq!(display, "Connection error: connection refused");
    }

    #[test]
    fn error_display_query() {
        let err = StoreError::Query("syntax error".to_string());
        let display = format!("{}", err);

        assert_eq!(display, "Query error: syntax error");
    }

    #[test]
    fn error_display_unique_violation() {
        let err = StoreError::UniqueViolation("duplicate canonical_name".to_string());
        let display = format!("{}", err);

        assert_eq!(display, "Unique constraint violation: duplicate canonical_name");
    }

    #[test]
    fn error_display_arango() {
        let err = StoreError::Arango {
            code: 1200,
            message: "duplicate key".to_string(),
        };
        let display = format!("{}", err);

        assert!(display.contains("ArangoDB error"));
        assert!(display.contains("1200"));
        assert!(display.contains("duplicate key"));
    }

    #[test]
    fn error_display_http_client() {
        let err = StoreError::HttpClient("timeout".to_string());
        let display = format!("{}", err);

        assert_eq!(display, "HTTP client error: timeout");
    }

    #[test]
    fn error_display_permission_denied() {
        let err = StoreError::PermissionDenied {
            permission: "rw".to_string(),
            operation: "insert".to_string(),
        };
        let display = format!("{}", err);

        assert!(display.contains("Insufficient permission"));
        assert!(display.contains("insert"));
    }

    #[test]
    fn error_display_serialization() {
        let err = StoreError::Serialization("invalid JSON".to_string());
        let display = format!("{}", err);

        assert_eq!(display, "Serialization error: invalid JSON");
    }

    #[test]
    fn error_display_invalid_server() {
        let err = StoreError::InvalidServer("not an ArangoDB instance".to_string());
        let display = format!("{}", err);

        assert_eq!(display, "Invalid server response: not an ArangoDB instance");
    }

    #[test]
    fn store_error_is_send_and_sync() {
        // Verify that StoreError implements required trait bounds for use in async contexts
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StoreError>();
    }

    #[test]
    fn store_error_matches_on_connection() {
        let err = StoreError::Connection("test".to_string());
        assert!(matches!(err, StoreError::Connection(_)));
    }

    #[test]
    fn store_error_matches_on_query() {
        let err = StoreError::Query("test".to_string());
        assert!(matches!(err, StoreError::Query(_)));
    }

    #[test]
    fn store_error_matches_on_entity_not_found() {
        let id = Uuid::new_v4();
        let err = StoreError::EntityNotFound(id);
        assert!(matches!(err, StoreError::EntityNotFound(_)));
    }

    #[test]
    fn store_error_from_http_client() {
        // The HttpClient variant maps to Connection error
        let err = StoreError::from(arangors::ClientError::HttpClient(
            "connection failed".to_string(),
        ));
        assert!(matches!(err, StoreError::Connection(_)));
    }

    #[test]
    fn store_error_from_serialization() {
        // Serde errors map to Serialization error
        // Create a serde_json error by deserializing invalid JSON
        let serde_err = serde_json::from_str::<serde_json::Value>("{invalid json}")
            .unwrap_err();
        let err = StoreError::from(arangors::ClientError::Serde(serde_err));
        assert!(matches!(err, StoreError::Serialization(_)));
    }
}
