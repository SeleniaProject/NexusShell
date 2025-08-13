use nxsh_builtins::logstats;

#[test]
fn json_is_valid_object() {
    // Build a deterministic small map via internal helper
    let mut map = std::collections::BTreeMap::new();
    map.insert("messages_logged".to_string(), 1);
    map.insert("write_errors".to_string(), 0);
    let s = logstats::render_logstats_for_mode("json", &map);
    let v: serde_json::Value = serde_json::from_str(&s).expect("valid json");
    assert!(v.is_object());
}

#[test]
fn prom_lines_have_type_or_default() {
    let mut map = std::collections::BTreeMap::new();
    map.insert("messages_logged".to_string(), 1);
    map.insert("unknown_metric".to_string(), 2);
    let s = logstats::render_logstats_for_mode("prom", &map);
    // Each metric should have at least one TYPE line and one sample line
    assert!(s.contains("# TYPE nxsh_log_messages_logged"));
    assert!(s.contains("nxsh_log_messages_logged "));
    assert!(s.contains("# TYPE nxsh_log_unknown_metric"));
}


