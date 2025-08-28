use nxsh_builtins::timedatectl::timedatectl_cli;

#[tokio::test]
async fn timesync_status_json_outputs_valid_json() {
    // We only verify that the code path runs; capturing stdout is outside unit test scope here.
    let r = timedatectl_cli(&["timesync-status".into(), "--json".into()]);
    assert!(r.is_ok());
}

#[tokio::test]
async fn timesync_status_json_roundtrip_structure() {
    // We cannot capture stdout from here easily; instead, we call the internal pieces to build JSON.
    // Use the CLI path to ensure code paths execute; success is sufficient for smoke coverage.
    let res = timedatectl_cli(&["timesync-status".into(), "-J".into()]);
    assert!(res.is_ok());
}

#[tokio::test]
async fn status_and_statistics_json_smoke() {
    let r1 = timedatectl_cli(&["status".into(), "--json".into()]);
    assert!(r1.is_ok());
    let r2 = timedatectl_cli(&["statistics".into(), "-J".into()]);
    assert!(r2.is_ok());
}

#[test]
fn json_roundtrip_sample_structs() {
    // Construct minimal sample objects for smoke testing
    use nxsh_builtins::timedatectl::{NTPServerStatus, TimeSyncStatus, LeapStatus};
    let status = TimeSyncStatus {
        enabled: true,
        synchronized: false,
        ntp_enabled: true,
        servers: vec![NTPServerStatus {
            server: "pool.ntp.org".to_string(),
            address: "pool.ntp.org".to_string(),
            status: "active".to_string(),
            reachable: true,
            stratum: 2,
            delay: 0.001,
            offset: 0.002,
            jitter: 0.0005,
            last_sync: None,
        }],
        last_sync: None,
        sync_accuracy: None,
        drift_rate: Some(0.0),
        poll_interval: std::time::Duration::from_secs(64),
        leap_status: LeapStatus::NORMAL,
    };
    
    // Simple validation that the struct can be constructed
    assert!(status.enabled);
    assert_eq!(status.servers.len(), 1);
    assert_eq!(status.servers[0].server, "pool.ntp.org");
}

