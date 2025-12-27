//! # PBS Exporter
//!
//! A professional-grade Prometheus metrics exporter for Proxmox Backup Server 4.x.
//!
//! ## Overview
//!
//! This crate provides a complete solution for exporting PBS metrics to Prometheus,
//! including:
//!
//! - Host system metrics (CPU, memory, disk, load averages)
//! - Datastore capacity metrics
//! - Backup snapshot metrics (count, last backup timestamp)
//! - PBS version information
//!
//! ## Quick Start
//!
//! ```no_run
//! use pbs_exporter::{config::Settings, client::PbsClient, metrics::MetricsCollector, server::start_server};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration
//!     let settings = Settings::load(Some("config/default.toml"))?;
//!     
//!     // Create PBS client
//!     let client = PbsClient::new(settings.pbs)?;
//!     
//!     let metrics = MetricsCollector::new(std::sync::Arc::new(client), 0).unwrap();
//!     
//!     // Start HTTP server
//!     start_server(&settings.exporter.listen_address, metrics).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration
//!
//! The exporter can be configured via:
//! - TOML configuration file
//! - Environment variables (with `PBS_EXPORTER_` prefix)
//! - Command-line arguments
//!
//! See [`config::Settings`] for details.
//!
//! ## Modules
//!
//! - [`client`] - PBS API client for fetching metrics data
//! - [`config`] - Configuration management
//! - [`error`] - Error types and handling
//! - [`metrics`] - Prometheus metrics definitions and collection
//! - [`server`] - HTTP server for exposing metrics

pub mod client;
pub mod config;
pub mod error;
pub mod metrics;
pub mod server;

pub use error::{PbsError, Result};
