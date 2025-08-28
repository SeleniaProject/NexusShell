use nxsh_ui::ui_ux::UIUXSystem;

#[test]
fn interactive_flow_basic() {
    let mut ui = UIUXSystem::new();
    let mut sess = ui.start_interactive_mode("cp").unwrap();
    // Step 0 requires 'source'
    assert!(!sess.can_advance());
    sess.set_param("source", "/tmp/a").unwrap();
    assert!(sess.can_advance());
    sess.advance().unwrap();
    // Step 1 requires 'destination'
    assert!(!sess.can_advance());
    sess.set_param("destination", "/tmp/b").unwrap();
    assert!(sess.can_advance());
    sess.advance().unwrap();
    // All required provided
    assert!(sess.is_complete());
    let out = sess.try_complete().unwrap();
    assert!(out.contains("cp"));
}

#[test]
fn interactive_unknown_param() {
    let mut ui = UIUXSystem::new();
    let mut sess = ui.start_interactive_mode("grep").unwrap();
    let err = sess.set_param("nonexist", "x").unwrap_err();
    assert!(format!("{err}").contains("Unknown parameter"));
}


