//! logstats (full) — requires core logging available
use anyhow::Result;
use std::collections::BTreeMap;
#[cfg(feature = "logging")]
use nxsh_core::logging::LoggingSystem;
use std::{fs, path::PathBuf};
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, PartialEq, Eq)]
enum OutputMode { Plain, JsonCompact, JsonPretty, Prometheus }

pub fn logstats_cli(args: &[String]) -> Result<()> {
    let mut mode = OutputMode::Plain;
    for a in args.iter().skip(1) {
        match a.as_str() {
            "--json" => mode = OutputMode::JsonCompact,
            "--pretty" => mode = OutputMode::JsonPretty,
            "--prom" | "--prometheus" => mode = OutputMode::Prometheus,
            "-h" | "--help" => { print_help(); return Ok(()); }
            _ => {}
        }
    }

    // Collect metrics from core logging subsystem
    let mut map: BTreeMap<String, u64> = BTreeMap::new();
    #[cfg(feature = "logging")]
    let sys = LoggingSystem::new(Default::default())?;
    #[cfg(not(feature = "logging"))]
    {
        // Without core logging, emit empty/default metrics
        match mode {
            OutputMode::Plain => println!("logging feature disabled"),
            OutputMode::JsonCompact => println!("{}", serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())),
            OutputMode::JsonPretty => println!("{}", serde_json::to_string_pretty(&map).unwrap_or_else(|_| "{}".to_string())),
        }
        return Ok(());
    }
    for (k, v) in sys.get_metrics() { map.insert(k, v); }
    let summary = sys.get_statistics();
    map.insert("current_file_size".to_string(), summary.current_file_size);
    map.insert("rotations_performed".to_string(), summary.rotations_performed);
    map.insert("write_errors".to_string(), summary.write_errors);
    map.insert("total_bytes_logged".to_string(), summary.total_bytes_logged);
    map.insert("messages_logged".to_string(), summary.messages_logged);
    map.insert("errors_logged".to_string(), summary.errors_logged);
    map.insert("warnings_logged".to_string(), summary.warnings_logged);
    map.insert("info_logged".to_string(), summary.info_logged);
    map.insert("debug_logged".to_string(), summary.debug_logged);
    map.insert("trace_logged".to_string(), summary.trace_logged);

    // Derive rates using on-disk snapshot (survives separate processes)
    let snapshot_path = default_snapshot_path(&sys);
    if let Ok((rates, new_snap)) = compute_rates(&map, snapshot_path.clone()) {
        // Insert derived metrics
        if let Some(v) = rates.rotations_per_sec { map.insert("rotations_per_sec".to_string(), v); }
        if let Some(v) = rates.write_errors_per_sec { map.insert("write_errors_per_sec".to_string(), v); }
        if let Some(v) = rates.bytes_per_sec { map.insert("bytes_per_sec".to_string(), v); }
        if let Some(v) = rates.messages_per_sec { map.insert("messages_per_sec".to_string(), v); }
        // Persist updated snapshot (best-effort)
        let _ = persist_snapshot(snapshot_path, &new_snap);
    }

    match mode {
        OutputMode::Plain => { for (k, v) in map { println!("{k}: {v}"); } }
        OutputMode::JsonCompact => { println!("{}", serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())); }
        OutputMode::JsonPretty => { println!("{}", serde_json::to_string_pretty(&map).unwrap_or_else(|_| "{}".to_string())); }
        OutputMode::Prometheus => { render_prometheus(&map); }
    }

    Ok(())
}

fn print_help() {
    println!(
        "Usage: logstats [OPTIONS]\n\n\
         Display logging subsystem statistics and derived rates.\n\n\
         Options:\n\
           --json      Output metrics as compact JSON object\n\
           --pretty    Output metrics as pretty-printed JSON\n\
           --prom, --prometheus  Output metrics in Prometheus text exposition format\n\
           -h, --help  Show this help and exit\n\n\
         Notes:\n\
           Rates (rotations_per_sec, write_errors_per_sec, bytes_per_sec, messages_per_sec)\n\
           are computed using a persisted snapshot file. The snapshot path can be overridden\n\
           by setting environment variable NXSH_LOGSTATS_SNAPSHOT_PATH."
    );
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StatsSnapshot {
    epoch_secs: u64,
    rotations_performed: u64,
    write_errors: u64,
    total_bytes_logged: u64,
    messages_logged: u64,
}

#[derive(Debug, Default)]
struct RateMetrics {
    rotations_per_sec: Option<u64>,
    write_errors_per_sec: Option<u64>,
    bytes_per_sec: Option<u64>,
    messages_per_sec: Option<u64>,
}

fn default_snapshot_path(sys: &LoggingSystem) -> PathBuf {
    // Allow overriding snapshot location for testing or customization
    if let Ok(p) = std::env::var("NXSH_LOGSTATS_SNAPSHOT_PATH") {
        return PathBuf::from(p);
    }
    // Reuse log_dir for snapshot placement
    let log_dir = sys.get_config().log_dir.clone();
    log_dir.join("logstats_snapshot.json")
}

fn compute_rates(current: &BTreeMap<String, u64>, snap_path: PathBuf) -> Result<(RateMetrics, StatsSnapshot)> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let cur = StatsSnapshot {
        epoch_secs: now,
        rotations_performed: *current.get("rotations_performed").unwrap_or(&0),
        write_errors: *current.get("write_errors").unwrap_or(&0),
        total_bytes_logged: *current.get("total_bytes_logged").unwrap_or(&0),
        messages_logged: *current.get("messages_logged").unwrap_or(&0),
    };

    let mut rates = RateMetrics::default();
    if let Ok(prev_bytes) = fs::read(&snap_path) {
        if let Ok(prev) = serde_json::from_slice::<StatsSnapshot>(&prev_bytes) {
            let dt = cur.epoch_secs.saturating_sub(prev.epoch_secs).max(1);
            rates.rotations_per_sec = Some((cur.rotations_performed.saturating_sub(prev.rotations_performed)) / dt);
            rates.write_errors_per_sec = Some((cur.write_errors.saturating_sub(prev.write_errors)) / dt);
            rates.bytes_per_sec = Some((cur.total_bytes_logged.saturating_sub(prev.total_bytes_logged)) / dt);
            rates.messages_per_sec = Some((cur.messages_logged.saturating_sub(prev.messages_logged)) / dt);
        }
    }

    Ok((rates, cur))
}

fn persist_snapshot(path: PathBuf, snap: &StatsSnapshot) -> Result<()> {
    if let Some(dir) = path.parent() { let _ = fs::create_dir_all(dir); }
    let bytes = serde_json::to_vec_pretty(snap)?;
    fs::write(path, bytes)?;
    Ok(())
}

/// Render Prometheus text exposition format for the collected map
fn render_prometheus(map: &BTreeMap<String, u64>) {
    // Prefix all metrics with nxsh_log_
    for (k, v) in map.iter() {
        let metric = format!("nxsh_log_{}", sanitize_metric_name(k));
        if let Some((mtype, help)) = metric_meta(k) {
            println!("# HELP {} {}", metric, help);
            println!("# TYPE {} {}", metric, mtype);
        } else {
            // Default to gauge if unknown
            println!("# TYPE {} gauge", metric);
        }
        println!("{} {}", metric, v);
    }
}

fn sanitize_metric_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':' => out.push(ch),
            _ => out.push('_'),
        }
    }
    // Ensure name does not start with a digit
    if out.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        format!("m_{}", out)
    } else {
        out
    }
}

fn metric_meta(key: &str) -> Option<(&'static str, &'static str)> {
    match key {
        "current_file_size" => Some(("gauge", "Current size of active log file in bytes")),
        "rotations_performed" => Some(("counter", "Total number of log rotations performed")),
        "write_errors" => Some(("counter", "Total number of logging write errors")),
        "total_bytes_logged" => Some(("counter", "Total bytes written to logs")),
        "messages_logged" => Some(("counter", "Total messages logged")),
        "errors_logged" => Some(("counter", "Total error-level messages logged")),
        "warnings_logged" => Some(("counter", "Total warning-level messages logged")),
        "info_logged" => Some(("counter", "Total info-level messages logged")),
        "debug_logged" => Some(("counter", "Total debug-level messages logged")),
        "trace_logged" => Some(("counter", "Total trace-level messages logged")),
        "rotations_per_sec" => Some(("gauge", "Estimated log rotations per second")),
        "write_errors_per_sec" => Some(("gauge", "Estimated logging write errors per second")),
        "bytes_per_sec" => Some(("gauge", "Estimated logging throughput in bytes per second")),
        "messages_per_sec" => Some(("gauge", "Estimated messages per second")),
        _ => None,
    }
}

// Expose a programmatic collection path for tests and internal callers
#[allow(dead_code)]
pub fn collect_logstats_map_for_tests() -> BTreeMap<String, u64> {
    let mut map: BTreeMap<String, u64> = BTreeMap::new();
    #[cfg(feature = "logging")]
    let sys = match LoggingSystem::new(Default::default()) {
        Ok(s) => s,
        Err(_) => return map,
    };
    #[cfg(not(feature = "logging"))]
    {
        return map;
    }
    #[cfg(feature = "logging")]
    {
        for (k, v) in sys.get_metrics() { map.insert(k, v); }
        let summary = sys.get_statistics();
        map.insert("current_file_size".to_string(), summary.current_file_size);
        map.insert("rotations_performed".to_string(), summary.rotations_performed);
        map.insert("write_errors".to_string(), summary.write_errors);
        map.insert("total_bytes_logged".to_string(), summary.total_bytes_logged);
        map.insert("messages_logged".to_string(), summary.messages_logged);
        map.insert("errors_logged".to_string(), summary.errors_logged);
        map.insert("warnings_logged".to_string(), summary.warnings_logged);
        map.insert("info_logged".to_string(), summary.info_logged);
        map.insert("debug_logged".to_string(), summary.debug_logged);

        let snapshot_path = default_snapshot_path(&sys);
        if let Ok((rates, _new_snap)) = compute_rates(&map, snapshot_path.clone()) {
            if let Some(v) = rates.rotations_per_sec { map.insert("rotations_per_sec".to_string(), v); }
            if let Some(v) = rates.write_errors_per_sec { map.insert("write_errors_per_sec".to_string(), v); }
            if let Some(v) = rates.bytes_per_sec { map.insert("bytes_per_sec".to_string(), v); }
            if let Some(v) = rates.messages_per_sec { map.insert("messages_per_sec".to_string(), v); }
        }
    }
    map
}


