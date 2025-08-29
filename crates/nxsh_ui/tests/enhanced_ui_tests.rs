use nxsh_ui::ui_ux::{PromptContext, UIUXSystem, ValidationResult};

#[test]
fn prompt_includes_user_host_and_path() {
    let ui = UIUXSystem::new();
    let ctx = PromptContext {
        username: "user".into(),
        hostname: "host".into(),
        current_path: "/home/user/project".into(),
        git_branch: Some("main".into()),
        last_exit_code: 0,
        is_admin: false,
    };
    let prompt = ui.render_prompt(&ctx);
    assert!(prompt.contains("user"));
    assert!(prompt.contains("host"));
}

#[test]
fn validation_empty_and_known_command() {
    let ui = UIUXSystem::new();
    assert!(matches!(ui.validate_command(""), ValidationResult::Empty));
    assert!(matches!(
        ui.validate_command("echo hi"),
        ValidationResult::Valid
    ));
}
