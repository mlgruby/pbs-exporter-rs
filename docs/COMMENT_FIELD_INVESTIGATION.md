# PBS Comment Field Investigation

## Issue

Comments are visible in the PBS web UI but not appearing in the exporter metrics.

## Investigation Results

### API Response Analysis

The PBS API endpoint `/admin/datastore/{store}/groups` returns:

```json
{
  "data": [{
    "backup-count": 27,
    "backup-id": "100",
    "backup-type": "ct",
    "files": [...],
    "last-backup": 1766800804,
    "owner": "pbs@pbs"
  }]
}
```

**Fields returned:**

- ✅ `backup-count`
- ✅ `backup-id`
- ✅ `backup-type`
- ✅ `files`
- ✅ `last-backup`
- ✅ `owner`
- ❌ **`comment` - NOT INCLUDED**

## Root Cause

The `/admin/datastore/{store}/groups` endpoint does **not** return the comment field, even though comments are visible in the PBS web UI.

## Possible Solutions

### Option 1: Different API Endpoint

Comments might be available from a different endpoint. Need to check:

- `/admin/datastore/{store}/groups/{type}/{id}` - Individual group details
- `/admin/datastore/{store}/snapshots` - Snapshot-level data
- PBS API documentation for comment-specific endpoints

### Option 2: Additional Query Parameters

The endpoint might support query parameters to include comments:

- `?include-comment=1`
- `?full=1`
- Other undocumented parameters

### Option 3: Comments Not Available via API

Comments might only be stored in the web UI's database and not exposed via the REST API in PBS 4.x.

## Current Status

**Exporter Implementation:**

- ✅ Code supports comment field (with 50-char truncation)
- ✅ Comment added as 4th Prometheus label
- ❌ PBS API doesn't return comments, so they show as empty (`comment=""`)

**Recommendation:**

1. Check PBS API documentation for comment support
2. Test alternative API endpoints
3. If comments aren't available via API, document this limitation

## Workaround

If comments aren't available via PBS API, users can:

1. Maintain a separate mapping file (backup_id → comment)
2. Use Prometheus relabeling to add comments from external source
3. Request PBS developers to add comment field to API response

## Files Modified

- `src/client.rs` - Added debug logging for API responses
- `src/metrics.rs` - Added comment as 4th label (ready when API supports it)
