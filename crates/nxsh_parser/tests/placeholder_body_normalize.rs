use nxsh_parser::ShellCommandParser;

#[test]
fn normalize_if_then_else_program_wrapping() {
    let parser = ShellCommandParser::new();
    let src = "if test 1 -eq 1 then echo ok else echo ng fi";
    let ast = parser.parse(src).unwrap();
    // Smoke test: parse succeeds and no panic due to double wrapping
    let s = format!("{ast}");
    assert!(s.contains("echo"));
}

#[test]
fn normalize_case_arm_body_wrapping() {
    let parser = ShellCommandParser::new();
    let src = "case x in _ ) echo any ;; esac";
    let ast = parser.parse(src).unwrap();
    let s = format!("{ast}");
    assert!(s.contains("echo"));
}


