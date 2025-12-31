// Page controller for serving frontend SPA
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};

/// Serve index.html for SPA routes (fallback for non-API routes)
pub async fn serve_spa() -> impl IntoResponse {
    // Serve index.html for SPA routing
    match tokio::fs::read_to_string("frontend/dist/index.html").await {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}
// This module is kept for future SSR support

