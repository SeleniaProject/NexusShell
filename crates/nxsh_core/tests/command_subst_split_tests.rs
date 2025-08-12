use nxsh_core::context::ShellContext;
use nxsh_core::executor::Executor;
use nxsh_parser::ast::{AstNode, QuoteType};

// Helper to build command using ArgDump builtin
fn argdump_with_sub(sub_output: &str) -> AstNode {
    let sub = AstNode::CommandSubstitution { command: Box::new(AstNode::StringLiteral { value: sub_output, quote_type: QuoteType::Double }), is_legacy: false };
    AstNode::Command { name: Box::new(AstNode::Word("__argdump")), args: vec![sub], redirections: vec![], background: false }
}

fn parse_count(stdout: &str) -> usize {
    for line in stdout.lines() { if let Some(rest) = line.strip_prefix("count=") { return rest.trim().parse().unwrap_or(0); } }
    0
}

#[test]
fn no_split_default() {
    let mut ctx = ShellContext::new();
    ctx.clear_global_timeout();
    let mut exec = Executor::new();
    let cmd = argdump_with_sub("a b  c");
    let result = exec.execute(&cmd, &mut ctx).expect("run");
    // Expect single argument passed (no splitting)
    assert_eq!(parse_count(&result.stdout), 1, "expected exactly 1 arg, got: {}\n{}", parse_count(&result.stdout), result.stdout);
}

#[test]
fn split_enabled_default_ifs() {
    let mut ctx = ShellContext::new();
    ctx.clear_global_timeout();
    ctx.set_var("NXSH_SUBST_SPLIT", "1");
    let mut exec = Executor::new();
    let cmd = argdump_with_sub("a b  c");
    let result = exec.execute(&cmd, &mut ctx).expect("run");
    let count = parse_count(&result.stdout);
    // Expect >=3 tokens (a, b, c) with double space collapsing producing empty token ignored -> at least 3
    assert!(count >= 3, "expected >=3 tokens when splitting with default IFS, got {}\n{}", count, result.stdout);
}

#[test]
fn split_enabled_custom_ifs() {
    let mut ctx = ShellContext::new();
    ctx.clear_global_timeout();
    ctx.set_var("NXSH_SUBST_SPLIT", "1");
    ctx.set_var("NXSH_IFS", ":");
    let mut exec = Executor::new();
    let cmd = argdump_with_sub("a:b::c");
    let result = exec.execute(&cmd, &mut ctx).expect("run");
    let count = parse_count(&result.stdout);
    // a:b::c with IFS ':' -> fields: a b  c (empty ignored) -> expect >=3 again
    assert!(count >= 3, "expected >=3 tokens with custom IFS ':' splitting a:b::c got {}\n{}", count, result.stdout);
}

#[test]
fn split_empty_output() {
    let mut ctx = ShellContext::new();
    ctx.clear_global_timeout();
    ctx.set_var("NXSH_SUBST_SPLIT", "1");
    let mut exec = Executor::new();
    let cmd = argdump_with_sub("");
    let result = exec.execute(&cmd, &mut ctx).expect("run");
    // Empty output should produce one empty field (our logic returns vec![""])
    assert_eq!(parse_count(&result.stdout), 1, "empty substitution should yield exactly one empty arg\n{}", result.stdout);
}
