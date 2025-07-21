use nxsh_builtins::{bg, fg};

#[test]
fn bg_fg_no_job() {
    // Should handle gracefully when no job present
    fg(None).unwrap();
    bg(None).unwrap();
} 