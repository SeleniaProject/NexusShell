use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use nxsh_core::{ShellError, ErrorKind};
use nxsh_core::error::RuntimeErrorKind;

pub fn man_cli(args: &[String]) -> Result<(), ShellError> {
    if args.is_empty() || args[0] == "--help" {
        print_help();
        return Ok(());
    }

    let mut section = None;
    let mut keyword = None;
    let mut apropos = false;
    let mut whatis_mode = false;
    let mut path_list = false;
    let mut _debug = false;
    let mut pager = None;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-k" | "--apropos" => apropos = true,
            "-f" | "--whatis" => whatis_mode = true,
            "-w" | "--where" | "--path" => path_list = true,
            "-d" | "--debug" => _debug = true,
            "-P" | "--pager" => {
                if i + 1 < args.len() {
                    pager = Some(args[i + 1].clone());
                    i += 1;
                }
            },
            arg if arg.starts_with('-') && arg.len() == 2 => {
                let section_char = &arg[1..2];
                if section_char.chars().all(|c| c.is_ascii_digit()) {
                    section = Some(section_char.to_string());
                }
            },
            _ => {
                keyword = Some(args[i].clone());
            }
        }
        i += 1;
    }

    if let Some(kw) = keyword {
        if apropos {
            return apropos_search(&kw);
        } else if whatis_mode {
            return whatis_lookup(&kw);
        } else if path_list {
            return show_manual_path(&kw, section.as_deref());
        } else {
            return show_manual_page(&kw, section.as_deref(), pager.as_deref());
        }
    }

    Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::TooFewArguments), "No manual entry specified"))
}

fn print_help() {
    println!("man - an interface to the system reference manuals

USAGE:
    man [OPTION...] [SECTION] PAGE...

OPTIONS:
    -f, --whatis              Display a short description from the manual page
    -k, --apropos             Search the short descriptions and manual page names
    -w, --where, --path       Don't show the manual pages, but print the location(s)
    -P PAGER, --pager=PAGER   Use program PAGER to display output
    -d, --debug              Emit debugging messages
    -h, --help               Show this help

SECTIONS:
    1   Executable programs or shell commands
    2   System calls
    3   Library calls
    4   Special files
    5   File formats and conventions
    6   Games
    7   Miscellaneous
    8   System administration commands
    9   Kernel routines");
}

fn apropos_search(keyword: &str) -> Result<(), ShellError> {
    // Try system apropos first
    if let Ok(output) = Command::new("apropos")
        .arg(keyword)
        .output()
    {
        if output.status.success() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            return Ok(());
        }
    }

    // Fallback: search through our built-in descriptions
    let descriptions = get_builtin_descriptions();
    let mut found = false;
    
    for (name, desc) in &descriptions {
        if name.contains(keyword) || desc.to_lowercase().contains(&keyword.to_lowercase()) {
            println!("{name} - {desc}");
            found = true;
        }
    }

    if !found {
        println!("{keyword}: nothing appropriate.");
    }

    Ok(())
}

fn whatis_lookup(keyword: &str) -> Result<(), ShellError> {
    // Try system whatis first
    if let Ok(output) = Command::new("whatis")
        .arg(keyword)
        .output()
    {
        if output.status.success() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            return Ok(());
        }
    }

    // Fallback: use our built-in descriptions
    let descriptions = get_builtin_descriptions();
    if let Some(desc) = descriptions.get(keyword) {
        println!("{keyword} - {desc}");
    } else {
        println!("{keyword}: nothing appropriate.");
    }

    Ok(())
}

fn show_manual_path(page: &str, section: Option<&str>) -> Result<(), ShellError> {
    let manual_paths = get_manual_paths();
    let mut found = false;

    for man_dir in &manual_paths {
        if let Some(sec) = section {
            let path = man_dir.join(format!("man{sec}")).join(format!("{page}.{sec}"));
            if path.exists() {
                println!("{}", path.display());
                found = true;
            }
            // Also check compressed versions
            for ext in &["gz", "bz2", "xz"] {
                let path = man_dir.join(format!("man{sec}")).join(format!("{page}.{sec}.{ext}"));
                if path.exists() {
                    println!("{}", path.display());
                    found = true;
                }
            }
        } else {
            // Search all sections
            for sec in 1..=9 {
                let path = man_dir.join(format!("man{sec}")).join(format!("{page}.{sec}"));
                if path.exists() {
                    println!("{}", path.display());
                    found = true;
                }
                // Also check compressed versions
                for ext in &["gz", "bz2", "xz"] {
                    let path = man_dir.join(format!("man{sec}")).join(format!("{page}.{sec}.{ext}"));
                    if path.exists() {
                        println!("{}", path.display());
                        found = true;
                    }
                }
            }
        }
    }

    if !found {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("No manual entry for {page}")));
    }

    Ok(())
}

fn show_manual_page(page: &str, section: Option<&str>, pager: Option<&str>) -> Result<(), ShellError> {
    // Try system man first
    let mut cmd = Command::new("man");
    if let Some(sec) = section {
        cmd.arg(sec);
    }
    cmd.arg(page);

    if let Some(pg) = pager {
        cmd.env("PAGER", pg);
    }

    match cmd.status() {
        Ok(status) if status.success() => return Ok(()),
        _ => {}
    }

    // Fallback: try to find and display manual page
    if let Ok(content) = find_manual_content(page, section) {
        display_content(&content, pager)?;
        return Ok(());
    }

    // Final fallback: show built-in help
    if let Some(help) = get_builtin_help(page) {
        display_content(&help, pager)?;
        return Ok(());
    }

    Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("No manual entry for {page}")))
}

fn get_manual_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // Standard Unix manual paths
    let standard_paths = [
        "/usr/share/man",
        "/usr/local/share/man",
        "/usr/man",
        "/usr/local/man",
        "/opt/local/share/man",
    ];

    for path in &standard_paths {
        if Path::new(path).is_dir() {
            paths.push(PathBuf::from(path));
        }
    }

    // Check MANPATH environment variable
    if let Ok(manpath) = std::env::var("MANPATH") {
        for path in manpath.split(':') {
            if !path.is_empty() && Path::new(path).is_dir() {
                paths.push(PathBuf::from(path));
            }
        }
    }

    paths
}

fn find_manual_content(page: &str, section: Option<&str>) -> Result<String, std::io::Error> {
    let manual_paths = get_manual_paths();

    for man_dir in &manual_paths {
        if let Some(sec) = section {
            if let Ok(content) = read_manual_file(&man_dir.join(format!("man{sec}")).join(format!("{page}.{sec}"))) {
                return Ok(content);
            }
        } else {
            // Search all sections
            for sec in 1..=9 {
                if let Ok(content) = read_manual_file(&man_dir.join(format!("man{sec}")).join(format!("{page}.{sec}"))) {
                    return Ok(content);
                }
            }
        }
    }

    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Manual page not found"))
}

fn read_manual_file(path: &Path) -> Result<String, std::io::Error> {
    if path.exists() {
        return fs::read_to_string(path);
    }

    // Try compressed versions
    for ext in &["gz", "bz2", "xz"] {
        let compressed_path = path.with_extension(format!("{}.{}", path.extension().unwrap_or_default().to_string_lossy(), ext));
        if compressed_path.exists() {
            // For simplicity, we'll just return a placeholder for compressed files
            return Ok(format!("Manual page found at: {} (compressed)", compressed_path.display()));
        }
    }

    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
}

fn display_content(content: &str, pager: Option<&str>) -> Result<(), ShellError> {
    if let Some(pg) = pager {
        if Command::new(pg)
            .arg("-")
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.take() {
                    let mut stdin = stdin;
                    stdin.write_all(content.as_bytes())?;
                }
                child.wait()
            })
            .is_ok() { return Ok(()) }
    }

    // Fallback: print directly
    print!("{content}");
    Ok(())
}

fn get_builtin_descriptions() -> HashMap<String, String> {
    let mut descriptions = HashMap::new();
    
    descriptions.insert("ls".to_string(), "list directory contents".to_string());
    descriptions.insert("cd".to_string(), "change the current directory".to_string());
    descriptions.insert("pwd".to_string(), "print name of current/working directory".to_string());
    descriptions.insert("cp".to_string(), "copy files or directories".to_string());
    descriptions.insert("mv".to_string(), "move (rename) files".to_string());
    descriptions.insert("rm".to_string(), "remove files or directories".to_string());
    descriptions.insert("mkdir".to_string(), "make directories".to_string());
    descriptions.insert("rmdir".to_string(), "remove empty directories".to_string());
    descriptions.insert("cat".to_string(), "concatenate files and print on the standard output".to_string());
    descriptions.insert("grep".to_string(), "print lines matching a pattern".to_string());
    descriptions.insert("find".to_string(), "search for files and directories".to_string());
    descriptions.insert("sort".to_string(), "sort lines of text files".to_string());
    descriptions.insert("head".to_string(), "output the first part of files".to_string());
    descriptions.insert("tail".to_string(), "output the last part of files".to_string());
    descriptions.insert("wc".to_string(), "print newline, word, and byte counts for each file".to_string());
    descriptions.insert("ps".to_string(), "report a snapshot of current processes".to_string());
    descriptions.insert("top".to_string(), "display and update sorted information about running processes".to_string());
    descriptions.insert("kill".to_string(), "terminate processes by PID or name".to_string());
    descriptions.insert("which".to_string(), "locate a command".to_string());
    descriptions.insert("whereis".to_string(), "locate the binary, source, and manual page files for a command".to_string());
    descriptions.insert("man".to_string(), "an interface to the system reference manuals".to_string());
    
    descriptions
}

fn get_builtin_help(command: &str) -> Option<String> {
    match command {
        "man" => Some("man - an interface to the system reference manuals\n\nUSAGE:\n    man [OPTION...] [SECTION] PAGE...\n\nFor more information, use 'man --help'".to_string()),
        _ => None
    }
}

