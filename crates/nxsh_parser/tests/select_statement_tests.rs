use nxsh_parser::ShellCommandParser;

#[test]
fn parse_select_with_options_and_body() {
    let p = ShellCommandParser::new();
    let src = "select x in a b c do echo done";
    let ast = p.parse(src).unwrap();
    let s = format!("{ast}");
    assert!(s.contains("echo"));
}

#[test]
fn parse_select_minimal_body() {
    let p = ShellCommandParser::new();
    // Without explicit options list (grammar allows optional)
    let src = "select item do echo ok done";
    let ast = p.parse(src).unwrap();
    let s = format!("{ast}");
    assert!(s.contains("echo"));
}
