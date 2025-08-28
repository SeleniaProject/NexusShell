//! `wget` builtin - Non-interactive network downloader with advanced features.
//!
//! Delegates to the system `wget` binary when available in `PATH` to preserve the
//! complete feature set and CLI surface area. When the binary is unavailable
//! (e.g. minimal containers or Windows without Git for Windows), it falls back
//! to an enhanced internal implementation that supports common wget operations.

use anyhow::{anyhow, Result, Context};
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufWriter, copy, Write};
#[cfg(feature = "net-http")]
use url::Url;
use which::which;

#[derive(Debug, Clone)]
pub struct WgetOptions {
    url: String,
    output: Option<String>,
    verbose: bool,
    quiet: bool,
    continue_download: bool,
    timeout: Option<u64>,
    tries: Option<u32>,
    user_agent: Option<String>,
    header: Vec<String>,
    use_internal: bool,
}

impl Default for WgetOptions {
    fn default() -> Self {
        Self {
            url: String::new(),
            output: None,
            verbose: false,
            quiet: false,
            continue_download: false,
            timeout: None,
            tries: Some(1),
            user_agent: None,
            header: Vec::new(),
            use_internal: false,
        }
    }
}

/// Entry point for the `wget` builtin.
pub fn wget_cli(args: &[String]) -> Result<()> {
    let options = parse_wget_args(args)?;
    
    // Prefer the full-featured system implementation when present (unless forced internal).
    if !options.use_internal {
        if let Ok(result) = try_external_wget(args) {
            return result;
        }
        
        if options.verbose {
            println!("wget: external binary not found, using internal implementation");
        }
    }
    
    // Use internal implementation
    run_internal_wget(&options)
}

fn try_external_wget(args: &[String]) -> Result<Result<()>> {
    if let Ok(path) = which("wget") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("wget: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    
    Err(anyhow!("wget: backend not found"))
}

fn parse_wget_args(args: &[String]) -> Result<WgetOptions> {
    let mut options = WgetOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_wget_help();
                std::process::exit(0);
            }
            "-v" | "--verbose" => {
                options.verbose = true;
            }
            "-q" | "--quiet" => {
                options.quiet = true;
            }
            "--internal" => {
                options.use_internal = true;
            }
            "-O" | "--output-document" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow!("wget: -O requires a filename"));
                }
                options.output = Some(args[i].clone());
            }
            "-c" | "--continue" => {
                options.continue_download = true;
            }
            "-T" | "--timeout" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow!("wget: -T requires a timeout value"));
                }
                options.timeout = Some(args[i].parse()
                    .map_err(|_| anyhow!("wget: invalid timeout value: {}", args[i]))?);
            }
            "-t" | "--tries" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow!("wget: -t requires a retry count"));
                }
                options.tries = Some(args[i].parse()
                    .map_err(|_| anyhow!("wget: invalid tries value: {}", args[i]))?);
            }
            "-U" | "--user-agent" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow!("wget: -U requires a user agent"));
                }
                options.user_agent = Some(args[i].clone());
            }
            "--header" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow!("wget: --header requires a header"));
                }
                options.header.push(args[i].clone());
            }
            arg if !arg.starts_with('-') => {
                if options.url.is_empty() {
                    options.url = arg.to_string();
                } else {
                    return Err(anyhow!("wget: too many URLs specified"));
                }
            }
            _ => {
                return Err(anyhow!("wget: unknown option: {}", args[i]));
            }
        }
        i += 1;
    }
    
    if options.url.is_empty() {
        return Err(anyhow!("wget: no URL specified"));
    }
    
    Ok(options)
}

fn print_wget_help() {
    println!("Usage: wget [options] URL");
    println!();
    println!("Options:");
    println!("  -h, --help                Show this help message");
    println!("  -v, --verbose             Enable verbose output");
    println!("  -q, --quiet               Turn off output");
    println!("  -O, --output-document=F   Write documents to FILE");
    println!("  -c, --continue            Resume getting a partially-downloaded file");
    println!("  -T, --timeout=SECONDS     Set the network timeout");
    println!("  -t, --tries=NUMBER        Set number of retries to NUMBER (0 unlimits)");
    println!("  -U, --user-agent=AGENT    Identify as AGENT instead of wget");
    println!("  --header=STRING           Insert STRING among the headers sent");
    println!("  --internal                Force use of internal implementation");
    println!();
    println!("Examples:");
    println!("  wget https://example.com/file.txt");
    println!("  wget -O myfile.txt https://example.com/file.txt");
    println!("  wget -v -c https://example.com/largefile.zip");
    println!("  wget --header='Authorization: Bearer token' https://api.example.com/data");
}

#[cfg(feature = "net-http")]
fn run_internal_wget(options: &WgetOptions) -> Result<()> {
    let parsed_url = Url::parse(&options.url).context("wget: invalid URL")?;
    
    // Determine output filename
    let output_path = if let Some(output) = &options.output {
        if output == "-" {
            // Write to stdout
            download_to_stdout(options)?;
            return Ok(());
        }
        PathBuf::from(output)
    } else {
        // Use filename from URL
        let default_name = parsed_url
            .path_segments()
            .and_then(|mut segments| segments.next_back())
            .filter(|s| !s.is_empty())
            .unwrap_or("index.html");
        PathBuf::from(default_name)
    };
    
    if !options.quiet {
        println!("--{}-- {}", 
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), 
                options.url);
        println!("Resolving {}...", parsed_url.host_str().unwrap_or("unknown"));
    }
    
    let mut attempt = 0;
    let max_tries = options.tries.unwrap_or(1);
    
    loop {
        attempt += 1;
        
        match download_file(options, &output_path) {
            Ok(()) => {
                if !options.quiet {
                    println!("'{}' saved", output_path.display());
                }
                return Ok(());
            }
            Err(e) => {
                if attempt >= max_tries {
                    return Err(e);
                }
                
                if !options.quiet {
                    println!("wget: retrying... (attempt {}/{})", attempt + 1, max_tries);
                }
                
                // Simple retry delay
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}

#[cfg(not(feature = "net-http"))]
fn run_internal_wget(_options: &WgetOptions) -> Result<()> {
    Err(anyhow!(
        "wget: internal HTTP disabled (built without 'net-http' feature); install system wget or rebuild with --features net-http"
    ))
}


/// Execute function stub
pub fn execute(_args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}

#[cfg(feature = "net-http")]
fn download_file(options: &WgetOptions, output_path: &Path) -> Result<()> {
    let mut request = ureq::get(&options.url);
    
    // Add headers
    for header in &options.header {
        if let Some(colon_pos) = header.find(':') {
            let name = header[..colon_pos].trim();
            let value = header[colon_pos + 1..].trim();
            request = request.set(name, value);
        }
    }
    
    // Add User-Agent if specified
    if let Some(ua) = &options.user_agent {
        request = request.set("User-Agent", ua);
    }
    
    // Set timeout if specified
    if let Some(timeout) = options.timeout {
        request = request.timeout(std::time::Duration::from_secs(timeout));
    }
    
    if options.verbose {
        println!("Connecting to {}...", options.url);
    }
    
    let response = request.call()
        .with_context(|| format!("wget: failed to fetch {}", options.url))?;
    
    if response.status() != 200 {
        return Err(anyhow!(
            "wget: server responded with HTTP status {}",
            response.status()
        ));
    }
    
    if options.verbose {
        println!("HTTP request sent, awaiting response... {} {}", 
                response.status(), response.status_text());
        
        if let Some(content_length) = response.header("Content-Length") {
            println!("Length: {content_length} bytes");
        }
        
        if let Some(content_type) = response.header("Content-Type") {
            println!("Content-Type: {content_type}");
        }
    }
    
    // Handle file writing with resume support
    let mut file = if options.continue_download && output_path.exists() {
        std::fs::OpenOptions::new()
            .append(true)
            .open(output_path)
            .with_context(|| format!("wget: cannot open file {output_path:?}"))?
    } else {
        File::create(output_path)
            .with_context(|| format!("wget: cannot create file {output_path:?}"))?
    };
    
    let mut writer = BufWriter::new(&mut file);
    let mut reader = response.into_reader();
    
    copy(&mut reader, &mut writer)
        .context("wget: failed while writing to file")?;
    
    writer.flush().context("wget: failed to flush file")?;
    
    Ok(())
}

#[cfg(feature = "net-http")]
fn download_to_stdout(options: &WgetOptions) -> Result<()> {
    let mut request = ureq::get(&options.url);
    
    // Add headers
    for header in &options.header {
        if let Some(colon_pos) = header.find(':') {
            let name = header[..colon_pos].trim();
            let value = header[colon_pos + 1..].trim();
            request = request.set(name, value);
        }
    }
    
    // Add User-Agent if specified
    if let Some(ua) = &options.user_agent {
        request = request.set("User-Agent", ua);
    }
    
    // Set timeout if specified
    if let Some(timeout) = options.timeout {
        request = request.timeout(std::time::Duration::from_secs(timeout));
    }
    
    let response = request.call()
        .with_context(|| format!("wget: failed to fetch {}", options.url))?;
    
    if response.status() != 200 {
        return Err(anyhow!(
            "wget: server responded with HTTP status {}",
            response.status()
        ));
    }
    
    let body = response.into_string()
        .context("wget: failed to read response body")?;
    
    print!("{body}");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_wget_args() {
        let args = vec!["https://example.com/file.txt".to_string()];
        let options = parse_wget_args(&args).expect("Failed to parse valid wget args");
        assert_eq!(options.url, "https://example.com/file.txt");
        assert!(options.output.is_none());
        
        let args = vec!["-O".to_string(), "output.txt".to_string(), "https://example.com/file.txt".to_string()];
        let options = parse_wget_args(&args).expect("Failed to parse wget args with output option");
        assert_eq!(options.output, Some("output.txt".to_string()));
    }
}
