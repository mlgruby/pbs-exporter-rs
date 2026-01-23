//! Tests for snapshot cardinality limiting.

use pbs_exporter::client::Snapshot;

/// Helper to create test snapshots for a backup group
fn create_test_snapshots(backup_type: &str, backup_id: &str, count: usize) -> Vec<Snapshot> {
    (0..count)
        .map(|i| Snapshot {
            backup_type: backup_type.to_string(),
            backup_id: backup_id.to_string(),
            backup_time: 1000 + (i as i64 * 100), // Incrementing timestamps
            comment: Some(format!("Snapshot {}", i)),
            size: Some(1024 * (i as u64 + 1)),
            verification: None,
            protected: Some(false),
        })
        .collect()
}

// WGT: Test snapshot history limit of 0 exposes all snapshots
#[test]
fn test_snapshot_history_limit_zero() {
    // Given: 10 snapshots for a single backup group and limit = 0
    let snapshots = create_test_snapshots("vm", "100", 10);
    let snapshot_history_limit = 0;

    // When: Sorting and applying the history limit logic
    let mut sorted_snapshots: Vec<_> = snapshots.iter().collect();
    sorted_snapshots.sort_by(|a, b| {
        a.backup_type
            .cmp(&b.backup_type)
            .then_with(|| a.backup_id.cmp(&b.backup_id))
            .then_with(|| b.backup_time.cmp(&a.backup_time))
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

        if snapshot_history_limit > 0 && group_counter >= snapshot_history_limit {
            continue;
        }
        group_counter += 1;
        exposed_count += 1;
    }

    // Then: All 10 snapshots should be exposed
    assert_eq!(exposed_count, 10);
}

// WGT: Test snapshot history limit of 1 exposes only latest snapshot
#[test]
fn test_snapshot_history_limit_one() {
    // Given: 10 snapshots for a single backup group and limit = 1
    let snapshots = create_test_snapshots("vm", "100", 10);
    let snapshot_history_limit = 1;

    // When: Sorting and applying the history limit logic
    let mut sorted_snapshots: Vec<_> = snapshots.iter().collect();
    sorted_snapshots.sort_by(|a, b| {
        a.backup_type
            .cmp(&b.backup_type)
            .then_with(|| a.backup_id.cmp(&b.backup_id))
            .then_with(|| b.backup_time.cmp(&a.backup_time))
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

        if snapshot_history_limit > 0 && group_counter >= snapshot_history_limit {
            continue;
        }
        group_counter += 1;
        exposed_count += 1;
    }

    // Then: Only 1 snapshot should be exposed
    assert_eq!(exposed_count, 1);
}

// WGT: Test snapshot history limit of 5 exposes 5 most recent snapshots
#[test]
fn test_snapshot_history_limit_five() {
    // Given: 10 snapshots for a single backup group and limit = 5
    let snapshots = create_test_snapshots("vm", "100", 10);
    let snapshot_history_limit = 5;

    // When: Sorting and applying the history limit logic
    let mut sorted_snapshots: Vec<_> = snapshots.iter().collect();
    sorted_snapshots.sort_by(|a, b| {
        a.backup_type
            .cmp(&b.backup_type)
            .then_with(|| a.backup_id.cmp(&b.backup_id))
            .then_with(|| b.backup_time.cmp(&a.backup_time))
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

        if snapshot_history_limit > 0 && group_counter >= snapshot_history_limit {
            continue;
        }
        group_counter += 1;
        exposed_count += 1;
    }

    // Then: Only 5 snapshots should be exposed
    assert_eq!(exposed_count, 5);
}

// WGT: Test snapshot history limiting across multiple backup groups
#[test]
fn test_snapshot_history_limit_multiple_groups() {
    // Given: Multiple backup groups with various snapshot counts and limit = 3
    let mut snapshots = Vec::new();
    snapshots.extend(create_test_snapshots("vm", "100", 10));
    snapshots.extend(create_test_snapshots("vm", "101", 8));
    snapshots.extend(create_test_snapshots("ct", "200", 6));

    let snapshot_history_limit = 3;

    // When: Sorting and applying the history limit logic
    let mut sorted_snapshots: Vec<_> = snapshots.iter().collect();
    sorted_snapshots.sort_by(|a, b| {
        a.backup_type
            .cmp(&b.backup_type)
            .then_with(|| a.backup_id.cmp(&b.backup_id))
            .then_with(|| b.backup_time.cmp(&a.backup_time))
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

        if snapshot_history_limit > 0 && group_counter >= snapshot_history_limit {
            continue;
        }
        group_counter += 1;
        exposed_count += 1;
    }

    // Then: Should expose 3 snapshots per group (3*3 = 9 total)
    assert_eq!(exposed_count, 9);
}

// WGT: Test snapshot sorting order (type, id, time DESC)
#[test]
fn test_snapshot_sorting_order() {
    // Given: Snapshots with different types, ids, and times in unsorted order
    let mut snapshots = [
        Snapshot {
            backup_type: "vm".to_string(),
            backup_id: "100".to_string(),
            backup_time: 1000,
            comment: None,
            size: None,
            verification: None,
            protected: None,
        },
        Snapshot {
            backup_type: "ct".to_string(),
            backup_id: "200".to_string(),
            backup_time: 2000,
            comment: None,
            size: None,
            verification: None,
            protected: None,
        },
        Snapshot {
            backup_type: "vm".to_string(),
            backup_id: "100".to_string(),
            backup_time: 3000,
            comment: None,
            size: None,
            verification: None,
            protected: None,
        },
    ];

    // When: Sorting by type, then id, then time DESC
    snapshots.sort_by(|a, b| {
        a.backup_type
            .cmp(&b.backup_type)
            .then_with(|| a.backup_id.cmp(&b.backup_id))
            .then_with(|| b.backup_time.cmp(&a.backup_time)) // Descending time
    });

    // Then: Snapshots should be ordered: ct/200 (2000), vm/100 (3000), vm/100 (1000)
    assert_eq!(snapshots[0].backup_type, "ct");
    assert_eq!(snapshots[0].backup_id, "200");
    assert_eq!(snapshots[0].backup_time, 2000);

    assert_eq!(snapshots[1].backup_type, "vm");
    assert_eq!(snapshots[1].backup_id, "100");
    assert_eq!(snapshots[1].backup_time, 3000); // Newer first

    assert_eq!(snapshots[2].backup_type, "vm");
    assert_eq!(snapshots[2].backup_id, "100");
    assert_eq!(snapshots[2].backup_time, 1000); // Older second
}
