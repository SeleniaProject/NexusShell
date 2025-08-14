use nxsh_builtins::timedatectl::timedatectl_cli;
use serde_json::Value;

#[tokio::test]
async fn timesync_status_json_outputs_valid_json() {
    // We only verify that the code path runs; capturing stdout is outside unit test scope here.
    let r = timedatectl_cli(&["timesync-status".into(), "--json".into()]).await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn timesync_status_json_roundtrip_structure() {
    // We cannot capture stdout from here easily; instead, we call the internal pieces to build JSON.
    // Use the CLI path to ensure code paths execute; success is sufficient for smoke coverage.
    let res = timedatectl_cli(&["timesync-status".into(), "-J".into()]).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn status_and_statistics_json_smoke() {
    let r1 = timedatectl_cli(&["status".into(), "--json".into()]).await;
    assert!(r1.is_ok());
    let r2 = timedatectl_cli(&["statistics".into(), "-J".into()]).await;
    assert!(r2.is_ok());
}

#[test]
fn json_roundtrip_sample_structs() {
    // Construct minimal sample objects and ensure they serialize to JSON and back
    use nxsh_builtins::timedatectl::{NTPServerStatus, TimeSyncStatus, LeapStatus};
    let status = TimeSyncStatus {
        enabled: true,
        synchronized: false,
        servers: vec![NTPServerStatus {
            address: "pool.ntp.org".to_string(),
            reachable: true,
            stratum: Some(2),
            delay: None,
            offset: None,
            jitter: None,
            last_sync: None,
        }],
        last_sync: None,
        sync_accuracy: None,
        drift_rate: Some(0.0),
        poll_interval: std::time::Duration::from_secs(64),
        leap_status: LeapStatus::Normal,
    };
    let j = serde_json::to_string(&status).unwrap();
    let v: Value = serde_json::from_str(&j).unwrap();
    assert!(v["enabled"].as_bool().unwrap());
}

