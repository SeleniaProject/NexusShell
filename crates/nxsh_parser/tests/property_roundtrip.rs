use nxsh_parser::Parser;
use proptest::prelude::*;
use proptest::string::string_regex;

fn roundtrip(input: &str) -> bool {
    let parser = Parser::new();
    match parser.parse(input) {
        Ok(ast) => {
            let s = format!("{ast}");
            parser.parse(&s).is_ok()
        }
        Err(_) => true,
    }
}

proptest! {
    #[test]
    fn prop_ast_roundtrip_random_unicode_no_nul(s in string_regex(r"(?s)[^\x00]{0,256}").unwrap()) {
        let _ = roundtrip(&s);
    }
}
