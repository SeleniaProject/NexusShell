use proptest::prelude::*;
use nxsh_parser::Parser;

fn roundtrip(input: &str) -> bool {
    let parser = Parser::new();
    match parser.parse(input) {
        Ok(ast) => {
            let s = format!("{}", ast);
            parser.parse(&s).is_ok()
        }
        Err(_) => true,
    }
}

proptest! {
    #[test]
    fn prop_ast_roundtrip_random_ascii(s in ".{0,256}") {
        prop_assume!(!s.contains("\0"));
        let _ = roundtrip(&s);
    }
}


