use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

#[derive(Default, Debug)]
struct Opts {
    binary: bool,
    check: bool,
    quiet: bool,
    status: bool,
    help: bool,
    files: Vec<String>,
}

/// sha256sum (simplified coreutils compatible subset)
/// Supported:
///  * Compute digests: sha256sum [FILE]...
///  * Read stdin if no FILE or FILE is '-'
///  * -b / --binary : mark output with '*'
///  * -c / --check  : verify sums from FILE(s) or stdin
///  * --quiet       : with --check, suppress OK lines
///  * --status      : with --check, suppress all output; exit status indicates success
///  * -h / --help   : help
///    Not (yet) supported: --warn, --strict, --tag, -z, --ignore-missing.
pub fn sha256sum_cli(args: &[String]) -> Result<()> {
    let opts = parse_args(args)?;
    if opts.help { print_help(); return Ok(()); }
    if opts.check { run_check_mode(&opts)?; } else { run_hash_mode(&opts)?; }
    Ok(())
}

fn parse_args(args: &[String]) -> Result<Opts> {
    let mut opts = Opts::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-b" | "--binary" => opts.binary = true,
            "-c" | "--check" => opts.check = true,
            "--quiet" => opts.quiet = true,
            "--status" => opts.status = true,
            "-h" | "--help" => opts.help = true,
            arg if !arg.starts_with('-') => opts.files.push(arg.to_string()),
            other => return Err(anyhow!("sha256sum: unrecognized option '{other}'")),
        }
        i += 1;
    }
    Ok(opts)
}

fn print_help() {
    println!("sha256sum - compute and check SHA256 message digest");
    println!("Usage: sha256sum [OPTION]... [FILE]...");
    println!("       sha256sum -c [OPTION]... [FILE]...");
    println!("Options:");
    println!("  -b, --binary        read files in binary mode (marker only)");
    println!("  -c, --check         read SHA256 sums from the FILEs and check them");
    println!("      --quiet         don't print OK for each successfully verified file");
    println!("      --status        don't output anything, status code shows success");
    println!("  -h, --help          display this help and exit");
}

fn run_hash_mode(opts: &Opts) -> Result<()> {
    // If no files, hash stdin
    if opts.files.is_empty() {
        let hash = hash_reader_to_hex(&mut io::stdin().lock())?;
        let marker = if opts.binary { '*' } else { ' ' };
    println!("{hash}{marker}-");
        return Ok(());
    }

    for name in &opts.files {
        if name == "-" { // stdin
            let hash = hash_reader_to_hex(&mut io::stdin().lock())?;
            let marker = if opts.binary { '*' } else { ' ' };
            println!("{hash}{marker}-");
            continue;
        }
        match File::open(name) {
            Ok(mut f) => {
                let hash = hash_reader_to_hex(&mut f)?;
                let marker = if opts.binary { '*' } else { ' ' };
                println!("{hash}{marker}{name}");
            }
            Err(e) => {
                eprintln!("sha256sum: {name}: {e}");
            }
        }
    }
    Ok(())
}

fn run_check_mode(opts: &Opts) -> Result<()> {
    // With -c, treat listed files as checksum list(s). If none, read from stdin.
    let mut total = 0usize;
    let mut ok = 0usize;
    let mut failed = 0usize;
    let mut open_failed = 0usize;

    if opts.files.is_empty() {
        verify_checksum_stream(&mut io::stdin().lock(), opts, &mut total, &mut ok, &mut failed, &mut open_failed)?;
    } else {
        for list_file in &opts.files {
            if list_file == "-" { // treat '-' as stdin list
                verify_checksum_stream(&mut io::stdin().lock(), opts, &mut total, &mut ok, &mut failed, &mut open_failed)?;
                continue;
            }
            match File::open(list_file) {
                Ok(f) => {
                    let mut reader = BufReader::new(f);
                    verify_checksum_stream(&mut reader, opts, &mut total, &mut ok, &mut failed, &mut open_failed)?;
                }
                Err(e) => {
                    eprintln!("sha256sum: {list_file}: {e}");
                    return Err(anyhow!("failed to open list file"));
                }
            }
        }
    }

    if !opts.status {
        if failed == 0 && open_failed == 0 {
            eprintln!("sha256sum: OK"); // Summary (non standard but helpful)
        } else if failed > 0 || open_failed > 0 {
            eprintln!("sha256sum: WARNING: {failed} computed checksum mismatches, {open_failed} unreadable files");
        }
    }

    if failed == 0 && open_failed == 0 { Ok(()) } else { Err(anyhow!("checksum verification failed")) }
}

fn verify_checksum_stream<R: BufRead>(reader: &mut R, opts: &Opts, total: &mut usize, ok: &mut usize, failed: &mut usize, open_failed: &mut usize) -> Result<()> {
    let mut line_buf = String::new();
    while {
        line_buf.clear();
        reader.read_line(&mut line_buf)? > 0
    } {
        let line = line_buf.trim_end_matches(['\n', '\r']);
        if line.is_empty() || line.starts_with('#') { continue; }
        // Expected formats:
        // <64hex><space><space><filename>
        // <64hex><space>*<filename>
        if line.len() < 66 { continue; }
        let (hash_part, rest) = line.split_at(64);
        if !hash_part.chars().all(|c| c.is_ascii_hexdigit()) { continue; }
        let rest = rest.trim_start();
        if rest.is_empty() { continue; }
        let (mode_char, filename) = match rest.chars().next().unwrap() { // safe unwrap (checked non-empty)
            '*' | ' ' => (rest.chars().next().unwrap(), &rest[1..]),
            _ => (' ', rest),
        };
        *total += 1;
        let fname_trim = filename.trim_start_matches([' ', '\t']);
        match File::open(fname_trim) {
            Ok(mut f) => {
                match hash_reader_to_hex(&mut f) {
                    Ok(actual) => {
                        if actual.eq_ignore_ascii_case(hash_part) {
                            *ok += 1;
                            if !opts.quiet && !opts.status { println!("{fname_trim}: OK"); }
                        } else {
                            *failed += 1;
                            if !opts.status { println!("{fname_trim}: FAILED"); }
                        }
                    }
                    Err(e) => {
                        *failed += 1;
                        if !opts.status { println!("{fname_trim}: FAILED ({e})"); }
                    }
                }
            }
            Err(e) => {
                *open_failed += 1;
                if !opts.status { println!("{fname_trim}: FAILED open ({e})"); }
            }
        }
        let _ = mode_char; // currently unused; placeholder for future text/binary distinction
    }
    Ok(())
}

fn hash_reader_to_hex<R: Read>(reader: &mut R) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf).context("failed to read input")?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

