//! Metric collection orchestration logic.

use super::MetricsCollector;
use crate::error::Result;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{error, info};

/// Collect all metrics from PBS.
pub(super) async fn collect(collector: &MetricsCollector) -> Result<()> {
    info!("Collecting metrics from PBS");
    let start = Instant::now();

    let result = match collect_internal(collector).await {
        Ok(_) => {
            collector.metrics().pbs_up.set(1.0);
            info!("Successfully collected metrics");
            Ok(())
        }
        Err(e) => {
            error!("Failed to collect metrics: {}", e);
            collector.metrics().pbs_up.set(0.0);
            Err(e)
        }
    };

    // Update scrape duration
    let duration = start.elapsed().as_secs_f64();
    collector
        .metrics()
        .exporter_scrape_duration_seconds
        .set(duration);

    // Update memory usage (Linux only)
    if let Ok(memory_bytes) = get_memory_usage() {
        collector
            .metrics()
            .exporter_memory_usage_bytes
            .set(memory_bytes as f64);
    }

    result
}

/// Get current memory usage in bytes (Linux only).
fn get_memory_usage() -> Result<u64> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        // Read /proc/self/statm: pages in virtual memory, resident set, shared, text, data
        let statm = fs::read_to_string("/proc/self/statm").map_err(crate::error::PbsError::Io)?;
        let parts: Vec<&str> = statm.split_whitespace().collect();
        if parts.len() >= 2 {
            // Second field is RSS in pages
            let rss_pages: u64 = parts[1]
                .parse()
                .map_err(|_| crate::error::PbsError::Metrics("Failed to parse RSS".to_string()))?;
            // Page size is typically 4096 bytes
            let page_size = 4096u64;
            return Ok(rss_pages * page_size);
        }
    }
    Ok(0)
}

pub(super) async fn collect_internal(collector: &MetricsCollector) -> Result<()> {
    let metrics = collector.metrics();
    let client = collector.client();

    // Reset all metrics to prevent stale data
    // This is crucial because we populate metrics dynamically based on current API state.
    // If an object (snapshot, task, drive) disappears or is filtered out,
    // we must ensure its corresponding metric is removed.
    metrics.pbs_up.set(0.0); // Will be set to 1.0 on success
    metrics.host_cpu_usage.set(0.0);
    metrics.host_io_wait.set(0.0);
    metrics.host_load1.set(0.0);
    metrics.host_load5.set(0.0);
    metrics.host_load15.set(0.0);
    metrics.host_memory_used_bytes.set(0.0);
    metrics.host_memory_total_bytes.set(0.0);
    metrics.host_memory_free_bytes.set(0.0);
    metrics.host_swap_used_bytes.set(0.0);
    metrics.host_swap_total_bytes.set(0.0);
    metrics.host_swap_free_bytes.set(0.0);
    metrics.host_rootfs_used_bytes.set(0.0);
    metrics.host_rootfs_total_bytes.set(0.0);
    metrics.host_rootfs_avail_bytes.set(0.0);
    metrics.host_uptime_seconds.set(0.0);

    metrics.datastore_total_bytes.reset();
    metrics.datastore_used_bytes.reset();
    metrics.datastore_available_bytes.reset();

    metrics.snapshot_count.reset();
    metrics.snapshot_info.reset();
    metrics.snapshot_size_bytes.reset();
    metrics.snapshot_verified.reset();
    metrics.snapshot_verification_timestamp.reset();
    metrics.snapshot_protected.reset();
    metrics.snapshot_last_timestamp_seconds.reset();

    metrics.task_total.reset();
    metrics.task_duration_seconds.reset();
    metrics.task_last_run_timestamp.reset();
    metrics.task_running.reset();

    metrics.gc_last_run_timestamp.reset();
    metrics.gc_duration_seconds.reset();
    metrics.gc_removed_bytes.reset();
    metrics.gc_pending_bytes.reset();
    metrics.gc_status.reset();

    metrics.tape_drive_info.reset();
    metrics.tape_drive_available.set(0.0);

    metrics.pbs_version.reset();

    // Collect node status
    let node_status = client.get_node_status().await?;
    super::updates::update_node_metrics(metrics, &node_status);

    // Collect datastore usage
    let datastores = client.get_datastore_usage().await?;
    super::updates::update_datastore_metrics(metrics, &datastores);

    // Map to store comments for tasks (worker_id -> comment)
    // Pre-allocate with estimated capacity
    let mut task_comment_map: HashMap<String, String> =
        HashMap::with_capacity(datastores.len() * 10);

    // Collect backup groups and snapshots for each datastore
    for ds in &datastores {
        // Fetch snapshots to get comments
        let snapshots = match client.get_snapshots(&ds.store).await {
            Ok(snaps) => snaps,
            Err(e) => {
                error!("Failed to get snapshots for {}: {}", ds.store, e);
                Vec::new()
            }
        };

        // Build a map of (backup_type, backup_id) -> (latest_time, comment)
        // Pre-allocate with estimated capacity
        let mut comment_map: HashMap<(String, String), (i64, Option<String>)> =
            HashMap::with_capacity(snapshots.len() / 5);

        for snapshot in &snapshots {
            // Use entry API to avoid cloning keys for lookup
            use std::collections::hash_map::Entry;

            match comment_map.entry((snapshot.backup_type.clone(), snapshot.backup_id.clone())) {
                Entry::Occupied(mut e) => {
                    // Only update if this snapshot is newer
                    if snapshot.backup_time > e.get().0 {
                        e.get_mut().0 = snapshot.backup_time;
                        e.get_mut().1 = snapshot.comment.clone();
                    }
                }
                Entry::Vacant(e) => {
                    e.insert((snapshot.backup_time, snapshot.comment.clone()));
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
        super::updates::update_snapshot_metrics(
            metrics,
            &ds.store,
            &snapshots,
            &comment_map,
            collector.snapshot_history_limit,
        );

        // Fetch backup groups
        match client.get_backup_groups(&ds.store).await {
            Ok(groups) => {
                super::updates::update_backup_metrics(metrics, &ds.store, &groups, &comment_map)
            }
            Err(e) => {
                error!("Failed to get backup groups for {}: {}", ds.store, e);
                // Continue with other datastores
            }
        }
    }

    // Collect tasks
    match client.get_tasks(Some(50)).await {
        Ok(tasks) => super::updates::update_task_metrics(metrics, &tasks, &task_comment_map),
        Err(e) => {
            error!("Failed to get tasks: {}", e);
        }
    }

    // Collect GC status for each datastore
    for ds in &datastores {
        match client.get_gc_status(&ds.store).await {
            Ok(gc_status) => super::updates::update_gc_metrics(metrics, &ds.store, &gc_status),
            Err(e) => {
                error!("Failed to get GC status for {}: {}", ds.store, e);
            }
        }
    }

    // Collect tape drives
    match client.get_tape_drives().await {
        Ok(drives) => super::updates::update_tape_metrics(metrics, &drives),
        Err(e) => {
            error!("Failed to get tape drives: {}", e);
        }
    }

    // Collect version info
    let version = client.get_version().await?;
    super::updates::update_version_metrics(metrics, &version);

    Ok(())
}
