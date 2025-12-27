//! PBS API client for communicating with Proxmox Backup Server.
//!
//! This module provides a client for interacting with the PBS REST API
//! to collect metrics data.

use crate::config::PbsConfig;
use crate::error::{PbsError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, warn};

/// PBS API client.
#[derive(Clone)]
pub struct PbsClient {
    client: Client,
    config: PbsConfig,
    auth_header: String,
}

impl PbsClient {
    /// Create a new PBS API client.
    ///
    /// # Arguments
    ///
    /// * `config` - PBS configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pbs_exporter::client::PbsClient;
    /// use pbs_exporter::config::PbsConfig;
    ///
    /// let config = PbsConfig {
    ///     endpoint: "https://pbs.example.com:8007".to_string(),
    ///     token_id: "user@pam!token".to_string(),
    ///     token_secret: "secret".to_string(),
    ///     verify_tls: false,
    ///     timeout_seconds: 5,
    ///     snapshot_history_limit: 0,
    /// };
    /// let client = PbsClient::new(config).unwrap();
    /// ```
    pub fn new(config: PbsConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .danger_accept_invalid_certs(!config.verify_tls)
            .build()?;

        let auth_header = format!("PBSAPIToken={}:{}", config.token_id, config.token_secret);

        Ok(Self {
            client,
            config,
            auth_header,
        })
    }

    /// Get node status (CPU, memory, disk, etc.).
    pub async fn get_node_status(&self) -> Result<NodeStatus> {
        let url = format!("{}/api2/json/nodes/localhost/status", self.config.endpoint);
        debug!("Fetching node status from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Failed to get node status: {}", response.status());
            return Err(PbsError::Api(response.error_for_status().unwrap_err()));
        }

        let body = response.text().await?;
        debug!("Raw API response: {}", body);
        
        let api_response: ApiResponse<NodeStatus> = serde_json::from_str(&body)
            .map_err(|e| PbsError::ParseError(format!("Failed to parse node status: {}. Body: {}", e, body)))?;
        Ok(api_response.data)
    }

    /// Get datastore usage information.
    pub async fn get_datastore_usage(&self) -> Result<Vec<DatastoreUsage>> {
        let url = format!("{}/api2/json/status/datastore-usage", self.config.endpoint);
        debug!("Fetching datastore usage from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Failed to get datastore usage: {}", response.status());
            return Err(PbsError::Api(response.error_for_status().unwrap_err()));
        }

        let api_response: ApiResponse<Vec<DatastoreUsage>> = response.json().await?;
        Ok(api_response.data)
    }

    /// Get backup groups for a specific datastore.
    pub async fn get_backup_groups(&self, datastore: &str) -> Result<Vec<BackupGroup>> {
        let url = format!(
            "{}/api2/json/admin/datastore/{}/groups",
            self.config.endpoint, datastore
        );
        debug!("Fetching backup groups from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            warn!(
                "Failed to get backup groups for {}: {}",
                datastore,
                response.status()
            );
            return Err(PbsError::Api(response.error_for_status().unwrap_err()));
        }

        let body = response.text().await?;
        debug!("Raw backup groups response for {}: {}", datastore, body);
        
        let api_response: ApiResponse<Vec<BackupGroup>> = serde_json::from_str(&body)
            .map_err(|e| PbsError::ParseError(format!("Failed to parse backup groups: {}. Body: {}", e, body)))?;
        Ok(api_response.data)
    }

    /// Get PBS version information.
    pub async fn get_version(&self) -> Result<VersionInfo> {
        let url = format!("{}/api2/json/version", self.config.endpoint);
        debug!("Fetching version from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Failed to get version: {}", response.status());
            return Err(PbsError::Api(response.error_for_status().unwrap_err()));
        }

        let api_response: ApiResponse<VersionInfo> = response.json().await?;
        Ok(api_response.data)
    }
}

/// Generic PBS API response wrapper.
#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    data: T,
}

/// Node status information from PBS.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeStatus {
    /// CPU usage (0.0 to 1.0)
    pub cpu: f64,
    /// I/O wait (0.0 to 1.0)
    pub wait: f64,
    /// Used memory in bytes
    pub memory: Memory,
    /// Root filesystem usage (PBS calls it "root" not "rootfs")
    pub root: Disk,
    /// Swap usage
    pub swap: Memory,
    /// Load averages [1min, 5min, 15min]
    pub loadavg: [f64; 3],
    /// Uptime in seconds
    pub uptime: u64,
}

/// Memory information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Memory {
    /// Used memory in bytes
    pub used: u64,
    /// Total memory in bytes
    pub total: u64,
    /// Free memory in bytes
    pub free: u64,
}

/// Disk information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Disk {
    /// Used disk space in bytes
    pub used: u64,
    /// Total disk space in bytes
    pub total: u64,
    /// Available disk space in bytes
    pub avail: u64,
}

/// Datastore usage information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatastoreUsage {
    /// Datastore name
    pub store: String,
    /// Total size in bytes
    pub total: u64,
    /// Used bytes
    pub used: u64,
    /// Available bytes
    pub avail: u64,
}

/// Backup group information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackupGroup {
    /// Backup type (vm, ct, host)
    #[serde(rename = "backup-type")]
    pub backup_type: String,
    /// Backup ID (VM ID, CT ID, or hostname)
    #[serde(rename = "backup-id")]
    pub backup_id: String,
    /// Number of snapshots in this group
    #[serde(rename = "backup-count")]
    pub backup_count: u64,
    /// Last backup timestamp (Unix epoch)
    #[serde(rename = "last-backup")]
    pub last_backup: i64,
    /// Optional comment
    #[serde(default)]
    pub comment: Option<String>,
}

/// Snapshot information from PBS.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Snapshot {
    /// Backup type (vm, ct, host)
    #[serde(rename = "backup-type")]
    pub backup_type: String,
    /// Backup ID (VM ID, CT ID, or hostname)
    #[serde(rename = "backup-id")]
    pub backup_id: String,
    /// Backup timestamp (Unix epoch)
    #[serde(rename = "backup-time")]
    pub backup_time: i64,
    /// Optional comment
    #[serde(default)]
    pub comment: Option<String>,
    /// Total snapshot size in bytes
    #[serde(default)]
    pub size: Option<u64>,
    /// Whether snapshot is protected from deletion
    #[serde(default)]
    pub protected: Option<bool>,
    /// Verification status
    #[serde(default)]
    pub verification: Option<VerificationStatus>,
}

/// Verification status information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VerificationStatus {
    /// Verification state (ok, failed, none, etc.)
    pub state: String,
}

impl PbsClient {
    /// Get snapshots for a specific datastore to extract comments.
    pub async fn get_snapshots(&self, datastore: &str) -> Result<Vec<Snapshot>> {
        let url = format!(
            "{}/api2/json/admin/datastore/{}/snapshots",
            self.config.endpoint, datastore
        );
        debug!("Fetching snapshots from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Failed to get snapshots for {}: {}", datastore, response.status());
            return Err(PbsError::Api(response.error_for_status().unwrap_err()));
        }

        let body = response.text().await?;
        debug!("Raw snapshots response for {}: {} bytes", datastore, body.len());
        
        let api_response: ApiResponse<Vec<Snapshot>> = serde_json::from_str(&body)
            .map_err(|e| PbsError::ParseError(format!("Failed to parse snapshots: {}. Body preview: {}...", e, &body[..body.len().min(200)])))?;
        Ok(api_response.data)
    }
}

/// PBS version information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionInfo {
    /// PBS version string
    pub version: String,
    /// Release information
    pub release: String,
    /// Repository ID
    pub repoid: String,
}

/// Task information from PBS.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Task {
    /// Unique process ID
    pub upid: String,
    /// Worker type (backup, verify, prune, sync, garbage_collection)
    #[serde(rename = "worker_type")]
    pub worker_type: String,
    /// Worker ID (datastore:type/id)
    #[serde(rename = "worker_id")]
    pub worker_id: Option<String>,
    /// Start timestamp
    pub starttime: i64,
    /// Task end time (if finished)
    pub endtime: Option<i64>,
    /// Task status
    pub status: Option<String>,
    /// Comment (if any)
    #[serde(default)]
    pub comment: Option<String>,
}

/// Garbage collection status for a datastore.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GcStatus {
    /// Total bytes on disk
    #[serde(rename = "disk-bytes")]
    pub disk_bytes: Option<u64>,
    /// Bytes reclaimed in last GC
    #[serde(rename = "removed-bytes")]
    pub removed_bytes: Option<u64>,
    /// Bytes that can be reclaimed
    #[serde(rename = "pending-bytes")]
    pub pending_bytes: Option<u64>,
    /// Last GC completion timestamp
    #[serde(rename = "last-run-endtime")]
    pub last_run_endtime: Option<i64>,
    /// Last GC status
    #[serde(rename = "last-run-state")]
    pub last_run_state: Option<String>,
    /// Last GC duration in seconds
    pub duration: Option<f64>,
}

/// Tape drive information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TapeDrive {
    /// Drive name
    pub name: String,
    /// Vendor
    #[serde(default)]
    pub vendor: Option<String>,
    /// Model
    #[serde(default)]
    pub model: Option<String>,
    /// Serial number
    #[serde(default)]
    pub serial: Option<String>,
}

impl PbsClient {
    /// Get recent tasks from PBS.
    pub async fn get_tasks(&self, limit: Option<u64>) -> Result<Vec<Task>> {
        let limit_param = limit.unwrap_or(50);
        let url = format!(
            "{}/api2/json/nodes/localhost/tasks?limit={}",
            self.config.endpoint, limit_param
        );
        debug!("Fetching tasks from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Failed to get tasks: {}", response.status());
            return Err(PbsError::Api(response.error_for_status().unwrap_err()));
        }

        let api_response: ApiResponse<Vec<Task>> = response.json().await?;
        Ok(api_response.data)
    }

    /// Get GC status for a datastore.
    pub async fn get_gc_status(&self, datastore: &str) -> Result<GcStatus> {
        let url = format!(
            "{}/api2/json/admin/datastore/{}/gc",
            self.config.endpoint, datastore
        );
        debug!("Fetching GC status from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Failed to get GC status for {}: {}", datastore, response.status());
            return Err(PbsError::Api(response.error_for_status().unwrap_err()));
        }

        let api_response: ApiResponse<GcStatus> = response.json().await?;
        Ok(api_response.data)
    }

    /// Get tape drives.
    pub async fn get_tape_drives(&self) -> Result<Vec<TapeDrive>> {
        let url = format!("{}/api2/json/tape/drive", self.config.endpoint);
        debug!("Fetching tape drives from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Failed to get tape drives: {}", response.status());
            return Err(PbsError::Api(response.error_for_status().unwrap_err()));
        }

        let api_response: ApiResponse<Vec<TapeDrive>> = response.json().await?;
        Ok(api_response.data)
    }
}
