use crate::auth::jwt::{Claims, JwtUtil};
use crate::dto::UserRole;
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Current user context stored in request extensions
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: i64,
    pub username: String,
    pub role: UserRole,
}

impl From<Claims> for CurrentUser {
    fn from(claims: Claims) -> Self {
        let role = match claims.role.as_str() {
            "Root" => UserRole::Root,
            "Admin" => UserRole::Admin,
            _ => UserRole::User,
        };

        CurrentUser {
            id: claims.sub,
            username: claims.username,
            role,
        }
    }
}

/// Authentication middleware
/// Extracts JWT token from Authorization header, verifies it, and stores user info in Extension
pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let token = extract_token_from_headers(request.headers())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify token
    let claims = JwtUtil::verify_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Ensure it's an access token
    if claims.token_type != "access" {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Create CurrentUser from claims
    let current_user = CurrentUser::from(claims);

    // Store in request extensions
    // In Axum 0.7+, we store the value directly, not wrapped in Extension
    request.extensions_mut().insert(Arc::new(current_user));

    Ok(next.run(request).await)
}

/// Optional authentication middleware
/// Similar to auth_middleware but doesn't return error if token is missing
/// Useful for endpoints that work with or without authentication
pub async fn optional_auth_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // Try to extract and verify token
    if let Some(token) = extract_token_from_headers(request.headers()) {
        if let Ok(claims) = JwtUtil::verify_token(&token) {
            if claims.token_type == "access" {
                let current_user = CurrentUser::from(claims);
                // In Axum 0.7+, we store the value directly, not wrapped in Extension
                request.extensions_mut().insert(Arc::new(current_user));
            }
        }
    }

    next.run(request).await
}

/// Extract JWT token from Authorization header
fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// Helper function to extract current user from Extension
/// Returns None if user is not authenticated
pub fn get_current_user(request: &Request) -> Option<Arc<CurrentUser>> {
    // In Axum 0.7+, extensions store values directly
    request
        .extensions()
        .get::<Arc<CurrentUser>>()
        .cloned()
}

/// Helper function to require authentication
/// Returns error if user is not authenticated
pub fn require_auth(request: &Request) -> Result<Arc<CurrentUser>, StatusCode> {
    get_current_user(request).ok_or(StatusCode::UNAUTHORIZED)
}

/// Check if user is admin or root
impl CurrentUser {
    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::Root)
    }

    pub fn is_root(&self) -> bool {
        matches!(self.role, UserRole::Root)
    }

    pub fn is_author_or_admin(&self, author_id: i64) -> bool {
        self.id == author_id || self.is_admin()
    }
}

/// Admin middleware
/// Checks if the user is authenticated and has admin or root role
/// This middleware must be used after auth_middleware
pub async fn admin_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get current user from request extensions (set by auth_middleware)
    let current_user = require_auth(&request)?;

    // Check if user is admin or root
    if !current_user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}