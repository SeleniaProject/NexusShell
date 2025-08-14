use serde_json::Value;

#[tokio::test]
async fn strict_json_roundtrip_for_status_timesync_and_statistics() {
    use nxsh_builtins::timedatectl::{TimedatectlManager, TimedatectlConfig};
    let i18n = nxsh_builtins::common::i18n::I18n::new();
    let mgr = TimedatectlManager::new(TimedatectlConfig::default(), i18n)
        .await
        .expect("manager new");

    // Status
    let status = mgr.get_status().await;
    let status_json = serde_json::to_value(&status).expect("serialize status");
    assert!(status_json.get("timezone").is_some());
    assert!(status_json.get("system_clock_synchronized").is_some());

    // Timesync status
    let ts = mgr.get_timesync_status().await.expect("timesync status");
    let ts_json: Value = serde_json::to_value(&ts).expect("serialize timesync");
    assert!(ts_json.get("enabled").is_some());
    assert!(ts_json.get("servers").is_some());

    // Statistics
    let stats = mgr.get_statistics().await;
    let stats_json: Value = serde_json::to_value(&stats).expect("serialize stats");
    assert!(stats_json.get("total_adjustments").is_some());
}


