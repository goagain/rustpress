use axum::response::Response;
use prometheus::Encoder;

/// Get Prometheus metrics
/// GET /api/admin/metrics
pub async fn get_metrics() -> Result<Response<String>, axum::http::StatusCode> {
    // Gather metrics
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();

    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let metrics = String::from_utf8(buffer)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(metrics)
        .unwrap())
}