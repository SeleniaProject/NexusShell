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

    // Parse docs/COMMANDS.md for declared commands (very naive: lines starting with | name |)
    let mut declared = BTreeSet::new();
    if let Ok(spec) = fs::read_to_string(manifest.join("docs").join("COMMANDS.md")) {
        for line in spec.lines() {
            if let Some(stripped) = line.strip_prefix("| ") { // table row
                if let Some(rest) = stripped.split('|').next() {
                    let candidate = rest.trim();
                    if !candidate.is_empty() && candidate.chars().all(|c| c.is_ascii_alphanumeric() || [ '-', '_'].contains(&c)) {
                        declared.insert(candidate.to_string());
                    }
                }
            }
        }
    }

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

    // Diff with existing COMMAND_STATUS.md if present
    let existing = manifest.join("COMMAND_STATUS.md");
    if existing.exists() {
        let old = fs::read_to_string(&existing)?;
        if old != md {
            let diff_path = manifest.join("COMMAND_STATUS.diff.md");
            // Very naive line-based diff (added/removed)
            let old_set: BTreeSet<&str> = old.lines().collect();
            let new_set: BTreeSet<&str> = md.lines().collect();
            let mut diff = String::from("# COMMAND_STATUS DIFF\n\n## Added\n");
            for l in new_set.difference(&old_set) { if l.starts_with("| `") { diff.push_str("+ "); diff.push_str(l); diff.push('\n'); } }
            diff.push_str("\n## Removed\n");
            for l in old_set.difference(&new_set) { if l.starts_with("| `") { diff.push_str("- "); diff.push_str(l); diff.push('\n'); } }
            fs::write(&diff_path, diff)?;
            
            if update_mode {
                // Update mode: write the new content and continue
                fs::write(&existing, &md)?;
                println!("Updated COMMAND_STATUS.md with new content");
            } else {
                // Check mode: report differences and exit with error code for CI
                eprintln!("Command status changed. See {}", diff_path.display());
                eprintln!("Run with --update flag to apply changes automatically");
                std::process::exit(4); // Non-zero to signal CI
            }
        } else {
            println!("COMMAND_STATUS.md is up to date");
        }
    } else if update_mode {
        // Create new COMMAND_STATUS.md in update mode
        fs::write(&existing, &md)?;
        println!("Created new COMMAND_STATUS.md");
    }
    Ok(())
}
