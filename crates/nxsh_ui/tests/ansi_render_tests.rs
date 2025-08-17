
#[test]
fn parse_basic_segments() {
    let line = "\x1b[31mRED\x1b[0m normal \x1b[1;32mBOLDGREEN\x1b[0m";
    let segs = nxsh_ui::ansi_render::parse_ansi_segments(line);
    assert!(!segs.is_empty());
    // Expect at least 3 segments: RED, normal, BOLDGREEN
    assert!(segs.len() >= 3);
}

#[test]
fn color_code_map() {
    let red = nxsh_ui::ansi_render::color_from_code(31);
    assert_eq!(red[3], 0xFF);
    let unknown = nxsh_ui::ansi_render::color_from_code(12345);
    assert_eq!(unknown[3], 0xFF);
}


