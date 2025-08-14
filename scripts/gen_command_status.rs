// Simple command status generator: scans builtins dispatch table generated list
// and outputs a markdown summary. (Lightweight placeholder until full spec diff impl.)
use std::{fs, path::PathBuf, env, collections::BTreeSet, io};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let update_mode = args.len() > 1 && args[1] == "--update";
    
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()));
    let builtins_lib = manifest.join("crates").join("nxsh_builtins").join("src").join("lib.rs");
    let content = fs::read_to_string(&builtins_lib)?;
    let start = content.find("pub fn execute_builtin").unwrap_or(0);
    let slice = &content[start..];
    let mut names: Vec<&str> = slice.match_indices("=>").filter_map(|(idx, _)| {
        // backtrack to preceding opening quote
        let pre = &slice[..idx];
        let q = pre.rfind('"')?;
        let pre2 = &pre[..q];
        let q2 = pre2.rfind('"')?;
        Some(&pre[q2+1..q])
    }).collect();
    names.sort();
    names.dedup();
    // Collect implemented builtin names
    let implemented: BTreeSet<String> = names.into_iter().map(|s| s.to_string()).collect();

    // Helper to parse command names from a Markdown table file (first cell in rows)
    fn parse_commands_from_md(path: &PathBuf) -> io::Result<BTreeSet<String>> {
        let mut set = BTreeSet::new();
        let text = fs::read_to_string(path)?;
        for line in text.lines() {
            if let Some(stripped) = line.strip_prefix("| ") {
                if let Some(rest) = stripped.split('|').next() {
                    let candidate = rest.trim();
                    if !candidate.is_empty()
                        && candidate.chars().all(|c| c.is_ascii_alphanumeric() || ['-', '_'].contains(&c))
                    {
                        set.insert(candidate.to_string());
                    }
                }
            }
        }
        Ok(set)
    }

    // Parse docs and spec command catalogs
    let docs_cmds = parse_commands_from_md(&manifest.join("docs").join("COMMANDS.md")).unwrap_or_default();
    let spec_cmds = parse_commands_from_md(&manifest.join("spec").join("COMMANDS.md")).unwrap_or_default();
    let declared: BTreeSet<String> = docs_cmds.union(&spec_cmds).cloned().collect();

    // Generate enhanced markdown with statistics
    let total_declared = declared.len();
    let total_implemented = implemented.len();
    let coverage_percent = if total_declared > 0 { 
        (total_implemented * 100) / total_declared 
    } else { 
        100 
    };

    let mut md = format!(
        "# COMMAND_STATUS (auto-generated)\n\n\
        **Implementation Coverage: {}/{} ({:.1}%)**\n\n\
        | Command | Status | Notes |\n\
        |---------|--------|-------|\n",
        total_implemented, total_declared, coverage_percent as f64
    );
    
    for n in &implemented { 
        md.push_str(&format!("| `{}` | ✅ Implemented | builtin |\n", n)); 
    }
    for missing in declared.difference(&implemented) { 
        md.push_str(&format!("| `{}` | 💤 Missing | Spec listed not yet implemented |\n", missing)); 
    }
    
    md.push_str(&format!("\n---\n*Last updated: {}*\n", 
        std::env::var("BUILD_DATE").unwrap_or_else(|_| "auto-generated".to_string())));
    
    let out = manifest.join("COMMAND_STATUS.generated.md");
    fs::write(&out, &md)?;
    println!("Generated {} with coverage {:.1}%", out.display(), coverage_percent as f64);

    // Prepare unified diff/report and checks
    let mut any_issue = false;
    let mut report = String::from("# COMMAND_STATUS DIFF\n");

    // Check docs <-> spec consistency
    let mut docs_only: Vec<String> = docs_cmds.difference(&spec_cmds).cloned().collect();
    let mut spec_only: Vec<String> = spec_cmds.difference(&docs_cmds).cloned().collect();
    if !docs_only.is_empty() || !spec_only.is_empty() {
        any_issue = true;
        docs_only.sort(); spec_only.sort();
        report.push_str("\n## Docs vs Spec Mismatch\n");
        if !docs_only.is_empty() {
            report.push_str("\n### Present only in docs/COMMANDS.md\n");
            for n in docs_only { report.push_str(&format!("+ `{}`\n", n)); }
        }
        if !spec_only.is_empty() {
            report.push_str("\n### Present only in spec/COMMANDS.md\n");
            for n in spec_only { report.push_str(&format!("- `{}`\n", n)); }
        }
    }

    // Diff with existing COMMAND_STATUS.md if present
    let existing = manifest.join("COMMAND_STATUS.md");
    if existing.exists() {
        let old = fs::read_to_string(&existing)?;
        if old != md {
            any_issue = true;
            // Very naive line-based diff (added/removed)
            let old_set: BTreeSet<&str> = old.lines().collect();
            let new_set: BTreeSet<&str> = md.lines().collect();
            report.push_str("\n## Status Table Changes\n\n### Added\n");
            for l in new_set.difference(&old_set) { if l.starts_with("| `") { report.push_str("+ "); report.push_str(l); report.push('\n'); } }
            report.push_str("\n### Removed\n");
            for l in old_set.difference(&new_set) { if l.starts_with("| `") { report.push_str("- "); report.push_str(l); report.push('\n'); } }
        }
    } else {
        // Missing status file is an issue in check mode
        any_issue = true;
        report.push_str("\n## Missing File\n\n`COMMAND_STATUS.md` not found. Run generator with `--update` to create it.\n");
    }

    // Write report and decide action
    if any_issue {
        let diff_path = manifest.join("COMMAND_STATUS.diff.md");
        fs::write(&diff_path, &report)?;
        if update_mode {
            // Update or create COMMAND_STATUS.md content
            fs::write(&existing, &md)?;
            println!("Updated/Created COMMAND_STATUS.md (see diff for details)");
        } else {
            eprintln!("Found documentation/status inconsistencies. See {}", diff_path.display());
            eprintln!("Run with --update flag to apply status table changes where possible");
            std::process::exit(4);
        }
    } else {
        println!("Documentation and status are consistent");
    }
    Ok(())
}
