# PBS Exporter - Metrics Coverage Report

## Currently Captured Metrics ✅

### Host System Metrics

| Metric | Description | Source |
| -------- | ------------- | -------- |
| `pbs_host_cpu_usage` | CPU usage (0.0-1.0) | `/nodes/localhost/status` |
| `pbs_host_io_wait` | I/O wait fraction | `/nodes/localhost/status` |
| `pbs_host_load1` | 1-minute load average | `/nodes/localhost/status` |
| `pbs_host_load5` | 5-minute load average | `/nodes/localhost/status` |
| `pbs_host_load15` | 15-minute load average | `/nodes/localhost/status` |
| `pbs_host_memory_used_bytes` | Used RAM | `/nodes/localhost/status` |
| `pbs_host_memory_total_bytes` | Total RAM | `/nodes/localhost/status` |
| `pbs_host_memory_free_bytes` | Free RAM | `/nodes/localhost/status` |
| `pbs_host_swap_used_bytes` | Used swap | `/nodes/localhost/status` |
| `pbs_host_swap_total_bytes` | Total swap | `/nodes/localhost/status` |
| `pbs_host_swap_free_bytes` | Free swap | `/nodes/localhost/status` |
| `pbs_host_rootfs_used_bytes` | Root FS used | `/nodes/localhost/status` |
| `pbs_host_rootfs_total_bytes` | Root FS total | `/nodes/localhost/status` |
| `pbs_host_rootfs_avail_bytes` | Root FS available | `/nodes/localhost/status` |
| `pbs_host_uptime_seconds` | System uptime | `/nodes/localhost/status` |

### Datastore Metrics

| Metric | Description | Labels |
| -------- | ------------- | -------- |
| `pbs_datastore_total_bytes` | Total datastore size | `datastore` |
| `pbs_datastore_used_bytes` | Used space | `datastore` |
| `pbs_datastore_available_bytes` | Available space | `datastore` |

### Backup Metrics

| Metric | Description | Labels |
| -------- | ------------- | -------- |
| `pbs_snapshot_count` | Number of snapshots | `datastore`, `backup_type`, `backup_id` |
| `pbs_snapshot_last_timestamp_seconds` | Last backup time (Unix) | `datastore`, `backup_type`, `backup_id` |
| `pbs_snapshot_info` | Snapshot timeline info (value=timestamp) | `datastore`, `backup_type`, `backup_id`, `comment`, `timestamp` |
| `pbs_snapshot_size_bytes` | Snapshot size | `datastore`, `backup_type`, `backup_id`, `comment`, `timestamp` |
| `pbs_snapshot_verified` | Verification status (1=ok, 0=other) | `datastore`, `backup_type`, `backup_id`, `comment`, `timestamp` |
| `pbs_snapshot_protected` | Protected status (1=yes, 0=no) | `datastore`, `backup_type`, `backup_id`, `comment`, `timestamp` |

### Task Metrics

| Metric | Description | Labels |
| -------- | ------------- | -------- |
| `pbs_task_total` | Total tasks count | `worker_type`, `status` |

- **`pbs_task_duration_seconds`**
  - Labels: `worker_type`, `status`, `worker_id`, `comment`
  - Value: Duration in seconds
  - *Note*: `comment` is populated from the task itself, or correlated from the latest snapshot for backup tasks if the task comment is empty.
- **`pbs_task_last_run_timestamp_seconds`**
  - Labels: `worker_type`
  - Value: Unix timestamp of last run
| `pbs_task_running` | Currently running tasks | `worker_type` |

### GC Metrics

| Metric | Description | Labels |
| -------- | ------------- | -------- |
| `pbs_gc_last_run_timestamp` | Last GC run time | `datastore` |
| `pbs_gc_duration_seconds` | Last GC duration | `datastore`, `status` |
| `pbs_gc_removed_bytes` | Bytes removed in GC | `datastore` |
| `pbs_gc_pending_bytes` | Bytes pending removal | `datastore` |
| `pbs_gc_status` | Last GC status (1=ok) | `datastore` |

### Tape Metrics

| Metric | Description | Labels |
| -------- | ------------- | -------- |
| `pbs_tape_drive_info` | Tape drive info (1=present) | `name`, `vendor`, `model`, `serial` |
| `pbs_tape_drive_available` | Available tape drives | - |

### Exporter Metrics

| Metric | Description | Labels |
| -------- | ------------- | -------- |
| `pbs_up` | Scrape success (1=success, 0=fail) | - |
| `pbs_version` | PBS version info | `version`, `release`, `repoid` |

## Configuration Options (`config/local.toml`)

### `pbs.snapshot_history_limit`

Controls how many snapshots per backup group are exposed as metrics.

- `0` (default): Expose **all** snapshots (full timeline).
- `1`: Expose only the **latest** snapshot per group.
- `N`: Expose the latest `N` snapshots per group.

This helps control metric cardinality in environments with long retention policies.

## Missing Metrics ❌

### Network I/O

**Status**: Not available in PBS `/nodes/localhost/status` API  
**Reason**: PBS API doesn't expose per-interface network statistics  
**Workaround**: Use node_exporter on PBS host for detailed network metrics

### Disk I/O  

**Status**: Not available in PBS `/nodes/localhost/status` API  
**Reason**: PBS API doesn't expose disk I/O statistics (reads/writes per second)  
**Workaround**: Use node_exporter on PBS host for detailed disk I/O metrics

### Backup Comments

**Status**: Available but not currently used  
**Issue**: Comments can be very long and Prometheus labels should be short  
**Solution**: Can add as optional label with truncation

## Recommendations

### For Complete System Monitoring

Deploy **node_exporter** on your PBS host to get:

- Network I/O (bytes/packets in/out per interface)
- Disk I/O (reads/writes, IOPS, latency)
- Per-CPU statistics
- Detailed filesystem metrics
- Process statistics

**Setup:**

```bash
# On PBS host
apt-get install prometheus-node-exporter
systemctl enable prometheus-node-exporter
systemctl start prometheus-node-exporter
```

Then scrape both:

- `pbs-exporter:9876/metrics` - PBS-specific metrics
- `pbs-host:9100/metrics` - System metrics via node_exporter

### For Backup Comments

**Option 1**: Add as truncated label (max 50 chars)

- Pros: Visible in Prometheus
- Cons: Long comments get cut off

**Option 2**: Store in separate info metric

- Pros: Full comment preserved
- Cons: Requires joining queries

**Option 3**: Don't include

- Pros: Keeps metrics clean
- Cons: Comments not visible

## What You're Getting

Your current setup captures:

- **CPU**: Usage, I/O wait, load averages
- **Memory**: RAM and swap usage
- **Disk**: Root filesystem capacity
- **Storage**: Datastore capacity per datastore
- **Backups**: Snapshot counts and last backup times
- **System**: Uptime, PBS version

- **Network I/O**: Not in PBS API (use node_exporter)
- **Disk I/O**: Not in PBS API (use node_exporter)
- **Comments**: Available but not exposed (can add if needed)

## Next Steps

1. **Add node_exporter** to PBS host for network/disk I/O
2. **Decide on comments**: Do you want them as labels?
3. **Create Grafana dashboard** combining both exporters

Would you like me to:

- Add comment support (with truncation)?
- Create a Grafana dashboard JSON?
- Document node_exporter setup?
