# Metrics Registration - Implementation Reference

Due to the large size of the metric registration code (~150 lines), this document serves as a reference for the implementation.

## Summary

Adding 11 new metrics across 3 categories:

**Task Metrics (4)**:

- `pbs_task_total` - Count by type and status
- `pbs_task_duration_seconds` - Duration per task
- `pbs_task_last_run_timestamp` - Last run time
- `pbs_task_running` - Currently running count

**GC Metrics (5)**:

- `pbs_gc_last_run_timestamp` - Last GC time
- `pbs_gc_duration_seconds` - GC duration
- `pbs_gc_removed_bytes` - Space reclaimed
- `pbs_gc_pending_bytes` - Reclaimable space
- `pbs_gc_status` - Last GC result

**Tape Metrics (2)**:

- `pbs_tape_drive_info` - Drive details
- `pbs_tape_drive_available` - Drive count

## Implementation Status

✅ Struct fields added to MetricsCollector  
⏳ Registration code (next step)  
⏳ Update functions (after registration)  
⏳ Collection logic (final step)

The registration follows the same pattern as existing metrics, creating GaugeVec instances and registering them with the Prometheus registry.
