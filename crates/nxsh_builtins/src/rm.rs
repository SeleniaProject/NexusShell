//! `rm` command - comprehensive file and directory removal implementation.

use anyhow::{anyhow, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

// Beautiful CUI design
use crate::common::{BuiltinContext, BuiltinResult};
use crate::ui_design::{ColorPalette, Colorize, Icons};

/// Options for rm command
#[derive(Debug, Clone)]
pub struct RmOptions {
    pub force: bool,
    pub interactive: InteractiveMode,
    pub recursive: bool,
    pub verbose: bool,
    pub preserve_root: bool,
    pub one_file_system: bool,
    pub dir: bool,
}

#[derive(Debug, Clone)]
pub enum InteractiveMode {
    Never,
    Once,
    Always,
}

impl Default for RmOptions {
    fn default() -> Self {
        Self {
            force: false,
            interactive: InteractiveMode::Never,
            recursive: false,
            verbose: false,
            preserve_root: true,
            one_file_system: false,
            dir: false,
        }
    }
}

/// Remove a file with the given options
fn remove_file(path: &Path, options: &RmOptions) -> Result<()> {
    if !path.exists() {
        if !options.force {
            return Err(anyhow!(
                "cannot remove '{}': No such file or directory",
                path.display()
            ));
        }
        return Ok(());
    }

    // Interactive confirmation
    if matches!(options.interactive, InteractiveMode::Always) {
        print!("rm: remove regular file '{}'? ", path.display());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().starts_with('y') && !input.trim().starts_with('Y') {
            return Ok(());
        }
    }

    match fs::remove_file(path) {
        Ok(()) => {
            if options.verbose {
                let palette = ColorPalette::new();
                println!(
                    "{} {} {}",
                    Icons::FOLDER,
                    "Removed file:".colorize(&palette.warning),
                    path.display().to_string().colorize(&palette.primary)
                );
            }
        }
        Err(e) => {
            return Err(anyhow!("cannot remove '{}': {}", path.display(), e));
        }
    }
    Ok(())
}

/// Remove a directory with the given options
fn remove_directory(path: &Path, options: &RmOptions) -> Result<()> {
    if !path.exists() {
        if !options.force {
            return Err(anyhow!(
                "cannot remove '{}': No such file or directory",
                path.display()
            ));
        }
        return Ok(());
    }

    if !options.recursive && !options.dir {
        return Err(anyhow!(
            "cannot remove '{}': Is a directory",
            path.display()
        ));
    }

    // Recursive removal
    if options.recursive {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                remove_directory(&entry_path, options)?;
            } else {
                remove_file(&entry_path, options)?;
            }
        }
    }

    // Remove the directory itself
    match fs::remove_dir(path) {
        Ok(()) => {
            if options.verbose {
                let palette = ColorPalette::new();
                println!(
                    "{} {} {}",
                    Icons::FOLDER,
                    "Removed directory:".colorize(&palette.warning),
                    path.display().to_string().colorize(&palette.primary)
                );
            }
        }
        Err(e) => {
            return Err(anyhow!(
                "cannot remove directory '{}': {}",
                path.display(),
                e
            ));
        }
    }
    Ok(())
}

/// Parse command line arguments
fn parse_args(args: &[String]) -> Result<(RmOptions, Vec<String>)> {
    let mut options = RmOptions::default();
    let mut files = Vec::new();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--force" => options.force = true,
            "-i" => options.interactive = InteractiveMode::Always,
            "-I" => options.interactive = InteractiveMode::Once,
            "-r" | "-R" | "--recursive" => options.recursive = true,
            "-v" | "--verbose" => options.verbose = true,
            "-d" | "--dir" => options.dir = true,
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            arg if arg.starts_with('-') => {
                return Err(anyhow!("invalid option: {}", arg));
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    if files.is_empty() {
        return Err(anyhow!("missing operand"));
    }

    Ok((options, files))
}

/// Print help message
fn print_help() {
    println!(
        "rm - remove files and directories

USAGE:
    rm [OPTIONS] FILE...

OPTIONS:
    -f, --force               Ignore nonexistent files, never prompt
    -i                        Prompt before every removal
    -I                        Prompt once before removing more than three files
    -r, -R, --recursive       Remove directories and their contents recursively
    -v, --verbose             Explain what is being done
    -d, --dir                 Remove empty directories
    --help                    Display this help and exit

EXAMPLES:
    rm file.txt               Remove a file
    rm -f file.txt            Force remove without prompting
    rm -r directory/          Remove directory recursively
    rm -rf temp/              Force remove directory
    rm -i *.txt               Interactive removal
    rm -v file1 file2         Verbose removal"
    );
}

/// Execute the rm builtin command
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    if args.is_empty() {
        eprintln!("rm: missing operand");
        return Ok(1);
    }

    let (options, files) = match parse_args(args) {
        Ok((opts, files)) => (opts, files),
        Err(e) => {
            eprintln!("rm: {e}");
            return Ok(1);
        }
    };

    // Special handling for interactive mode "once"
    if matches!(options.interactive, InteractiveMode::Once) && files.len() > 3 {
        print!("rm: remove {} arguments? ", files.len());
        io::stdout().flush().unwrap_or(());
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok()
            && !input.trim().starts_with('y')
            && !input.trim().starts_with('Y')
        {
            return Ok(0);
        }
    }

    for file in files {
        let path = Path::new(&file);

        // Root protection
        if options.preserve_root && path == Path::new("/") {
            eprintln!("rm: it is dangerous to operate recursively on '/'");
            continue;
        }

        let result = if path.is_dir() {
            remove_directory(path, &options)
        } else {
            remove_file(path, &options)
        };

        if let Err(e) = result {
            if !options.force {
                eprintln!("rm: {e}");
                return Ok(1);
            }
        }
    }

    Ok(0)
}
