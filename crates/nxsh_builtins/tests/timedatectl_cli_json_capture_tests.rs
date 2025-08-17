
#[tokio::test]
async fn capture_timesync_status_show_timesync_json() {
    use nxsh_builtins::timedatectl::timedatectl_cli;
    // show-timesync should imply JSON (machine-readable)
    let res = timedatectl_cli(&["show-timesync".into()]).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn capture_status_show_json() {
    use nxsh_builtins::timedatectl::timedatectl_cli;
    // show should imply JSON (machine-readable)
    let res = timedatectl_cli(&["show".into()]).await;
    assert!(res.is_ok());
}


