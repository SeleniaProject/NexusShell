use crate::common::{BuiltinContext, BuiltinResult};
use std::fs;
use std::path::Path;

/// Create links between files
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    if args.is_empty() {
        eprintln!("ln: missing file operand");
        return Ok(1);
    }

    let mut symbolic = false;
    let mut force = false;
    let mut verbose = false;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--symbolic" => symbolic = true,
            "-f" | "--force" => force = true,
            "-v" | "--verbose" => verbose = true,
            "-h" | "--help" => {
                print_help();
                return Ok(0);
            }
            arg if arg.starts_with('-') => {
                eprintln!("ln: invalid option '{arg}'");
                return Ok(1);
            }
            _ => files.push(&args[i]),
        }
        i += 1;
    }

    if files.len() < 2 {
        let default_file = String::new();
        let first_file = files.first().map(|s| s.as_str()).unwrap_or(&default_file);
        eprintln!("ln: missing destination file operand after '{first_file}'");
        return Ok(1);
    }

    let target = files[0];
    let link_name = files[1];

    if Path::new(link_name).exists() && !force {
        eprintln!("ln: failed to create link '{link_name}': File exists");
        return Ok(1);
    }

    if force && Path::new(link_name).exists() {
        if let Err(e) = fs::remove_file(link_name) {
            eprintln!("ln: cannot remove '{link_name}': {e}");
            return Ok(1);
        }
    }

    let result = if symbolic {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link_name)
        }
        #[cfg(windows)]
        {
            if Path::new(target).is_dir() {
                std::os::windows::fs::symlink_dir(target, link_name)
            } else {
                std::os::windows::fs::symlink_file(target, link_name)
            }
        }
    } else {
        #[cfg(unix)]
        {
            fs::hard_link(target, link_name)
        }
        #[cfg(windows)]
        {
            fs::hard_link(target, link_name)
        }
    };

    match result {
        Ok(()) => {
            if verbose {
                if symbolic {
                    println!("'{link_name}' -> '{target}'");
                } else {
                    println!("'{link_name}' => '{target}'");
                }
            }
            Ok(0)
        }
        Err(e) => {
            eprintln!(
                "ln: failed to create {} link '{}' -> '{}': {}",
                if symbolic { "symbolic" } else { "hard" },
                link_name,
                target,
                e
            );
            Ok(1)
        }
    }
}

fn print_help() {
    println!("Usage: ln [OPTION]... TARGET LINK_NAME");
    println!("Create a link to TARGET with the name LINK_NAME.");
    println!();
    println!("Options:");
    println!("  -s, --symbolic     create symbolic links instead of hard links");
    println!("  -f, --force        remove existing destination files");
    println!("  -v, --verbose      print name of each linked file");
    println!("  -h, --help         display this help and exit");
    println!();
    println!("Examples:");
    println!("  ln file1 file2          Create hard link 'file2' to 'file1'");
    println!("  ln -s file1 file2       Create symbolic link 'file2' to 'file1'");
}
