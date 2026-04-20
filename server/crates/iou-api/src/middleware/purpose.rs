//! Purpose Binding Middleware
//!
//! Valideert dat elk data access request een geldige purpose bevat
//! volgens IHH01: "Vastlegging en uitwisseling gegevens gebonden aan doelbinding"

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use iou_core::purpose::{Purpose as CorePurpose, PurposeError, PurposeId, PurposeRegistry};
use tracing::{debug, warn};

/// Header naam voor purpose ID
pub const HEADER_PURPOSE: &str = "X-Purpose-ID";

/// Purpose state in de applicatie
#[derive(Clone)]
pub struct PurposeState {
    pub registry: Arc<PurposeRegistry>,
}

impl PurposeState {
    pub fn new(registry: Arc<PurposeRegistry>) -> Self {
        Self { registry }
    }

    pub fn with_standard_purposes() -> Self {
        Self {
            registry: Arc::new(PurposeRegistry::new()),
        }
    }
}

impl Default for PurposeState {
    fn default() -> Self {
        Self::with_standard_purposes()
    }
}

/// Purpose context (stored in request extensions)
#[derive(Debug, Clone)]
pub struct PurposeContext {
    pub purpose_id: PurposeId,
    pub purpose: CorePurpose,
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

impl PurposeContext {
    pub fn new(purpose_id: PurposeId, purpose: CorePurpose) -> Self {
        Self {
            purpose_id,
            purpose,
            validated_at: chrono::Utc::now(),
        }
    }

    pub fn has_lawful_basis(&self, basis: iou_core::purpose::LawfulBasis) -> bool {
        self.purpose.lawful_basis == basis
    }

    pub fn can_use_category(&self, category: &str) -> bool {
        self.purpose.can_use_data_category(category)
    }
}

/// Purpose validation middleware
pub async fn purpose_middleware<B>(
    State(purpose_state): State<PurposeState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let start = std::time::Instant::now();

    // Extract purpose header
    let purpose_id = extract_purpose_header(req.headers())?;

    // Validate purpose
    let purpose = match validate_purpose(&purpose_state, &purpose_id).await {
        Ok(purpose) => purpose,
        Err(e) => {
            warn!(
                purpose_id = %purpose_id,
                error = %e,
                "Purpose validation failed"
            );
            return Err(StatusCode::FORBIDDEN);
        }
    };

    // Create context and store in extensions
    let context = PurposeContext::new(purpose_id.clone(), purpose.clone());
    req.extensions_mut().insert(context);

    let duration = start.elapsed();
    if duration.as_millis() > 10 {
        warn!(
            purpose_id = %purpose_id,
            duration_ms = duration.as_millis(),
            "Purpose validation slow"
        );
    } else {
        debug!(
            purpose_id = %purpose_id,
            lawful_basis = %purpose.lawful_basis,
            duration_ms = duration.as_millis(),
            "Purpose validated"
        );
    }

    Ok(next.run(req).await)
}

fn extract_purpose_header(headers: &HeaderMap) -> Result<PurposeId, StatusCode> {
    headers
        .get(HEADER_PURPOSE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or(StatusCode::BAD_REQUEST)
}

async fn validate_purpose(
    state: &PurposeState,
    purpose_id: &str,
) -> Result<CorePurpose, PurposeError> {
    state.registry.validate(purpose_id)
}

/// Axum extractor for PurposeContext
impl<S> axum::extract::FromRequestParts<S> for PurposeContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<PurposeContext>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iou_core::purpose::LawfulBasis;

    #[test]
    fn test_extract_purpose_header_success() {
        let mut headers = HeaderMap::new();
        headers.insert(HEADER_PURPOSE, "P001".parse().unwrap());

        let result = extract_purpose_header(&headers);
        assert_eq!(result.unwrap(), "P001");
    }

    #[test]
    fn test_extract_purpose_header_missing() {
        let headers = HeaderMap::new();
        let result = extract_purpose_header(&headers);
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_purpose_context_creation() {
        let purpose = CorePurpose::new(
            "P001",
            "Test Purpose",
            "Test description",
            LawfulBasis::WettelijkeVerplichting,
            "Owner",
        );

        let context = PurposeContext::new("P001".to_string(), purpose);

        assert_eq!(context.purpose_id, "P001");
        assert_eq!(context.purpose.name, "Test Purpose");
    }

    #[test]
    fn test_purpose_state_default() {
        let state = PurposeState::default();
        let purposes = state.registry.list_all();
        assert_eq!(purposes.len(), 15);
    }
}
