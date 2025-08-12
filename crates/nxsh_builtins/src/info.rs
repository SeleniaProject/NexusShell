use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use nxsh_core::{ShellError, ErrorKind};
use nxsh_core::error::RuntimeErrorKind;

pub fn info_cli(args: &[String]) -> Result<(), ShellError> {
    if args.is_empty() || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let mut node_name = None;
    let mut directory = None;
    let mut file_name = None;
    let mut output_file = None;
    let mut subnodes = false;
    let mut where_is = false;
    let mut usage = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--directory" => {
                if i + 1 < args.len() {
                    directory = Some(args[i + 1].clone());
                    i += 1;
                }
            },
            "-f" | "--file" => {
                if i + 1 < args.len() {
                    file_name = Some(args[i + 1].clone());
                    i += 1;
                }
            },
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 1;
                }
            },
            "--subnodes" => subnodes = true,
            "-w" | "--where" => where_is = true,
            "--usage" => usage = true,
            arg if !arg.starts_with('-') => {
                node_name = Some(arg.to_string());
            },
            _ => {
                return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Unknown option: {}", args[i])));
            }
        }
        i += 1;
    }

    if usage {
        print_usage();
        return Ok(());
    }

    if where_is {
        return show_info_location(&node_name.unwrap_or_else(|| "dir".to_string()));
    }

    if let Some(ref file) = file_name {
        return display_info_file(file, node_name.as_deref(), output_file.as_deref());
    }

    let node = node_name.unwrap_or_else(|| "dir".to_string());
    display_info_node(&node, directory.as_deref(), subnodes, output_file.as_deref())
}

fn print_help() {
    println!("info - read Info documents

USAGE:
    info [OPTION...] [MENU-ITEM...]

OPTIONS:
    -d, --directory=DIR       Add DIR to INFOPATH
    -f, --file=FILENAME       Specify Info file to visit
    -o, --output=FILE         Output to FILE instead of stdout
    -w, --where               Show location of Info file
    --usage                   Give a short usage message
    --subnodes                Recursively output menu items
    -h, --help                Show this help

DESCRIPTION:
    Read documentation in Info format. Info documents are typically found
    in /usr/share/info/ and contain detailed documentation for various programs.");
}

fn print_usage() {
    println!("Usage: info [OPTION...] [MENU-ITEM...]");
}

fn show_info_location(node: &str) -> Result<(), ShellError> {
    let info_paths = get_info_paths();
    let mut found = false;

    for info_dir in &info_paths {
        let info_file = info_dir.join(format!("{node}.info"));
        if info_file.exists() {
            println!("{}", info_file.display());
            found = true;
        }

        // Check compressed versions
        for ext in &["gz", "bz2", "xz"] {
            let compressed_file = info_dir.join(format!("{node}.info.{ext}"));
            if compressed_file.exists() {
                println!("{}", compressed_file.display());
                found = true;
            }
        }
    }

    if !found {
        println!("info: No info file for '{node}'");
    }

    Ok(())
}

fn display_info_file(filename: &str, node: Option<&str>, output: Option<&str>) -> Result<(), ShellError> {
    let path = Path::new(filename);
    
    if !path.exists() {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("File not found: {filename}")));
    }

    let content = match read_info_file(path) {
        Ok(content) => content,
        Err(e) => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Error reading file: {e}"))),
    };

    let display_content = if let Some(n) = node {
        extract_node_content(&content, n)
    } else {
        content
    };

    if let Some(output_file) = output {
        match fs::write(output_file, &display_content) {
            Ok(_) => println!("Output written to {output_file}"),
            Err(e) => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Error writing output: {e}"))),
        }
    } else {
        display_in_pager(&display_content)?;
    }

    Ok(())
}

fn display_info_node(node: &str, directory: Option<&str>, subnodes: bool, output: Option<&str>) -> Result<(), ShellError> {
    // Try system info command first
    let mut cmd = Command::new("info");
    
    if let Some(dir) = directory {
        cmd.arg("--directory").arg(dir);
    }
    
    if let Some(out) = output {
        cmd.arg("--output").arg(out);
    }
    
    if subnodes {
        cmd.arg("--subnodes");
    }
    
    cmd.arg(node);

    match cmd.output() {
        Ok(output) if output.status.success() => {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            return Ok(());
        },
        _ => {}
    }

    // Fallback: try to find and display info file
    if let Some(content) = find_info_content(node, directory) {
        let display_content = if subnodes {
            format!("{}\n\n{}", content, extract_subnodes(&content))
        } else {
            content
        };

        if let Some(output_file) = output {
            match fs::write(output_file, &display_content) {
                Ok(_) => println!("Output written to {output_file}"),
                Err(e) => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Error writing output: {e}"))),
            }
        } else {
            display_in_pager(&display_content)?;
        }
        return Ok(());
    }

    // Final fallback: show built-in info
    if let Some(info) = get_builtin_info(node) {
        if let Some(output_file) = output {
            match fs::write(output_file, &info) {
                Ok(_) => println!("Output written to {output_file}"),
                Err(e) => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Error writing output: {e}"))),
            }
        } else {
            display_in_pager(&info)?;
        }
        return Ok(());
    }

    Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("No info document for '{node}'")))
}

fn get_info_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // Standard Unix info paths
    let standard_paths = [
        "/usr/share/info",
        "/usr/local/share/info",
        "/usr/info",
        "/usr/local/info",
        "/opt/local/share/info",
    ];

    for path in &standard_paths {
        if Path::new(path).is_dir() {
            paths.push(PathBuf::from(path));
        }
    }

    // Check INFOPATH environment variable
    if let Ok(infopath) = std::env::var("INFOPATH") {
        for path in infopath.split(':') {
            if !path.is_empty() && Path::new(path).is_dir() {
                paths.push(PathBuf::from(path));
            }
        }
    }

    paths
}

fn find_info_content(node: &str, custom_dir: Option<&str>) -> Option<String> {
    let mut search_paths = Vec::new();
    
    if let Some(dir) = custom_dir {
        search_paths.push(PathBuf::from(dir));
    }
    
    search_paths.extend(get_info_paths());

    for info_dir in &search_paths {
        if let Ok(content) = read_info_file(&info_dir.join(format!("{node}.info"))) {
            return Some(content);
        }

        // Check compressed versions
        for ext in &["gz", "bz2", "xz"] {
            if let Ok(content) = read_compressed_info_file(&info_dir.join(format!("{node}.info.{ext}"))) {
                return Some(content);
            }
        }
    }

    None
}

fn read_info_file(path: &Path) -> Result<String, std::io::Error> {
    if !path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"));
    }
    
    fs::read_to_string(path)
}

fn read_compressed_info_file(path: &Path) -> Result<String, std::io::Error> {
    if !path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"));
    }
    
    // For simplicity, return a placeholder for compressed files
    Ok(format!("Info document found at: {} (compressed file)\n\nTo read this file, please use a proper info reader or decompress it first.", path.display()))
}

fn extract_node_content(content: &str, node_name: &str) -> String {
    // Simple node extraction - look for Node: markers
    if let Some(start) = content.find(&format!("Node: {node_name}")) {
        if let Some(end) = content[start..].find("\nNode:") {
            return content[start..start + end].to_string();
        } else {
            return content[start..].to_string();
        }
    }
    
    content.to_string()
}

fn extract_subnodes(content: &str) -> String {
    let mut subnodes = Vec::new();
    
    // Look for menu items
    if let Some(menu_start) = content.find("* Menu:") {
        let menu_section = &content[menu_start..];
        
        for line in menu_section.lines() {
            if line.starts_with('*') && line.contains("::") {
                // This is a menu item
                if let Some(desc_start) = line.find("::") {
                    let item = line[1..desc_start].trim();
                    subnodes.push(format!("  {item}"));
                }
            }
        }
    }

    if subnodes.is_empty() {
        "No subnodes found.".to_string()
    } else {
        format!("Subnodes:\n{}", subnodes.join("\n"))
    }
}

fn display_in_pager(content: &str) -> Result<(), ShellError> {
    // Try to use system pager
    if let Ok(pager) = std::env::var("PAGER") {
        if Command::new(&pager)
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

    // Fallback: try common pagers
    for pager in &["less", "more", "cat"] {
        match Command::new(pager)
            .arg("-")
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.take() {
                    let mut stdin = stdin;
                    stdin.write_all(content.as_bytes())?;
                }
                child.wait()
            }) {
            Ok(_) => return Ok(()),
            Err(_) => continue,
        }
    }

    // Final fallback: print directly
    print!("{content}");
    Ok(())
}

fn get_builtin_info(node: &str) -> Option<String> {
    let info_docs = get_builtin_info_docs();
    info_docs.get(node).cloned()
}

fn get_builtin_info_docs() -> HashMap<String, String> {
    let mut docs = HashMap::new();
    
    docs.insert("dir".to_string(), 
        "File: dir,  Node: Top,  This is the top of the INFO tree\n\n\
        This is the Info main menu (aka directory node).\n\
        A few useful Info commands:\n\n\
        * Menu:\n\
        * Basics: Basic commands and concepts\n\
        * Advanced: Advanced features\n\
        * Help: Getting help\n\n\
        Press 'q' to quit, 'h' for help.".to_string());
        
    docs.insert("info".to_string(),
        "File: info,  Node: Top,  Next: Getting Started\n\n\
        Info: An Introduction\n\
        *********************\n\n\
        The GNU Info program reads Info files, which contain structured\n\
        documentation. This manual describes how to read Info files and\n\
        how to create them.\n\n\
        * Menu:\n\
        * Getting Started:: Getting started using Info\n\
        * Expert Info:: Advanced Info commands\n\
        * Creating an Info File:: How to make your own Info file".to_string());

    docs.insert("bash".to_string(),
        "File: bash,  Node: Top,  Next: Introduction\n\n\
        Bash Features\n\
        *************\n\n\
        This text describes the features of Bash, the GNU shell.\n\n\
        Bash is the shell, or command language interpreter, for the GNU\n\
        operating system.  The name is an acronym for the 'Bourne-Again\n\
        SHell', a pun on Stephen Bourne, the author of the direct ancestor\n\
        of the current Unix shell sh.\n\n\
        * Menu:\n\
        * Introduction:: What is Bash?\n\
        * Basic Shell Features:: The shell syntax\n\
        * Shell Builtin Commands:: Commands built into the shell".to_string());

    docs
}
