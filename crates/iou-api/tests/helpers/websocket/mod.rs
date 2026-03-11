//! WebSocket test helper for testing real-time updates

use uuid::Uuid;

/// WebSocket test client for connecting to the document status endpoint
pub struct WebSocketTestClient {
    pub base_url: String,
}

impl WebSocketTestClient {
    /// Create a new WebSocket test client
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the server (e.g., "ws://localhost:3000")
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    /// Create a client for a specific document
    pub fn for_document(base_url: String, document_id: Uuid) -> String {
        format!("{}/api/ws/documents/{}", base_url, document_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_test_client_creation() {
        let client = WebSocketTestClient::new("ws://localhost:3000".to_string());
        assert_eq!(client.base_url, "ws://localhost:3000");
    }

    #[test]
    fn test_for_document_url() {
        let document_id = Uuid::new_v4();
        let url = WebSocketTestClient::for_document("ws://localhost:3000".to_string(), document_id);
        assert!(url.contains(&document_id.to_string()));
    }
}
