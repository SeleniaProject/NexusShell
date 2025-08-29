use nxsh_core::context::ShellContext;
use nxsh_core::executor::Executor;
use nxsh_parser::ast::AstNode;

fn sleep_command(seconds: &str) -> AstNode {
    AstNode::Command {
        name: Box::new(AstNode::Word("sleep")),
        args: vec![AstNode::Word(seconds)],
        redirections: vec![],
        background: false,
    }
}

#[test]
fn per_command_timeout_triggers() {
    // Ensure no stray global timeout from environment affects this test
    std::env::remove_var("NXSH_TIMEOUT_MS");
    let mut ctx = ShellContext::new();
    ctx.clear_global_timeout();
    ctx.set_per_command_timeout(Some(std::time::Duration::from_millis(200)));
    let mut exec = Executor::new();
    let ast = sleep_command("2");
    let result = exec.execute(&ast, &mut ctx).expect("execute");
    assert_eq!(
        result.exit_code, 124,
        "expected timeout exit code 124, got {}",
        result.exit_code
    );
}

#[test]
fn per_command_timeout_not_triggering() {
    // Ensure environment/global timeout does not force a timeout here
    std::env::remove_var("NXSH_TIMEOUT_MS");
    let mut ctx = ShellContext::new();
    ctx.clear_global_timeout();
    ctx.set_per_command_timeout(Some(std::time::Duration::from_secs(2)));
    let mut exec = Executor::new();
    let ast = sleep_command("0");
    let result = exec.execute(&ast, &mut ctx).expect("execute");
    assert_eq!(result.exit_code, 0);
}
