//! Axum middleware for collecting Prometheus metrics

use axum::{
    extract::MatchedPath,
    http::{Request, Response},
    middleware::Next,
};
use std::time::Instant;

/// Middleware to collect HTTP request metrics
pub async fn metrics_middleware(
    matched_path: Option<MatchedPath>,
    request: Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let start = Instant::now();
    let method = request.method().clone();
    let path = matched_path
        .as_ref()
        .map(|mp| mp.as_str())
        .unwrap_or("unknown");

    let response = next.run(request).await;
    let duration = start.elapsed();

    // Record metrics
    let status = response.status().as_u16().to_string();
    crate::metrics::HTTP_REQUESTS_TOTAL
        .with_label_values(&[method.as_str(), path, &status])
        .inc();

    crate::metrics::HTTP_REQUEST_DURATION
        .with_label_values(&[method.as_str(), path])
        .observe(duration.as_secs_f64());

    response
}