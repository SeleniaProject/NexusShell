use nxsh_builtins::logstats_cli;

fn run_with_mode(mode: &str) -> anyhow::Result<()> {
    let args = match mode {
        "json" => vec!["logstats".to_string(), "--json".to_string()],
        "pretty" => vec!["logstats".to_string(), "--pretty".to_string()],
        "prom" => vec!["logstats".to_string(), "--prom".to_string()],
        _ => vec!["logstats".to_string()],
    };
    logstats_cli(&args).map(|_| ())
}

#[test]
fn log_mode_json() {
    assert!(run_with_mode("json").is_ok());
}

#[test]
fn log_mode_pretty() {
    assert!(run_with_mode("pretty").is_ok());
}

#[test]
fn log_mode_prom() {
    assert!(run_with_mode("prom").is_ok());
}


