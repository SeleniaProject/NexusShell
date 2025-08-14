//! Tests for command substitution and global timeout logic
use nxsh_core::{Executor, ShellContext};
use nxsh_parser::Parser;
use std::sync::Once;
use std::time::{Duration, Instant};
use std::sync::{OnceLock, Mutex};

static INIT: Once = Once::new();
// Global lock to serialize environment mutations across tests in this module
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn ensure_initialized() {
    INIT.call_once(|| {
        let _ = nxsh_core::initialize();
        let _ = nxsh_hal::initialize();
    });
}

fn make_exec_ctx() -> (Executor, ShellContext) {
    ensure_initialized();
    (Executor::new(), ShellContext::new())
}

#[test]
fn test_command_substitution_basic() {
    // Serialize environment mutations to avoid interference with other tests
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    // Ensure no stray tiny timeout from other tests
    std::env::remove_var("NXSH_TIMEOUT_MS");
    let (mut exec, mut ctx) = make_exec_ctx();
    ctx.clear_global_timeout();
    // Ensure context does not arm timeout from env
    ctx.ensure_global_timeout_from_env();
    let parser = Parser::new();
    // Simple: echo $(echo inner)
    let ast = parser.parse("echo $(echo inner)").expect("parse");
    let res = exec.execute(&ast, &mut ctx).expect("execute");
    // We only assert execution path; output may vary depending on builtin availability
    // Ensure it did not timeout and exit code captured
    assert_ne!(res.exit_code, 124, "Should not timeout");
    std::env::remove_var("NXSH_TIMEOUT_MS");
}

#[test]
fn test_command_substitution_nested() {
    // Serialize environment mutations to avoid interference with other tests
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    std::env::remove_var("NXSH_TIMEOUT_MS");
    let (mut exec, mut ctx) = make_exec_ctx();
    ctx.clear_global_timeout();
    ctx.ensure_global_timeout_from_env();
    let parser = Parser::new();
    let ast = parser.parse("echo $(echo $(echo nested))").expect("parse");
    let res = exec.execute(&ast, &mut ctx).expect("execute");
    assert_ne!(res.exit_code, 124);
    std::env::remove_var("NXSH_TIMEOUT_MS");
}

#[test]
fn test_global_timeout_triggers() {
    // Serialize this test to avoid cross-test env var interference
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

    // Set very small timeout via env var before creating context
    std::env::set_var("NXSH_TIMEOUT_MS", "1");
    let (mut exec, mut ctx) = make_exec_ctx();
    // Some environments set env var after component init; ensure context consumes it
    ctx.ensure_global_timeout_from_env();
    // Busy loop: use a sequence of many no-op words
    let parser = Parser::new();
    let mut script = String::new();
    for _ in 0..2000 { script.push_str("echo a\n"); }
    let ast = parser.parse(&script).expect("parse");
    let start = Instant::now();
    let res = exec.execute(&ast, &mut ctx).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(res.exit_code, 124, "Expected timeout exit code 124");
    assert!(elapsed < Duration::from_secs(2), "Execution should abort quickly on timeout");
    // Cleanup for other tests
    std::env::remove_var("NXSH_TIMEOUT_MS");
}
