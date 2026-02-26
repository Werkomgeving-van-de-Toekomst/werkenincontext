//! JWT Authentication and Authorization Middleware
//!
//! Provides JWT-based authentication with role-based access control (RBAC),
//! organization-scoped data access, and audit trail for all actions.

use std::sync::Arc;

use axum::{
    extract::{Extension, Request},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;

/// JWT secret key (should come from environment in production)
const JWT_SECRET: &[u8] = b"iou-secret-key-change-in-production";

/// JWT token expiration time (24 hours)
const TOKEN_EXPIRATION_HOURS: i64 = 24;

/// Claims structure for JWT tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// User ID
    pub sub: String,

    /// User email
    pub email: String,

    /// Organization ID
    pub org_id: String,

    /// User roles
    pub roles: Vec<String>,

    /// Token issuance time
    pub iat: i64,

    /// Token expiration time
    pub exp: i64,

    /// Token issuer
    pub iss: String,
}

/// Authentication context extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub email: String,
    pub organization_id: Uuid,
    pub roles: Vec<Role>,
}

/// Role definitions for RBAC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    // System roles
    Admin,
    Auditor,

    // Domain roles
    DomainManager,
    DomainEditor,
    DomainViewer,

    // Object roles
    ObjectCreator,
    ObjectEditor,
    ObjectApprover,

    // Compliance roles
    ComplianceOfficer,
    ComplianceReviewer,

    // Woo roles
    WooOfficer,
    WooPublisher,
}

impl Role {
    /// Check if this role has a specific permission
    pub fn has_permission(&self, permission: Permission) -> bool {
        match self {
            Role::Admin => true, // Admin has all permissions

            Role::Auditor => matches!(
                permission,
                Permission::DomainRead
                    | Permission::ObjectRead
                    | Permission::AuditView
                    | Permission::ComplianceAssess
            ),

            Role::DomainManager => matches!(
                permission,
                Permission::DomainCreate
                    | Permission::DomainRead
                    | Permission::DomainUpdate
                    | Permission::DomainArchive
                    | Permission::ObjectRead
            ),

            Role::DomainEditor => matches!(
                permission,
                Permission::DomainRead | Permission::DomainUpdate | Permission::ObjectRead
            ),

            Role::DomainViewer => matches!(permission, Permission::DomainRead | Permission::ObjectRead),

            Role::ObjectCreator => matches!(
                permission,
                Permission::ObjectCreate | Permission::ObjectRead
            ),

            Role::ObjectEditor => matches!(
                permission,
                Permission::ObjectCreate
                    | Permission::ObjectRead
                    | Permission::ObjectUpdate
            ),

            Role::ObjectApprover => matches!(
                permission,
                Permission::ObjectRead
                    | Permission::ObjectUpdate
                    | Permission::ComplianceAssess
                    | Permission::ComplianceApprove
            ),

            Role::ComplianceOfficer => matches!(
                permission,
                Permission::ComplianceAssess
                    | Permission::ComplianceApprove
                    | Permission::ObjectRead
                    | Permission::ObjectClassify
            ),

            Role::ComplianceReviewer => matches!(
                permission,
                Permission::ComplianceAssess | Permission::ObjectRead
            ),

            Role::WooOfficer => matches!(
                permission,
                Permission::ObjectRead
                    | Permission::ObjectClassify
                    | Permission::WooPublish
                    | Permission::ComplianceAssess
            ),

            Role::WooPublisher => matches!(
                permission,
                Permission::WooPublish | Permission::ObjectRead
            ),
        }
    }
}

/// Permission types for RBAC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    // Domain permissions
    DomainCreate,
    DomainRead,
    DomainUpdate,
    DomainDelete,
    DomainArchive,

    // Object permissions
    ObjectCreate,
    ObjectRead,
    ObjectUpdate,
    ObjectDelete,
    ObjectClassify,

    // Compliance permissions
    ComplianceAssess,
    ComplianceApprove,
    WooPublish,

    // Admin permissions
    UserManage,
    RoleManage,
    OrganizationManage,
    AuditView,
}

/// Authentication error types
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    ExpiredToken,
    InsufficientPermissions,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::MissingToken => write!(f, "Missing authentication token"),
            AuthError::InvalidToken => write!(f, "Invalid authentication token"),
            AuthError::ExpiredToken => write!(f, "Authentication token has expired"),
            AuthError::InsufficientPermissions => write!(f, "Insufficient permissions"),
        }
    }
}

/// JWT service for token generation and validation
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    /// Create a new JWT service
    pub fn new() -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(JWT_SECRET),
            decoding_key: DecodingKey::from_secret(JWT_SECRET),
        }
    }

    /// Create a new JWT token for a user
    pub fn create_token(
        &self,
        user_id: Uuid,
        email: &str,
        organization_id: Uuid,
        roles: Vec<Role>,
    ) -> Result<String, AuthError> {
        let now = Utc::now();
        let expiration = now + Duration::hours(TOKEN_EXPIRATION_HOURS);

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            org_id: organization_id.to_string(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
            iat: now.timestamp(),
            exp: expiration.timestamp(),
            iss: "iou-modern".to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|_| AuthError::InvalidToken)
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let token_data = decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::ExpiredToken,
            _ => AuthError::InvalidToken,
        })?;

        Ok(token_data.claims)
    }
}

impl Default for JwtService {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension key for storing auth context in request state
pub struct AuthExtension;

/// Authentication middleware - validates JWT tokens and extracts user context
pub async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract token from Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(ApiError::Unauthorized("Missing authentication token".to_string()))?;

    // Check Bearer prefix
    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::Unauthorized("Invalid token format".to_string()));
    }

    let token = &auth_header[7..]; // Skip "Bearer "

    // Validate token
    let jwt_service = JwtService::new();
    let claims = jwt_service
        .validate_token(token)
        .map_err(|e| ApiError::Unauthorized(e.to_string()))?;

    // Parse UUIDs
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID in token".to_string()))?;
    let organization_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| ApiError::Unauthorized("Invalid organization ID in token".to_string()))?;

    // Parse roles
    let roles: Vec<Role> = claims
        .roles
        .iter()
        .filter_map(|r| r.parse().ok())
        .collect();

    // Create auth context
    let auth_context = AuthContext {
        user_id,
        email: claims.email,
        organization_id,
        roles,
    };

    // Store auth context in request extensions
    let mut req = req;
    req.extensions_mut().insert(auth_context);

    // Continue to handler
    Ok(next.run(req).await)
}

/// Optional authentication - doesn't fail if no token is present
pub async fn optional_auth_middleware(
    req: Request,
    next: Next,
) -> Response {
    // Try to extract and validate token, but don't fail if missing
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(auth_header) = auth_header {
        if auth_header.starts_with("Bearer ") {
            let token = &auth_header[7..];
            let jwt_service = JwtService::new();

            if let Ok(claims) = jwt_service.validate_token(token) {
                if let (Ok(user_id), Ok(org_id)) = (
                    Uuid::parse_str(&claims.sub),
                    Uuid::parse_str(&claims.org_id),
                ) {
                    let roles: Vec<Role> = claims
                        .roles
                        .iter()
                        .filter_map(|r| r.parse().ok())
                        .collect();

                    let auth_context = AuthContext {
                        user_id,
                        email: claims.email,
                        organization_id: org_id,
                        roles,
                    };

                    let mut req = req;
                    req.extensions_mut().insert(auth_context);
                    return next.run(req).await;
                }
            }
        }
    }

    // No valid token, continue without auth context
    next.run(req).await
}

/// Require specific permission - returns 403 if user doesn't have permission
pub fn require_permission(auth: &AuthContext, permission: Permission) -> Result<(), ApiError> {
    if auth
        .roles
        .iter()
        .any(|role| role.has_permission(permission))
    {
        Ok(())
    } else {
        Err(ApiError::Forbidden(format!(
            "Permission {:?} required",
            permission
        )))
    }
}

/// Check if user has any of the specified roles
pub fn has_any_role(auth: &AuthContext, roles: &[Role]) -> bool {
    auth.roles.iter().any(|r| roles.contains(r))
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String, // In production, this should be hashed
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserContext,
}

/// User context returned on login
#[derive(Debug, Serialize)]
pub struct UserContext {
    pub id: Uuid,
    pub email: String,
    pub organization_id: Uuid,
    pub roles: Vec<String>,
}

/// Handle login request
pub async fn login(
    Extension(db): Extension<Arc<crate::db::Database>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // TODO: Implement proper password verification with bcrypt
    // For now, this is a placeholder that accepts any login

    // Mock user lookup (in production, query the database)
    let user_id = Uuid::new_v4();
    let organization_id = Uuid::new_v4();

    // Determine roles based on email domain (mock logic)
    let roles = if req.email.ends_with("@admin.iou.nl") {
        vec![Role::Admin]
    } else if req.email.ends_with("@compliance.iou.nl") {
        vec![Role::ComplianceOfficer, Role::DomainViewer]
    } else if req.email.ends_with("@woo.iou.nl") {
        vec![Role::WooOfficer, Role::DomainViewer]
    } else {
        vec![Role::DomainViewer]
    };

    // Create JWT token
    let jwt_service = JwtService::new();
    let access_token = jwt_service.create_token(user_id, &req.email, organization_id, roles.clone())
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create token: {}", e)))?;

    // Log login attempt to audit trail
    log_audit_event(
        &db,
        user_id,
        organization_id,
        "login",
        &format!("User logged in: {}", req.email),
    ).await;

    Ok(Json(LoginResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: TOKEN_EXPIRATION_HOURS * 3600,
        user: UserContext {
            id: user_id,
            email: req.email,
            organization_id,
            roles: roles.iter().map(|r| r.to_string()).collect(),
        },
    }))
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Handle token refresh
pub async fn refresh_token(
    Extension(auth): Extension<AuthContext>,
    Json(_req): Json<RefreshRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Create new token with same user context
    let jwt_service = JwtService::new();
    let access_token = jwt_service.create_token(
        auth.user_id,
        &auth.email,
        auth.organization_id,
        auth.roles.clone(),
    )
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create token: {}", e)))?;

    Ok(Json(LoginResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: TOKEN_EXPIRATION_HOURS * 3600,
        user: UserContext {
            id: auth.user_id,
            email: auth.email.clone(),
            organization_id: auth.organization_id,
            roles: auth.roles.iter().map(|r| r.to_string()).collect(),
        },
    }))
}

/// Logout handler (client-side token invalidation)
pub async fn logout(
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<Arc<crate::db::Database>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Log logout to audit trail
    log_audit_event(
        &db,
        auth.user_id,
        auth.organization_id,
        "logout",
        &format!("User logged out: {}", auth.email),
    ).await;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Logged out successfully"
    })))
}

/// Log an audit event to the database
async fn log_audit_event(
    db: &Arc<crate::db::Database>,
    user_id: Uuid,
    organization_id: Uuid,
    event_type: &str,
    description: &str,
) {
    // TODO: Implement proper audit logging to database
    // For now, just log to tracing
    tracing::info!(
        user_id = %user_id,
        organization_id = %organization_id,
        event_type = %event_type,
        "{}",
        description
    );
}

// Implement Display for Role
impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Auditor => write!(f, "auditor"),
            Role::DomainManager => write!(f, "domain_manager"),
            Role::DomainEditor => write!(f, "domain_editor"),
            Role::DomainViewer => write!(f, "domain_viewer"),
            Role::ObjectCreator => write!(f, "object_creator"),
            Role::ObjectEditor => write!(f, "object_editor"),
            Role::ObjectApprover => write!(f, "object_approver"),
            Role::ComplianceOfficer => write!(f, "compliance_officer"),
            Role::ComplianceReviewer => write!(f, "compliance_reviewer"),
            Role::WooOfficer => write!(f, "woo_officer"),
            Role::WooPublisher => write!(f, "woo_publisher"),
        }
    }
}

// Implement FromStr for Role
impl std::str::FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Role::Admin),
            "auditor" => Ok(Role::Auditor),
            "domain_manager" => Ok(Role::DomainManager),
            "domain_editor" => Ok(Role::DomainEditor),
            "domain_viewer" => Ok(Role::DomainViewer),
            "object_creator" => Ok(Role::ObjectCreator),
            "object_editor" => Ok(Role::ObjectEditor),
            "object_approver" => Ok(Role::ObjectApprover),
            "compliance_officer" => Ok(Role::ComplianceOfficer),
            "compliance_reviewer" => Ok(Role::ComplianceReviewer),
            "woo_officer" => Ok(Role::WooOfficer),
            "woo_publisher" => Ok(Role::WooPublisher),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_token_creation_and_validation() {
        let service = JwtService::new();
        let user_id = Uuid::new_v4();
        let email = "test@example.com";
        let org_id = Uuid::new_v4();
        let roles = vec![Role::Admin];

        let token = service.create_token(user_id, email, org_id, roles).unwrap();
        let claims = service.validate_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(claims.org_id, org_id.to_string());
    }

    #[test]
    fn test_role_permissions() {
        assert!(Role::Admin.has_permission(Permission::UserManage));
        assert!(Role::DomainViewer.has_permission(Permission::DomainRead));
        assert!(!Role::DomainViewer.has_permission(Permission::DomainUpdate));
    }
}
