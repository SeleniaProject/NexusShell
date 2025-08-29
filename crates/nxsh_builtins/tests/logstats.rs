use nxsh_builtins::logstats_builtin::logstats_cli;
use std::{fs, thread, time::Duration};

// Basic CLI behavior tests for logstats
#[test]
fn logstats_help_works() {
    // Directly invoke builtin; ensure it completes successfully
    let args = vec!["logstats".to_string(), "--help".to_string()];
    let res = logstats_cli(&args);
    assert!(res.is_ok());
}

// Rate computation smoke test using persisted snapshot
#[test]
#[ignore] // TODO: Enable when logging feature is available
fn logstats_rates_snapshot_smoke() {
    // Use a temp snapshot file to avoid cross-test interference
    let dir = tempfile::tempdir().expect("tempdir");
    let snap = dir.path().join("snap.json");
    std::env::set_var("NXSH_LOGSTATS_SNAPSHOT_PATH", &snap);

    // First run: create snapshot via builtin
    let _ = logstats_cli(&["logstats".into(), "--json".into()]);

    // Ensure snapshot exists
    assert!(snap.exists(), "snapshot should be created");

    // Wait a bit to get non-zero dt
    thread::sleep(Duration::from_millis(1100));

    // Second run: compute rates
    let _ = logstats_cli(&["logstats".into(), "--json".into()]);
    // Snapshot file should still exist and be non-empty
    let bytes = fs::read(&snap).expect("read snapshot");
    assert!(!bytes.is_empty());

    // Cleanup
    std::env::remove_var("NXSH_LOGSTATS_SNAPSHOT_PATH");
    let _ = fs::remove_file(&snap);
}
