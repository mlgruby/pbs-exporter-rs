//! Error types for the PBS exporter.
//!
//! This module defines custom error types using `thiserror` for structured
//! error handling throughout the application.

use thiserror::Error;

/// Main error type for PBS exporter operations.
#[derive(Debug, Error)]
pub enum PbsError {
    /// Error communicating with PBS API
    #[error("PBS API error: {0}")]
    Api(#[from] reqwest::Error),

    /// Error parsing PBS API response
    #[error("Failed to parse PBS API response: {0}")]
    ParseError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Metrics error
    #[error("Metrics error: {0}")]
    Metrics(String),

    /// HTTP server error
    #[error("HTTP server error: {0}")]
    Server(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Result type alias for PBS operations.
pub type Result<T> = std::result::Result<T, PbsError>;
