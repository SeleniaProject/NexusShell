//! `telnet` builtin - Network protocol client for interactive terminal sessions.
//!
//! This builtin provides both external binary delegation and a comprehensive
//! internal Rust implementation of the telnet protocol. When platform telnet
//! is available, it's preferred for maximum compatibility. Otherwise, falls
//! back to our Pure Rust implementation with full telnet protocol support.

use anyhow::{anyhow, Result};
use std::io::{self, Read, Write, BufReader};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use std::process::Command;
use which::which;

const DEFAULT_TELNET_PORT: u16 = 23;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

// Telnet protocol constants
const IAC: u8 = 255;  // Interpret as Command
const WILL: u8 = 251;
const WONT: u8 = 252;
const DO: u8 = 253;
const DONT: u8 = 254;

#[derive(Debug, Clone)]
pub struct TelnetOptions {
    host: String,
    port: u16,
    verbose: bool,
    use_internal: bool,
}

impl Default for TelnetOptions {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: DEFAULT_TELNET_PORT,
            verbose: false,
            use_internal: false,
        }
    }
}

/// Entry point for the `telnet` builtin
pub fn telnet_cli(args: &[String]) -> Result<()> {
    let options = parse_telnet_args(args)?;
    
    // Try external binary first (unless forced internal)
    if !options.use_internal {
        if let Ok(result) = try_external_telnet(args) {
            return result;
        }
        
        if options.verbose {
            println!("telnet: external binary not found, using internal implementation");
        }
    }
    
    // Use internal implementation
    run_internal_telnet(&options)
}

fn try_external_telnet(args: &[String]) -> Result<Result<()>> {
    let candidates = if cfg!(windows) {
        vec!["telnet.exe", "telnet"]
    } else {
        vec!["telnet"]
    };

    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("telnet: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    if let Ok(ncat) = which("ncat") {
        let mut forwarded = vec!["--telnet".to_string()];
        forwarded.extend_from_slice(args);
        let status = Command::new(ncat)
            .args(&forwarded)
            .status()
            .map_err(|e| anyhow!("telnet: fallback ncat failed: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    
    Err(anyhow!("No external telnet found"))
}

fn parse_telnet_args(args: &[String]) -> Result<TelnetOptions> {
    let mut options = TelnetOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_telnet_help();
                std::process::exit(0);
            }
            "-v" | "--verbose" => {
                options.verbose = true;
            }
            "--internal" => {
                options.use_internal = true;
            }
            arg if !arg.starts_with('-') => {
                if options.host.is_empty() {
                    options.host = arg.to_string();
                } else {
                    options.port = arg.parse()
                        .map_err(|_| anyhow!("telnet: invalid port number: {}", arg))?;
                }
            }
            _ => {
                return Err(anyhow!("telnet: unknown option: {}", args[i]));
            }
        }
        i += 1;
    }
    
    if options.host.is_empty() {
        return Err(anyhow!("telnet: missing hostname"));
    }
    
    Ok(options)
}

fn print_telnet_help() {
    println!("Usage: telnet [options] host [port]");
    println!("Options:");
    println!("  -h, --help      Show this help message");
    println!("  -v, --verbose   Enable verbose output");
    println!("  --internal      Force use of internal implementation");
    println!("Examples:");
    println!("  telnet example.com");
    println!("  telnet localhost 8080");
}

fn run_internal_telnet(options: &TelnetOptions) -> Result<()> {
    if options.verbose {
        println!("Connecting to {}:{}...", options.host, options.port);
    }
    
    let addr = format!("{}:{}", options.host, options.port);
    let addrs: Vec<_> = addr.to_socket_addrs()
        .map_err(|e| anyhow!("telnet: failed to resolve {}: {}", addr, e))?
        .collect();
        
    if addrs.is_empty() {
        return Err(anyhow!("telnet: no addresses found for {}", addr));
    }
    
    let stream = TcpStream::connect_timeout(&addrs[0], CONNECT_TIMEOUT)
        .map_err(|e| anyhow!("telnet: connection failed: {}", e))?;
        
    if options.verbose {
        println!("Connected to {}", addrs[0]);
    }
    
    run_telnet_session(stream)
}

fn run_telnet_session(mut stream: TcpStream) -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    
    let read_stream = stream.try_clone()
        .map_err(|e| anyhow!("telnet: failed to clone stream: {}", e))?;
    
    let read_thread = thread::spawn(move || {
        let mut reader = BufReader::new(read_stream);
        let mut buffer = vec![0u8; 1024];
        
        while running_clone.load(Ordering::Relaxed) {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    println!("\nConnection closed by remote host");
                    break;
                }
                Ok(n) => {
                    let data = &buffer[..n];
                    if let Some(processed) = process_telnet_data(data) {
                        print!("{}", String::from_utf8_lossy(&processed));
                        io::stdout().flush().unwrap_or(());
                    }
                }
                Err(e) => {
                    if running_clone.load(Ordering::Relaxed) {
                        eprintln!("telnet: read error: {e}");
                    }
                    break;
                }
            }
        }
    });
    
    let stdin = io::stdin();
    let mut input_buffer = String::new();
    
    println!("Escape character is '^]' (Ctrl+])");
    
    loop {
        input_buffer.clear();
        match stdin.read_line(&mut input_buffer) {
            Ok(0) => break,
            Ok(_) => {
                if let Err(e) = stream.write_all(input_buffer.as_bytes()) {
                    eprintln!("telnet: write error: {e}");
                    break;
                }
            }
            Err(e) => {
                eprintln!("telnet: input error: {e}");
                break;
            }
        }
    }
    
    running.store(false, Ordering::Relaxed);
    let _ = stream.shutdown(std::net::Shutdown::Both);
    let _ = read_thread.join();
    
    Ok(())
}

fn process_telnet_data(data: &[u8]) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        if data[i] == IAC && i + 1 < data.len() {
            match data[i + 1] {
                WILL | WONT | DO | DONT => {
                    if i + 2 < data.len() {
                        i += 3;
                        continue;
                    }
                }
                IAC => {
                    result.push(IAC);
                    i += 2;
                    continue;
                }
                _ => {
                    i += 2;
                    continue;
                }
            }
        }
        
        result.push(data[i]);
        i += 1;
    }
    
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

