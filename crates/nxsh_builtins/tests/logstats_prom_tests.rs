use nxsh_builtins::logstats_cli;

#[test]
fn logstats_prometheus_format_contains_headers_and_metrics() {
    // Run in Prometheus mode
    let args = vec!["logstats".to_string(), "--prom".to_string()];
    let out = logstats_cli(&args).expect("logstats prom should succeed");
    let s = String::from_utf8_lossy(&out.stdout);

    // Basic Prometheus exposition format checks
    assert!(s.contains("# HELP"), "missing HELP header");
    assert!(s.contains("# TYPE"), "missing TYPE header");

    // At least one metric line with nxsh_ prefix should appear
    assert!(s.contains("nxsh_"), "should contain nxsh_* metrics");
}


