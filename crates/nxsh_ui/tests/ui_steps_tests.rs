use nxsh_ui::ui_ux::UIUXSystem;

#[test]
fn interactive_steps_for_common_commands() {
    let ui = UIUXSystem::new();
    let grep_steps = ui.get_command_steps("grep").unwrap();
    assert!(!grep_steps.is_empty());
    let cp_steps = ui.get_command_steps("cp").unwrap();
    assert!(!cp_steps.is_empty());
}


