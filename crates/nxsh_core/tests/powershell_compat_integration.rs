#![cfg(feature = "powershell_compat")]
// PowerShell compatibility integration tests
// These tests validate:
// - Environment-driven alias injection into ShellContext
// - Basic cmdlets and pipeline behavior (Get-Content | Measure-Object)
// - Get-Help on alias names

use std::{env, fs, io::Write};

use nxsh_core::{PowerShellCompat, PowerShellObject};
use nxsh_core::context::ShellContext;

// Serialize access to process-global environment variables across tests
static ENV_TEST_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();

#[test]
fn ps_alias_injection_enabled_by_env() {
    let _guard = ENV_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();

    // Ensure a clean slate
    env::remove_var("NXSH_DISABLE_PS_ALIASES");
    env::set_var("NXSH_ENABLE_PS_ALIASES", "1");

    let ctx = ShellContext::new();
    // Expect that common aliases are injected
    assert_eq!(ctx.get_alias("ls").as_deref(), Some("Get-ChildItem"));
    assert_eq!(ctx.get_alias("cd").as_deref(), Some("Set-Location"));
    assert_eq!(ctx.get_alias("echo").as_deref(), Some("Write-Output"));
}

#[test]
fn ps_alias_injection_respects_disable() {
    let _guard = ENV_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();

    env::set_var("NXSH_ENABLE_PS_ALIASES", "1");
    env::set_var("NXSH_DISABLE_PS_ALIASES", "1");

    let ctx = ShellContext::new();
    // When disabled, the alias map should not contain PS aliases injected by env
    assert!(ctx.get_alias("ls").is_none());
}

#[test]
fn pipeline_get_content_measure_object_counts_lines() {
    // Create a temporary file with known line count
    let mut tmp_path = std::env::temp_dir();
    tmp_path.push("nxsh_ps_pipeline_test.txt");
    let mut file = fs::File::create(&tmp_path).expect("create temp file");
    writeln!(file, "line1").unwrap();
    writeln!(file, "line2").unwrap();
    writeln!(file, "line3").unwrap();
    drop(file);

    let mut ps = PowerShellCompat::new();
    let pipeline = format!("Get-Content {} | Measure-Object", tmp_path.display());
    let objs = ps.execute_pipeline(&pipeline).expect("pipeline executes");

    assert_eq!(objs.len(), 1, "Measure-Object should return a single object");
    assert!(matches!(objs[0], PowerShellObject::Integer(3)), "Expected line count = 3");

    // Clean up
    let _ = fs::remove_file(&tmp_path);
}

#[test]
fn get_help_on_alias_reports_mapping() {
    let mut ps = PowerShellCompat::new();
    // Ensure default aliases include ls -> Get-ChildItem
    let res = ps.execute_command("Get-Help", vec!["ls".to_string()]).expect("exec Get-Help");
    assert!(res.output.contains("Alias: ls -> Get-ChildItem"));
}


