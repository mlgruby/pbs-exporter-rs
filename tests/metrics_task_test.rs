//! Tests for task metrics calculation.

use pbs_exporter::client::Task;

// WGT: Test task duration calculation for finished tasks
#[test]
fn test_task_duration_calculation() {
    // Given: A finished task with starttime and endtime
    let finished_task = Task {
        upid: "UPID:test".to_string(),
        worker_type: "backup".to_string(),
        worker_id: Some("datastore:vm/100".to_string()),
        starttime: 1000,
        endtime: Some(1100),
        status: Some("ok".to_string()),
        comment: Some("Test backup".to_string()),
    };

    // When: Calculating the duration
    let duration = finished_task.endtime.unwrap() - finished_task.starttime;

    // Then: Duration should be the difference between endtime and starttime
    assert_eq!(duration, 100);
}

// WGT: Test detection of running tasks without endtime
#[test]
fn test_task_running_detection() {
    // Given: A task without endtime and status "running"
    let running_task = Task {
        upid: "UPID:test".to_string(),
        worker_type: "backup".to_string(),
        worker_id: Some("datastore:vm/100".to_string()),
        starttime: 1000,
        endtime: None,
        status: Some("running".to_string()),
        comment: Some("Test backup".to_string()),
    };

    // When: Checking task properties
    // Then: Task should have no endtime and status should be "running"
    assert!(running_task.endtime.is_none());
    assert_eq!(
        running_task.status.as_deref().unwrap_or("unknown"),
        "running"
    );
}

// WGT: Test handling of different task status values
#[test]
fn test_task_status_variations() {
    // Given: A list of different task status values
    let statuses = vec!["ok", "error", "warning", "unknown", "running"];

    // When: Creating tasks with each status
    for status in statuses {
        let task = Task {
            upid: "UPID:test".to_string(),
            worker_type: "backup".to_string(),
            worker_id: Some("datastore:vm/100".to_string()),
            starttime: 1000,
            endtime: Some(1100),
            status: Some(status.to_string()),
            comment: None,
        };

        // Then: Task status should match the provided value
        assert_eq!(task.status.as_deref().unwrap(), status);
    }
}

// WGT: Test task comment fallback to empty string
#[test]
fn test_task_comment_fallback() {
    // Given: A task with no comment
    let task_no_comment = Task {
        upid: "UPID:test".to_string(),
        worker_type: "backup".to_string(),
        worker_id: Some("datastore:vm/100".to_string()),
        starttime: 1000,
        endtime: Some(1100),
        status: Some("ok".to_string()),
        comment: None,
    };

    // When: Accessing the comment field with fallback
    let comment = task_no_comment.comment.as_deref().unwrap_or("");

    // Then: Comment should fall back to empty string
    assert_eq!(comment, "");
}

// WGT: Test task grouping by worker_type and status
#[test]
fn test_task_grouping_by_type_and_status() {
    // Given: Multiple tasks with different worker_types and statuses
    let tasks = vec![
        Task {
            upid: "UPID:test1".to_string(),
            worker_type: "backup".to_string(),
            worker_id: Some("datastore:vm/100".to_string()),
            starttime: 1000,
            endtime: Some(1100),
            status: Some("ok".to_string()),
            comment: None,
        },
        Task {
            upid: "UPID:test2".to_string(),
            worker_type: "backup".to_string(),
            worker_id: Some("datastore:vm/101".to_string()),
            starttime: 1000,
            endtime: Some(1100),
            status: Some("ok".to_string()),
            comment: None,
        },
        Task {
            upid: "UPID:test3".to_string(),
            worker_type: "backup".to_string(),
            worker_id: Some("datastore:vm/102".to_string()),
            starttime: 1000,
            endtime: Some(1100),
            status: Some("error".to_string()),
            comment: None,
        },
        Task {
            upid: "UPID:test4".to_string(),
            worker_type: "garbage_collection".to_string(),
            worker_id: Some("datastore".to_string()),
            starttime: 1000,
            endtime: Some(1100),
            status: Some("ok".to_string()),
            comment: None,
        },
    ];

    // When: Grouping tasks by worker_type and status
    let mut counts: std::collections::HashMap<(&str, &str), u64> = std::collections::HashMap::new();
    for task in &tasks {
        let key = (
            task.worker_type.as_str(),
            task.status.as_deref().unwrap_or("unknown"),
        );
        *counts.entry(key).or_insert(0) += 1;
    }

    // Then: Counts should match expected groupings
    assert_eq!(counts.get(&("backup", "ok")), Some(&2));
    assert_eq!(counts.get(&("backup", "error")), Some(&1));
    assert_eq!(counts.get(&("garbage_collection", "ok")), Some(&1));
}
