# PBS Exporter - New Snapshot Metrics

## Summary

Added 4 new metrics that expose detailed information about individual snapshots, enabling timeline visualization and detailed monitoring.

## New Metrics

### 1. `pbs_snapshot_info`

**Purpose**: Snapshot timeline with individual timestamps  
**Value**: Unix timestamp of the snapshot  
**Labels**: `datastore`, `backup_type`, `backup_id`, `comment`, `timestamp`

**Example**:

```prometheus
pbs_snapshot_info{backup_id="100",backup_type="ct",comment="pihole",datastore="truenas-backup",timestamp="1762702206"} 1762702206
```

**Use Case**: Visualize backup timeline in Grafana, see backup frequency patterns

### 2. `pbs_snapshot_size_bytes`

**Purpose**: Individual snapshot sizes  
**Value**: Size in bytes  
**Labels**: `datastore`, `backup_type`, `backup_id`, `comment`, `timestamp`

**Example**:

```prometheus
pbs_snapshot_size_bytes{backup_id="100",backup_type="ct",comment="pihole",datastore="truenas-backup",timestamp="1762702206"} 1730677857
```

**Use Case**: Track storage growth over time, identify large backups, capacity planning

### 3. `pbs_snapshot_verified`

**Purpose**: Verification status  
**Value**: `1` = verified OK, `0` = failed/unknown  
**Labels**: `datastore`, `backup_type`, `backup_id`, `comment`, `timestamp`

**Example**:

```prometheus
pbs_snapshot_verified{backup_id="100",backup_type="ct",comment="pihole",datastore="truenas-backup",timestamp="1762702206"} 1
```

**Use Case**: Alert on verification failures, monitor backup integrity

### 4. `pbs_snapshot_protected`

**Purpose**: Protection status  
**Value**: `1` = protected, `0` = not protected  
**Labels**: `datastore`, `backup_type`, `backup_id`, `comment`, `timestamp`

**Example**:

```prometheus
pbs_snapshot_protected{backup_id="100",backup_type="ct",comment="pihole",datastore="truenas-backup",timestamp="1762702206"} 0
```

**Use Case**: Track which snapshots are protected from pruning

## Impact

**Before**: 106 lines of metrics  
**After**: 2018 lines of metrics

**What this enables**:

- Full backup timeline visualization in Grafana
- Storage growth trends per VM/CT
- Verification status monitoring
- Protection status tracking
- Historical analysis of backup patterns

## Grafana Query Examples

### Timeline Visualization

```promql
pbs_snapshot_info{backup_id="100"}
```

### Storage Growth Over Time

```promql
pbs_snapshot_size_bytes{backup_id="100"}
```

### Failed Verifications

```promql
pbs_snapshot_verified == 0
```

### Protected Snapshots Count

```promql
count(pbs_snapshot_protected == 1) by (backup_id)
```

## Technical Details

**Data Source**: `/api2/json/admin/datastore/{store}/snapshots`  
**Fields Used**: `backup-time`, `size`, `verification.state`, `protected`  
**Label Truncation**: Comments limited to 50 characters for Prometheus compatibility
