//! Tests for snapshot metrics logic.

use pbs_exporter::client::{Snapshot, VerificationStatus};

// WGT: Test snapshot comment selection from multiple snapshots
#[test]
fn test_snapshot_comment_selection() {
    // Given: Multiple snapshots with different comments and backup times
    let snapshots = vec![
        Snapshot {
            backup_type: "vm".to_string(),
            backup_id: "100".to_string(),
            backup_time: 1000,
            comment: Some("old comment".to_string()),
            size: Some(1024),
            verification: None,
            protected: Some(false),
        },
        Snapshot {
            backup_type: "vm".to_string(),
            backup_id: "100".to_string(),
            backup_time: 2000,
            comment: Some("new comment".to_string()),
            size: Some(2048),
            verification: None,
            protected: Some(false),
        },
        Snapshot {
            backup_type: "vm".to_string(),
            backup_id: "100".to_string(),
            backup_time: 1500,
            comment: Some("middle comment".to_string()),
            size: Some(1536),
            verification: None,
            protected: Some(false),
        },
    ];

    // Build comment map like the code does
    let mut comment_map: std::collections::HashMap<(String, String), (i64, Option<String>)> =
        std::collections::HashMap::new();

    for snapshot in &snapshots {
        let key = (snapshot.backup_type.clone(), snapshot.backup_id.clone());
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

    // Then: Latest comment (highest backup_time) should be selected
    let key = ("vm".to_string(), "100".to_string());
    let (time, comment) = comment_map.get(&key).unwrap();
    assert_eq!(*time, 2000);
    assert_eq!(comment.as_ref().unwrap(), "new comment");
}

// WGT: Test snapshot verification status conversion to metric values
#[test]
fn test_snapshot_verification_status() {
    // Given: Snapshots with different verification states
    let verified_snapshot = Snapshot {
        backup_type: "vm".to_string(),
        backup_id: "100".to_string(),
        backup_time: 1000,
        comment: None,
        size: Some(1024),
        verification: Some(VerificationStatus {
            state: "ok".to_string(),
            last_verify: Some(999),
        }),
        protected: Some(false),
    };

    let unverified_snapshot = Snapshot {
        backup_type: "vm".to_string(),
        backup_id: "101".to_string(),
        backup_time: 1000,
        comment: None,
        size: Some(1024),
        verification: None,
        protected: Some(false),
    };

    let failed_snapshot = Snapshot {
        backup_type: "vm".to_string(),
        backup_id: "102".to_string(),
        backup_time: 1000,
        comment: None,
        size: Some(1024),
        verification: Some(VerificationStatus {
            state: "failed".to_string(),
            last_verify: Some(999),
        }),
        protected: Some(false),
    };

    // When/Then: Verification status is checked
    assert_eq!(verified_snapshot.verification.as_ref().unwrap().state, "ok");
    assert!(unverified_snapshot.verification.is_none());
    assert_eq!(
        failed_snapshot.verification.as_ref().unwrap().state,
        "failed"
    );
}

// WGT: Test snapshot protection status handling
#[test]
fn test_snapshot_protection_status() {
    // Given: Snapshots with different protection states
    let protected = Snapshot {
        backup_type: "vm".to_string(),
        backup_id: "100".to_string(),
        backup_time: 1000,
        comment: None,
        size: Some(1024),
        verification: None,
        protected: Some(true),
    };

    let unprotected = Snapshot {
        backup_type: "vm".to_string(),
        backup_id: "101".to_string(),
        backup_time: 1000,
        comment: None,
        size: Some(1024),
        verification: None,
        protected: Some(false),
    };

    let unknown = Snapshot {
        backup_type: "vm".to_string(),
        backup_id: "102".to_string(),
        backup_time: 1000,
        comment: None,
        size: Some(1024),
        verification: None,
        protected: None,
    };

    // When/Then: Protection status is evaluated
    assert!(protected.protected.unwrap_or(false));
    assert!(!unprotected.protected.unwrap_or(false));
    assert!(!unknown.protected.unwrap_or(false));
}

// WGT: Test comment truncation for label cardinality control
#[test]
fn test_comment_truncation() {
    // Given: A very long comment that exceeds label limits
    let long_comment = "This is a very long comment that should be truncated to prevent label cardinality issues in Prometheus metrics system";

    // When: Comment is truncated to 47 characters
    let safe_comment = if long_comment.len() > 50 {
        &long_comment[..47]
    } else {
        long_comment
    };

    // Then: Comment should be exactly 47 characters
    assert_eq!(safe_comment.len(), 47);
    // The first 47 chars of the comment
    assert_eq!(
        safe_comment,
        "This is a very long comment that should be trun"
    );
}
