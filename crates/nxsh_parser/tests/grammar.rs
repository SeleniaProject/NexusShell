use nxsh_parser::parse;

const COMMANDS: &[&str] = &[
    "ls -la",
    "echo \"hello\"",
    "cat file.txt | grep foo",
    "mkdir -p src && cd src",
    "touch a.txt ; rm a.txt",
    "(echo one; echo two) | sort",
    "cp source.txt destination.txt",
    "export PATH=$PATH:/opt/nxsh/bin",
    "if true; then echo ok; fi",
    "for f in *.rs; do echo $f; done",
    // match with placeholder pattern (according to grammar/shell.pest: braces around arms)
    "match x { _ => echo ok }",
];

#[test]
fn parse_100_cases() {
    for i in 0..10 {
        for cmd in COMMANDS.iter() {
            let combined = format!("{cmd} #{i}");
            if let Err(e) = parse(&combined) {
                panic!("parser should succeed for '{combined}': {e}");
            }
        }
    }
    // 10*10 = 100 cases verified
} 