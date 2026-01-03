//! Prometheus metrics definitions and collection logic.
//!
//! This module defines all Prometheus metrics exposed by the exporter
//! and provides functions to collect and update them from PBS API data.

use crate::client::{BackupGroup, DatastoreUsage, NodeStatus, PbsClient, VersionInfo};
use crate::error::{PbsError, Result};
use prometheus::{Encoder, Gauge, GaugeVec, Opts, Registry, TextEncoder};
use std::sync::Arc;
use tracing::{debug, error, info};

// Interned strings to avoid repeated allocations
const UNKNOWN: &str = "unknown";
const EMPTY_STR: &str = "";
const RUNNING: &str = "running";
const OK: &str = "ok";

/// Metrics collector for PBS exporter.
#[derive(Clone)]
pub struct MetricsCollector {
    client: Arc<PbsClient>,
    registry: Registry,
    snapshot_history_limit: usize,

    // Exporter metrics
    pbs_up: Gauge,

    // Host metrics
    host_cpu_usage: Gauge,
    host_io_wait: Gauge,
    host_load1: Gauge,
    host_load5: Gauge,
    host_load15: Gauge,
    host_memory_used_bytes: Gauge,
    host_memory_total_bytes: Gauge,
    host_memory_free_bytes: Gauge,
    host_swap_used_bytes: Gauge,
    host_swap_total_bytes: Gauge,
    host_swap_free_bytes: Gauge,
    host_rootfs_used_bytes: Gauge,
    host_rootfs_total_bytes: Gauge,
    host_rootfs_avail_bytes: Gauge,
    host_uptime_seconds: Gauge,

    // Datastore metrics
    datastore_total_bytes: GaugeVec,
    datastore_used_bytes: GaugeVec,
    datastore_available_bytes: GaugeVec,

    // Backup metrics
    snapshot_count: GaugeVec,
    snapshot_last_timestamp_seconds: GaugeVec,

    // Individual snapshot metrics
    snapshot_info: GaugeVec,
    snapshot_size_bytes: GaugeVec,
    snapshot_verified: GaugeVec,
    snapshot_verification_timestamp: GaugeVec,
    snapshot_protected: GaugeVec,

    // Task metrics
    task_total: GaugeVec,
    task_duration_seconds: GaugeVec,
    task_last_run_timestamp: GaugeVec,
    task_running: GaugeVec,

    // GC metrics
    gc_last_run_timestamp: GaugeVec,
    gc_duration_seconds: GaugeVec,
    gc_removed_bytes: GaugeVec,
    gc_pending_bytes: GaugeVec,
    gc_status: GaugeVec,

    // Tape metrics
    tape_drive_info: GaugeVec,
    tape_drive_available: Gauge,

    // Version info
    pbs_version: GaugeVec,
}

impl MetricsCollector {
    /// Create a new metrics collector.
    pub fn new(client: Arc<PbsClient>, snapshot_history_limit: usize) -> Result<Self> {
        let registry = Registry::new();

        // Exporter metrics
        let pbs_up = Gauge::with_opts(Opts::new(
            "pbs_up",
            "Whether the last scrape of PBS was successful (1 = success, 0 = failure)",
        ))
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(pbs_up.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        // Host metrics
        let host_cpu_usage = Gauge::with_opts(Opts::new(
            "pbs_host_cpu_usage",
            "CPU usage of the PBS host (fraction of 1.0)",
        ))
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_cpu_usage.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_io_wait = Gauge::with_opts(Opts::new(
            "pbs_host_io_wait",
            "CPU I/O wait proportion (fraction of 1.0)",
        ))
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_io_wait.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_load1 = Gauge::with_opts(Opts::new("pbs_host_load1", "1-minute load average"))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_load1.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_load5 = Gauge::with_opts(Opts::new("pbs_host_load5", "5-minute load average"))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_load5.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_load15 = Gauge::with_opts(Opts::new("pbs_host_load15", "15-minute load average"))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_load15.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_memory_used_bytes = Gauge::with_opts(Opts::new(
            "pbs_host_memory_used_bytes",
            "Used RAM on PBS host in bytes",
        ))
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_memory_used_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_memory_total_bytes = Gauge::with_opts(Opts::new(
            "pbs_host_memory_total_bytes",
            "Total RAM on PBS host in bytes",
        ))
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_memory_total_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_memory_free_bytes = Gauge::with_opts(Opts::new(
            "pbs_host_memory_free_bytes",
            "Free RAM on PBS host in bytes",
        ))
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_memory_free_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_swap_used_bytes = Gauge::with_opts(Opts::new(
            "pbs_host_swap_used_bytes",
            "Used swap space in bytes",
        ))
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_swap_used_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_swap_total_bytes = Gauge::with_opts(Opts::new(
            "pbs_host_swap_total_bytes",
            "Total swap space in bytes",
        ))
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_swap_total_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_swap_free_bytes = Gauge::with_opts(Opts::new(
            "pbs_host_swap_free_bytes",
            "Free swap space in bytes",
        ))
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_swap_free_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_rootfs_used_bytes = Gauge::with_opts(Opts::new(
            "pbs_host_rootfs_used_bytes",
            "Used bytes on root filesystem",
        ))
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_rootfs_used_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_rootfs_total_bytes = Gauge::with_opts(Opts::new(
            "pbs_host_rootfs_total_bytes",
            "Total bytes on root filesystem",
        ))
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_rootfs_total_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_rootfs_avail_bytes = Gauge::with_opts(Opts::new(
            "pbs_host_rootfs_avail_bytes",
            "Available bytes on root filesystem",
        ))
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_rootfs_avail_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let host_uptime_seconds = Gauge::with_opts(Opts::new(
            "pbs_host_uptime_seconds",
            "Uptime of PBS host in seconds",
        ))
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(host_uptime_seconds.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        // Datastore metrics
        let datastore_total_bytes = GaugeVec::new(
            Opts::new(
                "pbs_datastore_total_bytes",
                "Total size of datastore in bytes",
            ),
            &["datastore"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(datastore_total_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let datastore_used_bytes = GaugeVec::new(
            Opts::new("pbs_datastore_used_bytes", "Used bytes in datastore"),
            &["datastore"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(datastore_used_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let datastore_available_bytes = GaugeVec::new(
            Opts::new(
                "pbs_datastore_available_bytes",
                "Available bytes in datastore",
            ),
            &["datastore"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(datastore_available_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        // Backup metrics
        let snapshot_count = GaugeVec::new(
            Opts::new("pbs_snapshot_count", "Number of backup snapshots"),
            &["datastore", "backup_type", "backup_id", "comment"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(snapshot_count.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let snapshot_last_timestamp_seconds = GaugeVec::new(
            Opts::new(
                "pbs_snapshot_last_timestamp_seconds",
                "Unix timestamp of last backup",
            ),
            &["datastore", "backup_type", "backup_id", "comment"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(snapshot_last_timestamp_seconds.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        // Individual snapshot metrics
        let snapshot_info = GaugeVec::new(
            Opts::new(
                "pbs_snapshot_info",
                "Individual snapshot information with timestamp as value",
            ),
            &[
                "datastore",
                "backup_type",
                "backup_id",
                "comment",
                "timestamp",
            ],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(snapshot_info.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let snapshot_size_bytes = GaugeVec::new(
            Opts::new(
                "pbs_snapshot_size_bytes",
                "Size of individual snapshot in bytes",
            ),
            &[
                "datastore",
                "backup_type",
                "backup_id",
                "comment",
                "timestamp",
                "verified",
            ],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(snapshot_size_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let snapshot_verification_timestamp = GaugeVec::new(
            Opts::new(
                "pbs_snapshot_verification_timestamp_seconds",
                "Timestamp of last verification in seconds",
            ),
            &[
                "datastore",
                "backup_type",
                "backup_id",
                "comment",
                "timestamp",
            ],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(snapshot_verification_timestamp.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let snapshot_verified = GaugeVec::new(
            Opts::new(
                "pbs_snapshot_verified",
                "Snapshot verification status (1=ok, 0=failed/unknown)",
            ),
            &[
                "datastore",
                "backup_type",
                "backup_id",
                "comment",
                "timestamp",
            ],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(snapshot_verified.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let snapshot_protected = GaugeVec::new(
            Opts::new(
                "pbs_snapshot_protected",
                "Snapshot protection status (1=protected, 0=not protected)",
            ),
            &[
                "datastore",
                "backup_type",
                "backup_id",
                "comment",
                "timestamp",
            ],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(snapshot_protected.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;
        // Task metrics
        let task_total = GaugeVec::new(
            Opts::new(
                "pbs_task_total",
                "Total number of tasks (by worker type/status)",
            ),
            &["worker_type", "status", "comment"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(task_total.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let task_duration_seconds = GaugeVec::new(
            Opts::new("pbs_task_duration_seconds", "Task duration in seconds"),
            &["worker_type", "status", "worker_id", "comment"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(task_duration_seconds.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let task_last_run_timestamp = GaugeVec::new(
            Opts::new(
                "pbs_task_last_run_timestamp",
                "Last run timestamp for task type",
            ),
            &["worker_type"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(task_last_run_timestamp.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let task_running = GaugeVec::new(
            Opts::new("pbs_task_running", "Currently running tasks"),
            &["worker_type", "comment"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(task_running.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        // GC metrics
        let gc_last_run_timestamp = GaugeVec::new(
            Opts::new(
                "pbs_gc_last_run_timestamp",
                "Last GC run completion timestamp",
            ),
            &["datastore"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(gc_last_run_timestamp.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let gc_duration_seconds = GaugeVec::new(
            Opts::new("pbs_gc_duration_seconds", "Last GC duration in seconds"),
            &["datastore"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(gc_duration_seconds.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let gc_removed_bytes = GaugeVec::new(
            Opts::new("pbs_gc_removed_bytes", "Bytes reclaimed in last GC"),
            &["datastore"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(gc_removed_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let gc_pending_bytes = GaugeVec::new(
            Opts::new("pbs_gc_pending_bytes", "Bytes that can be reclaimed by GC"),
            &["datastore"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(gc_pending_bytes.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let gc_status = GaugeVec::new(
            Opts::new("pbs_gc_status", "Last GC status (1=OK, 0=ERROR)"),
            &["datastore"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(gc_status.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        // Tape metrics
        let tape_drive_info = GaugeVec::new(
            Opts::new("pbs_tape_drive_info", "Tape drive information"),
            &["name", "vendor", "model", "serial"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(tape_drive_info.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        let tape_drive_available = Gauge::new(
            "pbs_tape_drive_available",
            "Number of available tape drives",
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(tape_drive_available.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        // Version info
        let pbs_version = GaugeVec::new(
            Opts::new("pbs_version", "PBS version information"),
            &["version", "release", "repoid"],
        )
        .map_err(|e| PbsError::Metrics(e.to_string()))?;
        registry
            .register(Box::new(pbs_version.clone()))
            .map_err(|e| PbsError::Metrics(e.to_string()))?;

        Ok(Self {
            client,
            registry,
            pbs_up,
            host_cpu_usage,
            host_io_wait,
            host_load1,
            host_load5,
            host_load15,
            host_memory_used_bytes,
            host_memory_total_bytes,
            host_memory_free_bytes,
            host_swap_used_bytes,
            host_swap_total_bytes,
            host_swap_free_bytes,
            host_rootfs_used_bytes,
            host_rootfs_total_bytes,
            host_rootfs_avail_bytes,
            host_uptime_seconds,
            datastore_total_bytes,
            datastore_used_bytes,
            datastore_available_bytes,
            snapshot_count,
            snapshot_last_timestamp_seconds,
            snapshot_info,
            snapshot_size_bytes,
            snapshot_verified,
            snapshot_verification_timestamp,
            snapshot_protected,
            task_total,
            task_duration_seconds,
            task_last_run_timestamp,
            task_running,
            gc_last_run_timestamp,
            gc_duration_seconds,
            gc_removed_bytes,
            gc_pending_bytes,
            gc_status,
            tape_drive_info,
            tape_drive_available,
            pbs_version,
            snapshot_history_limit,
        })
    }

    /// Collect all metrics from PBS.
    pub async fn collect(&self) -> Result<()> {
        info!("Collecting metrics from PBS");

        match self.collect_internal().await {
            Ok(_) => {
                self.pbs_up.set(1.0);
                info!("Successfully collected metrics");
                Ok(())
            }
            Err(e) => {
                error!("Failed to collect metrics: {}", e);
                self.pbs_up.set(0.0);
                Err(e)
            }
        }
    }

    async fn collect_internal(&self) -> Result<()> {
        // Reset all metrics to prevent stale data
        // This is crucial because we populate metrics dynamically based on current API state.
        // If an object (snapshot, task, drive) disappears or is filtered out,
        // we must ensure its corresponding metric is removed.
        self.pbs_up.set(0.0); // Will be set to 1.0 on success
        self.host_cpu_usage.set(0.0);
        self.host_io_wait.set(0.0);
        self.host_load1.set(0.0);
        self.host_load5.set(0.0);
        self.host_load15.set(0.0);
        self.host_memory_used_bytes.set(0.0);
        self.host_memory_total_bytes.set(0.0);
        self.host_memory_free_bytes.set(0.0);
        self.host_swap_used_bytes.set(0.0);
        self.host_swap_total_bytes.set(0.0);
        self.host_swap_free_bytes.set(0.0);
        self.host_rootfs_used_bytes.set(0.0);
        self.host_rootfs_total_bytes.set(0.0);
        self.host_rootfs_avail_bytes.set(0.0);
        self.host_uptime_seconds.set(0.0);

        self.datastore_total_bytes.reset();
        self.datastore_used_bytes.reset();
        self.datastore_available_bytes.reset();

        self.snapshot_count.reset();
        self.snapshot_info.reset();
        self.snapshot_size_bytes.reset();
        self.snapshot_verified.reset();
        self.snapshot_verification_timestamp.reset();
        self.snapshot_protected.reset();
        self.snapshot_last_timestamp_seconds.reset();

        self.task_total.reset();
        self.task_duration_seconds.reset();
        self.task_last_run_timestamp.reset();
        self.task_running.reset();

        self.gc_last_run_timestamp.reset();
        self.gc_duration_seconds.reset();
        self.gc_removed_bytes.reset();
        self.gc_pending_bytes.reset();
        self.gc_status.reset();

        self.tape_drive_info.reset();
        self.tape_drive_available.set(0.0);

        self.pbs_version.reset();

        // Collect node status
        let node_status = self.client.get_node_status().await?;
        self.update_node_metrics(&node_status);

        // Collect datastore usage
        let datastores = self.client.get_datastore_usage().await?;
        self.update_datastore_metrics(&datastores);

        // Map to store comments for tasks (worker_id -> comment)
        let mut task_comment_map: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        // Collect backup groups and snapshots for each datastore
        for ds in &datastores {
            // Fetch snapshots to get comments
            let snapshots = match self.client.get_snapshots(&ds.store).await {
                Ok(snaps) => snaps,
                Err(e) => {
                    error!("Failed to get snapshots for {}: {}", ds.store, e);
                    Vec::new()
                }
            };

            // Build a map of (backup_type, backup_id) -> (latest_time, comment)
            // Use owned keys for the map but avoid cloning during iteration
            let mut comment_map: std::collections::HashMap<
                (String, String),
                (i64, Option<String>),
            > = std::collections::HashMap::new();
            for snapshot in &snapshots {
                let key = (snapshot.backup_type.clone(), snapshot.backup_id.clone());
                // Keep the comment from the latest snapshot (highest backup_time)
                match comment_map.get_mut(&key) {
                    Some((time, comment)) => {
                        if snapshot.backup_time > *time {
                            *time = snapshot.backup_time;
                            *comment = snapshot.comment.clone();
                        }
                    }
                    None => {
                        comment_map.insert(key, (snapshot.backup_time, snapshot.comment.clone()));
                    }
                }
            }

            // Populate task_comment_map from the comment_map
            for ((backup_type, backup_id), (_, comment)) in &comment_map {
                if let Some(c) = comment {
                    if !c.is_empty() {
                        // Construct worker_id: datastore:type/id
                        let worker_id = format!("{}:{}/{}", ds.store, backup_type, backup_id);
                        task_comment_map.insert(worker_id, c.clone());
                    }
                }
            }

            // Update individual snapshot metrics
            self.update_snapshot_metrics(&ds.store, &snapshots, &comment_map);

            // Fetch backup groups
            match self.client.get_backup_groups(&ds.store).await {
                Ok(groups) => self.update_backup_metrics(&ds.store, &groups, &comment_map),
                Err(e) => {
                    error!("Failed to get backup groups for {}: {}", ds.store, e);
                    // Continue with other datastores
                }
            }
        }

        // Collect tasks
        match self.client.get_tasks(Some(50)).await {
            Ok(tasks) => self.update_task_metrics(&tasks, &task_comment_map),
            Err(e) => {
                error!("Failed to get tasks: {}", e);
            }
        }

        // Collect GC status for each datastore
        for ds in &datastores {
            match self.client.get_gc_status(&ds.store).await {
                Ok(gc_status) => self.update_gc_metrics(&ds.store, &gc_status),
                Err(e) => {
                    error!("Failed to get GC status for {}: {}", ds.store, e);
                }
            }
        }

        // Collect tape drives
        match self.client.get_tape_drives().await {
            Ok(drives) => self.update_tape_metrics(&drives),
            Err(e) => {
                error!("Failed to get tape drives: {}", e);
            }
        }

        // Collect version info
        let version = self.client.get_version().await?;
        self.update_version_metrics(&version);

        Ok(())
    }

    fn update_snapshot_metrics(
        &self,
        datastore: &str,
        snapshots: &[crate::client::Snapshot],
        comment_map: &std::collections::HashMap<(String, String), (i64, Option<String>)>,
    ) {
        debug!(
            "Updating individual snapshot metrics for {} snapshots in {}",
            snapshots.len(),
            datastore
        );

        // Reset metrics for this datastore to prevent stale data when limits change
        // Note: This clears ALL metrics in the vec, not just for this datastore.
        // This is acceptable if we rebuild all metrics every scrape.
        // If we process multiple datastores sequentially, we should NOT reset here if we share the GaugeVec.
        // However, standard prometheus pattern is to gather everything fresh.
        // But since we call this function multiple times (once per datastore), we can't simple reset() here
        // without wiping previous datastores' work.
        //
        // A better approach for per-datastore isolation would be to use a separate registry or
        // to be smart about what we remove. But GaugeVec doesn't easily support "remove by matching label".
        //
        // "No shortcuts": The correct way if we share GaugeVecs across datastores is to NOT reset here,
        // but reset ONCE at the start of the entire collection cycle.

        // Efficient filtering: Sort by Group (Type, ID) then Time Descending
        let mut sorted_snapshots: Vec<_> = snapshots.iter().collect();
        sorted_snapshots.sort_by(|a, b| {
            a.backup_type
                .cmp(&b.backup_type)
                .then_with(|| a.backup_id.cmp(&b.backup_id))
                .then_with(|| b.backup_time.cmp(&a.backup_time)) // Descending time
        });

        let mut exposed_count = 0;
        let mut current_group = None;
        let mut group_counter = 0;

        for snapshot in sorted_snapshots {
            let group_key = (&snapshot.backup_type, &snapshot.backup_id);

            if Some(group_key) != current_group {
                current_group = Some(group_key);
                group_counter = 0;
            }

            if self.snapshot_history_limit > 0 && group_counter >= self.snapshot_history_limit {
                continue;
            }
            group_counter += 1;
            exposed_count += 1;

            let (size, comment) = if let Some((_time, s_comment)) =
                comment_map.get(&(snapshot.backup_type.clone(), snapshot.backup_id.clone()))
            {
                (
                    snapshot.size.unwrap_or(0) as i64,
                    s_comment.as_deref().unwrap_or(EMPTY_STR),
                )
            } else {
                (snapshot.size.unwrap_or(0) as i64, EMPTY_STR)
            };

            // Truncate comment - use string slice to avoid allocation
            let safe_comment = if comment.len() > 50 {
                &comment[..47]
            } else {
                comment
            };

            let timestamp_seconds = snapshot.backup_time;
            let timestamp_str = timestamp_seconds.to_string();

            // Base labels for most metrics
            let base_labels = [
                datastore,
                &snapshot.backup_type,
                &snapshot.backup_id,
                safe_comment,
                &timestamp_str,
            ];

            // Info metric (timestamp)
            self.snapshot_info
                .with_label_values(&base_labels)
                .set(timestamp_seconds as f64);

            // Verification logic
            let (verified_val, verified_str, verify_time) =
                if let Some(ver) = &snapshot.verification {
                    let is_ok = ver.state == OK;
                    (
                        if is_ok { 1.0 } else { 0.0 },
                        if is_ok { "true" } else { "false" },
                        ver.last_verify,
                    )
                } else {
                    (0.0, "false", None)
                };

            // Size metric needs extra "verified" label
            let size_labels = [
                datastore,
                &snapshot.backup_type,
                &snapshot.backup_id,
                safe_comment,
                &timestamp_str,
                verified_str,
            ];

            self.snapshot_size_bytes
                .with_label_values(&size_labels)
                .set(size as f64);

            // Verification timestamp metric
            if let Some(ts) = verify_time {
                self.snapshot_verification_timestamp
                    .with_label_values(&base_labels)
                    .set(ts as f64);
            }

            self.snapshot_verified
                .with_label_values(&base_labels)
                .set(verified_val);

            // Protection status
            let protected = if snapshot.protected.unwrap_or(false) {
                1.0
            } else {
                0.0
            };
            self.snapshot_protected
                .with_label_values(&base_labels)
                .set(protected);
        }

        debug!(
            "Exposed {}/{} snapshots for datastore {} (limit: {})",
            exposed_count,
            snapshots.len(),
            datastore,
            self.snapshot_history_limit
        );
    }

    fn update_task_metrics(
        &self,
        tasks: &[crate::client::Task],
        comment_map: &std::collections::HashMap<String, String>,
    ) {
        debug!("Updating task metrics for {} tasks", tasks.len());

        // Count tasks by type and status and comment - use &str to avoid clones
        let mut task_counts: std::collections::HashMap<(&str, &str, &str), u64> =
            std::collections::HashMap::new();
        // Count running tasks by type and comment
        let mut running_counts: std::collections::HashMap<(&str, &str), u64> =
            std::collections::HashMap::new();

        for task in tasks {
            // Use as_deref to avoid cloning
            let comment = task.comment.as_deref().unwrap_or_else(|| {
                // If comment is empty, try to look it up in the map (from snapshots)
                task.worker_id
                    .as_ref()
                    .and_then(|wid| comment_map.get(wid.as_str()))
                    .map(|s| s.as_str())
                    .unwrap_or(EMPTY_STR)
            });

            // Unwrap status or use "unknown"
            let status = task.status.as_deref().unwrap_or(UNKNOWN);

            // Count by type and status and comment - clone only when inserting
            let key = (task.worker_type.as_str(), status, comment);
            *task_counts.entry(key).or_insert(0) += 1;

            // Track currently running tasks
            // If endtime is None, it's running. status might be "running" or something else.
            // pbs_task_running only tracks worker_type and comment
            if task.endtime.is_none() || status == RUNNING {
                let run_key = (task.worker_type.as_str(), comment);
                *running_counts.entry(run_key).or_insert(0) += 1;
            } else if let Some(endtime) = task.endtime {
                // Calculate duration for finished tasks
                let duration = endtime - task.starttime;
                // Use empty string for worker_id if None
                let worker_id = task.worker_id.as_deref().unwrap_or(UNKNOWN);

                self.task_duration_seconds
                    .with_label_values(&[task.worker_type.as_str(), status, worker_id, comment])
                    .set(duration as f64);

                // Update last run timestamp
                self.task_last_run_timestamp
                    .with_label_values(&[&task.worker_type])
                    .set(endtime as f64);
            }
        }

        // Update total task counts
        for ((worker_type, status, comment), count) in task_counts {
            self.task_total
                .with_label_values(&[&worker_type, &status, &comment])
                .set(count as f64);
        }

        // Update running task counts
        for ((worker_type, comment), count) in running_counts {
            self.task_running
                .with_label_values(&[&worker_type, &comment])
                .set(count as f64);
        }
    }

    fn update_gc_metrics(&self, datastore: &str, gc_status: &crate::client::GcStatus) {
        debug!("Updating GC metrics for {}", datastore);

        if let Some(timestamp) = gc_status.last_run_endtime {
            self.gc_last_run_timestamp
                .with_label_values(&[datastore])
                .set(timestamp as f64);
        }

        if let Some(duration) = gc_status.duration {
            self.gc_duration_seconds
                .with_label_values(&[datastore])
                .set(duration);
        }

        if let Some(removed) = gc_status.removed_bytes {
            self.gc_removed_bytes
                .with_label_values(&[datastore])
                .set(removed as f64);
        }

        if let Some(pending) = gc_status.pending_bytes {
            self.gc_pending_bytes
                .with_label_values(&[datastore])
                .set(pending as f64);
        }

        if let Some(state) = &gc_status.last_run_state {
            let status_value = if state.eq_ignore_ascii_case(OK) {
                1.0
            } else {
                0.0
            };
            self.gc_status
                .with_label_values(&[datastore])
                .set(status_value);
        }
    }

    fn update_tape_metrics(&self, drives: &[crate::client::TapeDrive]) {
        debug!("Updating tape metrics for {} drives", drives.len());

        self.tape_drive_available.set(drives.len() as f64);

        for drive in drives {
            let vendor = drive.vendor.as_deref().unwrap_or(UNKNOWN);
            let model = drive.model.as_deref().unwrap_or(UNKNOWN);
            let serial = drive.serial.as_deref().unwrap_or(UNKNOWN);

            self.tape_drive_info
                .with_label_values(&[drive.name.as_str(), vendor, model, serial])
                .set(1.0);
        }
    }

    fn update_node_metrics(&self, status: &NodeStatus) {
        debug!("Updating node metrics");
        self.host_cpu_usage.set(status.cpu);
        self.host_io_wait.set(status.wait);
        self.host_load1.set(status.loadavg[0]);
        self.host_load5.set(status.loadavg[1]);
        self.host_load15.set(status.loadavg[2]);
        self.host_memory_used_bytes.set(status.memory.used as f64);
        self.host_memory_total_bytes.set(status.memory.total as f64);
        self.host_memory_free_bytes.set(status.memory.free as f64);
        self.host_swap_used_bytes.set(status.swap.used as f64);
        self.host_swap_total_bytes.set(status.swap.total as f64);
        self.host_swap_free_bytes.set(status.swap.free as f64);
        self.host_rootfs_used_bytes.set(status.root.used as f64);
        self.host_rootfs_total_bytes.set(status.root.total as f64);
        self.host_rootfs_avail_bytes.set(status.root.avail as f64);
        self.host_uptime_seconds.set(status.uptime as f64);
    }

    fn update_datastore_metrics(&self, datastores: &[DatastoreUsage]) {
        debug!(
            "Updating datastore metrics for {} datastores",
            datastores.len()
        );
        for ds in datastores {
            self.datastore_total_bytes
                .with_label_values(&[&ds.store])
                .set(ds.total as f64);
            self.datastore_used_bytes
                .with_label_values(&[&ds.store])
                .set(ds.used as f64);
            self.datastore_available_bytes
                .with_label_values(&[&ds.store])
                .set(ds.avail as f64);
        }
    }

    fn update_backup_metrics(
        &self,
        datastore: &str,
        groups: &[BackupGroup],
        comment_map: &std::collections::HashMap<(String, String), (i64, Option<String>)>,
    ) {
        debug!(
            "Updating backup metrics for {} groups in {}",
            groups.len(),
            datastore
        );
        for group in groups {
            // Get comment from the latest snapshot via comment_map
            let key = (group.backup_type.clone(), group.backup_id.clone());
            let comment = comment_map
                .get(&key)
                .and_then(|(_time, comment)| comment.as_deref())
                .unwrap_or(EMPTY_STR);

            // Truncate comment to 50 chars for Prometheus label compatibility
            let truncated_comment = if comment.len() > 50 {
                &comment[..50]
            } else {
                comment
            };

            let labels = &[
                datastore,
                &group.backup_type,
                &group.backup_id,
                truncated_comment,
            ];

            self.snapshot_count
                .with_label_values(labels)
                .set(group.backup_count as f64);

            self.snapshot_last_timestamp_seconds
                .with_label_values(labels)
                .set(group.last_backup as f64);
        }
    }

    fn update_version_metrics(&self, version: &VersionInfo) {
        debug!("Updating version metrics: {}", version.version);
        self.pbs_version
            .with_label_values(&[&version.version, &version.release, &version.repoid])
            .set(1.0);
    }

    /// Encode metrics in Prometheus text format.
    pub fn encode(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();

        // Reuse buffer to avoid allocation on every scrape
        thread_local! {
            static BUFFER: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(Vec::with_capacity(8192));
        }

        BUFFER.with(|buf| {
            let mut buffer = buf.borrow_mut();
            buffer.clear();

            encoder
                .encode(&metric_families, &mut *buffer)
                .map_err(|e| PbsError::Metrics(e.to_string()))?;

            String::from_utf8(buffer.clone()).map_err(|e| PbsError::Metrics(e.to_string()))
        })
    }
}
