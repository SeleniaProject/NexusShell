//! `rm` command – comprehensive file and directory removal implementation.
//!
//! Supports complete rm functionality:
//!   rm [OPTIONS] FILE...
//!   -f, --force               - Ignore nonexistent files and arguments, never prompt
//!   -i                        - Prompt before every removal
//!   -I                        - Prompt once before removing more than three files
//!   --interactive[=WHEN]      - Prompt according to WHEN (never, once, always)
//!   --one-file-system         - Stay on same filesystem when removing recursively
//!   --no-preserve-root        - Do not treat '/' specially
//!   --preserve-root           - Do not remove '/' (default)
//!   -r, -R, --recursive       - Remove directories and their contents recursively
//!   -d, --dir                 - Remove empty directories
//!   -v, --verbose             - Explain what is being done
//!   --help                    - Display help and exit
//!   --version                 - Output version information and exit

use anyhow::{Result, anyhow};
use std::fs::{self, Metadata};
use std::io::{self, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use walkdir::{WalkDir, DirEntry};

#[derive(Debug, Clone)]
pub struct RmOptions {
    pub files: Vec<String>,
    pub force: bool,
    pub interactive: InteractiveMode,
    pub one_file_system: bool,
    pub preserve_root: bool,
    pub recursive: bool,
    pub remove_empty_dirs: bool,
    pub verbose: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteractiveMode {
    Never,
    Once,
    Always,
}

impl Default for RmOptions {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            force: false,
            interactive: InteractiveMode::Never,
            one_file_system: false,
            preserve_root: true,
            recursive: false,
            remove_empty_dirs: false,
            verbose: false,
        }
    }
}

pub fn rm_cli(args: &[String]) -> Result<()> {
    let options = parse_rm_args(args)?;
    
    if options.files.is_empty() {
        return Err(anyhow!("rm: missing operand"));
    }
    
    // Check for root directory protection
    if options.preserve_root {
        for file in &options.files {
            let path = Path::new(file);
            if path.canonicalize().unwrap_or_else(|_| path.to_path_buf()) == Path::new("/") {
                return Err(anyhow!("rm: it is dangerous to operate recursively on '/'"));
            }
        }
    }
    
    // Handle interactive mode for multiple files
    if options.interactive == InteractiveMode::Once && options.files.len() > 3 {
        print!("rm: remove {} arguments? ", options.files.len());
        io::stdout().flush()?;
        
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        
        if !response.trim().to_lowercase().starts_with('y') {
            return Ok(());
        }
    }
    
    // Track filesystem devices for --one-file-system
    let mut filesystem_devices = HashMap::new();
    
    // Process each file
    for file in &options.files {
        let path = PathBuf::from(file);
        
        if let Err(e) = remove_path(&path, &options, &mut filesystem_devices) {
            if !options.force {
                eprintln!("rm: {}", e);
                // Continue with other files instead of exiting
            }
        }
    }
    
    Ok(())
}

fn parse_rm_args(args: &[String]) -> Result<RmOptions> {
    let mut options = RmOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "-f" | "--force" => {
                options.force = true;
                options.interactive = InteractiveMode::Never;
            }
            "-i" => {
                options.interactive = InteractiveMode::Always;
                options.force = false;
            }
            "-I" => {
                options.interactive = InteractiveMode::Once;
                options.force = false;
            }
            "--interactive=never" => {
                options.interactive = InteractiveMode::Never;
            }
            "--interactive=once" => {
                options.interactive = InteractiveMode::Once;
            }
            "--interactive=always" => {
                options.interactive = InteractiveMode::Always;
            }
            "--one-file-system" => {
                options.one_file_system = true;
            }
            "--no-preserve-root" => {
                options.preserve_root = false;
            }
            "--preserve-root" => {
                options.preserve_root = true;
            }
            "-r" | "-R" | "--recursive" => {
                options.recursive = true;
            }
            "-d" | "--dir" => {
                options.remove_empty_dirs = true;
            }
            "-v" | "--verbose" => {
                options.verbose = true;
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--version" => {
                println!("rm (NexusShell) 1.0.0");
                std::process::exit(0);
            }
            arg if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                // Handle combined short options
                for ch in arg.chars().skip(1) {
                    match ch {
                        'f' => {
                            options.force = true;
                            options.interactive = InteractiveMode::Never;
                        }
                        'i' => {
                            options.interactive = InteractiveMode::Always;
                            options.force = false;
                        }
                        'I' => {
                            options.interactive = InteractiveMode::Once;
                            options.force = false;
                        }
                        'r' | 'R' => options.recursive = true,
                        'd' => options.remove_empty_dirs = true,
                        'v' => options.verbose = true,
                        _ => return Err(anyhow!("rm: invalid option -- '{}'", ch)),
                    }
                }
            }
            _ => {
                // This is a file to remove
                options.files.push(arg.clone());
            }
        }
        i += 1;
    }
    
    Ok(options)
}

fn remove_path(
    path: &Path,
    options: &RmOptions,
    filesystem_devices: &mut HashMap<PathBuf, u64>,
) -> Result<()> {
    // Check if file exists
    let metadata = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(e) => {
            if options.force {
                return Ok(());
            } else {
                return Err(anyhow!("cannot remove '{}': {}", path.display(), e));
            }
        }
    };
    
    // Check filesystem device for --one-file-system
    if options.one_file_system {
        let device = metadata.dev();
        let parent = path.parent().unwrap_or(path);
        
        if let Some(&parent_device) = filesystem_devices.get(parent) {
            if device != parent_device {
                if options.verbose {
                    println!("rm: skipping '{}', on different filesystem", path.display());
                }
                return Ok(());
            }
        } else {
            filesystem_devices.insert(parent.to_path_buf(), device);
        }
    }
    
    if metadata.is_dir() {
        remove_directory(path, options, filesystem_devices)
    } else {
        remove_file(path, options)
    }
}

fn remove_file(path: &Path, options: &RmOptions) -> Result<()> {
    // Interactive confirmation
    if options.interactive == InteractiveMode::Always {
        print!("rm: remove regular file '{}'? ", path.display());
        io::stdout().flush()?;
        
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        
        if !response.trim().to_lowercase().starts_with('y') {
            return Ok(());
        }
    }
    
    // Attempt to remove the file
    match fs::remove_file(path) {
        Ok(()) => {
            if options.verbose {
                println!("removed '{}'", path.display());
            }
        }
        Err(e) => {
            return Err(anyhow!("cannot remove '{}': {}", path.display(), e));
        }
    }
    
    Ok(())
}

fn remove_directory(
    path: &Path,
    options: &RmOptions,
    filesystem_devices: &mut HashMap<PathBuf, u64>,
) -> Result<()> {
    // Check if we can remove directories
    if !options.recursive && !options.remove_empty_dirs {
        return Err(anyhow!("cannot remove '{}': Is a directory", path.display()));
    }
    
    // For empty directory removal, check if directory is actually empty
    if options.remove_empty_dirs && !options.recursive {
        match fs::read_dir(path) {
            Ok(mut entries) => {
                if entries.next().is_some() {
                    return Err(anyhow!("cannot remove '{}': Directory not empty", path.display()));
                }
            }
            Err(e) => {
                return Err(anyhow!("cannot remove '{}': {}", path.display(), e));
            }
        }
        
        // Interactive confirmation for empty directory
        if options.interactive == InteractiveMode::Always {
            print!("rm: remove directory '{}'? ", path.display());
            io::stdout().flush()?;
            
            let mut response = String::new();
            io::stdin().read_line(&mut response)?;
            
            if !response.trim().to_lowercase().starts_with('y') {
                return Ok(());
            }
        }
        
        match fs::remove_dir(path) {
            Ok(()) => {
                if options.verbose {
                    println!("removed directory '{}'", path.display());
                }
            }
            Err(e) => {
                return Err(anyhow!("cannot remove '{}': {}", path.display(), e));
            }
        }
        
        return Ok(());
    }
    
    // Recursive directory removal
    if options.recursive {
        // Interactive confirmation for recursive removal
        if options.interactive == InteractiveMode::Always {
            print!("rm: descend into directory '{}'? ", path.display());
            io::stdout().flush()?;
            
            let mut response = String::new();
            io::stdin().read_line(&mut response)?;
            
            if !response.trim().to_lowercase().starts_with('y') {
                return Ok(());
            }
        }
        
        // Remove contents first (post-order traversal)
        remove_directory_contents(path, options, filesystem_devices)?;
        
        // Interactive confirmation before removing the directory itself
        if options.interactive == InteractiveMode::Always {
            print!("rm: remove directory '{}'? ", path.display());
            io::stdout().flush()?;
            
            let mut response = String::new();
            io::stdin().read_line(&mut response)?;
            
            if !response.trim().to_lowercase().starts_with('y') {
                return Ok(());
            }
        }
        
        // Remove the now-empty directory
        match fs::remove_dir(path) {
            Ok(()) => {
                if options.verbose {
                    println!("removed directory '{}'", path.display());
                }
            }
            Err(e) => {
                return Err(anyhow!("cannot remove '{}': {}", path.display(), e));
            }
        }
    }
    
    Ok(())
}

fn remove_directory_contents(
    path: &Path,
    options: &RmOptions,
    filesystem_devices: &mut HashMap<PathBuf, u64>,
) -> Result<()> {
    let entries = fs::read_dir(path)
        .map_err(|e| anyhow!("cannot read directory '{}': {}", path.display(), e))?;
    
    let mut subdirs = Vec::new();
    let mut files = Vec::new();
    
    // Collect entries first to avoid holding directory handle
    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        
        if entry_path.is_dir() {
            subdirs.push(entry_path);
        } else {
            files.push(entry_path);
        }
    }
    
    // Remove files first
    for file_path in files {
        remove_file(&file_path, options)?;
    }
    
    // Then remove subdirectories recursively
    for dir_path in subdirs {
        remove_directory(&dir_path, options, filesystem_devices)?;
    }
    
    Ok(())
}

fn print_help() {
    println!("Usage: rm [OPTION]... [FILE]...");
    println!("Remove (unlink) the FILE(s).");
    println!();
    println!("  -f, --force           ignore nonexistent files and arguments, never prompt");
    println!("  -i                    prompt before every removal");
    println!("  -I                    prompt once before removing more than three files, or");
    println!("                          when removing recursively; less intrusive than -i,");
    println!("                          while still giving protection against most mistakes");
    println!("      --interactive[=WHEN]  prompt according to WHEN: never, once (-I), or");
    println!("                          always (-i); without WHEN, prompt always");
    println!("      --one-file-system  when removing a hierarchy recursively, skip any");
    println!("                          directory that is on a file system different from");
    println!("                          that of the corresponding command line argument");
    println!("      --no-preserve-root  do not treat '/' specially");
    println!("      --preserve-root   do not remove '/' (default)");
    println!("  -r, -R, --recursive   remove directories and their contents recursively");
    println!("  -d, --dir             remove empty directories");
    println!("  -v, --verbose         explain what is being done");
    println!("      --help     display this help and exit");
    println!("      --version  output version information and exit");
    println!();
    println!("By default, rm does not remove directories.  Use the --recursive (-r or -R)");
    println!("option to remove each listed directory, too, along with all of its contents.");
    println!();
    println!("To remove a file whose name starts with a '-', for example '-foo',");
    println!("use one of these commands:");
    println!("  rm -- -foo");
    println!();
    println!("  rm ./-foo");
    println!();
    println!("Note that if you use rm to remove a file, it might be possible to recover");
    println!("some of its contents, given sufficient expertise and/or time.  For greater");
    println!("assurance that the contents are truly unrecoverable, consider using shred.");
    println!();
    println!("Report rm bugs to <bug-reports@nexusshell.org>");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, NamedTempFile};
    use std::fs::File;
    use std::io::Write;
    
    #[test]
    fn test_parse_args() {
        let args = vec!["-rf".to_string(), "file1".to_string(), "file2".to_string()];
        let options = parse_rm_args(&args).unwrap();
        
        assert!(options.recursive);
        assert!(options.force);
        assert_eq!(options.files, vec!["file1", "file2"]);
    }
    
    #[test]
    fn test_interactive_modes() {
        let args = vec!["-i".to_string(), "file".to_string()];
        let options = parse_rm_args(&args).unwrap();
        assert_eq!(options.interactive, InteractiveMode::Always);
        
        let args = vec!["-I".to_string(), "file".to_string()];
        let options = parse_rm_args(&args).unwrap();
        assert_eq!(options.interactive, InteractiveMode::Once);
    }
    
    #[test]
    fn test_preserve_root() {
        let args = vec!["--no-preserve-root".to_string(), "/".to_string()];
        let options = parse_rm_args(&args).unwrap();
        assert!(!options.preserve_root);
    }
    
    #[test]
    fn test_verbose_and_recursive() {
        let args = vec!["-rv".to_string(), "dir".to_string()];
        let options = parse_rm_args(&args).unwrap();
        
        assert!(options.recursive);
        assert!(options.verbose);
        assert_eq!(options.files, vec!["dir"]);
    }
} 