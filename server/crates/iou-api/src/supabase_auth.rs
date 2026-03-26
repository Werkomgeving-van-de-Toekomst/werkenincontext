//! Supabase Auth Client
//!
//! Provides authentication operations using Supabase Auth.

use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supabase Auth client
#[derive(Clone)]
pub struct SupabaseAuth {
    /// Supabase project URL
    base_url: String,

    /// API key
    api_key: String,

    /// HTTP client
    http_client: Client,
}

impl SupabaseAuth {
    /// Create a new Supabase Auth client
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            http_client: Client::new(),
        }
    }

    /// Get the auth endpoint URL
    fn auth_url(&self, path: &str) -> String {
        format!("{}/auth/v1/{}", self.base_url, path.trim_start_matches('/'))
    }

    /// Get default headers for requests
    fn headers(&self) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert("apikey", self.api_key.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }

    /// Sign up a new user with email and credentials
    pub async fn sign_up(&self, request: SignUpRequest) -> Result<AuthResponse, AuthError> {
        let url = self.auth_url("signup");

        let payload = SignUpPayload {
            email: request.email,
            credentials: request.credentials,
            data: request.user_metadata,
            phone: request.phone,
        };

        let response = self
            .http_client
            .post(&url)
            .headers(self.headers())
            .json(&payload)
            .send()
            .await
            .map_err(|e| AuthError::ConnectionError(e.to_string()))?;

        self.handle_auth_response(response).await
    }

    /// Sign in with email and credentials
    pub async fn sign_in(&self, request: SignInRequest) -> Result<AuthResponse, AuthError> {
        let url = self.auth_url("token?grant_type=credentials");

        let payload = SignInPayload {
            email: request.email,
            credentials: request.credentials,
        };

        let response = self
            .http_client
            .post(&url)
            .headers(self.headers())
            .json(&payload)
            .send()
            .await
            .map_err(|e| AuthError::ConnectionError(e.to_string()))?;

        self.handle_auth_response(response).await
    }

    /// Send a reset email for credentials
    pub async fn reset_credentials(&self, request: CredentialsResetRequest) -> Result<CredentialsResetResponse, AuthError> {
        let url = self.auth_url("recover");

        let payload = serde_json::json!({
            "email": request.email,
        });

        let response = self
            .http_client
            .post(&url)
            .headers(self.headers())
            .json(&payload)
            .send()
            .await
            .map_err(|e| AuthError::ConnectionError(e.to_string()))?;

        if response.status().is_success() {
            Ok(CredentialsResetResponse {
                email: request.email.to_string(),
                message: "Reset email sent".to_string(),
            })
        } else {
            Err(AuthError::ApiError("Failed to send reset email".to_string()))
        }
    }

    /// Get the current user (requires access token)
    pub async fn get_user(&self, access_token: &str) -> Result<User, AuthError> {
        let url = self.auth_url("user");

        let response = self
            .http_client
            .get(&url)
            .header("apikey", &self.api_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| AuthError::ConnectionError(e.to_string()))?;

        if response.status().is_success() {
            response
                .json()
                .await
                .map_err(|e| AuthError::ConnectionError(e.to_string()))
        } else {
            Err(AuthError::Unauthorized("Invalid access token".to_string()))
        }
    }

    /// Update user metadata (requires access token)
    pub async fn update_user(
        &self,
        access_token: &str,
        metadata: serde_json::Value,
    ) -> Result<User, AuthError> {
        let url = self.auth_url("user");

        let payload = serde_json::json!({ "data": metadata });

        let response = self
            .http_client
            .put(&url)
            .header("apikey", &self.api_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AuthError::ConnectionError(e.to_string()))?;

        if response.status().is_success() {
            response
                .json()
                .await
                .map_err(|e| AuthError::ConnectionError(e.to_string()))
        } else {
            Err(AuthError::ApiError("Failed to update user".to_string()))
        }
    }

    /// Handle auth response from Supabase
    async fn handle_auth_response(&self, response: reqwest::Response) -> Result<AuthResponse, AuthError> {
        let status = response.status();

        if status.is_success() {
            let body: AuthResponseInner = response
                .json()
                .await
                .map_err(|e| AuthError::ConnectionError(e.to_string()))?;

            Ok(AuthResponse {
                access_token: body.access_token,
                refresh_token: body.refresh_token,
                expires_in: body.expires_in,
                expires_at: Utc::now() + chrono::Duration::seconds(body.expires_in as i64),
                user: body.user,
            })
        } else {
            let error: AuthErrorResponse = response
                .json()
                .await
                .unwrap_or(AuthErrorResponse {
                    error: "Unknown error".to_string(),
                    error_description: format!("Status: {}", status),
                });

            match status.as_u16() {
                400 => Err(AuthError::BadRequest(error.error_description)),
                401 => Err(AuthError::Unauthorized(error.error_description)),
                422 => Err(AuthError::UnprocessableEntity(error.error_description)),
                _ => Err(AuthError::ApiError(error.error_description)),
            }
        }
    }

    /// Verify an access token
    pub async fn verify_token(&self, access_token: &str) -> Result<User, AuthError> {
        self.get_user(access_token).await
    }
}

// ============================================
// Request Types
// ============================================

/// Request for signing up a new user
#[derive(Debug, Clone, Serialize)]
pub struct SignUpRequest {
    pub email: String,
    pub credentials: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

/// Request for signing in
#[derive(Debug, Clone, Serialize)]
pub struct SignInRequest {
    pub email: String,
    pub credentials: String,
}

/// Request for credentials reset
#[derive(Debug, Clone, Serialize)]
pub struct CredentialsResetRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize)]
struct SignUpPayload {
    email: String,
    credentials: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SignInPayload {
    email: String,
    credentials: String,
}

// ============================================
// Response Types
// ============================================

/// Response from sign up or sign in
#[derive(Debug, Clone, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub expires_at: DateTime<Utc>,
    pub user: User,
}

#[derive(Debug, Clone, Deserialize)]
struct AuthResponseInner {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user: User,
}

/// User information from Supabase Auth
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub email_confirmed_at: Option<DateTime<Utc>>,
    pub phone_confirmed_at: Option<DateTime<Utc>>,
    pub last_sign_in_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub user_metadata: serde_json::Value,
    #[serde(default)]
    pub app_metadata: serde_json::Value,
}

/// Response from credentials reset request
#[derive(Debug, Clone)]
pub struct CredentialsResetResponse {
    pub email: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AuthErrorResponse {
    pub error: String,
    pub error_description: String,
}

// ============================================
// Error Types
// ============================================

/// Errors that can occur during auth operations
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    #[error("User not found")]
    UserNotFound,

    #[error("Email already registered")]
    EmailAlreadyRegistered,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_client_creation() {
        let auth = SupabaseAuth::new("https://test.supabase.co", "test-key");
        assert_eq!(auth.base_url, "https://test.supabase.co");
        assert_eq!(auth.api_key, "test-key");
    }
}
