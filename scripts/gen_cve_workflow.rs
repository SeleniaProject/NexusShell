// Simple generator to scaffold CVE hotfix workflow files
// Usage: cargo run -p nxsh_builtins --bin dummy (not wired) or run with rustc directly

use std::{env, fs, path::PathBuf};

fn main() {
    let cve_id = env::args().nth(1).unwrap_or_else(|| "CVE-YYYY-NNNNN".to_string());
    let mut out = PathBuf::from("workflows/cve_hotfix");
    let _ = fs::create_dir_all(&out);
    out.push(format!("{}.md", cve_id));
    let content = format!(
        "# Hotfix Workflow: {cve}\n\n- [ ] Branch: hotfix/{cve}\n- [ ] Repro/PoC\n- [ ] Patch + Tests\n- [ ] CI green\n- [ ] Release notes\n- [ ] Tag/Publish\n- [ ] Advisory + Postmortem\n",
        cve = cve_id
    );
    fs::write(&out, content).expect("failed to write workflow file");
    println!("Generated {}", out.display());
}

