//! `tree` command  Edirectory tree listing.
//! Usage: tree [DIR]
//! Prints ASCII tree up to default depth 3; use -L <n> to limit depth.

use anyhow::Result;
use walkdir::WalkDir;
use std::path::Path;
use tokio::task;

pub async fn tree_cli(args: &[String]) -> Result<()> {
    let mut depth = 3;
    let mut path = ".".to_string();
    let mut idx = 0;
    if args.get(0).map(|s| s.as_str()) == Some("-L") {
        depth = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
        idx = 2;
    }
    if let Some(p) = args.get(idx) { path = p.clone(); }
    let pbuf = Path::new(&path).to_path_buf();
    task::spawn_blocking(move || display_tree(pbuf, depth)).await??;
    Ok(())
}

fn display_tree(root: std::path::PathBuf, depth: usize) -> Result<()> {
    println!("{}", root.display());
    for entry in WalkDir::new(&root).min_depth(1).max_depth(depth) {
        let e = entry?;
        let depth_level = e.depth();
        let indent = "━E  ".repeat(depth_level - 1);
        let name = e.file_name().to_string_lossy();
        let branch = if e.depth() == depth { "└──" } else { "├──" };
        println!("{}{} {}", indent, branch, name);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    #[tokio::test]
    async fn tree_runs() {
        let d = tempdir().unwrap();
        fs::create_dir(d.path().join("sub")).unwrap();
        tree_cli(&[d.path().to_string_lossy().into()]).await.unwrap();
    }
} 
