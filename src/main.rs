use anyhow::Result;
use clap::Parser;
use pbs_exporter::{
    client::PbsClient, config::Settings, metrics::MetricsCollector, server::start_server,
};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// PBS Exporter - Prometheus metrics exporter for Proxmox Backup Server 4.x
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Load configuration
    let settings = Settings::load(args.config.as_deref())?;

    // Initialize logging
    init_logging(&settings.exporter.log_level)?;

    info!("Starting PBS Exporter");
    info!("PBS endpoint: {}", settings.pbs.endpoint);
    info!("Listen address: {}", settings.exporter.listen_address);

    // Create PBS client
    let client = PbsClient::new(settings.pbs.clone())?;
    info!("PBS client initialized");

    // Create metrics collector
    let client = std::sync::Arc::new(client);
    let metrics = MetricsCollector::new(client, settings.pbs.snapshot_history_limit)?;
    info!("Metrics collector initialized");

    // Start HTTP server
    info!("Starting HTTP server...");
    if let Err(e) = start_server(&settings.exporter.listen_address, metrics).await {
        error!("Server error: {}", e);
        return Err(e.into());
    }

    Ok(())
}

/// Initialize structured logging with tracing.
fn init_logging(log_level: &str) -> Result<()> {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}
