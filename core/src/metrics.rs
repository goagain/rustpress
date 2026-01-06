//! Prometheus metrics for RustPress

use lazy_static::lazy_static;
use prometheus::{register_counter_vec, register_histogram_vec, CounterVec, HistogramVec, Encoder, TextEncoder};

lazy_static! {
    /// HTTP request counter
    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "rustpress_http_requests_total",
        "Total number of HTTP requests",
        &["method", "endpoint", "status"]
    )
    .expect("Can't create HTTP_REQUESTS_TOTAL metric");

    /// HTTP request duration histogram
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "rustpress_http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "endpoint"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .expect("Can't create HTTP_REQUEST_DURATION metric");

    /// Database query counter
    pub static ref DATABASE_QUERIES_TOTAL: CounterVec = register_counter_vec!(
        "rustpress_database_queries_total",
        "Total number of database queries",
        &["operation", "table"]
    )
    .expect("Can't create DATABASE_QUERIES_TOTAL metric");

    /// Database query duration histogram
    pub static ref DATABASE_QUERY_DURATION: HistogramVec = register_histogram_vec!(
        "rustpress_database_query_duration_seconds",
        "Database query duration in seconds",
        &["operation", "table"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]
    )
    .expect("Can't create DATABASE_QUERY_DURATION metric");

    /// Posts counter
    pub static ref POSTS_TOTAL: CounterVec = register_counter_vec!(
        "rustpress_posts_total",
        "Total number of posts",
        &["status", "category"]
    )
    .expect("Can't create POSTS_TOTAL metric");

    /// Users counter
    pub static ref USERS_TOTAL: CounterVec = register_counter_vec!(
        "rustpress_users_total",
        "Total number of users",
        &["role", "status"]
    )
    .expect("Can't create USERS_TOTAL metric");

    /// Plugin operations counter
    pub static ref PLUGIN_OPERATIONS_TOTAL: CounterVec = register_counter_vec!(
        "rustpress_plugin_operations_total",
        "Total number of plugin operations",
        &["operation", "plugin_id"]
    )
    .expect("Can't create PLUGIN_OPERATIONS_TOTAL metric");
}

/// Get all metrics in Prometheus format
pub fn gather_metrics() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}

/// Initialize default metrics
pub fn init_metrics() {
    // This function can be called at application startup to ensure all metrics are registered
    // The lazy_static! declarations above already register them when first accessed
}