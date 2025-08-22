use anyhow::Result;
use std::io::{self, Read};
use std::fs::File;

/// CLI wrapper function for sha1sum command
pub fn sha1sum_cli(args: &[String]) -> Result<()> {
    let mut binary_mode = false; // currently unused in placeholder implementation
    let mut files = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-b" | "--binary" => {
                binary_mode = true;
            }
            "-h" | "--help" => {
                println!("sha1sum - compute and check SHA1 message digest");
                println!("Usage: sha1sum [OPTION]... [FILE]...");
                println!("  -b, --binary     read in binary mode");
                println!("  -h, --help       display this help and exit");
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                files.push(arg.to_string());
            }
            opt => {
                eprintln!("sha1sum: unrecognized option '{opt}'");
                return Err(anyhow::anyhow!("Invalid option"));
            }
        }
        i += 1;
    }

    if files.is_empty() {
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        let hash = compute_sha1(&buffer);
        println!("{hash}  -");
    } else {
        for filename in &files {
            let mut file = File::open(filename)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            let hash = compute_sha1(&buffer);
            let mode_char = if binary_mode { "*" } else { " " };
            println!("{hash}{mode_char}{filename}");
        }
    }
    Ok(())
}

fn compute_sha1(data: &[u8]) -> String {
    // Placeholder (non-cryptographic) hash; replace with real SHA1 if needed
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{hash:040x}")
}

