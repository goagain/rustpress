//! Basic Authentication middleware for RustPress

use axum::{
    extract::Request,
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::{engine::general_purpose, Engine as _};
use std::sync::Arc;

/// Basic Auth configuration
#[derive(Debug, Clone)]
pub struct BasicAuthConfig {
    pub username: String,
    pub password: String,
}

impl BasicAuthConfig {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    /// Create config from environment variables
    /// METRICS_USERNAME and METRICS_PASSWORD
    pub fn from_env() -> Option<Self> {
        let username = std::env::var("METRICS_USERNAME").ok()?;
        let password = std::env::var("METRICS_PASSWORD").ok()?;
        Some(Self::new(username, password))
    }
}

/// Basic authentication middleware
/// Extracts credentials from Authorization header and verifies them
pub async fn basic_auth_middleware(
    config: Arc<BasicAuthConfig>,
    request: Request,
    next: Next,
) -> Response {
    // Check if Authorization header exists
    let auth_header = match request.headers().get(header::AUTHORIZATION) {
        Some(header) => header,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Basic realm=\"RustPress Metrics\"")],
                "Missing Authorization header",
            )
                .into_response();
        }
    };

    // Parse Basic Auth header
    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                "Invalid Authorization header encoding",
            )
                .into_response();
        }
    };

    // Check if it's Basic auth
    if !auth_str.starts_with("Basic ") {
        return (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Basic realm=\"RustPress Metrics\"")],
            "Expected Basic authentication",
        )
            .into_response();
    }

    // Decode base64 credentials
    let base64_credentials = &auth_str[6..]; // Remove "Basic " prefix
    let credentials = match general_purpose::STANDARD.decode(base64_credentials) {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                "Invalid base64 encoding in Authorization header",
            )
                .into_response();
        }
    };

    // Parse username:password
    let credentials_str = match String::from_utf8(credentials) {
        Ok(s) => s,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                "Invalid UTF-8 encoding in credentials",
            )
                .into_response();
        }
    };

    let parts: Vec<&str> = credentials_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid credentials format, expected username:password",
        )
            .into_response();
    }

    let username = parts[0];
    let password = parts[1];

    // Verify credentials
    if username != config.username || password != config.password {
        return (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Basic realm=\"RustPress Metrics\"")],
            "Invalid credentials",
        )
            .into_response();
    }

    // Credentials are valid, proceed with request
    next.run(request).await
}

/// Extract basic auth credentials from Authorization header
/// Returns (username, password) tuple
pub fn extract_basic_credentials(headers: &HeaderMap) -> Result<(String, String), &'static str> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .ok_or("Missing Authorization header")?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| "Invalid Authorization header encoding")?;

    if !auth_str.starts_with("Basic ") {
        return Err("Expected Basic authentication");
    }

    let base64_credentials = &auth_str[6..];
    let credentials = general_purpose::STANDARD
        .decode(base64_credentials)
        .map_err(|_| "Invalid base64 encoding")?;

    let credentials_str = String::from_utf8(credentials)
        .map_err(|_| "Invalid UTF-8 encoding")?;

    let parts: Vec<&str> = credentials_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err("Invalid credentials format");
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_extract_basic_credentials_valid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==".parse().unwrap(), // Aladdin:open sesame
        );

        let result = extract_basic_credentials(&headers);
        assert!(result.is_ok());
        let (username, password) = result.unwrap();
        assert_eq!(username, "Aladdin");
        assert_eq!(password, "open sesame");
    }

    #[test]
    fn test_extract_basic_credentials_missing_header() {
        let headers = HeaderMap::new();
        let result = extract_basic_credentials(&headers);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing Authorization header");
    }

    #[test]
    fn test_extract_basic_credentials_wrong_scheme() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            "Bearer token123".parse().unwrap(),
        );

        let result = extract_basic_credentials(&headers);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Expected Basic authentication");
    }
}