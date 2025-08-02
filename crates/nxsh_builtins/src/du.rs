//! `du` command  Eestimate file space usage.
//! Usage: du [-h] [PATH]
//!   -h : human readable units
//! If PATH omitted, uses current directory.

use anyhow::Result;
use walkdir::WalkDir;
use bytesize::ByteSize;
use std::path::Path;
use tokio::task;

pub async fn du_cli(args: &[String]) -> Result<()> {
    let mut human = false;
    let mut path = ".".to_string();
    for arg in args {
        if arg == "-h" { human = true; continue; }
        path = arg.clone();
    }
    let p = Path::new(&path).to_path_buf();
    let size = task::spawn_blocking(move || calc_size(p)).await??;
    if human {
        println!("{}", bytesize::ByteSize::b(size).to_string_as(true));
    } else {
        println!("{}", size);
    }
    Ok(())
}

fn calc_size(root: std::path::PathBuf) -> Result<u64> {
    let mut total = 0u64;
    for entry in WalkDir::new(root) {
        let e = entry?;
        if e.file_type().is_file() {
            total += e.metadata()?.len();
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;
    #[tokio::test]
    async fn du_basic() {
        let d = tempdir().unwrap();
        let f = d.path().join("a.txt");
        let mut file = std::fs::File::create(&f).unwrap();
        writeln!(file, "hello").unwrap();
        du_cli(&[d.path().to_string_lossy().into()]).await.unwrap();
    }
} 
