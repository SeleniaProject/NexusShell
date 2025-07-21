use std::{fs, path::Path};
fn main() {
    let cases = vec!["echo hello", "ls"];
    let out_dir = Path::new("tests/golden");
    fs::create_dir_all(out_dir).unwrap();
    for case in cases {
        let file = out_dir.join(format!("{}.json", case.replace(' ', "_")));
        fs::write(file, format!("{{\"cmd\":\"{}\",\"stdout\":\"stub\"}}", case)).unwrap();
    }
} 