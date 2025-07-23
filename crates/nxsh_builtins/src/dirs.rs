//! `dirs` builtin – display contents of directory stack.
//! Currently supports basic usage: `dirs` prints stack from most-recent to oldest.
//! Options `-c` (clear), `-l` (long path) are stubbed for future expansion.

use anyhow::Result;
use once_cell::sync::Lazy;
use std::env;
use std::path::PathBuf;
use std::sync::Mutex;

pub static DIR_STACK: Lazy<Mutex<Vec<PathBuf>>> = Lazy::new(|| {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    Mutex::new(vec![cwd])
});

pub fn pushd(dir: PathBuf) {
    let mut stack = DIR_STACK.lock().unwrap();
    stack.push(dir);
}

pub fn popd() -> Option<PathBuf> {
    let mut stack = DIR_STACK.lock().unwrap();
    if stack.len() > 1 { stack.pop() } else { None }
}

/// Entry function for `dirs` builtin.
pub fn dirs_cli(args: &[String]) -> Result<()> {
    if args.get(0).map(|s| s.as_str()) == Some("-c") {
        // Clear stack except current dir.
        let cwd = env::current_dir()?;
        let mut stack = DIR_STACK.lock().unwrap();
        stack.clear();
        stack.push(cwd);
        return Ok(());
    }
    let stack = DIR_STACK.lock().unwrap();
    for (i, path) in stack.iter().rev().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", path.display());
    }
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn print_stack() {
        dirs_cli(&[]).unwrap();
    }
} 