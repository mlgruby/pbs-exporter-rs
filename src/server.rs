//! HTTP server for exposing Prometheus metrics.
//!
//! This module provides an Axum-based HTTP server that serves the `/metrics`
//! endpoint for Prometheus scraping and a `/health` endpoint for health checks.

use crate::error::Result;
use crate::metrics::MetricsCollector;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

/// Shared application state.
#[derive(Clone)]
struct AppState {
    metrics: Arc<MetricsCollector>,
}

/// Start the HTTP server.
///
/// # Arguments
///
/// * `listen_address` - Address to bind to (e.g., "0.0.0.0:9101")
/// * `metrics` - Metrics collector instance
///
/// # Examples
///
/// ```no_run
/// use pbs_exporter::server::start_server;
/// use pbs_exporter::metrics::MetricsCollector;
/// use pbs_exporter::client::PbsClient;
/// use pbs_exporter::config::PbsConfig;
///
/// #[tokio::main]
/// async fn main() {
///     let config = PbsConfig {
///         endpoint: "https://pbs.example.com:8007".to_string(),
///         token_id: "user@pam!token".to_string(),
///         token_secret: "secret".to_string(),
///         verify_tls: false,
///         timeout_seconds: 5,
///         snapshot_history_limit: 0,
///     };
///     let client = PbsClient::new(config).unwrap();
///     let metrics = MetricsCollector::new(std::sync::Arc::new(client), 0).unwrap();
///     start_server("0.0.0.0:9101", metrics).await.unwrap();
/// }
/// ```
pub async fn start_server(listen_address: &str, metrics: MetricsCollector) -> Result<()> {
    let state = AppState {
        metrics: Arc::new(metrics),
    };

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .route("/", get(root_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    info!("Starting HTTP server on {}", listen_address);

    let listener = TcpListener::bind(listen_address).await?;

    axum::serve(listener, app)
        .await
        .map_err(|e| crate::error::PbsError::Server(e.to_string()))?;

    Ok(())
}

/// Handler for /metrics endpoint.
async fn metrics_handler(State(state): State<AppState>) -> Response {
    info!("Received metrics scrape request");

    // Collect fresh metrics
    if let Err(e) = state.metrics.collect().await {
        warn!("Failed to collect metrics: {}", e);
        // Still return metrics, but pbs_up will be 0
    }

    // Encode metrics in Prometheus format
    match state.metrics.encode() {
        Ok(body) => (StatusCode::OK, body).into_response(),
        Err(e) => {
            warn!("Failed to encode metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to encode metrics: {}", e),
            )
                .into_response()
        }
    }
}

/// Handler for /health endpoint.
async fn health_handler() -> Response {
    (StatusCode::OK, "OK").into_response()
}

/// Handler for root endpoint.
async fn root_handler() -> Response {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>PBS Exporter</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        a { color: #0066cc; text-decoration: none; }
        a:hover { text-decoration: underline; }
        .info { background: #f0f0f0; padding: 15px; border-radius: 5px; margin: 20px 0; }
    </style>
</head>
<body>
    <h1>PBS Exporter</h1>
    <div class="info">
        <p>Prometheus metrics exporter for Proxmox Backup Server 4.x</p>
        <p><strong>Endpoints:</strong></p>
        <ul>
            <li><a href="/metrics">/metrics</a> - Prometheus metrics</li>
            <li><a href="/health">/health</a> - Health check</li>
        </ul>
    </div>
    <p>
        <a href="https://github.com/yourusername/pbs-exporter-rs">GitHub Repository</a>
    </p>
</body>
</html>
"#;

    (StatusCode::OK, html).into_response()
}
