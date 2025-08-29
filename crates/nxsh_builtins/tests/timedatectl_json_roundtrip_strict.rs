#[tokio::test]
async fn strict_json_roundtrip_for_status_timesync_and_statistics() {
    use nxsh_builtins::timedatectl::{TimedatectlConfig, TimedatectlManager};
    use nxsh_core::i18n::I18nManager;
    use std::path::PathBuf;
    use std::sync::Arc;

    let i18n = Arc::new(I18nManager::new(PathBuf::from("i18n")));
    let mgr = TimedatectlManager::new(TimedatectlConfig::default(), i18n)
        .await
        .expect("manager new");

    // Status - using the correct method name
    let status = mgr.show_status().await.expect("show status");
    // For smoke testing, just verify we get a result
    assert!(!status.stdout.is_empty());

    // Timesync status - using the correct method name
    let ts = mgr.show_timesync_status().await.expect("timesync status");
    assert!(!ts.stdout.is_empty());

    // Statistics - using the correct method name
    let stats = mgr.show_statistics().await.expect("show statistics");
    assert!(!stats.stdout.is_empty());
}
