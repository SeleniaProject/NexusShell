use nxsh_builtins::{bg::bg, fg::fg};
use nxsh_core::context::ShellContext;

#[test]
fn bg_fg_no_job() {
    // Should handle gracefully when no job present
    let mut ctx = ShellContext::new();
    let args: Vec<String> = vec![];

    // These should work without errors even if no jobs exist (they simulate success)
    assert!(fg(&mut ctx, &args).is_ok());
    assert!(bg(&mut ctx, &args).is_ok());
}
