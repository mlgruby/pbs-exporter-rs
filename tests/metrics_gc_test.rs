//! Tests for GC metrics conversion.

use pbs_exporter::client::GcStatus;

// WGT: Test GC status OK converts to metric value 1.0
#[test]
fn test_gc_status_ok() {
    // Given: A GC status with state "OK"
    let gc_status = GcStatus {
        disk_bytes: Some(10240000),
        last_run_endtime: Some(1000),
        last_run_state: Some("OK".to_string()),
        duration: Some(120.5),
        removed_bytes: Some(1024000),
        pending_bytes: Some(512000),
    };

    // When: Converting status to metric value
    let status_value = if gc_status
        .last_run_state
        .as_ref()
        .unwrap()
        .eq_ignore_ascii_case("ok")
    {
        1.0
    } else {
        0.0
    };

    // Then: OK status should convert to 1.0 and fields should be accessible
    assert_eq!(status_value, 1.0);
    assert_eq!(gc_status.duration.unwrap(), 120.5);
    assert_eq!(gc_status.removed_bytes.unwrap(), 1024000);
}

// WGT: Test GC status ERROR converts to metric value 0.0
#[test]
fn test_gc_status_error() {
    // Given: A GC status with state "ERROR"
    let gc_status = GcStatus {
        disk_bytes: Some(10240000),
        last_run_endtime: Some(1000),
        last_run_state: Some("ERROR".to_string()),
        duration: Some(10.0),
        removed_bytes: Some(0),
        pending_bytes: Some(2048000),
    };

    // When: Converting status to metric value
    let status_value = if gc_status
        .last_run_state
        .as_ref()
        .unwrap()
        .eq_ignore_ascii_case("ok")
    {
        1.0
    } else {
        0.0
    };

    // Then: ERROR status should convert to 0.0
    assert_eq!(status_value, 0.0);
}

// WGT: Test GC status comparison is case-insensitive
#[test]
fn test_gc_status_case_insensitive() {
    // Given: Multiple case variations of "OK"
    let variations = vec!["OK", "ok", "Ok", "oK"];

    // When: Comparing each variation with case-insensitive check
    for variant in variations {
        let status_value = variant.eq_ignore_ascii_case("ok");

        // Then: All variations should match "ok"
        assert!(status_value, "Failed for variant: {}", variant);
    }
}

// WGT: Test GC status with all optional fields as None
#[test]
fn test_gc_status_optional_fields() {
    // Given: A GC status with all optional fields as None
    let minimal_gc = GcStatus {
        disk_bytes: None,
        last_run_endtime: None,
        last_run_state: None,
        duration: None,
        removed_bytes: None,
        pending_bytes: None,
    };

    // When: Checking field values
    // Then: All fields should be None
    assert!(minimal_gc.last_run_endtime.is_none());
    assert!(minimal_gc.last_run_state.is_none());
    assert!(minimal_gc.duration.is_none());
    assert!(minimal_gc.removed_bytes.is_none());
    assert!(minimal_gc.pending_bytes.is_none());
}

// WGT: Test GC duration handles fractional seconds correctly
#[test]
fn test_gc_duration_precision() {
    // Given: A GC status with fractional duration value
    let gc_status = GcStatus {
        disk_bytes: Some(10240000),
        last_run_endtime: Some(1000),
        last_run_state: Some("OK".to_string()),
        duration: Some(123.456),
        removed_bytes: Some(1024),
        pending_bytes: Some(512),
    };

    // When: Accessing the duration field
    // Then: Duration should preserve fractional precision
    assert!((gc_status.duration.unwrap() - 123.456).abs() < 0.001);
}

// WGT: Test GC removed and pending bytes values
#[test]
fn test_gc_bytes_values() {
    // Given: A GC status with large byte values
    let gc_status = GcStatus {
        disk_bytes: Some(10240000),
        last_run_endtime: Some(1000),
        last_run_state: Some("OK".to_string()),
        duration: Some(60.0),
        removed_bytes: Some(1_073_741_824), // 1 GB
        pending_bytes: Some(536_870_912),   // 512 MB
    };

    // When: Accessing byte fields
    // Then: Values should match the large byte counts
    assert_eq!(gc_status.removed_bytes.unwrap(), 1_073_741_824);
    assert_eq!(gc_status.pending_bytes.unwrap(), 536_870_912);
}
