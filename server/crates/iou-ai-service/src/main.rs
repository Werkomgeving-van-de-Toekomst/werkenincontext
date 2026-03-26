//! Aparte AI-service: `/v1/chat` (cloud LLM) en `/v1/slm/chat` (OpenAI-compatibele SLM, o.a. Ollama).
//!
//! Zie `docs/architecture/ai-service.md` en `docs/architecture/ollama-models.md` (aanbevolen lichte modellen).

use axum::{
    extract::State,
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use iou_ai::{create_provider_from_env, create_slm_provider_from_env, ChatMessage, LlmBackend, LlmError};
use serde::Serialize;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    llm: Arc<dyn LlmBackend + Send + Sync>,
    slm: Arc<dyn LlmBackend + Send + Sync>,
    service_token: Option<String>,
}

#[derive(serde::Deserialize)]
struct ChatRequest {
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatResponse {
    content: String,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

enum ServiceError {
    Llm(LlmError),
    NotConfigured(&'static str),
    Unauthorized,
}

impl From<LlmError> for ServiceError {
    fn from(e: LlmError) -> Self {
        Self::Llm(e)
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ServiceError::Llm(e) => (StatusCode::BAD_GATEWAY, e.to_string()),
            ServiceError::NotConfigured(which) => (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("{which} is niet geconfigureerd (zie env LLM_* / SLM_*)"),
            ),
            ServiceError::Unauthorized => (StatusCode::UNAUTHORIZED, "Ongeldige of ontbrekende service-token".into()),
        };
        (status, Json(ErrorBody { error: msg })).into_response()
    }
}

fn check_service_token(state: &AppState, headers: &HeaderMap) -> Result<(), ServiceError> {
    let Some(expected) = state.service_token.as_ref() else {
        return Ok(());
    };
    if expected.is_empty() {
        return Ok(());
    }
    let got = headers
        .get("x-iou-ai-token")
        .and_then(|v| v.to_str().ok());
    if got == Some(expected.as_str()) {
        Ok(())
    } else {
        Err(ServiceError::Unauthorized)
    }
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok", "service": "iou-ai-service" }))
}

async fn chat_llm(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, ServiceError> {
    check_service_token(&state, &headers)?;
    if !state.llm.is_configured() {
        return Err(ServiceError::NotConfigured("Primaire LLM"));
    }
    let content = state.llm.chat(&body.messages).await?;
    Ok(Json(ChatResponse { content }))
}

async fn chat_slm(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, ServiceError> {
    check_service_token(&state, &headers)?;
    if !state.slm.is_configured() {
        return Err(ServiceError::NotConfigured("SLM"));
    }
    let content = state.slm.chat(&body.messages).await?;
    Ok(Json(ChatResponse { content }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let llm = create_provider_from_env().map_err(|e| anyhow::anyhow!(e))?;
    let slm = create_slm_provider_from_env().map_err(|e| anyhow::anyhow!(e))?;

    let service_token = std::env::var("IOU_AI_SERVICE_TOKEN").ok().filter(|s| !s.is_empty());

    let state = Arc::new(AppState {
        llm: Arc::from(llm),
        slm: Arc::from(slm),
        service_token,
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/chat", post(chat_llm))
        .route("/v1/slm/chat", post(chat_slm))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let host = std::env::var("IOU_AI_SERVICE_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("IOU_AI_SERVICE_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8090);
    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "iou-ai-service luistert");
    axum::serve(listener, app).await?;
    Ok(())
}
