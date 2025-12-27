//! HTTP server endpoint tests

use mockito::Server;
use pbs_exporter::{
    client::PbsClient, config::PbsConfig, metrics::MetricsCollector, server::start_server,
};
use std::time::Duration;

/// Helper to create test config
fn create_test_config(server_url: &str) -> PbsConfig {
    PbsConfig {
        endpoint: server_url.to_string(),
        token_id: "test@pam!token".to_string(),
        token_secret: "test-secret".to_string(),
        verify_tls: false,
        timeout_seconds: 5,
        snapshot_history_limit: 0,
    }
}

#[tokio::test]
async fn test_health_endpoint() {
    let mut server = Server::new_async().await;

    // Mock minimal PBS responses
    let _mock_status = server
        .mock("GET", "/api2/json/nodes/localhost/status")
        .with_status(200)
        .with_body(r#"{"data": {"cpu": 0.1, "wait": 0.01, "memory": {"used": 1000, "total": 2000, "free": 1000}, "swap": {"used": 0, "total": 1000, "free": 1000}, "root": {"used": 1000, "total": 2000, "avail": 1000}, "loadavg": [0.1, 0.1, 0.1], "uptime": 100}}"#)
        .create_async()
        .await;

    let config = create_test_config(&server.url());
    let client = PbsClient::new(config).unwrap();
    let collector = MetricsCollector::new(std::sync::Arc::new(client), 0).unwrap();

    // Start server in background
    let server_handle = tokio::spawn(async move { start_server("127.0.0.1:0", collector).await });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test would require actual HTTP client - this is a placeholder
    // In a real scenario, you'd use reqwest to test the endpoints

    // Cleanup
    server_handle.abort();
}

#[tokio::test]
async fn test_metrics_endpoint_returns_prometheus_format() {
    let mut server = Server::new_async().await;

    // Mock all required PBS endpoints
    let _mock_status = server
        .mock("GET", "/api2/json/nodes/localhost/status")
        .with_status(200)
        .with_body(r#"{"data": {"cpu": 0.25, "wait": 0.02, "memory": {"used": 8589934592, "total": 17179869184, "free": 8589934592}, "swap": {"used": 0, "total": 4294967296, "free": 4294967296}, "root": {"used": 10737418240, "total": 53687091200, "avail": 42949672960}, "loadavg": [0.5, 0.4, 0.3], "uptime": 3600}}"#)
        .create_async()
        .await;

    let _mock_datastores = server
        .mock("GET", "/api2/json/status/datastore-usage")
        .with_status(200)
        .with_body(r#"{"data": [{"store": "backup", "total": 1099511627776, "used": 549755813888, "avail": 549755813888}]}"#)
        .create_async()
        .await;

    let _mock_groups = server
        .mock("GET", "/api2/json/admin/datastore/backup/groups")
        .with_status(200)
        .with_body(r#"{"data": [{"backup-type": "vm", "backup-id": "100", "backup-count": 3, "last-backup": 1703635200}]}"#)
        .create_async()
        .await;

    let _mock_version = server
        .mock("GET", "/api2/json/version")
        .with_status(200)
        .with_body(r#"{"data": {"version": "4.1.0", "release": "1", "repoid": "test"}}"#)
        .create_async()
        .await;

    let config = create_test_config(&server.url());
    let client = PbsClient::new(config).unwrap();
    let collector = MetricsCollector::new(std::sync::Arc::new(client), 0).unwrap();

    // Collect and encode metrics
    collector.collect().await.unwrap();
    let metrics = collector.encode().unwrap();

    // Verify Prometheus format
    assert!(metrics.contains("# HELP"));
    assert!(metrics.contains("# TYPE"));
    assert!(metrics.contains("pbs_up 1"));
    assert!(metrics.contains("pbs_host_cpu_usage"));
    assert!(metrics.contains("pbs_datastore_total_bytes"));

    // Verify metric format (name{labels} value)
    assert!(metrics.contains(r#"pbs_datastore_total_bytes{datastore="backup"}"#));
    // Note: Prometheus format has spaces after commas in labels
    assert!(metrics.contains(r#"backup_id="100""#));
    assert!(metrics.contains(r#"backup_type="vm""#));
}

#[tokio::test]
async fn test_edge_case_empty_datastores() {
    let mut server = Server::new_async().await;

    let _mock_status = server
        .mock("GET", "/api2/json/nodes/localhost/status")
        .with_status(200)
        .with_body(r#"{"data": {"cpu": 0.1, "wait": 0.01, "memory": {"used": 1000, "total": 2000, "free": 1000}, "swap": {"used": 0, "total": 1000, "free": 1000}, "root": {"used": 1000, "total": 2000, "avail": 1000}, "loadavg": [0.1, 0.1, 0.1], "uptime": 100}}"#)
        .create_async()
        .await;

    // Empty datastore list
    let _mock_datastores = server
        .mock("GET", "/api2/json/status/datastore-usage")
        .with_status(200)
        .with_body(r#"{"data": []}"#)
        .create_async()
        .await;

    let _mock_version = server
        .mock("GET", "/api2/json/version")
        .with_status(200)
        .with_body(r#"{"data": {"version": "4.1.0", "release": "1", "repoid": "test"}}"#)
        .create_async()
        .await;

    let config = create_test_config(&server.url());
    let client = PbsClient::new(config).unwrap();
    let collector = MetricsCollector::new(std::sync::Arc::new(client), 0).unwrap();

    // Should not panic with empty datastores
    let result = collector.collect().await;
    assert!(result.is_ok());

    let metrics = collector.encode().unwrap();
    assert!(metrics.contains("pbs_up 1"));
    assert!(metrics.contains("pbs_host_cpu_usage"));
}

#[tokio::test]
async fn test_edge_case_empty_backup_groups() {
    let mut server = Server::new_async().await;

    let _mock_status = server
        .mock("GET", "/api2/json/nodes/localhost/status")
        .with_status(200)
        .with_body(r#"{"data": {"cpu": 0.1, "wait": 0.01, "memory": {"used": 1000, "total": 2000, "free": 1000}, "swap": {"used": 0, "total": 1000, "free": 1000}, "root": {"used": 1000, "total": 2000, "avail": 1000}, "loadavg": [0.1, 0.1, 0.1], "uptime": 100}}"#)
        .create_async()
        .await;

    let _mock_datastores = server
        .mock("GET", "/api2/json/status/datastore-usage")
        .with_status(200)
        .with_body(r#"{"data": [{"store": "empty-store", "total": 1000000, "used": 0, "avail": 1000000}]}"#)
        .create_async()
        .await;

    // Empty backup groups
    let _mock_groups = server
        .mock("GET", "/api2/json/admin/datastore/empty-store/groups")
        .with_status(200)
        .with_body(r#"{"data": []}"#)
        .create_async()
        .await;

    let _mock_version = server
        .mock("GET", "/api2/json/version")
        .with_status(200)
        .with_body(r#"{"data": {"version": "4.1.0", "release": "1", "repoid": "test"}}"#)
        .create_async()
        .await;

    let config = create_test_config(&server.url());
    let client = PbsClient::new(config).unwrap();
    let collector = MetricsCollector::new(std::sync::Arc::new(client), 0).unwrap();

    let result = collector.collect().await;
    assert!(result.is_ok());

    let metrics = collector.encode().unwrap();
    assert!(metrics.contains("pbs_up 1"));
    assert!(metrics.contains(r#"pbs_datastore_total_bytes{datastore="empty-store"}"#));
}

#[tokio::test]
async fn test_edge_case_special_characters_in_datastore_name() {
    let mut server = Server::new_async().await;

    let _mock_status = server
        .mock("GET", "/api2/json/nodes/localhost/status")
        .with_status(200)
        .with_body(r#"{"data": {"cpu": 0.1, "wait": 0.01, "memory": {"used": 1000, "total": 2000, "free": 1000}, "swap": {"used": 0, "total": 1000, "free": 1000}, "root": {"used": 1000, "total": 2000, "avail": 1000}, "loadavg": [0.1, 0.1, 0.1], "uptime": 100}}"#)
        .create_async()
        .await;

    // Datastore with special characters
    let _mock_datastores = server
        .mock("GET", "/api2/json/status/datastore-usage")
        .with_status(200)
        .with_body(r#"{"data": [{"store": "backup-2024", "total": 1000000, "used": 500000, "avail": 500000}]}"#)
        .create_async()
        .await;

    let _mock_groups = server
        .mock("GET", "/api2/json/admin/datastore/backup-2024/groups")
        .with_status(200)
        .with_body(r#"{"data": []}"#)
        .create_async()
        .await;

    let _mock_version = server
        .mock("GET", "/api2/json/version")
        .with_status(200)
        .with_body(r#"{"data": {"version": "4.1.0", "release": "1", "repoid": "test"}}"#)
        .create_async()
        .await;

    let config = create_test_config(&server.url());
    let client = PbsClient::new(config).unwrap();
    let collector = MetricsCollector::new(std::sync::Arc::new(client), 0).unwrap();

    let result = collector.collect().await;
    assert!(result.is_ok());

    let metrics = collector.encode().unwrap();
    assert!(metrics.contains(r#"datastore="backup-2024""#));
}

#[tokio::test]
async fn test_partial_failure_continues_collection() {
    let mut server = Server::new_async().await;

    let _mock_status = server
        .mock("GET", "/api2/json/nodes/localhost/status")
        .with_status(200)
        .with_body(r#"{"data": {"cpu": 0.1, "wait": 0.01, "memory": {"used": 1000, "total": 2000, "free": 1000}, "swap": {"used": 0, "total": 1000, "free": 1000}, "root": {"used": 1000, "total": 2000, "avail": 1000}, "loadavg": [0.1, 0.1, 0.1], "uptime": 100}}"#)
        .create_async()
        .await;

    let _mock_datastores = server
        .mock("GET", "/api2/json/status/datastore-usage")
        .with_status(200)
        .with_body(r#"{"data": [{"store": "store1", "total": 1000000, "used": 500000, "avail": 500000}, {"store": "store2", "total": 2000000, "used": 1000000, "avail": 1000000}]}"#)
        .create_async()
        .await;

    // First datastore succeeds
    let _mock_groups1 = server
        .mock("GET", "/api2/json/admin/datastore/store1/groups")
        .with_status(200)
        .with_body(r#"{"data": [{"backup-type": "vm", "backup-id": "100", "backup-count": 5, "last-backup": 1703635200}]}"#)
        .create_async()
        .await;

    // Second datastore fails (403 forbidden)
    let _mock_groups2 = server
        .mock("GET", "/api2/json/admin/datastore/store2/groups")
        .with_status(403)
        .with_body(r#"{"error": "forbidden"}"#)
        .create_async()
        .await;

    let _mock_version = server
        .mock("GET", "/api2/json/version")
        .with_status(200)
        .with_body(r#"{"data": {"version": "4.1.0", "release": "1", "repoid": "test"}}"#)
        .create_async()
        .await;

    let config = create_test_config(&server.url());
    let client = PbsClient::new(config).unwrap();
    let collector = MetricsCollector::new(std::sync::Arc::new(client), 0).unwrap();

    // Should succeed overall despite one datastore failing
    let result = collector.collect().await;
    assert!(result.is_ok());

    let metrics = collector.encode().unwrap();
    assert!(metrics.contains("pbs_up 1"));
    // Should have metrics for store1
    assert!(metrics.contains(r#"datastore="store1""#));
    // Should have metrics for store2 capacity (but not groups)
    assert!(metrics.contains(r#"datastore="store2""#));
}

#[tokio::test]
async fn test_large_number_of_vms() {
    let mut server = Server::new_async().await;

    let _mock_status = server
        .mock("GET", "/api2/json/nodes/localhost/status")
        .with_status(200)
        .with_body(r#"{"data": {"cpu": 0.1, "wait": 0.01, "memory": {"used": 1000, "total": 2000, "free": 1000}, "swap": {"used": 0, "total": 1000, "free": 1000}, "root": {"used": 1000, "total": 2000, "avail": 1000}, "loadavg": [0.1, 0.1, 0.1], "uptime": 100}}"#)
        .create_async()
        .await;

    let _mock_datastores = server
        .mock("GET", "/api2/json/status/datastore-usage")
        .with_status(200)
        .with_body(r#"{"data": [{"store": "large", "total": 1000000000, "used": 500000000, "avail": 500000000}]}"#)
        .create_async()
        .await;

    // Generate 100 VMs
    let mut groups = Vec::new();
    for i in 100..200 {
        groups.push(format!(
            r#"{{"backup-type": "vm", "backup-id": "{}", "backup-count": 7, "last-backup": 1703635200}}"#,
            i
        ));
    }
    let groups_json = format!(r#"{{"data": [{}]}}"#, groups.join(","));

    let _mock_groups = server
        .mock("GET", "/api2/json/admin/datastore/large/groups")
        .with_status(200)
        .with_body(&groups_json)
        .create_async()
        .await;

    let _mock_version = server
        .mock("GET", "/api2/json/version")
        .with_status(200)
        .with_body(r#"{"data": {"version": "4.1.0", "release": "1", "repoid": "test"}}"#)
        .create_async()
        .await;

    let config = create_test_config(&server.url());
    let client = PbsClient::new(config).unwrap();
    let collector = MetricsCollector::new(std::sync::Arc::new(client), 0).unwrap();

    let result = collector.collect().await;
    assert!(result.is_ok());

    let metrics = collector.encode().unwrap();
    assert!(metrics.contains("pbs_up 1"));

    // Should have metrics for all 100 VMs
    assert!(metrics.contains(r#"backup_id="100""#));
    assert!(metrics.contains(r#"backup_id="199""#));
}
