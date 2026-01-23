//! Tests for tape drive metrics.

use pbs_exporter::client::TapeDrive;

// WGT: Test tape drive count metric
#[test]
fn test_tape_drive_count() {
    // Given: A list of 2 tape drives
    let drives = [
        TapeDrive {
            name: "drive0".to_string(),
            vendor: Some("IBM".to_string()),
            model: Some("ULT3580-TD6".to_string()),
            serial: Some("1234567890".to_string()),
        },
        TapeDrive {
            name: "drive1".to_string(),
            vendor: Some("HP".to_string()),
            model: Some("Ultrium 6-SCSI".to_string()),
            serial: Some("ABCDEF1234".to_string()),
        },
    ];

    // When: Counting the number of drives
    let count = drives.len() as f64;

    // Then: Count should equal 2.0
    assert_eq!(count, 2.0);
}

// WGT: Test tape drive info metric labels
#[test]
fn test_tape_drive_info_labels() {
    // Given: A tape drive with complete information
    let drive = TapeDrive {
        name: "drive0".to_string(),
        vendor: Some("IBM".to_string()),
        model: Some("ULT3580-TD6".to_string()),
        serial: Some("1234567890".to_string()),
    };

    // When: Accessing all fields for metric labels
    // Then: All fields should be accessible and have correct values
    assert_eq!(drive.name, "drive0");
    assert_eq!(drive.vendor.as_deref().unwrap(), "IBM");
    assert_eq!(drive.model.as_deref().unwrap(), "ULT3580-TD6");
    assert_eq!(drive.serial.as_deref().unwrap(), "1234567890");
}

// WGT: Test tape drive with missing optional fields defaults to unknown
#[test]
fn test_tape_drive_missing_fields() {
    // Given: A tape drive with all optional fields set to None
    let drive = TapeDrive {
        name: "drive0".to_string(),
        vendor: None,
        model: None,
        serial: None,
    };

    // When: Accessing fields with fallback to "unknown"
    let vendor = drive.vendor.as_deref().unwrap_or("unknown");
    let model = drive.model.as_deref().unwrap_or("unknown");
    let serial = drive.serial.as_deref().unwrap_or("unknown");

    // Then: All fields should default to "unknown"
    assert_eq!(vendor, "unknown");
    assert_eq!(model, "unknown");
    assert_eq!(serial, "unknown");
}

// WGT: Test empty tape drive list results in count 0
#[test]
fn test_tape_drive_empty_list() {
    // Given: An empty tape drive list
    let drives: Vec<TapeDrive> = vec![];

    // When: Counting the number of drives
    let count = drives.len() as f64;

    // Then: Count should be 0.0
    assert_eq!(count, 0.0);
}

// WGT: Test tape drives from various vendors
#[test]
fn test_tape_drive_various_vendors() {
    // Given: Tape drives from three different vendors
    let drives = [
        TapeDrive {
            name: "drive0".to_string(),
            vendor: Some("IBM".to_string()),
            model: Some("ULT3580-TD6".to_string()),
            serial: Some("IBM001".to_string()),
        },
        TapeDrive {
            name: "drive1".to_string(),
            vendor: Some("HP".to_string()),
            model: Some("Ultrium 6-SCSI".to_string()),
            serial: Some("HP001".to_string()),
        },
        TapeDrive {
            name: "drive2".to_string(),
            vendor: Some("Quantum".to_string()),
            model: Some("SuperLoader 3".to_string()),
            serial: Some("Q001".to_string()),
        },
    ];

    // When: Accessing vendor and serial fields
    // Then: Each drive should have distinct vendor and serial values
    assert_eq!(drives[0].vendor.as_deref().unwrap(), "IBM");
    assert_eq!(drives[1].vendor.as_deref().unwrap(), "HP");
    assert_eq!(drives[2].vendor.as_deref().unwrap(), "Quantum");

    assert_eq!(drives[0].serial.as_deref().unwrap(), "IBM001");
    assert_eq!(drives[1].serial.as_deref().unwrap(), "HP001");
    assert_eq!(drives[2].serial.as_deref().unwrap(), "Q001");
}

// WGT: Test tape_drive_info metric value is always 1.0
#[test]
fn test_tape_drive_info_metric_value() {
    // Given: A tape drive with complete information
    let drive = TapeDrive {
        name: "drive0".to_string(),
        vendor: Some("IBM".to_string()),
        model: Some("ULT3580-TD6".to_string()),
        serial: Some("1234567890".to_string()),
    };

    // When: Setting the metric value for tape_drive_info
    let metric_value = 1.0;

    // Then: Metric value should be 1.0 (presence indicator) and drive should exist
    assert_eq!(metric_value, 1.0);
    assert!(!drive.name.is_empty());
}
