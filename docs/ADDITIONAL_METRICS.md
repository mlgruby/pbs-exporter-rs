# Additional Metrics - Potential Enhancements

Based on the PBS API investigation, here are additional metrics we could add:

## Currently Implemented

1. **Host Metrics**: CPU, Memory, Swap, Disk, Load, Uptime
2. **Datastore Metrics**: Total, Used, Available space
3. **Backup Group Metrics**: Snapshot count, Last backup time
4. **Individual Snapshot Metrics**: Timeline, Size, Verification, Protection, Comments

## Available But Not Yet Implemented

### 1. File-Level Metrics

**Source**: `files` array in snapshot data  
**Data Available**:

- Individual file names (`pct.conf.blob`, `root.pxar.didx`, etc.)
- File sizes
- Encryption mode (`none`, `encrypt`, `sign-only`)

**Potential Metrics**:

```prometheus
pbs_snapshot_file_size_bytes{...,filename="root.pxar.didx"} 1826393751
pbs_snapshot_file_encrypted{...,filename="root.pxar.didx"} 0
pbs_snapshot_files_count{...} 3
```

**Use Case**: Track which files are encrypted, identify large files

**Complexity**: HIGH (would add many metrics - each snapshot has 3-5 files)  
**Value**: MEDIUM (useful for encryption auditing)

### 2. Ownership Metrics

**Source**: `owner` field in snapshot data  
**Data Available**: Owner user/token (e.g., `pbs@pbs`, `root@pam!backup`)

**Potential Metrics**:

```prometheus
pbs_snapshot_info{...,owner="pbs@pbs"} 1762702206
```

**Use Case**: Track which user/token created backups  
**Complexity**: LOW (just add owner as label)  
**Value**: LOW (usually same owner for all backups)

### 3. Task/Job Metrics

**Source**: PBS has `/api2/json/admin/tasks` endpoint  
**Data Available**:

- Running tasks
- Task status (OK, ERROR, WARNING)
- Task duration
- Task type (backup, prune, verify, sync)

**Potential Metrics**:

```prometheus
pbs_task_running{type="verify"} 2
pbs_task_duration_seconds{type="backup",status="ok"} 145.3
pbs_task_last_run_timestamp{type="prune"} 1762702206
```

**Use Case**: Monitor backup jobs, alert on failed tasks  
**Complexity**: MEDIUM (new API endpoint)  
**Value**: HIGH (very useful for operations)

### 4. Prune/GC Metrics

**Source**: PBS has prune and garbage collection status  
**Data Available**:

- Last prune time
- Last GC time
- Space reclaimed
- Pending GC bytes

**Potential Metrics**:

```prometheus
pbs_datastore_last_gc_timestamp{datastore="..."} 1762702206
pbs_datastore_gc_pending_bytes{datastore="..."} 5368709120
pbs_datastore_last_prune_timestamp{datastore="..."} 1762702206
```

**Use Case**: Ensure GC is running, monitor space reclamation  
**Complexity**: MEDIUM (new API endpoints)  
**Value**: HIGH (important for storage management)

### 5. Sync Job Metrics (for remote datastores)

**Source**: `/api2/json/admin/sync` endpoint  
**Data Available**:

- Sync job status
- Last sync time
- Sync errors
- Remote datastore info

**Potential Metrics**:

```prometheus
pbs_sync_last_run_timestamp{job="remote-backup"} 1762702206
pbs_sync_status{job="remote-backup",status="ok"} 1
```

**Use Case**: Monitor replication to remote PBS  
**Complexity**: MEDIUM  
**Value**: HIGH (if using sync/replication)

### 6. Tape Backup Metrics (if applicable)

**Source**: PBS tape backup API  
**Data Available**:

- Tape pool status
- Media status
- Backup to tape jobs

**Potential Metrics**:

```prometheus
pbs_tape_media_available{pool="daily"} 5
pbs_tape_backup_last_run{pool="daily"} 1762702206
```

**Use Case**: Monitor tape backups  
**Complexity**: HIGH  
**Value**: HIGH (if using tapes)

## Recommendations

### High Priority (Should Add)

1. **Task/Job Metrics** - Essential for monitoring backup operations
2. **Prune/GC Metrics** - Important for storage management

### Medium Priority (Nice to Have)

1. **File-Level Encryption Status** - Useful for security auditing
2. **Sync Job Metrics** - If using replication

### Low Priority (Optional)

1. **Owner Labels** - Low value, adds cardinality
2. **Tape Metrics** - Only if using tape backups

## Implementation Effort

**Quick Wins** (1-2 hours):

- Add `owner` as label to existing metrics
- Add file count metric

**Medium Effort** (2-4 hours):

- Task/Job metrics
- Prune/GC metrics

**Large Effort** (4+ hours):

- File-level detailed metrics
- Sync job metrics
- Tape metrics

## Current Status

**Metrics Count**: 2018 lines  
**API Calls per Scrape**:

- 1x `/nodes/localhost/status`
- 1x `/admin/datastore`
- 1x `/admin/datastore/{store}/groups` per datastore
- 1x `/admin/datastore/{store}/snapshots` per datastore
- 1x `/version`

**Adding task metrics would add**: ~1 more API call  
**Adding GC metrics would add**: ~1 more API call per datastore

## My Recommendation

**Add next**:

1. **Task/Job Metrics** - Most valuable for operations
2. **GC/Prune Status** - Important for storage health

**Skip for now**:

- File-level details (too much cardinality)
- Owner labels (low value)
- Tape metrics (niche use case)

Would you like me to implement task/job metrics and GC status?
