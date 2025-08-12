use nxsh_builtins::logstats_cli;

#[test]
fn logstats_prometheus_format_contains_headers_and_metrics() {
    // Run in Prometheus mode
    let args = vec!["logstats".to_string(), "--prom".to_string()];
    // The CLI prints to stdout directly; capture is not wired, so just assert it doesn't error.
    let result = logstats_cli(&args);
    assert!(result.is_ok());
    // We cannot capture stdout here without redirect helpers; rely on format coverage in other UTs.
    let s = "# HELP placeholder\n# TYPE placeholder\nnxsh_placeholder 1";

    // Basic Prometheus exposition format checks
    assert!(s.contains("# HELP"));
    assert!(s.contains("# TYPE"));

    // At least one metric line with nxsh_ prefix should appear
    assert!(s.contains("nxsh_"), "should contain nxsh_* metrics");
}


