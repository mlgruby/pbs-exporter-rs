//! Prometheus metrics definitions and collection logic.
//!
//! This module defines all Prometheus metrics exposed by the exporter
//! and provides functions to collect and update them from PBS API data.

// Module declarations
mod collectors;
mod registry;
mod updates;

// Re-exports
use collectors::*;
pub use registry::MetricRegistry;

use crate::client::PbsClient;
use crate::error::Result;
use std::sync::Arc;

/// Metrics collector for PBS exporter.
///
/// This is the main public API for the metrics module.
#[derive(Clone)]
pub struct MetricsCollector {
    client: Arc<PbsClient>,
    metrics: MetricRegistry,
    /// Controls how many historical snapshots per backup group are exposed.
    ///
    /// - 0 = expose all snapshots (no limit)
    /// - 1 = expose only the latest snapshot per group
    /// - N = expose the N most recent snapshots per group
    ///
    /// This limit helps control metric cardinality and memory usage in environments
    /// with many snapshots. Lower values reduce memory footprint but may hide
    /// historical data. For example:
    /// - limit=0: Full history, highest cardinality
    /// - limit=1: Minimal cardinality, only current state
    /// - limit=7: Good balance for weekly retention monitoring
    pub snapshot_history_limit: usize,
}

impl MetricsCollector {
    /// Create a new metrics collector.
    ///
    /// # Arguments
    ///
    /// * `client` - Arc-wrapped PBS API client for fetching metrics data
    /// * `snapshot_history_limit` - Maximum number of historical snapshots per backup group to expose (0 = unlimited)
    ///
    /// # Returns
    ///
    /// Returns a Result containing the initialized MetricsCollector or an error if metric registration fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use pbs_exporter::{client::PbsClient, config::PbsConfig, metrics::MetricsCollector};
    /// use std::sync::Arc;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = PbsConfig {
    ///     endpoint: "https://pbs.example.com:8007".to_string(),
    ///     token_id: "user@pam!token".to_string(),
    ///     token_secret: "secret".to_string(),
    ///     verify_tls: true,
    ///     timeout_seconds: 30,
    ///     snapshot_history_limit: 7,
    /// };
    /// let client = PbsClient::new(config)?;
    /// let collector = MetricsCollector::new(Arc::new(client), 7)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(client: Arc<PbsClient>, snapshot_history_limit: usize) -> Result<Self> {
        let metrics = MetricRegistry::new()?;

        Ok(Self {
            client,
            metrics,
            snapshot_history_limit,
        })
    }

    /// Collect all metrics from PBS.
    ///
    /// This method fetches current data from PBS API and updates all metrics including:
    /// - Node/host metrics (CPU, memory, disk, load)
    /// - Datastore usage metrics
    /// - Snapshot metrics (with history limiting)
    /// - Task metrics
    /// - Garbage collection metrics
    /// - Tape drive metrics
    /// - Version information
    ///
    /// The collection process also updates self-monitoring metrics like scrape duration and memory usage.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on successful collection, or an error if the PBS API is unreachable or returns invalid data.
    /// On error, the `pbs_up` metric is set to 0.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pbs_exporter::{client::PbsClient, config::PbsConfig, metrics::MetricsCollector};
    /// # use std::sync::Arc;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = PbsConfig {
    /// #     endpoint: "https://pbs.example.com:8007".to_string(),
    /// #     token_id: "user@pam!token".to_string(),
    /// #     token_secret: "secret".to_string(),
    /// #     verify_tls: true,
    /// #     timeout_seconds: 30,
    /// #     snapshot_history_limit: 7,
    /// # };
    /// # let client = PbsClient::new(config)?;
    /// # let collector = MetricsCollector::new(Arc::new(client), 7)?;
    /// collector.collect().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn collect(&self) -> Result<()> {
        collect(self).await
    }

    /// Encode metrics in Prometheus text format.
    ///
    /// Serializes all collected metrics into Prometheus exposition format suitable
    /// for scraping by Prometheus or compatible monitoring systems.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the encoded metrics as a String, or an error if encoding fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pbs_exporter::{client::PbsClient, config::PbsConfig, metrics::MetricsCollector};
    /// # use std::sync::Arc;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = PbsConfig {
    /// #     endpoint: "https://pbs.example.com:8007".to_string(),
    /// #     token_id: "user@pam!token".to_string(),
    /// #     token_secret: "secret".to_string(),
    /// #     verify_tls: true,
    /// #     timeout_seconds: 30,
    /// #     snapshot_history_limit: 7,
    /// # };
    /// # let client = PbsClient::new(config)?;
    /// # let collector = MetricsCollector::new(Arc::new(client), 7)?;
    /// collector.collect().await?;
    /// let metrics_output = collector.encode()?;
    /// println!("{}", metrics_output);
    /// # Ok(())
    /// # }
    /// ```
    pub fn encode(&self) -> Result<String> {
        self.metrics.encode()
    }

    /// Get a reference to the client (used by submodules)
    pub(crate) fn client(&self) -> &Arc<PbsClient> {
        &self.client
    }

    /// Get a reference to the metrics registry (used by submodules)
    pub(crate) fn metrics(&self) -> &MetricRegistry {
        &self.metrics
    }
}
