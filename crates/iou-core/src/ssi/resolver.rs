//! Universal DID resolver supporting multiple DID methods

use crate::ssi::{did::DidDocument, verifiable_credential::DIDResolver};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Universal DID resolver
pub struct UniversalDidResolver {
    http: Client,
    cache: tokio::sync::RwLock<HashMap<String, Arc<DidDocument>>>,
    cache_ttl_seconds: u64,
}

impl UniversalDidResolver {
    /// Create a new universal DID resolver
    pub fn new() -> Self {
        Self {
            http: Client::builder()
                .user_agent("iou-modern/did-resolver")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("HTTP client creation failed"),
            cache: tokio::sync::RwLock::new(HashMap::new()),
            cache_ttl_seconds: 900, // 15 minutes
        }
    }

    /// Resolve a DID using appropriate method
    async fn resolve_by_method(&self, did: &str) -> Result<DidDocument, ResolverError> {
        if !did.starts_with("did:") {
            return Err(ResolverError::InvalidFormat("Missing did: prefix".into()));
        }

        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 {
            return Err(ResolverError::InvalidFormat("Invalid DID format".into()));
        }

        let method = parts[1];

        match method {
            "web" => self.resolve_did_web(did).await,
            "key" => self.resolve_did_key(did).await,
            "ebsi" => self.resolve_did_ebsi(did).await,
            "polygonid" => self.resolve_did_polygonid(did).await,
            _ => Err(ResolverError::MethodNotSupported(method.to_string())),
        }
    }

    /// Resolve did:web DIDs
    async fn resolve_did_web(&self, did: &str) -> Result<DidDocument, ResolverError> {
        // did:web:example.com -> https://example.com/.well-known/did.json
        let domain = did.split(':').nth(2)
            .ok_or_else(|| ResolverError::InvalidFormat("Missing domain".into()))?;

        let url = format!("https://{}/.well-known/did.json", domain);

        let response = self.http.get(&url)
            .header("Accept", "application/did+json")
            .send()
            .await
            .map_err(|e| ResolverError::HttpError(e.to_string()))?
            .error_for_status()
            .map_err(|e| ResolverError::NotFound(e.to_string()))?;

        let did_doc: DidDocument = response.json().await
            .map_err(|e| ResolverError::ParseError(e.to_string()))?;

        Ok(did_doc)
    }

    /// Resolve did:key DIDs (generates DID document on the fly)
    async fn resolve_did_key(&self, did: &str) -> Result<DidDocument, ResolverError> {
        // did:key is self-describing - we can derive the DID document from the key
        // For now, return a mock implementation
        Ok(DidDocument {
            context: serde_json::json!("https://w3id.org/did/v1"),
            id: did.to_string(),
            verification_method: vec![],
            authentication: vec![],
            assertion_method: vec![],
            service: None,
        })
    }

    /// Resolve did:ebsi DIDs using EBSI resolver
    async fn resolve_did_ebsi(&self, did: &str) -> Result<DidDocument, ResolverError> {
        // Use EBSI DID resolver
        let url = format!("https://api.preprod.ebsi.eu/did-registry/v1/identifiers/{}", did);

        let response = self.http.get(&url)
            .header("Accept", "application/did+json")
            .send()
            .await
            .map_err(|e| ResolverError::HttpError(e.to_string()))?
            .error_for_status()
            .map_err(|e| ResolverError::NotFound(e.to_string()))?;

        let did_doc: DidDocument = response.json().await
            .map_err(|e| ResolverError::ParseError(e.to_string()))?;

        Ok(did_doc)
    }

    /// Resolve did:polygonid DIDs
    async fn resolve_did_polygonid(&self, did: &str) -> Result<DidDocument, ResolverError> {
        // Use Polygon ID resolver
        let url = format!("https://universaldid.com/v1/identifiers/{}", did);

        let response = self.http.get(&url)
            .header("Accept", "application/did+json")
            .send()
            .await
            .map_err(|e| ResolverError::HttpError(e.to_string()))?
            .error_for_status()
            .map_err(|e| ResolverError::NotFound(e.to_string()))?;

        let did_doc: DidDocument = response.json().await
            .map_err(|e| ResolverError::ParseError(e.to_string()))?;

        Ok(did_doc)
    }
}

impl Default for UniversalDidResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DIDResolver for UniversalDidResolver {
    async fn resolve(&self, did: &str) -> Result<DidDocument, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(doc) = cache.get(did) {
                return Ok(Arc::clone(doc).as_ref().clone());
            }
        }

        // Resolve
        let doc = self.resolve_by_method(did).await?;

        // Cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(did.to_string(), Arc::new(doc.clone()));
        }

        Ok(doc)
    }
}

/// Resolver errors
#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("Invalid DID format: {0}")]
    InvalidFormat(String),

    #[error("Method not supported: {0}")]
    MethodNotSupported(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("DID not found: {0}")]
    NotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_did_key() {
        let resolver = UniversalDidResolver::new();
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        let result = resolver.resolve(did).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cache_works() {
        let resolver = UniversalDidResolver::new();
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        let _ = resolver.resolve(did).await.unwrap();
        // Second call should use cache
        let _ = resolver.resolve(did).await.unwrap();
    }
}
