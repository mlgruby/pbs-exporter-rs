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
    ///
    /// Fetches the current status of the PBS node including CPU usage, memory,
    /// swap, disk usage, load averages, and uptime.
    ///
    /// # Returns
    ///
    /// Returns a `NodeStatus` struct containing all node metrics, or an error if
    /// the API call fails or returns invalid data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API request fails (network issues, authentication failure)
    /// - The response status is not successful
    /// - The response JSON cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pbs_exporter::client::PbsClient;
    /// # use pbs_exporter::config::PbsConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = PbsConfig {
    /// #     endpoint: "https://pbs.example.com:8007".to_string(),
    /// #     token_id: "user@pam!token".to_string(),
    /// #     token_secret: "secret".to_string(),
    /// #     verify_tls: false,
    /// #     timeout_seconds: 5,
    /// #     snapshot_history_limit: 0,
    /// # };
    /// # let client = PbsClient::new(config)?;
    /// let status = client.get_node_status().await?;
    /// println!("CPU usage: {}", status.cpu);
    /// # Ok(())
    /// # }
    /// ```
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

        // Parse JSON directly from response stream to avoid buffering
        let api_response: ApiResponse<NodeStatus> = response
            .json()
            .await
            .map_err(|e| PbsError::ParseError(format!("Failed to parse node status: {}", e)))?;
        Ok(api_response.data)
    }

    /// Get datastore usage information.
    ///
    /// Fetches usage statistics for all configured datastores including total,
    /// used, and available bytes.
    ///
    /// # Returns
    ///
    /// Returns a vector of `DatastoreUsage` structs, one for each datastore,
    /// or an error if the API call fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the response cannot be parsed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pbs_exporter::client::PbsClient;
    /// # use pbs_exporter::config::PbsConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = PbsConfig {
    /// #     endpoint: "https://pbs.example.com:8007".to_string(),
    /// #     token_id: "user@pam!token".to_string(),
    /// #     token_secret: "secret".to_string(),
    /// #     verify_tls: false,
    /// #     timeout_seconds: 5,
    /// #     snapshot_history_limit: 0,
    /// # };
    /// # let client = PbsClient::new(config)?;
    /// let datastores = client.get_datastore_usage().await?;
    /// for ds in datastores {
    ///     println!("Datastore {}: {} bytes used", ds.store, ds.used);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// Fetches all backup groups (VM and container backups) from the specified
    /// datastore including backup count and last backup timestamp.
    ///
    /// # Arguments
    ///
    /// * `datastore` - Name of the datastore to query
    ///
    /// # Returns
    ///
    /// Returns a vector of `BackupGroup` structs containing backup type, ID,
    /// count, and last backup time, or an error if the API call fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The datastore doesn't exist or is not accessible
    /// - The API request fails
    /// - The response cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pbs_exporter::client::PbsClient;
    /// # use pbs_exporter::config::PbsConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = PbsConfig {
    /// #     endpoint: "https://pbs.example.com:8007".to_string(),
    /// #     token_id: "user@pam!token".to_string(),
    /// #     token_secret: "secret".to_string(),
    /// #     verify_tls: false,
    /// #     timeout_seconds: 5,
    /// #     snapshot_history_limit: 0,
    /// # };
    /// # let client = PbsClient::new(config)?;
    /// let groups = client.get_backup_groups("backup").await?;
    /// for group in groups {
    ///     println!("{}/{}: {} backups", group.backup_type, group.backup_id, group.backup_count);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

        // Parse JSON directly from response stream
        let api_response: ApiResponse<Vec<BackupGroup>> = response.json().await.map_err(|e| {
            PbsError::ParseError(format!(
                "Failed to parse backup groups for {}: {}",
                datastore, e
            ))
        })?;
        Ok(api_response.data)
    }

    /// Get PBS version information.
    ///
    /// Fetches the current version, release, and repository ID of the
    /// Proxmox Backup Server.
    ///
    /// # Returns
    ///
    /// Returns a `VersionInfo` struct containing version string, release
    /// information, and repository ID, or an error if the API call fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API request fails (network issues, authentication failure)
    /// - The response status is not successful
    /// - The response JSON cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pbs_exporter::client::PbsClient;
    /// # use pbs_exporter::config::PbsConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = PbsConfig {
    /// #     endpoint: "https://pbs.example.com:8007".to_string(),
    /// #     token_id: "user@pam!token".to_string(),
    /// #     token_secret: "secret".to_string(),
    /// #     verify_tls: false,
    /// #     timeout_seconds: 5,
    /// #     snapshot_history_limit: 0,
    /// # };
    /// # let client = PbsClient::new(config)?;
    /// let version = client.get_version().await?;
    /// println!("PBS version: {} ({})", version.version, version.release);
    /// # Ok(())
    /// # }
    /// ```
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
    /// Last verification timestamp (Unix epoch)
    #[serde(rename = "last-verify")]
    pub last_verify: Option<i64>,
}

impl PbsClient {
    /// Get snapshots for a specific datastore.
    ///
    /// Fetches all backup snapshots from the specified datastore including
    /// backup time, size, protection status, verification status, and comments.
    /// This is useful for extracting snapshot metadata and comments.
    ///
    /// # Arguments
    ///
    /// * `datastore` - Name of the datastore to query
    ///
    /// # Returns
    ///
    /// Returns a vector of `Snapshot` structs containing snapshot metadata
    /// including backup type, ID, timestamp, size, protection status,
    /// verification status, and optional comments.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The datastore doesn't exist or is not accessible
    /// - The API request fails
    /// - The response JSON cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pbs_exporter::client::PbsClient;
    /// # use pbs_exporter::config::PbsConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = PbsConfig {
    /// #     endpoint: "https://pbs.example.com:8007".to_string(),
    /// #     token_id: "user@pam!token".to_string(),
    /// #     token_secret: "secret".to_string(),
    /// #     verify_tls: false,
    /// #     timeout_seconds: 5,
    /// #     snapshot_history_limit: 0,
    /// # };
    /// # let client = PbsClient::new(config)?;
    /// let snapshots = client.get_snapshots("backup").await?;
    /// for snapshot in snapshots {
    ///     println!("{}/{} at {}", snapshot.backup_type, snapshot.backup_id, snapshot.backup_time);
    ///     if let Some(comment) = snapshot.comment {
    ///         println!("  Comment: {}", comment);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
            warn!(
                "Failed to get snapshots for {}: {}",
                datastore,
                response.status()
            );
            return Err(PbsError::Api(response.error_for_status().unwrap_err()));
        }

        // Parse JSON directly from response stream
        let api_response: ApiResponse<Vec<Snapshot>> = response.json().await.map_err(|e| {
            PbsError::ParseError(format!(
                "Failed to parse snapshots for {}: {}",
                datastore, e
            ))
        })?;

        debug!(
            "Fetched {} snapshots for {}",
            api_response.data.len(),
            datastore
        );
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
    ///
    /// Fetches a list of recent tasks (backup, verify, prune, sync, garbage
    /// collection) from the PBS node. Tasks include their start time, end time
    /// (if finished), status, and type.
    ///
    /// # Arguments
    ///
    /// * `limit` - Optional maximum number of tasks to return (default: 50)
    ///
    /// # Returns
    ///
    /// Returns a vector of `Task` structs containing task information including
    /// UPID, worker type, worker ID, start/end times, and status.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API request fails (network issues, authentication failure)
    /// - The response status is not successful
    /// - The response JSON cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pbs_exporter::client::PbsClient;
    /// # use pbs_exporter::config::PbsConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = PbsConfig {
    /// #     endpoint: "https://pbs.example.com:8007".to_string(),
    /// #     token_id: "user@pam!token".to_string(),
    /// #     token_secret: "secret".to_string(),
    /// #     verify_tls: false,
    /// #     timeout_seconds: 5,
    /// #     snapshot_history_limit: 0,
    /// # };
    /// # let client = PbsClient::new(config)?;
    /// let tasks = client.get_tasks(Some(10)).await?;
    /// for task in tasks {
    ///     println!("Task: {} ({})", task.worker_type, task.upid);
    ///     if let Some(status) = task.status {
    ///         println!("  Status: {}", status);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

    /// Get garbage collection status for a datastore.
    ///
    /// Fetches garbage collection (GC) statistics for the specified datastore
    /// including total disk usage, bytes reclaimed, pending bytes that can be
    /// reclaimed, last GC run time, status, and duration.
    ///
    /// # Arguments
    ///
    /// * `datastore` - Name of the datastore to query
    ///
    /// # Returns
    ///
    /// Returns a `GcStatus` struct containing GC statistics including disk bytes,
    /// removed bytes, pending bytes, last run time, state, and duration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The datastore doesn't exist or is not accessible
    /// - The API request fails
    /// - The response JSON cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pbs_exporter::client::PbsClient;
    /// # use pbs_exporter::config::PbsConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = PbsConfig {
    /// #     endpoint: "https://pbs.example.com:8007".to_string(),
    /// #     token_id: "user@pam!token".to_string(),
    /// #     token_secret: "secret".to_string(),
    /// #     verify_tls: false,
    /// #     timeout_seconds: 5,
    /// #     snapshot_history_limit: 0,
    /// # };
    /// # let client = PbsClient::new(config)?;
    /// let gc_status = client.get_gc_status("backup").await?;
    /// if let Some(pending) = gc_status.pending_bytes {
    ///     println!("Pending GC bytes: {}", pending);
    /// }
    /// if let Some(state) = gc_status.last_run_state {
    ///     println!("Last GC state: {}", state);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
            warn!(
                "Failed to get GC status for {}: {}",
                datastore,
                response.status()
            );
            return Err(PbsError::Api(response.error_for_status().unwrap_err()));
        }

        let api_response: ApiResponse<GcStatus> = response.json().await?;
        Ok(api_response.data)
    }

    /// Get configured tape drives.
    ///
    /// Fetches information about all configured tape drives in the PBS system
    /// including name, vendor, model, and serial number. This is useful for
    /// monitoring tape backup infrastructure.
    ///
    /// # Returns
    ///
    /// Returns a vector of `TapeDrive` structs containing drive information,
    /// or an empty vector if no tape drives are configured.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API request fails (network issues, authentication failure)
    /// - The response status is not successful
    /// - The response JSON cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pbs_exporter::client::PbsClient;
    /// # use pbs_exporter::config::PbsConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = PbsConfig {
    /// #     endpoint: "https://pbs.example.com:8007".to_string(),
    /// #     token_id: "user@pam!token".to_string(),
    /// #     token_secret: "secret".to_string(),
    /// #     verify_tls: false,
    /// #     timeout_seconds: 5,
    /// #     snapshot_history_limit: 0,
    /// # };
    /// # let client = PbsClient::new(config)?;
    /// let drives = client.get_tape_drives().await?;
    /// for drive in drives {
    ///     println!("Drive: {}", drive.name);
    ///     if let Some(model) = drive.model {
    ///         println!("  Model: {}", model);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
