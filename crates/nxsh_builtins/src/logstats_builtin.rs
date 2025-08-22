//! `logstats` builtin (stubbed) - logging feature disabled or trimmed build.
//!
//! This stub avoids pulling in the heavy structured logging stack when the
//! `logging` feature of nxsh_core is not enabled. It provides a graceful
//! message instead of failing to compile.

use anyhow::Result;

#[allow(dead_code)]
pub fn set_logging_system<T>(_logging: T) {}

pub fn logstats_cli(args: &[String]) -> Result<()> {
    let mut mode = OutputMode::Plain;
    for a in args.iter().skip(1) { // args[0] はコマンド名想宁E        match a.as_str() {
            "--json" => mode = OutputMode::JsonCompact,
            "--pretty" => mode = OutputMode::JsonPretty,
            "--prom" | "--prometheus" => {
                // In stub mode, emit minimal Prometheus exposition stating disabled
                println!("# HELP nxsh_log_available Logging subsystem availability");
                println!("# TYPE nxsh_log_available gauge");
                println!("nxsh_log_available 0");
                return Ok(());
            }
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            _ => {}
        }
    }

    match mode {
        OutputMode::Plain => {
            println!("Logging system not available in this build (feature 'logging' disabled).");
        }
        OutputMode::JsonCompact => {
            println!("{{\"error\":\"logging disabled\",\"available\":false}}");
        }
        OutputMode::JsonPretty => {
            println!(
                "{{\n  \"error\": \"logging disabled\",\n  \"available\": false\n}}"
            );
        }
    }

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum OutputMode { Plain, JsonCompact, JsonPretty }

fn print_help() {
    println!(
        "Usage: logstats [OPTIONS]\n\n\
         Display logging subsystem statistics (unavailable in this build).\n\n\
         Options:\n\
            --json      Output placeholder as compact JSON\n\
            --pretty    Output placeholder as pretty-printed JSON\n\
            --prom, --prometheus  Output minimal Prometheus metrics (availability only)\n\
           -h, --help  Show this help and exit"
    );
}

